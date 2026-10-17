// SPDX-License-Identifier: GPL-3.0-or-later

//! A minimal `f64` complex number.

use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };

    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// `|z|²`, avoiding the square root.
    pub fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn square(self) -> Self {
        Self::new(
            self.re * self.re - self.im * self.im,
            2.0 * self.re * self.im,
        )
    }

    /// `z^n` for any integer `n` (by repeated squaring; `z^0 = 1`).
    pub fn powi(self, n: i32) -> Self {
        let mut base = if n < 0 {
            Self::new(1.0, 0.0) / self
        } else {
            self
        };
        let mut e = n.unsigned_abs();
        let mut acc = Self::new(1.0, 0.0);
        while e > 0 {
            if e & 1 == 1 {
                acc = acc * base;
            }
            base = base.square();
            e >>= 1;
        }
        acc
    }

    pub fn scale(self, k: f64) -> Self {
        Self::new(self.re * k, self.im * k)
    }
}

impl Div for Complex {
    type Output = Self;
    fn div(self, o: Self) -> Self {
        let d = o.re * o.re + o.im * o.im;
        Self::new(
            (self.re * o.re + self.im * o.im) / d,
            (self.im * o.re - self.re * o.im) / d,
        )
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self::new(self.re + o.re, self.im + o.im)
    }
}

impl Sub for Complex {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self::new(self.re - o.re, self.im - o.im)
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }
}

impl Neg for Complex {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.re, -self.im)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_matches_mul() {
        let z = Complex::new(1.5, -0.25);
        assert_eq!(z.square(), z * z);
        assert_eq!(Complex::new(0.0, 1.0).square(), Complex::new(-1.0, 0.0));
        assert_eq!(Complex::new(3.0, 4.0).norm_sqr(), 25.0);
    }

    #[test]
    fn powers_and_division() {
        let z = Complex::new(0.3, -1.2);
        assert_eq!(z.powi(0), Complex::new(1.0, 0.0));
        assert_eq!(z.powi(1), z);
        assert_eq!(z.powi(2), z.square());
        let p4 = z.powi(4);
        let q = z.square().square();
        assert!((p4 - q).norm_sqr() < 1e-24);
        let r = z.powi(-2) * z.square();
        assert!((r - Complex::new(1.0, 0.0)).norm_sqr() < 1e-24);
        assert!(((z / z) - Complex::new(1.0, 0.0)).norm_sqr() < 1e-30);
        assert_eq!(
            Complex::new(1.0, 0.0) / Complex::new(0.0, 1.0),
            Complex::new(0.0, -1.0)
        );
    }
}
