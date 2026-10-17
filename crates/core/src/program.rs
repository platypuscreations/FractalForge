// SPDX-License-Identifier: GPL-3.0-or-later

//! Fractal types defined at run time, such as Fractint-format formulas.
//!
//! The built-in [`crate::types::FORMULA`] type dispatches to the program
//! attached to [`crate::Params::program`].

use std::fmt;
use std::sync::Arc;

use crate::{Complex, Params, Symmetry};

/// Where a point is: its value and its pixel position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PixelContext {
    pub pixel: Complex,
    pub col: u32,
    pub row: u32,
    pub width: u32,
    pub height: u32,
}

/// State a program keeps from one pixel to the next (Fractint formulas
/// keep their variables). Opaque to the renderer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Carry(pub Vec<Complex>);

/// A compiled run-time fractal definition.
pub trait FractalProgram: Send + Sync {
    /// The entry name (e.g. a formula's name in its `.frm` file).
    fn name(&self) -> &str;
    /// The full source text the program was compiled from, so files can be
    /// self-contained.
    fn source(&self) -> &str;
    /// State at the start of an image.
    fn new_carry(&self, params: &Params, width: u32, height: u32) -> Carry;
    /// The colour value of one point (see [`crate::types`]), updating the
    /// carried state.
    fn iterate(&self, at: &PixelContext, params: &Params, carry: &mut Carry) -> u32;
    /// Whether pixels depend on the pixels computed before them (through
    /// the carried state). Such programs are rendered one pixel at a time
    /// in Fractint's order and can't be resumed.
    fn order_dependent(&self) -> bool;
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
