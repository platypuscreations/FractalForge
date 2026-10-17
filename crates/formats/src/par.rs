// SPDX-License-Identifier: GPL-3.0-or-later

//! Fractint-format `.par` parameter files, following docs/specs/par.md
//! (cited as §n).
//!
//! Reading is tolerant (§9): where XFractint would refuse a whole entry
//! (unknown keys, bad values), we warn, skip the token and apply the rest,
//! and say that XFractint would refuse it.

use std::path::Path;

use fractalforge_core::{FractalType, Inside, Outside, Palette, Params, Window, types};
use fractalforge_render::PassMode;

use crate::Error;
use crate::gif::fractint_blocks::{inside_from_code, outside_from_code};
use crate::map::{self, expand6};

/// Entry names are matched on this many characters (§2.2).
const NAME_CHARS: usize = 18;
/// XFractint wraps body lines before this column (§7).
const LINE_LIMIT: usize = 72;

/// An entry as found in a file: its name, description and body text with
/// comments removed and continuation lines joined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawEntry {
    pub name: String,
    pub comment: Option<String>,
    pub body: String,
    /// Zero-based lines the entry spans, `{` line to `}` line.
    pub first_line: usize,
    pub last_line: usize,
    /// Structure problems XFractint wouldn't tolerate.
    pub warnings: Vec<String>,
}

/// Finds every entry in a file (§2.1, §9 steps 1–3).
pub fn entries(text: &str) -> Vec<RawEntry> {
    let lines: Vec<&str> = text
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let (code, comment) = split_comment(lines[i]);
        let Some(brace) = code.find('{') else {
            i += 1;
            continue;
        };
        let Some(name) = code[..brace].split_whitespace().next() else {
            i += 1;
            continue;
        };
        let mut warnings = Vec::new();
        if let Some(n) = (i..lines.len())
            .take_while(|&j| j == i || !lines[j - 1].contains('}'))
            .find(|&j| lines[j].len() > 512)
        {
            warnings.push(format!(
                "line {} is longer than 512 characters; XFractint splits such lines",
                n + 1
            ));
        }
        if name.chars().count() > NAME_CHARS {
            warnings.push(format!(
                "entry name longer than {NAME_CHARS} characters; XFractint crashes on such names"
            ));
        }
        let description = comment
            .map(|c| c.trim().to_owned())
            .filter(|c| !c.is_empty());
        let first_line = i;

        // Collect the body up to the first `}` outside a comment.
        let mut body = String::new();
        let mut rest = code[brace + 1..].to_owned();
        let mut last_line = i;
        let mut closed = false;
        loop {
            if let Some(end) = rest.find('}') {
                rest.truncate(end);
                closed = true;
            }
            if rest.contains('{') {
                warnings.push("'{' inside an entry; XFractint can't find such entries".into());
                rest = rest.replace('{', " ");
            }
            let trimmed = rest.trim_end();
            if let Some(joined) = trimmed.strip_suffix('\\').filter(|_| !closed) {
                // Continuation: the next line's leading whitespace is dropped
                // and nothing is inserted (§2.5).
                body.push_str(joined);
                last_line += 1;
                if last_line >= lines.len() {
                    break;
                }
                rest = split_comment(lines[last_line]).0.trim_start().to_owned();
                continue;
            }
            body.push_str(&rest);
            body.push('\n');
            if closed {
                break;
            }
            last_line += 1;
            if last_line >= lines.len() {
                warnings.push("entry has no closing '}'".into());
                break;
            }
            rest = split_comment(lines[last_line]).0.to_owned();
        }
        out.push(RawEntry {
            name: name.to_owned(),
            comment: description,
            body,
            first_line,
            last_line: last_line.min(lines.len().saturating_sub(1)),
            warnings,
        });
        // An entry's body can hold another `name {` line, which XFractint
        // finds as an entry of its own (§2.1), so keep scanning inside it.
        i += 1;
    }
    out
}

