//! Conditioning the program on what its evidence implies.
//!
//! Unit propagation over the completion of the rules ([`Propagation`]),
//! seeded by the evidence and the certain atoms, with each exclusive set
//! ([`super::exclusive`]) read as at most one true atom and each
//! exhaustive one ([`super::cover`]) as exactly one: an atom whose rules
//! are all dead is false, one with a rule whose body holds is true, a true
//! atom with one live rule makes that body hold, and a false atom makes
//! false the last open literal of each of its rules. Every world's model
//! satisfies the completion, so what follows holds in every world where
//! the evidence does.
//!
//! - A fact it fixes is conditioned on: its weight is a factor of both
//!   `P(q, e)` and `P(e)`, and it becomes certain.
//! - A derived atom it fixes is observed, and every rule that reads it
//!   reads its value instead. Where no derived atom depends on itself, the
//!   completion has one model in each world, the program's; a world whose
//!   rewritten model satisfies the observations is then one whose model
//!   is the same, so the worlds counted and their queries do not change.
//!   An observation that the rewrite makes certain is dropped.
//! - A derived atom it makes false, with a rule whose body has two open
//!   literals, is a clause the evidence implies; where the literals are
//!   facts or atoms equal to one, the clauses' implications between facts'
//!   literals are a graph, and the facts of a strongly connected component
//!   are equal, or opposite, in every world counted. All but one become
//!   copies of it, `g :- f` or `g :- \+f`, and its weight is the product of
//!   theirs, rescaled: the worlds where they differ count for nothing, and
//!   the rest are rescaled together.
//! - The clauses such atoms leave over facts, of any length, are handed to
//!   a SAT solver first ([`super::preprocess::backbone`],
//!   [`Propagation::backbones`]):
//!   a fact that every model of the clauses fixes is fixed as the
//!   propagation's facts are, and the clauses of two literals the solver
//!   ends with, learnt or proved, join the graph of implications. The
//!   clauses hold in every world counted, so what they imply does too.
//! - A fact `g` of probability one half that such clauses, of any length,
//!   define as an OR of other facts' literals, `g ⇔ l_1 ∨ ... ∨ l_k` as the
//!   clauses `g ∨ \+l_i` and `\+g ∨ l_1 ∨ ... ∨ l_k`, or as an AND
//!   likewise, is a gate (a Tseitin encoding's gate variable). For whatever
//!   else `R` reads `g`, `Σ_g ½ [g = f(l)] R(g) = ½ R(f(l))`: the
//!   definition's clauses go, `g` becomes a derived atom with the gate's
//!   rules, `g :- l_i` for an OR and `g :- l_1, ..., l_k` for an AND, and
//!   the half is a factor of both counts. For another probability the
//!   factor depends on `f(l)`, and the fact is left alone. The definitions
//!   taken read one another without a cycle: the least model of
//!   `g1 :- g2. g2 :- g1.` is not the clauses' `g1 ⇔ g2`. A gate is taken
//!   only where nothing reads it but the definitions of the other gates
//!   taken: no query asks it, and no rule or clause outside those
//!   definitions reads it. It then goes in the relevance restriction, and
//!   with it the facts only it read. A gate that something else reads is
//!   left a fact with its clauses: as a derived atom it would make a
//!   weighted CNF a program that reads a derived atom, which `td` lays out
//!   right-linear instead of as its decomposition folded
//!   ([`crate::ground::shape::reads_facts_alone`]); taken, such gates cost
//!   the weighted CNFs that have them memory or work.
//! - A row of a table that the rewrite leaves some heads false and others
//!   not is renormalized over the heads it keeps ([`super::renormalize`]).
//!   Each world counted keeps its weight.
//!
//! The factors both counts lose, the fixed facts' weights, the masses the
//! rescalings take out and the gates' halves, are kept as their product
//! (the program's `weight`): a conditional probability does not read it,
//! the probability of the evidence does.
//!
//! These are what Ace's compiler gets from the evidence: it restricts each
//! table to the evidence before it compiles, and c2d's unit resolution
//! works through the tables' zeros. The friends and smokers networks of
//! Ace's benchmarks observe the hard clauses of a Markov logic network,
//! `friends(j, k)` implies `friends(i, k)` among them, which fix no fact
//! but make each column of `friends` one variable. The blockmap and
//! mastermind networks observe deterministic nodes, and the evidence fixes
//! almost every variable. The gates are the syntactic case of the definable
//! variables that model counters' preprocessors eliminate (B+E, Lagniez,
//! Lonca and Marquis, IJCAI 2016), which a weighted count may eliminate
//! only where the variable weighs the same both ways: the weighted CNFs
//! write half their variables as such gates.

