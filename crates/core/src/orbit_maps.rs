// SPDX-License-Identifier: GPL-3.0-or-later

//! The 2D orbit-plotting map types (henon, hopalong, icons, kamtorus,
//! popcorn, …), following docs/specs/orbit-maps.md (cited as §n).
//!
//! Fixed sin, cos, log and atan come from the `libm` crate so that these
//! chaotic orbits are the same on every platform. XFractint's own images
//! follow glibc's results, so long orbits of martin, chip, quadruptwo and
//! threeply drift from them after a few hundred to a few thousand points
//! (§2.4); their attractors are the same.

use std::collections::VecDeque;

use crate::funcs::{FN_TABLE, call_index};
use crate::plot::Plotter;
use crate::{Complex, Inside, IterBuffer, Params};

/// Point → pixel with truncation toward zero (§2.3).
struct Mapper {
    x_min: f64,
    y_max: f64,
    sx: f64,
    sy: f64,
    w: f64,
    h: f64,
}

impl Mapper {
    fn new(p: &Params, width: u32, height: u32) -> Self {
        let w = &p.window;
        Self {
            x_min: w.x_min,
            y_max: w.y_max,
            sx: (width - 1) as f64 / (w.x_max - w.x_min),
            sy: (height - 1) as f64 / (w.y_max - w.y_min),
            w: width as f64,
            h: height as f64,
        }
    }

    fn pixel(&self, x: f64, y: f64) -> Option<(u32, u32)> {
        let fc = (x - self.x_min) * self.sx;
        let fr = (self.y_max - y) * self.sy;
        // Also false for NaN.
        (fc > -1.0 && fc < self.w && fr > -1.0 && fr < self.h).then_some((fc as u32, fr as u32))
    }

    fn set(&self, buf: &mut IterBuffer, x: f64, y: f64, v: u32) {
        if let Some((c, r)) = self.pixel(x, y) {
            buf.set(c, r, v);
        }
    }

    /// Adds a hit, up to 255 (§4.1).
    fn hit(&self, buf: &mut IterBuffer, x: f64, y: f64) {
        if let Some((c, r)) = self.pixel(x, y) {
            buf.set(c, r, (buf.get(c, r) + 1).min(255));
        }
    }
}

/// The colour of the first points, from `inside` (§3.3).
fn start_colour(p: &Params) -> u32 {
    match p.inside {
        Inside::Color(c) if c > 0 => c as u32,
        _ => 2,
    }
}

/// The time colour of point j (§3.3).
fn time_colour(c0: u32, j: u64) -> u32 {
    ((c0 as u64 - 1 + (j + 1) / 1001) % 255) as u32 + 1
}

