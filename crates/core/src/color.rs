// SPDX-License-Identifier: GPL-3.0-or-later

//! Mapping iteration counts to palette indices, and FractalForge's own
//! palettes.

use crate::iter::{INSIDE, UNKNOWN};
use crate::{Inside, Outside, Params};

/// Palette index for one iteration-buffer value. Unknown pixels get index 0.
///
/// Escape count n maps to `(n - 1) mod 256`, drawn as 1 where that is 0, so
/// index 0 stays free for the default inside colour. This is the mapping
/// Fractint-format images use (docs/specs/gif-fractint-blocks.md §9).
pub fn color_index(value: u32, params: &Params) -> u8 {
    match value {
        UNKNOWN => 0,
        INSIDE => match params.inside {
            Inside::Color(c) => c,
            Inside::MaxIter => outside_index(params.max_iter, params),
        },
        n => outside_index(n, params),
    }
}

fn outside_index(n: u32, params: &Params) -> u8 {
    match params.outside {
        Outside::Iter => match (n.wrapping_sub(1) % 256) as u8 {
            0 => 1,
            c => c,
        },
    }
}

/// Key colours of [`Palette::forge`].
pub const FORGE_KEYS: [[u8; 3]; 7] = [
    [0, 7, 100],
    [32, 107, 203],
    [237, 255, 255],
    [255, 170, 0],
    [200, 40, 20],
    [90, 10, 110],
    [0, 7, 100],
];

/// FractalForge's built-in palettes as (name, key colours).
pub const PRESETS: &[(&str, &[[u8; 3]])] = &[
    ("Forge", &FORGE_KEYS),
    (
        "Ember",
        &[
            [20, 0, 0],
            [180, 30, 0],
            [255, 160, 20],
            [255, 250, 200],
            [120, 20, 0],
            [20, 0, 0],
        ],
    ),
    (
        "Lagoon",
        &[
            [0, 20, 40],
            [0, 120, 140],
            [140, 240, 220],
            [250, 250, 240],
            [20, 80, 160],
            [0, 20, 40],
        ],
    ),
    ("Grey", &[[30, 30, 30], [235, 235, 235], [30, 30, 30]]),
    (
        "Spectrum",
        &[
            [255, 0, 0],
            [255, 255, 0],
            [0, 255, 0],
            [0, 255, 255],
            [0, 0, 255],
            [255, 0, 255],
            [255, 0, 0],
        ],
    ),
];

/// A 256-entry RGB palette.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette(pub [[u8; 3]; 256]);

impl Palette {
    /// FractalForge's default palette: black at index 0, then a smooth cycle
    /// through deep blue, cyan, white, gold, red and violet.
    pub fn forge() -> Self {
        Self::from_keys(&FORGE_KEYS)
    }

    /// A palette with black at index 0 and indices 1..=255 interpolated
    /// linearly through `keys` (at least two). Repeat the first key at the
    /// end for a seamless colour cycle.
    pub fn from_keys(keys: &[[u8; 3]]) -> Self {
        let mut colors = [[0u8; 3]; 256];
        let segments = keys.len().saturating_sub(1);
        if segments == 0 {
            let fill = keys.first().copied().unwrap_or([0; 3]);
            colors[1..].fill(fill);
            return Self(colors);
        }
        for (i, color) in colors.iter_mut().enumerate().skip(1) {
            // Position in 1/256ths of a segment.
            let t = (i - 1) * segments * 256 / 254;
            let seg = (t / 256).min(segments - 1);
            let f = (t - seg * 256) as u32;
            for ch in 0..3 {
                let a = keys[seg][ch] as u32;
                let b = keys[seg + 1][ch] as u32;
                color[ch] = ((a * (256 - f) + b * f) / 256) as u8;
            }
        }
        Self(colors)
    }

    /// Rotates indices 1..=255 by `steps`, leaving index 0 fixed (colour
    /// cycling).
    pub fn rotated(&self, steps: i32) -> Self {
        let mut out = self.clone();
        for i in 1..256usize {
            let src = (i as i64 - 1 - steps as i64).rem_euclid(255) as usize + 1;
            out.0[i] = self.0[src];
        }
        out
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::forge()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices() {
        let mut p = Params::for_type("mandel").unwrap();
        assert_eq!(color_index(1, &p), 1);
        assert_eq!(color_index(2, &p), 1);
        assert_eq!(color_index(3, &p), 2);
        assert_eq!(color_index(256, &p), 255);
        assert_eq!(color_index(257, &p), 1);
        assert_eq!(color_index(258, &p), 1);
        assert_eq!(color_index(INSIDE, &p), 0);
        assert_eq!(color_index(UNKNOWN, &p), 0);
        p.inside = Inside::MaxIter;
        assert_eq!(color_index(INSIDE, &p), 149);
    }

    #[test]
    fn palette_shape() {
        let p = Palette::forge();
        assert_eq!(p.0[0], [0, 0, 0]);
        assert_eq!(p.0[1], [0, 7, 100]);
        assert_eq!(p.rotated(255), p);
        assert_eq!(p.rotated(1).0[2], p.0[1]);
        assert_eq!(p.rotated(1).0[1], p.0[255]);
    }

    #[test]
    fn from_keys_hits_the_ends() {
        let p = Palette::from_keys(&[[0, 0, 0], [255, 255, 255]]);
        assert_eq!(p.0[1], [0, 0, 0]);
        assert_eq!(p.0[255], [255, 255, 255]);
        assert!(p.0.windows(2).skip(1).all(|w| w[0][0] <= w[1][0]));
        assert_eq!(Palette::from_keys(&[[9, 9, 9]]).0[200], [9, 9, 9]);
        for (_, keys) in PRESETS {
            assert_eq!(Palette::from_keys(keys).0[0], [0, 0, 0]);
        }
    }
}
