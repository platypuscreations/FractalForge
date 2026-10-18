// SPDX-License-Identifier: GPL-3.0-or-later

//! Strange attractors checked against XFractint
//! (corpus/xfractint/attractors-3d, docs/specs/attractors-3d.md).

use std::path::{Path, PathBuf};

use fractalforge_core::color_index;
use fractalforge_formats::gif::{GifImage, fractint_blocks};
use fractalforge_render::{Control, PassMode, Render};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The share of pixels equal to XFractint's, compared through its palette
/// (types.md §1.3), or `None` when the file's parameters aren't usable.
fn match_rate(gif: &Path) -> Option<(String, f64)> {
    let img = GifImage::read(&std::fs::read(gif).unwrap()).unwrap();
    let p = fractint_blocks::read(&img)?.params?;
    let mut r = Render::new(
        p.clone(),
        img.width.into(),
        img.height.into(),
        PassMode::One,
    )
    .ok()?;
    r.run(&Control::default());
    let canon: Vec<u8> = (0..256usize)
        .map(|i| {
            let c = img.palette.get(i);
            img.palette.iter().position(|x| Some(x) == c).unwrap_or(i) as u8
        })
        .collect();
    // Hit counts read the displayed colour back (§4.1): with palette
    // index 16 equal to 0, counts wrap at 16.
    let wraps = canon[16] == 0 && matches!(p.fractal.name, "icons" | "latoocarfian");
    let same = r
        .buffer()
        .data()
        .iter()
        .zip(&img.pixels)
        .filter(|&(&v, &px)| {
            let v = if wraps { v % 16 } else { v };
            canon[color_index(v, &p) as usize] == canon[px as usize]
        })
        .count();
    Some((
        p.fractal.name.to_owned(),
        same as f64 / img.pixels.len() as f64,
    ))
}

/// Lowest share of matching pixels per file; every other file must match
/// exactly.
const FLOORS: &[(&str, f64)] = &[
    // glibc's sin/cos aren't correctly rounded everywhere; ours are, so
    // long orbits drift from XFractint's (§6).
    ("pickover-def", 0.6),
    ("pickover-mi2", 0.97),
    ("pickover-mi2-rot0", 0.97),
    // The spec's partial models (§7, §8).
    ("lorenz3d1-def", 0.1),
    ("lorenz3d3-def", 0.75),
    ("lorenz3d3-mi2-rot0", 0.98),
    ("lorenz3d4-def", 0.8),
    ("lorenz3d4-mi2-rot0", 0.99),
    ("kamtorus3d-def", 0.95),
    ("kamtorus3d-rot0", 0.999),
    ("kamtorus3d-rot0-step01", 0.999),
    ("kamtorus3d-rot0-stop3", 0.999),
];

#[test]
fn corpus_matches_xfractint() {
    let dir = root().join("corpus/xfractint/attractors-3d");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "gif"))
        .collect();
    files.sort();
    let (mut exact, mut low) = (0, Vec::new());
    for f in &files {
        let stem = f.file_stem().unwrap().to_string_lossy().into_owned();
        // icons3d isn't modelled (§9); interrupted and reloaded images are
        // partial.
        if stem.starts_with("icons3d") || stem.ends_with("-int") || stem.ends_with("-rload") {
            continue;
        }
        let floor = FLOORS
            .iter()
            .find(|(n, _)| *n == stem)
            .map_or(1.0, |&(_, v)| v);
        match match_rate(f) {
            Some((_, rate)) if rate >= floor => exact += (rate == 1.0) as u32,
            Some((_, rate)) => low.push(format!(
                "{stem}: {:.3}% (want {:.1}%)",
                rate * 100.0,
                floor * 100.0
            )),
            None => low.push(format!("{stem}: parameters not usable")),
        }
    }
    assert!(low.is_empty(), "{}", low.join("\n"));
    assert!(exact >= 35, "{exact}");
}