use super::cover::Cover;
use super::preprocess::backbone;
use super::propagation::{Contradiction, Gate, Propagation, Val};
use super::renormalize::renormalize;
use super::tables::Tables;
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, Prob};
use crate::eval::Count;

/// What the evidence fixed and merged (`on_evidence`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Conditioned {
    /// Facts fixed.
    pub facts: usize,
    /// Derived atoms fixed and observed.
    pub atoms: usize,
    /// Facts made copies of another.
    pub merged: usize,
    /// The facts they copy.
    pub into: usize,
    /// Facts of probability one half made the AND or OR gates that the
    /// clauses define them as, which nothing else reads: the relevance
    /// restriction drops them with their definitions (`ground::finish`).
    pub gates: usize,
    /// The search for the facts every model of the clauses fixes, where it
    /// ran.
    pub backbones: Option<Backbones>,
    /// Rows of tables renormalized over the heads the evidence leaves them
    /// (`renormalize`).
    pub renormalized: usize,
}

impl Conditioned {
    /// Whether the conditioning fixed, merged or made gates of anything.
    pub fn any(&self) -> bool {
        self.facts + self.atoms + self.merged + self.into + self.gates + self.renormalized > 0
    }
}

/// What the search for the facts that every model of the clauses fixes did
/// (`Propagation::backbones`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Backbones {
    /// The solver's calls, conflicts and propagation steps, the backbone
    /// it proved and the candidates it left open.
    pub search: backbone::Search,
    /// The facts fixed beyond the propagation's, those the backbone's
    /// propagation fixed with it among them.
    pub fixed: usize,
}

/// Conditions `gp` on what its evidence implies, as described above, given
/// its tables and its cover. Leaves
/// the certain atoms to [`crate::ground::simplify::propagate_certain`], the
/// observations made certain to [`drop_settled_evidence`], and the gates
/// nothing reads to [`crate::ground::simplify::restrict_to_relevant`]; all three
/// are needed after it where anything was conditioned. A program whose
/// evidence cannot hold, or where a derived atom depends on itself, is left
/// as it is. With `renormalize_rows`, the rows of tables that lose heads
/// are renormalized over the heads they keep ([`renormalize`]). Returns
/// what it conditioned and the factor of `P(e)` it took out ([`rewrite`]).
pub(crate) fn on_evidence(
    gp: &mut GroundProgram,
    tables: &Tables,
    cover: &Cover,
    backbones: bool,
    renormalize_rows: bool,
) -> (Conditioned, Count) {
    if !observed(gp) || !tables.acyclic(gp) {
        return (Conditioned::default(), Count::ONE);
    }
    let (val, equal, gates, searched) = {
        let mut p = Propagation::new(gp, tables, cover);
        let seeded = (|| {
            p.propagate_evidence(&cover.certain)?;
            let mut clauses = p.clauses();
            let (mut implied, mut searched) = (Vec::new(), None);
            if backbones && !clauses.ends.is_empty() {
                // The facts of the gates the clauses define and nothing
                // else reads, which the rewrite sums out with their
                // definitions, are not fixed. Such a fact is a backbone
                // where its definition is a constant over the other
                // clauses' models. Fixed, the definition becomes a clause
                // over the gate's inputs that the others imply, and an
                // input that is a gate itself is then read outside the
                // definitions and stays a fact (on a weighted CNF, fixing
                // them sums out far fewer gates and multiplies td's work).
                let equal = p.equal_facts(&clauses, &[])?;
                let mut summed = vec![false; gp.num_atoms()];
                for g in p.unread(p.gates(&clauses, &equal), &gp.queries) {
                    summed[g.fact as usize] = true;
                }
                let (pairs, did) = p.backbones(&clauses, &equal, &summed)?;
                if did.fixed > 0 {
                    clauses = p.clauses();
                }
                (implied, searched) = (pairs, Some(did));
            }
            let equal = p.equal_facts(&clauses, &implied)?;
            let gates = p.unread(p.gates(&clauses, &equal), &gp.queries);
            Ok((equal, gates, searched))
        })();
        match seeded {
            Ok((equal, gates, searched)) => (p.val, equal, gates, searched),
            Err(Contradiction) => return (Conditioned::default(), Count::ONE),
        }
    };
    let (mut out, mut weight) = rewrite(gp, &val, &cover.certain, &equal, &gates, true);
    out.backbones = searched;
    if renormalize_rows {
        let (rows, factor) = renormalize(gp, tables, &val);
        out.renormalized = rows;
        weight = (weight * factor).normalized();
    }
    (out, weight)
}

