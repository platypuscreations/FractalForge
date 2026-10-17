// SPDX-License-Identifier: GPL-3.0-or-later

//! GIF89a block-structure reader and writer.
//!
//! Only what fractal images need: one full-frame 8-bit image with a global
//! colour table, plus application extensions kept as raw sub-blocks so their
//! exact byte layout round-trips.

use weezl::BitOrder;

use crate::Error;

const TRAILER: u8 = 0x3B;
const EXTENSION: u8 = 0x21;
const IMAGE: u8 = 0x2C;
const APPLICATION_LABEL: u8 = 0xFF;

/// Where an application extension sits relative to the image data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    BeforeImage,
    AfterImage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppExtension {
    pub identifier: [u8; 8],
    pub auth_code: [u8; 3],
    /// Data sub-blocks, each 1..=255 bytes.
    pub sub_blocks: Vec<Vec<u8>>,
    pub placement: Placement,
}

impl AppExtension {
    /// Splits `data` into maximal sub-blocks.
    pub fn new(identifier: [u8; 8], auth_code: [u8; 3], data: &[u8], placement: Placement) -> Self {
        Self {
            identifier,
            auth_code,
            sub_blocks: data.chunks(255).map(<[u8]>::to_vec).collect(),
            placement,
        }
    }

    /// The sub-blocks concatenated.
    pub fn data(&self) -> Vec<u8> {
        self.sub_blocks.concat()
    }

