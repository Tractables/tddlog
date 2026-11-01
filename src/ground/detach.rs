//! Evidence that no query depends on.
//!
//! `P(q | e) = P(q ∧ e) / P(e)`. Where the observations fall into parts
//! over disjoint facts, the count of a part whose facts no query reads is
//! a factor of both counts and cancels, unless it is zero, and then the
//! evidence is inconsistent and nothing is answered. A world in which the
//! part holds shows that it is not zero, since a fact strictly between
//! zero and one gives every world a positive weight. [`drop_detached`]
//! drops each part it finds such a world for, and the relevance pass after
//! it the rules only those parts read; a part it finds none for within its
//! budget stays, and the compile counts it as before.
//!
//! The parts are the components of the relevant program's graph of rule
//! heads and body atoms. Every atom there is an ancestor of a query or an
//! observation, so an atom joining two components would make an
//! observation, or a query, read both: two components share no fact.
//!
//! The search is a DPLL over the completion, with
//! [`super::condition`]'s unit propagation and its trail. It takes the
//! part's derived atoms consumers first, and where one is known true with
//! no rule known to hold, or known false with a rule not known dead, it
//! tries the first open literal of the live rule with the fewest, to hold
//! or to fail, and on a contradiction the opposite. Once every atom known
//! is so settled, the facts known with the others false are evaluated
//! through the rules, and the world counts where the part's observations
//! hold in it.
//!
//! ace_mildew observes dm_4, the one parent of the udbytte it queries. Once
//! conditioned, udbytte's rules read its own switches alone, and the rest
//! of the network, 28 948 facts, is a part whose count cancels; likewise
//! ace_diabetes's 905 observations over 6 270 facts, against the 7 facts
//! its two queries read.

use std::sync::{Arc, OnceLock};

use super::deps::is_cyclic;
use super::propagation::Propagation;
use super::{AtomId, AtomKind, GroundProgram, RulesByHead};
use crate::disjoint::{find, union};

/// Body literals the search may visit per body literal of the program,
/// over all parts, before it leaves the rest of the parts to the compile.
const VISITS_PER_LITERAL: u64 = 64;

/// Drops the observations of each part of `gp`'s evidence that no query
/// shares a fact with, where a world in which the part holds is found, and
/// says how many it dropped. A program with a cyclic component is left as
/// it is, as is an observation of an atom that cannot hold.
pub(crate) fn drop_detached(gp: &mut GroundProgram) -> usize {
    if gp.evidence.is_empty() {
        return 0;
    }
    let n = gp.num_atoms();
    let mut parent: Vec<usize> = (0..n).collect();
    for r in &gp.rules {
        for l in &r.body {
            union(&mut parent, r.head as usize, l.atom() as usize);
        }
    }
    let mut queried = vec![false; n];
    for &(_, q, _) in &gp.queries {
        if let Some(q) = q {
            let root = find(&mut parent, q as usize);
            queried[root] = true;
        }
    }
    // The detached parts by root, each with its observations.
    let mut part_of = vec![usize::MAX; n];
    let mut parts: Vec<Vec<(AtomId, bool)>> = Vec::new();
    for &(_, a, v) in &gp.evidence {
        let Some(a) = a else { continue };
        let root = find(&mut parent, a as usize);
        if queried[root] {
            continue;
        }
        if part_of[root] == usize::MAX {
            part_of[root] = parts.len();
            parts.push(Vec::new());
        }
        parts[part_of[root]].push((a, v));
    }
    if parts.is_empty() {
        return 0;
    }
    // The structure the conditioning left, or the one the rewrites will
    // read where it left none.
    let structure = gp.structure();
    gp.structure = OnceLock::from(Arc::clone(&structure));
    let cyclic = structure
        .sccs
        .iter()
        .any(|c| is_cyclic(gp, &structure.by_head, c));
    if cyclic {
        return 0;
    }
    let (by_head, sccs) = (&structure.by_head, &structure.sccs);
    // Each part's facts, and its derived atoms consumers first.
    let mut facts: Vec<Vec<AtomId>> = vec![Vec::new(); parts.len()];
    let mut derived: Vec<Vec<AtomId>> = vec![Vec::new(); parts.len()];
    for a in 0..n {
        let p = part_of[find(&mut parent, a)];
        if p != usize::MAX && gp.is_fact(a as AtomId) {
            facts[p].push(a as AtomId);
        }
    }
    for &a in sccs.iter().rev().flatten() {
        let p = part_of[find(&mut parent, a as usize)];
        if p != usize::MAX {
            derived[p].push(a);
        }
    }
    let literals: u64 = gp.rules.iter().map(|r| r.body.len() as u64).sum();
    let budget = VISITS_PER_LITERAL * (literals + n as u64);
    // Each exclusive set read as at most one true atom: two values of a
    // network's variable each made to hold on one path were found apart
    // only where their switches met.
    let mut search = Propagation::with_sets(gp, &structure.sets, vec![false; structure.sets.len()]);
    let mut dropped = vec![false; n];
    for (p, observed) in parts.iter().enumerate() {
        if search.work > budget {
            break;
        }
        if holds_somewhere(
            &mut search,
            gp,
            by_head,
            &facts[p],
            &derived[p],
            observed,
            budget,
        ) {
            for &(a, _) in observed {
                dropped[a as usize] = true;
            }
        }
    }
    drop(search);
    let before = gp.evidence.len();
    gp.evidence
        .retain(|&(_, a, _)| a.is_none_or(|a| !dropped[a as usize]));
    before - gp.evidence.len()
}

