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

/// Fractint's 3D viewing settings, used by the types that draw 3D objects
/// (gif-fractint-blocks.md §2.1, offsets 128–148).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct View3d {
    /// Rotation about x, y and z, in degrees.
    pub rotation: [i16; 3],
    /// Perspective distance; 0 = none.
    pub perspective: i16,
    /// Shift right and up, in percent of the view.
    pub xyshift: [i16; 2],
}

impl Default for View3d {
    fn default() -> Self {
        Self {
            rotation: [60, 30, 0],
            perspective: 0,
            xyshift: [0, 0],
        }
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

/// Number of type parameters, as in Fractint (`params=` takes up to 10).
pub const PARAM_COUNT: usize = 10;

/// Highest allowed iteration limit. Larger values collide with the buffer's
/// sentinel values.
pub const MAX_ITER_LIMIT: u32 = 1 << 30;

#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    pub fractal: &'static FractalType,
    pub window: Window,
    /// Type-specific parameters (e.g. the Julia constant in the first two).
    pub params: [f64; PARAM_COUNT],
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
    /// Mirror copying chosen explicitly (Fractint's `symmetry=xaxis` etc.),
    /// replacing the type's own.
    pub forced_symmetry: Option<crate::Symmetry>,
    /// The run-time definition for types that need one (formulas).
    pub program: Option<crate::Program>,
    /// Functions bound to fn1…fn4, as indices into Fractint's function
    /// table (gif-fractint-blocks.md §6.2). Default sin, sqr, sinh, cosh.
    pub functions: [u8; 4],
    /// 3D viewing, for types that use it.
    pub view3d: View3d,
    /// For deep zooms: the view with an arbitrary-precision centre.
    /// `window` then holds the same view rounded to doubles.
    pub deep: Option<crate::deep::DeepView>,
}

impl Params {
    /// Default parameters for a fractal type.
    pub fn new(fractal: &'static FractalType) -> Self {
        Self {
            fractal,
            window: fractal.default_window,
            params: {
                let mut p = [0.0; PARAM_COUNT];
                p[..fractal.default_params.len()].copy_from_slice(fractal.default_params);
                p
            },
            max_iter: 150,
            bailout: fractal.default_bailout,
            inside: Inside::Color(0),
            outside: Outside::Iter,
            decomp: 0,
            biomorph: None,
            use_symmetry: true,
            forced_symmetry: None,
            program: None,
            functions: [0, 6, 2, 3],
            view3d: View3d::default(),
            deep: None,
        }
    }

    /// Default parameters for the type named `name`.
    pub fn for_type(name: &str) -> Option<Self> {
        types::lookup(name).map(Self::new)
    }

    /// The Julia set for the constant `c` (a point of this parameter-plane
    /// image), in its type's default window. Parameters past the first two
    /// (e.g. phoenix's degree) and the other settings carry over.
    pub fn to_julia(&self, c: Complex) -> Option<Self> {
        let julia = types::julia_of(self.fractal)?;
        let mut p = self.clone();
        p.fractal = julia;
        p.window = julia.default_window;
        p.deep = None;
        p.params[0] = c.re;
        p.params[1] = c.im;
        p.forced_symmetry = None;
        Some(p)
    }

    /// The parameter plane this Julia set belongs to, in `window` (or its
    /// default window), with no start offset.
    pub fn to_mandel(&self, window: Option<Window>) -> Option<Self> {
        let mandel = types::mandel_of(self.fractal)?;
        let mut p = self.clone();
        p.fractal = mandel;
        p.window = window.unwrap_or(mandel.default_window);
        p.deep = None;
        p.params[0] = 0.0;
        p.params[1] = 0.0;
        p.forced_symmetry = None;
        Some(p)
    }

    /// The orbit of the point at `pixel`: at most `limit` iterates, in
    /// order. `None` for types without orbits.
    pub fn orbit(&self, at: &crate::PixelContext, limit: u32) -> Option<Vec<Complex>> {
        let mut p = self.clone();
        p.max_iter = p.max_iter.min(limit.saturating_add(1)).max(1);
        let mut points = Vec::new();
        let mut visit = |z: Complex| points.push(z);
        if let Some(prog) = &p.program {
            let mut carry = prog.0.new_carry(&p, at.width, at.height);
            if !prog.0.orbit(at, &p, &mut carry, &mut visit) {
                return None;
            }
        } else {
            (self.fractal.orbit?)(at.pixel, &p, &mut visit);
        }
        points.truncate(limit as usize);
        Some(points)
    }

