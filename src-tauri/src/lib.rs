// SPDX-License-Identifier: AGPL-3.0-or-later
// Delify Forge — local web development environment.
// Copyright (C) 2026 Delify Solutions.

mod commands;
mod error;
#[allow(dead_code)]
mod platform;

use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("starting Delify Forge");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::system::scan_system,
            commands::system::ping,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
