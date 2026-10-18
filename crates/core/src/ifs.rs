// SPDX-License-Identifier: GPL-3.0-or-later

//! Iterated function systems from Fractint-format `.ifs` files, following
//! docs/specs/ifs.md (cited as §n).

use crate::crand::CRand;
use crate::plot::Plotter;
use crate::program::{Carry, FractalProgram, PixelContext};
use crate::{IterBuffer, Params, Symmetry};

/// Most transforms in an entry (§1.3).
pub const MAX_TRANSFORMS: usize = 63;

/// An entry found in a `.ifs` file.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub name: String,
    pub three_d: bool,
    /// 1-based line of the header.
    pub line: usize,
    /// The entry's text from its header line to the line with its `}`.
    pub text: String,
    /// What's read as numbers: the lines after the header up to the `}`.
    body: String,
}

/// A parsed entry: 7 numbers per 2D transform, 13 per 3D one (§1.2).
#[derive(Clone, Debug, PartialEq)]
pub struct Ifs {
    pub three_d: bool,
    pub transforms: Vec<Vec<f64>>,
}

fn strip_comment(line: &str) -> &str {
    line.split(';').next().unwrap_or("")
}

/// Splits a header's text before `{` into the name and the 3D marker.
fn header(before_brace: &str) -> Option<(String, bool)> {
    let s = before_brace.trim();
    let end = s
        .find(|c: char| c.is_whitespace() || c == '(')
        .unwrap_or(s.len());
    let (name, rest) = s.split_at(end);
    if name.is_empty() {
        return None;
    }
    let marker: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
    Some((name.to_owned(), marker.eq_ignore_ascii_case("(3d)")))
}

/// Every entry of a file, in order (§1.1). An entry needs its `{` on the
/// header line and a `}` on a later line (or after numbers on one).
pub fn entries(text: &str) -> Vec<Entry> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = strip_comment(raw);
        let Some(brace) = line.find('{') else {
            continue;
        };
        let Some((name, three_d)) = header(&line[..brace]) else {
            continue;
        };
        // Everything after `{` on the header line is ignored.
        let mut body = String::new();
        let mut close = None;
        for (j, raw) in lines.iter().enumerate().skip(i + 1) {
            let l = strip_comment(raw);
            if let Some(k) = l.find('}') {
                body.push_str(&l[..k]);
                close = Some(j);
                break;
            }
            body.push_str(l);
            body.push('\n');
        }
        let Some(close) = close else {
            continue;
        };
        out.push(Entry {
            name,
            three_d,
            line: i + 1,
            text: lines[i..=close].join("\n"),
            body,
        });
    }
    out
}

/// The first entry named `name`, ignoring case.
pub fn find<'a>(all: &'a [Entry], name: &str) -> Option<&'a Entry> {
    all.iter().find(|e| e.name.eq_ignore_ascii_case(name))
}

fn number(tok: &str) -> Option<f64> {
    let ok = tok.chars().any(|c| c.is_ascii_digit())
        && tok
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | '+' | '-' | 'e' | 'E'));
    ok.then(|| tok.parse().ok()).flatten()
}

/// Reads an entry's numbers, with XFractint's messages for bad ones (§1.5).
pub fn parse(e: &Entry) -> Result<Ifs, String> {
    const INVALID: &str = "invalid IFS definition";
    let mut nums = Vec::new();
    for tok in e
        .body
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|t| !t.is_empty())
    {
        nums.push(number(tok).ok_or(INVALID)?);
    }
    if nums.is_empty() {
        return Err("Empty IFS definition".into());
    }
    let per = if e.three_d { 13 } else { 7 };
    if nums.len() > per * MAX_TRANSFORMS {
        return Err(format!("IFS definition has too many lines\n{INVALID}"));
    }
    if nums.len() % per != 0 {
        return Err(INVALID.into());
    }
    Ok(Ifs {
        three_d: e.three_d,
        transforms: nums.chunks(per).map(<[f64]>::to_vec).collect(),
    })
}

/// Loads the entry `name` from a file's text as a program for the `ifs`
/// type.
pub fn load(text: &str, name: &str) -> Result<crate::Program, String> {
    let all = entries(text);
    let e = find(&all, name).ok_or_else(|| format!("'{name}' file entry item not found"))?;
    let ifs = parse(e)?;
    Ok(crate::Program::new(IfsProgram {
        name: e.name.clone(),
        source: e.text.clone(),
        ifs,
    }))
}

/// An IFS entry attached to [`crate::Params::program`].
pub struct IfsProgram {
    name: String,
    source: String,
    pub ifs: Ifs,
}

impl FractalProgram for IfsProgram {
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
        Some(if self.ifs.three_d {
            Box::new(Chaos3d::new(&self.ifs, p, width, height))
        } else {
            Box::new(Chaos2d::new(&self.ifs, p, width, height))
        })
    }
}

