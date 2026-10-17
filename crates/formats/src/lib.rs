// SPDX-License-Identifier: GPL-3.0-or-later

//! File formats: GIF89a and PNG now; `.par` and `.map` later.

pub mod gif;
pub mod png;

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The data is malformed or can't be represented in the format.
    Invalid(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for Error {}
