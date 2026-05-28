// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::domain::scaffold::{self, ProjectTemplate};
use crate::domain::sites::{self, AddSiteRequest, Site};
use crate::domain::tools::{self, MacosDetector, ToolKind};
use crate::domain::{apache, logs, nginx, process::ProcessSupervisor};
use crate::error::{ForgeError, ForgeResult};
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
pub async fn update_site_https(
    state: State<'_, AppState>,
    id: i64,
    enabled: bool,
) -> ForgeResult<Site> {
    if enabled {
        let status = crate::commands::certs::mkcert_status();
        if !status.found {
            return Err(ForgeError::Other(
                "mkcert is not installed. Run: brew install mkcert nss".into(),
            ));
        }
        if !status.ca_installed {
            return Err(ForgeError::Other(
                "mkcert CA is not installed. Click \"Install local CA\" in the banner.".into(),
            ));
        }
    }

    let site = sites::update_https_enabled(&state.pool, id, enabled).await?;
    if let Err(e) = try_reload(&state.pool, &state.supervisor).await {
        // Roll back on failure so the DB doesn't say enabled while config failed.
        let _ = sites::update_https_enabled(&state.pool, id, !enabled).await;
        return Err(e);
    }
    Ok(site)
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
    let slug = tools::read_preference(&state.pool, ToolKind::Editor).await?;
    let detector = MacosDetector;
    let plan = tools::resolve(ToolKind::Editor, &slug, &detector);
    crate::platform::macos::execute_editor(&plan, &path)
}

#[tauri::command]
pub async fn open_site_terminal(state: State<'_, AppState>, id: i64) -> ForgeResult<()> {
    let site = sites::fetch_site(&state.pool, id).await?;
    let path = std::path::PathBuf::from(&site.path);
    let slug = tools::read_preference(&state.pool, ToolKind::Terminal).await?;
    let detector = MacosDetector;
    let plan = tools::resolve(ToolKind::Terminal, &slug, &detector);
    crate::platform::macos::execute_terminal(&plan, &path)
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

/// Status of the `composer` binary on the user's PATH.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComposerStatus {
    pub found: bool,
    pub version: Option<String>,
    pub source: Option<String>,
}

#[tauri::command]
pub fn composer_status() -> ComposerStatus {
    match crate::platform::macos::detect_composer() {
        Some(binary) => ComposerStatus {
            found: true,
            version: binary.version,
            source: Some(binary.source),
        },
        None => ComposerStatus {
            found: false,
            version: None,
            source: None,
        },
    }
}

/// Request body for `scaffold_and_add_site`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldAndAddSiteRequest {
    pub template: ProjectTemplate,
    #[serde(flatten)]
    pub site: AddSiteRequest,
}

#[tauri::command]
pub async fn scaffold_and_add_site(
    state: State<'_, AppState>,
    req: ScaffoldAndAddSiteRequest,
) -> ForgeResult<Site> {
    let path = std::path::PathBuf::from(&req.site.path);

    // Run scaffolding synchronously on a blocking thread so we don't block
    // the async executor during the (potentially long) composer run.
    let template = req.template.clone();
    let path_clone = path.clone();
    let outcome = tokio::task::spawn_blocking(move || scaffold::scaffold(&template, &path_clone))
        .await
        .map_err(|e| ForgeError::Other(e.to_string()))??;

    // Register the site in the DB.
    let site_result = sites::add(&state.pool, req.site).await;

    match site_result {
        Ok(site) => {
            let _ = try_reload(&state.pool, &state.supervisor).await;
            Ok(site)
        }
        Err(e) => {
            // DB insert failed — roll back any folder Forge created.
            scaffold::rollback(&path, &outcome);
            Err(e)
        }
    }
}

async fn try_reload(pool: &sqlx::SqlitePool, supervisor: &ProcessSupervisor) -> ForgeResult<()> {
    let nginx_status = supervisor.status(nginx::NGINX_PROCESS).await;
    if matches!(
        nginx_status.state,
        crate::domain::process::ProcessState::Running
    ) {
        nginx::reload(pool).await?;
    } else {
        nginx::regenerate(pool).await?;
    }

    apache::regenerate(pool).await?;
    let apache_status = supervisor.status(apache::APACHE_PROCESS).await;
    if matches!(
        apache_status.state,
        crate::domain::process::ProcessState::Running
    ) {
        apache::reload(pool).await?;
    }

    Ok(())
}
