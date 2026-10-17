// SPDX-License-Identifier: GPL-3.0-or-later

//! The data-driven registry of fractal types.

use std::fmt;

use crate::iter::INSIDE;
use crate::{Complex, Params, Window};

/// Symmetry a renderer may exploit to skip work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symmetry {
    None,
    /// Mirror image about the real axis: `f(conj z) = conj f(z)`.
    XAxis,
    /// Point symmetry about the origin: `f(-z) = -f(z)` up to colour.
    Origin,
}

/// Iterates one point and returns its escape count, or [`INSIDE`].
pub type IterateFn = fn(pixel: Complex, params: &Params) -> u32;

pub struct FractalType {
    /// Short identifier used in files and on the command line.
    pub name: &'static str,
    pub description: &'static str,
    pub default_window: Window,
    pub default_params: [f64; 4],
    /// Labels for the meaningful entries of `Params::params`.
    pub param_names: &'static [&'static str],
    pub iterate: IterateFn,
    /// The symmetry these particular parameters have.
    pub symmetry: fn(&Params) -> Symmetry,
    /// Whether an interrupted render can be checkpointed and resumed.
    pub resumable: bool,
    /// Type number in Fractint-format GIF blocks, if the type has one.
    pub fractint_number: Option<i16>,
}

impl fmt::Debug for FractalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FractalType").field(&self.name).finish()
    }
}

impl PartialEq for FractalType {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// Escape-time iteration of `z -> z² + c`: the number n (1..=max_iter) of
/// the first iterate with `|z|² > bailout`, or [`INSIDE`]. The starting point
/// itself is not tested, matching Fractint-format images.
fn escape_quadratic(mut z: Complex, c: Complex, max_iter: u32, bailout: f64) -> u32 {
    for n in 1..=max_iter {
        z = z.square() + c;
        if z.norm_sqr() > bailout {
            return n;
        }
    }
    INSIDE
}

fn mandel_iterate(pixel: Complex, p: &Params) -> u32 {
    // The parameters perturb the starting point.
    let z0 = Complex::new(p.params[0], p.params[1]);
    escape_quadratic(z0, pixel, p.max_iter, p.bailout)
}

fn mandel_symmetry(p: &Params) -> Symmetry {
    if p.params[1] == 0.0 {
        Symmetry::XAxis
    } else {
        Symmetry::None
    }
}

fn julia_iterate(pixel: Complex, p: &Params) -> u32 {
    let c = Complex::new(p.params[0], p.params[1]);
    escape_quadratic(pixel, c, p.max_iter, p.bailout)
}

fn julia_symmetry(_: &Params) -> Symmetry {
    Symmetry::Origin
}

pub static MANDEL: FractalType = FractalType {
    name: "mandel",
    description: "Mandelbrot set, z -> z^2 + c with c the pixel",
    default_window: Window::new(-2.5, 1.5, -1.5, 1.5),
    default_params: [0.0; 4],
    param_names: &[
        "Real perturbation of z(0)",
        "Imaginary perturbation of z(0)",
    ],
    iterate: mandel_iterate,
    symmetry: mandel_symmetry,
    resumable: true,
    fractint_number: Some(4),
};

pub static JULIA: FractalType = FractalType {
    name: "julia",
    description: "Julia set, z -> z^2 + c with z(0) the pixel",
    default_window: Window::new(-2.0, 2.0, -1.5, 1.5),
    default_params: [0.3, 0.6, 0.0, 0.0],
    param_names: &["Real part of c", "Imaginary part of c"],
    iterate: julia_iterate,
    symmetry: julia_symmetry,
    resumable: true,
    fractint_number: Some(6),
};

static ALL: [&FractalType; 2] = [&MANDEL, &JULIA];

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

    #[test]
    fn mandel_known_points() {
        let p = Params::new(&MANDEL);
        let it = |re, im| (MANDEL.iterate)(Complex::new(re, im), &p);
        assert_eq!(it(0.0, 0.0), INSIDE);
        assert_eq!(it(-1.0, 0.0), INSIDE);
        // c = 1: 0, 1, 2, 5 -> escapes when |z|² first exceeds 4, at n = 3.
        assert_eq!(it(1.0, 0.0), 3);
        // c = 2: 0, 2, 6 -> n = 2.
        assert_eq!(it(2.0, 0.0), 2);
        // Far away escapes after the first step.
        assert_eq!(it(10.0, 0.0), 1);
        // A point escaping exactly at the limit is outside.
        let mut p2 = p.clone();
        p2.max_iter = 3;
        assert_eq!((MANDEL.iterate)(Complex::new(1.0, 0.0), &p2), 3);
        p2.max_iter = 2;
        assert_eq!((MANDEL.iterate)(Complex::new(1.0, 0.0), &p2), INSIDE);
    }

    #[test]
    fn julia_known_points() {
        let mut p = Params::new(&JULIA);
        p.params = [0.0; 4];
        let it = |re, im| (JULIA.iterate)(Complex::new(re, im), &p);
        // With c = 0 the filled Julia set is the unit disc.
        assert_eq!(it(0.5, 0.5), INSIDE);
        // The start point isn't tested: 3 -> 9 escapes on the first iterate.
        assert_eq!(it(3.0, 0.0), 1);
        assert_eq!(it(1.5, 0.0), 1);
        assert_eq!(it(1.1, 0.0), 3); // 1.21, 1.4641, 2.1436 (|z|² 4.6)
    }

    #[test]
    fn julia_is_point_symmetric() {
        let p = Params::new(&JULIA);
        for &(re, im) in &[(0.1, 0.2), (-0.7, 0.3), (1.1, -0.9)] {
            assert_eq!(
                (JULIA.iterate)(Complex::new(re, im), &p),
                (JULIA.iterate)(Complex::new(-re, -im), &p)
            );
        }
    }

    #[test]
    fn mandel_symmetry_depends_on_perturbation() {
        let mut p = Params::new(&MANDEL);
        assert_eq!((MANDEL.symmetry)(&p), Symmetry::XAxis);
        p.params[1] = 0.1;
        assert_eq!((MANDEL.symmetry)(&p), Symmetry::None);
    }

    #[test]
    fn lookup_is_case_insensitive() {
        assert_eq!(lookup("Mandel"), Some(&MANDEL));
        assert_eq!(lookup("JULIA"), Some(&JULIA));
        assert_eq!(lookup("nope"), None);
    }
}
