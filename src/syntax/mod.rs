//! The source language: ProbLog's logic programs.
//!
//! Supported: constants (atoms, quoted strings, integers, floats),
//! variables and compound terms, lists among them (`[a, b | T]`);
//! probabilistic facts `p::a.` and rules `p::h :- body.`; annotated
//! disjunctions `p1::a; p2::b :- body.`; flexible probabilities, a term
//! that the body binds or an arithmetic expression over its variables
//! (`P::h(X) :- w(X, P).`, `1/W::h(X) :- w(X, W).`); negative heads
//! `\+h :- body.`; disjunctions and parentheses in bodies; negation as
//! failure (`\+` or `not`); unification `=` and the comparisons of terms;
//! `is` and the arithmetic comparisons with ProbLog's arithmetic; the
//! predicates of ProbLog's `library(lists)` and `library(apply)`,
//! `between/3`, `call/N` and `findall/3` over a deterministic goal;
//! `query/1` and `evidence/1,2`, and their patterns `query(p(X)) :-
//! body.` and `evidence(p(X)) :- body.`. No cut or `assert`. The grounder
//! grounds a program to the Datalog fragment the rest of the crate reads
//! ([`crate::ground`]). docs/design.md, "Language", gives the details.

pub mod dice;
mod lexer;
pub mod lpad;
mod parser;

pub use parser::{ParseError, parse_program};

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// A predicate name or a symbolic constant. Shared rather than owned: the
/// lexer hands out one per distinct name, and normalizing and grounding
/// copy atoms many times over (a network's rules each repeat their row's
/// parents and the switches of the heads before them), which with a
/// `String` would be an allocation per name and per constant.
pub(crate) type Name = Arc<str>;

/// Whether `a` and `b` are the same name: the same text, told by address
/// first, as the lexer hands out one name per text. `Arc` compares a `str`
/// by its text alone.
pub(crate) fn same_name(a: &Name, b: &Name) -> bool {
    Arc::ptr_eq(a, b) || a == b
}

/// The name of a list cell, `'[|]'(Head, Tail)`, as SWI-Prolog names it,
/// and of the empty list.
pub(crate) const CONS: &str = "[|]";
pub(crate) const NIL: &str = "[]";

/// A ground or non-ground term.
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Debug, Eq, Hash, PartialOrd, Ord)]
pub enum Term {
    /// A symbolic constant, e.g. `alice`, `'New York'`, `[]`.
    Const(Name),
    /// An integer constant.
    Int(i64),
    /// A variable, e.g. `X`. `_` is parsed to a fresh unique variable.
    Var(String),
    /// A floating point constant.
    Float(F64),
    /// A compound term `name(args)`; a list is a chain of `'[|]'/2` cells.
    Compound(Arc<Compound>),
}

// The derived equality, with names told by [`same_name`], with which the
// derived hash agrees.
impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Term::Const(a), Term::Const(b)) => same_name(a, b),
            (Term::Int(a), Term::Int(b)) => a == b,
            (Term::Var(a), Term::Var(b)) => a == b,
            (Term::Float(a), Term::Float(b)) => a == b,
            (Term::Compound(a), Term::Compound(b)) => Arc::ptr_eq(a, b) || **a == **b,
            _ => false,
        }
    }
}

/// A float equal to, hashed and ordered as its bits: `2.0` and `2` are
/// different terms, as in ProbLog, which tells constants by their text.
#[derive(Clone, Copy, Debug)]
pub struct F64(pub f64);

impl PartialEq for F64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for F64 {}

impl Hash for F64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl PartialOrd for F64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for F64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl F64 {
    /// A float as ProbLog holds one in a term: rounded to 15 decimal
    /// places, as its constants round every float they are made of, read
    /// or computed (`0.1 + 0.2` is `0.3`). A float of 2^53 or more is an
    /// integer already.
    pub(crate) fn rounded(x: f64) -> F64 {
        if !x.is_finite() || x.abs() >= 9_007_199_254_740_992.0 {
            return F64(x);
        }
        F64(format!("{x:.15}").parse().expect("a float"))
    }
}

/// A compound term: its name and arguments, with its hash, whether it is
/// ground and its depth computed once. Equality, comparison and dropping
/// follow the last argument, a list's tail, in a loop rather than by
/// recursion, so that a long list takes no deep stack.
#[derive(Debug)]
pub struct Compound {
    pub(crate) name: Name,
    pub(crate) args: Box<[Term]>,
    hash: u64,
    ground: bool,
    depth: u32,
}

impl Compound {
    /// The term `name(args)`, which has at least one argument.
    pub(crate) fn term(name: Name, args: Vec<Term>) -> Term {
        debug_assert!(!args.is_empty(), "a compound has arguments");
        let mut h = rustc_hash::FxHasher::default();
        name.hash(&mut h);
        args.hash(&mut h);
        let ground = args.iter().all(Term::is_ground);
        let depth = 1 + args.iter().map(Term::depth).max().unwrap_or(0);
        Term::Compound(Arc::new(Compound {
            name,
            args: args.into_boxed_slice(),
            hash: h.finish(),
            ground,
            depth,
        }))
    }
}

impl PartialEq for Compound {
    fn eq(&self, other: &Self) -> bool {
        let (mut a, mut b) = (self, other);
        loop {
            if std::ptr::eq(a, b) {
                return true;
            }
            if a.hash != b.hash || a.args.len() != b.args.len() || !same_name(&a.name, &b.name) {
                return false;
            }
            let n = a.args.len();
            if a.args[..n - 1] != b.args[..n - 1] {
                return false;
            }
            match (&a.args[n - 1], &b.args[n - 1]) {
                (Term::Compound(x), Term::Compound(y)) => (a, b) = (x, y),
                (x, y) => return x == y,
            }
        }
    }
}

impl Eq for Compound {}

