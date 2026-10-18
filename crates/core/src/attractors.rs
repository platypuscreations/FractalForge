// SPDX-License-Identifier: GPL-3.0-or-later

//! Strange attractors: lorenz, the 3D Lorenz family, rossler3d, pickover
//! and kamtorus3d, following docs/specs/attractors-3d.md (cited as §n).
//!
//! The Euler steps use the spec's exact operation orders (no fused
//! multiply-add): the systems are chaotic, so any other order leaves
//! XFractint's orbit after a few thousand iterates.

use crate::ext::cos_sin_f64;
use crate::plot::{Plotter, line};
use crate::{Complex, IterBuffer, Params, Window};

/// Colour of iterate j: 2 for the first 999, then a new colour every
/// 1001, wrapping from 255 to 1 (§2.2).
fn colour(j: i64) -> u32 {
    let c = if j <= 999 { 2 } else { 3 + (j - 1000) / 1001 };
    ((c - 1).rem_euclid(255) + 1) as u32
}

/// Beyond these pixel distances a point stops the drawing (§2.5; the exact
/// limits are unknown).
const FAR_COL: f64 = 6000.0;
const FAR_ROW: f64 = 4800.0;

#[derive(Clone, Copy, PartialEq)]
enum System {
    /// lorenz (plotted flat) and lorenz3d.
    Lorenz,
    Rossler,
    Pickover,
    /// The one-lobe system (n = 1) and its n-fold covers (§7).
    OneLobe(u32),
}

impl System {
    /// One Euler step or map iteration.
    fn step(self, [x, y, z]: [f64; 3], p: &[f64; 4]) -> [f64; 3] {
        let [dt, a, b, c] = *p;
        match self {
            System::Lorenz => {
                let xdt = dt * x;
                let ydt = dt * y;
                [
                    x + ((dt * a) * y - (dt * a) * x),
                    y + (((dt * b) * x - ydt) - xdt * z),
                    z + (ydt * x - (dt * c) * z),
                ]
            }
            System::Rossler => {
                let xdt = dt * x;
                let ydt = dt * y;
                [
                    x + (-ydt - (dt * z)),
                    y + (xdt + (dt * a) * y),
                    z + (((dt * b) + xdt * z) - (dt * c) * z),
                ]
            }
            System::Pickover => {
                let (pa, pb, pc, pd) = (p[0], p[1], p[2], p[3]);
                // Correctly rounded; XFractint's glibc sin/cos differ in the
                // last bit for some arguments, so long orbits drift (§6).
                let (_, s_ay) = cos_sin_f64(pa * y);
                let (c_bx, _) = cos_sin_f64(pb * x);
                let (_, s_cx) = cos_sin_f64(pc * x);
                let (c_dy, _) = cos_sin_f64(pd * y);
                let (_, s_x) = cos_sin_f64(x);
                [s_ay - z * c_bx, z * s_cx - c_dy, s_x]
            }
            System::OneLobe(1) => {
                let r = (x * x + y * y).sqrt();
                [
                    x + dt * (-(a + 1.0) * x + (a - b + z) * y + (1.0 - a) * r),
                    y + dt * ((b - a - z) * x - (a + 1.0) * y + (b + a - z) * r),
                    z + dt * (y / 2.0 - c * z),
                ]
            }
            System::OneLobe(n) => {
                // The n-fold cover W^n = w of the one-lobe system (§7.2).
                let big_w = Complex::new(x, y);
                let w = big_w.powi(n as i32);
                let (u, v) = (w.re, w.im);
                let r = (u * u + v * v).sqrt();
                let du = -(a + 1.0) * u + (a - b + z) * v + (1.0 - a) * r;
                let dv = (b - a - z) * u - (a + 1.0) * v + (b + a - z) * r;
                let dw = big_w * Complex::new(du, dv) / w.scale(n as f64);
                [x + dt * dw.re, y + dt * dw.im, z + dt * (v / 2.0 - c * z)]
            }
        }
    }
}

