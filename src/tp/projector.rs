//! Projection of the program's vtree onto a set of its variables.

use std::sync::Arc;

use tididi::Vtree;
use tididi::vtree::{VarId, VtreeIdx};

use super::CompileError;

/// Projection of the program's vtree onto a set of its variables in time
/// proportional to the set: the projected tree's nodes are the leaves of
/// the set and the lowest common ancestors of leaves adjacent in
/// pre-order, and the pre-order numbering with a range-minimum structure
/// over the depths answers an ancestor in constant time. Built once over
/// the program's vtree; `Vtree::project_to_vars` walks every node of it per
/// scope, which the evidence walk, with many small scopes, cannot afford.
///
/// Every query is between two leaves, neither above the other, so their
/// lowest common ancestor is the parent of the shallowest node after the
/// first up to the second. Each pre-order number holds its node's depth
/// and its parent's number in one word, depth high, so that the least word
/// of a range is a shallowest node's and names the ancestor itself: a
/// query reads the words and nothing else. The minima are kept per block
/// of [`BLOCK`] numbers, with a sparse table over the blocks, and a query
/// scans the parts of the two blocks at its ends: a sparse table over
/// every number takes the vtree's size times its height to build.
pub(super) struct Projector {
    /// The pre-order number of each variable's leaf, by variable id;
    /// `u32::MAX` for an id without a leaf.
    leaf_pre: Vec<u32>,
    /// Node by pre-order number.
    node: Vec<VtreeIdx>,
    /// By pre-order number, `depth << 32 | parent`: the depth of the node
    /// and the pre-order number of its parent (`u32::MAX` for the root).
    up: Vec<u64>,
    /// `table[k * blocks + i]`: the least word of blocks `i..i + 2^k`.
    table: Vec<u64>,
    blocks: usize,
    /// [`Projector::project`]'s leaves, their local variables in pre-order
    /// and their neighbours' ancestors' depths, kept between calls.
    leaves: Vec<(u32, VarId)>,
    locals: Vec<VarId>,
    depths: Vec<u32>,
}

/// The numbers per block of [`Projector`]'s range minimum.
const BLOCK: usize = 16;

impl Projector {
    pub(super) fn new(vtree: &Vtree) -> Self {
        let n = vtree.num_nodes();
        let mut leaf_pre = vec![u32::MAX; vtree.num_vars() as usize + 1];
        let (mut node, mut up) = (Vec::with_capacity(n), Vec::with_capacity(n));
        let mut stack = vec![(vtree.root(), 0u64, u32::MAX)];
        while let Some((t, depth, parent)) = stack.pop() {
            let pre = node.len() as u32;
            node.push(t);
            up.push(depth << 32 | u64::from(parent));
            if vtree.node(t).is_leaf() {
                leaf_pre[vtree.leaf_var(t).0 as usize] = pre;
            } else {
                let (l, r) = vtree.children(t);
                stack.push((r, depth + 1, pre));
                stack.push((l, depth + 1, pre));
            }
        }
        let blocks = n.div_ceil(BLOCK);
        let mut table: Vec<u64> = up
            .chunks(BLOCK)
            .map(|c| c.iter().copied().fold(u64::MAX, u64::min))
            .collect();
        let mut width = 1;
        while 2 * width <= blocks {
            let row = table.len() - blocks;
            for i in 0..blocks {
                let least = if i + 2 * width <= blocks {
                    table[row + i].min(table[row + i + width])
                } else {
                    u64::MAX
                };
                table.push(least);
            }
            width *= 2;
        }
        Projector {
            leaf_pre,
            node,
            up,
            table,
            blocks,
            leaves: Vec::new(),
            locals: Vec::new(),
            depths: Vec::new(),
        }
    }

    /// The least word ([`Projector::up`]) of the pre-order numbers `lo..=hi`.
    fn least(&self, lo: usize, hi: usize) -> u64 {
        let scan =
            |from: usize, to: usize| self.up[from..to].iter().copied().fold(u64::MAX, u64::min);
        let (first, last) = (lo / BLOCK, hi / BLOCK);
        if first == last {
            return scan(lo, hi + 1);
        }
        let ends = scan(lo, (first + 1) * BLOCK).min(scan(last * BLOCK, hi + 1));
        if first + 1 == last {
            return ends;
        }
        let (i, j) = (first + 1, last - 1);
        let k = (usize::BITS - 1 - (j - i + 1).leading_zeros()) as usize;
        let row = k * self.blocks;
        ends.min(self.table[row + i])
            .min(self.table[row + j + 1 - (1 << k)])
    }

