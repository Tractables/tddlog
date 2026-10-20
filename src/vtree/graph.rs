//! The interaction graph of a ground program, its min-fill elimination,
//! and the fact orders read off the program: the order the compiler
//! consumes the facts in and the reverse of a min-fill elimination.

use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};
use std::time::Duration;

use rustc_hash::FxHashSet;

use crate::ground::deps::Deps;
use crate::ground::shape::observed_false_alone;
use crate::ground::{AtomId, AtomKind, GroundProgram, RulesByHead};

/// Fact atoms in the order the compiler consumes them: the components of
/// derived atoms in dependency order (the order the fixpoint compiles them),
/// each fact at its first occurrence in the bodies of their rules; facts no
/// rule uses come last, in program order.
pub(crate) fn facts_by_use(gp: &GroundProgram, deps: &Deps) -> Vec<AtomId> {
    facts_at_first_use(gp, &deps.by_head, &deps.sccs)
}

/// Fact atoms at their first use by the components taken in the given
/// order; unused facts last, in program order.
pub(super) fn facts_at_first_use(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    components: &[Vec<AtomId>],
) -> Vec<AtomId> {
    // Derived atoms count as seen, so that one byte per literal tells a
    // fact not seen yet.
    let mut seen: Vec<bool> = gp
        .kinds
        .iter()
        .map(|k| matches!(k, AtomKind::Derived))
        .collect();
    let mut order = Vec::new();
    for scc in components {
        for &h in scc {
            for &ri in &by_head[h as usize] {
                for l in &gp.rules[ri].body {
                    let a = l.atom();
                    if !seen[a as usize] {
                        seen[a as usize] = true;
                        order.push(a);
                    }
                }
            }
        }
    }
    for (a, _) in gp.facts() {
        if !seen[a as usize] {
            order.push(a);
        }
    }
    order
}

/// The interaction graph of the ground program: one vertex per atom — facts
/// and derived alike — and, for every rule, a clique over its head and the
/// atoms of its body, each of the rules a rule of pairs stands for in
/// turn. Returns the deduplicated undirected edges as pairs `(u, v)` with
/// `u < v`. A rule with `k` distinct atoms contributes `k * (k - 1) / 2`
/// of them.
///
/// The head stays out of the clique of an atom the evidence observes false
/// and nothing else reads ([`observed_false_alone`]). Its formula is then
/// only ever conjoined negated, `⋀_r ¬body_r`, one clause per rule, and a
/// cut through those clauses carries nothing about the atom beyond whether
/// a clause has failed; as a vertex it would tie every body to every other,
/// as an atom observed true or read by another rule does. A noisy-or
/// finding observed absent so stops coupling its causes.
pub(crate) fn interaction_edges(gp: &GroundProgram) -> Vec<(u32, u32)> {
    let mut seen = FxHashSet::default();
    let mut edges = Vec::new();
    let mut clique: Vec<AtomId> = Vec::new();
    let alone = observed_false_alone(gp);
    for (head, body) in gp
        .rules
        .iter()
        .flat_map(|r| r.bodies().map(|body| (r.head, body)))
    {
        clique.clear();
        if !alone[head as usize] {
            clique.push(head);
        }
        clique.extend(body.iter().map(|l| l.atom()));
        clique.sort_unstable();
        clique.dedup();
        for (i, &u) in clique.iter().enumerate() {
            for &v in &clique[i + 1..] {
                if seen.insert((u, v)) {
                    edges.push((u, v));
                }
            }
        }
    }
    edges
}

/// The common neighbours of `a` and `b`, read off the smaller of their sets.
fn common(adjacent: &[FxHashSet<u32>], a: u32, b: u32) -> impl Iterator<Item = u32> + '_ {
    let (a, b) = (&adjacent[a as usize], &adjacent[b as usize]);
    let (small, large) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    small.iter().copied().filter(move |w| large.contains(w))
}

/// The graph a min-fill elimination runs on: the neighbours of each vertex
/// and how many edges join two of them, the triangles through it, so that
/// a vertex's fill, the pairs of its neighbours not yet adjacent, is read
/// off its degree and that count rather than recounted.
struct FillGraph {
    adjacent: Vec<FxHashSet<u32>>,
    triangles: Vec<u64>,
    /// The elimination at which a vertex was last touched, so that it is
    /// reported once.
    touched_at: Vec<u32>,
    eliminations: u32,
    /// The set operations done so far, in goatd's units of work: a lookup,
    /// an insertion, a removal or a step through a set is one.
    work: u64,
}

