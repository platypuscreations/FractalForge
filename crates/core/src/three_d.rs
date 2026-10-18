// SPDX-License-Identifier: GPL-3.0-or-later

//! Fractint-style 3D transforms of an existing image: the colour index of
//! each pixel becomes a height, and the surface is projected (planar or
//! onto a sphere) and drawn as points, lines or triangles, optionally lit by
//! a light source and drawn as a red/blue stereo pair. Follows
//! docs/specs/3d-transform.md (cited as T§n) and docs/specs/3d-lighting.md
//! (L§n).

/// Settings of a 3D transform, with Fractint's defaults. Percentages are
/// as typed (90 = 90 %), angles in degrees.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub rotation: [f64; 3],
    /// x and y scale in percent.
    pub scale: [f64; 2],
    /// Height scale in percent (`roughness`).
    pub roughness: f64,
    pub waterline: u8,
    /// −1 coarse grid, 0 points, 1 lines, 2 interpolated and 3 flat
    /// triangles, 4 bars, 5 and 6 light source (T§6, L§2).
    pub fill: i8,
    /// Perspective distance in percent; 0 = none.
    pub perspective: f64,
    pub xyshift: [f64; 2],
    /// Colours `lo..=hi` aren't drawn; off when `hi` is 0 (T§2.3).
    pub transparent: (u8, u8),
    /// Spherical projection. Its longitude, latitude and radius share the
    /// rotation and scale fields, as in Fractint (T§5.1): see
    /// [`Settings::longitude`].
    pub sphere: bool,
    pub light: [f64; 3],
    pub smoothing: u32,
    /// Ambient light, 0..=100.
    pub ambient: u32,
    /// 0 none, 1 alternate pixels, 2 superimposed (L§8).
    pub stereo: u8,
    pub interocular: f64,
    pub converge: f64,
    pub crop: [f64; 4],
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            rotation: [60.0, 30.0, 0.0],
            scale: [90.0, 90.0],
            roughness: 30.0,
            waterline: 0,
            fill: 0,
            perspective: 0.0,
            xyshift: [0.0, 0.0],
            transparent: (0, 0),
            sphere: false,
            light: [1.0, -1.0, 1.0],
            smoothing: 0,
            ambient: 20,
            stereo: 0,
            interocular: 0.0,
            converge: 0.0,
            crop: [4.0, 0.0, 0.0, 4.0],
        }
    }
}

impl Settings {
    /// The sphere's longitude range (the rotation x and y fields).
    pub fn longitude(&self) -> [f64; 2] {
        [self.rotation[0], self.rotation[1]]
    }

    /// The sphere's latitude range (the rotation z and scale x fields).
    pub fn latitude(&self) -> [f64; 2] {
        [self.rotation[2], self.scale[0]]
    }

    /// The sphere's radius in percent (the scale y field).
    pub fn radius(&self) -> f64 {
        self.scale[1]
    }

