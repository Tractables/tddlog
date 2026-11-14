//! The interaction graph of a ground program and its eliminations, and the
//! fact orders read off the program.
//!
//! The eliminations, goatd's traced ones (`goatd::elimination::trace`):
//! min-fill, plain or over weighted vertices ([`min_fill_trace`]), and an
//! elimination in a given order ([`eliminated_in_order`]); their states
//! ([`states`]); an order improved by simulated annealing on
//! its cliques' states ([`annealed`]); and the minor-min-width lower bound
//! on the treewidth ([`minor_min_width`]). The fact orders: the order the
//! compiler consumes the facts in ([`facts_by_use`], [`facts_by_level`])
//! and a pre-order of the min-fill elimination tree
//! ([`facts_by_minfill`]).

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use goatd::elimination::{Order, RunConfig, Trace, TraceOutcome};
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

/// Fact atoms in the order the compiler consumes them, up to the order of
/// components that do not depend on each other: [`facts_at_first_use`]
/// over the components level by level, a component's level the longest
/// chain of components it reads, and within a level in dependency order.
///
/// The dependency order is a depth-first search's, started from the atoms
/// in the order the grounder numbered them, and so it follows how the
/// program is written. Two netlists of one circuit that list one output
/// word's bits in opposite orders use the gates of its carry chain first
/// in different orders, while each gate's level is the same in both.
pub(crate) fn facts_by_level(gp: &GroundProgram, deps: &Deps) -> Vec<AtomId> {
    let mut component = vec![usize::MAX; gp.num_atoms()];
    for (c, scc) in deps.sccs.iter().enumerate() {
        for &a in scc {
            component[a as usize] = c;
        }
    }
    // The components come after those they read, so one pass in their
    // order sets every level.
    let mut level = vec![0usize; deps.sccs.len()];
    for (c, scc) in deps.sccs.iter().enumerate() {
        let above = scc
            .iter()
            .flat_map(|&h| &deps.by_head[h as usize])
            .flat_map(|&ri| &gp.rules[ri].body)
            .map(|l| component[l.atom() as usize])
            .filter(|&d| d != usize::MAX && d != c)
            .map(|d| level[d] + 1)
            .max();
        level[c] = above.unwrap_or(0);
    }
    let mut order: Vec<usize> = (0..deps.sccs.len()).collect();
    order.sort_by_key(|&c| level[c]);
    facts_at_first_use(gp, &deps.by_head, order.iter().map(|&c| &deps.sccs[c]))
}

/// Fact atoms at their first use by the components taken in the given
/// order; unused facts last, in program order.
pub(super) fn facts_at_first_use<'a>(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    components: impl IntoIterator<Item = &'a Vec<AtomId>>,
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

