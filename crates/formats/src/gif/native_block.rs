// SPDX-License-Identifier: GPL-3.0-or-later

//! FractalForge's own GIF application-extension block, holding a complete
//! render checkpoint. See `docs/specs/gif-native-block.md`.

use fractalforge_core::{
    Inside, IterBuffer, Outside, PARAM_COUNT, Palette, Params, Program, Symmetry, View3d, Window,
    types,
};
use fractalforge_formula::{Formula, parser};
use fractalforge_render::{PassMode, RenderState};

use super::container::{AppExtension, Placement};
use crate::Error;

/// Application identifier (8 bytes). Part of the file format; never change it.
pub const APP_IDENTIFIER: &[u8; 8] = b"FRCTFRGE";

/// Application authentication code (3 bytes). Part of the file format.
pub const AUTH_CODE: &[u8; 3] = b"1.0";

/// Payload layout version written by this code. Versions 2 (v0.1.0) and
/// 3 are still read; version 1 (pre-release) stored escape counts rather
/// than colour values and isn't.
pub const PAYLOAD_VERSION: u16 = 4;

const COMPRESSION_DEFLATE: u8 = 1;
/// Refuse to inflate bodies larger than this (a 16k × 16k buffer is 1 GiB).
const MAX_BODY: usize = 1 << 31;

/// A complete, resumable render.
#[derive(Clone, Debug, PartialEq)]
pub struct Checkpoint {
    pub params: Params,
    pub state: RenderState,
    pub buffer: IterBuffer,
    pub palette: Palette,
}

pub fn encode(c: &Checkpoint) -> AppExtension {
    let mut body = Vec::with_capacity(c.buffer.data().len() * 4 + 1024);
    let p = &c.params;
    let name = p.fractal.name.as_bytes();
    body.push(name.len() as u8);
    body.extend_from_slice(name);
    let w = &p.window;
    for v in [w.x_min, w.x_max, w.y_min, w.y_max] {
        body.extend_from_slice(&v.to_le_bytes());
    }
    for v in p.params {
        body.extend_from_slice(&v.to_le_bytes());
    }
    body.extend_from_slice(&p.max_iter.to_le_bytes());
    body.extend_from_slice(&p.bailout.to_le_bytes());
    // Modes are stored as their position in Inside::ALL / Outside::ALL,
    // followed by the palette index for the fixed-colour modes.
    let inside_tag = Inside::ALL
        .iter()
        .position(|m| same_mode_inside(*m, p.inside))
        .unwrap();
    let inside_color = if let Inside::Color(c) = p.inside {
        c
    } else {
        0
    };
    body.extend_from_slice(&[inside_tag as u8, inside_color]);
    let outside_tag = Outside::ALL
        .iter()
        .position(|m| same_mode_outside(*m, p.outside))
        .unwrap();
    let outside_color = if let Outside::Color(c) = p.outside {
        c
    } else {
        0
    };
    body.extend_from_slice(&[outside_tag as u8, outside_color]);
    body.extend_from_slice(&p.decomp.to_le_bytes());
    body.extend_from_slice(&p.biomorph.map_or(-1i64, i64::from).to_le_bytes());
    body.push(p.use_symmetry as u8);
    body.push(p.forced_symmetry.map_or(0, symmetry_tag));
    body.extend_from_slice(&p.functions);
    let (name, source) = p
        .program
        .as_ref()
        .map_or(("", ""), |prog| (prog.0.name(), prog.0.source()));
    for text in [name, source] {
        body.extend_from_slice(&(text.len() as u32).to_le_bytes());
        body.extend_from_slice(text.as_bytes());
    }
    let v = &p.view3d;
    for x in v.rotation.iter().chain([&v.perspective]).chain(&v.xyshift) {
        body.extend_from_slice(&x.to_le_bytes());
    }

    body.extend_from_slice(&c.buffer.width().to_le_bytes());
    body.extend_from_slice(&c.buffer.height().to_le_bytes());

    let s = &c.state;
    body.push(s.pass_mode.tag());
    body.extend_from_slice(&s.coarsest_step.to_le_bytes());
    body.extend_from_slice(&s.stage.to_le_bytes());
    body.extend_from_slice(&s.row.to_le_bytes());
    body.extend_from_slice(&s.elapsed_us.to_le_bytes());

    for rgb in &c.palette.0 {
        body.extend_from_slice(rgb);
    }
    for v in c.buffer.data() {
        body.extend_from_slice(&v.to_le_bytes());
    }

    let mut payload = Vec::new();
    payload.extend_from_slice(&PAYLOAD_VERSION.to_le_bytes());
    payload.push(COMPRESSION_DEFLATE);
    payload.extend_from_slice(&(body.len() as u64).to_le_bytes());
    payload.extend_from_slice(&miniz_oxide::deflate::compress_to_vec(&body, 6));
    AppExtension::new(*APP_IDENTIFIER, *AUTH_CODE, &payload, Placement::AfterImage)
}

