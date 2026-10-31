//! LPADs written for cplint and PITA ([`read`]): a source that holds the
//! directive `:- begin_lpad.` (or cplint's other spelling,
//! `:- begin_plp.`) is read as the ProbLog program this module writes for
//! it, which `tddlog --program` prints and the parser then reads
//! ([`crate::read`] finds the directive in a ProbLog input).
//!
//! The LPAD is the clauses between `:- begin_lpad.` and `:- end_lpad.`.
//! Its clauses `h1:p1 ; h2:p2 :- body.` are written `p1::h1; p2::h2 :-
//! body.`, and `a:p.` is written `p::a.`; a head without a probability
//! takes what the others leave. A probability that is not a number, a term
//! the body binds (`male:M/P ; female:F/P :- ..., P is M + F.`), is a
//! flexible probability and is written as it is. The clauses outside the
//! markers are cplint's database clauses, plain Prolog, which an LPAD body
//! calls through `db(Goal)` or the goal of a built-in such as `findall/3`:
//! those the LPAD's bodies name, and those they name in turn, are kept as
//! they are, and `db(Goal)` is written `Goal`. The rest (plots, samplers and
//! other programs over the LPAD) is left out, as are the directives and
//! goals but for PITA's depth bound and PRISM's switches,
//! `:- set_sw(S, Ps).`, read as annotated disjunctions of `msw/2`; a
//! source written for PITA(IND), whose assumptions tddlog's inference
//! does not take, is refused. The example queries `?- prob(Q, P).` in the
//! file become
//! `query(Q).`, and `?- prob(Q, E, P).` becomes `query(Q).` and an
//! `evidence(L).` for each literal `L` of the conjunction `E`. A program
//! holds one set of evidence: the queries asked under the evidence of the
//! file's first example query are written, and the others are named in a
//! comment and in [`Lpad::left_out`], as is a query under an intervention
//! `do(L)`, which ProbLog does not read. Everything else is passed
//! through, so that a clause that uses what tddlog does not read stops the
//! parser with its message. The program imports `library(lists)` where
//! it calls a predicate of it that it does not define, as ProbLog loads
//! the library only when asked.
//!
//! The clauses are found in the text as cplint's examples write them, as
//! the Python script `lpad2problog.py` (on the baselines branch) found
//! them before this reader replaced it, and the program is the one that
//! script wrote but for that import: `/* */` and `%` comments are
//! dropped, a clause ends at a `.` followed by white space outside quotes
//! and brackets, and the names a body mentions are the words outside
//! quotes that start with a lower-case letter.

use rustc_hash::{FxHashMap, FxHashSet};

use super::{Atom, Builtin, CONS, Compound, Literal, NIL, ParseError, Program, Term};
use crate::network::rows::{fsum, write_float};

/// An LPAD read as a ProbLog program.
#[derive(Clone, Debug, PartialEq)]
pub struct Lpad {
    /// The program, as `tddlog --program` prints it.
    pub program: String,
    /// The example queries the program leaves out, each with why, as its
    /// comments name them.
    pub left_out: Vec<String>,
}

/// The ProbLog program `source` is read as, where `source` holds the
/// directive `:- begin_lpad.` or `:- begin_plp.`; `None` for any other
/// source. The error is a clause the program cannot write: a disjunctive
/// head with no probabilities, with two heads without one or with one
/// beside a flexible probability, or a probability that is not a finite
/// number.
pub fn read(source: &str) -> Option<Result<Lpad, ParseError>> {
    if !(source.contains("begin_lpad") || source.contains("begin_plp")) {
        return None;
    }
    let raw = Stripped::of(source);
    let clauses = split_clauses(&raw.text);
    if !clauses.iter().any(|&(_, c)| marker(c) == Some(true)) {
        return None;
    }
    Some(convert(source, &raw, &clauses))
}

fn convert(source: &str, raw: &Stripped, clauses: &[(usize, &str)]) -> Result<Lpad, ParseError> {
    // (clause, in the LPAD, where it starts in the stripped text), in the
    // order of the file, directives and goals left out.
    let mut read: Vec<(String, bool, usize)> = Vec::new();
    let mut inside = false;
    for &(at, c) in clauses {
        if let Some(begin) = marker(c) {
            inside = begin;
        } else if !c.starts_with(":-") && !c.starts_with("?-") {
            let c = if inside { without_db(c) } else { c.to_string() };
            read.push((c, inside, at));
        }
    }
    // The database clauses the LPAD's bodies name, and those they name.
    let mut named: FxHashSet<String> = FxHashSet::default();
    for (c, lpad, _) in &read {
        if *lpad {
            mentioned(body(c), &mut named);
        }
    }
    let mut kept: Vec<bool> = read.iter().map(|(_, lpad, _)| *lpad).collect();
    let mut grew = true;
    while grew {
        grew = false;
        for (k, (c, _, _)) in read.iter().enumerate() {
            if !kept[k] && head_name(c).is_some_and(|h| named.contains(h)) {
                kept[k] = true;
                grew = true;
                mentioned(body(c), &mut named);
            }
        }
    }
    let at_line = |(at, msg): (usize, String)| ParseError {
        line: raw.line(source, at),
        msg,
    };
    let (depth, memo) = settings(clauses).map_err(at_line)?;
    // The LPAD's predicates, which take the depth under PITA's bound.
    let predicates: FxHashSet<(String, usize)> = read
        .iter()
        .filter(|(_, lpad, _)| *lpad)
        .flat_map(|(c, _, _)| {
            let head = c.split_once(":-").map_or(c.as_str(), |(h, _)| h);
            split_top(head, b';').into_iter().filter_map(|h| {
                let (atom, _) = head_prob(h).ok()?;
                atom_parts(atom).map(|(n, args)| (n.to_string(), args.len()))
            })
        })
        .collect();
    let mut out = Vec::new();
    // ProbLog loads `library(lists)` where the program asks for it.
    let defined: FxHashSet<&str> = read
        .iter()
        .zip(&kept)
        .filter(|(_, keep)| **keep)
        .filter_map(|((c, _, _), _)| head_name(c))
        .collect();
    if named
        .iter()
        .any(|n| Builtin::in_lists(n) && !defined.contains(n.as_str()))
    {
        out.push(":- use_module(library(lists)).".to_string());
    }
    let mut number = 0;
    for ((c, lpad_clause, at), _) in read.iter().zip(&kept).filter(|(_, keep)| **keep) {
        let error = |msg| ParseError {
            line: raw.line(source, *at),
            msg: format!("{msg} in the LPAD clause `{c}`"),
        };
        match depth {
            Some(_) if *lpad_clause => {
                number += 1;
                out.extend(bounded(c, number, &predicates).map_err(error)?);
            }
            _ => out.push(convert_clause(c).map_err(error)?),
        }
    }
    // PRISM's switches, as annotated disjunctions.
    let set_sw = directives(clauses, "set_sw");
    if let Some(&(at, _)) = set_sw.first() {
        if depth.is_some() {
            return Err(at_line((
                at,
                "PRISM's switches under PITA's depth bound".into(),
            )));
        }
        let mut values = Vec::new();
        for ((c, _, at), _) in read.iter().zip(&kept).filter(|(_, keep)| **keep) {
            match head_name(c) {
                Some("values") if !c.contains(":-") => values.push(c.as_str()),
                Some("values" | "msw") => {
                    return Err(at_line((*at, format!("`{c}` beside PRISM's switches"))));
                }
                _ => {}
            }
        }
        out.extend(switches(&set_sw, &values).map_err(at_line)?);
    }
    // One program holds one set of evidence: that of the first query not
    // asked under an intervention.
    let mut asked: Vec<(&str, Vec<&str>)> = Vec::new();
    for q in examples(source) {
        if !asked.contains(&q) {
            asked.push(q);
        }
    }
    let intervened = |e: &[&str]| e.iter().any(|lit| intervention(lit));
    let observed: Vec<&Vec<&str>> = asked
        .iter()
        .map(|(_, e)| e)
        .filter(|e| !intervened(e))
        .collect();
    let under: &[&str] = observed.first().map_or(&[], |e| e.as_slice());
    // Under the depth bound a query or an observation of a predicate of
    // the LPAD is of its atom at the bound.
    if let Some(depth) = depth {
        let mut bridged: Vec<(&str, usize)> = Vec::new();
        let asked_here = asked.iter().filter(|(_, e)| e == under).map(|(q, _)| *q);
        for lit in under.iter().copied().chain(asked_here) {
            let atom = lit.strip_prefix("\\+").map_or(lit, |a| unwrapped(trim(a)));
            let Some((name, args)) = atom_parts(atom) else {
                continue;
            };
            let n = args.len();
            if !predicates.contains(&(name.to_string(), n)) || bridged.contains(&(name, n)) {
                continue;
            }
            if n > 0 && predicates.contains(&(name.to_string(), n - 1)) {
                return Err(ParseError {
                    line: 1,
                    msg: format!(
                        "{name}/{n} asked under PITA's depth bound, where {name}/{} takes the depth as {name}/{n}",
                        n - 1
                    ),
                });
            }
            bridged.push((name, n));
            let vars: Vec<String> = (1..=n).map(|k| format!("X{k}")).collect();
            let at = vars
                .iter()
                .cloned()
                .chain([depth.to_string()])
                .collect::<Vec<_>>();
            let head = if n == 0 {
                name.to_string()
            } else {
                format!("{name}({})", vars.join(", "))
            };
            out.push(format!("{head} :- {name}({}).", at.join(", ")));
        }
    }
    out.extend(under.iter().map(|lit| format!("evidence({lit}).")));
    out.extend(
        asked
            .iter()
            .filter(|(_, e)| e == under)
            .map(|(q, _)| format!("query({q}).")),
    );
    let mut left_out = Vec::new();
    for (q, e) in &asked {
        if e != under {
            let goal = if e.is_empty() {
                format!("prob({q}, P)")
            } else {
                format!("prob({q}, ({}), P)", e.join(", "))
            };
            let why = if intervened(e) {
                "an intervention"
            } else {
                "other evidence"
            };
            let note = format!("left out, asked under {why}: {goal}");
            out.push(format!("% {note}"));
            left_out.push(note);
        }
    }
    let program = out.join("\n");
    // PITA's default makes a new choice at each call of msw/2.
    if let Some(&(at, _)) = set_sw.first()
        && !memo
        && let Ok(read) = super::parse_program(&program)
    {
        calls_once(&read, &predicates).map_err(|why| {
            at_line((
                at,
                format!(
                    "PRISM's msw/2 under PITA's default, which makes a new choice at each \
                     call: {why}; tddlog reads a switch as one choice, as PITA does under \
                     `:- set_pita(prism_memoization, true).`, and reads the default where \
                     each switch is called once"
                ),
            ))
        })?;
    }
    Ok(Lpad { program, left_out })
}

