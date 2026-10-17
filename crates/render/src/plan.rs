// SPDX-License-Identifier: GPL-3.0-or-later

//! The render plan: which pixels are computed in which order and which are
//! copied by symmetry, derived deterministically from the parameters, image
//! size and render state.
//!
//! Symmetry copies exactly the pixels Fractint-format renderers copy
//! (docs/specs/types.md §1.6), because off-centre views give different
//! pixels when copied than when computed.

use fractalforge_core::{Carry, Complex, IterBuffer, Params, PixelContext, Symmetry, UNKNOWN};

use crate::checkpoint::{PassMode, RenderState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stage {
    /// Visit the grid of computed pixels whose positions in the computed
    /// row and column lists are multiples of `step`.
    Level { step: u32, first: bool, guess: bool },
    /// Compute pixels of copied rows that no mirror filled (origin
    /// symmetry, where the mirrored column falls outside the image).
    Residue,
}

#[derive(Clone, Debug)]
pub(crate) struct Plan {
    pub width: u32,
    pub height: u32,
    pub stages: Vec<Stage>,
    symmetry: Symmetry,
    /// Rows and columns that are computed, in order.
    rows: Vec<u32>,
    cols: Vec<u32>,
    /// For a computed row/column, the row/column copied from it, if any.
    row_copy: Vec<Option<u32>>,
    col_copy: Vec<Option<u32>>,
    /// Origin symmetry: the column reflection constant T.
    t: i64,
    /// Copied rows, for the residue stage.
    residue_rows: Vec<u32>,
}

/// Rows `r` with `s/2 < r <= s` are copies of row `s - r` (§1.6). Returns
/// for each index the index it is copied from, if any.
fn copies(n: u32, s: Option<i64>) -> Vec<Option<u32>> {
    (0..n as i64)
        .map(|r| match s {
            Some(s) if 2 * r > s && r <= s => Some((s - r) as u32),
            _ => None,
        })
        .collect()
}

impl Plan {
    pub fn new(params: &Params, width: u32, height: u32, state: &RenderState) -> Self {
        let w = &params.window;
        let symmetry = if params.use_symmetry && !params.coloring_breaks_symmetry() {
            (params.fractal.symmetry)(params)
        } else {
            Symmetry::None
        };
        // S = round(2·ymax/Δy) and T = round(−2·xmin/Δx).
        let s = (height > 1)
            .then(|| (2.0 * w.y_max / ((w.y_max - w.y_min) / (height - 1) as f64)).round() as i64);
        let t = (width > 1)
            .then(|| (2.0 * -w.x_min / ((w.x_max - w.x_min) / (width - 1) as f64)).round() as i64);
        let (row_from, col_from) = match symmetry {
            Symmetry::None => (copies(height, None), copies(width, None)),
            Symmetry::XAxis | Symmetry::Origin => (copies(height, s), copies(width, None)),
            Symmetry::YAxis => (copies(height, None), copies(width, t)),
            Symmetry::XY => (copies(height, s), copies(width, t)),
        };

        let invert = |from: &[Option<u32>]| {
            let mut to = vec![None; from.len()];
            for (i, f) in from.iter().enumerate() {
                if let Some(src) = f {
                    to[*src as usize] = Some(i as u32);
                }
            }
            to
        };
        let rows: Vec<u32> = (0..height)
            .filter(|&r| row_from[r as usize].is_none())
            .collect();
        let cols: Vec<u32> = (0..width)
            .filter(|&c| col_from[c as usize].is_none())
            .collect();
        let residue_rows: Vec<u32> = match symmetry {
            Symmetry::Origin => (0..height)
                .filter(|&r| row_from[r as usize].is_some())
                .collect(),
            _ => Vec::new(),
        };

        let mut stages = Vec::new();
        let guess = state.pass_mode == PassMode::Guess;
        let mut step = state.coarsest_step;
        let mut first = true;
        while step >= 1 {
            stages.push(Stage::Level { step, first, guess });
            first = false;
            step /= 2;
        }
        if !residue_rows.is_empty() {
            stages.push(Stage::Residue);
        }

        Self {
            width,
            height,
            stages,
            symmetry,
            row_copy: invert(&row_from),
            col_copy: invert(&col_from),
            rows,
            cols,
            t: t.unwrap_or(0),
            residue_rows,
        }
    }

    /// Number of rows in a stage.
    pub fn stage_rows(&self, stage: usize) -> u32 {
        match self.stages[stage] {
            Stage::Level { step, .. } => (self.rows.len() as u32).div_ceil(step),
            Stage::Residue => self.residue_rows.len() as u32,
        }
    }

    pub fn total_rows(&self) -> u64 {
        (0..self.stages.len())
            .map(|s| self.stage_rows(s) as u64)
            .sum()
    }

    pub fn rows_done(&self, state: &RenderState) -> u64 {
        let before: u64 = (0..(state.stage as usize).min(self.stages.len()))
            .map(|s| self.stage_rows(s) as u64)
            .sum();
        before + state.row as u64
    }

    /// Computes (or guesses) the pixels of row `index` in `stage`, reading
    /// only pixels finished by earlier stages.
    pub fn row_work(
        &self,
        stage: usize,
        index: u32,
        params: &Params,
        buf: &IterBuffer,
        carry: &mut Carry,
    ) -> (u32, Vec<(u32, u32)>) {
        let px = Pixels::new(params, self.width, self.height);
        match self.stages[stage] {
            Stage::Level { step, first, guess } => {
                let vy = index * step;
                let y = self.rows[vy as usize];
                let (start, stride) = if first || vy % (2 * step) != 0 {
                    (0, step)
                } else {
                    (step, 2 * step)
                };
                let out = (start..self.cols.len() as u32)
                    .step_by(stride as usize)
                    .map(|vx| {
                        let x = self.cols[vx as usize];
                        // Pixels already known (taken from another program's
                        // partial image) are kept. In our own renders a
                        // pixel is never known before its stage visits it.
                        let known = buf.get(x, y);
                        if known != UNKNOWN {
                            return (x, known);
                        }
                        let guessed = if guess && !first {
                            self.guess(buf, vx, vy, step)
                        } else {
                            None
                        };
                        (x, guessed.unwrap_or_else(|| px.iterate(x, y, carry)))
                    })
                    .collect();
                (y, out)
            }
            Stage::Residue => {
                let y = self.residue_rows[index as usize];
                let out = buf
                    .row(y)
                    .iter()
                    .enumerate()
                    .filter(|&(_, &v)| v == UNKNOWN)
                    .map(|(x, _)| (x as u32, px.iterate(x as u32, y, carry)))
                    .collect();
                (y, out)
            }
        }
    }

    /// The value to fill without computing, if the 4×4 neighbourhood of
    /// coarser-grid points around this pixel's cell is uniform. Positions
    /// are indices into the computed row and column lists.
    fn guess(&self, buf: &IterBuffer, vx: u32, vy: u32, step: u32) -> Option<u32> {
        let s2 = 2 * step as i64;
        let last_x = (self.cols.len() as i64 - 1) / s2 * s2;
        let last_y = (self.rows.len() as i64 - 1) / s2 * s2;
        let cx0 = vx as i64 / s2 * s2;
        let cy0 = vy as i64 / s2 * s2;
        let mut value = None;
        for j in -1..=2 {
            let cy = self.rows[(cy0 + j * s2).clamp(0, last_y) as usize];
            for i in -1..=2 {
                let cx = self.cols[(cx0 + i * s2).clamp(0, last_x) as usize];
                let v = buf.get(cx, cy);
                match value {
                    None => value = Some(v),
                    Some(prev) if prev != v => return None,
                    _ => {}
                }
            }
        }
        value.filter(|&v| v != UNKNOWN)
    }

    /// Writes one row's results and their mirror copies.
    pub fn apply(&self, buf: &mut IterBuffer, y: u32, values: &[(u32, u32)], params: &Params) {
        let mirror = params.fractal.mirror_value;
        let reflect = |v: u32| mirror.map_or(v, |f| f(v, params));
        let row_copy = self.row_copy[y as usize];
        for &(x, v) in values {
            buf.set(x, y, v);
            match self.symmetry {
                Symmetry::None => {}
                Symmetry::XAxis => {
                    if let Some(ry) = row_copy {
                        buf.set(x, ry, reflect(v));
                    }
                }
                Symmetry::YAxis => {
                    if let Some(rx) = self.col_copy[x as usize] {
                        buf.set(rx, y, v);
                    }
                }
                Symmetry::XY => {
                    let col_copy = self.col_copy[x as usize];
                    if let Some(rx) = col_copy {
                        buf.set(rx, y, v);
                    }
                    if let Some(ry) = row_copy {
                        buf.set(x, ry, v);
                        if let Some(rx) = col_copy {
                            buf.set(rx, ry, v);
                        }
                    }
                }
                Symmetry::Origin => {
                    let mx = self.t - x as i64;
                    if let Some(ry) = row_copy
                        && (0..self.width as i64).contains(&mx)
                    {
                        buf.set(mx as u32, ry, v);
                    }
                }
            }
        }
    }
}

/// Maps pixel coordinates to the complex plane.
struct Pixels<'a> {
    params: &'a Params,
    dx: f64,
    dy: f64,
    width: u32,
    height: u32,
}

