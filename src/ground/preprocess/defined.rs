//! The facts of probability one half that the clauses over facts define,
//! eliminated by resolution: the part of Arjun's preprocessing (Soos and
//! Meel, ICCAD 2022) that weighted CNFs gain most from, run after the
//! conditioning on the evidence (`condition`) and before the vtree is
//! chosen, so that the compile and its vtree see fewer facts and clauses.
//!
//! It applies to a program that is a CNF over its facts
//! ([`crate::ground::shape::clauses_over_facts`]): every derived atom a
//! copy of a fact, `g :- f` or `g :- \+f`, or an atom observed false that
//! nothing reads, each of whose rules is the negation of a clause, as a
//! weighted CNF's program has them and the conditioning leaves them. Each
//! observation of a fact or a copy is a clause of one literal.
//!
//! An independent support of the clauses ([`super::support`]) is a set of
//! their facts of which every other fact is a function in their models. A
//! fact `y` outside it, of probability one half, is eliminated by
//! resolution (Davis and Putnam; as SatELite does, Eén and Biere, SAT 2005,
//! under another bound): the clauses that read `y` are replaced by their
//! resolvents on it, a CNF of `∃y F`. Each model of `∃y F` extends to one
//! model of `F` alone, whose `y` weighs one half, so that
//! `Σ_y ½ F(x, y) R(x) = ½ (∃y F)(x) R(x)` for whatever else `R` does not
//! read `y`: the weighted count of the program is half that of the program
//! with `y`'s clauses so replaced and `y` gone, and the half is a factor of
//! both `P(e)` and `P(q, e)` ([`crate::ground::GroundProgram::weight`]). The models
//! of `∃y F` are those of `F` with `y` forgotten, so that a fact that is a
//! function of the support in `F` still is in `∃y F`, and the eliminations
//! chain. For another probability `w(y)` depends on the value `y` takes,
//! and the fact is left alone.
//!
//! A query reads its fact, or the fact its copy reads, which is then not
//! eliminated: `R` above is the clauses and the query together. The stage
//! is sound for queries as for `P(e)`; the programs it meets are the
//! weighted CNFs, with no query (`--evidence-probability --no-queries`).
//!
//! A fact is eliminated where its clauses give no more than [`MOST_PAIRS`]
//! pairs to resolve and its resolvents, tautologies and clauses already in
//! the CNF left out, leave a minor of the primal graph ([`contracts`]),
//! cheapest first (the fewest pairs), in rounds until a round eliminates
//! none. The minor keeps the treewidth from growing, which a bound on the
//! count of clauses does not, and makes one unneeded. SatELite's bound, no
//! more resolvents than the clauses they replace, with 12 more allowed
//! (td2's choice for its search), took 59 facts and 30 clauses out of one,
//! `td`'s width from 19 to 37 and the compile from 4.3 M work units to past
//! 16 GB; with the minor, 14 facts go, the width is 15 and the compile
//! 0.4 M. With the minor as well, the bound of 12 took the same facts out
//! as no bound on 25 of 28 weighted CNFs, and on another took 491 out where
//! no bound takes 548: the compile ran out of memory, where without the
//! bound it takes 10.3 M work units. The clauses that stay keep their atoms
//! and rules; each resolvent is a new atom `$resolvent(k)` observed false,
//! whose one rule's body is the clause's negation, over the facts. The
//! atoms observed false whose clauses all went, the copies of facts
//! eliminated and the facts themselves are then read by nothing, and the
//! relevance restriction drops them.
//!
//! The conditioning's gates (`condition`) are the syntactic case: a fact
//! of probability one half that its clauses define as an AND or an OR,
//! read nowhere else. Its resolvents are tautologies, and its clauses go.
//!
//! The clauses that the others imply leave ([`super::implied`]) before the
//! search for the support, and the resolvents and clauses that the others
//! imply after the elimination: the models stay those of the CNF, and the
//! primal graph loses the cliques of long clauses. A clause dropped before
//! the search goes with its rule or observation, as an eliminated fact's
//! clauses go.

use rustc_hash::FxHashSet;

use super::implied::{ImpliedStats, implied_clauses};
use super::sat::Lit;
use super::support::{SupportStats, independent_support};
use crate::ground::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, Prob};
use crate::syntax::{Args, Atom, Term};

