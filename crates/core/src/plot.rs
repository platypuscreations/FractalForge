// SPDX-License-Identifier: GPL-3.0-or-later

//! Types that draw points or lines into the image (IFS, L-systems,
//! attractors) instead of computing each pixel on its own.
//!
//! A plotter owns its whole calculation; the renderer only calls it in
//! slices so it can show progress and stop. The buffer starts at colour
//! value 0 (the background) and the plotter writes palette indices.

use crate::IterBuffer;

pub trait Plotter: Send {
    /// Does up to `budget` units of work (points or segments).
    fn run(&mut self, buffer: &mut IterBuffer, budget: u64);
    fn is_done(&self) -> bool;
    /// Fraction of the work done, 0..=1.
    fn progress(&self) -> f64;
}

/// Starts a plotted image of `width` × `height` pixels.
pub type PlotFn =
    fn(params: &crate::Params, width: u32, height: u32) -> Result<Box<dyn Plotter>, String>;
