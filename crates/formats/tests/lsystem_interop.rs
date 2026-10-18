// SPDX-License-Identifier: GPL-3.0-or-later

//! L-system images and messages checked against XFractint
//! (corpus/xfractint/lsystem, docs/specs/lsystem.md).

use std::path::PathBuf;

use fractalforge_core::{color_index, lsystem};
use fractalforge_formats::gif::{GifImage, fractint_blocks};
use fractalforge_render::{Control, PassMode, Render};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The case's files: its GIF and stop message, in whichever round folder
/// holds them.
fn find_case(name: &str) -> Option<PathBuf> {
    let dir = root().join("corpus/xfractint/lsystem");
    std::fs::read_dir(&dir).ok()?.find_map(|e| {
        let d = e.ok()?.path();
        let has = |ext: &str| d.join(format!("{name}.{ext}")).exists();
        (has("gif") || has("stopmsg.txt")).then(|| d.join(name))
    })
}

#[test]
fn corpus_matches_xfractint() {
    let cases = std::fs::read_to_string(root().join("corpus/xfractint/lsystem/cases.txt")).unwrap();
    let inputs = root().join("tools/blackbox/lsystem/inputs");
    let (mut images, mut messages, mut problems) = (0, 0, Vec::new());
    for line in cases
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let mut it = line.split_whitespace();
        let name = it.next().unwrap();
        let args: Vec<&str> = it.collect();
        let arg = |k: &str| args.iter().find_map(|a| a.strip_prefix(k));
        let only = arg("only=").unwrap_or("");
        let (Some(file), Some(entry)) = (arg("lfile="), arg("lname=")) else {
            continue;
        };
        // XFractint's search of other files and its 8.3 names (§2.1) aren't
        // copied: only cases whose named file holds the entry count.
        if !only.split(',').any(|f| f.eq_ignore_ascii_case(file)) {
            continue;
        }
        let Some(base) = find_case(name) else {
            continue;
        };
        let Ok(text) = std::fs::read(inputs.join(file)) else {
            continue;
        };
        let text = String::from_utf8_lossy(&text);
        let msg = std::fs::read_to_string(base.with_extension("stopmsg.txt")).unwrap_or_default();
        let ours = lsystem::load(&text, entry);
        // Sizes above 800×600 only add a message (§2.6).
        let too_large = msg.contains("View window too large");
        if !msg.trim().is_empty() && !too_large {
            // Messages only for entries we find (the rest is file search),
            // and not after lines of about 1,000 characters, which crash
            // XFractint part-way (§2.2).
            if msg.contains("file entry item not found")
                || msg.contains("Oops")
                || text.lines().any(|l| l.len() >= 1000)
            {
                continue;
            }
            let want: Vec<&str> = msg
                .lines()
                .map(str::trim_end)
                .filter(|l| !l.is_empty())
                .collect();
            match &ours {
                Err(e) if e.lines().collect::<Vec<_>>() == want => messages += 1,
                Err(e) => problems.push(format!("{name}: XFractint {want:?}, we {e:?}")),
                Ok(_) => problems.push(format!("{name}: XFractint {want:?}, we drew it")),
            }
            continue;
        }
        let gif = base.with_extension("gif");
        let Ok(bytes) = std::fs::read(&gif) else {
            continue;
        };
        let img = GifImage::read(&bytes).unwrap();
        let mut p = fractint_blocks::read(&img).unwrap().params.unwrap();
        // Orders of 12 and more take minutes (§3).
        if p.params[0] >= 12.0 {
            continue;
        }
        let program = match ours {
            Ok(prog) => prog,
            // Found by XFractint's search of other files (§2.1).
            Err(e) if e.contains("file entry item not found") => continue,
            Err(e) => {
                problems.push(format!("{name}: XFractint drew it, we: {e}"));
                continue;
            }
        };
        p.program = Some(program);
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
        if differ == 0 {
            images += 1;
        } else {
            problems.push(format!("{name}: {differ} pixels differ"));
        }
    }
    eprintln!("{images} images and {messages} messages match");
    assert!(
        problems.is_empty(),
        "{} problems:\n{}",
        problems.len(),
        problems.join("\n")
    );
    assert!(images > 1100 && messages >= 50, "{images} {messages}");
}

/// `.par` entries with embedded `lsys:` L-systems render as XFractint's,
/// and saved entries load back (lsystem-par.md).
#[test]
fn par_entries() {
    use fractalforge_formats::par;
    let dir = root().join("tools/blackbox/ifs/par");
    let text = std::fs::read_to_string(dir.join("ip.par")).unwrap();
    let all = par::entries(&text);
    for (case, entry) in [
        ("lpar-enoname", "lenoname"),
        ("lpar-eself", "leself"),
        ("lpar-upper", "lupper"),
    ] {
        let p = par::load(par::find(&all, entry).unwrap(), Some(&dir), Some(&text))
            .unwrap()
            .params;
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
        let differ = r
            .buffer()
            .data()
            .iter()
            .zip(&img.pixels)
            .filter(|&(&v, &px)| {
                img.palette[color_index(v, &p) as usize] != img.palette[px as usize]
            })
            .count();
        assert_eq!(differ, 0, "{case}");
        // Saving embeds the L-system as lsys:NAME; loading gives it back.
        let out = par::save_entry("", "img", None, &p, PassMode::One, None).unwrap();
        assert!(out.contains("lname=") && out.contains("lsys:"), "{out}");
        let back = par::entries(&out);
        let q = par::load(par::find(&back, "img").unwrap(), None, Some(&out))
            .unwrap()
            .params;
        assert_eq!(q, p, "{out}");
    }
}
