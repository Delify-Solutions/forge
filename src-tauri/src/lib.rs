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

fn cleanup_orphans_from_previous_session() {
    use crate::domain::process::kill_orphan_pidfile;
    kill_orphan_pidfile(&crate::domain::dns::pid_path());
    kill_orphan_pidfile(&crate::domain::nginx::pid_path());
    kill_orphan_pidfile(&crate::domain::apache::pid_path());
    crate::platform::macos::kill_listeners_on_port(80, &["nginx"]);
    crate::platform::macos::kill_listeners_on_port(8288, &["httpd"]);
}

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

    cleanup_orphans_from_previous_session();

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
            commands::sites::update_site_php,
            commands::sites::update_site_web_server,
            commands::sites::update_site_https,
            commands::sites::add_site_alias,
            commands::sites::remove_site_alias,
            commands::sites::open_site_url,
            commands::sites::reveal_site_path,
            commands::sites::open_site_in_editor,
            commands::sites::open_site_terminal,
            commands::sites::tail_site_logs,
            commands::sites::composer_status,
            commands::sites::scaffold_and_add_site,
            commands::certs::mkcert_status,
            commands::certs::install_mkcert_ca,
            commands::wizard::set_dns_port,
            commands::wizard::setup_dns_resolver,
            commands::wizard::start_dnsmasq,
            commands::wizard::stop_dnsmasq,
            commands::wizard::start_nginx,
            commands::wizard::stop_nginx,
            commands::wizard::reload_nginx,
            commands::wizard::start_php_fpm,
            commands::wizard::stop_php_fpm,
            commands::wizard::services_status,
            commands::wizard::debug_reset_environment,
            commands::wizard::start_apache,
            commands::wizard::stop_apache,
            commands::wizard::reload_apache,
            commands::system::open_devtools,
            commands::bundles::list_bundles,
            commands::bundles::install_bundle,
            commands::bundles::uninstall_bundle,
            commands::tools::list_tool_catalog,
            commands::tools::get_preferred_tools,
            commands::tools::set_preferred_tool,
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
