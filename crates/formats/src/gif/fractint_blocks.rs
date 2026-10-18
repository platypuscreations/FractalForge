// SPDX-License-Identifier: GPL-3.0-or-later

//! Fractint-format application-extension blocks.
//!
//! Implemented only from `docs/specs/gif-fractint-blocks.md` (cited as §n),
//! which was derived by black-box analysis of saved files. Never from Fractint
//! source.

use fractalforge_core::color::wrap;
use fractalforge_core::{
    FractalType, Inside, IterBuffer, Outside, Params, Symmetry, Window, types,
};
use fractalforge_render::PassMode;

use super::container::{AppExtension, GifImage, Placement};

pub const IDENTIFIER: &[u8; 8] = b"fractint";
/// Main parameter block (§2.1).
pub const MAIN_AUTH: &[u8; 3] = b"001";
/// Resume data for an unfinished image (§4).
pub const RESUME_AUTH: &[u8; 3] = b"002";
/// Arbitrary-precision coordinates (§2.4).
pub const DEEP_AUTH: &[u8; 3] = b"005";
/// Entry name of a formula, L-system or IFS (frm-syntax.md §8.1).
pub const NAME_AUTH: &[u8; 3] = b"003";

const MAIN_LEN: usize = 504;
const RESUME_LEN: usize = 392;

