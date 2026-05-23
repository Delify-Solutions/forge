// SPDX-License-Identifier: AGPL-3.0-or-later
//
// PHP-FPM lifecycle. Multi-version support: one php-fpm master process
// manages multiple pools (one per installed PHP major.minor line), each
// listening on its own Unix socket.

use std::path::PathBuf;

use serde::Serialize;
use tera::{Context, Tera};

use crate::domain::bundle;
use crate::domain::process::{ProcessSpec, ProcessSupervisor};
use crate::error::{ForgeError, ForgeResult};
use crate::platform::macos as plat;
use crate::store;

pub const PHP_FPM_PROCESS: &str = "php-fpm";

fn runtime_dir() -> PathBuf {
    store::data_dir().join("runtime").join("php")
}

fn logs_dir() -> PathBuf {
    store::data_dir().join("logs").join("php")
}

fn config_path() -> PathBuf {
    runtime_dir().join("php-fpm.conf")
}

/// Socket path for a given PHP major.minor line (e.g. "8.3" -> "<runtime>/8.3.sock").
pub fn socket_path(line: &str) -> PathBuf {
    runtime_dir().join(format!("{line}.sock"))
}

/// Return all installed PHP major.minor lines (delegates to bundle catalog).
pub fn installed_lines() -> Vec<String> {
    bundle::installed_php_lines()
}

fn ensure_dirs() -> ForgeResult<()> {
    for dir in [runtime_dir(), logs_dir()] {
        std::fs::create_dir_all(&dir)
            .map_err(|e| ForgeError::Other(format!("create {}: {e}", dir.display())))?;
    }
    Ok(())
}

fn current_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "_www".to_string())
}

#[derive(Serialize)]
struct PoolCtx {
    name: String,
    socket: String,
}

/// Render a php-fpm.conf covering all installed PHP lines. Each line gets
/// its own `[php-<line>]` pool section with a dedicated socket.
pub fn render_config(lines: &[String]) -> ForgeResult<PathBuf> {
    ensure_dirs()?;

    let mut tera = Tera::default();
    tera.add_raw_template(
        "php-fpm.conf.tera",
        include_str!("../templates/php-fpm.conf.tera"),
    )
    .map_err(|e| ForgeError::Other(format!("tera load: {e}")))?;

    let pools: Vec<PoolCtx> = lines
        .iter()
        .map(|line| PoolCtx {
            name: line.clone(),
            socket: socket_path(line).to_string_lossy().to_string(),
        })
        .collect();

    let mut ctx = Context::new();
    ctx.insert("runtime_dir", &runtime_dir().to_string_lossy().to_string());
    ctx.insert("logs_dir", &logs_dir().to_string_lossy().to_string());
    ctx.insert("user", &current_user());
    ctx.insert("pools", &pools);

    let rendered = tera
        .render("php-fpm.conf.tera", &ctx)
        .map_err(|e| ForgeError::Other(format!("render php-fpm.conf: {e}")))?;
    let path = config_path();
    std::fs::write(&path, rendered)
        .map_err(|e| ForgeError::Other(format!("write php-fpm.conf: {e}")))?;
    Ok(path)
}

pub async fn start(supervisor: &ProcessSupervisor) -> ForgeResult<u32> {
    let binary = plat::detect_binary("php-fpm", &["--version"]).ok_or_else(|| {
        ForgeError::Other("php-fpm not found — install with: brew install php".into())
    })?;

    let lines = installed_lines();
    let effective_lines = if lines.is_empty() {
        // Fallback: if no bundle is installed but system php-fpm exists,
        // create a single "system" pool so existing behavior is preserved.
        vec!["system".to_string()]
    } else {
        lines
    };

    let config = render_config(&effective_lines)?;

    // Best-effort: remove stale sockets before spawn.
    for line in &effective_lines {
        let _ = std::fs::remove_file(socket_path(line));
    }

    let spec = ProcessSpec {
        name: PHP_FPM_PROCESS.to_string(),
        binary: binary.binary,
        args: vec![
            "--nodaemonize".to_string(),
            "--fpm-config".to_string(),
            config.to_string_lossy().to_string(),
        ],
        env: Vec::new(),
    };

    supervisor.start(spec).await
}

pub async fn stop(supervisor: &ProcessSupervisor) -> ForgeResult<()> {
    supervisor.stop(PHP_FPM_PROCESS).await
}
