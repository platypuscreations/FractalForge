// SPDX-License-Identifier: GPL-3.0-or-later

//! PNG export: the coloured image plus the parameters as text, so the file
//! documents how it was made. PNGs can't be resumed.

use fractalforge_core::{IterBuffer, Palette, Params, color_index};

use crate::Error;

/// Encodes the image as an 8-bit RGB PNG. Pixels not computed yet are black.
pub fn encode(params: &Params, buffer: &IterBuffer, palette: &Palette) -> Result<Vec<u8>, Error> {
    let (w, h) = (buffer.width(), buffer.height());
    let mut rgb = Vec::with_capacity(buffer.data().len() * 3);
    for &v in buffer.data() {
        rgb.extend_from_slice(&palette.0[color_index(v, params) as usize]);
    }
    let mut out = Vec::new();
    let err = |e: png::EncodingError| Error::Invalid(format!("PNG encoding failed: {e}"));
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        enc.add_text_chunk("Software".into(), "FractalForge".into())
            .map_err(err)?;
        enc.add_text_chunk("Comment".into(), describe(params))
            .map_err(err)?;
        let mut writer = enc.write_header().map_err(err)?;
        writer.write_image_data(&rgb).map_err(err)?;
    }
    Ok(out)
}

/// A one-line description of the parameters.
pub fn describe(p: &Params) -> String {
    let w = &p.window;
    let used = p
        .params
        .iter()
        .rposition(|&v| v != 0.0)
        .map_or(2, |i| i + 1)
        .max(2);
    let params: Vec<String> = p.params[..used].iter().map(f64::to_string).collect();
    format!(
        "type={} corners={}/{}/{}/{} params={} maxiter={} bailout={}",
        p.fractal.name,
        w.x_min,
        w.x_max,
        w.y_min,
        w.y_max,
        params.join("/"),
        p.max_iter,
        p.bailout
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_a_png() {
        let p = Params::for_type("mandel").unwrap();
        let b =
            IterBuffer::from_data(3, 2, vec![1, 2, 3, fractalforge_core::INSIDE, 5, 6]).unwrap();
        let bytes = encode(&p, &b, &Palette::forge()).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        let decoder = png::Decoder::new(std::io::Cursor::new(&bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut img = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut img).unwrap();
        assert_eq!((info.width, info.height), (3, 2));
        assert_eq!(&img[9..12], &[0, 0, 0]); // inside pixel is index 0 = black
        let text = &reader.info().uncompressed_latin1_text;
        assert!(
            text.iter()
                .any(|t| t.keyword == "Comment" && t.text.starts_with("type=mandel"))
        );
    }
}