impl FillGraph {
    /// The graph, with its triangles counted, or `None` where counting them
    /// takes more than `max_work` units.
    fn new(num_vertices: usize, edges: &[(u32, u32)], max_work: u64) -> Option<Self> {
        let mut adjacent: Vec<FxHashSet<u32>> = vec![FxHashSet::default(); num_vertices];
        for &(u, v) in edges {
            if u != v && adjacent[u as usize].insert(v) {
                adjacent[v as usize].insert(u);
            }
        }
        let work = 2 * edges.len() as u64;
        let mut graph = FillGraph {
            adjacent,
            triangles: vec![0; num_vertices],
            touched_at: vec![u32::MAX; num_vertices],
            eliminations: 0,
            work,
        };
        // Each triangle through `u` is met from both of its edges at `u`.
        for u in 0..num_vertices as u32 {
            for &w in &graph.adjacent[u as usize] {
                if u < w {
                    let (a, b) = (
                        graph.adjacent[u as usize].len(),
                        graph.adjacent[w as usize].len(),
                    );
                    graph.work += a.min(b) as u64;
                    let common = common(&graph.adjacent, u, w).count() as u64;
                    graph.triangles[u as usize] += common;
                    graph.triangles[w as usize] += common;
                }
            }
            if graph.work > max_work {
                return None;
            }
        }
        for t in &mut graph.triangles {
            *t /= 2;
        }
        Some(graph)
    }

    /// The vertex's score: its fill, its degree and its number, the least
    /// going first.
    fn score(&self, v: u32) -> (u64, u32, u32) {
        let degree = self.adjacent[v as usize].len() as u64;
        (
            degree * degree.saturating_sub(1) / 2 - self.triangles[v as usize],
            degree as u32,
            v,
        )
    }

    /// Eliminates `v`, whose fill is `fill`: its neighbours, left in
    /// `neighbours`, become a clique and it leaves the graph. `touched`
    /// gets, once each, the vertices whose score this changes: the
    /// neighbours, whose degree and triangles change, and the common
    /// neighbours of the two ends of a fill edge, whose triangles do. No
    /// other vertex's neighbours or the edges among them change. The pairs
    /// of neighbours are searched for the missing edges only until `fill`
    /// of them are found, and not at all where `v` is simplicial.
    ///
    /// False, with the graph left part way, where the work done reaches
    /// more than `max_work` units before `v` is gone.
    fn eliminate(
        &mut self,
        v: u32,
        fill: u64,
        neighbours: &mut Vec<u32>,
        touched: &mut Vec<u32>,
        max_work: u64,
    ) -> bool {
        self.eliminations += 1;
        touched.clear();
        neighbours.clear();
        neighbours.extend(self.adjacent[v as usize].iter().copied());
        self.work += neighbours.len() as u64;
        let mut missing = fill;
        'pairs: for (i, &a) in neighbours.iter().enumerate() {
            if self.work > max_work {
                return false;
            }
            for &b in &neighbours[i + 1..] {
                if missing == 0 {
                    break 'pairs;
                }
                self.work += 1;
                if self.adjacent[a as usize].contains(&b) {
                    continue;
                }
                missing -= 1;
                let (da, db) = (
                    self.adjacent[a as usize].len(),
                    self.adjacent[b as usize].len(),
                );
                self.work += da.min(db) as u64 + 2;
                // `v` is a common neighbour too: the edge closes a triangle
                // through it, which goes with it below.
                let mut closed = 0u64;
                for w in common(&self.adjacent, a, b) {
                    closed += 1;
                    self.triangles[w as usize] += 1;
                    if w != v && self.touched_at[w as usize] != self.eliminations {
                        self.touched_at[w as usize] = self.eliminations;
                        touched.push(w);
                    }
                }
                self.triangles[a as usize] += closed;
                self.triangles[b as usize] += closed;
                self.adjacent[a as usize].insert(b);
                self.adjacent[b as usize].insert(a);
            }
        }
        // With the neighbourhood a clique, each neighbour loses `v` and the
        // triangles through `v` and each of the others.
        let others = neighbours.len().saturating_sub(1) as u64;
        self.work += neighbours.len() as u64;
        for &a in neighbours.iter() {
            self.adjacent[a as usize].remove(&v);
            self.triangles[a as usize] -= others;
            if self.touched_at[a as usize] != self.eliminations {
                self.touched_at[a as usize] = self.eliminations;
                touched.push(a);
            }
        }
        self.adjacent[v as usize] = FxHashSet::default();
        true
    }
}

