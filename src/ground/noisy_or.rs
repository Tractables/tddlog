//! Noisy-or tables: a table of two values over parents of two values whose
//! one value is a noisy-or of the parents, written as that noisy-or where
//! nothing reads or asks its other value.
//!
//! A Bayesian network written one annotated disjunction per row
//! (`tddlog --program` of a network) holds a table `x` of values `b` and `c` over
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
//! rewritten too, and no query names it, and for a table observed only
//! where its parents are taken (below). An observation of `x(c)` is one
//! of `x(b)`, negated: each world holds exactly one of the two, and a
//! diagnosis network's findings are often observed absent so. A root, a
//! table of no parents, is a leak alone, a probabilistic fact. Every
//! other atom keeps its distribution, jointly with the others.
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
//! A table taken observed at its absent value goes further: the
//! observation, all of its causes off, reaches every table taken below
//! it, and each of those is taken only where its parents are, as an
//! observed table is. A diagnosis network's findings observed absent read
//! diseases, which are taken; the PR set's mastermind networks observe
//! gates absent over gates that stay, and written over them 31 of the 38
//! compiled differently, with 2.5 times the work in all.
//!
//! Taken where the program is table-shaped, its tables read from the
//! [`Shape`] the table path grounds ([`super::table`]), each table a
//! predicate whose head atoms are its values, and its rows sum to one up
//! to rounding as [`super::normalize`] reads them; the entries must match
//! the noisy-or to [`TOLERANCE`]. The rewritten program is table-shaped
//! too, and takes the table path: its shape is made from the program's
//! ([`Shape::rewritten`]) with each table taken written at its first
//! clause.
//!
//! **Tables of one atom.** A two-valued variable written as one atom `x`,
//! its other value `\+x`, and its table as one rule per row,
//!
//! ```text
//! p_1::x :- y_1, \+y_2.   p_2::x :- \+y_1, y_2.   ...
//! ```
//!
//! each row reading every parent once, as a literal of either sign, is no
//! table-shaped program: the grounder reads it through the generic steps,
//! and each row is a rule whose body is the row's literals and the facts
//! of its own that the normalization of `p_i::` made, read by no other
//! rule. The same noisy-or is found there, on the ground program
//! ([`rows_as_noisy_ors`]): a derived atom, in no cycle, whose two or more
//! rules read the same two or more parents and each a different
//! assignment of them, the rest of each body facts no other rule reads
//! and no query or observation names, is a table of `x` over its parents,
//! a row missing an assignment being one where `x` is false, and where
//! `x` is its noisy-or (`\+x` the absent value) its rules are written as
//! the noisy-or's, each fact a new one. `x` stays an atom, its readers,
//! queries and observations as they were, since the facts its rules read
//! were its rows' own. A table of one parent is left as it is: its
//! noisy-or has as many rules and facts as its rows.

use rustc_hash::{FxHashMap, FxHashSet};

use super::normalize::SUM_TOLERANCE;
use super::table::{Shape, Switched};
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, Prob};
use crate::syntax::{Annotation, Atom, Clause, HeadChoice, Literal, Program, Term};

/// How far an entry of a table may be from the noisy-or's.
const TOLERANCE: f64 = 1e-12;

/// The most parents of a table taken: one row per assignment.
const MOST_PARENTS: usize = 16;

/// A table of one atom: its parents and its rows by assignment, as
/// [`one_atom_rows`] reads them.
type AtomTable = (Vec<AtomId>, Vec<[f64; 2]>);

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

/// The noisy-or tables of a program written as noisy-ors ([`rewrite`]).
pub(super) struct Rewritten {
    /// Per table taken, its clauses.
    pub(super) written: Vec<Option<Written>>,
    /// How many tables were taken.
    pub(super) count: usize,
    /// The program's evidence, an observation of the absent value of a
    /// table taken written as one of its present value, negated; `None`
    /// where no observation is of an absent value.
    pub(super) evidence: Option<Vec<(Atom, bool)>>,
}

/// The noisy-or tables of `program`, of the tables `shape` in the
/// evaluation order `order` (each after the tables it reads), written as
/// noisy-ors; `None` where there is none to write.
pub(super) fn rewrite(shape: &Shape, order: &[usize], program: &Program) -> Option<Rewritten> {
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

    let chosen = choose(shape, order, program, &value, &rows, &dists);
    let count = chosen.iter().flatten().count();
    if count == 0 {
        return None;
    }
    let written = (0..n)
        .map(|t| {
            let f = chosen[t]?;
            let r = rows[t].as_ref().expect("a table taken has rows");
            Some(write(t, r, f, &values, &dists))
        })
        .collect();
    // An observation of a taken table's absent value, as one of its
    // present value.
    let present = |a: &Atom| {
        let p = shape.predicate_of(a)?;
        let f = chosen[p]?;
        (value(p, a)? == f.absent)
            .then(|| values[p].expect("a table taken has values")[1 - f.absent])
    };
    let evidence = program
        .evidence
        .iter()
        .any(|(a, _)| present(a).is_some())
        .then(|| {
            program
                .evidence
                .iter()
                .map(|(a, v)| match present(a) {
                    Some(b) => (b.clone(), !v),
                    None => (a.clone(), *v),
                })
                .collect()
        });
    Some(Rewritten {
        written,
        count,
        evidence,
    })
}

