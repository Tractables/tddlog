//! The clauses of a CNF that the others imply, left out: the clause store
//! is smaller and its primal graph sparser, with the same models, so that
//! a count over it is the count over the CNF whatever the weights. As
//! Arjun's sparsification does with its oracle (Soos and Meel, after
//! Korhonen and Järvisalo, CP 2021), for the elimination of defined facts
//! ([`super::defined`]), before the search for a support and again on the
//! resolvents after it.
//!
//! A clause `C` is implied by the others `F \ C` exactly where
//! `F \ C ∧ ¬C` has no model. The clauses of at least [`SHORTEST`]
//! literals are tried, the longest first: a long clause makes a clique of
//! the primal graph, and the weaker it is the likelier something else
//! implies it. A clause found implied leaves before the next is tried,
//! which is then asked against the clauses that stay: each clause that
//! leaves is implied by those that stay after it, by induction from the
//! last to leave, and the models do not change.
//!
//! A clause is tried only where it joins two variables that no clause
//! known to stay joins: those too short to be tried, and those the search
//! found not implied. A clause whose pairs such clauses all join leaves
//! the primal graph as it is, and it stays, since an implied clause can
//! help the compile, which conjoins it with the others below the root
//! (on one instance dropping 2 046 implied clauses whatever they
//! join took `td`'s width from 25 to 23, and the evidence walk from
//! 51 665 pairs to more than the memory). The pairs of a clause of more
//! than [`PAIRED`] literals are not counted, and such a clause is always
//! tried.
//!
//! The clauses are tried by unit propagation first, as vivification
//! tries them (Piette, Hamadi and Saïs, ECAI 2008): `C` leaves where
//! propagating the negation of its literals over the others meets a
//! conflict. Those it keeps are then searched, the longest first, within
//! [`CONFLICTS_PER_CALL`]: a search finds more, but a clause it keeps costs
//! it a model of the whole CNF. The clauses tried go in windows of
//! [`WINDOW`], one solver each: the clauses outside the window that stay
//! go in as they are, and each clause in the window with a guard `g_i`,
//! `C_i ∨ ¬g_i`. A test assumes the guards of the clauses of the window not
//! yet tried, in the reverse of the order they are tried in, then those of
//! the clauses kept, then the negation of `C`: a test shares all but the
//! last few of its assumptions with the one before and keeps their levels
//! ([`super::sat::Solver::keep_assumptions`]), and no test assumes more
//! than a window of guards. A search that stops at its conflict limit
//! keeps its clause, as does every clause not yet searched once the
//! propagation steps pass the budget.
//!
//! On weighted CNFs (after the conditioning), three clauses of 36 literals
//! that the others imply take the width of `td` on one from 41 to 23, and
//! on another from 40 to 16; on a third the 1 458 implied clauses of four
//! and five literals, and 2 245 resolvents after the elimination, take it
//! from 34 to 25.

use rustc_hash::FxHashSet;

use super::sat::{Lit, Outcome, Solver};

/// The fewest literals of a clause tried.
pub(in crate::ground) const SHORTEST: usize = 4;
/// The most literals of a clause whose pairs of variables are counted.
const PAIRED: usize = 64;
/// The clauses tried per solver.
const WINDOW: usize = 512;
/// The propagation steps per literal of the CNF, over the whole search,
/// after which the clauses not yet tried stay; the clauses loaded into
/// each window's solver count a step per literal.
const TICKS_PER_LITERAL: u64 = 2000;
/// Propagation steps the search may take however small the CNF.
const TICKS_FLOOR: u64 = 20_000_000;

/// The conflicts after which a search gives up and keeps its clause.
const CONFLICTS_PER_CALL: u64 = 100;

/// What the search did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImpliedStats {
    /// The clauses tried by propagation, those of them searched, those
    /// found implied either way, and those not searched for the budget;
    /// the clauses passed over because clauses known to stay join each
    /// pair of their variables.
    pub tried: u64,
    pub searched: u64,
    pub implied: u64,
    pub untried: u64,
    pub joined: u64,
    /// The searches' conflicts; the solvers' propagation steps, and the
    /// literals loaded into them.
    pub conflicts: u64,
    pub ticks: u64,
}