pub fn decode(ext: &AppExtension) -> Result<Checkpoint, Error> {
    let payload = ext.data();
    let mut r = Cursor(&payload);
    let version = r.u16()?;
    if !(2..=PAYLOAD_VERSION).contains(&version) {
        return Err(bad(format!("unsupported payload version {version}")));
    }
    if r.u8()? != COMPRESSION_DEFLATE {
        return Err(bad("unknown compression".into()));
    }
    let len = usize::try_from(r.u64()?).ok().filter(|&n| n <= MAX_BODY);
    let len = len.ok_or_else(|| bad("body too large".into()))?;
    let body = miniz_oxide::inflate::decompress_to_vec_with_limit(r.0, len)
        .map_err(|e| bad(format!("decompression failed: {e:?}")))?;
    if body.len() != len {
        return Err(bad("body length mismatch".into()));
    }

    let mut r = Cursor(&body);
    let name_len = r.u8()? as usize;
    let name =
        std::str::from_utf8(r.take(name_len)?).map_err(|_| bad("type name is not UTF-8".into()))?;
    let fractal =
        types::lookup(name).ok_or_else(|| bad(format!("unknown fractal type {name:?}")))?;
    let window = Window::new(r.f64()?, r.f64()?, r.f64()?, r.f64()?);
    let mut params = [0.0; PARAM_COUNT];
    let stored = if version >= 3 { PARAM_COUNT } else { 4 };
    for v in params.iter_mut().take(stored) {
        *v = r.f64()?;
    }
    let max_iter = r.u32()?;
    let bailout = r.f64()?;
    let inside = match (r.u8()?, r.u8()?) {
        (0, c) => Inside::Color(c),
        (t, _) => *Inside::ALL
            .get(t as usize)
            .ok_or_else(|| bad(format!("unknown inside mode {t}")))?,
    };
    let outside = match (r.u8()?, r.u8()?) {
        (1, c) => Outside::Color(c),
        (t, _) => *Outside::ALL
            .get(t as usize)
            .ok_or_else(|| bad(format!("unknown outside mode {t}")))?,
    };
    let decomp = r.u32()?;
    let biomorph = u32::try_from(r.u64()? as i64).ok();
    let use_symmetry = r.u8()? != 0;
    let mut forced_symmetry = None;
    let mut functions = [0, 6, 2, 3];
    let mut program = None;
    if version >= 3 {
        forced_symmetry = symmetry_from_tag(r.u8()?);
        functions.copy_from_slice(r.take(4)?);
        let mut text = || -> Result<String, Error> {
            let n = r.u32()? as usize;
            String::from_utf8(r.take(n)?.to_vec())
                .map_err(|_| bad("formula text isn't UTF-8".into()))
        };
        let (name, source) = (text()?, text()?);
        if !source.is_empty() {
            program = Some(compile_program(fractal, &name, &source)?);
        }
    }
    let mut view3d = View3d::default();
    if version >= 4 {
        let mut i16s = [0i16; 6];
        for v in &mut i16s {
            *v = i16::from_le_bytes(r.take(2)?.try_into().unwrap());
        }
        view3d = View3d {
            rotation: [i16s[0], i16s[1], i16s[2]],
            perspective: i16s[3],
            xyshift: [i16s[4], i16s[5]],
        };
    }
    let params = Params {
        fractal,
        window,
        params,
        max_iter,
        bailout,
        inside,
        outside,
        decomp,
        biomorph,
        use_symmetry,
        forced_symmetry,
        program,
        functions,
        view3d,
    };

    let width = r.u32()?;
    let height = r.u32()?;
    let tag = r.u8()?;
    let pass_mode =
        PassMode::from_tag(tag).ok_or_else(|| bad(format!("unknown pass mode {tag}")))?;
    let state = RenderState {
        pass_mode,
        coarsest_step: r.u32()?,
        stage: r.u32()?,
        row: r.u32()?,
        elapsed_us: r.u64()?,
    };

    let mut palette = Palette([[0; 3]; 256]);
    for rgb in palette.0.iter_mut() {
        rgb.copy_from_slice(r.take(3)?);
    }

    let n = (width as usize)
        .checked_mul(height as usize)
        .filter(|&n| n.checked_mul(4) == Some(r.0.len()))
        .ok_or_else(|| bad("iteration buffer size does not match the image".into()))?;
    let data: Vec<u32> =
        r.0.chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();
    debug_assert_eq!(data.len(), n);
    let buffer = IterBuffer::from_data(width, height, data).expect("length checked above");

    Ok(Checkpoint {
        params,
        state,
        buffer,
        palette,
    })
}

