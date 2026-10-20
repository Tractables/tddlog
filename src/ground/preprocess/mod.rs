//! The grounder's SAT stage: the backbones and equivalences a SAT solver
//! finds for the conditioning on the evidence ([`backbone`], [`sat`]),
//! and, where the program is then a CNF over its facts, the elimination of
//! the facts its clauses define ([`defined`]) outside an independent
//! support ([`support`]), once the clauses the others imply are gone
//! ([`implied`]).

pub(super) mod backbone;
pub(super) mod defined;
pub(super) mod implied;
pub(super) mod sat;
pub(super) mod support;

/// The random CNFs the SAT stage's tests check by enumeration.
#[cfg(test)]
mod testing {
    use super::sat::Lit;

    /// The models of `clauses` over `n` variables, at most 16, as bit sets.
    pub(super) fn models(n: usize, clauses: &[Vec<Lit>]) -> Vec<u32> {
        (0u32..1 << n)
            .filter(|&w| {
                clauses
                    .iter()
                    .all(|c| c.iter().any(|&l| (w >> (l >> 1) & 1 == 1) == (l & 1 == 0)))
            })
            .collect()
    }

    /// Draws from a linear congruential generator seeded with `seed`:
    /// each call a number below its argument.
    pub(super) fn draws(mut seed: u64) -> impl FnMut(u64) -> u64 {
        move |m| {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) % m
        }
    }

    /// `count` clauses over `n` variables of `2 + next(spread)` literals
    /// drawn at random, each sorted and with one literal per variable.
    pub(super) fn random_clauses(
        next: &mut impl FnMut(u64) -> u64,
        n: usize,
        count: usize,
        spread: u64,
    ) -> Vec<Vec<Lit>> {
        (0..count)
            .map(|_| {
                let len = 2 + next(spread) as usize;
                let mut c: Vec<Lit> = (0..len)
                    .map(|_| 2 * next(n as u64) as u32 + next(2) as u32)
                    .collect();
                c.sort_unstable();
                c.dedup_by(|a, b| *a >> 1 == *b >> 1);
                c
            })
            .collect()
    }

    /// `count` times, the clauses that make a variable drawn the AND, OR
    /// or XOR of two others drawn, where the three differ, added to
    /// `clauses`.
    pub(super) fn add_gates(
        next: &mut impl FnMut(u64) -> u64,
        n: usize,
        count: usize,
        clauses: &mut Vec<Vec<Lit>>,
    ) {
        for _ in 0..count {
            let (y, a, b) = (
                next(n as u64) as u32,
                next(n as u64) as u32,
                next(n as u64) as u32,
            );
            if y == a || y == b || a == b {
                continue;
            }
            let (y, a, b) = (2 * y, 2 * a, 2 * b);
            match next(3) {
                0 => clauses.extend([vec![y ^ 1, a], vec![y ^ 1, b], vec![y, a ^ 1, b ^ 1]]),
                1 => clauses.extend([vec![y, a ^ 1], vec![y, b ^ 1], vec![y ^ 1, a, b]]),
                _ => clauses.extend([
                    vec![y ^ 1, a, b],
                    vec![y ^ 1, a ^ 1, b ^ 1],
                    vec![y, a ^ 1, b],
                    vec![y, a, b ^ 1],
                ]),
            }
        }
    }
}
