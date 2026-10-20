//! Merging cubes that differ in one literal's sign (`x ∧ c` and `¬x ∧ c`
//! into `c`), which the `none_of` builder does before it builds the
//! negation of a disjunction of cubes ([`crate::tp::none_of`]) and the
//! indicator rewrite does to the rules of a table read in binary
//! ([`crate::indicators`]).

use rustc_hash::FxHashMap;

/// What [`merge_complements`] did.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Merged {
    No,
    Yes,
    /// It made a cube without literals: the disjunction holds everywhere.
    ToTrue,
}

/// Replaces each two cubes that differ in the sign of one literal alone,
/// `c ∧ x` and `c ∧ ¬x`, by `c`, until no two cubes are so, which leaves
/// their disjunction as it was; where some literal is in both signs, it
/// also drops a cube equal to another. A set of
/// values read in binary ([`crate::indicators`]) names each value by the
/// cube of its code, and the codes of a range of values merge to a few
/// cubes (mnist_sum96's cubes of a partial sum and a digit for each pair
/// that makes a sum, merged, are a thirtieth as many).
///
/// A pass pairs each cube with the one that differs at the lowest leaf
/// where one does, among those no lower leaf paired, and the cubes a pass
/// makes are tried in the next against every cube left: two cubes left
/// from one pass are not a pair. The leaves in both signs (`occurs`) are
/// the cubes' variables ([`CubeMerge`]); a cube's other literals are in any
/// pair's both cubes.
///
/// Where it merged, `origin` holds of each cube left the least number of
/// the cubes given that were merged into it, its own where it was given;
/// a cube given is merged into one cube left at most.
pub(crate) fn merge_complements(
    codes: &mut Vec<u32>,
    start: &mut Vec<u32>,
    occurs: &[u32],
    origin: &mut Vec<u32>,
) -> Merged {
    // A merge, of a pair or of two equal cubes, is of two cubes on the same
    // leaves, which a loop of clauses seldom has (bn_barley's builds merge
    // none), so this check comes before numbering each build's leaves and
    // contexts. Each cube's leaves, hashed, are looked for twice.
    let mut on: Vec<u64> = start
        .windows(2)
        .map(|w| {
            codes[w[0] as usize..w[1] as usize]
                .iter()
                .fold(CubeMerge::MIX, |h, &x| {
                    (h.rotate_left(5) ^ u64::from(x / 2)).wrapping_mul(CubeMerge::MIX)
                })
        })
        .collect();
    on.sort_unstable();
    if on.windows(2).all(|w| w[0] != w[1]) {
        return Merged::No;
    }
    // The leaves in both signs are the cubes' bits, numbered in order and
    // looked up per literal. A build past the check above numbers its own
    // leaves, a word each (`u32::MAX` for a leaf in one sign), rather than
    // over every node of the vtree for every loop of clauses, and looks
    // them up rather than searching the leaves sorted.
    let mut leaf: Vec<u32> = Vec::new();
    let mut place = vec![u32::MAX; occurs.len() / 2];
    for (v, signs) in occurs.as_chunks::<2>().0.iter().enumerate() {
        if signs[0] > 0 && signs[1] > 0 {
            place[v] = leaf.len() as u32;
            leaf.push(v as u32);
        }
    }
    if leaf.is_empty() {
        return Merged::No;
    }
    let place = |x: u32| {
        Some(place[x as usize / 2])
            .filter(|&p| p != u32::MAX)
            .map(|p| 2 * p + x % 2)
    };
    let mut cubes = CubeMerge::default();
    cubes.reset(leaf.len());
    let mut rest: Vec<u32> = Vec::new();
    for w in start.windows(2) {
        let lits = &codes[w[0] as usize..w[1] as usize];
        rest.clear();
        rest.extend(lits.iter().copied().filter(|&x| place(x).is_none()));
        let k = cubes.context(&rest);
        cubes
            .push(k, lits.iter().filter_map(|&x| place(x)))
            .expect("a cube reads a leaf once");
    }
    let merged = cubes.merge();
    if merged != Merged::Yes {
        return merged;
    }
    let mut kept_codes = Vec::with_capacity(codes.len());
    let mut kept_start = vec![0u32];
    origin.clear();
    for (c, o) in cubes.kept() {
        let mut rest = cubes.context_lits(c).iter().copied().peekable();
        for &p in cubes.lits(c) {
            let x = 2 * leaf[p as usize / 2] + p % 2;
            while let Some(y) = rest.next_if(|&y| y < x) {
                kept_codes.push(y);
            }
            kept_codes.push(x);
        }
        kept_codes.extend(rest);
        kept_start.push(kept_codes.len() as u32);
        origin.push(o);
    }
    *codes = kept_codes;
    *start = kept_start;
    Merged::Yes
}