/// `gp`, ground from a program whose noisy-or tables [`rewrite`] wrote,
/// observing as `evidence`, [`Rewritten::evidence`], says: each
/// observation `gp` holds of an absent value moved to the present value,
/// negated, its atom numbered as `gp` numbers it.
pub(super) fn observe(gp: &mut GroundProgram, evidence: &[(Atom, bool)]) {
    let moved: FxHashSet<&Atom> = evidence
        .iter()
        .zip(&gp.evidence)
        .filter(|((a, _), (b, _, _))| a != b)
        .map(|((a, _), _)| a)
        .collect();
    let mut id: FxHashMap<&Atom, AtomId> = FxHashMap::default();
    for (i, a) in gp.atoms.iter().enumerate() {
        if moved.contains(a) {
            id.insert(a, i as AtomId);
        }
    }
    let observed: Vec<(Atom, Option<AtomId>, bool)> = evidence
        .iter()
        .zip(&gp.evidence)
        .map(|((a, v), e)| {
            if moved.contains(a) {
                (a.clone(), id.get(a).copied(), *v)
            } else {
                e.clone()
            }
        })
        .collect();
    gp.evidence = observed;
}

/// The form each table of `rows` is taken in, `None` for a table not taken:
/// the absent value of each taken is one nothing reads or asks,
/// once the tables not taken read every value their rows read and those
/// taken the parents' values away from `u*`; and a table observed is taken
/// only where its parents are.
fn choose(
    shape: &Shape,
    order: &[usize],
    program: &Program,
    value: &impl Fn(usize, &Atom) -> Option<usize>,
    rows: &[Option<Rows>],
    dists: &[[f64; 2]],
) -> Vec<Option<Form>> {
    let n = shape.predicates();
    let mut needed = vec![0u8; n];
    let mut observed = vec![0u8; n];
    let asked = program.queries.iter().map(|(a, _)| (a, false));
    for (a, evidence) in asked.chain(program.evidence.iter().map(|(a, _)| (a, true))) {
        if let Some(p) = shape.predicate_of(a)
            && let Some(v) = value(p, a)
        {
            // An observation of either value is one of the value written;
            // a form whose absent value is not observed comes first.
            if evidence {
                observed[p] |= 1 << v;
            } else {
                needed[p] |= 1 << v;
            }
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
    // observed or under an observation of an absent value has its parents
    // taken (above). A table is under such an observation where a table
    // taken observed at the value its form leaves out reads it, or a
    // table taken under one does.
    let mut changed = true;
    while changed {
        changed = false;
        let mut under = vec![false; n];
        for &t in order.iter().rev() {
            if !taken[t] {
                continue;
            }
            let r = rows[t].as_ref().expect("a table taken has rows");
            let open = |f: &Form| needed[t] & (1 << f.absent) == 0;
            // How many parents a form of an observed table would leave with
            // no form open: the table would not be taken then.
            let blocked = |f: &Form| {
                let dist = r.dist(dists);
                r.parents
                    .iter()
                    .enumerate()
                    .filter(|&(i, &p)| {
                        let need = needed[p] | 1 << (f.base >> i & 1 ^ 1);
                        f.q(dist, i) < 1.0
                            && !(taken[p]
                                && rows[p].as_ref().is_some_and(|rp| {
                                    rp.forms
                                        .iter()
                                        .flatten()
                                        .any(|g| need & (1 << g.absent) == 0)
                                }))
                    })
                    .count()
            };
            let watched = observed[t] != 0 || under[t];
            let form = if !watched || r.parents.iter().all(|&p| taken[p]) {
                match chosen[t] {
                    Some(f) if open(&f) => Some(f),
                    _ => {
                        changed = true;
                        let mut forms = r.forms.iter().flatten().filter(|f| open(f));
                        if watched {
                            forms
                                .min_by_key(|f| (blocked(f), observed[t] >> f.absent & 1))
                                .copied()
                        } else {
                            forms.next().copied()
                        }
                    }
                }
            } else {
                changed = true;
                None
            };
            match form {
                Some(f) => {
                    let dist = r.dist(dists);
                    let below = under[t] || observed[t] >> f.absent & 1 == 1;
                    for (i, &p) in r.parents.iter().enumerate() {
                        if f.q(dist, i) < 1.0 {
                            needed[p] |= 1 << (f.base >> i & 1 ^ 1);
                            under[p] |= below;
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
    chosen
}

/// Table `t` of the rows `r`, taken in the form `f`, as clauses: the row
/// `u*` as written gives the leak; where `c` is certain there, each
/// flipped row as written gives its cause.
fn write(
    t: usize,
    r: &Rows,
    f: Form,
    values: &[Option<[&Atom; 2]>],
    dists: &[[f64; 2]],
) -> Written {
    let dist = r.dist(dists);
    let of = |p: usize| values[p].expect("a table taken and its parents are candidates");
    let present = of(t)[1 - f.absent];
    let head = |p: f64| {
        vec![HeadChoice {
            prob: (p < 1.0).then_some(Annotation::Number(p)),
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
    out
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
            d[value_of(values, &h.atom)?] += h.mass()?;
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

/// The tables of one atom of `gp` written as noisy-ors (above, "Tables of
/// one atom"): how many were. Their rows' facts are left unread, for the
/// relevance cut to take out.
pub(super) fn rows_as_noisy_ors(gp: &mut GroundProgram) -> usize {
    let n = gp.num_atoms();
    // How many literals read each atom, a named atom counted as read
    // twice: a fact read once and named by nothing is a row's own.
    let mut reads = vec![0u8; n];
    for l in gp.rules.iter().flat_map(|r| &r.body) {
        let k = &mut reads[l.atom() as usize];
        *k = k.saturating_add(1);
    }
    for a in gp.named_atoms() {
        reads[a as usize] = 2;
    }
    let own = |l: GLit| reads[l.atom() as usize] == 1 && gp.is_fact(l.atom());
    let by_head = gp.rules_by_head();
    let mut found: Vec<(AtomId, AtomTable, Form)> = Vec::new();
    for (x, rules) in by_head.iter().enumerate() {
        if rules.len() < 2 {
            continue;
        }
        let x = x as AtomId;
        if let Some((parents, dist)) = one_atom_rows(gp, x, rules, &own)
            && let [Some(f), _] = noisy_or_forms(&dist)
        {
            found.push((x, (parents, dist), f));
        }
    }
    if found.is_empty() {
        return 0;
    }
    // A table's parents do not depend on it where it is in no cycle.
    let mut cyclic = vec![false; n];
    for c in super::deps::derived_sccs(gp, &by_head) {
        if super::deps::is_cyclic(gp, &by_head, &c) {
            for a in c {
                cyclic[a as usize] = true;
            }
        }
    }
    found.retain(|(x, ..)| !cyclic[*x as usize]);
    if found.is_empty() {
        return 0;
    }
    let mut dropped = vec![false; gp.rules.len()];
    let mut written = Vec::new();
    for (x, (parents, dist), f) in &found {
        for &ri in &by_head[*x as usize] {
            dropped[ri] = true;
        }
        let fact = |gp: &mut GroundProgram, w: Prob| {
            let k = gp.num_atoms() as i64;
            let name = &gp.atoms[*x as usize];
            let mut args = name.args.as_slice().to_vec();
            args.push(Term::Int(k));
            gp.atoms
                .push(Atom::new(format!("{}$noisy_or", name.name), args));
            gp.kinds.push(AtomKind::Fact(w));
            k as AtomId
        };
        // The leak, `x` where it is likeliest absent; each parent at its
        // value away from that row, as in the table path's [`write`].
        let leak = dist[f.base][1];
        if leak > 0.0 {
            let l = fact(
                gp,
                Prob {
                    p: leak,
                    not: dist[f.base][0],
                },
            );
            written.push(GroundRule::new(*x, vec![GLit::Pos(l)]));
        }
        for (k, &p) in parents.iter().enumerate() {
            let q = f.q(dist, k);
            if q >= 1.0 {
                continue;
            }
            let flipped = f.base ^ 1 << k;
            let w = if f.leak == 1.0 {
                Prob {
                    p: dist[flipped][1],
                    not: dist[flipped][0],
                }
            } else {
                Prob { p: 1.0 - q, not: q }
            };
            let active = GLit::new(p, f.base >> k & 1 == 0);
            let body = if w.not == 0.0 {
                vec![active]
            } else {
                vec![active, GLit::Pos(fact(gp, w))]
            };
            written.push(GroundRule::new(*x, body));
        }
    }
    let mut i = 0;
    gp.rules.retain(|_| {
        i += 1;
        !dropped[i - 1]
    });
    gp.rules.extend(written);
    gp.structure = std::sync::OnceLock::new();
    found.len()
}

/// The parents of the rules `rules` of `x` in `gp` and their rows, by
/// assignment (bit `i` the sign of parent `i`): the probabilities of `x`
/// false and true, a row no rule has `x` false; `None` unless every rule
/// reads every parent once and a different assignment, the rest of its
/// body facts of its own (`own`) whose weights sum to one.
fn one_atom_rows(
    gp: &GroundProgram,
    x: AtomId,
    rules: &[usize],
    own: &impl Fn(GLit) -> bool,
) -> Option<AtomTable> {
    let mut parents: Vec<AtomId> = gp.rules[rules[0]]
        .body
        .iter()
        .filter(|&&l| !own(l))
        .map(|l| l.atom())
        .collect();
    parents.sort_unstable();
    let m = parents.len();
    if !(2..=MOST_PARENTS).contains(&m)
        || rules.len() > 1 << m
        || parents.windows(2).any(|w| w[0] == w[1])
        || parents.contains(&x)
    {
        return None;
    }
    let mut dist = vec![[1.0, 0.0]; 1 << m];
    let mut taken = vec![false; 1 << m];
    for &ri in rules {
        if gp.rules[ri].pairs {
            return None;
        }
        let (mut key, mut seen) = (0usize, 0usize);
        let mut row = Prob::TRUE;
        for &l in &gp.rules[ri].body {
            if own(l) {
                let AtomKind::Fact(w) = gp.kinds[l.atom() as usize] else {
                    unreachable!("a row's own atom is a fact")
                };
                row = Prob::cube([row, w.literal(l.is_positive())]);
                continue;
            }
            let k = parents.binary_search(&l.atom()).ok()?;
            if seen >> k & 1 == 1 {
                return None;
            }
            seen |= 1 << k;
            key |= usize::from(l.is_positive()) << k;
        }
        let total = row.p + row.not;
        if seen != (1 << m) - 1 || taken[key] || (total - 1.0).abs() > SUM_TOLERANCE {
            return None;
        }
        taken[key] = true;
        dist[key] = [row.not / total, row.p / total];
    }
    Some((parents, dist))
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
        let Rewritten {
            written,
            count: n,
            evidence,
        } = rewrite(&shape, &shape.evaluation_order()?, program)?;
        // The program written out: each table taken at its first clause,
        // the clauses numbered in order, observing what `rewrite` says.
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
            evidence: evidence.clone().unwrap_or_else(|| program.evidence.clone()),
            interleaved: Vec::new(),
            unknown_fails: program.unknown_fails,
            aggregate: program.aggregate,
        };
        let rows = ground_rows(&p)
            .unwrap_or_else(|| panic!("the rewritten program is table-shaped:\n{p}"));
        let shape = shape.rewritten(program, &written);
        let order = shape.evaluation_order().expect("no cycle");
        let mut gp = shape.ground(program, &order);
        if let Some(evidence) = &evidence {
            observe(&mut gp, evidence);
        }
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
        // leak of 0.1: as the UAI or_chain networks are read.
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
        // Asked at both values: x keeps its table, and d is read at both
        // values.
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
    fn an_observation_of_the_absent_value_is_one_of_the_present_value_negated() {
        // x observed at v_0, its absent value, at both values, or at v_1
        // false: written as its noisy-or, observed at v_1 false, as a
        // diagnosis program observes a finding absent.
        let table = "\
0.9::d(v_0); 0.1::d(v_1).
0.7::x(v_0); 0.3::x(v_1) :- d(v_0).
0.2::x(v_0); 0.8::x(v_1) :- d(v_1).
";
        let written = "0.1::d(v_1).\n0.3::x(v_1).\n0.7142857142857142::x(v_1) :- d(v_1).\n";
        for evidence in [
            "evidence(x(v_0)).\n",
            "evidence(x(v_0)).\nevidence(x(v_1),false).\n",
            "evidence(x(v_1),false).\n",
        ] {
            let (text, n) = rewritten(&format!("{table}{evidence}query(d(v_1)).\n"))
                .unwrap_or_else(|| panic!("{evidence}: rewritten"));
            assert_eq!(n, 2, "{evidence}");
            assert!(
                text.starts_with(written) && text.contains("evidence(x(v_1),false).\n"),
                "{evidence}: {text}"
            );
        }
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
        // Observed absent, w is under its observation, and so is y, which
        // it reads: y reads d, which stays, and stays a table; w, whose
        // parent stays, stays too, and e and z with them.
        let absent = read.replace("evidence(w(v_1)).", "evidence(w(v_0)).");
        assert_eq!(rewritten(&format!("{tables}{absent}")), None);
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
