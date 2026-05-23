// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::Serialize;

use crate::error::ForgeResult;

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
    pub nginx: EngineStatus,
    pub php: EngineStatus,
    pub php_fpm: EngineStatus,
    pub ports: Vec<PortStatus>,
    pub resolver: ResolverStatus,
}

#[tauri::command]
pub async fn ping() -> ForgeResult<String> {
    Ok("pong".to_string())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn scan_system() -> ForgeResult<SystemReport> {
    use crate::platform::macos as plat;

    let scan = tokio::task::spawn_blocking(|| {
        let prefix = plat::brew_prefix();
        let homebrew = HomebrewStatus {
            installed: prefix.is_some(),
            prefix: prefix.as_ref().map(|p| p.to_string_lossy().to_string()),
        };

        let nginx = engine(plat::detect_binary("nginx", &["-v"]));
        let php = engine(plat::detect_binary("php", &["--version"]));
        let php_fpm = engine(plat::detect_binary("php-fpm", &["--version"]));

        let ports = [80u16, 443, 5353]
            .into_iter()
            .map(|port| {
                let in_use = plat::port_in_use(port);
                PortStatus {
                    port,
                    in_use,
                    used_by: if in_use {
                        plat::process_using_port(port)
                    } else {
                        None
                    },
                }
            })
            .collect();

        let resolver = ResolverStatus {
            exists: plat::resolver_exists(),
            correct: plat::resolver_correct(),
        };

        SystemReport {
            homebrew,
            nginx,
            php,
            php_fpm,
            ports,
            resolver,
        }
    })
    .await
    .map_err(|e| crate::error::ForgeError::Other(e.to_string()))?;

    Ok(scan)
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn scan_system() -> ForgeResult<SystemReport> {
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
