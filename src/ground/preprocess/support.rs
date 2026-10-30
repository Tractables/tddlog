//! An independent support of a CNF: a set of its variables of which every
//! other variable is a function in its models. The facts of probability
//! one half outside it are eliminated by resolution ([`super::defined`]).
//!
//! A variable `y` is a function of a set `X` exactly where two models
//! that agree on `X` agree on `y` (Padoa's theorem), where the CNF twice,
//! `F(x) ∧ F(x')`, with `x_i = x'_i` for every `i` of `X`, has no model
//! with `y` and `\+y'`. The search is Arjun's (Soos and Meel, ICCAD 2022):
//! every variable starts in the support, and each candidate in turn leaves
//! it where it is a function of the others still in it; the variables
//! that left are functions of those that stay (by induction from the last
//! to leave, each a function of variables that stay or leave after it).
//! A variable that is not a candidate is never tried and stays: its
//! equality is a clause of its own. A candidate's equality is guarded by a
//! variable `e_i`, `e_i -> (x_i <-> x'_i)`, and a call assumes the guards
//! of the candidates in the support but `y`'s, then `y` and `\+y'`: the
//! guards of the candidates not yet tried first, in the reverse of the
//! order they are tried in, then those of the candidates kept, so that a
//! call shares all but the last few of its assumptions with the one before
//! and keeps their levels ([`super::sat::Solver::keep_assumptions`]). A
//! call that stops at its conflict limit keeps its variable, as does every
//! candidate left untried once the solver's propagation steps pass the
//! search's budget: a support larger than it could be is a support all
//! the same.
//!
//! Trying the candidates alone leaves every other variable to define them:
//! a variable that is not a candidate gains nothing by leaving, and the
//! fewer variables stay, the fewer candidates they define.
//!
//! The candidates are tried in Arjun's order, the fewest occurrences of a
//! variable's rarer sign first: the output of a gate read by few clauses is
//! tried before the inputs that many gates read, and leaves while they
//! stay. On weighted CNFs (after the conditioning) the support this leaves
//! is smaller than the one the order of the most clauses first leaves on 7
//! of 9 instances where the elimination falls short, and as small as
//! Arjun's where the search tries every candidate: 145 facts of 1 938 for
//! 275 on one instance, 114 for 215 on a second, 86 for 158 on a third.

use super::sat::{Lit, Outcome, Solver};

/// The conflicts after which a call gives up and keeps its variable.
const CONFLICTS_PER_CALL: u64 = 1000;
/// The solver's propagation steps per literal of the CNF, over the whole
/// search, after which the candidates not yet tried stay.
const TICKS_PER_LITERAL: u64 = 2000;
/// Propagation steps the search may take however small the CNF.
const TICKS_FLOOR: u64 = 20_000_000;

/// What the search did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SupportStats {
    /// The candidates tried.
    pub tried: u64,
    /// The candidates proved functions of the others.
    pub defined: u64,
    /// The candidates whose call stopped at its conflict limit.
    pub unknown: u64,
    /// The candidates the search did not try, for its budget.
    pub untried: u64,
    /// The solver's conflicts.
    pub conflicts: u64,
    /// The solver's propagation steps.
    pub ticks: u64,
}

