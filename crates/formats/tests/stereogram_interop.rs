// SPDX-License-Identifier: GPL-3.0-or-later

//! Random-dot stereograms checked against XFractint
//! (corpus/xfractint/stereogram, docs/specs/stereogram.md), with the seeds
//! recovered from its images.

use std::path::PathBuf;

use fractalforge_core::stereogram::{self, Bars, Pattern, Settings};
use fractalforge_formats::gif::GifImage;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/xfractint/stereogram")
}

#[test]
fn corpus_matches_xfractint() {
    let read = |p: PathBuf| GifImage::read(&std::fs::read(p).unwrap()).unwrap();
    let pat = read(root().join("depth/pat.gif"));
    let cases = std::fs::read_to_string(root().join("cases.txt")).unwrap();
    let (mut exact, mut problems) = (0, Vec::new());
    for line in cases
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let f: Vec<&str> = line.split_whitespace().collect();
        let [name, depth, width, effect, bars, seed, ..] = f[..] else {
            continue;
        };
        let out = read(root().join(format!("out/{name}.gif")));
        let d = read(root().join(format!("depth/{depth}.gif")));
        let s = Settings {
            depth: effect.parse().unwrap(),
            width_inches: width.parse().unwrap(),
            grayscale: bars == "gray",
            bars: match bars {
                "top" => Bars::Top,
                "none" => Bars::None,
                _ => Bars::Middle,
            },
            seed: seed.parse().unwrap_or(1),
        };
        let pattern = (seed == "-").then(|| Pattern {
            pixels: &pat.pixels,
            width: pat.width.into(),
            height: pat.height.into(),
        });
        let ours = stereogram::render(
            &d.pixels,
            out.width.into(),
            out.height.into(),
            &d.palette,
            &s,
            pattern.as_ref(),
        );
        let w = out.width as usize;
        let bad_rows = (0..out.height as usize)
            .filter(|y| ours[y * w..(y + 1) * w] != out.pixels[y * w..(y + 1) * w])
            .count();
        // Separations below 0 (l-d500, §3.4) and the grayscale
        // quantisation (§3.5) aren't modelled.
        let allowed = match name {
            "l-d500" => 2,
            n if n.starts_with("l-gray") => 10,
            _ => 0,
        };
        if bad_rows == 0 {
            exact += 1;
        } else if bad_rows > allowed {
            problems.push(format!("{name}: {bad_rows} rows differ"));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert!(exact >= 37, "{exact}");
}
