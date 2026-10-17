// SPDX-License-Identifier: GPL-3.0-or-later

//! FractalForge batch renderer.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use clap::{Args, Parser, Subcommand, ValueEnum};
use fractalforge_core::{Complex, Inside, Outside, PRESETS, Palette, Params, Window, types};
use fractalforge_formats::gif::{self, Checkpoint, Content};
use fractalforge_formats::par;
use fractalforge_render::{Control, Outcome, PassMode, Render};

#[derive(Parser)]
#[command(
    name = "fractalforge-cli",
    version,
    about = "FractalForge batch renderer"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Render an image, optionally starting from or resuming a saved GIF.
    Render(Box<RenderArgs>),
    /// Describe a saved GIF.
    Info { file: PathBuf },
    /// List the available fractal types.
    Types,
    /// List the entries in a .par file.
    Entries { file: PathBuf },
    /// Compare the pixels (and iteration data, when both have it) of two
    /// GIFs. Fails if more than the tolerated share of pixels differ.
    Compare {
        a: PathBuf,
        b: PathBuf,
        /// Percentage of pixels allowed to differ.
        #[arg(long, value_name = "PERCENT", default_value = "0", value_parser = parse_percent)]
        tolerance: f64,
        /// Compare displayed colours at 6 bits per channel (as Fractint keeps
        /// them) instead of palette indices.
        #[arg(long)]
        colors: bool,
    },
}

#[derive(Args)]
struct RenderArgs {
    /// A saved GIF (FractalForge or Fractint) or a .par file to take
    /// parameters (and with --resume, progress) from.
    input: Option<PathBuf>,
    /// Entry to use from a .par INPUT (default: the first), or the entry
    /// name to write to a .par OUTPUT (default: the file name).
    #[arg(long)]
    entry: Option<String>,
    /// Output file: .gif (resumable, Fractint-compatible), .png, .par
    /// (parameters only; adds or replaces the entry) or .map (palette only).
    #[arg(short, long)]
    output: PathBuf,
    /// Description for a .par entry.
    #[arg(long)]
    comment: Option<String>,
    /// Palette from a Fractint-format .map file.
    #[arg(long, conflicts_with = "palette")]
    map: Option<PathBuf>,
    /// Continue the unfinished render saved in INPUT.
    #[arg(long, requires = "input")]
    resume: bool,
    /// Stop after this percentage of the work and save a resumable GIF.
    #[arg(long, value_name = "PERCENT", value_parser = parse_percent)]
    stop_after: Option<f64>,
    /// Image size.
    #[arg(long, value_name = "WxH", value_parser = parse_size)]
    size: Option<(u32, u32)>,
    #[arg(long, value_enum)]
    passes: Option<Passes>,
    /// Built-in palette (see `types` for the list).
    #[arg(long, value_parser = parse_palette)]
    palette: Option<Palette>,
    #[command(flatten)]
    fractal: FractalArgs,
}

#[derive(Args, Default)]
struct FractalArgs {
    /// Fractal type (see `types`).
    #[arg(long = "type")]
    fractal_type: Option<String>,
    /// Window corners.
    #[arg(long, value_name = "XMIN/XMAX/YMIN/YMAX", value_parser = parse_floats::<4>, allow_hyphen_values = true)]
    corners: Option<[f64; 4]>,
    /// Window centre and magnification (1 shows a height of 2).
    #[arg(long, value_name = "X/Y/MAG", conflicts_with = "corners", value_parser = parse_floats::<3>, allow_hyphen_values = true)]
    center_mag: Option<[f64; 3]>,
    /// Type parameters, e.g. the Julia constant.
    #[arg(long, value_name = "P1[/P2[/P3[/P4]]]", value_parser = parse_params, allow_hyphen_values = true)]
    params: Option<[f64; 4]>,
    #[arg(long)]
    maxiter: Option<u32>,
    /// Escape when |z|² exceeds this.
    #[arg(long)]
    bailout: Option<f64>,
    /// Inside colouring: a palette index, or maxiter, zmag, bof60, bof61,
    /// atan, fmod or epscross (mandel and julia only for the modes).
    #[arg(long, value_parser = parse_inside)]
    inside: Option<Inside>,
    /// Outside colouring: a palette index, or iter, real, imag, mult, summ,
    /// atan, fmod or tdis.
    #[arg(long, value_parser = parse_outside)]
    outside: Option<Outside>,
    /// Decomposition: colour by angle in this many sectors (0 = off).
    #[arg(long)]
    decomp: Option<u32>,
    /// Biomorph colour value (sets the default bailout to 100).
    #[arg(long)]
    biomorph: Option<u32>,
    /// Compute every pixel instead of copying mirrored ones.
    #[arg(long)]
    no_symmetry: bool,
}

