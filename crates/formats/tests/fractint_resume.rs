// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading Fractint resume data, checked against XFractint's interrupted
//! saves and finished renders (corpus/xfractint/resume,
//! docs/specs/fractint-resume.md).

use std::path::PathBuf;

use fractalforge_core::{UNKNOWN, color_index};
use fractalforge_formats::gif::{GifImage, fractint_blocks};
use fractalforge_render::{Control, PassMode, Render};

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/xfractint/resume")
}

fn image(rel: &str) -> GifImage {
    GifImage::read(&std::fs::read(dir().join(rel)).unwrap()).unwrap()
}

/// Interrupted saves and their finished references.
fn pairs() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::fs::read_dir(dir().join("int"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter_map(|n| {
            let stem = n.strip_suffix(".gif")?;
            let base = stem.strip_prefix("i-").unwrap_or(stem);
            let (view, _secs) = base.rsplit_once('-')?;
            let reference = format!("ref/ref-{view}.gif");
            dir()
                .join(&reference)
                .exists()
                .then(|| (format!("int/{n}"), reference))
        })
        .collect();
    out.sort();
    out
}

/// Every pixel the reader keeps from an interrupted save is the final one.
#[test]
fn kept_pixels_are_final() {
    let pairs = pairs();
    assert!(pairs.len() > 60, "{} pairs", pairs.len());
    let mut total_kept = 0;
    for (int, reference) in &pairs {
        let img = image(int);
        let fin = image(reference);
        let info = fractint_blocks::read(&img).unwrap();
        let p = info.params.unwrap();
        // Some runs finished before the save key arrived (spec §4).
        let Some(buf) = fractint_blocks::partial_buffer(&img, p.fractal) else {
            assert!(info.finished, "{int}");
            continue;
        };
        let mut kept = 0;
        for (i, &v) in buf.data().iter().enumerate() {
            if v != UNKNOWN {
                kept += 1;
                assert_eq!(
                    v as u8,
                    fin.pixels[i],
                    "{int}: pixel {} {}",
                    i % 400,
                    i / 400
                );
            }
        }
        total_kept += kept;
    }
    assert!(total_kept > 1_000_000);
}

/// Finishing from the kept pixels reproduces XFractint's finished render
/// (palette indices compared through their colours, see types.md §1.3).
#[test]
fn resuming_reproduces_xfractints_render() {
    let mut failures = Vec::new();
    for (int, reference) in pairs() {
        // The cheaper views; the others use maxiter 100000.
        let cheap = ["MN-", "MNG-", "BOF-", "ORL-"];
        if !cheap.iter().any(|v| int.contains(v)) {
            continue;
        }
        let img = image(&int);
        let fin = image(&reference);
        let p = fractint_blocks::read(&img).unwrap().params.unwrap();
        let Some(buf) = fractint_blocks::partial_buffer(&img, p.fractal) else {
            continue;
        };
        let mut r = Render::from_partial(p.clone(), buf, PassMode::One).unwrap();
        r.run(&Control::default());
        let canon = |i: u8| {
            let c = img.palette[i as usize];
            img.palette.iter().position(|&x| x == c).unwrap()
        };
        let same = r
            .buffer()
            .data()
            .iter()
            .zip(&fin.pixels)
            .filter(|&(&v, &px)| canon(color_index(v, &p)) == canon(px))
            .count();
        let share = same as f64 / fin.pixels.len() as f64;
        // Kept pixels are exact (kept_pixels_are_final). The pixels we
        // compute differ from XFractint's in ~0.05% at these deep, high
        // maxiter views (floating-point rounding on long orbits, types.md
        // §4), and guessing references also differ where XFractint's guesses
        // were wrong (spec §3.3).
        let min = 0.999;
        if share < min {
            failures.push(format!("{int}: {:.4}%", share * 100.0));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
