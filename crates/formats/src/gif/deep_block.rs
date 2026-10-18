// SPDX-License-Identifier: GPL-3.0-or-later

//! Fractint's `fractint005` block: arbitrary-precision coordinates of deep
//! views, following docs/specs/deep-zoom.md (cited as §n).
//!
//! The block holds 22 slots of N + 6 bytes: a little-endian two's-complement
//! mantissa m of M = N + 4 bytes and an i16 exponent e, with
//! value = m / 2^(8(M−1)) · 256^(e+1) (§3.2).

use fractalforge_core::bignum::BigFixed;

/// Number of slots: corners (6), params (10), copy of the corners (6).
pub const SLOTS: usize = 22;

/// Bytes of the block for size N.
pub fn block_len(n: usize) -> usize {
    SLOTS * (n + 6)
}

/// The size N for a view whose closest pixels are `spacing` apart (§2):
/// the smallest multiple of 4 holding the decimal digits needed, at least 8.
pub fn size_for(spacing: f64) -> usize {
    let digits = (-spacing.log10()).ceil();
    let mut n = 8;
    while 8.0 * n as f64 * std::f64::consts::LOG10_2 - 3.38 < digits && n < 4096 {
        n += 4;
    }
    n
}

// ---- unsigned big integers as little-endian u64 words --------------------

fn bit_len(v: &[u64]) -> i64 {
    match v.iter().rposition(|&w| w != 0) {
        Some(i) => 64 * i as i64 + (64 - v[i].leading_zeros()) as i64,
        None => 0,
    }
}

fn shl(v: &[u64], s: usize) -> Vec<u64> {
    let (words, bits) = (s / 64, s % 64);
    let mut out = vec![0u64; v.len() + words + 1];
    for (i, &w) in v.iter().enumerate() {
        out[i + words] |= w << bits;
        if bits > 0 {
            out[i + words + 1] |= w >> (64 - bits);
        }
    }
    out
}

/// v / 2^s, rounded to nearest (ties up).
fn shr_round(v: &[u64], s: usize) -> Vec<u64> {
    if s == 0 {
        return v.to_vec();
    }
    let bit = |i: usize| -> bool {
        let (w, b) = (i / 64, i % 64);
        w < v.len() && (v[w] >> b) & 1 == 1
    };
    let (words, bits) = (s / 64, s % 64);
    let mut out: Vec<u64> = (words..v.len())
        .map(|i| {
            let lo = v[i] >> bits;
            let hi = if bits > 0 && i + 1 < v.len() {
                v[i + 1] << (64 - bits)
            } else {
                0
            };
            lo | hi
        })
        .collect();
    if bit(s - 1) {
        // Add one.
        for w in out.iter_mut() {
            let (r, carry) = w.overflowing_add(1);
            *w = r;
            if !carry {
                return out;
            }
        }
        out.push(1);
    }
    out
}

fn is_power_of_two(v: &[u64]) -> bool {
    v.iter().map(|w| w.count_ones()).sum::<u32>() == 1
}

/// One slot for `v` in normal form (§3.2), rounded to nearest.
pub fn encode_slot(v: &BigFixed, n: usize) -> Vec<u8> {
    let m_bytes = n + 4;
    let (neg, limbs) = v.to_parts();
    let frac_bits = 64 * (limbs.len() as i64 - 1);
    let b = bit_len(limbs);
    let mut out = vec![0u8; m_bytes + 2];
    if b == 0 {
        return out;
    }
    // |v| ∈ [2^(b−1−F), 2^(b−F)): the smallest e with |v| < 256^(e+1), or
    // |v| ≤ 256^(e+1) for negative values.
    let mut e = (b - 1 - frac_bits).div_euclid(8);
    if neg && is_power_of_two(limbs) && (b - 1 - frac_bits).rem_euclid(8) == 0 {
        e -= 1;
    }
    loop {
        // |m| = |v| · 256^(M−2−e), i.e. A shifted by 8(M−2−e) − F bits.
        let shift = 8 * (m_bytes as i64 - 2 - e) - frac_bits;
        let mag = if shift >= 0 {
            shl(limbs, shift as usize)
        } else {
            shr_round(limbs, (-shift) as usize)
        };
        // Normal range: |m| < 2^(8(M−1)), or = for negatives.
        let limit = 8 * (m_bytes as i64 - 1);
        let mb = bit_len(&mag);
        let fits = mb <= limit || (neg && mb == limit + 1 && is_power_of_two(&mag));
        if !fits {
            e += 1;
            continue;
        }
        let mut bytes: Vec<u8> = mag
            .iter()
            .flat_map(|w| w.to_le_bytes())
            .chain(std::iter::repeat(0))
            .take(m_bytes)
            .collect();
        if neg {
            // Two's complement.
            let mut carry = true;
            for byte in bytes.iter_mut() {
                let (r, c) = (!*byte).overflowing_add(carry as u8);
                *byte = r;
                carry = c;
            }
        }
        out[..m_bytes].copy_from_slice(&bytes);
        out[m_bytes..].copy_from_slice(&(e as i16).to_le_bytes());
        return out;
    }
}

