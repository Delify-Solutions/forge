// SPDX-License-Identifier: AGPL-3.0-or-later

use tauri::State;

use crate::domain::sites::{self, AddSiteRequest, Site};
use crate::error::ForgeResult;
use crate::AppState;

#[tauri::command]
pub async fn list_sites(state: State<'_, AppState>) -> ForgeResult<Vec<Site>> {
    sites::list(&state.pool).await
}

#[tauri::command]
pub async fn add_site(state: State<'_, AppState>, req: AddSiteRequest) -> ForgeResult<Site> {
    sites::add(&state.pool, req).await
}

#[tauri::command]
pub async fn remove_site(state: State<'_, AppState>, id: i64) -> ForgeResult<()> {
    sites::remove(&state.pool, id).await
}