/// Pairs of clauses resolved on one fact past which it is not tried.
const MOST_PAIRS: usize = 10_000;

/// Names the atoms of the resolvents. Cannot clash with a user predicate,
/// whose name never contains `$`.
const RESOLVENT_NAME: &str = "$resolvent";

/// What the stage did, for `--stats`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Defined {
    /// The facts of probability one half that no query reads, which may
    /// leave the support.
    pub candidates: usize,
    /// The search for the support.
    pub support: SupportStats,
    /// The facts eliminated.
    pub eliminated: usize,
    /// The clauses before.
    pub clauses: usize,
    /// The clauses after.
    pub left: usize,
    /// The search for implied clauses before the support.
    pub implied: ImpliedStats,
    /// The search for implied clauses after the elimination.
    pub implied_after: ImpliedStats,
}

/// Where a clause of the CNF comes from.
#[derive(Clone, Copy, Debug)]
enum Origin {
    /// A rule of an atom observed false.
    Rule(usize),
    /// An observation of a fact or a copy.
    Observation(usize),
}

/// The CNF of a program over its facts ([`cnf`]).
struct Cnf {
    /// The fact of each variable, ascending.
    facts: Vec<AtomId>,
    /// Each clause's literals sorted, on distinct variables.
    clauses: Vec<Vec<Lit>>,
    origin: Vec<Origin>,
    /// The rules and observations whose clause always holds, and the
    /// observations that repeat one before them: they go with any fact
    /// eliminated, the repeats with the clause they repeat.
    redundant: Vec<Origin>,
}

/// Eliminates from `gp`, where it is a CNF over its facts, the facts of
/// probability one half outside an independent support of its clauses
/// where their resolvents are few, as described above: what the stage did,
/// or `None` where it does not apply. Where it eliminated a fact, the
/// caller drops the observations the rewrite settled and restricts the
/// program to what is relevant, and halves the program's factor once per
/// fact eliminated.
pub(in crate::ground) fn eliminate_defined(gp: &mut GroundProgram) -> Option<Defined> {
    if gp.evidence.is_empty()
        || !gp.kinds.contains(&AtomKind::Fact(Prob::HALF))
        || !crate::ground::shape::clauses_over_facts(gp)
    {
        return None;
    }
    let copy = gp.copies_of_facts();
    let cnf = cnf(gp, &copy)?;
    let n = cnf.facts.len();
    let mut asked = vec![false; gp.num_atoms()];
    for &(_, q, _) in &gp.queries {
        if let Some(q) = q {
            asked[q as usize] = true;
            if let Some(c) = copy[q as usize] {
                asked[c.atom() as usize] = true;
            }
        }
    }
    let candidate: Vec<bool> = cnf
        .facts
        .iter()
        .map(|&f| gp.kinds[f as usize] == AtomKind::Fact(Prob::HALF) && !asked[f as usize])
        .collect();
    let candidates = candidate.iter().filter(|&&c| c).count();
    if candidates == 0 {
        return None;
    }
    // The clauses the others imply leave, with their rules and
    // observations.
    let (before, implied) = implied_clauses(n, &cnf.clauses);
    let stay: Vec<usize> = (0..cnf.clauses.len()).filter(|&i| before[i]).collect();
    let clauses: Vec<Vec<Lit>> = stay.iter().map(|&i| cnf.clauses[i].clone()).collect();
    let (support, search) = independent_support(n, &clauses, &candidate);
    let mut stats = Defined {
        candidates,
        support: search,
        eliminated: 0,
        clauses: cnf.clauses.len(),
        left: clauses.len(),
        implied,
        implied_after: ImpliedStats::default(),
    };
    let outside: Vec<bool> = (0..n).map(|v| candidate[v] && !support[v]).collect();
    // Nothing to eliminate where every candidate is in the support or a
    // resolvent is empty: the clauses then have no model, which the
    // compile finds as it is.
    let out = if outside.contains(&true) {
        eliminate(n, &clauses, &outside)
    } else {
        None
    };
    let mut kept = before;
    let mut resolvents = Vec::new();
    if let Some(out) = out {
        stats.eliminated = out.eliminated.iter().filter(|&&e| e).count();
        for (j, &i) in stay.iter().enumerate() {
            kept[i] = out.kept[j];
        }
        resolvents = out.resolvents;
    }
    if stats.eliminated > 0 {
        // The resolvents and clauses that the others imply leave too.
        let stay: Vec<usize> = (0..cnf.clauses.len()).filter(|&i| kept[i]).collect();
        let after: Vec<Vec<Lit>> = stay
            .iter()
            .map(|&i| cnf.clauses[i].clone())
            .chain(resolvents.iter().cloned())
            .collect();
        let (keep, implied_after) = implied_clauses(n, &after);
        stats.implied_after = implied_after;
        for (j, &i) in stay.iter().enumerate() {
            kept[i] = keep[j];
        }
        let mut j = stay.len();
        resolvents.retain(|_| {
            j += 1;
            keep[j - 1]
        });
    }
    stats.left = kept.iter().filter(|&&k| k).count() + resolvents.len();
    if stats.eliminated == 0 && stats.implied.implied == 0 {
        return Some(stats);
    }
    let mut dead_rule = vec![false; gp.rules.len()];
    let mut dead_observation = vec![false; gp.evidence.len()];
    let dropped = cnf
        .origin
        .iter()
        .zip(&kept)
        .filter_map(|(&o, &kept)| (!kept).then_some(o));
    for o in dropped.chain(cnf.redundant.iter().copied()) {
        match o {
            Origin::Rule(ri) => dead_rule[ri] = true,
            Origin::Observation(k) => dead_observation[k] = true,
        }
    }
    let mut k = 0;
    gp.rules.retain(|_| {
        k += 1;
        !dead_rule[k - 1]
    });
    let mut k = 0;
    gp.evidence.retain(|_| {
        k += 1;
        !dead_observation[k - 1]
    });
    for (k, clause) in resolvents.iter().enumerate() {
        let h = gp.atoms.len() as AtomId;
        let atom = Atom::new(RESOLVENT_NAME, Args::One(Term::Int(k as i64)));
        gp.atoms.push(atom.clone());
        gp.kinds.push(AtomKind::Derived);
        // The clause fails where each of its literals does.
        let body = clause
            .iter()
            .map(|&l| GLit::new(cnf.facts[(l >> 1) as usize], l & 1 == 1))
            .collect();
        gp.rules.push(GroundRule::new(h, body));
        gp.evidence.push((atom, Some(h), false));
    }
    Some(stats)
}

