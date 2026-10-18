// SPDX-License-Identifier: GPL-3.0-or-later

//! Complex math, the fractal type registry, fractal parameters, the
//! iteration buffer and colouring.
//!
//! See `docs/CLEANROOM.md` before contributing.

mod attractors;
pub mod bignum;
pub mod color;
pub mod complex;
pub mod crand;
pub mod deep;
pub mod ext;
pub mod funcs;
pub mod ifs;
pub mod iter;
pub mod lsystem;
pub mod orbit_color;
mod orbit_maps;
pub mod params;
pub mod plot;
pub mod program;
pub mod types;

pub use color::{PRESETS, Palette, color_index, pixel_value};
pub use complex::Complex;
pub use iter::{INSIDE, IterBuffer, UNKNOWN};
pub use params::{Inside, MAX_ITER_LIMIT, Outside, PARAM_COUNT, Params, View3d, Window};
pub use plot::{PlotFn, Plotter};
pub use program::{Carry, FractalProgram, PixelContext, Program};
pub use types::{FractalType, Symmetry};