/// Cubes as [`merge_complements`] merges them: each cube's literals over
/// the variables that occur in both signs among the cubes, numbered in
/// their order, as a list ascending, `2 * v + 1` for `v` and `2 * v` for
/// `¬v`, and its other literals, which no merge drops, as the number of
/// their set, its context. A cube of a context no other cube has pairs
/// with none. A cube's hash is the sum of a term for its context and one
/// for each literal, so that a literal flipped or dropped changes it by
/// a term or two, rather than hashing each cube in full for every pair
/// tried. Masks over the variables would cost every word of the mask
/// wherever a cube is hashed, compared or copied, and the cubes of the
/// large builds are short over many leaves.
#[derive(Default)]
pub(crate) struct CubeMerge {
    vars: usize,
    /// The literals of each context, sorted, one context after another
    /// from `context_start`; a context's number by the hash of its
    /// literals, and the next of the same hash; the cubes given of each.
    context_lits: Vec<u32>,
    context_start: Vec<u32>,
    context_by_hash: FxHashMap<u64, u32>,
    context_next: Vec<u32>,
    context_cubes: Vec<u32>,
    /// Of each cube, given or made: its context; its literals, one cube
    /// after another from `start`; its hash; whether it is left; the
    /// least number of the cubes given merged into it.
    context: Vec<u32>,
    lits: Vec<u32>,
    start: Vec<u32>,
    keys: Vec<u64>,
    alive: Vec<bool>,
    from: Vec<u32>,
    /// The cube left of each hash.
    by_key: FxHashMap<u64, u32>,
    /// Per variable, the first and last of the cubes that try it next in a
    /// pass, on a list through `tries`, each with the place of its literal
    /// and the next on the list.
    heads: Vec<(u32, u32)>,
    tries: Vec<(u32, u32, u32)>,
    /// The pairs found at a variable, each by the cube that would have
    /// found it first, before the cubes they make are numbered.
    found: Vec<(u32, u32, u32, u32)>,
}

impl CubeMerge {
    const MIX: u64 = 0x517c_c1b7_2722_0a95;

    /// Emptied, for cubes over `vars` variables.
    pub(crate) fn reset(&mut self, vars: usize) {
        self.vars = vars;
        self.context_lits.clear();
        self.context_start.clear();
        self.context_start.push(0);
        self.context_by_hash.clear();
        self.context_next.clear();
        self.context_cubes.clear();
        self.context.clear();
        self.lits.clear();
        self.start.clear();
        self.start.push(0);
    }

    /// The number of the context of the literals `lits`, sorted.
    pub(crate) fn context(&mut self, lits: &[u32]) -> u32 {
        let h = lits.iter().fold(Self::MIX, |h, &x| {
            (h.rotate_left(5) ^ u64::from(x)).wrapping_mul(Self::MIX)
        });
        let first = self.context_by_hash.get(&h).copied().unwrap_or(u32::MAX);
        let mut k = first;
        while k != u32::MAX {
            let (a, b) = (
                self.context_start[k as usize] as usize,
                self.context_start[k as usize + 1] as usize,
            );
            if self.context_lits[a..b] == *lits {
                return k;
            }
            k = self.context_next[k as usize];
        }
        let k = self.context_next.len() as u32;
        self.context_lits.extend_from_slice(lits);
        self.context_start.push(self.context_lits.len() as u32);
        self.context_next.push(first);
        self.context_by_hash.insert(h, k);
        self.context_cubes.push(0);
        k
    }

    /// How many contexts have been numbered.
    pub(crate) fn contexts(&self) -> usize {
        self.context_next.len()
    }