impl<'a> Pixels<'a> {
    fn new(params: &'a Params, width: u32, height: u32) -> Self {
        let w = &params.window;
        let span = |lo: f64, hi: f64, n: u32| {
            if n > 1 {
                (hi - lo) / (n - 1) as f64
            } else {
                0.0
            }
        };
        Self {
            params,
            dx: span(w.x_min, w.x_max, width),
            dy: span(w.y_min, w.y_max, height),
            width,
            height,
        }
    }

    fn iterate(&self, x: u32, y: u32, carry: &mut Carry) -> u32 {
        let w = &self.params.window;
        let c = Complex::new(w.x_min + x as f64 * self.dx, w.y_max - y as f64 * self.dy);
        if self.params.program.is_some() {
            let at = PixelContext {
                pixel: c,
                col: x,
                row: y,
                width: self.width,
                height: self.height,
            };
            return fractalforge_core::color::program_value(&at, self.params, carry);
        }
        fractalforge_core::pixel_value(c, self.params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_rule() {
        // S = 6: rows 4, 5, 6 copy rows 2, 1, 0; row 3 (the axis) is computed.
        let c = copies(9, Some(6));
        assert_eq!(c[..4], [None, None, None, None]);
        assert_eq!(c[4..7], [Some(2), Some(1), Some(0)]);
        assert_eq!(c[7..], [None, None]);
        // Axis outside the view: nothing copied.
        assert!(copies(9, Some(-4)).iter().all(Option::is_none));
        assert!(copies(9, Some(40)).iter().all(Option::is_none));
    }
}