/// Whether XFractint can be asked for this entry name: it crashes on names
/// of 19 or more characters (§2.2).
pub fn request_name_ok(name: &str) -> bool {
    name.chars().count() <= NAME_CHARS
}

fn split_comment(line: &str) -> (&str, Option<&str>) {
    match line.find(';') {
        Some(i) => (&line[..i], Some(&line[i + 1..])),
        None => (line, None),
    }
}

fn name_key(name: &str) -> String {
    name.chars()
        .take(NAME_CHARS)
        .collect::<String>()
        .to_lowercase()
}

/// The first entry whose name matches (case-insensitive, first 18
/// characters).
pub fn find<'a>(entries: &'a [RawEntry], name: &str) -> Option<&'a RawEntry> {
    let key = name_key(name);
    entries.iter().find(|e| name_key(&e.name) == key)
}

/// Parameters loaded from an entry.
#[derive(Clone, Debug, PartialEq)]
pub struct Loaded {
    pub params: Params,
    pub pass_mode: Option<PassMode>,
    pub palette: Option<Palette>,
    /// Everything ignored, approximated or questionable.
    pub warnings: Vec<String>,
    /// XFractint would refuse this entry outright (§5.4).
    pub xfractint_refuses: bool,
}

/// Settings an entry names; `None` = not given (§3: an entry changes only
/// what it names, and `type=` doesn't reset what came before).
#[derive(Default)]
struct State {
    fractal: Option<&'static FractalType>,
    window: Option<Window>,
    params: Option<[f64; 4]>,
    max_iter: Option<u32>,
    bailout: Option<f64>,
    inside: Option<Inside>,
    outside: Option<Outside>,
    decomp: Option<u32>,
    biomorph: Option<Option<u32>>,
    use_symmetry: Option<bool>,
    pass_mode: Option<PassMode>,
    reset: bool,
}

