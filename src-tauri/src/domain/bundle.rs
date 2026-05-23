// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Engine bundle catalog + downloader. Following CLAUDE.md decision 6 (DBngin
// pattern): we ship no engines inside the .app, but Forge fetches prebuilt
// tar.gz archives on demand into:
//
//   ~/Library/Application Support/Forge/engines/<engine>/<version>/
//
// The catalog is data — for MVP we hardcode entries; once we publish a real
// `forge-engines` repo it can be hot-loaded from a manifest URL.
//
// `install_bundle` streams the tarball to disk while reporting progress via
// the supplied callback, verifies SHA-256 against the catalog entry (when
// present), then untars to a temp dir and atomically renames into place so
// a partial install never leaves a half-extracted directory behind.

use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use futures_util::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{ForgeError, ForgeResult};
use crate::store;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleEntry {
    pub engine: String,
    pub version: String,
    pub display_name: String,
    pub url: String,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub bin_subpath: String,
    pub installed: bool,
    pub install_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum InstallProgress {
    Started { total_bytes: Option<u64> },
    Downloading { downloaded: u64, total: Option<u64> },
    Verifying,
    Extracting,
    Done { install_path: String },
    Failed { message: String },
}

pub fn engines_root() -> PathBuf {
    store::data_dir().join("engines")
}

pub fn bundle_dir(engine: &str, version: &str) -> PathBuf {
    engines_root().join(engine).join(version)
}

/// Return the binary path for an installed bundle, or None if absent.
pub fn installed_binary(engine: &str, version: &str, bin_subpath: &str) -> Option<PathBuf> {
    let path = bundle_dir(engine, version).join(bin_subpath);
    if path.exists() && path.is_file() {
        Some(path)
    } else {
        None
    }
}

/// Iterate all installed bundles for a given engine. Returns sorted versions
/// (lexical, which is fine for the small numeric versions we ship).
#[allow(dead_code)]
pub fn list_installed(engine: &str) -> Vec<String> {
    let root = engines_root().join(engine);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut versions: Vec<String> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    versions.sort();
    versions
}

/// Catalog of bundles Forge knows how to install. Versions are pinned so a
/// rolling upstream change can't surprise users.
///
/// The base URL can be overridden via `FORGE_BUNDLE_BASE_URL` to point a dev
/// build at a local fileserver while the public `forge-engines` release
/// pipeline is still being set up.
pub fn catalog() -> Vec<BundleEntry> {
    let base = std::env::var("FORGE_BUNDLE_BASE_URL").unwrap_or_else(|_| {
        "https://github.com/Delify-Solutions/forge-engines/releases/download".to_string()
    });

    let arch = if cfg!(target_arch = "aarch64") {
        "darwin-arm64"
    } else {
        "darwin-x64"
    };

    let entries = vec![
        ("nginx", "1.27.3", "nginx 1.27.3", "sbin/nginx"),
        ("php", "8.3.14", "PHP 8.3.14", "bin/php"),
        ("php-fpm", "8.3.14", "PHP-FPM 8.3.14", "sbin/php-fpm"),
        ("dnsmasq", "2.90", "dnsmasq 2.90", "sbin/dnsmasq"),
    ];

    entries
        .into_iter()
        .map(|(engine, version, display, bin_subpath)| {
            let archive = format!("{engine}-{version}-{arch}.tar.gz");
            let url = format!("{base}/{engine}-{version}/{archive}");
            let install_dir = bundle_dir(engine, version);
            let installed = installed_binary(engine, version, bin_subpath).is_some();
            BundleEntry {
                engine: engine.to_string(),
                version: version.to_string(),
                display_name: display.to_string(),
                url,
                sha256: None,
                size_bytes: None,
                bin_subpath: bin_subpath.to_string(),
                installed,
                install_path: installed.then(|| install_dir.to_string_lossy().to_string()),
            }
        })
        .collect()
}

pub fn find_entry(engine: &str, version: Option<&str>) -> Option<BundleEntry> {
    let mut matches: Vec<BundleEntry> = catalog()
        .into_iter()
        .filter(|e| e.engine == engine)
        .collect();
    if matches.is_empty() {
        return None;
    }
    if let Some(v) = version {
        matches.into_iter().find(|e| e.version == v)
    } else {
        matches.sort_by(|a, b| a.version.cmp(&b.version));
        matches.pop()
    }
}

pub async fn install_bundle<F>(entry: &BundleEntry, mut on_progress: F) -> ForgeResult<PathBuf>
where
    F: FnMut(InstallProgress) + Send + 'static,
{
    let install_dir = bundle_dir(&entry.engine, &entry.version);
    if install_dir.exists() {
        on_progress(InstallProgress::Done {
            install_path: install_dir.to_string_lossy().to_string(),
        });
        return Ok(install_dir);
    }

    let parent = install_dir
        .parent()
        .ok_or_else(|| ForgeError::Other("bundle install dir has no parent".into()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| ForgeError::Other(format!("create engines dir {}: {e}", parent.display())))?;

    let tmp_archive = parent.join(format!(
        ".{}-{}.tar.gz.partial",
        entry.engine, entry.version
    ));
    if tmp_archive.exists() {
        let _ = std::fs::remove_file(&tmp_archive);
    }

    on_progress(InstallProgress::Started {
        total_bytes: entry.size_bytes,
    });

    let client = reqwest::Client::builder()
        .user_agent(concat!("delify-forge/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| ForgeError::Other(format!("build http client: {e}")))?;

    let response = client
        .get(&entry.url)
        .send()
        .await
        .map_err(|e| ForgeError::Other(format!("download {} failed: {e}", entry.url)))?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(ForgeError::Other(format!(
            "download {} returned HTTP {status}",
            entry.url
        )));
    }

    let total = response.content_length().or(entry.size_bytes);

    use std::io::Write;
    let mut file = std::fs::File::create(&tmp_archive)
        .map_err(|e| ForgeError::Other(format!("create {}: {e}", tmp_archive.display())))?;
    let mut hasher = Sha256::new();
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| ForgeError::Other(format!("download stream: {e}")))?;
        hasher.update(&bytes);
        file.write_all(&bytes)
            .map_err(|e| ForgeError::Other(format!("write archive: {e}")))?;
        downloaded += bytes.len() as u64;
        on_progress(InstallProgress::Downloading { downloaded, total });
    }
    file.flush()
        .map_err(|e| ForgeError::Other(format!("flush archive: {e}")))?;
    drop(file);

    if let Some(expected) = &entry.sha256 {
        on_progress(InstallProgress::Verifying);
        let got = hex::encode(hasher.finalize());
        if !got.eq_ignore_ascii_case(expected) {
            let _ = std::fs::remove_file(&tmp_archive);
            return Err(ForgeError::Other(format!(
                "sha256 mismatch for {}: expected {expected}, got {got}",
                entry.engine
            )));
        }
    }

    on_progress(InstallProgress::Extracting);
    let staging = parent.join(format!(".{}-{}.staging", entry.engine, entry.version));
    if staging.exists() {
        std::fs::remove_dir_all(&staging)
            .map_err(|e| ForgeError::Other(format!("clean staging {}: {e}", staging.display())))?;
    }
    std::fs::create_dir_all(&staging)
        .map_err(|e| ForgeError::Other(format!("create staging {}: {e}", staging.display())))?;

    extract_tar_gz(&tmp_archive, &staging)?;

    if install_dir.exists() {
        let _ = std::fs::remove_dir_all(&install_dir);
    }
    std::fs::rename(&staging, &install_dir).map_err(|e| {
        ForgeError::Other(format!(
            "promote {} -> {}: {e}",
            staging.display(),
            install_dir.display()
        ))
    })?;

    let _ = std::fs::remove_file(&tmp_archive);

    let install_path = install_dir.to_string_lossy().to_string();
    on_progress(InstallProgress::Done {
        install_path: install_path.clone(),
    });
    Ok(install_dir)
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> ForgeResult<()> {
    let file = std::fs::File::open(archive)
        .map_err(|e| ForgeError::Other(format!("open {}: {e}", archive.display())))?;
    let gz = GzDecoder::new(file);
    let mut tar = tar::Archive::new(gz);
    tar.set_preserve_permissions(true);
    tar.unpack(dest)
        .map_err(|e| ForgeError::Other(format!("untar into {}: {e}", dest.display())))?;
    drop_first_level_if_single(dest)?;
    Ok(())
}

/// If the tarball has a single top-level directory (the common case for
/// upstream releases), move its contents up one level so the layout becomes
/// `<install_dir>/bin/...` instead of `<install_dir>/<engine>-<version>/bin/...`.
fn drop_first_level_if_single(dest: &Path) -> ForgeResult<()> {
    let entries: Vec<_> = std::fs::read_dir(dest)
        .map_err(|e| ForgeError::Other(format!("read dest {}: {e}", dest.display())))?
        .flatten()
        .collect();

    if entries.len() != 1 {
        return Ok(());
    }
    let single = &entries[0];
    if !single.file_type().map(|t| t.is_dir()).unwrap_or(false) {
        return Ok(());
    }

    let inner = single.path();
    let inner_entries: Vec<_> = std::fs::read_dir(&inner)
        .map_err(|e| ForgeError::Other(format!("read inner {}: {e}", inner.display())))?
        .flatten()
        .collect();
    for child in inner_entries {
        let target = dest.join(child.file_name());
        std::fs::rename(child.path(), &target).map_err(|e| {
            ForgeError::Other(format!(
                "lift {} -> {}: {e}",
                child.path().display(),
                target.display()
            ))
        })?;
    }
    let _ = std::fs::remove_dir(&inner);
    Ok(())
}

pub fn uninstall_bundle(engine: &str, version: &str) -> ForgeResult<()> {
    let dir = bundle_dir(engine, version);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)
            .map_err(|e| ForgeError::Other(format!("remove {}: {e}", dir.display())))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain_archive_to_disk(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn catalog_has_known_engines() {
        let cat = catalog();
        assert!(cat.iter().any(|e| e.engine == "nginx"));
        assert!(cat.iter().any(|e| e.engine == "php"));
        assert!(cat.iter().any(|e| e.engine == "php-fpm"));
        assert!(cat.iter().any(|e| e.engine == "dnsmasq"));
        for entry in &cat {
            assert!(entry
                .url
                .contains(&format!("{}-{}", entry.engine, entry.version)));
            assert!(!entry.bin_subpath.is_empty());
        }
    }

    #[test]
    fn find_entry_returns_pinned_version() {
        let entry = find_entry("nginx", Some("1.27.3")).expect("nginx 1.27.3 in catalog");
        assert_eq!(entry.engine, "nginx");
        assert_eq!(entry.version, "1.27.3");
    }

    #[test]
    fn extract_tar_gz_with_top_level_dir() {
        use std::io::Read as _;
        let tmp = tempdir();
        // Build a tar.gz in memory: outer/bin/hello.
        let mut buf = Vec::new();
        {
            use std::io::Write as _;
            let gz = flate2::write::GzEncoder::new(&mut buf, flate2::Compression::default());
            let mut tar = tar::Builder::new(gz);
            let mut header = tar::Header::new_gnu();
            header.set_size(5);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, "outer/bin/hello", "world".as_bytes())
                .unwrap();
            let mut gz_inner = tar.into_inner().unwrap();
            gz_inner.flush().unwrap();
            gz_inner.finish().unwrap();
        }
        let archive_path = drain_archive_to_disk(&tmp, "test.tar.gz", &buf);
        let dest = tmp.join("dest");
        std::fs::create_dir_all(&dest).unwrap();
        extract_tar_gz(&archive_path, &dest).unwrap();
        let lifted = dest.join("bin").join("hello");
        assert!(lifted.exists(), "expected {} to exist", lifted.display());
        let mut content = String::new();
        std::fs::File::open(lifted)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(content, "world");
    }

    fn tempdir() -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "forge-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).unwrap();
        base
    }
}
