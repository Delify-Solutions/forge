// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::error::{ForgeError, ForgeResult};

const PLAIN_PHP_TEMPLATE: &str = include_str!("../templates/plain_php.php");
const STATIC_TEMPLATE: &str = include_str!("../templates/static_index.html");

/// Which project skeleton to scaffold before registering the site.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProjectTemplate {
    None,
    PlainPhp,
    Static,
    Laravel,
}

/// Bookkeeping returned by `scaffold` so the caller can roll back safely.
#[derive(Debug, Clone)]
pub struct ScaffoldOutcome {
    /// `true` when Forge created the root folder; `false` when it already existed.
    pub created_root: bool,
}

/// Scaffold a project skeleton into `path` according to `template`.
///
/// - `None`: no-op — returns immediately with `created_root: false`.
/// - `PlainPhp` / `Static`: writes a bundled boilerplate file.
/// - `Laravel`: runs `composer create-project laravel/laravel <path> --prefer-dist`
///   with a 120 s hard timeout.
///
/// Returns `Err` if the folder is non-empty, composer is missing, or composer fails.
pub fn scaffold(template: &ProjectTemplate, path: &Path) -> ForgeResult<ScaffoldOutcome> {
    if *template == ProjectTemplate::None {
        return Ok(ScaffoldOutcome {
            created_root: false,
        });
    }

    let outcome = prepare_folder(path)?;

    match template {
        ProjectTemplate::None => unreachable!(),
        ProjectTemplate::PlainPhp => {
            std::fs::write(path.join("index.php"), PLAIN_PHP_TEMPLATE)
                .map_err(|e| ForgeError::Other(format!("failed to write index.php: {e}")))?;
        }
        ProjectTemplate::Static => {
            std::fs::write(path.join("index.html"), STATIC_TEMPLATE)
                .map_err(|e| ForgeError::Other(format!("failed to write index.html: {e}")))?;
        }
        ProjectTemplate::Laravel => {
            run_composer_create_project(path, &outcome)?;
        }
    }

    Ok(outcome)
}

/// Remove the folder Forge created during a failed scaffold.
///
/// Only acts when `outcome.created_root` is `true`. Ignores `NotFound` errors
/// so double-rollback is safe.
pub fn rollback(path: &Path, outcome: &ScaffoldOutcome) {
    if !outcome.created_root {
        return;
    }
    if let Err(e) = std::fs::remove_dir_all(path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(
                path = %path.display(),
                "scaffold rollback: failed to remove folder: {e}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `true` when the directory has no entries (does not recurse).
fn folder_is_empty(path: &Path) -> bool {
    match std::fs::read_dir(path) {
        Ok(mut iter) => iter.next().is_none(),
        Err(_) => false,
    }
}

/// Ensure the target folder is ready for scaffolding.
///
/// - Missing → create it, `created_root = true`.
/// - Exists and empty → use it, `created_root = false`.
/// - Exists and non-empty → error.
fn prepare_folder(path: &Path) -> ForgeResult<ScaffoldOutcome> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| ForgeError::Other(format!("failed to create folder: {e}")))?;
        return Ok(ScaffoldOutcome { created_root: true });
    }

    if !path.is_dir() {
        return Err(ForgeError::Other(format!(
            "path is not a directory: {}",
            path.display()
        )));
    }

    if !folder_is_empty(path) {
        return Err(ForgeError::Other(format!(
            "folder must be empty (including hidden files like .DS_Store): {}",
            path.display()
        )));
    }

    Ok(ScaffoldOutcome {
        created_root: false,
    })
}

