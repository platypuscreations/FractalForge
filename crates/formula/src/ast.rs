// SPDX-License-Identifier: GPL-3.0-or-later

//! The parsed form of a formula, independent of the source syntax.

use fractalforge_core::Complex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Const(Complex),
    Var(String),
    Neg(Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// `|e|`: the squared modulus (frm-semantics.md §3).
    Modulus(Box<Expr>),
    /// A built-in function or `fn1`…`fn4`.
    Call(String, Box<Expr>),
    /// `name = e`; its value is the assigned value.
    Assign(String, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    /// `if`/`elseif` arms in order, then the optional `else` body.
    If {
        arms: Vec<(Expr, Vec<Stmt>)>,
        otherwise: Vec<Stmt>,
    },
}

/// One formula entry.
#[derive(Clone, Debug, PartialEq)]
pub struct FormulaAst {
    pub name: String,
    /// The symmetry word in the entry header, if any (e.g. `XAXIS`).
    pub symmetry: Option<String>,
    pub init: Vec<Stmt>,
    pub iter: Vec<Stmt>,
}
