// SPDX-License-Identifier: GPL-3.0-or-later

//! Fractint-format `.map` palette files, following docs/specs/map.md
//! (cited as §n).

use fractalforge_core::Palette;

/// Grey used for entries a file doesn't give, in 6-bit units (§2.4).
pub const FILLER_6BIT: u8 = 40;

/// Expands a 6-bit channel to 8 bits as Fractint-format GIFs store it.
pub fn expand6(v: u8) -> u8 {
    (v << 2) | (v >> 4)
}

/// The grey filler colour in 8 bits.
pub fn filler() -> [u8; 3] {
    [expand6(FILLER_6BIT); 3]
}

/// Reads a map file the way XFractint does (§5): every line is one entry,
/// unparsable channels repeat the previous entry's values, values keep their
/// low 8 bits and then their top 6, and missing entries are grey. Returns
/// the palette and warnings about lines XFractint would likely misread.
pub fn read(text: &str) -> (Palette, Vec<String>) {
    let mut colors = [filler(); 256];
    let mut warnings = Vec::new();
    let mut prev = [0i64; 3];
    let mut count = 0;
    // A final newline ends the last line rather than starting another.
    let body = text.strip_suffix('\n').unwrap_or(text);
    for (i, line) in body.split('\n').enumerate() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if count == 256 {
            if !line.trim().is_empty() {
                warnings.push("lines after the 256th were ignored".into());
            }
            break;
        }
        let (values, parsed) = leading_ints(line);
        if parsed < 3 {
            warnings.push(format!(
                "line {}: expected three numbers; XFractint repeats the previous colour",
                i + 1
            ));
        }
        for (ch, v) in values.iter().take(parsed).enumerate() {
            if !(0..=255).contains(v) {
                warnings.push(format!("line {}: value {v} is outside 0-255", i + 1));
            }
            prev[ch] = *v;
        }
        colors[count] = prev.map(|v| expand6((v.rem_euclid(256) as u8) >> 2));
        count += 1;
    }
    if count < 256 {
        warnings.push(format!("only {count} entries; the rest are grey"));
    }
    (Palette(colors), warnings)
}

/// Up to three leading C-style decimal integers (§2.3), and how many parsed.
fn leading_ints(line: &str) -> ([i64; 3], usize) {
    let mut out = [0; 3];
    let mut rest = line;
    for (n, slot) in out.iter_mut().enumerate() {
        rest = rest.trim_start_matches([' ', '\t']);
        let sign_len = usize::from(rest.starts_with(['+', '-']));
        let digits = rest[sign_len..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        if digits == 0 {
            return (out, n);
        }
        let end = sign_len + digits;
        *slot = rest[..end].parse::<i64>().unwrap_or(i64::MAX);
        rest = &rest[end..];
        if n < 2 && !rest.starts_with([' ', '\t']) {
            return (out, n + 1);
        }
    }
    (out, 3)
}

/// Writes a map in XFractint's own layout (§3–§4): 256 lines of the 6-bit
/// values times 4.
pub fn write(palette: &Palette) -> String {
    let mut s = String::with_capacity(256 * 12);
    for [r, g, b] in palette.0 {
        s.push_str(&format!(
            "{:3} {:3} {:3}\n",
            (r >> 2) * 4,
            (g >> 2) * 4,
            (b >> 2) * 4
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_lines_like_xfractint() {
        let (p, _) = read("12 243 36\n  1   2   3  trailing text\n\n77 88\n300 -5 256\n");
        assert_eq!(p.0[0], [expand6(3), expand6(60), expand6(9)]);
        assert_eq!(p.0[1], [0, 0, 0]);
        // A blank line repeats the previous entry.
        assert_eq!(p.0[2], p.0[1]);
        // Two numbers: blue from the previous line.
        assert_eq!(p.0[3], [expand6(19), expand6(22), 0]);
        // 300 → 44 → 11; −5 → 251 → 62; 256 → 0.
        assert_eq!(p.0[4], [expand6(11), expand6(62), 0]);
        assert_eq!(p.0[5], filler());
        assert_eq!(filler(), [0xa2; 3]);
    }

    #[test]
    fn floats_and_commas_stop_parsing() {
        let (p, w) = read("40 40 40\n12.7 33.2 99.9\n0,255,0\n");
        assert_eq!(p.0[1], [expand6(3), expand6(10), expand6(10)]);
        assert_eq!(p.0[2][0], 0);
        assert!(!w.is_empty());
    }

    #[test]
    fn write_then_read_keeps_six_bits() {
        let pal = Palette::forge();
        let text = write(&pal);
        assert_eq!(text.lines().count(), 256);
        assert!(text.lines().all(|l| l.len() == 11));
        let (back, warnings) = read(&text);
        assert!(warnings.is_empty(), "{warnings:?}");
        for (a, b) in pal.0.iter().zip(back.0.iter()) {
            assert_eq!(a.map(|v| v >> 2), b.map(|v| v >> 2));
        }
    }
}
