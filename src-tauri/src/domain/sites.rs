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
    pub php_version: Option<String>,
    pub web_server: Option<String>,
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

fn validate_web_server(value: &str) -> ForgeResult<()> {
    match value {
        "nginx" | "apache" | "openlitespeed" => Ok(()),
        _ => Err(ForgeError::Other(format!(
            "invalid web_server '{value}': must be nginx, apache, or openlitespeed"
        ))),
    }
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
    let web_server = req.web_server.as_deref().unwrap_or("nginx");
    validate_web_server(web_server)?;
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

    let result =
        sqlx::query("INSERT INTO sites (name, path, php_version, web_server) VALUES (?, ?, ?, ?)")
            .bind(&req.name)
            .bind(&req.path)
            .bind(req.php_version.as_deref().unwrap_or("8.3"))
            .bind(web_server)
            .execute(pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(db) if db.is_unique_violation() => {
                    ForgeError::Other(format!("site name '{}' already exists", req.name))
                }
                other => ForgeError::Other(format!("add site: {other}")),
            })?;

    let id = result.last_insert_rowid();

    write_default_landing(path, &req.name);

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

const LANDING_TEMPLATE: &str = include_str!("../templates/landing.php");
const ENTRYPOINTS: [&str; 3] = ["index.php", "index.html", "index.htm"];

pub fn document_root(path: &std::path::Path) -> std::path::PathBuf {
    let public = path.join("public");
    if public.is_dir() && has_entrypoint(&public) {
        public
    } else {
        path.to_path_buf()
    }
}

fn has_entrypoint(dir: &std::path::Path) -> bool {
    ENTRYPOINTS
        .iter()
        .any(|candidate| dir.join(candidate).exists())
}

fn write_default_landing(dir: &std::path::Path, site_name: &str) {
    if !dir.is_dir() || has_entrypoint(dir) || has_entrypoint(&dir.join("public")) {
        return;
    }
    let rendered = LANDING_TEMPLATE.replace("__SITE_NAME__", site_name);
    let target = dir.join("index.php");
    if let Err(err) = std::fs::write(&target, rendered) {
        tracing::warn!(
            site = site_name,
            target = %target.display(),
            "failed to write default landing page: {err}"
        );
    }
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

/// Validate that a php_version string matches `<major>.<minor>` or `<major>.<minor>.<patch>`.
fn validate_php_version(v: &str) -> ForgeResult<()> {
    let parts: Vec<&str> = v.split('.').collect();
    let valid = match parts.len() {
        2 => parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())),
        3 => parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())),
        _ => false,
    };
    if !valid {
        return Err(ForgeError::Other(format!(
            "invalid php_version '{v}': must be <major>.<minor> or <major>.<minor>.<patch> (e.g. 8.3 or 8.3.14)"
        )));
    }
    Ok(())
}

pub async fn update_php_version(
    pool: &SqlitePool,
    id: i64,
    php_version: &str,
) -> ForgeResult<Site> {
    validate_php_version(php_version)?;

    let result = sqlx::query("UPDATE sites SET php_version = ? WHERE id = ?")
        .bind(php_version)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| ForgeError::Other(format!("update php_version: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(ForgeError::Other(format!("site {id} not found")));
    }

    let row = sqlx::query_as::<_, (i64, String, String, String, String, String)>(
        "SELECT id, name, path, php_version, web_server, created_at FROM sites WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| ForgeError::Other(format!("fetch updated site: {e}")))?;

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

pub async fn update_web_server(pool: &SqlitePool, id: i64, web_server: &str) -> ForgeResult<Site> {
    validate_web_server(web_server)?;

    let result = sqlx::query("UPDATE sites SET web_server = ? WHERE id = ?")
        .bind(web_server)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| ForgeError::Other(format!("update web_server: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(ForgeError::Other(format!("site {id} not found")));
    }

    let row = sqlx::query_as::<_, (i64, String, String, String, String, String)>(
        "SELECT id, name, path, php_version, web_server, created_at FROM sites WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| ForgeError::Other(format!("fetch updated site: {e}")))?;

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

    #[test]
    fn accepts_valid_php_versions() {
        assert!(validate_php_version("8.2").is_ok());
        assert!(validate_php_version("8.3").is_ok());
        assert!(validate_php_version("8.3.14").is_ok());
        assert!(validate_php_version("7.4").is_ok());
        assert!(validate_php_version("8.4.21").is_ok());
    }

    #[test]
    fn rejects_invalid_php_versions() {
        assert!(validate_php_version("system").is_err());
        assert!(validate_php_version("8").is_err());
        assert!(validate_php_version("8.x").is_err());
        assert!(validate_php_version("8.3.14.5").is_err());
        assert!(validate_php_version("abc").is_err());
        assert!(validate_php_version("").is_err());
        assert!(validate_php_version(".3").is_err());
        assert!(validate_php_version("8.").is_err());
    }

    #[test]
    fn uses_public_dir_when_it_contains_entrypoint() {
        let root = std::env::temp_dir().join(format!(
            "forge-site-root-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let public = root.join("public");
        std::fs::create_dir_all(&public).unwrap();
        std::fs::write(public.join("index.php"), "<?php").unwrap();

        assert_eq!(document_root(&root), public);

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn keeps_root_when_public_dir_has_no_entrypoint() {
        let root = std::env::temp_dir().join(format!(
            "forge-site-root-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("public")).unwrap();

        assert_eq!(document_root(&root), root);

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn accepts_known_web_servers() {
        assert!(validate_web_server("nginx").is_ok());
        assert!(validate_web_server("apache").is_ok());
        assert!(validate_web_server("openlitespeed").is_ok());
    }

    #[test]
    fn rejects_unknown_web_servers() {
        assert!(validate_web_server("").is_err());
        assert!(validate_web_server("iis").is_err());
        assert!(validate_web_server("Nginx").is_err());
        assert!(validate_web_server("ngnix").is_err());
    }
}