    /// The literals of cube `c`'s context.
    pub(crate) fn context_lits(&self, c: usize) -> &[u32] {
        let k = self.context[c] as usize;
        &self.context_lits[self.context_start[k] as usize..self.context_start[k + 1] as usize]
    }

    /// Adds a cube of context `k` and the literals `lits` over the
    /// variables, in any order; its number, or `None`, adding nothing,
    /// where it reads a variable twice.
    pub(crate) fn push(&mut self, k: u32, lits: impl IntoIterator<Item = u32>) -> Option<usize> {
        let at = self.lits.len();
        self.lits.extend(lits);
        self.lits[at..].sort_unstable();
        if self.lits[at..].windows(2).any(|w| w[0] / 2 == w[1] / 2) {
            self.lits.truncate(at);
            return None;
        }
        self.start.push(self.lits.len() as u32);
        self.context.push(k);
        self.context_cubes[k as usize] += 1;
        Some(self.context.len() - 1)
    }

    /// The literals of cube `c` over the variables, ascending.
    pub(crate) fn lits(&self, c: usize) -> &[u32] {
        &self.lits[self.start[c] as usize..self.start[c + 1] as usize]
    }

    /// Whether cube `c` reads variable `v`.
    pub(crate) fn reads(&self, c: usize, v: usize) -> bool {
        self.lits(c)
            .binary_search_by_key(&v, |&x| x as usize / 2)
            .is_ok()
    }

    /// The term of a literal, or of a context `k` as `k | 1 << 32`, in a
    /// cube's hash.
    fn term(x: u64) -> u64 {
        let z = (x ^ Self::MIX).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        let z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn key(&self, c: usize) -> u64 {
        self.lits(c)
            .iter()
            .fold(Self::term(u64::from(self.context[c]) | 1 << 32), |h, &x| {
                h.wrapping_add(Self::term(u64::from(x)))
            })
    }

    /// Merges the cubes as [`merge_complements`] does.
    pub(crate) fn merge(&mut self) -> Merged {
        let n = self.context.len();
        self.alive.clear();
        self.alive.resize(n, true);
        self.from.clear();
        self.from.extend(0..n as u32);
        let mut keys = std::mem::take(&mut self.keys);
        keys.clear();
        keys.extend((0..n).map(|c| self.key(c)));
        self.keys = keys;
        self.by_key.clear();
        self.by_key.reserve(n);
        let mut merged = false;
        let mut fresh = 0..n;
        let mut first = true;
        while !fresh.is_empty() {
            // A cube equal to one left is dropped; one whose hash another
            // holds is not found by others, and stays.
            for c in fresh.clone() {
                match self.by_key.entry(self.keys[c]) {
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(c as u32);
                    }
                    std::collections::hash_map::Entry::Occupied(mut e) => {
                        let d = *e.get() as usize;
                        if !self.alive[d] {
                            e.insert(c as u32);
                        } else if self.context[c] == self.context[d] && self.lits(c) == self.lits(d)
                        {
                            self.alive[c] = false;
                            merged = true;
                        }
                    }
                }
            }
            // The cubes try their literals by variable, every cube at one
            // variable before any at the next, and a cube goes on the list
            // of its next literal's variable only while it is left: written
            // out for every literal of every cube and sorted by variable,
            // most pairs would be of cubes already merged. In the first
            // pass both cubes of a pair are fresh,
            // and the one with the variable positive finds the other.
            self.heads.clear();
            self.heads.resize(self.vars, (u32::MAX, u32::MAX));
            self.tries.clear();
            for c in fresh.clone() {
                if self.alive[c] && self.context_cubes[self.context[c] as usize] >= 2 {
                    self.try_next(c, self.start[c] as usize, first);
                }
            }
            let made = self.context.len();
            for v in 0..self.vars {
                self.found.clear();
                let mut t = self.heads[v].0;
                while t != u32::MAX {
                    let (c, i, next) = self.tries[t as usize];
                    t = next;
                    let c = c as usize;
                    if !self.alive[c] {
                        continue;
                    }
                    let x = self.lits[i as usize];
                    let flipped = self.keys[c]
                        .wrapping_sub(Self::term(u64::from(x)))
                        .wrapping_add(Self::term(u64::from(x ^ 1)));
                    // The two the same but for `x` in `c` and its negation in
                    // `d`, at the same place since a variable's two literals
                    // are adjacent.
                    let d = self.by_key.get(&flipped).map(|&d| d as usize).filter(|&d| {
                        let (p, q) = (self.lits(c), self.lits(d));
                        d != c
                            && self.alive[d]
                            && self.context[d] == self.context[c]
                            && p.len() == q.len()
                            && p.iter()
                                .zip(q)
                                .all(|(&a, &b)| if a == x { b == x ^ 1 } else { a == b })
                    });
                    let Some(d) = d else {
                        self.try_next(c, i as usize + 1, first);
                        continue;
                    };
                    if self.context_lits(c).len() + self.lits(c).len() == 1 {
                        return Merged::ToTrue;
                    }
                    self.alive[c] = false;
                    self.alive[d] = false;
                    merged = true;
                    // After the first pass both cubes of a pair try the
                    // variable where both are fresh, the first in the pass
                    // finding the other; either makes the same cube.
                    let by = if !first && d >= fresh.start {
                        c.min(d)
                    } else {
                        c
                    };
                    self.found.push((by as u32, c as u32, d as u32, x));
                }
                // The cubes made are numbered in the order of the cubes that
                // found them, each pass's in the order of its variables.
                if !self.found.is_sorted_by_key(|f| f.0) {
                    self.found.sort_unstable_by_key(|f| f.0);
                }
                for f in 0..self.found.len() {
                    let (_, c, d, x) = self.found[f];
                    let (c, d) = (c as usize, d as usize);
                    let (s, e) = (self.start[c] as usize, self.start[c + 1] as usize);
                    let at = self.lits.len();
                    self.lits.extend_from_within(s..e);
                    let drop = self.lits[at..].binary_search(&x).expect("the cube reads x");
                    self.lits.remove(at + drop);
                    self.start.push(self.lits.len() as u32);
                    self.context.push(self.context[c]);
                    self.keys
                        .push(self.keys[c].wrapping_sub(Self::term(u64::from(x))));
                    self.alive.push(true);
                    self.from.push(self.from[c].min(self.from[d]));
                }
            }
            fresh = made..self.context.len();
            first = false;
        }
        if merged { Merged::Yes } else { Merged::No }
    }

