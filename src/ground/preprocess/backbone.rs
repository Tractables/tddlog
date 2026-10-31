//! The literals that every model of a set of clauses makes true, its
//! backbone, and pairs of literals equal in every model, found by a SAT
//! solver ([`super::sat`]), with the clauses of two literals the solver
//! holds at the end.
//!
//! The conditioning ([`super::super::condition`]) hands over the clauses its unit
//! propagation leaves over facts. Every world the evidence allows satisfies
//! them, so a literal true in all their models is true in every such world,
//! and so is a clause the solver learns or proves from them: the facts it
//! fixes, and the equalities its clauses of two literals add to the
//! conditioning's graph of implications, condition the program exactly, as
//! the propagation's own do.
//!
//! The backbone search is the classic one (Janota, Lynce and Marques-Silva,
//! AI Comm. 2015, algorithm 2; cadiback's): a model first, whose literals
//! are the candidates, then for each candidate `l` left a call under the
//! assumption `\+l`. A call that finds no model makes `l` a backbone
//! literal, a unit clause for the calls after; a model drops every
//! candidate false in it, and every candidate that it could flip, every
//! clause of the candidate having another true literal (cadiback's
//! flipping). The calls go in
//! rounds of growing conflict limits, [`CONFLICT_LIMITS`]: a candidate
//! whose call gave up waits for the next round.
//!
//! Then the equalities (SAT sweeping, as in Kuehlmann's equivalence
//! checking): the literals that agree in each of the first [`MODELS`]
//! models are grouped, and in each group the first and each other are
//! equal where no model has one without the other, two calls under
//! assumptions; a model that tells them apart splits the group.
//!
//! The search stops once the solver's propagation steps pass
//! [`TICKS_PER_LITERAL`] per literal of the clauses, plus [`TICKS_FLOOR`],
//! or [`TICKS_PER_LITERAL_PROVING`] where it has proved a backbone literal
//! by then, and leaves the candidates after it open. What it found by then
//! is sound: an open candidate is only a fact the conditioning does not
//! fix.
//!
//! A weighted CNF (9 688 variables, 38 865 clauses): 6 563 variables are
//! backbones, of which the conditioning's unit propagation fixes 213.

use super::sat::{Lit, Outcome, Solver};
use crate::ground::program::Lists;

/// The conflicts after which a call gives up on its candidate, round by
/// round. A backbone literal proved fixes others through the solver's
/// propagation and makes the later calls easier, so the first round leaves
/// few candidates for later: on one weighted CNF these limits prove all
/// 6 563 backbone variables, and rounds of 10, 100 and 1 000 conflicts
/// 1 457 before the search stops.
const CONFLICT_LIMITS: [u64; 2] = [1000, 10_000];
/// The conflicts after which a call for an equality gives up.
const CONFLICTS_PER_CALL: u64 = 100;
/// The solver's propagation steps per literal of the clauses, over the
/// whole search, after which it stops where it has proved no backbone
/// literal.
const TICKS_PER_LITERAL: u64 = 1000;
/// The propagation steps per literal after which a search that has proved a
/// backbone literal within [`TICKS_PER_LITERAL`] stops. On weighted CNFs
/// the backbone literals proved come in a cascade once a few are, and the
/// first few are slow to find: with 1 000 steps a literal the search fixed
/// 269, 2 and 2 facts on three of them and left 11 294, 15 485 and 6 828
/// candidates open; with 3 000 it fixed 8 967, 9 726 and 6 084 and left
/// none (368 M, 651 M and 149 M steps). Where the search proves nothing the
/// larger budget finds nothing either: on two others it spent 1.53 G and
/// 541 M steps with no fact fixed, against 517 M and 187 M.
const TICKS_PER_LITERAL_PROVING: u64 = 3000;
/// Propagation steps the search may take however small the clauses.
const TICKS_FLOOR: u64 = 10_000_000;
/// The models whose values group the literals that may be equal.
const MODELS: u32 = 64;

