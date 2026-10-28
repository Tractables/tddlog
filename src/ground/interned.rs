//! Distinct lists numbered in the order they are first seen, their items
//! end to end in one buffer, rather than a map keyed by the lists
//! themselves, which costs an allocation per list and a second copy of
//! each: the bodies of the exclusive sets ([`super::exclusive`]), and the
//! grounder's compound terms and placeholder tuples ([`super::engine`]),
//! tens of millions of terms where a demand reaches every state of a
//! grammar.

use std::hash::BuildHasher;

use rustc_hash::{FxBuildHasher, FxHashMap};

pub(super) struct Interned<T> {
    items: Vec<T>,
    /// Where each list starts in `items`; the next list's start, or the
    /// end of `items` for the last, ends it.
    starts: Vec<usize>,
    /// The last list with each hash, and per list the one before it with
    /// the same hash.
    by_hash: FxHashMap<u64, u32>,
    same_hash: Vec<u32>,
}

impl<T> Default for Interned<T> {
    fn default() -> Self {
        Interned {
            items: Vec::new(),
            starts: Vec::new(),
            by_hash: FxHashMap::default(),
            same_hash: Vec::new(),
        }
    }
}

impl<T: Copy + Eq + std::hash::Hash> Interned<T> {
    pub(super) fn with_capacity(lists: usize) -> Self {
        Interned {
            items: Vec::new(),
            starts: Vec::with_capacity(lists),
            by_hash: FxHashMap::with_capacity_and_hasher(lists, Default::default()),
            same_hash: Vec::with_capacity(lists),
        }
    }

    /// How many lists there are.
    pub(super) fn len(&self) -> usize {
        self.starts.len()
    }

    /// List `i`.
    pub(super) fn list(&self, i: usize) -> &[T] {
        let end = self.starts.get(i + 1).copied().unwrap_or(self.items.len());
        &self.items[self.starts[i]..end]
    }

    /// The number of `list`, where it is one of the lists.
    pub(super) fn get(&self, list: &[T]) -> Option<u32> {
        let mut b = *self.by_hash.get(&FxBuildHasher.hash_one(list))?;
        while b != u32::MAX {
            if self.list(b as usize) == list {
                return Some(b);
            }
            b = self.same_hash[b as usize];
        }
        None
    }

    /// The number of `list`, numbered anew if it is new, and whether it is.
    /// Its hash is looked up once, and a new list takes the entry found,
    /// rather than looked up and then inserted: nearly every body is new.
    pub(super) fn id(&mut self, list: &[T]) -> (u32, bool) {
        let Interned {
            items,
            starts,
            by_hash,
            same_hash,
        } = self;
        let last = by_hash
            .entry(FxBuildHasher.hash_one(list))
            .or_insert(u32::MAX);
        let mut b = *last;
        while b != u32::MAX {
            let start = starts[b as usize];
            let end = starts.get(b as usize + 1).copied().unwrap_or(items.len());
            if items[start..end] == *list {
                return (b, false);
            }
            b = same_hash[b as usize];
        }
        let id = starts.len() as u32;
        starts.push(items.len());
        items.extend_from_slice(list);
        same_hash.push(*last);
        *last = id;
        (id, true)
    }

    /// Whether the allocator gave room for `lists` more lists of `items`
    /// items in all, which [`Interned::id`] then adds without allocating.
    pub(super) fn try_reserve(&mut self, lists: usize, items: usize) -> bool {
        self.items.try_reserve(items).is_ok()
            && self.starts.try_reserve(lists).is_ok()
            && self.same_hash.try_reserve(lists).is_ok()
            && self.by_hash.try_reserve(lists).is_ok()
    }
}
