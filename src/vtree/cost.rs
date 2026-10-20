//! The cost model of the `families` layout: the groups each subtree of a
//! layout is open on, and the pairs its joins are charged.

use rustc_hash::FxHashMap;

use crate::ground::AtomId;

use super::forest::{Forest, ShapeNode};

/// The groups of [`super::families::families_forest`] seen from a subtree of its layout: a
/// leaf belongs to the groups `scope` names for it (a switch to its group and
/// the groups its group reads, a copy to the group that defines what it
/// copies), and a subtree is *open* on a group when it holds some of the
/// group's leaves but not all. Pairing a bag's items ([`Interfaces::paired`])
/// charges a join the product of the value counts of the groups its subtree
/// is open on: the states the diagrams there have to tell apart. A whole
/// layout is weighed by [`Interfaces::layout_cost`].
pub(super) struct Interfaces<'a> {
    scope: &'a Scope,
    total: Vec<u32>,
    ln_values: Vec<f64>,
}

/// The groups a subtree is open on, with how many of their leaves it holds.
pub(super) type Open = FxHashMap<usize, u32>;

/// The groups of each atom ([`Interfaces`]), by atom: a slice of one list,
/// empty for an atom of no group.
pub(super) struct Scope {
    at: Vec<(u32, u32)>,
    groups: Vec<usize>,
}

impl Scope {
    pub(super) fn new(num_atoms: usize) -> Self {
        Scope {
            at: vec![(0, 0); num_atoms],
            groups: Vec::new(),
        }
    }

    /// `a`'s groups are `groups`, in place of any it had.
    pub(super) fn set(&mut self, a: AtomId, groups: impl IntoIterator<Item = usize>) {
        let start = self.groups.len();
        self.groups.extend(groups);
        self.at[a as usize] = (start as u32, (self.groups.len() - start) as u32);
    }

    fn of(&self, a: AtomId) -> &[usize] {
        let (start, len) = self.at[a as usize];
        &self.groups[start as usize..(start + len) as usize]
    }
}

/// What [`Interfaces::layout_cost`] found under subtrees it weighed before,
/// by their roots: the charges it added up there, in its order, and the
/// groups the root is open on. The blocks of [`super::families::families_forest`] are under
/// every layout it weighs.
#[derive(Default)]
pub(super) struct Weighed {
    /// The place in `subtrees` of each node's, `u32::MAX` for none.
    index: Vec<u32>,
    subtrees: Vec<(Vec<f64>, Open)>,
}

impl Weighed {
    fn get(&self, node: usize) -> Option<&(Vec<f64>, Open)> {
        self.index
            .get(node)
            .filter(|&&k| k != u32::MAX)
            .map(|&k| &self.subtrees[k as usize])
    }

    fn insert(&mut self, node: usize, subtree: (Vec<f64>, Open)) {
        if self.index.len() <= node {
            self.index.resize(node + 1, u32::MAX);
        }
        self.index[node] = self.subtrees.len() as u32;
        self.subtrees.push(subtree);
    }
}

