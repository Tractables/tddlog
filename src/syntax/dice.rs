//! Dice programs (Holtzen, Van den Broeck and Millstein, OOPSLA 2020),
//! read as the ProbLog program this module writes for them ([`read`]),
//! which `tddlog --program` prints and the parser then reads.
//!
//! The program is evaluated symbolically: every `let` binds a value that
//! is a Boolean literal, an integer, or a tuple of values. An integer of
//! type `int(w)` is `w` bits, as in Dice, each a literal. Every `flip`
//! becomes a fresh probabilistic fact; a `discrete` and a `uniform` become
//! flips bit by bit as Dice draws them (`Translator::choice`); Boolean
//! operators and conditionals become rules over fresh atoms. Integer
//! operations work on the bits: `+` and `-` an adder, `*` shifted
//! additions, `==` a conjunction, `<` a comparator, a conditional a choice
//! of bits. The values an integer can take are tracked where their pairs
//! are few, so that a comparison every pair decides is a constant. An
//! `observe` inside a conditional branch holds only when the branch is
//! taken, so it becomes an evidence atom that is true whenever the path
//! condition is false.
//!
//! A chain `if c0 then e0 else if c1 then e1 ... else en` is merged once:
//! branch i is guarded by one atom, that c_i holds and no earlier
//! condition did, and values equal in several branches are merged under
//! the disjunction of their guards. Within a branch, a variable the guard
//! compares with a constant takes only the values the guard allows (under
//! a disjunction, the values either side allows): a constant where it
//! allows one, and the bits on which the values agree are constants, so
//! that a loop over a guarded-command step carries the values its state
//! reaches rather than every value of its type. Operations on the same
//! operands are made once. Functions are inlined at each call.
//!
//! A loop `iterate(f, init, k)` is written as a ProbLog user writes one,
//! recursion over a step index that the grounder unrolls: f's body once,
//! its atoms indexed by the step `T` (`step(T)`, 0 to k - 1), each flip a
//! probabilistic fact of its step, and the state before step `T` the atoms
//! `s(T)` of the state's Booleans and bits, `s(0)` the initial state's and
//! `s(T + 1)` the body's at step `T`; the body's observations hold at
//! every step, `failed` where one does not, and the evidence is that it is
//! false (`Translator::iterate`). A function that calls itself is
//! inlined up to Dice's recursion limit, 10: a call past it is
//! unreachable, an observation of false on its path, and returns its
//! declared type's default. The program's return value is queried: every
//! value of an integer (the conjunction of its bits), every component of a
//! tuple.
//!
//! A list is held as Dice holds one (its `lib/Passes.ml`): a pair of its
//! length, an integer of 4 bits, and exactly 10 elements as
//! nested pairs, those past the length its type's default value (false,
//! zero). `x :: xs` keeps the first 9 elements of `xs`, its length capped
//! at 10; `head` and `tail` of an empty list observe false on their path.
//! `binomial(w, n, p)` is the sum of `n` flips `p`, each added in a
//! conditional as Dice adds it, and `nth_bit(int(w, i), x)` is bit `i` of
//! `x` counted from the most significant.
//!
//! Supported: `let`, `if`-`then`-`else`, `flip`, `discrete`, `uniform`,
//! `binomial`, `int(w, k)`, `true`, `false`, `&&`, `||`, `!`, `^`, `<=>`,
//! `==`, `!=`, `<`, `<=`, `>`, `>=`, `+`, `-`, `*`, `<<` and `>>` by a
//! constant, `nth_bit`, tuples with `fst` and `snd`, lists with `[...]`,
//! `[] : list(t)`, `::`, `head`, `tail` and `length`, `observe`, `fun`
//! definitions and calls, `iterate`, recursion. Operator precedence is
//! Dice's (its `lib/Parser.mly`). Refused: `/`, `%`, `sample`, integers of
//! more than 63 bits, and a `uniform` over more than 2^24 values.
//!
//! The program is the one the Python script `dice2problog.py` (on the
//! baselines branch) wrote before this reader replaced it, byte for byte: its atoms are named in the same
//! order from the same hints, and its probabilities are written as
//! Python's `repr` writes a float.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use num_rational::BigRational;
use num_traits::ToPrimitive;
use rustc_hash::{FxHashMap, FxHashSet};

use super::ParseError;
use super::lpad::space;
use crate::network::rows::{fsum, write_float};

/// The ProbLog program the Dice program `source` is read as. The error
/// names the line of the token or expression that the program cannot be
/// read at: a syntax error, an unbound variable, a type error, or a
/// construct the reader does not support.
pub fn read(source: &str) -> Result<String, ParseError> {
    let tokens = tokenize(source)?;
    let program = Parser { toks: tokens, i: 0 }.program()?;
    Translator::new(&program).translate(&program.body)
}

/// Dice's recursion limit (its `lib/Passes.ml`): a function that calls
/// itself is inlined this deep, and the call past it observes false.
const RECURSION_LIMIT: usize = 10;

/// The elements a list holds (Dice's `max_list_length`, its recursion
/// limit): a longer literal is an error, and `::` on a full list drops its
/// last element.
const LIST_LENGTH: usize = RECURSION_LIMIT;

/// The most pairs of values whose results an integer operation lists, to
/// know the values the result can take.
const SUPPORT_PAIRS: usize = 1 << 16;

/// The widest integer: its values are `u64`s.
const MAX_WIDTH: u64 = 63;

/// The widest `uniform` drawn bit by bit: its masses are listed.
const MAX_UNIFORM_WIDTH: u64 = 24;

/// The suffix of an atom of a loop's step.
const STEP: &str = "(T)";

fn fail(line: usize, msg: impl Into<String>) -> ParseError {
    ParseError {
        line,
        msg: msg.into(),
    }
}

type Res<T> = Result<T, ParseError>;

// ---------------------------------------------------------------------
// Tokens

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tk {
    Id,
    Num,
    Op,
    Eof,
}

#[derive(Clone, Copy, Debug)]
struct Token<'a> {
    kind: Tk,
    text: &'a str,
    line: usize,
}

/// The operators of more than one character, longest first where one is
/// a prefix of another; then the single characters.
const LONG_OPS: [&str; 11] = [
    "<=>", "==", "!=", "<=", ">=", "<<", ">>", "&&", "||", "::", "->",
];
const SHORT_OPS: &str = "-+*/%<>^(){}[],=!:";

/// The tokens of `text`: identifiers `[A-Za-z_][A-Za-z0-9_']*`, numbers,
/// operators; `//` comments to the end of the line and `(* *)` comments
/// (not nested) are skipped.
fn tokenize(text: &str) -> Res<Vec<Token<'_>>> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let (mut pos, mut line) = (0, 1);
    loop {
        while let Some(c) = text[pos..].chars().next().filter(|&c| space(c)) {
            line += usize::from(c == '\n');
            pos += c.len_utf8();
        }
        if pos == b.len() {
            break;
        }
        let rest = &text[pos..];
        if rest.starts_with("//") {
            pos += rest.find('\n').unwrap_or(rest.len());
            continue;
        }
        if rest.starts_with("(*")
            && let Some(end) = rest[2..].find("*)")
        {
            line += rest[..end + 4].matches('\n').count();
            pos += end + 4;
            continue;
        }
        let digits = |from: usize| b[from..].iter().take_while(|c| c.is_ascii_digit()).count();
        let len = if b[pos].is_ascii_alphabetic() || b[pos] == b'_' {
            1 + b[pos + 1..]
                .iter()
                .take_while(|&&c| c.is_ascii_alphanumeric() || c == b'_' || c == b'\'')
                .count()
        } else if b[pos].is_ascii_digit() || b[pos] == b'.' {
            // `[0-9]*\.[0-9]+` or else `[0-9]+\.?[0-9]*`, then an exponent.
            let lead = digits(pos);
            let mut i = pos + lead;
            if b.get(i) == Some(&b'.') && digits(i + 1) > 0 {
                i += 1 + digits(i + 1);
            } else if lead > 0 {
                if b.get(i) == Some(&b'.') {
                    i += 1 + digits(i + 1);
                }
            } else {
                return Err(cannot(text, pos, line));
            }
            if matches!(b.get(i), Some(b'e' | b'E')) {
                let sign = usize::from(matches!(b.get(i + 1), Some(b'+' | b'-')));
                let d = digits(i + 1 + sign);
                if d > 0 {
                    i += 1 + sign + d;
                }
            }
            i - pos
        } else if let Some(op) = LONG_OPS.iter().find(|op| rest.starts_with(**op)) {
            op.len()
        } else if SHORT_OPS.as_bytes().contains(&b[pos]) {
            1
        } else {
            return Err(cannot(text, pos, line));
        };
        let kind = if b[pos].is_ascii_alphabetic() || b[pos] == b'_' {
            Tk::Id
        } else if b[pos].is_ascii_digit() || b[pos] == b'.' {
            Tk::Num
        } else {
            Tk::Op
        };
        out.push(Token {
            kind,
            text: &text[pos..pos + len],
            line,
        });
        pos += len;
    }
    Ok(out)
}

fn cannot(text: &str, pos: usize, line: usize) -> ParseError {
    let at: String = text[pos..].chars().take(30).collect();
    fail(line, format!("cannot read a Dice token at `{at}`"))
}

// ---------------------------------------------------------------------
// Syntax

struct Expr<'a> {
    line: usize,
    kind: Kind<'a>,
}

enum Kind<'a> {
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
    Iterate(&'a str, Box<Expr<'a>>, u64),
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

#[derive(Clone, Debug, PartialEq)]
enum Type {
    Bool,
    Int(u64),
    Tuple(Box<Type>, Box<Type>),
    List(Box<Type>),
}

struct Fun<'a> {
    name: &'a str,
    params: Vec<&'a str>,
    ret: Option<Type>,
    body: Expr<'a>,
}

