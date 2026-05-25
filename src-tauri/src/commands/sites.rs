// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::Serialize;
use tauri::State;

use crate::domain::sites::{self, AddSiteRequest, Site};
use crate::domain::{logs, nginx, process::ProcessSupervisor};
use crate::error::ForgeResult;
use crate::AppState;

#[tauri::command]
pub async fn list_sites(state: State<'_, AppState>) -> ForgeResult<Vec<Site>> {
    sites::list(&state.pool).await
}

#[tauri::command]
pub async fn add_site(state: State<'_, AppState>, req: AddSiteRequest) -> ForgeResult<Site> {
    let site = sites::add(&state.pool, req).await?;
    let _ = try_reload(&state.pool, &state.supervisor).await;
    Ok(site)
}

#[tauri::command]
pub async fn remove_site(state: State<'_, AppState>, id: i64) -> ForgeResult<()> {
    sites::remove(&state.pool, id).await?;
    let _ = try_reload(&state.pool, &state.supervisor).await;
    Ok(())
}

#[tauri::command]
pub async fn update_site_php(
    state: State<'_, AppState>,
    id: i64,
    php_version: String,
) -> ForgeResult<Site> {
    let site = sites::update_php_version(&state.pool, id, &php_version).await?;
    let _ = try_reload(&state.pool, &state.supervisor).await;
    Ok(site)
}

#[tauri::command]
pub async fn update_site_web_server(
    state: State<'_, AppState>,
    id: i64,
    web_server: String,
) -> ForgeResult<Site> {
    let site = sites::update_web_server(&state.pool, id, &web_server).await?;
    let _ = try_reload(&state.pool, &state.supervisor).await;
    Ok(site)
}

#[tauri::command]
pub async fn add_site_alias(
    state: State<'_, AppState>,
    id: i64,
    domain: String,
) -> ForgeResult<Site> {
    let site = sites::add_alias(&state.pool, id, &domain).await?;
    let _ = try_reload(&state.pool, &state.supervisor).await;
    Ok(site)
}

#[tauri::command]
pub async fn remove_site_alias(
    state: State<'_, AppState>,
    id: i64,
    domain: String,
) -> ForgeResult<Site> {
    let site = sites::remove_alias(&state.pool, id, &domain).await?;
    let _ = try_reload(&state.pool, &state.supervisor).await;
    Ok(site)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteLogsTail {
    pub error: Vec<String>,
    pub access: Vec<String>,
    pub error_missing: bool,
    pub access_missing: bool,
}

#[tauri::command]
pub async fn open_site_url(state: State<'_, AppState>, id: i64) -> ForgeResult<()> {
    let site = sites::fetch_site(&state.pool, id).await?;
    crate::platform::macos::open_url(&format!("http://{}", site.domain))
}

#[tauri::command]
pub async fn reveal_site_path(state: State<'_, AppState>, id: i64) -> ForgeResult<()> {
    let site = sites::fetch_site(&state.pool, id).await?;
    let path = std::path::PathBuf::from(&site.path);
    crate::platform::macos::reveal_path(&path)
}

#[tauri::command]
pub async fn open_site_in_editor(state: State<'_, AppState>, id: i64) -> ForgeResult<()> {
    let site = sites::fetch_site(&state.pool, id).await?;
    let path = std::path::PathBuf::from(&site.path);
    crate::platform::macos::open_in_editor(&path)
}

#[tauri::command]
pub async fn tail_site_logs(state: State<'_, AppState>, id: i64) -> ForgeResult<SiteLogsTail> {
    let site = sites::fetch_site(&state.pool, id).await?;
    let data_dir = crate::store::data_dir();
    let logs_dir = data_dir.join("logs").join("nginx");
    let error_path = logs_dir.join(format!("{}.error.log", site.name));
    let access_path = logs_dir.join(format!("{}.access.log", site.name));

    let error_result = logs::tail_lines(&error_path, 200)?;
    let access_result = logs::tail_lines(&access_path, 200)?;

    Ok(SiteLogsTail {
        error: error_result.lines,
        access: access_result.lines,
        error_missing: error_result.missing,
        access_missing: access_result.missing,
    })
}

async fn try_reload(pool: &sqlx::SqlitePool, supervisor: &ProcessSupervisor) -> ForgeResult<()> {
    let status = supervisor.status(nginx::NGINX_PROCESS).await;
    if matches!(status.state, crate::domain::process::ProcessState::Running) {
        nginx::reload(pool).await
    } else {
        nginx::regenerate(pool).await
    }
}
