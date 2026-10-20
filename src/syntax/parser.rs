//! Recursive-descent parser producing a [`Program`].

use super::lexer::{LexError, Lexer, Op, Spanned, Tok, kw};
use super::{
    Args, ArithOp, Atom, Clause, CmpOp, Expr, HeadChoice, Literal, Name, Program, QueryRule, Term,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub(crate) line: usize,
    pub(crate) msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.msg)
    }
}

impl std::error::Error for ParseError {}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError {
            line: e.line,
            msg: e.msg,
        }
    }
}

struct Parser<'a> {
    /// The tokens. Where the lexer meets an error, it stops: the tokens
    /// from there on read as the end of the input, and the parse reports
    /// the error instead of whatever it then failed on. An error before it
    /// in the source is reported instead.
    lexer: Lexer<'a>,
    /// The token at hand, `ahead[cur]`, and the one after it: a token
    /// takes the place of the one two before it, and stays there.
    ahead: [Spanned<'a>; 2],
    cur: bool,
    /// Counter for `_` anonymous variables, made unique per clause.
    anon: usize,
    /// Source clause counter, the `group` of the clauses it produces.
    group: u32,
    /// The literals of the conjunctions being read, the arguments of the
    /// compounds and the heads of a clause, each moved from here into a
    /// vector of its size rather than grown a push at a time and shrunk to
    /// fit. The arguments of an atom are read as terms into `terms`
    /// ([`Lexer::flat_arguments`]), and those of another compound into
    /// `args`.
    lits: Vec<Literal>,
    terms: Vec<Term>,
    args: Vec<PTerm>,
    heads: Vec<HeadChoice>,
}

/// Where a speculative parse started ([`Parser::mark`]).
struct Mark<'a> {
    lexer: (usize, usize, bool),
    ahead: [Spanned<'a>; 2],
    cur: bool,
    /// The lengths of [`Parser::lits`], [`Parser::terms`] and
    /// [`Parser::args`].
    lits: usize,
    terms: usize,
    args: usize,
}

pub fn parse_program(src: &str) -> Result<Program, ParseError> {
    let eof = || Spanned {
        tok: Tok::Eof,
        line: 1,
    };
    let mut p = Parser {
        lexer: Lexer::new(src),
        ahead: [eof(), eof()],
        cur: false,
        anon: 0,
        group: 0,
        lits: Vec::new(),
        terms: Vec::new(),
        args: Vec::new(),
        heads: Vec::new(),
    };
    p.bump();
    p.bump();
    let mut prog = Program::default();
    let mut parsed = Ok(());
    while !p.at(Tok::Eof) {
        p.anon = 0;
        parsed = p.clause(&mut prog);
        if parsed.is_err() {
            break;
        }
        p.group += 1;
    }
    match p.lexer.error() {
        Some(e) => Err(e.into()),
        None => parsed.map(|()| prog),
    }
}

/// Body syntax tree before flattening to disjunctive normal form.
enum Body {
    /// A conjunction of literals, in order; `true` the empty one.
    Lits(Vec<Literal>),
    /// Two or more conjuncts, in order, not all conjunctions of literals.
    And(Vec<Body>),
    Or(Box<Body>, Box<Body>),
}

