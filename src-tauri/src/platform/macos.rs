// SPDX-License-Identifier: AGPL-3.0-or-later
//
// macOS implementation of the platform abstraction.
//
// MVP scaffolding only — actual osascript / dnsmasq / launchd wiring lands
// in Bước 8 onward.

use crate::error::{ForgeError, ForgeResult};

pub fn placeholder() -> ForgeResult<()> {
    Err(ForgeError::NotImplemented(
        "macOS platform impl arrives in Bước 8",
    ))
}
