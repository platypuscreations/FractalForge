// SPDX-License-Identifier: GPL-3.0-or-later

//! The data-driven registry of fractal types.
//!
//! Iteration rules, defaults and symmetry follow docs/specs/types.md (cited
//! as §n) and, for mandel and julia, docs/specs/gif-fractint-blocks.md §9.
//!
//! Each type's iterate function returns the pixel's **colour value**: the
//! number Fractint-format images colour by (an escape count for most
//! types), or [`INSIDE`]. [`crate::color_index`] maps it to a palette index.

use std::fmt;

use crate::iter::INSIDE;
use crate::{Complex, Params, Window};

/// Mirror copying a renderer may (and, for Fractint-identical images, must)
/// use: see §1.6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symmetry {
    None,
    /// Rows below the real axis are copies of rows above it.
    XAxis,
    /// Columns right of the imaginary axis are copies of columns left of it.
    YAxis,
    /// Rows and columns are copied independently about the real and
    /// imaginary axes.
    XY,
    /// Pixels below the real axis are copies of the pixel reflected through
    /// the origin.
    Origin,
}

/// Returns the colour value of one point (see the module docs).
pub type IterateFn = fn(pixel: Complex, params: &Params) -> u32;

pub struct FractalType {
    /// Short identifier used in files and on the command line.
    pub name: &'static str,
    pub description: &'static str,
    pub default_window: Window,
    pub default_params: [f64; 4],
    /// Escape when `|z|²` exceeds this unless the user sets a bailout.
    pub default_bailout: f64,
    /// Labels for the meaningful entries of `Params::params`.
    pub param_names: &'static [&'static str],
    pub iterate: IterateFn,
    /// The symmetry these particular parameters have.
    pub symmetry: fn(&Params) -> Symmetry,
    /// How a colour value changes when copied to its mirror pixel, for
    /// types whose colours aren't symmetric (newtbasin's roots).
    pub mirror_value: Option<fn(u32, &Params) -> u32>,
    /// Whether the bailout setting is used.
    pub uses_bailout: bool,
    /// Whether an interrupted render can be checkpointed and resumed.
    pub resumable: bool,
    /// Type number in Fractint-format GIF blocks, if the type has one.
    pub fractint_number: Option<i16>,
    /// Supports the inside/outside/decomp/biomorph options (otherwise only
    /// the default colouring).
    pub orbit_coloring: bool,
    /// Defined by [`Params::program`] (formulas).
    pub needs_program: bool,
}

impl fmt::Debug for FractalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FractalType").field(&self.name).finish()
    }
}

impl FractalType {
    /// Whether the type's symmetry (when on) copies rows and columns
    /// independently, as opposed to through the origin.
    pub fn symmetry_kind_is_xy(&self) -> bool {
        self.name == BARNSLEYM1.name
    }
}

