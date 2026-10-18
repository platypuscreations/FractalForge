// SPDX-License-Identifier: GPL-3.0-or-later

//! Signed fixed-point numbers with a chosen number of 64-bit fraction
//! limbs, for deep-zoom coordinates and reference orbits.
//!
//! A value is ±(integer limb + fraction limbs)/2^(64·frac). Operations
//! truncate to the precision of their left operand.

use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigFixed {
    neg: bool,
    /// Little-endian: `limbs[..frac]` is the fraction, `limbs[frac]` the
    /// integer part.
    limbs: Vec<u64>,
    frac: usize,
}

impl BigFixed {
    pub fn zero(frac: usize) -> Self {
        Self {
            neg: false,
            limbs: vec![0; frac + 1],
            frac,
        }
    }

    /// The exact value as (negative, little-endian limbs with the integer
    /// limb last), for storage.
    pub fn to_parts(&self) -> (bool, &[u64]) {
        (self.neg, &self.limbs)
    }

    /// The inverse of [`BigFixed::to_parts`]; `None` without limbs.
    pub fn from_parts(neg: bool, limbs: Vec<u64>) -> Option<Self> {
        let frac = limbs.len().checked_sub(1)?;
        Some(Self { neg, limbs, frac }.normalise())
    }

    /// Number of 64-bit fraction limbs.
    pub fn frac_limbs(&self) -> usize {
        self.frac
    }

    pub fn is_zero(&self) -> bool {
        self.limbs.iter().all(|&l| l == 0)
    }

    fn normalise(mut self) -> Self {
        if self.is_zero() {
            self.neg = false;
        }
        self
    }

    /// The same value with `frac` fraction limbs (truncating or extending).
    pub fn with_frac(&self, frac: usize) -> Self {
        let mut limbs = vec![0; frac + 1];
        for (i, &l) in self.limbs.iter().enumerate() {
            // Limb i has weight 2^(64·(i − self.frac)).
            let j = i as isize - self.frac as isize + frac as isize;
            if (0..=frac as isize).contains(&j) {
                limbs[j as usize] = l;
            }
        }
        Self {
            neg: self.neg,
            limbs,
            frac,
        }
        .normalise()
    }

    /// The exact value of a double (truncated below the precision);
    /// |v| < 2⁶⁴.
    pub fn from_f64(v: f64, frac: usize) -> Self {
        let mut out = Self::zero(frac);
        if !v.is_finite() || v == 0.0 {
            return out;
        }
        out.neg = v < 0.0;
        let mut a = v.abs();
        // Integer limb, then each fraction limb.
        let int = a.trunc();
        out.limbs[frac] = int as u64;
        a -= int;
        for i in (0..frac).rev() {
            if a == 0.0 {
                break;
            }
            a *= 18446744073709551616.0; // 2^64
            let d = a.trunc();
            out.limbs[i] = d as u64;
            a -= d;
        }
        out.normalise()
    }

    pub fn to_f64(&self) -> f64 {
        // The top three non-zero limbs are plenty for 53 bits.
        let mut v = 0.0;
        let mut used = 0;
        for i in (0..=self.frac).rev() {
            if used == 0 && self.limbs[i] == 0 {
                continue;
            }
            let weight = 2f64.powi(64 * (i as i32 - self.frac as i32));
            v += self.limbs[i] as f64 * weight;
            used += 1;
            if used == 3 {
                break;
            }
        }
        if self.neg { -v } else { v }
    }

    fn cmp_mag(a: &[u64], b: &[u64]) -> Ordering {
        for (x, y) in a.iter().rev().zip(b.iter().rev()) {
            match x.cmp(y) {
                Ordering::Equal => {}
                o => return o,
            }
        }
        Ordering::Equal
    }

    fn add_mag(a: &[u64], b: &[u64]) -> Vec<u64> {
        let mut carry = 0u128;
        a.iter()
            .zip(b)
            .map(|(&x, &y)| {
                let s = x as u128 + y as u128 + carry;
                carry = s >> 64;
                s as u64
            })
            .collect()
    }

