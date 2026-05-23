// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Linux implementation of the platform abstraction.
// Stub: real impl arrives in V1.0.

use crate::error::{ForgeError, ForgeResult};

#[allow(dead_code)]
pub fn placeholder() -> ForgeResult<()> {
    Err(ForgeError::NotImplemented(
        "Linux platform impl arrives in V1.0",
    ))
}