/// The arguments of the directives `:- name(..).` among `clauses`, each
/// with where it starts.
fn directives<'a>(clauses: &[(usize, &'a str)], name: &str) -> Vec<(usize, Vec<&'a str>)> {
    clauses
        .iter()
        .filter_map(|&(at, c)| {
            let (n, args) = atom_parts(trim(c.strip_prefix(":-")?))?;
            (n == name).then_some((at, args))
        })
        .collect()
}

/// The annotated disjunctions of PRISM's switches as cplint's PITA reads
/// them: the directive `:- set_sw(S, [p1, ..., pn]).` gives the switch
/// `S` the values of the fact `values(T, [v1, ..., vn])` of which `S` is
/// an instance, and `msw(S, V)` holds where `S` takes the value `V`:
/// `p1::msw(S, v1); ...; pn::msw(S, vn).`, each instance of `S` a choice
/// of its own. A switch without such a fact, with two, or with a fact
/// that holds only some of its instances, two directives for one
/// instance, and probabilities that are not numbers summing to one are
/// refused.
fn switches(
    set_sw: &[(usize, Vec<&str>)],
    values: &[&str],
) -> Result<Vec<String>, (usize, String)> {
    let mut text: String = values.iter().map(|v| format!("{v}.\n")).collect();
    for (at, args) in set_sw {
        match args.as_slice() {
            [s, _] => text.push_str(&format!("'$sw'({s}).\n")),
            _ => return Err((*at, "set_sw/1, which sets no probabilities".into())),
        }
    }
    let read = super::parse_program(&text)
        .map_err(|e| (set_sw[0].0, format!("PRISM's switches: {}", e.msg)))?;
    let facts = |name: &str| -> Vec<&Atom> {
        read.clauses
            .iter()
            .map(|c| &c.heads[0].atom)
            .filter(|a| &*a.name == name)
            .collect()
    };
    let (facts, sws) = (facts("values"), facts("$sw"));
    let mut out = Vec::new();
    for (k, ((at, args), sw)) in set_sw.iter().zip(&sws).enumerate() {
        let s = &sw.args[0];
        let error = |why: String| (*at, format!("`set_sw({s}, {})`: {why}", args[1]));
        let probs = trim(args[1])
            .strip_prefix('[')
            .and_then(|l| l.strip_suffix(']'))
            .map(|l| {
                split_top(l, b',')
                    .into_iter()
                    .map(number)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()
            .map_err(error)?
            .and_then(|ps| ps.into_iter().collect::<Option<Vec<f64>>>())
            .filter(|ps| ps.iter().all(|p| (0.0..=1.0).contains(p)))
            .ok_or_else(|| error("the probabilities are not a list of numbers".into()))?;
        if (fsum(&probs) - 1.0).abs() > 1e-6 {
            return Err(error("the probabilities do not sum to one".into()));
        }
        if let Some(other) = sws[..k].iter().find(|o| unifiable(&o.args[0], s)) {
            return Err(error(format!("{} is set too", other.args[0])));
        }
        let of: Vec<&&Atom> = facts
            .iter()
            .filter(|v| v.args.len() == 2 && unifiable(&v.args[0], s))
            .collect();
        let [v] = of.as_slice() else {
            return Err(error(format!("{} facts values({s}, _)", of.len())));
        };
        let mut binds = FxHashMap::default();
        if !matches(&v.args[0], s, &mut binds) {
            return Err(error(format!(
                "values/2 gives {} only some of its values",
                v.args[0]
            )));
        }
        let items = list_items(&v.args[1])
            .filter(|items| items.iter().all(|t| t.is_ground()))
            .ok_or_else(|| error(format!("the values {} are no list of constants", v.args[1])))?;
        if items.len() != probs.len() {
            return Err(error(format!(
                "{} values and {} probabilities",
                items.len(),
                probs.len()
            )));
        }
        let mut heads = Vec::with_capacity(probs.len());
        for (p, v) in probs.into_iter().zip(items) {
            let p = annotation(Written::Number(p)).map_err(error)?;
            heads.push(format!("{p}::msw({s}, {v})"));
        }
        out.push(format!("{}.", heads.join("; ")));
    }
    Ok(out)
}

/// The items of the list `t`, where it is a proper list.
fn list_items(mut t: &Term) -> Option<Vec<&Term>> {
    let mut items = Vec::new();
    loop {
        match t {
            Term::Const(c) if &**c == NIL => return Some(items),
            Term::Compound(c) if &*c.name == CONS && c.args.len() == 2 => {
                items.push(&c.args[0]);
                t = &c.args[1];
            }
            _ => return None,
        }
    }
}

/// Whether the terms `a` and `b`, their variables named apart, unify.
/// The switches are read before grounding, on the source's terms; the
/// grounder's unifiers work on its interned terms (`ground/engine.rs`,
/// `ground/engine/topdown.rs`) or write a unification as literals
/// (`ground/terms.rs`), so this and [`matches`] are the source terms' own.
fn unifiable(a: &Term, b: &Term) -> bool {
    fn apart(t: &Term) -> Term {
        match t {
            Term::Var(v) => Term::Var(format!("#{v}")),
            Term::Compound(c) => Compound::term(c.name.clone(), c.args.iter().map(apart).collect()),
            t => t.clone(),
        }
    }
    fn walk(t: &Term, s: &FxHashMap<String, Term>) -> Term {
        let mut t = t.clone();
        while let Term::Var(v) = &t {
            match s.get(v) {
                Some(u) => t = u.clone(),
                None => break,
            }
        }
        t
    }
    fn unify(a: &Term, b: &Term, s: &mut FxHashMap<String, Term>) -> bool {
        let (a, b) = (walk(a, s), walk(b, s));
        match (&a, &b) {
            (Term::Var(x), Term::Var(y)) if x == y => true,
            (Term::Var(x), _) => {
                s.insert(x.clone(), b);
                true
            }
            (_, Term::Var(y)) => {
                s.insert(y.clone(), a);
                true
            }
            (Term::Compound(c), Term::Compound(d)) => {
                c.name == d.name
                    && c.args.len() == d.args.len()
                    && c.args
                        .iter()
                        .zip(d.args.iter())
                        .all(|(x, y)| unify(x, y, s))
            }
            _ => a == b,
        }
    }
    unify(a, &apart(b), &mut FxHashMap::default())
}

/// Whether the term `s` is an instance of the term `t`, its variables
/// bound in `m` as they are matched.
fn matches(t: &Term, s: &Term, m: &mut FxHashMap<String, Term>) -> bool {
    match (t, s) {
        (Term::Var(v), _) => match m.get(v) {
            Some(u) => u == s,
            None => {
                m.insert(v.clone(), s.clone());
                true
            }
        },
        (Term::Compound(c), Term::Compound(d)) => {
            c.name == d.name
                && c.args.len() == d.args.len()
                && c.args
                    .iter()
                    .zip(d.args.iter())
                    .all(|(x, y)| matches(x, y, m))
        }
        _ => t == s,
    }
}

/// Whether the clause `c` is a marker: `Some(true)` for `:- begin_lpad`
/// or `:- begin_plp`, `Some(false)` for the ends.
fn marker(c: &str) -> Option<bool> {
    let rest = trim(c.strip_prefix(":-")?);
    let (begin, kind) = if let Some(k) = rest.strip_prefix("begin_") {
        (true, k)
    } else {
        (false, rest.strip_prefix("end_")?)
    };
    matches!(kind, "lpad" | "plp").then_some(begin)
}

/// Whether the literal `lit` is an intervention, `do(L)`.
fn intervention(lit: &str) -> bool {
    lit.strip_prefix("do")
        .is_some_and(|r| trim_start(r).starts_with('('))
}

/// Python's white space (`str.isspace`), which the clauses are trimmed of.
pub(super) fn space(c: char) -> bool {
    matches!(c, ' ' | '\t'..='\r' | '\x1c'..='\x1f') || (!c.is_ascii() && c.is_whitespace())
}

/// A character of a word (Python's `\w`).
fn word(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

fn trim(s: &str) -> &str {
    s.trim_matches(space)
}

fn trim_start(s: &str) -> &str {
    s.trim_start_matches(space)
}

/// Whether the character before byte `i` of `s` is not a word's: a word
/// starting at `i` starts there.
fn starts_word(s: &str, i: usize) -> bool {
    !s[..i].chars().next_back().is_some_and(word)
}

/// The source without its comments: `/* */` blocks first, over lines,
/// then each line from its first `%` not after a backslash. `text`'s lines
/// are joined with `\n`, split where Python's `str.splitlines` splits.
struct Stripped {
    text: String,
    /// Where each line of `text` starts in the source without its blocks.
    starts: Vec<usize>,
    /// Each block taken out: where it was in the source without the blocks
    /// before it, and its length.
    blocks: Vec<(usize, usize)>,
}

impl Stripped {
    fn of(source: &str) -> Stripped {
        let mut unblocked = String::with_capacity(source.len());
        let mut blocks = Vec::new();
        let mut rest = source;
        while let Some(i) = rest.find("/*") {
            let Some(j) = rest[i + 2..].find("*/") else {
                break;
            };
            unblocked.push_str(&rest[..i]);
            blocks.push((unblocked.len(), j + 4));
            rest = &rest[i + j + 4..];
        }
        unblocked.push_str(rest);
        let mut text = String::with_capacity(unblocked.len());
        let mut starts = Vec::new();
        for (start, line) in lines(&unblocked) {
            if !starts.is_empty() {
                text.push('\n');
            }
            starts.push(start);
            let b = line.as_bytes();
            let comment = (0..b.len()).find(|&k| b[k] == b'%' && (k == 0 || b[k - 1] != b'\\'));
            text.push_str(&line[..comment.unwrap_or(line.len())]);
        }
        Stripped {
            text,
            starts,
            blocks,
        }
    }

    /// The line of `source` where byte `at` of the stripped text comes
    /// from.
    fn line(&self, source: &str, at: usize) -> usize {
        let line = self.text[..at].matches('\n').count();
        let line_start = self.text[..at].rfind('\n').map_or(0, |k| k + 1);
        let mut pos = self.starts[line] + (at - line_start);
        pos += self
            .blocks
            .iter()
            .take_while(|&&(q, _)| q <= pos)
            .map(|&(_, len)| len)
            .sum::<usize>();
        1 + source[..pos.min(source.len())].matches('\n').count()
    }
}

/// The lines of `s` with where each starts, split where Python's
/// `str.splitlines` splits them (`\r\n` once), no empty line after the
/// last break.
fn lines(s: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if matches!(
            c,
            '\n' | '\r'
                | '\x0b'
                | '\x0c'
                | '\x1c'
                | '\x1d'
                | '\x1e'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        ) {
            out.push((start, &s[start..i]));
            let mut end = i + c.len_utf8();
            if c == '\r' && chars.peek().is_some_and(|&(_, d)| d == '\n') {
                chars.next();
                end += 1;
            }
            start = end;
        }
    }
    if start < s.len() {
        out.push((start, &s[start..]));
    }
    out
}

/// The clauses of `text`, each trimmed with where it starts: a clause ends
/// at a `.` followed by white space or the end, outside quotes and
/// brackets.
fn split_clauses(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let b = text.as_bytes();
    let (mut start, mut depth, mut quote) = (0, 0i64, None);
    let mut push = |from: usize, to: usize| {
        let c = &text[from..to];
        let t = trim(c);
        if !t.is_empty() {
            out.push((from + (c.len() - trim_start(c).len()), t));
        }
    };
    for i in 0..b.len() {
        let c = b[i];
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if c == b'\'' || c == b'"' {
            quote = Some(c);
        } else if c == b'(' || c == b'[' {
            depth += 1;
        } else if c == b')' || c == b']' {
            depth -= 1;
        } else if c == b'.' && depth == 0 && text[i + 1..].chars().next().is_none_or(space) {
            push(start, i);
            start = i + 1;
        }
    }
    push(start, b.len());
    out
}

/// `s` split at each `sep` outside quotes and brackets.
fn split_top(s: &str, sep: u8) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut start, mut depth, mut quote) = (0, 0i64, None);
    for (i, &c) in s.as_bytes().iter().enumerate() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        if c == b'\'' || c == b'"' {
            quote = Some(c);
        } else if c == b'(' || c == b'[' {
            depth += 1;
        } else if c == b')' || c == b']' {
            depth -= 1;
        }
        if c == sep && depth == 0 {
            parts.push(&s[start..i]);
            start = i + 1;
        }
    }
    parts.push(&s[start..]);
    parts
}

