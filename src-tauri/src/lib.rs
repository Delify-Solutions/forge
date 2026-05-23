// SPDX-License-Identifier: AGPL-3.0-or-later
// Delify Forge — local web development environment.
// Copyright (C) 2026 Delify Solutions.

mod commands;
mod domain;
mod error;
mod platform;
mod store;

use sqlx::SqlitePool;
use tracing_subscriber::EnvFilter;

pub struct AppState {
    pub pool: SqlitePool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("starting Delify Forge");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");

    let pool = runtime.block_on(async {
        store::open_pool()
            .await
            .expect("failed to open application database")
    });

    let app_state = AppState { pool };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::system::scan_system,
            commands::system::ping,
            commands::sites::list_sites,
            commands::sites::add_site,
            commands::sites::remove_site,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