/// What the search did, for `--stats`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Search {
    /// The solver's calls.
    pub calls: u64,
    /// Their conflicts.
    pub conflicts: u64,
    /// Their propagation steps.
    pub ticks: u64,
    /// The variables the search proved backbones.
    pub backbones: usize,
    /// The pairs of literals it proved equal.
    pub equivalences: usize,
    /// The candidates left open where a call or the search gave up.
    pub open: usize,
}

/// The backbone and the clauses of two literals the solver holds at the
/// end.
pub(in crate::ground) struct Found {
    /// The backbone literals.
    pub(in crate::ground) units: Vec<Lit>,
    /// Each clause of two literals once, given or learnt, of variables not
    /// in the backbone.
    pub(in crate::ground) binaries: Vec<(Lit, Lit)>,
    pub(in crate::ground) search: Search,
}

/// The backbone of the clauses over `num_vars` variables whose literals
/// are `lits`, clause `k` ending at `ends[k]`; `None` where they have no
/// model.
pub(in crate::ground) fn search(num_vars: usize, lits: &[Lit], ends: &[usize]) -> Option<Found> {
    // Each clause's literals once, which the flipping counts, and no
    // clause with a literal and its negation.
    let (mut own, mut own_ends) = (
        Vec::with_capacity(lits.len()),
        Vec::with_capacity(ends.len()),
    );
    for k in 0..ends.len() {
        let start = own.len();
        own.extend_from_slice(&lits[if k == 0 { 0 } else { ends[k - 1] }..ends[k]]);
        own[start..].sort_unstable();
        let mut kept = start;
        for j in start..own.len() {
            if j == start || own[j] != own[kept - 1] {
                own[kept] = own[j];
                kept += 1;
            }
        }
        own.truncate(kept);
        if own[start..].windows(2).any(|w| w[0] ^ 1 == w[1]) {
            own.truncate(start);
        } else {
            own_ends.push(own.len());
        }
    }
    let (lits, ends) = (&own[..], &own_ends[..]);
    let mut s = Solver::new(num_vars);
    s.tick_limit = TICKS_FLOOR + TICKS_PER_LITERAL * lits.len() as u64;
    let proving_limit = TICKS_FLOOR + TICKS_PER_LITERAL_PROVING * lits.len() as u64;
    for k in 0..ends.len() {
        if !s.add_clause(&lits[if k == 0 { 0 } else { ends[k - 1] }..ends[k]]) {
            return None;
        }
    }
    let mut search = Search {
        calls: 1,
        ..Search::default()
    };
    // The calls that proved a backbone literal.
    let mut proved = 0;
    let first = s.solve(&[], u64::MAX);
    if first == Outcome::Unsat {
        return None;
    }
    let mut signatures = Signatures {
        bits: vec![0; num_vars],
        taken: 0,
    };
    if first == Outcome::Sat {
        signatures.take(&mut s);
        let occurrences = Occurrences::new(num_vars, lits, ends);
        let mut candidates = occurrences.flipped(&mut s);
        // The facts false in the model first. On one weighted CNF every
        // fact left is a backbone: proving the 387 false ones first took
        // 388 calls, the propagation fixing the others; with the candidates
        // alternating between false and true facts, the calls for the true
        // ones gave up at 1 000 conflicts each, and the search stopped
        // after 37 calls with 18 fixed. On another, with 1 000 steps a
        // literal, the search left 408 candidates open where it left 1 938
        // in the facts' order.
        candidates.sort_by_key(|&l| l & 1 == 0);
        // Rounds of growing conflict limits: a candidate a call gave up on
        // is tried again in the next round, after the others.
        'rounds: for limit in CONFLICT_LIMITS {
            let mut deferred: Vec<Lit> = Vec::new();
            let mut i = 0;
            while i < candidates.len() {
                let l = candidates[i];
                i += 1;
                if s.fixed(l).is_some() {
                    continue;
                }
                if s.ticks >= s.tick_limit {
                    if proved > 0 && s.tick_limit < proving_limit {
                        s.tick_limit = proving_limit;
                    } else {
                        deferred.extend_from_slice(&candidates[i - 1..]);
                        candidates = deferred;
                        break 'rounds;
                    }
                }
                search.calls += 1;
                match s.solve(&[l ^ 1], limit) {
                    Outcome::Unsat => {
                        proved += 1;
                        // No model has `\+l`, and the first has `l`.
                        let consistent = s.add_clause(&[l]);
                        debug_assert!(consistent);
                    }
                    Outcome::Sat => {
                        signatures.take(&mut s);
                        // The candidates the model keeps, and that it would
                        // not keep with them flipped.
                        let keeps =
                            |s: &mut Solver, c: Lit| s.holds(c) && !occurrences.flippable(s, c);
                        s.ticks += (candidates.len() - i + deferred.len()) as u64;
                        let mut kept = i;
                        for j in i..candidates.len() {
                            if keeps(&mut s, candidates[j]) {
                                candidates[kept] = candidates[j];
                                kept += 1;
                            }
                        }
                        candidates.truncate(kept);
                        deferred.retain(|&c| keeps(&mut s, c));
                    }
                    Outcome::Unknown => deferred.push(l),
                }
            }
            candidates = deferred;
        }
        search.open = candidates.iter().filter(|&&c| s.fixed(c).is_none()).count();
        equivalences(&mut s, &signatures, &mut search);
    } else {
        search.open = num_vars;
    }
    search.conflicts = s.conflicts;
    search.ticks = s.ticks;
    // The solver's state after a model holds that model; the units are
    // read at level zero.
    let units: Vec<Lit> = (0..num_vars as Lit)
        .filter_map(|v| {
            s.fixed(2 * v)
                .map(|holds| if holds { 2 * v } else { 2 * v + 1 })
        })
        .collect();
    search.backbones = units.len();
    let binaries = s.binary_clauses();
    Some(Found {
        units,
        binaries,
        search,
    })
}