impl FractalArgs {
    fn is_empty(&self) -> bool {
        self.fractal_type.is_none()
            && self.corners.is_none()
            && self.center_mag.is_none()
            && self.params.is_none()
            && self.maxiter.is_none()
            && self.bailout.is_none()
            && self.inside.is_none()
            && self.outside.is_none()
            && self.decomp.is_none()
            && self.biomorph.is_none()
            && !self.no_symmetry
    }

    /// Applies the overrides to `params` (or to the defaults of a new type).
    fn apply(&self, mut params: Params, aspect: f64) -> Result<Params, String> {
        if let Some(name) = &self.fractal_type {
            let t = types::lookup(name).ok_or_else(|| format!("unknown fractal type {name:?}"))?;
            if t != params.fractal {
                params = Params::new(t);
            }
        }
        if let Some(c) = &self.corners {
            params.window = Window::new(c[0], c[1], c[2], c[3]);
        }
        if let Some(c) = &self.center_mag {
            params.window = Window::from_center_mag(Complex::new(c[0], c[1]), c[2], aspect);
        }
        if let Some(p) = self.params {
            params.params = p;
        }
        if let Some(n) = self.maxiter {
            params.max_iter = n;
        }
        if let Some(b) = self.bailout {
            params.bailout = b;
        }
        if let Some(i) = self.inside {
            params.inside = i;
        }
        if let Some(o) = self.outside {
            params.outside = o;
        }
        if let Some(d) = self.decomp {
            params.decomp = d;
        }
        if let Some(b) = self.biomorph {
            params.biomorph = Some(b);
            if self.bailout.is_none() {
                params.bailout = params.default_bailout();
            }
        }
        if self.no_symmetry {
            params.use_symmetry = false;
        }
        params.validate()?;
        Ok(params)
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum Passes {
    #[value(name = "1")]
    One,
    #[value(name = "2")]
    Two,
    #[value(name = "g")]
    Guess,
}

impl From<Passes> for PassMode {
    fn from(p: Passes) -> Self {
        match p {
            Passes::One => PassMode::One,
            Passes::Two => PassMode::Two,
            Passes::Guess => PassMode::Guess,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Render(args) => render(*args),
        Command::Info { file } => info(&file),
        Command::Compare {
            a,
            b,
            tolerance,
            colors,
        } => compare(&a, &b, tolerance, colors),
        Command::Entries { file } => list_entries(&file),
        Command::Types => {
            for t in types::all() {
                println!("{:<10} {}", t.name, t.description);
            }
            let names: Vec<_> = PRESETS.iter().map(|(n, _)| n.to_lowercase()).collect();
            println!("\npalettes:  {}", names.join(", "));
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn load(path: &Path) -> Result<gif::Loaded, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    gif::load(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn compare(a: &Path, b: &Path, tolerance: f64, colors: bool) -> Result<(), String> {
    let (a, b) = (load(a)?, load(b)?);
    if (a.image.width, a.image.height) != (b.image.width, b.image.height) {
        return Err("images differ in size".into());
    }
    let n = a.image.pixels.len();
    let rgb6 = |img: &gif::GifImage, i: u8| img.palette.get(i as usize).map(|c| c.map(|v| v >> 2));
    let pixels = a
        .image
        .pixels
        .iter()
        .zip(&b.image.pixels)
        .filter(|&(&x, &y)| {
            if colors {
                rgb6(&a.image, x) != rgb6(&b.image, y)
            } else {
                x != y
            }
        })
        .count();
    let what = if colors { "colours" } else { "pixels" };
    println!(
        "{what}:{:pad$}{pixels} of {n} differ",
        "",
        pad = 11 - what.len()
    );
    let mut iter_diff = 0;
    if let (Content::Native(ca), Content::Native(cb)) = (&a.content, &b.content) {
        let (da, db) = (ca.buffer.data(), cb.buffer.data());
        iter_diff = da.iter().zip(db).filter(|(x, y)| x != y).count();
        println!("iterations: {iter_diff} of {n} differ");
    }
    if pixels.max(iter_diff) as f64 > tolerance * n as f64 {
        return Err("images differ".into());
    }
    Ok(())
}

/// What an input file can start a render from. (Made once per command, so
/// its size doesn't matter.)
#[allow(clippy::large_enum_variant)]
enum Source {
    Checkpoint(Box<Checkpoint>),
    /// Parameters from Fractint-format blocks or a .par entry: no usable
    /// progress.
    Fractint {
        params: Params,
        pass_mode: Option<PassMode>,
        size: Option<(u32, u32)>,
        finished: bool,
        palette: Option<Palette>,
        /// Pixels an unfinished Fractint image already has final.
        partial: Option<fractalforge_core::IterBuffer>,
    },
}

fn is_ext(path: &Path, ext: &str) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
}

fn load_par(path: &Path, entry: Option<&str>) -> Result<Source, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let all = par::entries(&text);
    let raw = match entry {
        Some(name) => par::find(&all, name)
            .ok_or_else(|| format!("{}: no entry named {name}", path.display()))?,
        None => all
            .first()
            .ok_or_else(|| format!("{}: no entries", path.display()))?,
    };
    let loaded = par::load(raw, path.parent()).map_err(|e| e.to_string())?;
    for w in &loaded.warnings {
        eprintln!("note: {}: {w}", raw.name);
    }
    Ok(Source::Fractint {
        params: loaded.params,
        pass_mode: loaded.pass_mode,
        size: None,
        finished: true,
        palette: loaded.palette,
        partial: None,
    })
}

fn list_entries(path: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    for e in par::entries(&text) {
        let kind = par::load(&e, path.parent()).map_or_else(
            |err| format!("({err})"),
            |l| l.params.fractal.name.to_owned(),
        );
        println!(
            "{:<19} {:<12} {}",
            e.name,
            kind,
            e.comment.as_deref().unwrap_or("")
        );
    }
    Ok(())
}

fn load_source(path: &Path, entry: Option<&str>) -> Result<Source, String> {
    if is_ext(path, "par") {
        return load_par(path, entry);
    }
    let loaded = load(path)?;
    let size = (loaded.image.width as u32, loaded.image.height as u32);
    match loaded.content {
        Content::Native(c) => Ok(Source::Checkpoint(c)),
        Content::Fractint(info) => {
            for w in &info.warnings {
                eprintln!("note: {w}");
            }
            let params = info.params.ok_or_else(|| {
                format!(
                    "{}: FractalForge can't render this Fractint image yet",
                    path.display()
                )
            })?;
            let mut colors = [[0u8; 3]; 256];
            for (c, p) in colors.iter_mut().zip(&loaded.image.palette) {
                *c = *p;
            }
            let partial = gif::fractint_blocks::partial_buffer(&loaded.image, params.fractal);
            Ok(Source::Fractint {
                params,
                pass_mode: Some(info.pass_mode),
                size: Some(size),
                finished: info.finished,
                palette: Some(Palette(colors)),
                partial,
            })
        }
        Content::Image => Err(format!(
            "{} has no fractal parameters FractalForge can read",
            path.display()
        )),
    }
}

fn render(args: RenderArgs) -> Result<(), String> {
    let source = args
        .input
        .as_deref()
        .map(|p| load_source(p, args.entry.as_deref()))
        .transpose()?;
    let mut palette = match &source {
        Some(Source::Checkpoint(c)) => Some(c.palette.clone()),
        Some(Source::Fractint { palette, .. }) => palette.clone(),
        None => None,
    };
    if let Some(path) = &args.map {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let (p, warnings) = fractalforge_formats::map::read(&text);
        for w in warnings {
            eprintln!("note: {}: {w}", path.display());
        }
        palette = Some(p);
    }
    if let Some(p) = &args.palette {
        palette = Some(p.clone());
    }
    let palette = palette.unwrap_or_default();

    let mut render = match source {
        Some(Source::Checkpoint(c)) if args.resume => {
            if !args.fractal.is_empty() || args.size.is_some() || args.passes.is_some() {
                return Err(
                    "--resume continues a render exactly; it can't change parameters, size or passes"
                        .into(),
                );
            }
            Render::resume(c.params, c.state, c.buffer).map_err(|e| e.to_string())?
        }
        Some(Source::Fractint {
            params,
            partial: Some(buf),
            ..
        }) if args.resume => {
            if !args.fractal.is_empty() || args.size.is_some() || args.passes.is_some() {
                return Err(
                    "--resume continues a render exactly; it can't change parameters, size or passes"
                        .into(),
                );
            }
            // Fractint's finished pixels are kept; the rest are computed in
            // one pass.
            Render::from_partial(params, buf, PassMode::One).map_err(|e| e.to_string())?
        }
        source => {
            let (base, size, file_mode) = match source {
                Some(Source::Checkpoint(c)) => {
                    let size = (c.buffer.width(), c.buffer.height());
                    (c.params, size, Some(c.state.pass_mode))
                }
                Some(Source::Fractint {
                    params,
                    pass_mode,
                    size,
                    finished,
                    ..
                }) => {
                    if args.resume && !finished {
                        eprintln!(
                            "note: this file's resume data can't be used; calculating from the start"
                        );
                    }
                    (params, size.unwrap_or((800, 600)), pass_mode)
                }
                None => {
                    let name = args.fractal.fractal_type.as_deref().unwrap_or("mandel");
                    let p = Params::for_type(name)
                        .ok_or_else(|| format!("unknown fractal type {name:?}"))?;
                    (p, (800, 600), None)
                }
            };
            let (w, h) = args.size.unwrap_or(size);
            let params = args.fractal.apply(base, w as f64 / h as f64)?;
            let mode = args
                .passes
                .map(PassMode::from)
                .or(file_mode)
                .unwrap_or(PassMode::Guess);
            if is_ext(&args.output, "par") || is_ext(&args.output, "map") {
                return write_settings(&args, &params, mode, &palette);
            }
            Render::new(params, w, h, mode).map_err(|e| e.to_string())?
        }
    };

    // Ctrl-C stops at the next batch boundary and still saves.
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancel);
    ctrlc::set_handler(move || flag.store(true, Ordering::Relaxed))
        .map_err(|e| format!("can't install Ctrl-C handler: {e}"))?;

    let control = Control {
        cancel: Some(&cancel),
        stop_after: args.stop_after,
    };
    let outcome = render.run(&control);
    let progress = render.progress();

    let state = render.checkpoint();
    let (params, _, buffer) = render.into_parts();
    let checkpoint = Checkpoint {
        params,
        state,
        buffer,
        palette,
    };
    let png = args
        .output
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("png"));
    let (bytes, notes) = if png {
        let bytes = fractalforge_formats::png::encode(
            &checkpoint.params,
            &checkpoint.buffer,
            &checkpoint.palette,
        )
        .map_err(|e| e.to_string())?;
        let mut notes = Vec::new();
        if outcome != Outcome::Complete {
            notes.push("PNG files can't be resumed; save as .gif to continue later".to_owned());
        }
        (bytes, notes)
    } else {
        let saved =
            gif::save(&checkpoint, outcome == Outcome::Complete).map_err(|e| e.to_string())?;
        (saved.bytes, saved.notes)
    };
    std::fs::write(&args.output, bytes).map_err(|e| format!("{}: {e}", args.output.display()))?;
    for n in &notes {
        eprintln!("note: {n}");
    }

    let secs = checkpoint.state.elapsed_us as f64 / 1e6;
    match outcome {
        Outcome::Complete => println!("complete in {secs:.2}s: {}", args.output.display()),
        Outcome::Stopped => println!(
            "stopped at {:.1}% after {secs:.2}s; resume with: fractalforge-cli render {} --resume -o OUT.gif",
            progress * 100.0,
            args.output.display()
        ),
    }
    Ok(())
}

/// Writes a .par entry or a .map palette instead of rendering.
fn write_settings(
    args: &RenderArgs,
    params: &Params,
    mode: PassMode,
    palette: &Palette,
) -> Result<(), String> {
    let out = &args.output;
    let text = if is_ext(out, "map") {
        fractalforge_formats::map::write(palette)
    } else {
        let stem = out
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = args
            .entry
            .clone()
            .unwrap_or_else(|| stem.chars().take(18).collect());
        let entry = par::write_entry(&name, args.comment.as_deref(), params, mode, Some(palette))
            .map_err(|e| e.to_string())?;
        let existing = std::fs::read_to_string(out).unwrap_or_default();
        par::upsert(&existing, &name, &entry)
    };
    std::fs::write(out, text).map_err(|e| format!("{}: {e}", out.display()))?;
    println!("wrote {}", out.display());
    Ok(())
}

fn print_params(p: &Params, pass_mode: PassMode) {
    let w = &p.window;
    println!("type:       {}", p.fractal.name);
    println!(
        "corners:    {}/{}/{}/{}",
        w.x_min, w.x_max, w.y_min, w.y_max
    );
    println!(
        "params:     {}/{}/{}/{}",
        p.params[0], p.params[1], p.params[2], p.params[3]
    );
    println!("maxiter:    {}", p.max_iter);
    println!("bailout:    {}", p.bailout);
    println!("inside:     {:?}", p.inside);
    println!("passes:     {pass_mode:?}");
}

fn info(path: &Path) -> Result<(), String> {
    let loaded = load(path)?;
    let img = &loaded.image;
    println!("image:      {}x{}", img.width, img.height);
    for ext in &img.extensions {
        println!(
            "extension:  {} {} ({} bytes, {:?})",
            String::from_utf8_lossy(&ext.identifier),
            String::from_utf8_lossy(&ext.auth_code),
            ext.data().len(),
            ext.placement
        );
    }
    match loaded.content {
        Content::Native(c) => {
            print_params(&c.params, c.state.pass_mode);
            println!("calc time:  {:.2}s", c.state.elapsed_us as f64 / 1e6);
            let r = Render::resume(c.params, c.state, c.buffer).map_err(|e| e.to_string())?;
            if r.is_complete() {
                println!("status:     complete");
            } else {
                println!("status:     {:.1}% done, resumable", r.progress() * 100.0);
            }
        }
        Content::Fractint(info) => {
            println!(
                "source:     Fractint-format blocks (type number {})",
                info.type_number
            );
            match &info.params {
                Some(p) => print_params(p, info.pass_mode),
                None => println!("type:       not supported yet"),
            }
            let status = if info.finished {
                "complete"
            } else {
                "unfinished"
            };
            println!("status:     {status}");
            for w in &info.warnings {
                println!("note:       {w}");
            }
        }
        Content::Image => println!("status:     plain image (no fractal data)"),
    }
    Ok(())
}

fn parse_palette(s: &str) -> Result<Palette, String> {
    PRESETS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(s))
        .map(|(_, keys)| Palette::from_keys(keys))
        .ok_or_else(|| format!("unknown palette {s:?}"))
}

fn parse_percent(s: &str) -> Result<f64, String> {
    let v: f64 = s
        .trim_end_matches('%')
        .parse()
        .map_err(|_| format!("not a percentage: {s}"))?;
    if (0.0..=100.0).contains(&v) {
        Ok(v / 100.0)
    } else {
        Err("percentage must be 0..=100".into())
    }
}

fn parse_size(s: &str) -> Result<(u32, u32), String> {
    let (w, h) = s
        .split_once(['x', 'X'])
        .ok_or("size must look like 800x600")?;
    let w: u32 = w.parse().map_err(|_| "bad width")?;
    let h: u32 = h.parse().map_err(|_| "bad height")?;
    if (1..=65535).contains(&w) && (1..=65535).contains(&h) {
        Ok((w, h))
    } else {
        Err("width and height must be 1..=65535".into())
    }
}

fn parse_floats<const N: usize>(s: &str) -> Result<[f64; N], String> {
    split_floats(s)?
        .try_into()
        .map_err(|_| format!("expected {N} values separated by '/'"))
}

/// One to four values; missing ones are 0.
fn parse_params(s: &str) -> Result<[f64; 4], String> {
    let v = split_floats(s)?;
    if !(1..=4).contains(&v.len()) {
        return Err("expected 1 to 4 values separated by '/'".into());
    }
    let mut out = [0.0; 4];
    out[..v.len()].copy_from_slice(&v);
    Ok(out)
}

fn split_floats(s: &str) -> Result<Vec<f64>, String> {
    s.split('/')
        .map(|p| {
            p.trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| format!("not a number: {p:?}"))
        })
        .collect()
}

fn parse_inside(s: &str) -> Result<Inside, String> {
    Inside::parse(s).ok_or_else(|| {
        "inside must be a palette index or maxiter, zmag, bof60, bof61, atan, fmod, epscross".into()
    })
}

fn parse_outside(s: &str) -> Result<Outside, String> {
    Outside::parse(s).ok_or_else(|| {
        "outside must be 0..=255 or iter, real, imag, mult, summ, atan, fmod, tdis".into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_render_options() {
        let cli = Cli::try_parse_from([
            "fractalforge-cli",
            "render",
            "-o",
            "x.gif",
            "--corners",
            "-2/1/-1.5/1.5",
            "--center-mag",
            "-0.5/0/2",
            "--params",
            "-0.8/0.156",
        ]);
        assert!(cli.is_err(), "corners and center-mag conflict");
        let cli = Cli::try_parse_from([
            "fractalforge-cli",
            "render",
            "-o",
            "x.gif",
            "--center-mag",
            "-0.5/0/2",
            "--params",
            "-0.8/0.156",
            "--size",
            "64x48",
            "--stop-after",
            "50%",
            "--inside",
            "maxiter",
        ])
        .unwrap();
        let Command::Render(r) = cli.command else {
            panic!()
        };
        assert_eq!(r.fractal.center_mag, Some([-0.5, 0.0, 2.0]));
        assert_eq!(r.fractal.params, Some([-0.8, 0.156, 0.0, 0.0]));
        assert_eq!(r.size, Some((64, 48)));
        assert_eq!(r.stop_after, Some(0.5));
        assert_eq!(r.fractal.inside, Some(Inside::MaxIter));
    }
}
