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

/// The graph a min-fill elimination runs on, its vertices weighted: the
/// neighbours of each vertex, the weight of the pairs of them and that of
/// the edges joining two of them, the triangles through it, so that a
/// vertex's fill, the weight of the pairs of its neighbours not yet
/// adjacent, is read off the two rather than recounted. A pair's or an
/// edge's weight is the product of its ends'; at weight one each, the
/// pairs are the degree's and the triangles are counted.
struct FillGraph {
    adjacent: Vec<FxHashSet<u32>>,
    weight: Vec<u64>,
    /// The weight of each vertex's neighbours, and of the pairs of them.
    around: Vec<u64>,
    pairs: Vec<u64>,
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
    /// The graph whose vertex `v` weighs `weight[v]`, with its triangles
    /// counted, or `None` where counting them takes more than `max_work`
    /// units. The weights are summed and multiplied exactly: those of a
    /// vertex's neighbours must sum to less than 2^32.
    fn new(edges: &[(u32, u32)], weight: &[u64], max_work: u64) -> Option<Self> {
        let num_vertices = weight.len();
        let mut adjacent: Vec<FxHashSet<u32>> = vec![FxHashSet::default(); num_vertices];
        for &(u, v) in edges {
            if u != v && adjacent[u as usize].insert(v) {
                adjacent[v as usize].insert(u);
            }
        }
        let work = 2 * edges.len() as u64;
        let (around, pairs) = adjacent
            .iter()
            .map(|set| {
                let (sum, squares) = set.iter().fold((0u64, 0u64), |(s, q), &u| {
                    let w = weight[u as usize];
                    (s + w, q + w * w)
                });
                (sum, (sum * sum - squares) / 2)
            })
            .unzip();
        let mut graph = FillGraph {
            adjacent,
            weight: weight.to_vec(),
            around,
            pairs,
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
                    let common: u64 = common(&graph.adjacent, u, w)
                        .map(|c| graph.weight[c as usize])
                        .sum();
                    graph.triangles[u as usize] += graph.weight[w as usize] * common;
                    graph.triangles[w as usize] += graph.weight[u as usize] * common;
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
        (
            self.pairs[v as usize] - self.triangles[v as usize],
            self.adjacent[v as usize].len() as u32,
            v,
        )
    }

    /// Eliminates `v`, whose fill is `fill`: its neighbours, left in
    /// `neighbours`, become a clique and it leaves the graph. `touched`
    /// gets, once each, the vertices whose score this changes: the
    /// neighbours, whose degree and triangles change, and the common
    /// neighbours of the two ends of a fill edge, whose triangles do. No
    /// other vertex's neighbours or the edges among them change. The pairs
    /// of neighbours are searched for the missing edges only until edges
    /// of weight `fill` are found, and not at all where `v` is simplicial.
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
                let (wa, wb) = (self.weight[a as usize], self.weight[b as usize]);
                missing -= wa * wb;
                let (da, db) = (
                    self.adjacent[a as usize].len(),
                    self.adjacent[b as usize].len(),
                );
                self.work += da.min(db) as u64 + 2;
                // `v` is a common neighbour too: the edge closes a triangle
                // through it, which goes with it below.
                let mut closed = 0u64;
                for w in common(&self.adjacent, a, b) {
                    closed += self.weight[w as usize];
                    self.triangles[w as usize] += wa * wb;
                    if w != v && self.touched_at[w as usize] != self.eliminations {
                        self.touched_at[w as usize] = self.eliminations;
                        touched.push(w);
                    }
                }
                self.triangles[a as usize] += wb * closed;
                self.triangles[b as usize] += wa * closed;
                self.pairs[a as usize] += wb * self.around[a as usize];
                self.pairs[b as usize] += wa * self.around[b as usize];
                self.around[a as usize] += wb;
                self.around[b as usize] += wa;
                self.adjacent[a as usize].insert(b);
                self.adjacent[b as usize].insert(a);
            }
        }
        // With the neighbourhood a clique, each neighbour loses `v`, the
        // pairs of `v` and each of its other neighbours, and the triangles
        // through `v` and each of the other neighbours of `v`.
        let wv = self.weight[v as usize];
        let around_v = self.around[v as usize];
        self.work += neighbours.len() as u64;
        for &a in neighbours.iter() {
            self.adjacent[a as usize].remove(&v);
            self.around[a as usize] -= wv;
            self.pairs[a as usize] -= wv * self.around[a as usize];
            self.triangles[a as usize] -= wv * (around_v - self.weight[a as usize]);
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
///
/// The bound cannot pass the width of a decomposition, so the contractions
/// stop where it reaches `width`, that of one at hand. The least degree,
/// the first bound, is read off the edges, which are distinct: on the
/// clique of a rule body of `k` atoms it is already `k - 1`, and the
/// contractions would cost `k^2 log k`.
pub(super) fn minor_min_width(num_vertices: usize, edges: &[(u32, u32)], width: usize) -> usize {
    let mut degree = vec![0; num_vertices];
    for &(u, v) in edges {
        if u != v {
            degree[u as usize] += 1;
            degree[v as usize] += 1;
        }
    }
    let least = degree.into_iter().min().unwrap_or(0);
    if least >= width {
        return least;
    }
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
        if bound >= width {
            break;
        }
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
    min_fill_cliques(gp.num_atoms(), &interaction_edges(gp)).states(&vec![1.0; gp.num_atoms()])
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
    /// log2 of the states of the elimination's cliques summed, a clique's
    /// the product of its vertices' and vertex `v` taking `2^bits[v]`:
    /// the entries variable elimination over the graph's variables would
    /// build in tables. The vertices the elimination left are one clique
    /// with each of those after it, in the order. At least 0, and 0 for a
    /// graph without vertices.
    pub(super) fn states(&self, bits: &[f64]) -> f64 {
        let n = self.order.len();
        let exact = self.at_elimination.len();
        let mut rest = vec![0.0; n - exact];
        for i in (exact..n).rev() {
            rest[i - exact] =
                bits[self.order[i] as usize] + rest.get(i + 1 - exact).copied().unwrap_or(0.0);
        }
        self.order
            .iter()
            .zip(&self.at_elimination)
            .map(|(&v, around)| {
                bits[v as usize] + around.iter().map(|&u| bits[u as usize]).sum::<f64>()
            })
            .chain(rest)
            .fold(f64::NEG_INFINITY, log2_add)
            .max(0.0)
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
    min_fill_within(edges, &vec![1; num_vertices], 2 * ELIMINATION_BUDGET_UNITS)
}

/// [`min_fill_cliques`] on the graph whose vertex `v` weighs `weight[v]`:
/// a vertex's fill is the weight of the edges eliminating it adds, an
/// edge's the product of its ends' weights. Where a vertex is a variable
/// and its weight the values it takes, the fill is the entries of the
/// tables the edges would add between the variables. Counting each edge
/// as one, min-fill eliminates a variable of 2 values as early as one of
/// 81, and so a network whose variables take from 2 values to 81 (the
/// proteins of the UAI 2022 PR set) as if they all took the same.
pub(super) fn weighted_min_fill(edges: &[(u32, u32)], weight: &[u64]) -> Elimination {
    min_fill_within(edges, weight, 2 * ELIMINATION_BUDGET_UNITS)
}

/// The elimination of the vertices of the graph whose edges are `edges`
/// in the order `order`, every vertex one at a time. `None` when a vertex
/// would go with more than `width` neighbours, or when eliminating takes
/// more than `max_work` units of work (two for each edge, and the square
/// of each vertex's neighbours when it goes), which is charged to goatd's
/// meter. The edges join distinct vertices.
///
/// Each vertex that goes leaves a clique of its neighbours, which takes in
/// the cliques it was in; a vertex's neighbours when it goes are its own
/// left and those of the cliques it is in, each clique read once, by the
/// first of its vertices to go, which takes time linear in the edges of
/// the eliminated graph.
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
    let n = order.len();
    let mut position = vec![0u32; n];
    for (i, &v) in order.iter().enumerate() {
        position[v as usize] = i as u32;
    }
    // The graph by its vertices' places in the order, as lists that start
    // at `start`.
    let mut start = vec![0usize; n + 1];
    for &(u, v) in edges {
        start[position[u as usize] as usize + 1] += 1;
        start[position[v as usize] as usize + 1] += 1;
    }
    for i in 0..n {
        start[i + 1] += start[i];
    }
    let mut next = start.clone();
    let mut adjacent = vec![0u32; 2 * edges.len()];
    for &(u, v) in edges {
        let (pu, pv) = (position[u as usize], position[v as usize]);
        adjacent[next[pu as usize]] = pv;
        next[pu as usize] += 1;
        adjacent[next[pv as usize]] = pu;
        next[pv as usize] += 1;
    }
    drop(next);
    // Per place: the cliques its vertex is in, by the place of the vertex
    // that left them; per clique, its vertices, and the clique that took it
    // in (itself while it stands).
    let mut cliques_of: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut members: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut taken_by: Vec<u32> = (0..n as u32).collect();
    let root = |taken_by: &mut [u32], mut c: u32| -> u32 {
        while taken_by[c as usize] != c {
            let up = taken_by[taken_by[c as usize] as usize];
            taken_by[c as usize] = up;
            c = up;
        }
        c
    };
    // The step at which each place and clique was last gathered.
    let mut seen = vec![u32::MAX; n];
    let mut read = vec![u32::MAX; n];
    let mut around: Vec<u32> = Vec::new();
    let mut work = 2 * edges.len() as u64;
    let mut at_elimination = Vec::with_capacity(n);
    for i in 0..n as u32 {
        around.clear();
        seen[i as usize] = i;
        for &u in &adjacent[start[i as usize]..start[i as usize + 1]] {
            if u > i && seen[u as usize] != i {
                seen[u as usize] = i;
                around.push(u);
            }
        }
        for c in std::mem::take(&mut cliques_of[i as usize]) {
            let c = root(&mut taken_by, c);
            if read[c as usize] == i {
                continue;
            }
            read[c as usize] = i;
            for u in std::mem::take(&mut members[c as usize]) {
                if u > i && seen[u as usize] != i {
                    seen[u as usize] = i;
                    around.push(u);
                }
            }
            taken_by[c as usize] = i;
        }
        if around.len() > width {
            goatd::meter::charge(work);
            return None;
        }
        work += (around.len() * around.len()) as u64;
        if work > max_work {
            goatd::meter::charge(work);
            return None;
        }
        for &u in &around {
            cliques_of[u as usize].push(i);
        }
        let mut bag: Vec<u32> = around.iter().map(|&u| order[u as usize]).collect();
        bag.sort_unstable();
        at_elimination.push(bag);
        members[i as usize] = around.clone();
    }
    goatd::meter::charge(work);
    Some(Elimination {
        order,
        at_elimination,
    })
}

/// [`weighted_min_fill`], stopped past `max_work` units.
fn min_fill_within(edges: &[(u32, u32)], weight: &[u64], max_work: u64) -> Elimination {
    let num_vertices = weight.len();
    let mut order = Vec::with_capacity(num_vertices);
    let mut at_elimination: Vec<Vec<u32>> = Vec::with_capacity(num_vertices);
    let mut eliminated = vec![false; num_vertices];
    let mut work = 0;
    if let Some(mut graph) = FillGraph::new(edges, weight, max_work) {
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

/// Vertices beyond which [`annealed`] does not search: it holds the graph
/// as a matrix of bits.
pub(super) const ANNEAL_MAX_VERTICES: usize = 4096;
/// Moves [`annealed`] tries at most, and the temperature schedule of its
/// annealing, in bits of the log2 states: a move that adds `d` bits is
/// taken with odds `e^(-d/t)`, `t` falling geometrically from the first to
/// the last over the moves the budget pays for.
const ANNEAL_MAX_MOVES: usize = 100_000;
const ANNEAL_T_START: f64 = 1.0;
const ANNEAL_T_END: f64 = 0.02;
/// How far a move takes a vertex most of the time ([`ANNEAL_NEAR`] of the
/// moves); the others take it anywhere. An elimination improves by
/// reordering a few neighbouring vertices at a time.
const ANNEAL_REACH: usize = 8;
const ANNEAL_NEAR: f64 = 0.7;
/// Moves per vertex the budget must pay for, at the start's evaluation,
/// for the search to run: fewer leave the walk hot near where it started.
/// On the UAI PR sets' proteins (300 to 500 groups) the budget pays for
/// a hundred moves a vertex or more and finds layouts up to 25 times
/// cheaper; on their grids and rus networks (over 1000 groups) it paid for
/// 5 to 20 and found no cheaper layout in 2 to 3 s.
const ANNEAL_MIN_MOVES_PER_VERTEX: u64 = 32;
/// The pairs of a layout's estimate for each unit [`annealed`] may spend
/// on it: the search is paid for by what it can save, so a cheap layout
/// is not searched.
const ANNEAL_PAIRS_PER_UNIT: f64 = 8.0;

/// The budget [`annealed`] gets on a graph of `num_vertices` vertices and
/// `num_edges` edges whose layout is estimated at `pairs`: a unit for each
/// [`ANNEAL_PAIRS_PER_UNIT`] pairs, up to twice [`ELIMINATION_BUDGET_UNITS`],
/// what an elimination spends before it gives up. `None` where the search
/// does not run: past [`ANNEAL_MAX_VERTICES`] vertices, or where the
/// budget does not pay for [`ANNEAL_MIN_MOVES_PER_VERTEX`] moves a vertex
/// at the least an evaluation costs, a word for each vertex and edge and
/// one more for each vertex.
pub(super) fn anneal_budget(num_vertices: usize, num_edges: usize, pairs: f64) -> Option<u64> {
    if !(2..=ANNEAL_MAX_VERTICES).contains(&num_vertices) {
        return None;
    }
    let budget = ((pairs / ANNEAL_PAIRS_PER_UNIT) as u64).min(2 * ELIMINATION_BUDGET_UNITS);
    let words = num_vertices.div_ceil(64) as u64;
    let evaluation = words * (2 * num_vertices + num_edges) as u64;
    (budget >= ANNEAL_MIN_MOVES_PER_VERTEX * num_vertices as u64 * evaluation).then_some(budget)
}

/// The elimination order `start` of the graph whose vertex `v` takes
/// `2^bits[v]` states, improved by simulated annealing on its states
/// ([`Elimination::states`]): a move takes one vertex to another place in
/// the order, and is taken where its states are lower or with the
/// annealing's odds. `None` where the graph has more than
/// [`ANNEAL_MAX_VERTICES`] vertices, where `budget` units do not pay for
/// [`ANNEAL_MIN_MOVES_PER_VERTEX`] moves a vertex, or where no order
/// beats `start`. The work, a word of the matrix read or written per unit,
/// is charged to goatd's meter and stops at `budget`; the generator is
/// seeded, so the result does not depend on the machine.
pub(super) fn annealed(
    edges: &[(u32, u32)],
    bits: &[f64],
    start: &[u32],
    budget: u64,
) -> Option<Vec<u32>> {
    let n = start.len();
    if !(2..=ANNEAL_MAX_VERTICES).contains(&n) {
        return None;
    }
    let words = n.div_ceil(64);
    let mut matrix = vec![0u64; n * words];
    for &(u, v) in edges {
        if u != v {
            matrix[u as usize * words + v as usize / 64] |= 1 << (v % 64);
            matrix[v as usize * words + u as usize / 64] |= 1 << (u % 64);
        }
    }
    let mut eval = Evaluation {
        matrix,
        scratch: vec![0; n * words],
        row: vec![0; words],
        around: Vec::new(),
        words,
        work: 0,
    };
    let mut order = start.to_vec();
    let mut cost = eval.states(bits, &order, f64::INFINITY)?;
    // The moves the budget pays for at the start's evaluation; a move that
    // is not taken stops part way, and costs less.
    let planned = ANNEAL_MAX_MOVES.min((budget / eval.work.max(1)) as usize);
    if (planned as u64) < ANNEAL_MIN_MOVES_PER_VERTEX * n as u64 {
        goatd::meter::charge(eval.work);
        return None;
    }
    let start_cost = cost;
    let mut best = order.clone();
    let mut best_cost = cost;
    let mut rng = super::Rng(0x9E37_79B9_7F4A_7C15);
    let mut moves = 0;
    while moves < planned && eval.work < budget {
        moves += 1;
        let t =
            ANNEAL_T_START * (ANNEAL_T_END / ANNEAL_T_START).powf(moves as f64 / planned as f64);
        let i = rng.below(n);
        let reach = if rng.unit() < ANNEAL_NEAR {
            ANNEAL_REACH
        } else {
            n
        };
        let (lo, hi) = (i.saturating_sub(reach), (i + reach).min(n - 1));
        let j = lo + rng.below(hi - lo + 1);
        if j == i {
            continue;
        }
        let rotate = |order: &mut [u32], forward: bool| match (i < j, forward) {
            (true, true) | (false, false) => order[i.min(j)..=i.max(j)].rotate_left(1),
            _ => order[i.min(j)..=i.max(j)].rotate_right(1),
        };
        // Taken where its states are under the current ones plus t times
        // the log of a uniform draw's inverse: the annealing's odds, with
        // which the evaluation can stop as soon as it passes them.
        let limit = cost - t * rng.unit().max(f64::MIN_POSITIVE).ln();
        rotate(&mut order, true);
        match eval.states(bits, &order, limit) {
            Some(c) => {
                cost = c;
                if c < best_cost {
                    best_cost = c;
                    best.copy_from_slice(&order);
                }
            }
            None => rotate(&mut order, false),
        }
    }
    goatd::meter::charge(eval.work);
    (best_cost < start_cost).then_some(best)
}

/// [`annealed`]'s evaluation of an order: the graph as a matrix of bits,
/// a row a vertex, and the buffers an elimination works in.
struct Evaluation {
    matrix: Vec<u64>,
    scratch: Vec<u64>,
    row: Vec<u64>,
    around: Vec<u32>,
    words: usize,
    work: u64,
}

impl Evaluation {
    /// The states of eliminating in `order` ([`Elimination::states`]), or
    /// `None` as soon as they pass `limit`.
    fn states(&mut self, bits: &[f64], order: &[u32], limit: f64) -> Option<f64> {
        let words = self.words;
        self.scratch.copy_from_slice(&self.matrix);
        self.work += self.scratch.len() as u64;
        let mut total = f64::NEG_INFINITY;
        for &v in order {
            let at = v as usize * words;
            self.row.copy_from_slice(&self.scratch[at..at + words]);
            self.around.clear();
            for (w, &x) in self.row.iter().enumerate() {
                let mut x = x;
                while x != 0 {
                    self.around.push((w * 64) as u32 + x.trailing_zeros());
                    x &= x - 1;
                }
            }
            let clique =
                bits[v as usize] + self.around.iter().map(|&u| bits[u as usize]).sum::<f64>();
            total = log2_add(total, clique);
            self.work += (words * (self.around.len() + 1)) as u64;
            if total > limit {
                return None;
            }
            // The neighbours become a clique, and `v` leaves the graph.
            for &u in &self.around {
                let at_u = u as usize * words;
                for (w, &x) in self.row.iter().enumerate() {
                    self.scratch[at_u + w] |= x;
                }
                self.scratch[at_u + u as usize / 64] &= !(1 << (u % 64));
                self.scratch[at_u + v as usize / 64] &= !(1 << (v % 64));
            }
        }
        Some(total.max(0.0))
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
        assert_eq!(minor_min_width(6, &clique(6), usize::MAX), 5);
        let cycle: Vec<(u32, u32)> = (0..8).map(|u| (u, (u + 1) % 8)).collect();
        assert_eq!(minor_min_width(8, &cycle, usize::MAX), 2);
        let path: Vec<(u32, u32)> = (0..7).map(|u| (u, u + 1)).collect();
        assert_eq!(minor_min_width(8, &path, usize::MAX), 1);
        assert_eq!(minor_min_width(3, &[], usize::MAX), 0);
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
        let bound = minor_min_width(16, &grid, usize::MAX);
        assert!((2..=4).contains(&bound), "{bound}");
        // Stopped at the width of a decomposition, the bound is the same:
        // it cannot pass that width.
        assert_eq!(minor_min_width(6, &clique(6), 5), 5);
        assert_eq!(minor_min_width(16, &grid, 4), bound);
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
            let whole = min_fill_within(&edges, &vec![1; n], u64::MAX);
            assert_eq!(whole.at_elimination.len(), n);
            let graph = goatd::Graph::new(n as u32, edges.clone());
            for max_work in [0, 10, 50, 100, 200, 400, 800] {
                let cut = min_fill_within(&edges, &vec![1; n], max_work);
                let exact = cut.at_elimination.len();
                assert_eq!(cut.order[..exact], whole.order[..exact]);
                assert_eq!(cut.at_elimination, whole.at_elimination[..exact]);
                let mut sorted = cut.order[exact..].to_vec();
                sorted.sort_unstable();
                assert_eq!(sorted, cut.order[exact..]);
                // The rest as one clique holds at least what eliminating
                // it would: the i-th vertex left has at most the others
                // after it as neighbours.
                let ones = vec![1.0; n];
                assert!(cut.states(&ones) >= whole.states(&ones));
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
    /// The elimination in order as it was first written, each vertex's
    /// neighbours added to each other's sets: the reference for
    /// [`eliminated_in_order`].
    fn eliminated_in_order_by_sets(
        edges: &[(u32, u32)],
        order: Vec<u32>,
        width: usize,
        max_work: u64,
    ) -> (Option<Elimination>, u64) {
        let mut adjacent: Vec<FxHashSet<u32>> = vec![FxHashSet::default(); order.len()];
        for &(u, v) in edges {
            adjacent[u as usize].insert(v);
            adjacent[v as usize].insert(u);
        }
        let mut work = 2 * edges.len() as u64;
        let mut at_elimination = Vec::with_capacity(order.len());
        for &v in &order {
            let mut neighbours: Vec<u32> = std::mem::take(&mut adjacent[v as usize])
                .into_iter()
                .collect();
            if neighbours.len() > width {
                return (None, work);
            }
            for &u in &neighbours {
                let set = &mut adjacent[u as usize];
                set.remove(&v);
                set.extend(neighbours.iter().copied().filter(|&w| w != u));
            }
            work += (neighbours.len() * neighbours.len()) as u64;
            if work > max_work {
                return (None, work);
            }
            neighbours.sort_unstable();
            at_elimination.push(neighbours);
        }
        (
            Some(Elimination {
                order,
                at_elimination,
            }),
            work,
        )
    }

    #[test]
    fn an_elimination_in_order_matches_the_one_by_sets() {
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut draw = |m: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % m
        };
        for round in 0..400 {
            let n = 1 + draw(60) as u32;
            let density = 1 + draw(8);
            let mut edges = Vec::new();
            for u in 0..n {
                for v in u + 1..n {
                    if draw(16) < density {
                        edges.push(if draw(2) == 0 { (u, v) } else { (v, u) });
                    }
                }
            }
            if round % 5 == 0 && !edges.is_empty() {
                let e = edges[draw(edges.len() as u64) as usize];
                edges.push(e);
            }
            let mut order: Vec<u32> = (0..n).collect();
            for i in (1..n as usize).rev() {
                order.swap(i, draw(i as u64 + 1) as usize);
            }
            let width = if round % 3 == 0 {
                draw(n as u64 + 1) as usize
            } else {
                usize::MAX
            };
            let max_work = if round % 4 == 0 {
                draw(4 * n as u64 * n as u64 + 1)
            } else {
                u64::MAX
            };
            let (want, want_work) =
                eliminated_in_order_by_sets(&edges, order.clone(), width, max_work);
            let before = goatd::meter::units_spent();
            let got = eliminated_in_order(&edges, order, width, max_work);
            assert_eq!(
                goatd::meter::units_spent() - before,
                want_work,
                "round {round}"
            );
            match (want, got) {
                (None, None) => {}
                (Some(want), Some(got)) => {
                    assert_eq!(want.order, got.order, "round {round}");
                    assert_eq!(want.at_elimination, got.at_elimination, "round {round}");
                }
                (want, got) => panic!(
                    "round {round}: by sets {:?}, in order {:?}",
                    want.is_some(),
                    got.is_some()
                ),
            }
        }
    }

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

    /// A graph's edges and its vertices' weights.
    type WeightedGraph = (Vec<(u32, u32)>, Vec<u64>);

    /// Random graphs of up to `max_n` vertices, about 40% of the pairs
    /// joined, each vertex weighing from 1 to 6.
    fn random_weighted_graphs(count: usize, max_n: u64) -> Vec<WeightedGraph> {
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        (0..count)
            .map(|_| {
                let n = 2 + (next() % max_n) as u32;
                let mut edges = Vec::new();
                for u in 0..n {
                    for v in u + 1..n {
                        if next() % 10 < 4 {
                            edges.push((u, v));
                        }
                    }
                }
                let weight = (0..n).map(|_| 1 + next() % 6).collect();
                (edges, weight)
            })
            .collect()
    }

    /// At equal weights, weighted min-fill is min-fill; where the weights
    /// differ, it fills in the edges between the light vertices first. On
    /// the cycle 0-1-2-3 whose odd vertices weigh 81, min-fill takes 0
    /// first, joining the heavy pair, and weighted min-fill takes 1.
    #[test]
    fn weighted_min_fill_at_equal_weights_is_min_fill() {
        for (edges, weight) in random_weighted_graphs(40, 30) {
            let n = weight.len();
            let plain = min_fill_cliques(n, &edges);
            let equal = weighted_min_fill(&edges, &vec![3; n]);
            assert_eq!(plain.order, equal.order);
        }
        let cycle = [(0, 1), (1, 2), (2, 3), (0, 3)];
        assert_eq!(min_fill_cliques(4, &cycle).order[0], 0);
        assert_eq!(weighted_min_fill(&cycle, &[2, 81, 2, 81]).order[0], 1);
    }

    /// The annealing returns an order of the same vertices whose states
    /// are below the start's, and its elimination decomposes the graph;
    /// a budget that does not pay for the moves it needs runs no search.
    #[test]
    fn annealing_lowers_the_states_of_an_order() {
        let mut improved = 0;
        for (edges, weight) in random_weighted_graphs(30, 40) {
            let n = weight.len();
            let bits: Vec<f64> = weight.iter().map(|&w| (w as f64).log2()).collect();
            let start: Vec<u32> = (0..n as u32).collect();
            let start_states = eliminated_in_order(&edges, start.clone(), usize::MAX, u64::MAX)
                .unwrap()
                .states(&bits);
            assert!(annealed(&edges, &bits, &start, 10).is_none());
            let Some(order) = annealed(&edges, &bits, &start, 1 << 22) else {
                continue;
            };
            improved += 1;
            let mut sorted = order.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, start);
            let elimination = eliminated_in_order(&edges, order, usize::MAX, u64::MAX).unwrap();
            assert!(elimination.states(&bits) < start_states);
            let graph = goatd::Graph::new(n as u32, edges.clone());
            let td = elimination.decomposition(&graph).expect("a decomposition");
            td.validate(&graph).expect("a valid decomposition");
        }
        assert!(improved > 10, "{improved}");
    }

    /// The annealing's budget grows with the layout's pairs, and is none
    /// where it does not pay for its moves or the graph is past the
    /// matrix's size.
    #[test]
    fn the_annealing_budget_pays_for_its_moves() {
        assert_eq!(anneal_budget(100, 300, 1e3), None);
        let budget = anneal_budget(100, 300, 1e12).expect("a large layout is searched");
        assert!(budget <= 2 * ELIMINATION_BUDGET_UNITS);
        assert_eq!(anneal_budget(ANNEAL_MAX_VERTICES + 1, 0, 1e30), None);
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
    /// and its parent is the first of its neighbours then to go after it;
    /// on weighted vertices the fill is weighted.
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
            check_least_score(&edges, &vec![1; n]);
            // Weighted, the fill is the weight of the pairs missing.
            let weight: Vec<u64> = (0..n).map(|_| 1 + next() % 81).collect();
            check_least_score(&edges, &weight);
        }
        // A star with chords between some of its leaves and a pendant path.
        let mut edges: Vec<(u32, u32)> = (1..40).map(|leaf| (0, leaf)).collect();
        edges.extend([(1, 2), (2, 3), (5, 9), (9, 11), (11, 5), (39, 40), (40, 41)]);
        check_least_score(&edges, &[1; 42]);
    }

    /// At weight one, [`min_fill_elimination`]'s order, which is
    /// [`weighted_min_fill`]'s, and its parents are checked too.
    fn check_least_score(edges: &[(u32, u32)], weight: &[u64]) {
        let n = weight.len();
        let order = weighted_min_fill(edges, weight).order;
        let parent = weight.iter().all(|&w| w == 1).then(|| {
            let (plain, parent) = min_fill_elimination(n, edges);
            assert_eq!(plain, order);
            parent
        });
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
                    .map(|(i, &a)| {
                        around[i + 1..]
                            .iter()
                            .filter(|&&b| !adjacent[a][b])
                            .map(|&b| weight[a] * weight[b])
                            .sum::<u64>()
                    })
                    .sum::<u64>();
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
            if let Some(parent) = &parent {
                assert_eq!(parent[v as usize], first, "{edges:?}");
            }
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
