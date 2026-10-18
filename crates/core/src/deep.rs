// SPDX-License-Identifier: GPL-3.0-or-later

//! Deep zoom: views whose centre needs more precision than a double, and
//! perturbation rendering for them.
//!
//! One reference orbit is iterated at the view's centre in high precision;
//! every pixel then iterates only its difference δ from that orbit, in
//! doubles: for z → z² + c,
//!
//! ```text
//! δ(n+1) = 2·Z(m)·δ(n) + δ(n)² + δc,   then m ← m + 1
//! ```
//!
//! When the pixel's orbit z = Z(m) + δ comes closer to 0 than δ itself, or
//! the reference orbit ends, the pixel is "rebased" onto the start of the
//! reference (δ ← z − Z(0), m ← 0), which keeps δ small and avoids the
//! glitches of plain perturbation (rebasing as published by Zhuoran, 2021).

use crate::bignum::{BigFixed, frac_limbs_for};
use crate::iter::INSIDE;
use crate::{Complex, Params, Window};

/// Views with a half height below this times the centre's magnitude (or 1)
/// need deep precision: pixels would otherwise be only a few units of the
/// last place apart.
pub const DEEP_BELOW: f64 = 1e-9;

/// A view centred on an arbitrary-precision point.
#[derive(Clone, Debug, PartialEq)]
pub struct DeepView {
    pub re: BigFixed,
    pub im: BigFixed,
    /// Half the width and height of the view; the corners are the centres
    /// of the edge pixels, as for [`Window`].
    pub half_w: f64,
    pub half_h: f64,
}

impl DeepView {
    /// Builds a deep view with enough precision for its size.
    pub fn new(re: &BigFixed, im: &BigFixed, half_w: f64, half_h: f64) -> Self {
        let frac = frac_limbs_for(half_w.min(half_h) / 4096.0);
        Self {
            re: re.with_frac(frac),
            im: im.with_frac(frac),
            half_w,
            half_h,
        }
    }

    /// The view parsed from decimal centre coordinates.
    pub fn parse(re: &str, im: &str, half_w: f64, half_h: f64) -> Option<Self> {
        let frac = frac_limbs_for(half_w.min(half_h) / 4096.0);
        Some(Self::new(
            &BigFixed::parse(re, frac)?,
            &BigFixed::parse(im, frac)?,
            half_w,
            half_h,
        ))
    }

    /// The double-precision window around the rounded centre.
    pub fn window(&self) -> Window {
        let (cx, cy) = (self.re.to_f64(), self.im.to_f64());
        Window::new(
            cx - self.half_w,
            cx + self.half_w,
            cy - self.half_h,
            cy + self.half_h,
        )
    }

    /// Enough decimal digits to place pixels of this view.
    pub fn digits(&self) -> usize {
        let pixel = self.half_w.min(self.half_h) / 4096.0;
        (-pixel.log10()).ceil().max(0.0) as usize + 3
    }
}

/// Whether a view of this centre and half height needs deep precision.
pub fn needs_deep(cx: f64, cy: f64, half_h: f64) -> bool {
    half_h < DEEP_BELOW * cx.abs().max(cy.abs()).max(1.0)
}

/// Whether perturbation can render these parameters (otherwise a deep view
/// is rendered from its rounded centre).
pub fn supports(p: &Params) -> bool {
    matches!(p.fractal.name, "mandel" | "julia") && p.program.is_none() && !p.needs_orbit_coloring()
}

/// A reference orbit, rounded to doubles.
#[derive(Debug)]
pub struct Reference {
    z: Vec<Complex>,
    /// julia: δc is 0 and pixels start at their offset.
    julia: bool,
}

fn big_pair(z: Complex, frac: usize) -> (BigFixed, BigFixed) {
    (
        BigFixed::from_f64(z.re, frac),
        BigFixed::from_f64(z.im, frac),
    )
}