    /// Applies one of Fractint's 3D command-line keys (`rotation=60/30/0`,
    /// `filltype=5`, …).
    pub fn apply(&mut self, key: &str, value: &str) -> Result<(), String> {
        let nums = || -> Result<Vec<f64>, String> {
            value
                .split('/')
                .map(|v| {
                    v.trim()
                        .parse::<f64>()
                        .map_err(|_| format!("{key}: bad number {v:?}"))
                })
                .collect()
        };
        let take = |dst: &mut [f64]| -> Result<(), String> {
            let v = nums()?;
            if v.len() > dst.len() {
                return Err(format!("{key} takes at most {} values", dst.len()));
            }
            dst[..v.len()].copy_from_slice(&v);
            Ok(())
        };
        let int = || -> Result<i64, String> {
            value
                .trim()
                .parse()
                .map_err(|_| format!("{key}: bad number {value:?}"))
        };
        match key {
            "rotation" => take(&mut self.rotation)?,
            "scalexyz" => {
                let mut v = [self.scale[0], self.scale[1], self.roughness];
                take(&mut v)?;
                (self.scale, self.roughness) = ([v[0], v[1]], v[2]);
            }
            "roughness" => self.roughness = nums()?[0],
            "waterline" => self.waterline = int()?.clamp(0, 255) as u8,
            "filltype" => {
                let f = int()?;
                if !(-1..=6).contains(&f) {
                    return Err("filltype must be -1 to 6".into());
                }
                self.fill = f as i8;
            }
            "perspective" => self.perspective = nums()?[0],
            "xyshift" => take(&mut self.xyshift)?,
            "transparent" => {
                let mut v = [0.0, 0.0];
                take(&mut v)?;
                self.transparent = (v[0].clamp(0.0, 255.0) as u8, v[1].clamp(0.0, 255.0) as u8);
            }
            "sphere" => self.sphere = matches!(value.to_ascii_lowercase().as_str(), "y" | "yes"),
            // Stored in the rotation and scale fields (T§5.1).
            "longitude" => take(&mut self.rotation[..2])?,
            "latitude" => {
                let mut v = [self.rotation[2], self.scale[0]];
                take(&mut v)?;
                (self.rotation[2], self.scale[0]) = (v[0], v[1]);
            }
            "radius" => self.scale[1] = nums()?[0],
            "lightsource" => take(&mut self.light)?,
            "smoothing" => self.smoothing = int()?.max(0) as u32,
            "ambient" => {
                let a = int()?;
                if !(0..=100).contains(&a) {
                    return Err("ambient must be 0 to 100".into());
                }
                self.ambient = a as u32;
            }
            "stereo" => {
                let st = int()?;
                if !(0..=2).contains(&st) {
                    return Err("stereo must be 0, 1 or 2".into());
                }
                self.stereo = st as u8;
            }
            "interocular" => self.interocular = nums()?[0],
            "converge" => self.converge = nums()?[0],
            "crop" => {
                let v = nums()?;
                if v.len() != 4 {
                    return Err("crop takes 4 values".into());
                }
                self.crop.copy_from_slice(&v);
            }
            _ => return Err(format!("unknown 3D setting {key:?}")),
        }
        Ok(())
    }
}

/// The source image: palette indices and the number of colours in its
/// colour table.
pub struct Source<'a> {
    pub pixels: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub colours: u32,
}

/// The result: `None` is background (Fractint stores it as the first black
/// palette entry, T§7).
pub struct Output {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Option<u8>>,
}

/// The output size: the source's, or enlarged to 4:3 unless within 4 % of
/// it (T§1.3; some sizes far from 4:3 are kept by Fractint by a rule not
/// known).
pub fn canvas_size(w: u32, h: u32) -> (u32, u32) {
    let r = 4.0 * h as f64 / (3.0 * w as f64);
    if (r - 1.0).abs() <= 0.04 + 1e-12 {
        (w, h)
    } else if w * 3 > h * 4 {
        (w, (w as f64 * 3.0 / 4.0 + 0.5) as u32)
    } else {
        ((h as f64 * 4.0 / 3.0 + 0.5) as u32, h)
    }
}

/// Rotates about x, then y, then z in (u right, v down, w toward the
/// viewer) coordinates, in the order of operations of T§3.1.
fn rotate(a: [f64; 3], p: [f64; 3]) -> [f64; 3] {
    let (sx, cx) = a[0].to_radians().sin_cos();
    let (sy, cy) = a[1].to_radians().sin_cos();
    let (sz, cz) = a[2].to_radians().sin_cos();
    let [mut u, mut v, mut w] = p;
    (v, w) = (v * cx - w * sx, v * sx + w * cx);
    (u, w) = (u * cy + w * sy, -u * sy + w * cy);
    (u, v) = (u * cz + v * sz, -u * sz + v * cz);
    [u, v, w]
}

/// A projected source point.
#[derive(Clone, Copy)]
struct Point {
    x: i64,
    y: i64,
    colour: u8,
    /// Not drawn itself, but still a vertex (T§2.3).
    transparent: bool,
    /// Type 4: the base point (colour 0), clamped to the image.
    base: (i64, i64),
}

