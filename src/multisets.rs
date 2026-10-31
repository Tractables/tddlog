//! Lists of literals numbered as multisets.

use crate::by_hash::ByHash;
use crate::ground::GLit;

/// Lists of literals numbered in the order they are first seen, two lists
/// being one where they hold the same literals as often each, that is
/// where they are equal once sorted. A list is looked up by a hash that
/// does not depend on the order of its literals, the sum of their mixed
/// bits ([`GLit::mixed`]), and compared by counting where the hash
/// matches, so that no list is sorted. Each is kept as first given, the
/// lists end to end in one buffer.
pub(crate) struct Multisets {
    lits: Vec<GLit>,
    /// Where each list ends in `lits`, after a 0.
    ends: Vec<usize>,
    /// The lists by their hash.
    by_hash: ByHash,
    /// A count per literal, zero between comparisons.
    count: Vec<i32>,
}

impl Multisets {
    /// No lists, of literals over atoms below `num_atoms`.
    pub(crate) fn new(num_atoms: usize) -> Self {
        Multisets {
            lits: Vec::new(),
            ends: vec![0],
            by_hash: ByHash::default(),
            count: vec![0; 2 * num_atoms],
        }
    }

    /// How many lists there are.
    pub(crate) fn len(&self) -> usize {
        self.by_hash.len()
    }

    /// List `id` as first given.
    pub(crate) fn get(&self, id: u32) -> &[GLit] {
        &self.lits[self.ends[id as usize]..self.ends[id as usize + 1]]
    }

    /// No lists, the counts kept.
    pub(crate) fn clear(&mut self) {
        self.lits.clear();
        self.ends.truncate(1);
        self.by_hash.clear();
    }

    /// The number of the list equal to `lits` as a multiset, if there is one.
    pub(crate) fn find(&mut self, lits: &[GLit]) -> Option<u32> {
        let Multisets {
            lits: all,
            ends,
            by_hash,
            count,
        } = self;
        by_hash.find(Self::hash(lits), |id| same(all, ends, count, id, lits))
    }

    /// The number of `lits` as a multiset, numbered anew if it is new, and
    /// whether it is.
    pub(crate) fn id(&mut self, lits: &[GLit]) -> (u32, bool) {
        let Multisets {
            lits: all,
            ends,
            by_hash,
            count,
        } = self;
        let (id, new) =
            by_hash.find_or_push(Self::hash(lits), |id| same(all, ends, count, id, lits));
        if new {
            all.extend_from_slice(lits);
            ends.push(all.len());
        }
        (id, new)
    }

    fn hash(lits: &[GLit]) -> u64 {
        lits.iter().fold(0u64, |h, &l| h.wrapping_add(l.mixed()))
    }
}

/// Whether list `id` of the lists that `ends` places in `lits` holds the
/// literals of `y` as often each, counted in `count`, which it leaves at
/// zero.
fn same(lits: &[GLit], ends: &[usize], count: &mut [i32], id: u32, y: &[GLit]) -> bool {
    let x = &lits[ends[id as usize]..ends[id as usize + 1]];
    if x.len() != y.len() {
        return false;
    }
    if x == y {
        return true;
    }
    let slot = |l: GLit| 2 * l.atom() as usize + l.is_positive() as usize;
    for &l in x {
        count[slot(l)] += 1;
    }
    for &l in y {
        count[slot(l)] -= 1;
    }
    // Each literal of `x` as often in `y`, and as many literals in all,
    // leaves none of `y` over.
    let same = x.iter().all(|&l| count[slot(l)] == 0);
    for &l in x.iter().chain(y) {
        count[slot(l)] = 0;
    }
    same
}

#[cfg(test)]
mod tests {
    use super::*;
    use GLit::{Neg, Pos};

    #[test]
    fn lists_are_one_where_they_are_equal_once_sorted() {
        let mut m = Multisets::new(4);
        assert_eq!(m.id(&[Pos(1), Neg(2)]), (0, true));
        assert_eq!(m.id(&[Neg(2), Pos(1)]), (0, false));
        assert_eq!(m.id(&[Pos(1), Pos(2)]), (1, true));
        assert_eq!(m.id(&[Pos(1), Neg(2), Pos(1)]), (2, true));
        assert_eq!(m.find(&[Pos(1), Pos(1), Neg(2)]), Some(2));
        assert_eq!(m.find(&[Pos(1), Neg(2), Neg(2)]), None);
        assert_eq!(m.find(&[]), None);
        assert_eq!(m.id(&[]), (3, true));
        assert_eq!(m.get(1), &[Pos(1), Pos(2)]);
        m.clear();
        assert_eq!(m.len(), 0);
        assert_eq!(m.id(&[Neg(2), Pos(1)]), (0, true));
    }
}