struct Program<'a> {
    funs: Vec<Fun<'a>>,
    body: Expr<'a>,
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
///       | 'iterate' '(' id ',' expr ',' num ')'
///       | 'binomial' '(' num ',' num ',' num ')'
///       | 'nth_bit' '(' 'int' '(' num ',' num ')' ',' expr ')'
///       | 'fst' atom | 'snd' atom | ('head' | 'tail' | 'length') expr7
///       | 'true' | 'false' | id ['(' args ')']
///       | '(' expr [',' expr] ')' | '[' expr {',' expr} ']' | '[' ']' ':' type
/// ```
///
/// `let`, `if` and `observe` extend as far right as they can, as Dice's
/// parser resolves those conflicts by shifting. Dice's `fst e` and `snd e`
/// also take the longest `e`, which a well-typed program never uses (no
/// binary operator returns a tuple), so here they take an atom. `head e`,
/// `tail e` and `length e` take the longest `e` too, of which a well-typed
/// program uses `::` alone (`tail xs :: ys` is `tail (xs :: ys)`), so here
/// they take the operators from `::` up (`expr7`). A chain of
/// `let`s and a chain of `else if`s are read in a loop and held as one
/// node: programs have thousands.
struct Parser<'a> {
    toks: Vec<Token<'a>>,
    i: usize,
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

    fn program(mut self) -> Res<Program<'a>> {
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

    fn expr(&mut self, level: u8) -> Res<Expr<'a>> {
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
            (Tk::Id, "flip") => {
                if self.at(Tk::Op, "(") {
                    self.take();
                    let p = self.float()?;
                    self.op(")")?;
                    Kind::Flip(p)
                } else {
                    Kind::Flip(self.float()?)
                }
            }
            (Tk::Id, "discrete") => {
                self.op("(")?;
                let mut ps = vec![self.float()?];
                while self.at(Tk::Op, ",") {
                    self.take();
                    ps.push(self.float()?);
                }
                self.op(")")?;
                Kind::Discrete(ps)
            }
            (Tk::Id, "int") => {
                self.op("(")?;
                let w = self.integer()?;
                self.op(",")?;
                let k = self.integer()?;
                self.op(")")?;
                Kind::Int(w, k)
            }
            (Tk::Id, "uniform") => {
                self.op("(")?;
                let w = self.integer()?;
                self.op(",")?;
                let lo = self.integer()?;
                self.op(",")?;
                let hi = self.integer()?;
                self.op(")")?;
                Kind::Uniform(w, lo, hi)
            }
            (Tk::Id, "iterate") => {
                self.op("(")?;
                let f = self.expect(Tk::Id, None)?.text;
                self.op(",")?;
                let init = self.expr(0)?;
                self.op(",")?;
                let k = self.integer()?;
                self.op(")")?;
                Kind::Iterate(f, Box::new(init), k)
            }
            (Tk::Id, "binomial") => {
                self.op("(")?;
                let w = self.integer()?;
                self.op(",")?;
                let n = self.integer()?;
                self.op(",")?;
                let p = self.float()?;
                self.op(")")?;
                Kind::Binomial(w, n, p)
            }
            (Tk::Id, "nth_bit") => {
                self.op("(")?;
                self.expect(Tk::Id, Some("int"))?;
                self.op("(")?;
                let w = self.integer()?;
                self.op(",")?;
                let i = self.integer()?;
                self.op(")")?;
                self.op(",")?;
                let x = self.expr(0)?;
                self.op(")")?;
                Kind::NthBit(w, i, Box::new(x))
            }
            (Tk::Id, "fst") => Kind::Fst(Box::new(self.atom()?)),
            (Tk::Id, "snd") => Kind::Snd(Box::new(self.atom()?)),
            (Tk::Id, op @ ("head" | "tail" | "length")) => {
                let x = Box::new(self.expr(precedence("::").expect("a level"))?);
                match op {
                    "head" => Kind::Head(x),
                    "tail" => Kind::Tail(x),
                    _ => Kind::Length(x),
                }
            }
            (Tk::Id, "true") => Kind::Bool(true),
            (Tk::Id, "false") => Kind::Bool(false),
            (Tk::Id, name) if UNSUPPORTED.contains(&name) => {
                return Err(fail(line, format!("Dice's `{name}` is not supported")));
            }
            (Tk::Id, name) => {
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
            _ => return Err(fail(line, format!("unexpected `{}`", t.text))),
        };
        Ok(Expr { line, kind })
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
fn calls(e: &Expr<'_>, name: &str) -> bool {
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

// ---------------------------------------------------------------------
// Values

/// A literal: an atom, its negation `\+atom`, `true` or `false`. In a
/// loop's body an atom of the step is `name(T)`, and the state after step
/// k `name(k)`.
type Lit = Rc<str>;

fn lit(s: &str) -> Lit {
    Rc::from(s)
}

fn is(l: &str, constant: &str) -> bool {
    l == constant
}

fn neg(l: &str) -> Lit {
    match l {
        "true" => lit("false"),
        "false" => lit("true"),
        _ => match l.strip_prefix("\\+") {
            Some(a) => lit(a),
            None => lit(&format!("\\+{l}")),
        },
    }
}

/// Whether `a` is the negation of `b`.
fn complementary(a: &str, b: &str) -> bool {
    a.strip_prefix("\\+") == Some(b) || b.strip_prefix("\\+") == Some(a)
}

/// Whether a literal is of the step of a loop's body.
fn at_step(l: &str) -> bool {
    l.ends_with(STEP)
}

/// An integer of `width` bits, modulo 2^width: `bits[k]` is the literal of
/// its bit k, least significant first (`true` or `false` where it is
/// constant). `support` holds every value it can take, sorted (it may hold
/// more), or is `None` where that set was not worked out. Its query atoms
/// are `name(v)`, where it has a name, else a fresh atom from `hint`;
/// `size` values from 0 are queried when it is the program's result. `id`
/// tells integers apart: operations on the same integers are made once.
struct Int {
    id: u64,
    width: u64,
    bits: Vec<Lit>,
    support: Option<Vec<u64>>,
    name: Option<Rc<str>>,
    hint: Option<Rc<str>>,
    size: u64,
}

impl Int {
    fn modulus(&self) -> u64 {
        1 << self.width
    }

    fn allows(&self, v: u64) -> bool {
        v < self.modulus()
            && self
                .support
                .as_ref()
                .is_none_or(|s| s.binary_search(&v).is_ok())
    }
}

/// A Boolean literal, an integer, or a tuple of values.
#[derive(Clone)]
enum Value {
    Bool(Lit),
    Int(Rc<Int>),
    Tuple(Rc<(Value, Value)>),
}

fn boolean(constant: bool) -> Value {
    Value::Bool(lit(if constant { "true" } else { "false" }))
}

impl Value {
    fn shape(&self) -> char {
        match self {
            Value::Bool(_) => 'b',
            Value::Int(_) => 'i',
            Value::Tuple(_) => 't',
        }
    }

    /// The Booleans and integers, depth first.
    fn flat(&self) -> Vec<Value> {
        let mut out = Vec::new();
        let mut stack = vec![self];
        while let Some(v) = stack.pop() {
            match v {
                Value::Tuple(t) => {
                    stack.push(&t.1);
                    stack.push(&t.0);
                }
                v => out.push(v.clone()),
            }
        }
        out
    }

    /// A name for each of [`Value::flat`]: `base`, and `base_1` and `base_2`
    /// for a tuple's components.
    fn flat_names(&self, base: &str) -> Vec<String> {
        match self {
            Value::Tuple(t) => {
                let mut out = t.0.flat_names(&format!("{base}_1"));
                out.extend(t.1.flat_names(&format!("{base}_2")));
                out
            }
            _ => vec![base.to_string()],
        }
    }

    /// A value of this one's shape from the parts [`Value::flat`] would
    /// give.
    fn rebuild(&self, parts: &mut impl Iterator<Item = Value>) -> Value {
        match self {
            Value::Tuple(t) => {
                let a = t.0.rebuild(parts);
                let b = t.1.rebuild(parts);
                Value::Tuple(Rc::new((a, b)))
            }
            _ => parts.next().expect("a part per Boolean and integer"),
        }
    }

    /// The literals of a Boolean or an integer.
    fn bits(&self) -> Vec<Lit> {
        match self {
            Value::Bool(l) => vec![l.clone()],
            Value::Int(x) => x.bits.clone(),
            Value::Tuple(_) => Vec::new(),
        }
    }
}

/// The variables in scope: each binding shadows the ones after it.
#[derive(Clone, Default)]
struct Env<'a>(Option<Rc<Binding<'a>>>);

struct Binding<'a> {
    name: &'a str,
    value: Value,
    next: Env<'a>,
}

impl<'a> Env<'a> {
    fn bind(&self, name: &'a str, value: Value) -> Env<'a> {
        Env(Some(Rc::new(Binding {
            name,
            value,
            next: self.clone(),
        })))
    }

    fn get(&self, name: &str) -> Option<&Value> {
        let mut at = &self.0;
        while let Some(b) = at {
            if b.name == name {
                return Some(&b.value);
            }
            at = &b.next.0;
        }
        None
    }
}

/// What a condition, or its negation, says of variables compared with
/// constants: each variable and the values it allows (1 and 0 for a
/// Boolean's true and false); `None` where two facts conflict.
type Facts<'a> = Option<BTreeMap<&'a str, BTreeSet<u64>>>;

/// The facts of two conditions that both hold.
fn meet<'a>(a: Facts<'a>, b: Facts<'a>) -> Facts<'a> {
    let (mut out, b) = (a?, b?);
    for (var, allowed) in b {
        let both: BTreeSet<u64> = match out.get(var) {
            Some(have) => have.intersection(&allowed).copied().collect(),
            None => allowed,
        };
        if both.is_empty() {
            return None;
        }
        out.insert(var, both);
    }
    Some(out)
}

/// The facts of a condition that holds where either of two does: the
/// union of what both say of a variable; `None` if neither can hold.
fn join<'a>(a: Facts<'a>, b: Facts<'a>) -> Facts<'a> {
    let (Some(a), Some(b)) = (&a, &b) else {
        return a.or(b);
    };
    Some(
        a.iter()
            .filter_map(|(var, x)| Some((*var, x.union(b.get(var)?).copied().collect())))
            .collect(),
    )
}

fn mirror(op: &str) -> Option<&'static str> {
    Some(match op {
        "==" => "==",
        "!=" => "!=",
        "<" => ">",
        "<=" => ">=",
        ">" => "<",
        ">=" => "<=",
        _ => return None,
    })
}

fn compare(op: &str, a: u64, b: u64) -> bool {
    match op {
        "==" => a == b,
        "!=" => a != b,
        "<" => a < b,
        "<=" => a <= b,
        ">" => a > b,
        _ => a >= b,
    }
}

/// Dice's int width for `n` values (`num_binary_digits (n - 1)`).
fn bits_for(n: usize) -> u64 {
    u64::from((n.max(1) - 1).checked_ilog2().map_or(0, |b| b + 1)).max(1)
}

/// The width of a list's length (Dice's `bit_length max_list_length`).
fn list_width() -> u64 {
    bits_for(LIST_LENGTH + 1)
}

/// The type of a value, a list's as the pair it is held as.
fn type_of(v: &Value) -> Type {
    match v {
        Value::Bool(_) => Type::Bool,
        Value::Int(x) => Type::Int(x.width),
        Value::Tuple(t) => Type::Tuple(Box::new(type_of(&t.0)), Box::new(type_of(&t.1))),
    }
}

/// A list's length and its [`LIST_LENGTH`] elements.
fn list_parts(v: &Value, name: &str, line: usize) -> Res<(Rc<Int>, Vec<Value>)> {
    let not_list = || fail(line, format!("{name}: a list is expected"));
    let Value::Tuple(t) = v else {
        return Err(not_list());
    };
    let Value::Int(n) = &t.0 else {
        return Err(not_list());
    };
    if n.width != list_width() {
        return Err(not_list());
    }
    let mut elems = Vec::with_capacity(LIST_LENGTH);
    let mut rest = &t.1;
    for _ in 1..LIST_LENGTH {
        let Value::Tuple(pair) = rest else {
            return Err(not_list());
        };
        elems.push(pair.0.clone());
        rest = &pair.1;
    }
    elems.push(rest.clone());
    Ok((n.clone(), elems))
}

