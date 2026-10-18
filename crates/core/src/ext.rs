// SPDX-License-Identifier: GPL-3.0-or-later

//! Binary floating point with a 64-bit significand, rounded to nearest
//! (ties to even) after every operation: the precision of x87 extended
//! registers, which Fractint-format L-system images depend on
//! (docs/specs/lsystem.md §6.2). Also cos and sin accurate to about 100
//! bits, through double-double arithmetic.

use std::cmp::Ordering;

/// ±man·2^exp with man normalised (top bit set), or zero (man = 0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ext {
    neg: bool,
    exp: i32,
    man: u64,
}

impl Ext {
    pub const ZERO: Ext = Ext {
        neg: false,
        exp: 0,
        man: 0,
    };

    /// Rounds ±m·2^e (with `sticky` set when nonzero bits below m were
    /// dropped) to 64 significant bits.
    fn round(neg: bool, m: u128, e: i32, sticky: bool) -> Ext {
        if m == 0 {
            return Ext::ZERO;
        }
        let lz = m.leading_zeros();
        let m = m << lz;
        let e = e - lz as i32;
        let mut hi = (m >> 64) as u64;
        let rest = m as u64;
        let half = 1u64 << 63;
        let up = rest > half || (rest == half && (sticky || hi & 1 == 1));
        let mut exp = e + 64;
        if up {
            hi = hi.wrapping_add(1);
            if hi == 0 {
                hi = half;
                exp += 1;
            }
        }
        Ext { neg, exp, man: hi }
    }

    /// The exact value of a double.
    pub fn from_f64(v: f64) -> Ext {
        if v == 0.0 || !v.is_finite() {
            return Ext::ZERO;
        }
        let bits = v.to_bits();
        let neg = bits >> 63 == 1;
        let biased = ((bits >> 52) & 0x7ff) as i32;
        let frac = bits & ((1 << 52) - 1);
        let (m, e) = if biased == 0 {
            (frac, -1074)
        } else {
            (frac | 1 << 52, biased - 1075)
        };
        Ext::round(neg, m as u128, e, false)
    }

    /// Rounded to the nearest double.
    pub fn to_f64(self) -> f64 {
        if self.man == 0 {
            return 0.0;
        }
        // 53 bits, rounded to nearest even.
        let low = self.man & 0x7ff;
        let mut m = self.man >> 11;
        if low > 0x400 || (low == 0x400 && m & 1 == 1) {
            m += 1;
        }
        let v = scale(m as f64, self.exp + 11);
        if self.neg { -v } else { v }
    }

    pub fn is_zero(self) -> bool {
        self.man == 0
    }

    fn cmp_mag(a: Ext, b: Ext) -> Ordering {
        match (a.is_zero(), b.is_zero()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.exp.cmp(&b.exp).then(a.man.cmp(&b.man)),
        }
    }

    /// The largest integer not above the value (saturating).
    pub fn floor(self) -> i64 {
        if self.is_zero() {
            return 0;
        }
        if self.exp >= 0 {
            let v = if self.exp >= 64 {
                i64::MAX as i128
            } else {
                (self.man as i128) << self.exp
            }
            .min(i64::MAX as i128) as i64;
            return if self.neg { -v } else { v };
        }
        let s = (-self.exp) as u32;
        let (q, frac) = if s >= 64 {
            (0, true)
        } else {
            (self.man >> s, self.man & ((1u64 << s) - 1) != 0)
        };
        let q = q.min(i64::MAX as u64) as i64;
        if self.neg { -q - frac as i64 } else { q }
    }
}

impl std::ops::Neg for Ext {
    type Output = Ext;

    fn neg(self) -> Ext {
        if self.is_zero() {
            self
        } else {
            Ext {
                neg: !self.neg,
                ..self
            }
        }
    }
}

impl std::ops::Add for Ext {
    type Output = Ext;

