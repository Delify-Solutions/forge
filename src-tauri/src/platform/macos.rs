// SPDX-License-Identifier: AGPL-3.0-or-later
//
// macOS implementation of the platform abstraction. The MVP exposes only
// detection helpers used by the system scan; lifecycle (osascript, dnsmasq,
// nginx supervision) lands in later bước.

use std::fs;
use std::net::{TcpListener, UdpSocket};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{ForgeError, ForgeResult};

const RESOLVER_PATH: &str = "/etc/resolver/test";
const BREW_PREFIX_CANDIDATES: &[&str] = &["/opt/homebrew", "/usr/local"];
const EDITOR_CANDIDATES: &[&str] = &["code", "cursor", "subl"];
// Clippy note: the items are `&'static str` literals but we omit the explicit
// `'static` annotation on the slice type because `const` items default to
// `'static` lifetime and clippy warns about the redundancy.

pub fn resolver_expected_content(port: u16) -> String {
    format!("nameserver 127.0.0.1\nport {port}\n")
}

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
    //
    // The PHP bundle ships both bin/php and sbin/php-fpm — when the caller
    // asks for "php-fpm" we look inside the "php" bundle for sbin/php-fpm.
    let (bundle_engine, bundle_subpath_override) = match name {
        "nginx" | "php" | "dnsmasq" => (Some(name), None::<&str>),
        "php-fpm" => (Some("php"), Some("sbin/php-fpm")),
        _ => (None, None),
    };
    if let Some(engine) = bundle_engine {
        let entries = crate::domain::bundle::catalog();
        let mut bundle_candidates: Vec<(String, PathBuf)> = entries
            .into_iter()
            .filter(|e| e.engine == engine)
            .filter_map(|e| {
                let subpath = bundle_subpath_override.unwrap_or(&e.bin_subpath);
                let path = crate::domain::bundle::installed_binary(&e.engine, &e.version, subpath)?;
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
    let tcp_in_use = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => false,
        Err(e) => e.kind() == std::io::ErrorKind::AddrInUse,
    };
    let udp_in_use = match UdpSocket::bind(("127.0.0.1", port)) {
        Ok(_) => false,
        Err(e) => e.kind() == std::io::ErrorKind::AddrInUse,
    };
    tcp_in_use || udp_in_use
}

#[allow(dead_code)]
pub fn process_using_port(port: u16) -> Option<String> {
    pids_and_name_using_port(port).map(|(_, name)| name)
}

pub fn pids_and_name_using_port(port: u16) -> Option<(Vec<u32>, String)> {
    let output = Command::new("/usr/sbin/lsof")
        .args(["-nP", "-i", &format!(":{port}"), "-FpcL"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut pids: Vec<u32> = Vec::new();
    let mut first_name: Option<String> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix('p') {
            if let Ok(parsed) = rest.trim().parse::<u32>() {
                pids.push(parsed);
            }
        } else if let Some(rest) = line.strip_prefix('c') {
            let trimmed = rest.trim();
            if !trimmed.is_empty() && first_name.is_none() {
                first_name = Some(trimmed.to_string());
            }
        }
    }
    if pids.is_empty() {
        return None;
    }
    let name = first_name.unwrap_or_default();
    Some((pids, name))
}

pub fn resolver_exists() -> bool {
    Path::new(RESOLVER_PATH).exists()
}

/// Kill any process belonging to the current user listening on `port` whose
/// command name is in `expected_names`. Used to reap nginx/dnsmasq from a
/// previous app instance that detached from the supervisor (e.g. master
/// process re-parented to launchd).
pub fn kill_listeners_on_port(port: u16, expected_names: &[&str]) {
    let Some((pids, name)) = pids_and_name_using_port(port) else {
        return;
    };
    let name_lower = name.to_lowercase();
    if !expected_names
        .iter()
        .any(|n| name_lower.contains(&n.to_lowercase()))
    {
        return;
    }
    for pid in &pids {
        let pid_i = *pid as i32;
        unsafe {
            libc::kill(pid_i, libc::SIGTERM);
        }
    }
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if !port_in_use(port) {
            return;
        }
    }
    for pid in &pids {
        let pid_i = *pid as i32;
        unsafe {
            libc::kill(pid_i, libc::SIGKILL);
        }
    }
}

pub fn resolver_correct(port: u16) -> bool {
    fs::read_to_string(RESOLVER_PATH)
        .map(|content| content == resolver_expected_content(port))
        .unwrap_or(false)
}

/// Write `/etc/resolver/test` via osascript, prompting the user once for
/// admin credentials through the native macOS dialog. Idempotent: if the
/// file already exists with the expected content, returns Ok without
/// prompting.
pub fn setup_resolver(port: u16) -> ForgeResult<()> {
    if resolver_correct(port) {
        tracing::info!("resolver already correct, skipping prompt");
        return Ok(());
    }

    let prompt = "Delify Forge needs admin access to route .test domains to your local machine.";
    let script = format!(
        "do shell script \"mkdir -p /etc/resolver && /usr/bin/printf 'nameserver 127.0.0.1\\nport {port}\\n' > /etc/resolver/test && /bin/chmod 644 /etc/resolver/test\" with administrator privileges with prompt \"{prompt}\""
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

    if !resolver_correct(port) {
        return Err(ForgeError::Other(
            "resolver write reported success but content does not match expected".into(),
        ));
    }
    Ok(())
}

pub fn remove_resolver() -> ForgeResult<()> {
    if !resolver_exists() {
        return Ok(());
    }

    let prompt =
        "Delify Forge needs admin access to reset local .test DNS routing for debug testing.";
    let script = format!(
        "do shell script \"if [ -f {RESOLVER_PATH} ]; then /bin/rm -f {RESOLVER_PATH}; fi\" with administrator privileges with prompt \"{prompt}\""
    );

    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| ForgeError::Other(format!("osascript spawn failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(ForgeError::Other(format!(
            "resolver reset failed (osascript exit {}): {stderr}",
            output.status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

/// Open `url` in the default browser via `/usr/bin/open`.
/// Fire-and-forget: we spawn and drop the child so the command returns
/// immediately without waiting for the browser process.
pub fn open_url(url: &str) -> ForgeResult<()> {
    let _child = Command::new("/usr/bin/open")
        .arg(url)
        .spawn()
        .map_err(|e| ForgeError::Other(format!("open_url failed: {e}")))?;
    // Why: the browser is an external long-lived process; awaiting it would
    // block the Tauri command for as long as the browser window stays open.
    Ok(())
}

/// Reveal `path` in Finder, selecting the folder via `/usr/bin/open -R`.
pub fn reveal_path(path: &Path) -> ForgeResult<()> {
    let _child = Command::new("/usr/bin/open")
        .arg("-R")
        .arg(path)
        .spawn()
        .map_err(|e| ForgeError::Other(format!("reveal_path failed: {e}")))?;
    Ok(())
}

/// Detect the first available editor binary from a hard-coded candidate list.
/// Returns `(binary_path, display_name)` — display_name is the candidate
/// string (e.g. "code", "cursor", "subl").
///
/// We augment `$PATH` with common install locations because Tauri app bundles
/// inherit the launchd `PATH` (`/usr/bin:/bin:/usr/sbin:/sbin`), which omits
/// Homebrew prefixes and per-user bins where editor CLIs typically live.
pub fn detect_editor() -> Option<(PathBuf, &'static str)> {
    let path_env = augmented_path();
    for &name in EDITOR_CANDIDATES {
        for dir in path_env.split(':').filter(|d| !d.is_empty()) {
            let candidate = PathBuf::from(dir).join(name);
            if candidate.is_file() {
                return Some((candidate, name));
            }
        }
    }
    None
}

fn augmented_path() -> String {
    let mut parts: Vec<String> = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    let extra: Vec<String> = {
        let mut v = vec![
            "/opt/homebrew/bin".to_string(),
            "/usr/local/bin".to_string(),
            "/opt/local/bin".to_string(),
        ];
        if let Ok(home) = std::env::var("HOME") {
            v.push(format!("{home}/.local/bin"));
            v.push(format!("{home}/bin"));
        }
        v
    };
    for dir in extra {
        if !parts.iter().any(|p| p == &dir) {
            parts.push(dir);
        }
    }
    parts.join(":")
}

/// Search the augmented PATH for a binary named `name`.
/// Returns the full path if found, `None` otherwise.
pub fn find_on_path(name: &str) -> Option<PathBuf> {
    let path_env = augmented_path();
    for dir in path_env.split(':').filter(|d| !d.is_empty()) {
        let candidate = PathBuf::from(dir).join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Map an editor candidate name to the macOS .app bundle name used by
/// `/usr/bin/open -a`. This is the fallback when the CLI shim is not on PATH
/// (e.g. user installed VS Code but never ran "Install 'code' command in PATH").
fn editor_app_name(candidate: &str) -> Option<&'static str> {
    match candidate {
        "code" => Some("Visual Studio Code"),
        "cursor" => Some("Cursor"),
        "subl" => Some("Sublime Text"),
        _ => None,
    }
}

pub fn application_bundle_exists(app_name: &str) -> bool {
    let user_apps = std::env::var("HOME").ok().map(|h| {
        PathBuf::from(h)
            .join("Applications")
            .join(format!("{app_name}.app"))
    });
    let system_app = PathBuf::from(format!("/Applications/{app_name}.app"));
    if system_app.is_dir() {
        return true;
    }
    if let Some(p) = user_apps {
        if p.is_dir() {
            return true;
        }
    }
    false
}

/// Open a new Terminal.app window with the working directory set to `path`.
/// Fire-and-forget: we spawn and drop the child so the command returns
/// immediately without waiting for Terminal.app to exit.
pub fn open_terminal(path: &Path) -> ForgeResult<()> {
    let path_str = path.to_string_lossy();
    // Escape backslashes and double quotes for the AppleScript string literal.
    // Single quotes do not need escaping inside AppleScript double-quoted strings.
    let escaped = path_str.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"tell application "Terminal"
    activate
    do script "cd \"{escaped}\""
end tell"#
    );
    let _child = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(&script)
        .spawn()
        .map_err(|e| ForgeError::Other(format!("open_terminal failed: {e}")))?;
    Ok(())
}

/// Open `path` in the detected editor. If no editor is found, returns a typed
/// error that the UI can display as a friendly message.
pub fn detect_mkcert() -> Option<DetectedBinary> {
    detect_binary("mkcert", &["-version"])
}

pub fn detect_composer() -> Option<DetectedBinary> {
    detect_binary("composer", &["--version"])
}

/// Execute a `LaunchPlan` produced by `domain::tools::resolve`.
///
/// - `Cli(binary)` → spawn `binary <path>`.
/// - `OpenWithApp(app)` → `/usr/bin/open -a "<app>" <path>`.
/// - `OpenWithArgs { app, args }` → `/usr/bin/open -na "<app>" --args <args...> <path>`.
///   For Warp/Tabby the path is the sole arg; for others the flag prefix is in `args`.
/// - `OsascriptTerminalApp` → today's Terminal.app AppleScript.
/// - `OsascriptIterm2` → iTerm2 dialect AppleScript.
/// - `SystemEditor` → `/usr/bin/open <path>`.
/// - `Auto` → delegates to `open_in_editor` (editor) or `open_terminal` (terminal).
///   The caller must pass the correct `kind` so we know which auto chain to use.
pub fn execute_editor(plan: &crate::domain::tools::LaunchPlan, path: &Path) -> ForgeResult<()> {
    use crate::domain::tools::LaunchPlan;
    match plan {
        LaunchPlan::Cli(binary) => {
            let _child = Command::new(binary)
                .arg(path)
                .spawn()
                .map_err(|e| ForgeError::Other(format!("execute_editor (cli) failed: {e}")))?;
            Ok(())
        }
        LaunchPlan::OpenWithApp(app) => {
            let _child = Command::new("/usr/bin/open")
                .arg("-a")
                .arg(app)
                .arg(path)
                .spawn()
                .map_err(|e| ForgeError::Other(format!("execute_editor (open -a) failed: {e}")))?;
            Ok(())
        }
        LaunchPlan::SystemEditor => {
            let _child = Command::new("/usr/bin/open")
                .arg(path)
                .spawn()
                .map_err(|e| ForgeError::Other(format!("execute_editor (system) failed: {e}")))?;
            Ok(())
        }
        LaunchPlan::Auto => open_in_editor(path),
        other => Err(ForgeError::Other(format!(
            "execute_editor: unexpected plan variant {other:?}"
        ))),
    }
}

/// Execute a terminal `LaunchPlan`.
pub fn execute_terminal(plan: &crate::domain::tools::LaunchPlan, path: &Path) -> ForgeResult<()> {
    use crate::domain::tools::LaunchPlan;
    match plan {
        LaunchPlan::OsascriptTerminalApp | LaunchPlan::Auto => open_terminal(path),
        LaunchPlan::OsascriptIterm2 => open_iterm2(path),
        LaunchPlan::OpenWithArgs { app, args } => {
            let path_str = path.to_string_lossy().to_string();
            let mut cmd = Command::new("/usr/bin/open");
            cmd.arg("-na").arg(app).arg("--args");
            for a in args {
                cmd.arg(a);
            }
            // Ghostty uses `--working-directory=<path>` (flag=value, no space).
            // All other terminals take the path as a separate final argument.
            if app == "Ghostty" {
                cmd.arg(format!("--working-directory={path_str}"));
            } else {
                cmd.arg(&path_str);
            }
            let _child = cmd.spawn().map_err(|e| {
                ForgeError::Other(format!("execute_terminal (open -na) failed: {e}"))
            })?;
            Ok(())
        }
        other => Err(ForgeError::Other(format!(
            "execute_terminal: unexpected plan variant {other:?}"
        ))),
    }
}

/// Open a new iTerm2 window with the working directory set to `path`.
fn open_iterm2(path: &Path) -> ForgeResult<()> {
    let path_str = path.to_string_lossy();
    let escaped = path_str.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"tell application "iTerm"
    activate
    set newWindow to (create window with default profile)
    tell current session of newWindow to write text "cd \"{escaped}\""
end tell"#
    );
    let _child = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(&script)
        .spawn()
        .map_err(|e| ForgeError::Other(format!("open_iterm2 failed: {e}")))?;
    Ok(())
}

pub fn open_in_editor(path: &Path) -> ForgeResult<()> {
    if let Some((binary, _name)) = detect_editor() {
        let _child = Command::new(&binary)
            .arg(path)
            .spawn()
            .map_err(|e| ForgeError::Other(format!("open_in_editor failed: {e}")))?;
        return Ok(());
    }

    for &name in EDITOR_CANDIDATES {
        let Some(app_name) = editor_app_name(name) else {
            continue;
        };
        if application_bundle_exists(app_name) {
            let _child = Command::new("/usr/bin/open")
                .arg("-a")
                .arg(app_name)
                .arg(path)
                .spawn()
                .map_err(|e| ForgeError::Other(format!("open_in_editor failed: {e}")))?;
            return Ok(());
        }
    }

    Err(ForgeError::Other(
        "No editor found. Install VS Code (`code`), Cursor, or Sublime (`subl`).".into(),
    ))
}
