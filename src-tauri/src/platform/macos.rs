// SPDX-License-Identifier: AGPL-3.0-or-later
//
// macOS implementation of the platform abstraction. The MVP exposes only
// detection helpers used by the system scan; lifecycle (osascript, dnsmasq,
// nginx supervision) lands in later bước.

use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{ForgeError, ForgeResult};

const RESOLVER_PATH: &str = "/etc/resolver/test";
const RESOLVER_EXPECTED: &str = "nameserver 127.0.0.1\nport 5353\n";
const BREW_PREFIX_CANDIDATES: &[&str] = &["/opt/homebrew", "/usr/local"];

#[derive(Debug, Clone)]
pub struct DetectedBinary {
    pub binary: PathBuf,
    pub version: Option<String>,
    pub source: String,
}

pub fn brew_prefix() -> Option<PathBuf> {
    if let Ok(output) = Command::new("brew").arg("--prefix").output() {
        if output.status.success() {
            let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !raw.is_empty() {
                let path = PathBuf::from(raw);
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    for candidate in BREW_PREFIX_CANDIDATES {
        let path = Path::new(candidate);
        if path.join("bin/brew").exists() {
            return Some(path.to_path_buf());
        }
    }
    None
}

pub fn detect_binary(name: &str, version_args: &[&str]) -> Option<DetectedBinary> {
    let prefix = brew_prefix();

    let mut candidates: Vec<(PathBuf, &'static str)> = Vec::new();

    // Highest priority: bundles Forge has installed itself. We probe every
    // installed version of the engine so a user with multiple PHPs lined up
    // still gets a deterministic answer (newest version wins, by lexical
    // sort of pinned x.y.z strings).
    let bundle_engine = match name {
        "php-fpm" => Some("php-fpm"),
        "nginx" | "php" | "dnsmasq" => Some(name),
        _ => None,
    };
    if let Some(engine) = bundle_engine {
        let entries = crate::domain::bundle::catalog();
        let mut bundle_candidates: Vec<(String, PathBuf)> = entries
            .into_iter()
            .filter(|e| e.engine == engine)
            .filter_map(|e| {
                let path =
                    crate::domain::bundle::installed_binary(&e.engine, &e.version, &e.bin_subpath)?;
                Some((e.version, path))
            })
            .collect();
        bundle_candidates.sort_by(|a, b| a.0.cmp(&b.0));
        if let Some((_, path)) = bundle_candidates.into_iter().next_back() {
            candidates.push((path, "forge"));
        }
    }

    if let Some(prefix) = &prefix {
        candidates.push((prefix.join("bin").join(name), "brew"));
        candidates.push((prefix.join("sbin").join(name), "brew"));
    }
    if let Ok(path_env) = std::env::var("PATH") {
        for dir in path_env.split(':') {
            candidates.push((PathBuf::from(dir).join(name), "system"));
        }
    }

    for (path, source) in candidates {
        if path.exists() && path.is_file() {
            let version = read_version(&path, version_args);
            return Some(DetectedBinary {
                binary: path,
                version,
                source: source.to_string(),
            });
        }
    }
    None
}

fn read_version(binary: &Path, version_args: &[&str]) -> Option<String> {
    let output = Command::new(binary).args(version_args).output().ok()?;
    let combined = if !output.stdout.is_empty() {
        output.stdout
    } else {
        output.stderr
    };
    let text = String::from_utf8_lossy(&combined);
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

pub fn port_in_use(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_err()
}

pub fn process_using_port(port: u16) -> Option<String> {
    let output = Command::new("/usr/sbin/lsof")
        .args(["-nP", "-iTCP", &format!(":{port}"), "-sTCP:LISTEN", "-Fc"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix('c') {
            let name = rest.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

pub fn resolver_exists() -> bool {
    Path::new(RESOLVER_PATH).exists()
}

pub fn resolver_correct() -> bool {
    fs::read_to_string(RESOLVER_PATH)
        .map(|content| content == RESOLVER_EXPECTED)
        .unwrap_or(false)
}

/// Write `/etc/resolver/test` via osascript, prompting the user once for
/// admin credentials through the native macOS dialog. Idempotent: if the
/// file already exists with the expected content, returns Ok without
/// prompting.
pub fn setup_resolver() -> ForgeResult<()> {
    if resolver_correct() {
        tracing::info!("resolver already correct, skipping prompt");
        return Ok(());
    }

    // Use printf with octal escape so the AppleScript shell text stays
    // straightforward to embed.
    let prompt = "Delify Forge needs admin access to route .test domains to your local machine.";
    let script = format!(
        "do shell script \"mkdir -p /etc/resolver && /usr/bin/printf 'nameserver 127.0.0.1\\nport 5353\\n' > /etc/resolver/test && /bin/chmod 644 /etc/resolver/test\" with administrator privileges with prompt \"{prompt}\""
    );

    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| ForgeError::Other(format!("osascript spawn failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(ForgeError::Other(format!(
            "resolver setup failed (osascript exit {}): {stderr}",
            output.status.code().unwrap_or(-1)
        )));
    }

    if !resolver_correct() {
        return Err(ForgeError::Other(
            "resolver write reported success but content does not match expected".into(),
        ));
    }
    Ok(())
}