/// The probabilities of `discrete(ps)` as read: a list whose sum is off
/// one by rounding in the source, by at most 1e-4 (the tolerance of Dice's
/// type check, its `Passes.ml`), is divided by its sum, as Dice does: each
/// bit of its encoding is a flip whose probability is a ratio of sums of
/// the list. Read as written, a sum over one is an error in ProbLog, and
/// one under one leaves the variable without a value in some worlds. The
/// sum is `math.fsum`'s, correctly rounded.
fn discrete_probs(ps: &[f64]) -> Vec<f64> {
    let total = fsum(ps);
    if total != 1.0 && (total - 1.0).abs() <= 1e-4 {
        ps.iter().map(|p| p / total).collect()
    } else {
        ps.to_vec()
    }
}

/// A probability as Python's `repr` writes it.
fn repr(p: f64) -> String {
    let mut s = String::new();
    write_float(&mut s, p);
    s
}

// ---------------------------------------------------------------------
// The translation

/// Operations made once: the key of each, by its operands.
#[derive(Clone, PartialEq, Eq, Hash)]
enum Key {
    Gate(Vec<Vec<Lit>>),
    Constant(u64, u64),
    Arith(&'static str, u64, u64),
    Shift(&'static str, u64, u64),
    And(Lit, Lit),
    Or(Lit, Lit),
    Any(Vec<Lit>),
    Restrict(u64, Vec<u64>),
    Merge(Vec<(Lit, u64)>),
}

/// What [`Translator::live`] makes of a disjunction of conjunctions.
enum Live {
    /// A constant, or one literal.
    Lit(Lit),
    /// The conjunctions left.
    Bodies(Vec<Vec<Lit>>),
}

/// The translation up to a point, which [`Translator::undo`] goes back to.
struct Mark {
    lines: usize,
    evidence: usize,
    counter: FxHashMap<String, usize>,
    taken: FxHashSet<String>,
    memos: FxHashMap<Key, Value>,
}

struct Translator<'a> {
    lines: Vec<String>,
    counter: FxHashMap<String, usize>,
    taken: FxHashSet<String>,
    funs: FxHashMap<&'a str, (&'a Fun<'a>, bool)>,
    evidence: Vec<Lit>,
    active: FxHashMap<&'a str, usize>,
    memos: FxHashMap<Key, Value>,
    /// In a loop's body: the step predicate, and the body's observations.
    in_loop: Option<String>,
    observed: Vec<Lit>,
    /// The loops' step literals.
    steps: FxHashSet<String>,
    ids: u64,
}

impl<'a> Translator<'a> {
    fn new(program: &'a Program<'a>) -> Self {
        let mut funs = FxHashMap::default();
        for f in &program.funs {
            funs.insert(f.name, (f, calls(&f.body, f.name)));
        }
        Translator {
            lines: Vec::new(),
            counter: FxHashMap::default(),
            taken: FxHashSet::default(),
            funs,
            evidence: Vec::new(),
            active: FxHashMap::default(),
            memos: FxHashMap::default(),
            in_loop: None,
            observed: Vec::new(),
            steps: FxHashSet::default(),
            ids: 0,
        }
    }

    fn translate(mut self, body: &'a Expr<'a>) -> Res<String> {
        let result = self.eval(body, &Env::default(), &[], "result")?;
        for a in std::mem::take(&mut self.evidence) {
            self.lines.push(match a.strip_prefix("\\+") {
                Some(atom) => format!("evidence({atom}, false)."),
                None => format!("evidence({a}, true)."),
            });
        }
        self.queries(&result, "result");
        let mut out = self.lines.join("\n");
        out.push('\n');
        Ok(out)
    }

    /// A new integer.
    fn int(
        &mut self,
        width: u64,
        bits: Vec<Lit>,
        support: Option<Vec<u64>>,
        name: Option<Rc<str>>,
        hint: Option<Rc<str>>,
        size: Option<u64>,
    ) -> Rc<Int> {
        let support = support.map(|mut s| {
            s.sort_unstable();
            s
        });
        let size = size.unwrap_or_else(|| match support.as_deref() {
            Some([.., last]) => last + 1,
            _ => 1 << width,
        });
        self.ids += 1;
        Rc::new(Int {
            id: self.ids,
            width,
            bits,
            support,
            name,
            hint,
            size,
        })
    }