    /// a − b for |a| ≥ |b|.
    fn sub_mag(a: &[u64], b: &[u64]) -> Vec<u64> {
        let mut borrow = 0i128;
        a.iter()
            .zip(b)
            .map(|(&x, &y)| {
                let d = x as i128 - y as i128 - borrow;
                borrow = (d < 0) as i128;
                d.rem_euclid(1 << 64) as u64
            })
            .collect()
    }

    pub fn add(&self, b: &BigFixed) -> BigFixed {
        let b = b.with_frac(self.frac);
        let (neg, limbs) = if self.neg == b.neg {
            (self.neg, Self::add_mag(&self.limbs, &b.limbs))
        } else if Self::cmp_mag(&self.limbs, &b.limbs) != Ordering::Less {
            (self.neg, Self::sub_mag(&self.limbs, &b.limbs))
        } else {
            (b.neg, Self::sub_mag(&b.limbs, &self.limbs))
        };
        Self {
            neg,
            limbs,
            frac: self.frac,
        }
        .normalise()
    }

    pub fn neg(&self) -> BigFixed {
        Self {
            neg: !self.neg,
            ..self.clone()
        }
        .normalise()
    }

    pub fn sub(&self, b: &BigFixed) -> BigFixed {
        self.add(&b.neg())
    }

    pub fn mul(&self, b: &BigFixed) -> BigFixed {
        let b = b.with_frac(self.frac);
        let n = self.limbs.len();
        let mut prod = vec![0u64; 2 * n];
        for (i, &x) in self.limbs.iter().enumerate() {
            if x == 0 {
                continue;
            }
            let mut carry = 0u128;
            for (j, &y) in b.limbs.iter().enumerate() {
                let t = prod[i + j] as u128 + x as u128 * y as u128 + carry;
                prod[i + j] = t as u64;
                carry = t >> 64;
            }
            let mut k = i + n;
            while carry != 0 && k < prod.len() {
                let t = prod[k] as u128 + carry;
                prod[k] = t as u64;
                carry = t >> 64;
                k += 1;
            }
        }
        Self {
            neg: self.neg != b.neg,
            limbs: prod[self.frac..self.frac + n].to_vec(),
            frac: self.frac,
        }
        .normalise()
    }

    pub fn square(&self) -> BigFixed {
        self.mul(self)
    }

    /// Division by a small positive integer (truncating).
    fn div_small(&self, d: u64) -> BigFixed {
        let mut rem = 0u128;
        let mut limbs = self.limbs.clone();
        for l in limbs.iter_mut().rev() {
            let cur = (rem << 64) | *l as u128;
            *l = (cur / d as u128) as u64;
            rem = cur % d as u128;
        }
        Self {
            neg: self.neg,
            limbs,
            frac: self.frac,
        }
        .normalise()
    }

    /// Multiplication by a small integer.
    fn mul_small(&self, m: u64) -> BigFixed {
        let mut carry = 0u128;
        let limbs = self
            .limbs
            .iter()
            .map(|&l| {
                let t = l as u128 * m as u128 + carry;
                carry = t >> 64;
                t as u64
            })
            .collect();
        Self {
            neg: self.neg,
            limbs,
            frac: self.frac,
        }
        .normalise()
    }

