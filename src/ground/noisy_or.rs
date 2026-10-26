//! Noisy-or tables: a table of two values over parents of two values whose
//! one value is a noisy-or of the parents, written as that noisy-or where
//! nothing reads, observes or asks its other value.
//!
//! A Bayesian network written one annotated disjunction per row
//! (`scripts/gen_bn.py`) holds a table `x` of values `b` and `c` over
//! parents `y_1, ..., y_m` of two values each as one row per assignment
//! `u` to the parents. With `u*` the row in which `c` is likeliest,
//! `L = P(c | u*)` and `q_i = P(c | u* with y_i flipped) / L`, the table is
//! a noisy-or (Pearl 1988, section 4.3.2) where every row has
//!
//! ```text
//! P(c | u) = L · ∏ q_i over the parents i at which u differs from u*.
//! ```
//!
//! `x(b)` then holds exactly when one of independent causes does: a leak
//! of probability `1 - L`, and each parent at its value away from `u*`,
//! `a_i`, with probability `1 - q_i`. The table is written so,
//!
//! ```text
//! (1-L)::x(b).   (1-q_1)::x(b) :- y_1(a_1).   ...   (1-q_m)::x(b) :- y_m(a_m).
//! ```
//!
//! which gives each row its distribution, each parent being exactly one of
//! its two values in every world, and `x(c)` is not written: the rewrite
//! is taken only where nothing reads `x(c)` once the readers taken are
//! rewritten too, and no query or observation names it, and for a table
//! observed only where its parents are taken (below). A root, a table of
//! no parents, is a leak alone, a probabilistic fact. Every other atom
//! keeps its distribution, jointly with the others.
//!
//! The rewrite is what lets the later steps see an or: the rows read each
//! parent at both values, as an exclusive set, and nothing inlines a
//! table's value read so. Rewritten, a table read at one value by one
//! reader is a view (`inline`), and a chain of such tables is one rule
//! per cause of the atom at its end. The UAI 2022 or_chain networks write
//! each finding as a chain of two-parent noisy-ors, one per cause, and
//! observe the findings present: as tables, the chains' atoms, each read
//! at both values, make the decompositions wide; as rules, each finding
//! reads its causes' facts and the program's decomposition is narrow
//! (with row classes leaving such tables alone, [`super::classes`]).
//!
//! A table observed whose parents are not all taken stays a table, and
//! its parents, read at both values, stay too. Observed as a table, `x(b)`
//! makes `x(c)` false and the body of each of its rows with it, which the
//! conditioning reads as clauses over the parents ([`super::condition`]),
//! and each parent is read at both values, through the clauses of its
//! exclusive set ([`crate::indicators`]). Observed as a noisy-or, it is an
//! or the conditioning does not read, and it reads a parent that stays a
//! table at one value: a value it was the only reader of keeps no copy,
//! and a query of that value is counted against the evidence on its own.
//! So an observed table taken over parents that stay costs more than it
//! saves (the PR set's fs_07, whose evidence then fixes far fewer atoms;
//! the MAR promedus networks, whose findings observed present are such
//! tables). A table not observed is taken over parents that stay: its
//! rules read one parent each rather than all of them together, which
//! narrows the decomposition (the PR set's mastermind networks). The
//! or_chain networks have all their tables taken either way.
//!
//! Taken where the program is table-shaped, its tables read from the
//! [`Shape`] the table path grounds ([`super::table`]), each table a
//! predicate whose head atoms are its values, and its rows sum to one up
//! to rounding as [`super::normalize`] reads them; the entries must match
//! the noisy-or to [`TOLERANCE`]. The rewritten program is table-shaped
//! too, and takes the table path: its shape is made from the program's
//! ([`Shape::rewritten`]) with each table taken written at its first
//! clause.

use super::normalize::SUM_TOLERANCE;
use super::table::{Shape, Switched};
use crate::syntax::{Atom, Clause, HeadChoice, Literal, Program};

/// How far an entry of a table may be from the noisy-or's.
const TOLERANCE: f64 = 1e-12;

/// The most parents of a table taken: one row per assignment.
const MOST_PARENTS: usize = 16;

/// A table's noisy-or form: its absent value `c`, the row `u*` (bit `i` the
/// value of parent `i`) and `L`; each parent's `q` is read off the rows
/// ([`Form::q`]).
#[derive(Clone, Copy)]
struct Form {
    absent: usize,
    base: usize,
    leak: f64,
}

