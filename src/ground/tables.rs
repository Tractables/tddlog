//! The tables of a ground program, as the rewrites of their rows see
//! them ([`Tables`]), over its structure ([`super::structure`]).

use rustc_hash::FxHashMap;

use super::structure::Structure;
use super::{AtomId, AtomKind, GLit, GroundProgram, RulesByHead};

/// The tables of a ground program, as the rewrites of their rows see them
/// ([`super::classes`], [`super::cover`]). A table is one exclusive set,
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

    /// Whether no atom of a table depends on an atom of the table. Only
    /// atoms at or after the table's earliest component can depend on one
    /// of it.
    pub(crate) fn independent(&mut self, gp: &GroundProgram, table: usize) -> bool {
        if let Some(&known) = self.independent.get(&table) {
            return known;
        }
        let sets = &self.structure.sets;
        let members: Vec<usize> = if table < sets.len() {
            sets[table].iter().map(|&a| a as usize).collect()
        } else {
            vec![table - sets.len()]
        };
        let earliest = members.iter().map(|&m| self.position[m]).min().unwrap_or(0);
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
        for &m in &members {
            push_body(m, &mut stack);
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