impl Hash for Compound {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl PartialOrd for Compound {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// An order for maps and sorting, not the standard order (`term_cmp`).
impl Ord for Compound {
    fn cmp(&self, other: &Self) -> Ordering {
        let (mut a, mut b) = (self, other);
        loop {
            let o = (&a.name, a.args.len()).cmp(&(&b.name, b.args.len()));
            if o.is_ne() {
                return o;
            }
            let n = a.args.len();
            let o = a.args[..n - 1].cmp(&b.args[..n - 1]);
            if o.is_ne() {
                return o;
            }
            match (&a.args[n - 1], &b.args[n - 1]) {
                (Term::Compound(x), Term::Compound(y)) => (a, b) = (x, y),
                (x, y) => return x.cmp(y),
            }
        }
    }
}

impl Drop for Compound {
    fn drop(&mut self) {
        // The compound arguments no other term holds are taken apart here,
        // one at a time, rather than dropped by recursion.
        fn take(args: &mut [Term], stack: &mut Vec<Arc<Compound>>) {
            for t in args {
                if matches!(t, Term::Compound(_))
                    && let Term::Compound(c) = std::mem::replace(t, Term::Int(0))
                {
                    stack.push(c);
                }
            }
        }
        let mut stack = Vec::new();
        take(&mut self.args, &mut stack);
        while let Some(c) = stack.pop() {
            if let Ok(mut inner) = Arc::try_unwrap(c) {
                take(&mut inner.args, &mut stack);
            }
        }
    }
}

impl Term {
    pub(crate) fn is_var(&self) -> bool {
        matches!(self, Term::Var(_))
    }

    /// Whether the term has no variable.
    pub(crate) fn is_ground(&self) -> bool {
        match self {
            Term::Var(_) => false,
            Term::Compound(c) => c.ground,
            _ => true,
        }
    }

    /// The depth of the term: a constant's or a variable's 0, a
    /// compound's one more than its deepest argument's.
    pub(crate) fn depth(&self) -> u32 {
        match self {
            Term::Compound(c) => c.depth,
            _ => 0,
        }
    }

    /// The compound's name and arguments.
    pub(crate) fn as_compound(&self) -> Option<(&Name, &[Term])> {
        match self {
            Term::Compound(c) => Some((&c.name, &c.args)),
            _ => None,
        }
    }

    /// The variables of the term, each once, in order of first occurrence,
    /// added to `out` where it does not hold them.
    pub(crate) fn vars_into(&self, out: &mut Vec<String>) {
        let mut stack = vec![self];
        while let Some(t) = stack.pop() {
            match t {
                Term::Var(v) if !out.contains(v) => out.push(v.clone()),
                Term::Compound(c) if !c.ground => stack.extend(c.args.iter().rev()),
                _ => {}
            }
        }
    }
}

/// The infix operators of ProbLog's parser, by name: `(priority, type)`.
fn infix(name: &str) -> Option<(u16, Assoc)> {
    use Assoc::*;
    Some(match name {
        ":-" => (1200, Xfx),
        ";" => (1100, Xfy),
        "->" => (1050, Xfy),
        "," => (1000, Xfy),
        "=" | "\\=" | "==" | "\\==" | "@<" | "@>" | "@=<" | "@>=" | "is" | "=:=" | "=\\=" | "<"
        | ">" | "=<" | ">=" => (700, Xfx),
        ":" => (600, Xfy),
        "+" | "-" | "/\\" | "\\/" | "xor" => (500, Yfx),
        "*" | "/" | "//" | "mod" | "rem" | "<<" | ">>" => (400, Yfx),
        "**" => (200, Xfx),
        "^" => (200, Xfy),
        _ => return None,
    })
}

/// The prefix operators of ProbLog's parser, by name, and their priority.
fn prefix(name: &str) -> Option<u16> {
    Some(match name {
        "-" | "+" | "\\" => 200,
        "\\+" => 900,
        _ => return None,
    })
}

/// How an infix operator groups: `xfx` neither way, `xfy` to the right,
/// `yfx` to the left.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Assoc {
    Xfx,
    Xfy,
    Yfx,
}

/// The priority and grouping of the infix operator `name`, for the parser.
pub(crate) fn infix_op(name: &str) -> Option<(u16, Assoc)> {
    infix(name)
}

/// The priority of the prefix operator `name`, for the parser.
pub(crate) fn prefix_op(name: &str) -> Option<u16> {
    prefix(name)
}

/// The infix operators printed infix. ProbLog prints a conjunction and a
/// disjunction as chains of their own ([`write_term`]) and `:-` in functor
/// form.
fn printed_infix(name: &str) -> Option<(u16, Assoc)> {
    match name {
        "," | ";" | ":-" => None,
        _ => infix(name),
    }
}

/// The prefix operators printed prefix: ProbLog prints `\+` in functor
/// form.
fn printed_prefix(name: &str) -> Option<u16> {
    match name {
        "\\+" => None,
        _ => prefix(name),
    }
}

/// The priority a term is printed at: its operator's, 0 for any other.
fn priority(t: &Term) -> u16 {
    match t.as_compound() {
        Some((name, [_, _])) => printed_infix(name).map_or(0, |(p, _)| p),
        Some((name, [_])) => printed_prefix(name).unwrap_or(0),
        _ => 0,
    }
}

/// The two-argument cells of the operator `name` along the right of `t`:
/// the operands of `a, b, c` or `a; b; c`.
fn chain<'a>(name: &str, t: &'a Compound) -> Vec<&'a Term> {
    let mut items = vec![&t.args[0]];
    let mut cell = t;
    loop {
        match &cell.args[1] {
            Term::Compound(next) if *next.name == *name && next.args.len() == 2 => {
                items.push(&next.args[0]);
                cell = next;
            }
            tail => {
                items.push(tail);
                return items;
            }
        }
    }
}