    /// Puts cube `c` on the list of the variable of its first literal from
    /// place `i` on, positive in the first pass, if it has one.
    fn try_next(&mut self, c: usize, i: usize, first: bool) {
        let Some(j) = (i..self.start[c + 1] as usize).find(|&j| !first || self.lits[j] % 2 == 1)
        else {
            return;
        };
        let v = self.lits[j] as usize / 2;
        let t = self.tries.len() as u32;
        self.tries.push((c as u32, j as u32, u32::MAX));
        match self.heads[v] {
            (u32::MAX, _) => self.heads[v] = (t, t),
            (_, last) => {
                self.tries[last as usize].2 = t;
                self.heads[v].1 = t;
            }
        }
    }

    /// The cubes left after [`merge`](Self::merge), each with the least
    /// number of the cubes given that were merged into it, its own where it
    /// was given.
    pub(crate) fn kept(&self) -> impl Iterator<Item = (usize, u32)> + '_ {
        (0..self.context.len())
            .filter(|&c| self.alive[c])
            .map(|c| (c, self.from[c]))
    }
}

#[cfg(test)]
mod tests {
    use super::{Merged, merge_complements};

    /// Merging leaves the cubes' disjunction as it was, over every
    /// assignment of eight leaves, and no two cubes left differ in one
    /// literal's sign alone, or are equal where it merged; the codes of a
    /// range of values in binary merge to its aligned blocks.
    #[test]
    fn merging_keeps_the_disjunction() {
        let holds = |codes: &[u32], start: &[u32], world: u32| {
            start.windows(2).any(|w| {
                codes[w[0] as usize..w[1] as usize]
                    .iter()
                    .all(|&x| (world >> (x / 2) & 1) == x % 2)
            })
        };
        let mut rng = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = |m: u64| {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng % m
        };
        for round in 0..400 {
            let (mut codes, mut start) = (Vec::new(), vec![0u32]);
            // Cubes of random literals, or the codes of random values over
            // the first `bits` leaves, some with a literal of another.
            let bits = 1 + next(5) as u32;
            for _ in 0..1 + next(24) {
                let mut lits: Vec<u32> = if round % 2 == 0 {
                    (0..1 + next(4)).map(|_| next(16) as u32).collect()
                } else {
                    let value = next(1 << bits) as u32;
                    let mut lits: Vec<u32> = (0..bits).map(|b| 2 * b + (value >> b & 1)).collect();
                    if next(2) == 0 {
                        lits.push(2 * (bits + next(8 - u64::from(bits)) as u32) + 1);
                    }
                    lits
                };
                lits.sort_unstable();
                lits.dedup();
                if lits.windows(2).any(|w| w[0] / 2 == w[1] / 2) {
                    continue;
                }
                codes.extend(lits);
                start.push(codes.len() as u32);
            }
            let (before, before_start) = (codes.clone(), start.clone());
            let mut occurs = vec![0u32; 16];
            for &x in &codes {
                occurs[x as usize] += 1;
            }
            let mut origin = Vec::new();
            let merged = merge_complements(&mut codes, &mut start, &occurs, &mut origin);
            if merged == Merged::ToTrue {
                assert!(
                    (0..256).all(|w| holds(&before, &before_start, w)),
                    "round {round}: {before:?} is not true"
                );
                continue;
            }
            for w in 0..256 {
                assert_eq!(
                    holds(&codes, &start, w),
                    holds(&before, &before_start, w),
                    "round {round}: {before:?} {before_start:?} became {codes:?} {start:?}"
                );
            }
            let cubes: Vec<&[u32]> = start
                .windows(2)
                .map(|w| &codes[w[0] as usize..w[1] as usize])
                .collect();
            for (i, a) in cubes.iter().enumerate() {
                for b in &cubes[i + 1..] {
                    assert!(merged == Merged::No || a != b, "round {round}: {a:?} twice");
                    let one_sign = a.len() == b.len()
                        && a.iter().zip(b.iter()).all(|(x, y)| x / 2 == y / 2)
                        && a.iter().zip(b.iter()).filter(|(x, y)| x != y).count() == 1;
                    assert!(!one_sign, "round {round}: {a:?} and {b:?} left");
                }
            }
            assert_eq!(
                merged == Merged::No,
                codes == before && start == before_start,
                "round {round}"
            );
            if merged == Merged::Yes {
                // Each cube left is in its origin, and no cube given is
                // the origin of two.
                let given: Vec<&[u32]> = before_start
                    .windows(2)
                    .map(|w| &before[w[0] as usize..w[1] as usize])
                    .collect();
                let mut seen = vec![false; given.len()];
                for (cube, &o) in cubes.iter().zip(&origin) {
                    assert!(
                        !std::mem::replace(&mut seen[o as usize], true),
                        "round {round}: origin {o} twice"
                    );
                    assert!(
                        cube.iter().all(|x| given[o as usize].contains(x)),
                        "round {round}: {cube:?} is not in its origin {:?}",
                        given[o as usize]
                    );
                }
            }
        }
        // The values 1 to 6 over three leaves: 1, 2..=3, 4..=5 and 6.
        let (mut codes, mut start) = (Vec::new(), vec![0u32]);
        for value in 1..=6u32 {
            codes.extend((0..3).map(|b| 2 * b + (value >> b & 1)));
            start.push(codes.len() as u32);
        }
        let mut occurs = vec![0u32; 6];
        for &x in &codes {
            occurs[x as usize] += 1;
        }
        assert_eq!(
            merge_complements(&mut codes, &mut start, &occurs, &mut Vec::new()),
            Merged::Yes
        );
        let mut cubes: Vec<Vec<u32>> = start
            .windows(2)
            .map(|w| codes[w[0] as usize..w[1] as usize].to_vec())
            .collect();
        cubes.sort();
        assert_eq!(
            cubes,
            vec![vec![0, 3, 5], vec![1, 2, 4], vec![2, 5], vec![3, 4]]
        );
    }
}