/// Conditions `gp` as [`on_evidence`] does where the propagation without
/// the exclusive sets and the cover settles it: every derived atom fixed,
/// a true one with a rule whose body holds and a false one with every
/// rule dead, and no derived atom depending on itself. The conditioned
/// program is then its queries' atoms, certain or false, and the facts
/// it no longer reads; what the sets add to the propagation is nothing
/// a settled atom needs, and what the conditioning fixes, a fact being
/// fixed only where the evidence leaves it one value, is the same.
/// Returns what it conditioned, or `None`, leaving `gp` as it is, where
/// the propagation leaves an atom open or unjustified, the evidence cannot
/// hold, or nothing is conditioned. The UAI 2022 blockmaps are networks
/// of deterministic tables but for a few dozen rows, whose evidence
/// settles nearly all of them thus (`ground::finish`).
pub(crate) fn settle(gp: &mut GroundProgram) -> Option<(Conditioned, Count)> {
    if !observed(gp) {
        return None;
    }
    let none = vec![false; gp.num_atoms()];
    let val = {
        let mut p = Propagation::with_sets(gp, &[], Vec::new());
        p.propagate_evidence(&none).ok()?;
        if !(0..gp.num_atoms()).all(|a| {
            gp.kinds[a] != AtomKind::Derived
                || (p.val[a] != Val::Unknown && p.justified(a as AtomId))
        }) {
            return None;
        }
        // Where every rule reads atoms numbered before its head, as the
        // grounder numbers an acyclic program it evaluates in order (the
        // table route's evaluation order, [`crate::ground::table`]), no atom
        // depends on itself, and the components are not needed to tell.
        let ordered = gp
            .rules
            .iter()
            .all(|r| r.body.iter().all(|l| l.atom() < r.head));
        if !ordered
            && crate::ground::deps::derived_sccs(gp, &p.by_head)
                .iter()
                .any(|c| crate::ground::deps::is_cyclic(gp, &p.by_head, c))
        {
            return None;
        }
        p.val
    };
    // Each derived atom fixed here is certain once the facts fixed are
    // propagated, a true one by its rule that holds and a false one by
    // its rules all dead, and [`drop_settled_evidence`] would drop its
    // observation: none is made.
    let (conditioned, weight) = rewrite(gp, &val, &none, &[], &[], false);
    conditioned.any().then_some((conditioned, weight))
}

/// Whether `gp` has evidence, none of it on an atom the grounding found
/// impossible observed true: evidence the conditioning can read.
fn observed(gp: &GroundProgram) -> bool {
    !gp.evidence.is_empty() && !gp.evidence.iter().any(|&(_, a, v)| a.is_none() && v)
}

