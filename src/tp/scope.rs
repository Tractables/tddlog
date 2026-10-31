//! The scopes formulas live on.
//!
//! A formula lives in a [`VtreeScope`]: the variables it may mention and
//! the home its diagram is built in. Without projection there is one scope,
//! the whole program. With it, each component has a scope over its ancestor
//! cone, and a formula crossing from one scope into a larger one is
//! [`TddBackend::embed`](super::TddBackend::embed)ed rather than rebuilt.
//! Scopes over the same variables are shared while they live
//! ([`ScopeTable`]), and the clause memo moves formulas between scopes of
//! the same shape ([`scope_shape`]).

use std::sync::{Arc, Weak};

use rustc_hash::FxHashMap;

use tididi::Vtree;
use tididi::vtree::{VarId, VtreeNode};

/// A vtree a formula lives on, with the renaming of the program's variables
/// into it.
///
/// The global scope carries the vtree the strategy built and renames nothing.
/// A projected scope carries the global vtree projected to some of the
/// program's variables ([`Projector`](super::projector::Projector)) and
/// those variables, ascending, so the leaf holding program variable
/// `vars[i]` is `VarId(i + 1)`.
#[derive(Clone)]
pub(crate) struct VtreeScope {
    pub(super) vtree: Arc<Vtree>,
    /// `None` for the global scope, whose renaming is the identity.
    pub(super) vars: Option<Arc<[VarId]>>,
}

impl VtreeScope {
    /// The vtree diagrams on this scope are built over.
    #[must_use]
    pub(crate) fn vtree(&self) -> &Arc<Vtree> {
        &self.vtree
    }

    /// The program variables this scope holds, ascending; `None` when it is
    /// the whole program.
    #[must_use]
    pub(crate) fn vars(&self) -> Option<&[VarId]> {
        self.vars.as_deref()
    }

    /// The leaf `v` sits on in this scope, or `None` when `v` is outside it.
    #[must_use]
    pub(crate) fn local_of(&self, v: VarId) -> Option<VarId> {
        match &self.vars {
            None => Some(v),
            Some(vars) => vars.binary_search(&v).ok().map(|i| VarId(i as u32 + 1)),
        }
    }

    /// The number of program variables in this scope.
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        match &self.vars {
            None => self.vtree.num_leaves() as usize,
            Some(vars) => vars.len(),
        }
    }

    /// Whether two scopes are the same allocation, which is what makes an
    /// embed between them free.
    #[must_use]
    pub(crate) fn same(&self, other: &VtreeScope) -> bool {
        Arc::ptr_eq(&self.vtree, &other.vtree)
    }
}

/// Scopes by their variables, so that formulas over the same variables share
/// one vtree allocation, which is what makes an embed between them free: the
/// components' cones under projection, and the scopes of the query phase,
/// which starts from the compile's.
///
/// A scope is held weakly: once no formula or caller holds it, its vtree
/// goes, and the same variables later get a projection built afresh, node
/// for node the one that went. Two formulas meet on a scope only while both
/// live, and then it is alive, so the scopes they share are the ones a table
/// holding every scope would give. The cones of a bit-blasted circuit, held
/// to the end of its compile, filled its memory, and the evidence walk makes
/// a scope for nearly every join, which the next join over its result
/// replaces.
#[derive(Default)]
pub(crate) struct ScopeTable {
    by_vars: FxHashMap<Arc<[VarId]>, Weak<Vtree>>,
    /// The number of entries at which those whose scope has gone are
    /// dropped: twice the entries left by the last time, so that each
    /// insertion pays for a constant number of entries looked at.
    purge_at: usize,
}

impl ScopeTable {
    /// The live scope over `vars`, ascending and distinct, if there is one.
    pub(crate) fn get(&self, vars: &[VarId]) -> Option<VtreeScope> {
        let (vars, vtree) = self.by_vars.get_key_value(vars)?;
        Some(VtreeScope {
            vtree: vtree.upgrade()?,
            vars: Some(Arc::clone(vars)),
        })
    }

    /// Hold `scope` as the scope over its variables; the global scope is
    /// not held.
    pub(crate) fn insert(&mut self, scope: &VtreeScope) {
        let Some(vars) = &scope.vars else { return };
        if self.by_vars.len() >= self.purge_at {
            self.by_vars.retain(|_, vtree| vtree.strong_count() > 0);
            self.purge_at = (2 * self.by_vars.len()).max(64);
        }
        self.by_vars
            .insert(Arc::clone(vars), Arc::downgrade(&scope.vtree));
    }
}