impl Interfaces<'_> {
    pub(super) fn new<'a>(
        scope: &'a Scope,
        leaves: impl Iterator<Item = AtomId>,
        values: &[f64],
    ) -> Interfaces<'a> {
        let mut total = vec![0u32; values.len()];
        for a in leaves {
            for &g in scope.of(a) {
                total[g] += 1;
            }
        }
        Interfaces {
            scope,
            total,
            ln_values: values.iter().map(|v| v.ln()).collect(),
        }
    }

    fn cost(&self, open: &Open) -> f64 {
        open.keys().map(|&g| self.ln_values[g]).sum()
    }

    pub(super) fn of_leaves(&self, forest: &Forest, node: usize) -> Open {
        let mut counts = Open::default();
        let mut count = |a: AtomId| {
            for &g in self.scope.of(a) {
                *counts.entry(g).or_insert(0) += 1;
            }
        };
        match forest.nodes[node] {
            ShapeNode::Leaf(a) => count(a),
            ShapeNode::Join(..) => forest.leaves_of(node).into_iter().for_each(count),
        }
        counts.retain(|&g, c| *c < self.total[g]);
        counts
    }

    fn join(&self, a: Open, b: Open) -> Open {
        let (mut open, small) = if a.len() >= b.len() { (a, b) } else { (b, a) };
        for (g, c) in small {
            let n = open.get(&g).copied().unwrap_or(0) + c;
            if n < self.total[g] {
                open.insert(g, n);
            } else {
                open.remove(&g);
            }
        }
        open
    }

    /// The logarithm of the join's charge, without building the join.
    fn join_cost(&self, a: &Open, b: &Open) -> f64 {
        let (big, small) = if a.len() >= b.len() { (a, b) } else { (b, a) };
        let mut cost = self.cost(big);
        for (&g, &c) in small {
            match big.get(&g) {
                Some(&d) if d + c >= self.total[g] => cost -= self.ln_values[g],
                Some(_) => {}
                None => cost += self.ln_values[g],
            }
        }
        cost
    }

    /// The pairs the joins under `root` are estimated to hold: each the
    /// product of the value counts of the groups either side is open on.
    /// A node pairs the states of its two sides over the groups they share
    /// as well as those they keep open, so a group that closes at the join
    /// is charged there too. Charged only the groups still open after the
    /// join, a wide min-fill layout of a grid comes out cheaper than the
    /// narrow one its groups' numbering gives, and its evidence is larger.
    ///
    /// The subtrees `weighed` holds are not walked again: their charges are
    /// added in the order the walk would add them, so the sum is the same
    /// to the last bit.
    pub(super) fn layout_cost(&self, forest: &Forest, root: usize, weighed: &Weighed) -> f64 {
        let mut total = 0.0;
        self.walk(forest, root, weighed, |charge| total += charge);
        total
    }

    /// The charges of the joins under `root` in post-order, handed to
    /// `add`, and the groups `root` is open on.
    fn walk(
        &self,
        forest: &Forest,
        root: usize,
        weighed: &Weighed,
        mut add: impl FnMut(f64),
    ) -> Open {
        // Allocated for the subtree's nodes and leaves, which the stack and
        // the open subtrees do not outgrow.
        let leaves = forest.size(root);
        let mut open: Vec<Open> = Vec::with_capacity(leaves);
        let mut stack = Vec::with_capacity(2 * leaves);
        stack.push((root, false));
        while let Some((i, expanded)) = stack.pop() {
            if !expanded && let Some((charges, root_open)) = weighed.get(i) {
                charges.iter().for_each(|&charge| add(charge));
                open.push(root_open.clone());
                continue;
            }
            match forest.nodes[i] {
                ShapeNode::Leaf(_) => open.push(self.of_leaves(forest, i)),
                ShapeNode::Join(l, r) if !expanded => {
                    stack.extend([(i, true), (r, false), (l, false)])
                }
                ShapeNode::Join(..) => {
                    let b = open.pop().expect("right done");
                    let a = open.pop().expect("left done");
                    let either = self.cost(&a)
                        + b.keys()
                            .filter(|g| !a.contains_key(g))
                            .map(|&g| self.ln_values[g])
                            .sum::<f64>();
                    add(either.exp());
                    open.push(self.join(a, b));
                }
            }
        }
        open.pop().expect("the root done")
    }

    /// `weighed` with the subtree under `root` in it.
    pub(super) fn weigh(&self, forest: &Forest, root: usize, weighed: &mut Weighed) {
        // A charge per join under `root`.
        let mut charges = Vec::with_capacity(forest.size(root) - 1);
        let open = self.walk(forest, root, weighed, |charge| charges.push(charge));
        weighed.insert(root, (charges, open));
    }

    /// Join `items` (subtrees with the groups they are open on) into one, the
    /// cheapest pair first: the pair whose join is open on the fewest values,
    /// then the pair with the fewest leaves, then the earliest. A bag's items
    /// in the order the groups are numbered, joined by position, put
    /// together whatever happens to be adjacent, which makes the evidence
    /// phase's largest formula larger. Beyond `PAIRED_UP_TO` items the pairs
    /// are too many to weigh, and the items are joined by position.
    pub(super) fn paired(
        &self,
        forest: &mut Forest,
        items: Vec<(usize, Open)>,
    ) -> Option<(usize, Open)> {
        const PAIRED_UP_TO: usize = 256;
        if items.len() > PAIRED_UP_TO {
            let mid = items.len() / 2;
            let mut items = items;
            let right = items.split_off(mid);
            let (l, a) = self.paired(forest, items)?;
            let (r, b) = self.paired(forest, right)?;
            let open = self.join(a, b);
            return Some((forest.join(l, r), open));
        }
        let key = |cost: f64, leaves: usize| ((cost * 1e9).round() as i64, leaves);
        let mut slots: Vec<Option<(usize, Open, usize)>> = items
            .into_iter()
            .map(|(node, open)| Some((node, open, forest.size(node))))
            .collect();
        let n = slots.len();
        let mut pair = vec![vec![(0i64, 0usize); n]; n];
        for i in 0..n {
            for j in i + 1..n {
                let (a, b) = (slots[i].as_ref().unwrap(), slots[j].as_ref().unwrap());
                pair[i][j] = key(self.join_cost(&a.1, &b.1), a.2 + b.2);
            }
        }
        for _ in 1..n {
            let (mut best, mut at) = ((i64::MAX, usize::MAX), (0, 0));
            for i in 0..n {
                if slots[i].is_none() {
                    continue;
                }
                for j in i + 1..n {
                    if slots[j].is_some() && pair[i][j] < best {
                        (best, at) = (pair[i][j], (i, j));
                    }
                }
            }
            let (i, j) = at;
            let (b, bo, bs) = slots[j].take().expect("a live slot");
            let (a, ao, as_) = slots[i].take().expect("a live slot");
            let merged = (forest.join(a, b), self.join(ao, bo), as_ + bs);
            for k in 0..n {
                if let Some(other) = &slots[k] {
                    let entry = key(self.join_cost(&merged.1, &other.1), merged.2 + other.2);
                    if k < i {
                        pair[k][i] = entry
                    } else {
                        pair[i][k] = entry
                    }
                }
            }
            slots[i] = Some(merged);
        }
        slots
            .into_iter()
            .flatten()
            .next()
            .map(|(node, open, _)| (node, open))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_joins_the_items_that_share_a_group_first() {
        // Leaves 1 and 3 are the two leaves of group 0, leaf 2 the only one
        // of group 1: joining 1 with 3 closes group 0, any other pair leaves
        // it open.
        let mut scope = Scope::new(4);
        scope.set(1, [0]);
        scope.set(2, [1]);
        scope.set(3, [0]);
        let interfaces = Interfaces::new(&scope, [1, 2, 3].into_iter(), &[5.0, 5.0]);
        let mut forest = Forest::default();
        let items: Vec<(usize, Open)> = [1, 2, 3]
            .into_iter()
            .map(|a| {
                let leaf = forest.leaf(a);
                (leaf, interfaces.of_leaves(&forest, leaf))
            })
            .collect();
        let (top, open) = interfaces.paired(&mut forest, items).expect("three items");
        assert!(open.is_empty(), "the whole tree is open on nothing");
        let ShapeNode::Join(left, right) = forest.nodes[top] else {
            panic!("the top is a join")
        };
        assert_eq!(forest.leaves_of(left), vec![1, 3]);
        assert_eq!(forest.leaves_of(right), vec![2]);
        // The join of 1 and 3 pairs its sides over group 0's five values,
        // the top join over nothing.
        let cost = interfaces.layout_cost(&forest, top, &Weighed::default());
        assert!((cost - 6.0).abs() < 1e-9, "{cost}");
        // Weighed once, a subtree adds the same charges in the same order.
        let mut weighed = Weighed::default();
        interfaces.weigh(&forest, left, &mut weighed);
        assert_eq!(
            interfaces.layout_cost(&forest, top, &weighed).to_bits(),
            cost.to_bits()
        );
    }
}
