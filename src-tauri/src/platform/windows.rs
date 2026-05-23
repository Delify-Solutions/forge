// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Windows implementation of the platform abstraction.
// Stub: real impl arrives in V2.0.

use crate::error::{ForgeError, ForgeResult};

#[allow(dead_code)]
pub fn placeholder() -> ForgeResult<()> {
    Err(ForgeError::NotImplemented(
        "Windows platform impl arrives in V2.0",
    ))
}