impl PartialEq for FractalType {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// The complex parameter `a = P1 + i·P2`.
fn a(p: &Params) -> Complex {
    Complex::new(p.params[0], p.params[1])
}

/// The standard escape loop (§1.2): advances the orbit with `step` and
/// returns the index e ≥ 1 of the first iterate for which `step` reports a
/// stop, or [`INSIDE`] if none has e < max_iter. The start isn't tested.
#[inline(always)]
fn escape<S>(mut state: S, max_iter: u32, mut step: impl FnMut(&mut S) -> bool) -> u32 {
    for e in 1..max_iter {
        if step(&mut state) {
            return e;
        }
    }
    INSIDE
}

// ---- symmetry rules ----------------------------------------------------

fn sym_none(_: &Params) -> Symmetry {
    Symmetry::None
}

fn sym_x(_: &Params) -> Symmetry {
    Symmetry::XAxis
}

fn sym_origin(_: &Params) -> Symmetry {
    Symmetry::Origin
}

/// x-axis symmetry only without a start offset.
fn sym_x_if_a_zero(p: &Params) -> Symmetry {
    if p.params[0] == 0.0 && p.params[1] == 0.0 {
        Symmetry::XAxis
    } else {
        Symmetry::None
    }
}

fn sym_xy_if_a_zero(p: &Params) -> Symmetry {
    if p.params[0] == 0.0 && p.params[1] == 0.0 {
        Symmetry::XY
    } else {
        Symmetry::None
    }
}

fn sym_x_if_p2_zero(p: &Params) -> Symmetry {
    if p.params[1] == 0.0 {
        Symmetry::XAxis
    } else {
        Symmetry::None
    }
}

// ---- mandel and julia (gif-fractint-blocks §9) --------------------------

/// Counting from z0 with the start untested: n is the first iterate past
/// the bailout (1..=max_iter); these two types colour by n − 1, minimum 1.
fn quadratic(mut z: Complex, c: Complex, p: &Params) -> u32 {
    for n in 1..=p.max_iter {
        z = z.square() + c;
        if z.norm_sqr() > p.bailout {
            return (n - 1).max(1);
        }
    }
    INSIDE
}

fn mandel(w: Complex, p: &Params) -> u32 {
    quadratic(a(p), w, p)
}

fn julia(w: Complex, p: &Params) -> u32 {
    quadratic(w, a(p), p)
}

// ---- §3 ----------------------------------------------------------------

fn lambda_step(z: Complex, l: Complex) -> Complex {
    l * z * (Complex::new(1.0, 0.0) - z)
}

fn lambda(w: Complex, p: &Params) -> u32 {
    let l = a(p);
    escape(w, p.max_iter, |z| {
        *z = lambda_step(*z, l);
        z.norm_sqr() > p.bailout
    })
}

fn mandellambda(w: Complex, p: &Params) -> u32 {
    escape(Complex::new(0.5, 0.0) + a(p), p.max_iter, |z| {
        *z = lambda_step(*z, w);
        z.norm_sqr() > p.bailout
    })
}

fn mandel4(w: Complex, p: &Params) -> u32 {
    escape(w + a(p), p.max_iter, |z| {
        *z = z.square().square() + w;
        z.norm_sqr() > p.bailout
    })
}

fn julia4(w: Complex, p: &Params) -> u32 {
    let c = a(p);
    escape(w, p.max_iter, |z| {
        *z = z.square().square() + c;
        z.norm_sqr() > p.bailout
    })
}

/// z(k+1) = z(k)² + z(k−1) + c, state (z(k), z(k−1)).
fn manowar_loop(z0: Complex, prev: Complex, c: Complex, p: &Params) -> u32 {
    escape((z0, prev), p.max_iter, |(z, prev)| {
        let next = z.square() + *prev + c;
        *prev = *z;
        *z = next;
        z.norm_sqr() > p.bailout
    })
}

fn manowar(w: Complex, p: &Params) -> u32 {
    manowar_loop(w + a(p), w, w, p)
}

fn manowarj(w: Complex, p: &Params) -> u32 {
    manowar_loop(w, w, a(p), p)
}

fn barnsley_loop(z0: Complex, c: Complex, p: &Params) -> u32 {
    let one = Complex::new(1.0, 0.0);
    escape(z0, p.max_iter, |z| {
        *z = if z.re >= 0.0 {
            (*z - one) * c
        } else {
            (*z + one) * c
        };
        z.norm_sqr() > p.bailout
    })
}

fn barnsleym1(w: Complex, p: &Params) -> u32 {
    barnsley_loop(w + a(p), w, p)
}

fn barnsleyj1(w: Complex, p: &Params) -> u32 {
    barnsley_loop(w, a(p), p)
}

fn spider(w: Complex, p: &Params) -> u32 {
    escape((w + a(p), w), p.max_iter, |(z, c)| {
        *z = z.square() + *c;
        *c = c.scale(0.5) + *z;
        z.norm_sqr() > p.bailout
    })
}

/// Magnet stop threshold for convergence to the fixed point 1: 2⁻¹⁵.
const MAGNET_CONVERGED: f64 = 1.0 / 32768.0;

fn magnet_loop(z0: Complex, c: Complex, p: &Params) -> u32 {
    let one = Complex::new(1.0, 0.0);
    let two = Complex::new(2.0, 0.0);
    escape(z0, p.max_iter, |z| {
        let q = (z.square() + c - one) / (z.scale(2.0) + c - two);
        *z = q.square();
        z.norm_sqr() > p.bailout || (*z - one).norm_sqr() < MAGNET_CONVERGED
    })
}

fn magnet1m(w: Complex, p: &Params) -> u32 {
    magnet_loop(a(p), w, p)
}

fn magnet1j(w: Complex, p: &Params) -> u32 {
    magnet_loop(w, a(p), p)
}

/// Newton convergence constant C (§3.12: anything in (0.9424745, 0.9424869]).
const NEWTON_C: f64 = 0.942478;

fn newton_degree(p: &Params) -> i32 {
    (p.params[0].trunc() as i32).max(1)
}

/// Newton's method for z^d = 1 from z0 = w. Returns (k, z(k)) for the first
/// k ≥ 0 that passes the convergence test, or `None` when n = k + 1 would
/// reach max_iter.
fn newton_orbit(w: Complex, p: &Params) -> Option<(u32, Complex)> {
    let d = newton_degree(p);
    let limit = NEWTON_C / d as f64;
    let one = Complex::new(1.0, 0.0);
    let mut z = w;
    for k in 0..p.max_iter.saturating_sub(1) {
        let zd1 = z.powi(d - 1);
        let zd = zd1 * z;
        if (zd - one).norm_sqr() < limit {
            return Some((k, z));
        }
        z = (zd.scale((d - 1) as f64) + one) / zd1.scale(d as f64);
    }
    None
}

fn newton(w: Complex, p: &Params) -> u32 {
    newton_orbit(w, p).map_or(INSIDE, |(k, _)| k + 1)
}

fn newtbasin(w: Complex, p: &Params) -> u32 {
    let Some((k, z)) = newton_orbit(w, p) else {
        return INSIDE;
    };
    let d = newton_degree(p);
    // Nearest root exp(2πij/d): the one whose angle is closest to z's.
    let turns = z.im.atan2(z.re) / std::f64::consts::TAU;
    let j = ((turns * d as f64).round() as i64).rem_euclid(d as i64) as u32;
    let stripe = if p.params[1] != 0.0 && k % 2 == 0 {
        8
    } else {
        0
    };
    1 + j + stripe
}

/// Mirroring conjugates z, so root j becomes root (d − j) mod d.
fn newtbasin_mirror(v: u32, p: &Params) -> u32 {
    if v == INSIDE {
        return v;
    }
    let d = newton_degree(p) as u32;
    let stripe = if v > 8 { 8 } else { 0 };
    let j = v - 1 - stripe;
    1 + (d - j) % d + stripe
}

/// z(k+1) = z^D + p·z^(D−1) + q·z(k−1) (D ≠ 0) or z² + p + q·z(k−1) (D = 0).
fn phoenix_loop(z0: Complex, pc: f64, qc: f64, p: &Params) -> u32 {
    let degree = p.params[2].trunc() as i32;
    escape((z0, Complex::ZERO), p.max_iter, |(z, prev)| {
        let next = if degree == 0 {
            z.square() + Complex::new(pc, 0.0) + prev.scale(qc)
        } else {
            let zd1 = z.powi(degree - 1);
            zd1 * *z + zd1.scale(pc) + prev.scale(qc)
        };
        *prev = *z;
        *z = next;
        z.norm_sqr() > p.bailout
    })
}

fn phoenix(w: Complex, p: &Params) -> u32 {
    phoenix_loop(w, p.params[0], p.params[1], p)
}

fn mandphoenix(w: Complex, p: &Params) -> u32 {
    phoenix_loop(w + a(p), w.re, w.im, p)
}

// ---- registry ----------------------------------------------------------

const MANDEL_WINDOW: Window = Window::new(-2.5, 1.5, -1.5, 1.5);
const STD_WINDOW: Window = Window::new(-2.0, 2.0, -1.5, 1.5);
const START_OFFSET: &[&str] = &["Real offset of z(0)", "Imaginary offset of z(0)"];
const CONSTANT: &[&str] = &["Real part of c", "Imaginary part of c"];

/// A type with the common defaults (bailout 4, resumable, no colour
/// mirroring).
#[allow(clippy::too_many_arguments)]
const fn std_type(
    name: &'static str,
    number: i16,
    description: &'static str,
    default_window: Window,
    default_params: [f64; 4],
    param_names: &'static [&'static str],
    iterate: IterateFn,
    symmetry: fn(&Params) -> Symmetry,
) -> FractalType {
    FractalType {
        name,
        description,
        default_window,
        default_params,
        default_bailout: 4.0,
        param_names,
        iterate,
        symmetry,
        mirror_value: None,
        uses_bailout: true,
        resumable: true,
        fractint_number: Some(number),
        orbit_coloring: false,
        needs_program: false,
    }
}

