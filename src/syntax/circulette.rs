//! Circulette netlists, read as the ProbLog program this module writes for
//! them ([`read`]), which `tddlog --program` prints and the parser then
//! reads.
//!
//! Circulette (Sandia National Laboratories, github.com/sandialabs/circulette)
//! is a tool for the fault analysis of digital circuits on Roulette. A
//! netlist, a `.cql` file, is written in Racket's syntax:
//!
//! ```text
//! (def-netlist half
//!   (hash 0 1 1 1)            ; the inputs: wire, width
//!   (hash 2 1 3 1)            ; the outputs
//!   (hash)                    ; the registers
//!   (4 (w^ 0 1))              ; each wire: its id and its expression
//!   (5 (w& 0 1))
//!   (2 (<w= 4))
//!   (3 (<w= 5)))
//! (define p-clean 0.99)
//! (define expected-output (list #f #f))
//! ```
//!
//! The circuit is evaluated for one clock cycle with every input and every
//! register false, on the lines the outputs read, transitively, where
//! reading a register reads its next value's line too (Circulette's slice).
//! Each logic gate of the slice, `w&`, `w||`, `w^` and `w~` (and, or,
//! exclusive or, not), is faulty with probability `1 - p-clean`, and a
//! fault flips its output. The observed bits are those of the outputs and
//! of the next values of the registers in the slice, and the circuit fails
//! where one of them differs from `expected-output`. Circulette asks for
//! the probability of the failure and for the posterior given it of every
//! single, pair and triple of faults. An input's bit is false where a
//! logic gate, an output or a register's next value reads it, but a
//! multiplexer reads it as a one, through any wiring, as a select and as
//! data: so Circulette's runs answer.
//!
//! An expression is a wire's id or one of
//!
//! - `(bv-const V W)`, the `W`-bit constant `V`;
//! - `(ws E (list I))`, bit `I` of `E`, bit 0 the least significant;
//! - `(wc (list E1 ... Ek))`, the concatenation, `E1` the least significant;
//! - `(wx S A B)`, a multiplexer: `B` where the bit `S` is one, else `A`;
//! - the logic gates `(w& A B)`, `(w|| A B)`, `(w^ A B)` and `(w~ A)`;
//! - `(<w= E)`, the wire is `E`, and `(<<= E)`, a register's next value
//!   (any other wire is `E`).
//!
//! A table is `(hash K V ...)` or `#hash((K . V) ...)`; a list
//! `(list ...)` or `'(...)`.
//!
//! The program states the netlist as facts: `gate_and(G, A, B)`,
//! `gate_or`, `gate_xor` and `gate_not(G, A)` for the logic gates,
//! `mux(W, S, A, B)`, `wire(W, I, A, J)` where bit `I` of `W` is bit `J`
//! of `A` (an assignment, a selection or a concatenation), `bit(W, I)`
//! where it is a constant one, `next(R, X)` for a register `R` whose next
//! value is `X`, and `output(W)`. An operand that is not a wire's id is a
//! wire of its own, `e(W, K)`, the `K`-th such operand on the line of `W`;
//! wiring is composed as it is read, so that a selection of a
//! concatenation is the bit it selects. One rule for each
//! kind of gate gives its value from its operands' bits, `bit(W, I)`; a
//! probabilistic rule gives each logic gate its fault, `fault(G)`, which
//! flips the gate's value; the failure, `failure`, is the evidence, and
//! its probability is `P(e)` (`--evidence-probability`). The queries are
//! every gate's fault, `fault(G)`, and every pair and triple of faults,
//! `faults(A, B)` and `faults(A, B, C)` with `A < B < C`, each a query of
//! its own, answered with its posterior given the failure.
//!
//! Refused, on the line of the form: an expected value of true, a logic
//! gate on a word wider than one bit, a selection of several bits, a gate
//! inside an expression, `sketch-reroll` with loops, a combinational cycle,
//! a wire read and not defined, a `p-faulty` other than `1 - p-clean`, and
//! any other form or definition. The semantics are those Circulette's
//! documentation and its netlists' comments state, and the answers of its
//! runs.

use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt::Write;

use super::{ParseError, float_repr};

/// The ProbLog program the netlist `source` is read as. The error names
/// the line of the form or of the gate the netlist cannot be read at.
pub fn read(source: &str) -> Result<String, ParseError> {
    let forms = parse(&tokenize(source)?)?;
    Netlist::of(&forms)?.program()
}

fn fail(line: usize, msg: impl Into<String>) -> ParseError {
    ParseError {
        line,
        msg: msg.into(),
    }
}

type Res<T> = Result<T, ParseError>;

// ---------------------------------------------------------------------
// Racket's syntax: the data of a netlist

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tok<'a> {
    /// `(` or `[` (with the closer it takes), or `#hash(`.
    Open {
        closer: u8,
        hash: bool,
    },
    Close(u8),
    Quote,
    Atom(&'a str),
}

/// The tokens of `text`, each with its line.
fn tokenize(text: &str) -> Res<Vec<(Tok<'_>, usize)>> {
    let b = text.as_bytes();
    let (mut i, mut line) = (0, 1);
    let mut out = Vec::new();
    while i < b.len() {
        let c = b[i];
        match c {
            b'\n' => {
                line += 1;
                i += 1;
            }
            _ if c.is_ascii_whitespace() => i += 1,
            b';' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'(' | b'[' => {
                let closer = if c == b'(' { b')' } else { b']' };
                out.push((
                    Tok::Open {
                        closer,
                        hash: false,
                    },
                    line,
                ));
                i += 1;
            }
            b')' | b']' => {
                out.push((Tok::Close(c), line));
                i += 1;
            }
            b'\'' => {
                out.push((Tok::Quote, line));
                i += 1;
            }
            b'"' => return Err(fail(line, "a string is not read in a netlist")),
            _ if b[i..].starts_with(b"#|") => {
                // A block comment, nested as Racket nests them.
                let (start, mut depth) = (line, 0usize);
                loop {
                    if i >= b.len() {
                        return Err(fail(start, "a block comment `#|` without its `|#`"));
                    }
                    if b[i..].starts_with(b"#|") {
                        depth += 1;
                        i += 2;
                    } else if b[i..].starts_with(b"|#") {
                        depth -= 1;
                        i += 2;
                        if depth == 0 {
                            break;
                        }
                    } else {
                        line += usize::from(b[i] == b'\n');
                        i += 1;
                    }
                }
            }
            _ if b[i..].starts_with(b"#hash(") => {
                out.push((
                    Tok::Open {
                        closer: b')',
                        hash: true,
                    },
                    line,
                ));
                i += 6;
            }
            _ => {
                let start = i;
                while i < b.len()
                    && !b[i].is_ascii_whitespace()
                    && !matches!(b[i], b'(' | b')' | b'[' | b']' | b'\'' | b'"' | b';')
                {
                    i += 1;
                }
                let atom = &text[start..i];
                if atom.starts_with('#') && !matches!(atom, "#t" | "#f" | "#true" | "#false") {
                    return Err(fail(line, format!("`{atom}` is not read in a netlist")));
                }
                out.push((Tok::Atom(atom), line));
            }
        }
    }
    Ok(out)
}