/// Total points: 1024·maxiter + 1 (§3.3).
fn point_count(p: &Params) -> u64 {
    1024 * p.max_iter as u64 + 1
}

/// §3.5: a point this far from the image (in pixel indices) stops the
/// calculation.
const FAR: f64 = 10000.0;

/// How a hit changes a pixel (§3.6).
fn hit(buffer: &mut IterBuffer, col: f64, row: f64, by_transform: bool, k: usize) {
    let (w, h) = (buffer.width() as f64, buffer.height() as f64);
    if (0.0..w).contains(&col) && (0.0..h).contains(&row) {
        let (x, y) = (col as u32, row as u32);
        let v = if by_transform {
            k as u32 + 1
        } else {
            (buffer.get(x, y) + 1) % 16
        };
        buffer.set(x, y, v);
    }
}

/// 16.16 fixed point: float32, scaled, truncated toward zero (§3.1).
fn fixed(v: f64) -> i64 {
    ((v as f32) as f64 * 65536.0) as i64
}

/// The 2D chaos game in 16.16 fixed point (§3.7).
struct Chaos2d {
    /// a b c d e f per transform, fixed point.
    maps: Vec<[i64; 6]>,
    /// Running sums of the fixed-point weights.
    sums: Vec<i64>,
    rng: CRand,
    x: i64,
    y: i64,
    done: u64,
    total: u64,
    by_transform: bool,
    x_min: f64,
    y_max: f64,
    sx: f64,
    sy: f64,
    span_x: f64,
    span_y: f64,
}

impl Chaos2d {
    fn new(ifs: &Ifs, p: &Params, width: u32, height: u32) -> Self {
        let mut sum = 0;
        let w = p.window;
        Self {
            maps: ifs
                .transforms
                .iter()
                .map(|t| std::array::from_fn(|i| fixed(t[i])))
                .collect(),
            sums: ifs
                .transforms
                .iter()
                .map(|t| {
                    sum += fixed(t[6]);
                    sum
                })
                .collect(),
            rng: CRand::new(1),
            x: 0,
            y: 0,
            done: 0,
            total: point_count(p),
            by_transform: p.params[0].abs() >= 1.0,
            x_min: w.x_min,
            y_max: w.y_max,
            sx: (width - 1) as f64,
            sy: (height - 1) as f64,
            span_x: w.x_max - w.x_min,
            span_y: w.y_max - w.y_min,
        }
    }
}

impl Plotter for Chaos2d {
    fn run(&mut self, buffer: &mut IterBuffer, budget: u64) {
        let end = self.total.min(self.done + budget);
        while self.done < end {
            self.done += 1;
            let r = (self.rng.next_u31() & 0x7fff) as i64;
            let k = self
                .sums
                .iter()
                .position(|&c| 2 * r <= c)
                .unwrap_or(self.maps.len() - 1);
            let [a, b, c, d, e, f] = self.maps[k];
            let (x, y) = (self.x, self.y);
            self.x = a * x / 65536 + b * y / 65536 + e;
            self.y = c * x / 65536 + d * y / 65536 + f;
            let col = ((self.x as f64 / 65536.0 - self.x_min) * self.sx / self.span_x).floor();
            let row = ((self.y_max - self.y as f64 / 65536.0) * self.sy / self.span_y).floor();
            if col.abs() + row.abs() > FAR || !(col.is_finite() && row.is_finite()) {
                self.done = self.total;
                return;
            }
            hit(buffer, col, row, self.by_transform, k);
        }
    }

    fn is_done(&self) -> bool {
        self.done >= self.total
    }

    fn progress(&self) -> f64 {
        self.done as f64 / self.total as f64
    }
}

/// Points used only to centre a 3D attractor (§6.2).
const MEASURE: u64 = 100;

/// The 3D chaos game in double precision, rotated to the screen (§6).
struct Chaos3d {
    maps: Vec<Vec<f64>>,
    /// Running sums of the weights, times 2³¹ (§4.4).
    sums: Vec<f64>,
    rng: CRand,
    point: [f64; 3],
    /// Screen rows u, v of the rotation (§6.3).
    rot: [[f64; 3]; 2],
    /// Bounding box of the measuring points in (u, v).
    bbox: [f64; 4],
    centre: (f64, f64),
    shift: (f64, f64),
    done: u64,
    total: u64,
    by_transform: bool,
    window: crate::Window,
    sx: f64,
    sy: f64,
}