/// A piece of a term being printed ([`write_term`]).
enum Piece<'a> {
    Term(&'a Term, u16),
    Name(&'a Name),
    /// An operator's name, written bare.
    Raw(&'a str),
    Text(&'static str),
}

/// `t` as ProbLog prints it, in a place that takes priorities up to `max`:
/// lists in brackets, operators infix with the parentheses their priorities
/// need, a conjunction `(a, b, c)` in parentheses of its own and with a
/// disjunction in it in parentheses, a disjunction `a; b; c` bare, and the
/// arguments of a compound and the items of a list without parentheses.
/// Printed from a stack of pieces, not by recursion.
fn write_term(f: &mut fmt::Formatter<'_>, t: &Term, max: u16) -> fmt::Result {
    let mut stack = vec![Piece::Term(t, max)];
    let mut pieces: Vec<Piece> = Vec::new();
    while let Some(piece) = stack.pop() {
        let (t, max) = match piece {
            Piece::Text(s) => {
                f.write_str(s)?;
                continue;
            }
            Piece::Name(s) => {
                write_const(f, s)?;
                continue;
            }
            Piece::Raw(s) => {
                f.write_str(s)?;
                continue;
            }
            Piece::Term(t, max) => (t, max),
        };
        let c = match t {
            Term::Compound(c) => c,
            other => {
                write_flat(f, other)?;
                continue;
            }
        };
        pieces.clear();
        let paren = priority(t) > max;
        if paren {
            pieces.push(Piece::Text("("));
        }
        match (&*c.name, &c.args[..]) {
            (CONS, [_, _]) => {
                pieces.push(Piece::Text("["));
                let mut cell = c;
                loop {
                    pieces.push(Piece::Term(&cell.args[0], 1200));
                    match &cell.args[1] {
                        Term::Compound(next) if &*next.name == CONS && next.args.len() == 2 => {
                            pieces.push(Piece::Text(", "));
                            cell = next;
                        }
                        Term::Const(nil) if &**nil == NIL => break,
                        tail => {
                            pieces.push(Piece::Text(" | "));
                            pieces.push(Piece::Term(tail, 1200));
                            break;
                        }
                    }
                }
                pieces.push(Piece::Text("]"));
            }
            (",", [_, _]) => {
                pieces.push(Piece::Text("("));
                for (i, a) in chain(",", c).into_iter().enumerate() {
                    if i > 0 {
                        pieces.push(Piece::Text(", "));
                    }
                    let or = matches!(a.as_compound(), Some((n, [_, _])) if &**n == ";");
                    if or {
                        pieces.push(Piece::Text("("));
                    }
                    pieces.push(Piece::Term(a, 1200));
                    if or {
                        pieces.push(Piece::Text(")"));
                    }
                }
                pieces.push(Piece::Text(")"));
            }
            (";", [_, _]) => {
                for (i, a) in chain(";", c).into_iter().enumerate() {
                    if i > 0 {
                        pieces.push(Piece::Text("; "));
                    }
                    pieces.push(Piece::Term(a, 1200));
                }
            }
            (name, [a, b]) if printed_infix(name).is_some() => {
                let (p, assoc) = printed_infix(name).expect("an operator");
                let left = if assoc == Assoc::Yfx { p } else { p - 1 };
                let right = if assoc == Assoc::Xfy { p } else { p - 1 };
                pieces.push(Piece::Term(a, left));
                let alphabetic = name.starts_with(|ch: char| ch.is_ascii_lowercase());
                if alphabetic {
                    pieces.push(Piece::Text(" "));
                }
                pieces.push(Piece::Raw(&c.name));
                if alphabetic {
                    pieces.push(Piece::Text(" "));
                }
                pieces.push(Piece::Term(b, right));
            }
            (name, [a]) if printed_prefix(name).is_some() => {
                pieces.push(Piece::Raw(&c.name));
                // `-(1)` is not the integer -1, and `- (a, b)` not a list
                // of arguments.
                if matches!(a, Term::Int(_) | Term::Float(_)) || priority(a) >= 1000 {
                    pieces.push(Piece::Text("("));
                    pieces.push(Piece::Term(a, 1200));
                    pieces.push(Piece::Text(")"));
                } else {
                    pieces.push(Piece::Term(a, printed_prefix(name).expect("an operator")));
                }
            }
            (_, args) => {
                pieces.push(Piece::Name(&c.name));
                pieces.push(Piece::Text("("));
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        pieces.push(Piece::Text(","));
                    }
                    pieces.push(Piece::Term(a, 1200));
                }
                pieces.push(Piece::Text(")"));
            }
        }
        if paren {
            pieces.push(Piece::Text(")"));
        }
        stack.extend(pieces.drain(..).rev());
    }
    Ok(())
}

/// A term other than a compound.
fn write_flat(f: &mut fmt::Formatter<'_>, t: &Term) -> fmt::Result {
    match t {
        Term::Const(s) => write_const(f, s),
        Term::Int(i) => write!(f, "{i}"),
        Term::Var(v) => f.write_str(v),
        Term::Float(x) => write_float(f, x.0),
        Term::Compound(_) => write_term(f, t, 1200),
    }
}

/// A float as Python's `repr` writes it, which ProbLog prints: the
/// shortest digits that read back as it, in fixed notation from 1e-4 up to
/// 1e16 (`2.0`, `0.0001`), in scientific notation otherwise (`1e-05`,
/// `1e+16`).
fn write_float(f: &mut fmt::Formatter<'_>, x: f64) -> fmt::Result {
    if x.is_nan() {
        return f.write_str("nan");
    }
    if x.is_infinite() {
        return f.write_str(if x > 0.0 { "inf" } else { "-inf" });
    }
    let s = format!("{x:e}");
    let (mantissa, exp) = s.split_once('e').expect("an exponent");
    let exp: i32 = exp.parse().expect("an integer exponent");
    let (sign, mantissa) = match mantissa.strip_prefix('-') {
        Some(m) => ("-", m),
        None => ("", mantissa),
    };
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let n = digits.len() as i32;
    f.write_str(sign)?;
    if !(-4..16).contains(&exp) {
        let (first, rest) = digits.split_at(1);
        f.write_str(first)?;
        if !rest.is_empty() {
            write!(f, ".{rest}")?;
        }
        return write!(
            f,
            "e{}{:02}",
            if exp < 0 { '-' } else { '+' },
            exp.unsigned_abs()
        );
    }
    if exp < 0 {
        f.write_str("0.")?;
        for _ in 0..(-exp - 1) {
            f.write_str("0")?;
        }
        f.write_str(&digits)
    } else if n <= exp + 1 {
        f.write_str(&digits)?;
        for _ in 0..(exp + 1 - n) {
            f.write_str("0")?;
        }
        f.write_str(".0")
    } else {
        let (int, frac) = digits.split_at(exp as usize + 1);
        write!(f, "{int}.{frac}")
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_term(f, self, 1200)
    }
}

/// A constant or a predicate name as written, quoted where it must be,
/// each backslash and quote inside escaped with a backslash: written
/// piece by piece, without copying the text.
fn write_const(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    if !needs_quotes(s) {
        return f.write_str(s);
    }
    f.write_str("'")?;
    let mut rest = s;
    while let Some(i) = rest.find(['\\', '\'']) {
        f.write_str(&rest[..i])?;
        f.write_str("\\")?;
        f.write_str(&rest[i..=i])?;
        rest = &rest[i + 1..];
    }
    f.write_str(rest)?;
    f.write_str("'")
}

fn needs_quotes(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => !chars.all(|c| c.is_alphanumeric() || c == '_'),
        // The empty list, `;`, `!` and names of symbol characters (`\+`,
        // `:-`) print bare.
        _ => {
            !(s == NIL
                || s == ";"
                || s == "!"
                || (!s.is_empty() && s.chars().all(|c| "+-*/\\^<>=~:.?@#&$".contains(c))))
        }
    }
}

