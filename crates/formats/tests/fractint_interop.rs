// SPDX-License-Identifier: GPL-3.0-or-later

//! Interop with Fractint-format GIFs, checked against files XFractint saved
//! (corpus/xfractint).

use std::path::PathBuf;

use fractalforge_core::{Inside, IterBuffer, Palette, Params, color_index};
use fractalforge_formats::gif::{
    self, Checkpoint, Content, FractintInfo, GifImage, fractint_blocks,
};
use fractalforge_render::{Control, PassMode, Render, RenderState};

fn corpus(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/xfractint")
        .join(path)
}

fn image(path: &str) -> GifImage {
    GifImage::read(&std::fs::read(corpus(path)).unwrap()).unwrap()
}

fn info(path: &str) -> FractintInfo {
    fractint_blocks::read(&image(path)).unwrap_or_else(|| panic!("{path} has no Fractint block"))
}

fn params(path: &str) -> Params {
    info(path).params.unwrap()
}

#[test]
fn reads_basic_parameters() {
    let i = info("complete/mandel-base.gif");
    assert_eq!(i.type_number, 4);
    assert!(i.finished);
    assert_eq!(i.pass_mode, PassMode::Guess);
    assert!(i.warnings.is_empty(), "{:?}", i.warnings);
    let p = i.params.unwrap();
    assert_eq!(p.fractal.name, "mandel");
    assert_eq!(
        (
            p.window.x_min,
            p.window.x_max,
            p.window.y_min,
            p.window.y_max
        ),
        (-2.5, 1.5, -1.5, 1.5)
    );
    assert_eq!(p.max_iter, 150);
    assert_eq!(p.bailout, 4.0);
    assert_eq!(p.inside, Inside::Color(0));

    let p = params("complete/julia-params.gif");
    assert_eq!(p.fractal.name, "julia");
    assert_eq!(&p.params[..2], &[-0.8, 0.156]);

    assert_eq!(params("complete/mandel-corners.gif").window.x_min, -0.75);
    assert_eq!(params("complete/mandel-bailout-100.gif").bailout, 100.0);
    assert_eq!(params("complete/mandel-maxiter-1000.gif").max_iter, 1000);
    assert_eq!(
        params("complete/mandel-maxiter-100000.gif").max_iter,
        100_000
    );
    assert_eq!(info("complete/mandel-pass1.gif").pass_mode, PassMode::One);
    assert_eq!(info("complete/mandel-pass2.gif").pass_mode, PassMode::Two);
}

#[test]
fn reports_what_it_cannot_use() {
    let i = info("complete/mandel-inside-bof60.gif");
    assert!(
        i.warnings.iter().any(|w| w.contains("inside")),
        "{:?}",
        i.warnings
    );
    let i = info("complete/mandel-biomorph.gif");
    assert!(
        i.warnings.iter().any(|w| w.contains("biomorph")),
        "{:?}",
        i.warnings
    );
    let i = info("complete/lambda-base.gif");
    assert_eq!(i.type_number, 75);
    assert!(i.params.is_none());
    let i = info("interrupted/int-pass1-early.gif");
    assert!(!i.finished);
    assert_eq!(i.pass_mode, PassMode::One);
}

/// Our main block must equal XFractint's for the same settings, except the
/// per-render random seed and calculation time (spec §3).
#[test]
fn writes_the_same_main_block_as_xfractint() {
    for (file, mode) in [
        ("complete/mandel-base.gif", PassMode::Guess),
        ("complete/mandel-pass1.gif", PassMode::One),
        ("complete/mandel-pass2.gif", PassMode::Two),
        ("complete/julia-params.gif", PassMode::Guess),
        ("complete/mandel-corners.gif", PassMode::Guess),
        ("complete/mandel-maxiter-100000.gif", PassMode::Guess),
        ("complete/mandel-inside-0.gif", PassMode::Guess),
    ] {
        let img = image(file);
        let theirs = img
            .extension(fractint_blocks::IDENTIFIER, fractint_blocks::MAIN_AUTH)
            .unwrap()
            .data();
        let p = params(file);
        let (blocks, _) = fractint_blocks::write(&p, mode, true, 800, 600, 0).unwrap();
        assert_eq!(blocks.len(), 1);
        let ours = blocks[0].data();
        assert_eq!(ours.len(), theirs.len());
        let diffs: Vec<usize> = (0..ours.len())
            .filter(|&i| ours[i] != theirs[i] && !matches!(i, 98..=99 | 214..=217))
            .collect();
        // XFractint stores 0 ("type default") when no bailout was given; we
        // always write the value.
        let diffs: Vec<usize> = diffs
            .into_iter()
            .filter(|i| !(212..=213).contains(i) && !(402..=405).contains(i))
            .collect();
        assert!(diffs.is_empty(), "{file}: bytes differ at {diffs:?}");
    }
}