/// A lower bound on the treewidth of the graph with `num_vertices`
/// vertices and `edges`: minor-min-width (Gogate and Dechter 2004), with
/// Bodlaender and Koster's min-d choice of the edge to contract. A graph's
/// treewidth is at least its least degree, and contracting an edge does not
/// raise it, so the vertex of least degree is contracted into its neighbour
/// of least degree (ties to the lower number) until no edge is left; the
/// bound is the largest least degree met on the way. Where a decomposition
/// is as narrow as this, no search finds a narrower one.
pub(super) fn minor_min_width(num_vertices: usize, edges: &[(u32, u32)]) -> usize {
    let mut adjacent: Vec<FxHashSet<u32>> = vec![FxHashSet::default(); num_vertices];
    for &(u, v) in edges {
        if u != v {
            adjacent[u as usize].insert(v);
            adjacent[v as usize].insert(u);
        }
    }
    let mut by_degree: BTreeSet<(usize, u32)> = (0..num_vertices)
        .map(|v| (adjacent[v].len(), v as u32))
        .collect();
    let mut bound = 0;
    while let Some((degree, v)) = by_degree.pop_first() {
        bound = bound.max(degree);
        let Some(&u) = adjacent[v as usize]
            .iter()
            .min_by_key(|&&w| (adjacent[w as usize].len(), w))
        else {
            continue;
        };
        // `v`'s neighbours become `u`'s.
        let neighbours = std::mem::take(&mut adjacent[v as usize]);
        by_degree.remove(&(adjacent[u as usize].len(), u));
        adjacent[u as usize].remove(&v);
        for &w in &neighbours {
            if w == u {
                continue;
            }
            by_degree.remove(&(adjacent[w as usize].len(), w));
            adjacent[w as usize].remove(&v);
            if adjacent[w as usize].insert(u) {
                adjacent[u as usize].insert(w);
            }
            by_degree.insert((adjacent[w as usize].len(), w));
        }
        by_degree.insert((adjacent[u as usize].len(), u));
    }
    bound
}

/// A min-fill elimination order over the whole graph, breaking ties on the
/// current degree and then on the vertex number, and the elimination tree:
/// the parent of each vertex (`u32::MAX` at a root) is, of its neighbours
/// when it goes, the first to go after it.
///
/// The search is exact: every vertex goes with the least score of those
/// left. Each vertex's fill is kept as its degree and the edges among its
/// neighbours, and an elimination corrects them for the vertices whose
/// neighbourhood or the edges inside it change ([`FillGraph::eliminate`]),
/// which costs the fill edges it adds, each checked against the smaller
/// set of neighbours of its two ends. A derived atom over `n` facts is a
/// star, whose leaves go first at constant cost each; rescoring every
/// vertex within two steps of the one that goes would cost the star about
/// `n^3 / 64` word operations. Scores live in a heap; an entry that is no
/// longer its vertex's score is passed over when it is popped, since every
/// change of a score pushes the new one.
///
/// Past twice [`ELIMINATION_BUDGET_UNITS`] of work the vertices left go
/// as one clique ([`Elimination`]), each the parent of the one before it.
pub(super) fn min_fill_elimination(
    num_vertices: usize,
    edges: &[(u32, u32)],
) -> (Vec<u32>, Vec<u32>) {
    let Elimination {
        order,
        at_elimination,
    } = min_fill_cliques(num_vertices, edges);
    let mut position = vec![0usize; num_vertices];
    for (i, &v) in order.iter().enumerate() {
        position[v as usize] = i;
    }
    let mut parent = vec![u32::MAX; num_vertices];
    for (&v, around) in order.iter().zip(&at_elimination) {
        if let Some(&p) = around.iter().min_by_key(|&&u| position[u as usize]) {
            parent[v as usize] = p;
        }
    }
    let rest = &order[at_elimination.len()..];
    for pair in rest.windows(2) {
        parent[pair[0] as usize] = pair[1];
    }
    (order, parent)
}

/// The weight of a min-fill elimination of the program's interaction
/// graph, as log2 of the sum over its eliminations of 2^k, k the vertex
/// and its neighbours when it goes: the entries of the tables an
/// elimination over Boolean atoms would build, a measure of what
/// compiling the program costs that tells programs whose widths differ
/// by a few vertices apart. Zero for a program without atoms.
pub(crate) fn min_fill_weight(gp: &GroundProgram) -> f64 {
    let elimination = min_fill_cliques(gp.num_atoms(), &interaction_edges(gp));
    elimination
        .clique_sizes()
        .fold(f64::NEG_INFINITY, |w, k| log2_add(w, k as f64))
        .max(0.0)
}