/// The values of the variables in the first models found, a bit per model,
/// the first model's highest.
struct Signatures {
    bits: Vec<u64>,
    taken: u32,
}

impl Signatures {
    /// Takes the model the solver holds, while fewer than [`MODELS`] are
    /// taken; charged as propagation steps.
    fn take(&mut self, s: &mut Solver) {
        if self.taken == MODELS {
            return;
        }
        self.taken += 1;
        for (v, b) in self.bits.iter_mut().enumerate() {
            *b = *b << 1 | u64::from(s.holds(2 * v as Lit));
        }
        s.ticks += self.bits.len() as u64;
    }
}

/// Proves literals equal where the models taken do not tell them apart
/// (SAT sweeping): the literals of the variables open at level zero,
/// each with the sign that is false in the last model taken, grouped by
/// their values in the models; in each group, the first and each other
/// are equal where no model has one without the other, two calls under
/// assumptions. Each direction proved is a clause of two literals added
/// to the solver; a model that tells them apart splits the group.
fn equivalences(s: &mut Solver, signatures: &Signatures, search: &mut Search) {
    // The bits of the models taken. A negated literal's key has the others
    // cleared: with fewer than [`MODELS`] models taken, a literal and the
    // negation of one equal to it otherwise differ in the bits no model
    // set, and were never tried. On a weighted CNF, searched over the
    // classes of equal facts, the search took fewer than 64 models and
    // proved 1 equality, leaving 542 facts where the search over the facts
    // left 520 (x(511) and the negation of x(200) among those not tried).
    let taken = if signatures.taken >= u64::BITS {
        u64::MAX
    } else {
        (1 << signatures.taken) - 1
    };
    let mut keyed: Vec<(u64, Lit)> = (0..s.num_vars() as Lit)
        .filter(|&v| s.fixed(2 * v).is_none())
        .map(|v| {
            let b = signatures.bits[v as usize];
            if b & 1 == 0 {
                (b, 2 * v)
            } else {
                (!b & taken, 2 * v + 1)
            }
        })
        .collect();
    keyed.sort_unstable();
    let mut groups: Vec<Vec<Lit>> = keyed
        .chunk_by(|a, b| a.0 == b.0)
        .filter(|g| g.len() > 1)
        .map(|g| g.iter().map(|&(_, l)| l).collect())
        .collect();
    while let Some(mut group) = groups.pop() {
        while group.len() > 1 {
            if s.ticks >= s.tick_limit {
                return;
            }
            let (r, x) = (group[0], group[1]);
            let mut apart = false;
            let mut proved = true;
            for (a, b) in [(r, x ^ 1), (r ^ 1, x)] {
                search.calls += 1;
                match s.solve(&[a, b], CONFLICTS_PER_CALL) {
                    Outcome::Unsat => {
                        s.add_clause(&[a ^ 1, b ^ 1]);
                    }
                    Outcome::Sat => {
                        apart = true;
                        break;
                    }
                    Outcome::Unknown => {
                        proved = false;
                        break;
                    }
                }
            }
            if apart {
                let first = s.holds(r);
                s.ticks += group.len() as u64;
                let (same, other): (Vec<Lit>, Vec<Lit>) =
                    group.iter().partition(|&&l| s.holds(l) == first);
                group = same;
                if other.len() > 1 {
                    groups.push(other);
                }
            } else {
                search.equivalences += usize::from(proved);
                group.swap_remove(1);
            }
        }
    }
}

