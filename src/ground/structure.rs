//! The structure of a ground program that the rewrites read
//! ([`Structure`]): its rules by head, its derived atoms' components in
//! dependency order, their units, and the exclusive sets
//! ([`super::exclusive`]), kept with the program it is of
//! ([`super::GroundProgram::structure`]).

use super::exclusive::{Found, exclusive_sets};
use super::{AtomId, GroundProgram, RulesByHead};

/// What the rewrites read of a ground program's structure: its rules by
/// head, its derived atoms' components in dependency order, the unit of
/// each atom in the sense of [`super::deps::components`], and the
/// exclusive sets. The grounder leaves it with the program it computed it
/// for ([`GroundProgram::structure`]), so that the indicators and `auto`
/// do not compute it again.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Structure {
    pub(crate) by_head: RulesByHead,
    pub(crate) sccs: Vec<Vec<AtomId>>,
    /// The unit of each derived atom, `usize::MAX` for a fact.
    pub(crate) base_of: Vec<usize>,
    /// How many units there are.
    pub(crate) units: usize,
    pub(crate) sets: Vec<Vec<AtomId>>,
    /// The predicate of each derived atom, numbered in dependency order,
    /// `usize::MAX` for a fact; and the derived atoms of each predicate.
    pub(super) predicate_of: Vec<usize>,
    pub(super) atoms_of: Vec<usize>,
    /// The atoms, rules and body literals of the program it is of, which a
    /// program changed since is unlikely to keep all three of.
    size: (usize, usize, usize),
}

impl Structure {
    pub(crate) fn of(gp: &GroundProgram) -> Self {
        // The sets are read off bodies that are conjunctions: the program
        // before the indicators, which write rules of pairs.
        assert!(
            gp.rules.iter().all(|r| !r.pairs),
            "the exclusive sets of a program with rules of pairs"
        );
        let (by_head, sccs, base_of, units) = Self::components(gp);
        let Found {
            sets,
            predicate_of,
            atoms_of,
        } = exclusive_sets(gp, &by_head, &base_of, &sccs);
        Structure {
            by_head,
            sccs,
            base_of,
            units,
            sets,
            predicate_of,
            atoms_of,
            size: Self::size_of(gp),
        }
    }

    /// The structure of `gp`, which switch sharing ([`super::share`]) made
    /// of the program this is the structure of, `map` the new number of
    /// each atom ([`super::simplify::compact`]). The sharing renames facts,
    /// each a table's own and read in rows that exclude each other, and
    /// drops those no rule reads any more: every derived atom reads the
    /// derived atoms it read before, so the components are the same, and
    /// two rules clash after as before, so the sets are the same, renumbered.
    pub(crate) fn renumbered(self, gp: &GroundProgram, map: &[AtomId]) -> Self {
        let kept = |v: Vec<usize>| -> Vec<usize> {
            v.into_iter()
                .zip(map)
                .filter(|&(_, &a)| a != AtomId::MAX)
                .map(|(x, _)| x)
                .collect()
        };
        let Structure {
            mut sccs,
            base_of,
            units,
            mut sets,
            predicate_of,
            atoms_of,
            ..
        } = self;
        for a in sccs.iter_mut().chain(&mut sets).flatten() {
            *a = map[*a as usize];
        }
        let renumbered = Structure {
            by_head: gp.rules_by_head(),
            sccs,
            base_of: kept(base_of),
            units,
            sets,
            predicate_of: kept(predicate_of),
            atoms_of,
            size: Self::size_of(gp),
        };
        debug_assert!(
            renumbered == Self::of(gp),
            "the structure renumbered is not the structure found anew"
        );
        renumbered
    }

    /// The rules by head, the components in dependency order, and the unit
    /// of each atom and how many units there are.
    pub(super) fn components(
        gp: &GroundProgram,
    ) -> (RulesByHead, Vec<Vec<AtomId>>, Vec<usize>, usize) {
        let super::deps::Deps { by_head, sccs } = super::deps::Deps::of(gp);
        let (rank, units) = super::deps::unit_ranks(gp, &by_head, &sccs);
        let mut base_of = vec![usize::MAX; gp.num_atoms()];
        for (scc, &unit) in sccs.iter().zip(&rank) {
            for &a in scc {
                base_of[a as usize] = unit;
            }
        }
        (by_head, sccs, base_of, units)
    }

    fn size_of(gp: &GroundProgram) -> (usize, usize, usize) {
        (
            gp.num_atoms(),
            gp.rules.len(),
            gp.rules.iter().map(|r| r.body.len()).sum(),
        )
    }

    /// Whether it can be `gp`'s: a cheap test against the program having
    /// been changed since, not a proof that it is.
    pub(super) fn fits(&self, gp: &GroundProgram) -> bool {
        self.size == Self::size_of(gp)
    }
}
