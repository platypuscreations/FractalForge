// SPDX-License-Identifier: GPL-3.0-or-later

//! Inside/outside, decomposition and biomorph colouring for the quadratic
//! types (mandel, julia), following docs/specs/coloring.md (cited as §n).
//!
//! The result is a final palette index 0..=255, stored as the pixel's colour
//! value ([`crate::color_index`] leaves values up to 255 unchanged).

use std::f64::consts::PI;

use crate::color::wrap;
use crate::{Complex, Inside, Outside, Params};

/// Threshold of the fmod and epscross tests (§5.7, §6.8).
const EPS: f64 = 0.01;
/// Scale of `inside=bof60` (§6.4).
const BOF60_SCALE: f64 = 75.0;

/// T(x): truncation toward zero, as a C cast from double to int.
fn t(x: f64) -> i64 {
    x as i64
}

/// P: the outside-mode map (§3).
fn outside_map(v: i64, max_iter: u32) -> u8 {
    if v <= 0 || v > max_iter as i64 {
        1
    } else {
        wrap(v as u32) as u8
    }
}

fn atan_value(z: Complex) -> i64 {
    t(z.im.atan2(z.re).abs() * 256.0 / PI)
}

fn fmod_index(last_small: Option<f64>) -> u8 {
    last_small.map_or(0, |m| t(m / EPS * 256.0).clamp(0, 255) as u8)
}

/// Statistics gathered along the orbit for the colouring modes.
struct Stats {
    /// Smallest |z|² over z₁…, and the first k where it occurs (§6.4–6.5).
    min_mag: f64,
    min_k: u32,
    /// |z|² of the last qualifying iterate below EPS (§5.7, §6.7).
    last_small: Option<f64>,
    /// Path length through the orbit (§5.8).
    path: f64,
    /// First epscross hit: index 2 or 6 (§6.8).
    eps: Option<u8>,
}

/// Colours one point of z → z² + c. `z0` is the start (mandel: the
/// perturbation; julia: the pixel). `julia` selects the julia conventions
/// of §2.2.
pub fn quadratic(z0: Complex, c: Complex, julia: bool, p: &Params) -> u32 {
    let b = p.bailout;
    let want_eps = p.inside == Inside::EpsCross;
    let want_path = p.outside == Outside::Tdis;
    let mut s = Stats {
        min_mag: f64::INFINITY,
        min_k: 0,
        last_small: None,
        path: 0.0,
        eps: None,
    };

    let z1 = z0.square() + c;
    let mut prev = z1;
    s.min_mag = z1.norm_sqr();
    s.min_k = 1;
    // Julia counts z1 in fmod and epscross, mandel doesn't (§5.7, §6.8).
    // A julia pixel whose z1 is already past the bailout is special (§2.2).
    let julia_z1_out = julia && z1.norm_sqr() > b;
    if julia {
        if want_path {
            s.path = (z1 - z0).norm_sqr().sqrt();
        }
        if !julia_z1_out {
            consider(&mut s, z1, want_eps);
        }
    }

    let mut escaped_at = None;
    let mut z = z1;
    for k in 2..=p.max_iter {
        z = z.square() + c;
        let mag = z.norm_sqr();
        if mag > b {
            escaped_at = Some(k);
            break;
        }
        if mag < s.min_mag {
            s.min_mag = mag;
            s.min_k = k;
        }
        consider(&mut s, z, want_eps);
        if want_path {
            s.path += (z - prev).norm_sqr().sqrt();
        }
        prev = z;
    }

    let index = match escaped_at {
        None => inside_index(z, &s, p),
        Some(k) => {
            if julia_z1_out {
                s.path = 0.0;
            }
            outside_index(k - 1, z, if julia_z1_out { z1 } else { z }, &s, p)
        }
    };
    index as u32
}

/// Records an iterate that comes before the final value.
fn consider(s: &mut Stats, z: Complex, want_eps: bool) {
    let mag = z.norm_sqr();
    if mag < EPS {
        s.last_small = Some(mag);
    }
    if want_eps && s.eps.is_none() {
        if z.re.abs() < EPS {
            s.eps = Some(2);
        } else if z.im.abs() < EPS {
            s.eps = Some(6);
        }
    }
}

/// §6, with `z` the last iterate z_maxiter.
fn inside_index(z: Complex, s: &Stats, p: &Params) -> u8 {
    let m = p.max_iter;
    let w = |v: i64| wrap(v.max(0) as u32) as u8;
    match p.inside {
        Inside::Color(c) => c,
        Inside::MaxIter => w(m as i64),
        Inside::ZMag => w(t(z.norm_sqr() * m as f64 / 2.0) + 1),
        Inside::Bof60 => w(t(s.min_mag.sqrt() * BOF60_SCALE)),
        Inside::Bof61 => w(s.min_k as i64),
        Inside::Atan => w(atan_value(z)),
        Inside::Fmod => fmod_index(s.last_small),
        Inside::EpsCross => s.eps.unwrap_or_else(|| w(m as i64)),
    }
}