    fn add(self, b: Ext) -> Ext {
        if self.is_zero() {
            return b;
        }
        if b.is_zero() {
            return self;
        }
        let (big, small) = if Ext::cmp_mag(self, b) == Ordering::Less {
            (b, self)
        } else {
            (self, b)
        };
        let d = (big.exp - small.exp) as u32;
        let wide = (small.man as u128) << 62;
        let (s, lost) = if d <= 126 {
            (wide >> d, d > 0 && wide & ((1u128 << d) - 1) != 0)
        } else {
            (0, true)
        };
        let b_wide = (big.man as u128) << 62;
        let m = if big.neg == small.neg {
            b_wide + s
        } else {
            // Dropped bits make the difference smaller by less than one
            // unit: subtract the unit and mark the result as a bit above.
            b_wide - s - lost as u128
        };
        Ext::round(big.neg, m, big.exp - 62, lost)
    }
}

impl std::ops::Sub for Ext {
    type Output = Ext;

    fn sub(self, b: Ext) -> Ext {
        self + -b
    }
}

impl std::ops::Mul for Ext {
    type Output = Ext;

    fn mul(self, b: Ext) -> Ext {
        if self.is_zero() || b.is_zero() {
            return Ext::ZERO;
        }
        let m = self.man as u128 * b.man as u128;
        Ext::round(self.neg != b.neg, m, self.exp + b.exp, false)
    }
}

/// `self / b`; zero for a zero divisor.
impl std::ops::Div for Ext {
    type Output = Ext;

    fn div(self, b: Ext) -> Ext {
        if self.is_zero() || b.is_zero() {
            return Ext::ZERO;
        }
        // 64 or 65 quotient bits, then 63 more for rounding.
        let d = b.man as u128;
        let num = (self.man as u128) << 64;
        let (q1, r1) = (num / d, num % d);
        let (q2, r2) = ((r1 << 63) / d, (r1 << 63) % d);
        Ext::round(
            self.neg != b.neg,
            q1 << 63 | q2,
            self.exp - 127 - b.exp,
            r2 != 0,
        )
    }
}

/// v·2^e without intermediate overflow or underflow.
fn scale(mut v: f64, mut e: i32) -> f64 {
    while e > 1000 {
        v *= 2f64.powi(1000);
        e -= 1000;
    }
    while e < -1000 {
        v *= 2f64.powi(-1000);
        e += 1000;
    }
    v * 2f64.powi(e)
}

impl PartialOrd for Ext {
    fn partial_cmp(&self, b: &Ext) -> Option<Ordering> {
        Some(match (self.neg && !self.is_zero(), b.neg && !b.is_zero()) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => Ext::cmp_mag(*self, *b),
            (true, true) => Ext::cmp_mag(*b, *self),
        })
    }
}

// ---- double-double, for cos and sin --------------------------------------

#[derive(Clone, Copy, Debug)]
struct Dd(f64, f64);

fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    (s, (a - (s - bb)) + (b - bb))
}

fn quick_two_sum(a: f64, b: f64) -> Dd {
    let s = a + b;
    Dd(s, b - (s - a))
}

impl Dd {
    fn add(self, y: Dd) -> Dd {
        let (s, e) = two_sum(self.0, y.0);
        let (t, f) = two_sum(self.1, y.1);
        let e = e + t;
        let r = quick_two_sum(s, e);
        quick_two_sum(r.0, r.1 + f)
    }

    fn mul(self, y: Dd) -> Dd {
        let p = self.0 * y.0;
        let e = self.0.mul_add(y.0, -p) + (self.0 * y.1 + self.1 * y.0);
        quick_two_sum(p, e)
    }

    fn div_f(self, n: f64) -> Dd {
        let q = self.0 / n;
        let p = q * n;
        let e = q.mul_add(n, -p);
        let r = ((self.0 - p) - e + self.1) / n;
        quick_two_sum(q, r)
    }

    fn neg(self) -> Dd {
        Dd(-self.0, -self.1)
    }
}

/// 2π as a double-double.
const TWO_PI: Dd = Dd(std::f64::consts::TAU, 2.4492935982947064e-16);