/// The rotation of §4.1 as rows (u, v, d).
fn rotation(angles: [i16; 3]) -> [[f64; 3]; 3] {
    let cs = |deg: i16| cos_sin_f64(-(deg as f64) * std::f64::consts::PI / 180.0);
    let (cx, sx) = cs(angles[0]);
    let (cy, sy) = cs(angles[1]);
    let (cz, sz) = cs(angles[2]);
    let rx = [[1.0, 0.0, 0.0], [0.0, cx, -sx], [0.0, sx, cx]];
    let ry = [[cy, 0.0, sy], [0.0, 1.0, 0.0], [-sy, 0.0, cy]];
    let rz = [[cz, -sz, 0.0], [sz, cz, 0.0], [0.0, 0.0, 1.0]];
    let mul = |m: [[f64; 3]; 3], n: [[f64; 3]; 3]| -> [[f64; 3]; 3] {
        std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| m[i][k] * n[k][j]).sum()))
    };
    mul(mul(rz, ry), rx)
}

fn apply(r: &[[f64; 3]; 3], p: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| r[i][0] * p[0] + r[i][1] * p[1] + r[i][2] * p[2])
}

/// A bounding box updated with "else if" (§4.2).
#[derive(Clone, Copy)]
struct Range {
    lo: f64,
    hi: f64,
}

impl Range {
    const EMPTY: Range = Range {
        lo: f64::INFINITY,
        hi: f64::NEG_INFINITY,
    };

    fn add(&mut self, v: f64) {
        if v < self.lo {
            self.lo = v;
        } else if v > self.hi {
            self.hi = v;
        }
    }
}

/// How plot pairs are made from orbit points.
enum View {
    /// lorenz: (x, z), nothing hidden.
    Flat,
    /// The 3D pipeline of §4: a hidden prefix of `hidden` iterates fixes
    /// the centre (and the depth for perspective).
    Rotated {
        r: [[f64; 3]; 3],
        hidden: u64,
        ranges: [Range; 3],
        centre: (f64, f64),
        perspective: f64,
        shift: (f64, f64),
    },
}

struct Attractor {
    system: System,
    p: [f64; 4],
    view: View,
    lines: bool,
    point: [f64; 3],
    /// Index of the last computed iterate.
    j: u64,
    /// Last iterate drawn.
    last: u64,
    /// Pixel of the previous drawn point, if it was inside the image.
    prev: Option<Option<(i64, i64)>>,
    window: Window,
    w: f64,
    h: f64,
    done: bool,
}

impl Attractor {
    /// The pixel of a plot pair, or `None` past the far limits (§2.1, §2.5).
    fn pixel(&self, u: f64, v: f64) -> Option<(i64, i64)> {
        let w = &self.window;
        let col = ((u - w.x_min) / (w.x_max - w.x_min) * (self.w - 1.0)).trunc();
        let row = ((w.y_max - v) / (w.y_max - w.y_min) * (self.h - 1.0)).trunc();
        (col.abs() <= FAR_COL && row.abs() <= FAR_ROW).then_some((col as i64, row as i64))
    }

    /// The plot pair of the current point, or `None` when drawing must
    /// stop (behind the viewer, §4.3).
    fn plot_pair(&self) -> Option<(f64, f64)> {
        match &self.view {
            View::Flat => Some((self.point[0], self.point[2])),
            View::Rotated {
                r,
                ranges,
                centre,
                perspective,
                ..
            } => {
                let [u, v, d] = apply(r, self.point);
                let (u, v) = (u - centre.0, v - centre.1);
                if *perspective == 0.0 {
                    return Some((u, v));
                }
                let depth = ranges[2];
                let e = perspective / 100.0 * (depth.hi - depth.lo);
                let den = e + (d - depth.hi);
                (den > 0.0).then(|| (e / den * u, e / den * v))
            }
        }
    }

    fn inside(&self, (c, r): (i64, i64)) -> bool {
        (0..self.w as i64).contains(&c) && (0..self.h as i64).contains(&r)
    }