/// A rounded DDA line, both ends included (T§6.1).
fn dda(a: (i64, i64), b: (i64, i64)) -> Vec<(i64, i64)> {
    let n = (b.0 - a.0).abs().max((b.1 - a.1).abs());
    if n == 0 {
        return vec![a];
    }
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            (
                (a.0 as f64 + (b.0 - a.0) as f64 * t + 0.5).floor() as i64,
                (a.1 as f64 + (b.1 - a.1) as f64 * t + 0.5).floor() as i64,
            )
        })
        .collect()
}

/// The pixels of a triangle: its DDA edges and, per row, everything from
/// the leftmost to the rightmost edge pixel (T§6.2).
fn triangle(a: (i64, i64), b: (i64, i64), c: (i64, i64)) -> Vec<(i64, i64)> {
    let mut rows: std::collections::BTreeMap<i64, (i64, i64)> = Default::default();
    for (p, q) in [(a, b), (b, c), (c, a)] {
        for (x, y) in dda(p, q) {
            let e = rows.entry(y).or_insert((x, x));
            *e = (e.0.min(x), e.1.max(x));
        }
    }
    rows.into_iter()
        .flat_map(|(y, (lo, hi))| (lo..=hi).map(move |x| (x, y)))
        .collect()
}

struct Canvas {
    w: i64,
    h: i64,
    px: Vec<Option<u8>>,
}

impl Canvas {
    fn put(&mut self, pts: impl IntoIterator<Item = (i64, i64)>, c: u8) {
        for (x, y) in pts {
            if (0..self.w).contains(&x) && (0..self.h).contains(&y) {
                self.px[(y * self.w + x) as usize] = Some(c);
            }
        }
    }
}

/// Points of the planar projection (T§3), keyed by source position.
fn project_planar(src: &Source, w: u32, h: u32, s: &Settings) -> Vec<Option<Point>> {
    let (sw, sh) = (src.width as usize, src.height as usize);
    let (scx, scy, r) = (s.scale[0] / 100.0, s.scale[1] / 100.0, s.roughness / 100.0);
    let z0 = (src.colours as f64 - 1.0) / 2.0;
    let (hx, hy) = ((w as f64 - 1.0) / 2.0, (h as f64 - 1.0) / 2.0);
    let (wf, hf) = (w as f64, h as f64);
    let (mut d, mut zmax) = (0.0, 0.0);
    if s.perspective != 0.0 {
        let mut zs = Vec::new();
        for a in [-hx, hx] {
            for b in [-hy, hy] {
                for z in [-z0, z0] {
                    zs.push(rotate(s.rotation, [a * scx, b * scy, z * r])[2]);
                }
            }
        }
        zmax = zs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let zmin = zs.iter().cloned().fold(f64::INFINITY, f64::min);
        d = (s.perspective / 100.0 * (zmax - zmin) + 1e-9).trunc();
    }
    let pcx = (w / 2) as f64 - 0.5 - hx;
    let pcy = (h / 2) as f64 - 0.5 - hy;
    let shx = (s.xyshift[0] * wf / 100.0).trunc();
    let shy = (s.xyshift[1] * hf / 100.0).trunc();
    let place = |x: usize, y: usize, c: f64| -> Option<(i64, i64)> {
        let [mut u, mut v, wz] = rotate(
            s.rotation,
            [(x as f64 - hx) * scx, (y as f64 - hy) * scy, (c - z0) * r],
        );
        u += shx;
        v -= shy;
        if s.perspective != 0.0 {
            let den = d - zmax - wz;
            if den <= 0.0 {
                return None;
            }
            let f = d / den;
            u = (u - pcx) * f + pcx;
            v = (v - pcy) * f + pcy;
        }
        Some((
            (wf / 2.0 + 0.5 + u).trunc() as i64,
            (hf / 2.0 + 0.5 + v).trunc() as i64,
        ))
    };
    let mut out = Vec::with_capacity(sw * sh);
    for y in 0..sh {
        for x in 0..sw {
            let c = src.pixels[y * sw + x].max(s.waterline);
            let transparent =
                s.transparent.1 != 0 && (s.transparent.0..=s.transparent.1).contains(&c);
            out.push(place(x, y, c as f64).map(|(px, py)| {
                let base = if s.fill == 4 {
                    place(x, y, 0.0).map_or((0, 0), |(bx, by)| {
                        (bx.clamp(0, w as i64 - 1), by.clamp(0, h as i64 - 1))
                    })
                } else {
                    (0, 0)
                };
                Point {
                    x: px,
                    y: py,
                    colour: c,
                    transparent,
                    base,
                }
            }));
        }
    }
    out
}

