//! The tables of a ground program, as the rewrites of their rows see
//! them ([`Tables`]), over its structure ([`super::structure`]).

use std::hash::Hasher;

use rustc_hash::{FxHashMap, FxHasher};

use super::structure::Structure;
use super::{AtomId, AtomKind, GLit, GroundProgram, RulesByHead};
use crate::by_hash::ByHash;

/// The most contexts [`Tables::exclusive`] compares two by two.
const MOST_COMPARED: usize = 64;

/// The tables of a ground program, as the rewrites of their rows see them
/// ([`super::rows`], [`super::cover`]). A table is one exclusive set,
/// the values of a network variable, or one derived atom in no set. A fact
/// is a table's own when no query or evidence names it and every rule that
/// reads it defines an atom of the table: a switch of the table's rows.
pub(crate) struct Tables {
    structure: Structure,
    set_of: Vec<usize>,
    /// The index of each atom's component in dependency order.
    position: Vec<usize>,
    /// The table of the heads of every rule that reads each atom, where
    /// they are of one table; [`Tables::NO_READER`] or
    /// [`Tables::READERS_DIFFER`] otherwise, rather than the rules that read
    /// each atom as a list per atom.
    readers_table: Vec<usize>,
    named: Vec<bool>,
    independent: FxHashMap<usize, bool>,
    stamp: Vec<usize>,
}

impl Tables {
    const NO_READER: usize = usize::MAX;
    const READERS_DIFFER: usize = usize::MAX - 1;

    /// The tables of `gp`, which switch sharing made of the program these
    /// were the tables of ([`Structure::renumbered`]).
    pub(crate) fn renumbered(self, gp: &GroundProgram, map: &[AtomId]) -> Self {
        Self::of_structure(gp, self.structure.renumbered(gp, map))
    }

    /// The tables of `gp`, whose structure is `structure`.
    pub(crate) fn of_structure(gp: &GroundProgram, structure: Structure) -> Self {
        let n = gp.num_atoms();
        let mut set_of = vec![usize::MAX; n];
        for (i, set) in structure.sets.iter().enumerate() {
            for &a in set {
                set_of[a as usize] = i;
            }
        }
        let mut position = vec![usize::MAX; n];
        for (i, scc) in structure.sccs.iter().enumerate() {
            for &a in scc {
                position[a as usize] = i;
            }
        }
        let mut named = vec![false; n];
        for a in gp.named_atoms() {
            named[a as usize] = true;
        }
        let mut tables = Tables {
            structure,
            set_of,
            position,
            readers_table: Vec::new(),
            named,
            independent: FxHashMap::default(),
            stamp: vec![0; n],
        };
        let mut readers_table = vec![Self::NO_READER; n];
        for r in &gp.rules {
            let t = tables.of(r.head);
            for l in &r.body {
                let seen = &mut readers_table[l.atom() as usize];
                if *seen == Self::NO_READER {
                    *seen = t;
                } else if *seen != t {
                    *seen = Self::READERS_DIFFER;
                }
            }
        }
        tables.readers_table = readers_table;
        tables
    }

    /// The structure of the program they are the tables of.
    pub(crate) fn into_structure(self) -> Structure {
        self.structure
    }

    /// The exclusive sets, a set after the sets its atoms read.
    pub(crate) fn sets(&self) -> &[Vec<AtomId>] {
        &self.structure.sets
    }

    /// The rules of each atom.
    pub(crate) fn by_head(&self) -> &RulesByHead {
        &self.structure.by_head
    }

    /// Whether no derived atom depends on itself.
    pub(crate) fn acyclic(&self, gp: &GroundProgram) -> bool {
        self.structure
            .sccs
            .iter()
            .all(|c| !super::deps::is_cyclic(gp, &self.structure.by_head, c))
    }

    /// The derived atoms in dependency order, by component.
    pub(crate) fn order(&self) -> Vec<AtomId> {
        let mut order: Vec<AtomId> = (0..self.position.len() as AtomId)
            .filter(|&a| self.position[a as usize] != usize::MAX)
            .collect();
        order.sort_by_key(|&a| self.position[a as usize]);
        order
    }

    /// The table of a derived atom: its exclusive set's index, or the number
    /// of sets plus the atom.
    pub(crate) fn of(&self, a: AtomId) -> usize {
        match self.set_of[a as usize] {
            usize::MAX => self.structure.sets.len() + a as usize,
            set => set,
        }
    }