impl Reference {
    /// The orbit of the view's centre, up to max_iter iterates or until it
    /// escapes (that iterate included).
    pub fn new(p: &Params, view: &DeepView) -> Self {
        let julia = p.fractal.name == "julia";
        let frac = view.re.frac_limbs();
        let a = Complex::new(p.params[0], p.params[1]);
        let (mut zr, mut zi, cr, ci) = if julia {
            let (cr, ci) = big_pair(a, frac);
            (view.re.clone(), view.im.clone(), cr, ci)
        } else {
            let (zr, zi) = big_pair(a, frac);
            (zr, zi, view.re.clone(), view.im.clone())
        };
        let mut z = Vec::with_capacity(p.max_iter.min(1 << 20) as usize + 1);
        z.push(Complex::new(zr.to_f64(), zi.to_f64()));
        let two = BigFixed::from_f64(2.0, frac);
        for _ in 0..p.max_iter {
            let r2 = zr.square();
            let i2 = zi.square();
            let next_i = two.mul(&zr).mul(&zi).add(&ci);
            zr = r2.sub(&i2).add(&cr);
            zi = next_i;
            let v = Complex::new(zr.to_f64(), zi.to_f64());
            z.push(v);
            // Stops past the bailout, and on NaN.
            if !matches!(
                v.norm_sqr().partial_cmp(&p.bailout),
                Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
            ) {
                break;
            }
        }
        Self { z, julia }
    }

    pub fn len(&self) -> usize {
        self.z.len()
    }

    pub fn is_empty(&self) -> bool {
        self.z.is_empty()
    }

    /// The colour value of the pixel `offset` from the view's centre, with
    /// the conventions of mandel and julia (gif-fractint-blocks.md §9).
    pub fn value(&self, offset: Complex, p: &Params) -> u32 {
        let (mut delta, dc) = if self.julia {
            (offset, Complex::ZERO)
        } else {
            (Complex::ZERO, offset)
        };
        let last = self.z.len() - 1;
        let mut m = 0;
        for n in 1..=p.max_iter {
            if m == last {
                // The reference ended: continue from its start.
                delta = self.z[m] + delta - self.z[0];
                m = 0;
            }
            delta = self.z[m].scale(2.0) * delta + delta.square() + dc;
            m += 1;
            let z = self.z[m] + delta;
            let r = z.norm_sqr();
            if r > p.bailout {
                return (n - 1).max(1);
            }
            if !r.is_finite() {
                return (n - 1).max(1);
            }
            if r < delta.norm_sqr() {
                delta = z - self.z[0];
                m = 0;
            }
        }
        INSIDE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// At an ordinary zoom, perturbation gives the plain iteration's
    /// values.
    #[test]
    fn matches_direct_iteration() {
        for name in ["mandel", "julia"] {
            let mut p = Params::for_type(name).unwrap();
            p.max_iter = 500;
            let view = DeepView::parse("-0.75", "0.1", 0.01, 0.0075).unwrap();
            let r = Reference::new(&p, &view);
            let mut differ = 0;
            for i in 0..40 {
                for j in 0..30 {
                    let off = Complex::new(-0.01 + i as f64 * 0.0005, 0.0075 - j as f64 * 0.0005);
                    let direct = (p.fractal.iterate)(Complex::new(-0.75, 0.1) + off, &p);
                    differ += (r.value(off, &p) != direct) as u32;
                }
            }
            // Rounding of the absolute coordinates differs slightly.
            assert!(differ <= 12, "{name}: {differ} of 1200 differ");
        }
    }

    #[test]
    fn deep_threshold() {
        assert!(!needs_deep(-0.75, 0.0, 1e-6));
        assert!(needs_deep(-0.75, 0.0, 1e-12));
        let v = DeepView::parse(
            "-0.7436438870371587047521915",
            "0.13182590420531197",
            1e-20,
            1e-20,
        )
        .unwrap();
        assert!(v.re.frac_limbs() >= 3);
        assert!(v.digits() >= 24);
    }
}