/// The arguments of an atom, read as a slice of terms. One argument, the
/// common case (each atom of a network is a variable's value), is held in
/// place: an atom read, copied or dropped allocates nothing for it.
#[derive(Clone)]
pub enum Args {
    One(Term),
    Many(Vec<Term>),
}

impl Args {
    pub fn as_slice(&self) -> &[Term] {
        match self {
            Args::One(t) => std::slice::from_ref(t),
            Args::Many(ts) => ts,
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Args::Many(Vec::new())
    }
}

impl std::ops::Deref for Args {
    type Target = [Term];
    fn deref(&self) -> &[Term] {
        self.as_slice()
    }
}

impl<'a> IntoIterator for &'a Args {
    type Item = &'a Term;
    type IntoIter = std::slice::Iter<'a, Term>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl From<Vec<Term>> for Args {
    fn from(mut ts: Vec<Term>) -> Self {
        if ts.len() == 1 {
            Args::One(ts.pop().expect("one term"))
        } else {
            Args::Many(ts)
        }
    }
}

impl From<Args> for Vec<Term> {
    fn from(args: Args) -> Self {
        match args {
            Args::One(t) => vec![t],
            Args::Many(ts) => ts,
        }
    }
}

impl FromIterator<Term> for Args {
    fn from_iter<I: IntoIterator<Item = Term>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        // One term, where the size hint says so, held in place; any other
        // number collected as a vector is, in one allocation where the
        // hint is exact.
        if iter.size_hint() == (1, Some(1))
            && let Some(t) = iter.next()
        {
            return Args::One(t);
        }
        Args::from(iter.collect::<Vec<Term>>())
    }
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl PartialEq for Args {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for Args {}

impl std::hash::Hash for Args {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl PartialOrd for Args {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Args {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

/// A predicate atom `name(args)`.
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Debug, Default, Eq, Hash, PartialOrd, Ord)]
pub struct Atom {
    pub name: Name,
    pub args: Args,
}

// As [`Term`]'s.
impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        same_name(&self.name, &other.name) && self.args == other.args
    }
}

impl Atom {
    pub(crate) fn new(name: impl Into<Name>, args: impl Into<Args>) -> Self {
        Atom {
            name: name.into(),
            args: args.into(),
        }
    }
    pub(crate) fn is_ground(&self) -> bool {
        self.args.iter().all(Term::is_ground)
    }
    /// The predicate signature `name/arity`.
    pub(crate) fn signature(&self) -> (&str, usize) {
        (&self.name, self.args.len())
    }
    /// The variables of the atom, in order of first occurrence.
    pub(crate) fn vars_into(&self, out: &mut Vec<String>) {
        for t in &self.args {
            t.vars_into(out);
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_const(f, &self.name)?;
        if !self.args.is_empty() {
            f.write_str("(")?;
            for (i, a) in self.args.iter().enumerate() {
                if i > 0 {
                    f.write_str(",")?;
                }
                write_term(f, a, 1200)?;
            }
            f.write_str(")")?;
        }
        Ok(())
    }
}

/// A comparison operator usable in rule bodies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum CmpOp {
    /// `=`: unification. The grounder takes compound terms apart
    /// ([`crate::ground`]); between flat terms it binds an unbound side.
    Eq,
    /// `\=`: both sides bound and different.
    Ne,
    /// `==`: identity, both sides must be bound.
    Identical,
    /// `\==`
    NotIdentical,
    /// `<`, `=<`, `>`, `>=`: both sides evaluated, numbers compared by
    /// value, other terms on the standard order.
    Lt,
    Le,
    Gt,
    Ge,
    /// `=:=` and `=\=`: both sides evaluated, compared as numbers.
    ArithEq,
    ArithNe,
    /// `@<`, `@=<`, `@>`, `@>=`: the standard order of terms.
    TermLt,
    TermLe,
    TermGt,
    TermGe,
}

impl CmpOp {
    /// Whether the operator evaluates its sides.
    pub(crate) fn arithmetic(self) -> bool {
        matches!(
            self,
            CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge | CmpOp::ArithEq | CmpOp::ArithNe
        )
    }