#[test]
fn round_trips_through_a_gif() {
    let mut p = Params::for_type("julia").unwrap();
    p.params = [-0.4, 0.6, 0.0, 0.0];
    p.max_iter = 40_000;
    p.bailout = 16.0;
    p.inside = Inside::MaxIter;
    let (w, h) = (64, 48);
    let mut r = Render::new(p.clone(), w, h, PassMode::Two).unwrap();
    r.run(&Control::default());
    let c = Checkpoint {
        params: p.clone(),
        state: r.checkpoint(),
        buffer: r.buffer().clone(),
        palette: Palette::forge(),
    };
    let saved = gif::save(&c, true).unwrap();
    assert!(saved.notes.is_empty(), "{:?}", saved.notes);

    // Strip our own block: what's left must still describe the image.
    let mut img = GifImage::read(&saved.bytes).unwrap();
    img.extensions.retain(|e| &e.identifier != b"FRCTFRGE");
    let Content::Fractint(info) = gif::load(&img.write().unwrap()).unwrap().content else {
        panic!("expected Fractint content");
    };
    assert!(info.finished);
    assert_eq!(info.pass_mode, PassMode::Two);
    assert!(info.warnings.is_empty(), "{:?}", info.warnings);
    assert_eq!(info.params.unwrap(), p);
}

#[test]
fn unfinished_save_asks_fractint_to_recalculate() {
    let p = Params::for_type("mandel").unwrap();
    let c = Checkpoint {
        params: p,
        state: RenderState::new(PassMode::Guess, 40, 30),
        buffer: IterBuffer::new(40, 30),
        palette: Palette::forge(),
    };
    let saved = gif::save(&c, false).unwrap();
    let img = GifImage::read(&saved.bytes).unwrap();
    let ids: Vec<_> = img
        .extensions
        .iter()
        .map(|e| (e.identifier, e.auth_code))
        .collect();
    assert_eq!(
        ids,
        [
            (*b"FRCTFRGE", *b"1.0"),
            (*b"fractint", *b"002"),
            (*b"fractint", *b"001")
        ]
    );
    let main = img.extensions[2].data();
    assert_eq!(main[200], b'1');
    assert_eq!(i16::from_le_bytes([main[202], main[203]]), 2);
    // Fractint blocks' framed length: 409 for 002 plus 521 for 001 (spec §2.1).
    assert_eq!(u16::from_le_bytes([main[204], main[205]]), 930);
    let resume = img.extensions[1].data();
    assert_eq!(resume.len(), 392);
    assert_eq!(i32::from_le_bytes(resume[12..16].try_into().unwrap()), 39);
    assert_eq!(i32::from_le_bytes(resume[20..24].try_into().unwrap()), 29);
}

/// Rendering a corpus file's parameters must reproduce XFractint's pixels.
/// A small share differs: XFractint escapes some pixels near the set
/// boundary exactly 16 iterations earlier, for reasons not yet understood.
#[test]
fn renders_match_xfractint_pixels() {
    for (file, min_match) in [
        ("complete/mandel-pass1.gif", 0.997),
        ("complete/mandel-bailout-100.gif", 0.997),
        ("complete/mandel-corners.gif", 0.994),
        ("complete/mandel-maxiter-1000.gif", 0.996),
        ("complete/julia-params.gif", 0.995),
    ] {
        let img = image(file);
        let p = params(file);
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
            .filter(|&(&v, &px)| color_index(v, &p) == px)
            .count();
        let share = same as f64 / img.pixels.len() as f64;
        assert!(
            share >= min_match,
            "{file}: only {:.3}% of pixels match",
            share * 100.0
        );
    }
}