/// A datum: an atom, or a list with the line it opens on (`hash` for a
/// `#hash(...)` literal).
#[derive(Clone, Debug)]
enum Datum<'a> {
    Atom(&'a str, usize),
    List(Vec<Datum<'a>>, usize, bool),
}

impl<'a> Datum<'a> {
    fn line(&self) -> usize {
        match self {
            Datum::Atom(_, line) | Datum::List(_, line, _) => *line,
        }
    }

    fn atom(&self) -> Option<&'a str> {
        match self {
            Datum::Atom(a, _) => Some(a),
            Datum::List(..) => None,
        }
    }

    /// The items of a list that is not a `#hash(...)` literal.
    fn items(&self) -> Option<&[Datum<'a>]> {
        match self {
            Datum::List(items, _, false) => Some(items),
            _ => None,
        }
    }

    /// The arguments of a list whose first item is the atom `head`.
    fn call(&self, head: &str) -> Option<&[Datum<'a>]> {
        let items = self.items()?;
        (items.first()?.atom()? == head).then(|| &items[1..])
    }
}

/// How deep data may nest, so that what reads them recursively is bounded:
/// a netlist's expressions nest a few lists deep.
const DEPTH: usize = 256;

/// The data of `tokens`.
fn parse<'a>(tokens: &[(Tok<'a>, usize)]) -> Res<Vec<Datum<'a>>> {
    enum Frame<'a> {
        List(Vec<Datum<'a>>, usize, u8, bool),
        Quote(usize),
    }
    let mut top = Vec::new();
    let mut stack: Vec<Frame<'a>> = Vec::new();
    for &(tok, line) in tokens {
        if stack.len() > DEPTH {
            return Err(fail(line, format!("data nested deeper than {DEPTH} lists")));
        }
        let mut done = match tok {
            Tok::Open { closer, hash } => {
                stack.push(Frame::List(Vec::new(), line, closer, hash));
                continue;
            }
            Tok::Quote => {
                stack.push(Frame::Quote(line));
                continue;
            }
            Tok::Atom(a) => Datum::Atom(a, line),
            Tok::Close(c) => match stack.pop() {
                Some(Frame::List(items, open, closer, hash)) if closer == c => {
                    Datum::List(items, open, hash)
                }
                _ => return Err(fail(line, format!("`{}` closes nothing", c as char))),
            },
        };
        // The datum goes into its list, through the quotes before it.
        loop {
            match stack.last_mut() {
                None => {
                    top.push(done);
                    break;
                }
                Some(Frame::List(items, ..)) => {
                    items.push(done);
                    break;
                }
                Some(Frame::Quote(line)) => {
                    let line = *line;
                    stack.pop();
                    done = Datum::List(vec![Datum::Atom("quote", line), done], line, false);
                }
            }
        }
    }
    match stack.last() {
        None => Ok(top),
        Some(Frame::List(_, line, ..) | Frame::Quote(line)) => {
            Err(fail(*line, "a list without its end"))
        }
    }
}

/// A non-negative integer atom.
fn natural(d: &Datum<'_>, what: &str) -> Res<u64> {
    d.atom()
        .filter(|a| a.bytes().all(|c| c.is_ascii_digit()))
        .and_then(|a| a.parse().ok())
        .ok_or_else(|| fail(d.line(), format!("{what} is a non-negative integer")))
}

/// A real number atom, written in decimal.
fn number(d: &Datum<'_>, what: &str) -> Res<f64> {
    d.atom()
        .filter(|a| {
            a.bytes().any(|c| c.is_ascii_digit())
                && a.bytes()
                    .all(|c| c.is_ascii_digit() || matches!(c, b'.' | b'e' | b'E' | b'+' | b'-'))
        })
        .and_then(|a| a.parse::<f64>().ok())
        .filter(|x| x.is_finite())
        .ok_or_else(|| fail(d.line(), format!("{what} is a number written in decimal")))
}

/// The pairs of a table, `(hash K V ...)` or `#hash((K . V) ...)`.
fn table<'d, 'a>(d: &'d Datum<'a>, what: &str) -> Res<Vec<(&'d Datum<'a>, &'d Datum<'a>)>> {
    let bad = || {
        fail(
            d.line(),
            format!("{what} is a table, `(hash K V ...)` or `#hash((K . V) ...)`"),
        )
    };
    match d {
        Datum::List(items, _, true) => items
            .iter()
            .map(|pair| match pair.items() {
                Some([k, dot, v]) if dot.atom() == Some(".") => Ok((k, v)),
                _ => Err(bad()),
            })
            .collect(),
        _ => {
            let args = d.call("hash").ok_or_else(bad)?;
            if args.len() % 2 != 0 {
                return Err(bad());
            }
            Ok(args.chunks(2).map(|kv| (&kv[0], &kv[1])).collect())
        }
    }
}

/// The items of a list, `(list ...)`, `'(...)` or `(quote (...))`.
fn list<'d, 'a>(d: &'d Datum<'a>, what: &str) -> Res<&'d [Datum<'a>]> {
    if let Some(items) = d.call("list") {
        return Ok(items);
    }
    match d.call("quote") {
        Some([quoted]) => quoted.items(),
        _ => None,
    }
    .ok_or_else(|| {
        fail(
            d.line(),
            format!("{what} is a list, `(list ...)` or `'(...)`"),
        )
    })
}

// ---------------------------------------------------------------------
// The netlist

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Logic {
    And,
    Or,
    Xor,
    Not,
}

impl Logic {
    const ALL: [(Logic, &'static str, &'static str, usize); 4] = [
        (Logic::And, "w&", "gate_and", 2),
        (Logic::Or, "w||", "gate_or", 2),
        (Logic::Xor, "w^", "gate_xor", 2),
        (Logic::Not, "w~", "gate_not", 1),
    ];

    fn fact(self) -> &'static str {
        Logic::ALL.iter().find(|g| g.0 == self).expect("listed").2
    }
}