    /// Draws the current point in `colour` (§2.4).
    fn draw(&mut self, buf: &mut IterBuffer, colour: u32) {
        let Some((u, v)) = self
            .plot_pair()
            .filter(|(u, v)| u.is_finite() && v.is_finite())
        else {
            self.done = true;
            return;
        };
        let Some(px) = self.pixel(u, v) else {
            self.done = true;
            return;
        };
        let here = self.inside(px).then_some(px);
        if let Some(px) = here {
            match self.prev {
                Some(Some(from)) if self.lines => line(buf, from, px, colour),
                _ => buf.set(px.0 as u32, px.1 as u32, colour),
            }
        }
        self.prev = Some(here);
    }

    /// Records the previous point without drawing it (the first point of
    /// a line drawing).
    fn remember(&mut self) {
        let px = self
            .plot_pair()
            .and_then(|(u, v)| self.pixel(u, v))
            .filter(|&px| self.inside(px));
        self.prev = Some(px);
    }
}

impl Plotter for Attractor {
    fn run(&mut self, buf: &mut IterBuffer, budget: u64) {
        for _ in 0..budget {
            if self.done {
                return;
            }
            let hidden = match &self.view {
                View::Flat => 0,
                View::Rotated { hidden, .. } => *hidden,
            };
            if self.j == 0 && hidden == 0 {
                // lorenz: P0 starts the first line (§3.2).
                self.remember();
            }
            if self.j >= self.last {
                self.done = true;
                return;
            }
            self.point = self.system.step(self.point, &self.p);
            self.j += 1;
            let j = self.j;
            match &mut self.view {
                View::Rotated {
                    r,
                    hidden,
                    ranges,
                    centre,
                    shift,
                    ..
                } if j <= *hidden => {
                    let q = apply(r, self.point);
                    for (range, v) in ranges.iter_mut().zip(q) {
                        range.add(v);
                    }
                    if j == *hidden {
                        *centre = (
                            (ranges[0].lo + ranges[0].hi) / 2.0 - shift.0,
                            (ranges[1].lo + ranges[1].hi) / 2.0 - shift.1,
                        );
                    }
                    continue;
                }
                _ => {}
            }
            if hidden > 0 && j == hidden + 1 {
                // §4.4: the first visible iterate starts the drawing.
                if self.lines {
                    self.remember();
                } else {
                    self.draw(buf, colour(hidden as i64));
                }
                continue;
            }
            let c = if hidden > 0 {
                colour(j as i64 - 1)
            } else {
                colour(j as i64)
            };
            self.draw(buf, c);
        }
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn progress(&self) -> f64 {
        if self.done {
            1.0
        } else {
            self.j as f64 / self.last as f64
        }
    }
}

fn attractor(
    system: System,
    hidden: Option<u64>,
    lines: bool,
    p: &Params,
    width: u32,
    height: u32,
) -> Result<Box<dyn Plotter>, String> {
    let n = 1024 * p.max_iter as u64;
    let w = p.window;
    let view = match hidden {
        None => View::Flat,
        Some(k) => View::Rotated {
            r: rotation(p.view3d.rotation),
            hidden: k,
            ranges: [Range::EMPTY; 3],
            centre: (0.0, 0.0),
            perspective: p.view3d.perspective as f64,
            shift: (
                p.view3d.xyshift[0] as f64 / 100.0 * (w.x_max - w.x_min),
                p.view3d.xyshift[1] as f64 / 100.0 * (w.y_max - w.y_min),
            ),
        },
    };
    Ok(Box::new(Attractor {
        system,
        p: [p.params[0], p.params[1], p.params[2], p.params[3]],
        view,
        lines,
        point: [1.0, 1.0, 1.0],
        j: 0,
        // 3D images use N + 1 iterates (§4.4).
        last: if hidden.is_some() { n + 1 } else { n },
        prev: None,
        window: w,
        w: width as f64,
        h: height as f64,
        done: false,
    }))
}

pub(crate) fn lorenz(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    attractor(System::Lorenz, None, true, p, w, h)
}

pub(crate) fn lorenz3d(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    attractor(System::Lorenz, Some(100), true, p, w, h)
}

pub(crate) fn rossler3d(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    attractor(System::Rossler, Some(500), true, p, w, h)
}

pub(crate) fn pickover(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    attractor(System::Pickover, Some(100), false, p, w, h)
}

pub(crate) fn lorenz3d1(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    attractor(System::OneLobe(1), Some(750), true, p, w, h)
}

pub(crate) fn lorenz3d3(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    attractor(System::OneLobe(3), Some(750), true, p, w, h)
}

pub(crate) fn lorenz3d4(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    attractor(System::OneLobe(4), Some(750), true, p, w, h)
}

// ---- kamtorus3d (§8, partial) ---------------------------------------------

/// Most orbits drawn (the loop otherwise ends at the far limit, §8).
const KAM_ORBITS: u64 = 100_000;

struct Kamtorus3d {
    cos_a: f64,
    sin_a: f64,
    step: f64,
    len: u64,
    r: [[f64; 3]; 3],
    /// Orbit start value o, orbit count, point within the orbit (1-based).
    o: f64,
    orbits: u64,
    k: u64,
    x: f64,
    y: f64,
    /// Iterates so far.
    i: u64,
    window: Window,
    w: f64,
    h: f64,
    done: bool,
}

impl Plotter for Kamtorus3d {
    fn run(&mut self, buf: &mut IterBuffer, budget: u64) {
        for _ in 0..budget {
            if self.done {
                return;
            }
            if self.k == 1 {
                self.x = self.o / 3.0;
                self.y = self.x;
            }
            let q = self.x * self.x - self.y;
            (self.x, self.y) = (
                self.x * self.cos_a + q * self.sin_a,
                self.x * self.sin_a - q * self.cos_a,
            );
            self.i += 1;
            let [u, v, _] = apply(&self.r, [self.x, self.y, self.o]);
            let w = &self.window;
            let col = ((u - w.x_min) / (w.x_max - w.x_min) * (self.w - 1.0)).trunc();
            let row = ((w.y_max - v) / (w.y_max - w.y_min) * (self.h - 1.0)).trunc();
            if !(col.abs() <= FAR_COL && row.abs() <= FAR_ROW) {
                self.done = true;
                return;
            }
            if (0.0..self.w).contains(&col) && (0.0..self.h).contains(&row) {
                let c = colour(self.i as i64 - self.len as i64);
                buf.set(col as u32, row as u32, c);
            }
            self.k += 1;
            if self.k > self.len {
                self.k = 1;
                self.o += self.step;
                self.orbits += 1;
                if self.orbits >= KAM_ORBITS {
                    self.done = true;
                }
            }
        }
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn progress(&self) -> f64 {
        if self.done {
            1.0
        } else {
            // The end isn't known in advance (§8).
            (self.orbits as f64 / 50.0).min(0.99)
        }
    }
}

pub(crate) fn kamtorus3d(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    if p.params[1].is_nan() || p.params[1] <= 0.0 {
        return Err("kamtorus3d needs a positive step (parameter 2)".into());
    }
    let (cos_a, sin_a) = cos_sin_f64(p.params[0]);
    Ok(Box::new(Kamtorus3d {
        cos_a,
        sin_a,
        step: p.params[1],
        len: p.params[3].trunc().max(0.0) as u64 + 1,
        r: rotation(p.view3d.rotation),
        o: 0.0,
        orbits: 0,
        k: 1,
        x: 0.0,
        y: 0.0,
        i: 0,
        window: p.window,
        w: w as f64,
        h: h as f64,
        done: false,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours() {
        assert_eq!(colour(1), 2);
        assert_eq!(colour(999), 2);
        assert_eq!(colour(1000), 3);
        assert_eq!(colour(2000), 3);
        assert_eq!(colour(2001), 4);
        assert_eq!(colour(-5), 2);
    }
}
