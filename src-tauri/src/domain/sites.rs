// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::error::{ForgeError, ForgeResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub domain: String,
    pub php_version: String,
    pub web_server: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSiteRequest {
    pub name: String,
    pub path: String,
}

fn validate_name(name: &str) -> ForgeResult<()> {
    if name.is_empty() || name.len() > 63 {
        return Err(ForgeError::Other("name must be 1-63 characters".into()));
    }
    let valid = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid || name.starts_with('-') || name.ends_with('-') {
        return Err(ForgeError::Other(
            "name must be kebab-case: lowercase letters, digits, dashes (no leading/trailing dash)"
                .into(),
        ));
    }
    Ok(())
}

pub async fn list(pool: &SqlitePool) -> ForgeResult<Vec<Site>> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, String, String)>(
        "SELECT id, name, path, php_version, web_server, created_at FROM sites ORDER BY created_at DESC, id DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ForgeError::Other(format!("list sites: {e}")))?;

    Ok(rows
        .into_iter()
        .map(
            |(id, name, path, php_version, web_server, created_at)| Site {
                domain: format!("{name}.test"),
                id,
                name,
                path,
                php_version,
                web_server,
                created_at,
            },
        )
        .collect())
}

pub async fn add(pool: &SqlitePool, req: AddSiteRequest) -> ForgeResult<Site> {
    validate_name(&req.name)?;
    let path = std::path::Path::new(&req.path);
    if !path.exists() {
        return Err(ForgeError::Other(format!(
            "path does not exist: {}",
            req.path
        )));
    }
    if !path.is_dir() {
        return Err(ForgeError::Other(format!(
            "path is not a directory: {}",
            req.path
        )));
    }

    let result = sqlx::query("INSERT INTO sites (name, path) VALUES (?, ?)")
        .bind(&req.name)
        .bind(&req.path)
        .execute(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                ForgeError::Other(format!("site name '{}' already exists", req.name))
            }
            other => ForgeError::Other(format!("add site: {other}")),
        })?;

    let id = result.last_insert_rowid();

    let row = sqlx::query_as::<_, (i64, String, String, String, String, String)>(
        "SELECT id, name, path, php_version, web_server, created_at FROM sites WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| ForgeError::Other(format!("fetch added site: {e}")))?;

    Ok(Site {
        domain: format!("{}.test", row.1),
        id: row.0,
        name: row.1,
        path: row.2,
        php_version: row.3,
        web_server: row.4,
        created_at: row.5,
    })
}

pub async fn remove(pool: &SqlitePool, id: i64) -> ForgeResult<()> {
    let result = sqlx::query("DELETE FROM sites WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| ForgeError::Other(format!("remove site: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(ForgeError::Other(format!("site {id} not found")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_kebab_case() {
        assert!(validate_name("myapp").is_ok());
        assert!(validate_name("my-app").is_ok());
        assert!(validate_name("app-2").is_ok());
        assert!(validate_name("a").is_ok());
    }

    #[test]
    fn rejects_invalid_names() {
        assert!(validate_name("").is_err());
        assert!(validate_name("MyApp").is_err());
        assert!(validate_name("my_app").is_err());
        assert!(validate_name("-leading").is_err());
        assert!(validate_name("trailing-").is_err());
        assert!(validate_name("space app").is_err());
        let long = "a".repeat(64);
        assert!(validate_name(&long).is_err());
    }
}
