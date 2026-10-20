//! `decision`'s layout: the tree of the decisions the grounder read facts
//! as guards in ([`crate::ground::Decisions`]), over the facts of the
//! program it names, with the facts it does not name on a right-linear
//! spine below the root's right, in the fan-in order of the shallowest
//! roots first ([`crate::ground::deps::fan_in`]), the first placed read
//! first.
//!
//! A fact the tree leaves out that its readers read beside one part of the
//! tree only goes above the lowest node holding that part instead
//! ([`Tree::homes`]), below the names there. Alea.jl's conjugate programs are
//! three numbers drawn by trees of decisions, each with low flips outside
//! its tree, and evidence on two sums that ripple-carry adders compute,
//! each from two of the numbers. On the spine, the low flips put the
//! adders' gates at the root, where the query walk joined their links
//! beside every value of the three numbers (3.25 G pairs at 18 bits);
//! above the join of the two numbers each adder reads, the walk's largest
//! diagram holds 5 M.
//!
//! The program is laid out with three kinds of names ([`Tree::name`]), each a
//! variable of its own tied to its definition by a link the evidence
//! observes ([`crate::indicators::name_once`]), one for all its readers:
//!
//! - Each context `S` that a decision `l` enters from `S'` is named,
//!   `c_S :- c_S', l`, and its name sits at the top of its subtree. A rule
//!   whose body holds the cube of a context reads the deepest such
//!   context's name in place of the cube's literals. A formula inside the
//!   subtree then tells apart the worlds that reach the context by one
//!   variable at its top, not by the decisions above it. Without the
//!   names, an if-then-else over two values held in disjoint subtrees
//!   holds their product: the rest of the diagram keeps every value of the
//!   branch not taken beside every value of the one taken, and a mux tree
//!   over `k` selector bits held `4^k` pairs (Alea.jl's coinBias, whose
//!   compile took 33 G work units at 16 bits, 0.36 G with the names).
//! - The derived atoms of the tree that atoms reading facts outside it
//!   read (the bits of the value a mux tree selects, which comparators
//!   read beside the flips of the root's spine) are named for those
//!   readers, and their names sit above the lowest node of the tree that
//!   holds their cones. The readers' formulas are then over the spine and
//!   the names, not the tree.
//! - The derived atoms outside the tree whose cones hold no fact of it,
//!   read beside a name of the tree by its readers, are named for those
//!   readers too, on the spine. In Alea.jl's genes every bit of a
//!   reading reads `y`, the or of the T genes, beside the bits of the two
//!   values the tree selects. Expanded, `y`'s formula made 98% of the
//!   pairs of the readers of the tree's root at T = 50, settling built it
//!   again with each composition of a reader, about 23 times, and the
//!   compile's time grew with T; named, `y` is one variable in each
//!   reader. Atoms whose cones share an atom with another's are not
//!   named: their readers read them together, through the facts they
//!   share. In Alea.jl's altermu2 each observation compares a draw with
//!   the bits of `mu1 + mu2`, which the adder's carries tie together.
//!   Named, the compile's largest formula grew from 10 342 pairs to
//!   528 520 at 11 bits; at 13 bits 179 formulas held over a million
//!   pairs each (8.4 M over 23 variables) and the compile ran out of
//!   memory, where with the bits expanded it answers in 26 s.
//!
//! The values that the contexts' names keep apart are those the readers
//! outside the tree read, so where nothing outside the tree reads it,
//! nothing is named. Alea.jl's caesar is such a program: its key and its
//! letters are trees of decisions, and the sums of a letter and the key
//! read nothing else. There the names are variables the compile holds
//! apart until the evidence conjoins their links, and the contexts' names
//! alone took caesar_huge's compile from 0.47 G to 5.36 G work units (its
//! largest formula from 1 154 to 4 848 pairs).
//!
//! The cut names above each node of the tree are a segment that settling
//! keeps or composes away ([`crate::tp::settle`]): they stay where the
//! joint of their links, which the evidence holds, is smaller than what
//! their readers gain composed with the links. coinBias's comparators,
//! which read every bit of the bias the tree selects, grow without bound
//! composed, and its names stay. Where the readers combine the values the
//! tree selects bit by bit (the genes programs), a node's joint holds
//! every combination of its values, and most nodes' readers are
//! composed. The names beside the tree are a segment of their own,
//! settled first, so that the tree's readers are composed over their
//! names rather than their formulas. The contexts' names are in no
//! segment and stay: the links of the tree's atoms read them, and so do
//! the readers composed with those links.

