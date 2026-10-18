// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading `.frm` files, following docs/specs/frm-syntax.md (cited as §n).

use std::fmt;

use fractalforge_core::Complex;

use crate::ast::{BinOp, Expr, FormulaAst, Stmt};

/// Built-in function names (§6.2); `fn1`…`fn4` are bound by `function=`.
pub const FUNCTIONS: [&str; 34] = [
    "abs", "acos", "acosh", "asin", "asinh", "atan", "atanh", "cabs", "ceil", "conj", "cos",
    "cosh", "cosxx", "cotan", "cotanh", "exp", "flip", "floor", "fn1", "fn2", "fn3", "fn4", "imag",
    "log", "real", "round", "sin", "sinh", "sqr", "sqrt", "srand", "tan", "tanh", "trunc",
];

fn is_function(name: &str) -> bool {
    FUNCTIONS.contains(&name)
}

const KEYWORDS: [&str; 4] = ["if", "elseif", "else", "endif"];

/// Identifiers and number literals are limited to 32 characters (§1.7).
const MAX_TOKEN: usize = 32;
/// 200 or more jump keywords are rejected (§1.7).
const MAX_JUMPS: usize = 199;
/// At most three errors are reported (§7.1).
const MAX_ERRORS: usize = 3;

/// One entry found in a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    /// The qualifier word, upper-cased with blanks removed (§1.5).
    pub qualifier: Option<String>,
    /// The entry's full text, header to `}`, for self-contained storage.
    pub text: String,
    /// Body lines with their line numbers (the header line is 1, §7.1).
    lines: Vec<(usize, String)>,
    /// A `(` without `)` on the header line (§1.2).
    bad_qualifier: bool,
}

/// A formula error, numbered like XFractint's (§7.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub code: Option<u8>,
    pub line: usize,
    pub message: String,
    pub statement: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.code {
            Some(c) => write!(f, "Error({c}) at line {}: {}", self.line, self.message)?,
            None => write!(f, "{}", self.message)?,
        }
        if !self.statement.is_empty() {
            write!(f, "\n  {}", self.statement)?;
        }
        Ok(())
    }
}

/// A parsed entry and notes about constructs that behave unexpectedly.
#[derive(Clone, Debug, PartialEq)]
pub struct Parsed {
    pub ast: FormulaAst,
    pub warnings: Vec<String>,
}

// ---- §1: file structure ------------------------------------------------

