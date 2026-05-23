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

#[tauri::command]
pub async fn scan_system() -> ForgeResult<SystemReport> {
    // Real detection lands in Bước 6. For now, return an empty skeleton so
    // the frontend can wire up the wizard against a deterministic shape.
    Ok(SystemReport {
        homebrew: HomebrewStatus {
            installed: false,
            prefix: None,
        },
        nginx: EngineStatus {
            found: false,
            binary: None,
            version: None,
            source: None,
        },
        php: EngineStatus {
            found: false,
            binary: None,
            version: None,
            source: None,
        },
        php_fpm: EngineStatus {
            found: false,
            binary: None,
            version: None,
            source: None,
        },
        ports: Vec::new(),
        resolver: ResolverStatus {
            exists: false,
            correct: false,
        },
    })
}