    /// The comparison that holds of two bound sides where this one does
    /// not, for a negated comparison: `\+ X = Y` is `X \= Y` and
    /// `\+ X \= Y` is `X == Y`, both of which bind nothing.
    pub(crate) fn negated(self) -> CmpOp {
        match self {
            CmpOp::Eq => CmpOp::Ne,
            CmpOp::Ne => CmpOp::Identical,
            CmpOp::Identical => CmpOp::NotIdentical,
            CmpOp::NotIdentical => CmpOp::Identical,
            CmpOp::Lt => CmpOp::Ge,
            CmpOp::Le => CmpOp::Gt,
            CmpOp::Gt => CmpOp::Le,
            CmpOp::Ge => CmpOp::Lt,
            CmpOp::ArithEq => CmpOp::ArithNe,
            CmpOp::ArithNe => CmpOp::ArithEq,
            CmpOp::TermLt => CmpOp::TermGe,
            CmpOp::TermLe => CmpOp::TermGt,
            CmpOp::TermGt => CmpOp::TermLe,
            CmpOp::TermGe => CmpOp::TermLt,
        }
    }

    /// The operator written `text`.
    pub(crate) fn of(text: &str) -> Option<CmpOp> {
        Some(match text {
            "=" => CmpOp::Eq,
            "\\=" => CmpOp::Ne,
            "==" => CmpOp::Identical,
            "\\==" => CmpOp::NotIdentical,
            "<" => CmpOp::Lt,
            "=<" => CmpOp::Le,
            ">" => CmpOp::Gt,
            ">=" => CmpOp::Ge,
            "=:=" => CmpOp::ArithEq,
            "=\\=" => CmpOp::ArithNe,
            "@<" => CmpOp::TermLt,
            "@=<" => CmpOp::TermLe,
            "@>" => CmpOp::TermGt,
            "@>=" => CmpOp::TermGe,
            _ => return None,
        })
    }

    /// Whether the comparison holds of two values, evaluated where the
    /// operator evaluates its sides.
    pub(crate) fn holds(self, a: &Term, b: &Term) -> bool {
        let ordered = || num_cmp(a, b).unwrap_or_else(|| term_cmp(a, b));
        match self {
            CmpOp::Eq | CmpOp::Identical => a == b,
            CmpOp::Ne | CmpOp::NotIdentical => a != b,
            CmpOp::ArithEq => num_cmp(a, b).is_some_and(Ordering::is_eq),
            CmpOp::ArithNe => num_cmp(a, b).is_some_and(Ordering::is_ne),
            CmpOp::Lt => ordered().is_lt(),
            CmpOp::Le => ordered().is_le(),
            CmpOp::Gt => ordered().is_gt(),
            CmpOp::Ge => ordered().is_ge(),
            CmpOp::TermLt => term_cmp(a, b).is_lt(),
            CmpOp::TermLe => term_cmp(a, b).is_le(),
            CmpOp::TermGt => term_cmp(a, b).is_gt(),
            CmpOp::TermGe => term_cmp(a, b).is_ge(),
        }
    }
}

/// Two numbers compared by value; `None` where either is not a number.
pub(crate) fn num_cmp(a: &Term, b: &Term) -> Option<Ordering> {
    match (a, b) {
        (Term::Int(x), Term::Int(y)) => Some(x.cmp(y)),
        (Term::Int(x), Term::Float(y)) => (*x as f64).partial_cmp(&y.0),
        (Term::Float(x), Term::Int(y)) => x.0.partial_cmp(&(*y as f64)),
        (Term::Float(x), Term::Float(y)) => x.0.partial_cmp(&y.0),
        _ => None,
    }
}

/// The standard order of terms as ProbLog's `struct_cmp` gives it:
/// variables, numbers, atoms by their text, compound terms by arity, then
/// name, then arguments. Two numbers of the same value put the float first;
/// two of different values fall through `struct_cmp`'s comparison by value
/// to its comparison of names, and are ordered by their text: `10` before
/// `2` and `-1` before `-2`.
pub(crate) fn term_cmp(a: &Term, b: &Term) -> Ordering {
    fn rank(t: &Term) -> u8 {
        match t {
            Term::Var(_) => 0,
            Term::Float(_) | Term::Int(_) => 1,
            Term::Const(_) => 2,
            Term::Compound(_) => 3,
        }
    }
    let (mut a, mut b) = (a, b);
    loop {
        match (a, b) {
            (Term::Var(x), Term::Var(y)) => return x.cmp(y),
            (Term::Const(x), Term::Const(y)) => return x.cmp(y),
            (Term::Compound(x), Term::Compound(y)) => {
                let o = (x.args.len(), &x.name).cmp(&(y.args.len(), &y.name));
                if o.is_ne() {
                    return o;
                }
                let n = x.args.len();
                for (s, t) in x.args[..n - 1].iter().zip(&y.args[..n - 1]) {
                    let o = term_cmp(s, t);
                    if o.is_ne() {
                        return o;
                    }
                }
                (a, b) = (&x.args[n - 1], &y.args[n - 1]);
            }
            _ => {
                return match num_cmp(a, b) {
                    Some(Ordering::Equal) => {
                        matches!(a, Term::Int(_)).cmp(&matches!(b, Term::Int(_)))
                    }
                    Some(_) => a.to_string().cmp(&b.to_string()),
                    None => rank(a).cmp(&rank(b)),
                };
            }
        }
    }
}

impl fmt::Display for CmpOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CmpOp::Eq => "=",
            CmpOp::Ne => "\\=",
            CmpOp::Identical => "==",
            CmpOp::NotIdentical => "\\==",
            CmpOp::Lt => "<",
            CmpOp::Le => "=<",
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
            CmpOp::ArithEq => "=:=",
            CmpOp::ArithNe => "=\\=",
            CmpOp::TermLt => "@<",
            CmpOp::TermLe => "@=<",
            CmpOp::TermGt => "@>",
            CmpOp::TermGe => "@>=",
        })
    }
}

/// A predicate of ProbLog's `library(lists)`, or `between/3`, which the
/// grounder evaluates on ground arguments where the program defines no
/// predicate of its name and arity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Builtin {
    Member,
    Memberchk,
    Select,
    Append,
    Length,
    Nth0,
    Nth1,
    Last,
    Reverse,
    SumList,
    MaxList,
    MinList,
    Sort,
    Between,
}

