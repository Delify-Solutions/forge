// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;
use std::sync::OnceLock;

use directories_next::ProjectDirs;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::error::{ForgeError, ForgeResult};

pub const DNS_PORT_SETTING_KEY: &str = "dns_port";
pub const DEFAULT_DNS_PORT: u16 = 5533;

pub fn is_valid_dns_port(port: u16) -> bool {
    port > 0
}

pub async fn get_setting(pool: &SqlitePool, key: &str) -> ForgeResult<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| ForgeError::Other(format!("read setting {key}: {e}")))?;
    Ok(row.map(|r| r.0))
}

pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> ForgeResult<()> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value"
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(|e| ForgeError::Other(format!("write setting {key}: {e}")))?;
    Ok(())
}

pub async fn dns_port(pool: &SqlitePool) -> ForgeResult<u16> {
    let value = get_setting(pool, DNS_PORT_SETTING_KEY).await?;
    let parsed = value
        .as_deref()
        .and_then(|raw| raw.parse::<u16>().ok())
        .filter(|port| is_valid_dns_port(*port));
    Ok(parsed.unwrap_or(DEFAULT_DNS_PORT))
}

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn data_dir() -> &'static PathBuf {
    DATA_DIR.get_or_init(|| {
        ProjectDirs::from("vn", "Delify", "Forge")
            .map(|d| d.data_dir().to_path_buf())
            .unwrap_or_else(|| std::env::temp_dir().join("delify-forge"))
    })
}

pub async fn open_pool() -> ForgeResult<SqlitePool> {
    let dir = data_dir();
    std::fs::create_dir_all(dir)?;
    let db_path = dir.join("forge.db");

    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .map_err(|e| ForgeError::Other(format!("open db: {e}")))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| ForgeError::Other(format!("migrate: {e}")))?;

    tracing::info!("opened sqlite at {}", db_path.display());
    Ok(pool)
}