/// Run `composer create-project laravel/laravel <path> --prefer-dist` with a
/// 120 s hard timeout. Captures stderr and surfaces it on failure.
fn run_composer_create_project(path: &Path, outcome: &ScaffoldOutcome) -> ForgeResult<()> {
    let composer_path = find_composer().ok_or_else(|| {
        ForgeError::Other("composer is not installed. Run: brew install composer".into())
    })?;

    let path_str = path.to_string_lossy().to_string();

    let mut child = Command::new(&composer_path)
        .args([
            "create-project",
            "laravel/laravel",
            &path_str,
            "--prefer-dist",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ForgeError::Other(format!("failed to spawn composer: {e}")))?;

    let deadline = Instant::now() + Duration::from_secs(120);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                }
                // Collect stderr for the error message.
                let stderr = child
                    .stderr
                    .take()
                    .and_then(|mut r| {
                        use std::io::Read;
                        let mut buf = String::new();
                        r.read_to_string(&mut buf).ok()?;
                        Some(buf)
                    })
                    .unwrap_or_default();
                let trimmed = stderr.trim().to_string();
                rollback(path, outcome);
                return Err(ForgeError::Other(format!("composer failed: {trimmed}")));
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    rollback(path, outcome);
                    return Err(ForgeError::Other("composer timed out after 120 s".into()));
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            Err(e) => {
                let _ = child.kill();
                rollback(path, outcome);
                return Err(ForgeError::Other(format!("composer wait error: {e}")));
            }
        }
    }
}

/// Locate the `composer` binary using the platform detection helper.
fn find_composer() -> Option<PathBuf> {
    crate::platform::macos::detect_composer().map(|b| b.binary)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "forge-scaffold-{}-{}",
            label,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn plain_php_writes_index_php() {
        let dir = tmp_dir("plain-php");
        std::fs::create_dir_all(&dir).unwrap();

        let outcome = scaffold(&ProjectTemplate::PlainPhp, &dir).unwrap();
        assert!(!outcome.created_root);

        let content = std::fs::read_to_string(dir.join("index.php")).unwrap();
        assert_eq!(content, PLAIN_PHP_TEMPLATE);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn static_writes_index_html() {
        let dir = tmp_dir("static");
        std::fs::create_dir_all(&dir).unwrap();

        let outcome = scaffold(&ProjectTemplate::Static, &dir).unwrap();
        assert!(!outcome.created_root);

        let content = std::fs::read_to_string(dir.join("index.html")).unwrap();
        assert_eq!(content, STATIC_TEMPLATE);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn none_makes_no_filesystem_changes() {
        let dir = tmp_dir("none");
        // Do NOT create the directory — None should not touch the filesystem.
        let outcome = scaffold(&ProjectTemplate::None, &dir).unwrap();
        assert!(!outcome.created_root);
        assert!(!dir.exists(), "None template must not create any folder");
    }

    #[test]
    fn rejects_non_empty_folder() {
        let dir = tmp_dir("nonempty");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("existing.txt"), "data").unwrap();

        let result = scaffold(&ProjectTemplate::PlainPhp, &dir);
        assert!(result.is_err(), "should reject non-empty folder");
        // The existing file must still be there.
        assert!(dir.join("existing.txt").exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn creates_folder_when_missing_and_records_created_root() {
        let dir = tmp_dir("missing");
        assert!(!dir.exists());

        let outcome = scaffold(&ProjectTemplate::PlainPhp, &dir).unwrap();
        assert!(outcome.created_root, "should record created_root = true");
        assert!(dir.join("index.php").exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn existing_empty_folder_records_created_root_false() {
        let dir = tmp_dir("empty");
        std::fs::create_dir_all(&dir).unwrap();

        let outcome = scaffold(&ProjectTemplate::PlainPhp, &dir).unwrap();
        assert!(!outcome.created_root, "should record created_root = false");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rollback_removes_created_folder_and_is_noop_when_not_created() {
        // Case 1: created_root = true → folder should be removed.
        let dir = tmp_dir("rollback-created");
        std::fs::create_dir_all(&dir).unwrap();
        let outcome_created = ScaffoldOutcome { created_root: true };
        rollback(&dir, &outcome_created);
        assert!(!dir.exists(), "rollback should remove the folder");

        // Case 2: created_root = false → folder should remain.
        let dir2 = tmp_dir("rollback-existing");
        std::fs::create_dir_all(&dir2).unwrap();
        let outcome_existing = ScaffoldOutcome {
            created_root: false,
        };
        rollback(&dir2, &outcome_existing);
        assert!(
            dir2.exists(),
            "rollback should not remove pre-existing folder"
        );

        std::fs::remove_dir_all(&dir2).ok();
    }
}
