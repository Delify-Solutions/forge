// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::Serialize;
use tauri::State;

use crate::error::ForgeResult;
use crate::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub found: bool,
    pub binary: Option<String>,
    pub version: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortStatus {
    pub port: u16,
    pub in_use: bool,
    pub used_by: Option<String>,
    pub owned_by_forge: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverStatus {
    pub exists: bool,
    pub correct: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomebrewStatus {
    pub installed: bool,
    pub prefix: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemReport {
    pub homebrew: HomebrewStatus,
    pub dnsmasq: EngineStatus,
    pub nginx: EngineStatus,
    pub php: EngineStatus,
    pub php_fpm: EngineStatus,
    pub dns_port: u16,
    pub ports: Vec<PortStatus>,
    pub resolver: ResolverStatus,
    pub installed_php_versions: Vec<String>,
    pub installed_php_lines: Vec<String>,
}

#[tauri::command]
pub async fn ping() -> ForgeResult<String> {
    Ok("pong".to_string())
}

#[tauri::command]
pub async fn open_devtools(window: tauri::Window) -> ForgeResult<()> {
    use tauri::Manager;
    if let Some(w) = window.get_webview_window("main") {
        w.open_devtools();
    }
    Ok(())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn scan_system(state: State<'_, AppState>) -> ForgeResult<SystemReport> {
    use crate::domain::bundle;
    use crate::platform::macos as plat;

    let dns_port = crate::store::dns_port(&state.pool).await?;
    let mut forge_pids: std::collections::HashSet<u32> =
        state.supervisor.running_pids().await.into_iter().collect();
    for pidfile in [
        crate::domain::dns::pid_path(),
        crate::domain::nginx::pid_path(),
    ] {
        if let Ok(content) = std::fs::read_to_string(&pidfile) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                if pid > 1 {
                    let alive = unsafe { libc::kill(pid as i32, 0) == 0 };
                    if alive {
                        forge_pids.insert(pid);
                    }
                }
            }
        }
    }

    let scan = tokio::task::spawn_blocking(move || {
        let prefix = plat::brew_prefix();
        let homebrew = HomebrewStatus {
            installed: prefix.is_some(),
            prefix: prefix.as_ref().map(|p| p.to_string_lossy().to_string()),
        };

        let dnsmasq = engine(plat::detect_binary("dnsmasq", &["--version"]));
        let nginx = engine(plat::detect_binary("nginx", &["-v"]));
        let php = engine(plat::detect_binary("php", &["--version"]));
        let php_fpm = engine(plat::detect_binary("php-fpm", &["--version"]));

        let ports = [80u16, dns_port]
            .into_iter()
            .map(|port| {
                let in_use = plat::port_in_use(port);
                let (used_by, owned_by_forge) = if in_use {
                    match plat::pids_and_name_using_port(port) {
                        Some((pids, name)) => {
                            let owned = pids.iter().any(|p| forge_pids.contains(p));
                            (Some(name), owned)
                        }
                        None => (None, false),
                    }
                } else {
                    (None, false)
                };
                PortStatus {
                    port,
                    in_use,
                    used_by,
                    owned_by_forge,
                }
            })
            .collect();

        let resolver = ResolverStatus {
            exists: plat::resolver_exists(),
            correct: plat::resolver_correct(dns_port),
        };

        let installed_php_versions = bundle::installed_php_versions();
        let installed_php_lines = bundle::installed_php_lines();

        SystemReport {
            homebrew,
            dnsmasq,
            nginx,
            php,
            php_fpm,
            dns_port,
            ports,
            resolver,
            installed_php_versions,
            installed_php_lines,
        }
    })
    .await
    .map_err(|e| crate::error::ForgeError::Other(e.to_string()))?;

    Ok(scan)
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn scan_system(_state: State<'_, AppState>) -> ForgeResult<SystemReport> {
    Err(crate::error::ForgeError::NotImplemented(
        "scan_system is only implemented for macOS in MVP",
    ))
}

#[cfg(target_os = "macos")]
fn engine(detected: Option<crate::platform::macos::DetectedBinary>) -> EngineStatus {
    match detected {
        Some(d) => EngineStatus {
            found: true,
            binary: Some(d.binary.to_string_lossy().to_string()),
            version: d.version,
            source: Some(d.source),
        },
        None => EngineStatus {
            found: false,
            binary: None,
            version: None,
            source: None,
        },
    }
}