/// Points of the spherical projection (T§5).
fn project_sphere(src: &Source, w: u32, h: u32, s: &Settings) -> Vec<Option<Point>> {
    let (sw, sh) = (src.width as usize, src.height as usize);
    let (wf, hf) = (w as f64, h as f64);
    let (lon_r, lat_r) = (s.longitude(), s.latitude());
    let r0 = s.radius() / 100.0 * hf / 2.0;
    let rough = s.roughness / 100.0;
    let n = src.colours as f64;
    let mut out = Vec::with_capacity(sw * sh);
    for y in 0..sh {
        let lon = (lon_r[0] + (lon_r[1] - lon_r[0]) * y as f64 / hf).to_radians();
        for x in 0..sw {
            let lat = (lat_r[0] + (lat_r[1] - lat_r[0]) * x as f64 / wf).to_radians();
            let c = src.pixels[y * sw + x].max(s.waterline);
            let transparent =
                s.transparent.1 != 0 && (s.transparent.0..=s.transparent.1).contains(&c);
            // The radius law with roughness is a hypothesis (T§5.3).
            let r = r0 * (1.0 - rough * (n - 1.0 - c as f64) / 800.0);
            out.push((lat.cos() * lon.sin() > 0.0).then(|| Point {
                x: (wf / 2.0 + r * lat.sin()).trunc() as i64,
                y: (hf / 2.0 + r * lat.cos() * lon.cos()).trunc() as i64,
                colour: c,
                transparent,
                base: (0, 0),
            }));
        }
    }
    out
}

