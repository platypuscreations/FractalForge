// SPDX-License-Identifier: GPL-3.0-or-later

//! Fractal types defined at run time, such as Fractint-format formulas.
//!
//! The built-in [`crate::types::FORMULA`] type dispatches to the program
//! attached to [`crate::Params::program`].

use std::fmt;
use std::sync::Arc;

use crate::{Complex, Params, Symmetry};

/// A compiled run-time fractal definition.
pub trait FractalProgram: Send + Sync {
    /// The entry name (e.g. a formula's name in its `.frm` file).
    fn name(&self) -> &str;
    /// The full source text the program was compiled from, so files can be
    /// self-contained.
    fn source(&self) -> &str;
    /// The colour value of one point (see [`crate::types`]).
    fn iterate(&self, pixel: Complex, params: &Params) -> u32;
    /// The mirror copying these parameters allow.
    fn symmetry(&self, params: &Params) -> Symmetry;
}

/// A shared program. Two are equal when their name and source are.
#[derive(Clone)]
pub struct Program(pub Arc<dyn FractalProgram>);

impl Program {
    pub fn new(p: impl FractalProgram + 'static) -> Self {
        Self(Arc::new(p))
    }
}

impl PartialEq for Program {
    fn eq(&self, other: &Self) -> bool {
        self.0.name() == other.0.name() && self.0.source() == other.0.source()
    }
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Program").field(&self.0.name()).finish()
    }
}
