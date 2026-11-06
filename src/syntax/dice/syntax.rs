//! The syntax tree of a Dice program and its parser ([`Parser`]), a
//! recursive descent over the tokens with Dice's operator precedence (its
//! `lib/Parser.mly`). A program is its function definitions and its body,
//! each an expression, where a chain of `let`s and a chain of `else if`s
//! are one node each; the translation ([`super::translate`]) evaluates the
//! body and inlines the functions at each call.

use super::Res;
use super::tokens::{Tk, Token, misplaced};
use crate::syntax::fail;

pub(super) struct Expr<'a> {
    pub(super) line: usize,
    pub(super) kind: Kind<'a>,
}

pub(super) enum Kind<'a> {
    /// A chain of `let`s and the body they bind for.
    Let(Vec<(&'a str, Expr<'a>)>, Box<Expr<'a>>),
    /// A chain `if c0 then e0 else if c1 then e1 ... else en`: the arms
    /// and the last `else`.
    If(Vec<(Expr<'a>, Expr<'a>)>, Box<Expr<'a>>),
    Observe(Box<Expr<'a>>),
    Not(Box<Expr<'a>>),
    Bin(&'a str, Box<Expr<'a>>, Box<Expr<'a>>),
    /// `<<` or `>>` by a constant.
    Shift(&'a str, Box<Expr<'a>>, u64),
    Flip(f64),
    Discrete(Vec<f64>),
    /// `int(width, value)`.
    Int(u64, u64),
    /// `uniform(width, lo, hi)`.
    Uniform(u64, u64, u64),
    Iterate(&'a str, Box<Expr<'a>>, Count<'a>),
    Fst(Box<Expr<'a>>),
    Snd(Box<Expr<'a>>),
    /// `binomial(width, n, p)`.
    Binomial(u64, u64, f64),
    /// `nth_bit(int(width, i), x)`.
    NthBit(u64, u64, Box<Expr<'a>>),
    /// `[x1, ..., xn]`.
    List(Vec<Expr<'a>>),
    /// `[] : t`.
    Nil(Type),
    /// `head`, `tail` or `length` of a list.
    Head(Box<Expr<'a>>),
    Tail(Box<Expr<'a>>),
    Length(Box<Expr<'a>>),
    Bool(bool),
    Call(&'a str, Vec<Expr<'a>>),
    Var(&'a str),
    Tuple(Box<Expr<'a>>, Box<Expr<'a>>),
}

/// The count of an `iterate`: a number, or a place `$N` that no argument
/// filled, written as it stands in the program.
#[derive(Clone, Copy)]
pub(super) enum Count<'a> {
    Number(u64),
    Place(&'a str),
}

impl std::fmt::Display for Count<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Count::Number(k) => write!(f, "{k}"),
            Count::Place(p) => f.write_str(p),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Type {
    Bool,
    Int(u64),
    Tuple(Box<Type>, Box<Type>),
    List(Box<Type>),
}

pub(super) struct Fun<'a> {
    pub(super) name: &'a str,
    pub(super) params: Vec<&'a str>,
    pub(super) ret: Option<Type>,
    pub(super) body: Expr<'a>,
}

pub(super) struct Program<'a> {
    pub(super) funs: Vec<Fun<'a>>,
    pub(super) body: Expr<'a>,
}

/// Dice's binary operators and their precedence, low to high (its
/// `lib/Parser.mly`); all but `::` associate to the left. `!` binds
/// between `&&` and `<=>` ([`NOT_LEVEL`]).
fn precedence(op: &str) -> Option<u8> {
    Some(match op {
        "||" => 1,
        "&&" => 2,
        "<=>" => 4,
        "^" => 5,
        "<" | "<=" | ">" | ">=" | "!=" => 6,
        "::" => 7,
        "+" | "-" | "==" | "<<" | ">>" => 8,
        "*" | "/" | "%" => 9,
        _ => return None,
    })
}

const NOT_LEVEL: u8 = 3;

/// Names that Dice reads and this reader refuses.
const UNSUPPORTED: [&str; 1] = ["sample"];

/// Recursive descent for Dice's expression language:
///
/// ```text
/// expr := 'let' id '=' expr 'in' expr | 'if' expr 'then' expr 'else' expr
///       | 'observe' expr | '!' expr | expr binop expr | atom
/// atom := 'flip' num | 'flip' '(' num ')' | 'discrete' '(' nums ')'
///       | 'int' '(' num ',' num ')' | 'uniform' '(' num ',' num ',' num ')'
///       | 'iterate' '(' id ',' expr ',' (num | place) ')'
///       | 'binomial' '(' num ',' num ',' num ')'
///       | 'nth_bit' '(' 'int' '(' num ',' num ')' ',' expr ')'
///       | ('fst' | 'snd' | 'head' | 'tail' | 'length') expr
///       | 'true' | 'false' | id ['(' args ')']
///       | '(' expr [',' expr] ')' | '[' expr {',' expr} ']' | '[' ']' ':' type
/// ```
///
/// `let`, `if`, `observe`, `fst`, `snd`, `head`, `tail` and `length`
/// extend as far right as they can, as Dice's parser resolves those
/// conflicts by shifting: `fst s == x` is `fst (s == x)` and `snd s << 1`
/// is `snd (s << 1)`, each a type error, as Dice reads them, and
/// `tail xs :: ys` is `tail (xs :: ys)`. A chain of
/// `let`s and a chain of `else if`s are read in a loop and held as one
/// node: programs have thousands.
pub(super) struct Parser<'a> {
    pub(super) toks: Vec<Token<'a>>,
    pub(super) i: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Token<'a> {
        self.toks.get(self.i).copied().unwrap_or(Token {
            kind: Tk::Eof,
            text: "the end of the program",
            line: self.toks.last().map_or(1, |t| t.line),
        })
    }

    fn at(&self, kind: Tk, text: &str) -> bool {
        let t = self.peek();
        t.kind == kind && t.text == text
    }

    fn take(&mut self) -> Token<'a> {
        let t = self.peek();
        self.i += 1;
        t
    }

    fn expect(&mut self, kind: Tk, text: Option<&str>) -> Res<Token<'a>> {
        let t = self.peek();
        if t.kind == Tk::Place {
            return Err(misplaced(t));
        }
        if t.kind != kind || text.is_some_and(|s| s != t.text) {
            let want = match (text, kind) {
                (Some(s), _) => format!("`{s}`"),
                (None, Tk::Id) => "a name".into(),
                (None, _) => "a number".into(),
            };
            return Err(fail(t.line, format!("expected {want}, found `{}`", t.text)));
        }
        Ok(self.take())
    }

    fn op(&mut self, text: &str) -> Res<Token<'a>> {
        self.expect(Tk::Op, Some(text))
    }

    fn float(&mut self) -> Res<f64> {
        let t = self.expect(Tk::Num, None)?;
        t.text
            .parse()
            .map_err(|_| fail(t.line, format!("`{}` is not a number", t.text)))
    }

    fn integer(&mut self) -> Res<u64> {
        let t = self.expect(Tk::Num, None)?;
        if !t.text.bytes().all(|c| c.is_ascii_digit()) {
            return Err(fail(
                t.line,
                format!("expected an integer, found `{}`", t.text),
            ));
        }
        t.text
            .parse()
            .map_err(|_| fail(t.line, format!("the integer {} is too large", t.text)))
    }

    pub(super) fn program(mut self) -> Res<Program<'a>> {
        let mut funs = Vec::new();
        while self.at(Tk::Id, "fun") {
            funs.push(self.fun()?);
        }
        let body = self.expr(0)?;
        let t = self.peek();
        if t.kind != Tk::Eof {
            return Err(fail(
                t.line,
                format!("unexpected `{}` after the program", t.text),
            ));
        }
        Ok(Program { funs, body })
    }

    fn fun(&mut self) -> Res<Fun<'a>> {
        self.take();
        let name = self.expect(Tk::Id, None)?.text;
        self.op("(")?;
        let mut params = Vec::new();
        while !self.at(Tk::Op, ")") {
            params.push(self.expect(Tk::Id, None)?.text);
            if self.at(Tk::Op, ":") {
                // A type annotation, not checked.
                self.take();
                self.ty()?;
            }
            if self.at(Tk::Op, ",") {
                self.take();
            }
        }
        self.op(")")?;
        let mut ret = None;
        if self.at(Tk::Op, ":") {
            self.take();
            ret = Some(self.ty()?);
        }
        self.op("{")?;
        let body = self.expr(0)?;
        self.op("}")?;
        Ok(Fun {
            name,
            params,
            ret,
            body,
        })
    }

    /// `bool`, `int(w)`, `(t, t)` or `list(t)`.
    fn ty(&mut self) -> Res<Type> {
        let t = self.take();
        match (t.kind, t.text) {
            (Tk::Id, "bool") => Ok(Type::Bool),
            (Tk::Id, "int") => {
                self.op("(")?;
                let w = self.integer()?;
                self.op(")")?;
                Ok(Type::Int(w))
            }
            (Tk::Id, "list") => {
                self.op("(")?;
                let inner = self.ty()?;
                self.op(")")?;
                Ok(Type::List(Box::new(inner)))
            }
            (Tk::Op, "(") => {
                let a = self.ty()?;
                self.op(",")?;
                let b = self.ty()?;
                self.op(")")?;
                Ok(Type::Tuple(Box::new(a), Box::new(b)))
            }
            _ => Err(fail(t.line, format!("unexpected `{}` in a type", t.text))),
        }
    }

    pub(super) fn expr(&mut self, level: u8) -> Res<Expr<'a>> {
        let mut e = self.prefix()?;
        loop {
            let t = self.peek();
            let prec = if t.kind == Tk::Op {
                precedence(t.text)
            } else {
                None
            };
            let Some(prec) = prec.filter(|&p| p >= level) else {
                return Ok(e);
            };
            self.take();
            let kind = if matches!(t.text, "<<" | ">>") {
                Kind::Shift(t.text, Box::new(e), self.integer()?)
            } else {
                // `::` associates to the right.
                let next = if t.text == "::" { prec } else { prec + 1 };
                Kind::Bin(t.text, Box::new(e), Box::new(self.expr(next)?))
            };
            e = Expr { line: t.line, kind };
        }
    }

    fn prefix(&mut self) -> Res<Expr<'a>> {
        let t = self.peek();
        let line = t.line;
        if t.kind != Tk::Id && !(t.kind == Tk::Op && t.text == "!") {
            return self.atom();
        }
        match t.text {
            "let" => {
                let mut binds = Vec::new();
                while self.at(Tk::Id, "let") {
                    self.take();
                    let name = self.expect(Tk::Id, None)?.text;
                    self.op("=")?;
                    let value = self.expr(0)?;
                    self.expect(Tk::Id, Some("in"))?;
                    binds.push((name, value));
                }
                let body = self.expr(0)?;
                Ok(let_chain(line, binds, body))
            }
            "if" => {
                let mut arms = Vec::new();
                loop {
                    self.take();
                    let c = self.expr(0)?;
                    self.expect(Tk::Id, Some("then"))?;
                    let a = self.expr(0)?;
                    self.expect(Tk::Id, Some("else"))?;
                    arms.push((c, a));
                    if !self.at(Tk::Id, "if") {
                        break;
                    }
                }
                let otherwise = self.expr(0)?;
                Ok(if_chain(line, arms, otherwise))
            }
            "observe" => {
                self.take();
                let e = self.expr(0)?;
                Ok(Expr {
                    line,
                    kind: Kind::Observe(Box::new(e)),
                })
            }
            "!" => {
                self.take();
                let e = self.expr(NOT_LEVEL + 1)?;
                Ok(Expr {
                    line,
                    kind: Kind::Not(Box::new(e)),
                })
            }
            _ => self.atom(),
        }
    }

    fn atom(&mut self) -> Res<Expr<'a>> {
        let t = self.take();
        let line = t.line;
        let kind = match (t.kind, t.text) {
            (Tk::Id, name) => self.id_atom(name, line)?,
            (Tk::Op, "(") => {
                let e = self.expr(0)?;
                if self.at(Tk::Op, ",") {
                    self.take();
                    let f = self.expr(0)?;
                    self.op(")")?;
                    Kind::Tuple(Box::new(e), Box::new(f))
                } else {
                    self.op(")")?;
                    return Ok(e);
                }
            }
            (Tk::Op, "[") if self.at(Tk::Op, "]") => {
                self.take();
                self.op(":")?;
                Kind::Nil(self.ty()?)
            }
            (Tk::Op, "[") => {
                let mut items = vec![self.expr(0)?];
                while self.at(Tk::Op, ",") {
                    self.take();
                    items.push(self.expr(0)?);
                }
                self.op("]")?;
                Kind::List(items)
            }
            (Tk::Place, _) => return Err(misplaced(t)),
            _ => return Err(fail(line, format!("unexpected `{}`", t.text))),
        };
        Ok(Expr { line, kind })
    }

    /// An `iterate`'s count: a number or a place without an argument.
    fn count(&mut self) -> Res<Count<'a>> {
        if self.peek().kind == Tk::Place {
            return Ok(Count::Place(self.take().text));
        }
        Ok(Count::Number(self.integer()?))
    }

    fn int_pair(&mut self) -> Res<(u64, u64)> {
        self.op("(")?;
        let w = self.integer()?;
        self.op(",")?;
        let k = self.integer()?;
        self.op(")")?;
        Ok((w, k))
    }

    fn id_atom(&mut self, name: &'a str, line: usize) -> Res<Kind<'a>> {
        let kind = match name {
            "flip" => {
                if self.at(Tk::Op, "(") {
                    self.take();
                    let p = self.float()?;
                    self.op(")")?;
                    Kind::Flip(p)
                } else {
                    Kind::Flip(self.float()?)
                }
            }
            "discrete" => {
                self.op("(")?;
                let mut ps = vec![self.float()?];
                while self.at(Tk::Op, ",") {
                    self.take();
                    ps.push(self.float()?);
                }
                self.op(")")?;
                Kind::Discrete(ps)
            }
            "int" => {
                let (w, k) = self.int_pair()?;
                Kind::Int(w, k)
            }
            "uniform" => {
                self.op("(")?;
                let w = self.integer()?;
                self.op(",")?;
                let lo = self.integer()?;
                self.op(",")?;
                let hi = self.integer()?;
                self.op(")")?;
                Kind::Uniform(w, lo, hi)
            }
            "iterate" => {
                self.op("(")?;
                let f = self.expect(Tk::Id, None)?.text;
                self.op(",")?;
                let init = self.expr(0)?;
                self.op(",")?;
                let k = self.count()?;
                self.op(")")?;
                Kind::Iterate(f, Box::new(init), k)
            }
            "binomial" => {
                self.op("(")?;
                let w = self.integer()?;
                self.op(",")?;
                let n = self.integer()?;
                self.op(",")?;
                let p = self.float()?;
                self.op(")")?;
                Kind::Binomial(w, n, p)
            }
            "nth_bit" => {
                self.op("(")?;
                self.expect(Tk::Id, Some("int"))?;
                let (w, i) = self.int_pair()?;
                self.op(",")?;
                let x = self.expr(0)?;
                self.op(")")?;
                Kind::NthBit(w, i, Box::new(x))
            }
            op @ ("fst" | "snd" | "head" | "tail" | "length") => {
                let x = Box::new(self.expr(0)?);
                match op {
                    "fst" => Kind::Fst(x),
                    "snd" => Kind::Snd(x),
                    "head" => Kind::Head(x),
                    "tail" => Kind::Tail(x),
                    _ => Kind::Length(x),
                }
            }
            "true" => Kind::Bool(true),
            "false" => Kind::Bool(false),
            name if UNSUPPORTED.contains(&name) => {
                return Err(fail(line, format!("Dice's `{name}` is not supported")));
            }
            name => {
                if self.at(Tk::Op, "(") {
                    self.take();
                    let mut args = Vec::new();
                    while !self.at(Tk::Op, ")") {
                        args.push(self.expr(0)?);
                        if self.at(Tk::Op, ",") {
                            self.take();
                        }
                    }
                    self.op(")")?;
                    Kind::Call(name, args)
                } else {
                    Kind::Var(name)
                }
            }
        };
        Ok(kind)
    }
}

/// `let binds in body`, a `let` chain in `body` joined to `binds`.
fn let_chain<'a>(line: usize, mut binds: Vec<(&'a str, Expr<'a>)>, body: Expr<'a>) -> Expr<'a> {
    let body = match body.kind {
        Kind::Let(more, body) => {
            binds.extend(more);
            body
        }
        kind => Box::new(Expr {
            line: body.line,
            kind,
        }),
    };
    Expr {
        line,
        kind: Kind::Let(binds, body),
    }
}