/// Finds every well-formed entry in a file (§1.2, §9.1 steps 1–2).
/// Entries without a closing `}`, or with a `{` in the body, are skipped
/// because XFractint can't find them.
pub fn entries(text: &str) -> Vec<Entry> {
    let text = text.split('\u{1a}').next().unwrap_or("");
    let lines: Vec<&str> = text
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    let mut out = Vec::new();
    let (mut li, mut col) = (0usize, 0usize);
    while li < lines.len() {
        let line = &lines[li][col.min(lines[li].len())..];
        let code = line.split(';').next().unwrap_or("");
        let Some(brace) = code.find('{') else {
            li += 1;
            col = 0;
            continue;
        };
        let head = &code[..brace];
        let trimmed = head.trim_start_matches([' ', '\t']);
        let name: String = trimmed
            .chars()
            .take_while(|c| !matches!(c, ' ' | '\t' | '(' | '[' | '{'))
            .collect();
        let after_name = trimmed[name.len()..].trim_start_matches([' ', '\t']);
        let qualifier = after_name.strip_prefix('(').and_then(|q| {
            q.find(')')
                .map(|end| q[..end].replace([' ', '\t'], "").to_ascii_uppercase())
        });
        let bad_qualifier = after_name.starts_with('(') && qualifier.is_none();

        // Body: from after `{` to the first `}` outside a comment.
        let start_col = col + brace + 1;
        let mut body_lines = Vec::new();
        let (mut bl, mut bc) = (li, start_col);
        let mut end = None;
        let mut broken = None;
        while bl < lines.len() {
            let full = lines[bl];
            let rest = &full[bc.min(full.len())..];
            let code_part = rest.split(';').next().unwrap_or("");
            let close = code_part.find('}');
            let open = code_part.find('{');
            if let Some(o) = open
                && close.is_none_or(|c| o < c)
                && broken.is_none()
            {
                broken = Some((bl, bc + o));
            }
            match close {
                Some(c) => {
                    body_lines.push((bl - li + 1, rest[..c].to_owned()));
                    end = Some((bl, bc + c + 1));
                    break;
                }
                None => {
                    body_lines.push((bl - li + 1, rest.to_owned()));
                    bl += 1;
                    bc = 0;
                }
            }
        }
        let Some((end_line, end_col)) = end else {
            // No closing brace: not an entry; later lines may hold entries.
            li += 1;
            col = 0;
            continue;
        };
        if let Some((bl, bc)) = broken {
            // A `{` in the body: not an entry (§1.2). The `{` may start the
            // next entry, so scan again from its line.
            if bl == li {
                col = bc + 1;
            } else {
                li = bl;
                col = 0;
            }
            continue;
        }
        if !name.is_empty() {
            let mut entry_text = String::new();
            for (i, l) in lines.iter().enumerate().take(end_line + 1).skip(li) {
                let from = if i == li { col } else { 0 };
                let to = if i == end_line { end_col } else { l.len() };
                entry_text.push_str(&l[from.min(to)..to]);
                if i != end_line {
                    entry_text.push('\n');
                }
            }
            out.push(Entry {
                name,
                qualifier,
                text: entry_text.trim().to_owned(),
                lines: body_lines,
                bad_qualifier,
            });
        }
        // Another entry may start after the `}` on the same line (§1.2).
        li = end_line;
        col = end_col;
    }
    out
}

/// The first entry with this name, compared without case (§1.3).
pub fn find<'a>(entries: &'a [Entry], name: &str) -> Option<&'a Entry> {
    entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))
}

// ---- §2: statements ----------------------------------------------------

/// A statement with its line number and section.
struct RawStmt {
    line: usize,
    text: String,
    iteration: bool,
}

/// Comments, continuations and the statement split (§9.1 steps 3–5).
fn statements(entry: &Entry) -> Result<Vec<RawStmt>, ParseError> {
    if entry.bad_qualifier {
        return Err(ParseError {
            code: None,
            line: 1,
            message: "Didn't find matching ')' in symmetry declaration".into(),
            statement: String::new(),
        });
    }
    // Logical lines: comments removed, `\`-continuations joined, other
    // backslashes dropped.
    let mut logical: Vec<(usize, String)> = Vec::new();
    let mut pending: Option<(usize, String)> = None;
    for (n, raw) in &entry.lines {
        let code = raw.split(';').next().unwrap_or("");
        let trimmed = code.trim_end_matches([' ', '\t']);
        let (body, continues) = match trimmed.strip_suffix('\\') {
            Some(b) => (b, true),
            None => (code, false),
        };
        let body = body.replace('\\', "");
        let (line, mut acc) = pending.take().unwrap_or((*n, String::new()));
        acc.push_str(&body);
        if continues {
            pending = Some((line, acc));
        } else {
            logical.push((line, acc));
        }
    }
    if let Some(p) = pending {
        logical.push(p);
    }

    let mut out = Vec::new();
    let mut iteration = false;
    let mut colons = 0;
    for (line, text) in logical {
        // Blanks are removed before tokenizing (§1.6).
        let s: String = text.chars().filter(|c| !matches!(c, ' ' | '\t')).collect();
        let bytes = s.as_bytes();
        let mut cur = String::new();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'('
                && let Some(n) = complex_literal_len(&bytes[i..])
            {
                cur.push_str(&s[i..i + n]);
                i += n;
                continue;
            }
            match bytes[i] {
                b',' | b':' => {
                    if !cur.is_empty() {
                        out.push(RawStmt {
                            line,
                            text: std::mem::take(&mut cur),
                            iteration,
                        });
                    }
                    if bytes[i] == b':' {
                        colons += 1;
                        if colons > 1 {
                            return Err(ParseError {
                                code: Some(34),
                                line,
                                message: "Only one \":\" permitted in a formula".into(),
                                statement: s.clone(),
                            });
                        }
                        iteration = true;
                    }
                }
                _ => cur.push(bytes[i] as char),
            }
            i += 1;
        }
        if !cur.is_empty() {
            out.push(RawStmt {
                line,
                text: cur,
                iteration,
            });
        }
    }
    if colons == 0 {
        // Without `:` every statement is an iteration statement (§2.3).
        for s in &mut out {
            s.iteration = true;
        }
    }
    if out.is_empty() {
        return Err(ParseError {
            code: None,
            line: 1,
            message: "Formula has no executable instructions".into(),
            statement: String::new(),
        });
    }
    Ok(out)
}

