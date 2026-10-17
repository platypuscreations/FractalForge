// SPDX-License-Identifier: GPL-3.0-or-later

//! The render plan: which rows are visited in which order, derived
//! deterministically from the parameters, image size and render state.

use fractalforge_core::{Complex, IterBuffer, Params, Symmetry, UNKNOWN};

use crate::checkpoint::{PassMode, RenderState};

/// How computed pixels are copied to the other side of a symmetry axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mirror {
    None,
    /// Row `y` is copied to row `k - y`.
    Rows {
        k: i64,
    },
    /// Pixel `(x, y)` is copied to `(kx - x, ky - y)`.
    Origin {
        kx: i64,
        ky: i64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stage {
    /// Visit the grid of pixels whose coordinates (relative to the region)
    /// are multiples of `step`.
    Level { step: u32, first: bool, guess: bool },
    /// Compute pixels outside the region that no mirror filled.
    Residue,
}

#[derive(Clone, Debug)]
pub(crate) struct Plan {
    pub width: u32,
    pub height: u32,
    /// Rows `r0..=r1` are computed; the rest are filled by mirroring.
    pub r0: u32,
    pub r1: u32,
    pub mirror: Mirror,
    pub stages: Vec<Stage>,
    /// Rows outside the region, for the residue stage.
    residue_rows: Vec<u32>,
}

impl Plan {
    pub fn new(params: &Params, width: u32, height: u32, state: &RenderState) -> Self {
        let (mirror, r0, r1) = symmetry_region(params, width, height);

        let mut stages = Vec::new();
        let guess = state.pass_mode == PassMode::Guess;
        let mut step = state.coarsest_step;
        let mut first = true;
        while step >= 1 {
            stages.push(Stage::Level { step, first, guess });
            first = false;
            step /= 2;
        }

        let residue_rows: Vec<u32> = match mirror {
            Mirror::Origin { .. } => (0..height).filter(|y| !(r0..=r1).contains(y)).collect(),
            _ => Vec::new(),
        };
        if !residue_rows.is_empty() {
            stages.push(Stage::Residue);
        }

        Self {
            width,
            height,
            r0,
            r1,
            mirror,
            stages,
            residue_rows,
        }
    }

    fn region_height(&self) -> u32 {
        self.r1 - self.r0 + 1
    }

    /// Number of rows in a stage.
    pub fn stage_rows(&self, stage: usize) -> u32 {
        match self.stages[stage] {
            Stage::Level { step, .. } => self.region_height().div_ceil(step),
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
    ) -> (u32, Vec<(u32, u32)>) {
        let px = Pixels::new(params, self.width, self.height);
        match self.stages[stage] {
            Stage::Level { step, first, guess } => {
                let vy = index * step;
                let y = self.r0 + vy;
                let (start, stride) = if first || vy % (2 * step) != 0 {
                    (0, step)
                } else {
                    (step, 2 * step)
                };
                let out = (start..self.width)
                    .step_by(stride as usize)
                    .map(|x| {
                        let guessed = if guess && !first {
                            self.guess(buf, x, vy, step)
                        } else {
                            None
                        };
                        (x, guessed.unwrap_or_else(|| px.iterate(x, y)))
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
                    .map(|(x, _)| (x as u32, px.iterate(x as u32, y)))
                    .collect();
                (y, out)
            }
        }
    }

    /// The value to fill without computing, if the 4×4 neighbourhood of
    /// coarser-grid points around this pixel's cell is uniform.
    fn guess(&self, buf: &IterBuffer, x: u32, vy: u32, step: u32) -> Option<u32> {
        let s2 = 2 * step as i64;
        let last_x = (self.width as i64 - 1) / s2 * s2;
        let last_y = (self.region_height() as i64 - 1) / s2 * s2;
        let cx0 = x as i64 / s2 * s2;
        let cy0 = vy as i64 / s2 * s2;
        let mut value = None;
        for j in -1..=2 {
            let cy = (cy0 + j * s2).clamp(0, last_y) as u32 + self.r0;
            for i in -1..=2 {
                let cx = (cx0 + i * s2).clamp(0, last_x) as u32;
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

    /// Writes one row's results and their mirror images.
    pub fn apply(&self, buf: &mut IterBuffer, y: u32, values: &[(u32, u32)]) {
        for &(x, v) in values {
            buf.set(x, y, v);
            if let Some((mx, my)) = self.mirror_of(x, y) {
                buf.set(mx, my, v);
            }
        }
    }

    fn mirror_of(&self, x: u32, y: u32) -> Option<(u32, u32)> {
        let (mx, my) = match self.mirror {
            Mirror::None => return None,
            Mirror::Rows { k } => (x as i64, k - y as i64),
            Mirror::Origin { kx, ky } => (kx - x as i64, ky - y as i64),
        };
        let in_image =
            (0..self.width as i64).contains(&mx) && (0..self.height as i64).contains(&my);
        let in_region = (self.r0 as i64..=self.r1 as i64).contains(&my);
        (in_image && !in_region).then_some((mx as u32, my as u32))
    }
}

/// Picks the mirror and the contiguous row range to compute.
fn symmetry_region(params: &Params, width: u32, height: u32) -> (Mirror, u32, u32) {
    let all = (Mirror::None, 0, height - 1);
    if height < 3 {
        return all;
    }
    let w = &params.window;
    // Twice the (fractional) row index of the line im = 0.
    let ky = (2.0 * w.y_max / ((w.y_max - w.y_min) / (height - 1) as f64)).round();
    let rows_ok = ky > 0.0 && ky < 2.0 * (height - 1) as f64;

    let mirror = match (params.fractal.symmetry)(params) {
        Symmetry::None => return all,
        Symmetry::XAxis if rows_ok => Mirror::Rows { k: ky as i64 },
        Symmetry::Origin if rows_ok && width >= 3 => {
            let kx = (2.0 * -w.x_min / ((w.x_max - w.x_min) / (width - 1) as f64)).round();
            if kx > 0.0 && kx < 2.0 * (width - 1) as f64 {
                Mirror::Origin {
                    kx: kx as i64,
                    ky: ky as i64,
                }
            } else {
                return all;
            }
        }
        _ => return all,
    };

    // Compute the larger side so the computed rows stay contiguous.
    let k = ky as u32;
    if k < height {
        (mirror, k.div_ceil(2), height - 1)
    } else {
        (mirror, 0, k / 2)
    }
}

/// Maps pixel coordinates to the complex plane.
struct Pixels<'a> {
    params: &'a Params,
    dx: f64,
    dy: f64,
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
        }
    }

    fn iterate(&self, x: u32, y: u32) -> u32 {
        let w = &self.params.window;
        let c = Complex::new(w.x_min + x as f64 * self.dx, w.y_max - y as f64 * self.dy);
        (self.params.fractal.iterate)(c, self.params)
    }
}
