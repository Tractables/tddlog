//! The forest a layout is built in, over atom ids, and the vtree it
//! finishes as.

use tididi::Vtree;
use tididi::vtree::{VarId, VtreeIdx, VtreeNode};

use crate::ground::AtomId;

/// Where [`Forest::balanced`] splits the items `lo..hi`: the left half
/// is `lo..halves(lo, hi)`.
pub(super) fn halves(lo: usize, hi: usize) -> usize {
    lo + (hi - lo) / 2
}

/// A vtree under construction, over atom ids rather than variable numbers: the
/// numbering is only fixed at the end, from the finished tree's left-to-right
/// leaf order. The nodes live in one arena, so no step here recurses over the
/// tree's depth.
#[derive(Default)]
pub(super) struct Forest {
    pub(super) nodes: Vec<ShapeNode>,
    /// The number of leaves under each node.
    sizes: Vec<usize>,
}

#[derive(Clone, Copy)]
pub(super) enum ShapeNode {
    Leaf(AtomId),
    Join(usize, usize),
}

impl Forest {
    pub(super) fn leaf(&mut self, atom: AtomId) -> usize {
        self.nodes.push(ShapeNode::Leaf(atom));
        self.sizes.push(1);
        self.nodes.len() - 1
    }

    pub(super) fn join(&mut self, left: usize, right: usize) -> usize {
        self.nodes.push(ShapeNode::Join(left, right));
        self.sizes.push(self.sizes[left] + self.sizes[right]);
        self.nodes.len() - 1
    }

    /// The number of leaves under `root`.
    pub(super) fn size(&self, root: usize) -> usize {
        self.sizes[root]
    }

    /// The leaves under `root`, left to right. Both vectors are allocated
    /// for the leaves, more than the stack holds.
    pub(super) fn leaves_of(&self, root: usize) -> Vec<AtomId> {
        let (mut out, mut stack) = (
            Vec::with_capacity(self.sizes[root]),
            Vec::with_capacity(self.sizes[root]),
        );
        stack.push(root);
        while let Some(i) = stack.pop() {
            match self.nodes[i] {
                ShapeNode::Leaf(a) => out.push(a),
                ShapeNode::Join(l, r) => {
                    stack.push(r);
                    stack.push(l);
                }
            }
        }
        out
    }

    /// Combine `items` into one tree by halving the list, which adds depth
    /// logarithmic in their number rather than linear.
    pub(super) fn balanced(&mut self, items: &[usize]) -> Option<usize> {
        self.balanced_with(items, &|_, _| &[])
    }

    /// [`Forest::balanced`], with the atoms `atop(lo, hi)` gives on a spine
    /// ([`Forest::spine`]) above the join of `items[lo..hi]`, for each join
    /// of two or more items short of all of them that the halving makes.
    pub(super) fn balanced_with<'a>(
        &mut self,
        items: &[usize],
        atop: &dyn Fn(usize, usize) -> &'a [AtomId],
    ) -> Option<usize> {
        self.halving(items, 0, items.len(), atop)
    }

    fn halving<'a>(
        &mut self,
        items: &[usize],
        lo: usize,
        hi: usize,
        atop: &dyn Fn(usize, usize) -> &'a [AtomId],
    ) -> Option<usize> {
        match hi - lo {
            0 => None,
            1 => Some(items[lo]),
            _ => {
                let mid = halves(lo, hi);
                let left = self.halving(items, lo, mid, atop).expect("a nonempty half");
                let right = self.halving(items, mid, hi, atop).expect("a nonempty half");
                let joined = self.join(left, right);
                if hi - lo < items.len() {
                    self.spine(atop(lo, hi), Some(joined))
                } else {
                    Some(joined)
                }
            }
        }
    }

    /// `atoms` on a right-linear spine above `below`, the first topmost.
    pub(super) fn spine(&mut self, atoms: &[AtomId], below: Option<usize>) -> Option<usize> {
        let mut acc = below;
        for &atom in atoms.iter().rev() {
            let leaf = self.leaf(atom);
            acc = Some(match acc {
                None => leaf,
                Some(rest) => self.join(leaf, rest),
            });
        }
        acc
    }

    /// The finished vtree, carrying `VarId(i + 1)` on its `i`-th leaf from the
    /// left, and the atom order that numbering realizes.
    pub(super) fn finish(&self, root: usize) -> (Vtree, Vec<AtomId>) {
        enum Step {
            Visit(usize),
            Combine,
        }
        let mut nodes: Vec<VtreeNode> = Vec::with_capacity(2 * self.nodes.len());
        let mut order: Vec<AtomId> = Vec::new();
        let mut built: Vec<VtreeIdx> = Vec::new();
        let mut stack = vec![Step::Visit(root)];
        while let Some(step) = stack.pop() {
            match step {
                Step::Visit(i) => match self.nodes[i] {
                    ShapeNode::Leaf(atom) => {
                        order.push(atom);
                        built.push(VtreeIdx(nodes.len() as u32));
                        nodes.push(VtreeNode::Leaf {
                            var: VarId(order.len() as u32),
                            parent: None,
                        });
                    }
                    ShapeNode::Join(left, right) => {
                        stack.push(Step::Combine);
                        stack.push(Step::Visit(right));
                        stack.push(Step::Visit(left));
                    }
                },
                Step::Combine => {
                    let right = built.pop().expect("a join's right subtree is built");
                    let left = built.pop().expect("a join's left subtree is built");
                    built.push(VtreeIdx(nodes.len() as u32));
                    nodes.push(VtreeNode::Internal {
                        left,
                        right,
                        parent: None,
                    });
                }
            }
        }
        let root = built.pop().expect("the traversal ends on the root");
        let num_vars = order.len() as u32;
        let vtree = Vtree::from_nodes(nodes, root, num_vars)
            .expect("a tree over 1..=n, each variable once");
        (vtree, order)
    }
}
