//! Recursive-descent parser producing a [`Program`], with an operator
//! precedence parser for terms.

use super::lexer::{LexError, Lexer, Op, Spanned, Tok, kw};
use super::{
    Annotation, Args, Assoc, Atom, CONS, Clause, CmpOp, Compound, F64, Findall, HeadChoice,
    Literal, NIL, Name, Program, QueryRule, Term, infix_op, prefix_op,
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
    /// The aggregates `f<V>` read in the clause at hand
    /// ([`Parser::aggregate`]).
    aggregates: usize,
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
    /// The line of the last token [`Parser::expect`] moved past: at a
    /// clause's start, the end of the clause before it.
    expected: usize,
    /// The `consult/1` directives read, where the program is read for them
    /// ([`consults`]); `None` where they are refused ([`parse_program`]).
    consults: Option<Vec<Consult>>,
}

/// A directive `:- consult(File).`: the file it names, and its first and
/// last lines, on which it stands alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Consult {
    pub file: String,
    pub lines: (usize, usize),
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

/// The ProbLog program `src`. Another input, an LPAD among them, is read
/// as a program first ([`crate::read`]). A `consult/1` directive is
/// refused: the files it names are read in its place first
/// ([`crate::consult`]).
pub fn parse_program(src: &str) -> Result<Program, ParseError> {
    parse(src, None).map(|(program, _)| program)
}

/// The `consult/1` directives of the ProbLog program `src`, in order.
pub fn consults(src: &str) -> Result<Vec<Consult>, ParseError> {
    parse(src, Some(Vec::new())).map(|(_, consults)| consults.expect("read"))
}

fn parse(
    src: &str,
    consults: Option<Vec<Consult>>,
) -> Result<(Program, Option<Vec<Consult>>), ParseError> {
    let eof = || Spanned {
        tok: Tok::Eof,
        line: 1,
    };
    let mut p = Parser {
        lexer: Lexer::new(src),
        ahead: [eof(), eof()],
        cur: false,
        anon: 0,
        aggregates: 0,
        group: 0,
        lits: Vec::new(),
        terms: Vec::new(),
        args: Vec::new(),
        heads: Vec::new(),
        expected: 0,
        consults,
    };
    p.bump();
    p.bump();
    let mut prog = Program::default();
    let mut parsed = Ok(());
    while !p.at(Tok::Eof) {
        p.anon = 0;
        p.aggregates = 0;
        let line = p.line();
        parsed = p.clause(&mut prog);
        if parsed.is_ok() && p.aggregates != 0 {
            parsed = Err(ParseError {
                line,
                msg: AGGREGATE_ONLY.into(),
            });
        }
        if parsed.is_err() {
            break;
        }
        p.group += 1;
    }
    match p.lexer.error() {
        Some(e) => Err(e.into()),
        None => parsed.map(|()| (prog, p.consults)),
    }
}

/// Body syntax tree before flattening to disjunctive normal form.
enum Body {
    /// A conjunction of literals, in order; `true` the empty one.
    Lits(Vec<Literal>),
    /// Two or more conjuncts, in order.
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

    /// Whether a disjunction comes after another literal of a conjunction,
    /// where the alternatives [`Body::dnf`] lists are not in the order in
    /// which resolution tries them ([`Program::interleaved`]).
    fn interleaved(&self) -> bool {
        match self {
            Body::Lits(_) => false,
            Body::Or(a, b) => a.interleaved() || b.interleaved(),
            Body::And(cs) => cs.iter().enumerate().any(|(k, c)| {
                c.interleaved()
                    || (!matches!(c, Body::Lits(_))
                        && cs[..k]
                            .iter()
                            .any(|d| !matches!(d, Body::Lits(l) if l.is_empty())))
            }),
        }
    }
}

/// The name of the term `'$aggregate'(f, V)` the parser reads `f<V>` as,
/// until the clause whose head it ends is rewritten ([`Parser::aggregate`]).
const AGGREGATE: &str = "$aggregate";

/// The error of an aggregate `f<V>` read elsewhere.
const AGGREGATE_ONLY: &str = "an aggregate f<V> is read only as the last argument of a clause's \
                              one head, not annotated";

/// `t` with its variables renamed apart from those of the clause it is
/// read in: `X` is `X$`.
fn renamed(t: &Term) -> Term {
    match t {
        Term::Var(v) => Term::Var(format!("{v}$")),
        Term::Compound(c) if !t.is_ground() => {
            Compound::term(c.name.clone(), c.args.iter().map(renamed).collect())
        }
        _ => t.clone(),
    }
}

/// `l` with its variables renamed apart ([`renamed`]).
fn renamed_literal(l: &Literal) -> Literal {
    let atom = |a: &Atom| {
        Atom::new(
            a.name.clone(),
            a.args.iter().map(renamed).collect::<Vec<_>>(),
        )
    };
    match l {
        Literal::Pos(a) => Literal::Pos(atom(a)),
        Literal::Neg(a) => Literal::Neg(atom(a)),
        Literal::Cmp(op, a, b) => Literal::Cmp(*op, renamed(a), renamed(b)),
        Literal::Is(a, b) => Literal::Is(renamed(a), renamed(b)),
        Literal::Findall(fa) => Literal::Findall(Box::new(Findall {
            template: renamed(&fa.template),
            goal: fa
                .goal
                .iter()
                .map(|alt| alt.iter().map(renamed_literal).collect())
                .collect(),
            result: renamed(&fa.result),
            interleaved: fa.interleaved,
            all: fa.all,
        })),
        Literal::Builtin(..) | Literal::Collect(_) => unreachable!("made by the grounder"),
    }
}

