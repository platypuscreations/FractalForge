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

/// A Bresenham line from one pixel to another, both ends included, whose
/// ties go to the larger coordinate (lsystem.md §6.5, attractors-3d.md
/// §2.3). Pixels outside the image are skipped.
pub(crate) fn line(buf: &mut IterBuffer, (x0, y0): (i64, i64), (x1, y1): (i64, i64), colour: u32) {
    let (w, h) = (buf.width() as i64, buf.height() as i64);
    let mut set = |x: i64, y: i64| {
        if (0..w).contains(&x) && (0..h).contains(&y) {
            buf.set(x as u32, y as u32, colour);
        }
    };
    let (dx, dy) = (x1 - x0, y1 - y0);
    let n = dx.abs().max(dy.abs());
    if n == 0 {
        return set(x0, y0);
    }
    // ⌊a + t·d/D + ½⌋ exactly, with D > 0.
    let along = |a: i64, t: i64, d: i64, big: i64| -> i64 {
        let (d, big) = if big < 0 { (-d, -big) } else { (d, big) };
        let num = (2 * a as i128 * big as i128) + 2 * t as i128 * d as i128 + big as i128;
        num.div_euclid(2 * big as i128) as i64
    };
    for i in 0..=n {
        if dx.abs() >= dy.abs() {
            let x = x0 + i * dx.signum();
            set(x, along(y0, x - x0, dy, dx));
        } else {
            let y = y0 + i * dy.signum();
            set(along(x0, y - y0, dx, dy), y);
        }
    }
}