/// `gp` rewritten on the values `val` the propagation found, `certain` the
/// atoms it was given as certain, with the facts `equal` merged and the
/// facts of `gates` made the gates' atoms ([`on_evidence`]); with
/// `observe`, each derived atom it fixed that the program did not know
/// observed. Returns what it conditioned and the factor of `P(e)` the
/// program no longer counts: the weights of the facts fixed, for each fact
/// the copies of which it took the weights, the mass of the worlds where
/// they agree, and a half for each gate. One where nothing was
/// conditioned.
fn rewrite(
    gp: &mut GroundProgram,
    val: &[Val],
    certain: &[bool],
    equal: &[(AtomId, AtomId, bool)],
    gates: &[Gate],
    observe: bool,
) -> (Conditioned, Count) {
    // What the program knew before: certain atoms, atoms without rules and
    // observations.
    let mut known = vec![false; gp.num_atoms()];
    let mut ruled = vec![false; gp.num_atoms()];
    for r in &gp.rules {
        ruled[r.head as usize] = true;
        known[r.head as usize] |= r.body.is_empty();
    }
    for &(_, a, _) in &gp.evidence {
        if let Some(a) = a {
            known[a as usize] = true;
        }
    }
    for a in 0..gp.num_atoms() {
        known[a] |= certain[a] || (gp.kinds[a] == AtomKind::Derived && !ruled[a]);
    }
    let (mut out, mut factor) = (Conditioned::default(), Count::ONE);
    for a in 0..gp.num_atoms() {
        if val[a] == Val::Unknown || known[a] {
            continue;
        }
        match gp.kinds[a] {
            AtomKind::Fact(_) => out.facts += 1,
            AtomKind::Derived => out.atoms += 1,
        }
    }
    out.gates = gates.len();
    if !out.any() && equal.is_empty() {
        return (Conditioned::default(), Count::ONE);
    }
    // Fixed facts become certain, their weights a factor of `P(e)`.
    for (kind, &v) in gp.kinds.iter_mut().zip(val) {
        if let (AtomKind::Fact(p), Val::True | Val::False) = (*kind, v) {
            factor = (factor * Count::new(p.of(v == Val::True))).normalized();
            *kind = AtomKind::Fact(if v == Val::True {
                Prob::TRUE
            } else {
                Prob::FALSE
            });
        }
    }
    // Fixed derived atoms are read as their values.
    let fixed = |l: GLit| match (gp.kinds[l.atom() as usize], val[l.atom() as usize]) {
        (AtomKind::Derived, Val::True) => Some(l.is_positive()),
        (AtomKind::Derived, Val::False) => Some(!l.is_positive()),
        _ => None,
    };
    // The clauses that define the gates go.
    let mut defining = vec![false; gp.rules.len()];
    for gate in gates {
        for &ri in &gate.clauses {
            defining[ri as usize] = true;
        }
    }
    let mut rules = Vec::with_capacity(gp.rules.len());
    for (ri, mut r) in std::mem::take(&mut gp.rules).into_iter().enumerate() {
        if defining[ri] || r.body.iter().any(|&l| fixed(l) == Some(false)) {
            continue;
        }
        // The body kept in its own allocation.
        r.body.retain(|&l| fixed(l).is_none());
        rules.push(GroundRule::new(r.head, r.body));
    }
    gp.rules = rules;
    if observe {
        for a in 0..gp.num_atoms() {
            if gp.kinds[a] == AtomKind::Derived && val[a] != Val::Unknown && !known[a] {
                gp.evidence
                    .push((gp.atoms[a].clone(), Some(a as AtomId), val[a] == Val::True));
            }
        }
    }
    // Equal facts: each copy's weight goes into the fact it copies, and the
    // mass of the worlds where the two agree, which the rescaling takes
    // out, into the program's factor.
    let mut weight: Vec<Option<Prob>> = vec![None; gp.num_atoms()];
    for &(g, f, same) in equal {
        let p = |a: AtomId| match gp.kinds[a as usize] {
            AtomKind::Fact(p) => p,
            AtomKind::Derived => unreachable!("an open fact"),
        };
        let w = weight[f as usize].unwrap_or(p(f));
        let wg = p(g).literal(same);
        let (t, e) = (w.p * wg.p, w.not * wg.not);
        factor = (factor * Count::new(t + e)).normalized();
        weight[f as usize] = Some(Prob {
            p: t / (t + e),
            not: e / (t + e),
        });
    }
    for &(g, f, same) in equal {
        gp.kinds[g as usize] = AtomKind::Derived;
        gp.rules.push(GroundRule::new(g, vec![GLit::new(f, same)]));
    }
    for (f, w) in weight.iter().enumerate() {
        if let Some(w) = w {
            gp.kinds[f] = AtomKind::Fact(*w);
            out.into += 1;
        }
    }
    out.merged = equal.len();
    // Each gate's fact becomes a derived atom with the gate's rules, and
    // the weight one half of each of its values, which its definition
    // summed out, a factor of the counts.
    for gate in gates {
        gp.kinds[gate.fact as usize] = AtomKind::Derived;
        gp.rules.extend(gate.rules());
    }
    (out, factor.shifted(-(gates.len() as i64)))
}

