// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether a formula's pixels depend on the pixels computed before it.
//!
//! Formula variables keep their values from one pixel to the next
//! (frm-semantics.md §1.5). A formula that reads a variable before giving it
//! a value in the current pixel therefore depends on calculation order and
//! must be computed sequentially; one that always assigns first can be
//! computed in any order (in parallel).

use std::collections::HashSet;

use crate::ast::{Expr, FormulaAst, Stmt};

/// Names Fractint sets afresh for every pixel (§1.5).
pub const REFRESHED: [&str; 4] = ["pixel", "scrnpix", "whitesq", "rand"];

/// Names set once at the start of an image (§2.1, §8).
pub const START_VALUES: [&str; 14] = [
    "p1", "p2", "p3", "p4", "p5", "pi", "e", "maxit", "scrnmax", "center", "magxmag", "rotskew",
    "ismand", "lastsqr",
];

pub fn order_dependent(f: &FormulaAst) -> bool {
    let mut assigned_anywhere = HashSet::new();
    for s in f.init.iter().chain(&f.iter) {
        collect_assigned(s, &mut assigned_anywhere);
    }
    let needs_value = |name: &str| {
        !REFRESHED.contains(&name)
            && !(START_VALUES.contains(&name) && !assigned_anywhere.contains(name))
    };
    let mut a = Walk {
        assigned: HashSet::new(),
        dependent: false,
        needs_value: &needs_value,
    };
    a.stmts(&f.init);
    // The first iteration runs with only what the initialisation assigned;
    // later iterations have at least as much.
    a.stmts(&f.iter);
    a.dependent
}

fn collect_assigned(s: &Stmt, out: &mut HashSet<String>) {
    fn expr(e: &Expr, out: &mut HashSet<String>) {
        match e {
            Expr::Assign(n, v) => {
                out.insert(n.clone());
                expr(v, out);
            }
            Expr::Neg(a) | Expr::Modulus(a) | Expr::Call(_, a) => expr(a, out),
            Expr::Binary(_, a, b) => {
                expr(a, out);
                expr(b, out);
            }
            Expr::Const(_) | Expr::Var(_) => {}
        }
    }
    match s {
        Stmt::Expr(e) => expr(e, out),
        Stmt::If { arms, otherwise } => {
            for (c, body) in arms {
                expr(c, out);
                body.iter().for_each(|s| collect_assigned(s, out));
            }
            otherwise.iter().for_each(|s| collect_assigned(s, out));
        }
    }
}

struct Walk<'a> {
    assigned: HashSet<String>,
    dependent: bool,
    needs_value: &'a dyn Fn(&str) -> bool,
}

impl Walk<'_> {
    fn stmts(&mut self, body: &[Stmt]) {
        for s in body {
            self.stmt(s);
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Expr(e) => self.expr(e),
            Stmt::If { arms, otherwise } => {
                let before = self.assigned.clone();
                let mut after: Option<HashSet<String>> = None;
                let mut cond_assigned = before.clone();
                let mut bodies: Vec<&[Stmt]> = arms.iter().map(|(_, b)| b.as_slice()).collect();
                bodies.push(otherwise);
                for (i, body) in bodies.iter().enumerate() {
                    if let Some((c, _)) = arms.get(i) {
                        self.assigned = cond_assigned.clone();
                        self.expr(c);
                        cond_assigned = self.assigned.clone();
                    }
                    self.assigned = cond_assigned.clone();
                    self.stmts(body);
                    after = Some(match after {
                        None => self.assigned.clone(),
                        Some(a) => a.intersection(&self.assigned).cloned().collect(),
                    });
                }
                // Assigned on every path.
                self.assigned = after.unwrap_or(before);
            }
        }
    }

    fn expr(&mut self, e: &Expr) {
        match e {
            Expr::Var(n) => {
                if !self.assigned.contains(n) && (self.needs_value)(n) {
                    self.dependent = true;
                }
            }
            Expr::Assign(n, v) => {
                self.expr(v);
                self.assigned.insert(n.clone());
            }
            Expr::Neg(a) | Expr::Modulus(a) | Expr::Call(_, a) => self.expr(a),
            Expr::Binary(_, a, b) => {
                self.expr(a);
                self.expr(b);
            }
            Expr::Const(_) => {}
        }
    }
}
