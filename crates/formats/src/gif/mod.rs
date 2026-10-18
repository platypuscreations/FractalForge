// SPDX-License-Identifier: GPL-3.0-or-later

//! GIF89a save and load: the container, Fractint-format extension blocks and
//! our own native block.
//!
//! On load the native block is preferred, then the Fractint-format blocks,
//! and otherwise the file is treated as a plain image.

pub mod container;
pub mod deep_block;
pub mod fractint_blocks;
pub mod native_block;

use fractalforge_core::color_index;

pub use container::{AppExtension, GifImage, Placement};
pub use fractint_blocks::FractintInfo;
pub use native_block::Checkpoint;

use crate::Error;

/// What a loaded GIF could be resumed or re-rendered from.
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    /// A FractalForge checkpoint: exact parameters, progress and iterations.
    Native(Box<Checkpoint>),
    /// Parameters from Fractint-format blocks only.
    Fractint(Box<FractintInfo>),
    /// No fractal information; just a picture.
    Image,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Loaded {
    pub image: GifImage,
    pub content: Content,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saved {
    pub bytes: Vec<u8>,
    /// What the Fractint-format blocks couldn't express, for the user.
    pub notes: Vec<String>,
}

/// Encodes a checkpoint (finished or not) as a GIF. Pixels not yet computed
/// are written as palette index 0.
///
/// The file carries our own block and, when the type has a Fractint
/// equivalent, Fractint-format blocks (main block last).
pub fn save(c: &Checkpoint, finished: bool) -> Result<Saved, Error> {
    let (w, h) = (c.buffer.width(), c.buffer.height());
    let too_big = || Error::Invalid(format!("{w}x{h} is larger than a GIF can hold"));
    let width = u16::try_from(w).map_err(|_| too_big())?;
    let height = u16::try_from(h).map_err(|_| too_big())?;
    let image = GifImage {
        width,
        height,
        palette: c.palette.0.to_vec(),
        background: 0,
        aspect: 0,
        pixels: c
            .buffer
            .data()
            .iter()
            .map(|&v| color_index(v, &c.params))
            .collect(),
        extensions: vec![native_block::encode(c)],
    };
    let mut image = image;
    let mut notes = Vec::new();
    // Fractint continues from the first pixel (in row order) we haven't
    // computed; a row below the image means "nothing left".
    let resume_at = (!finished).then(|| {
        c.buffer
            .data()
            .iter()
            .position(|&v| v == fractalforge_core::UNKNOWN)
            .map_or((h, 0), |i| ((i as u32) / w, (i as u32) % w))
    });
    match fractint_blocks::write(
        &c.params,
        c.state.pass_mode,
        resume_at,
        w,
        h,
        c.state.elapsed_us,
    ) {
        Some((blocks, n)) => {
            image.extensions.extend(blocks);
            notes = n;
        }
        None => notes.push(format!(
            "type {} has no Fractint equivalent; Fractint can't read this file's parameters",
            c.params.fractal.name
        )),
    }
    Ok(Saved {
        bytes: image.write()?,
        notes,
    })
}

/// The palette of red/cyan stereo images (FractalForge's own: Fractint
/// saves its default palette instead, 3d-lighting.md §8.5). Mode 1 values
/// below 128 are eye A in red, from 128 eye B in cyan; mode 2 packs the two
/// levels as 16·B + A.
pub fn stereo_palette(mode: u8) -> fractalforge_core::Palette {
    let mut p = [[0u8; 3]; 256];
    for (i, c) in p.iter_mut().enumerate() {
        *c = match mode {
            1 if i < 128 => [((128 - i) * 2).min(255) as u8, 0, 0],
            1 => {
                let v = ((256 - i) * 2).min(255) as u8;
                [0, v, v]
            }
            _ => {
                let (a, b) = ((i % 16) as u8 * 17, (i / 16) as u8 * 17);
                [a, b, b]
            }
        };
    }
    fractalforge_core::Palette(p)
}

/// Encodes a 3D transform's output as a GIF with Fractint's block for it
/// (3d-transform.md §7). Background pixels get the first black palette
/// entry, or 0. `input_main` is the source image's `fractint001` block.
pub fn save_3d(
    out: &fractalforge_core::three_d::Output,
    palette: &fractalforge_core::Palette,
    settings: &fractalforge_core::three_d::Settings,
    input_main: Option<&[u8]>,
) -> Result<Vec<u8>, Error> {
    let too_big = || {
        Error::Invalid(format!(
            "{}x{} is larger than a GIF can hold",
            out.width, out.height
        ))
    };
    let width = u16::try_from(out.width).map_err(|_| too_big())?;
    let height = u16::try_from(out.height).map_err(|_| too_big())?;
    let palette = if matches!(settings.stereo, 1 | 2) {
        stereo_palette(settings.stereo)
    } else {
        palette.clone()
    };
    let black = palette.0.iter().position(|c| *c == [0, 0, 0]).unwrap_or(0) as u8;
    let image = GifImage {
        width,
        height,
        palette: palette.0.to_vec(),
        background: black,
        aspect: 0,
        pixels: out.pixels.iter().map(|p| p.unwrap_or(black)).collect(),
        extensions: vec![fractint_blocks::write_3d(input_main, settings)],
    };
    image.write()
}

pub fn load(bytes: &[u8]) -> Result<Loaded, Error> {
    let image = GifImage::read(bytes)?;
    let content = match image.extension(native_block::APP_IDENTIFIER, native_block::AUTH_CODE) {
        Some(ext) => {
            let c = native_block::decode(ext)?;
            if (c.buffer.width(), c.buffer.height()) != (image.width as u32, image.height as u32) {
                return Err(Error::Invalid(
                    "FractalForge block size differs from the image".into(),
                ));
            }
            Content::Native(Box::new(c))
        }
        None => match fractint_blocks::read(&image) {
            Some(info) => Content::Fractint(Box::new(info)),
            None => Content::Image,
        },
    };
    Ok(Loaded { image, content })
}