/// Applies an entry on top of FractalForge's defaults. `dir` is where
/// `map=` and `colors=@` files are looked up (the `.par` file's folder).
pub fn load(entry: &RawEntry, dir: Option<&Path>) -> Result<Loaded, Error> {
    let mut st = State::default();
    let mut palette: Option<Palette> = None;
    let mut map_reference = false;
    let mut warnings = entry.warnings.clone();
    let mut refuses = !entry.warnings.is_empty();
    let mut bad = |msg: String, warnings: &mut Vec<String>| {
        warnings.push(format!("{msg} (XFractint would refuse this entry)"));
        refuses = true;
    };

    for token in entry.body.split_whitespace() {
        let (key, value) = match token.split_once('=') {
            Some((k, v)) => (k, Some(v)),
            None => (token, None),
        };
        if key != key.to_lowercase()
            || value.is_some_and(|v| is_keyword(key) && v != v.to_lowercase())
        {
            bad(
                format!("'{token}': XFractint accepts only lowercase"),
                &mut warnings,
            );
        }
        let key_l = key.to_lowercase();
        let v = value.unwrap_or("");
        let result: Result<(), String> = (|| {
            match key_l.as_str() {
                "reset" => {
                    if value.is_some() {
                        integer(v)?;
                    }
                    st = State {
                        reset: true,
                        ..State::default()
                    };
                }
                "type" => {
                    let t = types::lookup(v)
                        .ok_or_else(|| format!("type '{v}' isn't supported yet"))?;
                    st.fractal = Some(t);
                }
                "passes" => {
                    st.pass_mode = Some(match v.to_lowercase().as_str() {
                        "1" => PassMode::One,
                        "2" => PassMode::Two,
                        "g" => PassMode::Guess,
                        "3" | "b" | "t" | "s" | "d" | "o" => {
                            warnings.push(format!("passes={v} isn't supported; using guessing"));
                            PassMode::Guess
                        }
                        _ => return Err(format!("bad passes value '{v}'")),
                    });
                }
                "corners" => {
                    let c = numbers(v)?;
                    if c.len() != 4 && c.len() != 6 {
                        return Err("corners needs 4 or 6 values".into());
                    }
                    if c.len() == 6 && (c[4] != c[0] || c[5] != c[2]) {
                        warnings.push(
                            "rotated or skewed view isn't supported; using the corners".into(),
                        );
                    }
                    st.window = Some(Window::new(c[0], c[1], c[2], c[3]));
                }
                "center-mag" => {
                    let c = numbers(v)?;
                    if !(3..=6).contains(&c.len()) {
                        return Err("center-mag needs 3 to 6 values".into());
                    }
                    let (rot, skew) = (*c.get(4).unwrap_or(&0.0), *c.get(5).unwrap_or(&0.0));
                    if rot != 0.0 || skew != 0.0 {
                        warnings.push(
                            "rotated or skewed view isn't supported; ignoring rotation and skew"
                                .into(),
                        );
                    }
                    st.window = Some(center_mag(c[0], c[1], c[2], *c.get(3).unwrap_or(&1.0)));
                }
                "params" => {
                    let p = numbers(v)?;
                    if p.len() > 10 {
                        return Err("params takes at most 10 values".into());
                    }
                    if p.iter().skip(4).any(|&x| x != 0.0) {
                        warnings.push("parameters 5-10 are ignored".into());
                    }
                    let mut out = [0.0; 4];
                    for (o, x) in out.iter_mut().zip(&p) {
                        *o = *x;
                    }
                    st.params = Some(out);
                }
                "maxiter" => {
                    let n = integer(v)?;
                    if n < 1 {
                        return Err("maxiter must be positive".into());
                    }
                    st.max_iter = Some(n as u32);
                }
                "bailout" => {
                    let b = number(v)?;
                    if b > 0.0 {
                        st.bailout = Some(b);
                    } else {
                        st.bailout = None;
                    }
                }
                "bailoutest" => {
                    if !v.eq_ignore_ascii_case("mod") {
                        warnings.push(format!(
                            "bailoutest={v} isn't supported; using the modulus test"
                        ));
                    }
                }
                "inside" => st.inside = Some(parse_inside(v)?),
                "outside" => st.outside = Some(parse_outside(v)?),
                "decomp" => {
                    let d = numbers(v)?;
                    st.decomp = Some(d.first().ok_or("decomp needs a value")?.max(0.0) as u32);
                    if let Some(&b) = d.get(1).filter(|&&b| b > 0.0) {
                        st.bailout = Some(b);
                    }
                }
                "biomorph" => {
                    let b = integer(v)?;
                    st.biomorph = Some((b >= 0).then_some(b as u32));
                }
                "symmetry" => match v.to_lowercase().as_str() {
                    "none" => st.use_symmetry = Some(false),
                    other => {
                        warnings.push(format!("symmetry={other} isn't supported; using automatic"))
                    }
                },
                "colors" => {
                    palette = Some(match v.strip_prefix('@') {
                        Some(file) => {
                            map_reference = true;
                            load_map(file, dir, &mut warnings)?
                        }
                        None => decode_colors(v)?,
                    });
                }
                "map" => {
                    map_reference = true;
                    palette = Some(load_map(v, dir, &mut warnings)?);
                }
                // Settings with no effect in FractalForge.
                "float" | "periodicity" | "rseed" | "cyclerange" | "viewwindows" | "fillcolor" => {}
                "logmap" | "potential" | "invert" | "distest" | "finattract" | "function"
                | "formulafile" | "formulaname" | "olddemmcolors" | "nobof" => {
                    if !matches!(v.to_lowercase().as_str(), "" | "0" | "no" | "n" | "-1") {
                        warnings.push(format!("{key_l} isn't supported yet and was ignored"));
                    }
                }
                _ => return Err(format!("unknown setting '{token}'")),
            }
            Ok(())
        })();
        if let Err(e) = result {
            if e.starts_with("type '") {
                return Err(Error::Invalid(format!("entry {}: {e}", entry.name)));
            }
            bad(format!("{e}; skipped"), &mut warnings);
        }
    }

    if map_reference {
        bad(
            "palette file reference: XFractint looks for these only in its own maps directory"
                .into(),
            &mut warnings,
        );
    }
    let fractal = st.fractal.unwrap_or(&types::MANDEL);
    let mut p = Params::new(fractal);
    if let Some(w) = st.window {
        p.window = w;
    }
    if let Some(x) = st.params {
        p.params = x;
    }
    if let Some(m) = st.max_iter {
        p.max_iter = m;
    }
    // After `reset`, inside defaults to 1 (§3).
    p.inside = st.inside.unwrap_or(if st.reset {
        Inside::Color(1)
    } else {
        Inside::Color(0)
    });
    p.outside = st.outside.unwrap_or(Outside::Iter);
    p.decomp = st.decomp.unwrap_or(0);
    p.biomorph = st.biomorph.flatten();
    p.use_symmetry = st.use_symmetry.unwrap_or(true);
    p.bailout = st.bailout.unwrap_or_else(|| p.default_bailout());
    if p.needs_orbit_coloring() && !p.fractal.orbit_coloring {
        warnings.push(format!(
            "colouring options aren't supported for {} yet and were ignored",
            p.fractal.name
        ));
        p.outside = Outside::Iter;
        p.decomp = 0;
        p.biomorph = None;
        if !matches!(p.inside, Inside::Color(_) | Inside::MaxIter) {
            p.inside = Inside::MaxIter;
        }
    }
    p.validate()
        .map_err(|e| Error::Invalid(format!("entry {}: {e}", entry.name)))?;
    Ok(Loaded {
        params: p,
        pass_mode: st.pass_mode,
        palette,
        warnings,
        xfractint_refuses: refuses,
    })
}

