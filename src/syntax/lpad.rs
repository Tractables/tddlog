//! LPADs written for cplint and PITA ([`read`]): a source that holds the
//! directive `:- begin_lpad.` (or cplint's other spelling,
//! `:- begin_plp.`) is read as the ProbLog program this module writes for
//! it, which `tddlog --program` prints and the parser then reads
//! ([`crate::read`] finds the directive in a ProbLog input).
//!
//! The LPAD is the clauses between `:- begin_lpad.` and `:- end_lpad.`.
//! Its clauses `h1:p1 ; h2:p2 :- body.` are written `p1::h1; p2::h2 :-
//! body.`, and `a:p.` is written `p::a.`; a head without a probability
//! takes what the others leave. The clauses outside the markers are
//! cplint's database clauses, plain Prolog, which an LPAD body calls
//! through `db(Goal)` or the goal of a built-in such as `findall/3`: those
//! the LPAD's bodies name, and those they name in turn, are kept as they
//! are, and `db(Goal)` is written `Goal`. The rest (plots, samplers and
//! other programs over the LPAD) is left out, as are the directives and
//! goals. The example queries `?- prob(Q, P).` in the file become
//! `query(Q).`, and `?- prob(Q, E, P).` becomes `query(Q).` and an
//! `evidence(L).` for each literal `L` of the conjunction `E`. A program
//! holds one set of evidence: the queries asked under the evidence of the
//! file's first example query are written, and the others are named in a
//! comment and in [`Lpad::left_out`], as is a query under an intervention
//! `do(L)`, which ProbLog does not read. Everything else is passed
//! through, so that a clause that uses what tddlog does not read stops the
//! parser with its message.
//!
//! The clauses are found in the text as cplint's examples write them, as
//! the Python script `lpad2problog.py` (on the baselines branch) found
//! them before this reader replaced it, and the program is the one that
//! script wrote: `/* */` and `%` comments are dropped, a clause ends at a `.` followed by white space
//! outside quotes and brackets, and the names a body mentions are the
//! words outside quotes that start with a lower-case letter.

use rustc_hash::FxHashSet;

use super::ParseError;
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
/// head with no probabilities or with two heads without one, or a
/// probability that is not a finite number.
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
    let mut out = Vec::new();
    for ((c, _, at), _) in read.iter().zip(&kept).filter(|(_, keep)| **keep) {
        out.push(convert_clause(c).map_err(|msg| ParseError {
            line: raw.line(source, *at),
            msg: format!("{msg} in the LPAD clause `{c}`"),
        })?);
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
    Ok(Lpad {
        program: out.join("\n"),
        left_out,
    })
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
    let mut heads = split_top(head, b';')
        .into_iter()
        .map(head_prob)
        .collect::<Result<Vec<_>, _>>()?;
    if heads.iter().all(|(_, p)| p.is_none()) {
        return match heads.as_slice() {
            [(atom, None)] => Ok(format!("{atom}{tail}.")),
            _ => Err("a disjunctive head without probabilities".into()),
        };
    }
    let given: Vec<f64> = heads.iter().filter_map(|&(_, p)| p).collect();
    let total = fsum(&given);
    let mut missing = heads.iter_mut().filter(|(_, p)| p.is_none());
    if let Some(rest) = missing.next() {
        if missing.next().is_some() {
            return Err("two heads without a probability".into());
        }
        let left = 1.0 - total;
        rest.1 = Some(if left > 0.0 { left } else { 0.0 });
    }
    let mut lhs = String::new();
    for (k, (atom, p)) in heads.iter().enumerate() {
        let p = p.expect("every head has its probability");
        if !p.is_finite() {
            return Err(format!("the probability {p}"));
        }
        if k > 0 {
            lhs.push_str("; ");
        }
        if p == p.trunc() {
            lhs.push_str(&format!("{p:.1}"));
        } else {
            write_float(&mut lhs, p);
        }
        lhs.push_str("::");
        lhs.push_str(atom);
    }
    Ok(format!("{lhs}{tail}."))
}

/// A head `atom:p` as the atom and its probability, a head without one as
/// itself; an error where the probability divides by zero.
fn head_prob(head: &str) -> Result<(&str, Option<f64>), String> {
    if let [atom, p] = split_top(head, b':')[..]
        && let Some(p) = number(p)?
    {
        return Ok((trim(atom), Some(p)));
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
        ] {
            let source =
                format!("/* a\nblock */ :- begin_lpad.\n\nc:0.5.\n{clause}\n:- end_lpad.\n");
            let e = read(&source).unwrap().unwrap_err();
            assert_eq!(e.line, 5, "{clause}");
            assert!(e.msg.contains(error), "{clause}: {}", e.msg);
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
}