/// cos and sin of a double-double angle, to about 100 bits for moderate
/// angles.
fn cos_sin_dd(x: Dd) -> (Dd, Dd) {
    // Reduce to about [−π, π].
    let k = (x.0 / TWO_PI.0).round();
    let x = x.add(TWO_PI.mul(Dd(k, 0.0)).neg());
    let (mut c, mut s) = (Dd(0.0, 0.0), Dd(0.0, 0.0));
    let mut term = Dd(1.0, 0.0);
    for n in 0..80u32 {
        match n % 4 {
            0 => c = c.add(term),
            1 => s = s.add(term),
            2 => c = c.add(term.neg()),
            _ => s = s.add(term.neg()),
        }
        term = term.mul(x).div_f((n + 1) as f64);
        if term.0.abs() < 1e-40 {
            break;
        }
    }
    (c, s)
}

fn ext_to_dd(v: Ext) -> Dd {
    let hi_man = v.man & !0x7ff;
    let lo_man = v.man & 0x7ff;
    let sign = if v.neg { -1.0 } else { 1.0 };
    Dd(
        scale(sign * hi_man as f64, v.exp),
        scale(sign * lo_man as f64, v.exp),
    )
}

fn dd_to_ext(v: Dd) -> Ext {
    Ext::from_f64(v.0) + Ext::from_f64(v.1)
}

/// cos and sin of an extended-precision angle, rounded to extended.
pub fn cos_sin(x: Ext) -> (Ext, Ext) {
    let (c, s) = cos_sin_dd(ext_to_dd(x));
    (dd_to_ext(c), dd_to_ext(s))
}

/// cos and sin of a double angle, correctly rounded to doubles (barring
/// values within about 2⁻¹⁰⁰ of a tie).
pub fn cos_sin_f64(x: f64) -> (f64, f64) {
    let (c, s) = cos_sin_dd(Dd(x, 0.0));
    (c.0 + c.1, s.0 + s.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(v: f64) -> Ext {
        Ext::from_f64(v)
    }

    #[test]
    fn exact_round_trips_and_ops() {
        for v in [1.0, -2.5, 0.1, 1e-300, 123456.789] {
            assert_eq!(e(v).to_f64(), v);
        }
        assert_eq!((e(1.5) + e(2.25)).to_f64(), 3.75);
        assert_eq!((e(1.5) - e(2.25)).to_f64(), -0.75);
        assert_eq!((e(3.0) * e(-0.5)).to_f64(), -1.5);
        assert_eq!((e(1.0) / e(4.0)).to_f64(), 0.25);
        assert_eq!(e(2.5).floor(), 2);
        assert_eq!(e(-2.5).floor(), -3);
        assert_eq!(e(-2.0).floor(), -2);
        assert!(e(-1.0) < e(0.5) && e(2.0) > e(1.0) && e(-3.0) < e(-2.0));
    }

    #[test]
    fn keeps_64_bits() {
        // 1 + 2⁻⁶³ is exact in extended precision but not in double.
        let tiny = e(2f64.powi(-63));
        let one = e(1.0);
        assert_ne!(one + tiny, one);
        assert_eq!(one + tiny - one, tiny);
        // 1 + 2⁻⁶⁴ is a tie and rounds to even (1).
        assert_eq!(one + e(2f64.powi(-64)), one);
        // 1/3 has the 64-bit pattern 0xAAAA…AB.
        let third = one / e(3.0);
        assert_eq!(third.man, 0xAAAA_AAAA_AAAA_AAAB);
    }

    #[test]
    fn trig_is_accurate() {
        for x in [0.1, 1.0, 2.5, 6.0, 100.0, -7.25] {
            let (c, s) = cos_sin_f64(x);
            // Platform libms may be one unit in the last place off.
            assert!(
                (c - x.cos()).abs() <= 2.5e-16 && (s - x.sin()).abs() <= 2.5e-16,
                "{x}"
            );
        }
        // Exact to the last bit: sin(2.5) = 0.59847214410395649…
        assert_eq!(cos_sin_f64(2.5).1, 0.5984721441039565);
        let (c, s) = cos_sin(Ext::ZERO);
        assert_eq!((c.to_f64(), s.to_f64()), (1.0, 0.0));
    }
}
