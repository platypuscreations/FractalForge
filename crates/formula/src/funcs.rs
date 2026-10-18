// SPDX-License-Identifier: GPL-3.0-or-later

//! Formula functions and operators, as defined in docs/specs/frm-semantics.md
//! (cited as §n). Operations that Fractint treats as math errors (§1.7)
//! return `Err`.

use std::f64::consts::FRAC_PI_2;

use fractalforge_core::Complex;

/// A math error ends the pixel as escaped (§1.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MathError;

pub type R = Result<Complex, MathError>;

const ONE: Complex = Complex::new(1.0, 0.0);
const I: Complex = Complex::new(0.0, 1.0);

fn c(re: f64, im: f64) -> Complex {
    Complex::new(re, im)
}

/// Complex division; a zero divisor is a math error.
pub fn div(a: Complex, b: Complex) -> R {
    if b.re == 0.0 && b.im == 0.0 {
        return Err(MathError);
    }
    Ok(a / b)
}

/// Principal logarithm; log 0 is a math error.
pub fn log(z: Complex) -> R {
    if z.re == 0.0 && z.im == 0.0 {
        return Err(MathError);
    }
    Ok(c(z.norm_sqr().sqrt().ln(), z.im.atan2(z.re)))
}

pub fn exp(z: Complex) -> Complex {
    let r = z.re.exp();
    c(r * z.im.cos(), r * z.im.sin())
}

/// `a ^ b = exp(b · log a)`; 0^b is a math error (§3).
pub fn pow(a: Complex, b: Complex) -> R {
    Ok(exp(b * log(a)?))
}

/// Principal square root in polar form (§4.1).
pub fn sqrt(z: Complex) -> Complex {
    let r = z.norm_sqr().sqrt().sqrt();
    let half = z.im.atan2(z.re) / 2.0;
    c(r * half.cos(), r * half.sin())
}

pub fn sin(z: Complex) -> Complex {
    c(z.re.sin() * z.im.cosh(), z.re.cos() * z.im.sinh())
}

pub fn cos(z: Complex) -> Complex {
    c(z.re.cos() * z.im.cosh(), -(z.re.sin() * z.im.sinh()))
}

pub fn sinh(z: Complex) -> Complex {
    c(z.re.sinh() * z.im.cos(), z.re.cosh() * z.im.sin())
}

pub fn cosh(z: Complex) -> Complex {
    c(z.re.cosh() * z.im.cos(), z.re.sinh() * z.im.sin())
}

// tan and relatives divide by a denominator that is exactly 0 at their
// poles, which is a math error (§1.7: tan(π/2), cotan(0)).

pub fn tan(z: Complex) -> R {
    let (x, y) = (2.0 * z.re, 2.0 * z.im);
    let d = x.cos() + y.cosh();
    if d == 0.0 {
        return Err(MathError);
    }
    Ok(c(x.sin() / d, y.sinh() / d))
}

pub fn tanh(z: Complex) -> R {
    let (x, y) = (2.0 * z.re, 2.0 * z.im);
    let d = x.cosh() + y.cos();
    if d == 0.0 {
        return Err(MathError);
    }
    Ok(c(x.sinh() / d, y.sin() / d))
}

pub fn cotan(z: Complex) -> R {
    let (x, y) = (2.0 * z.re, 2.0 * z.im);
    let d = y.cosh() - x.cos();
    if d == 0.0 {
        return Err(MathError);
    }
    Ok(c(x.sin() / d, -y.sinh() / d))
}

pub fn cotanh(z: Complex) -> R {
    let (x, y) = (2.0 * z.re, 2.0 * z.im);
    let d = x.cosh() - y.cos();
    if d == 0.0 {
        return Err(MathError);
    }
    Ok(c(x.sinh() / d, -y.sin() / d))
}

/// −i·log(i·z + √(1 − z²)).
pub fn asin(z: Complex) -> R {
    let w = log(I * z + sqrt(ONE - z.square()))?;
    Ok(c(w.im, -w.re))
}

/// log(z + √(z² + 1)).
pub fn asinh(z: Complex) -> R {
    log(z + sqrt(z.square() + ONE))
}

/// −i·log(z + √(z² − 1)) (not the principal arccos, §4.1).
pub fn acos(z: Complex) -> R {
    let w = log(z + sqrt(z.square() - ONE))?;
    Ok(c(w.im, -w.re))
}

/// log(z + √(z² − 1)) (not the principal arccosh, §4.1).
pub fn acosh(z: Complex) -> R {
    log(z + sqrt(z.square() - ONE))
}

/// Principal arctangent, except that on the cut above +i the real part is
/// −π/2; at ±i it returns its argument (§4.1).
pub fn atan(z: Complex) -> R {
    if z.re == 0.0 && (z.im == 1.0 || z.im == -1.0) {
        return Ok(z);
    }
    // atan z = (i/2)·(log(1 − iz) − log(1 + iz))
    let iz = I * z;
    let w = log(ONE - iz)? - log(ONE + iz)?;
    let mut r = c(-w.im / 2.0, w.re / 2.0);
    if z.re == 0.0 && z.im.abs() > 1.0 {
        r.re = -FRAC_PI_2;
    }
    Ok(r)
}

/// Principal inverse hyperbolic tangent; at ±1 it returns its argument.
pub fn atanh(z: Complex) -> R {
    if z.im == 0.0 && (z.re == 1.0 || z.re == -1.0) {
        return Ok(z);
    }
    // atanh z = (log(1 + z) − log(1 − z)) / 2
    let w = log(ONE + z)? - log(ONE - z)?;
    Ok(c(w.re / 2.0, w.im / 2.0))
}