fn is_keyword(key: &str) -> bool {
    matches!(
        key,
        "type" | "passes" | "inside" | "outside" | "symmetry" | "bailoutest"
    )
}

fn number(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| format!("'{s}' isn't a number"))
}

/// Integer keys accept fractions and exponents and truncate (§4).
fn integer(s: &str) -> Result<i64, String> {
    Ok(number(s)?.trunc() as i64)
}

/// A `/`-separated list; empty items are errors (§4).
fn numbers(s: &str) -> Result<Vec<f64>, String> {
    s.split('/').map(number).collect()
}

fn parse_inside(v: &str) -> Result<Inside, String> {
    if let Ok(n) = v.parse::<i64>() {
        if n >= 0 {
            return Ok(Inside::Color(
                fractalforge_core::color::wrap(n.min(u32::MAX as i64) as u32) as u8,
            ));
        }
        return inside_from_code(n as i16).ok_or_else(|| format!("inside={v} isn't supported"));
    }
    Inside::parse(&v.to_lowercase()).ok_or_else(|| format!("inside={v} isn't supported"))
}

fn parse_outside(v: &str) -> Result<Outside, String> {
    if let Ok(n) = v.parse::<i64>() {
        if (0..=255).contains(&n) {
            return Ok(Outside::Color(n as u8));
        }
        return outside_from_code(n as i16).ok_or_else(|| format!("outside={v} isn't supported"));
    }
    Outside::parse(&v.to_lowercase()).ok_or_else(|| format!("outside={v} isn't supported"))
}

/// `center-mag` without rotation or skew (§5.2): half height 1/mag, half
/// width (4/3)/(mag·xmag).
pub fn center_mag(cx: f64, cy: f64, mag: f64, xmag: f64) -> Window {
    let h = 1.0 / mag;
    let w = h * (4.0 / 3.0) / xmag;
    Window::new(cx - w, cx + w, cy - h, cy + h)
}

