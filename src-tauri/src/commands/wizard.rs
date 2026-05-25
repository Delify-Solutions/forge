// SPDX-License-Identifier: AGPL-3.0-or-later

use tauri::State;

use crate::domain::dns;
use crate::domain::nginx;
use crate::domain::php;
use crate::domain::process::ProcessStatus;
use crate::error::{ForgeError, ForgeResult};
use crate::store;
use crate::AppState;

async fn resolve_requested_dns_port(
    pool: &sqlx::SqlitePool,
    port: Option<u16>,
) -> ForgeResult<u16> {
    match port {
        Some(p) if store::is_valid_dns_port(p) => {
            store::set_setting(pool, store::DNS_PORT_SETTING_KEY, &p.to_string()).await?;
            Ok(p)
        }
        _ => store::dns_port(pool).await,
    }
}

#[tauri::command]
pub async fn set_dns_port(state: State<'_, AppState>, port: u16) -> ForgeResult<()> {
    if !store::is_valid_dns_port(port) {
        return Err(ForgeError::Other(
            "DNS port must be between 1 and 65535".into(),
        ));
    }
    store::set_setting(&state.pool, store::DNS_PORT_SETTING_KEY, &port.to_string()).await
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn setup_dns_resolver(state: State<'_, AppState>, port: Option<u16>) -> ForgeResult<()> {
    use crate::platform::macos as plat;

    let dns_port = resolve_requested_dns_port(&state.pool, port).await?;

    tokio::task::spawn_blocking(move || plat::setup_resolver(dns_port))
        .await
        .map_err(|e| ForgeError::Other(e.to_string()))?
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn setup_dns_resolver(
    _state: State<'_, AppState>,
    _port: Option<u16>,
) -> ForgeResult<()> {
    Err(ForgeError::NotImplemented(
        "setup_dns_resolver is only implemented for macOS in MVP",
    ))
}

#[tauri::command]
pub async fn start_dnsmasq(state: State<'_, AppState>, port: Option<u16>) -> ForgeResult<u32> {
    let dns_port = resolve_requested_dns_port(&state.pool, port).await?;
    dns::start(&state.supervisor, dns_port).await
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
pub async fn start_php_fpm(state: State<'_, AppState>) -> ForgeResult<u32> {
    php::start(&state.supervisor).await
}

#[tauri::command]
pub async fn stop_php_fpm(state: State<'_, AppState>) -> ForgeResult<()> {
    php::stop(&state.supervisor).await
}

#[tauri::command]
pub async fn services_status(state: State<'_, AppState>) -> ForgeResult<Vec<ProcessStatus>> {
    Ok(state.supervisor.statuses().await)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn debug_reset_environment(state: State<'_, AppState>) -> ForgeResult<()> {
    use crate::platform::macos as plat;
    use crate::store;

    state.supervisor.shutdown_all().await;

    let data_dir = store::data_dir();
    for name in ["engines", "runtime", "logs"] {
        let path = data_dir.join(name);
        if path.exists() {
            std::fs::remove_dir_all(&path)
                .map_err(|e| ForgeError::Other(format!("remove {}: {e}", path.display())))?;
        }
    }

    tokio::task::spawn_blocking(plat::remove_resolver)
        .await
        .map_err(|e| ForgeError::Other(e.to_string()))??;

    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn debug_reset_environment(_state: State<'_, AppState>) -> ForgeResult<()> {
    Err(ForgeError::NotImplemented(
        "debug_reset_environment is only implemented for macOS in MVP",
    ))
}
