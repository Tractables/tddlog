//! The fold of a rooted forest of units into a vtree shape.
//!
//! A layout's *units* are sets of facts laid out together, each fact owned
//! by one unit, and a rooted forest over them ([`UnitTree`]). The units'
//! graph and the order the layout eliminates them in give the forest (the
//! elimination tree); `decision`'s comes from the grounder's decisions. A
//! unit and its children are a bag of a tree decomposition and the bags
//! below it (`td`, `dynamic`), a block of components and the blocks its
//! rules read (`closed`), a node of the grounder's decisions and its two
//! halves (`decision`). The fold builds each unit's node from its
//! children's nodes, bottom up, and joins the roots' nodes. Every layout
//! with a tree but `families` uses one node rule ([`stacked`]); `families`
//! pairs a bag's blocks and its children by their interfaces instead.

use rustc_hash::FxHashMap;

use crate::ground::AtomId;

use super::forest::Forest;

/// A rooted forest of units: the elimination tree of their graph under an
/// order, or the grounder's decisions.
#[derive(Clone, Default)]
pub(super) struct UnitTree {
    /// Every unit, each after its children.
    pub(super) order: Vec<usize>,
    /// Each unit's children, in the order their nodes are joined.
    pub(super) children: Vec<Vec<usize>>,
    /// The roots, in the order their nodes are joined.
    pub(super) roots: Vec<usize>,
}

/// The fold of `tree`: each unit's node, made by `node` from the unit and
/// its children's nodes (those that have one), children first; the roots'
/// nodes.
pub(super) fn fold<N>(tree: &UnitTree, mut node: impl FnMut(usize, Vec<N>) -> Option<N>) -> Vec<N> {
    let mut made: Vec<Option<N>> = tree.children.iter().map(|_| None).collect();
    for &u in &tree.order {
        let below = tree.children[u]
            .iter()
            .filter_map(|&c| made[c].take())
            .collect();
        made[u] = node(u, below);
    }
    tree.roots.iter().filter_map(|&r| made[r].take()).collect()
}

/// The fold of every layout with a tree but `families`: a unit's own facts
/// `own[u]` on a right-linear spine, the first topmost, above the balanced
/// join of its children's nodes, and the atoms `atop` gives above that; the
/// roots' nodes balanced. A unit's own facts sit on a spine rather than in
/// the balanced join: the compile tests them one after another, and a spine
/// lets a diagram finish with one before it reaches the next. `None` when
/// no unit has a fact.
pub(super) fn stacked(
    tree: &UnitTree,
    own: &[Vec<AtomId>],
    atop: &Atop,
) -> Option<(Forest, usize)> {
    let mut forest = Forest::default();
    let tops = fold(tree, |u, below: Vec<usize>| {
        let joined = forest.balanced_with(&below, &|lo, hi| atop.join(u, lo, hi));
        let spine = forest.spine(&own[u], joined);
        forest.spine(atop.unit(u), spine)
    });
    let root = forest.balanced(&tops)?;
    Some((forest, root))
}

/// Atoms laid atop a fold's nodes ([`stacked`]): above a unit's node, or
/// above the join of some of a unit's children. They are names, each
/// standing for an atom decided elsewhere: `td`'s separator copies
/// ([`super::separators`]), `decision`'s contexts and cut atoms.
#[derive(Default)]
pub(super) struct Atop {
    /// By the unit.
    pub(super) units: FxHashMap<usize, Vec<AtomId>>,
    /// By the unit and the range `lo..hi` of its children that have a node.
    pub(super) joins: FxHashMap<(usize, usize, usize), Vec<AtomId>>,
}

impl Atop {
    /// The atoms above unit `u`'s node, the first topmost.
    pub(super) fn unit(&self, u: usize) -> &[AtomId] {
        self.units.get(&u).map_or(&[], Vec::as_slice)
    }

    /// The atoms above the join of unit `u`'s children with nodes `lo..hi`,
    /// the first topmost.
    pub(super) fn join(&self, u: usize, lo: usize, hi: usize) -> &[AtomId] {
        self.joins.get(&(u, lo, hi)).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vtree::forest::ShapeNode;

    fn text(forest: &Forest, at: usize) -> String {
        match forest.nodes[at] {
            ShapeNode::Leaf(a) => a.to_string(),
            ShapeNode::Join(l, r) => format!("({} {})", text(forest, l), text(forest, r)),
        }
    }

    /// Two leaf units under a third, a name atop the third, and a root of
    /// its own beside it: the third's facts on a spine above the join of
    /// the two, the name above that, the two roots joined.
    #[test]
    fn stacked_lays_facts_on_a_spine_above_the_children_and_names_above_that() {
        let tree = UnitTree {
            order: vec![0, 1, 2, 3],
            children: vec![vec![], vec![], vec![0, 1], vec![]],
            roots: vec![2, 3],
        };
        let own = [vec![1], vec![2], vec![3, 4], vec![5]];
        let mut atop = Atop::default();
        atop.units.insert(2, vec![9]);
        let (forest, root) = stacked(&tree, &own, &atop).expect("facts");
        assert_eq!(text(&forest, root), "((9 (3 (4 (1 2)))) 5)");
        // A unit with no fact and no child has no node.
        let none = stacked(&tree, &[vec![], vec![], vec![], vec![]], &Atop::default());
        assert!(none.is_none());
    }
}