    /// The pre-order number of `v`'s leaf, `None` where `v` has none.
    fn leaf(&self, v: VarId) -> Option<u32> {
        self.leaf_pre
            .get(v.0 as usize)
            .copied()
            .filter(|&p| p != u32::MAX)
    }

    /// The lowest node holding every variable of `vars`, which is not empty.
    pub(super) fn lowest_over(&self, vars: &[VarId]) -> VtreeIdx {
        let (lo, hi) = vars
            .iter()
            .map(|&v| self.leaf(v).expect("a scope's variables are the program's") as usize)
            .fold((usize::MAX, 0), |(lo, hi), p| (lo.min(p), hi.max(p)));
        assert!(lo <= hi, "a scope holds a variable");
        if lo == hi {
            return self.node[lo];
        }
        self.node[self.least(lo + 1, hi) as u32 as usize]
    }

    /// The vtree over `vars`, ascending and distinct, with local ids
    /// `1..=vars.len()` in that order: the projection of `vtree` onto them.
    ///
    /// Its nodes are the leaves and the lowest common ancestors of leaves
    /// adjacent in pre-order, and the depth in `vtree` of each such ancestor
    /// places it: `Vtree::from_in_order` builds the tree from the leaves in
    /// that order and those depths, with no node list to check.
    pub(super) fn project(&mut self, vtree: &Vtree, vars: &[VarId]) -> Result<Vtree, CompileError> {
        // The leaves in pre-order, each with its local variable; the
        // variables are numbered in the vtree's leaf order, so these come
        // sorted but for a vtree searched since.
        let mut leaves = std::mem::take(&mut self.leaves);
        leaves.clear();
        let mut sorted = true;
        for (i, &v) in vars.iter().enumerate() {
            let p = self.leaf(v).ok_or(CompileError::NotInScope(v))?;
            sorted &= leaves.last().is_none_or(|&(q, _)| q < p);
            leaves.push((p, VarId(i as u32 + 1)));
        }
        if !sorted {
            leaves.sort_unstable();
        }
        let (mut locals, mut depths) = (
            std::mem::take(&mut self.locals),
            std::mem::take(&mut self.depths),
        );
        locals.clear();
        depths.clear();
        locals.extend(leaves.iter().map(|&(_, var)| var));
        // The ancestor of two leaves neither above the other is the parent
        // of the shallowest node after the first up to the second.
        depths.extend(
            leaves
                .windows(2)
                .map(|w| ((self.least(w[0].0 as usize + 1, w[1].0 as usize) >> 32) - 1) as u32),
        );
        let projected = Vtree::from_in_order(&locals, &depths, vars.len() as u32)
            .expect("the leaves carry the local variables, each once");
        self.leaves = leaves;
        self.locals = locals;
        self.depths = depths;
        Ok(projected.with_context(Arc::clone(vtree.context())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The projection built from lowest common ancestors is the one
    /// `Vtree::project_to_vars` builds by walking every node, node for
    /// node, on random trees and random variable sets of every size, and
    /// its lowest node over a set is the library's lowest common ancestor.
    /// The last trees span blocks enough for every row of the sparse table
    /// to answer.
    #[test]
    fn the_projector_agrees_with_the_library_projection() {
        for seed in 0..46u64 {
            let n = 1 + (seed % 23) as u32 * 3 + if seed >= 40 { 40 * seed as u32 } else { 0 };
            let vtree = Vtree::random(n, seed);
            let mut projector = Projector::new(&vtree);
            let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            for _ in 0..12 {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let vars: Vec<VarId> = (1..=n)
                    .filter(|v| (state >> (v % 61)) & 1 == 1)
                    .map(VarId)
                    .collect();
                if vars.is_empty() {
                    continue;
                }
                let mut local = vec![None; n as usize + 1];
                for (i, v) in vars.iter().enumerate() {
                    local[v.0 as usize] = Some(VarId(i as u32 + 1));
                }
                let folded = vars
                    .iter()
                    .map(|&v| vtree.leaf_of(v).unwrap())
                    .reduce(|x, y| vtree.lca(x, y))
                    .unwrap();
                assert_eq!(
                    projector.lowest_over(&vars),
                    folded,
                    "seed {seed} vars {vars:?}"
                );
                let expected = vtree
                    .project_to_vars(|v| local[v.0 as usize], vars.len() as u32)
                    .unwrap();
                let got = projector.project(&vtree, &vars).unwrap();
                assert_eq!(
                    got.num_nodes(),
                    expected.num_nodes(),
                    "seed {seed} vars {vars:?}"
                );
                for t in 0..got.num_nodes() {
                    assert_eq!(
                        got.node(VtreeIdx(t as u32)),
                        expected.node(VtreeIdx(t as u32)),
                        "seed {seed} vars {vars:?} node {t}"
                    );
                }
            }
        }
    }
}