impl Form {
    /// The `q` of parent `i`, of the table's rows `dist`.
    fn q(&self, dist: &[[f64; 2]], i: usize) -> f64 {
        dist[self.base ^ 1 << i][self.absent] / self.leak
    }
}

/// A table of two values whose rows partition its parents' values and
/// that has a noisy-or form.
struct Rows<'s> {
    /// The parents' tables, in the order the first row reads them.
    parents: &'s [usize],
    /// Where its rows start among every table's (`dists` in [`rewrite`]):
    /// per assignment (bit `i` the value of parent `i`), the probabilities
    /// of the two values.
    at: usize,
    /// Its forms, by absent value.
    forms: [Option<Form>; 2],
}

impl Rows<'_> {
    /// Its rows, of every table's `dists`.
    fn dist<'d>(&self, dists: &'d [[f64; 2]]) -> &'d [[f64; 2]] {
        &dists[self.at..self.at + (1 << self.parents.len())]
    }
}

/// A table written as a noisy-or (above): its clauses, each with the table
/// its body reads (`None` for the leak, which reads none).
pub(super) type Written = Vec<(Clause, Option<usize>)>;

/// The noisy-or tables of `program`, of the tables `shape` in the
/// evaluation order `order` (each after the tables it reads), written as
/// noisy-ors: per table taken its clauses, and how many tables were taken;
/// `None` where there is none to write.
pub(super) fn rewrite(
    shape: &Shape,
    order: &[usize],
    program: &Program,
) -> Option<(Vec<Option<Written>>, usize)> {
    let n = shape.predicates();
    // The candidates: the tables of two values whose parents are
    // candidates, each with its values. A table's parents are the
    // predicates its rows read ([`Shape::reads`]), and a value is told by
    // its arguments among the table's two ([`value_of`]), so that no atom
    // is hashed: the detection costs little on the programs it takes
    // nothing from.
    let mut values: Vec<Option<[&Atom; 2]>> = vec![None; n];
    for &t in order {
        if shape.reads(t).iter().all(|&p| values[p].is_some()) {
            values[t] = two_values(shape.clauses(t));
        }
    }
    if values.iter().all(Option::is_none) {
        return None;
    }
    // The value of the candidate `p` that its atom `atom` is.
    let value = |p: usize, atom: &Atom| value_of(values[p]?, atom);
    // The candidates whose rows partition their parents' values and fit a
    // noisy-or, their rows held together in `dists` so that no table
    // allocates its own.
    let mut dists: Vec<[f64; 2]> = Vec::new();
    let rows: Vec<Option<Rows>> = (0..n)
        .map(|t| {
            let at = dists.len();
            let r = values[t].and_then(|vs| {
                let parents = table_rows(shape.clauses(t), shape.reads(t), vs, &value, &mut dists)?;
                let forms = noisy_or_forms(&dists[at..]);
                forms
                    .iter()
                    .any(Option::is_some)
                    .then_some(Rows { parents, at, forms })
            });
            if r.is_none() {
                dists.truncate(at);
            }
            r
        })
        .collect();
    if rows.iter().all(Option::is_none) {
        return None;
    }

    // The values something reads, asks or observes: of the tables not
    // rewritten every value their rows read, of those rewritten the
    // parents' values away from `u*`.
    let mut needed = vec![0u8; n];
    let mut observed = vec![false; n];
    let asked = program.queries.iter().map(|(a, _)| (a, false));
    for (a, evidence) in asked.chain(program.evidence.iter().map(|(a, _)| (a, true))) {
        if let Some(p) = shape.predicate_of(a)
            && let Some(v) = value(p, a)
        {
            needed[p] |= 1 << v;
            observed[p] |= evidence;
        }
    }
    let mut chosen: Vec<Option<Form>> = vec![None; n];
    let mut taken: Vec<bool> = rows.iter().map(Option::is_some).collect();
    let need_all = |t: usize, needed: &mut [u8]| {
        for (l, &p) in shape
            .clauses(t)
            .iter()
            .flat_map(|s| &s.clause.body)
            .zip(shape.reads(t))
        {
            if let Literal::Pos(a) = l
                && let Some(v) = value(p, a)
            {
                needed[p] |= 1 << v;
            }
        }
    };
    for t in (0..n).filter(|&t| !taken[t]) {
        need_all(t, &mut needed);
    }
    // Needs only grow and tables only leave, so this ends; each table
    // taken keeps a form whose absent value nothing needs, and one
    // observed has its parents taken (above).
    let mut changed = true;
    while changed {
        changed = false;
        for &t in order.iter().rev() {
            if !taken[t] {
                continue;
            }
            let r = rows[t].as_ref().expect("a table taken has rows");
            let open = |f: &Form| needed[t] & (1 << f.absent) == 0;
            let form = if !observed[t] || r.parents.iter().all(|&p| taken[p]) {
                if chosen[t].as_ref().is_some_and(open) {
                    continue;
                }
                r.forms.iter().flatten().find(|f| open(f)).copied()
            } else {
                None
            };
            changed = true;
            match form {
                Some(f) => {
                    let dist = r.dist(&dists);
                    for (i, &p) in r.parents.iter().enumerate() {
                        if f.q(dist, i) < 1.0 {
                            needed[p] |= 1 << (f.base >> i & 1 ^ 1);
                        }
                    }
                    chosen[t] = Some(f);
                }
                None => {
                    taken[t] = false;
                    chosen[t] = None;
                    need_all(t, &mut needed);
                }
            }
        }
    }
    let count = taken.iter().filter(|&&t| t).count();
    if count == 0 {
        return None;
    }

    // Each table taken as clauses: the row u* as written gives the leak;
    // where c is certain there, each flipped row as written gives its cause.
    let written = (0..n)
        .map(|t| {
            let f = chosen[t]?;
            let r = rows[t].as_ref().expect("a table taken has rows");
            let dist = r.dist(&dists);
            let of = |p: usize| values[p].expect("a table taken and its parents are candidates");
            let present = of(t)[1 - f.absent];
            let head = |p: f64| {
                vec![HeadChoice {
                    prob: (p < 1.0).then_some(p),
                    atom: present.clone(),
                    negated: false,
                }]
            };
            let leak = dist[f.base][1 - f.absent];
            let mut out = Vec::with_capacity(r.parents.len() + 1);
            if leak > 0.0 {
                out.push((
                    Clause {
                        heads: head(leak),
                        body: Vec::new(),
                        group: 0,
                    },
                    None,
                ));
            }
            for (k, &p) in r.parents.iter().enumerate() {
                let q = f.q(dist, k);
                if q >= 1.0 {
                    continue;
                }
                let flipped = f.base ^ 1 << k;
                let cause = if f.leak == 1.0 {
                    dist[flipped][1 - f.absent]
                } else {
                    1.0 - q
                };
                let active = of(p)[f.base >> k & 1 ^ 1].clone();
                out.push((
                    Clause {
                        heads: head(cause),
                        body: vec![Literal::Pos(active)],
                        group: 0,
                    },
                    Some(p),
                ));
            }
            Some(out)
        })
        .collect();
    Some((written, count))
}

