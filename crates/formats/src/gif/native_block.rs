// SPDX-License-Identifier: GPL-3.0-or-later

//! FractalForge's own GIF application-extension block, holding a complete
//! render checkpoint. See `docs/specs/gif-native-block.md`.

use fractalforge_core::{Inside, IterBuffer, Outside, Palette, Params, Window, types};
use fractalforge_render::{PassMode, RenderState};

use super::container::{AppExtension, Placement};
use crate::Error;

/// Application identifier (8 bytes). Part of the file format; never change it.
pub const APP_IDENTIFIER: &[u8; 8] = b"FRCTFRGE";

/// Application authentication code (3 bytes). Part of the file format.
pub const AUTH_CODE: &[u8; 3] = b"1.0";

/// Payload layout version written by this code. Version 1 (pre-release)
/// stored escape counts rather than colour values and isn't read.
pub const PAYLOAD_VERSION: u16 = 2;

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
    if version != PAYLOAD_VERSION {
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
    let params = [r.f64()?, r.f64()?, r.f64()?, r.f64()?];
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
        program: None,
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
        params.params = [-0.8, 0.156, 1e-300, -0.0];
        params.inside = Inside::Bof61;
        params.outside = Outside::Color(9);
        params.decomp = 300;
        params.biomorph = Some(77);
        params.max_iter = 123_456;
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