fn sign(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// A function slot applied to a real value. Overflowing results become
/// 1.0 for the popcorn family (§2.4); otherwise they stay infinite.
fn slot(index: u8, v: f64, overflow_to_one: bool) -> f64 {
    let r = if FN_TABLE.get(index as usize) == Some(&"tan") {
        // §2.4: the complex tangent formula with zero imaginary part.
        (2.0 * v).sin() / ((2.0 * v).cos() + 1.0)
    } else {
        match call_index(index, Complex::new(v, 0.0)) {
            Ok(z) => z.re,
            Err(_) => f64::NAN,
        }
    };
    if overflow_to_one && !r.is_finite() {
        1.0
    } else {
        r
    }
}

// ---- single orbits (§3, §4) --------------------------------------------

#[derive(Clone, Copy)]
enum Map {
    Henon,
    Hopalong,
    Martin,
    Gingerbread,
    Chip,
    Quadruptwo,
    Threeply,
    Icons,
    Latoocarfian,
}

/// One orbit of 1024·maxiter iterates after the start point.
struct SingleOrbit {
    map: Map,
    p: [f64; 6],
    functions: [u8; 4],
    x: f64,
    y: f64,
    /// Index of the next point to plot (the start is 0).
    j: u64,
    last: u64,
    /// `None` for hit counts.
    c0: Option<u32>,
    mapper: Mapper,
}

impl SingleOrbit {
    fn step(&self, x: f64, y: f64) -> (f64, f64) {
        let [a, b, c, ..] = self.p;
        match self.map {
            Map::Henon => ((1.0 + y) - (a * x) * x, b * x),
            Map::Hopalong => (y - sign(x) * (b * x - c).abs().sqrt(), a - x),
            Map::Martin => (y - libm::sin(x), a - x),
            Map::Gingerbread => ((1.0 - y) + x.abs(), x),
            Map::Chip => {
                let l1 = libm::log((b * x - c).abs());
                let l2 = libm::log((c * x - b).abs());
                (
                    y - (sign(x) * libm::cos(l1 * l1)) * libm::atan(l2 * l2),
                    a - x,
                )
            }
            Map::Quadruptwo => {
                let l1 = libm::log((b * x - c).abs());
                let l2 = libm::log((c * x - b).abs());
                (y - (sign(x) * libm::sin(l1)) * libm::atan(l2 * l2), a - x)
            }
            Map::Threeply => {
                let inner = (libm::sin(x) * libm::cos(b) + c) - x * libm::sin((a + b) + c);
                (y - sign(x) * inner.abs(), a - x)
            }
            Map::Icons => {
                let [lambda, alpha, beta, gamma, omega, n] = self.p;
                let zz = x * x + y * y;
                let (mut u, mut v) = (x, y);
                for _ in 0..(n.trunc() as i64 - 2).max(0) {
                    (u, v) = (u * x - v * y, u * y + v * x);
                }
                let w = x * u - y * v;
                let p = (lambda + alpha * zz) + beta * w;
                (
                    (p * x + gamma * u) - omega * y,
                    (p * y - gamma * v) + omega * x,
                )
            }
            Map::Latoocarfian => {
                let [a, b, c, d, ..] = self.p;
                let f = |i: usize, v: f64| slot(self.functions[i], v, false);
                (f(0, y * b) + c * f(1, x * b), f(2, x * a) + d * f(3, y * a))
            }
        }
    }

    fn plot(&self, buf: &mut IterBuffer) {
        match self.c0 {
            Some(c0) => self
                .mapper
                .set(buf, self.x, self.y, time_colour(c0, self.j)),
            None => self.mapper.hit(buf, self.x, self.y),
        }
    }
}

impl Plotter for SingleOrbit {
    fn run(&mut self, buf: &mut IterBuffer, budget: u64) {
        for _ in 0..budget {
            if self.j > self.last {
                return;
            }
            if self.j > 0 {
                (self.x, self.y) = self.step(self.x, self.y);
                // An overflowed orbit draws nothing more (§4.3).
                if !(self.x.is_finite() && self.y.is_finite()) {
                    self.j = self.last + 1;
                    return;
                }
            }
            self.plot(buf);
            self.j += 1;
        }
    }

    fn is_done(&self) -> bool {
        self.j > self.last
    }

    fn progress(&self) -> f64 {
        (self.j as f64 / (self.last + 1) as f64).min(1.0)
    }
}

fn single(map: Map, p: &Params, width: u32, height: u32) -> Box<dyn Plotter> {
    let mut q = [0.0; 6];
    q.copy_from_slice(&p.params[..6]);
    let (x, y) = match map {
        Map::Henon | Map::Latoocarfian => (1.0, 1.0),
        Map::Gingerbread => (q[0], q[1]),
        Map::Icons => (0.01, 0.003),
        _ => (0.0, 0.0),
    };
    let hits = matches!(map, Map::Icons | Map::Latoocarfian);
    Box::new(SingleOrbit {
        map,
        p: q,
        functions: p.functions,
        x,
        y,
        j: 0,
        last: 1024 * p.max_iter as u64,
        c0: (!hits).then(|| start_colour(p)),
        mapper: Mapper::new(p, width, height),
    })
}

macro_rules! single_plot {
    ($name:ident, $map:expr) => {
        pub(crate) fn $name(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
            Ok(single($map, p, w, h))
        }
    };
}

single_plot!(henon, Map::Henon);
single_plot!(hopalong, Map::Hopalong);
single_plot!(martin, Map::Martin);
single_plot!(gingerbreadman, Map::Gingerbread);
single_plot!(chip, Map::Chip);
single_plot!(quadruptwo, Map::Quadruptwo);
single_plot!(threeply, Map::Threeply);
single_plot!(icons, Map::Icons);
single_plot!(latoocarfian, Map::Latoocarfian);

// ---- kamtorus (§5) -----------------------------------------------------

struct Kamtorus {
    cos_a: f64,
    sin_a: f64,
    step: f64,
    stop: f64,
    /// Points per orbit.
    len: u64,
    t: f64,
    /// Orbit number and the next point within it (1-based).
    o: u64,
    k: u64,
    x: f64,
    y: f64,
    c0: u32,
    mapper: Mapper,
}

impl Plotter for Kamtorus {
    fn run(&mut self, buf: &mut IterBuffer, budget: u64) {
        for _ in 0..budget {
            if self.is_done() {
                return;
            }
            if self.k == 1 {
                self.x = self.t / 3.0;
                self.y = self.x;
            }
            let q = self.x * self.x - self.y;
            (self.x, self.y) = (
                self.x * self.cos_a + q * self.sin_a,
                self.x * self.sin_a - q * self.cos_a,
            );
            let colour = if self.o == 0 {
                self.c0
            } else {
                time_colour(self.c0, (self.o - 1) * self.len + self.k)
            };
            self.mapper.set(buf, self.x, self.y, colour);
            self.k += 1;
            if self.k > self.len {
                self.k = 1;
                self.o += 1;
                // Repeated addition, not o·step (§5).
                self.t += self.step;
            }
        }
    }

    fn is_done(&self) -> bool {
        // Runs while t ≤ stop.
        !matches!(
            self.t.partial_cmp(&self.stop),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        )
    }

    fn progress(&self) -> f64 {
        if self.stop > 0.0 {
            (self.t / self.stop).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

pub(crate) fn kamtorus(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    let a = p.params[0];
    if p.params[1].is_nan() || p.params[1] <= 0.0 {
        return Err("kamtorus needs a positive step (parameter 2)".into());
    }
    Ok(Box::new(Kamtorus {
        cos_a: libm::cos(a),
        sin_a: libm::sin(a),
        step: p.params[1],
        stop: p.params[2],
        len: (p.params[3].trunc().max(0.0)) as u64 + 1,
        t: 0.0,
        o: 0,
        k: 1,
        x: 0.0,
        y: 0.0,
        c0: start_colour(p),
        mapper: Mapper::new(p, w, h),
    }))
}

// ---- popcorn (§6) and popcornjul (§7) ----------------------------------

/// One popcorn step from (x, y), both coordinates from the old values.
fn popcorn_step(x: f64, y: f64, p: &Params) -> (f64, f64) {
    let (h, t) = (p.params[0], p.params[2]);
    let f = |i: usize, v: f64| slot(p.functions[i], v, true);
    (x - h * f(0, y + f(1, t * y)), y - h * f(2, x + f(3, t * x)))
}

struct Popcorn {
    p: Params,
    width: u32,
    height: u32,
    /// Next start pixel, in row order.
    next: u64,
    mapper: Mapper,
}

impl Plotter for Popcorn {
    fn run(&mut self, buf: &mut IterBuffer, budget: u64) {
        let w = &self.p.window;
        let dx = (w.x_max - w.x_min) / (self.width - 1) as f64;
        let dy = (w.y_max - w.y_min) / (self.height - 1) as f64;
        let mut spent = 0;
        while spent < budget && !self.is_done() {
            let (r, c) = (self.next / self.width as u64, self.next % self.width as u64);
            let colour = (r as u32 + 1) % 256;
            let (mut x, mut y) = (w.x_min + c as f64 * dx, w.y_max - r as f64 * dy);
            for _ in 1..self.p.max_iter {
                (x, y) = popcorn_step(x, y, &self.p);
                self.mapper.set(buf, x, y, colour);
                if (x * x + y * y).is_nan() || x * x + y * y > 4.0 {
                    break;
                }
            }
            spent += self.p.max_iter as u64;
            self.next += 1;
        }
    }

    fn is_done(&self) -> bool {
        self.next >= self.width as u64 * self.height as u64
    }

    fn progress(&self) -> f64 {
        self.next as f64 / (self.width as f64 * self.height as f64)
    }
}

pub(crate) fn popcorn(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    Ok(Box::new(Popcorn {
        p: p.clone(),
        width: w,
        height: h,
        next: 0,
        mapper: Mapper::new(p, w, h),
    }))
}

/// popcornjul: escape time with popcorn's step (§7).
pub(crate) fn popcornjul(w: Complex, p: &Params) -> u32 {
    let (mut x, mut y) = (w.re, w.im);
    for e in 1..p.max_iter {
        (x, y) = popcorn_step(x, y, p);
        if (x * x + y * y).is_nan() || x * x + y * y > 4.0 {
            return e;
        }
    }
    crate::INSIDE
}

// ---- grids: dynamic (§8) and mandelcloud (§9) --------------------------

struct Grid {
    p: Params,
    n: u64,
    /// Next grid point, row-major over n + 1 rows of n columns.
    idx: u64,
    cloud: bool,
    offset: u64,
    mapper: Mapper,
}

impl Plotter for Grid {
    fn run(&mut self, buf: &mut IterBuffer, budget: u64) {
        let w = self.p.window;
        let mut spent = 0;
        while spent < budget && !self.is_done() {
            let (i, j) = (self.idx / self.n, self.idx % self.n);
            let x0 = w.x_min + (j as f64 + 0.5) * (w.x_max - w.x_min) / self.n as f64;
            let y0 = w.y_max - (i as f64 + 0.5) * (w.y_max - w.y_min) / self.n as f64;
            let colour = ((self.idx + self.offset) % 255) as u32 + 1;
            let (mut x, mut y) = (x0, y0);
            if self.cloud {
                for _ in 1..self.p.max_iter {
                    (x, y) = (x * x - y * y + x0, 2.0 * x * y + y0);
                    if x * x + y * y > 4.0 {
                        break;
                    }
                    self.mapper.set(buf, x, y, colour);
                }
            } else {
                let (dt, a, b) = (self.p.params[1], self.p.params[2], self.p.params[3]);
                let f = |v: f64| slot(self.p.functions[0], v, false);
                self.mapper.set(buf, x, y, colour);
                for _ in 1..self.p.max_iter {
                    (x, y) = (
                        x - dt * libm::sin(y + a * f(b * y)),
                        y + dt * libm::sin(x + a * f(b * x)),
                    );
                    self.mapper.set(buf, x, y, colour);
                }
            }
            spent += self.p.max_iter as u64;
            self.idx += 1;
        }
    }

    fn is_done(&self) -> bool {
        self.idx >= (self.n + 1) * self.n
    }

    fn progress(&self) -> f64 {
        self.idx as f64 / ((self.n + 1) * self.n).max(1) as f64
    }
}

fn grid(p: &Params, w: u32, h: u32, cloud: bool) -> Result<Box<dyn Plotter>, String> {
    let n = p.params[0].trunc();
    if !(1.0..=100_000.0).contains(&n) {
        return Err("the grid size (parameter 1) must be 1 to 100000".into());
    }
    let offset = match p.inside {
        Inside::Color(c) if !cloud => c as u64,
        _ => 0,
    };
    Ok(Box::new(Grid {
        p: p.clone(),
        n: n as u64,
        idx: 0,
        cloud,
        offset,
        mapper: Mapper::new(p, w, h),
    }))
}

pub(crate) fn dynamic(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    grid(p, w, h, false)
}

pub(crate) fn mandelcloud(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    grid(p, w, h, true)
}

// ---- julia_inverse (§10) -----------------------------------------------

/// Most preimages visited (the model of §10).
const INVERSE_STEPS: u64 = 2_000_000;

struct JuliaInverse {
    c: Complex,
    cap: u32,
    queue: VecDeque<Complex>,
    steps: u64,
    mapper: Mapper,
}

impl Plotter for JuliaInverse {
    fn run(&mut self, buf: &mut IterBuffer, budget: u64) {
        for _ in 0..budget {
            if self.is_done() {
                return;
            }
            let z = self.queue.pop_front().unwrap();
            self.steps += 1;
            if let Some((col, row)) = self.mapper.pixel(z.re, z.im) {
                let hits = buf.get(col, row);
                if hits >= self.cap {
                    continue;
                }
                buf.set(col, row, hits + 1);
            }
            let w = crate::funcs::sqrt(z - self.c);
            self.queue.push_back(Complex::ZERO - w);
            self.queue.push_back(w);
        }
    }

    fn is_done(&self) -> bool {
        self.queue.is_empty() || self.steps >= INVERSE_STEPS
    }

    fn progress(&self) -> f64 {
        if self.is_done() {
            1.0
        } else {
            self.steps as f64 / INVERSE_STEPS as f64
        }
    }
}

pub(crate) fn julia_inverse(p: &Params, w: u32, h: u32) -> Result<Box<dyn Plotter>, String> {
    let c = Complex::new(p.params[0], p.params[1]);
    let fixed = Complex::new(0.5, 0.0) - crate::funcs::sqrt(Complex::new(0.25, 0.0) - c);
    Ok(Box::new(JuliaInverse {
        c,
        cap: (p.params[2].trunc() as i64).clamp(1, 255) as u32,
        queue: VecDeque::from([fixed]),
        steps: 0,
        mapper: Mapper::new(p, w, h),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_colours() {
        assert_eq!(time_colour(2, 0), 2);
        assert_eq!(time_colour(2, 999), 2);
        assert_eq!(time_colour(2, 1000), 3);
        assert_eq!(time_colour(255, 1000), 1);
    }

    #[test]
    fn truncation_toward_zero() {
        let p = Params::for_type("henon").unwrap();
        let m = Mapper::new(&p, 400, 300);
        let dx = (p.window.x_max - p.window.x_min) / 399.0;
        assert_eq!(
            m.pixel(p.window.x_min - 0.5 * dx, 0.0).map(|q| q.0),
            Some(0)
        );
        assert_eq!(m.pixel(p.window.x_min - 1.5 * dx, 0.0), None);
        assert_eq!(m.pixel(f64::NAN, 0.0), None);
    }
}