use rustc_hash::FxHashMap;

use super::forest::Forest;
use crate::ground::deps::Deps;
use crate::ground::guards::DecisionNode;
use crate::ground::{AtomId, AtomKind, Decisions, GLit, GroundProgram, GroundRule};
use crate::indicators;
use crate::syntax::{Atom, Term};

/// `decision`'s program with its names ([`Tree::name`]), the names to put
/// above each node of the decisions' tree, outermost first, and a note for
/// `--stats`.
pub(super) struct Named {
    pub(super) gp: GroundProgram,
    pub(super) above: Vec<Vec<AtomId>>,
    pub(super) note: String,
}

/// The decisions' tree over the facts of a program, which the program's
/// names ([`Tree::name`]) and the homes of the facts it leaves out
/// ([`Tree::homes`]) are read from. The program is acyclic, as is every
/// program with decisions (the guards apply to no other).
pub(super) struct Tree<'a> {
    decisions: &'a Decisions,
    gp: &'a GroundProgram,
    /// `gp`'s dependencies.
    deps: &'a Deps,
    /// `gp`'s facts by name.
    by_name: FxHashMap<&'a Atom, AtomId>,
    /// Each context's decision ([`decided`]).
    decision: Vec<Option<GLit>>,
    /// Whether each atom is a fact of the tree ([`tree_facts`]).
    in_tree: Vec<bool>,
    /// The range of leaves each node holds ([`leaf_ranges`]).
    range: Vec<(u32, u32)>,
}

impl<'a> Tree<'a> {
    /// The tree of `decisions` over the facts of `gp`, whose dependencies
    /// are `deps`.
    pub(super) fn of(decisions: &'a Decisions, gp: &'a GroundProgram, deps: &'a Deps) -> Self {
        debug_assert!(deps.sccs.iter().all(|scc| scc.len() == 1));
        let by_name = facts_by_name(gp);
        let decision = decided(decisions, &by_name);
        let in_tree = tree_facts(decisions, gp, &by_name, &decision);
        Tree {
            decisions,
            gp,
            deps,
            by_name,
            decision,
            in_tree,
            range: leaf_ranges(&decisions.nodes),
        }
    }