/// log2(2^a + 2^b), exact where either is far below the other.
pub(crate) fn log2_add(a: f64, b: f64) -> f64 {
    let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
    if lo == f64::NEG_INFINITY {
        hi
    } else {
        hi + (lo - hi).exp2().ln_1p() / std::f64::consts::LN_2
    }
}

/// The work, in goatd's units ([`goatd::meter`]), at which an elimination
/// order stops being computed exactly. goatd's min-degree and min-fill take
/// it as their soft budget: past it they score from stale entries, and past
/// twice it they put the vertices left in one bag. tddlog's own min-fill
/// ([`min_fill_cliques`]), whose scoring is already cheap, stops at twice
/// it the same way, and the eliminations in a given order
/// ([`eliminated_in_order`]) give up there. Exact elimination is cubic in a
/// dense graph's degree, and unbounded it takes most of the run on the
/// large weighted CNFs.
pub(super) const ELIMINATION_BUDGET_UNITS: u64 = 1_000_000_000;

/// [`ELIMINATION_BUDGET_UNITS`] as the soft budget goatd's eliminations
/// take. It is read on goatd's meter, which must be armed around them
/// ([`goatd::meter::arm`]); otherwise it would be wall-clock time.
pub(super) fn elimination_budget() -> Duration {
    Duration::from_millis(goatd::meter::milliseconds_for_units(
        ELIMINATION_BUDGET_UNITS,
    ))
}

/// An elimination of a graph's vertices, min-fill's
/// ([`min_fill_cliques`]) or in a given order ([`eliminated_in_order`]):
/// the vertices in the order they go, and the neighbours of each of the
/// first `at_elimination.len()` of them when it goes. Where the
/// elimination stopped, the vertices after those go as one clique.
///
/// goatd has a min-fill of its own (`goatd::elimination::Order::MinFill`),
/// which `td` and `families` call, but it returns a tree decomposition,
/// not the order and each vertex's neighbours when it goes, which
/// `dynamic`'s two starts and the split's weight ([`min_fill_weight`])
/// read. It
/// also reduces the graph first and breaks ties by a seeded salt, so its
/// order is not this one, ties going by fill, degree and number.
pub(super) struct Elimination {
    pub(super) order: Vec<u32>,
    pub(super) at_elimination: Vec<Vec<u32>>,
}

impl Elimination {
    /// The size of each vertex's clique when it goes, the vertex included,
    /// in the order.
    fn clique_sizes(&self) -> impl Iterator<Item = usize> + '_ {
        let n = self.order.len();
        self.at_elimination
            .iter()
            .map(|around| around.len() + 1)
            .chain((self.at_elimination.len()..n).map(move |i| n - i))
    }

    /// The tree decomposition of `graph`, the graph eliminated, that the
    /// elimination gives: a bag for each vertex it eliminated, holding it
    /// and its neighbours when it went, whose parent is the bag of the
    /// first of those neighbours to go; the vertices it left form one bag,
    /// the root.
    pub(super) fn decomposition(&self, graph: &goatd::Graph) -> Option<goatd::TreeDecomposition> {
        let n = self.order.len();
        let exact = self.at_elimination.len();
        let mut position = vec![0usize; n];
        for (i, &v) in self.order.iter().enumerate() {
            position[v as usize] = i;
        }
        // The bag a vertex goes in: its own, or the last one, of the
        // vertices left. Bags are numbered from the last, the root.
        let num_bags = exact + usize::from(exact < n);
        let last = num_bags.saturating_sub(1);
        let bag_of = |u: u32| position[u as usize].min(exact);
        let mut bags = Vec::with_capacity(num_bags);
        let mut tree_edges = Vec::new();
        for (i, (&v, around)) in self.order.iter().zip(&self.at_elimination).enumerate() {
            if let Some(&first) = around.iter().min_by_key(|&&u| position[u as usize]) {
                tree_edges.push((last - i, last - bag_of(first)));
            }
            bags.push(around.iter().copied().chain([v]).collect());
        }
        if exact < n {
            bags.push(self.order[exact..].to_vec());
        }
        bags.reverse();
        goatd::TreeDecomposition::new_trusted(graph, bags, tree_edges).ok()
    }
}