    /// Parses a decimal number such as `-0.743643887037158704752191506114774`
    /// or `1.5e-3`. The integer part must be below 2⁶⁴.
    pub fn parse(s: &str, frac: usize) -> Option<BigFixed> {
        let s = s.trim();
        let (mant, exp) = match s.find(['e', 'E']) {
            Some(i) => (&s[..i], s[i + 1..].parse::<i64>().ok()?),
            None => (s, 0),
        };
        let (neg, mant) = match mant.strip_prefix('-') {
            Some(m) => (true, m),
            None => (false, mant.strip_prefix('+').unwrap_or(mant)),
        };
        let (int, fr) = mant.split_once('.').unwrap_or((mant, ""));
        if int.is_empty() && fr.is_empty()
            || !int.bytes().chain(fr.bytes()).all(|c| c.is_ascii_digit())
        {
            return None;
        }
        // Move the point by the exponent: digits before and after it.
        let digits: Vec<u8> = int.bytes().chain(fr.bytes()).map(|c| c - b'0').collect();
        let point = int.len() as i64 + exp;
        let digit = |i: i64| -> u8 {
            if i < 0 || i >= digits.len() as i64 {
                0
            } else {
                digits[i as usize]
            }
        };
        let mut whole: u64 = 0;
        for i in 0..point.max(0) {
            whole = whole.checked_mul(10)?.checked_add(digit(i) as u64)?;
        }
        // The fraction, from its last digit back: f = (f + d) / 10.
        let work = frac + 1;
        let mut f = Self::zero(work);
        let first = point.max(0);
        let last = (digits.len() as i64).max(first);
        for i in (first..last).rev() {
            f.limbs[work] = digit(i) as u64;
            f = f.div_small(10);
        }
        // Leading zeros between the point and the first digit.
        for _ in point..0 {
            f = f.div_small(10);
        }
        f.limbs[work] = whole;
        let v = f.with_frac(frac);
        Some(if neg { v.neg() } else { v })
    }

    /// `digits` decimal digits after the point, rounded to nearest.
    pub fn to_decimal(&self, digits: usize) -> String {
        let mut s = String::new();
        if self.neg {
            s.push('-');
        }
        let half = Self::parse(&format!("5e-{}", digits + 1), self.frac)
            .unwrap_or_else(|| Self::zero(self.frac));
        let mut rounded = self.clone();
        rounded.neg = false;
        let rounded = rounded.add(&half);
        s.push_str(&rounded.limbs[self.frac].to_string());
        if digits > 0 {
            s.push('.');
            let mut f = rounded;
            f.limbs[self.frac] = 0;
            for _ in 0..digits {
                f = f.mul_small(10);
                s.push((b'0' + f.limbs[self.frac] as u8) as char);
                f.limbs[self.frac] = 0;
            }
        }
        s
    }
}

impl PartialOrd for BigFixed {
    fn partial_cmp(&self, b: &BigFixed) -> Option<Ordering> {
        let b = b.with_frac(self.frac);
        Some(match (self.neg, b.neg) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => Self::cmp_mag(&self.limbs, &b.limbs),
            (true, true) => Self::cmp_mag(&b.limbs, &self.limbs),
        })
    }
}

/// Fraction limbs needed to place pixels `pixel` apart around a centre of
/// magnitude up to a few units, with 64 bits to spare.
pub fn frac_limbs_for(pixel: f64) -> usize {
    let bits = (-pixel.abs().max(1e-300).log2()).max(0.0) + 64.0;
    (bits / 64.0).ceil() as usize + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic() {
        let a = BigFixed::from_f64(1.25, 2);
        let b = BigFixed::from_f64(-0.5, 2);
        assert_eq!(a.add(&b).to_f64(), 0.75);
        assert_eq!(b.sub(&a).to_f64(), -1.75);
        assert_eq!(a.mul(&b).to_f64(), -0.625);
        assert_eq!(b.square().to_f64(), 0.25);
        assert!(b < a);
    }

    #[test]
    fn decimal_round_trip() {
        let s = "-0.7436438870371587047521915061147740001";
        let v = BigFixed::parse(s, 3).unwrap();
        assert_eq!(v.to_decimal(37), s);
        assert_eq!(BigFixed::parse("2.5e-3", 2).unwrap().to_f64(), 0.0025);
        assert_eq!(BigFixed::parse("12", 1).unwrap().to_decimal(0), "12");
        assert!(BigFixed::parse("1.2.3", 1).is_none());
    }

    #[test]
    fn precision_beyond_f64() {
        let tiny = BigFixed::parse("1e-40", 3).unwrap();
        let one = BigFixed::from_f64(1.0, 3);
        let sum = one.add(&tiny);
        assert_eq!(sum.sub(&one), tiny);
        assert!(sum > one);
    }
}