/// A built-in function resolved at compile time.
pub type Builtin = fn(Complex) -> R;

/// The built-in function with this name (§4.1), or `None` if it isn't one.
pub fn lookup(name: &str) -> Option<Builtin> {
    Some(match name {
        "sin" => |z| Ok(sin(z)),
        "cos" => |z| Ok(cos(z)),
        // Cosine with the imaginary part negated.
        "cosxx" => |z| Ok(c(cos(z).re, -cos(z).im)),
        "tan" => tan,
        "cotan" => cotan,
        "sinh" => |z| Ok(sinh(z)),
        "cosh" => |z| Ok(cosh(z)),
        "tanh" => tanh,
        "cotanh" => cotanh,
        "exp" => |z| Ok(exp(z)),
        "log" => log,
        "sqr" => |z| Ok(z.square()),
        "sqrt" => |z| Ok(sqrt(z)),
        "abs" => |z| Ok(c(z.re.abs(), z.im.abs())),
        "cabs" => |z| Ok(c(z.norm_sqr().sqrt(), 0.0)),
        "conj" => |z| Ok(c(z.re, -z.im)),
        "real" => |z| Ok(c(z.re, 0.0)),
        "imag" => |z| Ok(c(z.im, 0.0)),
        "flip" => |z| Ok(c(z.im, z.re)),
        "floor" => |z| Ok(c(z.re.floor(), z.im.floor())),
        "ceil" => |z| Ok(c(z.re.ceil(), z.im.ceil())),
        "trunc" => |z| Ok(c(z.re.trunc(), z.im.trunc())),
        "round" => |z| Ok(c((z.re + 0.5).floor(), (z.im + 0.5).floor())),
        "asin" => asin,
        "asinh" => asinh,
        "acos" => acos,
        "acosh" => acosh,
        "atan" => atan,
        "atanh" => atanh,
        // Parses in Fractint, but its value is unknown (frm-semantics.md
        // §2.1): the identity here.
        "srand" => Ok,
        _ => return None,
    })
}

/// Calls a built-in function by name. `None` for unknown names.
pub fn call(name: &str, z: Complex) -> Option<R> {
    lookup(name).map(|f| f(z))
}

/// The functions `function=` can bind to fn1…fn4, by their stored index
/// (gif-fractint-blocks.md §6.2, frm-semantics.md §4.2).
pub const FN_TABLE: [&str; 31] = [
    "sin", "cosxx", "sinh", "cosh", "exp", "log", "sqr", "recip", "ident", "cos", "tan", "tanh",
    "cotan", "cotanh", "flip", "conj", "zero", "asin", "asinh", "acos", "acosh", "atan", "atanh",
    "cabs", "abs", "sqrt", "floor", "ceil", "trunc", "round", "one",
];

/// Calls the function at `index` of [`FN_TABLE`] (unknown indices act as
/// the identity).
pub fn call_index(index: u8, z: Complex) -> R {
    match FN_TABLE.get(index as usize).copied() {
        Some("recip") => div(ONE, z),
        Some("ident") | None => Ok(z),
        Some("zero") => Ok(Complex::ZERO),
        Some("one") => Ok(ONE),
        Some(name) => call(name, z).unwrap_or(Ok(z)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Complex, b: Complex) -> bool {
        (a - b).norm_sqr() < 1e-24
    }

    #[test]
    fn errors() {
        assert!(div(ONE, Complex::ZERO).is_err());
        assert!(log(Complex::ZERO).is_err());
        assert!(pow(Complex::ZERO, c(2.0, 0.0)).is_err());
        assert!(cotan(Complex::ZERO).is_err());
        // cos(π) + cosh(0) is exactly 0 in floating point.
        assert!(tan(c(std::f64::consts::FRAC_PI_2, 0.0)).is_err());
    }

    #[test]
    fn identities() {
        let z = c(0.3, -0.7);
        assert!(close(sqrt(z).square(), z));
        assert!(close(exp(log(z).unwrap()), z));
        assert!(close(tan(z).unwrap(), sin(z) / cos(z)));
        assert!(close(cotan(z).unwrap(), cos(z) / sin(z)));
        assert!(close(tanh(z).unwrap(), sinh(z) / cosh(z)));
        assert!(close(sin(asin(z).unwrap()), z));
        assert!(close(tan(atan(z).unwrap()).unwrap(), z));
        assert!(close(tanh(atanh(z).unwrap()).unwrap(), z));
        assert!(close(pow(c(-4.0, 0.0), c(0.5, 0.0)).unwrap(), c(0.0, 2.0)));
    }

    #[test]
    fn special_cases() {
        assert_eq!(atan(c(0.0, 1.0)).unwrap(), c(0.0, 1.0));
        assert_eq!(atanh(c(-1.0, 0.0)).unwrap(), c(-1.0, 0.0));
        assert_eq!(atan(c(0.0, 2.0)).unwrap().re, -FRAC_PI_2);
        assert_eq!(atan(c(0.0, -2.0)).unwrap().re, -FRAC_PI_2);
        assert_eq!(call("imag", c(1.0, 2.0)).unwrap().unwrap(), c(2.0, 0.0));
        assert_eq!(call("round", c(-0.5, 2.5)).unwrap().unwrap(), c(0.0, 3.0));
        assert!(call("cot", ONE).is_none());
        assert_eq!(call_index(16, ONE).unwrap(), Complex::ZERO);
        assert_eq!(FN_TABLE[0..4], ["sin", "cosxx", "sinh", "cosh"]);
    }
}