fn load_map(name: &str, dir: Option<&Path>, warnings: &mut Vec<String>) -> Result<Palette, String> {
    let mut file = std::path::PathBuf::from(name);
    if file.extension().is_none() {
        file.set_extension("map");
    }
    // XFractint looks only in its own maps directory (§6.5); we look next
    // to the .par file, then as given.
    let candidates = [
        dir.map(|d| d.join(file.file_name().unwrap_or_default())),
        Some(file.clone()),
    ];
    for path in candidates.into_iter().flatten() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            let (p, w) = map::read(&text);
            warnings.extend(w.into_iter().map(|w| format!("{}: {w}", path.display())));
            return Ok(p);
        }
    }
    Err(format!("can't find palette file {name}"))
}

/// The value of one `colors=` symbol (§6.1).
fn symbol(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'A'..=b'Z' => Some(c - b'A' + 10),
        b'_'..=b'z' => Some(c - b'_' + 36),
        _ => None,
    }
}

fn symbol_char(v: u8) -> char {
    (match v {
        0..=9 => b'0' + v,
        10..=35 => b'A' + v - 10,
        _ => b'_' + v - 36,
    }) as char
}

/// Decodes a `colors=` string (§6.7) into an 8-bit palette.
pub fn decode_colors(s: &str) -> Result<Palette, String> {
    let bytes = s.as_bytes();
    let mut out: Vec<[u8; 3]> = Vec::with_capacity(256);
    let mut pending = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let end = s[i..].find('>').ok_or("unclosed '<' in colors")? + i;
            let n: usize = s[i + 1..end]
                .parse()
                .map_err(|_| "bad run length in colors")?;
            if n < 2 || out.is_empty() {
                return Err("colors runs need n >= 2 and a colour before them".into());
            }
            pending = n;
            i = end + 1;
            continue;
        }
        let triple = bytes
            .get(i..i + 3)
            .ok_or("incomplete colour at the end of colors")?;
        let mut b = [0u8; 3];
        for (o, &c) in b.iter_mut().zip(triple) {
            *o = symbol(c).ok_or_else(|| format!("bad character '{}' in colors", c as char))?;
        }
        i += 3;
        if pending > 0 {
            let a = *out.last().unwrap();
            for k in 1..=pending {
                let mut e = [0u8; 3];
                for ch in 0..3 {
                    let (x, y) = (a[ch] as f64, b[ch] as f64);
                    e[ch] = (x + (y - x) * k as f64 / (pending + 1) as f64 + 0.5).floor() as u8;
                }
                out.push(e);
            }
            pending = 0;
        }
        out.push(b);
        if out.len() > 256 {
            return Err("colors has more than 256 entries".into());
        }
    }
    if pending > 0 {
        return Err("colors ends with a run".into());
    }
    let mut colors = [map::filler(); 256];
    for (c, e) in colors.iter_mut().zip(&out) {
        *c = e.map(expand6);
    }
    Ok(Palette(colors))
}

/// Encodes a palette as 256 explicit `colors=` entries (no runs, so it
/// loads back exactly; §8 step 6).
pub fn encode_colors(p: &Palette) -> String {
    p.0.iter()
        .flat_map(|c| c.map(|v| symbol_char(v >> 2)))
        .collect()
}

/// Formats a number so it reads back exactly.
fn num(v: f64) -> String {
    let plain = format!("{v}");
    if plain.len() <= 20 {
        plain
    } else {
        format!("{v:e}")
    }
}