/// Length of a number literal at the start of `b` (digits, point,
/// exponent with optional sign), if there is one.
fn number_len(b: &[u8]) -> usize {
    let mut i = 0;
    let digits = |b: &[u8], mut i: usize| {
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        i
    };
    let start = i;
    i = digits(b, i);
    if i < b.len() && b[i] == b'.' {
        i = digits(b, i + 1);
    }
    if i == start || (i == start + 1 && b[start] == b'.') {
        return 0;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        let k = digits(b, j);
        if k > j {
            i = k;
        }
    }
    i
}

/// `(` [-] number `,` [-] number `)` at the start of `b` (§3.3).
fn complex_literal_len(b: &[u8]) -> Option<usize> {
    let mut i = 1;
    for close in *b",)" {
        if b.get(i) == Some(&b'-') {
            i += 1;
        }
        let n = number_len(&b[i.min(b.len())..]);
        if n == 0 {
            return None;
        }
        i += n;
        if b.get(i) != Some(&close) {
            return None;
        }
        i += 1;
    }
    Some(i)
}

/// Where a number literal has a signed exponent (`1e+2`), the position of
/// its `e`. Fractint reads such a literal as the whole number followed by
/// the sign and digits again (§3.2).
fn signed_exponent(text: &str) -> Option<usize> {
    let p = text.find(['e', 'E'])?;
    matches!(text.as_bytes().get(p + 1), Some(b'+' | b'-')).then_some(p)
}

fn quirk_note(text: &str, value: f64, quirks: &mut Vec<String>) {
    quirks.push(format!(
        "'{text}' is read as {value}, as Fractint reads it (the signed exponent counts twice)"
    ));
}

/// The value of a number inside a complex literal, with the quirk applied
/// (`(1e+1,0)` has real part 11, §3.2).
fn literal_value(text: &str, quirks: &mut Vec<String>) -> f64 {
    let v: f64 = text.parse().unwrap_or(f64::NAN);
    let Some(p) = signed_exponent(text) else {
        return v;
    };
    let digits: f64 = text[p + 2..].parse().unwrap_or(0.0);
    let value = if text.as_bytes()[p + 1] == b'+' {
        v + digits
    } else {
        v - digits
    };
    quirk_note(text, value, quirks);
    value
}

// ---- §3: tokens --------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Cplx(Complex),
    Ident(String),
    Op(&'static str),
    LParen,
    RParen,
    Bar,
}

fn err(code: u8, message: &str) -> (u8, String) {
    (code, message.to_owned())
}

