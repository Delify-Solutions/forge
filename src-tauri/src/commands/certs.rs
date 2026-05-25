// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::Serialize;

use crate::domain::certs;
use crate::error::{ForgeError, ForgeResult};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McKertStatus {
    pub found: bool,
    pub version: Option<String>,
    pub ca_installed: bool,
}

#[tauri::command]
pub fn mkcert_status() -> McKertStatus {
    match crate::platform::macos::detect_mkcert() {
        Some(binary) => McKertStatus {
            found: true,
            version: binary.version,
            ca_installed: certs::ca_installed(),
        },
        None => McKertStatus {
            found: false,
            version: None,
            ca_installed: false,
        },
    }
}

#[tauri::command]
pub async fn install_mkcert_ca() -> ForgeResult<()> {
    tokio::task::spawn_blocking(certs::install_ca)
        .await
        .map_err(|e| ForgeError::Other(e.to_string()))?
}