/// §4–§5 and §7–§8 for an outside pixel with escape count `n` and final
/// value `z`; `decomp_z` is the value decomposition uses (§2.2).
fn outside_index(n: u32, z: Complex, decomp_z: Complex, s: &Stats, p: &Params) -> u8 {
    if let Some(e) = s.eps {
        return e;
    }
    if p.decomp > 0 {
        return decomp_index(decomp_z, p.decomp);
    }
    if let Some(b) = p.biomorph {
        let lim = p.bailout.sqrt();
        if z.re.abs() < lim || z.im.abs() < lim {
            return wrap(b) as u8;
        }
    }
    let n_i = n as i64;
    let m = p.max_iter;
    match p.outside {
        Outside::Iter => outside_map(n_i, m),
        Outside::Color(c) => c,
        Outside::Real => outside_map(n_i + 7 + t(z.re), m),
        Outside::Imag => outside_map(n_i + 7 + t(z.im), m),
        Outside::Mult => outside_map(t(n as f64 * z.re / z.im), m),
        Outside::Summ => outside_map(n_i + t(z.re + z.im), m),
        Outside::Atan => outside_map(atan_value(z), m),
        Outside::Fmod => fmod_index(s.last_small),
        Outside::Tdis => outside_map(t(s.path), m),
    }
}

/// §7: the angle sector of `z`.
fn decomp_index(z: Complex, k: u32) -> u8 {
    let sectors: u32 = match k {
        1 | 3 => 4,
        2 => 2,
        256 => 256,
        257.. => 128,
        _ => 1 << (31 - k.leading_zeros()),
    };
    let f = z.im.atan2(z.re).rem_euclid(2.0 * PI) / (2.0 * PI);
    let sector = (t(f * sectors as f64) as u32).min(sectors - 1);
    (sector + u32::from(k < 256)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Params;

    fn mandel(inside: Inside, outside: Outside) -> Params {
        let mut p = Params::for_type("mandel").unwrap();
        p.inside = inside;
        p.outside = outside;
        p
    }

    #[test]
    fn maps() {
        assert_eq!(outside_map(0, 150), 1);
        assert_eq!(outside_map(-5, 150), 1);
        assert_eq!(outside_map(151, 150), 1);
        assert_eq!(outside_map(150, 150), 150);
        assert_eq!(outside_map(300, 1000), 45);
        assert_eq!(t(-0.9), 0);
        assert_eq!(t(-2.4), -2);
    }

    #[test]
    fn decomp_sectors() {
        let up = Complex::new(0.0, 1.0);
        let down = Complex::new(0.0, -1.0);
        assert_eq!(decomp_index(up, 2), 1);
        assert_eq!(decomp_index(down, 2), 2);
        assert_eq!(decomp_index(Complex::new(1.0, 0.01), 4), 1);
        assert_eq!(decomp_index(down, 4), 4); // 3π/2 → sector 3 (+1)
        assert_eq!(decomp_index(Complex::new(1.0, 0.0), 256), 0);
        assert_eq!(decomp_index(down, 300), 96); // 128 sectors, 0-based
        assert_eq!(decomp_index(up, 1), 2); // K = 1 uses 4 sectors
    }

    #[test]
    fn default_colouring_matches_the_fast_path() {
        let p = mandel(Inside::MaxIter, Outside::Iter);
        for &(re, im) in &[(0.3, 0.5), (-0.75, 0.1), (2.5, 0.0), (-1.8, 0.01)] {
            let c = Complex::new(re, im);
            let fast = crate::color_index((p.fractal.iterate)(c, &p), &p);
            let orbit = quadratic(Complex::ZERO, c, false, &p);
            assert_eq!(fast as u32, orbit, "{re} {im}");
        }
    }

    #[test]
    fn inside_modes_produce_indices() {
        let c = Complex::new(-0.1, 0.1); // deep inside the main cardioid
        for mode in Inside::ALL {
            let p = mandel(mode, Outside::Iter);
            let v = quadratic(Complex::ZERO, c, false, &p);
            assert!(v <= 255, "{mode:?} gave {v}");
        }
        let p = mandel(Inside::MaxIter, Outside::Iter);
        assert_eq!(quadratic(Complex::ZERO, c, false, &p), 150);
    }
}