    /// The program with its contexts, the atoms its tree's readers outside
    /// it read and the atoms they read beside those named (module doc);
    /// `None` where nothing outside the tree reads it.
    pub(super) fn name(&self) -> Option<Named> {
        let (gp, deps) = (self.gp, self.deps);
        let n = gp.num_atoms();
        let contexts = &self.decisions.contexts;
        let nodes = &self.decisions.nodes;
        let decision = &self.decision;
        // Whether each atom's cone holds a fact outside the tree; the atoms
        // of the tree that a rule of such an atom reads are cut.
        let outside = self.cones_holding(|f| !self.in_tree[f]);
        let mut cut_of: Vec<bool> = vec![false; n];
        let mut cut: Vec<AtomId> = Vec::new();
        for r in gp.rules.iter().filter(|r| outside[r.head as usize]) {
            for l in &r.body {
                let b = l.atom();
                if !gp.is_fact(b) && !outside[b as usize] && !cut_of[b as usize] {
                    cut_of[b as usize] = true;
                    cut.push(b);
                }
            }
        }
        if cut.is_empty() {
            return None;
        }
        // Whether each atom's cone holds a fact of the tree; the derived
        // atoms whose cones hold none, read beside a cut atom by its
        // readers, are named for those readers too.
        let inside = self.cones_holding(|f| self.in_tree[f]);
        let (beside, beside_of) = self.beside(&outside, &inside, &cut_of);
        let (reads, named_contexts) = self.context_reads();
        // The rewrite: the context atoms, then the names and their links.
        let mut out = gp.without_rules();
        let mut atom_of_context: Vec<AtomId> = vec![AtomId::MAX; contexts.len()];
        for &k in &named_contexts {
            out.atoms
                .push(Atom::new("$cx", vec![Term::Int(i64::from(k))]));
            out.kinds.push(AtomKind::Derived);
            atom_of_context[k as usize] = (out.atoms.len() - 1) as AtomId;
        }
        let mut var_of: FxHashMap<AtomId, AtomId> = FxHashMap::default();
        let mut name_atom = |out: &mut GroundProgram, a: AtomId| -> AtomId {
            let var = indicators::name_once(out, a);
            var_of.insert(a, var);
            var
        };
        let mut var_of_context: Vec<AtomId> = vec![AtomId::MAX; contexts.len()];
        for &k in &named_contexts {
            var_of_context[k as usize] = name_atom(&mut out, atom_of_context[k as usize]);
        }
        let cut_var: Vec<AtomId> = cut.iter().map(|&a| name_atom(&mut out, a)).collect();
        for &a in &beside {
            name_atom(&mut out, a);
        }
        // The rules: each context's from its parent's name and its
        // decision, and the program's with the names read in place of what
        // they name.
        for &k in &named_contexts {
            let c = &contexts[k as usize];
            let mut body: Vec<GLit> = c
                .parent
                .map(|p| GLit::Pos(var_of_context[p as usize]))
                .into_iter()
                .collect();
            body.push(decision[k as usize].expect("a named context's decision"));
            out.rules
                .push(GroundRule::new(atom_of_context[k as usize], body));
        }
        let mut cube: Vec<GLit> = Vec::new();
        let mut rules: Vec<GroundRule> = Vec::with_capacity(gp.rules.len());
        for (ri, r) in gp.rules.iter().enumerate() {
            let mut body = r.body.clone();
            if let Some(c) = reads[ri] {
                cube.clear();
                let mut up = Some(c);
                while let Some(k) = up {
                    cube.push(decision[k as usize].expect("a named context's decision"));
                    up = contexts[k as usize].parent;
                }
                body.retain(|l| !cube.contains(l));
                body.insert(0, GLit::Pos(var_of_context[c as usize]));
            }
            if outside[r.head as usize] {
                let reads_tree = inside[r.head as usize];
                for l in &mut body {
                    let b = l.atom() as usize;
                    if cut_of.get(b) == Some(&true)
                        || (reads_tree && beside_of.get(b) == Some(&true))
                    {
                        *l = GLit::new(var_of[&l.atom()], l.is_positive());
                    }
                }
            }
            rules.push(GroundRule {
                head: r.head,
                body,
                pairs: r.pairs,
            });
        }
        // The range of the tree each atom's cone holds, in the rewrite.
        let mut held: Vec<(u32, u32)> = vec![NOWHERE; out.atoms.len()];
        for (i, node) in nodes.iter().enumerate() {
            if let DecisionNode::Leaf(name) = node
                && let Some(&f) = self.by_name.get(name)
            {
                held[f as usize] = self.range[i];
            }
        }
        for &k in &named_contexts {
            held[var_of_context[k as usize] as usize] =
                self.range[contexts[k as usize].node as usize];
        }
        for &a in deps.sccs.iter().flatten() {
            if !outside[a as usize] {
                held[a as usize] = deps.by_head[a as usize]
                    .iter()
                    .flat_map(|&ri| &rules[ri].body)
                    .fold(NOWHERE, |h, l| join(h, held[l.atom() as usize]));
            }
        }
        // Each cut name above the lowest node holding its atom's cone, and
        // each context's name above its subtree, outermost.
        let mut above: Vec<Vec<AtomId>> = vec![Vec::new(); nodes.len()];
        for &k in &named_contexts {
            above[contexts[k as usize].node as usize].push(var_of_context[k as usize]);
        }
        let mut cut_node: Vec<usize> = Vec::with_capacity(cut.len());
        for (&a, &var) in cut.iter().zip(&cut_var) {
            let at = lowest(nodes, &self.range, held[a as usize]);
            above[at].push(var);
            cut_node.push(at);
        }
        out.rules.extend(rules);
        out.encoding.segments = segments(out.encoding.indicators.len(), &cut_node, beside.len());
        let note = format!(
            "naming: {} contexts of the decisions, {} atoms of their tree, {} beside it",
            named_contexts.len(),
            cut.len(),
            beside.len()
        );
        Some(Named {
            gp: out,
            above,
            note,
        })
    }

