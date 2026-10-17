// SPDX-License-Identifier: GPL-3.0-or-later

//! Everything needed to reproduce an image, independent of image size.

use crate::Complex;
use crate::types::{self, FractalType};

/// The rectangle of the complex plane shown in the image. The corner values
/// are the centres of the edge pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl Window {
    pub const fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    /// Builds a window from its centre, a magnification (1 = a height of 2
    /// units) and the image aspect ratio (width / height).
    pub fn from_center_mag(center: Complex, mag: f64, aspect: f64) -> Self {
        let half_h = 1.0 / mag;
        let half_w = half_h * aspect;
        Self::new(
            center.re - half_w,
            center.re + half_w,
            center.im - half_h,
            center.im + half_h,
        )
    }

    pub fn is_valid(&self) -> bool {
        [self.x_min, self.x_max, self.y_min, self.y_max]
            .iter()
            .all(|v| v.is_finite())
            && self.x_min < self.x_max
            && self.y_min < self.y_max
    }
}

/// Colour used for points that never escape. All but `Color` and
/// `MaxIter` follow docs/specs/coloring.md §6 and need orbit colouring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inside {
    /// A fixed palette index.
    Color(u8),
    /// The colour of the maximum iteration count.
    MaxIter,
    /// By the magnitude of the last iterate.
    ZMag,
    /// By how close the orbit came to the origin.
    Bof60,
    /// By when the orbit came closest to the origin.
    Bof61,
    /// By the angle of the last iterate.
    Atan,
    /// By the last iterate close to the origin.
    Fmod,
    /// Marks orbits that come close to an axis (also recolours outside).
    EpsCross,
}

/// Colouring for points that escape (docs/specs/coloring.md §5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outside {
    /// By escape count.
    Iter,
    /// A fixed palette index.
    Color(u8),
    /// Escape count plus the real part of the final value.
    Real,
    /// Escape count plus the imaginary part of the final value.
    Imag,
    /// Escape count times the final value's real/imaginary ratio.
    Mult,
    /// Escape count plus the final value's real and imaginary parts.
    Summ,
    /// By the angle of the final value.
    Atan,
    /// By the last iterate close to the origin.
    Fmod,
    /// By the length of the orbit's path.
    Tdis,
}

impl Inside {
    pub const ALL: [Inside; 8] = [
        Inside::Color(0),
        Inside::MaxIter,
        Inside::ZMag,
        Inside::Bof60,
        Inside::Bof61,
        Inside::Atan,
        Inside::Fmod,
        Inside::EpsCross,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Inside::Color(_) => "colour",
            Inside::MaxIter => "maxiter",
            Inside::ZMag => "zmag",
            Inside::Bof60 => "bof60",
            Inside::Bof61 => "bof61",
            Inside::Atan => "atan",
            Inside::Fmod => "fmod",
            Inside::EpsCross => "epscross",
        }
    }

    /// Parses a mode name or a palette index (wrapped as Fractint does).
    pub fn parse(s: &str) -> Option<Self> {
        if let Ok(n) = s.parse::<u32>() {
            return Some(Inside::Color(crate::color::wrap(n) as u8));
        }
        Self::ALL[1..]
            .iter()
            .copied()
            .find(|m| m.name().eq_ignore_ascii_case(s))
    }
}

impl Outside {
    pub const ALL: [Outside; 9] = [
        Outside::Iter,
        Outside::Color(0),
        Outside::Real,
        Outside::Imag,
        Outside::Mult,
        Outside::Summ,
        Outside::Atan,
        Outside::Fmod,
        Outside::Tdis,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Outside::Iter => "iter",
            Outside::Color(_) => "colour",
            Outside::Real => "real",
            Outside::Imag => "imag",
            Outside::Mult => "mult",
            Outside::Summ => "summ",
            Outside::Atan => "atan",
            Outside::Fmod => "fmod",
            Outside::Tdis => "tdis",
        }
    }

    /// Parses a mode name or a palette index 0..=255.
    pub fn parse(s: &str) -> Option<Self> {
        if let Ok(n) = s.parse::<u8>() {
            return Some(Outside::Color(n));
        }
        Self::ALL
            .iter()
            .copied()
            .find(|m| m.name().eq_ignore_ascii_case(s) && *m != Outside::Color(0))
    }
}

/// Highest allowed iteration limit. Larger values collide with the buffer's
/// sentinel values.
pub const MAX_ITER_LIMIT: u32 = 1 << 30;