/// Whether a literal named `name` of `arity` arguments is read other than
/// as an atom: a comparison, `is`, a negation, an operator, `findall/3`,
/// `all/3`, `forall/2`. A scope `S:A` is an atom of the predicate `:/2`,
/// as ProbLog reads it without its library(scope).
fn special(name: &str, arity: usize) -> bool {
    match arity {
        1 => name == "not" || prefix_op(name).is_some(),
        2 => (name != ":" && infix_op(name).is_some()) || name == "forall",
        3 => name == "findall" || name == "all",
        _ => false,
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
            self.expected = self.line();
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

    fn clause(&mut self, prog: &mut Program) -> Result<(), ParseError> {
        if self.at(Tok::Neck) {
            let (first, before) = (self.line(), self.expected);
            self.bump();
            let t = self.term(1200)?;
            let last = self.line();
            self.expect(Tok::Dot)?;
            let Some((name, args)) = t.into_compound() else {
                return self.err("unsupported directive");
            };
            if &*name == "consult" {
                return self.consult(args, before, (first, last));
            }
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
                // `query(p(X)) :- body.` or `evidence(p(X)) :- body.`: a
                // query rule.
                self.bump();
                let body = self.body()?;
                self.expect(Tok::Dot)?;
                let evidence = id == kw::EVIDENCE;
                let (pattern, positive) =
                    self.asked(if evidence { "evidence" } else { "query" }, start)?;
                for alt in body.dnf() {
                    prog.query_rules.push(QueryRule {
                        pattern: pattern.clone(),
                        positive,
                        evidence,
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
        let mut heads = self.heads()?;
        let mut body = if self.at(Tok::Neck) {
            self.bump();
            self.body()?
        } else {
            Body::Lits(Vec::new())
        };
        if self.aggregates > 0 {
            body = self.aggregate(&mut heads, body)?;
        }
        self.expect(Tok::Dot)?;
        let group = self.group;
        match body {
            // A conjunction of literals, the common body, is its one
            // alternative.
            Body::Lits(body) => prog.clauses.push(Clause { heads, body, group }),
            // The heads go to the last alternative, and copies to the
            // others.
            body => {
                if body.interleaved() {
                    prog.interleaved.push(group);
                }
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

    /// The directive `consult(args)` on `lines`, after a clause that ends
    /// on line `before`, read where the parse reads them.
    fn consult(
        &mut self,
        args: Vec<PTerm>,
        before: usize,
        lines: (usize, usize),
    ) -> Result<(), ParseError> {
        let file = match <[PTerm; 1]>::try_from(args) {
            Ok([t]) => match t.into_term() {
                Term::Const(file) => file.to_string(),
                t => return self.err(format!("consult({t}): the file is named by an atom")),
            },
            Err(args) => return self.err(format!("unsupported directive consult/{}", args.len())),
        };
        if self.consults.is_none() {
            return self.err(format!(
                "consult({file}) reads a file: give tddlog the program as a file, \
                 which it reads with the files it consults"
            ));
        }
        // The directive is replaced by its file's lines.
        if before == lines.0 || (!self.at(Tok::Eof) && self.line() == lines.1) {
            return self.err(format!("consult({file}) must stand on lines of its own"));
        }
        self.consults
            .as_mut()
            .expect("read")
            .push(Consult { file, lines });
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
                let q = self.asked(name, start)?;
                prog.queries.push(q);
                Ok(())
            }
            ("evidence", 1 | 2) => {
                let e = self.asked(name, start)?;
                prog.evidence.push(e);
                Ok(())
            }
            // The list predicates are built in ([`super::Builtin`]), and the
            // grounder adds the clauses of those of the libraries that are
            // written as clauses: library(aggregate)'s where it is loaded.
            ("use_module", 1) => {
                let t = self.args.pop().expect("an argument").into_term();
                match t.as_compound() {
                    Some((library, [Term::Const(name)]))
                        if &**library == "library"
                            && matches!(&**name, "lists" | "apply" | "aggregate") =>
                    {
                        prog.aggregate |= &**name == "aggregate";
                        Ok(())
                    }
                    _ => self.err(format!("unsupported directive use_module({t})")),
                }
            }
            // ProbLog's flag for a call of a predicate without clauses:
            // an error, its default, or a failure.
            ("unknown", 1) => {
                let t = self.args.pop().expect("an argument").into_term();
                match &t {
                    Term::Const(flag) if matches!(&**flag, "error" | "fail") => {
                        prog.unknown_fails = &**flag == "fail";
                        Ok(())
                    }
                    _ => self.err(format!("unsupported directive unknown({t})")),
                }
            }
            _ => self.err(format!("unsupported directive {name}/{arity}")),
        }
    }

    /// An atom or `\+ atom`, as `(atom, positive)`.
    /// The atom and sign that `query(A)`, `evidence(A)` or `evidence(A, V)`
    /// asks, its arguments on [`Parser::args`] from `start`, taken from
    /// there: the sign of `\+` in `A`, and for evidence its value `V`.
    fn asked(&mut self, name: &str, start: usize) -> Result<(Atom, bool), ParseError> {
        let value = match (name, self.args.len() - start) {
            ("query", 1) | ("evidence", 1) => true,
            ("evidence", 2) => match self.args.pop().expect("an argument") {
                PTerm::Term(Term::Const(s)) if &*s == "true" => true,
                PTerm::Term(Term::Const(s)) if &*s == "false" => false,
                _ => return self.err("evidence/2 expects true or false"),
            },
            (_, arity) => return self.err(format!("unsupported directive {name}/{arity}")),
        };
        let t = self.args.pop().expect("an argument");
        let what = if name == "query" {
            "query/1"
        } else {
            "evidence"
        };
        let (a, sign) = self.signed_atom(t, what)?;
        Ok((a, sign == value))
    }

    fn signed_atom(&self, t: PTerm, what: &str) -> Result<(Atom, bool), ParseError> {
        let negated = matches!(t.functor(), Some((n, 1)) if n == "\\+" || n == "not");
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
        // `P::atom`: a number, or a term read up to `::`, which is the
        // head itself where no `::` follows.
        let prob = match (self.peek(), self.peek2()) {
            (Tok::Float(p), Tok::DoubleColon) => {
                self.bump();
                self.bump();
                Annotation::Number(p)
            }
            (Tok::Int(p), Tok::DoubleColon) => {
                self.bump();
                self.bump();
                Annotation::Number(p as f64)
            }
            (Tok::Naf, _) => return self.negated_head(None),
            _ => {
                let t = self.term(999)?;
                if !self.at(Tok::DoubleColon) {
                    let line = self.line();
                    let atom = t.into_atom().ok_or_else(|| ParseError {
                        line,
                        msg: "expected an atom".into(),
                    })?;
                    return Ok(HeadChoice {
                        prob: None,
                        atom,
                        negated: false,
                    });
                }
                self.bump();
                self.annotation(t.into_term())?
            }
        };
        if let Annotation::Number(p) = prob
            && !(0.0..=1.0 + 1e-12).contains(&p)
        {
            return self.err(format!("probability {p} out of range"));
        }
        self.negated_head(Some(prob))
    }

    /// The probability a term before `::` writes: a number, or a fraction
    /// `a/b` of two, as a number; `t(...)`, a learnable probability, is
    /// refused; any other term is a flexible probability.
    fn annotation(&self, t: Term) -> Result<Annotation, ParseError> {
        let number = |t: &Term| match t {
            Term::Int(i) => Some(*i as f64),
            Term::Float(x) => Some(x.0),
            _ => None,
        };
        if let Some(p) = number(&t) {
            return Ok(Annotation::Number(p));
        }
        match t.as_compound() {
            Some((name, [a, b])) if &**name == "/" => {
                if let (Some(a), Some(b)) = (number(a), number(b)) {
                    if b == 0.0 {
                        return self.err("expected a nonzero denominator");
                    }
                    return Ok(Annotation::Number(a / b));
                }
            }
            Some((name, [_])) if &**name == "t" => {
                return self.err("learnable probabilities t(...) are not supported");
            }
            _ => {}
        }
        Ok(Annotation::Flexible(t))
    }

    /// The head after its probability, if any: `\+atom` or `atom`.
    fn negated_head(&mut self, prob: Option<Annotation>) -> Result<HeadChoice, ParseError> {
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

    /// ProbLog's aggregate `h(G1, ..., Gn, f<V>) :- Body`, its one head
    /// not annotated, or the same in a scope `S:h(...)`, as
    /// library(aggregate)'s aggregate/5 computes it: the head `h(G1, ...,
    /// Gn, R)` over the body `all(([G1, ..., Gn], V), Body, L),
    /// enum_groups(L, [G1, ..., Gn], Vs), f(Vs, R)`. The solutions of the
    /// body are grouped by `[G1, ..., Gn]`, and `R` is `f` of the values
    /// `V` of a group. aggregate/5 calls the body with its variables free,
    /// and the all/3 reads it with them renamed apart ([`renamed`]).
    fn aggregate(&mut self, heads: &mut [HeadChoice], body: Body) -> Result<Body, ParseError> {
        let once = self.aggregates == 1;
        self.aggregates = 0;
        let [head] = heads else {
            return self.err(AGGREGATE_ONLY);
        };
        if !once || head.prob.is_some() || head.negated {
            return self.err(AGGREGATE_ONLY);
        }
        // The groups, the function and the variable of the aggregate in the
        // last of `args`.
        let split = |args: &[Term]| match args.split_last() {
            Some((last, groups)) => match last.as_compound() {
                Some((m, [Term::Const(f), v])) if &**m == AGGREGATE => {
                    Some((groups.to_vec(), f.clone(), v.clone()))
                }
                _ => None,
            },
            None => None,
        };
        let r = Term::Var("$R".into());
        let with_r = |groups: &[Term]| groups.iter().chain([&r]).cloned().collect::<Vec<_>>();
        let atom = &head.atom;
        let (groups, f, v, atom) = match atom.args.as_slice() {
            [s, Term::Compound(c)] if &*atom.name == ":" && split(&c.args).is_some() => {
                let (groups, f, v) = split(&c.args).expect("an aggregate");
                let scoped = Compound::term(c.name.clone(), with_r(&groups));
                let atom = Atom::new(atom.name.clone(), vec![s.clone(), scoped]);
                (groups, f, v, atom)
            }
            args => match split(args) {
                Some((groups, f, v)) => {
                    let atom = Atom::new(atom.name.clone(), with_r(&groups));
                    (groups, f, v, atom)
                }
                None => return self.err(AGGREGATE_ONLY),
            },
        };
        head.atom = atom;
        let (cons, nil) = (self.lexer.intern(CONS), Term::Const(self.lexer.intern(NIL)));
        let list = |items: Vec<Term>| {
            items
                .into_iter()
                .rev()
                .fold(nil.clone(), |t, x| Compound::term(cons.clone(), vec![x, t]))
        };
        let pair = vec![list(groups.iter().map(renamed).collect()), renamed(&v)];
        let interleaved = body.interleaved();
        let goal = body
            .dnf()
            .iter()
            .map(|alt| alt.iter().map(renamed_literal).collect())
            .collect();
        let (l, vs) = (Term::Var("$L".into()), Term::Var("$Vs".into()));
        let collect = Findall {
            template: Compound::term(self.lexer.intern(","), pair),
            goal,
            result: l.clone(),
            interleaved,
            all: true,
        };
        let groups = Atom::new(
            self.lexer.intern("enum_groups"),
            vec![l, list(groups), vs.clone()],
        );
        Ok(Body::Lits(vec![
            Literal::Findall(Box::new(collect)),
            Literal::Pos(groups),
            Literal::Pos(Atom::new(f, vec![vs, r])),
        ]))
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
            // Either a parenthesised body or a parenthesised term in a
            // comparison. Try the body reading first; an operator after the
            // closing paren means it was a term.
            let save = self.mark();
            self.bump();
            if let Ok(b) = self.body()
                && self.at(Tok::RParen)
            {
                self.bump();
                if self
                    .infix_at()
                    .is_none_or(|(name, ..)| name == "," || name == ";")
                {
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
        let t = self.term(999)?;
        self.literal_of(t)
    }

    /// The literal a term written as a goal is.
    fn literal_of(&self, t: PTerm) -> Result<Literal, ParseError> {
        let (name, mut args) = match t {
            PTerm::Atom(name, args) if !special(&name, args.len()) => {
                return Ok(Literal::Pos(Atom { name, args }));
            }
            PTerm::Atom(name, args) => (name, Vec::from(args)),
            PTerm::Term(Term::Compound(c)) => (c.name.clone(), c.args.to_vec()),
            PTerm::Term(Term::Const(c)) => {
                return Ok(match &*c {
                    "true" => Literal::Cmp(CmpOp::Eq, Term::Int(0), Term::Int(0)),
                    "fail" | "false" => Literal::Cmp(CmpOp::Ne, Term::Int(0), Term::Int(0)),
                    _ => Literal::Pos(Atom::new(c, Args::default())),
                });
            }
            PTerm::Term(other) => return self.err(format!("expected a literal, found {other}")),
        };
        match (&*name, args.len()) {
            ("is", 2) => {
                let e = args.pop().expect("two arguments");
                let v = args.pop().expect("two arguments");
                Ok(Literal::Is(v, e))
            }
            (op, 2) if CmpOp::of(op).is_some() => {
                let b = args.pop().expect("two arguments");
                let a = args.pop().expect("two arguments");
                Ok(Literal::Cmp(CmpOp::of(op).expect("a comparison"), a, b))
            }
            ("\\+" | "not", 1) => self.negation(args.pop().expect("an argument")),
            ("findall" | "all", 3) => {
                let result = args.pop().expect("three arguments");
                let goal = self.goal(args.pop().expect("three arguments"))?;
                let template = args.pop().expect("three arguments");
                let interleaved = goal.interleaved();
                Ok(Literal::Findall(Box::new(Findall {
                    template,
                    goal: goal.dnf(),
                    result,
                    interleaved,
                    all: &*name == "all",
                })))
            }
            // forall(C, A) holds where (C, \+ A) has no solution:
            // findall(_, (C, \+ A), []).
            ("forall", 2) => {
                let unless = self.negation(args.pop().expect("two arguments"))?;
                let goal = self.goal(args.pop().expect("two arguments"))?;
                let interleaved = goal.interleaved();
                let mut alts = goal.dnf();
                for alt in &mut alts {
                    alt.push(unless.clone());
                }
                Ok(Literal::Findall(Box::new(Findall {
                    template: Term::Const("true".into()),
                    goal: alts,
                    result: Term::Const(super::NIL.into()),
                    interleaved,
                    all: false,
                })))
            }
            ("->", 2) => self.err("if-then-else is not supported"),
            (",", 2) | (";", 2) => {
                // A parenthesised body the body reading gave up on: its
                // first part that is not a literal says why.
                self.goal(Compound::term(name.clone(), args))?;
                self.err("expected a literal, found a conjunction")
            }
            (op, n) if special(op, n) => {
                let t = Compound::term(name.clone(), args);
                self.err(format!("expected a literal, found {t}"))
            }
            _ => Ok(Literal::Pos(Atom::new(name, args))),
        }
    }

    /// The literal `\+ t`.
    fn negation(&self, t: Term) -> Result<Literal, ParseError> {
        const COMPOUND: &str =
            "negation of a negation or a compound goal is not supported; define a predicate for it";
        if matches!(t.as_compound(), Some((n, [_, _])) if matches!(&**n, "," | ";")) {
            return self.err(COMPOUND);
        }
        match self.literal_of(PTerm::Term(t))? {
            Literal::Pos(a) => Ok(Literal::Neg(a)),
            Literal::Cmp(op, a, b) => Ok(Literal::Cmp(op.negated(), a, b)),
            Literal::Is(..) => self.err("negation of `is` is not supported"),
            Literal::Findall(_) => self.err("negation of findall/3 is not supported"),
            _ => self.err(COMPOUND),
        }
    }

    /// The body a goal term is: conjunctions and disjunctions of literals.
    fn goal(&self, t: Term) -> Result<Body, ParseError> {
        match t.as_compound() {
            Some((name, [a, b])) if &**name == "," => {
                let mut conjuncts = Vec::new();
                for part in [a, b] {
                    match self.goal(part.clone())? {
                        Body::And(cs) => conjuncts.extend(cs),
                        other => conjuncts.push(other),
                    }
                }
                Ok(Body::And(conjuncts))
            }
            Some((name, [a, b])) if &**name == ";" => Ok(Body::Or(
                Box::new(self.goal(a.clone())?),
                Box::new(self.goal(b.clone())?),
            )),
            _ if matches!(t, Term::Var(_) | Term::Int(_) | Term::Float(_)) => {
                self.err(format!("a goal must be an atom, found {t}"))
            }
            _ => Ok(Body::Lits(vec![self.literal_of(PTerm::Term(t))?])),
        }
    }

    /// Parse an atom: a head, or the atom of a query or of evidence.
    fn atom(&mut self) -> Result<Atom, ParseError> {
        let t = self.term(999)?;
        let line = self.line();
        t.into_atom().ok_or_else(|| ParseError {
            line,
            msg: "expected an atom".into(),
        })
    }

    /// The infix operator at hand: its name, priority and grouping. A
    /// comma and a semicolon are operators inside a term in parentheses,
    /// above the priority of an argument or a literal.
    fn infix_at(&self) -> Option<(&'static str, u16, Assoc)> {
        let name = match self.peek() {
            Tok::Op(op) => op.text(),
            Tok::Name(kw::IS) => "is",
            Tok::Name(kw::MOD) => "mod",
            Tok::Name(kw::REM) => "rem",
            Tok::Name(kw::XOR) => "xor",
            Tok::Comma => ",",
            Tok::Semi => ";",
            _ => return None,
        };
        let (p, assoc) = infix_op(name)?;
        Some((name, p, assoc))
    }

    /// A term of priority up to `max`: a primary term followed by infix
    /// operators, each argument read at the priority its operator allows.
    fn term(&mut self, max: u16) -> Result<PTerm, ParseError> {
        let (mut left, mut priority) = self.primary()?;
        while let Some((name, p, assoc)) = self.infix_at() {
            let left_max = if assoc == Assoc::Yfx { p } else { p - 1 };
            if p > max || priority > left_max {
                break;
            }
            self.bump();
            let right = self.term(if assoc == Assoc::Xfy { p } else { p - 1 })?;
            let name = self.lexer.intern(name);
            left = PTerm::Term(Compound::term(
                name,
                vec![left.into_term(), right.into_term()],
            ));
            priority = p;
        }
        Ok(left)
    }

    /// A term without an infix operator at its top, and its priority: a
    /// prefix operator's, 0 for any other.
    fn primary(&mut self) -> Result<(PTerm, u16), ParseError> {
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
            return Ok((
                match args {
                    Some(args) => PTerm::Atom(n, args),
                    None => self.arguments(n)?,
                },
                0,
            ));
        }
        // `f<V>`, ProbLog's aggregate ([`Parser::aggregate`]).
        if let (Tok::Name(id), Tok::Op(Op::Lt)) = (self.peek(), self.peek2()) {
            let m = self.mark();
            self.bump();
            self.bump();
            if let Tok::Var(v) = self.peek()
                && v != "_"
                && matches!(self.peek2(), Tok::Op(Op::Gt))
            {
                let args = vec![
                    Term::Const(self.lexer.name(id).clone()),
                    Term::Var(v.to_owned()),
                ];
                self.bump();
                self.bump();
                self.aggregates += 1;
                let name = self.lexer.intern(AGGREGATE);
                return Ok((PTerm::Term(Compound::term(name, args)), 0));
            }
            self.rewind(m);
        }
        if let Some(t) = self.flat_term() {
            return Ok((PTerm::Term(t), 0));
        }
        // An operator standing alone as an argument or an item of a list
        // is the atom of its name, as in ProbLog: `[+, -, *, /]`.
        if let Tok::Op(op) = self.peek()
            && matches!(
                self.peek2(),
                Tok::Comma | Tok::RParen | Tok::RBracket | Tok::Bar
            )
        {
            self.bump();
            return Ok((PTerm::Term(Term::Const(self.lexer.intern(op.text()))), 0));
        }
        let (name, p) = match self.peek() {
            Tok::Name(kw::NOT) | Tok::Naf => ("\\+", 900),
            Tok::Op(op @ (Op::Minus | Op::Plus | Op::Backslash)) => {
                (op.text(), prefix_op(op.text()).expect("a prefix operator"))
            }
            Tok::LParen => {
                self.bump();
                let t = self.term(1200)?;
                self.expect(Tok::RParen)?;
                return Ok((t, 0));
            }
            Tok::LBracket => return Ok((self.list()?, 0)),
            other => return self.err(format!("unexpected {}", self.lexer.show(other))),
        };
        self.bump();
        let arg = self.term(p)?.into_term();
        let name = self.lexer.intern(name);
        Ok((PTerm::Term(Compound::term(name, vec![arg])), p))
    }

    /// A list, `[` at hand: `[]`, `[a, b]` or `[a, b | T]`.
    fn list(&mut self) -> Result<PTerm, ParseError> {
        self.bump();
        let nil = Term::Const(self.lexer.intern(NIL));
        if self.at(Tok::RBracket) {
            self.bump();
            return Ok(PTerm::Term(nil));
        }
        let mut items = vec![self.term(999)?.into_term()];
        while self.at(Tok::Comma) {
            self.bump();
            items.push(self.term(999)?.into_term());
        }
        let tail = if self.at(Tok::Bar) {
            self.bump();
            self.term(999)?.into_term()
        } else {
            nil
        };
        self.expect(Tok::RBracket)?;
        let cons = self.lexer.intern(CONS);
        Ok(PTerm::Term(items.into_iter().rev().fold(tail, |t, x| {
            Compound::term(cons.clone(), vec![x, t])
        })))
    }

    /// The flat term at hand, if it is one, taken: a number, a negative
    /// one, a variable (`_` a fresh one), or a name not followed by `(`
    /// (`not` before a term is the prefix operator). Inlined, as it is
    /// called per argument.
    #[inline(always)]
    fn flat_term(&mut self) -> Option<Term> {
        let t = match (self.peek(), self.peek2()) {
            (Tok::Int(i), _) => Term::Int(i),
            (Tok::Float(x), _) => Term::Float(F64::rounded(x)),
            (Tok::Var("_"), _) => {
                self.anon += 1;
                Term::Var(format!("_G{}", self.anon))
            }
            (Tok::Var(v), _) => Term::Var(v.to_owned()),
            (
                Tok::Name(kw::NOT),
                Tok::Name(_) | Tok::Var(_) | Tok::LParen | Tok::LBracket | Tok::Naf,
            ) => return None,
            (Tok::Name(id), _) if !self.at2(Tok::LParen) => {
                Term::Const(self.lexer.name(id).clone())
            }
            (Tok::Op(Op::Minus), Tok::Int(i)) => {
                self.bump();
                Term::Int(-i)
            }
            (Tok::Op(Op::Minus), Tok::Float(x)) => {
                self.bump();
                Term::Float(F64::rounded(-x))
            }
            _ => return None,
        };
        self.bump();
        Some(t)
    }

    /// The compound of `name`, its `(` read, over the arguments up to its
    /// `)`. The arguments of an atom are read in the lexer
    /// ([`Lexer::flat_arguments`]), those of any other compound here.
    fn arguments(&mut self, name: Name) -> Result<PTerm, ParseError> {
        let start = self.args.len();
        self.argument_list()?;
        Ok(self.compound(name, start))
    }

    /// The arguments of a compound, its `(` read, up to and past its `)`,
    /// onto [`Parser::args`].
    fn argument_list(&mut self) -> Result<(), ParseError> {
        let arg = self.term(999)?;
        self.args.push(arg);
        while self.at(Tok::Comma) {
            self.bump();
            let arg = self.term(999)?;
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
                    .map(PTerm::into_term)
                    .collect::<Args>(),
            )
        } else {
            let args = self.args.drain(start..).map(PTerm::into_term).collect();
            PTerm::Term(Compound::term(name, args))
        }
    }
}

/// A term as the parser reads it: a compound with flat arguments, the
/// common one, as an atom's name and arguments, read as an atom without a
/// compound made for it.
#[derive(Clone, Debug)]
enum PTerm {
    Term(Term),
    Atom(Name, Args),
}

impl PTerm {
    fn is_flat(&self) -> bool {
        matches!(self, PTerm::Term(t) if !matches!(t, Term::Compound(_)))
    }
    fn into_term(self) -> Term {
        match self {
            PTerm::Term(t) => t,
            PTerm::Atom(name, args) => Compound::term(name, args.into()),
        }
    }
    /// An atom: a constant or a compound.
    fn into_atom(self) -> Option<Atom> {
        match self {
            PTerm::Atom(name, args) => Some(Atom { name, args }),
            PTerm::Term(Term::Const(c)) => Some(Atom::new(c, Args::default())),
            PTerm::Term(Term::Compound(c)) => Some(Atom::new(c.name.clone(), c.args.to_vec())),
            PTerm::Term(_) => None,
        }
    }
    /// A compound's name and arity.
    fn functor(&self) -> Option<(&str, usize)> {
        match self {
            PTerm::Atom(n, args) => Some((n, args.len())),
            PTerm::Term(Term::Compound(c)) => Some((&c.name, c.args.len())),
            PTerm::Term(_) => None,
        }
    }
    /// A compound's name and arguments.
    fn into_compound(self) -> Option<(Name, Vec<PTerm>)> {
        match self {
            PTerm::Atom(n, args) => {
                Some((n, Vec::from(args).into_iter().map(PTerm::Term).collect()))
            }
            PTerm::Term(Term::Compound(c)) => Some((
                c.name.clone(),
                c.args.iter().cloned().map(PTerm::Term).collect(),
            )),
            PTerm::Term(_) => None,
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
        assert_eq!(p.clauses[0].heads[0].mass(), Some(0.3));
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
        assert!(p.interleaved.is_empty());
        let p = parse_program("a :- b, (c ; d).\ne :- (c ; d), b.\n").unwrap();
        assert_eq!(p.interleaved, vec![0]);
    }

    #[test]
    fn arithmetic_and_alt_neck() {
        let p = parse_program(
            "1/3::a <- true.\np(X) :- q(Y), X is Y*2+1, X > -3, X =:= 7, Z is 2 ** -1 / 4.\n",
        )
        .unwrap();
        assert!((p.clauses[0].heads[0].mass().unwrap() - 1.0 / 3.0).abs() < 1e-12);
        let c = &p.clauses[1];
        assert_eq!(c.body.len(), 5);
        assert_eq!(c.body[1].to_string(), "X is Y*2+1");
        assert_eq!(c.body[2].to_string(), "X>-3");
        assert!(matches!(c.body[3], Literal::Cmp(CmpOp::ArithEq, _, _)));
        assert_eq!(c.body[4].to_string(), "Z is 2**-1/4");
        let p = parse_program("p(X) :- X is (1 + 2) * 3 - 4 - 5 mod 2.\n").unwrap();
        assert_eq!(p.clauses[0].body[0].to_string(), "X is (1+2)*3-4-5 mod 2");
    }

    #[test]
    fn a_term_before_the_double_colon_is_a_probability() {
        let p = parse_program(
            "P::h(X) :- w(X, P).\n1 / W::g(X); 0.5::k :- w(X, W).\n(1 / 4)::a. 0.5::\\+b.\n",
        )
        .unwrap();
        let flexible = |c: usize, h: usize| match &p.clauses[c].heads[h].prob {
            Some(Annotation::Flexible(t)) => t.to_string(),
            other => panic!("{other:?}"),
        };
        assert_eq!(flexible(0, 0), "P");
        assert_eq!(flexible(1, 0), "1/W");
        assert_eq!(p.clauses[1].heads[1].mass(), Some(0.5));
        assert_eq!(p.clauses[2].heads[0].mass(), Some(0.25));
        assert!(p.clauses[3].heads[0].negated);
        let text = p.to_string();
        assert_eq!(parse_program(&text).unwrap().to_string(), text);
        for (source, error) in [
            ("t(0.5)::a.", "learnable"),
            ("t(_)::a :- b.", "learnable"),
            ("1/0::a.", "nonzero denominator"),
            ("1/0.5::a.", "probability 2 out of range"),
            ("1.5::a.", "out of range"),
        ] {
            let e = parse_program(source).unwrap_err();
            assert!(e.msg.contains(error), "{source}: {}", e.msg);
        }
    }

    #[test]
    fn terms_lists_and_floats() {
        let p = parse_program(
            "s([the, X | T], 0.5, -1.5e3, f(g(X), [])) :- T = [a|[b]], Y = T - 1, \\+ Y = a.\n",
        )
        .unwrap();
        let c = &p.clauses[0];
        assert_eq!(
            c.heads[0].atom.to_string(),
            "s([the, X | T],0.5,-1500.0,f(g(X),[]))"
        );
        assert_eq!(c.body[0].to_string(), "T=[a, b]");
        assert_eq!(c.body[1].to_string(), "Y=T-1");
        assert!(matches!(c.body[2], Literal::Cmp(CmpOp::Ne, _, _)));
        let p = parse_program(
            ":- use_module(library(lists)).\nq :- findall(X-Y, (p(X), r(X, Y) ; s(X, Y)), L), length(L, N).\n",
        )
        .unwrap();
        let Literal::Findall(fa) = &p.clauses[0].body[0] else {
            panic!("a findall");
        };
        assert_eq!(fa.template.to_string(), "X-Y");
        assert_eq!(fa.goal.len(), 2);
        assert!(!fa.interleaved);
        assert!(parse_program(":- use_module(library(apply)).\n").is_ok());
        assert!(parse_program(":- use_module(library(assert)).\n").is_err());
        // An operator alone is an atom; before a term, a prefix operator.
        let p =
            parse_program("ops([+, -, *, /]). p([N, +|T], -, - X) :- q(N, T, X, /).\n").unwrap();
        assert_eq!(p.clauses[0].heads[0].atom.to_string(), "ops([+, -, *, /])");
        assert_eq!(p.clauses[1].heads[0].atom.to_string(), "p([N, + | T],-,-X)");
        assert_eq!(p.clauses[1].body[0].to_string(), "q(N,T,X,/)");
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
        // Evidence rules, the value as evidence/2 gives it.
        let p = parse_program(
            "evidence(p(X)) :- a(X).\nevidence(q(X), false) :- a(X).\nevidence(\\+r(X)) :- a(X).\n",
        )
        .unwrap();
        let rules: Vec<(String, bool, bool)> = p
            .query_rules
            .iter()
            .map(|r| (r.pattern.to_string(), r.positive, r.evidence))
            .collect();
        assert_eq!(
            rules,
            [
                ("p(X)".to_string(), true, true),
                ("q(X)".to_string(), false, true),
                ("r(X)".to_string(), false, true)
            ]
        );
        assert_eq!(parse_program(&p.to_string()).unwrap(), p);
        let e = parse_program("evidence(p(X), maybe) :- a(X).").unwrap_err();
        assert!(e.msg.contains("true or false"), "{}", e.msg);
        let e = parse_program("query(p(X), true) :- a(X).").unwrap_err();
        assert!(e.msg.contains("query/2"), "{}", e.msg);
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
        let p = parse_program("p(a, f(_), _).\n").unwrap();
        assert_eq!(p.clauses[0].heads[0].atom.to_string(), "p(a,f(_G1),_G2)");
    }

    #[test]
    fn atom_arguments_read_in_the_lexer() {
        // The arguments the lexer reads: names, quoted ones, variables,
        // `_`, numbers and negative ones, with blanks, comments and lines
        // between them, in heads, bodies and queries.
        let src = "p(a, 'B c', X, _, 3, - 4, -5, _, 0.5) :- q(_, a), \\+ r(b,\n  % c\n  X).\nquery(s(1)).\n0.5::t(x); 0.5::u(y, z).\n";
        let p = parse_program(src).unwrap();
        let c = &p.clauses[0];
        assert_eq!(
            c.heads[0].atom.to_string(),
            "p(a,'B c',X,_G1,3,-4,-5,_G2,0.5)"
        );
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
        assert_eq!(
            err("p(a,\n\n b c)."),
            "line 3: expected `)`, found name `c`"
        );
        assert_eq!(err("p(a,\n $)."), "line 2: unexpected character `$`");
        assert_eq!(err("p(a, b"), "line 1: expected `)`, found end of input");
        assert_eq!(err("foo :- 3."), "line 1: expected a literal, found 3");
        assert_eq!(
            err("foo :- X + 1."),
            "line 1: expected a literal, found X+1"
        );
        assert_eq!(
            err("foo :- \\+ (a, b)."),
            "line 1: negation of a negation or a compound goal is not supported; define a predicate for it"
        );
        // A compound argument, `\+` and an atom named `not` inside query/1
        // and evidence/2, read with the anonymous variables in order.
        let p = parse_program("evidence(\\+ e(_, 1), true).\nquery(not(q(_))).\n").unwrap();
        assert_eq!(p.evidence[0].0.to_string(), "e(_G1,1)");
        assert!(!p.evidence[0].1);
        assert_eq!(p.queries[0].0.to_string(), "q(_G1)");
        assert!(!p.queries[0].1);
        let p = parse_program("p(a, - b).\n").unwrap();
        assert_eq!(p.clauses[0].heads[0].atom.to_string(), "p(a,-b)");
    }

    /// An aggregate `f<V>` ends a head, in a scope too: the clause collects
    /// the solutions of its body, renamed apart, with all/3, groups them by
    /// the head's other arguments and applies `f` to a group's values. It
    /// is refused anywhere else.
    #[test]
    fn aggregates_end_heads() {
        let p = parse_program(
            ":- use_module(library(aggregate)).\n\
             d(D, avg<S>) :- p(X, S), q(X, D).\n\
             s:t(max<S>) :- p(_, S) ; r(S).\n",
        )
        .unwrap();
        assert!(p.aggregate);
        let c = &p.clauses[0];
        assert_eq!(c.heads[0].atom.to_string(), "d(D,$R)");
        let Literal::Findall(fa) = &c.body[0] else {
            panic!("an all/3");
        };
        assert!(fa.all);
        assert_eq!(fa.template.to_string(), "([D$], S$)");
        let goal: Vec<String> = fa.goal[0].iter().map(Literal::to_string).collect();
        assert_eq!(goal, ["p(X$,S$)", "q(X$,D$)"]);
        assert_eq!(c.body[1].to_string(), "enum_groups($L,[D],$Vs)");
        assert_eq!(c.body[2].to_string(), "avg($Vs,$R)");
        let c = &p.clauses[1];
        assert_eq!(c.heads[0].atom.to_string(), "s:t($R)");
        let Literal::Findall(fa) = &c.body[0] else {
            panic!("an all/3");
        };
        assert_eq!(fa.goal.len(), 2);
        for src in [
            "p(avg<X>, Y) :- q(X, Y).\n",
            "0.5::p(avg<X>) :- q(X).\n",
            "p(avg<X>) ; r(avg<X>) :- q(X).\n",
            "p :- q(avg<X>).\n",
            "query(p(avg<X>)).\n",
        ] {
            let err = parse_program(src).unwrap_err().to_string();
            assert!(err.contains("aggregate f<V>"), "{src}: {err}");
        }
        // Otherwise `<` and `>` compare.
        let p = parse_program("p(X) :- a < b, X > 1, a < X.\n").unwrap();
        assert_eq!(p.clauses[0].body.len(), 3);
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
            err("p :- (q -> r ; s)."),
            "line 1: if-then-else is not supported"
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