pub static MANDEL: FractalType = FractalType {
    orbit_coloring: true,
    ..std_type(
        "mandel",
        4,
        "Mandelbrot set: z → z² + c, c = pixel",
        MANDEL_WINDOW,
        [0.0; 4],
        START_OFFSET,
        mandel,
        sym_x_if_a_zero,
    )
};
pub static JULIA: FractalType = FractalType {
    orbit_coloring: true,
    ..std_type(
        "julia",
        6,
        "Julia set: z → z² + c, z(0) = pixel",
        STD_WINDOW,
        [0.3, 0.6, 0.0, 0.0],
        CONSTANT,
        julia,
        sym_origin,
    )
};
pub static LAMBDA: FractalType = std_type(
    "lambda",
    75,
    "Lambda map: z → λz(1 − z), z(0) = pixel",
    Window::new(-1.5, 2.5, -1.5, 1.5),
    [0.85, 0.6, 0.0, 0.0],
    &["Real part of λ", "Imaginary part of λ"],
    lambda,
    sym_none,
);
pub static MANDELLAMBDA: FractalType = std_type(
    "mandellambda",
    82,
    "Lambda map: z → λz(1 − z), λ = pixel, z(0) = ½",
    Window::new(-3.0, 5.0, -3.0, 3.0),
    [0.0; 4],
    START_OFFSET,
    mandellambda,
    sym_x_if_a_zero,
);
pub static MANDEL4: FractalType = std_type(
    "mandel4",
    155,
    "Quartic Mandelbrot: z → z⁴ + c, c = pixel",
    STD_WINDOW,
    [0.0; 4],
    START_OFFSET,
    mandel4,
    sym_x_if_a_zero,
);
pub static JULIA4: FractalType = std_type(
    "julia4",
    156,
    "Quartic Julia: z → z⁴ + c, z(0) = pixel",
    STD_WINDOW,
    [0.6, 0.55, 0.0, 0.0],
    CONSTANT,
    julia4,
    sym_origin,
);
pub static MANOWAR: FractalType = std_type(
    "manowar",
    9,
    "Man-o'-war: z → z² + z(previous) + c, c = pixel",
    MANDEL_WINDOW,
    [0.0; 4],
    START_OFFSET,
    manowar,
    sym_x_if_a_zero,
);
pub static MANOWARJ: FractalType = std_type(
    "manowarj",
    108,
    "Man-o'-war Julia: z → z² + z(previous) + c, z(0) = pixel",
    MANDEL_WINDOW,
    [0.0; 4],
    CONSTANT,
    manowarj,
    sym_none,
);
pub static BARNSLEYM1: FractalType = std_type(
    "barnsleym1",
    76,
    "Barnsley M1: z → (z ∓ 1)·c by the sign of Re z, c = pixel",
    STD_WINDOW,
    [0.0; 4],
    START_OFFSET,
    barnsleym1,
    sym_xy_if_a_zero,
);
pub static BARNSLEYJ1: FractalType = std_type(
    "barnsleyj1",
    77,
    "Barnsley J1: z → (z ∓ 1)·c by the sign of Re z, z(0) = pixel",
    STD_WINDOW,
    [0.6, 1.1, 0.0, 0.0],
    CONSTANT,
    barnsleyj1,
    sym_origin,
);
pub static SPIDER: FractalType = std_type(
    "spider",
    93,
    "Spider: z → z² + c, then c → c/2 + z, c(0) = pixel",
    MANDEL_WINDOW,
    [0.0; 4],
    START_OFFSET,
    spider,
    sym_x_if_a_zero,
);
pub static MAGNET1M: FractalType = FractalType {
    default_bailout: 100.0,
    ..std_type(
        "magnet1m",
        96,
        "Magnet model I, parameter plane (c = pixel)",
        Window::new(-4.0, 4.0, -3.0, 3.0),
        [0.0; 4],
        &["Real part of z(0)", "Imaginary part of z(0)"],
        magnet1m,
        sym_x_if_a_zero,
    )
};
pub static MAGNET1J: FractalType = FractalType {
    default_bailout: 100.0,
    ..std_type(
        "magnet1j",
        97,
        "Magnet model I, z(0) = pixel",
        Window::new(-8.0, 8.0, -6.0, 6.0),
        [0.0; 4],
        CONSTANT,
        magnet1j,
        sym_x_if_p2_zero,
    )
};
pub static NEWTON: FractalType = FractalType {
    uses_bailout: false,
    ..std_type(
        "newton",
        5,
        "Newton's method for z^d = 1, coloured by step count",
        STD_WINDOW,
        [3.0, 0.0, 0.0, 0.0],
        &["Degree d"],
        newton,
        sym_x,
    )
};
pub static NEWTBASIN: FractalType = FractalType {
    uses_bailout: false,
    mirror_value: Some(newtbasin_mirror),
    ..std_type(
        "newtbasin",
        2,
        "Newton's method for z^d = 1, coloured by root",
        STD_WINDOW,
        [3.0, 0.0, 0.0, 0.0],
        &["Degree d", "Stripes (non-zero = on)"],
        newtbasin,
        sym_x,
    )
};
pub static PHOENIX: FractalType = std_type(
    "phoenix",
    148,
    "Phoenix: z → z² + p + q·z(previous), z(0) = pixel",
    STD_WINDOW,
    [0.56667, -0.5, 0.0, 0.0],
    &["p", "q", "Degree (0 = classic)"],
    phoenix,
    sym_x,
);
pub static MANDPHOENIX: FractalType = std_type(
    "mandphoenix",
    150,
    "Phoenix parameter plane: p + iq = pixel",
    MANDEL_WINDOW,
    [0.0; 4],
    &[
        "Real offset of z(0)",
        "Imaginary offset of z(0)",
        "Degree (0 = classic)",
    ],
    mandphoenix,
    sym_none,
);

