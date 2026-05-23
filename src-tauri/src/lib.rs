// SPDX-License-Identifier: AGPL-3.0-or-later
// Delify Forge — local web development environment.
// Copyright (C) 2026 Delify Solutions.

mod commands;
mod domain;
mod error;
mod platform;
mod store;

use sqlx::SqlitePool;
use tauri::{Manager, RunEvent};
use tracing_subscriber::EnvFilter;

use crate::domain::process::ProcessSupervisor;

pub struct AppState {
    pub pool: SqlitePool,
    pub supervisor: ProcessSupervisor,
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

    let app_state = AppState {
        pool,
        supervisor: ProcessSupervisor::new(),
    };

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::system::scan_system,
            commands::system::ping,
            commands::sites::list_sites,
            commands::sites::add_site,
            commands::sites::remove_site,
            commands::wizard::setup_dns_resolver,
            commands::wizard::start_dnsmasq,
            commands::wizard::stop_dnsmasq,
            commands::wizard::start_nginx,
            commands::wizard::stop_nginx,
            commands::wizard::reload_nginx,
            commands::wizard::services_status,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(move |app_handle, event| {
        if let RunEvent::ExitRequested { .. } = &event {
            tracing::info!("exit requested, shutting down supervised processes");
            let state = app_handle.state::<AppState>();
            tauri::async_runtime::block_on(state.supervisor.shutdown_all());
        }
    });
}