    /// Per atom, whether its cone holds a fact that `fact` holds of: for
    /// a fact whether it does, and for a derived atom whether a rule of it
    /// reads such an atom.
    fn cones_holding(&self, fact: impl Fn(usize) -> bool) -> Vec<bool> {
        let (gp, deps) = (self.gp, self.deps);
        let mut holds: Vec<bool> = (0..gp.num_atoms())
            .map(|a| gp.is_fact(a as AtomId) && fact(a))
            .collect();
        for &a in deps.sccs.iter().flatten() {
            holds[a as usize] = deps.by_head[a as usize]
                .iter()
                .any(|&ri| gp.rules[ri].body.iter().any(|l| holds[l.atom() as usize]));
        }
        holds
    }

    /// The derived atoms whose cones hold a fact outside the tree
    /// (`outside`) and none of it (`inside`) that a rule reading a cut atom
    /// (`cut_of`) reads, in the order the rules read them, and which atoms
    /// they are; but for those whose cones share an atom with another's: a
    /// name parts its atom from its cone, and atoms that share their cone's
    /// facts (the bits of a sum) are correlated through them, which the
    /// readers of their names no longer see.
    fn beside(
        &self,
        outside: &[bool],
        inside: &[bool],
        cut_of: &[bool],
    ) -> (Vec<AtomId>, Vec<bool>) {
        let (gp, deps) = (self.gp, self.deps);
        let n = gp.num_atoms();
        let mut beside_of: Vec<bool> = vec![false; n];
        let mut beside: Vec<AtomId> = Vec::new();
        for r in gp.rules.iter().filter(|r| outside[r.head as usize]) {
            if !r.body.iter().any(|l| cut_of[l.atom() as usize]) {
                continue;
            }
            for l in &r.body {
                let b = l.atom();
                if !gp.is_fact(b)
                    && outside[b as usize]
                    && !inside[b as usize]
                    && !beside_of[b as usize]
                {
                    beside_of[b as usize] = true;
                    beside.push(b);
                }
            }
        }
        // Each one's cone walked in turn, an atom owned by the first to
        // reach it.
        const NONE: usize = usize::MAX;
        let mut owner = vec![NONE; n];
        let mut shared = vec![false; beside.len()];
        let mut stack: Vec<AtomId> = Vec::new();
        for (i, &b) in beside.iter().enumerate() {
            stack.push(b);
            while let Some(x) = stack.pop() {
                match owner[x as usize] {
                    o if o == i => {}
                    NONE => {
                        owner[x as usize] = i;
                        stack.extend(
                            deps.by_head[x as usize]
                                .iter()
                                .flat_map(|&ri| gp.rules[ri].body.iter().map(|l| l.atom())),
                        );
                    }
                    o => {
                        shared[i] = true;
                        shared[o] = true;
                    }
                }
            }
        }
        for (&b, _) in beside.iter().zip(&shared).filter(|&(_, &s)| s) {
            beside_of[b as usize] = false;
        }
        let beside = beside
            .into_iter()
            .zip(shared)
            .filter_map(|(b, s)| (!s).then_some(b))
            .collect();
        (beside, beside_of)
    }

