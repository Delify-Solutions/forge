// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Platform abstraction. Cross-platform-ready architecture, but only the
// macOS implementation is wired up for MVP. Linux and Windows arrive in
// V1.0 and V2.0 respectively.

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[allow(unused_imports)]
#[cfg(target_os = "macos")]
pub use macos as current;

#[allow(unused_imports)]
#[cfg(target_os = "linux")]
pub use linux as current;

#[allow(unused_imports)]
#[cfg(target_os = "windows")]
pub use windows as current;
