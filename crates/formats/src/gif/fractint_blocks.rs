// SPDX-License-Identifier: GPL-3.0-or-later

//! Fractint-format application-extension blocks.
//!
//! Implemented only from `docs/specs/gif-fractint-blocks.md` (cited as §n),
//! which was derived by black-box analysis of saved files. Never from Fractint
//! source.

use fractalforge_core::{Inside, Outside, Params, Window, types};
use fractalforge_render::PassMode;

use super::container::{AppExtension, GifImage, Placement};

pub const IDENTIFIER: &[u8; 8] = b"fractint";
/// Main parameter block (§2.1).
pub const MAIN_AUTH: &[u8; 3] = b"001";
/// Resume data for an unfinished image (§4).
pub const RESUME_AUTH: &[u8; 3] = b"002";
/// Arbitrary-precision coordinates (§2.4).
pub const DEEP_AUTH: &[u8; 3] = b"005";

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
            match p.validate() {
                Ok(()) => Some(p),
                Err(e) => {
                    warnings.push(format!("unusable parameters: {e}"));
                    None
                }
            }
        }
    };

    Some(FractintInfo {
        type_number,
        params,
        pass_mode,
        finished,
        warnings,
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
    p.params = [f(44), f(52), f(270), f(278)];
    if modern && (354..402).step_by(8).any(|off| f(off) != 0.0) {
        warnings.push("parameters 5-10 are ignored".into());
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
    // 0 means the type's default (§5.2).
    let bailout = b
        .i32(402)
        .filter(|&v| v > 0)
        .or(b.i16(212).filter(|&v| v > 0).map(i32::from));
    if let Some(v) = bailout {
        p.bailout = v as f64;
    }
    // 999 = automatic, 0 = none; other forced values aren't supported.
    match b.i16(124).unwrap_or(999) {
        999 => {}
        0 => p.use_symmetry = false,
        v => warnings.push(format!(
            "forced symmetry setting {v} isn't supported; using automatic"
        )),
    }
    if b.i16(406).unwrap_or(0) != 0 {
        warnings.push("bailout tests other than the default modulus test are ignored".into());
    }

    p.inside = match b.i16(104).unwrap_or(0) {
        n @ 0..=255 => Inside::Color(n as u8),
        n if n > 255 => Inside::Color((n % 256) as u8),
        -1 => Inside::MaxIter,
        n => {
            warnings.push(format!(
                "inside colouring {n} isn't supported; using colour 0"
            ));
            Inside::Color(0)
        }
    };
    p.outside = match b.i16(182).unwrap_or(-1) {
        -1 => Outside::Iter,
        n => {
            warnings.push(format!(
                "outside colouring {n} isn't supported; colouring by iteration"
            ));
            Outside::Iter
        }
    };

    let unsupported = [
        (b.i16(102).unwrap_or(-1) != -1, "biomorph"),
        (b.u16(120).unwrap_or(0) != 0, "decomposition"),
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

/// Builds the Fractint-format blocks for an image: a resume block when the
/// render is unfinished, then the main block (always last, §1.2).
///
/// Returns the blocks and notes about anything that couldn't be expressed.
pub fn write(
    params: &Params,
    pass_mode: PassMode,
    finished: bool,
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
    for (off, v) in [44, 52, 270, 278].into_iter().zip(params.params) {
        put(&mut m, off, &v.to_le_bytes());
    }

    let max_iter = params.max_iter.min(i32::MAX as u32) as i32;
    put(&mut m, 408, &max_iter.to_le_bytes());
    put(&mut m, 8, &(max_iter.min(32767) as i16).to_le_bytes());

    // Fractint stores an integer bailout; 0 means the type's default.
    let bailout = if params.bailout == params.fractal.default_bailout {
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

    let inside: i16 = match params.inside {
        Inside::Color(c) => c.into(),
        Inside::MaxIter => -1,
    };
    put(&mut m, 104, &inside.to_le_bytes());
    let outside: i16 = match params.outside {
        Outside::Iter => -1,
    };
    put(&mut m, 182, &outside.to_le_bytes());
    let symmetry: i16 = if params.use_symmetry { 999 } else { 0 };
    put(&mut m, 124, &symmetry.to_le_bytes());

    // Screen size (hypothesis, §2.1 offsets 70/72).
    put(&mut m, 70, &(width.min(32767) as i16).to_le_bytes());
    put(&mut m, 72, &(height.min(32767) as i16).to_le_bytes());

    let time = (elapsed_us / 10_000).min(u32::MAX as u64) as u32;
    put(&mut m, 214, &time.to_le_bytes());

    let mut blocks = Vec::new();
    let pass = if finished {
        put(&mut m, 202, &4i16.to_le_bytes());
        match pass_mode {
            PassMode::One => b'1',
            PassMode::Two => b'2',
            PassMode::Guess => b'g',
        }
    } else {
        // Ask Fractint to calculate the image from the start (§5.4): the
        // resume semantics of our pass modes don't map onto Fractint's yet.
        put(&mut m, 202, &2i16.to_le_bytes());
        notes.push("Fractint will recalculate this unfinished image from the start".into());
        let mut r = [0u8; RESUME_LEN];
        for (off, v) in [
            (0, 2),
            (4, 1),
            (8, 0),
            (12, width as i32 - 1),
            (16, 0),
            (20, height as i32 - 1),
            (24, 0),
            (28, 16),
            (32, 0),
            (36, 0),
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
