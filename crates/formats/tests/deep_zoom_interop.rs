// SPDX-License-Identifier: GPL-3.0-or-later

//! Deep zooms checked against XFractint (corpus/xfractint/deep-zoom,
//! docs/specs/deep-zoom.md): its fractint005 blocks, its pixels, and the
//! blocks we write.

use std::path::PathBuf;

use fractalforge_core::color_index;
use fractalforge_formats::gif::{GifImage, fractint_blocks};
use fractalforge_render::{Control, PassMode, Render};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn image(rel: &str) -> GifImage {
    let path = root().join("corpus/xfractint/deep-zoom").join(rel);
    GifImage::read(&std::fs::read(path).unwrap()).unwrap()
}

/// Share of pixels equal to XFractint's when rendering its deep file.
fn match_rate(img: &GifImage) -> f64 {
    let p = fractint_blocks::read(img).unwrap().params.unwrap();
    assert!(p.deep.is_some(), "the file's view should be read as deep");
    let mut r = Render::new(
        p.clone(),
        img.width.into(),
        img.height.into(),
        PassMode::One,
    )
    .unwrap();
    r.run(&Control::default());
    let same = r
        .buffer()
        .data()
        .iter()
        .zip(&img.pixels)
        .filter(|&(&v, &px)| img.palette[color_index(v, &p) as usize] == img.palette[px as usize])
        .count();
    same as f64 / img.pixels.len() as f64
}

/// Perturbation reproduces XFractint's deep pixels; at 1e40 XFractint's own
/// limited precision shows in about 1 % of them (§8.2).
#[test]
fn deep_pixels_match_xfractint() {
    for (file, floor) in [
        ("pixels/px-m15.gif", 0.9999),
        ("pixels/px-m15g.gif", 0.9999),
        ("pixels/px-m20.gif", 0.9999),
        ("pixels/px-j20.gif", 0.9999),
        ("pixels/px-m40.gif", 0.985),
    ] {
        let rate = match_rate(&image(file));
        assert!(rate >= floor, "{file}: {:.4}%", rate * 100.0);
    }
}

/// The view we read from a crafted file is the location XFractint rendered
/// (deep-zoom.md §5.1: k01 moves the slots to view V2, the doubles stay V1).
#[test]
fn reads_the_slots_not_the_doubles() {
    let v2 = fractint_blocks::read(&image("base/base-v2.gif"))
        .unwrap()
        .params
        .unwrap();
    let k01 = fractint_blocks::read(&image("crafted/k01-both.gif"))
        .unwrap()
        .params
        .unwrap();
    let (a, b) = (v2.deep.unwrap(), k01.deep.unwrap());
    assert!(a.im.sub(&b.im).to_f64().abs() < 1e-30);
    assert!(a.re.sub(&b.re).to_f64().abs() < 1e-30);
    // k12: the julia constant comes from slots 6/7 (0, −1), not the doubles.
    let k12 = fractint_blocks::read(&image("crafted/k12-jslot.gif"))
        .unwrap()
        .params
        .unwrap();
    assert_eq!(&k12.params[..2], &[0.0, -1.0]);
}

/// What we write for a deep view decodes to the same view, with the flags
/// XFractint needs, and slots 16–21 copying 0–5.
#[test]
fn writes_a_readable_deep_block() {
    let img = image("pixels/px-m20.gif");
    let p = fractint_blocks::read(&img).unwrap().params.unwrap();
    let (blocks, _) = fractint_blocks::write(&p, PassMode::One, None, 400, 300, 0).unwrap();
    let main = blocks.last().unwrap().data();
    let i16_at = |o: usize| i16::from_le_bytes([main[o], main[o + 1]]);
    assert_eq!((i16_at(412), i16_at(414)), (1, 12));
    let deep = blocks
        .iter()
        .find(|b| &b.auth_code == fractint_blocks::DEEP_AUTH)
        .unwrap()
        .data();
    let slot = deep.len() / 22;
    assert_eq!(deep[..6 * slot], deep[16 * slot..]);
    // Read back through a GIF with our blocks.
    let mut ours = img.clone();
    ours.extensions = blocks;
    let back = fractint_blocks::read(&ours).unwrap().params.unwrap();
    let (a, b) = (p.deep.unwrap(), back.deep.unwrap());
    assert!(a.re.sub(&b.re).to_f64().abs() < 1e-35);
    assert!((a.half_h / b.half_h - 1.0).abs() < 1e-12);
}
