// SPDX-License-Identifier: AGPL-3.0-or-later

use tauri::State;

use crate::domain::sites::{self, AddSiteRequest, Site};
use crate::domain::{nginx, process::ProcessSupervisor};
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

async fn try_reload(pool: &sqlx::SqlitePool, supervisor: &ProcessSupervisor) -> ForgeResult<()> {
    let status = supervisor.status(nginx::NGINX_PROCESS).await;
    if matches!(status.state, crate::domain::process::ProcessState::Running) {
        nginx::reload(pool).await
    } else {
        nginx::regenerate(pool).await
    }
}