impl Builtin {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Builtin::Member => "member",
            Builtin::Memberchk => "memberchk",
            Builtin::Select => "select",
            Builtin::Append => "append",
            Builtin::Length => "length",
            Builtin::Nth0 => "nth0",
            Builtin::Nth1 => "nth1",
            Builtin::Last => "last",
            Builtin::Reverse => "reverse",
            Builtin::SumList => "sum_list",
            Builtin::MaxList => "max_list",
            Builtin::MinList => "min_list",
            Builtin::Sort => "sort",
            Builtin::Between => "between",
        }
    }

    pub(crate) fn arity(self) -> usize {
        match self {
            Builtin::Select
            | Builtin::Append
            | Builtin::Nth0
            | Builtin::Nth1
            | Builtin::Between => 3,
            _ => 2,
        }
    }

    /// The built-in `name/arity`.
    pub(crate) fn of(name: &str, arity: usize) -> Option<Builtin> {
        let b = match name {
            "member" => Builtin::Member,
            "memberchk" => Builtin::Memberchk,
            "select" => Builtin::Select,
            "append" => Builtin::Append,
            "length" => Builtin::Length,
            "nth0" => Builtin::Nth0,
            "nth1" => Builtin::Nth1,
            "last" => Builtin::Last,
            "reverse" => Builtin::Reverse,
            "sum_list" => Builtin::SumList,
            "max_list" => Builtin::MaxList,
            "min_list" => Builtin::MinList,
            "sort" => Builtin::Sort,
            "between" => Builtin::Between,
            _ => return None,
        };
        (b.arity() == arity).then_some(b)
    }

    /// Whether `name` is the name of a predicate of `library(lists)` the
    /// grounder evaluates.
    pub(crate) fn in_lists(name: &str) -> bool {
        [2, 3]
            .into_iter()
            .filter_map(|arity| Builtin::of(name, arity))
            .any(|b| b != Builtin::Between)
    }

    /// Whether the built-in runs with the arguments bound that `bound`
    /// flags: the list it reads (the third of `append/3`'s or both others),
    /// with `nth1/3`'s index, as ProbLog's library computes `I - 1` first,
    /// or the bounds of `between/3`. It then binds the others.
    pub(crate) fn runs(self, bound: &[bool]) -> bool {
        match self {
            Builtin::Member | Builtin::Memberchk | Builtin::Select | Builtin::Nth0 => bound[1],
            Builtin::Nth1 => bound[0] && bound[1],
            Builtin::Length
            | Builtin::Last
            | Builtin::SumList
            | Builtin::MaxList
            | Builtin::MinList
            | Builtin::Reverse
            | Builtin::Sort => bound[0],
            Builtin::Append => bound[2] || (bound[0] && bound[1]),
            Builtin::Between => bound[0] && bound[1],
        }
    }
}

impl fmt::Display for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.name(), self.arity())
    }
}

/// `findall(Template, Goal, Result)` as read: the goal's alternatives,
/// each a conjunction. The grounder lowers it to a [`Collect`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Findall {
    pub(crate) template: Term,
    pub(crate) goal: Vec<Vec<Literal>>,
    pub(crate) result: Term,
    /// Whether a disjunction of the goal comes after another literal
    /// ([`Program::interleaved`]).
    pub(crate) interleaved: bool,
}

/// A findall lowered by the grounder: `result` is the list of the template
/// values of the tuples of `aux/arity` whose first arguments are the values
/// of `shared`, in the order of their derivations. A tuple's next argument
/// is its template value, and the others are the goal's other variables.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Collect {
    pub(crate) aux: Name,
    pub(crate) arity: usize,
    pub(crate) shared: Vec<Term>,
    pub(crate) result: Term,
    /// The findall as written, for the errors that name it.
    pub(crate) text: Arc<str>,
}

impl Collect {
    /// The tuples of the auxiliary predicate the findall reads: its shared
    /// arguments, and a variable of its own at each other position.
    pub(crate) fn pattern(&self) -> Atom {
        let mut args = self.shared.clone();
        args.extend((args.len()..self.arity).map(|i| Term::Var(format!("$P{i}"))));
        Atom::new(self.aux.clone(), args)
    }
}

/// A body literal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Literal {
    Pos(Atom),
    /// Negation as failure, `\+ atom`.
    Neg(Atom),
    Cmp(CmpOp, Term, Term),
    /// `Term is Expr`.
    Is(Term, Term),
    Findall(Box<Findall>),
    /// A built-in over its arguments, negated where the flag says so: a
    /// call to one as the grounder reads it ([`Builtin`]).
    Builtin(Builtin, Vec<Term>, bool),
    Collect(Box<Collect>),
}

impl Literal {
    /// The variables of the literal, in order of first occurrence.
    pub(crate) fn vars_into(&self, out: &mut Vec<String>) {
        match self {
            Literal::Pos(a) | Literal::Neg(a) => a.vars_into(out),
            Literal::Cmp(_, a, b) | Literal::Is(a, b) => {
                a.vars_into(out);
                b.vars_into(out);
            }
            Literal::Findall(fa) => {
                fa.template.vars_into(out);
                for l in fa.goal.iter().flatten() {
                    l.vars_into(out);
                }
                fa.result.vars_into(out);
            }
            Literal::Builtin(_, args, _) => {
                for t in args {
                    t.vars_into(out);
                }
            }
            Literal::Collect(c) => {
                for t in &c.shared {
                    t.vars_into(out);
                }
                c.result.vars_into(out);
            }
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Pos(a) => write!(f, "{a}"),
            Literal::Neg(a) => write!(f, "\\+{a}"),
            Literal::Cmp(op, a, b) => {
                write_term(f, a, 699)?;
                write!(f, "{op}")?;
                write_term(f, b, 699)
            }
            Literal::Is(v, e) => {
                write_term(f, v, 699)?;
                f.write_str(" is ")?;
                write_term(f, e, 699)
            }
            Literal::Findall(fa) => {
                f.write_str("findall(")?;
                write_term(f, &fa.template, 999)?;
                f.write_str(",(")?;
                for (k, alt) in fa.goal.iter().enumerate() {
                    if k > 0 {
                        f.write_str(";")?;
                    }
                    if alt.is_empty() {
                        f.write_str("true")?;
                    }
                    for (i, l) in alt.iter().enumerate() {
                        if i > 0 {
                            f.write_str(",")?;
                        }
                        write!(f, "{l}")?;
                    }
                }
                f.write_str("),")?;
                write_term(f, &fa.result, 999)?;
                f.write_str(")")
            }
            Literal::Builtin(b, args, negated) => {
                if *negated {
                    f.write_str("\\+")?;
                }
                write!(f, "{}", Atom::new(b.name(), args.clone()))
            }
            Literal::Collect(c) => f.write_str(&c.text),
        }
    }
}