/// The clauses of each literal.
struct Occurrences<'a> {
    lits: &'a [Lit],
    ends: &'a [usize],
    occ: Lists<u32>,
}

impl<'a> Occurrences<'a> {
    fn new(num_vars: usize, lits: &'a [Lit], ends: &'a [usize]) -> Self {
        let clauses = || {
            (0..ends.len()).flat_map(|c| {
                lits[if c == 0 { 0 } else { ends[c - 1] }..ends[c]]
                    .iter()
                    .map(move |&l| (l as usize, c as u32))
            })
        };
        Occurrences {
            lits,
            ends,
            occ: Lists::new(2 * num_vars, clauses),
        }
    }

    fn clause(&self, c: u32) -> &'a [Lit] {
        let c = c as usize;
        &self.lits[if c == 0 { 0 } else { self.ends[c - 1] }..self.ends[c]]
    }

    fn of(&self, l: Lit) -> &[u32] {
        &self.occ[l as usize]
    }

    /// Whether the model the solver holds stays one with `l`, true in it,
    /// made false: every clause of `l` has another true literal. The
    /// literals scanned are charged as propagation steps.
    fn flippable(&self, s: &mut Solver, l: Lit) -> bool {
        for &c in self.of(l) {
            let clause = self.clause(c);
            s.ticks += clause.len() as u64;
            if !clause.iter().any(|&m| m != l && s.holds(m)) {
                return false;
            }
        }
        true
    }

    /// The literals of the model the solver holds, but for those fixed at
    /// level zero and those flipped: a literal whose every clause has
    /// another true literal is flipped, in variable order, the counts
    /// updated, so that what is flipped later sees the model with the
    /// earlier flips.
    fn flipped(&self, s: &mut Solver) -> Vec<Lit> {
        let n = s.num_vars();
        let mut trues: Vec<u32> = (0..self.ends.len() as u32)
            .map(|c| self.clause(c).iter().filter(|&&l| s.holds(l)).count() as u32)
            .collect();
        s.ticks += self.lits.len() as u64;
        let mut out = Vec::new();
        for v in 0..n as Lit {
            let l = if s.holds(2 * v) { 2 * v } else { 2 * v + 1 };
            if s.fixed(l).is_some() {
                continue;
            }
            // `l` is true in the model with the flips so far where it was
            // not flipped, which it was not: each variable is visited once.
            let mine = self.of(l);
            s.ticks += mine.len() as u64;
            if mine.iter().all(|&c| trues[c as usize] >= 2) {
                for &c in mine {
                    trues[c as usize] -= 1;
                }
                for &c in self.of(l ^ 1) {
                    trues[c as usize] += 1;
                }
            } else {
                out.push(l);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DIMACS-style clauses, 1-based and signed, as `search` takes them.
    fn flat(clauses: &[&[i32]]) -> (Vec<Lit>, Vec<usize>) {
        let mut lits = Vec::new();
        let mut ends = Vec::new();
        for c in clauses {
            lits.extend(
                c.iter()
                    .map(|&x| 2 * (x.unsigned_abs() - 1) + u32::from(x < 0)),
            );
            ends.push(lits.len());
        }
        (lits, ends)
    }

    /// The backbone by enumeration, as solver literals, ascending.
    fn enumerated(n: usize, clauses: &[&[i32]]) -> Option<Vec<Lit>> {
        let models: Vec<u64> = (0..1u64 << n)
            .filter(|&w| {
                clauses.iter().all(|c| {
                    c.iter()
                        .any(|&x| (w >> (x.unsigned_abs() - 1) & 1 == 1) == (x > 0))
                })
            })
            .collect();
        if models.is_empty() {
            return None;
        }
        let mut out = Vec::new();
        for v in 0..n {
            if models.iter().all(|&w| w >> v & 1 == 1) {
                out.push(2 * v as Lit);
            } else if models.iter().all(|&w| w >> v & 1 == 0) {
                out.push(2 * v as Lit + 1);
            }
        }
        Some(out)
    }

    #[test]
    fn a_chain_of_implications_and_a_clause_fix_its_variables() {
        // 1 or 2, 1 implies 3, 2 implies 3, 3 implies 4: 3 and 4 hold in
        // every model, which unit propagation does not find.
        let clauses: &[&[i32]] = &[&[1, 2], &[-1, 3], &[-2, 3], &[-3, 4], &[1, 5, 6]];
        let (lits, ends) = flat(clauses);
        let found = search(6, &lits, &ends).expect("a model");
        assert_eq!(found.units, [4, 6]);
        assert_eq!(found.search.open, 0);
        assert_eq!(Some(found.units), enumerated(6, clauses));
    }

    #[test]
    fn a_variable_equal_to_the_negation_of_another_is_proved_so_from_few_models() {
        // 2 is the negation of 1 and 4 equals 3, each through clauses of
        // three literals; the search takes fewer than 64 models.
        let clauses: &[&[i32]] = &[
            &[1, 2, 5],
            &[1, 2, -5],
            &[-1, -2, 5],
            &[-1, -2, -5],
            &[3, -4, 6],
            &[3, -4, -6],
            &[-3, 4, 6],
            &[-3, 4, -6],
            &[1, 3, 7],
        ];
        let (lits, ends) = flat(clauses);
        let found = search(7, &lits, &ends).expect("a model");
        assert_eq!(found.search.equivalences, 2);
        for (a, b) in [(0, 2), (1, 3), (4, 7), (5, 6)] {
            assert!(
                found.binaries.contains(&(a, b)) || found.binaries.contains(&(b, a)),
                "the clause {a} {b}"
            );
        }
    }

    #[test]
    fn the_backbone_agrees_with_enumeration() {
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for round in 0..400 {
            let n = 10;
            let m = 20 + (next() % 30) as usize;
            let clauses: Vec<Vec<i32>> = (0..m)
                .map(|_| {
                    let k = 2 + (next() % 2) as usize;
                    (0..k)
                        .map(|_| {
                            let v = (next() % n as u64) as i32 + 1;
                            if next() & 1 == 1 { v } else { -v }
                        })
                        .collect()
                })
                .collect();
            let refs: Vec<&[i32]> = clauses.iter().map(Vec::as_slice).collect();
            let (lits, ends) = flat(&refs);
            let found = search(n, &lits, &ends);
            assert_eq!(
                found.as_ref().map(|f| f.units.clone()),
                enumerated(n, &refs),
                "round {round}: {clauses:?}"
            );
            if let Some(found) = found {
                // Every clause of two the solver holds holds in every model.
                let models: Vec<u64> = (0..1u64 << n)
                    .filter(|&w| {
                        refs.iter().all(|c| {
                            c.iter()
                                .any(|&x| (w >> (x.unsigned_abs() - 1) & 1 == 1) == (x > 0))
                        })
                    })
                    .collect();
                let holds = |w: u64, l: Lit| (w >> (l >> 1) & 1 == 1) == (l & 1 == 0);
                for &(a, b) in &found.binaries {
                    assert!(
                        models.iter().all(|&w| holds(w, a) || holds(w, b)),
                        "round {round}: ({a}, {b}) is implied"
                    );
                }
            }
        }
    }
}