/// The CNF of `gp`, a CNF over its facts whose copies are `copy`: a clause
/// per rule of an atom observed false and per observation of a fact or a
/// copy, over the facts they read, a copy read as its fact's literal;
/// `None` where an observation contradicts another or a clause is empty,
/// which the compile counts as it is, or where a rule stands for pairs.
fn cnf(gp: &GroundProgram, copy: &[Option<GLit>]) -> Option<Cnf> {
    let by_head = gp.rules_by_head();
    // A literal as a fact and whether the literal is that fact's positive
    // one.
    let fact_literal = |l: GLit| -> Option<(AtomId, bool)> {
        if gp.is_fact(l.atom()) {
            Some((l.atom(), l.is_positive()))
        } else {
            copy[l.atom() as usize].map(|c| (c.atom(), c.is_positive() == l.is_positive()))
        }
    };
    let mut clauses: Vec<(Vec<(AtomId, bool)>, Origin)> = Vec::new();
    let mut repeated = Vec::new();
    let mut observed: Vec<Option<bool>> = vec![None; gp.num_atoms()];
    for (k, &(_, a, value)) in gp.evidence.iter().enumerate() {
        let Some(a) = a else {
            // An impossible atom observed false holds; observed true, the
            // evidence cannot hold.
            if value {
                return None;
            }
            continue;
        };
        match observed[a as usize] {
            Some(v) if v != value => return None,
            Some(_) => {
                repeated.push(Origin::Observation(k));
                continue;
            }
            None => observed[a as usize] = Some(value),
        }
        if let Some(lit) = fact_literal(GLit::Pos(a)) {
            clauses.push((vec![(lit.0, lit.1 == value)], Origin::Observation(k)));
        } else if value {
            return None;
        } else {
            for &ri in &by_head[a as usize] {
                let r = &gp.rules[ri];
                if r.pairs {
                    return None;
                }
                let mut clause = Vec::with_capacity(r.body.len());
                for &l in &r.body {
                    let (f, positive) = fact_literal(l)?;
                    clause.push((f, !positive));
                }
                clauses.push((clause, Origin::Rule(ri)));
            }
        }
    }
    let mut var_of = vec![u32::MAX; gp.num_atoms()];
    for (c, _) in &clauses {
        for &(f, _) in c {
            var_of[f as usize] = 0;
        }
    }
    let mut facts = Vec::new();
    for (a, v) in var_of.iter_mut().enumerate() {
        if *v == 0 {
            *v = facts.len() as u32;
            facts.push(a as AtomId);
        }
    }
    let mut out = Cnf {
        facts,
        clauses: Vec::with_capacity(clauses.len()),
        origin: Vec::with_capacity(clauses.len()),
        redundant: repeated,
    };
    for (c, origin) in clauses {
        let mut lits: Vec<Lit> = c
            .iter()
            .map(|&(f, positive)| 2 * var_of[f as usize] + u32::from(!positive))
            .collect();
        lits.sort_unstable();
        lits.dedup();
        if lits.is_empty() {
            return None;
        }
        if lits.windows(2).any(|w| w[0] ^ 1 == w[1]) {
            out.redundant.push(origin);
        } else {
            out.clauses.push(lits);
            out.origin.push(origin);
        }
    }
    Some(out)
}

