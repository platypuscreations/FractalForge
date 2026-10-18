// SPDX-License-Identifier: GPL-3.0-or-later

//! IFS images checked against XFractint (corpus/xfractint/ifs,
//! docs/specs/ifs.md).

use std::path::PathBuf;

use fractalforge_core::{color_index, ifs};
use fractalforge_formats::gif::{GifImage, fractint_blocks};
use fractalforge_render::{Control, PassMode, Render};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Pixels that differ from XFractint's after merging duplicate palette
/// entries (types.md §1.3), and the number of lit pixels in its image.
fn differences(gif: &PathBuf, file_text: &str, entry: &str) -> Result<(usize, usize), String> {
    let img = GifImage::read(&std::fs::read(gif).unwrap()).unwrap();
    let info = fractint_blocks::read(&img).ok_or("no Fractint block")?;
    let mut p = info.params.ok_or("no params")?;
    let canon: Vec<u8> = (0..256usize)
        .map(|i| {
            let c = img.palette.get(i);
            img.palette.iter().position(|x| Some(x) == c).unwrap_or(i) as u8
        })
        .collect();
    let lit = img
        .pixels
        .iter()
        .filter(|&&px| canon[px as usize] != 0)
        .count();
    let program = match ifs::load(file_text, entry) {
        Ok(prog) => prog,
        // XFractint saved an empty image after its message (§1.5).
        Err(_) => return Ok((lit, lit)),
    };
    p.program = Some(program);
    let mut r = Render::new(
        p.clone(),
        img.width.into(),
        img.height.into(),
        PassMode::One,
    )
    .map_err(|e| e.to_string())?;
    r.run(&Control::default());
    let differ = r
        .buffer()
        .data()
        .iter()
        .zip(&img.pixels)
        .filter(|&(&v, &px)| canon[color_index(v, &p) as usize] != canon[px as usize])
        .count();
    Ok((differ, lit))
}

#[test]
fn batch_corpus_matches_xfractint() {
    let dir = root().join("corpus/xfractint/ifs");
    let inputs = root().join("tools/blackbox/ifs/inputs");
    let cases = std::fs::read_to_string(dir.join("cases.txt")).unwrap();
    let (mut exact, mut checked, mut problems) = (0, 0, Vec::new());
    for line in cases.lines().filter(|l| !l.starts_with('#')) {
        let fields: Vec<&str> = line.split('|').map(str::trim).collect();
        let [case, "batch", args, ..] = fields[..] else {
            continue;
        };
        let gif = dir.join(format!("{case}.gif"));
        if !gif.exists() {
            continue;
        }
        let arg = |k: &str| args.split_whitespace().find_map(|a| a.strip_prefix(k));
        let (Some(file), entry) = (arg("ifsfile="), arg("ifs=").unwrap_or("")) else {
            continue;
        };
        // Not modelled (§6.6), or XFractint's own rounding at pixel
        // boundaries in long one-pixel lines (§3.4).
        // Reloads aren't calculations (§5).
        if args.contains("perspective=")
            || args.contains("stereo=")
            || args.contains("filename=")
            || case.starts_with("line-")
        {
            continue;
        }
        // A missing file found through other files in the directory (§2.1).
        let Ok(text) = std::fs::read_to_string(inputs.join(file)) else {
            continue;
        };
        checked += 1;
        match differences(&gif, &text, entry) {
            Ok((0, _)) => exact += 1,
            // §7: boundary pixels, at most 0.5 % of the lit ones.
            Ok((d, lit)) if d <= 8 || d * 200 <= lit => {}
            Ok((d, lit)) => problems.push(format!("{case}: {d} of {lit} lit pixels differ")),
            Err(e) => problems.push(format!("{case}: {e}")),
        }
    }
    assert!(checked > 170, "{checked}");
    assert!(exact * 10 >= checked * 8, "only {exact} of {checked} exact");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    eprintln!("{exact} of {checked} cases exact");
}

/// The full-size image from the main corpus matches exactly.
#[test]
fn fftri_matches_exactly() {
    let text = std::fs::read_to_string(root().join("tools/blackbox/inputs/ff.ifs")).unwrap();
    let gif = root().join("corpus/xfractint/complete/ifs-fftri.gif");
    assert_eq!(differences(&gif, &text, "fftri").unwrap().0, 0);
}