/// The text of `s` from `start` up to the bracket that closes the one
/// before it, or to the end.
fn balanced(s: &str, start: usize) -> &str {
    let mut depth = 1;
    for (i, &c) in s.as_bytes().iter().enumerate().skip(start) {
        if c == b'(' || c == b'[' {
            depth += 1;
        } else if c == b')' || c == b']' {
            depth -= 1;
            if depth == 0 {
                return &s[start..i];
            }
        }
    }
    &s[start..]
}

/// `s` without the parentheses around the whole of it.
fn unwrapped(mut s: &str) -> &str {
    while s.starts_with('(') && balanced(s, 1).len() + 2 == s.len() {
        s = trim(&s[1..s.len() - 1]);
    }
    s
}

/// `clause` with each call `db(Goal)` of a database goal written `Goal`.
fn without_db(clause: &str) -> String {
    let mut clause = clause.to_string();
    while let Some(i) = clause
        .match_indices("db(")
        .map(|(i, _)| i)
        .find(|&i| starts_word(&clause, i))
    {
        let goal = balanced(&clause, i + 3).to_string();
        let end = (i + 3 + goal.len() + 1).min(clause.len());
        clause = format!("{}{goal}{}", &clause[..i], &clause[end..]);
    }
    clause
}

/// The body of `clause`, after its first `:-`.
fn body(clause: &str) -> &str {
    clause.split_once(":-").map_or("", |(_, b)| b)
}