/// Tokenizes one statement (blanks already removed).
fn tokenize(s: &str, quirks: &mut Vec<String>) -> Result<Vec<Tok>, (u8, String)> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'(' {
            if let Some(n) = complex_literal_len(&b[i..]) {
                let inner = &s[i + 1..i + n - 1];
                let (re, im) = inner.split_once(',').unwrap();
                let part = |t: &str, q: &mut Vec<String>| match t.strip_prefix('-') {
                    Some(r) => -literal_value(r, q),
                    None => literal_value(t, q),
                };
                out.push(Tok::Cplx(Complex::new(part(re, quirks), part(im, quirks))));
                i += n;
                continue;
            }
            out.push(Tok::LParen);
            i += 1;
            continue;
        }
        if c.is_ascii_digit() || (c == b'.' && b.get(i + 1).is_some_and(u8::is_ascii_digit)) {
            let n = number_len(&b[i..]);
            if n == 0 {
                return Err(err(27, "Invalid constant expression"));
            }
            let text = &s[i..i + n];
            // A number can't run into letters, digits or another point.
            if b.get(i + n)
                .is_some_and(|x| x.is_ascii_alphanumeric() || *x == b'.' || *x == b'_')
            {
                return Err(err(27, "Invalid constant expression"));
            }
            if n > MAX_TOKEN {
                return Err(err(33, "Variable or constant exceeds 32 character limit"));
            }
            // With a signed exponent the token has the whole number's value
            // but ends before the sign, which is read again (§3.2).
            let value: f64 = text.parse().unwrap_or(f64::NAN);
            out.push(Tok::Num(value));
            match signed_exponent(text) {
                Some(p) => {
                    let tail: f64 = text[p + 2..].parse().unwrap_or(0.0);
                    let sign = if b[i + p + 1] == b'+' { 1.0 } else { -1.0 };
                    quirk_note(text, value + sign * tail, quirks);
                    i += p + 1;
                }
                None => i += n,
            }
            continue;
        }
        if c == b'.' {
            return Err(err(27, "Invalid constant expression"));
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let n = b[i..]
                .iter()
                .take_while(|x| x.is_ascii_alphanumeric() || **x == b'_')
                .count();
            if b.get(i + n) == Some(&b'.') {
                return Err(err(26, "Illegal variable name"));
            }
            if n > MAX_TOKEN {
                return Err(err(33, "Variable or constant exceeds 32 character limit"));
            }
            out.push(Tok::Ident(s[i..i + n].to_ascii_lowercase()));
            i += n;
            continue;
        }
        let two = &s[i..(i + 2).min(s.len())];
        let op2 = ["<=", ">=", "==", "!=", "&&", "||"]
            .into_iter()
            .find(|o| *o == two);
        if let Some(o) = op2 {
            out.push(Tok::Op(o));
            i += 2;
            continue;
        }
        let op1 = match c {
            b'+' => "+",
            b'-' => "-",
            b'*' => "*",
            b'/' => "/",
            b'^' => "^",
            b'<' => "<",
            b'>' => ">",
            b'=' => "=",
            b')' => {
                out.push(Tok::RParen);
                i += 1;
                continue;
            }
            b'|' => {
                out.push(Tok::Bar);
                i += 1;
                continue;
            }
            b'!' | b'&' => return Err(err(4, "Undefined Operator")),
            _ => return Err(err(28, "This character not supported by parser")),
        };
        out.push(Tok::Op(op1));
        i += 1;
    }
    Ok(out)
}

// ---- §4: expressions ---------------------------------------------------

struct P<'a> {
    t: &'a [Tok],
    i: usize,
    quirks: &'a mut Vec<String>,
    /// Literal tokens that a `-(` quirk turns into 0 (§4).
    zeroed: Vec<usize>,
}

type PResult<T> = Result<T, (u8, String)>;