/// An unfinished IFS image is saved as interrupted, without resume data,
/// and its name block matches XFractint's.
#[test]
fn unfinished_save_matches_xfractint_blocks() {
    let dir = root().join("corpus/xfractint/ifs");
    let theirs = GifImage::read(&std::fs::read(dir.join("int-20.gif")).unwrap()).unwrap();
    let info = fractint_blocks::read(&theirs).unwrap();
    let mut p = info.params.unwrap();
    let text = std::fs::read_to_string(root().join("tools/blackbox/inputs/ff.ifs")).unwrap();
    let name = info.formula_name.unwrap();
    p.program = Some(ifs::load(&text, &name).unwrap());
    let (blocks, _) = fractint_blocks::write(&p, PassMode::One, Some((5, 0)), 400, 300, 0).unwrap();
    assert!(
        blocks
            .iter()
            .all(|b| &b.auth_code != fractint_blocks::RESUME_AUTH)
    );
    let main = blocks.last().unwrap().data();
    assert_eq!(i16::from_le_bytes([main[202], main[203]]), 3);
    let ours = blocks
        .iter()
        .find(|b| &b.auth_code == fractint_blocks::NAME_AUTH)
        .unwrap()
        .data();
    let theirs = theirs
        .extension(fractint_blocks::IDENTIFIER, fractint_blocks::NAME_AUTH)
        .unwrap()
        .data();
    assert_eq!(ours, theirs);
}

/// Entries of a `.par` file with embedded `ifs:` bodies render as
/// XFractint's (ifs.md §10.2).
#[test]
fn embedded_par_entries_match_xfractint() {
    use fractalforge_formats::par;
    let dir = root().join("tools/blackbox/ifs/par");
    let text = std::fs::read_to_string(dir.join("ip.par")).unwrap();
    let all = par::entries(&text);
    for (case, entry) in [
        ("par-enoname", "enoname"),
        ("par-eself", "eself"),
        ("par-eupper", "eupper"),
        ("par-e3d", "e3d"),
    ] {
        let loaded = par::load(par::find(&all, entry).unwrap(), Some(&dir), Some(&text)).unwrap();
        let p = loaded.params;
        let img = GifImage::read(
            &std::fs::read(root().join(format!("corpus/xfractint/ifs/{case}.gif"))).unwrap(),
        )
        .unwrap();
        let mut r = Render::new(
            p.clone(),
            img.width.into(),
            img.height.into(),
            PassMode::One,
        )
        .unwrap();
        r.run(&Control::default());
        let canon: Vec<u8> = (0..256usize)
            .map(|i| {
                let c = img.palette.get(i);
                img.palette.iter().position(|x| Some(x) == c).unwrap_or(i) as u8
            })
            .collect();
        let differ = r
            .buffer()
            .data()
            .iter()
            .zip(&img.pixels)
            .filter(|&(&v, &px)| canon[color_index(v, &p) as usize] != canon[px as usize])
            .count();
        assert_eq!(differ, 0, "{case}");
    }
}

/// Saved parameters embed the entry and load back to the same image
/// settings, 3D view included.
#[test]
fn par_round_trip() {
    use fractalforge_formats::par;
    let text =
        std::fs::read_to_string(root().join("tools/blackbox/ifs/inputs/ffleaf.ifs")).unwrap();
    for name in ["ffleaf", "ffspiral"] {
        let mut p = fractalforge_core::Params::for_type("ifs").unwrap();
        p.program = Some(ifs::load(&text, name).unwrap());
        p.params[0] = 1.0;
        p.view3d.rotation = [10, -20, 30];
        p.view3d.xyshift = [3, 4];
        let out = par::save_entry("", "img", None, &p, PassMode::One, None).unwrap();
        assert!(out.contains(&format!("ifs={name}")) && out.contains(&format!("ifs:{name}")));
        assert_eq!(
            out.contains("rotation=10/-20/30"),
            name == "ffspiral",
            "{out}"
        );
        let all = par::entries(&out);
        let back = par::load(par::find(&all, "img").unwrap(), None, Some(&out))
            .unwrap()
            .params;
        let mut want = p.clone();
        if name == "ffleaf" {
            // 2D entries don't save the 3D view.
            want.view3d = back.view3d;
        }
        assert_eq!(back, want, "{out}");
    }
}