/// The name a word that starts with a lower-case letter at byte `i` of
/// `s` is, if one does.
fn name_at(s: &str, i: usize) -> Option<&str> {
    if !s.as_bytes()[i].is_ascii_lowercase() || !starts_word(s, i) {
        return None;
    }
    let len = s[i..].find(|c| !word(c)).unwrap_or(s.len() - i);
    Some(&s[i..i + len])
}

/// The name of `clause`'s head.
fn head_name(clause: &str) -> Option<&str> {
    let head = trim(clause.split_once(":-").map_or(clause, |(h, _)| h));
    (!head.is_empty()).then(|| name_at(head, 0)).flatten()
}

/// Adds to `names` the names `text` mentions outside quotes, which hold
/// those of the predicates it calls. What is in quotes is taken out first,
/// so that the text on either side meets.
fn mentioned(text: &str, names: &mut FxHashSet<String>) {
    let mut unquoted = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(['\'', '"']) {
        let quote = rest.as_bytes()[i] as char;
        match rest[i + 1..].find(quote) {
            Some(len) => {
                unquoted.push_str(&rest[..i]);
                rest = &rest[i + len + 2..];
            }
            None => {
                unquoted.push_str(&rest[..=i]);
                rest = &rest[i + 1..];
            }
        }
    }
    unquoted.push_str(rest);
    let mut i = 0;
    while i < unquoted.len() {
        match name_at(&unquoted, i) {
            Some(name) => {
                names.insert(name.to_string());
                i += name.len();
            }
            None => i += unquoted[i..].chars().next().map_or(1, char::len_utf8),
        }
    }
}

/// The ProbLog clause of an LPAD clause, or what it cannot write.
fn convert_clause(clause: &str) -> Result<String, String> {
    let (head, tail) = match clause.split_once(":-") {
        Some((h, b)) => (h, format!(" :- {}", trim(b))),
        None => (clause, String::new()),
    };
    let mut lhs = String::new();
    for (k, (atom, p)) in heads(head)?.into_iter().enumerate() {
        if k > 0 {
            lhs.push_str("; ");
        }
        if let Some(p) = p {
            lhs.push_str(&annotation(p)?);
            lhs.push_str("::");
        }
        lhs.push_str(atom);
    }
    Ok(format!("{lhs}{tail}."))
}

/// The heads of an LPAD clause's head `head`, each with its probability
/// where any has one, a head without one taking what the others leave; or
/// what the program cannot write.
fn heads(head: &str) -> Result<Vec<(&str, Option<Written<'_>>)>, String> {
    let mut heads = split_top(head, b';')
        .into_iter()
        .map(head_prob)
        .collect::<Result<Vec<_>, _>>()?;
    if heads.iter().all(|(_, p)| p.is_none()) {
        return match heads.len() {
            1 => Ok(heads),
            _ => Err("a disjunctive head without probabilities".into()),
        };
    }
    // A flexible probability is written as it is, and what it leaves to a
    // head without one is not a number the reader knows.
    let mut given = Vec::with_capacity(heads.len());
    for (_, p) in &heads {
        match p {
            Some(Written::Number(p)) => given.push(*p),
            Some(Written::Term(_)) if heads.iter().any(|(_, p)| p.is_none()) => {
                return Err("a head without a probability beside a flexible one".into());
            }
            _ => {}
        }
    }
    let total = fsum(&given);
    let mut missing = heads.iter_mut().filter(|(_, p)| p.is_none());
    if let Some(rest) = missing.next() {
        if missing.next().is_some() {
            return Err("two heads without a probability".into());
        }
        let left = 1.0 - total;
        rest.1 = Some(Written::Number(if left > 0.0 { left } else { 0.0 }));
    }
    Ok(heads)
}

/// The probability `p` as ProbLog's annotation writes it.
fn annotation(p: Written) -> Result<String, String> {
    Ok(match p {
        Written::Term(t) => t.to_string(),
        Written::Number(p) if !p.is_finite() => return Err(format!("the probability {p}")),
        Written::Number(p) if p == p.trunc() => format!("{p:.1}"),
        Written::Number(p) => {
            let mut s = String::new();
            write_float(&mut s, p);
            s
        }
    })
}

/// Why PITA's default reading of `msw/2`, a new choice at each call, may
/// differ from one choice for each instance of a switch, the annotated
/// disjunctions [`switches`] writes; `Ok` where it cannot. It cannot where,
/// in PITA's evaluation of each query under the evidence, which tables the
/// predicates of the LPAD (`tabled`), each instance of a switch is called
/// once:
/// - the clause of a call `msw(S, V)` has one head, of a predicate called
///   in one way: by the query or an observation alone, or, not recursive,
///   from one place in a clause of such a predicate, with arguments that
///   the call binds or that nothing before the place does;
/// - the literals before the call are calls of the LPAD's predicates,
///   whose answers PITA tables once each, or tests and evaluations, and
///   what they bind and the call does not is in `S`, which they ground;
/// - no two calls of `msw/2` have switches that unify, and none is under a
///   negation or in a findall.
fn calls_once(program: &Program, tabled: &FxHashSet<(String, usize)>) -> Result<(), String> {
    if !program.query_rules.is_empty() {
        return Err("a query rule".into());
    }
    let calls = Calls::of(program);
    for (q, _) in &program.queries {
        let roots: Vec<&Atom> = std::iter::once(q)
            .chain(program.evidence.iter().map(|(a, _)| a))
            .collect();
        calls.check(&roots, tabled)?;
    }
    Ok(())
}

type Pred<'a> = (&'a str, usize);

/// A call in a clause's body: the clause, the place in the body, the atom,
/// and whether a findall makes it, once for each solution of its goal.
struct Call<'p> {
    clause: usize,
    at: usize,
    atom: &'p Atom,
    repeated: bool,
}

/// The calls of a program's clauses ([`calls_once`]).
struct Calls<'p> {
    program: &'p Program,
    by_head: FxHashMap<Pred<'p>, Vec<usize>>,
    calls: Vec<Call<'p>>,
}