/// An independent support of the CNF of `n` variables and `clauses`, each
/// clause's literals sorted and on distinct variables, of which only the
/// variables `candidate` marks may leave: each variable's membership. The
/// candidates are tried in the order of the fewest occurrences of their
/// rarer sign first.
pub(in crate::ground) fn independent_support(
    n: usize,
    clauses: &[Vec<Lit>],
    candidate: &[bool],
) -> (Vec<bool>, SupportStats) {
    let mut stats = SupportStats::default();
    let mut support = vec![true; n];
    if !candidate.contains(&true) {
        return (support, stats);
    }
    let copy = |l: Lit| l + 2 * n as Lit;
    // The guards of the candidates, numbered after the copies.
    let mut guard_of = vec![Lit::MAX; n];
    let mut guards = 0;
    for (i, g) in guard_of.iter_mut().enumerate() {
        if candidate[i] {
            *g = 2 * (2 * n + guards) as Lit;
            guards += 1;
        }
    }
    let mut s = Solver::new(2 * n + guards);
    s.keep_assumptions = true;
    let mut literals = 0u64;
    let mut occurrences = vec![0u32; 2 * n];
    for c in clauses {
        literals += c.len() as u64;
        for &l in c {
            occurrences[l as usize] += 1;
        }
        let theirs: Vec<Lit> = c.iter().map(|&l| copy(l)).collect();
        if !s.add_clause(c) || !s.add_clause(&theirs) {
            // No model: every variable is a function of the others, and
            // the elimination finds the empty clause.
            return (support, stats);
        }
    }
    for (i, &e) in guard_of.iter().enumerate() {
        let (xi, ci) = (2 * i as Lit, copy(2 * i as Lit));
        if e == Lit::MAX {
            s.add_clause(&[xi ^ 1, ci]);
            s.add_clause(&[xi, ci ^ 1]);
        } else {
            s.add_clause(&[e ^ 1, xi ^ 1, ci]);
            s.add_clause(&[e ^ 1, xi, ci ^ 1]);
        }
    }
    s.tick_limit = TICKS_FLOOR + TICKS_PER_LITERAL * literals;
    let mut order: Vec<usize> = (0..n).filter(|&i| candidate[i]).collect();
    order.sort_by_key(|&i| (occurrences[2 * i].min(occurrences[2 * i + 1]), i));
    let mut kept: Vec<Lit> = Vec::new();
    let mut assumptions: Vec<Lit> = Vec::with_capacity(order.len() + 2);
    for (k, &y) in order.iter().enumerate() {
        if s.ticks >= s.tick_limit {
            stats.untried = (order.len() - k) as u64;
            break;
        }
        assumptions.clear();
        assumptions.extend(order[k + 1..].iter().rev().map(|&i| guard_of[i]));
        assumptions.extend_from_slice(&kept);
        assumptions.push(2 * y as Lit);
        assumptions.push(copy(2 * y as Lit) ^ 1);
        stats.tried += 1;
        match s.solve(&assumptions, CONFLICTS_PER_CALL) {
            Outcome::Unsat => {
                support[y] = false;
                stats.defined += 1;
            }
            Outcome::Sat => kept.push(guard_of[y]),
            Outcome::Unknown => {
                stats.unknown += 1;
                kept.push(guard_of[y]);
            }
        }
    }
    stats.conflicts = s.conflicts;
    stats.ticks = s.ticks;
    (support, stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::preprocess::testing::{add_gates, draws, models, random_clauses};

    /// Whether every variable outside `support` is a function of those in
    /// it, by enumeration of the models.
    fn is_support(n: usize, clauses: &[Vec<Lit>], support: &[bool]) -> bool {
        let models = models(n, clauses);
        let mask: u32 = (0..n).filter(|&i| support[i]).map(|i| 1 << i).sum();
        models
            .iter()
            .all(|&a| models.iter().all(|&b| a & mask != b & mask || a == b))
    }

    /// Random CNFs of clauses of two and three literals, some with gates
    /// (a variable equal to the and, or or xor of two others), and random
    /// candidates, whose support is checked by enumeration; a gate's output
    /// that is a candidate leaves it where its inputs stay, and a variable
    /// that is not a candidate stays.
    #[test]
    fn the_support_determines_every_other_variable() {
        let mut next = draws(11);
        let mut smaller = 0;
        for case in 0..300 {
            let n = 4 + case % 9;
            let mut clauses = random_clauses(&mut next, n, n / 2 + case % 3, 2);
            add_gates(&mut next, n, 1 + case % 4, &mut clauses);
            for c in &mut clauses {
                c.sort_unstable();
            }
            let candidate: Vec<bool> = (0..n).map(|_| case % 5 == 0 || next(4) > 0).collect();
            let (support, stats) = independent_support(n, &clauses, &candidate);
            assert!(
                is_support(n, &clauses, &support),
                "case {case}: {clauses:?} with support {support:?}"
            );
            assert!(
                (0..n).all(|i| support[i] || candidate[i]),
                "case {case}: only candidates leave"
            );
            assert_eq!(
                stats.defined as usize,
                support.iter().filter(|&&s| !s).count()
            );
            if !models(n, &clauses).is_empty() {
                assert_eq!(
                    stats.tried as usize,
                    candidate.iter().filter(|&&c| c).count(),
                    "case {case}: every candidate tried"
                );
            }
            smaller += usize::from(stats.defined > 0);
        }
        assert!(
            smaller > 150,
            "the gates leave the support: {smaller} of 300"
        );
    }

    #[test]
    fn a_gate_output_leaves_and_its_inputs_stay() {
        // 2 is the AND of 0 and 1, and 3 their XOR: both leave, the inputs
        // stay. With 0 not a candidate, it stays whatever defines it.
        let clauses: Vec<Vec<Lit>> = vec![
            vec![0, 5],
            vec![2, 5],
            vec![1, 3, 4],
            vec![0, 2, 7],
            vec![1, 3, 7],
            vec![1, 2, 6],
            vec![0, 3, 6],
        ];
        let (support, _) = independent_support(4, &clauses, &[true; 4]);
        assert_eq!(support.iter().filter(|&&s| s).count(), 2, "{support:?}");
        assert!(is_support(4, &clauses, &support));
        let (support, _) = independent_support(4, &clauses, &[false, true, true, true]);
        assert!(
            support[0] && is_support(4, &clauses, &support),
            "{support:?}"
        );
        assert_eq!(support.iter().filter(|&&s| s).count(), 2, "{support:?}");
    }
}
