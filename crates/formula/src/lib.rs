// SPDX-License-Identifier: GPL-3.0-or-later

//! Fractint-format `.frm` formulas: parsing (docs/specs/frm-syntax.md) and
//! evaluation (docs/specs/frm-semantics.md, cited as §n).

pub mod analysis;
pub mod ast;
pub mod funcs;
pub mod parser;
mod vm;

use fractalforge_core::orbit_color::Orbit;
use fractalforge_core::{
    Carry, Complex, FractalProgram, INSIDE, Params, PixelContext, Program, Symmetry,
};

use ast::FormulaAst;
use vm::{Op, Slots};

/// A compiled formula.
pub struct Formula {
    name: String,
    source: String,
    symmetry: Option<String>,
    init: Vec<Op>,
    iter: Vec<Op>,
    slots: Vec<String>,
    order_dependent: bool,
    /// Slots of the predefined names, when the formula uses them.
    pixel: Option<u16>,
    scrnpix: Option<u16>,
    whitesq: Option<u16>,
    rand: Option<u16>,
    z: Option<u16>,
}

impl Formula {
    /// Compiles a parsed entry. `source` is kept for self-contained files.
    pub fn compile(ast: &FormulaAst, source: &str) -> Result<Self, String> {
        let mut slots = Slots::default();
        let init = vm::compile(&ast.init, &mut slots)?;
        let iter = vm::compile(&ast.iter, &mut slots)?;
        Ok(Self {
            name: ast.name.clone(),
            source: source.to_owned(),
            symmetry: ast.symmetry.clone(),
            order_dependent: analysis::order_dependent(ast),
            pixel: slots.get("pixel"),
            scrnpix: slots.get("scrnpix"),
            whitesq: slots.get("whitesq"),
            rand: slots.get("rand"),
            z: slots.get("z"),
            slots: slots.names,
            init,
            iter,
        })
    }

    pub fn into_program(self) -> Program {
        Program::new(self)
    }

    /// The value of a predefined name at the start of an image (§2.1).
    fn start_value(name: &str, p: &Params, width: u32, height: u32) -> Complex {
        let w = &p.window;
        let pair = |i: usize| Complex::new(p.params[i], p.params[i + 1]);
        match name {
            "p1" => pair(0),
            "p2" => pair(2),
            "p3" => pair(4),
            "p4" => pair(6),
            "p5" => pair(8),
            "pi" => Complex::new(std::f64::consts::PI, 0.0),
            "e" => Complex::new(std::f64::consts::E, 0.0),
            "maxit" => Complex::new(p.max_iter as f64, 0.0),
            "scrnmax" => Complex::new(width as f64, height as f64),
            "center" => Complex::new((w.x_min + w.x_max) / 2.0, (w.y_min + w.y_max) / 2.0),
            // xmag: how much narrower the view is than 4:3 (par.md §5.2).
            "magxmag" => Complex::new(
                2.0 / (w.y_max - w.y_min),
                (w.y_max - w.y_min) * (4.0 / 3.0) / (w.x_max - w.x_min),
            ),
            "ismand" => Complex::new(1.0, 0.0),
            // rotskew (rotated views aren't supported) and lastsqr are 0.
            _ => Complex::ZERO,
        }
    }

    /// A per-pixel pseudo-random value. Fractint's generator is unknown
    /// (§2.1), so this only has to be deterministic and position-dependent.
    fn rand_value(at: &PixelContext) -> Complex {
        let mut x = (at.row as u64) << 32 | at.col as u64;
        let mut next = || {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((x >> 33) as f64) / (1u64 << 31) as f64 * 32768.0
        };
        Complex::new(next(), next())
    }
}