/// Programs are called through [`crate::color::program_value`], which
/// supplies the pixel position and carried state.
fn program_iterate(_: Complex, _: &Params) -> u32 {
    INSIDE
}

fn program_symmetry(p: &Params) -> Symmetry {
    p.program
        .as_ref()
        .map_or(Symmetry::None, |prog| prog.0.symmetry(p))
}

/// A Fractint-format formula; the program comes from a `.frm` file.
pub static FORMULA: FractalType = FractalType {
    needs_program: true,
    // The program applies the colouring options itself.
    orbit_coloring: true,
    ..std_type(
        "formula",
        73,
        "A formula from a .frm file",
        STD_WINDOW,
        [0.0; 4],
        &["p1 (real)", "p1 (imaginary)", "p2 (real)", "p2 (imaginary)"],
        program_iterate,
        program_symmetry,
    )
};

static ALL: [&FractalType; 18] = [
    &MANDEL,
    &JULIA,
    &LAMBDA,
    &MANDELLAMBDA,
    &MANDEL4,
    &JULIA4,
    &MANOWAR,
    &MANOWARJ,
    &BARNSLEYM1,
    &BARNSLEYJ1,
    &SPIDER,
    &MAGNET1M,
    &MAGNET1J,
    &NEWTON,
    &NEWTBASIN,
    &PHOENIX,
    &MANDPHOENIX,
    &FORMULA,
];

