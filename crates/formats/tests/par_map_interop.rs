// SPDX-License-Identifier: GPL-3.0-or-later

//! .par and .map reading checked against what XFractint did with the same
//! files (corpus/xfractint/par-map, docs/specs/par.md and map.md).

use std::collections::HashMap;
use std::path::PathBuf;

use fractalforge_formats::gif::{GifImage, fractint_blocks};
use fractalforge_formats::{map, par};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn inputs() -> PathBuf {
    root().join("tools/blackbox/par-map/inputs")
}

/// case name → XFractint arguments, from cases.txt.
fn cases() -> HashMap<String, Vec<String>> {
    let text = std::fs::read_to_string(root().join("corpus/xfractint/par-map/cases.txt")).unwrap();
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let name = it.next()?.to_owned();
            Some((name, it.map(str::to_owned).collect()))
        })
        .collect()
}

/// case name → whether XFractint produced an image, from results-read.txt.
fn outcomes() -> HashMap<String, bool> {
    let text =
        std::fs::read_to_string(root().join("corpus/xfractint/par-map/results-read.txt")).unwrap();
    text.lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            Some((it.next()?.to_owned(), it.next()? == "image;"))
        })
        .collect()
}

fn read_gif(case: &str) -> Option<GifImage> {
    let path = root().join(format!("corpus/xfractint/par-map/read/{case}.gif"));
    std::fs::read(path)
        .ok()
        .map(|b| GifImage::read(&b).unwrap())
}

fn six_bit(p: &[[u8; 3]]) -> Vec<[u8; 3]> {
    p.iter().map(|c| c.map(|v| v >> 2)).collect()
}

/// A case of the form `@file.par/entry` (plus `viewwindows=2`).
fn single_entry(args: &[String]) -> Option<(String, String)> {
    let rest: Vec<&String> = args.iter().filter(|a| *a != "viewwindows=2").collect();
    match rest.as_slice() {
        [a] => {
            let (file, entry) = a.strip_prefix('@')?.split_once('/')?;
            Some((file.to_owned(), entry.to_owned()))
        }
        _ => None,
    }
}

#[test]
fn map_files_give_xfractints_palettes() {
    let mut checked = 0;
    for (case, args) in cases() {
        // Cases with exactly one map= (twice-loaded cases are covered by
        // the "later wins" rule and skipped here).
        let maps: Vec<&str> = args.iter().filter_map(|a| a.strip_prefix("map=")).collect();
        let [file] = maps.as_slice() else { continue };
        if args
            .iter()
            .any(|a| a.starts_with('@') || a.starts_with("colors="))
        {
            continue;
        }
        // map.md §2.3: XFractint's values for an unparsable first line are
        // leftovers from elsewhere; the spec says to use 0.
        if case == "m-mcommas" {
            continue;
        }
        let Some(img) = read_gif(&case) else { continue };
        let mut path = inputs().join(file);
        if path.extension().is_none() {
            path.set_extension("map");
        }
        let (pal, _) = map::read(&std::fs::read_to_string(&path).unwrap());
        assert_eq!(six_bit(&pal.0), six_bit(&img.palette), "{case}");
        checked += 1;
    }
    assert!(checked >= 15, "only {checked} map cases");
}

#[test]
fn par_entries_match_xfractint() {
    let outcomes = outcomes();
    let mut accepted = 0;
    let mut refused = 0;
    let mut problems = Vec::new();
    for (case, args) in cases() {
        let Some((file, entry)) = single_entry(&args) else {
            continue;
        };
        let Some(&made_image) = outcomes.get(&case) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(inputs().join(&file)) else {
            continue;
        };
        let all = par::entries(&text);
        let found = par::request_name_ok(&entry)
            .then(|| par::find(&all, &entry))
            .flatten();
        let loaded = found.map(|e| par::load(e, Some(&inputs())));
        if made_image {
            accepted += 1;
            let l = match loaded {
                Some(Ok(l)) => l,
                // Types FractalForge doesn't render yet.
                Some(Err(e)) if e.to_string().contains("isn't supported yet") => continue,
                _ => {
                    problems.push(format!("{case}: XFractint accepted, we couldn't load"));
                    continue;
                }
            };
            // Where we warned (rotated views, empty values), we knowingly
            // differ.
            if !l.warnings.is_empty() {
                continue;
            }
            let Some(img) = read_gif(&case) else { continue };
            let theirs = fractint_blocks::read(&img).and_then(|i| i.params);
            let Some(t) = theirs else { continue };
            let p = &l.params;
            if (p.fractal, p.window, p.params, p.max_iter, p.inside)
                != (t.fractal, t.window, t.params, t.max_iter, t.inside)
            {
                problems.push(format!("{case}: params differ: ours {p:?} theirs {t:?}"));
            }
            if let Some(pal) = &l.palette
                && six_bit(&pal.0) != six_bit(&img.palette)
            {
                problems.push(format!("{case}: palette differs"));
            }
        } else {
            refused += 1;
            let flagged = match &loaded {
                None => true,
                Some(Err(_)) => true,
                Some(Ok(l)) => l.xfractint_refuses,
            };
            if !flagged {
                problems.push(format!(
                    "{case}: XFractint refused {args:?}, we didn't flag it"
                ));
            }
        }
    }
    eprintln!("{accepted} accepted, {refused} refused cases checked");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