/// The deduplicated undirected edges `(u, v)` with `u < v` of `cliques`.
pub(super) fn edges_of_cliques(cliques: &[Vec<usize>]) -> Vec<(usize, usize)> {
    let mut seen = FxHashSet::default();
    let mut edges = Vec::new();
    for clique in cliques {
        for (i, &u) in clique.iter().enumerate() {
            for &v in &clique[i + 1..] {
                if u != v {
                    let e = (u.min(v), u.max(v));
                    if seen.insert(e) {
                        edges.push(e);
                    }
                }
            }
        }
    }
    edges
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
    let alone = observed_false_alone(gp);
    let cliques: Vec<Vec<usize>> = gp
        .rules
        .iter()
        .flat_map(|r| r.bodies().map(|body| (r.head, body)))
        .map(|(head, body)| {
            let mut clique: Vec<usize> = (!alone[head as usize])
                .then_some(head as usize)
                .into_iter()
                .chain(body.iter().map(|l| l.atom() as usize))
                .collect();
            clique.sort_unstable();
            clique.dedup();
            clique
        })
        .collect();
    edges_of_cliques(&cliques)
        .into_iter()
        .map(|(u, v)| (u as u32, v as u32))
        .collect()
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

/// The weight of a min-fill elimination of the program's interaction
/// graph, as log2 of the sum over its eliminations of 2^k, k the vertex
/// and its neighbours when it goes: the entries of the tables an
/// elimination over Boolean atoms would build, a measure of what
/// compiling the program costs that tells programs whose widths differ
/// by a few vertices apart. Zero for a program without atoms.
pub(crate) fn min_fill_weight(gp: &GroundProgram) -> f64 {
    let graph = goatd::Graph::new(gp.num_atoms() as u32, interaction_edges(gp));
    let trace = min_fill_trace(&graph, None).expect("min-fill takes every graph");
    states(&trace, &vec![1.0; gp.num_atoms()])
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
/// twice it they put the vertices left in one bag. The traced min-fill
/// ([`min_fill_trace`]) takes twice it as both budgets, and so stays exact until
/// it stops, and the eliminations in a given order
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

/// goatd's min-fill elimination of `graph` (`goatd::elimination::trace`),
/// over every vertex, ties going by degree and then by number, and its fill
/// weighted by `weights` where they are given: a vertex's fill is the weight
/// of the edges eliminating it adds, an edge's the product of its ends'
/// weights. Where a vertex is a variable and its weight the values it takes,
/// the fill is the entries of the tables the edges would add between the
/// variables; counting each edge as one, min-fill eliminates a variable of 2
/// values as early as one of 81, and so a network whose variables take from
/// 2 values to 81 (the proteins of the UAI 2022 PR set) as if they all took
/// the same.
///
/// Past twice [`ELIMINATION_BUDGET_UNITS`] of work, on goatd's meter, the
/// vertices left go as one clique, in the order of their numbers. `None`
/// where goatd refuses the weights (one under 1, or a sum of 2^32 or more).
pub(super) fn min_fill_trace(graph: &goatd::Graph, weights: Option<&[u64]>) -> Option<Trace> {
    let ranks: Vec<u32> = (0..graph.num_vertices()).collect();
    let order = match weights {
        Some(weights) => Order::MinFillWeighted {
            ranks: &ranks,
            weights,
        },
        None => Order::MinFillRanked { ranks: &ranks },
    };
    traced(graph, order, true)
}

/// The elimination of `graph`'s vertices in `order`, every vertex one at a
/// time (`goatd::elimination::trace` with `Order::Ranked`). `None` when a
/// vertex would go with more than `width` neighbours, or past twice
/// [`ELIMINATION_BUDGET_UNITS`] of work.
pub(super) fn eliminated_in_order(
    graph: &goatd::Graph,
    order: &[u32],
    width: usize,
) -> Option<Trace> {
    let mut ranks = vec![0u32; order.len()];
    for (i, &v) in order.iter().enumerate() {
        ranks[v as usize] = i as u32;
    }
    let width = u32::try_from(width).ok();
    traced_within(graph, Order::Ranked { ranks: &ranks }, false, width)
}

fn traced(graph: &goatd::Graph, order: Order<'_>, complete: bool) -> Option<Trace> {
    traced_within(graph, order, complete, None)
}

/// goatd's trace of `order` on `graph` within twice
/// [`ELIMINATION_BUDGET_UNITS`] and `width`, the vertices left at the budget
/// going as one clique where `complete`. The budget is read on goatd's meter,
/// armed here where the caller has not.
fn traced_within(
    graph: &goatd::Graph,
    order: Order<'_>,
    complete: bool,
    width: Option<u32>,
) -> Option<Trace> {
    let _meter = (!goatd::meter::is_armed()).then(|| goatd::meter::arm(Instant::now()));
    let budget = Some(2 * elimination_budget());
    let config = RunConfig::default()
        .with_budgets(budget, budget)
        .with_width_bound(width)
        .with_deadline_completion(complete);
    match goatd::elimination::trace(graph, order, 0, config).ok()? {
        TraceOutcome::Completed(trace) | TraceOutcome::CompletedAtDeadline(_, trace) => Some(trace),
        _ => None,
    }
}

/// log2 of the states of an elimination's cliques summed, a clique's the
/// product of its vertices' and vertex `v` taking `2^bits[v]`: the entries
/// variable elimination over the graph's variables would build in tables.
/// The vertices the elimination left are one clique with each of those
/// after it, in the order. At least 0, and 0 for a graph without vertices.
pub(super) fn states(trace: &Trace, bits: &[f64]) -> f64 {
    let (order, at_elimination) = (trace.order(), trace.neighbours());
    let n = order.len();
    let exact = at_elimination.len();
    let mut rest = vec![0.0; n - exact];
    for i in (exact..n).rev() {
        rest[i - exact] = bits[order[i] as usize] + rest.get(i + 1 - exact).copied().unwrap_or(0.0);
    }
    order
        .iter()
        .zip(at_elimination)
        .map(|(&v, around)| {
            bits[v as usize] + around.iter().map(|&u| bits[u as usize]).sum::<f64>()
        })
        .chain(rest)
        .fold(f64::NEG_INFINITY, log2_add)
        .max(0.0)
}

/// An elimination's order and its elimination tree: the parent of each
/// vertex (`u32::MAX` at a root) is, of its neighbours when it goes, the
/// first to go after it. The vertices the elimination left go as one
/// clique, each the parent of the one before it.
fn elimination_tree(trace: &Trace) -> (Vec<u32>, Vec<u32>) {
    let (order, at_elimination) = (trace.order(), trace.neighbours());
    let mut position = vec![0usize; order.len()];
    for (i, &v) in order.iter().enumerate() {
        position[v as usize] = i;
    }
    let mut parent = vec![u32::MAX; order.len()];
    for (&v, around) in order.iter().zip(at_elimination) {
        if let Some(&p) = around.iter().min_by_key(|&&u| position[u as usize]) {
            parent[v as usize] = p;
        }
    }
    for pair in order[at_elimination.len()..].windows(2) {
        parent[pair[0] as usize] = pair[1];
    }
    (order.to_vec(), parent)
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
/// ([`states`]): a move takes one vertex to another place in
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
    /// The states of eliminating in `order` ([`states`]), or
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
    let graph = goatd::Graph::new(gp.num_atoms() as u32, interaction_edges(gp));
    let trace = min_fill_trace(&graph, None).expect("min-fill takes every graph");
    let (order, parent) = elimination_tree(&trace);
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
    use crate::ground::shape::QueriedCubes;
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

    /// The min-fill elimination's order and elimination tree.
    fn min_fill_tree(n: usize, edges: &[(u32, u32)]) -> (Vec<u32>, Vec<u32>) {
        let graph = goatd::Graph::new(n as u32, edges.to_vec());
        elimination_tree(&min_fill_trace(&graph, None).expect("a min-fill"))
    }

    #[test]
    fn min_fill_eliminates_every_vertex_once() {
        // Two triangles sharing a vertex, plus an isolated one.
        let edges = [(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2)];
        let (mut order, _) = min_fill_tree(6, &edges);
        assert_eq!(order.len(), 6);
        order.sort_unstable();
        assert_eq!(order, (0..6).collect::<Vec<u32>>());
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
            let graph = goatd::Graph::new(n as u32, edges);
            let plain = min_fill_trace(&graph, None).unwrap();
            let equal = min_fill_trace(&graph, Some(&vec![3; n])).unwrap();
            assert_eq!(plain.order(), equal.order());
        }
        let cycle = goatd::Graph::new(4, vec![(0, 1), (1, 2), (2, 3), (0, 3)]);
        assert_eq!(min_fill_trace(&cycle, None).unwrap().order()[0], 0);
        let weighted = min_fill_trace(&cycle, Some(&[2, 81, 2, 81])).unwrap();
        assert_eq!(weighted.order()[0], 1);
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
            let graph = goatd::Graph::new(n as u32, edges.clone());
            let start_states = states(
                &eliminated_in_order(&graph, &start, usize::MAX).unwrap(),
                &bits,
            );
            assert!(annealed(&edges, &bits, &start, 10).is_none());
            let Some(order) = annealed(&edges, &bits, &start, 1 << 22) else {
                continue;
            };
            improved += 1;
            let mut sorted = order.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, start);
            let elimination = eliminated_in_order(&graph, &order, usize::MAX).unwrap();
            assert!(states(&elimination, &bits) < start_states);
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
        let (order, _) = min_fill_tree(5, &edges);
        assert!(
            order[0] == 0 || order[0] == 4,
            "an end of the path goes first, got {order:?}"
        );
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
        let (order, parent) = min_fill_tree(leaves as usize + 1, &edges);
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

    /// In the model the layouts read ([`QueriedCubes`]), a query of a cube of
    /// facts that nothing else reads ties none of its facts to another,
    /// however the cube is written: in one rule, through a helper atom, over
    /// a negated fact or at another arity. Observed, or read by a rule that
    /// is not such a cube, it keeps its clique.
    #[test]
    fn a_queried_cube_ties_no_fact_to_another() {
        // The evidence reads x, y and z without tying them to each other.
        let facts = "0.3::a. 0.4::b. 0.5::c. 0.6::d. 0.2::x. 0.7::y. 0.8::z. \
                     e :- a, x. e :- b, y. e :- c, z. e :- d. evidence(e).";
        let tied = |rules: &str| {
            let gp = ground_source(&format!("{facts} {rules}"));
            let gp = QueriedCubes::split(&gp).map_or(gp, |(_, model, _)| model);
            let edges: FxHashSet<(u32, u32)> = interaction_edges(&gp).into_iter().collect();
            let (x, y, z) = (atom(&gp, "x"), atom(&gp, "y"), atom(&gp, "z"));
            [edge(x, y), edge(x, z), edge(y, z)].map(|e| edges.contains(&e))
        };
        for rules in [
            "q :- x, y, z. query(q).",
            "q :- h, z. h :- x, \\+y. query(q).",
            "q(1, X) :- p(X), z. p(2) :- x, y. query(q(1, 2)).",
        ] {
            assert_eq!(tied(rules), [false; 3], "{rules}");
        }
        for rules in [
            "q :- x, y. query(q). evidence(q, false).",
            "q :- x, y. r :- q, z. r :- d. query(r).",
        ] {
            assert!(tied(rules)[0], "{rules}");
        }
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
        let (order, parent) = min_fill_tree(7, &edges);
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