    /// For each rule, the deepest context whose cube its body holds, whose
    /// name it reads in place of the cube's literals; and the contexts
    /// named, those and the contexts they are entered from.
    fn context_reads(&self) -> (Vec<Option<u32>>, Vec<u32>) {
        let contexts = &self.decisions.contexts;
        // The contexts entered from each context (`ROOT` for the root) by
        // each literal, and their depths.
        const ROOT: u32 = u32::MAX;
        let mut child: FxHashMap<(u32, GLit), u32> = FxHashMap::default();
        let mut depth = vec![0u32; contexts.len()];
        for k in (0..contexts.len()).rev() {
            if let Some(l) = self.decision[k] {
                let p = contexts[k].parent;
                depth[k] = p.map_or(1, |p| depth[p as usize] + 1);
                child.insert((p.unwrap_or(ROOT), l), k as u32);
            }
        }
        let mut reads: Vec<Option<u32>> = vec![None; self.gp.rules.len()];
        let mut used = vec![false; contexts.len()];
        let mut stack: Vec<u32> = Vec::new();
        for (ri, r) in self.gp.rules.iter().enumerate() {
            let mut best: Option<u32> = None;
            stack.clear();
            stack.push(ROOT);
            while let Some(at) = stack.pop() {
                for &l in &r.body {
                    if let Some(&c) = child.get(&(at, l)) {
                        if best.is_none_or(|b| depth[c as usize] > depth[b as usize]) {
                            best = Some(c);
                        }
                        stack.push(c);
                    }
                }
            }
            reads[ri] = best;
            let mut up = best;
            while let Some(c) = up.filter(|&c| !used[c as usize]) {
                used[c as usize] = true;
                up = contexts[c as usize].parent;
            }
        }
        let named = (0..contexts.len() as u32)
            .filter(|&k| used[k as usize])
            .collect();
        (reads, named)
    }

    /// The facts the decisions' tree leaves out that one part of it reads,
    /// each in the list of the node it goes above: the lowest that holds the
    /// cones of the literals its readers' rules read beside it, or, for a rule
    /// whose other literals hold nothing of the tree, what its head is read
    /// beside, and so on up. A fact read beside the whole tree, or beside
    /// nothing of it, stays on the root's spine.
    pub(super) fn homes(&self) -> Vec<Vec<AtomId>> {
        let (gp, deps) = (self.gp, self.deps);
        let nodes = &self.decisions.nodes;
        let mut homes: Vec<Vec<AtomId>> = vec![Vec::new(); nodes.len()];
        if nodes.is_empty() {
            return homes;
        }
        let n = gp.num_atoms();
        let (by_name, tree, range) = (&self.by_name, &self.in_tree, &self.range);
        // The leaves of the tree's facts each atom's cone holds, and those of
        // all of them.
        let mut held: Vec<(u32, u32)> = vec![NOWHERE; n];
        let mut all = NOWHERE;
        for (i, node) in nodes.iter().enumerate() {
            if let DecisionNode::Leaf(name) = node
                && let Some(&f) = by_name.get(name)
                && tree[f as usize]
            {
                held[f as usize] = range[i];
                all = join(all, range[i]);
            }
        }
        for &a in deps.sccs.iter().flatten() {
            held[a as usize] = deps.by_head[a as usize]
                .iter()
                .flat_map(|&ri| &gp.rules[ri].body)
                .fold(NOWHERE, |h, l| join(h, held[l.atom() as usize]));
        }
        // What each atom is read beside, its readers before it.
        let mut beside: Vec<(u32, u32)> = vec![NOWHERE; n];
        let mut after: Vec<(u32, u32)> = Vec::new();
        for &h in deps.sccs.iter().flatten().rev() {
            let up = beside[h as usize];
            for &ri in &deps.by_head[h as usize] {
                let body = &gp.rules[ri].body;
                // `after[k]`: what the literals after the `k`-th hold.
                after.clear();
                after.resize(body.len() + 1, NOWHERE);
                for k in (0..body.len()).rev() {
                    after[k] = join(after[k + 1], held[body[k].atom() as usize]);
                }
                let mut before = NOWHERE;
                for (k, l) in body.iter().enumerate() {
                    let others = join(before, after[k + 1]);
                    let x = l.atom() as usize;
                    beside[x] = join(beside[x], if others.0 <= others.1 { others } else { up });
                    before = join(before, held[x]);
                }
            }
        }
        let whole = lowest(nodes, range, all);
        for (f, _) in gp.facts() {
            let r = beside[f as usize];
            if !tree[f as usize] && r.0 <= r.1 {
                let at = lowest(nodes, range, r);
                if at != whole {
                    homes[at].push(f);
                }
            }
        }
        homes
    }
}