    /// Whether the image depends on [`Params::view3d`] (3D IFS entries
    /// and the 3D attractors).
    pub fn uses_view3d(&self) -> bool {
        match self.fractal.name {
            "ifs" => self.program.as_ref().is_some_and(crate::ifs::is_3d),
            "lorenz3d" | "rossler3d" | "pickover" | "lorenz3d1" | "lorenz3d3" | "lorenz3d4"
            | "kamtorus3d" => true,
            _ => false,
        }
    }

    /// Half the width and height of the view.
    pub fn half_spans(&self) -> (f64, f64) {
        match &self.deep {
            Some(d) => (d.half_w, d.half_h),
            None => (
                (self.window.x_max - self.window.x_min) / 2.0,
                (self.window.y_max - self.window.y_min) / 2.0,
            ),
        }
    }

    /// Changes the view by `op`, a change of window (zoom, pan, …), keeping
    /// full precision for deep views: `op` sees the window relative to the
    /// view's centre. Switches to or from a deep view as the size needs.
    pub fn reframe(&mut self, op: impl Fn(&Window) -> Window) {
        use crate::bignum::BigFixed;
        use crate::deep::{DeepView, needs_deep};
        let (hw, hh) = self.half_spans();
        let (re, im) = match &self.deep {
            Some(d) => (d.re.clone(), d.im.clone()),
            None => {
                let w = &self.window;
                let frac = crate::bignum::frac_limbs_for(hh / 4096.0);
                (
                    BigFixed::from_f64((w.x_min + w.x_max) / 2.0, frac),
                    BigFixed::from_f64((w.y_min + w.y_max) / 2.0, frac),
                )
            }
        };
        if self.deep.is_none() {
            let new = op(&self.window);
            let (cx, cy) = ((new.x_min + new.x_max) / 2.0, (new.y_min + new.y_max) / 2.0);
            if !needs_deep(cx, cy, (new.y_max - new.y_min) / 2.0) {
                self.window = new;
                return;
            }
        }
        let rel = op(&Window::new(-hw, hw, -hh, hh));
        let (dx, dy) = ((rel.x_min + rel.x_max) / 2.0, (rel.y_min + rel.y_max) / 2.0);
        let (nhw, nhh) = ((rel.x_max - rel.x_min) / 2.0, (rel.y_max - rel.y_min) / 2.0);
        let frac = crate::bignum::frac_limbs_for(nhw.min(nhh) / 4096.0).max(re.frac_limbs());
        let re = re.with_frac(frac).add(&BigFixed::from_f64(dx, frac));
        let im = im.with_frac(frac).add(&BigFixed::from_f64(dy, frac));
        self.set_view(DeepView::new(&re, &im, nhw, nhh));
    }

