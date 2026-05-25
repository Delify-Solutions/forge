// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::error::{ForgeError, ForgeResult};

const MAX_READ_BYTES: u64 = 1024 * 1024; // 1 MiB

/// Result of a `tail_lines` call.
pub struct TailResult {
    pub lines: Vec<String>,
    /// `true` only when the file does not exist (NotFound). Never `true` for
    /// permission or other I/O errors — those surface as `Err`.
    pub missing: bool,
}

/// Read the last `n` lines from `path`.
///
/// - File missing → `Ok(TailResult { lines: [], missing: true })`.
/// - Any other I/O error → `Err(ForgeError::Other(...))`.
/// - Success → `Ok(TailResult { lines: ..., missing: false })`.
///
/// A trailing empty line produced by a final newline is dropped.
pub fn tail_lines(path: &Path, n: usize) -> ForgeResult<TailResult> {
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(TailResult {
                lines: vec![],
                missing: true,
            });
        }
        Err(e) => {
            return Err(ForgeError::Other(format!("read {}: {}", path.display(), e)));
        }
    };

    let file_len = file
        .seek(SeekFrom::End(0))
        .map_err(|e| ForgeError::Other(format!("read {}: {}", path.display(), e)))?;

    let read_from = file_len.saturating_sub(MAX_READ_BYTES);

    file.seek(SeekFrom::Start(read_from))
        .map_err(|e| ForgeError::Other(format!("read {}: {}", path.display(), e)))?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|e| ForgeError::Other(format!("read {}: {}", path.display(), e)))?;

    // Drop trailing empty line from a final newline.
    if buf.ends_with('\n') {
        buf.pop();
        if buf.ends_with('\r') {
            buf.pop();
        }
    }

    let lines: Vec<String> = buf.lines().map(|l| l.to_string()).collect();
    let total = lines.len();
    let start = total.saturating_sub(n);
    Ok(TailResult {
        lines: lines[start..].to_vec(),
        missing: false,
    })
}

/// Walk `candidates` in order and return the first one whose binary exists on
/// PATH. Returns `(PathBuf, display_name)` where `display_name` is the
/// candidate string itself (e.g. "code", "cursor", "subl").
///
/// Factored out of the platform layer so it can be unit-tested without
/// touching the real filesystem.
pub fn select_first_existing(candidates: &[&str]) -> Option<(std::path::PathBuf, String)> {
    let path_env = std::env::var("PATH").unwrap_or_default();
    select_first_existing_in_path(candidates, &path_env)
}

fn select_first_existing_in_path(
    candidates: &[&str],
    path_env: &str,
) -> Option<(std::path::PathBuf, String)> {
    for &name in candidates {
        for dir in path_env.split(':') {
            let candidate = std::path::PathBuf::from(dir).join(name);
            if candidate.is_file() {
                return Some((candidate, name.to_string()));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "forge-tail-test-{}.log",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn empty_file_returns_empty_vec() {
        let p = write_tmp("");
        let result = tail_lines(&p, 10).unwrap();
        assert!(!result.missing);
        assert!(result.lines.is_empty());
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn missing_file_returns_missing_true() {
        let p = std::path::PathBuf::from("/tmp/forge-nonexistent-log-file-xyz.log");
        let result = tail_lines(&p, 10).unwrap();
        assert!(result.missing);
        assert!(result.lines.is_empty());
    }

    #[test]
    fn directory_path_returns_err() {
        let dir = std::env::temp_dir().join(format!(
            "forge-tail-dir-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let result = tail_lines(&dir, 5);
        assert!(result.is_err());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn fewer_than_n_lines_returns_all() {
        let content = "line1\nline2\nline3\n";
        let p = write_tmp(content);
        let result = tail_lines(&p, 10).unwrap();
        assert!(!result.missing);
        assert_eq!(result.lines, vec!["line1", "line2", "line3"]);
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn more_than_n_lines_returns_last_n() {
        let content: String = (1..=20).map(|i| format!("line{i}\n")).collect();
        let p = write_tmp(&content);
        let result = tail_lines(&p, 5).unwrap();
        assert!(!result.missing);
        assert_eq!(result.lines.len(), 5);
        assert_eq!(result.lines[0], "line16");
        assert_eq!(result.lines[4], "line20");
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn large_file_still_returns_last_n() {
        // Build a string > 1 MiB so the seek-from-end path is exercised.
        // Each line is 100 bytes + newline = 101 bytes.
        // 11_000 lines * 101 bytes = ~1.08 MiB.
        let line = "x".repeat(100);
        let content: String = (0..11_000).map(|_| format!("{line}\n")).collect();
        assert!(content.len() > 1024 * 1024, "test data must exceed 1 MiB");
        let p = write_tmp(&content);
        let result = tail_lines(&p, 200).unwrap();
        assert!(!result.missing);
        assert_eq!(result.lines.len(), 200);
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn select_first_existing_returns_none_for_bogus_names() {
        let result = select_first_existing(&["__forge_no_such_editor_xyz__"]);
        assert!(result.is_none());
    }

    #[test]
    fn select_first_existing_picks_first_match() {
        // Write two tiny executable-like files to a temp dir and verify the
        // function returns the first one in the candidate list.
        let dir = std::env::temp_dir().join(format!(
            "forge-editor-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("forge_test_editor_a");
        let b = dir.join("forge_test_editor_b");
        std::fs::write(&a, b"").unwrap();
        std::fs::write(&b, b"").unwrap();

        let path_env = format!("{}:", dir.display());
        let result = select_first_existing_in_path(
            &["forge_test_editor_a", "forge_test_editor_b"],
            &path_env,
        );
        assert!(result.is_some());
        let (path, name) = result.unwrap();
        assert_eq!(name, "forge_test_editor_a");
        assert_eq!(path, a);

        std::fs::remove_dir_all(dir).ok();
    }
}