/// The settling segment of each of a program's `indicators` copies
/// ([`crate::tp::settle`]) after [`Tree::name`]: the cut names above each
/// node of the tree, whose nodes are `cut_node`, a segment of their own,
/// numbered up the tree, so that settling, which spends at most the
/// compile's work, settles the many small nodes before the few near the
/// root, whose readers cost the most to compose. The `beside` names after
/// them come first, segment 0, and the copies before them, the contexts',
/// are in none.
fn segments(indicators: usize, cut_node: &[usize], beside: usize) -> Vec<usize> {
    let mut nodes = cut_node.to_vec();
    nodes.sort_unstable();
    nodes.dedup();
    let before = indicators - cut_node.len() - beside;
    let shift = usize::from(beside > 0);
    std::iter::repeat_n(usize::MAX, before)
        .chain(
            cut_node
                .iter()
                .map(|n| shift + nodes.binary_search(n).expect("the node of a cut name")),
        )
        .chain(std::iter::repeat_n(0, beside))
        .collect()
}

/// The facts of `gp` by name.
fn facts_by_name(gp: &GroundProgram) -> FxHashMap<&Atom, AtomId> {
    gp.facts()
        .map(|(a, _)| (&gp.atoms[a as usize], a))
        .collect()
}

/// Each context's decision, where it and its parents' are facts of `gp`.
fn decided(decisions: &Decisions, by_name: &FxHashMap<&Atom, AtomId>) -> Vec<Option<GLit>> {
    let contexts = &decisions.contexts;
    let mut decision: Vec<Option<GLit>> = vec![None; contexts.len()];
    // Parents come after their children.
    for k in (0..contexts.len()).rev() {
        let c = &contexts[k];
        if c.parent.is_none_or(|p| decision[p as usize].is_some()) {
            decision[k] = by_name.get(&c.fact).map(|&f| GLit::new(f, c.positive));
        }
    }
    decision
}

/// The facts of the tree: the decisions and the facts below a context.
fn tree_facts(
    decisions: &Decisions,
    gp: &GroundProgram,
    by_name: &FxHashMap<&Atom, AtomId>,
    decision: &[Option<GLit>],
) -> Vec<bool> {
    let nodes = &decisions.nodes;
    let mut under = vec![false; nodes.len()];
    for c in &decisions.contexts {
        under[c.node as usize] = true;
    }
    let mut tree = vec![false; gp.num_atoms()];
    for l in decision.iter().flatten() {
        tree[l.atom() as usize] = true;
    }
    for i in (0..nodes.len()).rev() {
        match &nodes[i] {
            &DecisionNode::Join(l, r) => {
                under[l as usize] |= under[i];
                under[r as usize] |= under[i];
            }
            DecisionNode::Leaf(name) => {
                if under[i]
                    && let Some(&f) = by_name.get(name)
                {
                    tree[f as usize] = true;
                }
            }
        }
    }
    tree
}

/// A range of leaves of the decisions' tree that holds none.
const NOWHERE: (u32, u32) = (u32::MAX, 0);

/// The range that holds `a` and `b`.
fn join(a: (u32, u32), b: (u32, u32)) -> (u32, u32) {
    (a.0.min(b.0), a.1.max(b.1))
}

