// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Generic process supervisor for engines (nginx, dnsmasq, php-fpm) the
// app spawns and keeps an eye on. The implementation is deliberately
// minimal for MVP: start, stop, status. Auto-restart on crash is V0.2.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::error::{ForgeError, ForgeResult};

#[cfg(target_os = "macos")]
fn disclaim_trampoline() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let candidates = [
        dir.join("forge-disclaim"),
        dir.join("../Resources/forge-disclaim"),
        dir.join("../Resources/_up_/forge-disclaim"),
    ];
    candidates.into_iter().find(|c| c.is_file())
}

#[cfg(not(target_os = "macos"))]
fn disclaim_trampoline() -> Option<PathBuf> {
    None
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProcessState {
    Stopped,
    Running,
    Crashed,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessStatus {
    pub name: String,
    pub state: ProcessState,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ProcessSpec {
    pub name: String,
    pub binary: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

struct Entry {
    #[allow(dead_code)]
    spec: ProcessSpec,
    child: Option<Child>,
}

#[derive(Default)]
pub struct ProcessSupervisor {
    inner: Arc<Mutex<HashMap<String, Entry>>>,
}

impl ProcessSupervisor {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(&self, spec: ProcessSpec) -> ForgeResult<u32> {
        let mut guard = self.inner.lock().await;

        if let Some(entry) = guard.get_mut(&spec.name) {
            if let Some(child) = entry.child.as_mut() {
                if let Ok(None) = child.try_wait() {
                    let pid = child.id().unwrap_or(0);
                    return Ok(pid);
                }
            }
        }

        let mut cmd = match disclaim_trampoline() {
            Some(trampoline) => {
                let mut c = Command::new(trampoline);
                c.arg(&spec.binary);
                c.args(&spec.args);
                c
            }
            None => {
                let mut c = Command::new(&spec.binary);
                c.args(&spec.args);
                c
            }
        };
        cmd.kill_on_drop(true)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());
        for (k, v) in &spec.env {
            cmd.env(k, v);
        }

        let child = cmd.spawn().map_err(|e| {
            ForgeError::Other(format!(
                "spawn {} ({}): {e}",
                spec.name,
                spec.binary.display()
            ))
        })?;
        let pid = child.id().unwrap_or(0);

        guard.insert(
            spec.name.clone(),
            Entry {
                spec: spec.clone(),
                child: Some(child),
            },
        );

        tracing::info!(
            name = %spec.name,
            pid,
            binary = %spec.binary.display(),
            "process started"
        );
        Ok(pid)
    }

    pub async fn stop(&self, name: &str) -> ForgeResult<()> {
        let mut guard = self.inner.lock().await;
        if let Some(entry) = guard.get_mut(name) {
            if let Some(mut child) = entry.child.take() {
                if let Some(id) = child.id() {
                    tracing::info!(name = %name, pid = id, "process stopping");
                }
                child.start_kill().ok();
                tokio::time::timeout(std::time::Duration::from_secs(3), child.wait())
                    .await
                    .ok();
            }
        }
        Ok(())
    }

    pub async fn status(&self, name: &str) -> ProcessStatus {
        let mut guard = self.inner.lock().await;
        let entry = match guard.get_mut(name) {
            Some(e) => e,
            None => {
                return ProcessStatus {
                    name: name.to_string(),
                    state: ProcessState::Stopped,
                    pid: None,
                };
            }
        };

        let Some(child) = entry.child.as_mut() else {
            return ProcessStatus {
                name: name.to_string(),
                state: ProcessState::Stopped,
                pid: None,
            };
        };

        match child.try_wait() {
            Ok(None) => ProcessStatus {
                name: name.to_string(),
                state: ProcessState::Running,
                pid: child.id(),
            },
            Ok(Some(_status)) => ProcessStatus {
                name: name.to_string(),
                state: ProcessState::Crashed,
                pid: None,
            },
            Err(_) => ProcessStatus {
                name: name.to_string(),
                state: ProcessState::Crashed,
                pid: None,
            },
        }
    }

    pub async fn statuses(&self) -> Vec<ProcessStatus> {
        let mut guard = self.inner.lock().await;
        let names: Vec<String> = guard.keys().cloned().collect();
        let mut out = Vec::with_capacity(names.len());
        for name in names {
            if let Some(entry) = guard.get_mut(&name) {
                let state = match entry.child.as_mut().map(|c| c.try_wait()) {
                    Some(Ok(None)) => ProcessState::Running,
                    Some(Ok(Some(_))) => ProcessState::Crashed,
                    Some(Err(_)) => ProcessState::Crashed,
                    None => ProcessState::Stopped,
                };
                let pid = entry.child.as_ref().and_then(|c| c.id());
                out.push(ProcessStatus { name, state, pid });
            }
        }
        out
    }

    pub async fn running_pids(&self) -> Vec<u32> {
        let mut guard = self.inner.lock().await;
        let names: Vec<String> = guard.keys().cloned().collect();
        let mut out = Vec::with_capacity(names.len());
        for name in names {
            if let Some(entry) = guard.get_mut(&name) {
                if let Some(child) = entry.child.as_mut() {
                    if let Ok(None) = child.try_wait() {
                        if let Some(pid) = child.id() {
                            out.push(pid);
                        }
                    }
                }
            }
        }
        out
    }

    pub async fn shutdown_all(&self) {
        let names: Vec<String> = {
            let guard = self.inner.lock().await;
            guard.keys().cloned().collect()
        };
        for name in names {
            let _ = self.stop(&name).await;
        }
    }
}

/// Best-effort: if `pidfile` contains a live PID, send SIGTERM, wait briefly,
/// then SIGKILL. Used to clean up engine processes left behind by a previous
/// app instance that didn't shut down cleanly (e.g. binary was replaced).
pub fn kill_orphan_pidfile(pidfile: &std::path::Path) {
    let Ok(content) = std::fs::read_to_string(pidfile) else {
        return;
    };
    let Ok(pid) = content.trim().parse::<i32>() else {
        let _ = std::fs::remove_file(pidfile);
        return;
    };
    if pid <= 1 {
        let _ = std::fs::remove_file(pidfile);
        return;
    }
    unsafe {
        if libc::kill(pid, 0) != 0 {
            let _ = std::fs::remove_file(pidfile);
            return;
        }
        libc::kill(pid, libc::SIGTERM);
    }
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        unsafe {
            if libc::kill(pid, 0) != 0 {
                let _ = std::fs::remove_file(pidfile);
                return;
            }
        }
    }
    unsafe {
        libc::kill(pid, libc::SIGKILL);
    }
    let _ = std::fs::remove_file(pidfile);
}
