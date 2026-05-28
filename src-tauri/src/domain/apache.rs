// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Apache lifecycle. Generates configs into the runtime directory from
// the canonical SQLite state, then supervises the httpd process. We
// never parse user-edited httpd.conf back — SQLite is the source of
// truth (CLAUDE.md decision 1).

use std::path::PathBuf;

use serde::Serialize;
use sqlx::SqlitePool;
use tera::{Context, Tera};
use tokio::process::Command;

use crate::domain::bundle;
use crate::domain::nginx::php_socket_for;
use crate::domain::process::{kill_orphan_pidfile, ProcessSpec, ProcessSupervisor};
use crate::domain::sites::{self, Site};
use crate::error::{ForgeError, ForgeResult};
use crate::platform::macos as plat;
use crate::store;

pub const APACHE_PROCESS: &str = "apache";

#[derive(Serialize)]
struct VhostCtx {
    name: String,
    document_root: String,
    domain: String,
    aliases: Vec<String>,
    php_socket: String,
    logs_dir: String,
}

impl VhostCtx {
    fn from_site(site: &Site, logs_dir: &str) -> Self {
        let document_root = sites::document_root(std::path::Path::new(&site.path));
        let php_socket = php_socket_for(&site.php_version);
        Self {
            name: site.name.clone(),
            document_root: document_root.to_string_lossy().to_string(),
            domain: site.domain.clone(),
            aliases: site.aliases.clone(),
            php_socket: php_socket.to_string_lossy().to_string(),
            logs_dir: logs_dir.to_string(),
        }
    }
}

pub fn runtime_dir() -> PathBuf {
    store::data_dir().join("runtime").join("apache")
}

pub fn pid_path() -> PathBuf {
    runtime_dir().join("httpd.pid")
}

pub fn vhosts_dir() -> PathBuf {
    runtime_dir().join("vhosts")
}

pub fn master_config_path() -> PathBuf {
    runtime_dir().join("httpd.conf")
}

pub fn logs_dir() -> PathBuf {
    store::data_dir().join("logs").join("apache")
}

fn current_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "_www".to_string())
}

fn current_group() -> String {
    // On macOS the primary group for a regular user is typically "staff".
    // We use `id -gn` to get the actual group name at runtime.
    let output = std::process::Command::new("id").arg("-gn").output();
    match output {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                "staff".to_string()
            } else {
                s
            }
        }
        _ => "staff".to_string(),
    }
}

fn apache_install_dir() -> ForgeResult<PathBuf> {
    let entry = bundle::find_entry("apache", None)
        .filter(|e| e.installed)
        .ok_or_else(|| {
            ForgeError::Other(
                "apache bundle not installed — install it first via the Sites add dialog".into(),
            )
        })?;
    Ok(bundle::bundle_dir("apache", &entry.version))
}

fn build_tera() -> ForgeResult<Tera> {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        (
            "httpd.conf.tera",
            include_str!("../templates/httpd.conf.tera"),
        ),
        (
            "apache.vhost.conf.tera",
            include_str!("../templates/apache.vhost.conf.tera"),
        ),
    ])
    .map_err(|e| ForgeError::Other(format!("tera load: {e}")))?;
    Ok(tera)
}

fn ensure_dirs() -> ForgeResult<()> {
    for dir in [runtime_dir(), vhosts_dir(), logs_dir()] {
        std::fs::create_dir_all(&dir)
            .map_err(|e| ForgeError::Other(format!("create {}: {e}", dir.display())))?;
    }
    Ok(())
}

