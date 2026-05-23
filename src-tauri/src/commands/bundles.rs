// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Tauri command surface for engine bundles: list catalog, install (with
// streaming progress over an ipc Channel), uninstall.

use tauri::ipc::Channel;

use crate::domain::bundle::{self, BundleEntry, InstallProgress};
use crate::error::{ForgeError, ForgeResult};

#[tauri::command]
pub async fn list_bundles() -> ForgeResult<Vec<BundleEntry>> {
    Ok(bundle::catalog())
}

#[tauri::command]
pub async fn install_bundle(
    engine: String,
    version: Option<String>,
    on_progress: Channel<InstallProgress>,
) -> ForgeResult<BundleEntry> {
    let entry = bundle::find_entry(&engine, version.as_deref()).ok_or_else(|| {
        ForgeError::Other(format!(
            "no bundle in catalog for engine={engine} version={version:?}"
        ))
    })?;

    let progress_channel = on_progress.clone();
    let result = bundle::install_bundle(&entry, move |p| {
        if let Err(err) = progress_channel.send(p) {
            tracing::warn!(?err, "failed to forward install progress");
        }
    })
    .await;

    match result {
        Ok(_) => {
            // Re-read catalog so the returned entry has installed=true.
            let updated = bundle::find_entry(&engine, Some(&entry.version)).unwrap_or(entry);
            Ok(updated)
        }
        Err(e) => {
            let _ = on_progress.send(InstallProgress::Failed {
                message: e.to_string(),
            });
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn uninstall_bundle(engine: String, version: String) -> ForgeResult<()> {
    bundle::uninstall_bundle(&engine, &version)
}
