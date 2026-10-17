// SPDX-License-Identifier: GPL-3.0-or-later

//! A minimal `f64` complex number.

use std::ops::{Add, Mul, Neg, Sub};

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
}
