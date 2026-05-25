// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

use crate::error::{ForgeError, ForgeResult};
use crate::store;

pub fn cert_dir() -> PathBuf {
    store::data_dir().join("runtime").join("certs")
}

pub struct CertPaths {
    pub crt: PathBuf,
    pub key: PathBuf,
    pub hosts: PathBuf,
}

pub fn cert_paths(name: &str) -> CertPaths {
    let dir = cert_dir();
    CertPaths {
        crt: dir.join(format!("{name}.crt")),
        key: dir.join(format!("{name}.key")),
        hosts: dir.join(format!("{name}.hosts")),
    }
}

fn sorted_hosts(hosts: &[String]) -> Vec<String> {
    let mut sorted = hosts.to_vec();
    sorted.sort();
    sorted
}

fn hosts_file_content(hosts: &[String]) -> String {
    let sorted = sorted_hosts(hosts);
    let mut content = String::new();
    for h in sorted {
        content.push_str(&h);
        content.push('\n');
    }
    content
}

fn read_hosts_file(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn hosts_match_disk(path: &Path, hosts: &[String]) -> bool {
    let expected = hosts_file_content(hosts);
    read_hosts_file(path)
        .map(|content| content == expected)
        .unwrap_or(false)
}

pub fn ensure_cert(name: &str, hosts: &[String]) -> ForgeResult<()> {
    let mkcert = crate::platform::macos::detect_binary("mkcert", &["-version"])
        .ok_or_else(|| ForgeError::Other("mkcert not found".into()))?;
    ensure_cert_with(
        &move |crt, key, h| run_mkcert(&mkcert.binary, crt, key, h),
        name,
        hosts,
    )
}

#[allow(clippy::type_complexity)]
pub fn ensure_cert_with(
    runner: &dyn Fn(&Path, &Path, &[String]) -> ForgeResult<()>,
    name: &str,
    hosts: &[String],
) -> ForgeResult<()> {
    let paths = cert_paths(name);
    let all_exist = paths.crt.exists() && paths.key.exists() && paths.hosts.exists();

    if all_exist && hosts_match_disk(&paths.hosts, hosts) {
        return Ok(());
    }

    std::fs::create_dir_all(cert_dir())
        .map_err(|e| ForgeError::Other(format!("create cert dir: {e}")))?;

    let sorted = sorted_hosts(hosts);
    runner(&paths.crt, &paths.key, &sorted)?;

    std::fs::write(&paths.hosts, hosts_file_content(&sorted))
        .map_err(|e| ForgeError::Other(format!("write hosts file: {e}")))?;

    Ok(())
}

fn run_mkcert(binary: &Path, crt: &Path, key: &Path, hosts: &[String]) -> ForgeResult<()> {
    let mut cmd = std::process::Command::new(binary);
    cmd.arg("-cert-file").arg(crt);
    cmd.arg("-key-file").arg(key);
    for h in hosts {
        cmd.arg(h);
    }
    let output = cmd
        .output()
        .map_err(|e| ForgeError::Other(format!("mkcert spawn: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ForgeError::Other(format!("mkcert failed: {stderr}")));
    }
    Ok(())
}

pub fn delete_cert(name: &str) {
    let paths = cert_paths(name);
    for path in [paths.crt, paths.key, paths.hosts] {
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
}

pub fn ca_installed() -> bool {
    let Some(mkcert) = crate::platform::macos::detect_binary("mkcert", &["-version"]) else {
        return false;
    };
    let output = std::process::Command::new(&mkcert.binary)
        .arg("-CAROOT")
        .output()
        .ok();
    let ca_root = output.and_then(|o| {
        if o.status.success() {
            Some(PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
        } else {
            None
        }
    });
    let ca_pem_exists = ca_root
        .as_ref()
        .map(|p| p.join("rootCA.pem").exists())
        .unwrap_or(false);
    if !ca_pem_exists {
        return false;
    }
    let check = std::process::Command::new("/usr/bin/security")
        .args([
            "find-certificate",
            "-c",
            "mkcert",
            "/Library/Keychains/System.keychain",
        ])
        .output()
        .ok();
    check.map(|o| o.status.success()).unwrap_or(false)
}

pub fn install_ca() -> ForgeResult<()> {
    let Some(mkcert) = crate::platform::macos::detect_binary("mkcert", &["-version"]) else {
        return Err(ForgeError::Other("mkcert not found".into()));
    };

    let prompt =
        "Delify Forge needs admin access to install the local mkcert CA into the system keychain.";
    let script = format!(
        "do shell script \"{} -install\" with administrator privileges with prompt \"{prompt}\"",
        mkcert.binary.display()
    );

    let output = std::process::Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| ForgeError::Other(format!("osascript spawn failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(ForgeError::Other(format!(
            "CA install failed (osascript exit {}): {stderr}",
            output.status.code().unwrap_or(-1)
        )));
    }

    if !ca_installed() {
        return Err(ForgeError::Other(
            "CA install reported success but the keychain check still fails — try again or run `mkcert -install` in Terminal".into(),
        ));
    }
    Ok(())
}

pub fn uninstall_ca() -> ForgeResult<()> {
    let Some(mkcert) = crate::platform::macos::detect_binary("mkcert", &["-version"]) else {
        return Ok(());
    };

    let prompt = "Delify Forge needs admin access to uninstall the local mkcert CA from the system keychain.";
    let script = format!(
        "do shell script \"{} -uninstall\" with administrator privileges with prompt \"{prompt}\"",
        mkcert.binary.display()
    );

    let output = std::process::Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| ForgeError::Other(format!("osascript spawn failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(ForgeError::Other(format!(
            "CA uninstall failed (osascript exit {}): {stderr}",
            output.status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn ensure_cert_skips_when_hosts_match() {
        let name = "skip-test";
        let hosts = vec!["a.test".to_string(), "b.test".to_string()];
        let real_dir = cert_dir();
        let real_crt = real_dir.join(format!("{name}.crt"));
        let real_key = real_dir.join(format!("{name}.key"));
        let real_hosts = real_dir.join(format!("{name}.hosts"));

        for p in [&real_crt, &real_key, &real_hosts] {
            let _ = std::fs::remove_file(p);
        }
        std::fs::create_dir_all(&real_dir).unwrap();
        std::fs::write(&real_crt, "crt").unwrap();
        std::fs::write(&real_key, "key").unwrap();
        std::fs::write(&real_hosts, "a.test\nb.test\n").unwrap();

        let count = AtomicUsize::new(0);
        let runner = |_crt: &Path, _key: &Path, _hosts: &[String]| -> ForgeResult<()> {
            count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        };

        ensure_cert_with(&runner, name, &hosts).unwrap();
        assert_eq!(
            count.load(Ordering::SeqCst),
            0,
            "runner should not be called when hosts match"
        );

        for p in [&real_crt, &real_key, &real_hosts] {
            let _ = std::fs::remove_file(p);
        }
    }

    #[test]
    fn ensure_cert_regenerates_when_hosts_change() {
        let name = "regen-test";
        let real_dir = cert_dir();
        let real_crt = real_dir.join(format!("{name}.crt"));
        let real_key = real_dir.join(format!("{name}.key"));
        let real_hosts = real_dir.join(format!("{name}.hosts"));

        for p in [&real_crt, &real_key, &real_hosts] {
            let _ = std::fs::remove_file(p);
        }
        std::fs::create_dir_all(&real_dir).unwrap();
        std::fs::write(&real_crt, "crt").unwrap();
        std::fs::write(&real_key, "key").unwrap();
        std::fs::write(&real_hosts, "old.test\n").unwrap();

        let count = AtomicUsize::new(0);
        let runner = |_crt: &Path, _key: &Path, _hosts: &[String]| -> ForgeResult<()> {
            count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        };

        let new_hosts = vec!["a.test".to_string(), "b.test".to_string()];
        ensure_cert_with(&runner, name, &new_hosts).unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);

        let written = std::fs::read_to_string(&real_hosts).unwrap();
        assert_eq!(written, "a.test\nb.test\n");

        for p in [&real_crt, &real_key, &real_hosts] {
            let _ = std::fs::remove_file(p);
        }
    }

    #[test]
    fn ensure_cert_regenerates_when_files_missing() {
        let name = "missing-test";
        let real_dir = cert_dir();
        let real_crt = real_dir.join(format!("{name}.crt"));
        let real_key = real_dir.join(format!("{name}.key"));
        let real_hosts = real_dir.join(format!("{name}.hosts"));

        for p in [&real_crt, &real_key, &real_hosts] {
            let _ = std::fs::remove_file(p);
        }
        std::fs::create_dir_all(&real_dir).unwrap();
        // Only crt exists
        std::fs::write(&real_crt, "crt").unwrap();

        let count = AtomicUsize::new(0);
        let runner = |_crt: &Path, _key: &Path, _hosts: &[String]| -> ForgeResult<()> {
            count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        };

        let hosts = vec!["a.test".to_string()];
        ensure_cert_with(&runner, name, &hosts).unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);

        for p in [&real_crt, &real_key, &real_hosts] {
            let _ = std::fs::remove_file(p);
        }
    }

    #[test]
    fn delete_cert_is_idempotent() {
        let name = "delete-test";
        let real_dir = cert_dir();
        let real_crt = real_dir.join(format!("{name}.crt"));
        let real_key = real_dir.join(format!("{name}.key"));
        let real_hosts = real_dir.join(format!("{name}.hosts"));

        std::fs::create_dir_all(&real_dir).unwrap();
        std::fs::write(&real_crt, "crt").unwrap();
        std::fs::write(&real_key, "key").unwrap();
        std::fs::write(&real_hosts, "a.test\n").unwrap();

        delete_cert(name);
        assert!(!real_crt.exists());
        assert!(!real_key.exists());
        assert!(!real_hosts.exists());

        // Second delete should not panic
        delete_cert(name);
    }
}