impl P<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }

    fn eat_op(&mut self, ops: &[&str]) -> Option<&'static str> {
        if let Some(Tok::Op(o)) = self.peek()
            && ops.contains(o)
        {
            let o = *o;
            self.i += 1;
            return Some(o);
        }
        None
    }

    fn expression(&mut self) -> PResult<Expr> {
        let mut e = self.comparison()?;
        while let Some(o) = self.eat_op(&["&&", "||"]) {
            let r = self.comparison()?;
            e = bin(if o == "&&" { BinOp::And } else { BinOp::Or }, e, r);
        }
        Ok(e)
    }

    fn comparison(&mut self) -> PResult<Expr> {
        let mut e = self.sum()?;
        while let Some(o) = self.eat_op(&["<", "<=", ">", ">=", "==", "!="]) {
            let r = self.sum()?;
            let op = match o {
                "<" => BinOp::Lt,
                "<=" => BinOp::Le,
                ">" => BinOp::Gt,
                ">=" => BinOp::Ge,
                "==" => BinOp::Eq,
                _ => BinOp::Ne,
            };
            e = bin(op, e, r);
        }
        Ok(e)
    }

    fn sum(&mut self) -> PResult<Expr> {
        let mut e = self.product()?;
        while let Some(o) = self.eat_op(&["+", "-"]) {
            let r = self.product()?;
            e = bin(if o == "+" { BinOp::Add } else { BinOp::Sub }, e, r);
        }
        Ok(e)
    }

    fn product(&mut self) -> PResult<Expr> {
        let mut e = self.power()?;
        while let Some(o) = self.eat_op(&["*", "/"]) {
            let r = self.power()?;
            e = bin(if o == "*" { BinOp::Mul } else { BinOp::Div }, e, r);
        }
        Ok(e)
    }

    fn power(&mut self) -> PResult<Expr> {
        let mut e = self.unary()?;
        while self.eat_op(&["^"]).is_some() {
            if matches!(self.peek(), Some(Tok::Op("-"))) {
                return Err(err(32, "Negative exponent must be enclosed in parens"));
            }
            let r = self.primary()?;
            e = bin(BinOp::Pow, e, r);
        }
        Ok(e)
    }

    fn unary(&mut self) -> PResult<Expr> {
        if self.eat_op(&["-"]).is_none() {
            return self.primary();
        }
        // The `-(` literal quirk (§4): the minus is lost and the first
        // literal inside the parentheses counts as 0. A complex literal
        // carries its own parentheses.
        if matches!(self.peek(), Some(Tok::Cplx(_))) {
            self.zeroed.push(self.i);
            self.quirks.push(
                "'-' before a complex constant is read as Fractint reads it: the value counts as 0"
                    .into(),
            );
            return self.primary();
        }
        if matches!(self.peek(), Some(Tok::LParen)) {
            let mut j = self.i;
            while matches!(self.t.get(j), Some(Tok::LParen)) {
                j += 1;
            }
            if matches!(self.t.get(j), Some(Tok::Num(_) | Tok::Cplx(_))) {
                self.zeroed.push(j);
                self.quirks.push(
                    "'-(' followed by a number is read as Fractint reads it: the minus is lost and that number counts as 0"
                        .into(),
                );
                return self.unary_operand();
            }
        }
        Ok(Expr::Neg(Box::new(self.unary_operand()?)))
    }

    /// A unary operand: further minus signs or a primary.
    fn unary_operand(&mut self) -> PResult<Expr> {
        if matches!(self.peek(), Some(Tok::Op("-"))) {
            self.unary()
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> PResult<Expr> {
        let idx = self.i;
        let Some(tok) = self.peek().cloned() else {
            return Err(err(0, "Should be an Argument"));
        };
        self.i += 1;
        let zero = self.zeroed.contains(&idx);
        match tok {
            Tok::Num(v) => Ok(Expr::Const(Complex::new(if zero { 0.0 } else { v }, 0.0))),
            Tok::Cplx(c) => Ok(Expr::Const(if zero { Complex::ZERO } else { c })),
            Tok::LParen => {
                match self.peek() {
                    None => return Err(err(3, "Need more ')'")),
                    Some(Tok::RParen) => return Err(err(0, "Should be an Argument")),
                    _ => {}
                }
                let e = self.expression()?;
                match self.peek() {
                    Some(Tok::RParen) => {
                        self.i += 1;
                        Ok(e)
                    }
                    None => Err(err(3, "Need more ')'")),
                    // Assignments only at statement level (§2.5).
                    Some(Tok::Op("=")) => {
                        Err(err(25, "Only variables are allowed to left of assignment"))
                    }
                    Some(_) => Err(err(1, "Should be an Operator")),
                }
            }
            Tok::Bar => {
                let e = self.expression()?;
                match self.peek() {
                    Some(Tok::Bar) => {
                        self.i += 1;
                        Ok(Expr::Modulus(Box::new(e)))
                    }
                    None => Err(err(
                        30,
                        "Unmatched modulus operator \"|\" in this expression",
                    )),
                    Some(_) => Err(err(0, "Should be an Argument")),
                }
            }
            Tok::Ident(name) => {
                match name.as_str() {
                    "else" | "endif" => {
                        return Err(err(15, "No characters may follow this jump instruction"));
                    }
                    "if" | "elseif" => {
                        return Err(err(
                            16,
                            "Jump instruction missing required (boolean argument)",
                        ));
                    }
                    _ => {}
                }
                if is_function(&name) {
                    let arg = match self.peek().cloned() {
                        Some(Tok::Cplx(c)) => {
                            self.i += 1;
                            Expr::Const(c)
                        }
                        Some(Tok::LParen) => {
                            self.i += 1;
                            if matches!(self.peek(), Some(Tok::RParen)) {
                                return Err(err(0, "Should be an Argument"));
                            }
                            let e = self.expression()?;
                            match self.peek() {
                                Some(Tok::RParen) => self.i += 1,
                                None => return Err(err(3, "Need more ')'")),
                                Some(_) => return Err(err(1, "Should be an Operator")),
                            }
                            e
                        }
                        _ => return Err(err(31, "Can't use function name as variable")),
                    };
                    return Ok(Expr::Call(name, Box::new(arg)));
                }
                if matches!(self.peek(), Some(Tok::LParen | Tok::Cplx(_))) {
                    return Err(err(5, "Undefined Function"));
                }
                Ok(Expr::Var(name))
            }
            Tok::RParen => Err(err(2, "')' needs a matching '('")),
            Tok::Op(_) => Err(err(0, "Should be an Argument")),
        }
    }

    /// Checks that nothing is left after a complete expression.
    fn finish(&self) -> PResult<()> {
        match self.peek() {
            None => Ok(()),
            Some(Tok::RParen) => Err(err(2, "')' needs a matching '('")),
            Some(Tok::Op("=")) => Err(err(25, "Only variables are allowed to left of assignment")),
            Some(_) => Err(err(1, "Should be an Operator")),
        }
    }
}

fn bin(op: BinOp, a: Expr, b: Expr) -> Expr {
    Expr::Binary(op, Box::new(a), Box::new(b))
}

/// One parsed statement, before `if` blocks are assembled.
enum Item {
    Stmt(Expr),
    If(Expr),
    ElseIf(Expr),
    Else,
    EndIf,
}

/// Parses one statement. Keyword statements still count structurally when
/// they have an error, as in Fractint (one error per statement, §7.1).
fn parse_statement(text: &str, quirks: &mut Vec<String>) -> (Option<Item>, Option<(u8, String)>) {
    let toks = match tokenize(text, quirks) {
        Ok(t) => t,
        Err(e) => {
            // Still recognise a keyword so blocks stay balanced.
            let lower = text.to_ascii_lowercase();
            let item = if lower.starts_with("elseif(") {
                Some(Item::ElseIf(Expr::Const(Complex::ZERO)))
            } else if lower.starts_with("if(") {
                Some(Item::If(Expr::Const(Complex::ZERO)))
            } else {
                None
            };
            return (item, Some(e));
        }
    };
    // Keywords (§5). An assignment to a keyword (`endif = 5`) is only an
    // error; it doesn't open or close a block.
    if let Some(Tok::Ident(k)) = toks.first()
        && KEYWORDS.contains(&k.as_str())
        && matches!(toks.get(1), Some(Tok::Op("=")))
    {
        let e = match k.as_str() {
            "else" | "endif" => err(15, "No characters may follow this jump instruction"),
            _ => err(16, "Jump instruction missing required (boolean argument)"),
        };
        return (None, Some(e));
    }
    if let Some(Tok::Ident(k)) = toks.first()
        && KEYWORDS.contains(&k.as_str())
    {
        let trailing =
            (toks.len() > 1).then(|| err(15, "No characters may follow this jump instruction"));
        return match k.as_str() {
            "else" => (Some(Item::Else), trailing),
            "endif" => (Some(Item::EndIf), trailing),
            _ => {
                let is_if = k == "if";
                let make = |c: Expr| if is_if { Item::If(c) } else { Item::ElseIf(c) };
                if !matches!(toks.get(1), Some(Tok::LParen)) {
                    let e = err(16, "Jump instruction missing required (boolean argument)");
                    return (Some(make(Expr::Const(Complex::ZERO))), Some(e));
                }
                let mut p = P {
                    t: &toks,
                    i: 1,
                    quirks,
                    zeroed: Vec::new(),
                };
                match p.primary().and_then(|c| p.finish().map(|()| c)) {
                    Ok(c) => (Some(make(c)), None),
                    Err(e) => (Some(make(Expr::Const(Complex::ZERO))), Some(e)),
                }
            }
        };
    }
    match parse_plain(&toks, quirks) {
        Ok(e) => (Some(Item::Stmt(e)), None),
        Err(e) => (None, Some(e)),
    }
}

/// An expression statement, with statement-level assignment chains
/// (`a = b = expr`, §2.5).
fn parse_plain(toks: &[Tok], quirks: &mut Vec<String>) -> PResult<Expr> {
    let mut targets = Vec::new();
    let mut i = 0;
    while let (Some(Tok::Ident(n)), Some(Tok::Op("="))) = (toks.get(i), toks.get(i + 1)) {
        if is_function(n) {
            return Err(err(31, "Can't use function name as variable"));
        }
        targets.push(n.clone());
        i += 2;
    }
    if i == 0 && matches!(toks.get(1), Some(Tok::Op("="))) {
        return Err(err(25, "Only variables are allowed to left of assignment"));
    }
    let mut p = P {
        t: toks,
        i,
        quirks,
        zeroed: Vec::new(),
    };
    let mut e = p.expression()?;
    p.finish()?;
    for n in targets.into_iter().rev() {
        e = Expr::Assign(n, Box::new(e));
    }
    Ok(e)
}

/// Assembles `if` blocks (§5) from one section's items.
fn blocks(
    items: Vec<(usize, Item)>,
    init: bool,
    last_line: usize,
    errors: &mut Vec<ParseError>,
) -> Vec<Stmt> {
    // Stack of open blocks: (arms so far, current condition, current body,
    // in else, line).
    struct Open {
        arms: Vec<(Expr, Vec<Stmt>)>,
        cond: Option<Expr>,
        body: Vec<Stmt>,
        in_else: bool,
    }
    let mut top: Vec<Stmt> = Vec::new();
    let mut stack: Vec<Open> = Vec::new();
    let push = |stack: &mut Vec<Open>, top: &mut Vec<Stmt>, s: Stmt| match stack.last_mut() {
        Some(o) => o.body.push(s),
        None => top.push(s),
    };
    let error = |errors: &mut Vec<ParseError>, line: usize, code: u8, m: &str| {
        errors.push(ParseError {
            code: Some(code),
            line,
            message: m.into(),
            statement: String::new(),
        });
    };
    for (line, item) in items {
        match item {
            Item::Stmt(e) => push(&mut stack, &mut top, Stmt::Expr(e)),
            Item::If(c) => stack.push(Open {
                arms: vec![],
                cond: Some(c),
                body: vec![],
                in_else: false,
            }),
            Item::ElseIf(_) | Item::Else if stack.is_empty() => {
                error(errors, line, 19, "Misplaced \"else\" or \"elseif()\"");
            }
            Item::ElseIf(c) => {
                let o = stack.last_mut().unwrap();
                if o.in_else {
                    error(
                        errors,
                        line,
                        17,
                        "Next jump after \"else\" must be \"endif\"",
                    );
                    continue;
                }
                let cond = o.cond.take().unwrap();
                o.arms.push((cond, std::mem::take(&mut o.body)));
                o.cond = Some(c);
            }
            Item::Else => {
                let o = stack.last_mut().unwrap();
                if o.in_else {
                    error(
                        errors,
                        line,
                        17,
                        "Next jump after \"else\" must be \"endif\"",
                    );
                    continue;
                }
                let cond = o.cond.take().unwrap();
                o.arms.push((cond, std::mem::take(&mut o.body)));
                o.in_else = true;
            }
            Item::EndIf => {
                let Some(mut o) = stack.pop() else {
                    error(errors, line, 18, "\"endif\" has no matching \"if\"");
                    continue;
                };
                let otherwise = if o.in_else {
                    o.body
                } else {
                    o.arms.push((o.cond.take().unwrap(), o.body));
                    Vec::new()
                };
                push(
                    &mut stack,
                    &mut top,
                    Stmt::If {
                        arms: o.arms,
                        otherwise,
                    },
                );
            }
        }
    }
    // Reported at the section's last statement (§5).
    if !stack.is_empty() {
        if init {
            error(
                errors,
                last_line,
                20,
                "\"if()\" in initialization has no matching \"endif\"",
            );
        } else {
            error(errors, last_line, 21, "\"if()\" has no matching \"endif\"");
        }
    }
    top
}

/// Parses an entry (§9).
pub fn parse(entry: &Entry) -> Result<Parsed, Vec<ParseError>> {
    let raw = statements(entry).map_err(|e| vec![e])?;
    let mut quirks = Vec::new();
    let mut errors = Vec::new();
    let mut init_items = Vec::new();
    let mut iter_items = Vec::new();
    let mut jumps = 0;
    for s in &raw {
        let (item, error) = parse_statement(&s.text, &mut quirks);
        if let Some(item) = item {
            if !matches!(item, Item::Stmt(_)) {
                jumps += 1;
            }
            if s.iteration {
                iter_items.push((s.line, item));
            } else {
                init_items.push((s.line, item));
            }
        }
        if let Some((code, message)) = error {
            errors.push(ParseError {
                code: Some(code),
                line: s.line,
                message,
                statement: s.text.clone(),
            });
        }
    }
    let last_line = |iteration: bool| {
        raw.iter()
            .rev()
            .find(|s| s.iteration == iteration)
            .map(|s| s.line)
            .unwrap_or(1)
    };
    if jumps > MAX_JUMPS {
        let line = raw.last().map_or(1, |s| s.line);
        errors.push(ParseError {
            code: Some(23),
            line,
            message: "Formula has too many jump commands".into(),
            statement: String::new(),
        });
    }
    let init = blocks(init_items, true, last_line(false), &mut errors);
    let iter = blocks(iter_items, false, last_line(true), &mut errors);
    if !errors.is_empty() {
        errors.sort_by_key(|e| e.line);
        errors.truncate(MAX_ERRORS);
        return Err(errors);
    }
    quirks.sort();
    quirks.dedup();
    Ok(Parsed {
        ast: FormulaAst {
            name: entry.name.clone(),
            symmetry: entry.qualifier.clone(),
            init,
            iter,
        },
        warnings: quirks,
    })
}