impl FractalProgram for Formula {
    fn name(&self) -> &str {
        &self.name
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn new_carry(&self, params: &Params, width: u32, height: u32) -> Carry {
        Carry(
            self.slots
                .iter()
                .map(|n| Self::start_value(n, params, width, height))
                .collect(),
        )
    }

    /// §1.1 and §8: run the initialisation, then iterate up to M − 1 times;
    /// the last statement decides whether to continue.
    fn iterate(&self, at: &PixelContext, params: &Params, carry: &mut Carry) -> u32 {
        let vars = &mut carry.0;
        if vars.len() < self.slots.len() {
            vars.resize(self.slots.len(), Complex::ZERO);
        }
        let set = |vars: &mut Vec<Complex>, slot: Option<u16>, v: Complex| {
            if let Some(i) = slot {
                vars[i as usize] = v;
            }
        };
        set(vars, self.pixel, at.pixel);
        set(
            vars,
            self.scrnpix,
            Complex::new(at.col as f64, at.row as f64),
        );
        set(
            vars,
            self.whitesq,
            Complex::new(((at.row + at.col) % 2) as f64, 0.0),
        );
        set(vars, self.rand, Self::rand_value(at));

        let fns = params.functions;
        let mut stack = Vec::with_capacity(16);
        let mut orbit = params.needs_orbit_coloring().then(|| Orbit::new(params));
        let z = |vars: &Vec<Complex>| self.z.map_or(Complex::ZERO, |i| vars[i as usize]);
        let escaped = |k: u32, vars: &Vec<Complex>, orbit: &Option<Orbit>| match orbit {
            Some(o) => o.outside(k, z(vars), params) as u32,
            None => k,
        };

        if vm::run(&self.init, vars, &mut stack, fns).is_err() {
            return escaped(1, vars, &orbit);
        }
        for k in 1..params.max_iter {
            stack.clear();
            match vm::run(&self.iter, vars, &mut stack, fns) {
                Err(_) => return escaped(k, vars, &orbit),
                // An empty iteration section never stops (frm-syntax.md §2.3).
                Ok(v) if v.re == 0.0 && !self.iter.is_empty() => return escaped(k, vars, &orbit),
                Ok(_) => {}
            }
            if let Some(o) = &mut orbit {
                o.record(z(vars));
            }
        }
        match &orbit {
            Some(o) => o.inside(z(vars), params) as u32,
            None => INSIDE,
        }
    }

    fn order_dependent(&self) -> bool {
        self.order_dependent
    }

    /// §6.1–6.2.
    fn symmetry(&self, p: &Params) -> Symmetry {
        let Some(word) = &self.symmetry else {
            return Symmetry::None;
        };
        let word = word.to_ascii_uppercase();
        // PI_SYM has an underscore in its name: split off only the
        // _NOPARM / _NOREAL / _NOIMAG condition.
        let word = word.replacen("PI_SYM", "PI", 1);
        let (base, cond) = word.split_once('_').unwrap_or((&word, ""));
        // Real parts are the odd-numbered params (P1, P3, …), imaginary
        // parts the even ones; P4 and above count like P1–P3 (hypothesis).
        let real_set = p.params.iter().step_by(2).any(|&v| v != 0.0);
        let imag_set = p.params.iter().skip(1).step_by(2).any(|&v| v != 0.0);
        let allowed = match cond {
            "" => true,
            "NOPARM" => !real_set && !imag_set,
            "NOREAL" => !real_set,
            "NOIMAG" => !imag_set,
            _ => false,
        };
        if !allowed {
            return Symmetry::None;
        }
        match base {
            "XAXIS" => Symmetry::XAxis,
            "YAXIS" => Symmetry::YAxis,
            "XYAXIS" => Symmetry::XY,
            "ORIGIN" => Symmetry::Origin,
            "PI" => Symmetry::Pi,
            _ => Symmetry::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::{BinOp, Expr, Stmt};
    use fractalforge_core::types;

    fn var(n: &str) -> Box<Expr> {
        Box::new(Expr::Var(n.into()))
    }
    fn num(re: f64) -> Box<Expr> {
        Box::new(Expr::Const(Complex::new(re, 0.0)))
    }
    fn bin(op: BinOp, a: Box<Expr>, b: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Binary(op, a, b))
    }
    fn set(n: &str, e: Box<Expr>) -> Stmt {
        Stmt::Expr(Expr::Assign(n.into(), e))
    }

    /// `z = pixel: z = z*z + pixel, |z| <= 4`
    fn quad() -> FormulaAst {
        FormulaAst {
            name: "quad".into(),
            symmetry: Some("XAXIS".into()),
            init: vec![set("z", var("pixel"))],
            iter: vec![
                set(
                    "z",
                    bin(
                        BinOp::Add,
                        bin(BinOp::Mul, var("z"), var("z")),
                        var("pixel"),
                    ),
                ),
                Stmt::Expr(*bin(BinOp::Le, Box::new(Expr::Modulus(var("z"))), num(4.0))),
            ],
        }
    }

    fn params(f: Formula) -> Params {
        let mut p = Params::new(&types::FORMULA);
        p.program = Some(f.into_program());
        p
    }

    fn run(p: &Params, re: f64, im: f64) -> u32 {
        let at = PixelContext {
            pixel: Complex::new(re, im),
            col: 0,
            row: 0,
            width: 1,
            height: 1,
        };
        let prog = p.program.as_ref().unwrap();
        let mut carry = prog.0.new_carry(p, 1, 1);
        prog.0.iterate(&at, p, &mut carry)
    }

    #[test]
    fn quadratic_counts_like_the_spec() {
        let f = Formula::compile(&quad(), "").unwrap();
        assert!(!f.order_dependent());
        let p = params(f);
        // e = index of the first iterate that fails; the start isn't tested.
        // pixel 1: z = 1 → 2 (|z|² 4, true) → 5 (false) at k = 2.
        assert_eq!(run(&p, 1.0, 0.0), 2);
        // pixel 3 starts outside but the first iterate (12) fails: k = 1.
        assert_eq!(run(&p, 3.0, 0.0), 1);
        assert_eq!(run(&p, 0.0, 0.0), INSIDE);
        assert_eq!(p.program.as_ref().unwrap().0.symmetry(&p), Symmetry::XAxis);
    }

    #[test]
    fn errors_escape_and_conditions_use_real_parts() {
        // init: w = 1/0 (unused) → escape at count 1 (§1.7).
        let f = FormulaAst {
            name: "er".into(),
            symmetry: None,
            init: vec![set("w", bin(BinOp::Div, num(1.0), num(0.0)))],
            iter: vec![Stmt::Expr(*num(1.0))],
        };
        assert_eq!(run(&params(Formula::compile(&f, "").unwrap()), 0.5, 0.5), 1);
        // Condition (0, 5) is false: only the real part counts (§1.2).
        let f = FormulaAst {
            name: "t".into(),
            symmetry: None,
            init: vec![],
            iter: vec![Stmt::Expr(Expr::Const(Complex::new(0.0, 5.0)))],
        };
        assert_eq!(run(&params(Formula::compile(&f, "").unwrap()), 0.0, 0.0), 1);
    }

    #[test]
    fn if_else_and_order_dependence() {
        // if (real(pixel) < 0) w = 1 else w = 0 endif; iteration: w
        let f = FormulaAst {
            name: "ife".into(),
            symmetry: None,
            init: vec![Stmt::If {
                arms: vec![(
                    *bin(
                        BinOp::Lt,
                        Box::new(Expr::Call("real".into(), var("pixel"))),
                        num(0.0),
                    ),
                    vec![set("w", num(1.0))],
                )],
                otherwise: vec![set("w", num(0.0))],
            }],
            iter: vec![Stmt::Expr(*var("w"))],
        };
        let c = Formula::compile(&f, "").unwrap();
        assert!(!c.order_dependent());
        let p = params(c);
        assert_eq!(run(&p, -1.0, 0.0), INSIDE);
        assert_eq!(run(&p, 1.0, 0.0), 1);

        // Reading z before assigning it carries the previous pixel's value.
        let mut zun = quad();
        zun.init = vec![];
        assert!(Formula::compile(&zun, "").unwrap().order_dependent());
        // Assigning only in one branch isn't enough.
        let f = FormulaAst {
            name: "half".into(),
            symmetry: None,
            init: vec![Stmt::If {
                arms: vec![(*var("pixel"), vec![set("w", num(1.0))])],
                otherwise: vec![],
            }],
            iter: vec![Stmt::Expr(*var("w"))],
        };
        assert!(Formula::compile(&f, "").unwrap().order_dependent());
        // Using p1 is fine unless the formula assigns it.
        let f = FormulaAst {
            name: "p".into(),
            symmetry: None,
            init: vec![],
            iter: vec![Stmt::Expr(*var("p1"))],
        };
        assert!(!Formula::compile(&f, "").unwrap().order_dependent());
        let f = FormulaAst {
            name: "ap1".into(),
            symmetry: None,
            init: vec![set("p1", bin(BinOp::Add, var("p1"), num(0.001)))],
            iter: vec![Stmt::Expr(*var("p1"))],
        };
        assert!(Formula::compile(&f, "").unwrap().order_dependent());
    }

    #[test]
    fn symmetry_qualifiers() {
        let mut ast = quad();
        for (word, p1, want) in [
            ("XAXIS_NOPARM", 0.0, Symmetry::XAxis),
            ("XAXIS_NOPARM", 0.1, Symmetry::None),
            ("XAXIS_NOIMAG", 0.1, Symmetry::XAxis),
            ("XAXIS_NOREAL", 0.1, Symmetry::None),
            ("YAXIS", 0.0, Symmetry::YAxis),
            ("XYAXIS", 0.0, Symmetry::XY),
            ("ORIGIN", 0.0, Symmetry::Origin),
            ("PI_SYM", 0.0, Symmetry::Pi),
            ("PI_SYM_NOPARM", 0.1, Symmetry::None),
            ("NOSYM", 0.0, Symmetry::None),
        ] {
            ast.symmetry = Some(word.into());
            let mut p = params(Formula::compile(&ast, "").unwrap());
            p.params[0] = p1;
            assert_eq!(
                p.program.as_ref().unwrap().0.symmetry(&p),
                want,
                "{word} {p1}"
            );
        }
    }

    #[test]
    fn unknown_functions_are_rejected() {
        let f = FormulaAst {
            name: "u".into(),
            symmetry: None,
            init: vec![],
            iter: vec![Stmt::Expr(Expr::Call("cot".into(), var("z")))],
        };
        assert!(Formula::compile(&f, "").is_err());
    }
}