/// Light-source colours of every source point (L§3–§5, §7.1); `None` in
/// row 0 and column 0.
fn light_colours(src: &Source, s: &Settings) -> Vec<Option<u8>> {
    let (sw, sh) = (src.width as usize, src.height as usize);
    let (sx, sy, r) = (s.scale[0] / 100.0, s.scale[1] / 100.0, s.roughness / 100.0);
    let z = |x: usize, y: usize| src.pixels[y * sw + x].max(s.waterline) as f64 * r;
    // The light in (u right, v down, w toward the viewer) coordinates.
    let mut l = [s.light[0], -s.light[1], s.light[2]];
    // Fill 6: the light is scaled like the surface and turns with it, so
    // neither needs rotating (L§4.2). Fill 5: the normals are rotated.
    if s.fill == 6 {
        l = [l[0] * sx, l[1] * sy, l[2] * r];
    }
    let norm = |v: [f64; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let ln = norm(l);
    let l = if ln > 0.0 {
        l.map(|v| v / ln)
    } else {
        [0.0; 3]
    };
    let ambient = (255.0 * s.ambient as f64 / 100.0).ceil();
    let mut out = vec![None; sw * sh];
    for y in 1..sh {
        let mut acc: Option<[f64; 3]> = None;
        for x in 1..sw {
            let gx = (z(x, y) - z(x - 1, y)) / sx;
            let gy = (z(x, y) - z(x, y - 1)) / sy;
            // (−gx, +gy, 1) with y up is (−gx, −gy, 1) with v down.
            let mut n = [-gx, -gy, 1.0];
            if s.smoothing > 0 {
                let m = norm(n);
                n = n.map(|v| v / m);
                let k = s.smoothing as f64;
                n = match acc {
                    Some(a) => std::array::from_fn(|i| (k * a[i] + n[i]) / (k + 1.0)),
                    None => n,
                };
                acc = Some(n);
            }
            if s.fill == 5 {
                n = rotate(s.rotation, n);
            }
            let m = norm(n);
            let c = ((n[0] * l[0] + n[1] * l[1] + n[2] * l[2]) / m).max(0.0);
            let v = (254.0 * (1.0 - c) * (255.0 - ambient) / 255.0)
                .floor()
                .max(1.0);
            out[y * sw + x] = Some(v as u8);
        }
    }
    out
}

/// Draws the surface (T§4, §6) onto a canvas.
fn draw(src: &Source, w: u32, h: u32, s: &Settings) -> Vec<Option<u8>> {
    let pts = if s.sphere {
        project_sphere(src, w, h, s)
    } else {
        project_planar(src, w, h, s)
    };
    let lit = matches!(s.fill, 5 | 6).then(|| light_colours(src, s));
    let (sw, sh) = (src.width as usize, src.height as usize);
    let mut cv = Canvas {
        w: w as i64,
        h: h as i64,
        px: vec![None; (w * h) as usize],
    };
    let at = |x: usize, y: usize| pts[y * sw + x];
    let colour = |x: usize, y: usize| -> Option<u8> {
        match &lit {
            Some(v) => v[y * sw + x],
            None => Some(at(x, y)?.colour),
        }
    };
    let fill = if s.sphere { 0 } else { s.fill };
    if fill == -1 {
        // A coarse grid (T§6.5).
        let ys = (h / 20).max(1) as usize;
        let xs = ((ys as u64 * w as u64 * 3) / (4 * h as u64)).max(1) as usize;
        let mut rows: Vec<usize> = (0..sh).step_by(ys).collect();
        let mut cols: Vec<usize> = (0..sw).step_by(xs).collect();
        if rows.last() != Some(&(sh - 1)) {
            rows.push(sh - 1);
        }
        if cols.last() != Some(&(sw - 1)) {
            cols.push(sw - 1);
        }
        for (ri, &y) in rows.iter().enumerate() {
            for (ci, &x) in cols.iter().enumerate() {
                let Some(q) = at(x, y).filter(|q| !q.transparent) else {
                    continue;
                };
                if ri == 0 && ci == 0 {
                    cv.put([(q.x, q.y)], q.colour);
                }
                if ci > 0
                    && let Some(o) = at(cols[ci - 1], y)
                {
                    cv.put(dda((o.x, o.y), (q.x, q.y)), q.colour);
                }
                if ri > 0
                    && let Some(o) = at(x, rows[ri - 1])
                {
                    cv.put(dda((o.x, o.y), (q.x, q.y)), q.colour);
                }
            }
        }
        return cv.px;
    }
    for y in 0..sh {
        for x in 0..sw {
            let Some(q) = at(x, y) else { continue };
            if q.transparent {
                continue;
            }
            let p = (q.x, q.y);
            match fill {
                1 => match x.checked_sub(1).and_then(|l| at(l, y)) {
                    Some(o) => cv.put(dda((o.x, o.y), p), q.colour),
                    None => cv.put([p], q.colour),
                },
                2 | 3 | 5 | 6 => {
                    let Some(cp) = colour(x, y) else { continue };
                    let left = x.checked_sub(1).and_then(|l| at(l, y));
                    if y == 0 {
                        match left {
                            Some(o) => cv.put(dda((o.x, o.y), p), cp),
                            None => cv.put([p], cp),
                        }
                        continue;
                    }
                    let up = at(x, y - 1);
                    let ur = (x + 1 < sw).then(|| at(x + 1, y - 1)).flatten();
                    // Type 3: flat triangles in P's colour. Types 2, 5, 6:
                    // flat triangles with a weighted mix of the corner
                    // colours, which reproduces Fractint at 1:1 (L§6);
                    // its rule for larger triangles is unknown (T§6.7).
                    let mix = |weights: &[(u32, Option<u8>)]| -> Option<u8> {
                        if fill == 3 {
                            return Some(q.colour);
                        }
                        let (mut sum, mut total) = (0u32, 0u32);
                        for &(wgt, c) in weights {
                            sum += wgt * c? as u32;
                            total += wgt;
                        }
                        Some((sum / total) as u8)
                    };
                    if let (Some(u), Some(r)) = (up, ur) {
                        let c = mix(&[
                            (1, Some(cp)),
                            (3, colour(x, y - 1)),
                            (2, colour(x + 1, y - 1)),
                        ]);
                        if let Some(c) = c {
                            cv.put(triangle(p, (u.x, u.y), (r.x, r.y)), c);
                        }
                    }
                    if let (Some(u), Some(l)) = (up, left) {
                        let cu = colour(x, y - 1);
                        let c = match colour(x - 1, y) {
                            Some(cl) => mix(&[(2, Some(cp)), (1, Some(cl)), (3, cu)]),
                            // No colour on the left (column 0 of a light
                            // fill): Fractint mixes 3:1 (L§6).
                            None => mix(&[(1, Some(cp)), (3, cu)]),
                        };
                        if let Some(c) = c {
                            cv.put(triangle(p, (l.x, l.y), (u.x, u.y)), c);
                        }
                    }
                }
                4 => cv.put(dda(p, q.base), q.colour),
                _ => cv.put([p], q.colour),
            }
        }
    }
    cv.px
}

/// The 3D transform of `src`.
pub fn render(src: &Source, s: &Settings) -> Output {
    let (w, h) = canvas_size(src.width, src.height);
    let mono = draw(src, w, h, s);
    let pixels = match s.stereo {
        1 | 2 => stereo(&mono, w, h, s),
        _ => mono,
    };
    Output {
        width: w,
        height: h,
        pixels,
    }
}

/// The two eye images, each the mono image moved sideways, combined as in
/// L§8 (without perspective; with it the parallax is unknown).
fn stereo(mono: &[Option<u8>], w: u32, h: u32, s: &Settings) -> Vec<Option<u8>> {
    let wf = w as f64;
    let d = (wf * (s.interocular / 2.0 + s.converge) / 100.0).round() as i64;
    let eye = |x: i64, y: i64, shift: i64, lo: f64, hi: f64| -> Option<u8> {
        let fx = x as f64;
        if !(fx > wf * lo / 100.0 && fx < wf - wf * hi / 100.0) {
            return None;
        }
        let sx = x - shift;
        if sx < 0 || sx >= w as i64 {
            return None;
        }
        let c = mono[(y * w as i64 + sx) as usize]?;
        Some(128 - c.div_ceil(2))
    };
    let mut out = vec![None; (w * h) as usize];
    for y in 0..h as i64 {
        for x in 0..w as i64 {
            let a = eye(x, y, d, s.crop[0], s.crop[1]);
            let b = eye(x, y, -d, s.crop[2], s.crop[3]);
            out[(y * w as i64 + x) as usize] = match s.stereo {
                1 => {
                    if (x + y) % 2 == 0 {
                        a
                    } else {
                        b.map(|e| e + 128)
                    }
                }
                _ => {
                    let level = |e: Option<u8>| e.map_or(0, |e| e / 9 + 1);
                    let v = 16 * level(b) + level(a);
                    (v != 0).then_some(v)
                }
            };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_rule() {
        assert_eq!(canvas_size(400, 300), (400, 300));
        assert_eq!(canvas_size(98, 75), (98, 75));
        assert_eq!(canvas_size(105, 75), (105, 79));
        assert_eq!(canvas_size(100, 100), (133, 100));
    }

    /// Identity: each pixel moves one right and one down (T§3.2).
    #[test]
    fn identity_shifts_by_one() {
        let px: Vec<u8> = (0..12).map(|i| i as u8 + 1).collect();
        let src = Source {
            pixels: &px,
            width: 4,
            height: 3,
            colours: 256,
        };
        let s = Settings {
            rotation: [0.0; 3],
            scale: [100.0, 100.0],
            roughness: 0.0,
            ..Settings::default()
        };
        let out = render(&src, &s);
        assert_eq!(out.pixels[5], Some(1));
        assert_eq!(out.pixels[0], None);
    }
}
