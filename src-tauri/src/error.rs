// SPDX-License-Identifier: AGPL-3.0-or-later

use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum ForgeError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not implemented yet: {0}")]
    NotImplemented(&'static str),

    #[error("{0}")]
    Other(String),
}

impl serde::Serialize for ForgeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type ForgeResult<T> = Result<T, ForgeError>;