/// Whether a world is found, within `budget` of the search's work, in
/// which the part whose facts and derived atoms (consumers first) these
/// are has the `observed` values. Leaves the search as it found it.
fn holds_somewhere(
    search: &mut Propagation,
    gp: &GroundProgram,
    by_head: &RulesByHead,
    facts: &[AtomId],
    derived: &[AtomId],
    observed: &[(AtomId, bool)],
    budget: u64,
) -> bool {
    let mark = search.mark();
    let seeded = (|| {
        for &f in facts {
            match gp.kinds[f as usize] {
                AtomKind::Fact(p) if p.not <= 0.0 => search.assign(f, true)?,
                AtomKind::Fact(p) if p.p <= 0.0 => search.assign(f, false)?,
                _ => {}
            }
        }
        for &(a, v) in observed {
            search.assign(a, v)?;
        }
        search.run()
    })();
    let found = seeded.is_ok()
        && decide(search, derived, budget)
        && satisfied(search, gp, by_head, facts, derived, observed);
    search.undo(mark);
    found
}

/// The DPLL: every atom of `derived` known settled, or `false` where the
/// decisions run out or the budget does. A decision is a literal, the
/// trail's length before it, whether its opposite is being tried, and how
/// far the scan of `derived` had come.
fn decide(search: &mut Propagation, derived: &[AtomId], budget: u64) -> bool {
    let mut decisions: Vec<(super::GLit, bool, usize, bool, usize)> = Vec::new();
    // An atom before `at` stays settled while no decision is taken back:
    // the propagation settles an atom it assigns from its rules' bodies,
    // and assigns from its value only the body atoms after it.
    let mut at = 0;
    loop {
        if search.work > budget {
            return false;
        }
        while at < derived.len() && search.justified(derived[at]) {
            at += 1;
        }
        let Some(&a) = derived.get(at) else {
            return true;
        };
        let (l, holds) = search.to_justify(a);
        decisions.push((l, holds, search.mark(), false, at));
        let mut failed = search
            .assign_literal(l, holds)
            .and_then(|()| search.run())
            .is_err();
        while failed {
            let Some((l, holds, mark, flipped, from)) = decisions.pop() else {
                return false;
            };
            search.undo(mark);
            if flipped {
                continue;
            }
            decisions.push((l, holds, mark, true, from));
            at = from;
            failed = search
                .assign_literal(l, !holds)
                .and_then(|()| search.run())
                .is_err();
        }
    }
}

/// Whether the part's observations hold in the world of the facts known,
/// the others false, evaluated through the rules of its derived atoms,
/// which come consumers first.
fn satisfied(
    search: &Propagation,
    gp: &GroundProgram,
    by_head: &RulesByHead,
    facts: &[AtomId],
    derived: &[AtomId],
    observed: &[(AtomId, bool)],
) -> bool {
    let mut world: rustc_hash::FxHashMap<AtomId, bool> = facts
        .iter()
        .map(|&f| (f, search.value(f) == Some(true)))
        .collect();
    for &a in derived.iter().rev() {
        let holds = by_head[a as usize].iter().any(|&ri| {
            gp.rules[ri]
                .body
                .iter()
                .all(|l| world.get(&l.atom()).copied().unwrap_or(false) == l.is_positive())
        });
        world.insert(a, holds);
    }
    observed.iter().all(|&(a, v)| world.get(&a) == Some(&v))
}