/// What [`eliminate`] left.
struct Eliminated {
    /// Per clause given, whether it stays.
    kept: Vec<bool>,
    /// The resolvents that stay.
    resolvents: Vec<Vec<Lit>>,
    /// Per variable, whether it was eliminated.
    eliminated: Vec<bool>,
}

/// The CNF of `n` variables and `clauses` with the variables that
/// `candidate` marks eliminated where their clauses give no more than
/// [`MOST_PAIRS`] pairs and their resolvents leave a minor of the primal
/// graph ([`contracts`]); `None` where a resolvent is empty and the CNF has
/// no model.
fn eliminate(n: usize, clauses: &[Vec<Lit>], candidate: &[bool]) -> Option<Eliminated> {
    let mut db: Vec<Vec<Lit>> = clauses.to_vec();
    let mut alive = vec![true; db.len()];
    let mut occurs: Vec<Vec<u32>> = vec![Vec::new(); 2 * n];
    for (i, c) in db.iter().enumerate() {
        for &l in c {
            occurs[l as usize].push(i as u32);
        }
    }
    let mut present: FxHashSet<Vec<Lit>> = db.iter().cloned().collect();
    let mut eliminated = vec![false; n];
    let mut resolvents: Vec<Vec<Lit>> = Vec::new();
    let mut fresh: FxHashSet<Vec<Lit>> = FxHashSet::default();
    let mut mark = vec![0u32; n];
    let mut stamp = 0u32;
    loop {
        let live = |occurs: &Vec<Vec<u32>>, alive: &[bool], l: usize| {
            occurs[l].iter().filter(|&&c| alive[c as usize]).count()
        };
        let mut order: Vec<(usize, usize)> = (0..n)
            .filter(|&y| candidate[y] && !eliminated[y])
            .map(|y| {
                (
                    live(&occurs, &alive, 2 * y) * live(&occurs, &alive, 2 * y + 1),
                    y,
                )
            })
            .collect();
        order.sort_unstable();
        let mut changed = false;
        for (_, y) in order {
            occurs[2 * y].retain(|&c| alive[c as usize]);
            occurs[2 * y + 1].retain(|&c| alive[c as usize]);
            let (pos, neg) = (&occurs[2 * y], &occurs[2 * y + 1]);
            if pos.len() * neg.len() > MOST_PAIRS {
                continue;
            }
            resolvents.clear();
            fresh.clear();
            for &p in pos {
                for &q in neg {
                    let Some(r) = resolve(&db[p as usize], &db[q as usize], 2 * y as Lit) else {
                        continue;
                    };
                    if !present.contains(&r) && fresh.insert(r.clone()) {
                        resolvents.push(r);
                    }
                }
            }
            if !contracts(&db, &alive, &occurs, &resolvents, &mut mark, &mut stamp) {
                continue;
            }
            for &c in pos.iter().chain(neg) {
                alive[c as usize] = false;
                present.remove(&db[c as usize]);
            }
            occurs[2 * y].clear();
            occurs[2 * y + 1].clear();
            for r in resolvents.drain(..) {
                if r.is_empty() {
                    return None;
                }
                let i = db.len() as u32;
                for &l in &r {
                    occurs[l as usize].push(i);
                }
                present.insert(r.clone());
                db.push(r);
                alive.push(true);
            }
            eliminated[y] = true;
            changed = true;
        }
        if !changed {
            break;
        }
    }
    let given = clauses.len();
    let kept = alive[..given].to_vec();
    let resolvents = db
        .into_iter()
        .zip(alive)
        .skip(given)
        .filter_map(|(c, a)| a.then_some(c))
        .collect();
    Some(Eliminated {
        kept,
        resolvents,
        eliminated,
    })
}