impl Chaos3d {
    fn new(ifs: &Ifs, p: &Params, width: u32, height: u32) -> Self {
        let mut sum = 0.0;
        let [a, b, c] = p.view3d.rotation.map(|d| (d as f64).to_radians());
        let rx = [
            [1.0, 0.0, 0.0],
            [0.0, a.cos(), a.sin()],
            [0.0, -a.sin(), a.cos()],
        ];
        let ry = [
            [b.cos(), 0.0, -b.sin()],
            [0.0, 1.0, 0.0],
            [b.sin(), 0.0, b.cos()],
        ];
        let rz = [
            [c.cos(), c.sin(), 0.0],
            [-c.sin(), c.cos(), 0.0],
            [0.0, 0.0, 1.0],
        ];
        let mul = |m: [[f64; 3]; 3], n: [[f64; 3]; 3]| -> [[f64; 3]; 3] {
            std::array::from_fn(|i| {
                std::array::from_fn(|j| (0..3).map(|k| m[i][k] * n[k][j]).sum())
            })
        };
        let r = mul(mul(rz, ry), rx);
        let w = p.window;
        let [sx, sy] = p.view3d.xyshift.map(|s| s as f64 / 100.0);
        Self {
            maps: ifs.transforms.clone(),
            sums: ifs
                .transforms
                .iter()
                .map(|t| {
                    sum += t[12];
                    sum * 2_147_483_648.0
                })
                .collect(),
            rng: CRand::new(1),
            point: [0.0; 3],
            rot: [r[0], r[1]],
            bbox: [
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ],
            centre: (0.0, 0.0),
            shift: (sx * (w.x_max - w.x_min), sy * (w.y_max - w.y_min)),
            done: 0,
            total: point_count(p),
            by_transform: p.params[0].abs() >= 1.0,
            window: w,
            sx: (width - 1) as f64,
            sy: (height - 1) as f64,
        }
    }
}

impl Plotter for Chaos3d {
    fn run(&mut self, buffer: &mut IterBuffer, budget: u64) {
        let end = self.total.min(self.done + budget);
        while self.done < end {
            self.done += 1;
            let r = self.rng.next_u31() as f64;
            let k = self
                .sums
                .iter()
                .position(|&c| r <= c)
                .unwrap_or(self.maps.len() - 1);
            let m = &self.maps[k];
            let [x, y, z] = self.point;
            self.point = std::array::from_fn(|i| {
                m[3 * i] * x + m[3 * i + 1] * y + m[3 * i + 2] * z + m[9 + i]
            });
            let dot = |row: &[f64; 3]| row.iter().zip(&self.point).map(|(a, b)| a * b).sum();
            let (u, v): (f64, f64) = (dot(&self.rot[0]), dot(&self.rot[1]));
            if self.done <= MEASURE {
                let b = &mut self.bbox;
                *b = [b[0].min(u), b[1].max(u), b[2].min(v), b[3].max(v)];
                if self.done == MEASURE {
                    self.centre = (
                        (b[0] + b[1]) / 2.0 - self.shift.0,
                        (b[2] + b[3]) / 2.0 - self.shift.1,
                    );
                }
                continue;
            }
            let w = &self.window;
            let col = ((u - self.centre.0 - w.x_min) * self.sx / (w.x_max - w.x_min)).trunc();
            let row = ((w.y_max - (v - self.centre.1)) * self.sy / (w.y_max - w.y_min)).trunc();
            if col.abs() + row.abs() > FAR || !(col.is_finite() && row.is_finite()) {
                self.done = self.total;
                return;
            }
            hit(buffer, col, row, self.by_transform, k);
        }
    }

    fn is_done(&self) -> bool {
        self.done >= self.total
    }

    fn progress(&self) -> f64 {
        self.done as f64 / self.total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "; test\nffkoch {\n  0.5 0 0 0.5 0 0 0.33 ; one\n 0.5 0 0 0.5 0.5 0 0.33\n0.5,0,0,0.5,0.25,0.5,0.34 }\nthree (3D) {\n 1 0 0 0 1 0 0 0 1 0 0 0 1\n}\nbad {\n 1 2 3\n}\nopen {\n 1 2\n";

    #[test]
    fn reads_entries() {
        let all = entries(FILE);
        let names: Vec<_> = all.iter().map(|e| (e.name.as_str(), e.three_d)).collect();
        assert_eq!(names, [("ffkoch", false), ("three", true), ("bad", false)]);
        let ifs = parse(find(&all, "FFKOCH").unwrap()).unwrap();
        assert_eq!(ifs.transforms.len(), 3);
        assert_eq!(ifs.transforms[2][4], 0.25);
        assert_eq!(parse(&all[1]).unwrap().transforms.len(), 1);
        assert_eq!(parse(&all[2]).unwrap_err(), "invalid IFS definition");
    }

    #[test]
    fn text_after_brace_is_ignored() {
        let all = entries("a { 9 9 9 9 9 9 9\n0 0 0 0 1 2 1\n}\n");
        assert_eq!(
            parse(&all[0]).unwrap().transforms,
            [vec![0., 0., 0., 0., 1., 2., 1.]]
        );
    }

    #[test]
    fn fixed_point_truncates_toward_zero() {
        assert_eq!(fixed(0.0001), 6);
        assert_eq!(fixed(-0.0001), -6);
        assert_eq!(fixed(0.99999999), 65536);
    }
}
