// SPDX-License-Identifier: GPL-3.0-or-later

//! Random-dot stereograms (autostereograms) of an image, whose colours are
//! the depth, following docs/specs/stereogram.md (cited as §n).

use crate::crand::CRand;

/// Where the calibration bars go (§6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bars {
    Middle,
    Top,
    None,
}

/// Stereogram settings, with Fractint's defaults.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// Depth effect; negative swaps near and far.
    pub depth: i32,
    /// Width of the image on screen, in inches (`monitorwidth`).
    pub width_inches: f64,
    /// Depth from the colours' brightness instead of their index.
    pub grayscale: bool,
    pub bars: Bars,
    /// Seed of the random dots (Fractint uses the time).
    pub seed: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            depth: 100,
            width_inches: 10.0,
            grayscale: false,
            bars: Bars::Middle,
            seed: 1,
        }
    }
}

/// The farthest separation in inches; fitted to [1.865625, 1.865853]
/// (§3.3).
const FAR_INCHES: f64 = 125.0 / 67.0;

/// A tiling pattern for the free pixels instead of random dots (§7).
pub struct Pattern<'a> {
    pub pixels: &'a [u8],
    pub width: u32,
    pub height: u32,
}

/// The stereogram of an image of `width` × `height` palette indices with
/// palette `palette`: new indices into the same palette (§8).
pub fn render(
    pixels: &[u8],
    width: u32,
    height: u32,
    palette: &[[u8; 3]],
    s: &Settings,
    pattern: Option<&Pattern>,
) -> Vec<u8> {
    let (w, h) = (width as usize, height as usize);
    // Depth values: indices, or brightness (§3.5, an approximation).
    let value: Vec<f64> = (0..256)
        .map(|i| match (s.grayscale, palette.get(i)) {
            (true, Some(c)) => 0.30 * c[0] as f64 + 0.59 * c[1] as f64 + 0.11 * c[2] as f64,
            _ => i as f64,
        })
        .collect();
    let depth: Vec<f64> = pixels.iter().map(|&c| value[c as usize]).collect();
    let lo = depth.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = depth.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let lmax = (s.depth.unsigned_abs() as i64 * w as i64) / 4000;
    let sep = |c: f64| -> i64 {
        let mut cn = if hi > lo {
            ((c - lo) * 255.0 / (hi - lo)).floor() as i64
        } else {
            0
        };
        if s.depth < 0 {
            cn = 256 - cn;
        }
        let level = (cn * (lmax + 1) - 1).div_euclid(256);
        (w as f64 / s.width_inches * FAR_INCHES * (1.0 - (lmax - level) as f64 * 8.0 / w as f64))
            .floor() as i64
    };
    let mut rng = CRand::new(s.seed);
    let mut out = vec![0u8; w * h];
    let mut same = vec![0usize; w];
    for y in 0..h {
        // One rand() per pixel, row by row, used or not (§5).
        let free: Vec<u8> = match pattern {
            Some(p) => (0..w)
                .map(|x| {
                    let (pw, ph) = (p.width as usize, p.height as usize);
                    p.pixels[(y % ph) * pw + x % pw]
                })
                .collect(),
            None => (0..w).map(|_| (rng.next_u31() & 255) as u8).collect(),
        };
        // Link pixels that must be equal (§4).
        for (x, v) in same.iter_mut().enumerate() {
            *v = x;
        }
        for x in 0..w {
            let s = sep(depth[y * w + x]);
            // Negative separations aren't modelled (§3.4).
            if s < 0 {
                continue;
            }
            let mut left = x as i64 - (s + (y as i64 & 1)) / 2;
            let mut right = left + s;
            if left < 0 || right >= w as i64 {
                continue;
            }
            let mut k = same[left as usize] as i64;
            while k != left && k != right {
                if k < right {
                    left = k;
                } else {
                    (left, right) = (right, k);
                }
                k = same[left as usize] as i64;
            }
            same[left as usize] = right as usize;
        }
        let row = &mut out[y * w..(y + 1) * w];
        for x in (0..w).rev() {
            row[x] = if same[x] == x { free[x] } else { row[same[x]] };
        }
    }
    if s.bars != Bars::None {
        draw_bars(&mut out, w, h, &depth, &sep, palette, s.bars);
    }
    out
}

/// The two calibration bars (§6).
fn draw_bars(
    out: &mut [u8],
    w: usize,
    h: usize,
    depth: &[f64],
    sep: &dyn Fn(f64) -> i64,
    palette: &[[u8; 3]],
    bars: Bars,
) {
    let ybar = if bars == Bars::Middle { h / 2 } else { 8 };
    let xh = w / 16;
    let xs = w / 2 - xh..=w / 2 + xh;
    let ys = ybar.saturating_sub(8)..=(ybar + 8).min(h - 1);
    let (mut total, mut n) = (0i64, 0i64);
    for y in ys.clone() {
        for x in xs.clone() {
            total += sep(depth[y * w + x]);
            n += 1;
        }
    }
    let half = (total.div_euclid(n.max(1)) / 2) as usize;
    // The brightest palette entry, the first on ties.
    let colour = palette
        .iter()
        .enumerate()
        .max_by_key(|&(i, c)| {
            (
                c.iter().map(|&v| v as u32).sum::<u32>(),
                std::cmp::Reverse(i),
            )
        })
        .map_or(255, |(i, _)| i as u8);
    let (bar_w, bar_h) = (w / 200 + 1, h / 20 + 1);
    for y in ybar..(ybar + bar_h).min(h) {
        for x0 in [(w / 2).saturating_sub(half), w / 2 + half] {
            for x in x0..(x0 + bar_w).min(w) {
                out[y * w + x] = colour;
            }
        }
    }
}