impl Body {
    /// Alternatives, each a conjunction of literals.
    fn dnf(self) -> Vec<Vec<Literal>> {
        match self {
            Body::Lits(lits) => vec![lits],
            Body::Or(a, b) => {
                let mut v = a.dnf();
                v.extend(b.dnf());
                v
            }
            Body::And(conjuncts) => {
                let mut xs: Vec<Vec<Literal>> = vec![Vec::new()];
                for conjunct in conjuncts {
                    match conjunct {
                        // Literals join every alternative, moved into the
                        // last: one alternative, the common case, copies
                        // nothing.
                        Body::Lits(lits) => {
                            let (last, rest) = xs.split_last_mut().expect("an alternative");
                            for x in rest {
                                x.extend_from_slice(&lits);
                            }
                            last.extend(lits);
                        }
                        other => {
                            let ys = other.dnf();
                            let mut out = Vec::with_capacity(xs.len() * ys.len());
                            for x in xs {
                                for y in &ys {
                                    let mut c = x.clone();
                                    c.extend(y.iter().cloned());
                                    out.push(c);
                                }
                            }
                            xs = out;
                        }
                    }
                }
                xs
            }
        }
    }
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Tok<'a> {
        self.ahead[usize::from(self.cur)].tok
    }
    fn peek2(&self) -> Tok<'a> {
        self.ahead[usize::from(!self.cur)].tok
    }
    /// Whether the token at hand is `tok`, a token that carries nothing
    /// but its kind. Compared as values, tokens take a call per
    /// comparison.
    fn at(&self, tok: Tok) -> bool {
        std::mem::discriminant(&self.peek()) == std::mem::discriminant(&tok)
    }
    /// [`Parser::at`] of the token after it.
    fn at2(&self, tok: Tok) -> bool {
        std::mem::discriminant(&self.peek2()) == std::mem::discriminant(&tok)
    }
    fn line(&self) -> usize {
        self.ahead[usize::from(self.cur)].line
    }
    /// The token at hand, and move past it.
    fn bump(&mut self) -> Tok<'a> {
        let after = self.lexer.token();
        let tok = std::mem::replace(&mut self.ahead[usize::from(self.cur)], after).tok;
        self.cur = !self.cur;
        tok
    }
    /// Where the parse stands, for a speculative parse to come back to.
    fn mark(&self) -> Mark<'a> {
        Mark {
            lexer: self.lexer.mark(),
            ahead: self.ahead,
            cur: self.cur,
            lits: self.lits.len(),
            terms: self.terms.len(),
            args: self.args.len(),
        }
    }
    fn rewind(&mut self, m: Mark<'a>) {
        self.lexer.rewind(m.lexer);
        self.ahead = m.ahead;
        self.cur = m.cur;
        // A parse that failed left its literals and arguments.
        self.lits.truncate(m.lits);
        self.terms.truncate(m.terms);
        self.args.truncate(m.args);
    }
    fn err<T>(&self, msg: impl Into<String>) -> Result<T, ParseError> {
        Err(ParseError {
            line: self.line(),
            msg: msg.into(),
        })
    }
    /// Move past `tok`, a token that carries nothing but its kind
    /// ([`Parser::at`]).
    fn expect(&mut self, tok: Tok) -> Result<(), ParseError> {
        if self.at(tok) {
            self.bump();
            Ok(())
        } else {
            self.err(format!(
                "expected {}, found {}",
                self.lexer.show(tok),
                self.lexer.show(self.peek())
            ))
        }
    }
    fn is_op(&self, op: Op) -> bool {
        self.peek() == Tok::Op(op)
    }

    fn clause(&mut self, prog: &mut Program) -> Result<(), ParseError> {
        if self.at(Tok::Neck) {
            self.bump();
            let t = self.pterm()?;
            self.expect(Tok::Dot)?;
            let Some((name, args)) = t.into_compound() else {
                return self.err("unsupported directive");
            };
            let start = self.args.len();
            self.args.extend(args);
            return self.directive(&name, start, prog);
        }
        if let Tok::Name(id @ (kw::QUERY | kw::EVIDENCE)) = self.peek()
            && self.at2(Tok::LParen)
        {
            // An atom inside: the arguments read as tokens, past the
            // lexer's reading of an atom's, onto `args`.
            self.bump();
            self.bump();
            let start = self.args.len();
            self.argument_list()?;
            if self.at(Tok::Neck) {
                // `query(p(X)) :- body.`: a query rule.
                self.bump();
                let body = self.body()?;
                self.expect(Tok::Dot)?;
                if id != kw::QUERY || self.args.len() - start != 1 {
                    return self.err("only query/1 may have a body");
                }
                let t = self.args.pop().expect("an argument");
                let (pattern, positive) = self.signed_atom(t, "query/1")?;
                for alt in body.dnf() {
                    prog.query_rules.push(QueryRule {
                        pattern: pattern.clone(),
                        positive,
                        body: alt,
                    });
                }
                return Ok(());
            }
            self.expect(Tok::Dot)?;
            return self.directive(
                if id == kw::QUERY { "query" } else { "evidence" },
                start,
                prog,
            );
        }
        let heads = self.heads()?;
        let body = if self.at(Tok::Neck) {
            self.bump();
            self.body()?
        } else {
            Body::Lits(Vec::new())
        };
        self.expect(Tok::Dot)?;
        let group = self.group;
        match body {
            // A conjunction of literals, the common body, is its one
            // alternative.
            Body::Lits(body) => prog.clauses.push(Clause { heads, body, group }),
            // The heads go to the last alternative, and copies to the
            // others.
            body => {
                let alts = body.dnf();
                let last = alts.len() - 1;
                let mut heads = heads;
                for (k, body) in alts.into_iter().enumerate() {
                    let heads = if k < last {
                        heads.clone()
                    } else {
                        std::mem::take(&mut heads)
                    };
                    prog.clauses.push(Clause { heads, body, group });
                }
            }
        }
        Ok(())
    }

    /// The directive `name` over the arguments from `start` on in
    /// [`Parser::args`], taken from there.
    fn directive(
        &mut self,
        name: &str,
        start: usize,
        prog: &mut Program,
    ) -> Result<(), ParseError> {
        let arity = self.args.len() - start;
        match (name, arity) {
            ("query", 1) => {
                let t = self.args.pop().expect("an argument");
                let (q, pos) = self.signed_atom(t, "query/1")?;
                prog.queries.push((q, pos));
                Ok(())
            }
            ("evidence", 1) => {
                let t = self.args.pop().expect("an argument");
                let (a, pol) = self.signed_atom(t, "evidence")?;
                prog.evidence.push((a, pol));
                Ok(())
            }
            ("evidence", 2) => {
                let value = self.args.pop().expect("an argument");
                let t = self.args.pop().expect("an argument");
                let (a, pol) = self.signed_atom(t, "evidence")?;
                let v = match value {
                    PTerm::Const(s) if &*s == "true" => true,
                    PTerm::Const(s) if &*s == "false" => false,
                    _ => return self.err("evidence/2 expects true or false"),
                };
                prog.evidence.push((a, pol == v));
                Ok(())
            }
            _ => self.err(format!("unsupported directive {name}/{arity}")),
        }
    }

    /// An atom or `\+ atom`, as `(atom, positive)`.
    fn signed_atom(&self, t: PTerm, what: &str) -> Result<(Atom, bool), ParseError> {
        let negated = matches!(t.functor(), Some((n, 1)) if &**n == "\\+" || &**n == "not");
        let (t, pos) = if negated {
            let (_, mut args) = t.into_compound().expect("a compound");
            (args.pop().unwrap(), false)
        } else {
            (t, true)
        };
        t.into_atom().map(|a| (a, pos)).ok_or_else(|| ParseError {
            line: self.line(),
            msg: format!("{what} expects an atom"),
        })
    }

    fn heads(&mut self) -> Result<Vec<HeadChoice>, ParseError> {
        let start = self.heads.len();
        let head = self.head()?;
        self.heads.push(head);
        while self.at(Tok::Semi) {
            self.bump();
            let head = self.head()?;
            self.heads.push(head);
        }
        Ok(self.heads.split_off(start))
    }

    fn head(&mut self) -> Result<HeadChoice, ParseError> {
        // `P::atom` where P is a number or a fraction `a/b`.
        let prob = match (self.peek(), self.peek2()) {
            (Tok::Float(p), Tok::DoubleColon) => {
                self.bump();
                self.bump();
                Some(p)
            }
            (Tok::Int(p), Tok::DoubleColon) => {
                self.bump();
                self.bump();
                Some(p as f64)
            }
            (Tok::Int(a), Tok::Op(Op::Slash)) => {
                self.bump();
                self.bump();
                let b = match self.bump() {
                    Tok::Int(b) if b != 0 => b,
                    Tok::Float(b) if b != 0.0 => return self.fraction_tail(a as f64, b),
                    _ => return self.err("expected a nonzero denominator"),
                };
                self.expect(Tok::DoubleColon)?;
                Some(a as f64 / b as f64)
            }
            (Tok::Name(kw::T), Tok::LParen) => {
                let save = self.mark();
                let _ = self.pterm()?;
                if self.at(Tok::DoubleColon) {
                    return self.err("learnable probabilities t(...) are not supported");
                }
                self.rewind(save);
                None
            }
            _ => None,
        };
        if let Some(p) = prob
            && !(0.0..=1.0 + 1e-12).contains(&p)
        {
            return self.err(format!("probability {p} out of range"));
        }
        let negated = if self.at(Tok::Naf) {
            self.bump();
            true
        } else {
            false
        };
        let atom = self.atom()?;
        Ok(HeadChoice {
            prob,
            atom,
            negated,
        })
    }

    fn fraction_tail(&mut self, a: f64, b: f64) -> Result<HeadChoice, ParseError> {
        self.expect(Tok::DoubleColon)?;
        let atom = self.atom()?;
        Ok(HeadChoice {
            prob: Some(a / b),
            atom,
            negated: false,
        })
    }

    // body := conj (';' conj)*
    fn body(&mut self) -> Result<Body, ParseError> {
        let mut b = self.conj()?;
        while self.at(Tok::Semi) {
            self.bump();
            let rhs = self.conj()?;
            b = Body::Or(Box::new(b), Box::new(rhs));
        }
        Ok(b)
    }

    // conj := unit (',' unit)*
    fn conj(&mut self) -> Result<Body, ParseError> {
        // Literals wait in `lits`, and those before a unit that is not one,
        // if any, and those after the last such unit are a conjunct each.
        let start = self.lits.len();
        let mut conjuncts = Vec::new();
        loop {
            if let Some(unit) = self.unit()? {
                if self.lits.len() > start {
                    conjuncts.push(Body::Lits(self.lits.split_off(start)));
                }
                conjuncts.push(unit);
            }
            if !self.at(Tok::Comma) {
                break;
            }
            self.bump();
        }
        // A conjunction of literals alone, the common body, is returned
        // without the vector of conjuncts.
        if conjuncts.is_empty() {
            return Ok(Body::Lits(self.lits.split_off(start)));
        }
        if self.lits.len() > start {
            conjuncts.push(Body::Lits(self.lits.split_off(start)));
        }
        Ok(if conjuncts.len() == 1 {
            conjuncts.pop().expect("a conjunct")
        } else {
            Body::And(conjuncts)
        })
    }

    // unit := '(' body ')' | literal
    /// A parenthesised body, or `None` for a literal, pushed on
    /// [`Parser::lits`].
    fn unit(&mut self) -> Result<Option<Body>, ParseError> {
        if self.at(Tok::LParen) {
            // Either a parenthesised body or a parenthesised expression in a
            // comparison. Try the body reading first; a comparison operator
            // after the closing paren means it was an expression.
            let save = self.mark();
            self.bump();
            if let Ok(b) = self.body()
                && self.at(Tok::RParen)
            {
                self.bump();
                if !matches!(self.peek(), Tok::Op(_) | Tok::Name(kw::IS)) {
                    return Ok(Some(b));
                }
            }
            self.rewind(save);
        }
        let l = self.literal()?;
        self.lits.push(l);
        Ok(None)
    }

    fn literal(&mut self) -> Result<Literal, ParseError> {
        if matches!(self.peek(), Tok::Naf | Tok::Name(kw::NOT)) {
            self.bump();
            let paren = self.at(Tok::LParen);
            if paren {
                self.bump();
            }
            let a = self.atom()?;
            if paren {
                if self.at(Tok::Comma) || self.at(Tok::Semi) {
                    return self.err(
                        "negation of a compound goal is not supported; define a predicate for it",
                    );
                }
                self.expect(Tok::RParen)?;
            }
            return Ok(Literal::Neg(a));
        }
        if self.peek() == Tok::Name(kw::TRUE) && !self.at2(Tok::LParen) {
            self.bump();
            return Ok(Literal::Cmp(
                CmpOp::Eq,
                Expr::Term(Term::Int(0)),
                Expr::Term(Term::Int(0)),
            ));
        }
        if matches!(self.peek(), Tok::Name(kw::FAIL | kw::FALSE)) && !self.at2(Tok::LParen) {
            self.bump();
            return Ok(Literal::Cmp(
                CmpOp::Ne,
                Expr::Term(Term::Int(0)),
                Expr::Term(Term::Int(0)),
            ));
        }
        // `Expr op Expr`, `Var is Expr`, or an atom.
        let lhs = if let Tok::Name(_) = self.peek() {
            let t = self.pterm()?;
            let followed_by_op = matches!(self.peek(), Tok::Op(_) | Tok::Name(kw::IS | kw::MOD));
            if !followed_by_op {
                let line = self.line();
                return t.into_atom().map(Literal::Pos).ok_or_else(|| ParseError {
                    line,
                    msg: "expected a literal".into(),
                });
            }
            let line = self.line();
            let first = t.into_term().map(Expr::Term).ok_or_else(|| ParseError {
                line,
                msg: "function symbols are not supported in expressions".into(),
            })?;
            self.arith_low(Some(first))?
        } else {
            self.arith_low(None)?
        };
        if self.peek() == Tok::Name(kw::IS) {
            self.bump();
            let rhs = self.arith_low(None)?;
            let v = match lhs {
                Expr::Term(t) => t,
                _ => return self.err("left side of `is` must be a variable or number"),
            };
            return Ok(Literal::Is(v, rhs));
        }
        if let Tok::Op(op) = self.peek() {
            let op = match op {
                Op::Eq => CmpOp::Eq,
                Op::Ne => CmpOp::Ne,
                Op::Identical => CmpOp::Identical,
                Op::NotIdentical => CmpOp::NotIdentical,
                Op::Lt => CmpOp::Lt,
                Op::Le => CmpOp::Le,
                Op::Gt => CmpOp::Gt,
                Op::Ge => CmpOp::Ge,
                Op::Plus | Op::Minus | Op::Star | Op::Slash => {
                    return self.err(format!("unexpected operator {}", op.text()));
                }
            };
            self.bump();
            let rhs = self.arith_low(None)?;
            return Ok(Literal::Cmp(op, lhs, rhs));
        }
        match lhs {
            Expr::Term(t) => self.err(format!("expected a literal, found {t}")),
            _ => self.err("expected a literal, found an arithmetic expression"),
        }
    }

    /// Parse an atom in body position (for `\+`).
    fn atom(&mut self) -> Result<Atom, ParseError> {
        let t = self.pterm()?;
        let line = self.line();
        t.into_atom().ok_or_else(|| ParseError {
            line,
            msg: "expected an atom".into(),
        })
    }

    fn arith_low(&mut self, first: Option<Expr>) -> Result<Expr, ParseError> {
        let mut e = self.arith_high(first)?;
        loop {
            let op = if self.is_op(Op::Plus) {
                ArithOp::Add
            } else if self.is_op(Op::Minus) {
                ArithOp::Sub
            } else {
                break;
            };
            self.bump();
            let rhs = self.arith_high(None)?;
            e = Expr::Bin(op, Box::new(e), Box::new(rhs));
        }
        Ok(e)
    }

    fn arith_high(&mut self, first: Option<Expr>) -> Result<Expr, ParseError> {
        let mut e = match first {
            Some(e) => e,
            None => self.arith_unary()?,
        };
        loop {
            let op = if self.is_op(Op::Star) {
                ArithOp::Mul
            } else if self.is_op(Op::Slash) {
                // `//` lexes as two `/` tokens. ProbLog's `/` divides in
                // floating point (`X is 4 / 2` binds 2.0, which is not 2).
                if self.peek2() != Tok::Op(Op::Slash) {
                    return self.err("`/` divides in floating point, which tddlog does not; `//` divides integers");
                }
                self.bump();
                ArithOp::Div
            } else if self.peek() == Tok::Name(kw::MOD) {
                ArithOp::Mod
            } else {
                break;
            };
            self.bump();
            let rhs = self.arith_unary()?;
            e = Expr::Bin(op, Box::new(e), Box::new(rhs));
        }
        Ok(e)
    }

    fn arith_unary(&mut self) -> Result<Expr, ParseError> {
        if self.is_op(Op::Minus) {
            self.bump();
            let e = self.arith_unary()?;
            return Ok(match e {
                Expr::Term(Term::Int(i)) => Expr::Term(Term::Int(-i)),
                other => Expr::Neg(Box::new(other)),
            });
        }
        if self.is_op(Op::Plus) {
            self.bump();
            return self.arith_unary();
        }
        match self.peek() {
            Tok::LParen => {
                self.bump();
                let e = self.arith_low(None)?;
                self.expect(Tok::RParen)?;
                Ok(e)
            }
            Tok::Int(i) => {
                self.bump();
                Ok(Expr::Term(Term::Int(i)))
            }
            Tok::Float(x) => self.err(format!(
                "floating point numbers are not supported in expressions ({x})"
            )),
            Tok::Var(_) | Tok::Name(_) => {
                let t = self.pterm()?;
                let line = self.line();
                t.into_term().map(Expr::Term).ok_or_else(|| ParseError {
                    line,
                    msg: "function symbols are not supported in expressions".into(),
                })
            }
            other => self.err(format!("unexpected {}", self.lexer.show(other))),
        }
    }

    /// A possibly compound term.
    fn pterm(&mut self) -> Result<PTerm, ParseError> {
        if let Tok::Name(id) = self.peek()
            && self.at2(Tok::LParen)
        {
            // The arguments of an atom are read in the lexer, past the `(`
            // at hand: the two tokens moved past here are the ones after
            // its `)`. Of another compound, here, as tokens.
            let n = self.lexer.name(id).clone();
            let args = self.lexer.flat_arguments(&mut self.terms, &mut self.anon);
            self.bump();
            self.bump();
            return match args {
                Some(args) => Ok(PTerm::Atom(n, args)),
                None => self.arguments(n),
            };
        }
        if let Some(t) = self.flat_term() {
            return Ok(t.into());
        }
        match self.bump() {
            Tok::Float(x) => self.err(format!("floating point constants are not supported ({x})")),
            Tok::Op(Op::Minus) => {
                // `-` and an integer is one, which `flat_term` takes.
                let other = self.bump();
                self.err(format!("unexpected {} after `-`", self.lexer.show(other)))
            }
            Tok::Naf => {
                let start = self.args.len();
                let inner = self.pterm()?;
                self.args.push(inner);
                Ok(self.compound("\\+".into(), start))
            }
            Tok::LParen => {
                let t = self.pterm()?;
                self.expect(Tok::RParen)?;
                Ok(t)
            }
            other => self.err(format!("unexpected {}", self.lexer.show(other))),
        }
    }

    /// The flat term at hand, if it is one, taken: an integer, a negative
    /// one, a variable (`_` a fresh one), or a name not followed by `(`.
    /// Inlined, as it is called per argument.
    #[inline(always)]
    fn flat_term(&mut self) -> Option<Term> {
        let t = match (self.peek(), self.peek2()) {
            (Tok::Int(i), _) => Term::Int(i),
            (Tok::Var("_"), _) => {
                self.anon += 1;
                Term::Var(format!("_G{}", self.anon))
            }
            (Tok::Var(v), _) => Term::Var(v.to_owned()),
            (Tok::Name(id), _) if !self.at2(Tok::LParen) => {
                Term::Const(self.lexer.name(id).clone())
            }
            (Tok::Op(Op::Minus), Tok::Int(i)) => {
                self.bump();
                Term::Int(-i)
            }
            _ => return None,
        };
        self.bump();
        Some(t)
    }

    /// The compound of `name`, its `(` read, over the arguments up to its
    /// `)`: allowing `\+ x` inside `evidence(..)`. The arguments of an
    /// atom are read in the lexer ([`Lexer::flat_arguments`]), those of
    /// any other compound here.
    fn arguments(&mut self, name: Name) -> Result<PTerm, ParseError> {
        let start = self.args.len();
        self.argument_list()?;
        Ok(self.compound(name, start))
    }

    /// The arguments of a compound, its `(` read, up to and past its `)`,
    /// onto [`Parser::args`].
    fn argument_list(&mut self) -> Result<(), ParseError> {
        let arg = self.pterm()?;
        self.args.push(arg);
        while self.at(Tok::Comma) {
            self.bump();
            let arg = self.pterm()?;
            self.args.push(arg);
        }
        self.expect(Tok::RParen)
    }

    /// The compound of `name` over the arguments from `start` on in
    /// [`Parser::args`], taken from there.
    fn compound(&mut self, name: Name, start: usize) -> PTerm {
        if self.args[start..].iter().all(PTerm::is_flat) {
            PTerm::Atom(
                name,
                self.args
                    .drain(start..)
                    .map(|t| t.into_term().expect("a flat term"))
                    .collect::<Args>(),
            )
        } else {
            PTerm::Compound(name, self.args.split_off(start))
        }
    }
}