/// The two values of the table of `clauses`, its distinct head atoms in
/// the order they come; `None` unless it has two.
fn two_values<'a>(clauses: &[Switched<'a>]) -> Option<[&'a Atom; 2]> {
    let mut heads = clauses
        .iter()
        .flat_map(|s| &s.clause.heads)
        .map(|h| &h.atom);
    let first = heads.next()?;
    let second = heads.find(|a| value_of([first, first], a).is_none())?;
    heads
        .all(|a| value_of([first, second], a).is_some())
        .then_some([first, second])
}

/// Which of a table's two `values` `atom` is, told by their arguments,
/// whose names compare by address first ([`crate::syntax::same_name`]).
fn value_of(values: [&Atom; 2], atom: &Atom) -> Option<usize> {
    values.iter().position(|a| a.args == atom.args)
}

/// The parents of the table of `clauses`, which read the tables `reads`,
/// with its rows appended to `dists`, where its clauses read the same
/// parents, at each assignment once, and each clause's heads, of the
/// table's two `values`, sum to one up to rounding; `value` tells a
/// parent's value from its atom. Where they do not, it may leave rows in
/// `dists` (which [`rewrite`] cuts).
fn table_rows<'s>(
    clauses: &[Switched],
    reads: &'s [usize],
    values: [&Atom; 2],
    value: &impl Fn(usize, &Atom) -> Option<usize>,
    dists: &mut Vec<[f64; 2]>,
) -> Option<&'s [usize]> {
    let m = clauses[0].clause.body.len();
    let parents = &reads[..m];
    if (1..m).any(|i| parents[..i].contains(&parents[i]))
        || m > MOST_PARENTS
        || clauses.len() != 1 << m
    {
        return None;
    }
    let at = dists.len();
    dists.resize(at + (1 << m), [f64::NAN; 2]);
    let dist = &mut dists[at..];
    let mut reads = reads;
    for &Switched { clause, .. } in clauses {
        if clause.body.len() != m {
            return None;
        }
        let (ys, rest) = reads.split_at(m);
        reads = rest;
        // Each parent once, at one of its values: bit k of the key is the
        // value of parent k.
        let (mut key, mut seen) = (0usize, 0usize);
        for (l, &p) in clause.body.iter().zip(ys) {
            let Literal::Pos(a) = l else { return None };
            let k = parents.iter().position(|&q| q == p)?;
            if seen >> k & 1 == 1 {
                return None;
            }
            seen |= 1 << k;
            key |= value(p, a)? << k;
        }
        if !dist[key][0].is_nan() {
            return None;
        }
        let mut d = [0.0f64; 2];
        for h in &clause.heads {
            d[value_of(values, &h.atom)?] += h.prob.unwrap_or(1.0);
        }
        let total = d[0] + d[1];
        if (total - 1.0).abs() > SUM_TOLERANCE {
            return None;
        }
        dist[key] = [d[0] / total, d[1] / total];
    }
    Some(parents)
}