/// Rebuilds the program stored in the block (its entry text).
fn compile_program(
    fractal: &fractalforge_core::FractalType,
    name: &str,
    source: &str,
) -> Result<Program, Error> {
    if let Some(kind) = crate::programs::Kind::of_type(fractal)
        && kind != crate::programs::Kind::Formula
    {
        return kind
            .load(source, name)
            .map(|(p, _)| p)
            .map_err(|e| bad(format!("stored {} {name}: {e}", kind.noun())));
    }
    let entries = parser::entries(source);
    let entry = parser::find(&entries, name)
        .ok_or_else(|| bad(format!("stored formula {name} can't be read")))?;
    let parsed =
        parser::parse(entry).map_err(|e| bad(format!("stored formula {name}: {}", e[0])))?;
    Formula::compile(&parsed.ast, &entry.text)
        .map(Formula::into_program)
        .map_err(|e| bad(format!("stored formula {name}: {e}")))
}

fn symmetry_tag(s: Symmetry) -> u8 {
    match s {
        Symmetry::None => 5,
        Symmetry::XAxis => 1,
        Symmetry::YAxis => 2,
        Symmetry::XY => 3,
        Symmetry::Origin => 4,
        Symmetry::Pi => 6,
    }
}

fn symmetry_from_tag(t: u8) -> Option<Symmetry> {
    Some(match t {
        1 => Symmetry::XAxis,
        2 => Symmetry::YAxis,
        3 => Symmetry::XY,
        4 => Symmetry::Origin,
        5 => Symmetry::None,
        6 => Symmetry::Pi,
        _ => return None,
    })
}

fn same_mode_inside(a: Inside, b: Inside) -> bool {
    std::mem::discriminant(&a) == std::mem::discriminant(&b)
}

fn same_mode_outside(a: Outside, b: Outside) -> bool {
    std::mem::discriminant(&a) == std::mem::discriminant(&b)
}

fn bad(msg: String) -> Error {
    Error::Invalid(format!("FractalForge block: {msg}"))
}

struct Cursor<'a>(&'a [u8]);

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        if self.0.len() < n {
            return Err(bad("truncated".into()));
        }
        let (head, tail) = self.0.split_at(n);
        self.0 = tail;
        Ok(head)
    }

    fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, Error> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, Error> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn f64(&mut self) -> Result<f64, Error> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fractalforge_core::{INSIDE, UNKNOWN};

    #[test]
    fn identifier_is_ascii_and_fixed() {
        assert_eq!(APP_IDENTIFIER, b"FRCTFRGE");
        assert_eq!(AUTH_CODE, b"1.0");
        assert!(APP_IDENTIFIER.is_ascii());
    }

    fn sample() -> Checkpoint {
        let mut params = Params::for_type("julia").unwrap();
        params.params[..4].copy_from_slice(&[-0.8, 0.156, 1e-300, -0.0]);
        params.params[9] = 7.5;
        params.forced_symmetry = Some(Symmetry::YAxis);
        params.functions = [9, 4, 5, 10];
        params.inside = Inside::Bof61;
        params.outside = Outside::Color(9);
        params.decomp = 300;
        params.biomorph = Some(77);
        params.max_iter = 123_456;
        params.view3d = View3d {
            rotation: [-10, 200, 7],
            perspective: 150,
            xyshift: [3, -4],
        };
        let data = (0..15u32 * 7)
            .map(|i| [i, INSIDE, UNKNOWN][i as usize % 3])
            .collect();
        Checkpoint {
            params,
            state: RenderState {
                pass_mode: PassMode::Guess,
                coarsest_step: 4,
                stage: 2,
                row: 3,
                elapsed_us: u64::MAX,
            },
            buffer: IterBuffer::from_data(15, 7, data).unwrap(),
            palette: Palette::forge().rotated(17),
        }
    }

    #[test]
    fn round_trip() {
        let c = sample();
        let ext = encode(&c);
        assert!(ext.is(APP_IDENTIFIER, AUTH_CODE));
        let back = decode(&ext).unwrap();
        assert_eq!(back, c);
        // -0.0 and tiny values survive bit for bit.
        assert_eq!(back.params.params[3].to_bits(), (-0.0f64).to_bits());
    }

    #[test]
    fn ifs_round_trip() {
        let mut c = sample();
        let text = "; c\ntri (3D) {\n 0.5 0 0 0 0.5 0 0 0 0.5 0 0 0 1\n}\n";
        c.params = Params::for_type("ifs").unwrap();
        c.params.program = Some(fractalforge_core::ifs::load(text, "TRI").unwrap());
        let back = decode(&encode(&c)).unwrap();
        assert_eq!(back, c);
        assert_eq!(back.params.program.unwrap().0.name(), "tri");
    }

    #[test]
    fn rejects_corruption() {
        let ext = encode(&sample());
        let mut data = ext.data();
        data[0] = 99; // version
        let bad_ext = AppExtension::new(*APP_IDENTIFIER, *AUTH_CODE, &data, Placement::BeforeImage);
        assert!(decode(&bad_ext).is_err());
        let mut data = ext.data();
        data.truncate(data.len() - 10);
        let bad_ext = AppExtension::new(*APP_IDENTIFIER, *AUTH_CODE, &data, Placement::BeforeImage);
        assert!(decode(&bad_ext).is_err());
    }
}