    /// A new atom name from `base`: base, base_1, base_2, ..., the first
    /// not taken (base_1 may be another base's).
    fn fresh(&mut self, base: &str) -> String {
        let mut base: String = base
            .chars()
            .map(|c| {
                let c = c.to_ascii_lowercase();
                if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        if base.is_empty() {
            base.push('v');
        }
        if !base.starts_with(|c: char| c.is_ascii_lowercase()) {
            base.insert_str(0, "v_");
        }
        let mut n = self.counter.get(&base).copied().unwrap_or(0);
        let name = loop {
            let name = if n == 0 {
                base.clone()
            } else {
                format!("{base}_{n}")
            };
            if !self.taken.contains(&name) {
                break name;
            }
            n += 1;
        };
        self.counter.insert(base, n + 1);
        self.taken.insert(name.clone());
        name
    }

    fn rule(&mut self, head: &str, body: Vec<Lit>) {
        let body: Vec<Lit> = body.into_iter().filter(|b| !is(b, "true")).collect();
        if body.iter().any(|b| is(b, "false")) {
            return;
        }
        // A loop's rule reads the step's atoms first and their negations
        // after them, as a ProbLog programmer writes it: the grounder then
        // binds the step from the atoms the round before added. A rule to
        // the next step (its head at T1) reads `step(T), T1 is T + 1`
        // first, so that a top-down grounder (ProbLog's) calls the step's
        // atoms with T bound; an atom called at an unbound step can reach
        // its own negation, which ProbLog stops at as a negative cycle.
        let mut body = if head.ends_with("(T1)") {
            body
        } else {
            let first: Vec<Lit> = body
                .iter()
                .filter(|b| at_step(b) && !b.starts_with("\\+") && !self.steps.contains(&***b))
                .cloned()
                .collect();
            let rest = body.iter().filter(|b| !first.contains(b)).cloned();
            let mut ordered = first.clone();
            ordered.extend(rest);
            ordered
        };
        if at_step(head)
            && !body.iter().any(|b| at_step(b) && !b.starts_with("\\+"))
            && let Some(step) = &self.in_loop
        {
            // The step binds T.
            body.insert(0, lit(&format!("{step}{STEP}")));
        }
        self.lines.push(if body.is_empty() {
            format!("{head}.")
        } else {
            format!("{head} :- {}.", body.join(", "))
        });
    }

    /// The conjunctions `bodies` without constants and repeats: `true` or
    /// `false` where their disjunction is constant, the one literal of a
    /// single one-literal body, else the bodies left.
    fn live(bodies: Vec<Vec<Lit>>) -> Live {
        let mut live: Vec<Vec<Lit>> = Vec::new();
        for body in bodies {
            let mut kept: Vec<Lit> = Vec::new();
            for b in body {
                if !is(&b, "true") && !kept.contains(&b) {
                    kept.push(b);
                }
            }
            if kept.iter().any(|b| is(b, "false"))
                || kept
                    .iter()
                    .any(|b| kept.iter().any(|c| complementary(b, c)))
            {
                continue;
            }
            if kept.is_empty() {
                return Live::Lit(lit("true"));
            }
            live.push(kept);
        }
        match live.len() {
            0 => Live::Lit(lit("false")),
            1 if live[0].len() == 1 => Live::Lit(live[0][0].clone()),
            _ => Live::Bodies(live),
        }
    }

    /// The literal for the disjunction of conjunctions `bodies`: a
    /// constant or one literal where [`Translator::live`] folds them to
    /// it, else `head` defined by one rule per body.
    fn define(&mut self, head: &str, bodies: Vec<Vec<Lit>>) -> Lit {
        let live = match Self::live(bodies) {
            Live::Lit(l) => return l,
            Live::Bodies(live) => live,
        };
        let head = if self.in_loop.is_some() && live.iter().flatten().any(|b| at_step(b)) {
            format!("{head}{STEP}")
        } else {
            head.to_string()
        };
        for body in live {
            self.rule(&head, body);
        }
        lit(&head)
    }

    /// `make()` once per key.
    fn memo(&mut self, key: Key, make: impl FnOnce(&mut Self) -> Res<Value>) -> Res<Value> {
        if let Some(v) = self.memos.get(&key) {
            return Ok(v.clone());
        }
        let v = make(self)?;
        self.memos.insert(key, v.clone());
        Ok(v)
    }

    fn memo_lit(&mut self, key: Key, make: impl FnOnce(&mut Self) -> Lit) -> Lit {
        match self.memo(key, |t| Ok(Value::Bool(make(t)))) {
            Ok(Value::Bool(l)) => l,
            _ => unreachable!("a literal's key holds a literal"),
        }
    }

    /// [`Translator::define`] under a fresh atom from `name`, made only
    /// where the bodies do not fold; the same bodies give the same atom.
    fn gate(&mut self, name: &str, bodies: Vec<Vec<Lit>>) -> Lit {
        let live = match Self::live(bodies) {
            Live::Lit(l) => return l,
            Live::Bodies(live) => live,
        };
        self.memo_lit(Key::Gate(live.clone()), |t| {
            let head = t.fresh(name);
            t.define(&head, live)
        })
    }

    fn xor(&mut self, a: &Lit, b: &Lit, name: &str) -> Lit {
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        self.gate(name, vec![vec![a.clone(), neg(b)], vec![neg(a), b.clone()]])
    }

    fn conj(&mut self, a: &Lit, b: &Lit, name: &str) -> Lit {
        if is(a, "false") || is(b, "false") {
            return lit("false");
        }
        if is(a, "true") || a == b {
            return b.clone();
        }
        if is(b, "true") {
            return a.clone();
        }
        let (a, b) = (a.clone(), b.clone());
        self.memo_lit(Key::And(a.clone(), b.clone()), |t| {
            let head = t.fresh(name);
            t.define(&head, vec![vec![a, b]])
        })
    }

    fn disj(&mut self, guards: &[Lit], name: &str) -> Lit {
        if guards.len() == 1 {
            return guards[0].clone();
        }
        self.memo_lit(Key::Any(guards.to_vec()), |t| {
            let head = t.fresh(name);
            t.define(&head, guards.iter().map(|g| vec![g.clone()]).collect())
        })
    }

    /// A flip `p`: a probabilistic fact, or a constant where `p` is 0 or 1.
    fn flip(&mut self, p: f64, name: &str) -> Lit {
        if p <= 0.0 || p >= 1.0 {
            return lit(if p >= 1.0 { "true" } else { "false" });
        }
        let a = self.fresh(name);
        let p = repr(p);
        match &self.in_loop {
            Some(step) => {
                self.lines.push(format!("{p}::{a}{STEP} :- {step}{STEP}."));
                lit(&format!("{a}{STEP}"))
            }
            None => {
                self.lines.push(format!("{p}::{a}."));
                lit(&a)
            }
        }
    }

    fn literal(&self, value: &Value, name: &str, line: usize) -> Res<Lit> {
        match value {
            Value::Bool(l) => Ok(l.clone()),
            _ => Err(fail(line, format!("{name}: a Boolean is expected"))),
        }
    }

    fn integer(&self, value: &Value, name: &str, line: usize) -> Res<Rc<Int>> {
        match value {
            Value::Int(x) => Ok(x.clone()),
            _ => Err(fail(line, format!("{name}: an integer is expected"))),
        }
    }

    fn constant(&mut self, k: u64, width: u64, line: usize) -> Res<Rc<Int>> {
        if width > MAX_WIDTH {
            return Err(fail(
                line,
                format!("int({width}): integers of more than {MAX_WIDTH} bits are not supported"),
            ));
        }
        if k >= 1 << width {
            return Err(fail(
                line,
                format!(
                    "integer constant {k} out of range for {} values",
                    1u64 << width
                ),
            ));
        }
        let v = self.memo(Key::Constant(k, width), |t| {
            let bits = (0..width)
                .map(|i| lit(if k >> i & 1 == 1 { "true" } else { "false" }))
                .collect();
            Ok(Value::Int(t.int(
                width,
                bits,
                Some(vec![k]),
                None,
                None,
                None,
            )))
        })?;
        self.integer(&v, "a constant", line)
    }

    /// A categorical over the values `0..ps.len()` in `width` bits, as
    /// Dice encodes it (`gen_discrete` in its `lib/Passes.ml`): from the
    /// most significant bit down, bit k is a flip whose probability is the
    /// mass of the values with bit k set among those that agree on the
    /// bits above it, over the mass of those values. One flip serves every
    /// setting of the bits above where that ratio is the same (a uniform
    /// over a power of two takes one flip a bit), a ratio of 0 or 1 is a
    /// constant, and the settings that serve a flip are read as a decision
    /// on the bits above, settings of no mass left out. The masses are
    /// summed exactly and each ratio is rounded once.
    fn choice(&mut self, ps: &[f64], width: u64, name: &str, line: usize) -> Res<Value> {
        let a = self.fresh(name);
        let modulus = 1usize << width;
        // prefix[i]: the exact mass of the values below i.
        let mut prefix = Vec::with_capacity(modulus + 1);
        let mut sum = BigRational::from_integer(0.into());
        prefix.push(sum.clone());
        for v in 0..modulus {
            if let Some(&p) = ps.get(v) {
                let exact = BigRational::from_float(p)
                    .ok_or_else(|| fail(line, format!("{p} is not a probability")))?;
                sum += exact;
            }
            prefix.push(sum.clone());
        }
        let zero = BigRational::from_integer(0.into());
        let mut bits: Vec<Lit> = vec![lit("false"); width as usize];
        for k in (0..width as usize).rev() {
            // The ratio at each setting u of the bits above k.
            let settings = modulus >> (k + 1);
            let mut ratio: Vec<Option<f64>> = vec![None; settings];
            for (u, r) in ratio.iter_mut().enumerate() {
                let (lo, mid, hi) = (u << (k + 1), (u << (k + 1)) + (1 << k), (u + 1) << (k + 1));
                let total = &prefix[hi] - &prefix[lo];
                if total != zero {
                    let set = &prefix[hi] - &prefix[mid];
                    *r = Some((set / total).to_f64().unwrap_or(f64::NAN));
                }
            }
            let mut flips: Vec<(f64, Lit)> = Vec::new();
            for r in ratio.iter().flatten() {
                if !flips.iter().any(|(s, _)| s == r) {
                    let f = if *r == 0.0 {
                        lit("false")
                    } else if *r == 1.0 {
                        lit("true")
                    } else {
                        self.flip(*r, &format!("{a}_f{k}"))
                    };
                    flips.push((*r, f));
                }
            }
            let mut bodies = Vec::new();
            for (r, f) in &flips {
                for mut t in paths(&ratio, &bits, k, 0, settings, *r) {
                    t.push(f.clone());
                    bodies.push(t);
                }
            }
            bits[k] = self.gate(&format!("{a}_b{k}"), bodies);
        }
        let support = ps
            .iter()
            .enumerate()
            .filter(|(_, p)| **p > 0.0)
            .map(|(v, _)| v as u64)
            .collect();
        Ok(Value::Int(self.int(
            width,
            bits,
            Some(support),
            Some(Rc::from(a.as_str())),
            None,
            Some(ps.len() as u64),
        )))
    }

    fn same_width(&self, x: &Int, y: &Int, op: &str, line: usize) -> Res<()> {
        if x.width != y.width {
            return Err(fail(
                line,
                format!(
                    "{op}: integers of {} and {} values",
                    x.modulus(),
                    y.modulus()
                ),
            ));
        }
        Ok(())
    }

    /// The values `f` takes on the values of `x` and `y`, where both are
    /// known and their pairs not too many; else `None`.
    fn results<T: Ord>(x: &Int, y: &Int, f: impl Fn(u64, u64) -> T) -> Option<BTreeSet<T>> {
        let (xs, ys) = (x.support.as_ref()?, y.support.as_ref()?);
        if xs.len() * ys.len() > SUPPORT_PAIRS {
            return None;
        }
        Some(
            xs.iter()
                .flat_map(|&a| ys.iter().map(move |&b| (a, b)))
                .map(|(a, b)| f(a, b))
                .collect(),
        )
    }

    /// The bits of `xs + ys + carry` modulo 2^len(xs), the carry rippling
    /// from bit 0.
    fn add_bits(&mut self, xs: &[Lit], ys: &[Lit], mut carry: Lit, name: &str) -> Vec<Lit> {
        let mut out = Vec::new();
        for (k, (x, y)) in xs.iter().zip(ys).enumerate() {
            let h = self.xor(x, y, name);
            out.push(self.xor(&h, &carry, name));
            if k + 1 < xs.len() {
                carry = self.gate(name, vec![vec![x.clone(), y.clone()], vec![carry, h]]);
            }
        }
        out
    }

    /// `x op y` for `+`, `-` and `*`, modulo 2^width: an adder; `x - y` as
    /// `x + ~y + 1`; `x * y` as the sum of `x` shifted by each bit of `y`
    /// that is set. Its atoms are `NAME_b...`; `NAME` is left to its query
    /// atoms.
    fn arith(
        &mut self,
        op: &'static str,
        x: Rc<Int>,
        y: Rc<Int>,
        name: &str,
        line: usize,
    ) -> Res<Value> {
        self.same_width(&x, &y, op, line)?;
        let hint = name;
        let name = format!("{name}_b");
        self.memo(Key::Arith(op, x.id, y.id), |t| {
            let modulus = u128::from(x.modulus());
            let f = |a: u64, b: u64| -> u64 {
                let (a, b) = (u128::from(a), u128::from(b));
                (match op {
                    "+" => a + b,
                    "-" => a + modulus - b,
                    _ => a * b,
                } % modulus) as u64
            };
            let support = Self::results(&x, &y, f);
            if let Some(s) = &support
                && s.len() == 1
            {
                let k = *s.first().expect("one value");
                return Ok(Value::Int(t.constant(k, x.width, line)?));
            }
            let bits = match op {
                "+" => t.add_bits(&x.bits, &y.bits, lit("false"), &name),
                "-" => {
                    let not_y: Vec<Lit> = y.bits.iter().map(|b| neg(b)).collect();
                    t.add_bits(&x.bits, &not_y, lit("true"), &name)
                }
                _ => {
                    let w = x.width as usize;
                    let mut bits = vec![lit("false"); w];
                    for (i, yi) in y.bits.iter().enumerate() {
                        if !is(yi, "false") {
                            let mut part = vec![lit("false"); i];
                            for xj in &x.bits[..w - i] {
                                part.push(t.conj(yi, xj, &name));
                            }
                            bits = t.add_bits(&bits, &part, lit("false"), &name);
                        }
                    }
                    bits
                }
            };
            let support = support.map(|s| s.into_iter().collect());
            Ok(Value::Int(t.int(
                x.width,
                bits,
                support,
                None,
                Some(Rc::from(hint)),
                None,
            )))
        })
    }

    fn shift(&mut self, op: &'static str, x: Rc<Int>, k: u64, name: &str) -> Res<Value> {
        self.memo(Key::Shift(op, x.id, k), |t| {
            let w = x.width as usize;
            let k_bits = usize::try_from(k).unwrap_or(usize::MAX).min(w);
            let (bits, support): (Vec<Lit>, Option<Vec<u64>>) = if op == "<<" {
                let mut bits = vec![lit("false"); k_bits];
                bits.extend(x.bits.iter().take(w - k_bits).cloned());
                let support = x.support.as_ref().map(|s| {
                    s.iter()
                        .map(|&a| {
                            if k >= x.width {
                                0
                            } else {
                                ((u128::from(a) << k) % u128::from(x.modulus())) as u64
                            }
                        })
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect()
                });
                (bits, support)
            } else {
                let mut bits: Vec<Lit> = x.bits[k_bits..].to_vec();
                bits.extend(std::iter::repeat_n(lit("false"), k_bits));
                let support = x.support.as_ref().map(|s| {
                    s.iter()
                        .map(|&a| if k >= 64 { 0 } else { a >> k })
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect()
                });
                (bits, support)
            };
            Ok(Value::Int(t.int(
                x.width,
                bits,
                support,
                None,
                Some(Rc::from(name)),
                None,
            )))
        })
    }

    /// The Boolean `x op y` for integers `x` and `y`: `==` bit by bit, `<`
    /// by a comparator from bit 0 up (`x < y` where its bit is below y's,
    /// or equal and `x < y` below it); a constant where every pair of
    /// their values gives the same.
    fn compare(&mut self, op: &str, x: Rc<Int>, y: Rc<Int>, name: &str, line: usize) -> Res<Value> {
        self.same_width(&x, &y, op, line)?;
        let negated = |v: Res<Value>| -> Res<Value> {
            match v? {
                Value::Bool(l) => Ok(Value::Bool(neg(&l))),
                _ => unreachable!("a comparison is a Boolean"),
            }
        };
        let op: &'static str = match op {
            "!=" => return negated(self.compare("==", x, y, name, line)),
            ">" => return self.compare("<", y, x, name, line),
            ">=" => return self.compare("<=", y, x, name, line),
            "<=" => return negated(self.compare("<", y, x, name, line)),
            "==" => "==",
            _ => "<",
        };
        self.memo(Key::Arith(op, x.id, y.id), |t| {
            if let Some(outcomes) = Self::results(&x, &y, |a, b| compare(op, a, b))
                && outcomes.len() == 1
            {
                return Ok(boolean(*outcomes.first().expect("one outcome")));
            }
            let bname = format!("{name}_b");
            let same: Vec<Lit> = x
                .bits
                .iter()
                .zip(&y.bits)
                .map(|(a, b)| neg(&t.xor(a, b, &bname)))
                .collect();
            if op == "==" {
                return Ok(Value::Bool(t.gate(name, vec![same])));
            }
            let mut lt = lit("false");
            for ((a, b), eq) in x.bits.iter().zip(&y.bits).zip(same) {
                lt = t.gate(name, vec![vec![neg(a), b.clone()], vec![eq, lt]]);
            }
            Ok(Value::Bool(lt))
        })
    }

    /// The literal for `l == r` on Boolean literals.
    fn equal(&mut self, l: &Lit, r: &Lit, name: &str) -> Lit {
        if is(l, "true") || is(l, "false") {
            return if is(l, "true") { r.clone() } else { neg(r) };
        }
        if is(r, "true") || is(r, "false") {
            return if is(r, "true") { l.clone() } else { neg(l) };
        }
        let head = self.fresh(name);
        self.define(
            &head,
            vec![vec![l.clone(), r.clone()], vec![neg(l), neg(r)]],
        )
    }

    fn binary(&mut self, op: &str, l: Value, r: Value, name: &str, line: usize) -> Res<Value> {
        if op == "::" {
            return self.cons(l, r, name, line);
        }
        if matches!(op, "/" | "%") {
            return Err(fail(line, format!("Dice's operator {op} is not supported")));
        }
        if matches!(op, "&&" | "||") {
            let (l, r) = (self.literal(&l, name, line)?, self.literal(&r, name, line)?);
            if op == "&&" {
                return Ok(Value::Bool(self.conj(&l, &r, name)));
            }
            if is(&l, "true") || is(&r, "true") {
                return Ok(boolean(true));
            }
            if is(&l, "false") || l == r {
                return Ok(Value::Bool(r));
            }
            if is(&r, "false") {
                return Ok(Value::Bool(l));
            }
            return Ok(Value::Bool(self.memo_lit(
                Key::Or(l.clone(), r.clone()),
                |t| {
                    let head = t.fresh(name);
                    t.define(&head, vec![vec![l], vec![r]])
                },
            )));
        }
        if matches!(op, "^" | "<=>") || (matches!(op, "==" | "!=") && l.shape() == 'b') {
            // Boolean equality and its negation; Dice reads == and != on
            // integers only, this reader on Booleans too.
            let (a, b) = (self.literal(&l, name, line)?, self.literal(&r, name, line)?);
            let e = self.equal(&a, &b, name);
            return Ok(Value::Bool(if matches!(op, "^" | "!=") {
                neg(&e)
            } else {
                e
            }));
        }
        if l.shape() == 't' || r.shape() == 't' {
            return Err(fail(line, format!("{name}: {op} on a tuple")));
        }
        let (x, y) = (self.integer(&l, name, line)?, self.integer(&r, name, line)?);
        match op {
            "+" => self.arith("+", x, y, name, line),
            "-" => self.arith("-", x, y, name, line),
            "*" => self.arith("*", x, y, name, line),
            _ => self.compare(op, x, y, name, line),
        }
    }

    /// An observation of `c` on `path`: it holds when the branch is not
    /// taken or the condition is true; in a loop's body, at each step.
    fn observe(&mut self, c: Lit, path: &[Lit]) {
        let a = self.fresh("obs") + if self.in_loop.is_some() { STEP } else { "" };
        let mut body = path.to_vec();
        body.push(c.clone());
        self.rule(&a, body);
        for (i, p) in path.iter().enumerate() {
            let mut body = path[..i].to_vec();
            body.push(neg(p));
            self.rule(&a, body);
        }
        if path.is_empty() && (is(&c, "true") || is(&c, "false")) {
            if is(&c, "false") {
                self.lines.push("evidence(false_observation, true).".into());
            }
            return;
        }
        let a = lit(&a);
        if self.in_loop.is_some() {
            self.observed.push(a);
        } else {
            self.evidence.push(a);
        }
    }

    /// Translate expression `e` under the variables `env` and the path
    /// condition `path` (the literals that hold when it is evaluated);
    /// `name` is the hint for the atoms it makes.
    fn eval(&mut self, e: &'a Expr<'a>, env: &Env<'a>, path: &[Lit], name: &str) -> Res<Value> {
        let line = e.line;
        match &e.kind {
            Kind::Let(binds, body) => {
                let mut env = env.clone();
                for (x, v) in binds {
                    let value = self.eval(v, &env, path, x)?;
                    env = env.bind(x, value);
                }
                self.eval(body, &env, path, name)
            }
            Kind::Var(x) => env
                .get(x)
                .cloned()
                .ok_or_else(|| fail(line, format!("unbound variable {x}"))),
            Kind::Bool(b) => Ok(boolean(*b)),
            Kind::Flip(p) => Ok(Value::Bool(self.flip(*p, name))),
            Kind::Discrete(ps) => {
                let ps = discrete_probs(ps);
                self.choice(&ps, bits_for(ps.len()), name, line)
            }
            &Kind::Uniform(width, lo, hi) => {
                if width > MAX_UNIFORM_WIDTH {
                    return Err(fail(
                        line,
                        format!(
                            "uniform({width}, {lo}, {hi}): a uniform over more than 2^{MAX_UNIFORM_WIDTH} values is not supported"
                        ),
                    ));
                }
                if !(lo < hi && hi <= 1 << width) {
                    return Err(fail(
                        line,
                        format!("uniform({width}, {lo}, {hi}): empty or out of range"),
                    ));
                }
                let p = 1.0 / (hi - lo) as f64;
                let ps: Vec<f64> = (0..hi).map(|v| if v < lo { 0.0 } else { p }).collect();
                self.choice(&ps, width, name, line)
            }
            &Kind::Int(width, k) => Ok(Value::Int(self.constant(k, width, line)?)),
            Kind::Tuple(a, b) => {
                let a = self.eval(a, env, path, &format!("{name}_1"))?;
                let b = self.eval(b, env, path, &format!("{name}_2"))?;
                Ok(Value::Tuple(Rc::new((a, b))))
            }
            Kind::Fst(x) | Kind::Snd(x) => match self.eval(x, env, path, name)? {
                Value::Tuple(t) => Ok(if matches!(e.kind, Kind::Fst(_)) {
                    t.0.clone()
                } else {
                    t.1.clone()
                }),
                _ => Err(fail(line, format!("{name}: a tuple is expected"))),
            },
            &Kind::Binomial(width, n, p) => self.binomial(width, n, p, name, line),
            &Kind::NthBit(width, i, ref x) => {
                let v = self.eval(x, env, path, name)?;
                let x = self.integer(&v, name, line)?;
                if x.width != width || i >= width {
                    return Err(fail(
                        line,
                        format!(
                            "nth_bit(int({width}, {i}), x): x has {} bits, bit {i} of {width} is asked",
                            x.width
                        ),
                    ));
                }
                Ok(Value::Bool(x.bits[(width - 1 - i) as usize].clone()))
            }
            Kind::List(items) => {
                if items.len() > LIST_LENGTH {
                    return Err(fail(
                        line,
                        format!(
                            "a list of {} elements: Dice's lists hold at most {LIST_LENGTH}",
                            items.len()
                        ),
                    ));
                }
                let mut elems = Vec::new();
                for (k, x) in items.iter().enumerate() {
                    elems.push(self.eval(x, env, path, &format!("{name}_{}", k + 1))?);
                }
                let n = self.constant(items.len() as u64, list_width(), line)?;
                let t = type_of(&elems[0]);
                self.list(n, elems, &t, name, line)
            }
            Kind::Nil(t) => match t {
                Type::List(_) => self.default(Some(t), name, line),
                _ => Err(fail(
                    line,
                    format!("[] : {t:?}: an empty list has a list type"),
                )),
            },
            Kind::Head(x) | Kind::Tail(x) | Kind::Length(x) => {
                let v = self.eval(x, env, path, name)?;
                let (n, elems) = list_parts(&v, name, line)?;
                if let Kind::Length(_) = e.kind {
                    return Ok(Value::Int(n));
                }
                // Dice's head and tail of an empty list observe false.
                let zero = self.constant(0, list_width(), line)?;
                let empty = self.compare("==", n.clone(), zero, name, line)?;
                let empty = self.literal(&empty, name, line)?;
                if !is(&empty, "false") {
                    self.observe(neg(&empty), path);
                }
                if let Kind::Head(_) = e.kind {
                    return Ok(elems[0].clone());
                }
                let one = self.constant(1, list_width(), line)?;
                let n = self.arith("-", n, one, name, line)?;
                let n = self.integer(&n, name, line)?;
                let t = type_of(&elems[0]);
                self.list(n, elems[1..].to_vec(), &t, name, line)
            }
            Kind::Not(x) => {
                let v = self.eval(x, env, path, name)?;
                Ok(Value::Bool(neg(&self.literal(&v, name, line)?)))
            }
            Kind::Bin(..) | Kind::Shift(..) => self.chain(e, env, path, name),
            Kind::Observe(x) => {
                let v = self.eval(x, env, path, name)?;
                let c = self.literal(&v, name, line)?;
                self.observe(c, path);
                Ok(boolean(true))
            }
            Kind::If(arms, otherwise) => self.conditional(arms, otherwise, env, path, name, line),
            Kind::Call(f, args) => {
                let Some(&(fun, _)) = self.funs.get(f) else {
                    return Err(fail(line, format!("unknown function {f}")));
                };
                if fun.params.len() != args.len() {
                    return Err(fail(
                        line,
                        format!(
                            "{f}: {} parameters, {} arguments",
                            fun.params.len(),
                            args.len()
                        ),
                    ));
                }
                let mut values = Vec::new();
                for (p, x) in fun.params.iter().zip(args) {
                    values.push(self.eval(x, env, path, &format!("{f}_{p}"))?);
                }
                self.call(f, values, path, name, line)
            }
            Kind::Iterate(f, init, k) => {
                let init = self.eval(init, env, path, name)?;
                self.iterate(f, init, *k, path, name, line)
            }
        }
    }

    /// A binary operation or a shift whose left operand is a chain of
    /// them, `((a op b) op c) ...`, evaluated left to right without
    /// recursing down the chain: programs write conjunctions of thousands.
    fn chain(&mut self, e: &'a Expr<'a>, env: &Env<'a>, path: &[Lit], name: &str) -> Res<Value> {
        let mut spine = Vec::new();
        let mut first = e;
        while let Kind::Bin(_, l, _) | Kind::Shift(_, l, _) = &first.kind {
            spine.push(first);
            first = l;
        }
        let mut acc = self.eval(first, env, path, name)?;
        for node in spine.into_iter().rev() {
            acc = match &node.kind {
                Kind::Bin(op, _, r) => {
                    let r = self.eval(r, env, path, name)?;
                    self.binary(op, acc, r, name, node.line)?
                }
                Kind::Shift(op, _, k) => {
                    let x = self.integer(&acc, name, node.line)?;
                    let op = if *op == "<<" { "<<" } else { ">>" };
                    self.shift(op, x, *k, name)?
                }
                _ => unreachable!("the spine holds operations"),
            };
        }
        Ok(acc)
    }

    /// A chain `if c0 then e0 else if c1 then e1 ... else en`, taken as
    /// one: branch i is guarded by g_i, that c_i holds and no earlier
    /// condition did, an atom shared by everything the branch computes,
    /// and the branches' values are merged once. Within a branch, a
    /// variable the guard compares with a constant takes only the values
    /// the guard allows (a constant where it allows one): what the branch
    /// computes is read only under its guard.
    fn conditional(
        &mut self,
        arms: &'a [(Expr<'a>, Expr<'a>)],
        otherwise: &'a Expr<'a>,
        env: &Env<'a>,
        path: &[Lit],
        name: &str,
        line: usize,
    ) -> Res<Value> {
        let unmet = || fail(line, "a branch no value of its variables reaches");
        let (cname, gname) = (format!("{name}_c"), format!("{name}_g"));
        let mut branches: Vec<(Lit, Value)> = Vec::new();
        let mut reach = lit("true");
        let mut known: Facts<'a> = Some(BTreeMap::new());
        for (i, (cond, then)) in arms.iter().enumerate() {
            let at = self.refine(env, &known)?.ok_or_else(unmet)?;
            let v = self.eval(cond, &at, &within(path, &reach), &cname)?;
            let c = self.literal(&v, name, line)?;
            let guard = self.conj(&reach, &c, &gname);
            let facts = self.facts(cond, &at, true);
            let inside = self.refine(&at, &facts)?;
            if !is(&guard, "false")
                && let Some(inside) = inside
            {
                let v = self.eval(
                    then,
                    &inside,
                    &within(path, &guard),
                    &format!("{name}_then"),
                )?;
                branches.push((guard, v));
            }
            reach = self.conj(&reach, &neg(&c), &gname);
            known = meet(known, self.facts(cond, &at, false));
            if is(&reach, "false") || known.is_none() {
                break;
            }
            if i + 1 == arms.len() {
                let at = self.refine(env, &known)?.ok_or_else(unmet)?;
                let v = self.eval(
                    otherwise,
                    &at,
                    &within(path, &reach),
                    &format!("{name}_else"),
                )?;
                branches.push((reach.clone(), v));
            }
        }
        if branches.is_empty() {
            return Err(unmet());
        }
        self.merge(branches, name, line)
    }

    /// The parts of `fname(state)` as a loop's step: its atoms of the
    /// step, its observations in `observed`.
    fn body(
        &mut self,
        fname: &'a str,
        state: Value,
        path: &[Lit],
        name: &str,
        step: &str,
        line: usize,
    ) -> Res<Vec<Value>> {
        self.in_loop = Some(step.to_string());
        self.observed.clear();
        let out = self.call(fname, vec![state.clone()], path, name, line);
        self.in_loop = None;
        let (ends, starts) = (out?.flat(), state.flat());
        let same = |p: &Value, q: &Value| match (p, q) {
            (Value::Int(p), Value::Int(q)) => p.width == q.width,
            _ => p.shape() == q.shape(),
        };
        if ends.len() != starts.len() || starts.iter().zip(&ends).any(|(p, q)| !same(p, q)) {
            return Err(fail(
                line,
                format!("{fname}: the loop step returns a value of another type than its state"),
            ));
        }
        Ok(ends)
    }

    fn mark(&self) -> Mark {
        Mark {
            lines: self.lines.len(),
            evidence: self.evidence.len(),
            counter: self.counter.clone(),
            taken: self.taken.clone(),
            memos: self.memos.clone(),
        }
    }

    /// Back to what the translation was at `mark`.
    fn undo(&mut self, mark: Mark) {
        self.lines.truncate(mark.lines);
        self.evidence.truncate(mark.evidence);
        self.counter = mark.counter;
        self.taken = mark.taken;
        self.memos = mark.memos;
    }

    /// `iterate(fname, init, k)` as recursion over a step index, which the
    /// grounder unrolls: fname's body once, under `step(T)` for T from 0
    /// to k - 1, its flips probabilistic facts of the step and its atoms
    /// of the step where they read one (`name(T)`). Each literal of the
    /// state (a Boolean, a bit of an integer) is an atom s of its own,
    /// `s(0)` the initial state's literal and `s(T1)`, T1 = T + 1, the
    /// body's at step T, and the value is the state at k; a literal the
    /// step keeps (the body returns the initial state's literal where the
    /// state holds it, as the key of the Caesar cipher) is no atom but
    /// itself. Which literals are kept, and the values the state's
    /// integers take (for the comparisons and the conditions that they
    /// decide), are a fixpoint over the step: the initial state's, and
    /// what the body makes of them, until no evaluation of the body adds
    /// to them (the evaluations before the last are undone). The step's
    /// observations hold at every step: `failed` holds where one does not
    /// at some step, and the evidence is that it is false.
    fn iterate(
        &mut self,
        fname: &'a str,
        init: Value,
        k: u64,
        path: &[Lit],
        name: &str,
        line: usize,
    ) -> Res<Value> {
        if k == 0 {
            return Ok(init);
        }
        if self.in_loop.is_some() {
            return Err(fail(line, "iterate inside a loop is not supported"));
        }
        let start = init.flat();
        let mut state = Loop {
            kept: start.iter().map(|p| vec![true; p.bits().len()]).collect(),
            values: start
                .iter()
                .map(|p| match p {
                    Value::Int(x) => x.support.clone(),
                    _ => None,
                })
                .collect(),
            atoms: start.iter().map(|p| vec![None; p.bits().len()]).collect(),
            hints: init.flat_names(fname),
            start,
        };
        let bases = state.hints.clone();
        let mut step: Option<String> = None;
        loop {
            for (i, start) in state.start.iter().enumerate() {
                for (j, atom) in state.atoms[i].iter_mut().enumerate() {
                    if !state.kept[i][j] && atom.is_none() {
                        let base = match start {
                            Value::Bool(_) => bases[i].clone(),
                            _ => format!("{}_b{j}", bases[i]),
                        };
                        *atom = Some(self.fresh(&base));
                    }
                }
            }
            if step.is_none() {
                step = Some(self.fresh("step"));
            }
            let l = step.clone().expect("a step predicate");
            self.steps.insert(format!("{l}{STEP}"));
            let mark = self.mark();
            let at = self.state(&state, &init, STEP);
            let ends = self.body(fname, at, path, name, &l, line);
            self.undo(mark);
            let ends = ends?;
            let mut grew = false;
            for (i, (p, q)) in state.start.iter().zip(&ends).enumerate() {
                for (j, (a, b)) in p.bits().iter().zip(q.bits()).enumerate() {
                    if state.kept[i][j] && *a != b {
                        state.kept[i][j] = false;
                        grew = true;
                    }
                }
                if let (Value::Int(_), Some(values), Value::Int(q)) = (p, &state.values[i], q) {
                    let more = q.support.as_ref().map(|s| {
                        values
                            .iter()
                            .chain(s)
                            .copied()
                            .collect::<BTreeSet<u64>>()
                            .into_iter()
                            .collect::<Vec<u64>>()
                    });
                    if more.as_ref() != Some(values) {
                        state.values[i] = more;
                        grew = true;
                    }
                }
            }
            if !grew {
                break;
            }
        }
        let l = step.expect("a step predicate");
        for (i, start) in state.start.iter().enumerate() {
            for (j, b) in start.bits().into_iter().enumerate() {
                if !state.kept[i][j] {
                    let head = format!("{}(0)", state.atom(i, j));
                    self.rule(&head, vec![b]);
                }
            }
        }
        self.lines.push(format!("{l}(0)."));
        if k > 1 {
            self.lines
                .push(format!("{l}(T1) :- {l}(T), T < {}, T1 is T + 1.", k - 1));
        }
        let at = self.state(&state, &init, STEP);
        let ends = self.body(fname, at, path, name, &l, line)?;
        for (i, q) in ends.iter().enumerate() {
            if let Value::Int(q) = q
                && let Some(h) = q.name.as_ref().or(q.hint.as_ref())
            {
                state.hints[i] = h.to_string();
            }
            for (j, b) in q.bits().into_iter().enumerate() {
                if !state.kept[i][j] {
                    let head = format!("{}(T1)", state.atom(i, j));
                    self.rule(&head, vec![lit(&format!("{l}(T)")), lit("T1 is T + 1"), b]);
                }
            }
        }
        if !self.observed.is_empty() {
            let failed = self.fresh("failed");
            for o in std::mem::take(&mut self.observed) {
                self.rule(&failed, vec![lit(&format!("{l}(T)")), neg(&o)]);
            }
            self.evidence.push(neg(&failed));
        }
        Ok(self.state(&state, &init, &format!("({k})")))
    }

    /// The state of a loop at `index` (`(T)`, or the last step's).
    fn state(&mut self, state: &Loop, init: &Value, index: &str) -> Value {
        let mut parts = Vec::new();
        for (i, p) in state.start.iter().enumerate() {
            if state.kept[i].iter().all(|&k| k) {
                parts.push(p.clone());
                continue;
            }
            let ls: Vec<Lit> = p
                .bits()
                .into_iter()
                .enumerate()
                .map(|(j, b)| {
                    if state.kept[i][j] {
                        b
                    } else {
                        lit(&format!("{}{index}", state.atom(i, j)))
                    }
                })
                .collect();
            parts.push(match p {
                Value::Int(x) => Value::Int(self.int(
                    x.width,
                    ls,
                    state.values[i].clone(),
                    None,
                    Some(Rc::from(state.hints[i].as_str())),
                    None,
                )),
                _ => Value::Bool(ls[0].clone()),
            });
        }
        init.rebuild(&mut parts.into_iter())
    }

    fn call(
        &mut self,
        fname: &'a str,
        args: Vec<Value>,
        path: &[Lit],
        name: &str,
        line: usize,
    ) -> Res<Value> {
        let Some(&(fun, recursive)) = self.funs.get(fname) else {
            return Err(fail(line, format!("unknown function {fname}")));
        };
        if fun.params.len() != args.len() {
            return Err(fail(
                line,
                format!(
                    "{fname}: {} parameters, {} arguments",
                    fun.params.len(),
                    args.len()
                ),
            ));
        }
        let depth = self.active.get(fname).copied().unwrap_or(0);
        if recursive && depth > RECURSION_LIMIT {
            // Dice's last copy of a recursive function: `observe false` on
            // this path, and the return type's default value.
            self.observe(lit("false"), path);
            return self.default(fun.ret.as_ref(), fname, line);
        }
        self.active.insert(fname, depth + 1);
        let mut env = Env::default();
        for (p, a) in fun.params.iter().zip(args) {
            env = env.bind(p, a);
        }
        let out = self.eval(&fun.body, &env, path, name);
        self.active.insert(fname, depth);
        out
    }

    fn default(&mut self, t: Option<&Type>, fname: &str, line: usize) -> Res<Value> {
        match t {
            None => Err(fail(
                line,
                format!("{fname}: a recursive function needs a return type"),
            )),
            Some(Type::Bool) => Ok(boolean(false)),
            Some(Type::Int(w)) => Ok(Value::Int(self.constant(0, *w, line)?)),
            Some(Type::Tuple(a, b)) => {
                let a = self.default(Some(a), fname, line)?;
                let b = self.default(Some(b), fname, line)?;
                Ok(Value::Tuple(Rc::new((a, b))))
            }
            Some(Type::List(t)) => {
                // The empty list: length 0, every element the default.
                let d = self.default(Some(t), fname, line)?;
                let zero = self.constant(0, list_width(), line)?;
                self.list(zero, Vec::new(), &type_of(&d), fname, line)
            }
        }
    }

    /// The list of length `n` whose first elements are `elems`, each of
    /// type `t`, and its others `t`'s default.
    fn list(
        &mut self,
        n: Rc<Int>,
        mut elems: Vec<Value>,
        t: &Type,
        name: &str,
        line: usize,
    ) -> Res<Value> {
        if let Some(x) = elems.iter().map(type_of).find(|x| x != t) {
            return Err(fail(
                line,
                format!("{name}: a list of elements of types {t:?} and {x:?}"),
            ));
        }
        while elems.len() < LIST_LENGTH {
            elems.push(self.default(Some(t), name, line)?);
        }
        let mut rest = elems.pop().expect("an element");
        while let Some(x) = elems.pop() {
            rest = Value::Tuple(Rc::new((x, rest)));
        }
        Ok(Value::Tuple(Rc::new((Value::Int(n), rest))))
    }

    /// `x :: xs`, as Dice makes it: x, then the elements of xs but its
    /// last; the length xs's plus one, unless xs is full.
    fn cons(&mut self, x: Value, xs: Value, name: &str, line: usize) -> Res<Value> {
        let (n, mut elems) = list_parts(&xs, name, line)?;
        let t = type_of(&elems[0]);
        let full = self.constant(LIST_LENGTH as u64, list_width(), line)?;
        let one = self.constant(1, list_width(), line)?;
        let at_full = self.compare("==", n.clone(), full.clone(), name, line)?;
        let at_full = self.literal(&at_full, name, line)?;
        let more = self.arith("+", n, one, name, line)?;
        let n = match &*at_full {
            "false" => more,
            "true" => Value::Int(full),
            _ => self.merge(
                vec![(at_full.clone(), Value::Int(full)), (neg(&at_full), more)],
                name,
                line,
            )?,
        };
        let n = self.integer(&n, name, line)?;
        elems.pop();
        elems.insert(0, x);
        self.list(n, elems, &t, name, line)
    }

    /// `binomial(width, n, p)`, as Dice writes it: from 0, `n` times
    /// `if flip p then x + 1 else x`.
    fn binomial(&mut self, width: u64, n: u64, p: f64, name: &str, line: usize) -> Res<Value> {
        let mut x = self.constant(0, width, line)?;
        if n >= x.modulus() {
            return Err(fail(
                line,
                format!(
                    "binomial({width}, {n}, p): {n} out of range for {} values",
                    x.modulus()
                ),
            ));
        }
        let one = self.constant(1, width, line)?;
        for _ in 0..n {
            let f = self.flip(p, name);
            let more = self.arith("+", x.clone(), one.clone(), name, line)?;
            let v = match &*f {
                "true" => more,
                "false" => Value::Int(x),
                _ => self.merge(
                    vec![(f.clone(), more), (neg(&f), Value::Int(x))],
                    name,
                    line,
                )?,
            };
            x = self.integer(&v, name, line)?;
        }
        Ok(Value::Int(x))
    }

    /// What condition `e`, or its negation where not `positive`, says of
    /// variables compared with constants.
    fn facts(&self, e: &'a Expr<'a>, env: &Env<'a>, positive: bool) -> Facts<'a> {
        let one = |var: &'a str, allowed: BTreeSet<u64>| Some(BTreeMap::from([(var, allowed)]));
        match &e.kind {
            Kind::Not(x) => self.facts(x, env, !positive),
            Kind::Var(x) if matches!(env.get(x), Some(Value::Bool(_))) => {
                one(x, BTreeSet::from([u64::from(positive)]))
            }
            Kind::Bin(op, l, r) => {
                let (op, l, r) = (*op, &**l, &**r);
                if (op == "&&" && positive) || (op == "||" && !positive) {
                    return meet(self.facts(l, env, positive), self.facts(r, env, positive));
                }
                if matches!(op, "&&" | "||") {
                    return join(self.facts(l, env, positive), self.facts(r, env, positive));
                }
                let Some(m) = mirror(op) else {
                    return Some(BTreeMap::new());
                };
                let (op, l, r) = match (&l.kind, &r.kind) {
                    (Kind::Int(..), Kind::Var(_)) => (m, r, l),
                    _ => (op, l, r),
                };
                if let (Kind::Var(x), &Kind::Int(_, k)) = (&l.kind, &r.kind)
                    && let Some(Value::Int(v)) = env.get(x)
                    && let Some(support) = &v.support
                {
                    return one(
                        x,
                        support
                            .iter()
                            .copied()
                            .filter(|&a| compare(op, a, k) == positive)
                            .collect(),
                    );
                }
                Some(BTreeMap::new())
            }
            _ => Some(BTreeMap::new()),
        }
    }

    /// `env` with each variable in `known` restricted to the values it
    /// allows; `None` if one allows none of its values.
    fn refine(&mut self, env: &Env<'a>, known: &Facts<'a>) -> Res<Option<Env<'a>>> {
        let Some(known) = known else {
            return Ok(None);
        };
        let mut out = env.clone();
        for (&var, allowed) in known {
            let Some(x) = env.get(var) else {
                continue;
            };
            match x {
                Value::Bool(l) => {
                    let left: Vec<bool> = [true, false]
                        .into_iter()
                        .filter(|&b| {
                            allowed.contains(&u64::from(b))
                                && !is(l, if b { "false" } else { "true" })
                        })
                        .collect();
                    match left.as_slice() {
                        [] => return Ok(None),
                        [b] => out = out.bind(var, boolean(*b)),
                        _ => {}
                    }
                }
                Value::Int(x) => {
                    let Some(have) = &x.support else {
                        continue;
                    };
                    let support: Vec<u64> = have
                        .iter()
                        .copied()
                        .filter(|a| allowed.contains(a))
                        .collect();
                    if support.is_empty() {
                        return Ok(None);
                    }
                    if support.len() == 1 {
                        let k = self.constant(support[0], x.width, 0)?;
                        out = out.bind(var, Value::Int(k));
                    } else if support.len() < have.len() {
                        // The bits on which the values left agree are
                        // constants.
                        let bits: Vec<Lit> = x
                            .bits
                            .iter()
                            .enumerate()
                            .map(|(k, b)| {
                                let ones = support.iter().filter(|&&a| a >> k & 1 == 1).count();
                                if ones > 0 && ones < support.len() {
                                    b.clone()
                                } else {
                                    lit(if support[0] >> k & 1 == 1 {
                                        "true"
                                    } else {
                                        "false"
                                    })
                                }
                            })
                            .collect();
                        let x = x.clone();
                        let v = self.memo(Key::Restrict(x.id, support.clone()), |t| {
                            Ok(Value::Int(t.int(
                                x.width,
                                bits,
                                Some(support),
                                x.name.clone(),
                                x.hint.clone(),
                                Some(x.size),
                            )))
                        })?;
                        out = out.bind(var, v);
                    }
                }
                Value::Tuple(_) => {}
            }
        }
        Ok(Some(out))
    }

    /// The value of a conditional whose branch values are `branches`,
    /// pairs (guard, value) whose guards are exclusive and exhaustive where
    /// the conditional is evaluated. Equal values are merged first, under
    /// the disjunction of their guards.
    fn merge(&mut self, branches: Vec<(Lit, Value)>, name: &str, line: usize) -> Res<Value> {
        let value = branches[0].1.clone();
        if branches.iter().any(|(_, v)| v.shape() != value.shape()) {
            return Err(fail(line, format!("{name}: branches of different shapes")));
        }
        if let Value::Tuple(_) = value {
            let part = |first: bool| -> Vec<(Lit, Value)> {
                branches
                    .iter()
                    .map(|(g, v)| match v {
                        Value::Tuple(t) => {
                            (g.clone(), if first { t.0.clone() } else { t.1.clone() })
                        }
                        _ => unreachable!("every branch is a tuple"),
                    })
                    .collect()
            };
            let a = self.merge(part(true), &format!("{name}_1"), line)?;
            let b = self.merge(part(false), &format!("{name}_2"), line)?;
            return Ok(Value::Tuple(Rc::new((a, b))));
        }
        // The values, each with its guards, in the order of the branches.
        let mut groups: Vec<(Value, Vec<Lit>)> = Vec::new();
        for (g, v) in branches {
            let same = |w: &Value| match (w, &v) {
                (Value::Bool(a), Value::Bool(b)) => a == b,
                (Value::Int(a), Value::Int(b)) => a.id == b.id,
                _ => false,
            };
            match groups.iter_mut().find(|(w, _)| same(w)) {
                Some((_, gs)) => gs.push(g),
                None => groups.push((v, vec![g])),
            }
        }
        if groups.len() == 1 {
            return Ok(value);
        }
        // The guards are exclusive and exhaustive, so the value with the
        // most of them is guarded by the negation of the others' where
        // those are fewer.
        groups.sort_by_key(|(_, gs)| std::cmp::Reverse(gs.len()));
        let gname = format!("{name}_g");
        let rest: Vec<Lit> = groups[1..]
            .iter()
            .flat_map(|(_, gs)| gs.iter().cloned())
            .collect();
        let first = if rest.len() < groups[0].1.len() {
            neg(&self.disj(&rest, &gname))
        } else {
            self.disj(&groups[0].1, &gname)
        };
        let mut parts = vec![(groups[0].0.clone(), first)];
        for (v, gs) in &groups[1..] {
            let g = self.disj(gs, &gname);
            parts.push((v.clone(), g));
        }
        if let Value::Bool(_) = value {
            let head = self.fresh(name);
            let bodies = parts
                .iter()
                .map(|(v, g)| vec![g.clone(), v.bits()[0].clone()])
                .collect();
            return Ok(Value::Bool(self.define(&head, bodies)));
        }
        let ints: Vec<(Rc<Int>, Lit)> = parts
            .into_iter()
            .map(|(v, g)| match v {
                Value::Int(x) => (x, g),
                _ => unreachable!("every branch is an integer"),
            })
            .collect();
        let Value::Int(first) = &value else {
            unreachable!("an integer");
        };
        for (x, _) in &ints {
            self.same_width(first, x, "if", line)?;
        }
        let key = Key::Merge(ints.iter().map(|(x, g)| (g.clone(), x.id)).collect());
        let width = first.width;
        self.memo(key, |t| {
            let support = ints
                .iter()
                .map(|(x, _)| x.support.clone())
                .collect::<Option<Vec<_>>>()
                .map(|all| {
                    all.into_iter()
                        .flatten()
                        .collect::<BTreeSet<u64>>()
                        .into_iter()
                        .collect()
                });
            let bname = format!("{name}_b");
            let mut bits = Vec::new();
            for (k, first) in ints[0].0.bits.iter().enumerate() {
                let distinct: BTreeSet<&Lit> = ints.iter().map(|(x, _)| &x.bits[k]).collect();
                bits.push(if distinct.len() == 1 {
                    first.clone()
                } else {
                    t.gate(
                        &bname,
                        ints.iter()
                            .map(|(x, g)| vec![g.clone(), x.bits[k].clone()])
                            .collect(),
                    )
                });
            }
            let size = ints.iter().map(|(x, _)| x.size).max();
            Ok(Value::Int(t.int(
                width,
                bits,
                support,
                None,
                Some(Rc::from(name)),
                size,
            )))
        })
    }

    fn queries(&mut self, v: &Value, name: &str) {
        match v {
            Value::Int(x) => {
                // Each value's atom: the conjunction of its bits' literals.
                let q = match x.name.as_ref() {
                    Some(n) => n.to_string(),
                    None => self.fresh(x.hint.as_deref().unwrap_or(name)),
                };
                for k in 0..x.size {
                    let atom = format!("{q}({k})");
                    if x.allows(k) {
                        let body = x
                            .bits
                            .iter()
                            .enumerate()
                            .map(|(i, b)| if k >> i & 1 == 1 { b.clone() } else { neg(b) })
                            .collect();
                        self.rule(&atom, body);
                    }
                    self.lines.push(format!("query({atom})."));
                }
            }
            Value::Bool(l) => {
                if is(l, "true") || is(l, "false") || l.ends_with(')') {
                    // A constant, or an atom of a loop's last step: an atom
                    // of its own, as a Boolean's query has no argument.
                    let a = self.fresh(name);
                    let body = if is(l, "false") { "fail" } else { l };
                    self.lines.push(format!("{a} :- {body}."));
                    self.lines.push(format!("query({a})."));
                } else {
                    self.lines.push(format!("query({l})."));
                }
            }
            Value::Tuple(t) => {
                self.queries(&t.0, &format!("{name}_1"));
                self.queries(&t.1, &format!("{name}_2"));
            }
        }
    }
}

/// A loop's state as [`Translator::iterate`] works it out: the initial
/// state's parts, which of their literals the step keeps, the values each
/// integer takes, the atom of each literal not kept, and each part's hint.
struct Loop {
    start: Vec<Value>,
    kept: Vec<Vec<bool>>,
    values: Vec<Option<Vec<u64>>>,
    atoms: Vec<Vec<Option<String>>>,
    hints: Vec<String>,
}

impl Loop {
    fn atom(&self, i: usize, j: usize) -> &str {
        self.atoms[i][j]
            .as_deref()
            .expect("a literal not kept has an atom")
    }
}

fn within(path: &[Lit], guard: &Lit) -> Vec<Lit> {
    let mut out = path.to_vec();
    if !is(guard, "true") {
        out.push(guard.clone());
    }
    out
}

/// The conjunctions of literals of the bits above `k` that hold for the
/// settings in `[lo, lo + size)` whose ratio is `r`.
fn paths(
    ratio: &[Option<f64>],
    bits: &[Lit],
    k: usize,
    lo: usize,
    size: usize,
    r: f64,
) -> Vec<Vec<Lit>> {
    let range = &ratio[lo..lo + size];
    let mine = range.iter().filter(|x| **x == Some(r)).count();
    if mine == 0 {
        return Vec::new();
    }
    if mine == range.iter().filter(|x| x.is_some()).count() {
        return vec![Vec::new()];
    }
    let half = size / 2;
    // The bit that splits the range.
    let j = k + (usize::BITS - half.leading_zeros()) as usize;
    let mut out: Vec<Vec<Lit>> = paths(ratio, bits, k, lo, half, r)
        .into_iter()
        .map(|t| std::iter::once(neg(&bits[j])).chain(t).collect())
        .collect();
    out.extend(
        paths(ratio, bits, k, lo + half, half, r)
            .into_iter()
            .map(|t| std::iter::once(bits[j].clone()).chain(t).collect()),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(text: &str) -> String {
        fn show(e: &Expr<'_>) -> String {
            match &e.kind {
                Kind::Bin(op, l, r) => format!("({} {op} {})", show(l), show(r)),
                Kind::Not(x) => format!("!{}", show(x)),
                Kind::Var(x) => x.to_string(),
                _ => "?".into(),
            }
        }
        let toks = tokenize(text).unwrap();
        show(&Parser { toks, i: 0 }.expr(0).unwrap())
    }

    /// `!` binds looser than `==` and `<`, tighter than `&&` (Dice's
    /// `lib/Parser.mly`); `==` shares the level of `+` and `-`.
    #[test]
    fn precedence_is_dices() {
        assert_eq!(tree("!a == b && c < d + e"), "(!(a == b) && (c < (d + e)))");
        assert_eq!(tree("a || b && c"), "(a || (b && c))");
        assert_eq!(tree("a - b - c"), "((a - b) - c)");
        // `::` associates to the right, between the comparisons and `+`.
        assert_eq!(tree("a < b :: c :: d + e"), "(a < (b :: (c :: (d + e))))");
    }

    /// A list is its length, 4 bits, and ten elements as nested pairs;
    /// `head` and `tail` of a list that may be empty observe that it is
    /// not, and a list of known length needs no observation.
    #[test]
    fn a_list_is_its_length_and_ten_elements() {
        // The length, the constant 1, is queried at 0 and 1.
        let text = read("let xs = [flip 0.5, true] in tail xs").unwrap();
        let queries = text.lines().filter(|l| l.starts_with("query(")).count();
        assert_eq!(queries, 2 + 10, "{text}");
        assert!(!text.contains("evidence("), "{text}");
        let text =
            read("let xs = if flip 0.5 then [true] else [] : list(bool) in head xs").unwrap();
        assert_eq!(text.matches("evidence(").count(), 1, "{text}");
    }

    #[test]
    fn tokens_are_the_scripts() {
        let toks = tokenize("let x' = flip .5 in (* a\ncomment *) x' <=> 1.e2 // end").unwrap();
        let texts: Vec<&str> = toks.iter().map(|t| t.text).collect();
        assert_eq!(
            texts,
            ["let", "x'", "=", "flip", ".5", "in", "x'", "<=>", "1.e2"]
        );
        assert_eq!(toks[6].line, 2);
        assert!(tokenize("a # b").is_err());
    }

    /// Dice accepts a list within 1e-4 of one and normalizes it (each bit
    /// a ratio of sums); noDice's networks print thirds as 0.333333.
    #[test]
    fn a_discrete_off_one_by_rounding_is_normalized() {
        for ps in [&[0.333333, 0.333333, 0.333333][..], &[0.5, 0.50005]] {
            let got = discrete_probs(ps);
            assert!((fsum(&got) - 1.0).abs() <= 1e-15);
            assert!((got[0] / got[got.len() - 1] - ps[0] / ps[ps.len() - 1]).abs() <= 1e-15);
        }
        assert_eq!(discrete_probs(&[0.25, 0.75]), [0.25, 0.75]);
        assert_eq!(discrete_probs(&[0.25, 0.7]), [0.25, 0.7]);
    }

    #[test]
    fn bits_for_counts_dices_digits() {
        let got: Vec<u64> = [1, 2, 3, 4, 5, 8, 9].iter().map(|&n| bits_for(n)).collect();
        assert_eq!(got, [1, 1, 2, 2, 3, 3, 4]);
    }

    /// The probabilities of the flips of a program's facts.
    fn facts(text: &str) -> Vec<f64> {
        let mut ps: Vec<f64> = text
            .lines()
            .filter_map(|l| l.split_once("::"))
            .map(|(p, _)| p.parse().unwrap())
            .collect();
        ps.sort_by(f64::total_cmp);
        ps
    }

    /// A discrete is drawn bit by bit from the most significant, as Dice
    /// draws it: bit 1 with the mass of 2 and 3; bit 0 with that of 1
    /// among 0 and 1, and of 3 among 2 and 3, each ratio of the masses as
    /// written, exactly. A uniform over a power of two takes one flip a
    /// bit. No atom stands for one value but the queries.
    #[test]
    fn an_integer_is_its_bits() {
        let text = read("let x = discrete(0.1, 0.2, 0.3, 0.4) in x").unwrap();
        let exact = |x: f64| BigRational::from_float(x).unwrap();
        let p: Vec<BigRational> = [0.1, 0.2, 0.3, 0.4].into_iter().map(exact).collect();
        let ratio = |a: BigRational, b: BigRational| (a / b).to_f64().unwrap();
        let mut want = vec![
            ratio(&p[2] + &p[3], &p[0] + &p[1] + &p[2] + &p[3]),
            ratio(p[1].clone(), &p[0] + &p[1]),
            ratio(p[3].clone(), &p[2] + &p[3]),
        ];
        want.sort_by(f64::total_cmp);
        assert_eq!(facts(&text), want, "{text}");
        let text =
            read("let x = uniform(3, 0, 8) in let y = uniform(3, 0, 8) in x + y == int(3, 5)")
                .unwrap();
        assert_eq!(facts(&text), [0.5; 6]);
        assert!(
            !text.lines().any(|l| {
                let head = l.split([' ', '.']).next().unwrap();
                head.ends_with(')') && !l.starts_with("query(")
            }),
            "{text}"
        );
    }

    /// Under `x == 1 || x == 3`, x takes the values 1 and 3 alone, whose
    /// bit 0 is set: in that branch bit 0 is the constant true, so the
    /// result's bit 0, true in the other branch too, is constant, and the
    /// values with bit 0 clear have no rule.
    #[test]
    fn a_disjunctive_guard_restricts_its_branch() {
        let text = read(
            "let x = discrete(0.1, 0.2, 0.3, 0.4) in\n\
             if x == int(2, 1) || x == int(2, 3) then x else int(2, 1)",
        )
        .unwrap();
        let heads: Vec<&str> = text
            .lines()
            .filter(|l| !l.starts_with("query("))
            .map(|l| l.split(" :- ").next().unwrap().trim_end_matches('.'))
            .collect();
        let queries: Vec<&str> = text
            .lines()
            .filter_map(|l| l.strip_prefix("query(")?.strip_suffix(")."))
            .collect();
        let defined: Vec<bool> = queries.iter().map(|q| heads.contains(q)).collect();
        assert_eq!(defined, [false, true, false, true], "{text}");
    }

    #[test]
    fn what_the_reader_does_not_support_is_refused() {
        for (program, error) in [
            ("int(3, 4) / int(3, 2)", "operator / is not supported"),
            ("int(3, 4) % int(3, 2)", "operator % is not supported"),
            ("sample flip 0.5", "`sample` is not supported"),
            ("[flip 0.5, int(2, 1)]", "a list of elements of types"),
            ("int(2, 1) :: [true]", "a list of elements of types"),
            (
                "[true, true, true, true, true, true, true, true, true, true, true]",
                "hold at most 10",
            ),
            ("[] : bool", "an empty list has a list type"),
            ("head (true, false)", "a list is expected"),
            ("nth_bit(int(3, 1), int(2, 1))", "x has 2 bits"),
            ("nth_bit(int(2, 2), int(2, 1))", "bit 2 of 2"),
            ("binomial(2, 4, 0.5)", "4 out of range for 4 values"),
            ("int(2, 1) + int(3, 1)", "integers of 4 and 8 values"),
            ("int(2, 4)", "out of range for 4 values"),
            ("int(64, 4)", "more than 63 bits"),
            ("let x = flip 0.5 in y", "unbound variable y"),
            ("f(flip 0.5)", "unknown function f"),
            (
                "fun f(a: bool) { a } f(true, false)",
                "1 parameters, 2 arguments",
            ),
            ("uniform(2, 3, 3)", "empty or out of range"),
            ("flip 0.5 + int(2, 1)", "an integer is expected"),
        ] {
            let e = read(program).expect_err(program);
            assert!(e.msg.contains(error), "{program}: {e}");
        }
    }
}
