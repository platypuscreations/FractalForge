// SPDX-License-Identifier: GPL-3.0-or-later

//! Compiles formula statements to a small stack machine and runs them
//! (frm-semantics.md, cited as §n).

use std::collections::HashMap;

use fractalforge_core::Complex;

use crate::ast::{BinOp, Expr, Stmt};
use crate::funcs::{self, Builtin, MathError};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Op {
    Const(Complex),
    Load(u16),
    /// Stores the top of the stack, leaving it there.
    Store(u16),
    Neg,
    Bin(BinOp),
    Modulus,
    Call(Builtin),
    /// `fn1`…`fn4`, bound through `function=`.
    CallFn(u8),
    /// Pops the value of a statement into the "last value" register.
    Last,
    /// Pops a condition into the last-value register and jumps if false.
    JumpIfFalse(u32),
    Jump(u32),
}

/// Variable names to slots.
#[derive(Default)]
pub(crate) struct Slots {
    pub names: Vec<String>,
    index: HashMap<String, u16>,
}

impl Slots {
    pub fn slot(&mut self, name: &str) -> u16 {
        if let Some(&i) = self.index.get(name) {
            return i;
        }
        let i = self.names.len() as u16;
        self.names.push(name.to_owned());
        self.index.insert(name.to_owned(), i);
        i
    }

    pub fn get(&self, name: &str) -> Option<u16> {
        self.index.get(name).copied()
    }
}

/// Compiles statements; errors name unknown functions.
pub(crate) fn compile(body: &[Stmt], slots: &mut Slots) -> Result<Vec<Op>, String> {
    let mut code = Vec::new();
    for s in body {
        stmt(s, slots, &mut code)?;
    }
    Ok(code)
}

fn stmt(s: &Stmt, slots: &mut Slots, code: &mut Vec<Op>) -> Result<(), String> {
    match s {
        Stmt::Expr(e) => {
            expr(e, slots, code)?;
            code.push(Op::Last);
        }
        Stmt::If { arms, otherwise } => {
            let mut to_end = Vec::new();
            for (cond, body) in arms {
                expr(cond, slots, code)?;
                let skip = code.len();
                code.push(Op::JumpIfFalse(0));
                for s in body {
                    stmt(s, slots, code)?;
                }
                to_end.push(code.len());
                code.push(Op::Jump(0));
                code[skip] = Op::JumpIfFalse(code.len() as u32);
            }
            for s in otherwise {
                stmt(s, slots, code)?;
            }
            let end = code.len() as u32;
            for j in to_end {
                code[j] = Op::Jump(end);
            }
        }
    }
    Ok(())
}

fn expr(e: &Expr, slots: &mut Slots, code: &mut Vec<Op>) -> Result<(), String> {
    match e {
        Expr::Const(c) => code.push(Op::Const(*c)),
        Expr::Var(n) => code.push(Op::Load(slots.slot(n))),
        Expr::Assign(n, v) => {
            expr(v, slots, code)?;
            code.push(Op::Store(slots.slot(n)));
        }
        Expr::Neg(a) => {
            expr(a, slots, code)?;
            code.push(Op::Neg);
        }
        Expr::Modulus(a) => {
            expr(a, slots, code)?;
            code.push(Op::Modulus);
        }
        Expr::Binary(op, a, b) => {
            expr(a, slots, code)?;
            expr(b, slots, code)?;
            code.push(Op::Bin(*op));
        }
        Expr::Call(name, a) => {
            expr(a, slots, code)?;
            let fn_index = ["fn1", "fn2", "fn3", "fn4"].iter().position(|f| f == name);
            match (fn_index, funcs::lookup(name)) {
                (Some(i), _) => code.push(Op::CallFn(i as u8)),
                (None, Some(f)) => code.push(Op::Call(f)),
                (None, None) => return Err(format!("unknown function '{name}'")),
            }
        }
    }
    Ok(())
}

fn truth(v: Complex) -> bool {
    // False exactly when the real part is 0; NaN counts as true (§1.2).
    v.re != 0.0
}

fn flag(b: bool) -> Complex {
    Complex::new(if b { 1.0 } else { 0.0 }, 0.0)
}

/// Runs compiled code. Returns the value of the last statement or
/// condition evaluated.
pub(crate) fn run(
    code: &[Op],
    vars: &mut [Complex],
    stack: &mut Vec<Complex>,
    fns: [u8; 4],
) -> Result<Complex, MathError> {
    let mut last = Complex::ZERO;
    let mut pc = 0;
    while pc < code.len() {
        match code[pc] {
            Op::Const(c) => stack.push(c),
            Op::Load(i) => stack.push(vars[i as usize]),
            Op::Store(i) => vars[i as usize] = *stack.last().unwrap(),
            Op::Neg => {
                // 0 − a, so a zero part stays +0: (−1)^0.5 is +i, not −i
                // (frm-semantics.md §3, `mpowneg`).
                let a = stack.pop().unwrap();
                stack.push(Complex::new(0.0 - a.re, 0.0 - a.im));
            }
            Op::Modulus => {
                let a = stack.pop().unwrap();
                stack.push(Complex::new(a.norm_sqr(), 0.0));
            }
            Op::Bin(op) => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(match op {
                    BinOp::Add => a + b,
                    BinOp::Sub => a - b,
                    BinOp::Mul => a * b,
                    BinOp::Div => funcs::div(a, b)?,
                    BinOp::Pow => funcs::pow(a, b)?,
                    // Comparisons use real parts only (§3).
                    BinOp::Lt => flag(a.re < b.re),
                    BinOp::Le => flag(a.re <= b.re),
                    BinOp::Gt => flag(a.re > b.re),
                    BinOp::Ge => flag(a.re >= b.re),
                    BinOp::Eq => flag(a.re == b.re),
                    BinOp::Ne => flag(a.re != b.re),
                    BinOp::And => flag(truth(a) && truth(b)),
                    BinOp::Or => flag(truth(a) || truth(b)),
                });
            }
            Op::Call(f) => {
                let a = stack.pop().unwrap();
                stack.push(f(a)?);
            }
            Op::CallFn(i) => {
                let a = stack.pop().unwrap();
                stack.push(funcs::call_index(fns[i as usize], a)?);
            }
            Op::Last => last = stack.pop().unwrap(),
            Op::JumpIfFalse(t) => {
                last = stack.pop().unwrap();
                if !truth(last) {
                    pc = t as usize;
                    continue;
                }
            }
            Op::Jump(t) => {
                pc = t as usize;
                continue;
            }
        }
        pc += 1;
    }
    Ok(last)
}
