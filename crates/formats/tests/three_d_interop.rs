// SPDX-License-Identifier: GPL-3.0-or-later

//! 3D transforms checked against XFractint (corpus/xfractint/3d-transform
//! and 3d-lighting; docs/specs/3d-transform.md and 3d-lighting.md).

use std::path::{Path, PathBuf};

use fractalforge_core::three_d::{self, Settings, Source};
use fractalforge_formats::gif::GifImage;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Shares of pixels equal to XFractint's (through its palette) and within
/// one palette index of it; `None` when the case can't be compared (its
/// size or settings, or it isn't a 3D render).
fn rate(input: &Path, output: &Path, args: &[&str]) -> Option<(f64, f64)> {
    if !args.contains(&"3d=yes")
        && !args
            .iter()
            .any(|a| a.starts_with("filltype=") || a.starts_with("rotation="))
    {
        return None;
    }
    let src = GifImage::read(&std::fs::read(input).ok()?).ok()?;
    let out = GifImage::read(&std::fs::read(output).ok()?).ok()?;
    let mut s = Settings::default();
    for a in args {
        let (k, v) = a.split_once('=')?;
        if matches!(k, "3d" | "filename" | "viewwindows" | "map" | "batch") {
            continue;
        }
        s.apply(k, v).ok()?;
    }
    let r = three_d::render(
        &Source {
            pixels: &src.pixels,
            width: src.width.into(),
            height: src.height.into(),
            colours: src.palette.len() as u32,
        },
        &s,
    );
    if (r.width, r.height) != (out.width.into(), out.height.into()) {
        return None;
    }
    let pal = &out.palette;
    let canon = |i: u8| {
        pal.iter()
            .position(|c| *c == pal[i as usize])
            .unwrap_or(i as usize)
    };
    let black = pal.iter().position(|c| *c == [0, 0, 0]).unwrap_or(0) as u8;
    let (mut same, mut near) = (0, 0);
    for (&v, &px) in r.pixels.iter().zip(&out.pixels) {
        let (a, b) = (canon(v.unwrap_or(black)), canon(px));
        same += (a == b) as usize;
        near += (a.abs_diff(b) <= 1) as usize;
    }
    let n = out.pixels.len() as f64;
    Some((same as f64 / n, near as f64 / n))
}

fn run(corpus: &str, inputs: &Path, outputs: &Path) -> Vec<(String, String, (f64, f64))> {
    let cases = std::fs::read_to_string(root().join(corpus).join("cases.txt")).unwrap();
    let mut out = Vec::new();
    for line in cases.lines().filter(|l| !l.starts_with('#')) {
        let t: Vec<&str> = line.split_whitespace().collect();
        let [name, input, args @ ..] = &t[..] else {
            continue;
        };
        let gif = outputs.join(format!("{name}.gif"));
        if let Some(r) = rate(&inputs.join(input), &gif, args) {
            out.push((name.to_string(), args.join(" "), r));
        }
    }
    out
}

/// Every comparable case of both corpora, with the rules of the specs.
/// Exact matches are expected for most geometry cases; light fills with
/// ambient light are within one palette index (3d-lighting.md §5), and the
/// remaining cases are the specs' open questions (type 2 colours, sphere
/// roughness, ties at fractional scales, perspective stereo).
#[test]
fn corpus_matches_xfractint() {
    let t = root().join("corpus/xfractint/3d-transform");
    let geometry = run(
        "corpus/xfractint/3d-transform",
        &t.join("in"),
        &t.join("out"),
    );
    let l = root().join("corpus/xfractint/3d-lighting");
    let light = run(
        "corpus/xfractint/3d-lighting",
        &root().join("tools/blackbox/3d-lighting/inputs"),
        &l,
    );
    let count = |cases: &[(String, String, (f64, f64))], f: &dyn Fn(f64, f64) -> bool| {
        cases.iter().filter(|(_, _, (e, n))| f(*e, *n)).count()
    };
    let exact = |e: f64, _: f64| e >= 0.9999;
    let near = |_: f64, n: f64| n >= 0.99;
    eprintln!(
        "geometry: {} cases, {} exact, {} within one index; lighting: {} cases, {} exact, {} within one index",
        geometry.len(),
        count(&geometry, &exact),
        count(&geometry, &near),
        light.len(),
        count(&light, &exact),
        count(&light, &near)
    );
    assert!(geometry.len() > 280 && light.len() > 550);
    assert!(count(&geometry, &exact) >= 210);
    assert!(count(&geometry, &near) >= 255);
    assert!(count(&light, &near) >= 550);
    // The default settings and each plain fill type match exactly.
    for name in [
        "m3d-def", "flat5", "o-def", "fill0", "fill1", "fill3", "fill4", "fill-1", "blk3d",
    ] {
        let (_, _, (e, _)) = geometry
            .iter()
            .find(|(n, _, _)| n == name)
            .unwrap_or_else(|| panic!("{name} missing"));
        assert!(*e >= 0.9999, "{name}: {:.3}%", e * 100.0);
    }
}

/// Our block for a 3D output equals XFractint's, apart from the time and
/// the random seed (3d-transform.md §7).
#[test]
fn writes_xfractints_block() {
    use fractalforge_formats::gif::fractint_blocks;
    let t = root().join("corpus/xfractint/3d-transform");
    let read = |p: PathBuf| GifImage::read(&std::fs::read(p).unwrap()).unwrap();
    for (input, output) in [("mandel.gif", "m3d-def"), ("flat5.gif", "flat5")] {
        let src = read(t.join("in").join(input));
        let theirs = read(t.join("out").join(format!("{output}.gif")));
        let main = |g: &GifImage| {
            g.extension(fractint_blocks::IDENTIFIER, fractint_blocks::MAIN_AUTH)
                .map(|e| e.data())
        };
        let ours = fractint_blocks::write_3d(main(&src).as_deref(), &Settings::default()).data();
        let theirs = main(&theirs).unwrap();
        let differ: Vec<usize> = (0..ours.len())
            .filter(|&i| {
                ours[i] != theirs[i] && !(98..102).contains(&i) && !(214..218).contains(&i)
            })
            .collect();
        assert!(differ.is_empty(), "{output}: bytes {differ:?} differ");
    }
}
