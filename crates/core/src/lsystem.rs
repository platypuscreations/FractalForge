// SPDX-License-Identifier: GPL-3.0-or-later

//! L-systems from Fractint-format `.l` files, following
//! docs/specs/lsystem.md (cited as §n). The arithmetic is the spec's
//! exactly (§6.2): turtle positions in extended precision ([`Ext`]), the
//! scale and aspect factor in single precision.

use crate::ext::{self, Ext};
use crate::plot::Plotter;
use crate::program::{Carry, FractalProgram, PixelContext};
use crate::{IterBuffer, Params, Symmetry};

/// Prefix of L-systems embedded in `.par` files (lsystem-par.md).
pub const PAR_PREFIX: &str = "lsys:";

/// Longest rewritten string drawn, in symbols.
const MAX_SYMBOLS: usize = 1 << 27;

/// Characters that can't have rules (§2.4).
const RESERVED: &[u8] = b"+-|![]@<>/\\c";

/// An entry found in a `.l` file.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// Lowercase, as Fractint compares names.
    pub name: String,
    /// 1-based line of the header.
    pub line: usize,
    /// The entry's text from its header line to its closing line.
    pub text: String,
    /// The rest of the header line and the following lines.
    body: Vec<String>,
}

/// A parsed L-system.
#[derive(Clone, Debug, PartialEq)]
pub struct LSystem {
    /// Number of directions.
    pub angle: u32,
    pub axiom: Vec<u8>,
    /// Replacement per (lowercase) symbol.
    pub rules: Vec<Option<Vec<u8>>>,
}

/// The file's lines, long ones split after 255 characters (§2.2).
fn lines(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in text.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let chars: Vec<char> = raw.chars().collect();
        let mut chunks = chars.chunks(255).peekable();
        if chunks.peek().is_none() {
            out.push(String::new());
        }
        for c in chunks {
            out.push(c.iter().collect());
        }
    }
    out
}

/// Every entry of a file, in order (§2.3). An entry is found only if a `}`
/// follows its `{` before the next line with a `{`.
pub fn entries(text: &str) -> Vec<Entry> {
    let all = lines(text);
    let code: Vec<&str> = all
        .iter()
        .map(|l| l.split(';').next().unwrap_or(""))
        .collect();
    let mut out = Vec::new();
    for (i, line) in code.iter().enumerate() {
        let Some((head, rest)) = line.split_once('{') else {
            continue;
        };
        let head = head.trim().trim_start_matches('}').trim();
        let name = head
            .split(|c: char| c.is_whitespace() || c == '(' || c == '[')
            .next()
            .unwrap_or("")
            .to_lowercase();
        let close = if rest.contains('}') {
            Some(i)
        } else {
            code.iter()
                .enumerate()
                .skip(i + 1)
                .find(|(_, l)| l.contains('{') || l.contains('}'))
                .filter(|(_, l)| !l.contains('{'))
                .map(|(j, _)| j)
        };
        let Some(close) = close else {
            continue;
        };
        let mut body = vec![rest.to_owned()];
        body.extend(code[i + 1..].iter().map(|l| l.to_string()));
        out.push(Entry {
            name,
            line: i + 1,
            text: all[i..=close].join("\n"),
            body,
        });
    }
    out
}

/// Characters of a name that are compared (§2.7, hypothesis).
const NAME_CHARS: usize = 18;

/// The first entry named `name`, ignoring case and anything past 18
/// characters.
pub fn find<'a>(all: &'a [Entry], name: &str) -> Option<&'a Entry> {
    let key = |n: &str| {
        n.to_lowercase()
            .chars()
            .take(NAME_CHARS)
            .collect::<String>()
    };
    let name = key(name);
    all.iter().find(|e| key(&e.name) == name)
}

/// C `atoi`: optional sign, digits, anything after ignored.
fn atoi(s: &str) -> i64 {
    let (neg, digits) = match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    };
    let v = digits
        .bytes()
        .take_while(u8::is_ascii_digit)
        .fold(0i64, |v, d| {
            v.saturating_mul(10).saturating_add((d - b'0') as i64)
        });
    if neg { -v } else { v }
}