/// A head's probability: a number, or a flexible probability, a term whose
/// variables the clause binds (`P::h(X) :- w(X, P).`, `1/W::h(X) :- w(X,
/// W).`), which is a number in each ground instance of the clause.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Annotation {
    Number(f64),
    Flexible(Term),
}

/// One head alternative of a clause with its probability annotation.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HeadChoice {
    /// `None` for a deterministic head.
    pub(crate) prob: Option<Annotation>,
    pub(crate) atom: Atom,
    /// A negative head literal `\+atom`: when the body holds, the atom is
    /// made false (ProbLog's "negative head literals").
    pub(crate) negated: bool,
}

impl HeadChoice {
    /// The head's mass among its clause's heads: its probability, or one
    /// for a head without one; `None` for a flexible probability, which
    /// only a ground instance of the clause knows.
    pub(crate) fn mass(&self) -> Option<f64> {
        match &self.prob {
            None => Some(1.0),
            Some(Annotation::Number(p)) => Some(*p),
            Some(Annotation::Flexible(_)) => None,
        }
    }
}

/// A clause: one or more (annotated) heads and a body.
///
/// A plain fact or rule has one head with `prob == None`. A probabilistic fact or
/// rule has one head with a probability. An annotated disjunction has several.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Clause {
    pub(crate) heads: Vec<HeadChoice>,
    pub(crate) body: Vec<Literal>,
    /// Clauses produced from one source clause by splitting a body
    /// disjunction share a group, and thereby their probabilistic switches.
    pub(crate) group: u32,
}

impl Clause {
    /// The deterministic rule `head :- body` of `group`.
    pub(crate) fn rule(head: Atom, body: Vec<Literal>, group: u32) -> Self {
        Clause {
            heads: vec![HeadChoice {
                prob: None,
                atom: head,
                negated: false,
            }],
            body,
            group,
        }
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, h) in self.heads.iter().enumerate() {
            if i > 0 {
                f.write_str("; ")?;
            }
            match &h.prob {
                None => {}
                Some(Annotation::Number(p)) => write!(f, "{p}::")?,
                Some(Annotation::Flexible(t)) => {
                    write_term(f, t, 999)?;
                    f.write_str("::")?;
                }
            }
            if h.negated {
                f.write_str("\\+")?;
            }
            write!(f, "{}", h.atom)?;
        }
        if !self.body.is_empty() {
            f.write_str(" :- ")?;
            for (i, l) in self.body.iter().enumerate() {
                if i > 0 {
                    f.write_str(", ")?;
                }
                write!(f, "{l}")?;
            }
        }
        f.write_str(".")
    }
}

/// `query(pattern) :- body.`: every instance of `pattern` under a grounding
/// of `body` is a query; `evidence(pattern, v) :- body.` makes each one
/// evidence instead.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct QueryRule {
    pub(crate) pattern: Atom,
    /// The query's sign (`query(\+a)`), or the evidence's value.
    pub(crate) positive: bool,
    pub(crate) evidence: bool,
    pub(crate) body: Vec<Literal>,
}

/// A parsed program.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Program {
    pub(crate) clauses: Vec<Clause>,
    pub(crate) query_rules: Vec<QueryRule>,
    /// `query(atom)` directives with polarity (`query(\+a)` asks for
    /// `P(¬a)`); atoms may be non-ground, then every possible ground instance
    /// is a query.
    pub(crate) queries: Vec<(Atom, bool)>,
    /// `evidence(atom, true|false)` directives; `(atom, polarity)`.
    pub(crate) evidence: Vec<(Atom, bool)>,
    /// The groups of the clauses split from a body whose disjunction comes
    /// after another literal, whose alternatives are not in the order in
    /// which Prolog's resolution tries them (`a(X), (b ; c)` tries `b`
    /// then `c` for each `X`).
    pub(crate) interleaved: Vec<u32>,
}