/// A hash of the shape of `vtree` up to mirrors, local variable `v` at a
/// leaf read as its slot, `slots[v - 1]`; `hashes` holds one per node.
///
/// [`super::TddBackend::transport`] from a scope with slots `from_slots` onto one
/// with slots `slots`, by the map the clause memo makes, which sends local
/// variable `k + 1` to the variable in slot `from_slots[k]`, succeeds only
/// where the two hash the same, whatever the trees:
///
/// - The leaves it pairs are in one slot. The map sends `k + 1` to a
///   variable in slot `from_slots[k]` or, where no variable is in that
///   slot, to none, which the embed refuses.
/// - No level of the destination is free, so there is no pass-through. The
///   embed refuses a map that sends two variables to one, and both trees
///   have one leaf for each of the same number of variables, so every leaf
///   of the destination is an image.
/// - Its only other freedom is a mirror at any node. It pairs each source
///   node with a destination node whose children are the images of its
///   children, in one order or the other, and a leaf with its image.
///
/// A leaf's hash is a function of its slot, and an internal node's of the
/// two hashes of its children, the smaller first. So from the leaves up,
/// each paired destination node hashes as its source node does, and the
/// roots agree. Two shapes that differ can still hash the same, which
/// costs a transport that fails.
pub(crate) fn scope_shape(vtree: &Vtree, slots: &[u32], hashes: &mut Vec<u64>) -> u64 {
    // The finalizer of SplitMix64.
    fn mix(mut x: u64) -> u64 {
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^ (x >> 31)
    }
    hashes.clear();
    hashes.resize(vtree.num_nodes(), 0);
    for t in vtree.bottomup() {
        hashes[t.idx()] = match vtree.node(t) {
            VtreeNode::Leaf { var, .. } => mix(u64::from(slots[var.0 as usize - 1])),
            VtreeNode::Internal { left, right, .. } => {
                let (a, b) = (hashes[left.idx()], hashes[right.idx()]);
                mix(a.min(b) ^ mix(a.max(b) ^ 0x9e37_79b9_7f4a_7c15))
            }
        };
    }
    hashes[vtree.root().idx()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tp::TddBackend;
    use tididi::vtree::VtreeIdx;

    /// A table gives back a scope over the same variables as the one it
    /// was handed, the same allocation, for as long as something holds that
    /// scope, and nothing once nothing does; the scope built afresh for
    /// those variables is the one that went, node for node. Entries whose
    /// scope has gone are dropped as the table grows.
    #[test]
    fn a_scope_table_holds_scopes_while_they_live() {
        let tree = Arc::new(Vtree::random(40, 7));
        tree.context().run(|engine| {
            let backend = TddBackend::new(engine, tree.clone());
            let mut table = ScopeTable::default();
            let vars: Vec<VarId> = [3, 8, 9, 21, 34].map(VarId).to_vec();
            let scope = backend.scope(&vars).unwrap();
            table.insert(&scope);
            let found = table.get(&vars).expect("the scope lives");
            assert!(found.same(&scope));
            assert!(table.get(&vars[1..]).is_none());
            table.insert(&backend.global());
            assert_eq!(table.by_vars.len(), 1, "the global scope is not held");
            let nodes: Vec<VtreeNode> = (0..scope.vtree().num_nodes())
                .map(|t| scope.vtree().node(VtreeIdx(t as u32)).clone())
                .collect();
            drop((scope, found));
            assert!(table.get(&vars).is_none(), "nothing holds the scope");
            let again = backend.scope(&vars).unwrap();
            for (t, node) in nodes.iter().enumerate() {
                assert_eq!(again.vtree().node(VtreeIdx(t as u32)), node);
            }
            table.insert(&again);

            // Scopes made and let go, one over each pair of variables, with
            // `again` held throughout.
            for v in 1..40 {
                for w in v + 1..=40 {
                    let gone = backend.scope(&[VarId(v), VarId(w)]).unwrap();
                    table.insert(&gone);
                    assert!(table.by_vars.len() <= 64, "{} entries", table.by_vars.len());
                }
            }
            assert!(table.get(&vars).unwrap().same(&again));
            assert!(table.get(&[VarId(39), VarId(40)]).is_none());
        });
    }
}