/// The min-fill elimination of [`min_fill_elimination`], each vertex's
/// neighbours when it goes, in the order. The work it does is charged to
/// goatd's meter; past twice [`ELIMINATION_BUDGET_UNITS`] it stops, and
/// the vertices left go as one clique, in the order of their numbers.
pub(super) fn min_fill_cliques(num_vertices: usize, edges: &[(u32, u32)]) -> Elimination {
    min_fill_within(num_vertices, edges, 2 * ELIMINATION_BUDGET_UNITS)
}

/// The elimination of the vertices of the graph whose edges are `edges`
/// in the order `order`, every vertex one at a time. `None` when a vertex
/// would go with more than `width` neighbours, or when eliminating takes
/// more than `max_work` units of work (a step through a set, an insertion
/// or a removal is one), which is charged to goatd's meter.
///
/// goatd eliminates in a caller's order too (`Order::Ranked`), with the
/// same bags, but numbers them otherwise, and the families layout roots a
/// decomposition at its first bag: rooted as goatd numbers it, grid20's
/// `families:0` takes 60% more work.
pub(super) fn eliminated_in_order(
    edges: &[(u32, u32)],
    order: Vec<u32>,
    width: usize,
    max_work: u64,
) -> Option<Elimination> {
    let mut adjacent: Vec<FxHashSet<u32>> = vec![FxHashSet::default(); order.len()];
    for &(u, v) in edges {
        adjacent[u as usize].insert(v);
        adjacent[v as usize].insert(u);
    }
    let mut work = 2 * edges.len() as u64;
    let mut at_elimination = Vec::with_capacity(order.len());
    for &v in &order {
        let neighbours: Vec<u32> = std::mem::take(&mut adjacent[v as usize])
            .into_iter()
            .collect();
        if neighbours.len() > width {
            goatd::meter::charge(work);
            return None;
        }
        for &u in &neighbours {
            let set = &mut adjacent[u as usize];
            set.remove(&v);
            set.extend(neighbours.iter().copied().filter(|&w| w != u));
        }
        work += (neighbours.len() * neighbours.len()) as u64;
        if work > max_work {
            goatd::meter::charge(work);
            return None;
        }
        at_elimination.push(neighbours);
    }
    goatd::meter::charge(work);
    Some(Elimination {
        order,
        at_elimination,
    })
}

/// [`min_fill_cliques`], stopped past `max_work` units.
fn min_fill_within(num_vertices: usize, edges: &[(u32, u32)], max_work: u64) -> Elimination {
    let mut order = Vec::with_capacity(num_vertices);
    let mut at_elimination: Vec<Vec<u32>> = Vec::with_capacity(num_vertices);
    let mut eliminated = vec![false; num_vertices];
    let mut work = 0;
    if let Some(mut graph) = FillGraph::new(num_vertices, edges, max_work) {
        let mut heap: BinaryHeap<Reverse<(u64, u32, u32)>> = (0..num_vertices as u32)
            .map(|v| Reverse(graph.score(v)))
            .collect();
        let mut neighbours = Vec::new();
        let mut touched = Vec::new();
        while order.len() < num_vertices {
            let Reverse(entry) = heap.pop().expect("every live vertex has an entry");
            let v = entry.2;
            if eliminated[v as usize] || graph.score(v) != entry {
                continue;
            }
            if !graph.eliminate(v, entry.0, &mut neighbours, &mut touched, max_work) {
                break;
            }
            eliminated[v as usize] = true;
            order.push(v);
            at_elimination.push(neighbours.clone());
            heap.extend(touched.iter().map(|&u| Reverse(graph.score(u))));
        }
        work = graph.work;
    }
    goatd::meter::charge(work);
    order.extend((0..num_vertices as u32).filter(|&v| !eliminated[v as usize]));
    Elimination {
        order,
        at_elimination,
    }
}

/// The facts in a pre-order of the min-fill elimination tree of the
/// interaction graph: a vertex before the vertices eliminated under it, the
/// last eliminated first at every branch, each subtree finished before the
/// next begins. Derived atoms are eliminated with the rest and then
/// dropped: they constrain the order without becoming variables.
///
/// Every such order eliminates in reverse with the same fill. The reverse
/// elimination order itself interleaves the subtrees: on a noisy-or network
/// it puts the diseases first and every link last, so the cut above the
/// links holds every assignment to the diseases. The post-order (children
/// first) costs the random graphs more work.
pub(crate) fn facts_by_minfill(gp: &GroundProgram) -> Vec<AtomId> {
    let edges = interaction_edges(gp);
    let (order, parent) = min_fill_elimination(gp.num_atoms(), &edges);
    pre_order(&order, &parent)
        .into_iter()
        .filter(|&a| gp.is_fact(a))
        .collect()
}