/// Whether the primal graph of the CNF `db` (the clauses `alive`, each
/// variable's by `occurs`) with a variable's clauses replaced by
/// `resolvents` is a minor of the graph as it is: each pair of variables a
/// resolvent reads together is adjacent already, or every pair that is not
/// has one variable `x` in common, a neighbour of the variable eliminated,
/// into which the elimination contracts it. The treewidth of a minor is at
/// most the graph's, so that the eliminations, each a minor of the graph
/// before it, never widen the decomposition `td` lays its vtree out on: a
/// fact defined as the AND of two others and read by one clause more goes,
/// as an equivalence does, where one read by two clauses that share no
/// variable with its inputs stays. `mark` is a stamp per variable.
fn contracts(
    db: &[Vec<Lit>],
    alive: &[bool],
    occurs: &[Vec<u32>],
    resolvents: &[Vec<Lit>],
    mark: &mut [u32],
    stamp: &mut u32,
) -> bool {
    // Each variable a resolvent reads, with the resolvent.
    let mut reads: Vec<(u32, usize)> = resolvents
        .iter()
        .enumerate()
        .flat_map(|(k, r)| r.iter().map(move |&l| (l >> 1, k)))
        .collect();
    reads.sort_unstable();
    // The variables every new edge so far reads; `u32::MAX` once one of
    // them is not.
    let mut ends: Option<[u32; 2]> = None;
    let mut i = 0;
    while i < reads.len() {
        let u = reads[i].0;
        *stamp += 1;
        for l in [2 * u, 2 * u + 1] {
            for &c in &occurs[l as usize] {
                if alive[c as usize] {
                    for &m in &db[c as usize] {
                        mark[(m >> 1) as usize] = *stamp;
                    }
                }
            }
        }
        while i < reads.len() && reads[i].0 == u {
            for &l in &resolvents[reads[i].1] {
                let v = l >> 1;
                if v <= u || mark[v as usize] == *stamp {
                    continue;
                }
                let e = ends.get_or_insert([u, v]);
                for z in e.iter_mut() {
                    if *z != u && *z != v {
                        *z = u32::MAX;
                    }
                }
                if *e == [u32::MAX; 2] {
                    return false;
                }
            }
            i += 1;
        }
    }
    true
}

