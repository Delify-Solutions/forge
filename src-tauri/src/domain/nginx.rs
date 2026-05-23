// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Nginx lifecycle. Generates configs into the runtime directory from
// the canonical SQLite state, then supervises the nginx process. We
// never parse user-edited nginx.conf back — SQLite is the source of
// truth (CLAUDE.md decision 1).

use std::path::PathBuf;

use serde::Serialize;
use sqlx::SqlitePool;
use tera::{Context, Tera};
use tokio::process::Command;

use crate::domain::process::{ProcessSpec, ProcessSupervisor};
use crate::domain::sites::{self, Site};
use crate::error::{ForgeError, ForgeResult};
use crate::platform::macos as plat;
use crate::store;

pub const NGINX_PROCESS: &str = "nginx";

#[derive(Serialize)]
struct SiteCtx {
    name: String,
    path: String,
    domain: String,
}

impl From<Site> for SiteCtx {
    fn from(s: Site) -> Self {
        Self {
            name: s.name,
            path: s.path,
            domain: s.domain,
        }
    }
}

fn runtime_dir() -> PathBuf {
    store::data_dir().join("runtime").join("nginx")
}

fn sites_dir() -> PathBuf {
    runtime_dir().join("sites")
}

fn logs_dir() -> PathBuf {
    store::data_dir().join("logs").join("nginx")
}

fn master_config_path() -> PathBuf {
    runtime_dir().join("nginx.conf")
}

fn site_config_path(name: &str) -> PathBuf {
    sites_dir().join(format!("{name}.conf"))
}

fn php_socket_path() -> PathBuf {
    crate::domain::php::socket_path()
}

fn nginx_prefix(binary: &std::path::Path) -> PathBuf {
    binary
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("/opt/homebrew"))
}

fn build_tera() -> ForgeResult<Tera> {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        (
            "nginx.conf.tera",
            include_str!("../templates/nginx.conf.tera"),
        ),
        (
            "site.conf.tera",
            include_str!("../templates/site.conf.tera"),
        ),
    ])
    .map_err(|e| ForgeError::Other(format!("tera load: {e}")))?;
    Ok(tera)
}

fn ensure_dirs() -> ForgeResult<()> {
    for dir in [runtime_dir(), sites_dir(), logs_dir()] {
        std::fs::create_dir_all(&dir)
            .map_err(|e| ForgeError::Other(format!("create {}: {e}", dir.display())))?;
    }
    Ok(())
}

