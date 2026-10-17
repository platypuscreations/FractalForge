// SPDX-License-Identifier: GPL-3.0-or-later

//! Complex math, the fractal type registry, fractal parameters, the
//! iteration buffer and colouring.
//!
//! See `docs/CLEANROOM.md` before contributing.

pub mod color;
pub mod complex;
pub mod iter;
pub mod orbit_color;
pub mod params;
pub mod types;

pub use color::{PRESETS, Palette, color_index, pixel_value};
pub use complex::Complex;
pub use iter::{INSIDE, IterBuffer, UNKNOWN};
pub use params::{Inside, MAX_ITER_LIMIT, Outside, Params, Window};
pub use types::{FractalType, Symmetry};