/// The resolvent on `y` of `a`, which reads `y`, and `b`, which reads its
/// complement, both sorted: `None` where it is a tautology.
fn resolve(a: &[Lit], b: &[Lit], y: Lit) -> Option<Vec<Lit>> {
    let mut r = Vec::with_capacity(a.len() + b.len() - 2);
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        let l = match (a.get(i), b.get(j)) {
            (Some(&x), Some(&z)) if x == z => {
                i += 1;
                j += 1;
                x
            }
            (Some(&x), Some(&z)) if x < z => {
                i += 1;
                x
            }
            (Some(_), Some(&z)) => {
                j += 1;
                z
            }
            (Some(&x), None) => {
                i += 1;
                x
            }
            (None, Some(&z)) => {
                j += 1;
                z
            }
            (None, None) => unreachable!(),
        };
        if l >> 1 == y >> 1 {
            continue;
        }
        if r.last().is_some_and(|&last: &Lit| last ^ 1 == l) {
            return None;
        }
        r.push(l);
    }
    Some(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::preprocess::testing::{add_gates, draws, models, random_clauses};

    /// Random CNFs with gates: the models of what is left are those of the
    /// CNF with the variables eliminated forgotten, and the clauses kept
    /// are clauses given.
    #[test]
    fn elimination_forgets_the_variables_it_eliminates() {
        let mut next = draws(5);
        let mut eliminated_any = 0;
        for case in 0..300 {
            let n = 4 + case % 9;
            let mut clauses = random_clauses(&mut next, n, n / 3 + case % 3, 2);
            add_gates(&mut next, n, 1 + case % 4, &mut clauses);
            for c in &mut clauses {
                c.sort_unstable();
            }
            clauses.sort();
            clauses.dedup();
            let candidate: Vec<bool> = (0..n).map(|_| next(3) > 0).collect();
            let before = models(n, &clauses);
            match eliminate(n, &clauses, &candidate) {
                None => assert!(
                    before.is_empty(),
                    "case {case}: an empty resolvent of {clauses:?}"
                ),
                Some(out) => {
                    let eliminated = &out.eliminated;
                    assert!((0..n).all(|y| !eliminated[y] || candidate[y]));
                    let after: Vec<Vec<Lit>> = clauses
                        .iter()
                        .zip(&out.kept)
                        .filter(|&(_, &k)| k)
                        .map(|(c, _)| c.clone())
                        .chain(out.resolvents.iter().cloned())
                        .collect();
                    assert!(
                        after
                            .iter()
                            .flatten()
                            .all(|&l| !eliminated[(l >> 1) as usize]),
                        "case {case}: an eliminated variable is read"
                    );
                    let mask: u32 = (0..n).filter(|&y| !eliminated[y]).map(|y| 1 << y).sum();
                    let mut forgotten: Vec<u32> = before.iter().map(|&w| w & mask).collect();
                    forgotten.sort_unstable();
                    forgotten.dedup();
                    let mut kept: Vec<u32> = models(n, &after)
                        .into_iter()
                        .filter(|&w| w & !mask == 0)
                        .collect();
                    kept.sort_unstable();
                    assert_eq!(
                        forgotten, kept,
                        "case {case}: {clauses:?} eliminated {eliminated:?} to {after:?}"
                    );
                    eliminated_any += usize::from(eliminated.iter().any(|&e| e));
                }
            }
        }
        assert!(
            eliminated_any > 150,
            "eliminations met: {eliminated_any} of 300"
        );
    }

    /// The treewidth of the primal graph of `clauses` over `n` variables,
    /// by Bodlaender et al.'s dynamic program over the sets of variables
    /// eliminated first.
    fn treewidth(n: usize, clauses: &[Vec<Lit>]) -> u32 {
        let mut adjacent = vec![0u32; n];
        for c in clauses {
            for &l in c {
                for &m in c {
                    if l >> 1 != m >> 1 {
                        adjacent[(l >> 1) as usize] |= 1 << (m >> 1);
                    }
                }
            }
        }
        // The variables outside `s` and `v` that `v` reaches through `s`.
        let q = |s: u32, v: usize| {
            let (mut seen, mut stack, mut out) = (1u32 << v, vec![v], 0u32);
            while let Some(u) = stack.pop() {
                for w in 0..n {
                    if adjacent[u] >> w & 1 == 1 && seen >> w & 1 == 0 {
                        seen |= 1 << w;
                        if s >> w & 1 == 1 {
                            stack.push(w);
                        } else {
                            out |= 1 << w;
                        }
                    }
                }
            }
            out.count_ones()
        };
        let mut tw = vec![0u32; 1 << n];
        for s in 1u32..1 << n {
            tw[s as usize] = (0..n)
                .filter(|&v| s >> v & 1 == 1)
                .map(|v| tw[(s & !(1 << v)) as usize].max(q(s & !(1 << v), v)))
                .min()
                .unwrap();
        }
        tw[(1 << n) - 1]
    }

    /// Random CNFs with gates and random candidates: the primal graph
    /// of what is left is no wider than the CNF's; a gate read by one
    /// clause more goes, and one read by two that share no variable with
    /// its inputs stays.
    #[test]
    fn the_eliminations_leave_a_graph_no_wider() {
        let mut next = draws(17);
        let mut eliminated_any = 0;
        for case in 0..200 {
            let n = 5 + case % 7;
            let mut clauses = random_clauses(&mut next, n, n / 2 + case % 4, 2);
            for _ in 0..1 + case % 3 {
                let (y, a, b) = (
                    2 * next(n as u64) as u32,
                    2 * next(n as u64) as u32,
                    2 * next(n as u64) as u32,
                );
                if y != a && y != b && a != b {
                    clauses.extend([vec![y ^ 1, a], vec![y ^ 1, b], vec![y, a ^ 1, b ^ 1]]);
                }
            }
            for c in &mut clauses {
                c.sort_unstable();
            }
            clauses.sort();
            clauses.dedup();
            let candidate: Vec<bool> = (0..n).map(|_| next(3) == 0).collect();
            let Some(out) = eliminate(n, &clauses, &candidate) else {
                continue;
            };
            let after: Vec<Vec<Lit>> = clauses
                .iter()
                .zip(&out.kept)
                .filter(|&(_, &k)| k)
                .map(|(c, _)| c.clone())
                .chain(out.resolvents.iter().cloned())
                .collect();
            assert!(
                treewidth(n, &after) <= treewidth(n, &clauses),
                "case {case}: {clauses:?} to {after:?}"
            );
            eliminated_any += usize::from(out.eliminated.iter().any(|&e| e));
        }
        assert!(
            eliminated_any > 100,
            "eliminations met: {eliminated_any} of 200"
        );
        // 4 is the AND of 0 and 1, read by `5 or not 4`, and by `6 or not 4`
        // besides.
        let gate = vec![vec![1, 3, 8], vec![0, 9], vec![2, 9]];
        let one = [gate.clone(), vec![vec![9, 10]]].concat();
        let candidate = [false, false, false, false, true, false, false];
        assert!(eliminate(7, &one, &candidate).unwrap().eliminated[4]);
        let two = [gate, vec![vec![9, 10], vec![9, 12]]].concat();
        assert!(!eliminate(7, &two, &candidate).unwrap().eliminated[4]);
    }

    /// Random weighted CNFs with gates, the support found and the facts of
    /// one half outside it eliminated: the weighted count of what is left,
    /// halved once per fact eliminated, is the count of the CNF, by
    /// enumeration.
    #[test]
    fn the_support_and_the_elimination_keep_the_weighted_count() {
        let mut next = draws(23);
        let count = |n: usize, clauses: &[Vec<Lit>], p: &[f64], gone: &[bool]| -> f64 {
            models(n, clauses)
                .iter()
                .filter(|&&w| (0..n).all(|v| !gone[v] || w >> v & 1 == 0))
                .map(|&w| {
                    (0..n)
                        .filter(|&v| !gone[v])
                        .map(|v| if w >> v & 1 == 1 { p[v] } else { 1.0 - p[v] })
                        .product::<f64>()
                })
                .sum()
        };
        let mut eliminated_any = 0;
        for case in 0..300 {
            let n = 4 + case % 9;
            let p: Vec<f64> = (0..n)
                .map(|_| [0.5, 0.5, 0.3, 0.8][next(4) as usize])
                .collect();
            let mut clauses = random_clauses(&mut next, n, n / 3 + case % 3, 2);
            add_gates(&mut next, n, 1 + case % 4, &mut clauses);
            for c in &mut clauses {
                c.sort_unstable();
            }
            let candidate: Vec<bool> = p.iter().map(|&q| q == 0.5).collect();
            let (support, _) = independent_support(n, &clauses, &candidate);
            let outside: Vec<bool> = (0..n).map(|v| candidate[v] && !support[v]).collect();
            let want = count(n, &clauses, &p, &[false; 16][..n]);
            match eliminate(n, &clauses, &outside) {
                None => assert_eq!(want, 0.0, "case {case}"),
                Some(out) => {
                    let after: Vec<Vec<Lit>> = clauses
                        .iter()
                        .zip(&out.kept)
                        .filter(|&(_, &k)| k)
                        .map(|(c, _)| c.clone())
                        .chain(out.resolvents.iter().cloned())
                        .collect();
                    let k = out.eliminated.iter().filter(|&&e| e).count();
                    let got = count(n, &after, &p, &out.eliminated) / f64::from(1u32 << k);
                    assert!(
                        (got - want).abs() < 1e-12,
                        "case {case}: {got} against {want}: {clauses:?} to {after:?}"
                    );
                    eliminated_any += usize::from(k > 0);
                }
            }
        }
        assert!(
            eliminated_any > 100,
            "eliminations met: {eliminated_any} of 300"
        );
    }
}
