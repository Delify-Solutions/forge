// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;
use std::sync::OnceLock;

use directories_next::ProjectDirs;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::error::{ForgeError, ForgeResult};

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