/// The leaves of the decisions' tree in depth-first order, and the range
/// of them each node holds.
fn leaf_ranges(nodes: &[DecisionNode]) -> Vec<(u32, u32)> {
    let mut range: Vec<(u32, u32)> = vec![NOWHERE; nodes.len()];
    let mut next = 0u32;
    let mut walk: Vec<usize> = nodes.len().checked_sub(1).into_iter().collect();
    while let Some(i) = walk.pop() {
        match nodes[i] {
            DecisionNode::Join(l, r) => walk.extend([r as usize, l as usize]),
            DecisionNode::Leaf(_) => {
                range[i] = (next, next);
                next += 1;
            }
        }
    }
    for i in 0..nodes.len() {
        if let DecisionNode::Join(l, r) = nodes[i] {
            range[i] = join(range[l as usize], range[r as usize]);
        }
    }
    range
}

/// The lowest node of the decisions' tree that holds the leaves `(lo,
/// hi)`, the root where they are none.
fn lowest(nodes: &[DecisionNode], range: &[(u32, u32)], (lo, hi): (u32, u32)) -> usize {
    let mut at = nodes.len() - 1;
    if lo <= hi {
        while let DecisionNode::Join(l, r) = nodes[at] {
            let inside = |c: u32| range[c as usize].0 <= lo && hi <= range[c as usize].1;
            at = match (inside(l), inside(r)) {
                (true, _) => l as usize,
                (_, true) => r as usize,
                _ => break,
            };
        }
    }
    at
}

/// The decisions' tree over the facts of `gp` (above), and its root, with
/// the names `above` puts over its nodes (outermost first). Where the
/// evidence fixed every fact the decisions name, the spine alone; `None`
/// where `gp` has no facts.
pub(super) fn decision_forest(
    decisions: &Decisions,
    gp: &GroundProgram,
    above: &[Vec<AtomId>],
    homes: &[Vec<AtomId>],
) -> Option<(Forest, usize)> {
    let by_name = facts_by_name(gp);
    // A fact placed above its home leaves its own leaf.
    let mut placed = vec![false; gp.num_atoms()];
    for &f in homes.iter().flatten() {
        placed[f as usize] = true;
    }
    let mut forest = Forest::default();
    // Per node, where it went in the forest, if any of its facts is `gp`'s.
    let mut at: Vec<Option<usize>> = Vec::with_capacity(decisions.nodes.len());
    for (i, n) in decisions.nodes.iter().enumerate() {
        let mut here =
            match n {
                DecisionNode::Leaf(name) => by_name
                    .get(name)
                    .filter(|&&a| !placed[a as usize])
                    .map(|&a| {
                        placed[a as usize] = true;
                        forest.leaf(a)
                    }),
                &DecisionNode::Join(l, r) => match (at[l as usize], at[r as usize]) {
                    (Some(l), Some(r)) => Some(forest.join(l, r)),
                    (l, r) => l.or(r),
                },
            };
        // The names outermost, the facts placed here below them.
        let names = above.get(i).into_iter().flatten();
        for &v in names.chain(homes.get(i).into_iter().flatten()).rev() {
            placed[v as usize] = true;
            let leaf = forest.leaf(v);
            here = Some(match here {
                Some(h) => forest.join(leaf, h),
                None => leaf,
            });
        }
        at.push(here);
    }
    let root = at.last().copied().flatten();
    let mut rest: Vec<AtomId> = gp
        .facts()
        .map(|(a, _)| a)
        .filter(|&a| !placed[a as usize])
        .collect();
    if rest.is_empty() {
        return root.map(|root| (forest, root));
    }
    let met = crate::ground::deps::fan_in(gp, false);
    rest.sort_unstable_by_key(|&a| met[a as usize]);
    // `(root (r1 (r2 ...)))`, built from its innermost join out, the first
    // fact placed innermost (read first).
    let mut spine = None;
    for &a in &rest {
        let leaf = forest.leaf(a);
        spine = Some(match spine {
            Some(s) => forest.join(leaf, s),
            None => leaf,
        });
    }
    let spine = spine.expect("some rest");
    let root = match root {
        Some(root) => forest.join(root, spine),
        None => spine,
    };
    Some((forest, root))
}