/// Writes one entry (§7–§8).
pub fn write_entry(
    name: &str,
    comment: Option<&str>,
    p: &Params,
    pass_mode: PassMode,
    palette: Option<&Palette>,
) -> Result<String, Error> {
    let valid = !name.is_empty()
        && name.chars().count() <= NAME_CHARS
        && !name.chars().any(|c| c.is_whitespace() || "{};".contains(c));
    if !valid {
        return Err(Error::Invalid(format!(
            "entry names need 1-{NAME_CHARS} characters without spaces or {{ }} ;"
        )));
    }
    let mut tokens = vec!["reset=2004".to_owned(), format!("type={}", p.fractal.name)];
    match pass_mode {
        PassMode::Guess => {}
        PassMode::One => tokens.push("passes=1".into()),
        PassMode::Two => tokens.push("passes=2".into()),
    }
    let w = &p.window;
    tokens.push(format!(
        "corners={}/{}/{}/{}",
        num(w.x_min),
        num(w.x_max),
        num(w.y_min),
        num(w.y_max)
    ));
    let used = p.fractal.param_names.len().max(
        p.params
            .iter()
            .rposition(|&x| x != 0.0)
            .map_or(0, |i| i + 1),
    );
    if used > 0 {
        let list: Vec<String> = p.params[..used].iter().map(|&x| num(x)).collect();
        tokens.push(format!("params={}", list.join("/")));
    }
    if p.max_iter != 150 {
        tokens.push(format!("maxiter={}", p.max_iter));
    }
    if p.bailout != p.default_bailout() {
        tokens.push(format!("bailout={}", num(p.bailout)));
    }
    tokens.push(format!(
        "inside={}",
        match p.inside {
            Inside::Color(c) => c.to_string(),
            m => m.name().to_owned(),
        }
    ));
    match p.outside {
        Outside::Iter => {}
        Outside::Color(c) => tokens.push(format!("outside={c}")),
        m => tokens.push(format!("outside={}", m.name())),
    }
    if p.decomp != 0 {
        tokens.push(format!("decomp={}", p.decomp));
    }
    if let Some(b) = p.biomorph {
        tokens.push(format!("biomorph={b}"));
    }
    if !p.use_symmetry {
        tokens.push("symmetry=none".into());
    }

    let mut out = format!("{name:<19}{{");
    if let Some(c) = comment.filter(|c| !c.is_empty()) {
        out.push_str(" ; ");
        out.push_str(&c.replace(['\n', '\r'], " "));
    }
    out.push('\n');
    let mut line = String::from(" ");
    for t in &tokens {
        if line.len() + 1 + t.len() > LINE_LIMIT && line.len() > 1 {
            out.push_str(&line);
            out.push('\n');
            line = String::from(" ");
        }
        line.push(' ');
        line.push_str(t);
    }
    out.push_str(&line);
    out.push('\n');
    if let Some(pal) = palette {
        let colors = format!("colors={}", encode_colors(pal));
        let chunks: Vec<&[u8]> = colors.as_bytes().chunks(LINE_LIMIT).collect();
        for (i, chunk) in chunks.iter().enumerate() {
            out.push_str("  ");
            out.push_str(std::str::from_utf8(chunk).unwrap());
            out.push_str(if i + 1 < chunks.len() { "\\\n" } else { "\n" });
        }
    }
    out.push_str("  }\n\n");
    Ok(out)
}

