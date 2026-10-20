//! Renormalizing the rows of tables over the heads the evidence leaves
//! them, part of the conditioning on the evidence ([`super::condition`]).
//! A row of a table, an annotated disjunction's chain of switches under
//! one assignment of its parents, that the conditioning leaves some heads
//! false and others not is renormalized over the heads it keeps
//! ([`renormalize`]): a chain over those heads alone, and the mass it
//! keeps on a fact its context requires, so that the false heads leave the
//! program and the evidence walk reads the table's level without them. A
//! row that keeps a head made true reads one fact of the head's
//! probability instead, where that is sound. Each world counted keeps its
//! weight, and the masses taken out are a factor of both counts.

use rustc_hash::FxHashMap;

use super::normalize::chain_weights;
use super::propagation::Val;
use super::tables::Tables;
use super::{AtomId, AtomKind, BITS_FROM, GLit, GroundProgram, GroundRule, Prob};
use crate::eval::Count;
use crate::syntax::{Atom, Term};

/// The mass of no head of a row whose rules' own literals are `chains`,
/// where they are an annotated disjunction's chain: in some order, chain
/// `i` the negations of the switches of chains `0` to `i - 1` and switch
/// `i`, the last one perhaps the negations alone. That is the product of
/// the negations of the switches, each its own weight, or zero where the
/// last chain is the negations alone. None for chains of another shape.
fn no_head(chains: &[Vec<GLit>], weight_of: &impl Fn(GLit) -> Prob) -> Option<f64> {
    let mut order: Vec<&Vec<GLit>> = chains.iter().collect();
    order.sort_by_key(|c| c.iter().filter(|l| !l.is_positive()).count());
    let mut switches: Vec<GLit> = Vec::with_capacity(order.len());
    for (i, chain) in order.iter().enumerate() {
        let positive: Vec<GLit> = chain.iter().copied().filter(|l| l.is_positive()).collect();
        let mut negations: Vec<GLit> = chain
            .iter()
            .copied()
            .filter(|l| !l.is_positive())
            .map(|l| GLit::Pos(l.atom()))
            .collect();
        negations.sort_unstable();
        let mut before = switches.clone();
        before.sort_unstable();
        if negations != before {
            return None;
        }
        match positive[..] {
            [s] => switches.push(s),
            [] if i + 1 == order.len() => return Some(0.0),
            _ => return None,
        }
    }
    Some(Prob::cube(switches.iter().map(|&s| weight_of(GLit::Neg(s.atom())))).p)
}