/// Which of the `clauses` of the CNF of `n` variables stay, each clause's
/// literals on distinct variables: every clause of at least [`SHORTEST`]
/// literals that the others that stay imply leaves.
pub(in crate::ground) fn implied_clauses(
    n: usize,
    clauses: &[Vec<Lit>],
) -> (Vec<bool>, ImpliedStats) {
    let mut stats = ImpliedStats::default();
    let mut keep = vec![true; clauses.len()];
    let mut tried: Vec<usize> = (0..clauses.len())
        .filter(|&i| clauses[i].len() >= SHORTEST)
        .collect();
    if tried.is_empty() {
        return (keep, stats);
    }
    tried.sort_by_key(|&i| (std::cmp::Reverse(clauses[i].len()), i));
    let literals: u64 = clauses.iter().map(|c| c.len() as u64).sum();
    let budget = TICKS_FLOOR + TICKS_PER_LITERAL * literals;
    // The clauses that stay whatever is found after, and the pairs of
    // variables those of at most `PAIRED` literals join: the clauses too
    // short to be tried, then each found not implied by the search.
    let key = |a: Lit, b: Lit| {
        let (a, b) = ((a >> 1) as u64, (b >> 1) as u64);
        a.min(b) << 32 | a.max(b)
    };
    let mut settled = vec![false; clauses.len()];
    let mut joins: FxHashSet<u64> = FxHashSet::default();
    let settle =
        |i: usize, settled: &mut Vec<bool>, joins: &mut FxHashSet<u64>, ticks: &mut u64| {
            settled[i] = true;
            let c = &clauses[i];
            if c.len() <= PAIRED {
                *ticks += (c.len() * (c.len() - 1) / 2) as u64;
                for (k, &a) in c.iter().enumerate() {
                    for &b in &c[k + 1..] {
                        joins.insert(key(a, b));
                    }
                }
            }
        };
    for i in (0..clauses.len()).filter(|&i| clauses[i].len() < SHORTEST) {
        settle(i, &mut settled, &mut joins, &mut stats.ticks);
    }
    // Whether clauses that stay join every pair of the variables of `c`.
    let joined = |joins: &FxHashSet<u64>, c: &[Lit]| {
        c.len() <= PAIRED
            && c.iter()
                .enumerate()
                .all(|(k, &a)| c[k + 1..].iter().all(|&b| joins.contains(&key(a, b))))
    };
    let mut in_window = vec![u32::MAX; clauses.len()];
    let mut assumptions: Vec<Lit> = Vec::new();
    let mut kept: Vec<Lit> = Vec::new();
    for search in [false, true] {
        let left: Vec<usize> = tried
            .iter()
            .copied()
            .filter(|&i| keep[i] && !settled[i])
            .collect();
        for (w, window) in left.chunks(WINDOW).enumerate() {
            if stats.ticks >= budget {
                stats.untried = (left.len() - w * WINDOW) as u64;
                break;
            }
            for (k, &i) in window.iter().enumerate() {
                in_window[i] = k as u32;
            }
            let guard = |k: usize| 2 * (n + k) as Lit;
            let mut s = Solver::new(n + window.len());
            s.keep_assumptions = true;
            let mut ok = true;
            for (i, c) in clauses.iter().enumerate() {
                if !keep[i] {
                    continue;
                }
                stats.ticks += c.len() as u64;
                ok &= if in_window[i] == u32::MAX {
                    s.add_clause(c)
                } else {
                    let mut g = c.clone();
                    g.push(guard(in_window[i] as usize) ^ 1);
                    s.add_clause(&g)
                };
            }
            if !ok {
                // No model: every clause is implied, and the compile finds
                // the empty set of models as it is.
                return (vec![true; clauses.len()], ImpliedStats::default());
            }
            s.tick_limit = budget.saturating_sub(stats.ticks);
            kept.clear();
            for (k, &i) in window.iter().enumerate() {
                if s.ticks >= s.tick_limit {
                    stats.untried = (left.len() - w * WINDOW - k) as u64;
                    break;
                }
                if joined(&joins, &clauses[i]) {
                    stats.joined += 1;
                    settle(i, &mut settled, &mut joins, &mut stats.ticks);
                    kept.push(guard(k));
                    continue;
                }
                assumptions.clear();
                assumptions.extend((k + 1..window.len()).rev().map(guard));
                assumptions.extend_from_slice(&kept);
                assumptions.extend(clauses[i].iter().map(|&l| l ^ 1));
                let implied = if search {
                    stats.searched += 1;
                    s.solve(&assumptions, CONFLICTS_PER_CALL) == Outcome::Unsat
                } else {
                    stats.tried += 1;
                    s.propagation_refutes(&assumptions)
                };
                if implied {
                    keep[i] = false;
                    stats.implied += 1;
                } else {
                    kept.push(guard(k));
                    if search {
                        settle(i, &mut settled, &mut joins, &mut stats.ticks);
                    }
                }
            }
            stats.conflicts += s.conflicts;
            stats.ticks += s.ticks;
            for &i in window {
                in_window[i] = u32::MAX;
            }
            if stats.untried > 0 {
                break;
            }
        }
    }
    (keep, stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::preprocess::testing::{draws, models, random_clauses};

    /// Random CNFs with clauses that the others imply (resolvents and
    /// clauses that others subsume, and long clauses at random): the
    /// clauses that stay have the models of the CNF, by enumeration, and
    /// the clauses that leave are among those tried.
    #[test]
    fn the_clauses_that_stay_have_the_same_models() {
        let mut next = draws(5);
        let mut left = 0;
        for case in 0..300 {
            let n = 5 + case % 8;
            let mut clauses = random_clauses(&mut next, n, n + case % 5, 4);
            // A clause some other subsumes, and a resolvent of two.
            for _ in 0..1 + case % 3 {
                let a = clauses[next(clauses.len() as u64) as usize].clone();
                let mut wider = a.clone();
                for _ in 0..2 {
                    let v = next(n as u64) as u32;
                    if !wider.iter().any(|&l| l >> 1 == v) {
                        wider.push(2 * v + next(2) as u32);
                    }
                }
                wider.sort_unstable();
                clauses.push(wider);
                let (p, q) = (
                    next(clauses.len() as u64) as usize,
                    next(clauses.len() as u64) as usize,
                );
                if let Some(&l) = clauses[p].iter().find(|&&l| clauses[q].contains(&(l ^ 1))) {
                    let mut r: Vec<Lit> = clauses[p]
                        .iter()
                        .chain(&clauses[q])
                        .copied()
                        .filter(|&x| x >> 1 != l >> 1)
                        .collect();
                    r.sort_unstable();
                    r.dedup();
                    if !r.windows(2).any(|w| w[0] ^ 1 == w[1]) {
                        clauses.push(r);
                    }
                }
            }
            let (keep, stats) = implied_clauses(n, &clauses);
            let after: Vec<Vec<Lit>> = clauses
                .iter()
                .zip(&keep)
                .filter(|&(_, &k)| k)
                .map(|(c, _)| c.clone())
                .collect();
            assert_eq!(
                models(n, &after),
                models(n, &clauses),
                "case {case}: {clauses:?} to {after:?}"
            );
            assert!(
                clauses
                    .iter()
                    .zip(&keep)
                    .all(|(c, &k)| k || c.len() >= SHORTEST),
                "case {case}: only the clauses tried leave"
            );
            assert_eq!(stats.implied as usize, keep.iter().filter(|&&k| !k).count());
            left += usize::from(stats.implied > 0);
        }
        assert!(left > 100, "implied clauses met: {left} of 300");
    }

    #[test]
    fn a_long_clause_the_others_imply_leaves() {
        // 0 ∨ 1 and 2 ∨ 3 imply 0 ∨ 1 ∨ 2 ∨ 3 ∨ 4; the long clause 0 ∨ 2 ∨ 4
        // ∨ 6 is implied by nothing and stays, as do the short ones.
        let clauses: Vec<Vec<Lit>> = vec![
            vec![0, 2],
            vec![4, 6],
            vec![0, 2, 4, 6, 8],
            vec![0, 4, 8, 12],
        ];
        let (keep, stats) = implied_clauses(7, &clauses);
        assert_eq!(keep, vec![true, true, false, true]);
        assert_eq!(
            (stats.tried, stats.searched, stats.implied, stats.untried),
            (2, 1, 1, 0)
        );
        // 0 ∨ 1 ∨ 2 ∨ 3 subsumes 0 ∨ 1 ∨ 2 ∨ 3 ∨ 4, which is tried first and
        // leaves; the other, whose pairs only the longer one joined, is
        // then tried against what stays, and stays.
        let clauses: Vec<Vec<Lit>> = vec![vec![0, 2, 4, 6], vec![0, 2, 4, 6, 8], vec![1, 3]];
        let (keep, stats) = implied_clauses(5, &clauses);
        assert_eq!(keep, vec![true, false, true]);
        assert_eq!((stats.tried, stats.implied, stats.joined), (2, 1, 0));
        // A clause twice: the first tried leaves, and the other, asked
        // against what stays, stays.
        let clauses: Vec<Vec<Lit>> = vec![vec![0, 2, 4, 6], vec![0, 2, 4, 6], vec![1, 3]];
        let (keep, _) = implied_clauses(4, &clauses);
        assert_eq!(keep, vec![false, true, true]);
        // 0 ∨ 1 ∨ 2 subsumes 0 ∨ 1 ∨ 2 ∨ 3, but the clauses of three
        // literals and fewer, which stay, join every pair of its variables:
        // it stays untried.
        let clauses: Vec<Vec<Lit>> =
            vec![vec![0, 2, 4], vec![0, 2, 6], vec![4, 6], vec![0, 2, 4, 6]];
        let (keep, stats) = implied_clauses(4, &clauses);
        assert_eq!(keep, vec![true; 4]);
        assert_eq!((stats.tried, stats.joined), (0, 1));
    }

    #[test]
    fn the_windows_cover_every_clause_tried() {
        // More clauses tried than a window holds: each long clause is
        // subsumed by a short one, and each leaves.
        let n = 2 * WINDOW + 40;
        let mut clauses: Vec<Vec<Lit>> = Vec::new();
        for v in 0..n - 4 {
            clauses.push(vec![2 * v as Lit, 2 * (v + 1) as Lit]);
            clauses.push(vec![
                2 * v as Lit,
                2 * (v + 1) as Lit,
                2 * (v + 2) as Lit + 1,
                2 * (v + 3) as Lit,
            ]);
        }
        let (keep, stats) = implied_clauses(n, &clauses);
        assert_eq!(stats.implied as usize, n - 4);
        assert!(keep.iter().zip(&clauses).all(|(&k, c)| k == (c.len() == 2)));
    }
}