    /// Sets the view from a precise centre, as a deep view only when its
    /// size needs one.
    pub fn set_view(&mut self, view: crate::deep::DeepView) {
        let w = view.window();
        let (cx, cy) = (view.re.to_f64(), view.im.to_f64());
        if crate::deep::needs_deep(cx, cy, view.half_h) {
            self.window = w;
            self.deep = Some(view);
        } else {
            self.window = w;
            self.deep = None;
        }
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
        match &self.deep {
            // The rounded window of a deep view may have no width.
            Some(d) => {
                if !(d.half_w > 0.0
                    && d.half_h > 0.0
                    && d.half_w.is_finite()
                    && d.half_h.is_finite())
                {
                    return Err(format!(
                        "invalid deep view size {} × {}",
                        d.half_w, d.half_h
                    ));
                }
            }
            None => {
                if !self.window.is_valid() {
                    return Err(format!("invalid window {:?}", self.window));
                }
            }
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
    fn julia_toggle_round_trip() {
        let m = Params::for_type("mandphoenix").unwrap();
        let mut m = m;
        m.params[2] = 3.0;
        let j = m.to_julia(Complex::new(0.25, -0.5)).unwrap();
        assert_eq!(j.fractal.name, "phoenix");
        assert_eq!(&j.params[..3], &[0.25, -0.5, 3.0]);
        let back = j.to_mandel(Some(m.window)).unwrap();
        assert_eq!(back.fractal.name, "mandphoenix");
        assert_eq!(back.window, m.window);
        assert_eq!(&back.params[..3], &[0.0, 0.0, 3.0]);
        assert!(
            Params::for_type("spider")
                .unwrap()
                .to_julia(Complex::ZERO)
                .is_none()
        );
        assert!(
            Params::for_type("newton")
                .unwrap()
                .to_mandel(None)
                .is_none()
        );
    }

    /// The parameter plane's colour at c is the Julia set's (for that c)
    /// at the parameter plane's start value.
    #[test]
    fn julia_pairs_share_the_map() {
        let c = Complex::new(0.31, 0.47);
        // mandel starts at 0; mandel4 and barnsleym1 at the pixel.
        for (m, start) in [("mandel", Complex::ZERO), ("mandel4", c), ("barnsleym1", c)] {
            let mp = Params::for_type(m).unwrap();
            let jp = mp.to_julia(c).unwrap();
            assert_eq!(
                (mp.fractal.iterate)(c, &mp),
                (jp.fractal.iterate)(start, &jp),
                "{m}"
            );
        }
    }

    #[test]
    fn orbit_ends_at_escape() {
        let p = Params::for_type("mandel").unwrap();
        let at = crate::PixelContext {
            pixel: Complex::new(1.0, 1.0),
            col: 0,
            row: 0,
            width: 1,
            height: 1,
        };
        // 0 → 1+i → 1+3i, which escapes.
        assert_eq!(
            p.orbit(&at, 100).unwrap(),
            vec![Complex::new(1.0, 1.0), Complex::new(1.0, 3.0)]
        );
        let inside = crate::PixelContext {
            pixel: Complex::ZERO,
            ..at
        };
        assert_eq!(p.orbit(&inside, 20).unwrap().len(), 20);
    }

    /// Zooming far in keeps the exact centre; zooming back out returns to
    /// an ordinary window.
    #[test]
    fn reframe_goes_deep_and_back() {
        let mut p = Params::for_type("mandel").unwrap();
        let zoom = |w: &Window, f: f64, u: f64| {
            let (cx, cy) = (w.x_min + u * (w.x_max - w.x_min), (w.y_min + w.y_max) / 2.0);
            let (hw, hh) = (
                (w.x_max - w.x_min) / (2.0 * f),
                (w.y_max - w.y_min) / (2.0 * f),
            );
            let (nx, ny) = (cx - (u - 0.5) * 2.0 * hw, cy);
            Window::new(nx - hw, nx + hw, ny - hh, ny + hh)
        };
        for _ in 0..30 {
            p.reframe(|w| zoom(w, 10.0, 0.75));
        }
        let d = p.deep.clone().expect("deep after 30 zooms");
        assert!((d.half_h - 1.5e-30).abs() < 1e-40);
        // Zooming about the point at u = 0.75 keeps it fixed: the centre
        // tends to 0.5 (within the doubles of the early, shallow zooms),
        // while the deep view keeps digits far below a double's.
        assert!((d.re.to_f64() - 0.5).abs() < 1e-15);
        let before = d.re.clone();
        // A pan by a tenth of the width moves the precise centre by that.
        p.reframe(|w| {
            let d = (w.x_max - w.x_min) / 10.0;
            Window::new(w.x_min + d, w.x_max + d, w.y_min, w.y_max)
        });
        let moved = p.deep.as_ref().unwrap().re.sub(&before).to_f64();
        assert!(
            (moved / (2.0 * d.half_w / 10.0) - 1.0).abs() < 1e-14,
            "{moved}"
        );
        for _ in 0..30 {
            p.reframe(|w| zoom(w, 0.1, 0.5));
        }
        assert!(p.deep.is_none());
        assert!((p.window.y_max - 1.5).abs() < 1e-6, "{:?}", p.window);
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
