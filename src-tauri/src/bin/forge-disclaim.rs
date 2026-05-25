// SPDX-License-Identifier: AGPL-3.0-or-later
//
// forge-disclaim — tiny trampoline used by the Forge supervisor on macOS.
//
// Why: when the Forge .app bundle launches a child process directly, macOS
// (Sequoia+) inherits a "provenance sandbox" responsibility chain. The child
// (e.g. php-fpm) ends up unable to read user files outside paths macOS has
// auto-allowed for Forge — even when the file mode would otherwise permit it.
// The kernel reports this as EPERM ("Operation not permitted").
//
// Apple's private API `responsibility_spawnattrs_setdisclaim` opts a child
// out of that responsibility chain. We can't call it from `tokio::process::
// Command` directly, so this binary is the smallest stable trampoline:
//
//   forge-disclaim <real-binary> [args...]
//
// It posix_spawns <real-binary> with the disclaim attribute set, then exits
// with the same PID being tracked by the parent supervisor. Because we
// posix_spawn (not exec), the supervisor sees this trampoline's PID, and
// forge-disclaim then waits on the real child so the supervisor's wait stays
// correct.

use std::env;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::process::ExitCode;
use std::ptr;

#[allow(non_camel_case_types)]
type posix_spawnattr_t = *mut libc::c_void;

extern "C" {
    fn posix_spawnattr_init(attr: *mut posix_spawnattr_t) -> c_int;
    fn posix_spawnattr_destroy(attr: *mut posix_spawnattr_t) -> c_int;
    fn posix_spawn(
        pid: *mut libc::pid_t,
        path: *const c_char,
        file_actions: *const libc::c_void,
        attrp: *const posix_spawnattr_t,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> c_int;

    static environ: *const *mut c_char;
}

type DisclaimFn = unsafe extern "C" fn(*mut posix_spawnattr_t, c_int) -> c_int;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("forge-disclaim: usage: {} <binary> [args...]", args[0]);
        return ExitCode::from(2);
    }

    let bin = CString::new(args[1].clone()).expect("binary path");
    let mut argv_storage: Vec<CString> = args[1..]
        .iter()
        .map(|a| CString::new(a.clone()).expect("arg"))
        .collect();
    let mut argv: Vec<*mut c_char> = argv_storage
        .iter_mut()
        .map(|s| s.as_ptr() as *mut c_char)
        .collect();
    argv.push(ptr::null_mut());

    unsafe {
        let mut attr: posix_spawnattr_t = ptr::null_mut();
        if posix_spawnattr_init(&mut attr) != 0 {
            eprintln!("forge-disclaim: posix_spawnattr_init failed");
            return ExitCode::from(1);
        }

        let sym = libc::dlsym(
            libc::RTLD_DEFAULT,
            c"responsibility_spawnattrs_setdisclaim".as_ptr() as *const c_char,
        );
        if !sym.is_null() {
            let f: DisclaimFn = std::mem::transmute(sym);
            let _ = f(&mut attr, 1);
        }

        let mut pid: libc::pid_t = 0;
        let rc = posix_spawn(
            &mut pid,
            bin.as_ptr(),
            ptr::null(),
            &attr,
            argv.as_ptr(),
            environ,
        );
        posix_spawnattr_destroy(&mut attr);
        if rc != 0 {
            eprintln!(
                "forge-disclaim: posix_spawn({}) failed: {}",
                args[1],
                std::io::Error::from_raw_os_error(rc)
            );
            return ExitCode::from(1);
        }

        let mut status: c_int = 0;
        if libc::waitpid(pid, &mut status, 0) < 0 {
            return ExitCode::from(1);
        }
        if libc::WIFEXITED(status) {
            return ExitCode::from(libc::WEXITSTATUS(status) as u8);
        }
        if libc::WIFSIGNALED(status) {
            return ExitCode::from(128 + libc::WTERMSIG(status) as u8);
        }
        ExitCode::from(0)
    }
}