/// The chain of `arms` then `otherwise`, a chain in `otherwise` (`else
/// (if ...)`) joined to it.
fn if_chain<'a>(line: usize, mut arms: Vec<(Expr<'a>, Expr<'a>)>, otherwise: Expr<'a>) -> Expr<'a> {
    let otherwise = match otherwise.kind {
        Kind::If(more, otherwise) => {
            arms.extend(more);
            otherwise
        }
        kind => Box::new(Expr {
            line: otherwise.line,
            kind,
        }),
    };
    Expr {
        line,
        kind: Kind::If(arms, otherwise),
    }
}

/// Whether `e` calls the function `name`, or loops over it.
pub(super) fn calls(e: &Expr<'_>, name: &str) -> bool {
    let mut stack = vec![e];
    while let Some(e) = stack.pop() {
        match &e.kind {
            Kind::Call(f, args) => {
                if *f == name {
                    return true;
                }
                stack.extend(args);
            }
            Kind::Iterate(f, init, _) => {
                if *f == name {
                    return true;
                }
                stack.push(init);
            }
            Kind::Let(binds, body) => {
                stack.extend(binds.iter().map(|(_, v)| v));
                stack.push(body);
            }
            Kind::If(arms, otherwise) => {
                for (c, a) in arms {
                    stack.push(c);
                    stack.push(a);
                }
                stack.push(otherwise);
            }
            Kind::Observe(x)
            | Kind::Not(x)
            | Kind::Fst(x)
            | Kind::Snd(x)
            | Kind::Head(x)
            | Kind::Tail(x)
            | Kind::Length(x)
            | Kind::NthBit(_, _, x)
            | Kind::Shift(_, x, _) => stack.push(x),
            Kind::Bin(_, l, r) | Kind::Tuple(l, r) => {
                stack.push(l);
                stack.push(r);
            }
            Kind::List(items) => stack.extend(items),
            Kind::Flip(_)
            | Kind::Discrete(_)
            | Kind::Int(..)
            | Kind::Uniform(..)
            | Kind::Binomial(..)
            | Kind::Nil(_)
            | Kind::Bool(_)
            | Kind::Var(_) => {}
        }
    }
    false
}