/// Drops the observations of certain atoms that hold: a true atom with a
/// rule whose body is empty, a false derived atom with no rules.
pub(crate) fn drop_settled_evidence(gp: &mut GroundProgram) {
    let mut certain = vec![false; gp.num_atoms()];
    let mut ruled = vec![false; gp.num_atoms()];
    for r in &gp.rules {
        ruled[r.head as usize] = true;
        if r.body.is_empty() {
            certain[r.head as usize] = true;
        }
    }
    let kinds = &gp.kinds;
    gp.evidence.retain(|&(_, a, v)| match a {
        Some(a) if v => !certain[a as usize],
        Some(a) => ruled[a as usize] || kinds[a as usize] != AtomKind::Derived,
        None => true,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::program::test_program;
    use GLit::{Neg, Pos};

    #[test]
    fn settles_what_the_propagation_alone_decides() {
        // Fact 0 of 0.3, 1 of 0.6; a :- f0. b :- \+f0. c :- a, f1. c :- b.
        // Observing c and a fixes f0 and then f1 through c's one live rule,
        // and every derived atom: P(e) = 0.3 · 0.6.
        let rules: &[(AtomId, &[GLit])] = &[
            (2, &[Pos(0)]),
            (3, &[Neg(0)]),
            (4, &[Pos(2), Pos(1)]),
            (4, &[Pos(3)]),
        ];
        let mut gp = test_program(&[0.3, 0.6], 3, rules, &[(4, true), (2, true)]);
        let (conditioned, weight) = settle(&mut gp).expect("settled");
        assert_eq!((conditioned.facts, conditioned.atoms), (2, 1));
        assert!((weight.to_f64() - 0.18).abs() < 1e-12);
        assert_eq!(
            gp.kinds[..2],
            [
                AtomKind::Fact(Prob::new(1.0)),
                AtomKind::Fact(Prob::new(1.0))
            ]
        );
        // The fixed facts propagated, every derived atom is certain, and
        // its observation dropped, as one of b (false) would be.
        crate::ground::simplify::propagate_certain(&mut gp);
        drop_settled_evidence(&mut gp);
        assert!(gp.evidence.is_empty());
        assert!(!gp.rules.iter().any(|r| r.head == 3));
        // Observing c alone leaves f0 open, and with it a and b.
        let mut gp = test_program(&[0.3, 0.6], 3, rules, &[(4, true)]);
        let before = gp.clone();
        assert!(settle(&mut gp).is_none());
        assert_eq!(gp.rules, before.rules);
        // A derived atom that depends on itself: p :- p, observed, is
        // fixed and its rule holds, but its least model is false.
        let mut gp = test_program(&[], 1, &[(0, &[Pos(0)])], &[(0, true)]);
        assert!(settle(&mut gp).is_none());
        // A rule that reads an atom numbered after its head leaves the
        // components to tell: a :- b. b :- f0, with a observed, is
        // acyclic and settled; a :- b. b :- a, both observed, is a cycle.
        let mut gp = test_program(&[0.3], 2, &[(1, &[Pos(2)]), (2, &[Pos(0)])], &[(1, true)]);
        let (conditioned, weight) = settle(&mut gp).expect("settled");
        assert_eq!((conditioned.facts, conditioned.atoms), (1, 1));
        assert!((weight.to_f64() - 0.3).abs() < 1e-12);
        let mut gp = test_program(
            &[],
            2,
            &[(0, &[Pos(1)]), (1, &[Pos(0)])],
            &[(0, true), (1, true)],
        );
        assert!(settle(&mut gp).is_none());
    }
}