/// Adds `entry_text` to a file, replacing an entry of the same name in
/// place and keeping everything else byte for byte (§7).
pub fn upsert(file_text: &str, name: &str, entry_text: &str) -> String {
    let all = entries(file_text);
    if let Some(e) = find(&all, name) {
        let lines: Vec<&str> = file_text.split_inclusive('\n').collect();
        let mut out: String = lines[..e.first_line].concat();
        out.push_str(entry_text.trim_end_matches('\n'));
        out.push('\n');
        out.push_str(&lines[(e.last_line + 1).min(lines.len())..].concat());
        return out;
    }
    let mut out = file_text.to_owned();
    if !out.is_empty() && !out.ends_with("\n\n") {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(entry_text);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "stray text\r\n\
first   { ; a description\n\
  type=julia params=-0.8/\\\n\
      0.156 maxiter=200 ; comment } with brace\n\
  corners=-1.5/1.5/-1.125/1.125\n\
  }\n\
\n\
Second { reset type=mandel maxiter=77;maxiter=998\n\
}\n";

    #[test]
    fn finds_and_reads_entries() {
        let all = entries(FILE);
        assert_eq!(
            all.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
            ["first", "Second"]
        );
        assert_eq!(all[0].comment.as_deref(), Some("a description"));
        assert_eq!((all[0].first_line, all[0].last_line), (1, 5));
        let l = load(find(&all, "FIRST").unwrap(), None).unwrap();
        assert!(l.warnings.is_empty(), "{:?}", l.warnings);
        assert_eq!(l.params.fractal.name, "julia");
        assert_eq!(l.params.params[..2], [-0.8, 0.156]);
        assert_eq!(l.params.max_iter, 200);
        assert_eq!(l.params.window, Window::new(-1.5, 1.5, -1.125, 1.125));
        assert_eq!(l.params.inside, Inside::Color(0));
        let s = load(find(&all, "second").unwrap(), None).unwrap();
        assert_eq!(s.params.max_iter, 77);
        // After reset, inside defaults to 1.
        assert_eq!(s.params.inside, Inside::Color(1));
    }

    #[test]
    fn tolerant_of_what_xfractint_refuses() {
        let all = entries("e { type=julia frobnicate=7 maxiter=abc MaxIter=9 }");
        let l = load(&all[0], None).unwrap();
        assert!(l.xfractint_refuses);
        assert_eq!(l.warnings.len(), 3, "{:?}", l.warnings);
        assert_eq!(l.params.max_iter, 9);
        assert!(load(&entries("e { type=nosuch }")[0], None).is_err());
    }

    #[test]
    fn colors_runs_and_filler() {
        let p = decode_colors("000<2>zzz").unwrap();
        // 63/3 = 21, 2·63/3 = 42.
        assert_eq!(p.0[1], [expand6(21); 3]);
        assert_eq!(p.0[2], [expand6(42); 3]);
        assert_eq!(p.0[3], [expand6(63); 3]);
        assert_eq!(p.0[4], map::filler());
        assert_eq!(
            decode_colors("ABCabc_`z").unwrap().0[1],
            [expand6(38), expand6(39), expand6(40)]
        );
        assert!(decode_colors("<3>zzz").is_err());
        assert!(decode_colors("000<1>zzz").is_err());
        assert!(decode_colors("00").is_err());
        assert!(decode_colors("0[0").is_err());
    }

    #[test]
    fn write_then_read() {
        let mut p = Params::for_type("julia").unwrap();
        p.params = [-0.8, 0.156, 0.0, 0.0];
        p.window = Window::new(-1.5, 1.5, -1.0e-20, 1.25);
        p.max_iter = 1000;
        p.outside = Outside::Real;
        p.decomp = 4;
        p.use_symmetry = false;
        let pal = Palette::forge();
        let text =
            write_entry("wjulia", Some("from tests"), &p, PassMode::One, Some(&pal)).unwrap();
        assert!(text.starts_with("wjulia             { ; from tests\n  reset=2004 type=julia"));
        assert!(text.lines().all(|l| l.len() <= 75), "{text}");
        let all = entries(&text);
        let l = load(&all[0], None).unwrap();
        assert!(!l.xfractint_refuses, "{:?}", l.warnings);
        assert_eq!(l.params, p);
        assert_eq!(l.pass_mode, Some(PassMode::One));
        let back = l.palette.unwrap();
        for (a, b) in pal.0.iter().zip(back.0.iter()) {
            assert_eq!(a.map(|v| v >> 2), b.map(|v| v >> 2));
        }
    }

    #[test]
    fn upsert_replaces_in_place() {
        let p = Params::for_type("mandel").unwrap();
        let e = write_entry("first", None, &p, PassMode::Guess, None).unwrap();
        let out = upsert(FILE, "FIRST", &e);
        assert!(out.starts_with("stray text\r\nfirst              {\n"));
        assert!(out.contains("Second { reset"));
        assert_eq!(entries(&out).len(), 2);
        let appended = upsert(FILE, "third", &e.replace("first ", "third "));
        assert_eq!(entries(&appended).len(), 3);
        assert!(appended.starts_with(FILE));
    }
}