/// The rows of `gp`'s tables that lose heads to the conditioning
/// renormalized over the heads they keep, after the conditioning's rewrite
/// with the values `val` it was given.
///
/// A row is the rules of a table ([`Tables`]) with one context, the body
/// literals that are not the table's own facts: an annotated disjunction's
/// chain of switches under one assignment of its parents. Where the
/// conditioning made some of its heads false and left others, the row
/// weighs `θ_h`, the product of its switches' weights along `h`'s chain,
/// for each head `h`, and keeps the mass `m`, the `θ` of the heads not
/// false and what the row leaves to no head. It becomes a chain over the
/// heads kept with weights `θ_h / m`, and `m` a fact `w` that the context
/// requires: `bad :- context, \+w`, observed false. In each world counted
/// the row's head is one it keeps (the others are false), and it weighs
/// `θ_h / m · m`, as before. A row without a context puts `m` into the
/// conditioning's factor. Rows that keep no head, or lose no mass, stay.
///
/// `bad` is an atom of its own, one for all the tables, unless a context
/// of the table reads a set of [`BITS_FROM`] values or more, which the
/// indicators read in binary. Such a table's rows put `w` first in the
/// chain of the heads they keep (`h :- context, w, ...`; sound, since the
/// evidence forces `w` where the context holds) and their masses' rules
/// on a head of the table the conditioning made false, which stays
/// observed false: each row's false heads merged into one, first in its
/// chain, in the table's own unit. Read through an atom of their own,
/// the masses of a table over a set read in binary do not compile in
/// memory (bn_mildew_ev's `dm_4`, 210 rows over the 100 values of
/// `dm_3`, where most of its values are ruled out); elsewhere the atom
/// of their own compiles smaller.
///
/// The chains of a row must exclude each other, and its switches be read
/// by no other row, but where the table's contexts exclude each other
/// pairwise: each reads one atom of each of the same exclusive sets,
/// positively, and nothing else, as the rows of a network's table do.
/// Such a table's rows share their new switches (the `k`-th of each
/// probability) and their masses' facts, as [`crate::ground::share`] does: in a
/// world where one row's context holds the others' do not, and a fact read
/// under them sums out of their part.
///
/// A row that keeps a head `h` the conditioning made true is not
/// renormalized. In a table whose contexts exclude each other pairwise,
/// `h` holds where the row's context does through that row alone (its
/// rules are the table's, and no other row's context holds there), so the
/// row weighs `θ_h` there: it becomes `h :- context, t` with a fact `t` of
/// weight `θ_h`, shared by the rows of one `θ_h`, and its false heads go.
/// That is not done where a context reads a set read in binary, and a row
/// of any other table stays as it is: the head's observation carries the
/// evidence. The rows of an observed child of two sets read in binary
/// (tests/it/binary.rs) did not compile in memory renormalized, the mass's
/// constraint beside the head's observation, nor as `h :- context, t`.
/// Returns the rows rewritten and the factor.
pub(super) fn renormalize(gp: &mut GroundProgram, tables: &Tables, val: &[Val]) -> (usize, Count) {
    let n = gp.num_atoms();
    let num_sets = tables.sets().len();
    let false_head =
        |h: AtomId| gp.kinds[h as usize] == AtomKind::Derived && val[h as usize] == Val::False;
    // Each table's heads: whether one is false, one neither false nor
    // true, one true; and whether a rule of it reads a fact not named, and
    // one of its own facts (below). A row can change only in a table with
    // a false head and one not false.
    let mut kinds = vec![[false; 5]; num_sets + n];
    for r in &gp.rules {
        kinds[tables.of(r.head)][if false_head(r.head) {
            0
        } else if val[r.head as usize] == Val::True {
            2
        } else {
            1
        }] = true;
    }
    let may = |e: &[bool; 5]| e[0] && (e[1] || e[2]);
    if !kinds.iter().any(may) {
        return (0, Count::ONE);
    }
    // Each table's own facts as the program reads them now: facts not
    // named, of probability strictly between 0 and 1 or not, read only by
    // rules of the table.
    let mut named = vec![false; n];
    for a in gp.named_atoms() {
        named[a as usize] = true;
    }
    // A row that changes keeps a head and loses one, each on a chain of
    // the table's own facts that excludes the other's: a table none of
    // whose rules reads one of its own facts has no such row. First the
    // tables none of whose rules reads a fact not named, before the
    // facts' readers are known.
    for r in &gp.rules {
        let e = &mut kinds[tables.of(r.head)];
        if may(e)
            && !e[3]
            && r.body
                .iter()
                .any(|l| gp.is_fact(l.atom()) && !named[l.atom() as usize])
        {
            e[3] = true;
        }
    }
    if !kinds.iter().any(|e| may(e) && e[3]) {
        return (0, Count::ONE);
    }
    const NONE: usize = usize::MAX;
    const MANY: usize = usize::MAX - 1;
    let mut reader = vec![NONE; n];
    for r in &gp.rules {
        let t = tables.of(r.head);
        for l in &r.body {
            let x = &mut reader[l.atom() as usize];
            *x = if *x == NONE || *x == t { t } else { MANY };
        }
    }
    let own = |a: AtomId, t: usize| gp.is_fact(a) && !named[a as usize] && reader[a as usize] == t;
    // A set read in binary.
    let wide = |l: &GLit| {
        tables.of(l.atom()) < num_sets && tables.sets()[tables.of(l.atom())].len() >= BITS_FROM
    };
    // Then the tables none of whose rules reads one of its own facts. One
    // whose heads not false are true changes only where its contexts
    // exclude each other and read no set in binary (below): each literal
    // of a context positive, of a set not read in binary.
    for r in &gp.rules {
        let t = tables.of(r.head);
        let e = &mut kinds[t];
        if !may(e) || !e[3] {
            continue;
        }
        for l in &r.body {
            if own(l.atom(), t) {
                e[4] = true;
            } else if !e[1] && (!l.is_positive() || tables.of(l.atom()) >= num_sets || wide(l)) {
                e[0] = false;
            }
        }
    }
    let live = |t: usize| may(&kinds[t]) && kinds[t][4];
    // The rows of the tables left, by table and context.
    struct Row {
        context: Vec<GLit>,
        rules: Vec<usize>,
    }
    let mut rows: Vec<Row> = Vec::new();
    let mut row_of: FxHashMap<(usize, Vec<GLit>), usize> = FxHashMap::default();
    let mut table_rows: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
    for (ri, r) in gp.rules.iter().enumerate() {
        let t = tables.of(r.head);
        if !live(t) {
            continue;
        }
        let mut context: Vec<GLit> = r
            .body
            .iter()
            .copied()
            .filter(|l| !own(l.atom(), t))
            .collect();
        context.sort_unstable();
        context.dedup();
        let next = rows.len();
        let id = *row_of.entry((t, context.clone())).or_insert(next);
        if id == next {
            rows.push(Row {
                context,
                rules: Vec::new(),
            });
            table_rows.entry(t).or_default().push(id);
        }
        rows[id].rules.push(ri);
    }
    // Whether a table's contexts exclude each other pairwise.
    let aligned = |ids: &[usize]| -> bool {
        let sets_of = |c: &[GLit]| -> Option<Vec<usize>> {
            let mut s: Vec<usize> = Vec::with_capacity(c.len());
            for l in c {
                let k = tables.of(l.atom());
                if !l.is_positive() || k >= num_sets {
                    return None;
                }
                s.push(k);
            }
            s.sort_unstable();
            (s.windows(2).all(|p| p[0] != p[1])).then_some(s)
        };
        match ids.first().map(|&i| sets_of(&rows[i].context)) {
            Some(Some(first)) if !first.is_empty() => ids[1..]
                .iter()
                .all(|&i| sets_of(&rows[i].context).as_ref() == Some(&first)),
            _ => false,
        }
    };
    // Each own fact's row, to tell the switches read by one row alone.
    let mut row_of_fact: FxHashMap<AtomId, usize> = FxHashMap::default();
    for (&t, ids) in &table_rows {
        for &id in ids {
            for &ri in &rows[id].rules {
                for l in &gp.rules[ri].body {
                    if own(l.atom(), t) {
                        let e = row_of_fact.entry(l.atom()).or_insert(id);
                        if *e != id {
                            *e = usize::MAX;
                        }
                    }
                }
            }
        }
    }
    let weight_of = |l: GLit| match gp.kinds[l.atom() as usize] {
        AtomKind::Fact(p) => p.literal(l.is_positive()),
        AtomKind::Derived => unreachable!("an own fact"),
    };
    // The rewrite of each row renormalized: its rules dropped, the new
    // rules emitted at its first rule.
    let mut new_atoms: Vec<(Atom, AtomKind)> = Vec::new();
    let mut emitted: FxHashMap<usize, Vec<GroundRule>> = FxHashMap::default();
    let mut dropped = vec![false; gp.rules.len()];
    let mut evidence: Vec<AtomId> = Vec::new();
    let mut factor = Count::ONE;
    let mut renormalized = 0;
    let mut order: Vec<usize> = table_rows.keys().copied().collect();
    order.sort_unstable();
    for t in order {
        let ids = &table_rows[&t];
        let shared = aligned(ids);
        // The masses first in each row's chain and on a false head of the
        // table where a context reads a set read in binary.
        let mass_first = ids.iter().any(|&i| rows[i].context.iter().any(wide));
        let mut switches: FxHashMap<((u64, u64), usize), AtomId> = FxHashMap::default();
        let mut masses: FxHashMap<(u64, u64), AtomId> = FxHashMap::default();
        let mut known: FxHashMap<(u64, u64), AtomId> = FxHashMap::default();
        let mut bad = AtomId::MAX;
        for &id in ids {
            let Row { context, rules } = &rows[id];
            // The chains, their weights and heads.
            let chain = |ri: usize| -> Vec<GLit> {
                let mut c: Vec<GLit> = gp.rules[ri]
                    .body
                    .iter()
                    .copied()
                    .filter(|l| own(l.atom(), t))
                    .collect();
                c.sort_unstable();
                c
            };
            let chains: Vec<Vec<GLit>> = rules.iter().map(|&ri| chain(ri)).collect();
            if !shared
                && chains
                    .iter()
                    .flatten()
                    .any(|l| row_of_fact.get(&l.atom()) != Some(&id))
            {
                continue;
            }
            let clash = |x: &[GLit], y: &[GLit]| x.iter().any(|&l| y.contains(&l.negated()));
            if (0..chains.len())
                .any(|i| (i + 1..chains.len()).any(|j| !clash(&chains[i], &chains[j])))
            {
                continue;
            }
            let mut heads: Vec<(AtomId, f64)> = Vec::new();
            for (k, &ri) in rules.iter().enumerate() {
                let theta = Prob::cube(chains[k].iter().map(|&l| weight_of(l))).p;
                let h = gp.rules[ri].head;
                match heads.iter_mut().find(|(x, _)| *x == h) {
                    Some((_, w)) => *w += theta,
                    None => heads.push((h, theta)),
                }
            }
            let total: f64 = heads.iter().map(|&(_, w)| w).sum();
            let lost: f64 = heads
                .iter()
                .filter(|&&(h, _)| false_head(h))
                .map(|&(_, w)| w)
                .sum();
            let kept: Vec<(AtomId, f64)> = heads
                .iter()
                .copied()
                .filter(|&(h, w)| !false_head(h) && w > 0.0)
                .collect();
            if total > 1.0 + 1e-9 || lost <= 1e-12 || kept.is_empty() {
                continue;
            }
            // The mass of no head, from the chains' weights, not as `1 -
            // total`, which keeps the rounding of the heads' masses.
            let Some(none) = no_head(&chains, &weight_of) else {
                continue;
            };
            let m: f64 = kept.iter().map(|&(_, w)| w).sum::<f64>() + none;
            if m >= 1.0 - 1e-12 || m <= 0.0 {
                continue;
            }
            // A row that keeps a head the conditioning made true: in a table
            // whose contexts exclude each other and read no set in binary,
            // `h :- context, t` with `t` of weight `θ_h`; elsewhere as it
            // is.
            if kept.iter().any(|&(h, _)| val[h as usize] == Val::True) {
                if !shared || mass_first || kept.len() != 1 {
                    continue;
                }
                renormalized += 1;
                let first = rules.iter().copied().min().expect("a row has rules");
                for &ri in rules {
                    dropped[ri] = true;
                }
                let (h, theta) = kept[0];
                // Its weight false is the mass of the row's other heads and
                // of none, each its own number.
                let theta = Prob {
                    p: theta,
                    not: lost + none,
                };
                let t = *known.entry(theta.bits()).or_insert_with(|| {
                    let t = (n + new_atoms.len()) as AtomId;
                    new_atoms.push((
                        Atom::new("$rn_t", vec![Term::Int(t as i64)]),
                        AtomKind::Fact(theta),
                    ));
                    t
                });
                let mut body = context.clone();
                body.push(GLit::Pos(t));
                emitted
                    .entry(first)
                    .or_default()
                    .push(GroundRule::new(h, body));
                continue;
            }
            renormalized += 1;
            let first = rules.iter().copied().min().expect("a row has rules");
            for &ri in rules {
                dropped[ri] = true;
            }
            // The mass on a fact `w` the context requires (none where the
            // row has no context: a constant). With `mass_first`, `w` heads
            // the kept heads' chain too, the row's own first switch.
            // Its weight false is the mass lost, its own number.
            let mw = Prob { p: m, not: lost };
            let mass = |new_atoms: &mut Vec<(Atom, AtomKind)>,
                        masses: &mut FxHashMap<(u64, u64), AtomId>|
             -> AtomId {
                match masses.get(&mw.bits()) {
                    Some(&w) if shared => w,
                    _ => {
                        let w = (n + new_atoms.len()) as AtomId;
                        new_atoms.push((
                            Atom::new("$rn_w", vec![Term::Int(w as i64)]),
                            AtomKind::Fact(mw),
                        ));
                        masses.insert(mw.bits(), w);
                        w
                    }
                }
            };
            let first_w =
                (mass_first && !context.is_empty()).then(|| mass(&mut new_atoms, &mut masses));
            let out = emitted.entry(first).or_default();
            let full = none == 0.0;
            let kept_masses: Vec<f64> = kept.iter().map(|&(_, w)| w).collect();
            let mut qs: Vec<Prob> = Vec::with_capacity(kept.len());
            chain_weights(&kept_masses, none, &mut qs);
            let mut negs: Vec<GLit> = Vec::new();
            let mut seen: FxHashMap<(u64, u64), usize> = FxHashMap::default();
            for (i, (&(h, _), &q)) in kept.iter().zip(&qs).enumerate() {
                let mut body = context.clone();
                if let Some(w) = first_w {
                    body.push(GLit::Pos(w));
                }
                body.extend_from_slice(&negs);
                if i + 1 == kept.len() && full {
                    out.push(GroundRule::new(h, body));
                    break;
                }
                let k = seen.entry(q.bits()).or_insert(0);
                let key = (q.bits(), *k);
                *k += 1;
                let fresh =
                    |kind: AtomKind, name: &str, atoms: &mut Vec<(Atom, AtomKind)>| -> AtomId {
                        let a = (n + atoms.len()) as AtomId;
                        atoms.push((Atom::new(name, vec![Term::Int(a as i64)]), kind));
                        a
                    };
                let s = match switches.get(&key) {
                    Some(&s) if shared => s,
                    _ => {
                        let s = fresh(AtomKind::Fact(q), "$rn_s", &mut new_atoms);
                        switches.insert(key, s);
                        s
                    }
                };
                body.push(GLit::Pos(s));
                out.push(GroundRule::new(h, body));
                negs.push(GLit::Neg(s));
            }
            if context.is_empty() {
                factor = (factor * Count::new(m)).normalized();
                continue;
            }
            let w = first_w.unwrap_or_else(|| mass(&mut new_atoms, &mut masses));
            if bad == AtomId::MAX {
                if mass_first {
                    // The row's mass lost goes to a head of the table the
                    // conditioning made false, one for the table: the
                    // row's false heads merged into one, first in its
                    // chain, in the table's own unit.
                    bad = heads
                        .iter()
                        .map(|&(h, _)| h)
                        .find(|&h| false_head(h))
                        .expect("a row that lost mass has a false head");
                    if !gp.evidence.iter().any(|&(_, a, v)| a == Some(bad) && !v) {
                        evidence.push(bad);
                    }
                } else {
                    bad = (n + new_atoms.len()) as AtomId;
                    new_atoms.push((
                        Atom::new("$rn_bad", vec![Term::Int(bad as i64)]),
                        AtomKind::Derived,
                    ));
                    evidence.push(bad);
                }
            }
            let mut body = context.clone();
            body.push(GLit::Neg(w));
            out.push(GroundRule::new(bad, body));
        }
    }
    if renormalized == 0 {
        return (0, Count::ONE);
    }
    for (atom, kind) in new_atoms {
        gp.atoms.push(atom);
        gp.kinds.push(kind);
    }
    let old = std::mem::take(&mut gp.rules);
    let mut rules = Vec::with_capacity(old.len());
    for (ri, r) in old.into_iter().enumerate() {
        if let Some(new) = emitted.remove(&ri) {
            rules.extend(new);
        }
        if !dropped[ri] {
            rules.push(r);
        }
    }
    gp.rules = rules;
    for bad in evidence {
        gp.evidence
            .push((gp.atoms[bad as usize].clone(), Some(bad), false));
    }
    (renormalized, factor)
}