/// A wire's expression.
#[derive(Clone, Debug)]
enum Expr {
    Wire(u64),
    /// Its bits, the least significant first.
    Const(Vec<bool>),
    Select(Box<Expr>, usize),
    /// Its parts, the least significant first.
    Concat(Vec<Expr>),
    Logic(Logic, Vec<Expr>),
    Mux(Box<[Expr; 3]>),
    /// `<w=`, and `<<=` but for a register.
    Assign(Box<Expr>),
    /// `<<=` of a register.
    Next(Box<Expr>),
}

impl Expr {
    /// The expression `d`: a line's whole expression (`top`), an operation,
    /// or an operand, which is wiring (a wire, a constant, a selection or a
    /// concatenation).
    fn of(d: &Datum<'_>, top: bool) -> Res<Expr> {
        let line = d.line();
        if d.atom().is_some() {
            if top {
                return Err(fail(
                    line,
                    "a wire's expression is an operation: `(<w= W)` is wire W",
                ));
            }
            return Ok(Expr::Wire(natural(d, "a wire")?));
        }
        let (head, args) = match d.items() {
            Some([head, args @ ..]) if head.atom().is_some() => {
                (head.atom().expect("an atom"), args)
            }
            _ => return Err(fail(line, "an expression is a wire or a list `(OP ...)`")),
        };
        let arity = |n: usize| {
            if args.len() == n {
                Ok(())
            } else {
                Err(fail(
                    line,
                    format!("`{head}` takes {n} argument(s), not {}", args.len()),
                ))
            }
        };
        let wiring = |d: &Datum<'_>| Expr::of(d, false);
        let gate = || {
            if top {
                Ok(())
            } else {
                Err(fail(
                    line,
                    format!(
                        "`{head}` inside an expression is not read: a gate is a wire's whole expression"
                    ),
                ))
            }
        };
        Ok(match head {
            "bv-const" => {
                arity(2)?;
                let (v, w) = (
                    natural(&args[0], "a constant")?,
                    natural(&args[1], "a width")?,
                );
                if !(1..=64).contains(&w) || (w < 64 && v >> w != 0) {
                    return Err(fail(
                        line,
                        format!("(bv-const {v} {w}) is no constant of 1 to 64 bits"),
                    ));
                }
                Expr::Const((0..w).map(|i| v >> i & 1 == 1).collect())
            }
            "ws" => {
                arity(2)?;
                let at = list(&args[1], "a selection's bits")?;
                let [bit] = at else {
                    return Err(fail(
                        line,
                        "a selection `ws` of several bits is not read: one bit, `(list I)`",
                    ));
                };
                Expr::Select(Box::new(wiring(&args[0])?), natural(bit, "a bit")? as usize)
            }
            "wc" => {
                arity(1)?;
                let parts = list(&args[0], "a concatenation's parts")?;
                if parts.is_empty() {
                    return Err(fail(line, "a concatenation `wc` of no parts"));
                }
                Expr::Concat(parts.iter().map(wiring).collect::<Res<_>>()?)
            }
            "wx" => {
                gate()?;
                arity(3)?;
                let [s, a, b] = [&args[0], &args[1], &args[2]].map(wiring);
                Expr::Mux(Box::new([s?, a?, b?]))
            }
            "<w=" | "<<=" => {
                gate()?;
                arity(1)?;
                let e = Box::new(wiring(&args[0])?);
                if head == "<w=" {
                    Expr::Assign(e)
                } else {
                    Expr::Next(e)
                }
            }
            _ => match Logic::ALL.iter().find(|g| g.1 == head) {
                Some(&(kind, _, _, n)) => {
                    gate()?;
                    arity(n)?;
                    Expr::Logic(kind, args.iter().map(wiring).collect::<Res<_>>()?)
                }
                None => {
                    return Err(fail(
                        line,
                        format!("`{head}` is not an operator of a netlist"),
                    ));
                }
            },
        })
    }

    /// The wires `self` reads.
    fn reads(&self, out: &mut Vec<u64>) {
        match self {
            Expr::Wire(w) => out.push(*w),
            Expr::Const(_) => {}
            Expr::Select(e, _) | Expr::Assign(e) | Expr::Next(e) => e.reads(out),
            Expr::Concat(es) | Expr::Logic(_, es) => es.iter().for_each(|e| e.reads(out)),
            Expr::Mux(es) => es.iter().for_each(|e| e.reads(out)),
        }
    }
}

/// A wire's line: its id, its expression and the line it is on.
struct Line {
    id: u64,
    expr: Expr,
    line: usize,
}

/// A netlist as read: its tables, its lines and its parameters.
struct Netlist<'a> {
    name: &'a str,
    /// The line of `def-netlist`.
    line: usize,
    inputs: Vec<(u64, usize)>,
    outputs: Vec<(u64, usize)>,
    registers: Vec<(u64, usize)>,
    lines: Vec<Line>,
    /// The probability of a fault, `1 - p-clean`.
    fault: f64,
}

