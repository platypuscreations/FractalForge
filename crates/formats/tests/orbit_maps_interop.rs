// SPDX-License-Identifier: GPL-3.0-or-later

//! Orbit-plotting map types checked against XFractint
//! (corpus/xfractint/orbit-maps, docs/specs/orbit-maps.md).

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
/// exactly. Measured values minus a margin, since function-slot sin/cos
/// come from the platform's libm.
const FLOORS: &[(&str, f64)] = &[
    // Fixed sin/cos/log/atan from the libm crate drift from glibc's on long
    // chaotic orbits (§2.4).
    ("chip-def", 0.85),
    ("chip-mi2z", 0.95),
    ("chip-p", 0.97),
    ("martin-def", 0.90),
    ("quadruptwo-def", 0.75),
    ("quadruptwo-p", 0.93),
    ("threeply-def", 0.92),
    // x87 sin in function slots isn't modelled (§2.4, §4.3).
    ("latoocarfian-f1c", 0.90),
    ("latoocarfian-f2c", 0.90),
    ("latoocarfian-f3c", 0.90),
    ("latoocarfian-f4c", 0.90),
    ("latoocarfian-ssss-mi10", 0.85),
    ("latoocarfian-ssss-mi40", 0.80),
    ("latoocarfian-ssss", 0.75),
    // tan near its poles (§2.4).
    ("popcorn-st-mi10", 0.995),
    ("popcorn-st", 0.85),
    ("popcornjul-def", 0.999),
    ("popcornjul-st", 0.90),
    // The spec's own hypotheses (§9, §10).
    ("mandelcloud-def", 0.99),
    ("mandelcloud-mi10", 0.99),
    ("mandelcloud-n10", 0.999),
    ("mandelcloud-n2-u", 0.9999),
    ("julia_inverse-c", 0.99),
    ("julia_inverse-def", 0.98),
    ("julia_inverse-p8", 0.98),
    ("julia_inverse-q64", 0.98),
    ("julia_inverse-rs1", 0.98),
    // inside=300 starts at colour 1 in XFractint (§3.3); FractalForge
    // reads inside values above 255 wrapped, as for escape-time types.
    ("henon-in300-u", 0.97),
];

#[test]
fn corpus_matches_xfractint() {
    let mut files = Vec::new();
    for sub in ["", "res800"] {
        let dir = root().join("corpus/xfractint/orbit-maps").join(sub);
        for e in std::fs::read_dir(&dir).unwrap() {
            let path = e.unwrap().path();
            if path.extension().is_some_and(|x| x == "gif") {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut low = Vec::new();
    let mut types = std::collections::BTreeSet::new();
    for f in &files {
        let stem = f.file_stem().unwrap().to_string_lossy().into_owned();
        let floor = FLOORS
            .iter()
            .find(|(n, _)| *n == stem)
            .map_or(1.0, |&(_, v)| v);
        match match_rate(f) {
            Some((t, rate)) => {
                types.insert(t);
                if rate < floor {
                    low.push(format!(
                        "{stem}: {:.3}% (want {:.1}%)",
                        rate * 100.0,
                        floor * 100.0
                    ));
                }
            }
            None => low.push(format!("{stem}: parameters not usable")),
        }
    }
    assert_eq!(files.len(), 77);
    assert_eq!(types.len(), 15, "{types:?}");
    assert!(low.is_empty(), "{}", low.join("\n"));
}

/// popcornjul resumes from XFractint's work list; plotted types such as
/// henon are calculated again (orbit-maps.md §11).
#[test]
fn interrupted_images_finish_like_xfractint() {
    let dir = root().join("corpus/xfractint/orbit-maps/resume");
    let read = |n: &str| GifImage::read(&std::fs::read(dir.join(n)).unwrap()).unwrap();
    for (partial, reference, resumes) in [
        ("int-pjul.gif", "ref-pjul.gif", true),
        ("int-henon.gif", "ref-henon.gif", false),
    ] {
        let img = read(partial);
        let info = fractint_blocks::read(&img).unwrap();
        assert!(!info.finished, "{partial}");
        let p = info.params.unwrap();
        let buf = fractint_blocks::partial_buffer(&img, p.fractal);
        assert_eq!(buf.is_some(), resumes, "{partial}");
        let (w, h) = (img.width.into(), img.height.into());
        let mut r = match buf {
            Some(b) => Render::from_partial(p.clone(), b, PassMode::One).unwrap(),
            None => Render::new(p.clone(), w, h, PassMode::One).unwrap(),
        };
        r.run(&Control::default());
        let want = read(reference);
        let differ = r
            .buffer()
            .data()
            .iter()
            .zip(&want.pixels)
            .filter(|&(&v, &px)| {
                want.palette[color_index(v, &p) as usize] != want.palette[px as usize]
            })
            .count();
        assert!(
            differ <= 10,
            "{partial}: {differ} pixels differ from {reference}"
        );
    }
}