/// The value of a slot, with `frac` fraction limbs (truncated).
pub fn decode_slot(slot: &[u8], n: usize, frac: usize) -> Option<BigFixed> {
    let m_bytes = n + 4;
    if slot.len() != m_bytes + 2 {
        return None;
    }
    let e = i16::from_le_bytes([slot[m_bytes], slot[m_bytes + 1]]) as i64;
    let mut bytes = slot[..m_bytes].to_vec();
    let neg = bytes[m_bytes - 1] & 0x80 != 0;
    if neg {
        let mut carry = true;
        for byte in bytes.iter_mut() {
            let (r, c) = (!*byte).overflowing_add(carry as u8);
            *byte = r;
            carry = c;
        }
    }
    let words: Vec<u64> = bytes
        .chunks(8)
        .map(|c| {
            let mut w = [0u8; 8];
            w[..c.len()].copy_from_slice(c);
            u64::from_le_bytes(w)
        })
        .collect();
    // value = |m| · 2^(8(e+2−M)); as limbs over 2^(64·frac).
    let shift = 8 * (e + 2 - m_bytes as i64) + 64 * frac as i64;
    let mag = if shift >= 0 {
        shl(&words, shift as usize)
    } else {
        // Truncate: drop the bits below the precision.
        let s = (-shift) as usize;
        let (w, b) = (s / 64, s % 64);
        (w..words.len())
            .map(|i| {
                let lo = words[i] >> b;
                let hi = if b > 0 && i + 1 < words.len() {
                    words[i + 1] << (64 - b)
                } else {
                    0
                };
                lo | hi
            })
            .collect()
    };
    let mut limbs = vec![0u64; frac + 1];
    for (i, w) in mag.into_iter().enumerate() {
        if i < limbs.len() {
            limbs[i] = w;
        } else if w != 0 {
            return None; // beyond the integer limb
        }
    }
    BigFixed::from_parts(neg, limbs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn big(s: &str) -> BigFixed {
        BigFixed::parse(s, 4).unwrap()
    }

    /// The examples of §3.2 (most significant mantissa bytes first).
    #[test]
    fn normal_form_examples() {
        let n = 12;
        let top = |v: &str| {
            let s = encode_slot(&big(v), n);
            let e = i16::from_le_bytes([s[n + 4], s[n + 5]]);
            let bytes: Vec<u8> = s[..n + 4].iter().rev().take(3).copied().collect();
            (e, bytes)
        };
        assert_eq!(top("1"), (0, vec![0x00, 0x01, 0x00]));
        assert_eq!(top("0.00390625"), (-1, vec![0x00, 0x01, 0x00]));
        assert_eq!(top("-1"), (-1, vec![0xff, 0x00, 0x00]));
        assert_eq!(top("-0.00390625"), (-2, vec![0xff, 0x00, 0x00]));
        assert_eq!(top("0.5"), (-1, vec![0x00, 0x80, 0x00]));
        assert_eq!(top("256"), (1, vec![0x00, 0x01, 0x00]));
        assert_eq!(top("300.5"), (1, vec![0x00, 0x01, 0x2c]));
        assert_eq!(encode_slot(&big("0"), n), vec![0; n + 6]);
    }

    #[test]
    fn round_trips() {
        for v in [
            "-0.743643887037158704752191506114774",
            "0.131825904205311970493132056385139",
            "-1",
            "1e-30",
            "-300.5",
        ] {
            let x = big(v);
            let s = encode_slot(&x, 20);
            let back = decode_slot(&s, 20, 4).unwrap();
            assert!(back.sub(&x).to_f64().abs() < 1e-45, "{v}");
        }
    }

    #[test]
    fn sizes() {
        // §2 at 400×300: N = 8 up to mag 5e12, 12 at 1e15, 16 from
        // 6.75e22, 20 from 6.75e32.
        let spacing = |mag: f64| 2.0 / mag / 299.0;
        assert_eq!(size_for(spacing(5e12)), 8);
        assert_eq!(size_for(spacing(1e15)), 12);
        assert_eq!(size_for(spacing(6.6e22)), 12);
        assert_eq!(size_for(spacing(6.75e22)), 16);
        assert_eq!(size_for(spacing(6.75e32)), 20);
        assert_eq!(size_for(spacing(1e300)), 128);
        assert_eq!(block_len(12), 396);
    }
}