/// Reads an entry, with XFractint's messages for bad ones (§2.4–2.6).
pub fn parse(e: &Entry) -> Result<LSystem, String> {
    let mut errors = Vec::new();
    let mut angle = None;
    let mut axiom: Option<Vec<u8>> = None;
    let mut rules: Vec<Option<Vec<u8>>> = vec![None; 256];
    for (i, raw) in e.body.iter().enumerate() {
        let n = i + 1;
        let l = raw.trim().to_lowercase();
        if l.starts_with('}') {
            break;
        }
        let l = l.trim_start_matches('=').trim();
        if l.is_empty() {
            continue;
        }
        let blank = |c: char| c == ' ' || c == '\t';
        let key_end = l.find(|c: char| blank(c) || c == '=').unwrap_or(l.len());
        let key = &l[..key_end];
        let rest = l[key_end..].trim_start_matches(|c: char| blank(c) || c == '=');
        let val_end = rest.find(blank).unwrap_or(rest.len());
        let value = &rest[..val_end];
        let extra = rest[val_end..].trim_start_matches(blank);
        // The first two words still count (§2.6).
        if !extra.is_empty() {
            let word = extra.split(blank).next().unwrap_or("");
            errors.push(format!("Extra text after command line {n}: {word}"));
        }
        match key {
            "angle" => angle = Some(atoi(value)),
            "axiom" => axiom = Some(value.as_bytes().to_vec()).filter(|a| !a.is_empty()),
            k if k.len() == 1 => {
                let c = k.as_bytes()[0];
                if RESERVED.contains(&c) {
                    errors.push(format!(
                        "Syntax error line {n}: Redefined reserved symbol {k}"
                    ));
                } else {
                    // Several rules for a symbol are joined (§2.4).
                    rules[c as usize]
                        .get_or_insert_with(Vec::new)
                        .extend_from_slice(value.as_bytes());
                }
            }
            k => errors.push(format!("Syntax error line {n}: {k}")),
        }
    }
    if axiom.is_none() {
        errors.push("Error:  no axiom".into());
    }
    let angle = angle.filter(|a| (3..=50).contains(a));
    if angle.is_none() {
        errors.push("Error:  illegal or missing angle".into());
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(LSystem {
        angle: angle.unwrap() as u32,
        axiom: axiom.unwrap(),
        rules,
    })
}

fn strip_prefix_ci<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    s.get(..prefix.len())
        .filter(|p| p.eq_ignore_ascii_case(prefix))
        .map(|_| &s[prefix.len()..])
}

/// Loads entry `name` from a file's text as a program for the `lsystem`
/// type. An entry embedded in a `.par` file (`lsys:NAME`) is stored
/// without its prefix.
pub fn load(text: &str, name: &str) -> Result<crate::Program, String> {
    let all = entries(text);
    let e = find(&all, name).ok_or_else(|| format!("'{name}' file entry item not found"))?;
    let lsys = parse(e)?;
    let (name, source) = match strip_prefix_ci(name, PAR_PREFIX) {
        Some(bare) => {
            let at = e.text.to_lowercase().find(PAR_PREFIX).unwrap_or(0);
            let mut source = e.text.clone();
            source.replace_range(at..at + PAR_PREFIX.len(), "");
            (bare.to_owned(), source)
        }
        None => (name.to_owned(), e.text.clone()),
    };
    Ok(crate::Program::new(LSystemProgram { name, source, lsys }))
}

/// An L-system attached to [`crate::Params::program`].
pub struct LSystemProgram {
    name: String,
    source: String,
    pub lsys: LSystem,
}