impl<'p> Calls<'p> {
    fn of(program: &'p Program) -> Self {
        fn atoms<'p>(
            body: &'p [Literal],
            at: Option<usize>,
            out: &mut Vec<(usize, &'p Atom, bool)>,
        ) {
            for (i, l) in body.iter().enumerate() {
                let place = at.unwrap_or(i);
                match l {
                    Literal::Pos(a) | Literal::Neg(a) => out.push((place, a, at.is_some())),
                    Literal::Findall(f) => {
                        for conj in &f.goal {
                            atoms(conj, Some(place), out);
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut by_head: FxHashMap<Pred, Vec<usize>> = FxHashMap::default();
        let mut calls = Vec::new();
        for (k, c) in program.clauses.iter().enumerate() {
            for h in &c.heads {
                let ks = by_head.entry(h.atom.signature()).or_default();
                if ks.last() != Some(&k) {
                    ks.push(k);
                }
            }
            let mut out = Vec::new();
            atoms(&c.body, None, &mut out);
            calls.extend(out.into_iter().map(|(at, atom, repeated)| Call {
                clause: k,
                at,
                atom,
                repeated,
            }));
        }
        Calls {
            program,
            by_head,
            calls,
        }
    }

    /// The clauses of the predicates `from` reaches through the calls of
    /// their clauses.
    fn reach(&self, mut from: Vec<Pred<'p>>) -> FxHashSet<usize> {
        let (mut seen, mut clauses) = (FxHashSet::default(), FxHashSet::default());
        while let Some(p) = from.pop() {
            if seen.insert(p) {
                for &k in self.by_head.get(&p).into_iter().flatten() {
                    clauses.insert(k);
                    from.extend(
                        self.calls
                            .iter()
                            .filter(|c| c.clause == k)
                            .map(|c| c.atom.signature()),
                    );
                }
            }
        }
        clauses
    }

    fn recursive(&self, p: Pred<'p>) -> bool {
        let callees = self
            .calls
            .iter()
            .filter(|c| {
                self.program.clauses[c.clause]
                    .heads
                    .iter()
                    .any(|h| h.atom.signature() == p)
            })
            .map(|c| c.atom.signature())
            .collect();
        self.reach(callees).into_iter().any(|k| {
            self.program.clauses[k]
                .heads
                .iter()
                .any(|h| h.atom.signature() == p)
        })
    }

    /// How the evaluation of `roots`, which reaches the clauses `reached`,
    /// calls `p`, where it calls it in one way: each argument bound or not.
    fn way(
        &self,
        p: Pred<'p>,
        roots: &[&'p Atom],
        reached: &FxHashSet<usize>,
        ways: &mut FxHashMap<Pred<'p>, Option<Vec<bool>>>,
    ) -> Option<Vec<bool>> {
        if let Some(w) = ways.get(&p) {
            return w.clone();
        }
        ways.insert(p, None);
        let asked: Vec<&&Atom> = roots.iter().filter(|a| a.signature() == p).collect();
        let sites: Vec<&Call> = self
            .calls
            .iter()
            .filter(|c| reached.contains(&c.clause) && c.atom.signature() == p)
            .collect();
        let w = match (asked.as_slice(), sites.as_slice()) {
            ([a], []) => Some(a.args.iter().map(Term::is_ground).collect()),
            ([], [site]) if !site.repeated && !self.recursive(p) => {
                let c = &self.program.clauses[site.clause];
                match c.heads.as_slice() {
                    [head] => self
                        .way(head.atom.signature(), roots, reached, ways)
                        .and_then(|w| {
                            let bound = bound(&head.atom, &w);
                            let before = before(&c.body[..site.at]);
                            let mut vs = Vec::new();
                            site.atom.vars_into(&mut vs);
                            vs.iter()
                                .all(|v| bound.contains(v) || !before.contains(v))
                                .then(|| {
                                    site.atom
                                        .args
                                        .iter()
                                        .map(|t| term_vars(t).iter().all(|v| bound.contains(v)))
                                        .collect()
                                })
                        }),
                    _ => None,
                }
            }
            _ => None,
        };
        ways.insert(p, w.clone());
        w
    }

    /// [`calls_once`] for the evaluation of `roots`.
    fn check(&self, roots: &[&'p Atom], tabled: &FxHashSet<(String, usize)>) -> Result<(), String> {
        let reached = self.reach(roots.iter().map(|a| a.signature()).collect());
        let mut ways = FxHashMap::default();
        let mut switches: Vec<&Term> = Vec::new();
        for call in &self.calls {
            if !reached.contains(&call.clause) || call.atom.signature() != ("msw", 2) {
                continue;
            }
            let c = &self.program.clauses[call.clause];
            let lit = &c.body[call.at];
            if call.repeated || !matches!(lit, Literal::Pos(_)) {
                return Err(format!("msw/2 under a negation or in a findall, in `{c}`"));
            }
            let [head] = c.heads.as_slice() else {
                return Err(format!(
                    "msw/2 in the body of `{c}`, an annotated disjunction"
                ));
            };
            let p = head.atom.signature();
            let Some(w) = self.way(p, roots, &reached, &mut ways) else {
                return Err(format!(
                    "{}/{}, whose clause `{c}` calls it, can be called in more than one way",
                    p.0, p.1
                ));
            };
            let bound = bound(&head.atom, &w);
            for l in &c.body[..call.at] {
                let once = match l {
                    Literal::Pos(a) => {
                        a.signature() == ("msw", 2)
                            || tabled.contains(&(a.name.to_string(), a.args.len()))
                    }
                    Literal::Cmp(..) | Literal::Is(..) => true,
                    _ => false,
                };
                if !once {
                    return Err(format!(
                        "`{l}` before `{lit}` in `{c}` can give a solution twice"
                    ));
                }
            }
            let before = before(&c.body[..call.at]);
            let s = &call.atom.args[0];
            let in_s = term_vars(s);
            if let Some(v) = before
                .iter()
                .find(|v| !bound.contains(*v) && !in_s.contains(v))
            {
                return Err(format!("`{lit}` in `{c}` is called for each value of {v}"));
            }
            if let Some(v) = in_s
                .iter()
                .find(|v| !bound.contains(*v) && !before.contains(*v))
            {
                return Err(format!("the switch of `{lit}` in `{c}` has {v} free"));
            }
            if let Some(o) = switches.iter().find(|o| unifiable(o, s)) {
                return Err(format!(
                    "two calls of msw/2 can call the switch of both {o} and {s}"
                ));
            }
            switches.push(s);
        }
        Ok(())
    }
}

/// The variables of `head` in the arguments a call binds (`way`).
fn bound(head: &Atom, way: &[bool]) -> FxHashSet<String> {
    head.args
        .iter()
        .zip(way)
        .filter(|(_, b)| **b)
        .flat_map(|(t, _)| term_vars(t))
        .collect()
}

/// The variables of the literals `body`.
fn before(body: &[Literal]) -> FxHashSet<String> {
    let mut out = Vec::new();
    for l in body {
        l.vars_into(&mut out);
    }
    out.into_iter().collect()
}

fn term_vars(t: &Term) -> Vec<String> {
    let mut out = Vec::new();
    t.vars_into(&mut out);
    out
}

/// The depth PITA's depth bound asks the queries at, where the source
/// sets the bound (`:- set_pita(depth_bound, true).`): the depth it sets
/// (`:- set_pita(depth, N).`), 5 by default, as PITA's; and whether PITA
/// makes one choice for each instance of a PRISM switch
/// (`:- set_pita(prism_memoization, true).`, [`switches`]). A source written
/// for PITA(IND) (`:- pitaind.`) is refused: it asks for the probabilities
/// PITA computes under the assumption that a conjunction's literals are
/// independent and a disjunction's independent or, under `set_pitaind(or,
/// exc)`, exclusive, which differ from the program's where the assumption
/// does not hold.
fn settings(clauses: &[(usize, &str)]) -> Result<(Option<u32>, bool), (usize, String)> {
    let (mut bound, mut depth, mut memo) = (false, 5, false);
    for &(at, c) in clauses {
        let Some(goal) = c.strip_prefix(":-").map(trim) else {
            continue;
        };
        let Some((name, args)) = atom_parts(goal) else {
            continue;
        };
        match (name, args.as_slice()) {
            ("pitaind" | "set_pitaind", _) | ("use_module", ["library(pitaind)"]) => {
                return Err((
                    at,
                    format!(
                        "`{goal}`: PITA(IND) assumes a conjunction's literals independent and a \
                         disjunction's independent or exclusive; tddlog's inference is exact and \
                         does not take these assumptions"
                    ),
                ));
            }
            ("set_pita", ["depth_bound", v]) => bound = *v == "true",
            ("set_pita", ["prism_memoization", v]) => memo = *v == "true",
            ("set_pita", ["depth", v]) => {
                depth = v
                    .parse()
                    .map_err(|_| (at, format!("PITA's depth `{v}`, not a count")))?;
            }
            _ => {}
        }
    }
    Ok((bound.then_some(depth), memo))
}

/// The ProbLog clauses of the LPAD clause `clause`, the LPAD's `number`th,
/// under PITA's depth bound: each predicate of the LPAD (`lpad`) takes the
/// depth as its last argument. A clause with a body holds at a depth `D`
/// of 1 or more where its body holds at `D - 1`, and a fact at every
/// depth. The choice among a clause's heads is made once for each
/// instance of the clause, whatever the depth, as PITA makes it: a switch
/// over the clause's variables (`'$choice_N'`), which each head's clause
/// reads after its body.
fn bounded(
    clause: &str,
    number: usize,
    lpad: &FxHashSet<(String, usize)>,
) -> Result<Vec<String>, String> {
    let clause = named_apart(clause);
    let (head, body) = match clause.split_once(":-") {
        Some((h, b)) => (h, Some(b)),
        None => (clause.as_str(), None),
    };
    let heads = heads(head)?;
    let vars = variables(&clause);
    let fresh = |base: &str| {
        let mut name = base.to_string();
        while vars.contains(&name.as_str()) {
            name.push('_');
        }
        name
    };
    let (d, d1) = (fresh("D"), fresh("D1"));
    let mut lits = Vec::new();
    if let Some(body) = body {
        lits.push(format!("{d} >= 1"));
        lits.push(format!("{d1} is {d} - 1"));
        for lit in split_top(body, b',') {
            lits.push(bounded_literal(trim(lit), &d1, lpad)?);
        }
    }
    // The switch's alternative for the `k`th head.
    let choice = |k: usize| {
        let args: Vec<String> = vars
            .iter()
            .map(|v| v.to_string())
            .chain([k.to_string()])
            .collect();
        format!("'$choice_{number}'({})", args.join(", "))
    };
    let mut out = Vec::new();
    let probabilistic = heads.iter().any(|(_, p)| p.is_some());
    if probabilistic {
        let mut alternatives = Vec::with_capacity(heads.len());
        for (k, (_, p)) in heads.iter().enumerate() {
            let p = match p.expect("every head has its probability") {
                Written::Term(_) => {
                    return Err("a flexible probability under PITA's depth bound".into());
                }
                p => annotation(p)?,
            };
            alternatives.push(format!("{p}::{}", choice(k + 1)));
        }
        out.push(format!("{}.", alternatives.join("; ")));
    }
    let at = if body.is_some() { d.as_str() } else { "_" };
    for (k, (atom, _)) in heads.iter().enumerate() {
        let head = with_arg(atom, at).ok_or_else(|| format!("the head `{atom}`, not an atom"))?;
        let mut lits = lits.clone();
        if probabilistic {
            lits.push(choice(k + 1));
        }
        out.push(if lits.is_empty() {
            format!("{head}.")
        } else {
            format!("{head} :- {}.", lits.join(", "))
        });
    }
    Ok(out)
}

/// The body literal `lit` of a clause under the depth bound, its call of
/// a predicate of the LPAD (`lpad`) at the depth `d`; what it cannot write
/// where it calls one inside another goal.
fn bounded_literal(
    lit: &str,
    d: &str,
    lpad: &FxHashSet<(String, usize)>,
) -> Result<String, String> {
    let (negated, goal) = match lit.strip_prefix("\\+") {
        Some(g) => ("\\+ ", unwrapped(trim(g))),
        None => ("", lit),
    };
    if let Some((name, args)) = atom_parts(goal)
        && lpad.contains(&(name.to_string(), args.len()))
    {
        return Ok(format!("{negated}{}", with_arg(goal, d).expect("an atom")));
    }
    let mut names = FxHashSet::default();
    mentioned(lit, &mut names);
    if lpad.iter().any(|(n, _)| names.contains(n)) {
        return Err(format!(
            "a call of the LPAD's predicates inside `{lit}` under PITA's depth bound"
        ));
    }
    Ok(lit.to_string())
}

/// The name and the arguments of the atom `text`, if it is one.
fn atom_parts(text: &str) -> Option<(&str, Vec<&str>)> {
    let text = trim(text);
    if text.is_empty() {
        return None;
    }
    let name = name_at(text, 0)?;
    let rest = &text[name.len()..];
    if rest.is_empty() {
        return Some((name, Vec::new()));
    }
    let inner = balanced(rest.strip_prefix('(')?, 0);
    (inner.len() + 2 == rest.len())
        .then(|| (name, split_top(inner, b',').into_iter().map(trim).collect()))
}

/// The atom `atom` with `arg` as a last argument.
fn with_arg(atom: &str, arg: &str) -> Option<String> {
    let (name, args) = atom_parts(atom)?;
    let args: Vec<&str> = args.into_iter().chain([arg]).collect();
    Some(format!("{name}({})", args.join(", ")))
}

/// The variables of `text` outside quotes, each once, in their order.
fn variables(text: &str) -> Vec<&str> {
    let mut vars = Vec::new();
    for (i, len) in words(text) {
        let w = &text[i..i + len];
        if (w.starts_with('_') || w.starts_with(|c: char| c.is_uppercase())) && !vars.contains(&w) {
            vars.push(w);
        }
    }
    vars
}

/// `clause` with each anonymous variable `_` named apart from the others.
fn named_apart(clause: &str) -> String {
    let vars = variables(clause);
    let mut out = String::with_capacity(clause.len());
    let (mut last, mut k) = (0, 0);
    for (i, len) in words(clause) {
        if &clause[i..i + len] != "_" {
            continue;
        }
        let name = loop {
            k += 1;
            let name = format!("_{k}");
            if !vars.contains(&name.as_str()) {
                break name;
            }
        };
        out.push_str(&clause[last..i]);
        out.push_str(&name);
        last = i + len;
    }
    out.push_str(&clause[last..]);
    out
}

/// Where each word of `text` outside quotes starts, with its length.
fn words(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut quote = None;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if c == '\'' || c == '"' {
            quote = Some(c);
        } else if word(c) && starts_word(text, i) {
            let len = text[i..].find(|c| !word(c)).unwrap_or(text.len() - i);
            out.push((i, len));
            while chars.peek().is_some_and(|&(j, _)| j < i + len) {
                chars.next();
            }
        }
    }
    out
}

/// A head's probability as an LPAD writes it: a number, or a term its body
/// binds, a flexible probability (`male:M/P :- ..., P is F + M.`).
#[derive(Clone, Copy)]
enum Written<'a> {
    Number(f64),
    Term(&'a str),
}

/// A head `atom:p` as the atom and its probability, a head without one as
/// itself; an error where the probability divides by zero or is empty.
fn head_prob(head: &str) -> Result<(&str, Option<Written<'_>>), String> {
    if let [atom, p] = split_top(head, b':')[..] {
        return Ok((
            trim(atom),
            Some(match number(p)? {
                Some(p) => Written::Number(p),
                None if trim(p).is_empty() => return Err("an empty probability".into()),
                None => Written::Term(trim(p)),
            }),
        ));
    }
    Ok((trim(head), None))
}

/// The number `expr` writes, a float or a fraction `n/d` of two decimals
/// (`1/4`); `None` for anything else.
fn number(expr: &str) -> Result<Option<f64>, String> {
    let expr = trim(expr);
    if let Some((n, d)) = expr.split_once('/') {
        let (n, d) = (n.trim_end_matches(space), trim_start(d));
        if decimal(n, true) && decimal(d, false) {
            let (n, d): (f64, f64) = (n.parse().expect("a decimal"), d.parse().expect("a decimal"));
            if d == 0.0 {
                return Err(format!("the probability {expr}, a division by zero"));
            }
            return Ok(Some(n / d));
        }
    }
    Ok(expr.parse().ok())
}

/// Whether `s` is digits with an optional fraction, and where `exponent`
/// allows, an optional exponent: `\d+(\.\d+)?([eE]-?\d+)?`.
fn decimal(s: &str, exponent: bool) -> bool {
    let digits = |s: &str| s.len() - s.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let mut s = s;
    let n = digits(s);
    if n == 0 {
        return false;
    }
    s = &s[n..];
    if let Some(r) = s.strip_prefix('.') {
        let n = digits(r);
        if n == 0 {
            return false;
        }
        s = &r[n..];
    }
    if exponent && let Some(r) = s.strip_prefix(['e', 'E']) {
        let r = r.strip_prefix('-').unwrap_or(r);
        let n = digits(r);
        if n == 0 {
            return false;
        }
        s = &r[n..];
    }
    s.is_empty()
}

/// The example queries `?- prob(Q, P)` and `?- prob(Q, E, P)` of the
/// source, in its order, each as `Q` and the literals of the conjunction
/// `E`. They sit in `/** <examples> ... */` blocks, so they are read from
/// the source as it is, comments and all.
fn examples(source: &str) -> Vec<(&str, Vec<&str>)> {
    let mut asked = Vec::new();
    for (_, line) in lines(source) {
        let goal = trim(line);
        if !goal.starts_with("?-") {
            continue;
        }
        let Some(at) = goal
            .match_indices("prob(")
            .map(|(i, _)| i)
            .find(|&i| starts_word(goal, i))
        else {
            continue;
        };
        let args: Vec<&str> = split_top(balanced(goal, at + 5), b',')
            .into_iter()
            .map(trim)
            .collect();
        match args[..] {
            [q, _] => asked.push((q, Vec::new())),
            [q, e, _] => asked.push((
                q,
                split_top(unwrapped(e), b',')
                    .into_iter()
                    .map(trim)
                    .collect(),
            )),
            _ => {}
        }
    }
    asked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn program(source: &str) -> String {
        read(source).expect("an LPAD").expect("read").program
    }

    fn lpad(body: &str) -> String {
        format!(":- use_module(library(pita)).\n:- pita.\n:- begin_lpad.\n{body}:- end_lpad.\n")
    }

    #[test]
    fn a_source_is_an_lpad_where_it_holds_the_begin_marker() {
        assert!(read("a:0.5.\nquery(a).\n").is_none());
        assert!(read("0.5::a.\n% :- begin_lpad.\nquery(a).\n").is_none());
        assert!(read("0.5::begin_lpad.\nquery(begin_lpad).\n").is_none());
        assert_eq!(program(":- begin_plp.\na:0.5.\n:- end_plp.\n"), "0.5::a.");
        assert_eq!(program(":-begin_lpad.\na:0.5.\n"), "0.5::a.");
    }

    #[test]
    fn annotated_disjunctions_and_a_head_that_takes_the_rest() {
        assert_eq!(
            program(&lpad(
                "a:0.3.\nb:0.2 ; c:0.5 :- a.\nd:1/4 ; e.\nf:1e-05 ; g :- \\+ a.\n"
            )),
            "0.3::a.\n0.2::b; 0.5::c :- a.\n0.25::d; 0.75::e.\n1e-05::f; 0.99999::g :- \\+ a."
        );
        // A probability that is a whole number, and a rest of nothing.
        assert_eq!(program(&lpad("a:1 ; b.\n")), "1.0::a; 0.0::b.");
    }

    #[test]
    fn a_flexible_probability_is_written_as_it_is() {
        assert_eq!(
            program(&lpad(
                "male:M/P ; female:F/P :- n(M, F), P is F+M.\nred(P):P.\nc:1-Q ; d:0.5 :- q(Q).\n"
            )),
            "M/P::male; F/P::female :- n(M, F), P is F+M.\nP::red(P).\n1-Q::c; 0.5::d :- q(Q)."
        );
    }

    #[test]
    fn the_database_clauses_the_lpad_calls_are_kept() {
        // q through db/1, r through q; plot, a program over the LPAD that
        // ProbLog's language does not read, is left out, as are the
        // directives and the comments.
        let source = "r(1).\n:- begin_lpad.\np(X):0.5 :- db(q(X)).\n:- end_lpad.\n\
                      q(X) :- r(X). % q's comment\n\
                      plot(P) :- ( prob(p(1), P) -> true ; P = 0 ).\n:- dynamic(s/1).\n";
        assert_eq!(program(source), "r(1).\n0.5::p(X) :- q(X).\nq(X) :- r(X).");
        // A name in quotes is not a call.
        let source = "s(1).\n:- begin_lpad.\np:0.5 :- X = 's'.\n:- end_lpad.\n";
        assert_eq!(program(source), "0.5::p :- X = 's'.");
    }

    #[test]
    fn a_list_predicate_the_program_calls_is_imported() {
        assert_eq!(
            program(&lpad("p(X):0.5 :- member(X, [1, 2]).\n")),
            ":- use_module(library(lists)).\n0.5::p(X) :- member(X, [1, 2])."
        );
        // Not one the program defines, nor between/3, which is built in.
        assert_eq!(
            program(&lpad(
                "p(X):0.5 :- member(X), between(1, 2, X).\nmember(1).\n"
            )),
            "0.5::p(X) :- member(X), between(1, 2, X).\nmember(1)."
        );
    }

    #[test]
    fn the_evidence_is_each_literal_of_the_conjunction() {
        let source = lpad("a:0.5.\nb:0.5.\nc :- a ; b.\n")
            + "/** <examples>\n?- prob(c, (a, \\+ b), P).\n*/\n";
        assert_eq!(
            program(&source),
            "0.5::a.\n0.5::b.\nc :- a ; b.\nevidence(a).\nevidence(\\+ b).\nquery(c)."
        );
    }

    #[test]
    fn a_query_under_other_evidence_or_an_intervention_is_named_and_left_out() {
        let source = lpad("a:0.5.\nb:0.5 :- a.\n")
            + "?- prob(b, do(a), P).\n?- prob(b, P).\n?- prob(b, a, P).\n?- prob(b, P), bar(P, C).\n?- mc_prob(b, P).\n";
        let read = read(&source).unwrap().unwrap();
        let notes = [
            "left out, asked under an intervention: prob(b, (do(a)), P)",
            "left out, asked under other evidence: prob(b, (a), P)",
        ];
        assert_eq!(read.left_out, notes);
        assert_eq!(
            read.program,
            format!(
                "0.5::a.\n0.5::b :- a.\nquery(b).\n% {}\n% {}",
                notes[0], notes[1]
            )
        );
    }

    #[test]
    fn a_clause_the_program_cannot_write_is_refused_on_its_line() {
        for (clause, error) in [
            ("a ; b :- c.", "a disjunctive head without probabilities"),
            ("a ; b ; c:0.5.", "two heads without a probability"),
            ("a:1/0.", "a division by zero"),
            ("a:inf.", "the probability inf"),
            (
                "a:P ; b :- p(P).",
                "a head without a probability beside a flexible one",
            ),
            ("a: ; b:0.5.", "an empty probability"),
        ] {
            let source =
                format!("/* a\nblock */ :- begin_lpad.\n\nc:0.5.\n{clause}\n:- end_lpad.\n");
            let e = read(&source).unwrap().unwrap_err();
            assert_eq!(e.line, 5, "{clause}");
            assert!(e.msg.contains(error), "{clause}: {}", e.msg);
        }
    }

    #[test]
    fn under_pitas_depth_bound_each_predicate_of_the_lpad_takes_the_depth() {
        let source = ":- set_pita(depth_bound,true).\n:- set_pita(depth,2).\n".to_string()
            + &lpad(
                "num(0).\nnum(s(X)):0.5 :- num(X).\nmore(_):0.3.\n\
                 small(X) :- num(X), \\+ more(X).\np(D) :- small(D), db(q(D)).\n",
            )
            + "q(1).\n?- prob(small(s(0)),P).\n?- prob(p(1),P).\n";
        assert_eq!(
            program(&source),
            "num(0, _).\n\
             0.5::'$choice_2'(X, 1).\n\
             num(s(X), D) :- D >= 1, D1 is D - 1, num(X, D1), '$choice_2'(X, 1).\n\
             0.3::'$choice_3'(_1, 1).\n\
             more(_1, _) :- '$choice_3'(_1, 1).\n\
             small(X, D) :- D >= 1, D1 is D - 1, num(X, D1), \\+ more(X, D1).\n\
             p(D, D_) :- D_ >= 1, D1 is D_ - 1, small(D, D1), q(D).\n\
             q(1).\n\
             small(X1) :- small(X1, 2).\n\
             p(X1) :- p(X1, 2).\n\
             query(small(s(0))).\nquery(p(1))."
        );
        // The depth is 5 where the source does not set it, and a
        // disjunction's heads share one switch.
        let source = lpad("a:0.2 ; b:0.3 :- c.\nc.\n").replace(
            ":- begin_lpad.",
            ":- begin_lpad.\n:- set_pita(depth_bound,true).",
        ) + "?- prob(a,P).\n";
        assert_eq!(
            program(&source),
            "0.2::'$choice_1'(1); 0.3::'$choice_1'(2).\n\
             a(D) :- D >= 1, D1 is D - 1, c(D1), '$choice_1'(1).\n\
             b(D) :- D >= 1, D1 is D - 1, c(D1), '$choice_1'(2).\n\
             c(_).\n\
             a :- a(5).\n\
             query(a)."
        );
    }

    #[test]
    fn what_the_depth_bound_cannot_write_is_refused() {
        for (clause, error) in [
            (
                "a:P :- p(P).",
                "a flexible probability under PITA's depth bound",
            ),
            (
                "a :- findall(X, p(X), L).",
                "a call of the LPAD's predicates inside",
            ),
            (
                "a :- (p(1) ; p(2)).",
                "a call of the LPAD's predicates inside",
            ),
        ] {
            let source = format!(
                ":- set_pita(depth_bound,true).\n:- begin_lpad.\np(1):0.5.\n{clause}\n:- end_lpad.\n"
            );
            let e = read(&source).unwrap().unwrap_err();
            assert_eq!(e.line, 4, "{clause}");
            assert!(e.msg.contains(error), "{clause}: {}", e.msg);
        }
        let e = read(":- set_pita(depth_bound,true).\n:- set_pita(depth,a).\n:- begin_lpad.\n")
            .unwrap()
            .unwrap_err();
        assert_eq!(e.line, 2);
        assert!(e.msg.contains("PITA's depth `a`"), "{}", e.msg);
    }

    #[test]
    fn a_source_written_for_pita_ind_is_refused() {
        for directive in [
            ":- use_module(library(pitaind)).",
            ":- pitaind.",
            ":- set_pitaind(or,exc).",
        ] {
            let source = format!("\n{directive}\n:- begin_lpad.\na:0.5.\n:- end_lpad.\n");
            let e = read(&source).unwrap().unwrap_err();
            assert_eq!(e.line, 2, "{directive}");
            assert!(
                e.msg.contains("tddlog's inference is exact"),
                "{directive}: {}",
                e.msg
            );
        }
    }

    #[test]
    fn comments_and_line_breaks_are_read_as_python_reads_them() {
        let source = ":- begin_lpad.\r\na:0.5 /* x */ :- b. % c\r\nb:0.2.\n:- end_lpad.\n";
        assert_eq!(program(source), "0.5::a :- b.\n0.2::b.");
        // A `%` after a backslash does not start a comment.
        assert_eq!(
            program(":- begin_lpad.\na:0.5 :- X = '\\%'.\n"),
            "0.5::a :- X = '\\%'."
        );
    }

    /// cplint's coinmsw.pl, PRISM's switches under PITA's default.
    const COIN: &str = "values(throw(_),[heads,tails]).
:- set_sw(throw(fair),[0.5,0.5]).
:- set_sw(throw(biased),[0.6,0.4]).
values(fairness,[fair,biased]).
:- set_sw(fairness,[0.9,0.1]).
res(Coin,R):- toss(Coin),fairness(Coin,Fairness),msw(throw(Fairness),R).
fairness(_Coin,Fairness):-msw(fairness,Fairness).
toss(coin).
";

    #[test]
    fn prisms_switches_are_annotated_disjunctions() {
        assert_eq!(
            program(&lpad(COIN)),
            "values(throw(_),[heads,tails]).
values(fairness,[fair,biased]).
res(Coin,R) :- toss(Coin),fairness(Coin,Fairness),msw(throw(Fairness),R).
fairness(_Coin,Fairness) :- msw(fairness,Fairness).
toss(coin).
0.5::msw(throw(fair), heads); 0.5::msw(throw(fair), tails).
0.6::msw(throw(biased), heads); 0.4::msw(throw(biased), tails).
0.9::msw(fairness, fair); 0.1::msw(fairness, biased)."
        );
        // A switch with a variable: a choice for each instance.
        assert_eq!(
            program(&lpad(
                "values(flu(_), [1, 0]).\n:- set_sw(flu(_X), [0.7, 0.3]).\ns(X) :- f(X), msw(flu(X), 1).\nf(bob).\n"
            )),
            "values(flu(_), [1, 0]).\ns(X) :- f(X), msw(flu(X), 1).\nf(bob).\n0.7::msw(flu(_X), 1); 0.3::msw(flu(_X), 0)."
        );
    }

    #[test]
    fn under_pitas_default_a_switch_called_twice_is_refused() {
        let refused = |body: &str, memo: bool| {
            let memo = if memo {
                ":- set_pita(prism_memoization, true).\n"
            } else {
                ""
            };
            read(&format!("{memo}{}?- prob(q, P).\n", lpad(body)))
                .unwrap()
                .err()
                .map(|e| e.msg)
        };
        let coin = "values(c, [h, t]).\n:- set_sw(c, [0.6, 0.4]).\n";
        for body in [
            // Twice in one derivation, by two calls and by one.
            "q :- msw(c, h), msw(c, h).\n",
            "q :- member(_, [1, 2]), msw(c, h).\n",
            "q :- r(1), r(2).\nr(_) :- msw(c, h).\n",
            // In two derivations of one answer.
            "q :- msw(c, h).\nq :- msw(c, t).\n",
            "q :- r.\nr :- msw(c, h).\nq :- r.\n",
            // In a recursion.
            "q :- r(2).\nr(0).\nr(N) :- N > 0, msw(c, h), M is N - 1, r(M).\n",
        ] {
            let e = refused(&format!("{coin}{body}"), false).unwrap_or_else(|| panic!("{body}"));
            assert!(e.contains("under PITA's default"), "{body}: {e}");
            assert_eq!(refused(&format!("{coin}{body}"), true), None, "{body}");
        }
        // Each instance called once.
        for body in [
            "q :- msw(c, h).\n",
            "q :- r(X), msw(d(X), h).\nr(1). r(2).\nvalues(d(_), [h, t]).\n:- set_sw(d(_), [0.5, 0.5]).\n",
        ] {
            assert_eq!(refused(&format!("{coin}{body}"), false), None, "{body}");
        }
        assert_eq!(refused(&lpad(COIN), false), None);
    }

    #[test]
    fn a_switch_whose_values_cannot_be_read_is_refused() {
        for (body, why) in [
            (":- set_sw(c, [0.6, 0.4]).\n", "0 facts values(c, _)"),
            (
                "values(c, [h]).\n:- set_sw(c, [0.6, 0.4]).\n",
                "1 values and 2 probabilities",
            ),
            (
                "values(c, [h, t]).\n:- set_sw(c, [0.6, 0.6]).\n",
                "do not sum to one",
            ),
            (
                "values(c, [h, t]).\n:- set_sw(c, uniform).\n",
                "not a list of numbers",
            ),
            (
                "values(c(a), [h, t]).\n:- set_sw(c(_), [0.5, 0.5]).\n",
                "only some of its values",
            ),
            (
                "values(c(_), [h, t]).\n:- set_sw(c(_), [0.5, 0.5]).\n:- set_sw(c(a), [0.5, 0.5]).\n",
                "is set too",
            ),
        ] {
            let e = read(&lpad(body)).unwrap().unwrap_err();
            assert!(e.msg.contains(why), "{body}: {}", e.msg);
        }
    }
}
