//! Entries numbered as they come and found by a hash of their own: per
//! hash the last entry with it, and per entry the one before it with the
//! same hash. The entries are the caller's, kept in its own arrays by
//! number and compared by a test it gives, so that none is held twice: the
//! lists of [`crate::multisets`] and of the grounder's interned terms, end
//! to end in one buffer, a relation's tuples, the rules kept so far. A map
//! keyed by the entries themselves would hold a second copy of each, and
//! for a list an allocation of its own.

use rustc_hash::FxHashMap;

/// No entry: the end of a chain.
const NONE: u32 = u32::MAX;

#[derive(Clone, Default)]
pub(crate) struct ByHash {
    /// The last entry with each hash.
    last: FxHashMap<u64, u32>,
    /// Per entry, the one before it with the same hash, or [`NONE`].
    before: Vec<u32>,
}

impl ByHash {
    /// No entries, with room for `n`.
    pub(crate) fn with_capacity(n: usize) -> Self {
        ByHash {
            last: FxHashMap::with_capacity_and_hasher(n, Default::default()),
            before: Vec::with_capacity(n),
        }
    }

    /// How many entries there are.
    pub(crate) fn len(&self) -> usize {
        self.before.len()
    }

    /// No entries, the room kept.
    pub(crate) fn clear(&mut self) {
        self.last.clear();
        self.before.clear();
    }

    /// Whether the allocator gave room for `n` more entries, which
    /// [`ByHash::push`] then adds without allocating.
    pub(crate) fn try_reserve(&mut self, n: usize) -> bool {
        self.before.try_reserve(n).is_ok() && self.last.try_reserve(n).is_ok()
    }

    /// The last entry of hash `hash` that `same` holds of.
    #[inline]
    pub(crate) fn find(&self, hash: u64, mut same: impl FnMut(u32) -> bool) -> Option<u32> {
        let mut k = *self.last.get(&hash)?;
        while k != NONE {
            if same(k) {
                return Some(k);
            }
            k = self.before[k as usize];
        }
        None
    }

    /// A new entry of hash `hash`, numbered [`ByHash::len`], at which the
    /// caller keeps it.
    #[inline]
    pub(crate) fn push(&mut self, hash: u64) -> u32 {
        let k = self.before.len() as u32;
        let before = self.last.insert(hash, k).unwrap_or(NONE);
        self.before.push(before);
        k
    }

    /// The last entry of hash `hash` that `same` holds of, and false; or,
    /// where none does, a new entry ([`ByHash::push`]) and true. The hash
    /// is looked up once, and a new entry takes the place found, rather
    /// than the hash being looked up and then inserted.
    #[inline]
    pub(crate) fn find_or_push(
        &mut self,
        hash: u64,
        mut same: impl FnMut(u32) -> bool,
    ) -> (u32, bool) {
        let ByHash { last, before } = self;
        let place = last.entry(hash).or_insert(NONE);
        let mut k = *place;
        while k != NONE {
            if same(k) {
                return (k, false);
            }
            k = before[k as usize];
        }
        let k = before.len() as u32;
        before.push(*place);
        *place = k;
        (k, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_are_found_by_hash_and_test() {
        let items = [10, 20, 10, 30];
        let mut by_hash = ByHash::default();
        // 10 and 30 share a hash.
        let hash = |x: i32| (x % 20) as u64;
        let mut kept: Vec<i32> = Vec::new();
        let mut numbers = Vec::new();
        for x in items {
            let (k, new) = by_hash.find_or_push(hash(x), |k| kept[k as usize] == x);
            if new {
                kept.push(x);
            }
            numbers.push((k, new));
        }
        assert_eq!(numbers, [(0, true), (1, true), (0, false), (2, true)]);
        assert_eq!(by_hash.len(), 3);
        assert_eq!(by_hash.find(hash(30), |k| kept[k as usize] == 30), Some(2));
        assert_eq!(by_hash.find(hash(10), |k| kept[k as usize] == 10), Some(0));
        assert_eq!(by_hash.find(hash(50), |k| kept[k as usize] == 50), None);
        assert_eq!(by_hash.find(hash(45), |_| true), None);
        assert_eq!(by_hash.push(hash(50)), 3);
        by_hash.clear();
        assert_eq!(by_hash.len(), 0);
        assert_eq!(by_hash.find(hash(10), |_| true), None);
    }
}