pub fn all() -> &'static [&'static FractalType] {
    &ALL
}

pub fn by_fractint_number(number: i16) -> Option<&'static FractalType> {
    ALL.iter()
        .copied()
        .find(|t| t.fractint_number == Some(number))
}

pub fn lookup(name: &str) -> Option<&'static FractalType> {
    ALL.iter()
        .copied()
        .find(|t| t.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(t: &FractalType, p: &Params, re: f64, im: f64) -> u32 {
        (t.iterate)(Complex::new(re, im), p)
    }

    #[test]
    fn mandel_known_points() {
        let p = Params::new(&MANDEL);
        assert_eq!(at(&MANDEL, &p, 0.0, 0.0), INSIDE);
        assert_eq!(at(&MANDEL, &p, -1.0, 0.0), INSIDE);
        // c = 1: z = 1, 2, 5 → n = 3 → colour value 2.
        assert_eq!(at(&MANDEL, &p, 1.0, 0.0), 2);
        // c = 10 escapes on the first iterate: value 1 (minimum).
        assert_eq!(at(&MANDEL, &p, 10.0, 0.0), 1);
        // Escaping exactly at the limit is still outside: max_iter 3, n = 3.
        let mut p3 = p.clone();
        p3.max_iter = 3;
        assert_eq!(at(&MANDEL, &p3, 1.0, 0.0), 2);
        p3.max_iter = 2;
        assert_eq!(at(&MANDEL, &p3, 1.0, 0.0), INSIDE);
    }

    #[test]
    fn julia_known_points() {
        let mut p = Params::new(&JULIA);
        p.params = [0.0; 4];
        assert_eq!(at(&JULIA, &p, 0.5, 0.5), INSIDE);
        assert_eq!(at(&JULIA, &p, 3.0, 0.0), 1);
        // 1.1 → 1.21, 1.4641, 2.1436 (|z|² 4.6): n = 3 → value 2.
        assert_eq!(at(&JULIA, &p, 1.1, 0.0), 2);
    }

    #[test]
    fn escape_types_colour_by_first_escaping_iterate() {
        // lambda with λ = 1: z → z(1 − z). From 3: −6 → e = 1.
        let mut p = Params::new(&LAMBDA);
        p.params = [1.0, 0.0, 0.0, 0.0];
        assert_eq!(at(&LAMBDA, &p, 3.0, 0.0), 1);
        // mandel4 from z0 = c = 1.2: 1.2⁴ + 1.2 = 3.27 (|z|² 10.7) → e = 1.
        let p = Params::new(&MANDEL4);
        assert_eq!(at(&MANDEL4, &p, 1.2, 0.0), 1);
        assert_eq!(at(&MANDEL4, &p, 0.0, 0.0), INSIDE);
    }

    #[test]
    fn newton_counts_steps_plus_one() {
        let p = Params::new(&NEWTON);
        // A root itself passes the test before any step: colour 1.
        assert_eq!(at(&NEWTON, &p, 1.0, 0.0), 1);
        assert!(at(&NEWTON, &p, 0.3, 0.2) > 1);
        // The three basins of z³ = 1 get colours 1, 2, 3.
        let b = Params::new(&NEWTBASIN);
        assert_eq!(at(&NEWTBASIN, &b, 1.0, 0.0), 1);
        assert_eq!(at(&NEWTBASIN, &b, -0.5, 0.866), 2);
        assert_eq!(at(&NEWTBASIN, &b, -0.5, -0.866), 3);
        assert_eq!(newtbasin_mirror(2, &b), 3);
        assert_eq!(newtbasin_mirror(1, &b), 1);
        assert_eq!(newtbasin_mirror(11, &b), 10);
    }

    #[test]
    fn magnet_uses_its_own_bailout() {
        let p = Params::new(&MAGNET1M);
        assert_eq!(p.bailout, 100.0);
        assert_ne!(at(&MAGNET1M, &p, 3.9, 2.9), INSIDE);
    }

    #[test]
    fn symmetry_rules() {
        let mut p = Params::new(&MANDEL);
        assert_eq!((MANDEL.symmetry)(&p), Symmetry::XAxis);
        p.params[0] = 0.1;
        assert_eq!((MANDEL.symmetry)(&p), Symmetry::None);
        let b = Params::new(&BARNSLEYM1);
        assert_eq!((BARNSLEYM1.symmetry)(&b), Symmetry::XY);
        let j = Params::new(&JULIA4);
        assert_eq!((JULIA4.symmetry)(&j), Symmetry::Origin);
        let l = Params::new(&LAMBDA);
        assert_eq!((LAMBDA.symmetry)(&l), Symmetry::None);
    }

    #[test]
    fn registry() {
        assert_eq!(all().len(), 18);
        for t in all() {
            assert_eq!(lookup(t.name), Some(*t));
            assert_eq!(by_fractint_number(t.fractint_number.unwrap()), Some(*t));
            assert_eq!(Params::new(t).validate().is_ok(), !t.needs_program);
        }
        assert_eq!(lookup("Mandel"), Some(&MANDEL));
        assert_eq!(lookup("nope"), None);
    }
}