/// The pre-order of an elimination forest: a vertex, then the subtrees of
/// its children, the last eliminated first, and the roots the same way.
fn pre_order(order: &[u32], parent: &[u32]) -> Vec<AtomId> {
    let mut children: Vec<Vec<u32>> = vec![Vec::new(); order.len()];
    let mut roots = Vec::new();
    for &v in order {
        match parent[v as usize] {
            u32::MAX => roots.push(v),
            p => children[p as usize].push(v),
        }
    }
    // Children and roots are listed in elimination order, so a stack pushed
    // in that order pops the last eliminated first.
    let mut visited = Vec::with_capacity(order.len());
    let mut stack = roots;
    while let Some(v) = stack.pop() {
        visited.push(v);
        stack.extend_from_slice(&children[v as usize]);
    }
    visited
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vtree::tests::ground_smokers;

    #[test]
    fn minor_min_width_bounds_cliques_cycles_trees_and_grids() {
        let clique = |n: u32| -> Vec<(u32, u32)> {
            (0..n)
                .flat_map(|u| (u + 1..n).map(move |v| (u, v)))
                .collect()
        };
        assert_eq!(minor_min_width(6, &clique(6)), 5);
        let cycle: Vec<(u32, u32)> = (0..8).map(|u| (u, (u + 1) % 8)).collect();
        assert_eq!(minor_min_width(8, &cycle), 2);
        let path: Vec<(u32, u32)> = (0..7).map(|u| (u, u + 1)).collect();
        assert_eq!(minor_min_width(8, &path), 1);
        assert_eq!(minor_min_width(3, &[]), 0);
        // The 4 x 4 grid has treewidth 4; the bound is at most that.
        let mut grid = Vec::new();
        for r in 0..4u32 {
            for c in 0..4u32 {
                if c < 3 {
                    grid.push((4 * r + c, 4 * r + c + 1));
                }
                if r < 3 {
                    grid.push((4 * r + c, 4 * r + c + 4));
                }
            }
        }
        let bound = minor_min_width(16, &grid);
        assert!((2..=4).contains(&bound), "{bound}");
    }

    #[test]
    fn min_fill_eliminates_every_vertex_once() {
        // Two triangles sharing a vertex, plus an isolated one.
        let edges = [(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2)];
        let (mut order, _) = min_fill_elimination(6, &edges);
        assert_eq!(order.len(), 6);
        order.sort_unstable();
        assert_eq!(order, (0..6).collect::<Vec<u32>>());
    }

    /// Stopped by its work at every point, the min-fill eliminates the
    /// vertices it had eliminated by then as before, and the rest go as one
    /// clique: a valid decomposition, each of those the parent of the one
    /// before it, and a weight that counts the clique.
    #[test]
    fn a_min_fill_stopped_by_its_work_puts_the_rest_in_one_bag() {
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        for _ in 0..40 {
            let n = 2 + (next() % 30) as usize;
            let mut edges = Vec::new();
            for u in 0..n as u32 {
                for v in u + 1..n as u32 {
                    if next() % 10 < 4 {
                        edges.push((u, v));
                    }
                }
            }
            let whole = min_fill_within(n, &edges, u64::MAX);
            assert_eq!(whole.at_elimination.len(), n);
            let graph = goatd::Graph::new(n as u32, edges.clone());
            for max_work in [0, 10, 50, 100, 200, 400, 800] {
                let cut = min_fill_within(n, &edges, max_work);
                let exact = cut.at_elimination.len();
                assert_eq!(cut.order[..exact], whole.order[..exact]);
                assert_eq!(cut.at_elimination, whole.at_elimination[..exact]);
                let mut sorted = cut.order[exact..].to_vec();
                sorted.sort_unstable();
                assert_eq!(sorted, cut.order[exact..]);
                assert_eq!(cut.clique_sizes().count(), n);
                let td = cut.decomposition(&graph).expect("a decomposition");
                td.validate(&graph).expect("a valid decomposition");
                if exact < n {
                    assert!(
                        td.bags().iter().any(|b| b.vertices().len() == n - exact),
                        "the rest in one bag"
                    );
                }
            }
        }
    }

    /// Past its work, an elimination in a given order gives up.
    #[test]
    fn an_elimination_in_order_past_its_work_gives_up() {
        let edges: Vec<(u32, u32)> = (0..8)
            .flat_map(|u| (u + 1..8).map(move |v| (u, v)))
            .collect();
        let graph = goatd::Graph::new(8, edges.clone());
        let order: Vec<u32> = (0..8).collect();
        assert!(eliminated_in_order(&edges, order.clone(), usize::MAX, 50).is_none());
        assert!(eliminated_in_order(&edges, order.clone(), 6, u64::MAX).is_none());
        let td = eliminated_in_order(&edges, order, 7, u64::MAX)
            .and_then(|e| e.decomposition(&graph))
            .expect("no limit");
        td.validate(&graph).expect("a valid decomposition");
        assert_eq!(td.treewidth(), 7);
    }

    #[test]
    fn min_fill_takes_the_simplicial_vertices_first() {
        // A path: every elimination but the last two adds no fill, and the
        // ends are the ones with the smallest degree.
        let edges = [(0, 1), (1, 2), (2, 3), (3, 4)];
        let (order, _) = min_fill_elimination(5, &edges);
        assert!(
            order[0] == 0 || order[0] == 4,
            "an end of the path goes first, got {order:?}"
        );
    }

    /// Every vertex goes with the least fill, then degree, then number, of
    /// the vertices left, counted afresh on the graph as eliminated so far,
    /// and its parent is the first of its neighbours then to go after it.
    #[test]
    fn min_fill_takes_the_least_score_at_every_step() {
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        for _ in 0..300 {
            let n = 1 + (next() % 24) as usize;
            let density = next() % 11;
            let mut edges = Vec::new();
            for u in 0..n as u32 {
                for v in u + 1..n as u32 {
                    if next() % 10 < density {
                        edges.push(if next() % 2 == 0 { (u, v) } else { (v, u) });
                    }
                }
            }
            // A repeated edge and a self-loop change nothing.
            if let Some(&e) = edges.first() {
                edges.push(e);
                edges.push((e.0, e.0));
            }
            check_least_score(n, &edges);
        }
        // A star with chords between some of its leaves and a pendant path.
        let mut edges: Vec<(u32, u32)> = (1..40).map(|leaf| (0, leaf)).collect();
        edges.extend([(1, 2), (2, 3), (5, 9), (9, 11), (11, 5), (39, 40), (40, 41)]);
        check_least_score(42, &edges);
    }

    fn check_least_score(n: usize, edges: &[(u32, u32)]) {
        let (order, parent) = min_fill_elimination(n, edges);
        let mut adjacent = vec![vec![false; n]; n];
        for &(u, v) in edges {
            if u != v {
                adjacent[u as usize][v as usize] = true;
                adjacent[v as usize][u as usize] = true;
            }
        }
        let mut position = vec![0; n];
        for (i, &v) in order.iter().enumerate() {
            position[v as usize] = i;
        }
        let mut live = vec![true; n];
        let neighbours = |adjacent: &[Vec<bool>], live: &[bool], v: usize| -> Vec<usize> {
            (0..n).filter(|&u| live[u] && adjacent[v][u]).collect()
        };
        for &v in &order {
            let score = |u: usize| {
                let around = neighbours(&adjacent, &live, u);
                let fill = around
                    .iter()
                    .enumerate()
                    .map(|(i, &a)| around[i + 1..].iter().filter(|&&b| !adjacent[a][b]).count())
                    .sum::<usize>();
                (fill, around.len(), u)
            };
            let least = (0..n)
                .filter(|&u| live[u])
                .map(score)
                .min()
                .expect("a vertex is left");
            assert_eq!(least.2, v as usize, "{edges:?}: {order:?}");
            let around = neighbours(&adjacent, &live, v as usize);
            let first = around
                .iter()
                .copied()
                .min_by_key(|&u| position[u])
                .map_or(u32::MAX, |u| u as u32);
            assert_eq!(parent[v as usize], first, "{edges:?}");
            for &a in &around {
                for &b in &around {
                    adjacent[a][b] |= a != b;
                }
            }
            live[v as usize] = false;
        }
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..n as u32).collect::<Vec<u32>>());
    }

    /// A derived atom over many facts is a star: its leaves go first, in
    /// number order, until the centre is a leaf itself, and each leaf's
    /// parent is the centre. A rescoring of every vertex two steps from the
    /// one that goes would take the cube of the leaves over 64 word
    /// operations here, some 10^11.
    #[test]
    fn min_fill_eliminates_a_large_star_leaves_first() {
        let leaves = 20_000u32;
        let edges: Vec<(u32, u32)> = (1..=leaves).map(|leaf| (0, leaf)).collect();
        let (order, parent) = min_fill_elimination(leaves as usize + 1, &edges);
        let expected: Vec<u32> = (1..leaves).chain([0, leaves]).collect();
        assert_eq!(order, expected);
        assert!((1..leaves).all(|leaf| parent[leaf as usize] == 0));
        assert_eq!((parent[0], parent[leaves as usize]), (leaves, u32::MAX));
    }

    fn ground_source(source: &str) -> GroundProgram {
        let program = crate::syntax::parse_program(source).expect("parses");
        crate::ground::ground(&program, &crate::ground::GroundOptions::default()).expect("grounds")
    }

    fn atom(gp: &GroundProgram, name: &str) -> u32 {
        gp.atoms
            .iter()
            .position(|a| a.to_string() == name)
            .expect("an atom of the program") as u32
    }

    fn edge(u: u32, v: u32) -> (u32, u32) {
        (u.min(v), u.max(v))
    }

    /// A finding observed absent and read by nothing is two clauses, one per
    /// cause, and ties neither cause to the other; observed present it is a
    /// vertex beside both.
    #[test]
    fn an_atom_observed_false_alone_ties_no_body_to_another() {
        let rules =
            "0.3::d1. 0.4::d2. 0.5::c1. 0.6::c2. f :- d1, c1. f :- d2, c2. query(d1). query(d2).";
        let gp = ground_source(&format!("{rules} evidence(f,false)."));
        let edges: FxHashSet<(u32, u32)> = interaction_edges(&gp).into_iter().collect();
        let (d1, d2, c1, c2, f) = (
            atom(&gp, "d1"),
            atom(&gp, "d2"),
            atom(&gp, "c1"),
            atom(&gp, "c2"),
            atom(&gp, "f"),
        );
        assert!(
            edges.contains(&edge(d1, c1)) && edges.contains(&edge(d2, c2)),
            "{edges:?}"
        );
        assert!(edges.iter().all(|&(u, v)| u != f && v != f), "{edges:?}");
        assert!(!edges.contains(&edge(d1, d2)), "{edges:?}");
        let gp = ground_source(&format!("{rules} evidence(f,true)."));
        let edges: FxHashSet<(u32, u32)> = interaction_edges(&gp).into_iter().collect();
        let (d1, d2, f) = (atom(&gp, "d1"), atom(&gp, "d2"), atom(&gp, "f"));
        assert!(
            edges.contains(&edge(f, d1)) && edges.contains(&edge(f, d2)),
            "{edges:?}"
        );
    }

    /// The elimination tree's parent of a vertex is the first of its
    /// neighbours to go after it, and the pre-order finishes a subtree
    /// before it starts the next.
    #[test]
    fn the_pre_order_keeps_a_subtree_together() {
        // A star of three arms, leaf-middle-centre: leaves 0, 1, 2, middles
        // 3, 4, 5, centre 6. Min-fill takes the leaves first, then the
        // middles, so the reverse elimination order interleaves the arms.
        let edges = [(0, 3), (3, 6), (1, 4), (4, 6), (2, 5), (5, 6)];
        let (order, parent) = min_fill_elimination(7, &edges);
        assert_eq!(order, [0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(parent, [3, 4, 5, 6, 6, 6, u32::MAX]);
        assert_eq!(pre_order(&order, &parent), [6, 5, 2, 4, 1, 3, 0]);
    }

    /// The interaction graph holds one clique per rule.
    #[test]
    fn interaction_graph_has_a_clique_per_rule() {
        let gp = ground_smokers();
        let edges: FxHashSet<(u32, u32)> = interaction_edges(&gp).into_iter().collect();
        for r in &gp.rules {
            let mut atoms: Vec<AtomId> = std::iter::once(r.head)
                .chain(r.body.iter().map(|l| l.atom()))
                .collect();
            atoms.sort_unstable();
            atoms.dedup();
            for (i, &u) in atoms.iter().enumerate() {
                for &v in &atoms[i + 1..] {
                    assert!(edges.contains(&(u, v)), "missing edge {u}-{v}");
                }
            }
        }
    }
}