impl Program {
    /// The program without its queries, `query` facts and rules alike: it
    /// is then ground for its evidence alone, and what no observation
    /// depends on is left out (`--no-queries`).
    pub fn without_queries(mut self) -> Program {
        self.queries.clear();
        self.query_rules.retain(|r| r.evidence);
        self
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.clauses {
            writeln!(f, "{c}")?;
        }
        for (a, v) in &self.evidence {
            writeln!(f, "evidence({a},{v}).")?;
        }
        for (q, pos) in &self.queries {
            if *pos {
                writeln!(f, "query({q}).")?;
            } else {
                writeln!(f, "query(\\+{q}).")?;
            }
        }
        for r in &self.query_rules {
            if r.evidence {
                write!(f, "evidence({},{}) :- ", r.pattern, r.positive)?;
            } else {
                let sign = if r.positive { "" } else { "\\+" };
                write!(f, "query({sign}{}) :- ", r.pattern)?;
            }
            for (i, l) in r.body.iter().enumerate() {
                if i > 0 {
                    f.write_str(", ")?;
                }
                write!(f, "{l}")?;
            }
            writeln!(f, ".")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The list of `items` ending in `tail`.
    fn list(items: Vec<Term>, tail: Term) -> Term {
        let cons: Name = CONS.into();
        items
            .into_iter()
            .rev()
            .fold(tail, |t, x| Compound::term(cons.clone(), vec![x, t]))
    }

    #[test]
    fn a_constant_is_quoted_and_escaped_as_written() {
        for s in [
            "a", "aB_1", "Ab", "<5", "it's", "a\\b", "\\'", "''", "é'x", "x é", "",
        ] {
            let copied = if needs_quotes(s) {
                format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
            } else {
                s.to_string()
            };
            assert_eq!(Term::Const(s.into()).to_string(), copied, "{s}");
        }
        assert_eq!(
            Atom::new("p", vec![Term::Const("it's".into()), Term::Int(3)]).to_string(),
            "p('it\\'s',3)"
        );
    }

    #[test]
    fn floats_print_as_python_writes_them() {
        for (x, s) in [
            (2.0, "2.0"),
            (0.1, "0.1"),
            (1200.0, "1200.0"),
            (0.0001, "0.0001"),
            (0.00001, "1e-05"),
            (1e16, "1e+16"),
            (1234567890123456.0, "1234567890123456.0"),
            (1.5e-7, "1.5e-07"),
            (-2.5, "-2.5"),
            (-0.0, "-0.0"),
            (1e100, "1e+100"),
            (0.30000000000000004, "0.30000000000000004"),
        ] {
            assert_eq!(Term::Float(F64(x)).to_string(), s, "{x}");
        }
    }

    #[test]
    fn terms_print_as_problog_prints_them() {
        let c = |n: &str, args: Vec<Term>| Compound::term(n.into(), args);
        let (a, b, x) = (
            Term::Const("a".into()),
            Term::Const("b".into()),
            Term::Var("X".into()),
        );
        let list = list(vec![a.clone(), b.clone()], x.clone());
        assert_eq!(list.to_string(), "[a, b | X]");
        assert_eq!(Term::Const(NIL.into()).to_string(), "[]");
        let (two, three, four) = (Term::Int(2), Term::Int(3), Term::Int(4));
        let sum = c(
            "+",
            vec![c("*", vec![two.clone(), three.clone()]), four.clone()],
        );
        assert_eq!(sum.to_string(), "2*3+4");
        let prod = c(
            "*",
            vec![two.clone(), c("+", vec![three.clone(), four.clone()])],
        );
        assert_eq!(prod.to_string(), "2*(3+4)");
        let right = c(
            "-",
            vec![two.clone(), c("-", vec![three.clone(), four.clone()])],
        );
        assert_eq!(right.to_string(), "2-(3-4)");
        let left = c("-", vec![c("-", vec![two.clone(), three]), four]);
        assert_eq!(left.to_string(), "2-3-4");
        assert_eq!(c("mod", vec![x.clone(), two]).to_string(), "X mod 2");
        assert_eq!(c(",", vec![a.clone(), b.clone()]).to_string(), "(a, b)");
        assert_eq!(
            Atom::new(
                "p",
                vec![c("-", vec![a.clone(), b.clone()]), c("f", vec![a, b])]
            )
            .to_string(),
            "p(a-b,f(a,b))"
        );
        assert_eq!(c("-", vec![Term::Int(1)]).to_string(), "-(1)");
        assert_eq!(c("-", vec![x]).to_string(), "-X");
    }

    #[test]
    fn conjunctions_and_disjunctions_print_as_problog_prints_them() {
        let c = |n: &str, args: Vec<Term>| Compound::term(n.into(), args);
        let (a, b, d) = (
            Term::Const("a".into()),
            Term::Const("b".into()),
            Term::Const("d".into()),
        );
        let and = |x: &Term, y: &Term| c(",", vec![x.clone(), y.clone()]);
        let or = |x: &Term, y: &Term| c(";", vec![x.clone(), y.clone()]);
        // A conjunction in parentheses of its own, also as an argument or
        // an item, a disjunction in it too; a disjunction bare elsewhere.
        assert_eq!(and(&a, &and(&b, &d)).to_string(), "(a, b, d)");
        assert_eq!(and(&a, &or(&b, &d)).to_string(), "(a, (b; d))");
        assert_eq!(or(&a, &or(&b, &d)).to_string(), "a; b; d");
        assert_eq!(c("f", vec![or(&a, &b)]).to_string(), "f(a; b)");
        assert_eq!(c("f", vec![and(&a, &b)]).to_string(), "f((a, b))");
        let items = list(
            vec![and(
                &Term::Int(1),
                &list(vec![Term::Int(1)], Term::Const(NIL.into())),
            )],
            Term::Const(NIL.into()),
        );
        assert_eq!(items.to_string(), "[(1, [1])]");
        // `\+` and `:-` in functor form, an operator below 1000 in an
        // argument without parentheses.
        assert_eq!(c("\\+", vec![a.clone()]).to_string(), "\\+(a)");
        assert_eq!(c(":-", vec![a.clone(), b.clone()]).to_string(), ":-(a,b)");
        assert_eq!(c("f", vec![c("->", vec![a, b])]).to_string(), "f(a->b)");
    }

    #[test]
    fn the_standard_order_of_terms() {
        let c = |n: &str, args: Vec<Term>| Compound::term(n.into(), args);
        // Numbers as ProbLog sorts them: by their text where their values
        // differ.
        let ordered = [
            Term::Var("X".into()),
            Term::Int(-1),
            Term::Int(-2),
            Term::Float(F64(1.0)),
            Term::Int(1),
            Term::Float(F64(1.5)),
            Term::Int(10),
            Term::Int(2),
            Term::Const("a".into()),
            Term::Const("b".into()),
            c("f", vec![Term::Int(2)]),
            c("g", vec![Term::Int(1)]),
            c("f", vec![Term::Int(1), Term::Int(1)]),
        ];
        for (i, s) in ordered.iter().enumerate() {
            for (j, t) in ordered.iter().enumerate() {
                assert_eq!(term_cmp(s, t), i.cmp(&j), "{s} {t}");
            }
        }
    }

    #[test]
    fn a_long_list_is_compared_and_dropped_without_deep_recursion() {
        let n = 200_000;
        let long = |k: i64| list((0..n).map(Term::Int).collect(), Term::Int(k));
        let (x, y) = (long(0), long(0));
        assert_eq!(x, y);
        assert!(long(0) < long(1));
        assert_eq!(term_cmp(&long(0), &long(1)), Ordering::Less);
        assert_eq!(x.depth(), n as u32);
    }
}