#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    pub fractal: &'static FractalType,
    pub window: Window,
    /// Type-specific parameters (e.g. the Julia constant in the first two).
    pub params: [f64; 4],
    pub max_iter: u32,
    /// Escape when `|z|²` exceeds this.
    pub bailout: f64,
    pub inside: Inside,
    pub outside: Outside,
    /// Decomposition: colour outside points by the angle of the final value
    /// in this many sectors (rounded as in coloring.md §7); 0 = off.
    pub decomp: u32,
    /// Biomorph: outside points near an axis get this colour value.
    pub biomorph: Option<u32>,
    /// Copy mirrored pixels where the type is symmetric (the usual
    /// setting). Off computes every pixel; off-centre views then differ.
    pub use_symmetry: bool,
    /// The run-time definition for types that need one (formulas).
    pub program: Option<crate::Program>,
    /// Functions bound to fn1…fn4, as indices into Fractint's function
    /// table (gif-fractint-blocks.md §6.2). Default sin, sqr, sinh, cosh.
    pub functions: [u8; 4],
}

impl Params {
    /// Default parameters for a fractal type.
    pub fn new(fractal: &'static FractalType) -> Self {
        Self {
            fractal,
            window: fractal.default_window,
            params: fractal.default_params,
            max_iter: 150,
            bailout: fractal.default_bailout,
            inside: Inside::Color(0),
            outside: Outside::Iter,
            decomp: 0,
            biomorph: None,
            use_symmetry: true,
            program: None,
            functions: [0, 6, 2, 3],
        }
    }

    /// Default parameters for the type named `name`.
    pub fn for_type(name: &str) -> Option<Self> {
        types::lookup(name).map(Self::new)
    }

    /// Whether any colouring option needs per-orbit statistics.
    pub fn needs_orbit_coloring(&self) -> bool {
        !matches!(self.inside, Inside::Color(_) | Inside::MaxIter)
            || self.outside != Outside::Iter
            || self.decomp != 0
            || self.biomorph.is_some()
    }

    /// Colouring options with which Fractint-format renderers compute every
    /// pixel instead of copying mirrored ones (coloring.md §13).
    pub fn coloring_breaks_symmetry(&self) -> bool {
        self.decomp != 0
            || matches!(
                self.outside,
                Outside::Imag | Outside::Summ | Outside::Mult | Outside::Tdis
            )
    }

    /// The bailout used when none is chosen: the type's, or 100 with
    /// biomorph (coloring.md §8).
    pub fn default_bailout(&self) -> f64 {
        if self.biomorph.is_some() {
            100.0
        } else {
            self.fractal.default_bailout
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.fractal.needs_program && self.program.is_none() {
            return Err(format!("type {} needs a formula", self.fractal.name));
        }
        if self.needs_orbit_coloring() && !self.fractal.orbit_coloring {
            return Err(format!(
                "type {} supports only the default inside and outside colouring",
                self.fractal.name
            ));
        }
        if !self.window.is_valid() {
            return Err(format!("invalid window {:?}", self.window));
        }
        if self.max_iter == 0 || self.max_iter > MAX_ITER_LIMIT {
            return Err(format!(
                "max iterations must be 1..={MAX_ITER_LIMIT}, got {}",
                self.max_iter
            ));
        }
        if !(self.bailout.is_finite() && self.bailout > 0.0) {
            return Err(format!("bailout must be positive, got {}", self.bailout));
        }
        if self.params.iter().any(|p| !p.is_finite()) {
            return Err("type parameters must be finite".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_validate() {
        for t in types::all().iter().filter(|t| !t.needs_program) {
            Params::new(t).validate().unwrap();
        }
    }

    #[test]
    fn center_mag_round_trip() {
        let w = Window::from_center_mag(Complex::new(-0.5, 0.25), 2.0, 4.0 / 3.0);
        assert_eq!(w.y_max - w.y_min, 1.0);
        assert!(((w.x_max - w.x_min) - 4.0 / 3.0).abs() < 1e-15);
        assert!(((w.x_min + w.x_max) / 2.0 + 0.5).abs() < 1e-15);
    }

    #[test]
    fn rejects_bad_values() {
        let mut p = Params::for_type("mandel").unwrap();
        p.max_iter = 0;
        assert!(p.validate().is_err());
        let mut p = Params::for_type("mandel").unwrap();
        p.window.x_max = p.window.x_min;
        assert!(p.validate().is_err());
        let mut p = Params::for_type("mandel").unwrap();
        p.bailout = f64::NAN;
        assert!(p.validate().is_err());
    }
}
