//! The source language: a Datalog subset of ProbLog.
//!
//! Supported: constants (atoms, quoted strings, integers), variables, `p::fact.`,
//! annotated disjunctions `p1::a; p2::b :- body.`, rules with negation as failure
//! (`\+`), comparison builtins and `is` with integer arithmetic in bodies, and
//! the `query/1` and `evidence/1,2` directives. No function symbols, lists or
//! cuts.

mod lexer;
mod parser;

pub use parser::{ParseError, parse_program};

use std::fmt;
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

/// A ground or non-ground term. Function symbols are not supported.
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Debug, Eq, Hash, PartialOrd, Ord)]
pub enum Term {
    /// A symbolic constant, e.g. `alice`, `'New York'`.
    Const(Name),
    /// An integer constant.
    Int(i64),
    /// A variable, e.g. `X`. `_` is parsed to a fresh unique variable.
    Var(String),
}

// The derived equality, with names told by [`same_name`], with which the
// derived hash agrees.
impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Term::Const(a), Term::Const(b)) => same_name(a, b),
            (Term::Int(a), Term::Int(b)) => a == b,
            (Term::Var(a), Term::Var(b)) => a == b,
            _ => false,
        }
    }
}

impl Term {
    pub(crate) fn is_var(&self) -> bool {
        matches!(self, Term::Var(_))
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Const(s) => write_const(f, s),
            Term::Int(i) => write!(f, "{i}"),
            Term::Var(v) => f.write_str(v),
        }
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
        _ => true,
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
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Args {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
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
        self.args.iter().all(|t| !t.is_var())
    }
    /// The predicate signature `name/arity`.
    pub(crate) fn signature(&self) -> (&str, usize) {
        (&self.name, self.args.len())
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
                write!(f, "{a}")?;
            }
            f.write_str(")")?;
        }
        Ok(())
    }
}

/// A comparison operator usable in rule bodies. Operands are evaluated
/// arithmetic expressions when they contain operators, terms otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum CmpOp {
    /// `=`: equality of bound terms; with exactly one side an unbound
    /// variable it binds that variable.
    Eq,
    /// `\=`
    Ne,
    /// `==`: identity, both sides must be bound.
    Identical,
    /// `\==`
    NotIdentical,
    /// `<`, `=<`, `>`, `>=` on the standard order of terms; also `=:=` and
    /// `=\=` on numbers, mapped to `Eq`/`Ne` after evaluation.
    Lt,
    Le,
    Gt,
    Ge,
}

impl CmpOp {
    pub(crate) fn holds(self, a: &Term, b: &Term) -> bool {
        match self {
            CmpOp::Eq | CmpOp::Identical => a == b,
            CmpOp::Ne | CmpOp::NotIdentical => a != b,
            CmpOp::Lt => term_cmp(a, b).is_lt(),
            CmpOp::Le => term_cmp(a, b).is_le(),
            CmpOp::Gt => term_cmp(a, b).is_gt(),
            CmpOp::Ge => term_cmp(a, b).is_ge(),
        }
    }
}

/// Prolog standard order restricted to our terms: integers before atoms.
fn term_cmp(a: &Term, b: &Term) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (Term::Int(x), Term::Int(y)) => x.cmp(y),
        (Term::Int(_), _) => Ordering::Less,
        (_, Term::Int(_)) => Ordering::Greater,
        (Term::Const(x), Term::Const(y)) => x.cmp(y),
        (Term::Var(x), Term::Var(y)) => x.cmp(y),
        (Term::Var(_), _) => Ordering::Less,
        (_, Term::Var(_)) => Ordering::Greater,
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
        })
    }
}

/// Integer arithmetic operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ArithOp {
    Add,
    Sub,
    Mul,
    /// `//`: integer division, rounding down.
    Div,
    Mod,
}

impl fmt::Display for ArithOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ArithOp::Add => "+",
            ArithOp::Sub => "-",
            ArithOp::Mul => "*",
            ArithOp::Div => "//",
            ArithOp::Mod => "mod",
        })
    }
}

/// An arithmetic expression over integers and variables.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Expr {
    Term(Term),
    Bin(ArithOp, Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

impl Expr {
    /// Variables of the expression.
    pub(crate) fn vars(&self, out: &mut Vec<Term>) {
        match self {
            Expr::Term(t) => {
                if t.is_var() && !out.contains(t) {
                    out.push(t.clone());
                }
            }
            Expr::Bin(_, a, b) => {
                a.vars(out);
                b.vars(out);
            }
            Expr::Neg(a) => a.vars(out),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Term(t) => write!(f, "{t}"),
            Expr::Bin(op, a, b) => write!(f, "({a} {op} {b})"),
            Expr::Neg(a) => write!(f, "-({a})"),
        }
    }
}

/// A body literal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Literal {
    Pos(Atom),
    /// Negation as failure, `\+ atom`.
    Neg(Atom),
    Cmp(CmpOp, Expr, Expr),
    /// `Var is Expr`.
    Is(Term, Expr),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Pos(a) => write!(f, "{a}"),
            Literal::Neg(a) => write!(f, "\\+{a}"),
            Literal::Cmp(op, a, b) => write!(f, "{a}{op}{b}"),
            Literal::Is(v, e) => write!(f, "{v} is {e}"),
        }
    }
}

/// One head alternative of a clause with its probability annotation.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HeadChoice {
    /// `None` for a deterministic head.
    pub(crate) prob: Option<f64>,
    pub(crate) atom: Atom,
    /// A negative head literal `\+atom`: when the body holds, the atom is
    /// made false (ProbLog's "negative head literals").
    pub(crate) negated: bool,
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
            if let Some(p) = h.prob {
                write!(f, "{p}::")?;
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
/// of `body` is a query.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct QueryRule {
    pub(crate) pattern: Atom,
    pub(crate) positive: bool,
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
}

impl Program {
    /// The program without its queries, `query` facts and rules alike: it
    /// is then ground for its evidence alone, and what no observation
    /// depends on is left out (`--no-queries`).
    pub fn without_queries(mut self) -> Program {
        self.queries.clear();
        self.query_rules.clear();
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
            let sign = if r.positive { "" } else { "\\+" };
            write!(f, "query({sign}{}) :- ", r.pattern)?;
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
}