    /// The atoms of `table`: an exclusive set's, or the one atom of a table
    /// of one ([`Tables::of`]).
    pub(super) fn members(&self, table: usize) -> impl Iterator<Item = AtomId> + '_ {
        let sets = &self.structure.sets;
        let (set, one) = match sets.get(table) {
            Some(set) => (set.as_slice(), None),
            None => (&[][..], Some((table - sets.len()) as AtomId)),
        };
        set.iter().copied().chain(one)
    }

    /// The table whose own fact `s` is, if it is a fact and one's own.
    pub(crate) fn owner(&self, gp: &GroundProgram, s: AtomId) -> Option<usize> {
        if !gp.is_fact(s) || self.named[s as usize] {
            return None;
        }
        match self.readers_table[s as usize] {
            Self::NO_READER | Self::READERS_DIFFER => None,
            table => Some(table),
        }
    }

    /// Whether two sorted bodies cannot both hold: a literal of one is the
    /// negation of a literal of the other, or they read two different atoms
    /// of one exclusive set.
    pub(crate) fn clash(&self, x: &[GLit], y: &[GLit]) -> bool {
        x.iter().any(|&l| {
            y.binary_search(&l.negated()).is_ok()
                || (l.is_positive()
                    && self.set_of[l.atom() as usize] != usize::MAX
                    && y.iter().any(|&m| {
                        m.is_positive()
                            && m.atom() != l.atom()
                            && self.set_of[m.atom() as usize] == self.set_of[l.atom() as usize]
                    }))
        })
    }

    /// Whether every two of `n` contexts clash ([`Tables::clash`]),
    /// `context(i)` the literals of the `i`-th: at once where, of the
    /// exclusive sets that every context reads one atom of positively, no
    /// two contexts read the same atoms, and two by two where they are
    /// [`MOST_COMPARED`] at most. The rows of a table ([`super::rows`]) and
    /// the readers of a choice ([`super::contexts`]) are tested so. A choice
    /// may have tens of thousands of readers, each context the rest of a
    /// rule's body, so the contexts are walked in place, and the walk stops
    /// at the first two that read the same atoms of the sets kept so far:
    /// they read the same atoms of the sets kept at the end too.
    pub(crate) fn exclusive<'a, I>(&self, n: usize, context: impl Fn(usize) -> I) -> bool
    where
        I: Iterator<Item = &'a GLit>,
    {
        if n < 2 {
            return true;
        }
        let set = |l: &GLit| {
            Some(self.set_of[l.atom() as usize]).filter(|&s| l.is_positive() && s != usize::MAX)
        };
        // The sets the first context reads, then each context's atom of
        // each, a set dropped where a context reads none of it or two, and
        // the contexts walked so far found by their atoms of the sets kept,
        // all of them found anew where a set is dropped.
        let mut sets: Vec<usize> = context(0).filter_map(set).collect();
        sets.sort_unstable();
        sets.dedup();
        let w = sets.len();
        let mut atoms = vec![AtomId::MAX; n * w];
        let mut kept = vec![true; w];
        let mut left = w;
        let mut seen = ByHash::default();
        let mut distinct = w > 0;
        for i in 0..n {
            let read = &mut atoms[i * w..(i + 1) * w];
            for l in context(i) {
                if let Some(j) = set(l).and_then(|s| sets.binary_search(&s).ok()) {
                    if read[j] == AtomId::MAX {
                        read[j] = l.atom();
                    } else {
                        kept[j] = false;
                    }
                }
            }
            for (j, &a) in read.iter().enumerate() {
                kept[j] &= a != AtomId::MAX;
            }
            let now = kept.iter().filter(|&&k| k).count();
            if now == 0 {
                distinct = false;
                break;
            }
            let from = if now < left {
                seen.clear();
                0
            } else {
                i
            };
            left = now;
            let tuple = |k: usize| atoms[k * w..(k + 1) * w].iter().zip(&kept);
            let new = (from..=i).all(|k| {
                let mut h = FxHasher::default();
                tuple(k)
                    .filter(|(_, on)| **on)
                    .for_each(|(&a, _)| h.write_u32(a));
                seen.find_or_push(h.finish(), |o| {
                    tuple(o as usize)
                        .zip(tuple(k))
                        .all(|((a, on), (b, _))| !*on || a == b)
                })
                .1
            });
            if !new {
                distinct = false;
                break;
            }
        }
        if distinct {
            return true;
        }
        if n > MOST_COMPARED {
            return false;
        }
        let contexts: Vec<Vec<GLit>> = (0..n)
            .map(|i| {
                let mut c: Vec<GLit> = context(i).copied().collect();
                c.sort_unstable();
                c
            })
            .collect();
        (0..n).all(|i| (i + 1..n).all(|j| self.clash(&contexts[i], &contexts[j])))
    }

    /// Whether no atom of a table depends on an atom of the table. Only
    /// atoms at or after the table's earliest component can depend on one
    /// of it.
    pub(crate) fn independent(&mut self, gp: &GroundProgram, table: usize) -> bool {
        if let Some(&known) = self.independent.get(&table) {
            return known;
        }
        let earliest = self
            .members(table)
            .map(|m| self.position[m as usize])
            .min()
            .unwrap_or(0);
        let mark = self.independent.len() + 1;
        let mut stack: Vec<usize> = Vec::new();
        let by_head = &self.structure.by_head;
        let push_body = |a: usize, stack: &mut Vec<usize>| {
            for &ri in &by_head[a] {
                for l in &gp.rules[ri].body {
                    let b = l.atom() as usize;
                    if gp.kinds[b] == AtomKind::Derived {
                        stack.push(b);
                    }
                }
            }
        };
        for m in self.members(table) {
            push_body(m as usize, &mut stack);
        }
        let mut ok = true;
        while let Some(b) = stack.pop() {
            if self.of(b as AtomId) == table {
                ok = false;
                break;
            }
            if self.stamp[b] == mark || self.position[b] < earliest {
                continue;
            }
            self.stamp[b] = mark;
            push_body(b, &mut stack);
        }
        self.independent.insert(table, ok);
        ok
    }
}
