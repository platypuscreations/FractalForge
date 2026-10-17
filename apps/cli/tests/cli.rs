// SPDX-License-Identifier: GPL-3.0-or-later

//! End-to-end tests of the CLI: golden images and save/resume through GIF
//! files. Set `UPDATE_GOLDENS=1` to rewrite the golden images.

use std::path::{Path, PathBuf};
use std::process::Command;

use fractalforge_formats::gif::{self, Content};

/// Small renders covering each type and pass mode.
const GOLDEN_CASES: &[(&str, &[&str])] = &[
    ("mandel-guess", &["--passes", "g"]),
    (
        "mandel-zoom-one",
        &[
            "--passes",
            "1",
            "--center-mag",
            "-0.745/0.113/40",
            "--maxiter",
            "500",
        ],
    ),
    (
        "mandel-inside-maxiter",
        &["--passes", "2", "--inside", "maxiter", "--maxiter", "64"],
    ),
    (
        "julia-two",
        &["--type", "julia", "--params", "-0.8/0.156", "--passes", "2"],
    ),
    (
        "julia-guess-offcenter",
        &[
            "--type",
            "julia",
            "--corners",
            "-1.2/1.9/-0.7/1.3",
            "--passes",
            "g",
        ],
    ),
];

/// Share of pixels allowed to differ from a golden image, for floating-point
/// differences between platforms.
const TOLERANCE: f64 = 0.001;

fn cli(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_fractalforge-cli"))
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "fractalforge-cli {args:?} failed:\n{stdout}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn load(path: &Path) -> gif::Loaded {
    gif::load(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn golden_images() {
    let dir = temp_dir("golden");
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let update = std::env::var_os("UPDATE_GOLDENS").is_some();
    for (name, args) in GOLDEN_CASES {
        let out = dir.join(format!("{name}.gif"));
        let mut full = vec!["render", "-o", out.to_str().unwrap(), "--size", "160x120"];
        full.extend_from_slice(args);
        cli(&full);

        let golden = golden_dir.join(format!("{name}.gif"));
        if update {
            std::fs::copy(&out, &golden).unwrap();
            continue;
        }
        let (got, want) = (load(&out).image, load(&golden).image);
        assert_eq!((got.width, got.height), (want.width, want.height), "{name}");
        let diff = got
            .pixels
            .iter()
            .zip(&want.pixels)
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            diff as f64 <= TOLERANCE * got.pixels.len() as f64,
            "{name}: {diff} pixels differ from {}",
            golden.display()
        );
    }
}

/// Stopping, saving, reloading and resuming must reproduce an uninterrupted
/// render exactly.
#[test]
fn stop_and_resume_through_gif() {
    let dir = temp_dir("resume");
    let p = |n: &str| dir.join(n).to_str().unwrap().to_owned();
    for (name, args) in GOLDEN_CASES {
        let mut base = vec!["--size", "211x157"];
        base.extend_from_slice(args);

        let mut full = vec!["render", "-o"];
        let full_path = p(&format!("{name}-full.gif"));
        full.push(&full_path);
        full.extend_from_slice(&base);
        cli(&full);
        let expected = load(Path::new(&full_path));

        for pct in ["0", "10", "45", "80"] {
            let part = p(&format!("{name}-{pct}.gif"));
            let mut stop = vec!["render", "-o", &part, "--stop-after", pct];
            stop.extend_from_slice(&base);
            assert!(cli(&stop).contains("stopped"), "{name} at {pct}%");

            let done = p(&format!("{name}-{pct}-done.gif"));
            cli(&["render", &part, "--resume", "-o", &done]);
            let got = load(Path::new(&done));
            assert_eq!(
                got.image.pixels, expected.image.pixels,
                "{name} resumed from {pct}%"
            );
            let (Content::Native(a), Content::Native(b)) = (&got.content, &expected.content) else {
                panic!("missing FractalForge block");
            };
            assert_eq!(a.buffer, b.buffer, "{name} resumed from {pct}%");
            assert_eq!(a.state.stage, b.state.stage);
        }
    }
}

#[test]
fn rerender_from_saved_parameters() {
    let dir = temp_dir("rerender");
    let a = dir.join("a.gif");
    let b = dir.join("b.gif");
    cli(&[
        "render",
        "-o",
        a.to_str().unwrap(),
        "--size",
        "90x60",
        "--type",
        "julia",
        "--maxiter",
        "77",
    ]);
    // Taking parameters from a.gif, but at a different size and pass mode.
    cli(&[
        "render",
        a.to_str().unwrap(),
        "-o",
        b.to_str().unwrap(),
        "--size",
        "45x30",
        "--passes",
        "1",
    ]);
    let Content::Native(c) = load(&b).content else {
        panic!()
    };
    assert_eq!(c.params.fractal.name, "julia");
    assert_eq!(c.params.max_iter, 77);
    assert_eq!((c.buffer.width(), c.buffer.height()), (45, 30));
}

#[test]
fn resume_refuses_changes() {
    let dir = temp_dir("refuse");
    let a = dir.join("a.gif");
    cli(&[
        "render",
        "-o",
        a.to_str().unwrap(),
        "--size",
        "50x40",
        "--stop-after",
        "20",
    ]);
    let out = Command::new(env!("CARGO_BIN_EXE_fractalforge-cli"))
        .args([
            "render",
            a.to_str().unwrap(),
            "--resume",
            "--maxiter",
            "9",
            "-o",
            "/dev/null",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
}
