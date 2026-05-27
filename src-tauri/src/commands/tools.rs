// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::Serialize;
use tauri::State;

use crate::domain::tools::{self, MacosDetector, ToolDetector, ToolKind};
use crate::error::ForgeResult;
use crate::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCatalogEntry {
    pub slug: String,
    pub label: String,
    pub cli: Option<String>,
    pub bundle: Option<String>,
    pub installed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCatalog {
    pub editors: Vec<ToolCatalogEntry>,
    pub terminals: Vec<ToolCatalogEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreferredTools {
    pub editor: String,
    pub terminal: String,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Return the full editor and terminal catalogs with per-entry installation
/// status. Detection runs at call time (not cached).
#[tauri::command]
pub async fn list_tool_catalog() -> ForgeResult<ToolCatalog> {
    let detector = MacosDetector;
    let editors = build_entries(ToolKind::Editor, &detector);
    let terminals = build_entries(ToolKind::Terminal, &detector);
    Ok(ToolCatalog { editors, terminals })
}

/// Return the currently persisted preferred editor and terminal slugs.
/// Defaults to `"auto"` when a key is absent.
#[tauri::command]
pub async fn get_preferred_tools(state: State<'_, AppState>) -> ForgeResult<PreferredTools> {
    let editor = tools::read_preference(&state.pool, ToolKind::Editor).await?;
    let terminal = tools::read_preference(&state.pool, ToolKind::Terminal).await?;
    Ok(PreferredTools { editor, terminal })
}

/// Persist the preferred tool for `kind`. Rejects unknown slugs.
#[tauri::command]
pub async fn set_preferred_tool(
    state: State<'_, AppState>,
    kind: ToolKind,
    slug: String,
) -> ForgeResult<()> {
    tools::write_preference(&state.pool, kind, &slug).await
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_entries(kind: ToolKind, detector: &dyn ToolDetector) -> Vec<ToolCatalogEntry> {
    tools::catalog(kind)
        .iter()
        .map(|entry| {
            // `auto` and `system` are always "installed" (they are special slugs,
            // not real tools that need to be detected).
            let installed =
                matches!(entry.slug, "auto" | "system") || tools::is_installed(entry, detector);
            ToolCatalogEntry {
                slug: entry.slug.to_string(),
                label: entry.label.to_string(),
                cli: entry.cli.map(|s| s.to_string()),
                bundle: entry.bundle.map(|s| s.to_string()),
                installed,
            }
        })
        .collect()
}