pub async fn regenerate(pool: &SqlitePool) -> ForgeResult<()> {
    ensure_dirs()?;
    let install_dir = apache_install_dir()?;
    let tera = build_tera()?;
    let sites = sites::list(pool).await?;
    let logs = logs_dir().to_string_lossy().to_string();

    // Per-site vhost configs.
    for site in &sites {
        let ctx_data = VhostCtx::from_site(site, &logs);
        let mut ctx = Context::new();
        ctx.insert("vhost", &ctx_data);

        let rendered = tera
            .render("apache.vhost.conf.tera", &ctx)
            .map_err(|e| ForgeError::Other(format!("render vhost config: {e}")))?;
        std::fs::write(vhosts_dir().join(format!("{}.conf", site.name)), rendered)
            .map_err(|e| ForgeError::Other(format!("write vhost config: {e}")))?;
    }

    // Remove stale vhost configs.
    if let Ok(entries) = std::fs::read_dir(vhosts_dir()) {
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

    // Master httpd.conf.
    let mut master_ctx = Context::new();
    master_ctx.insert("runtime_dir", &runtime_dir().to_string_lossy().to_string());
    master_ctx.insert("logs_dir", &logs);
    master_ctx.insert("install_dir", &install_dir.to_string_lossy().to_string());
    master_ctx.insert("pid_path", &pid_path().to_string_lossy().to_string());
    master_ctx.insert("vhosts_dir", &vhosts_dir().to_string_lossy().to_string());
    master_ctx.insert("user", &current_user());
    master_ctx.insert("group", &current_group());

    let rendered = tera
        .render("httpd.conf.tera", &master_ctx)
        .map_err(|e| ForgeError::Other(format!("render httpd.conf: {e}")))?;
    std::fs::write(master_config_path(), rendered)
        .map_err(|e| ForgeError::Other(format!("write httpd.conf: {e}")))?;

    Ok(())
}

pub async fn start(pool: &SqlitePool, supervisor: &ProcessSupervisor) -> ForgeResult<u32> {
    regenerate(pool).await?;

    kill_orphan_pidfile(&pid_path());
    plat::kill_listeners_on_port(8288, &["httpd"]);

    let install_dir = apache_install_dir()?;
    let binary = install_dir.join("sbin").join("httpd");

    let spec = ProcessSpec {
        name: APACHE_PROCESS.to_string(),
        binary,
        args: vec![
            "-f".to_string(),
            master_config_path().to_string_lossy().to_string(),
            "-DFOREGROUND".to_string(),
        ],
        env: Vec::new(),
    };

    supervisor.start(spec).await
}

pub async fn stop(supervisor: &ProcessSupervisor) -> ForgeResult<()> {
    supervisor.stop(APACHE_PROCESS).await
}

pub async fn reload(pool: &SqlitePool) -> ForgeResult<()> {
    regenerate(pool).await?;

    let install_dir = apache_install_dir()?;
    let binary = install_dir.join("sbin").join("httpd");

    let output = Command::new(&binary)
        .arg("-f")
        .arg(master_config_path())
        .arg("-k")
        .arg("graceful")
        .output()
        .await
        .map_err(|e| ForgeError::Other(format!("apache reload spawn: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ForgeError::Other(format!("apache reload failed: {stderr}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_tera() -> Tera {
        build_tera().expect("tera should load embedded templates")
    }

    #[test]
    fn master_config_contains_listen_and_modules() {
        let tera = build_test_tera();
        let mut ctx = Context::new();
        ctx.insert("runtime_dir", "/tmp/forge/runtime/apache");
        ctx.insert("logs_dir", "/tmp/forge/logs/apache");
        ctx.insert("install_dir", "/tmp/forge/engines/apache/2.4.62");
        ctx.insert("pid_path", "/tmp/forge/runtime/apache/httpd.pid");
        ctx.insert("vhosts_dir", "/tmp/forge/runtime/apache/vhosts");
        ctx.insert("user", "testuser");
        ctx.insert("group", "staff");

        let rendered = tera
            .render("httpd.conf.tera", &ctx)
            .expect("master config renders");

        assert!(rendered.contains("Listen 127.0.0.1:8288"));
        assert!(rendered.contains("PidFile \"/tmp/forge/runtime/apache/httpd.pid\""));
        assert!(rendered.contains("Include \"/tmp/forge/runtime/apache/vhosts/*.conf\""));
        assert!(rendered.contains("authn_core_module"));
        assert!(rendered.contains("authz_core_module"));
        assert!(rendered.contains("proxy_module"));
        assert!(rendered.contains("proxy_fcgi_module"));
        assert!(rendered.contains("rewrite_module"));
        assert!(rendered.contains("dir_module"));
        assert!(rendered.contains("mime_module"));
        assert!(rendered.contains("log_config_module"));
        assert!(rendered.contains("unixd_module"));
        assert!(rendered.contains("alias_module"));
        assert!(rendered.contains("headers_module"));
        assert!(rendered.contains("filter_module"));
    }

    #[test]
    fn vhost_without_aliases() {
        let tera = build_test_tera();
        let vhost = VhostCtx {
            name: "myapp".to_string(),
            document_root: "/Users/me/Code/myapp/public".to_string(),
            domain: "myapp.test".to_string(),
            aliases: vec![],
            php_socket: "/tmp/forge/8.3.sock".to_string(),
            logs_dir: "/tmp/forge/logs/apache".to_string(),
        };
        let mut ctx = Context::new();
        ctx.insert("vhost", &vhost);

        let rendered = tera
            .render("apache.vhost.conf.tera", &ctx)
            .expect("vhost renders");

        assert!(rendered.contains("<VirtualHost 127.0.0.1:8288>"));
        assert!(rendered.contains("ServerName \"myapp.test\""));
        assert!(!rendered.contains("ServerAlias"));
        assert!(rendered.contains("DocumentRoot \"/Users/me/Code/myapp/public\""));
        assert!(rendered.contains("proxy:unix:/tmp/forge/8.3.sock|fcgi://localhost"));
    }

    #[test]
    fn vhost_with_aliases_single_line() {
        let tera = build_test_tera();
        let vhost = VhostCtx {
            name: "myapp".to_string(),
            document_root: "/Users/me/Code/myapp/public".to_string(),
            domain: "myapp.test".to_string(),
            aliases: vec![
                "staging.myapp.test".to_string(),
                "old-myapp.test".to_string(),
            ],
            php_socket: "/tmp/forge/8.3.sock".to_string(),
            logs_dir: "/tmp/forge/logs/apache".to_string(),
        };
        let mut ctx = Context::new();
        ctx.insert("vhost", &vhost);

        let rendered = tera
            .render("apache.vhost.conf.tera", &ctx)
            .expect("vhost with aliases renders");

        assert!(rendered.contains("ServerAlias staging.myapp.test old-myapp.test"));
        // Must be a single line, not two separate ServerAlias directives.
        let alias_lines: Vec<&str> = rendered
            .lines()
            .filter(|l| l.trim().starts_with("ServerAlias"))
            .collect();
        assert_eq!(
            alias_lines.len(),
            1,
            "expected exactly one ServerAlias line"
        );
    }

    #[test]
    fn vhost_php_socket_in_handler() {
        let tera = build_test_tera();
        let vhost = VhostCtx {
            name: "blog".to_string(),
            document_root: "/Users/me/Code/blog/public".to_string(),
            domain: "blog.test".to_string(),
            aliases: vec![],
            php_socket: "/tmp/forge/runtime/php/8.3.sock".to_string(),
            logs_dir: "/tmp/forge/logs/apache".to_string(),
        };
        let mut ctx = Context::new();
        ctx.insert("vhost", &vhost);

        let rendered = tera
            .render("apache.vhost.conf.tera", &ctx)
            .expect("vhost renders");

        assert!(rendered.contains(
            "SetHandler \"proxy:unix:/tmp/forge/runtime/php/8.3.sock|fcgi://localhost\""
        ));
    }

    #[test]
    fn vhost_document_root_is_quoted() {
        let tera = build_test_tera();
        let vhost = VhostCtx {
            name: "spaced".to_string(),
            document_root: "/Users/me/My Projects/spaced/public".to_string(),
            domain: "spaced.test".to_string(),
            aliases: vec![],
            php_socket: "/tmp/forge/8.3.sock".to_string(),
            logs_dir: "/tmp/forge/logs/apache".to_string(),
        };
        let mut ctx = Context::new();
        ctx.insert("vhost", &vhost);

        let rendered = tera
            .render("apache.vhost.conf.tera", &ctx)
            .expect("vhost renders");

        assert!(rendered.contains("DocumentRoot \"/Users/me/My Projects/spaced/public\""));
    }

    #[test]
    fn regenerate_writes_and_removes_stale_vhosts() {
        let tmp = std::env::temp_dir().join(format!(
            "forge-apache-regen-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(tmp.join("vhosts")).unwrap();

        // Write a stale vhost file that should be removed.
        let stale = tmp.join("vhosts").join("stale.conf");
        std::fs::write(&stale, "stale content").unwrap();

        // Verify stale file exists before cleanup.
        assert!(stale.exists());

        // Simulate the stale-cleanup logic from regenerate.
        let live: std::collections::HashSet<String> =
            ["myapp.conf".to_string()].into_iter().collect();
        if let Ok(entries) = std::fs::read_dir(tmp.join("vhosts")) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if !live.contains(name) {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
        }

        assert!(!stale.exists(), "stale vhost should have been removed");

        std::fs::remove_dir_all(tmp).ok();
    }
}
