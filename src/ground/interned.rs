//! Distinct lists numbered in the order they are first seen, their items
//! end to end in one buffer, rather than a map keyed by the lists
//! themselves, which costs an allocation per list and a second copy of
//! each: the bodies of the exclusive sets ([`super::exclusive`]), and the
//! grounder's compound terms and placeholder tuples ([`super::engine`]),
//! tens of millions of terms where a demand reaches every state of a
//! grammar.

use std::hash::BuildHasher;

use rustc_hash::FxBuildHasher;

use crate::by_hash::ByHash;

pub(super) struct Interned<T> {
    items: Vec<T>,
    /// Where each list starts in `items`; the next list's start, or the
    /// end of `items` for the last, ends it.
    starts: Vec<usize>,
    /// The lists by the hash of their items.
    by_hash: ByHash,
}

impl<T> Default for Interned<T> {
    fn default() -> Self {
        Interned {
            items: Vec::new(),
            starts: Vec::new(),
            by_hash: ByHash::default(),
        }
    }
}

/// List `i` of the lists that `starts` places in `items`.
fn list<'a, T>(items: &'a [T], starts: &[usize], i: usize) -> &'a [T] {
    let end = starts.get(i + 1).copied().unwrap_or(items.len());
    &items[starts[i]..end]
}

impl<T: Copy + Eq + std::hash::Hash> Interned<T> {
    pub(super) fn with_capacity(lists: usize) -> Self {
        Interned {
            items: Vec::new(),
            starts: Vec::with_capacity(lists),
            by_hash: ByHash::with_capacity(lists),
        }
    }

    /// How many lists there are.
    pub(super) fn len(&self) -> usize {
        self.starts.len()
    }

    /// List `i`.
    pub(super) fn list(&self, i: usize) -> &[T] {
        list(&self.items, &self.starts, i)
    }

    /// The number of `list`, where it is one of the lists.
    pub(super) fn get(&self, list: &[T]) -> Option<u32> {
        self.by_hash.find(FxBuildHasher.hash_one(list), |b| {
            self.list(b as usize) == list
        })
    }

    /// The number of `list`, numbered anew if it is new, and whether it is.
    /// Its hash is looked up once ([`ByHash::find_or_push`]): nearly every
    /// body is new.
    pub(super) fn id(&mut self, list: &[T]) -> (u32, bool) {
        let Interned {
            items,
            starts,
            by_hash,
        } = self;
        let (id, new) = by_hash.find_or_push(FxBuildHasher.hash_one(list), |b| {
            self::list(items, starts, b as usize) == list
        });
        if new {
            starts.push(items.len());
            items.extend_from_slice(list);
        }
        (id, new)
    }

    /// Whether the allocator gave room for `lists` more lists of `items`
    /// items in all, which [`Interned::id`] then adds without allocating.
    pub(super) fn try_reserve(&mut self, lists: usize, items: usize) -> bool {
        self.items.try_reserve(items).is_ok()
            && self.starts.try_reserve(lists).is_ok()
            && self.by_hash.try_reserve(lists)
    }
}
