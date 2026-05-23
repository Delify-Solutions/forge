// SPDX-License-Identifier: AGPL-3.0-or-later

use tauri::State;

use crate::domain::dns;
use crate::domain::nginx;
use crate::domain::process::ProcessStatus;
use crate::error::{ForgeError, ForgeResult};
use crate::AppState;

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn setup_dns_resolver() -> ForgeResult<()> {
    use crate::platform::macos as plat;

    tokio::task::spawn_blocking(plat::setup_resolver)
        .await
        .map_err(|e| ForgeError::Other(e.to_string()))?
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn setup_dns_resolver() -> ForgeResult<()> {
    Err(ForgeError::NotImplemented(
        "setup_dns_resolver is only implemented for macOS in MVP",
    ))
}

#[tauri::command]
pub async fn start_dnsmasq(state: State<'_, AppState>) -> ForgeResult<u32> {
    dns::start(&state.supervisor).await
}

#[tauri::command]
pub async fn stop_dnsmasq(state: State<'_, AppState>) -> ForgeResult<()> {
    dns::stop(&state.supervisor).await
}

#[tauri::command]
pub async fn start_nginx(state: State<'_, AppState>) -> ForgeResult<u32> {
    nginx::start(&state.pool, &state.supervisor).await
}

#[tauri::command]
pub async fn stop_nginx(state: State<'_, AppState>) -> ForgeResult<()> {
    nginx::stop(&state.supervisor).await
}

#[tauri::command]
pub async fn reload_nginx(state: State<'_, AppState>) -> ForgeResult<()> {
    nginx::reload(&state.pool).await
}

#[tauri::command]
pub async fn services_status(state: State<'_, AppState>) -> ForgeResult<Vec<ProcessStatus>> {
    Ok(state.supervisor.statuses().await)
}
