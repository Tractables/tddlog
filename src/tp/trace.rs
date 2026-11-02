//! What a compile records for a trace of its run ([`CompileTrace`]), where
//! its caller asked for one (`crate::Options::trace`): each component in the
//! order it compiled, its atoms, the rounds of its fixpoint, the pairs of
//! each formula an update made, and the operations and work it took. The
//! lists are bounded, so that a large program's trace stays small: past
//! them only counts are kept. A compile asked for none records nothing,
//! and a record reads no diagram: the pairs are those the compile counts
//! for its own bounds.

use crate::ground::AtomId;

/// The components a trace describes one by one; those after them are
/// counted together ([`CompileTrace::rest`]).
pub const COMPONENTS: usize = 64;
/// The atoms of a component a trace names.
pub const ATOMS: usize = 8;
/// The updates of a component a trace gives the pairs of.
pub const UPDATES: usize = 64;

/// One component's compile.
#[derive(Clone, Debug, Default)]
pub struct ComponentTrace {
    /// Its first [`ATOMS`] atoms, in the order the compile holds them.
    pub atoms: Vec<AtomId>,
    /// How many atoms it has.
    pub size: usize,
    /// Whether it reads its own atoms, and so was iterated to a fixpoint.
    pub cyclic: bool,
    /// Whether it reads its own atoms through negation, and so was
    /// compiled to its well-founded model (`aft`).
    pub negation: bool,
    /// The rounds of its fixpoint, 0 for a component that is not cyclic.
    pub rounds: usize,
    /// Its atom updates, and the pairs of the largest formula one made.
    pub updates: usize,
    pub largest: usize,
    /// The first [`UPDATES`] of them: the atom, the pairs of the formula it
    /// made, and the round, 0 outside a fixpoint. An update of a fixpoint
    /// that left its formula as it was is one whose pairs are `None`.
    pub sizes: Vec<(AtomId, Option<usize>, usize)>,
    /// The operations it ran, by kind: conjunctions, disjunctions,
    /// minimizations, equivalence tests and negations.
    pub operations: [usize; 5],
    /// The engine's work units it took.
    pub work_units: u64,
}

/// The components after the first [`COMPONENTS`], counted together.
#[derive(Clone, Copy, Debug, Default)]
pub struct RestTrace {
    pub components: usize,
    pub cyclic: usize,
    pub updates: usize,
    /// The pairs of the largest formula an update of theirs made.
    pub largest: usize,
    pub operations: [usize; 5],
    pub work_units: u64,
}

/// A compile's record for a trace.
#[derive(Clone, Debug, Default)]
pub struct CompileTrace {
    /// The fact atom of each variable of the vtree, by variable, from 1;
    /// `AtomId::MAX` for a variable of none.
    pub variables: Vec<AtomId>,
    /// The first [`COMPONENTS`] components in the order they compiled.
    pub components: Vec<ComponentTrace>,
    pub rest: RestTrace,
    /// The component under way, with the operations and work counted at
    /// its start.
    open: Option<(ComponentTrace, [usize; 5], u64)>,
}

impl CompileTrace {
    /// A record of a compile whose facts have the variables `var_of`.
    pub(crate) fn new(var_of: &[Option<tididi::vtree::VarId>]) -> Self {
        let num_vars = var_of.iter().flatten().map(|v| v.0 as usize).max();
        let mut variables = vec![AtomId::MAX; num_vars.unwrap_or(0)];
        for (a, v) in var_of.iter().enumerate() {
            if let Some(v) = v {
                variables[v.0 as usize - 1] = a as AtomId;
            }
        }
        CompileTrace {
            variables,
            ..CompileTrace::default()
        }
    }

    /// The component of `atoms` begins, or goes on where an earlier run
    /// stopped it, with the compile's operations and work so far.
    pub(crate) fn start(&mut self, atoms: &[AtomId], operations: [usize; 5], work: u64) {
        if self.open.is_none() {
            let component = ComponentTrace {
                atoms: atoms.iter().take(ATOMS).copied().collect(),
                size: atoms.len(),
                ..ComponentTrace::default()
            };
            self.open = Some((component, operations, work));
        }
    }

    /// The component under way starts over: an earlier run stopped it, and
    /// the next builds it again from its start.
    pub(crate) fn restart(&mut self) {
        self.open = None;
    }

    /// The component under way updated `atom` to a formula of `pairs`, or
    /// left it as it was (`None`), in its fixpoint's round `round`, 0
    /// outside one.
    pub(crate) fn update(&mut self, atom: AtomId, pairs: Option<usize>, round: usize) {
        if let Some((c, _, _)) = &mut self.open {
            c.updates += 1;
            c.largest = c.largest.max(pairs.unwrap_or(0));
            if c.sizes.len() < UPDATES {
                c.sizes.push((atom, pairs, round));
            }
            c.rounds = c.rounds.max(round);
        }
    }

    /// The component under way is cyclic: iterated to a fixpoint, or where
    /// `negation`, compiled to its well-founded model in `updates` updates
    /// besides the one of each atom's last formula, which [`Self::update`]
    /// records.
    pub(crate) fn cyclic(&mut self, negation: bool, updates: usize) {
        if let Some((c, _, _)) = &mut self.open {
            c.cyclic = true;
            c.negation |= negation;
            c.updates += updates;
        }
    }

    /// The component under way is finished, with the compile's operations
    /// and work so far.
    pub(crate) fn finish(&mut self, operations: [usize; 5], work: u64) {
        let Some((mut c, ops, from)) = self.open.take() else {
            return;
        };
        for (k, n) in c.operations.iter_mut().enumerate() {
            *n = operations[k] - ops[k];
        }
        c.work_units = work - from;
        if self.components.len() < COMPONENTS {
            self.components.push(c);
            return;
        }
        let r = &mut self.rest;
        r.components += 1;
        r.cyclic += usize::from(c.cyclic);
        r.updates += c.updates;
        r.largest = r.largest.max(c.largest);
        for (k, n) in r.operations.iter_mut().enumerate() {
            *n += c.operations[k];
        }
        r.work_units += c.work_units;
    }
}
