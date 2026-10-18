// SPDX-License-Identifier: GPL-3.0-or-later

//! Formula images checked against XFractint (corpus/xfractint/frm-semantics,
//! docs/specs/frm-semantics.md and frm-syntax.md).

use std::path::PathBuf;

use fractalforge_core::{Params, color_index};
use fractalforge_formats::gif::{GifImage, fractint_blocks};
use fractalforge_formula::{Formula, parser};
use fractalforge_render::{Control, PassMode, Render};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Renders a formula GIF's own parameters with `frm_file`'s entry and
/// returns the share of pixels equal to XFractint's.
pub fn formula_match(gif: &PathBuf, frm_text: &str) -> Result<f64, String> {
    let img = GifImage::read(&std::fs::read(gif).unwrap()).unwrap();
    let info = fractint_blocks::read(&img).ok_or("no Fractint block")?;
    let name = info.formula_name.clone().ok_or("no formula name")?;
    let mut p: Params = info.params.ok_or("no params")?;
    let all = parser::entries(frm_text);
    let entry = parser::find(&all, &name).ok_or_else(|| format!("no entry {name}"))?;
    let parsed = parser::parse(entry).map_err(|e| format!("{name}: {}", e[0]))?;
    let f = Formula::compile(&parsed.ast, &entry.text)?;
    p.program = Some(f.into_program());
    let mut r = Render::new(
        p.clone(),
        img.width.into(),
        img.height.into(),
        PassMode::One,
    )
    .map_err(|e| e.to_string())?;
    r.run(&Control::default());
    let same = r
        .buffer()
        .data()
        .iter()
        .zip(&img.pixels)
        .filter(|&(&v, &px)| color_index(v, &p) == px)
        .count();
    Ok(same as f64 / img.pixels.len() as f64)
}

#[test]
fn semantics_corpus_matches_xfractint() {
    let frm = std::fs::read_to_string(root().join("tools/blackbox/frm-semantics/sem.frm")).unwrap();
    let dir = root().join("corpus/xfractint/frm-semantics");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".gif"))
        .collect();
    names.sort();
    let mut low = Vec::new();
    for n in &names {
        let stem = n.trim_end_matches(".gif");
        // Error cases: XFractint wrote a message and an all-colour-1 image.
        if dir.join(format!("{stem}.stopmsg.txt")).exists() {
            continue;
        }
        // Fractint's random generator is unknown (frm-semantics.md §2.1).
        if stem.contains("rand") || stem.starts_with("rnd") {
            continue;
        }
        match formula_match(&dir.join(n), &frm) {
            Ok(share) => {
                if share < 0.999 {
                    low.push(format!("{stem}: {:.3}%", share * 100.0));
                }
            }
            Err(e) => low.push(format!("{stem}: {e}")),
        }
    }
    assert!(names.len() > 300);
    assert!(low.is_empty(), "below 99.9%:\n{}", low.join("\n"));
}

/// What XFractint reported for a syntax case (`<case>.stopmsg.txt`).
fn reported_errors(msg: &str) -> Vec<(u8, usize)> {
    msg.lines()
        .filter_map(|l| {
            let rest = l.trim_start().strip_prefix("Error(")?;
            let (code, rest) = rest.split_once(')')?;
            let line = rest
                .trim_start()
                .strip_prefix("at line ")?
                .split(':')
                .next()?;
            Some((code.parse().ok()?, line.trim().parse().ok()?))
        })
        .collect()
}

