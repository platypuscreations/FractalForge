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

/// Colour used for points that never escape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inside {
    /// A fixed palette index.
    Color(u8),
    /// The colour an outside point would get at the maximum iteration count.
    MaxIter,
}

/// Colouring for points that escape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outside {
    /// Colour by escape iteration count.
    Iter,
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
    /// Copy mirrored pixels where the type is symmetric (the usual
    /// setting). Off computes every pixel; off-centre views then differ.
    pub use_symmetry: bool,
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
            use_symmetry: true,
        }
    }

    /// Default parameters for the type named `name`.
    pub fn for_type(name: &str) -> Option<Self> {
        types::lookup(name).map(Self::new)
    }

    pub fn validate(&self) -> Result<(), String> {
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
        for t in types::all() {
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