/// The noisy-or forms of a table's rows `dist`, by absent value: one for
/// each value that can be its absent one.
fn noisy_or_forms(dist: &[[f64; 2]]) -> [Option<Form>; 2] {
    let m = dist.len().trailing_zeros() as usize;
    let mut forms = [None; 2];
    for (absent, form) in forms.iter_mut().enumerate() {
        let p = |key: usize| dist[key][absent];
        let base = (0..dist.len()).fold(0, |best, key| if p(key) > p(best) { key } else { best });
        let leak = p(base);
        if leak <= 0.0 {
            continue;
        }
        let f = Form { absent, base, leak };
        let mut q = [0.0; MOST_PARENTS];
        for (i, qi) in q[..m].iter_mut().enumerate() {
            *qi = f.q(dist, i);
        }
        let fits = (0..dist.len()).all(|key| {
            let off = key ^ base;
            let want = (0..m)
                .filter(|&i| off >> i & 1 == 1)
                .fold(leak, |acc, i| acc * q[i]);
            (p(key) - want).abs() <= TOLERANCE
        });
        if fits {
            *form = Some(f);
        }
    }
    forms
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::ground::table::ground_rows;
    use crate::syntax::{Term, parse_program};

    /// The program `source` rewritten, as text, and how many tables were.
    /// The rewritten program is table-shaped, and its grounding is the one
    /// the shape made from `source`'s gives ([`Shape::rewritten`]).
    fn rewritten(source: &str) -> Option<(String, usize)> {
        let program = parse_program(source).unwrap();
        let (p, n) = rewritten_alike(&program)?;
        Some((p.to_string(), n))
    }

    /// `program` rewritten, and how many tables were, where its shape
    /// rewritten grounds as the program rewritten does.
    fn rewritten_alike(program: &Program) -> Option<(Program, usize)> {
        let shape = Shape::of(program)?;
        let (written, n) = rewrite(&shape, &shape.evaluation_order()?, program)?;
        // The program written out: each table taken at its first clause,
        // the clauses numbered in order.
        let mut clauses = Vec::new();
        let mut done = vec![false; written.len()];
        for clause in &program.clauses {
            let t = shape
                .predicate_of(&clause.heads[0].atom)
                .expect("a clause heads a table");
            match &written[t] {
                None => clauses.push(clause.clone()),
                Some(w) if !std::mem::replace(&mut done[t], true) => {
                    clauses.extend(w.iter().map(|(c, _)| c.clone()))
                }
                Some(_) => {}
            }
        }
        for (g, clause) in clauses.iter_mut().enumerate() {
            clause.group = g as u32;
        }
        let p = Program {
            clauses,
            query_rules: Vec::new(),
            queries: program.queries.clone(),
            evidence: program.evidence.clone(),
            interleaved: Vec::new(),
        };
        let rows = ground_rows(&p)
            .unwrap_or_else(|| panic!("the rewritten program is table-shaped:\n{p}"));
        let shape = shape.rewritten(program, &written);
        let order = shape.evaluation_order().expect("no cycle");
        let gp = shape.ground(program, &order);
        assert_eq!(gp.atoms, rows.atoms, "atoms of\n{p}");
        assert_eq!(gp.kinds, rows.kinds, "kinds of\n{p}");
        assert_eq!(gp.rules, rows.rules, "rules of\n{p}");
        assert_eq!(gp.queries, rows.queries, "queries of\n{p}");
        assert_eq!(gp.evidence, rows.evidence, "evidence of\n{p}");
        Some((p, n))
    }

    #[test]
    fn a_chain_of_noisy_ors_is_one_rule_per_cause() {
        // Two causes, a two-parent noisy-or of them (0.6 and 0.5 inhibit,
        // no leak) and a link that adds a third cause (0.5 inhibits) with a
        // leak of 0.1: as gen_bn.py writes the UAI or_chain networks.
        let source = "\
0.9::d1(v_0); 0.1::d1(v_1).
0.8::d2(v_0); 0.2::d2(v_1).
0.7::d3(v_0); 0.3::d3(v_1).
x(v_0) :- d1(v_0), d2(v_0).
0.5::x(v_0); 0.5::x(v_1) :- d1(v_0), d2(v_1).
0.6::x(v_0); 0.4::x(v_1) :- d1(v_1), d2(v_0).
0.3::x(v_0); 0.7::x(v_1) :- d1(v_1), d2(v_1).
0.9::f(v_0); 0.1::f(v_1) :- d3(v_0), x(v_0).
f(v_1) :- d3(v_0), x(v_1).
0.45::f(v_0); 0.55::f(v_1) :- d3(v_1), x(v_0).
f(v_1) :- d3(v_1), x(v_1).
evidence(f(v_1)).
";
        let (text, n) = rewritten(source).expect("rewritten");
        assert_eq!(n, 5);
        let want = "\
0.1::d1(v_1).
0.2::d2(v_1).
0.3::d3(v_1).
0.4::x(v_1) :- d1(v_1).
0.5::x(v_1) :- d2(v_1).
0.1::f(v_1).
0.5::f(v_1) :- d3(v_1).
f(v_1) :- x(v_1).
evidence(f(v_1),true).
";
        assert_eq!(text, want);
    }

    #[test]
    fn a_value_read_asked_or_observed_stays() {
        let table = "\
0.9::d(v_0); 0.1::d(v_1).
0.7::x(v_0); 0.3::x(v_1) :- d(v_0).
0.2::x(v_0); 0.8::x(v_1) :- d(v_1).
";
        // Observed at both values, or asked at both: x keeps its table,
        // and d is read at both values.
        assert_eq!(
            rewritten(&format!(
                "{table}evidence(x(v_0)).\nevidence(x(v_1),false).\n"
            )),
            None
        );
        assert_eq!(
            rewritten(&format!("{table}query(x(v_0)).\nquery(x(v_1)).\n")),
            None
        );
        // Read at both values by a table that is no noisy-or (an xor), and
        // so the parents of each.
        let xor = "\
0.5::y(v_0); 0.5::y(v_1).
z(v_0) :- x(v_0), y(v_0).
z(v_1) :- x(v_0), y(v_1).
z(v_1) :- x(v_1), y(v_0).
z(v_0) :- x(v_1), y(v_1).
query(z(v_1)).
";
        assert_eq!(rewritten(&format!("{table}{xor}")), None);
        // Asked at one value: written as its noisy-or, with d a fact.
        let (text, n) = rewritten(&format!("{table}query(x(v_1)).\n")).expect("rewritten");
        assert_eq!(n, 2);
        assert_eq!(
            text,
            "0.1::d(v_1).\n0.3::x(v_1).\n0.7142857142857142::x(v_1) :- d(v_1).\nquery(x(v_1)).\n"
        );
    }

    #[test]
    fn an_observed_table_whose_parent_stays_stays() {
        // y, a noisy-or of d and e, and z, a noisy-or of e, are observed
        // present, and d is asked at both values. Written as a noisy-or, y
        // would read d at one value while d stays a table: y stays, e, which
        // y then reads at both values, stays, and z with it.
        let tables = "\
0.9::d(v_0); 0.1::d(v_1).
0.8::e(v_0); 0.2::e(v_1).
y(v_0) :- d(v_0), e(v_0).
0.5::y(v_0); 0.5::y(v_1) :- d(v_0), e(v_1).
0.4::y(v_0); 0.6::y(v_1) :- d(v_1), e(v_0).
0.2::y(v_0); 0.8::y(v_1) :- d(v_1), e(v_1).
0.7::z(v_0); 0.3::z(v_1) :- e(v_0).
0.1::z(v_0); 0.9::z(v_1) :- e(v_1).
";
        let observed = "evidence(y(v_1)).\nevidence(z(v_1)).\n";
        assert_eq!(
            rewritten(&format!(
                "{tables}{observed}query(d(v_0)).\nquery(d(v_1)).\n"
            )),
            None
        );
        // Asked at one value, d is written as a fact, and the four tables
        // are written.
        assert_eq!(
            rewritten(&format!("{tables}{observed}query(d(v_1)).\n")).map(|(_, n)| n),
            Some(4)
        );
        // Not observed, y is written over d, which stays, and so are e, z
        // and w, which reads y's value and is observed.
        let read = "\
0.6::w(v_0); 0.4::w(v_1) :- y(v_0).
0.1::w(v_0); 0.9::w(v_1) :- y(v_1).
evidence(w(v_1)).
evidence(z(v_1)).
query(d(v_0)).
query(d(v_1)).
";
        let (text, n) = rewritten(&format!("{tables}{read}")).expect("rewritten");
        assert_eq!(n, 4, "{text}");
        assert!(
            text.contains("0.6::y(v_1) :- d(v_1).\n")
                && text.contains("0.9::d(v_0); 0.1::d(v_1).\n"),
            "{text}"
        );
    }

    /// `program` with each name held anew, shared with no other atom.
    fn held_apart(program: &Program) -> Program {
        let apart = |a: &Atom| Atom {
            name: Arc::from(&*a.name),
            args: a
                .args
                .iter()
                .map(|t| {
                    if let Term::Const(c) = t {
                        Term::Const(Arc::from(&**c))
                    } else {
                        t.clone()
                    }
                })
                .collect(),
        };
        let mut p = program.clone();
        for c in &mut p.clauses {
            for h in &mut c.heads {
                h.atom = apart(&h.atom);
            }
            for l in &mut c.body {
                if let Literal::Pos(a) | Literal::Neg(a) = l {
                    *a = apart(a);
                }
            }
        }
        for (a, _) in p.queries.iter_mut().chain(&mut p.evidence) {
            *a = apart(a);
        }
        p
    }

    #[test]
    fn values_are_told_by_their_text_where_names_are_not_shared() {
        // A value is told by its names, by address first ([`value_of`]),
        // as the parser holds one name per text; with every name held
        // anew, by their text, to the same tables written.
        let source = "\
0.5::a(v_0); 0.5::a(v_1).
0.25::b(v_0); 0.75::b(v_1).
x(v_0) :- a(v_0), b(v_0).
x(v_0) :- a(v_0), b(v_1).
x(v_0) :- a(v_1), b(v_0).
x(v_1) :- a(v_1), b(v_1).
0.6::c(v_0); 0.4::c(v_1) :- x(v_0).
0.3::c(v_0); 0.7::c(v_1) :- x(v_1).
evidence(c(v_0)).
";
        let program = parse_program(source).unwrap();
        let (p, n) = rewritten_alike(&program).expect("rewritten");
        let (q, m) =
            rewritten_alike(&held_apart(&program)).expect("rewritten with its names apart");
        assert_eq!(n, 4);
        assert_eq!((q.to_string(), m), (p.to_string(), n));
    }

    #[test]
    fn an_and_is_an_or_of_its_absent_values() {
        // a ∧ b written as a table: its value v_0 holds where either parent
        // is v_0, and v_1 is the absent value where only v_0 is read.
        let source = "\
0.5::a(v_0); 0.5::a(v_1).
0.25::b(v_0); 0.75::b(v_1).
x(v_0) :- a(v_0), b(v_0).
x(v_0) :- a(v_0), b(v_1).
x(v_0) :- a(v_1), b(v_0).
x(v_1) :- a(v_1), b(v_1).
evidence(x(v_0)).
";
        let (text, n) = rewritten(source).expect("rewritten");
        assert_eq!(n, 3);
        assert_eq!(
            text,
            "0.5::a(v_0).\n0.25::b(v_0).\nx(v_0) :- a(v_0).\nx(v_0) :- b(v_0).\nevidence(x(v_0),true).\n"
        );
    }

    #[test]
    fn a_table_that_is_no_noisy_or_or_no_partition_stays() {
        // 0.3 is not 0.5 · 0.4: no noisy-or with either absent value, and
        // its parents are read at both values.
        let source = "\
0.5::a(v_0); 0.5::a(v_1).
0.5::b(v_0); 0.5::b(v_1).
x(v_0) :- a(v_0), b(v_0).
0.5::x(v_0); 0.5::x(v_1) :- a(v_0), b(v_1).
0.4::x(v_0); 0.6::x(v_1) :- a(v_1), b(v_0).
0.3::x(v_0); 0.7::x(v_1) :- a(v_1), b(v_1).
evidence(x(v_1)).
";
        assert_eq!(rewritten(source), None);
        // A table of one value is no table of two: a is read at v_0 alone.
        let one = "\
0.5::a(v_0); 0.5::a(v_1).
x(v_0) :- a(v_0).
evidence(x(v_0)).
";
        assert_eq!(
            rewritten(one),
            Some((
                "0.5::a(v_0).\nx(v_0) :- a(v_0).\nevidence(x(v_0),true).\n".to_string(),
                1
            ))
        );
        // A row missing or twice: the rows do not partition the parent's
        // values, and x stays as written; a, read at v_0 alone, is a fact.
        let twice = "\
0.5::a(v_0); 0.5::a(v_1).
0.5::x(v_0); 0.5::x(v_1) :- a(v_0).
0.5::x(v_0); 0.5::x(v_1) :- a(v_0).
evidence(x(v_1)).
";
        let kept = "0.5::x(v_0); 0.5::x(v_1) :- a(v_0).\n";
        let want = format!("0.5::a(v_0).\n{kept}{kept}evidence(x(v_1),true).\n");
        assert_eq!(rewritten(twice), Some((want, 1)));
        // A program with variables is no table.
        assert_eq!(rewritten("0.5::p(X) :- q(X).\nq(1).\nquery(p(1)).\n"), None);
    }

    #[test]
    fn the_shape_rewritten_numbers_as_the_program_rewritten() {
        // x's clauses apart in the program, read before its parents' own,
        // the first at e, which x written as a noisy-or leaves out (its q is
        // one), and d. The program written meets d before e, and z, between
        // x's clauses, before e's: its predicates are numbered w, x, d, z,
        // e, against w, x, e, d, z, and evaluated d, x, w, z, e, against
        // e, d, x, w, z.
        let source = "\
w(v_0) :- x(v_0).
0.3::w(v_0); 0.7::w(v_1) :- x(v_1).
x(v_0) :- e(v_0), d(v_0).
0.5::x(v_0); 0.5::x(v_1) :- e(v_0), d(v_1).
0.2::z(v_0); 0.8::z(v_1) :- d(v_0).
x(v_0) :- e(v_1), d(v_0).
0.5::x(v_0); 0.5::x(v_1) :- e(v_1), d(v_1).
0.4::z(v_0); 0.6::z(v_1) :- d(v_1).
0.9::d(v_0); 0.1::d(v_1).
0.5::e(v_0); 0.5::e(v_1).
query(w(v_1)).
query(z(v_0)).
query(z(v_1)).
evidence(e(v_1)).
";
        let (text, n) = rewritten(source).expect("rewritten");
        assert_eq!(n, 3);
        let want = "\
0.7::w(v_1) :- x(v_1).
0.5::x(v_1) :- d(v_1).
0.2::z(v_0); 0.8::z(v_1) :- d(v_0).
0.4::z(v_0); 0.6::z(v_1) :- d(v_1).
0.9::d(v_0); 0.1::d(v_1).
0.5::e(v_1).
evidence(e(v_1),true).
query(w(v_1)).
query(z(v_0)).
query(z(v_1)).
";
        assert_eq!(text, want);
    }
}