/// Every syntax case: XFractint's error codes and lines, or its pixels.
#[test]
fn syntax_corpus_matches_xfractint() {
    let read = |p: PathBuf| std::fs::read(p).map(|b| String::from_utf8_lossy(&b).into_owned());
    let cases = read(root().join("corpus/xfractint/frm-syntax/cases.txt")).unwrap();
    let inputs = root().join("tools/blackbox/frm-syntax/inputs");
    let mut problems = Vec::new();
    let mut checked = (0, 0);
    for line in cases
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let mut it = line.split_whitespace();
        let case = it.next().unwrap();
        let args: Vec<&str> = it.collect();
        let arg = |k: &str| args.iter().find_map(|a| a.strip_prefix(k));
        let (Some(file), Some(name)) = (arg("formulafile="), arg("formulaname=")) else {
            continue;
        };
        if args.iter().any(|a| a.starts_with('@')) || name.contains("rand") {
            continue;
        }
        // Not parser behaviour, or knowingly different (frm-syntax.md):
        // names over 18 characters and 12 params are refused as arguments,
        // a Latin-1 name couldn't reach XFractint, XFractint silently fails
        // on 2,000 statements, `1e-` at the end is unknown, rotated views
        // (v3, v4) and ismand=no (v14) aren't supported, negp9 shows a
        // context-dependent variant of the `-(` quirk, and c5d is rendered
        // by XFractint with a later entry's body (FractalForge refuses it).
        const SKIP: [&str; 9] = [
            "s-long-m2000",
            "u-e18",
            "v-v3",
            "v-v4",
            "v-v14",
            "v-v19",
            "p-negp9",
            "t-brace-c5d",
            "t-latin1-name",
        ];
        if name.len() > 18 || SKIP.contains(&case) {
            continue;
        }
        let Ok(text) = read(inputs.join(file)) else {
            continue;
        };
        let folder = case.split('-').next().unwrap();
        let dir = root().join("corpus/xfractint/frm-syntax").join(folder);
        let gif = dir.join(format!("{case}.gif"));
        let msg = read(dir.join(format!("{case}.stopmsg.txt"))).unwrap_or_default();
        let all = fractalforge_formula::parser::entries(&text);
        let entry = fractalforge_formula::parser::find(&all, name);
        let ours = entry.map(fractalforge_formula::parser::parse);
        let theirs = reported_errors(&msg);
        if !theirs.is_empty() {
            checked.0 += 1;
            let got: Vec<(u8, usize)> = match &ours {
                Some(Err(errs)) => errs
                    .iter()
                    .map(|e| (e.code.unwrap_or(255), e.line))
                    .collect(),
                _ => Vec::new(),
            };
            if got != theirs {
                problems.push(format!("{case}: XFractint {theirs:?}, we {got:?}"));
            }
        } else if !msg.trim().is_empty() {
            // Other refusals: missing entry, no instructions, bad qualifier.
            checked.0 += 1;
            if matches!(ours, Some(Ok(_))) {
                problems.push(format!(
                    "{case}: XFractint said {:?}, we parsed it",
                    msg.lines().next()
                ));
            }
        } else if gif.exists() {
            checked.1 += 1;
            match formula_match(&gif, &text) {
                Ok(share) if share >= 0.999 => {}
                Ok(share) => {
                    problems.push(format!("{case}: {:.3}% of pixels match", share * 100.0))
                }
                Err(e) => problems.push(format!("{case}: {e}")),
            }
        }
    }
    assert!(checked.0 > 150 && checked.1 > 500, "{checked:?}");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The `fractint003` block we write for a formula equals XFractint's
/// (name and the p1–p5 / ismand flags, frm-syntax.md §8.1).
#[test]
fn name_block_matches_xfractint() {
    let read = |p: PathBuf| fractalforge_formula::read_text(&p).unwrap();
    let cases = read(root().join("corpus/xfractint/frm-syntax/cases.txt"));
    let mut checked = 0;
    for line in cases.lines() {
        let mut it = line.split_whitespace();
        let Some(case) = it.next() else { continue };
        if !(case.starts_with("r3-store-s") || case.starts_with("b-rp")) {
            continue;
        }
        let args: Vec<&str> = it.collect();
        let arg = |k: &str| args.iter().find_map(|a| a.strip_prefix(k)).unwrap();
        let gif = root().join(format!(
            "corpus/xfractint/frm-syntax/{}/{case}.gif",
            case.split('-').next().unwrap()
        ));
        let Ok(bytes) = std::fs::read(&gif) else {
            continue;
        };
        let img = GifImage::read(&bytes).unwrap();
        let theirs = img
            .extension(fractint_blocks::IDENTIFIER, fractint_blocks::NAME_AUTH)
            .unwrap()
            .data();
        let text = read(
            root()
                .join("tools/blackbox/frm-syntax/inputs")
                .join(arg("formulafile=")),
        );
        let (program, _) = fractalforge_formula::load(&text, arg("formulaname=")).unwrap();
        let mut p = fractint_blocks::read(&img).unwrap().params.unwrap();
        p.program = Some(program);
        let (blocks, _) = fractint_blocks::write(&p, PassMode::One, None, 400, 300, 0).unwrap();
        let ours = blocks
            .iter()
            .find(|b| &b.auth_code == fractint_blocks::NAME_AUTH)
            .unwrap()
            .data();
        assert_eq!(ours, theirs, "{case}");
        checked += 1;
    }
    assert!(checked >= 15, "{checked}");
}