impl<'a> Netlist<'a> {
    fn of(forms: &[Datum<'a>]) -> Res<Netlist<'a>> {
        let mut netlist: Option<Netlist<'a>> = None;
        let mut defined: FxHashMap<&str, (&Datum<'a>, usize)> = FxHashMap::default();
        for form in forms {
            let line = form.line();
            let items = form
                .items()
                .ok_or_else(|| fail(line, "a netlist's forms are lists"))?;
            match items.first().and_then(|h| h.atom()) {
                Some("def-netlist") => {
                    if netlist.is_some() {
                        return Err(fail(line, "a second `def-netlist`"));
                    }
                    netlist = Some(Netlist::from_def(&items[1..], line)?);
                }
                Some("define") => {
                    let [_, name, value] = items else {
                        return Err(fail(line, "a definition is `(define NAME VALUE)`"));
                    };
                    let name = name
                        .atom()
                        .ok_or_else(|| fail(line, "a definition's name is a name"))?;
                    if !matches!(
                        name,
                        "p-clean" | "p-faulty" | "expected-output" | "internal-signals"
                    ) {
                        return Err(fail(
                            line,
                            format!("(define {name} ...) is not read in a netlist"),
                        ));
                    }
                    if defined.insert(name, (value, line)).is_some() {
                        return Err(fail(line, format!("a second definition of {name}")));
                    }
                }
                Some("sketch-reroll") => match items {
                    [_, _, loops] if loops.call("loops").is_some_and(<[_]>::is_empty) => {}
                    _ => {
                        return Err(fail(
                            line,
                            "`sketch-reroll` is read with no loops, `(sketch-reroll NAME (loops))`",
                        ));
                    }
                },
                _ => {
                    return Err(fail(
                        line,
                        "a netlist's forms are def-netlist, define and sketch-reroll",
                    ));
                }
            }
        }
        let mut netlist = netlist.ok_or_else(|| fail(1, "no `def-netlist`"))?;
        let at = netlist.line;
        let value = |name: &str| {
            defined
                .get(name)
                .copied()
                .ok_or_else(|| fail(at, format!("no (define {name} ...)")))
        };
        let (p, line) = value("p-clean")?;
        let p_clean = number(p, "p-clean")?;
        if !(0.0..=1.0).contains(&p_clean) {
            return Err(fail(
                line,
                format!("p-clean {p_clean} is not a probability"),
            ));
        }
        netlist.fault = 1.0 - p_clean;
        if let Some(&(p, line)) = defined.get("p-faulty") {
            let p_faulty = number(p, "p-faulty")?;
            if (p_faulty - netlist.fault).abs() > 1e-12 {
                return Err(fail(
                    line,
                    format!(
                        "p-faulty {p_faulty} is not 1 - p-clean: a fault's probability is 1 - p-clean"
                    ),
                ));
            }
        }
        if let Some(&(signals, line)) = defined.get("internal-signals") {
            let ids = signals
                .call("hash")
                .or_else(|| signals.call("list"))
                .ok_or_else(|| fail(line, "internal-signals lists wires, `(hash W ...)`"))?;
            for id in ids {
                natural(id, "an internal signal")?;
            }
        }
        let (expected, line) = value("expected-output")?;
        let expected = list(expected, "expected-output")?;
        for e in expected {
            match e.atom() {
                Some("#f" | "#false") => {}
                Some("#t" | "#true") => {
                    return Err(fail(
                        e.line(),
                        "an expected value of true is not read: every observed bit is expected false",
                    ));
                }
                _ => return Err(fail(e.line(), "an expected value is #t or #f")),
            }
        }
        let observed: usize = netlist
            .outputs
            .iter()
            .chain(&netlist.registers)
            .map(|o| o.1)
            .sum();
        if observed == 0 {
            return Err(fail(
                at,
                "a netlist with no outputs and no registers observes nothing",
            ));
        }
        if expected.len() != observed {
            return Err(fail(
                line,
                format!(
                    "expected-output has {} values for the {observed} bits of the outputs and the registers",
                    expected.len()
                ),
            ));
        }
        Ok(netlist)
    }

    /// The netlist of `(def-netlist NAME INPUTS OUTPUTS REGISTERS LINE ...)`
    /// on line `line`, with no fault probability yet.
    fn from_def(args: &[Datum<'a>], line: usize) -> Res<Netlist<'a>> {
        let [name, inputs, outputs, registers, lines @ ..] = args else {
            return Err(fail(
                line,
                "a netlist is `(def-netlist NAME INPUTS OUTPUTS REGISTERS (WIRE EXPR) ...)`",
            ));
        };
        let name = name
            .atom()
            .ok_or_else(|| fail(line, "a netlist's name is a name"))?;
        let mut seen = FxHashSet::default();
        let mut widths = |d: &Datum<'_>, what: &str| -> Res<Vec<(u64, usize)>> {
            table(d, what)?
                .into_iter()
                .map(|(k, v)| {
                    let (id, w) = (natural(k, "a wire")?, natural(v, "a width")?);
                    if w == 0 {
                        return Err(fail(v.line(), format!("{what}: wire {id} has no bits")));
                    }
                    if !seen.insert(id) {
                        return Err(fail(k.line(), format!("wire {id} is declared twice")));
                    }
                    Ok((id, w as usize))
                })
                .collect()
        };
        let (inputs, outputs, registers) = (
            widths(inputs, "the inputs")?,
            widths(outputs, "the outputs")?,
            widths(registers, "the registers")?,
        );
        let lines = lines
            .iter()
            .map(|d| match d.items() {
                Some([id, expr]) => Ok(Line {
                    id: natural(id, "a wire")?,
                    expr: Expr::of(expr, true)?,
                    line: d.line(),
                }),
                _ => Err(fail(d.line(), "a wire's line is `(WIRE EXPR)`")),
            })
            .collect::<Res<_>>()?;
        Ok(Netlist {
            name,
            line,
            inputs,
            outputs,
            registers,
            lines,
            fault: 0.0,
        })
    }

    /// The width of every wire: an input's and a register's as declared,
    /// any other's its expression's. Fails on a wire read and not defined,
    /// a combinational cycle, and an expression whose operands' widths do
    /// not fit it.
    fn widths(&self) -> Res<FxHashMap<u64, usize>> {
        let mut widths: FxHashMap<u64, usize> =
            self.inputs.iter().chain(&self.registers).copied().collect();
        let mut at: FxHashMap<u64, usize> = FxHashMap::default();
        for (k, l) in self.lines.iter().enumerate() {
            if self.inputs.iter().any(|i| i.0 == l.id) {
                return Err(fail(l.line, format!("wire {} is an input", l.id)));
            }
            if at.insert(l.id, k).is_some() {
                return Err(fail(l.line, format!("wire {} is defined twice", l.id)));
            }
        }
        let registers: FxHashSet<u64> = self.registers.iter().map(|r| r.0).collect();
        for &(id, _) in self.outputs.iter().chain(&self.registers) {
            if !at.contains_key(&id) {
                return Err(fail(self.line, format!("wire {id} is not defined")));
            }
        }
        // Depth first, without recursion, from each line; a register's line
        // is its next value, and reading a register reads its value now.
        let reads: Vec<Vec<u64>> = self
            .lines
            .iter()
            .map(|l| {
                let mut r = Vec::new();
                l.expr.reads(&mut r);
                r
            })
            .collect();
        let mut open: FxHashSet<u64> = FxHashSet::default();
        for (k, l) in self.lines.iter().enumerate() {
            if registers.contains(&l.id) {
                if !matches!(l.expr, Expr::Next(_)) {
                    return Err(fail(
                        l.line,
                        format!("register {} is defined by `<<=`", l.id),
                    ));
                }
                continue;
            }
            if widths.contains_key(&l.id) {
                continue;
            }
            let mut stack = vec![(k, 0)];
            open.insert(l.id);
            while let Some(&mut (k, ref mut next)) = stack.last_mut() {
                if let Some(&w) = reads[k].get(*next) {
                    *next += 1;
                    if widths.contains_key(&w) {
                        continue;
                    }
                    let line = self.lines[k].line;
                    let &j = at
                        .get(&w)
                        .ok_or_else(|| fail(line, format!("wire {w} is read and not defined")))?;
                    if !open.insert(w) {
                        return Err(fail(line, format!("wire {w} is on a combinational cycle")));
                    }
                    stack.push((j, 0));
                } else {
                    let l = &self.lines[k];
                    let w = self.width(&l.expr, &widths, l.line)?;
                    widths.insert(l.id, w);
                    open.remove(&l.id);
                    stack.pop();
                }
            }
        }
        for l in self.lines.iter().filter(|l| registers.contains(&l.id)) {
            let w = self.width(&l.expr, &widths, l.line)?;
            if w != widths[&l.id] {
                return Err(fail(
                    l.line,
                    format!(
                        "register {} is declared {} bits wide and its next value is {w}",
                        l.id, widths[&l.id]
                    ),
                ));
            }
        }
        for &(id, w) in &self.outputs {
            if widths[&id] != w {
                return Err(fail(
                    self.lines[at[&id]].line,
                    format!(
                        "output {id} is declared {w} bits wide and its expression is {}",
                        widths[&id]
                    ),
                ));
            }
        }
        Ok(widths)
    }

    /// The width of `e` on line `line`, given the widths of the wires it
    /// reads.
    fn width(&self, e: &Expr, widths: &FxHashMap<u64, usize>, line: usize) -> Res<usize> {
        let width = |e: &Expr| self.width(e, widths, line);
        Ok(match e {
            Expr::Wire(w) => *widths
                .get(w)
                .ok_or_else(|| fail(line, format!("wire {w} is read and not defined")))?,
            Expr::Const(bits) => bits.len(),
            Expr::Select(e, i) => {
                let w = width(e)?;
                if *i >= w {
                    return Err(fail(line, format!("bit {i} of a word of {w} bits")));
                }
                1
            }
            Expr::Concat(es) => es.iter().map(width).sum::<Res<usize>>()?,
            Expr::Logic(_, es) => {
                for e in es {
                    let w = width(e)?;
                    if w != 1 {
                        return Err(fail(
                            line,
                            format!(
                                "a logic gate on a word of {w} bits is not read: its operands are bits"
                            ),
                        ));
                    }
                }
                1
            }
            Expr::Mux(es) => {
                let [s, a, b] = [&es[0], &es[1], &es[2]].map(width);
                let (s, a, b) = (s?, a?, b?);
                if s != 1 || a != b {
                    return Err(fail(
                        line,
                        format!(
                            "a multiplexer selects with one bit between words of one width, not {s}, {a} and {b}"
                        ),
                    ));
                }
                a
            }
            Expr::Assign(e) | Expr::Next(e) => width(e)?,
        })
    }

    /// The wires the outputs read, transitively, the outputs among them:
    /// reading a register reads its next value's line too. Circulette
    /// evaluates these lines only, so that a gate outside them is no fault
    /// and a register outside them is not observed.
    fn slice(&self) -> FxHashSet<u64> {
        let at: FxHashMap<u64, &Expr> = self.lines.iter().map(|l| (l.id, &l.expr)).collect();
        let mut live = FxHashSet::default();
        let mut todo: Vec<u64> = self.outputs.iter().map(|o| o.0).collect();
        while let Some(w) = todo.pop() {
            if live.insert(w)
                && let Some(e) = at.get(&w)
            {
                e.reads(&mut todo);
            }
        }
        live
    }

    /// The ProbLog program of the netlist.
    fn program(&self) -> Res<String> {
        let widths = self.widths()?;
        let live = self.slice();
        let registers: Vec<(u64, usize)> = self
            .registers
            .iter()
            .filter(|r| live.contains(&r.0))
            .copied()
            .collect();
        let is_register: FxHashSet<u64> = registers.iter().map(|r| r.0).collect();
        // The lines that are wiring, by wire: a register's line is its next
        // value, and reading the register reads its value now, false.
        let declared: FxHashSet<u64> = self.registers.iter().map(|r| r.0).collect();
        let wiring = self
            .lines
            .iter()
            .filter(|l| !declared.contains(&l.id))
            .filter_map(|l| match &l.expr {
                Expr::Assign(e) | Expr::Next(e) => Some((l.id, &**e)),
                e @ (Expr::Const(_) | Expr::Select(..) | Expr::Concat(_)) => Some((l.id, e)),
                _ => None,
            })
            .collect();
        let mut w = Writer {
            widths: &widths,
            wiring,
            inputs: self.inputs.iter().map(|i| i.0).collect(),
            out: String::new(),
            kinds: Vec::new(),
            wires: false,
            ones: false,
            muxes: false,
            muxed_inputs: false,
        };
        // The facts of the netlist, in the order of its lines.
        let ids = |t: &[(u64, usize)]| {
            if t.is_empty() {
                " none".to_string()
            } else {
                t.iter().map(|(id, w)| format!(" {id}/{w}")).collect()
            }
        };
        let mut head = String::new();
        writeln!(
            head,
            "% The Circulette netlist {}, one clock cycle with every input and register\n\
             % false, on the lines the outputs read. A logic gate is faulty with\n\
             % probability 1 - p-clean, and a fault flips its output. The circuit fails\n\
             % where an output, or the next value of a register the outputs read, has a\n\
             % bit that is one; the failure is the evidence, its probability P(e), and\n\
             % each single, pair and triple of faults a query.\n\
             % Inputs (wire/bits):{}. Outputs:{}. Registers read:{}.",
            self.name,
            ids(&self.inputs),
            ids(&self.outputs),
            ids(&registers)
        )
        .expect("a String takes any text");
        for l in self.lines.iter().filter(|l| live.contains(&l.id)) {
            let mut parts = 0;
            match &l.expr {
                Expr::Next(e) if is_register.contains(&l.id) => {
                    let x = w.node(e, l.id, &mut parts);
                    w.fact(format_args!("next({}, {x})", l.id));
                }
                Expr::Assign(e) | Expr::Next(e) => w.wiring(&l.id.to_string(), e),
                e @ (Expr::Const(_) | Expr::Select(..) | Expr::Concat(_)) => {
                    w.wiring(&l.id.to_string(), e)
                }
                Expr::Wire(_) => unreachable!("a line's expression is an operation"),
                Expr::Logic(kind, es) => {
                    let ops: Vec<String> = es.iter().map(|e| w.node(e, l.id, &mut parts)).collect();
                    w.fact(format_args!(
                        "{}({}, {})",
                        kind.fact(),
                        l.id,
                        ops.join(", ")
                    ));
                    if !w.kinds.contains(kind) {
                        w.kinds.push(*kind);
                    }
                }
                Expr::Mux(es) => {
                    let ops: Vec<String> = es
                        .iter()
                        .map(|e| w.mux_operand(e, l.id, &mut parts))
                        .collect();
                    w.fact(format_args!("mux({}, {})", l.id, ops.join(", ")));
                    w.muxes = true;
                }
            }
        }
        for (id, _) in &self.outputs {
            w.fact(format_args!("output({id})"));
        }
        let mut out = head + &w.out;
        let mut rule = |r: &str| {
            out.push_str(r);
            out.push('\n');
        };
        let gates = !w.kinds.is_empty();
        if gates {
            rule("% A logic gate's value, before its fault.");
            for kind in [Logic::And, Logic::Or, Logic::Xor, Logic::Not] {
                if !w.kinds.contains(&kind) {
                    continue;
                }
                let fact = kind.fact();
                let args = if kind == Logic::Not { "_" } else { "_, _" };
                rule(&format!("gate(G) :- {fact}(G, {args})."));
                let bodies: &[&str] = match kind {
                    Logic::And => &["gate_and(G, A, B), bit(A, 0), bit(B, 0)"],
                    Logic::Or => &["gate_or(G, A, _), bit(A, 0)", "gate_or(G, _, B), bit(B, 0)"],
                    Logic::Xor => &[
                        "gate_xor(G, A, B), bit(A, 0), \\+ bit(B, 0)",
                        "gate_xor(G, A, B), \\+ bit(A, 0), bit(B, 0)",
                    ],
                    Logic::Not => &["gate_not(G, A), \\+ bit(A, 0)"],
                };
                for body in bodies {
                    rule(&format!("value(G) :- {body}."));
                }
            }
            rule("% Each gate's fault flips its value.");
            rule(&format!("{}::fault(G) :- gate(G).", float_repr(self.fault)));
            rule("bit(G, 0) :- value(G), \\+ fault(G).");
            rule("bit(G, 0) :- fault(G), \\+ value(G).");
        }
        if w.muxed_inputs {
            rule("% A multiplexer reads an input's bits, through any wiring, as ones; a gate,");
            rule("% an output and a register read them as false.");
        }
        if w.muxes {
            rule("% A multiplexer's word is its second input's where its select is one.");
            rule("bit(W, I) :- mux(W, S, A, _), \\+ bit(S, 0), bit(A, I).");
            rule("bit(W, I) :- mux(W, S, _, B), bit(S, 0), bit(B, I).");
        }
        if w.wires {
            rule("% Bit I of W is bit J of A.");
            rule("bit(W, I) :- wire(W, I, A, J), bit(A, J).");
        }
        rule("% The failure: an output or a register's next value with a bit that is one.");
        if !(gates || w.muxes || w.wires || w.ones) {
            // No bit is ever one, and bit/2 has no clause to read.
            rule("failure :- fail.");
        } else {
            if !self.outputs.is_empty() {
                rule("failure :- output(W), bit(W, _).");
            }
            if !registers.is_empty() {
                rule("failure :- next(_, X), bit(X, _).");
            }
        }
        rule("evidence(failure).");
        if gates {
            rule("% The posterior of each fault, and of each pair and triple of faults.");
            rule("query(fault(G)) :- gate(G).");
            rule("query(faults(A, B)) :- gate(A), gate(B), A < B.");
            rule("query(faults(A, B, C)) :- gate(A), gate(B), A < B, gate(C), B < C.");
            rule("faults(A, B) :- fault(A), fault(B).");
            rule("faults(A, B, C) :- fault(A), fault(B), fault(C).");
        }
        Ok(out)
    }
}

/// The netlist's facts as the program writes them, and which kinds of
/// rules they need.
struct Writer<'w> {
    widths: &'w FxHashMap<u64, usize>,
    /// The expression of each line that is wiring, by its wire.
    wiring: FxHashMap<u64, &'w Expr>,
    inputs: FxHashSet<u64>,
    out: String,
    /// The kinds of logic gates the netlist has, in the order they come.
    kinds: Vec<Logic>,
    wires: bool,
    ones: bool,
    muxes: bool,
    /// Whether a multiplexer reads an input's bit.
    muxed_inputs: bool,
}

impl Writer<'_> {
    fn fact(&mut self, f: std::fmt::Arguments<'_>) {
        writeln!(self.out, "{f}.").expect("a String takes any text");
    }

    /// The node of `e`, an operand on the line of `line`: a wire's id, or a
    /// wire of its own, `e(LINE, K)` for the `K`-th such operand of the
    /// line, wired as `e` is.
    fn node(&mut self, e: &Expr, line: u64, parts: &mut usize) -> String {
        if let Expr::Wire(id) = e {
            return id.to_string();
        }
        *parts += 1;
        let node = format!("e({line}, {parts})");
        self.wiring(&node, e);
        node
    }

    /// The node of `e`, an operand of the multiplexer on the line of
    /// `line`, as [`Writer::node`] has it; but where a bit of `e` is an
    /// input's, through any wiring, Circulette's multiplexer reads it as a
    /// one, and the operand is a wire of its own with ones for those bits.
    fn mux_operand(&mut self, e: &Expr, line: u64, parts: &mut usize) -> String {
        let mut bits = Vec::new();
        self.bits(e, &mut bits);
        let mut read = false;
        for b in &mut bits {
            if let Source::Of(a, j) = *b
                && self.is_input_bit(a, j)
            {
                *b = Source::Const(true);
                read = true;
            }
        }
        if !read {
            return self.node(e, line, parts);
        }
        self.muxed_inputs = true;
        *parts += 1;
        let node = format!("e({line}, {parts})");
        self.bit_facts(&node, bits);
        node
    }

    /// Whether bit `j` of wire `a` is an input's bit, through the lines
    /// that are wiring.
    fn is_input_bit(&self, mut a: u64, mut j: usize) -> bool {
        let mut bits = Vec::new();
        loop {
            if self.inputs.contains(&a) {
                return true;
            }
            let Some(e) = self.wiring.get(&a) else {
                return false;
            };
            bits.clear();
            self.bits(e, &mut bits);
            match bits[j] {
                Source::Of(b, k) => (a, j) = (b, k),
                Source::Const(_) => return false,
            }
        }
    }

    /// The facts that wire `node` as `e`, wiring.
    fn wiring(&mut self, node: &str, e: &Expr) {
        let mut bits = Vec::new();
        self.bits(e, &mut bits);
        self.bit_facts(node, bits);
    }

    /// The facts that wire `node` as `bits`, the least significant first: a
    /// `wire` fact for each bit read from another wire, a `bit` fact for
    /// each constant one.
    fn bit_facts(&mut self, node: &str, bits: Vec<Source>) {
        for (i, b) in bits.into_iter().enumerate() {
            match b {
                Source::Of(a, j) => {
                    self.wires = true;
                    self.fact(format_args!("wire({node}, {i}, {a}, {j})"));
                }
                Source::Const(true) => {
                    self.ones = true;
                    self.fact(format_args!("bit({node}, {i})"));
                }
                Source::Const(false) => {}
            }
        }
    }

    /// The bits of the wiring `e`, the least significant first.
    fn bits(&self, e: &Expr, out: &mut Vec<Source>) {
        match e {
            Expr::Wire(w) => out.extend((0..self.widths[w]).map(|j| Source::Of(*w, j))),
            Expr::Const(bits) => out.extend(bits.iter().map(|&b| Source::Const(b))),
            Expr::Select(e, i) => {
                let mut of = Vec::new();
                self.bits(e, &mut of);
                out.push(of[*i]);
            }
            Expr::Concat(es) => es.iter().for_each(|e| self.bits(e, out)),
            _ => unreachable!("wiring"),
        }
    }
}

/// Where a bit of wiring comes from: a bit of a wire, or a constant.
#[derive(Clone, Copy)]
enum Source {
    Of(u64, usize),
    Const(bool),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A netlist of `lines` with one-bit inputs 0 and 1 and the one-bit
    /// output 2, and `rest` after it.
    fn netlist(lines: &str, rest: &str) -> String {
        format!(
            "(def-netlist t (hash 0 1 1 1) (hash 2 1) (hash)\n{lines})\n\
             (define p-clean 0.99)\n(define expected-output (list #f))\n{rest}"
        )
    }

    /// A half adder's program, whole: its facts in the order of its lines,
    /// the rules of the kinds of gates it has, and Circulette's queries.
    #[test]
    fn a_half_adder() {
        let text = "\
            (def-netlist half (hash 0 1 1 1) (hash 2 1 3 1) (hash)\n\
              (4 (w^ 0 1)) (5 (w& 0 1)) (2 (<w= 4)) (3 (<w= 5)))\n\
            (define p-clean 0.99)\n\
            (define expected-output (list #f #f))\n";
        let expected = "\
% The Circulette netlist half, one clock cycle with every input and register
% false, on the lines the outputs read. A logic gate is faulty with
% probability 1 - p-clean, and a fault flips its output. The circuit fails
% where an output, or the next value of a register the outputs read, has a
% bit that is one; the failure is the evidence, its probability P(e), and
% each single, pair and triple of faults a query.
% Inputs (wire/bits): 0/1 1/1. Outputs: 2/1 3/1. Registers read: none.
gate_xor(4, 0, 1).
gate_and(5, 0, 1).
wire(2, 0, 4, 0).
wire(3, 0, 5, 0).
output(2).
output(3).
% A logic gate's value, before its fault.
gate(G) :- gate_and(G, _, _).
value(G) :- gate_and(G, A, B), bit(A, 0), bit(B, 0).
gate(G) :- gate_xor(G, _, _).
value(G) :- gate_xor(G, A, B), bit(A, 0), \\+ bit(B, 0).
value(G) :- gate_xor(G, A, B), \\+ bit(A, 0), bit(B, 0).
% Each gate's fault flips its value.
0.010000000000000009::fault(G) :- gate(G).
bit(G, 0) :- value(G), \\+ fault(G).
bit(G, 0) :- fault(G), \\+ value(G).
% Bit I of W is bit J of A.
bit(W, I) :- wire(W, I, A, J), bit(A, J).
% The failure: an output or a register's next value with a bit that is one.
failure :- output(W), bit(W, _).
evidence(failure).
% The posterior of each fault, and of each pair and triple of faults.
query(fault(G)) :- gate(G).
query(faults(A, B)) :- gate(A), gate(B), A < B.
query(faults(A, B, C)) :- gate(A), gate(B), A < B, gate(C), B < C.
faults(A, B) :- fault(A), fault(B).
faults(A, B, C) :- fault(A), fault(B), fault(C).
";
        assert_eq!(read(text).unwrap(), expected);
    }

    /// Wiring is composed as it is read: a selection of a concatenation is
    /// the bit it selects, a constant's ones are `bit` facts, and an
    /// operand that is not a wire is a wire of its own on its gate's line.
    #[test]
    fn wiring_is_composed() {
        let text = netlist(
            "(6 (w~ 0))\n\
             (3 (wc (list 6 (bv-const 2 2) 1)))\n\
             (4 (ws (wc (list 1 3)) (list 1)))\n\
             (5 (wx 4 (ws (wc (list 0 (bv-const 1 1))) (list 1)) (ws 3 (list 2))))\n\
             (2 (<w= 5))",
            "",
        );
        let program = read(&text).unwrap();
        let facts = [
            "gate_not(6, 0).",
            // 3 is gate 6, a zero, a one and wire 1, from its bit 0 up.
            "wire(3, 0, 6, 0).",
            "bit(3, 2).",
            "wire(3, 3, 1, 0).",
            // Bit 1 of (1, 3) is bit 0 of 3.
            "wire(4, 0, 3, 0).",
            // The multiplexer's operands before it, bit 1 of (0, 1) a one.
            "bit(e(5, 1), 0).",
            "wire(e(5, 2), 0, 3, 2).",
            "mux(5, 4, e(5, 1), e(5, 2)).",
            "wire(2, 0, 5, 0).",
            "output(2).",
        ];
        let written: Vec<&str> = program
            .lines()
            .filter(|l| !l.starts_with('%'))
            .take(facts.len())
            .collect();
        assert_eq!(written, facts, "{program}");
    }

    /// Only the lines the outputs read are written: a gate no output reads
    /// is no fault, and a register no output reads is not observed.
    #[test]
    fn the_outputs_slice() {
        let text = "\
            (def-netlist t (hash 0 1) (hash 2 1) (hash 5 1 6 1)\n\
              (3 (w~ 0)) (4 (w& 0 0)) (5 (<<= 3)) (6 (<<= 4)) (7 (w~ 5)) (2 (<w= 7)))\n\
            (define p-clean 0.99)\n\
            (define expected-output (list #f #f #f))\n";
        let program = read(text).unwrap();
        let facts: Vec<&str> = program.lines().filter(|l| !l.starts_with('%')).collect();
        assert_eq!(
            facts[..5],
            [
                "gate_not(3, 0).",
                "next(5, 3).",
                "gate_not(7, 5).",
                "wire(2, 0, 7, 0).",
                "output(2).",
            ],
            "{program}"
        );
        assert!(program.contains("Registers read: 5/1.\n"), "{program}");
        assert!(
            !program.contains("gate_and") && !program.contains("next(6"),
            "{program}"
        );
    }

    /// A multiplexer reads an input's bits as ones, directly or through
    /// wiring, where a gate reads them as false.
    #[test]
    fn a_multiplexer_reads_inputs_as_ones() {
        let text = netlist("(3 (<w= 0)) (4 (w~ 1)) (5 (wx 3 4 1)) (2 (<w= 5))", "");
        let program = read(&text).unwrap();
        let facts: Vec<&str> = program.lines().filter(|l| !l.starts_with('%')).collect();
        assert_eq!(
            facts[..7],
            [
                "wire(3, 0, 0, 0).",
                "gate_not(4, 1).",
                "bit(e(5, 1), 0).",
                "bit(e(5, 2), 0).",
                "mux(5, e(5, 1), 4, e(5, 2)).",
                "wire(2, 0, 5, 0).",
                "output(2).",
            ],
            "{program}"
        );
        assert!(
            program.contains("% A multiplexer reads an input's bits"),
            "{program}"
        );
    }

    /// Racket's other ways to write the data: brackets, nested block
    /// comments, `#hash` tables, quoted lists, `#true` and `#false`.
    #[test]
    fn racket_syntax() {
        let text = "#| a #| nested |# comment |#\n\
            [def-netlist t #hash((0 . 1)) #hash((2 . 1)) #hash()\n\
              (2 [w~ 0])]\n\
            (define p-clean 0.99)\n\
            (define expected-output '(#false))\n\
            (define internal-signals (hash 2))\n\
            (sketch-reroll t (loops))\n";
        let program = read(text).unwrap();
        assert!(program.contains("gate_not(2, 0).\n"), "{program}");
        assert!(
            program.contains("value(G) :- gate_not(G, A), \\+ bit(A, 0).\n"),
            "{program}"
        );
    }

    /// What the reader does not take is refused on its line.
    #[test]
    fn what_the_reader_does_not_take_is_refused() {
        let gate = "(3 (w& 0 1)) (2 (<w= 3))";
        for (text, line, error) in [
            (
                netlist(gate, "(define p-faulty 0.02)"),
                5,
                "p-faulty 0.02 is not 1 - p-clean",
            ),
            (
                netlist(gate, "").replace("(list #f)", "(list #t)"),
                4,
                "an expected value of true",
            ),
            (
                netlist(gate, "").replace("(list #f)", "(list #f #f)"),
                4,
                "has 2 values for the 1 bits",
            ),
            (
                netlist(gate, "(define x 1)"),
                5,
                "(define x ...) is not read",
            ),
            (
                netlist(gate, "(require circulette)"),
                5,
                "forms are def-netlist, define",
            ),
            (
                netlist(gate, "(sketch-reroll t (loops (3 4)))"),
                5,
                "with no loops",
            ),
            (
                netlist("(3 (w& 0 1 0)) (2 (<w= 3))", ""),
                2,
                "`w&` takes 2 argument(s), not 3",
            ),
            (
                netlist("(3 (w+ 0 1)) (2 (<w= 3))", ""),
                2,
                "`w+` is not an operator",
            ),
            (
                netlist("(2 (<w= (w& 0 1)))", ""),
                2,
                "`w&` inside an expression",
            ),
            (
                netlist("(2 (w& (wx 0 1 1) 1))", ""),
                2,
                "`wx` inside an expression",
            ),
            (
                netlist("(2 0)", ""),
                2,
                "a wire's expression is an operation",
            ),
            (
                netlist("(3 (wc (list 0 1))) (2 (w~ 3))", ""),
                2,
                "a logic gate on a word of 2 bits",
            ),
            (
                netlist("(3 (wc (list 0 1))) (2 (ws 3 (list 0 1)))", ""),
                2,
                "of several bits",
            ),
            (
                netlist("(2 (ws 0 (list 1)))", ""),
                2,
                "bit 1 of a word of 1 bits",
            ),
            (
                netlist("(2 (wx (wc (list 0 1)) 0 1))", ""),
                2,
                "a multiplexer selects with one bit",
            ),
            (
                netlist("(2 (<w= (bv-const 2 1)))", ""),
                2,
                "(bv-const 2 1) is no constant",
            ),
            (
                netlist("(3 (w~ 4)) (4 (w~ 3)) (2 (<w= 3))", ""),
                2,
                "on a combinational cycle",
            ),
            (
                netlist("(2 (w~ 9))", ""),
                2,
                "wire 9 is read and not defined",
            ),
            (
                netlist("(2 (w~ 0)) (2 (w~ 1))", ""),
                2,
                "wire 2 is defined twice",
            ),
            (
                netlist("(0 (w~ 1)) (2 (w~ 1))", ""),
                2,
                "wire 0 is an input",
            ),
            (netlist("(3 (w~ 1))", ""), 1, "wire 2 is not defined"),
            (
                netlist("(2 (<w= (wc (list 0 1))))", ""),
                2,
                "output 2 is declared 1 bits wide",
            ),
            (
                netlist(gate, "").replace("(hash 2 1) (hash)", "(hash) (hash)"),
                1,
                "observes nothing",
            ),
            (
                netlist(gate, "")
                    .replace("(hash 2 1) (hash)", "(hash 2 1) (hash 4 1)\n(4 (<w= 0))")
                    .replace("(list #f)", "(list #f #f)"),
                2,
                "register 4 is defined by `<<=`",
            ),
            (
                netlist(gate, "").replace("(hash 2 1)", "(hash 1 1)"),
                1,
                "wire 1 is declared twice",
            ),
            (
                netlist(gate, "").replace("(hash 2 1)", "(hash 2 0)"),
                1,
                "wire 2 has no bits",
            ),
            (
                netlist(gate, "").replace("(hash 2 1)", "(hash 2)"),
                1,
                "the outputs is a table",
            ),
            (
                netlist(gate, "").replace("(define p-clean 0.99)\n", ""),
                1,
                "no (define p-clean ...)",
            ),
            (
                netlist(gate, "(define p-clean 0.9)"),
                5,
                "a second definition of p-clean",
            ),
            (
                netlist(gate, "").replace("0.99", "1.5"),
                3,
                "p-clean 1.5 is not a probability",
            ),
            (
                netlist(gate, "").replace("0.99", "x"),
                3,
                "p-clean is a number",
            ),
            (
                netlist(gate, "(def-netlist u (hash) (hash) (hash))"),
                5,
                "a second `def-netlist`",
            ),
            ("(define p-clean 0.99)".to_string(), 1, "no `def-netlist`"),
            (netlist(gate, "\"text\""), 5, "a string"),
            (netlist(gate, "#rx\"a\""), 5, "`#rx` is not read"),
            (netlist(gate, "(define x"), 5, "a list without its end"),
            (netlist(gate, ")"), 5, "`)` closes nothing"),
            (netlist(gate, "(list 1]"), 5, "`]` closes nothing"),
            (netlist(gate, "#| open"), 5, "without its `|#`"),
            (
                format!("{}0{}", "(".repeat(300), ")".repeat(300)),
                1,
                "nested deeper than 256",
            ),
        ] {
            let e = read(&text).expect_err(&text);
            assert!(e.msg.contains(error), "{text}: {e}");
            assert_eq!(e.line, line, "{text}: {e}");
        }
    }
}