/// The main block of a plain finished Mandelbrot save (§ Appendix A), with
/// the per-render seed (98–99) and time (214–217) zeroed. Writers start from
/// it so the bytes we don't understand keep their usual values (§5.3).
#[rustfmt::skip]
const BASELINE: [u8; MAIN_LEN] = [
    0x46, 0x72, 0x61, 0x63, 0x74, 0x61, 0x6c, 0x00, 0x96, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x04, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x3f, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0xf8, 0xbf, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x3f, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x13, 0x00, 0x20, 0x03, 0x58, 0x02, 0x00, 0x01, 0x11, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x90, 0x01, 0xe7, 0x03, 0x00, 0x00,
    0x3c, 0x00, 0x1e, 0x00, 0x00, 0x00, 0x5a, 0x00, 0x5a, 0x00, 0x1e, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x14, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x50, 0x00, 0x64, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xc0,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0xbf, 0x67, 0x00, 0x04, 0x00, 0x09, 0x02, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x02, 0x03, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x3b, 0x01, 0x00, 0xd4, 0x07, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0xff, 0x00, 0x47, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
    0x8f, 0xc2, 0xf5, 0x28, 0x5c, 0x8f, 0xea, 0xbf, 0x8f, 0xc2, 0xf5, 0x28, 0x5c, 0x8f, 0xea, 0xbf,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd0, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd0, 0xbf,
    0x80, 0x00, 0x00, 0x00, 0x00, 0x41, 0x00, 0x00, 0x00, 0x41, 0x00, 0x00, 0xe0, 0x40, 0x00, 0x00,
    0x20, 0x41, 0x00, 0x00, 0xc0, 0x41, 0x00, 0x00, 0x20, 0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x96, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7b, 0x14, 0xae, 0x47, 0xe1, 0x7a,
    0x84, 0x3f, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x9a, 0x99, 0x99, 0x99, 0x99, 0x99,
    0xa9, 0x3f, 0x9a, 0x99, 0x99, 0x99, 0x99, 0x99, 0xa9, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Parameters read from Fractint-format blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct FractintInfo {
    pub type_number: i16,
    /// `None` when the type isn't supported yet.
    pub params: Option<Params>,
    pub pass_mode: PassMode,
    /// False when the image was saved before the calculation finished.
    pub finished: bool,
    /// Settings in the file that FractalForge ignored or approximated.
    pub warnings: Vec<String>,
    /// For formula images: the entry name. The caller must find the formula
    /// (the file name isn't stored) and attach it to `params`, which are
    /// returned unvalidated until then.
    pub formula_name: Option<String>,
}

/// Reads the Fractint-format blocks of an image, if it has a main block.
pub fn read(image: &GifImage) -> Option<FractintInfo> {
    let main = image.extension(IDENTIFIER, MAIN_AUTH)?.data();
    let b = Bytes(&main);
    let mut warnings = Vec::new();
    if b.0.get(..8) != Some(b"Fractal\0") {
        warnings.push("Fractint parameter block has an unexpected signature".into());
    }
    let type_number = b.i16(10)?;
    // Byte 76 gates the later fields (§2.1, §5.3).
    let modern = b.i16(76).is_some_and(|v| v >= 17) && main.len() >= MAIN_LEN;
    if !modern {
        warnings.push("old Fractint parameter block: only the basic fields were read".into());
    }

    let pass_mode = match b.u8(200).filter(|_| modern) {
        Some(b'1') => PassMode::One,
        Some(b'2') => PassMode::Two,
        Some(b'g') | None => PassMode::Guess,
        Some(c) => {
            warnings.push(format!(
                "pass mode '{}' isn't supported; using guessing",
                c as char
            ));
            PassMode::Guess
        }
    };
    let finished = image.extension(IDENTIFIER, RESUME_AUTH).is_none();

    let params = match types::by_fractint_number(type_number) {
        None => {
            warnings.push(format!(
                "Fractint type number {type_number} isn't supported yet"
            ));
            None
        }
        Some(fractal) => {
            let mut p = Params::new(fractal);
            read_params(&b, modern, &mut p, &mut warnings);
            if image.extension(IDENTIFIER, DEEP_AUTH).is_some() {
                warnings.push("deep-zoom coordinates were rounded to double precision".into());
            }
            if fractal.needs_program {
                Some(p)
            } else {
                match p.validate() {
                    Ok(()) => Some(p),
                    Err(e) => {
                        warnings.push(format!("unusable parameters: {e}"));
                        None
                    }
                }
            }
        }
    };
    let formula_name = image.extension(IDENTIFIER, NAME_AUTH).and_then(|e| {
        let d = e.data();
        let name = &d[..d.len().min(40)];
        let end = name.iter().position(|&c| c == 0).unwrap_or(name.len());
        (end > 0).then(|| String::from_utf8_lossy(&name[..end]).into_owned())
    });

    Some(FractintInfo {
        type_number,
        params,
        pass_mode,
        finished,
        warnings,
        formula_name,
    })
}

fn read_params(b: &Bytes, modern: bool, p: &mut Params, warnings: &mut Vec<String>) {
    let f = |off| b.f64(off).unwrap_or(0.0);
    p.window = Window::new(f(12), f(20), f(28), f(36));
    if modern {
        let (x3, y3) = (f(184), f(192));
        if x3 != p.window.x_min || y3 != p.window.y_min {
            warnings.push(
                "rotated or skewed view (third corner) isn't supported; using the corners".into(),
            );
        }
    }
    for (v, off) in p.params.iter_mut().zip(PARAM_OFFSETS) {
        *v = if modern || off < 354 { f(off) } else { 0.0 };
    }

    // 408 is authoritative; 8 is a clamped 16-bit copy (§2.1 notes).
    let max_iter = b
        .i32(408)
        .filter(|&v| modern && v > 0)
        .or(b.i16(8).map(i32::from));
    p.max_iter = max_iter.filter(|&v| v > 0).map_or(150, |v| v as u32);

    if !modern {
        return;
    }
    // 3D viewing (gif-fractint-blocks.md §2.1).
    let i = |off| b.i16(off).unwrap_or(0);
    p.view3d = fractalforge_core::View3d {
        rotation: [i(128), i(130), i(132)],
        perspective: i(144),
        xyshift: [i(146), i(148)],
    };
    // fn1–fn4 bindings (gif-fractint-blocks.md §6.2).
    for (i, f) in p.functions.iter_mut().enumerate() {
        if let Some(v) = b.u8(218 + i) {
            *f = v;
        }
    }
    // Biomorph and decomposition come first: biomorph changes the default
    // bailout (coloring.md §8).
    if let Some(b) = b.i16(102).filter(|&v| v >= 0) {
        p.biomorph = Some(b as u32);
    }
    p.decomp = b.u16(120).unwrap_or(0) as u32;

    // 0 means the default bailout (§5.2).
    let bailout = b
        .i32(402)
        .filter(|&v| v > 0)
        .or(b.i16(212).filter(|&v| v > 0).map(i32::from));
    p.bailout = bailout.map_or_else(|| p.default_bailout(), |v| v as f64);
    // 999 = automatic, 0 = none; other forced values aren't supported.
    match b.i16(124).unwrap_or(999) {
        999 => {}
        0 => p.use_symmetry = false,
        v => match symmetry_from_code(v) {
            Some(s) => p.forced_symmetry = Some(s),
            None => warnings.push(format!(
                "forced symmetry setting {v} isn't supported; using automatic"
            )),
        },
    }
    if b.i16(406).unwrap_or(0) != 0 {
        warnings.push("bailout tests other than the default modulus test are ignored".into());
    }

    p.inside = match b.i16(104).unwrap_or(0) {
        n if n >= 0 => Inside::Color(wrap(n as u32) as u8),
        -102 => {
            warnings.push("inside=period isn't supported; using the maxiter colour".into());
            Inside::MaxIter
        }
        n => INSIDE_CODES.iter().find(|(c, _)| *c == n).map_or_else(
            || {
                warnings.push(format!(
                    "inside colouring {n} isn't supported; using colour 0"
                ));
                Inside::Color(0)
            },
            |(_, m)| *m,
        ),
    };
    p.outside = match b.i16(182).unwrap_or(-1) {
        n @ 0..=255 => Outside::Color(n as u8),
        n => OUTSIDE_CODES.iter().find(|(c, _)| *c == n).map_or_else(
            || {
                warnings.push(format!(
                    "outside colouring {n} isn't supported; colouring by iteration"
                ));
                Outside::Iter
            },
            |(_, m)| *m,
        ),
    };
    if p.needs_orbit_coloring() && !p.fractal.orbit_coloring {
        warnings.push(format!(
            "inside/outside/decomposition/biomorph options aren't supported for {} yet and were ignored",
            p.fractal.name
        ));
        p.inside = match p.inside {
            Inside::Color(c) => Inside::Color(c),
            _ => Inside::MaxIter,
        };
        p.outside = Outside::Iter;
        p.decomp = 0;
        p.biomorph = None;
        p.bailout = bailout.map_or(p.fractal.default_bailout, |v| v as f64);
    }

    let unsupported = [
        (b.i32(420).unwrap_or(0) != 0, "logarithmic palette"),
        (b.i32(424).unwrap_or(0) != 0, "distance estimator"),
        (b.f64(428).unwrap_or(0.0) != 0.0, "inversion"),
        (b.i16(222).unwrap_or(0) != 0, "finite attractor"),
        (b.f32(86).unwrap_or(0.0) != 0.0, "continuous potential"),
        (b.i16(180).unwrap_or(0) != 0, "stereogram"),
    ];
    for (_, what) in unsupported.iter().filter(|(on, _)| *on) {
        warnings.push(format!("{what} isn't supported yet and was ignored"));
    }
}

/// Pixels of an unfinished Fractint-format image that are certainly final:
/// everything outside the resume block's work items, and inside them the
/// final pixels and exact block origins of fractint-resume.md §4. Returns
/// those pixels' colour indices, with every other pixel unknown; `None` when
/// the image is finished or carries no usable resume data.
pub fn partial_buffer(image: &GifImage, fractal: &FractalType) -> Option<IterBuffer> {
    // Plotted types save their own orbit state instead of a work list
    // (orbit-maps.md §11); FractalForge calculates them again.
    if fractal.plot.is_some() {
        return None;
    }
    let main = image.extension(IDENTIFIER, MAIN_AUTH)?.data();
    let resume = image.extension(IDENTIFIER, RESUME_AUTH)?.data();
    let b = Bytes(&main);
    // Fractint resumes only with status 2 and a resume block (§7).
    if b.i16(202) != Some(2) {
        return None;
    }
    let (w, h) = (image.width as i64, image.height as i64);
    let r = Bytes(&resume);
    let count = r.i32(4)?.clamp(0, 12) as usize;
    let mode = b.u8(200)?;
    // Guessing pass count, hypothesis for unseen sizes (§3.3).
    let guess_passes = (64 - ((w / 50).max(1) as u64).leading_zeros() as i64 - 1).max(1);

    let mut known = vec![true; (w * h) as usize];
    for k in 0..count {
        let at = 8 + 32 * k;
        let v = |i: usize| r.i32(at + 4 * i).unwrap_or(0) as i64;
        let (left, right) = (v(0).clamp(0, w - 1), v(1).clamp(0, w - 1));
        let (top, bottom) = (v(2).clamp(0, h - 1), v(3).clamp(0, h - 1));
        let (row, sym, pass, col) = (v(4), v(5), v(6), v(7));
        if row < 0 {
            // Fractint recomputes everything (§7).
            return Some(IterBuffer::new(w as u32, h as u32));
        }
        let mirrored = sym == 17 || sym == 51;
        let last = if mirrored {
            top + (bottom - top) / 2
        } else {
            bottom
        };
        let xy = sym == 51 && fractal.symmetry_kind_is_xy();
        let origin = sym == 51 && !xy;
        let last_col = if xy { left + (right - left) / 2 } else { right };
        for y in top..=bottom {
            for x in left..=right {
                // State of the computed partner of a copied pixel (§2.2).
                let (mut ry, mut rx) = (y, x);
                if y > last {
                    ry = top + bottom - y;
                    if origin {
                        rx = left + right - x;
                    }
                }
                if x > last_col {
                    rx = left + right - x;
                }
                known[(y * w + x) as usize] =
                    inner_known(mode, guess_passes, row, col, pass, ry, rx);
            }
        }
    }
    let data = known
        .iter()
        .zip(&image.pixels)
        .map(|(&k, &px)| {
            if k {
                px as u32
            } else {
                fractalforge_core::UNKNOWN
            }
        })
        .collect();
    IterBuffer::from_data(w as u32, h as u32, data)
}

/// Whether pixel (r, c) of a work item is final or an exact block origin
/// (§4.1). Unanalysed modes keep nothing (§4.3).
fn inner_known(mode: u8, n: i64, row: i64, col: i64, pass: i64, r: i64, c: i64) -> bool {
    let before = r < row || (r == row && c < col);
    match (mode, pass) {
        (b'1', _) => before,
        (b'2', 0) => {
            let block_before = (r - r % 2) < row || ((r - r % 2) == row && (c - c % 2) < col);
            block_before && r % 2 == 0 && c % 2 == 0
        }
        (b'2', _) => before || (r % 2 == 0 && c % 2 == 0),
        (b'g', p) if p < n => {
            let b = 1i64 << (n - 1 - p);
            let band = 2 * b;
            if r < row {
                b == 1 || (r % b == 0 && c % b == 0)
            } else if r == row {
                c % band == 0
            } else if r >= row + band && p > 0 {
                r % band == 0 && c % band == 0
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Offsets of params 1–10 in the main block (gif-fractint-blocks.md §2.1).
const PARAM_OFFSETS: [usize; 10] = [44, 52, 270, 278, 354, 362, 370, 378, 386, 394];

/// Forced symmetry codes at offset 124: 1 x-axis and 4 origin (par.md
/// §5.1), 2 y-axis (frm-syntax.md §8.1), 3 xy-axis (hypothesis).
fn symmetry_from_code(code: i16) -> Option<Symmetry> {
    Some(match code {
        1 => Symmetry::XAxis,
        2 => Symmetry::YAxis,
        3 => Symmetry::XY,
        4 => Symmetry::Origin,
        _ => return None,
    })
}

fn symmetry_code(s: Symmetry) -> i16 {
    match s {
        Symmetry::None => 0,
        Symmetry::XAxis => 1,
        Symmetry::YAxis => 2,
        Symmetry::XY => 3,
        Symmetry::Origin => 4,
        // No Fractint code was observed for forcing PI_SYM.
        Symmetry::Pi => 999,
    }
}

/// Inside colouring codes at offset 104 (coloring.md §6; n ≥ 0 = colour n).
const INSIDE_CODES: [(i16, Inside); 7] = [
    (-1, Inside::MaxIter),
    (-59, Inside::ZMag),
    (-60, Inside::Bof60),
    (-61, Inside::Bof61),
    (-104, Inside::Atan),
    (-103, Inside::Fmod),
    (-100, Inside::EpsCross),
];

/// Outside colouring codes at offset 182 (coloring.md §5; n ≥ 0 = colour n).
const OUTSIDE_CODES: [(i16, Outside); 8] = [
    (-1, Outside::Iter),
    (-2, Outside::Real),
    (-3, Outside::Imag),
    (-4, Outside::Mult),
    (-5, Outside::Summ),
    (-6, Outside::Atan),
    (-7, Outside::Fmod),
    (-8, Outside::Tdis),
];

/// The inside mode for a Fractint code (negative), if supported.
pub(crate) fn inside_from_code(code: i16) -> Option<Inside> {
    INSIDE_CODES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, m)| *m)
}

/// The outside mode for a Fractint code (negative), if supported.
pub(crate) fn outside_from_code(code: i16) -> Option<Outside> {
    OUTSIDE_CODES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, m)| *m)
}

fn inside_code(m: Inside) -> i16 {
    match m {
        Inside::Color(c) => c.into(),
        m => INSIDE_CODES.iter().find(|(_, x)| *x == m).unwrap().0,
    }
}

fn outside_code(m: Outside) -> i16 {
    match m {
        Outside::Color(c) => c.into(),
        m => OUTSIDE_CODES.iter().find(|(_, x)| *x == m).unwrap().0,
    }
}

/// Builds the Fractint-format blocks for an image: a resume block when the
/// render is unfinished, then the main block (always last, §1.2).
///
/// Returns the blocks and notes about anything that couldn't be expressed.
///
/// `resume_at` is `None` for a finished image, otherwise the first pixel
/// (row, column) in row order that isn't final: every pixel before it must
/// be final.
pub fn write(
    params: &Params,
    pass_mode: PassMode,
    resume_at: Option<(u32, u32)>,
    width: u32,
    height: u32,
    elapsed_us: u64,
) -> Option<(Vec<AppExtension>, Vec<String>)> {
    let type_number = params.fractal.fractint_number?;
    let mut notes = Vec::new();
    let mut m = BASELINE;
    let w = &params.window;

    put(&mut m, 10, &type_number.to_le_bytes());
    for (off, v) in [
        (12, w.x_min),
        (20, w.x_max),
        (28, w.y_min),
        (36, w.y_max),
        (184, w.x_min),
        (192, w.y_min),
    ] {
        put(&mut m, off, &v.to_le_bytes());
    }
    for (off, v) in PARAM_OFFSETS.into_iter().zip(params.params) {
        put(&mut m, off, &v.to_le_bytes());
    }
    put(&mut m, 218, &params.functions);
    let v = &params.view3d;
    for (off, x) in [
        (128, v.rotation[0]),
        (130, v.rotation[1]),
        (132, v.rotation[2]),
        (144, v.perspective),
        (146, v.xyshift[0]),
        (148, v.xyshift[1]),
    ] {
        put(&mut m, off, &x.to_le_bytes());
    }
    // A drawn 3D IFS entry (ifs.md §2.3).
    if *params.fractal == types::IFS
        && params
            .program
            .as_ref()
            .is_some_and(fractalforge_core::ifs::is_3d)
    {
        put(&mut m, 252, &(-1i16).to_le_bytes());
    }

    let max_iter = params.max_iter.min(i32::MAX as u32) as i32;
    put(&mut m, 408, &max_iter.to_le_bytes());
    put(&mut m, 8, &(max_iter.min(32767) as i16).to_le_bytes());

    // Fractint stores an integer bailout; 0 means the type's default.
    let bailout = if params.bailout == params.default_bailout() {
        0
    } else {
        params.bailout.round().clamp(1.0, i32::MAX as f64) as i32
    };
    if bailout != 0 && bailout as f64 != params.bailout {
        notes.push(format!(
            "Fractint stores whole-number bailouts; wrote {bailout} for {}",
            params.bailout
        ));
    }
    put(&mut m, 402, &bailout.to_le_bytes());
    let bailout16 = if bailout <= 32767 { bailout as i16 } else { 0 };
    put(&mut m, 212, &bailout16.to_le_bytes());

    put(&mut m, 104, &inside_code(params.inside).to_le_bytes());
    let outside = outside_code(params.outside);
    put(&mut m, 182, &outside.to_le_bytes());
    let symmetry: i16 = match (params.use_symmetry, params.forced_symmetry) {
        (false, _) => 0,
        (true, Some(s)) => symmetry_code(s),
        (true, None) => 999,
    };
    let biomorph = params
        .biomorph
        .map_or(-1, |b| b.min(i16::MAX as u32) as i16);
    put(&mut m, 102, &biomorph.to_le_bytes());
    let decomp = params.decomp.min(u16::MAX as u32) as u16;
    if decomp as u32 != params.decomp {
        notes.push(format!(
            "Fractint stores decomposition up to {}; wrote that",
            u16::MAX
        ));
    }
    put(&mut m, 120, &decomp.to_le_bytes());
    put(&mut m, 124, &symmetry.to_le_bytes());

    // Screen size (hypothesis, §2.1 offsets 70/72).
    put(&mut m, 70, &(width.min(32767) as i16).to_le_bytes());
    put(&mut m, 72, &(height.min(32767) as i16).to_le_bytes());

    let time = (elapsed_us / 10_000).min(u32::MAX as u64) as u32;
    put(&mut m, 214, &time.to_le_bytes());

    let mut blocks = Vec::new();
    let pass = if resume_at.is_some() && params.fractal.plot.is_some() {
        // Plotted types: interrupted, with no resume data (ifs.md §5).
        put(&mut m, 202, &3i16.to_le_bytes());
        notes.push("Fractint recalculates this unfinished image from the start".into());
        b'g'
    } else if resume_at.is_none() {
        put(&mut m, 202, &4i16.to_le_bytes());
        match pass_mode {
            PassMode::One => b'1',
            PassMode::Two => b'2',
            PassMode::Guess => b'g',
        }
    } else {
        // A one-pass work item from the first unfinished pixel, without
        // mirroring (fractint-resume.md §6.2, §6.5): Fractint keeps every
        // pixel before it and computes the rest, whatever our pass mode was.
        let (row, col) = resume_at.unwrap_or((0, 0));
        put(&mut m, 202, &2i16.to_le_bytes());
        if pass_mode != PassMode::One {
            notes.push("Fractint continues this unfinished image in one-pass mode".into());
        }
        let mut r = [0u8; RESUME_LEN];
        for (off, v) in [
            (0, 2),
            (4, 1),
            (8, 0),
            (12, width as i32 - 1),
            (16, row as i32),
            (20, height as i32 - 1),
            (24, row as i32),
            (28, 16),
            (32, 0),
            (36, col as i32),
        ] {
            put(&mut r, off, &v.to_le_bytes());
        }
        blocks.push(AppExtension::new(
            *IDENTIFIER,
            *RESUME_AUTH,
            &r,
            Placement::AfterImage,
        ));
        b'1'
    };
    m[200] = pass;

    if let Some(prog) = &params.program {
        blocks.push(if *params.fractal == types::FORMULA {
            name_block(prog)
        } else {
            entry_name_block(prog)
        });
    }
    blocks.push(AppExtension::new(
        *IDENTIFIER,
        *MAIN_AUTH,
        &m,
        Placement::AfterImage,
    ));
    // Total length of the Fractint application extensions, framing included.
    let total: usize = blocks.iter().map(framed_len).sum();
    let main = blocks.last_mut().unwrap();
    let mut data = main.data();
    put(&mut data, 204, &(total as u16).to_le_bytes());
    *main = AppExtension::new(*IDENTIFIER, *MAIN_AUTH, &data, Placement::AfterImage);

    Some((blocks, notes))
}

/// The `fractint003` block of a formula image (frm-syntax.md §8.1): the
/// entry name, flags for which of p1–p5 and `ismand` it mentions, and the
/// `ismand` setting (1).
fn name_block(prog: &fractalforge_core::Program) -> AppExtension {
    let mut d = [0u8; 66];
    let name = prog.0.name().as_bytes();
    let n = name.len().min(39);
    d[..n].copy_from_slice(&name[..n]);
    for (off, var) in [
        (40, "p1"),
        (42, "p2"),
        (44, "p3"),
        (46, "ismand"),
        (50, "p4"),
        (52, "p5"),
    ] {
        put(&mut d, off, &i16::from(prog.0.mentions(var)).to_le_bytes());
    }
    put(&mut d, 48, &1i16.to_le_bytes());
    AppExtension::new(*IDENTIFIER, *NAME_AUTH, &d, Placement::AfterImage)
}

/// The `fractint003` block of an IFS or L-system image: just the entry
/// name, NUL-padded (ifs.md §2.3).
fn entry_name_block(prog: &fractalforge_core::Program) -> AppExtension {
    let mut d = [0u8; 66];
    let name = prog.0.name().as_bytes();
    let n = name.len().min(47);
    d[..n].copy_from_slice(&name[..n]);
    AppExtension::new(*IDENTIFIER, *NAME_AUTH, &d, Placement::AfterImage)
}

/// Bytes an extension occupies in the file: `21 FF 0B`, 11 header bytes,
/// each sub-block with its length byte, and the terminator.
fn framed_len(e: &AppExtension) -> usize {
    3 + 11 + e.sub_blocks.iter().map(|b| b.len() + 1).sum::<usize>() + 1
}

fn put(buf: &mut [u8], off: usize, bytes: &[u8]) {
    buf[off..off + bytes.len()].copy_from_slice(bytes);
}

struct Bytes<'a>(&'a [u8]);

impl Bytes<'_> {
    fn get<const N: usize>(&self, off: usize) -> Option<[u8; N]> {
        self.0.get(off..off + N).map(|s| s.try_into().unwrap())
    }
    fn u8(&self, off: usize) -> Option<u8> {
        self.0.get(off).copied()
    }
    fn i16(&self, off: usize) -> Option<i16> {
        self.get(off).map(i16::from_le_bytes)
    }
    fn u16(&self, off: usize) -> Option<u16> {
        self.get(off).map(u16::from_le_bytes)
    }
    fn i32(&self, off: usize) -> Option<i32> {
        self.get(off).map(i32::from_le_bytes)
    }
    fn f32(&self, off: usize) -> Option<f32> {
        self.get(off).map(f32::from_le_bytes)
    }
    fn f64(&self, off: usize) -> Option<f64> {
        self.get(off).map(f64::from_le_bytes)
    }
}