    pub fn is(&self, identifier: &[u8; 8], auth_code: &[u8; 3]) -> bool {
        &self.identifier == identifier && &self.auth_code == auth_code
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GifImage {
    pub width: u16,
    pub height: u16,
    /// Global colour table (padded to 256 entries when written).
    pub palette: Vec<[u8; 3]>,
    pub background: u8,
    pub aspect: u8,
    /// One palette index per pixel, row-major, `width * height` entries.
    pub pixels: Vec<u8>,
    pub extensions: Vec<AppExtension>,
}

impl GifImage {
    pub fn extension(&self, identifier: &[u8; 8], auth_code: &[u8; 3]) -> Option<&AppExtension> {
        self.extensions.iter().find(|e| e.is(identifier, auth_code))
    }

    pub fn write(&self) -> Result<Vec<u8>, Error> {
        let n = self.width as usize * self.height as usize;
        if n == 0 || self.pixels.len() != n {
            return Err(Error::Invalid(
                "pixel count does not match image size".into(),
            ));
        }
        if self.palette.len() > 256 {
            return Err(Error::Invalid("palette has more than 256 colours".into()));
        }
        let mut out = Vec::with_capacity(n / 2 + 1024);
        out.extend_from_slice(b"GIF89a");
        out.extend_from_slice(&self.width.to_le_bytes());
        out.extend_from_slice(&self.height.to_le_bytes());
        // Global colour table present, 8 bits of colour resolution, 256 entries.
        out.extend_from_slice(&[0xF7, self.background, self.aspect]);
        for i in 0..256 {
            out.extend_from_slice(self.palette.get(i).unwrap_or(&[0, 0, 0]));
        }

        self.write_extensions(&mut out, Placement::BeforeImage);

        out.push(IMAGE);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out.extend_from_slice(&self.width.to_le_bytes());
        out.extend_from_slice(&self.height.to_le_bytes());
        out.push(0); // no local colour table, not interlaced
        out.push(8); // LZW minimum code size
        let lzw = weezl::encode::Encoder::new(BitOrder::Lsb, 8)
            .encode(&self.pixels)
            .map_err(|e| Error::Invalid(format!("LZW encoding failed: {e}")))?;
        write_sub_blocks(&mut out, lzw.chunks(255));

        self.write_extensions(&mut out, Placement::AfterImage);
        out.push(TRAILER);
        Ok(out)
    }

    fn write_extensions(&self, out: &mut Vec<u8>, placement: Placement) {
        for ext in self.extensions.iter().filter(|e| e.placement == placement) {
            out.extend_from_slice(&[EXTENSION, APPLICATION_LABEL, 11]);
            out.extend_from_slice(&ext.identifier);
            out.extend_from_slice(&ext.auth_code);
            write_sub_blocks(out, ext.sub_blocks.iter().map(Vec::as_slice));
        }
    }

    /// Parses a GIF87a/89a file, keeping the first image and all
    /// application extensions.
    pub fn read(bytes: &[u8]) -> Result<Self, Error> {
        let mut r = Reader { bytes, pos: 0 };
        let sig = r.take(6)?;
        if sig != b"GIF89a" && sig != b"GIF87a" {
            return Err(Error::Invalid("not a GIF file".into()));
        }
        let width = r.u16()?;
        let height = r.u16()?;
        let flags = r.u8()?;
        let background = r.u8()?;
        let aspect = r.u8()?;
        let mut palette = Vec::new();
        if flags & 0x80 != 0 {
            palette = read_color_table(&mut r, flags)?;
        }
        if width == 0 || height == 0 {
            return Err(Error::Invalid("image has zero size".into()));
        }

        let mut pixels: Option<Vec<u8>> = None;
        let mut extensions = Vec::new();
        loop {
            match r.u8()? {
                TRAILER => break,
                EXTENSION => {
                    let label = r.u8()?;
                    if label == APPLICATION_LABEL {
                        let size = r.u8()? as usize;
                        let header = r.take(size)?;
                        let sub_blocks = read_sub_blocks(&mut r)?;
                        if size == 11 {
                            extensions.push(AppExtension {
                                identifier: header[..8].try_into().unwrap(),
                                auth_code: header[8..11].try_into().unwrap(),
                                sub_blocks,
                                placement: if pixels.is_some() {
                                    Placement::AfterImage
                                } else {
                                    Placement::BeforeImage
                                },
                            });
                        }
                    } else {
                        read_sub_blocks(&mut r)?;
                    }
                }
                IMAGE => {
                    let frame = read_frame(&mut r)?;
                    if pixels.is_none() {
                        pixels = Some(frame.into_canvas(width, height, background));
                    }
                }
                // Tolerate junk after the image the way most decoders do.
                _ if pixels.is_some() => break,
                b => return Err(Error::Invalid(format!("unexpected block 0x{b:02X}"))),
            }
        }

        Ok(Self {
            width,
            height,
            palette,
            background,
            aspect,
            pixels: pixels.ok_or_else(|| Error::Invalid("GIF contains no image".into()))?,
            extensions,
        })
    }
}

struct Frame {
    left: u16,
    top: u16,
    width: u16,
    height: u16,
    pixels: Vec<u8>,
}

impl Frame {
    /// Places the frame on a canvas the size of the logical screen.
    fn into_canvas(self, width: u16, height: u16, background: u8) -> Vec<u8> {
        if (self.left, self.top, self.width, self.height) == (0, 0, width, height) {
            return self.pixels;
        }
        let mut canvas = vec![background; width as usize * height as usize];
        for y in 0..self.height as usize {
            let cy = y + self.top as usize;
            if cy >= height as usize {
                break;
            }
            for x in 0..self.width as usize {
                let cx = x + self.left as usize;
                if cx < width as usize {
                    canvas[cy * width as usize + cx] = self.pixels[y * self.width as usize + x];
                }
            }
        }
        canvas
    }
}

fn read_frame(r: &mut Reader) -> Result<Frame, Error> {
    let left = r.u16()?;
    let top = r.u16()?;
    let width = r.u16()?;
    let height = r.u16()?;
    let flags = r.u8()?;
    if flags & 0x80 != 0 {
        read_color_table(r, flags)?; // local tables are ignored
    }
    let min_code_size = r.u8()?;
    if !(2..=8).contains(&min_code_size) {
        return Err(Error::Invalid(format!("bad LZW code size {min_code_size}")));
    }
    let data = read_sub_blocks(r)?.concat();

    let n = width as usize * height as usize;
    let mut decoded = Vec::with_capacity(n);
    let result = weezl::decode::Decoder::new(BitOrder::Lsb, min_code_size)
        .into_vec(&mut decoded)
        .decode(&data);
    if let Err(e) = result.status
        && decoded.len() < n
    {
        return Err(Error::Invalid(format!("LZW data is corrupt: {e}")));
    }
    // Missing pixels (a truncated stream) become 0.
    decoded.resize(n, 0);

    let pixels = if flags & 0x40 != 0 {
        deinterlace(&decoded, width as usize, height as usize)
    } else {
        decoded
    };
    Ok(Frame {
        left,
        top,
        width,
        height,
        pixels,
    })
}

fn deinterlace(data: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = vec![0; data.len()];
    let order = [(0, 8), (4, 8), (2, 4), (1, 2)]
        .into_iter()
        .flat_map(|(start, step)| (start..height).step_by(step));
    for (src_row, dst_row) in order.enumerate() {
        out[dst_row * width..(dst_row + 1) * width]
            .copy_from_slice(&data[src_row * width..(src_row + 1) * width]);
    }
    out
}

fn read_color_table(r: &mut Reader, flags: u8) -> Result<Vec<[u8; 3]>, Error> {
    let entries = 2usize << (flags & 0x07);
    let raw = r.take(entries * 3)?;
    Ok(raw.chunks(3).map(|c| [c[0], c[1], c[2]]).collect())
}

fn read_sub_blocks(r: &mut Reader) -> Result<Vec<Vec<u8>>, Error> {
    let mut blocks = Vec::new();
    loop {
        let len = r.u8()? as usize;
        if len == 0 {
            return Ok(blocks);
        }
        blocks.push(r.take(len)?.to_vec());
    }
}

fn write_sub_blocks<'a>(out: &mut Vec<u8>, blocks: impl Iterator<Item = &'a [u8]>) {
    for b in blocks.filter(|b| !b.is_empty()) {
        debug_assert!(b.len() <= 255);
        out.push(b.len() as u8);
        out.extend_from_slice(b);
    }
    out.push(0);
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        let end = self.pos.checked_add(n).filter(|&e| e <= self.bytes.len());
        let end = end.ok_or_else(|| Error::Invalid("GIF file is truncated".into()))?;
        let s = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(s)
    }

    fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, Error> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> GifImage {
        GifImage {
            width: 37,
            height: 11,
            palette: (0..=255).map(|i| [i as u8, 255 - i as u8, 7]).collect(),
            background: 0,
            aspect: 0,
            pixels: (0..37 * 11).map(|i| (i * 7 % 256) as u8).collect(),
            extensions: vec![
                AppExtension::new(*b"TESTAPP1", *b"1.0", &[1; 600], Placement::BeforeImage),
                AppExtension::new(*b"TESTAPP2", *b"xyz", &[9, 8, 7], Placement::AfterImage),
            ],
        }
    }

    #[test]
    fn round_trip() {
        let img = sample();
        let bytes = img.write().unwrap();
        assert_eq!(&bytes[..6], b"GIF89a");
        assert_eq!(GifImage::read(&bytes).unwrap(), img);
        let ext = &img.extensions[0];
        assert_eq!(
            ext.sub_blocks.iter().map(Vec::len).collect::<Vec<_>>(),
            [255, 255, 90]
        );
    }

    #[test]
    fn deinterlaces() {
        let h = 10;
        // Interlaced row order for 10 rows: 0, 8, 4, 2, 6, 1, 3, 5, 7, 9.
        let stored: Vec<u8> = [0, 8, 4, 2, 6, 1, 3, 5, 7, 9].into_iter().collect();
        assert_eq!(deinterlace(&stored, 1, h), (0..10).collect::<Vec<u8>>());
    }

    #[test]
    fn rejects_garbage() {
        assert!(GifImage::read(b"").is_err());
        assert!(GifImage::read(b"PNG\x89 not a gif").is_err());
        let mut bytes = sample().write().unwrap();
        bytes.truncate(bytes.len() / 2);
        assert!(GifImage::read(&bytes).is_err());
    }

    #[test]
    fn reads_xfractint_corpus() {
        let dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/xfractint/complete"
        );
        let mut n = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "gif") {
                let img = GifImage::read(&std::fs::read(&path).unwrap()).unwrap();
                assert_eq!(img.pixels.len(), img.width as usize * img.height as usize);
                // Re-encoding preserves pixels and extension bytes.
                let again = GifImage::read(&img.write().unwrap()).unwrap();
                assert_eq!(again.pixels, img.pixels, "{}", path.display());
                assert_eq!(again.extensions, img.extensions, "{}", path.display());
                n += 1;
            }
        }
        assert!(n > 0);
    }
}