impl FractalProgram for LSystemProgram {
    fn name(&self) -> &str {
        &self.name
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn new_carry(&self, _: &Params, _: u32, _: u32) -> Carry {
        Carry::default()
    }

    fn iterate(&self, _: &PixelContext, _: &Params, _: &mut Carry) -> u32 {
        0
    }

    fn order_dependent(&self) -> bool {
        false
    }

    fn symmetry(&self, _: &Params) -> Symmetry {
        Symmetry::None
    }

    fn plotter(&self, p: &Params, width: u32, height: u32) -> Option<Box<dyn Plotter>> {
        Some(match Drawing::new(&self.lsys, p, width, height) {
            Ok(d) => Box::new(d),
            Err(e) => Box::new(Failed(e)),
        })
    }
}

/// A drawing that couldn't start; draws nothing.
struct Failed(#[allow(dead_code)] String);

impl Plotter for Failed {
    fn run(&mut self, _: &mut IterBuffer, _: u64) {}
    fn is_done(&self) -> bool {
        true
    }
    fn progress(&self) -> f64 {
        1.0
    }
}

// ---- rewriting (§4) ------------------------------------------------------

/// Length of a command's argument at `s[i..]` (§5.2): `I`/`Q` letters for
/// `@`, then digits and dots.
fn arg_len(cmd: u8, s: &[u8]) -> usize {
    let letters = if cmd == b'@' {
        s.iter().take_while(|c| matches!(c, b'i' | b'q')).count()
    } else {
        0
    };
    letters
        + s[letters..]
            .iter()
            .take_while(|c| c.is_ascii_digit() || **c == b'.')
            .count()
}

/// Rewrites `order` times; the arguments of commands in `protect` are
/// copied unchanged.
fn expand(l: &LSystem, order: u32, protect: &[u8]) -> Result<Vec<u8>, String> {
    let mut s = l.axiom.clone();
    for _ in 0..order {
        let mut out = Vec::with_capacity(s.len() * 2);
        let mut i = 0;
        while i < s.len() {
            let c = s[i];
            if protect.contains(&c) {
                let n = 1 + arg_len(c, &s[i + 1..]);
                out.extend_from_slice(&s[i..i + n]);
                i += n;
                continue;
            }
            match &l.rules[c as usize] {
                Some(r) => out.extend_from_slice(r),
                None => out.push(c),
            }
            i += 1;
            if out.len() > MAX_SYMBOLS {
                return Err(format!(
                    "the L-system has more than {MAX_SYMBOLS} symbols at this order"
                ));
            }
        }
        if out == s {
            break;
        }
        s = out;
    }
    Ok(s)
}

/// The value of an argument: the longest `digits[.digits]` prefix (§5.2).
fn number(arg: &[u8]) -> f64 {
    let int = arg.iter().take_while(|c| c.is_ascii_digit()).count();
    let mut end = int;
    if arg.get(int) == Some(&b'.') {
        end += 1 + arg[int + 1..]
            .iter()
            .take_while(|c| c.is_ascii_digit())
            .count();
    }
    std::str::from_utf8(&arg[..end])
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(0.0)
}

// ---- the turtle (§5, §6) --------------------------------------------------

#[derive(Clone, Copy)]
struct State {
    x: Ext,
    y: Ext,
    k: usize,
    deg: f64,
    colour: u32,
    rev: bool,
    size: Ext,
}

struct Turtle {
    n: usize,
    /// Direction per index k (§6.3).
    table: Vec<(Ext, Ext)>,
    aspect: Ext,
    /// The degree heading's direction, for the last heading used.
    degree_cache: Option<(f64, (Ext, Ext))>,
}

impl Turtle {
    fn new(n: u32, aspect: Ext) -> Self {
        let step = Ext::from_f64(std::f64::consts::TAU) / Ext::from_f64(n as f64);
        let table = (0..n)
            .map(|k| {
                let (c, s) = ext::cos_sin(step * Ext::from_f64(k as f64));
                (c * aspect, s)
            })
            .collect();
        Self {
            n: n as usize,
            table,
            aspect,
            degree_cache: None,
        }
    }

    fn degree_direction(&mut self, deg: f64) -> (Ext, Ext) {
        if let Some((d, dir)) = self.degree_cache
            && d.to_bits() == deg.to_bits()
        {
            return dir;
        }
        let r = deg * std::f64::consts::PI / 180.0;
        let (c, s) = ext::cos_sin_f64(r);
        let dir = (Ext::from_f64(c) * self.aspect, Ext::from_f64(s));
        self.degree_cache = Some((deg, dir));
        dir
    }
}

/// What a pass does with the turtle's moves.
enum Pass {
    /// Pass 1: the extent of every position.
    Measure { min: (Ext, Ext), max: (Ext, Ext) },
    /// Pass 2: draw.
    Draw,
}

struct Walk {
    s: Vec<u8>,
    i: usize,
    st: State,
    stack: Vec<State>,
    /// Colour commands take effect (pass 2 only, §6.1).
    colours: bool,
    done: bool,
}

impl Walk {
    fn new(s: Vec<u8>, st: State, colours: bool) -> Self {
        Self {
            s,
            i: 0,
            st,
            stack: Vec::new(),
            colours,
            done: false,
        }
    }

    /// Runs up to `budget` symbols, reporting each segment (`draw`) and
    /// position.
    fn run(&mut self, t: &mut Turtle, budget: u64, mut visit: impl FnMut(&State, Ext, Ext, bool)) {
        let mut left = budget;
        while left > 0 && !self.done {
            let Some(&c) = self.s.get(self.i) else {
                self.done = true;
                return;
            };
            left -= 1;
            self.i += 1;
            let st = &mut self.st;
            match c {
                b'f' | b'g' | b'd' | b'm' => {
                    let (dx, dy) = if matches!(c, b'f' | b'g') {
                        t.table[st.k]
                    } else {
                        t.degree_direction(st.deg)
                    };
                    let nx = st.x + st.size * dx;
                    let ny = st.y + st.size * dy;
                    visit(st, nx, ny, matches!(c, b'f' | b'd'));
                    st.x = nx;
                    st.y = ny;
                }
                b'+' => st.k = (st.k + if st.rev { 1 } else { t.n - 1 }) % t.n,
                b'-' => st.k = (st.k + if st.rev { t.n - 1 } else { 1 }) % t.n,
                b'|' => st.k = (st.k + t.n / 2) % t.n,
                b'!' => st.rev = !st.rev,
                b'/' | b'\\' => {
                    let n = arg_len(c, &self.s[self.i..]);
                    let v = number(&self.s[self.i..self.i + n]);
                    self.i += n;
                    if (c == b'/') != st.rev {
                        st.deg += v;
                    } else {
                        st.deg -= v;
                    }
                }
                b'@' => {
                    let n = arg_len(c, &self.s[self.i..]);
                    let arg = &self.s[self.i..self.i + n];
                    self.i += n;
                    let letters = arg.iter().take_while(|c| matches!(c, b'i' | b'q')).count();
                    let mut v = number(&arg[letters..]);
                    if arg[..letters].contains(&b'q') {
                        v = v.sqrt();
                    }
                    st.size = if arg[..letters].contains(&b'i') {
                        if v == 0.0 {
                            Ext::ZERO
                        } else {
                            st.size / Ext::from_f64(v)
                        }
                    } else {
                        st.size * Ext::from_f64(v)
                    };
                }
                b'c' | b'<' | b'>' if self.colours => {
                    let n = arg_len(c, &self.s[self.i..]);
                    let v = number(&self.s[self.i..self.i + n]).trunc();
                    self.i += n;
                    let v = (v.min(1e18) as u64 % 256) as u32;
                    st.colour = match c {
                        b'c' => v,
                        b'<' => match (st.colour + v) % 256 {
                            0 => 1,
                            x => x,
                        },
                        _ => match (st.colour + 256 - v) % 256 {
                            0 => 255,
                            x => x,
                        },
                    };
                }
                b'[' => self.stack.push(*st),
                b']' => match self.stack.pop() {
                    Some(saved) => *st = saved,
                    // §5.3: a `]` with nothing saved ends the pass.
                    None => self.done = true,
                },
                _ => {}
            }
        }
    }
}

/// The pixel of a coordinate: rounded half up (§6.5).
fn pixel(v: Ext) -> i64 {
    (v + Ext::from_f64(0.5)).floor()
}

/// Both passes over the rewritten string.
struct Drawing {
    turtle: Turtle,
    width: u32,
    height: u32,
    measure: Walk,
    extent: Pass,
    /// The drawing pass, once measuring is done.
    draw: Option<Walk>,
    /// Pass 2's string when it differs from pass 1's (§4).
    s2: Option<Vec<u8>>,
    total: f64,
}

impl Drawing {
    fn new(l: &LSystem, p: &Params, width: u32, height: u32) -> Result<Self, String> {
        let order = p.params[0].trunc().clamp(0.0, 1e6) as u32;
        let s1 = expand(l, order, b"@/\\")?;
        // Colour arguments are rewritten in pass 1 only when digits or dots
        // have rules (§4).
        let digit_rules = b"0123456789."
            .iter()
            .any(|&c| l.rules[c as usize].is_some());
        let s2 = digit_rules
            .then(|| expand(l, order, b"c<>@/\\"))
            .transpose()?;
        let aspect = ((width as f64 / height as f64) * 0.75) as f32;
        let zero = Ext::ZERO;
        let start = State {
            x: zero,
            y: zero,
            k: 0,
            deg: 0.0,
            colour: 15,
            rev: false,
            size: Ext::from_f64(1.0),
        };
        let total = (s1.len() + s2.as_ref().map_or(s1.len(), Vec::len)) as f64;
        Ok(Self {
            turtle: Turtle::new(l.angle, Ext::from_f64(aspect as f64)),
            width,
            height,
            measure: Walk::new(s1, start, false),
            extent: Pass::Measure {
                min: (zero, zero),
                max: (zero, zero),
            },
            draw: None,
            s2,
            total: total.max(1.0),
        })
    }

    /// Scale and start point from the extent (§6.4).
    fn start_drawing(&mut self) {
        let Pass::Measure { min, max } = self.extent else {
            return;
        };
        let (w, h) = (self.width as f64, self.height as f64);
        let bw = max.0 - min.0;
        let bh = max.1 - min.1;
        let mut fit: Option<Ext> = None;
        for (room, span) in [(w - 10.0, bw), (h - 6.0, bh)] {
            if span > Ext::ZERO {
                let s = Ext::from_f64(room) / span;
                fit = Some(match fit {
                    Some(f) if f < s => f,
                    _ => s,
                });
            }
        }
        let scale = Ext::from_f64(fit.map_or(1.0, |f| f.to_f64() as f32 as f64));
        let half = Ext::from_f64(0.5);
        let centre = |lo: Ext, hi: Ext, size: f64| {
            Ext::from_f64((size - 1.0) / 2.0) - (lo + hi) * half * scale
        };
        let st = State {
            x: centre(min.0, max.0, w),
            y: centre(min.1, max.1, h),
            k: 0,
            deg: 0.0,
            colour: 15,
            rev: false,
            size: scale,
        };
        let s = self
            .s2
            .take()
            .unwrap_or_else(|| std::mem::take(&mut self.measure.s));
        self.extent = Pass::Draw;
        self.draw = Some(Walk::new(s, st, true));
    }
}

impl Plotter for Drawing {
    fn run(&mut self, buf: &mut IterBuffer, budget: u64) {
        if self.draw.is_none() {
            let Pass::Measure { min, max } = &mut self.extent else {
                return;
            };
            self.measure.run(&mut self.turtle, budget, |_, x, y, _| {
                if x < min.0 {
                    min.0 = x;
                }
                if x > max.0 {
                    max.0 = x;
                }
                if y < min.1 {
                    min.1 = y;
                }
                if y > max.1 {
                    max.1 = y;
                }
            });
            if self.measure.done {
                self.start_drawing();
            }
            return;
        }
        let walk = self.draw.as_mut().unwrap();
        walk.run(&mut self.turtle, budget, |st, x, y, draw| {
            if draw {
                crate::plot::line(
                    buf,
                    (pixel(st.x), pixel(st.y)),
                    (pixel(x), pixel(y)),
                    st.colour,
                );
            }
        });
    }

    fn is_done(&self) -> bool {
        self.draw.as_ref().is_some_and(|d| d.done)
    }

    fn progress(&self) -> f64 {
        let first = if self.draw.is_some() {
            self.measure_len()
        } else {
            self.measure.i as f64
        };
        let second = self.draw.as_ref().map_or(0, |d| d.i) as f64;
        ((first + second) / self.total).min(1.0)
    }
}

impl Drawing {
    fn measure_len(&self) -> f64 {
        self.total - self.draw.as_ref().map_or(0, |d| d.s.len()) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "; c\nkoch {\n angle 6\n axiom F--F--F\n F=F+F--F+F\n}\nbad {\n angle 2\n FF=G\n + = F\n axiom F F\n}\n";

    #[test]
    fn reads_entries_and_errors() {
        let all = entries(FILE);
        let k = parse(find(&all, "KOCH").unwrap()).unwrap();
        assert_eq!(k.angle, 6);
        assert_eq!(k.axiom, b"f--f--f");
        assert_eq!(k.rules[b'f' as usize].as_deref(), Some(&b"f+f--f+f"[..]));
        let err = parse(find(&all, "bad").unwrap()).unwrap_err();
        assert_eq!(
            err,
            "Syntax error line 3: ff\nSyntax error line 4: Redefined reserved symbol +\n\
             Extra text after command line 5: f\nError:  illegal or missing angle"
        );
    }

    #[test]
    fn arguments_are_not_rewritten() {
        let l = parse(&entries("e {\nangle 4\naxiom F@2F\n2=FF\nF=F\n}")[0]).unwrap();
        assert_eq!(expand(&l, 2, b"@/\\").unwrap(), b"f@2f");
    }

    #[test]
    fn numbers() {
        assert_eq!(number(b"2.5.5"), 2.5);
        assert_eq!(number(b".5"), 0.5);
        assert_eq!(number(b"."), 0.0);
        assert_eq!(atoi("6x"), 6);
        assert_eq!(atoi("-4"), -4);
    }
}
