// SPDX-License-Identifier: AGPL-3.0-or-later
//
// PHP-FPM lifecycle. MVP exposes a single 'system' pool — multi-version
// support arrives in V0.2 (per-PHP-version pool sharing the same Tera
// template).

use std::path::PathBuf;

use tera::{Context, Tera};

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

pub fn socket_path() -> PathBuf {
    runtime_dir().join("system.sock")
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

pub fn render_config() -> ForgeResult<PathBuf> {
    ensure_dirs()?;

    let mut tera = Tera::default();
    tera.add_raw_template(
        "php-fpm.conf.tera",
        include_str!("../templates/php-fpm.conf.tera"),
    )
    .map_err(|e| ForgeError::Other(format!("tera load: {e}")))?;

    let mut ctx = Context::new();
    ctx.insert("runtime_dir", &runtime_dir().to_string_lossy().to_string());
    ctx.insert("logs_dir", &logs_dir().to_string_lossy().to_string());
    ctx.insert("socket_path", &socket_path().to_string_lossy().to_string());
    ctx.insert("user", &current_user());

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

    let config = render_config()?;

    // Best-effort: remove a stale socket before spawn.
    let _ = std::fs::remove_file(socket_path());

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
