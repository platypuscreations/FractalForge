// SPDX-License-Identifier: GPL-3.0-or-later

//! GIF89a save and load: the container, Fractint-format extension blocks and
//! our own native block.
//!
//! On load the native block is preferred, then the Fractint-format blocks,
//! and otherwise the file is treated as a plain image.

pub mod container;
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
    match fractint_blocks::write(
        &c.params,
        c.state.pass_mode,
        finished,
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