/// Parser-internal term that still allows compounds, so `query(p(X))` parses.
#[derive(Clone, Debug)]
enum PTerm {
    Const(Name),
    Int(i64),
    Var(String),
    /// A compound with flat arguments, the atom's own.
    Atom(Name, Args),
    /// A compound with a compound among its arguments.
    Compound(Name, Vec<PTerm>),
}

impl From<Term> for PTerm {
    fn from(t: Term) -> PTerm {
        match t {
            Term::Const(c) => PTerm::Const(c),
            Term::Int(i) => PTerm::Int(i),
            Term::Var(v) => PTerm::Var(v),
        }
    }
}

impl PTerm {
    fn is_flat(&self) -> bool {
        matches!(self, PTerm::Const(_) | PTerm::Int(_) | PTerm::Var(_))
    }
    fn into_term(self) -> Option<Term> {
        match self {
            PTerm::Const(c) => Some(Term::Const(c)),
            PTerm::Int(i) => Some(Term::Int(i)),
            PTerm::Var(v) => Some(Term::Var(v)),
            PTerm::Atom(..) | PTerm::Compound(..) => None,
        }
    }
    /// An atom: a constant or a compound with flat arguments.
    fn into_atom(self) -> Option<Atom> {
        match self {
            PTerm::Const(c) => Some(Atom {
                name: c,
                args: Args::default(),
            }),
            PTerm::Atom(name, args) => Some(Atom { name, args }),
            _ => None,
        }
    }
    /// A compound's name and arity.
    fn functor(&self) -> Option<(&Name, usize)> {
        match self {
            PTerm::Atom(n, args) => Some((n, args.len())),
            PTerm::Compound(n, args) => Some((n, args.len())),
            _ => None,
        }
    }
    /// A compound's name and arguments.
    fn into_compound(self) -> Option<(Name, Vec<PTerm>)> {
        match self {
            PTerm::Atom(n, args) => {
                Some((n, Vec::from(args).into_iter().map(PTerm::from).collect()))
            }
            PTerm::Compound(n, args) => Some((n, args)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_smokers() {
        let src = r#"
            % smokers
            0.3::stress(X) :- person(X).
            0.2::influences(X,Y) :- person(X), person(Y).
            person(a). person(b).
            friend(a,b).
            smokes(X) :- stress(X).
            smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).
            0.4::asthma(X) :- smokes(X).
            query(smokes(a)).
            evidence(smokes(b), true).
            evidence(\+asthma(a)).
        "#;
        let p = parse_program(src).unwrap();
        assert_eq!(p.clauses.len(), 8);
        assert_eq!(p.queries.len(), 1);
        assert_eq!(p.evidence.len(), 2);
        assert!(!p.evidence[1].1);
        assert_eq!(p.clauses[0].heads[0].prob, Some(0.3));
        assert_eq!(p.clauses[6].body.len(), 3);
        let text = p.to_string();
        let p2 = parse_program(&text).unwrap();
        assert_eq!(p.to_string(), p2.to_string());
    }

    #[test]
    fn parses_annotated_disjunction_and_comparisons() {
        let src = "0.5::a(1); 0.3::b(2) :- c(X), X \\= 3, \\+ d(X), X < 10, X == 2.\n";
        let p = parse_program(src).unwrap();
        let c = &p.clauses[0];
        assert_eq!(c.heads.len(), 2);
        assert_eq!(c.body.len(), 5);
        assert!(matches!(c.body[1], Literal::Cmp(CmpOp::Ne, _, _)));
        assert!(matches!(c.body[2], Literal::Neg(_)));
        assert!(matches!(c.body[4], Literal::Cmp(CmpOp::Identical, _, _)));
    }

    #[test]
    fn body_disjunction_and_groups() {
        let p =
            parse_program("0.5::c :- (d, e); f.\nany :- p(1); p(2).\nnested :- ((\\+ c, c), a).\n")
                .unwrap();
        assert_eq!(p.clauses.len(), 5);
        assert_eq!(p.clauses[0].group, p.clauses[1].group);
        assert_ne!(p.clauses[1].group, p.clauses[2].group);
        assert_eq!(p.clauses[0].body.len(), 2);
        assert_eq!(p.clauses[1].body.len(), 1);
        assert_eq!(p.clauses[4].body.len(), 3);
    }

    #[test]
    fn arithmetic_and_alt_neck() {
        let p =
            parse_program("1/3::a <- true.\np(X) :- q(Y), X is Y*2+1, X > -3, X =:= 7.\n").unwrap();
        assert!((p.clauses[0].heads[0].prob.unwrap() - 1.0 / 3.0).abs() < 1e-12);
        let c = &p.clauses[1];
        assert_eq!(c.body.len(), 4);
        assert_eq!(c.body[1].to_string(), "X is ((Y * 2) + 1)");
        assert_eq!(c.body[2].to_string(), "X>-3");
    }

    #[test]
    fn refuses_float_division() {
        // ProbLog's `/` binds 2.0 for 4 / 2, which is not 2.
        let e = parse_program("p(X) :- q(Y), X is Y / 2.\n").unwrap_err();
        assert!(e.msg.contains("`//`"), "{e}");
        assert!(parse_program("p(X) :- q(Y), X is Y // 2.\n").is_ok());
    }

    #[test]
    fn negative_heads_and_queries() {
        let p = parse_program("0.5::\\+a :- c.\nquery(\\+a).\nquery(not(b)).\n").unwrap();
        assert!(p.clauses[0].heads[0].negated);
        assert_eq!(
            p.queries,
            vec![
                (Atom::new("a", vec![]), false),
                (Atom::new("b", vec![]), false)
            ]
        );
    }

    #[test]
    fn query_rules_and_not_prefix() {
        let p = parse_program("query(p(X)) :- a(X), not b(X).\n").unwrap();
        assert_eq!(p.query_rules.len(), 1);
        assert_eq!(p.query_rules[0].pattern.to_string(), "p(X)");
        assert_eq!(p.query_rules[0].body.len(), 2);
        assert!(matches!(p.query_rules[0].body[1], Literal::Neg(_)));
    }

    #[test]
    fn rejects_function_symbols() {
        assert!(parse_program("p(f(X)).").is_err());
    }

    #[test]
    fn quoted_atoms_and_anonymous_vars() {
        let p = parse_program("p('Hello World', _, _, \"x\").").unwrap();
        let a = &p.clauses[0].heads[0].atom;
        assert_eq!(a.args[0], Term::Const("Hello World".into()));
        assert_ne!(a.args[1], a.args[2]);
        assert_eq!(a.to_string(), "p('Hello World',_G1,_G2,x)");
    }

    #[test]
    fn flat_arguments_before_a_compound_one() {
        // The flat arguments read before a compound one stay its first,
        // and anonymous variables are numbered in the order they are read.
        let err = parse_program(":- foo(a, -1, _, g(_), _).")
            .unwrap_err()
            .to_string();
        assert_eq!(err, "line 1: unsupported directive foo/5");
        let p = parse_program("query(p(_, -2, b)).\nevidence(\\+ q(_, 0, _), true).\n").unwrap();
        assert_eq!(p.queries[0].0.to_string(), "p(_G1,-2,b)");
        assert_eq!(p.evidence[0].0.to_string(), "q(_G1,0,_G2)");
        assert!(!p.evidence[0].1);
    }

    #[test]
    fn atom_arguments_read_in_the_lexer() {
        // The arguments the lexer reads: names, quoted ones, variables,
        // `_`, integers and negative ones, with blanks, comments and lines
        // between them, in heads, bodies and queries.
        let src = "p(a, 'B c', X, _, 3, - 4, -5, _) :- q(_, a), \\+ r(b,\n  % c\n  X).\nquery(s(1)).\n0.5::t(x); 0.5::u(y, z).\n";
        let p = parse_program(src).unwrap();
        let c = &p.clauses[0];
        assert_eq!(c.heads[0].atom.to_string(), "p(a,'B c',X,_G1,3,-4,-5,_G2)");
        assert_eq!(c.body[0].to_string(), "q(_G3,a)");
        assert_eq!(c.body[1].to_string(), "\\+r(b,X)");
        assert_eq!(p.queries[0].0.to_string(), "s(1)");
        assert_eq!(p.clauses[1].heads.len(), 2);
        assert_eq!(p.clauses[1].heads[1].atom.to_string(), "u(y,z)");
        // The lines counted past them.
        let err = |src: &str| parse_program(src).unwrap_err().to_string();
        assert_eq!(
            err("p(a,\n b).\nq(c) d.\n"),
            "line 3: expected `.`, found name `d`"
        );
    }

    #[test]
    fn other_arguments_read_by_the_parser() {
        // Where an argument is not flat or a token not of an atom, the
        // parser reads the tokens the lexer gave back, with its errors on
        // their lines.
        let err = |src: &str| parse_program(src).unwrap_err().to_string();
        assert_eq!(err("p(a, f(b))."), "line 1: expected an atom");
        assert_eq!(
            err("p(a,\n\n b c)."),
            "line 3: expected `)`, found name `c`"
        );
        assert_eq!(
            err("p(a, 1.5)."),
            "line 1: floating point constants are not supported (1.5)"
        );
        assert_eq!(err("p(a, - b)."), "line 1: unexpected name `b` after `-`");
        assert_eq!(err("p(a,\n $)."), "line 2: unexpected character `$`");
        assert_eq!(err("p(a, b"), "line 1: expected `)`, found end of input");
        assert_eq!(err("foo(_) :- bar(_, g(_))."), "line 1: expected a literal");
        // A compound argument, `\+` and an atom named `not` inside query/1
        // and evidence/2, read with the anonymous variables in order.
        let p = parse_program("evidence(\\+ e(_, 1), true).\nquery(not(q(_))).\n").unwrap();
        assert_eq!(p.evidence[0].0.to_string(), "e(_G1,1)");
        assert!(!p.evidence[0].1);
        assert_eq!(p.queries[0].0.to_string(), "q(_G1)");
        assert!(!p.queries[0].1);
    }

    #[test]
    fn reports_the_first_error() {
        let err = |src: &str| parse_program(src).unwrap_err().to_string();
        // The lexer's error where the parse reaches it, also at the end of
        // the input and inside a parenthesised body read speculatively.
        assert_eq!(err("p :- q.\nq $ r.\n"), "line 2: unexpected character `$`");
        assert_eq!(err("p.\n'open"), "line 2: unterminated quoted atom");
        assert_eq!(err("p :- (q, $)."), "line 1: unexpected character `$`");
        assert_eq!(
            err("p :- q([a|T])."),
            "line 1: lists are not supported (`[`)"
        );
        // A parse error before it.
        assert_eq!(
            err("a :- b c.\nd $.\n"),
            "line 1: expected `.`, found name `c`"
        );
        // Speculation reads past the tokens it comes back to.
        let p = parse_program("a :- (b ; c), d.\nx(X) :- X is (1+2)*3.\n").unwrap();
        assert_eq!(p.clauses.len(), 3);
    }
}
