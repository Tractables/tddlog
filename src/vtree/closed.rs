//! The `closed` layout, bushy over the program's dependencies.

use crate::disjoint::find;
use crate::ground::deps::Deps;
use crate::ground::{AtomId, GroundProgram};

use super::forest::Forest;

/// The `closed` vtree: bushy over the program's dependencies.
///
/// Every formula tddlog stores is over facts alone, the derived atoms it
/// reads already composed away. At a vtree node over the facts `F`, a
/// formula's TDD keeps one node per class of assignments to `F` that leaves
/// the rest of the formula the same. When the atoms that read `F` also read
/// atoms decided outside `F`, a class is a whole function from those outside
/// values to the values inside, so the count is exponential in the number of
/// such functions: doubly exponential in the atoms read across the cut the
/// wrong way. When nothing inside `F` reads an atom from outside it, a class
/// is an assignment to the atoms `F` decides for its readers, and the count
/// is singly exponential in them. A right-linear vtree over the reverse of a
/// dependency order has only subtrees of the second kind, and so does this
/// one; a vtree from a tree decomposition of the undirected interaction
/// graph, or of the Clark completion, does not know the direction and has
/// subtrees of the first kind (`docs/design.md`).
///
/// The components of [`super::order::search`] are taken in its order. Each is a
/// *block*: its facts not yet placed, on a right-linear spine whose
/// first-used fact is deepest, above a balanced join of the blocks its rules
/// read. A block read by a later component becomes part of that
/// component's block, so a block read by two components is absorbed by the
/// first and the second reads the first's whole block. Blocks read by one
/// component and sharing nothing are siblings, and that is the only place
/// the tree branches: on a program where every component reads the one
/// before it this is the right-linear vtree over the reverse of the
/// search's first-use order. The blocks nothing reads, and the facts no
/// rule reads, are joined balanced at the root.
///
/// A join is not taken where the component's rules pair the blocks' atoms
/// up rather than read them in combination. A TDD node at the join holds a
/// pair for every combination of the two sides' classes, whatever the
/// formula above does with them. The rows of a Bayesian network's table read
/// every combination of the parents' values, so the product is paid in any
/// vtree and the join costs nothing extra; that is where `closed` wins.
/// UW-CSE's `aux1_prof_nb_publications(P) :-
/// advised_by(S, P), student_nb_publications(S, three_or_more)` reads the
/// i-th atom of one block with the i-th of the other: an OR of pairwise
/// ANDs, exponential across any cut that puts the two sides apart. A
/// pairing is at least two distinct combinations of read atoms among the
/// rules that read two blocks or more, no atom in two of them. The
/// component's block is then right-linear over its facts in first-use
/// order. Fewer combinations than the product of the atoms read per block
/// is not enough to call a pairing: a deterministic table lists only the
/// rows that give its head (bn_link's pedigree tables do not finish
/// linearized). One combination is a single conjunction across the join,
/// cheap on any vtree, and none means the rules read one block each, a
/// disjunction of per-block functions, which is `dice_noisyOr` and the cora
/// family, where the join wins.
///
/// Returns `None` when the program has no facts.
pub(super) fn closed_forest(gp: &GroundProgram, deps: &Deps) -> Option<(Forest, usize, String)> {
    gp.facts().next()?;
    let by_head = &deps.by_head;
    let (components, search_note) = super::order::search(gp, by_head, deps.sccs.clone());
    let num_atoms = gp.num_atoms();
    let mut component_of = vec![usize::MAX; num_atoms];
    for (i, c) in components.iter().enumerate() {
        for &a in c {
            component_of[a as usize] = i;
        }
    }
    let mut forest = Forest::default();
    // Union-find over blocks, and the forest node of each block's tree,
    // current at the roots.
    let mut parent: Vec<usize> = Vec::new();
    let mut tree: Vec<usize> = Vec::new();
    let mut size: Vec<usize> = Vec::new();
    let mut block_of = vec![usize::MAX; num_atoms];
    let mut fresh = vec![false; num_atoms];
    // Each fact's place in first-use order, which a linearized block follows.
    let mut first_use = vec![usize::MAX; num_atoms];
    let mut used = 0usize;
    let (mut joins, mut widest, mut linearized) = (0usize, 0usize, 0usize);
    // The join whose second-largest side has the most facts, the one a
    // product of class counts is likeliest to hurt: its component's first
    // atom and the facts of each side.
    let mut largest: Option<(AtomId, Vec<usize>)> = None;
    for (ci, comp) in components.iter().enumerate() {
        let mut reads: Vec<usize> = Vec::new();
        let mut new: Vec<AtomId> = Vec::new();
        // The (block, atom) reads of each rule that reads two blocks or more,
        // distinct.
        let mut combinations: Vec<Vec<(usize, AtomId)>> = Vec::new();
        for &h in comp {
            for &ri in &by_head[h as usize] {
                let mut rule_reads: Vec<(usize, AtomId)> = Vec::new();
                for l in &gp.rules[ri].body {
                    let a = l.atom();
                    let ai = a as usize;
                    if block_of[ai] != usize::MAX && (gp.is_fact(a) || component_of[ai] != ci) {
                        let b = find(&mut parent, block_of[ai]);
                        if !reads.contains(&b) {
                            reads.push(b);
                        }
                        rule_reads.push((b, a));
                    } else if gp.is_fact(a) && !fresh[ai] {
                        fresh[ai] = true;
                        first_use[ai] = used;
                        used += 1;
                        new.push(a);
                    }
                }
                rule_reads.sort_unstable();
                rule_reads.dedup();
                if rule_reads.first().map(|r| r.0) != rule_reads.last().map(|r| r.0)
                    && !combinations.contains(&rule_reads)
                {
                    combinations.push(rule_reads);
                }
            }
        }
        let mut paired: Vec<(usize, AtomId)> = combinations.iter().flatten().copied().collect();
        paired.sort_unstable();
        let disjoint = paired.windows(2).all(|w| w[0] != w[1]);
        let pairing = reads.len() > 1 && combinations.len() >= 2 && disjoint;
        let items: Vec<usize> = reads.iter().map(|&b| tree[b]).collect();
        let block_size = new.len() + reads.iter().map(|&r| size[r]).sum::<usize>();
        let mut acc = forest.balanced(&items);
        if pairing {
            linearized += 1;
            let mut all: Vec<AtomId> = items.iter().flat_map(|&t| forest.leaves_of(t)).collect();
            all.extend(new.iter().copied());
            all.sort_unstable_by_key(|&a| first_use[a as usize]);
            acc = None;
            new = all;
        }
        for &f in &new {
            let leaf = forest.leaf(f);
            acc = Some(match acc {
                None => leaf,
                Some(rest) => forest.join(leaf, rest),
            });
        }
        let Some(acc) = acc else { continue };
        if reads.len() > 1 && !pairing {
            joins += 1;
            widest = widest.max(reads.len());
            let mut sides: Vec<usize> = reads.iter().map(|&b| size[b]).collect();
            sides.sort_unstable_by(|x, y| y.cmp(x));
            if largest.as_ref().is_none_or(|(_, best)| sides[1] > best[1]) {
                largest = Some((comp[0], sides));
            }
        }
        let b = parent.len();
        parent.push(b);
        tree.push(acc);
        size.push(block_size);
        for &r in &reads {
            parent[r] = b;
        }
        for &a in new.iter().chain(comp.iter()) {
            block_of[a as usize] = b;
        }
    }
    let mut tops: Vec<usize> = (0..parent.len())
        .filter(|&b| parent[b] == b)
        .map(|b| tree[b])
        .collect();
    let unread = tops.len();
    let stray: Vec<AtomId> = gp
        .facts()
        .map(|(a, _)| a)
        .filter(|&a| block_of[a as usize] == usize::MAX)
        .collect();
    for a in stray {
        tops.push(forest.leaf(a));
    }
    let root = forest.balanced(&tops)?;
    let mut note = format!(
        "{search_note}\nclosed: {} components, {joins} of them joining two or more blocks (at most {widest}), \
         {unread} blocks at the root, {linearized} pairing joins made right-linear",
        components.len(),
    );
    if let Some((a, sides)) = largest {
        let sides: Vec<String> = sides.iter().map(|n| n.to_string()).collect();
        note += &format!(
            "; the largest join at {}, of blocks of {} facts",
            gp.atoms[a as usize],
            sides.join(" + ")
        );
    }
    Some((forest, root, note))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::shape::nothing_to_order;
    use crate::ground::{GroundOptions, ground};
    use crate::syntax::parse_program;
    use crate::vtree::graph::facts_at_first_use;
    use crate::vtree::{Kind, Layout, build};
    use tididi::Vtree;
    use tididi::vtree::{VarId, VtreeIdx, VtreeNode};

    /// `src` ground with its derived atoms as written, each its own
    /// component (`GroundOptions::views` and `contexts` off): the shapes
    /// the layout is tested on.
    fn as_written(src: &str) -> GroundProgram {
        ground(
            &parse_program(src).expect("parses"),
            &GroundOptions {
                views: false,
                contexts: false,
                ..GroundOptions::default()
            },
        )
        .expect("grounds")
    }

    /// The right-linear vtree over the reverse of the order in which the
    /// compiler consumes the facts when it takes the components in the
    /// searched order `closed` starts from: what `closed` builds where it
    /// finds nothing to join.
    fn searched_right_linear(gp: &GroundProgram) -> (String, Vec<AtomId>) {
        let by_head = gp.rules_by_head();
        let sccs = crate::ground::deps::derived_sccs(gp, &by_head);
        let (components, _) = crate::vtree::order::search(gp, &by_head, sccs);
        let mut order = facts_at_first_use(gp, &by_head, &components);
        order.reverse();
        let vars: Vec<VarId> = (1..=order.len() as u32).map(VarId).collect();
        (
            Vtree::linear_from_order(&vars)
                .expect("nonempty order")
                .to_text(),
            order,
        )
    }

    /// `closed` branches where two parts of the program share nothing until
    /// a later rule reads both, and nowhere else.
    #[test]
    fn closed_branches_only_where_the_dependencies_do() {
        let facts_under =
            |vtree: &Vtree, order: &[AtomId], idx: VtreeIdx, gp: &GroundProgram| -> Vec<String> {
                let mut out = Vec::new();
                let mut stack = vec![idx];
                while let Some(i) = stack.pop() {
                    match vtree.node(i) {
                        VtreeNode::Leaf { var, .. } => {
                            out.push(gp.atoms[order[var.0 as usize - 1] as usize].to_string())
                        }
                        VtreeNode::Internal { left, right, .. } => stack.extend([*left, *right]),
                    }
                }
                out.sort();
                out
            };
        // `p` and `q` read disjoint facts and `r` reads both: some node
        // joins the two, and `c`, the fact `r` introduces, sits above it.
        // Each reads two facts, so that neither is a fact under another
        // name, and `s` reads `q` alone, so that the order search does not
        // merge `p` and `q` as one variable's values. Six facts bound any
        // compile below what a search costs, so the components keep the
        // grounder's order (`order::search`), `r` before `s`.
        let src = "0.5::a. 0.5::b. 0.5::c. 0.5::d. 0.5::e. 0.5::f.
                   p :- a, d. q :- b, e. r :- p, q, c. s :- q, f.
                   query(r). query(s).";
        let gp = as_written(src);
        let (Layout { vtree, order, .. }, _) =
            build(&gp, &Deps::of(&gp), Kind::Closed).expect("builds");
        let join = (0..vtree.num_nodes() as u32)
            .map(VtreeIdx)
            .find(|&i| match vtree.node(i) {
                VtreeNode::Internal { left, right, .. } => {
                    let mut sides = [
                        facts_under(&vtree, &order, *left, &gp),
                        facts_under(&vtree, &order, *right, &gp),
                    ];
                    sides.sort();
                    sides == [["a", "d"], ["b", "e"]]
                }
                VtreeNode::Leaf { .. } => false,
            });
        let join = join.expect("a node joins p's facts and q's");
        let above = vtree.node(join).parent().expect("the join is not the root");
        let (left, _) = vtree.children(above);
        assert_eq!(facts_under(&vtree, &order, left, &gp), ["c"]);
        // A chain, each atom reading the one before: nothing to join, and
        // the vtree is right-linear over the searched order, leaf for leaf.
        let src = "0.5::a. 0.5::b. 0.5::c.
                   p :- a. q :- p, b. r :- q, c.
                   query(r).";
        let gp = as_written(src);
        let (
            Layout {
                vtree: closed,
                order: closed_order,
                ..
            },
            _,
        ) = build(&gp, &Deps::of(&gp), Kind::Closed).expect("builds");
        let (linear, linear_order) = searched_right_linear(&gp);
        assert_eq!(closed.to_text(), linear);
        assert_eq!(closed_order, linear_order);
    }

    /// `q` reads the block of `u` and the block of `v`, and its rules pair
    /// their atoms up (`x1` with `y1`, `x2` with `y2`, `u` with `v`) instead
    /// of reading every combination: three combinations of the nine. The
    /// join is not taken and the vtree is right-linear over the searched
    /// order, leaf for leaf.
    #[test]
    fn closed_does_not_join_blocks_whose_atoms_are_paired() {
        let src = "0.3::a1. 0.4::a2. 0.5::a3. 0.6::a4. 0.3::b1. 0.4::b2. 0.5::b3. 0.6::b4. 0.7::c.
                   x1 :- a1, a2. x2 :- a3, a4. u :- x1, x2.
                   y1 :- b1, b2. y2 :- b3, b4. v :- y1, y2.
                   q :- x1, y1, c. q :- x2, y2, c. q :- u, v, c.
                   query(q).";
        let gp = as_written(src);
        let (
            Layout {
                vtree: closed,
                order: closed_order,
                note,
                ..
            },
            _,
        ) = build(&gp, &Deps::of(&gp), Kind::Closed).expect("builds");
        let note = note.unwrap_or_default();
        assert!(note.contains("1 pairing joins made right-linear"), "{note}");
        let (linear, linear_order) = searched_right_linear(&gp);
        assert_eq!(closed.to_text(), linear);
        assert_eq!(closed_order, linear_order);
    }

    /// A closure grounded from its source is a recursive component and an
    /// atom for the sink past it, two components with nothing to order
    /// between them; two components without recursion have an order.
    #[test]
    fn a_recursive_core_leaves_nothing_to_order() {
        let components = |gp: &GroundProgram| {
            let by_head = gp.rules_by_head();
            crate::ground::deps::components(
                gp,
                &by_head,
                crate::ground::deps::derived_sccs(gp, &by_head),
            )
            .len()
        };
        let graph = "0.5::edge(1,2). 0.6::edge(2,3). 0.7::edge(3,2). 0.8::edge(2,4).
                     path(X,Y) :- edge(X,Y).
                     path(X,Y) :- edge(X,Z), path(Z,Y).
                     query(path(1,3)). query(path(1,4)).";
        let gp = as_written(graph);
        assert_eq!(components(&gp), 2);
        assert!(nothing_to_order(&gp));
        // Undirected, each link read both ways through `conn`, which reads
        // the fact alone: the component reads the facts as `conn` atoms
        // (`simplify::alias_fact_atoms`).
        let undirected = "0.5::edge(1,2). 0.6::edge(2,3). 0.7::edge(3,1). 0.8::edge(2,4).
                          conn(X,Y) :- edge(X,Y). conn(X,Y) :- edge(Y,X).
                          path(X,Y) :- conn(X,Y).
                          path(X,Y) :- conn(X,Z), path(Z,Y).
                          query(path(1,3)). query(path(1,4)).";
        let gp = ground(
            &parse_program(undirected).expect("parses"),
            &GroundOptions::default(),
        )
        .expect("grounds");
        assert!(nothing_to_order(&gp));
        let chain = "0.5::a. 0.5::b. 0.5::c. 0.5::d.
                     x :- a, c. y :- x, b, d.
                     query(y).";
        let gp = as_written(chain);
        assert_eq!(components(&gp), 2);
        assert!(!nothing_to_order(&gp));
    }
}