pub async fn regenerate(pool: &SqlitePool) -> ForgeResult<()> {
    ensure_dirs()?;
    let binary = plat::detect_binary("nginx", &["-v"]).ok_or_else(|| {
        ForgeError::Other("nginx not found — install with: brew install nginx".into())
    })?;
    let prefix = nginx_prefix(&binary.binary);

    let tera = build_tera()?;
    let sites = sites::list(pool).await?;

    let php_socket = php_socket_path();

    // Per-site configs.
    for site in &sites {
        let mut ctx = Context::new();
        ctx.insert("site", &SiteCtx::from(site.clone()));
        ctx.insert("logs_dir", &logs_dir().to_string_lossy().to_string());
        ctx.insert("nginx_prefix", &prefix.to_string_lossy().to_string());
        ctx.insert("php_socket", &php_socket.to_string_lossy().to_string());

        let rendered = tera
            .render("site.conf.tera", &ctx)
            .map_err(|e| ForgeError::Other(format!("render site config: {e}")))?;
        std::fs::write(site_config_path(&site.name), rendered)
            .map_err(|e| ForgeError::Other(format!("write site config: {e}")))?;
    }

    // Remove stale per-site configs that no longer correspond to a row.
    if let Ok(entries) = std::fs::read_dir(sites_dir()) {
        let live: std::collections::HashSet<String> =
            sites.iter().map(|s| format!("{}.conf", s.name)).collect();
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if !live.contains(name) {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }

    // Master config.
    let mut master_ctx = Context::new();
    let site_ctxs: Vec<SiteCtx> = sites.into_iter().map(SiteCtx::from).collect();
    master_ctx.insert("sites", &site_ctxs);
    master_ctx.insert("runtime_dir", &runtime_dir().to_string_lossy().to_string());
    master_ctx.insert("logs_dir", &logs_dir().to_string_lossy().to_string());
    master_ctx.insert("nginx_prefix", &prefix.to_string_lossy().to_string());

    let rendered = tera
        .render("nginx.conf.tera", &master_ctx)
        .map_err(|e| ForgeError::Other(format!("render nginx.conf: {e}")))?;
    std::fs::write(master_config_path(), rendered)
        .map_err(|e| ForgeError::Other(format!("write nginx.conf: {e}")))?;

    Ok(())
}

pub async fn start(pool: &SqlitePool, supervisor: &ProcessSupervisor) -> ForgeResult<u32> {
    regenerate(pool).await?;

    let binary = plat::detect_binary("nginx", &["-v"]).ok_or_else(|| {
        ForgeError::Other("nginx not found — install with: brew install nginx".into())
    })?;
    let prefix = nginx_prefix(&binary.binary);

    let spec = ProcessSpec {
        name: NGINX_PROCESS.to_string(),
        binary: binary.binary,
        args: vec![
            "-p".to_string(),
            prefix.to_string_lossy().to_string(),
            "-c".to_string(),
            master_config_path().to_string_lossy().to_string(),
            "-e".to_string(),
            logs_dir()
                .join("nginx.error.log")
                .to_string_lossy()
                .to_string(),
        ],
        env: Vec::new(),
    };

    supervisor.start(spec).await
}

pub async fn stop(supervisor: &ProcessSupervisor) -> ForgeResult<()> {
    supervisor.stop(NGINX_PROCESS).await
}

pub async fn reload(pool: &SqlitePool) -> ForgeResult<()> {
    regenerate(pool).await?;

    let binary = plat::detect_binary("nginx", &["-v"])
        .ok_or_else(|| ForgeError::Other("nginx not found".into()))?;
    let prefix = nginx_prefix(&binary.binary);

    let output = Command::new(&binary.binary)
        .arg("-p")
        .arg(prefix)
        .arg("-c")
        .arg(master_config_path())
        .arg("-s")
        .arg("reload")
        .output()
        .await
        .map_err(|e| ForgeError::Other(format!("nginx reload spawn: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ForgeError::Other(format!("nginx reload failed: {stderr}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_compile_and_render() {
        let tera = build_tera().expect("tera should load embedded templates");

        // Master config with no sites — must still produce a valid-looking
        // file with the catch-all 404 server block.
        let mut master_ctx = Context::new();
        let empty_sites: Vec<SiteCtx> = Vec::new();
        master_ctx.insert("sites", &empty_sites);
        master_ctx.insert("runtime_dir", "/tmp/forge/runtime/nginx");
        master_ctx.insert("logs_dir", "/tmp/forge/logs/nginx");
        master_ctx.insert("nginx_prefix", "/opt/homebrew");

        let rendered = tera
            .render("nginx.conf.tera", &master_ctx)
            .expect("master config renders");
        assert!(rendered.contains("worker_processes 2;"));
        assert!(rendered.contains("listen 80 default_server;"));
        assert!(rendered.contains("daemon off;"));

        // Per-site config.
        let mut site_ctx = Context::new();
        site_ctx.insert(
            "site",
            &SiteCtx {
                name: "myapp".to_string(),
                path: "/Users/me/Code/myapp".to_string(),
                domain: "myapp.test".to_string(),
            },
        );
        site_ctx.insert("logs_dir", "/tmp/forge/logs/nginx");
        site_ctx.insert("nginx_prefix", "/opt/homebrew");
        site_ctx.insert("php_socket", "/tmp/forge/php.sock");
        let rendered = tera
            .render("site.conf.tera", &site_ctx)
            .expect("site config renders");
        assert!(rendered.contains("server_name myapp.test;"));
        assert!(rendered.contains("root /Users/me/Code/myapp;"));
        assert!(rendered.contains("fastcgi_pass unix:/tmp/forge/php.sock;"));
    }
}
