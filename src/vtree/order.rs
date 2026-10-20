//! A dependency order of the derived components chosen by local search on
//! a size proxy for the right-linear TDDs the compile order stands for.
//!
//! `closed` lays the facts out in the order the compiler consumes them where
//! nothing joins, so the order of the components decides the vtree. Any dependency order is
//! sound; they differ enormously in the diagrams they give. The proxy for
//! the size of atom `a`'s formula under an order is
//!
//! ```text
//! size(a) ≈ Σ_{p in span(a)} 2^{bits_a(p)} · facts introduced at p
//! ```
//!
//! where the span runs from the first ancestor of `a` to `a` itself,
//! `bits_a(p)` counts the ancestors of `a` (components and facts) that are
//! placed at or before position `p` and still awaited by a consumer among
//! `a`'s ancestors at or after `p`, and a fact is introduced at the position
//! of its first consumer. The TDD over a right-linear vtree is the OBDD over
//! the reversed order with no level skipped: at the cut after position `p`
//! it has one node per distinct function the remaining facts induce, at
//! most the assignments to the atoms alive across the cut, and since it
//! keeps a node at every level, every fact inside the span costs the width
//! of its cut whether the formula depends on it or not.
//! On the Bayesian networks the proxy ranks orders as the compiled sizes
//! do (water's deterministic tables make it overestimate, but the order it
//! prefers is still the smallest of those tried).
//!
//! Components that belong together are merged before the search
//! ([`deps::components`]): the values of a network variable, which share switch
//! facts or, when deterministic, dependencies and consumers. Without the
//! merge the proxy charges a bit per switch kept alive between separated
//! values and no single move can pay. The search is simulated annealing
//! over insertion moves that keep every component after its dependencies
//! and before its consumers, from the compiler's own component order. The
//! objective is the proxy summed over the components with the most
//! ancestors, since those own the large formulas. Work is bounded in
//! evaluations, not seconds, so the order does not depend on the machine's
//! load; programs beyond [`MAX_UNITS`] components or [`MAX_FACTS`] facts,
//! or where the budget cannot pay for [`MIN_MOVES_PER_UNIT`] moves per
//! component, keep the compiler's order.

use rustc_hash::FxHashMap;

use crate::ground::{AtomId, AtomKind, GroundProgram, RulesByHead, deps};

/// Components beyond which the search is skipped.
pub(crate) const MAX_UNITS: usize = 4000;
/// Facts beyond which the search is skipped.
pub(crate) const MAX_FACTS: usize = 50_000;
/// Components whose proxies are summed: those with the most ancestors.
const SCORED: usize = 32;
/// Graph steps the search may spend evaluating orders: per evaluation, one
/// per component, one per user of a fact more than one component reads,
/// one per consumer of a component of a scored set, and one per position of
/// a scored span. All but the spans are fixed by the program, so the budget
/// is a bound on the program's size times the moves taken, and the same on
/// every machine.
///
/// The budget is small because on most programs a larger search was most
/// of `closed`'s cost without changing the compile.
const WORK_BUDGET: u64 = 40_000_000;
/// Graph steps the search may spend without beating its best order.
const STALL_BUDGET: u64 = 16_000_000;
/// Insertion moves tried at most, and the temperature schedule of the
/// annealing (in bits of the log2 objective). On a small program the
/// moves bound the search before the steps do; uwcse6 finds its best order
/// past move 18 000.
const MAX_MOVES: usize = 20_000;
/// Moves the search may take without once beating the order it started
/// from: a search that improves an order does so within a few dozen moves.
const PROBE_MOVES: usize = 200;
/// Moves per component that [`WORK_BUDGET`] must pay for, at the steps of
/// the first evaluation, for the search to run. The walk improves an order
/// by moving each component many times. Where the budget pays for a move
/// or two per component, the walk ends while still hot, near the order it
/// started from, and its steps cost more than the compile it orders.
const MIN_MOVES_PER_UNIT: u64 = 4;
const T_START: f64 = 1.0;
const T_END: f64 = 0.02;

struct Program {
    /// Components in the compiler's order.
    units: Vec<Vec<AtomId>>,
    /// Components a component depends on (its rules' derived body atoms
    /// outside itself), and the reverse.
    deps: Vec<Vec<usize>>,
    consumers: Vec<Vec<usize>>,
    /// Facts each component's rules use, and the components using each
    /// fact (`fact_users[k]` for the k-th fact in `facts`). `facts_of`
    /// holds only the facts with more than one user: a fact only one
    /// component reads is introduced there and last read there whatever the
    /// order, so it never crosses a cut and the sweep adds nothing for it.
    facts_of: Vec<Vec<usize>>,
    fact_users: Vec<Vec<usize>>,
    /// How many facts each component is the only reader of. All the
    /// objective needs of them is that they are introduced wherever their
    /// component sits, which is one addition per component instead of one
    /// per fact.
    solo_facts: Vec<f64>,
    /// The facts with more than one user, whose introduction point is the
    /// one thing about a fact the order decides. The merge joins the
    /// components that read a fact, so this is empty unless the merge was
    /// refused for closing a cycle (on the benchmark suite, the UW-CSE
    /// programs only).
    shared_facts: Vec<usize>,
    /// Bits a component is worth when alive: log2 of its atoms plus one.
    weight: Vec<f64>,
    /// Ancestor components of each component, as sorted lists.
    ancestors: Vec<Vec<usize>>,
    /// Components scored by the objective.
    scored: Vec<usize>,
    /// Where the program has [`CONE_FACTS`] facts or fewer, a bound on the
    /// compile's nodes that holds whatever the order: each atom's formula
    /// over the `k` facts of its component's ancestor cone at most
    /// `2^(k+1)` nodes.
    cone_bound: Option<f64>,
}

/// Facts up to which [`Program::cone_bound`] is computed, one bit each of
/// a mask per component. Beyond, the bound exceeds anything a search costs.
const CONE_FACTS: usize = 64;

impl Program {
    fn new(gp: &GroundProgram, by_head: &RulesByHead, units: Vec<Vec<AtomId>>) -> Self {
        let n = gp.num_atoms();
        let mut unit_of = vec![usize::MAX; n];
        for (i, unit) in units.iter().enumerate() {
            for &a in unit {
                unit_of[a as usize] = i;
            }
        }
        let mut fact_index: FxHashMap<AtomId, usize> = FxHashMap::default();
        let mut fact_users: Vec<Vec<usize>> = Vec::new();
        let mut deps = vec![Vec::new(); units.len()];
        let mut consumers = vec![Vec::new(); units.len()];
        let mut facts_of = vec![Vec::new(); units.len()];
        for (i, unit) in units.iter().enumerate() {
            for &h in unit {
                for &ri in &by_head[h as usize] {
                    for l in &gp.rules[ri].body {
                        let b = l.atom();
                        match gp.kinds[b as usize] {
                            AtomKind::Derived => {
                                let j = unit_of[b as usize];
                                if j != i {
                                    deps[i].push(j);
                                }
                            }
                            AtomKind::Fact(_) => {
                                let k = *fact_index.entry(b).or_insert_with(|| {
                                    fact_users.push(Vec::new());
                                    fact_users.len() - 1
                                });
                                facts_of[i].push(k);
                            }
                        }
                    }
                }
            }
            deps[i].sort_unstable();
            deps[i].dedup();
            facts_of[i].sort_unstable();
            facts_of[i].dedup();
            for &j in &deps[i] {
                consumers[j].push(i);
            }
            for &k in &facts_of[i] {
                fact_users[k].push(i);
            }
        }
        // Ancestors, in the compiler's order (dependencies come first).
        let mut ancestors: Vec<Vec<usize>> = vec![Vec::new(); units.len()];
        for i in 0..units.len() {
            let mut set: Vec<usize> = Vec::new();
            for &j in &deps[i] {
                set.push(j);
                set.extend_from_slice(&ancestors[j]);
            }
            set.sort_unstable();
            set.dedup();
            ancestors[i] = set;
        }
        let weight = units
            .iter()
            .map(|u| ((u.len() + 1) as f64).log2())
            .collect();
        let mut scored: Vec<usize> = (0..units.len())
            .filter(|&i| !ancestors[i].is_empty())
            .collect();
        scored.sort_by_key(|&i| (std::cmp::Reverse(ancestors[i].len()), i));
        scored.truncate(SCORED);
        // A component's cone is its own facts and its dependencies' cones;
        // dependencies come first.
        let cone_bound = (fact_users.len() <= CONE_FACTS).then(|| {
            let mut cone = vec![0u64; units.len()];
            let mut bound = 0f64;
            for i in 0..units.len() {
                let own = facts_of[i].iter().fold(0u64, |m, &k| m | 1 << k);
                cone[i] = deps[i].iter().fold(own, |m, &j| m | cone[j]);
                bound += units[i].len() as f64 * 2f64.powi(cone[i].count_ones() as i32 + 1);
            }
            bound
        });
        let mut solo_facts = vec![0f64; units.len()];
        let mut shared_facts: Vec<usize> = Vec::new();
        for (k, users) in fact_users.iter().enumerate() {
            match users.as_slice() {
                [only] => solo_facts[*only] += 1.0,
                _ => shared_facts.push(k),
            }
        }
        for of in facts_of.iter_mut() {
            of.retain(|&k| fact_users[k].len() > 1);
        }
        Program {
            units,
            deps,
            consumers,
            facts_of,
            fact_users,
            solo_facts,
            shared_facts,
            weight,
            ancestors,
            scored,
            cone_bound,
        }
    }

    /// The proxy summed over the scored components, in log2, for the order
    /// that puts component `u` at `pos[u]`, and the node visits the
    /// evaluation took.
    fn objective(&self, pos: &[usize], scratch: &mut Scratch) -> (f64, u64) {
        let n = pos.len();
        // `introduced` is read at every position of a span and has to start
        // at zero. The other three buffers are written before they are read
        // — `fact_pos` at every index in the loop below, `in_set` and
        // `fact_seen` inside the scored loop, which puts back whatever it
        // sets — so they are sized once and never cleared.
        scratch.introduced.clear();
        scratch.introduced.resize(n, 0.0);
        scratch.fact_pos.resize(self.fact_users.len(), usize::MAX);
        let mut work = n as u64;
        for (u, &owned) in self.solo_facts.iter().enumerate() {
            if owned != 0.0 {
                scratch.introduced[pos[u]] += owned;
            }
        }
        for &k in &self.shared_facts {
            let p = self.fact_users[k]
                .iter()
                .map(|&u| pos[u])
                .min()
                .expect("a fact has a user");
            scratch.fact_pos[k] = p;
            scratch.introduced[p] += 1.0;
            work += self.fact_users[k].len() as u64;
        }
        scratch.in_set.resize(n, false);
        scratch.fact_seen.resize(self.fact_users.len(), false);
        let mut total = 0f64;
        for &a in &self.scored {
            let set = &self.ancestors[a];
            for &u in set {
                scratch.in_set[u] = true;
            }
            scratch.in_set[a] = true;
            let first = set
                .iter()
                .map(|&u| pos[u])
                .min()
                .unwrap_or(pos[a])
                .min(pos[a]);
            let last = pos[a];
            let span = last - first + 1;
            // `add` and `rem` are accumulators over the span and have to
            // start at zero. They are sized to the whole order once and the
            // sweep below puts back the entries it reads, which are the only
            // ones this component writes, so neither is cleared here: that
            // would be two more passes over the span per scored component.
            scratch.add.resize(n, 0.0);
            scratch.rem.resize(n, 0.0);
            scratch.touched.clear();
            for &u in set.iter().chain(std::iter::once(&a)) {
                // The component itself, awaited by consumers inside the set.
                let mut last_use = None;
                for &c in &self.consumers[u] {
                    if scratch.in_set[c] {
                        last_use = Some(last_use.map_or(pos[c], |l: usize| l.max(pos[c])));
                    }
                }
                work += self.consumers[u].len() as u64;
                if let Some(l) = last_use {
                    scratch.add[pos[u] - first] += self.weight[u];
                    scratch.rem[l - first] += self.weight[u];
                }
                // Its facts, read at their first consumer anywhere, awaited
                // by their last consumer inside the set.
                for &k in &self.facts_of[u] {
                    if scratch.fact_seen[k] {
                        continue;
                    }
                    scratch.fact_seen[k] = true;
                    scratch.touched.push(k);
                    let p = scratch.fact_pos[k];
                    let l = self.fact_users[k]
                        .iter()
                        .filter(|&&c| scratch.in_set[c])
                        .map(|&c| pos[c])
                        .max()
                        .unwrap();
                    work += self.fact_users[k].len() as u64;
                    if l > p {
                        scratch.add[p.max(first) - first] += 1.0;
                        scratch.rem[l - first] += 1.0;
                    }
                }
            }
            let mut bits = 0f64;
            let mut mine = 0f64;
            // `2^bits` is taken only at a position that introduces a fact
            // after the bits changed: most positions of a scored span
            // introduce nothing or lie between changes.
            let mut scale = 1f64;
            let mut stale = false;
            for q in 0..span {
                let add = scratch.add[q];
                if add != 0.0 {
                    bits += add;
                    stale = true;
                    scratch.add[q] = 0.0;
                }
                let introduced = scratch.introduced[first + q];
                if introduced != 0.0 {
                    if stale {
                        scale = bits.min(900.0).exp2();
                        stale = false;
                    }
                    mine += scale * introduced;
                }
                let rem = scratch.rem[q];
                if rem != 0.0 {
                    bits -= rem;
                    stale = true;
                    scratch.rem[q] = 0.0;
                }
            }
            total += mine;
            work += span as u64;
            for &u in set {
                scratch.in_set[u] = false;
            }
            scratch.in_set[a] = false;
            for &k in &scratch.touched {
                scratch.fact_seen[k] = false;
            }
        }
        (total.max(1.0).log2(), work)
    }
}

#[derive(Default)]
struct Scratch {
    introduced: Vec<f64>,
    fact_pos: Vec<usize>,
    in_set: Vec<bool>,
    fact_seen: Vec<bool>,
    add: Vec<f64>,
    rem: Vec<f64>,
    /// The facts one scored component's span touched, to clear `fact_seen`
    /// with. One buffer reused, not one allocation per scored component per
    /// move.
    touched: Vec<usize>,
}

/// A small deterministic generator (xorshift64*).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// The components of `sccs` (in a dependency order) reordered by the
/// search, still a dependency order; `note` tells what the search did.
pub(crate) fn search(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    sccs: Vec<Vec<AtomId>>,
) -> (Vec<Vec<AtomId>>, String) {
    let num_facts = gp.facts().count();
    if sccs.len() > MAX_UNITS || num_facts > MAX_FACTS {
        let note = format!(
            "order: {} components, {} facts: search skipped",
            sccs.len(),
            num_facts
        );
        return (sccs, note);
    }
    let program = Program::new(gp, by_head, deps::components(gp, by_head, sccs));
    let n = program.units.len();
    let mut scratch = Scratch::default();
    let positions = |order: &[usize]| -> Vec<usize> {
        let mut pos = vec![0; order.len()];
        for (p, &u) in order.iter().enumerate() {
            pos[u] = p;
        }
        pos
    };
    let mut order: Vec<usize> = (0..n).collect();
    let (mut cost_c, mut work) = program.objective(&positions(&order), &mut scratch);
    // The least a search costs is the probe's evaluations. Where the
    // compile is bounded below that whatever the order, no order can repay
    // it (detached evidence can leave a few facts under hundreds of
    // components).
    if let Some(bound) = program.cone_bound
        && bound <= (PROBE_MOVES as u64 * work) as f64
    {
        let note = format!(
            "order: {n} components, {} facts, at most {bound:.0} nodes whatever the order: search skipped",
            program.fact_users.len()
        );
        return (program.units, note);
    }
    if WORK_BUDGET < MIN_MOVES_PER_UNIT * n as u64 * work {
        let note = format!("order: {n} components at {work} steps a move: search skipped");
        return (program.units, note);
    }
    let start_cost = cost_c;
    let mut pos = positions(&order);
    // A move takes a component from position `i` to position `j`, which
    // rewrites `i..=j` and leaves the rest of the order alone, so the loop
    // makes the move in the order it holds and rotates it back when the
    // walk does not take it: two rotations of the range cost less than four
    // copies of the whole order, and most of a long walk's moves are short.
    let mut best = order.clone();
    let mut best_cost = cost_c;
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut moves = 0;
    let mut accepted = 0;
    // The move the order returned was found at, which is what a budget
    // would have to keep, and the move the first improvement came at, which
    // is what a probe would have to see. Deterministic: the search is
    // bounded in evaluations and its generator is seeded, so it does not
    // depend on the machine.
    let mut best_move = 0;
    let mut first_move = 0;
    let mut best_work = work;
    while moves < MAX_MOVES && work < WORK_BUDGET && work - best_work < STALL_BUDGET && n > 1 {
        // A search that has not once beaten the order it started from is on
        // a plateau it does not leave (on the grids the walk accepts most
        // moves and returns its input). Stop and return it now.
        if first_move == 0 && moves >= PROBE_MOVES {
            break;
        }
        moves += 1;
        let t = T_START * (T_END / T_START).powf(moves as f64 / MAX_MOVES as f64);
        // Move a component anywhere between its last dependency and its
        // first consumer.
        let u = rng.below(n);
        let i = pos[u];
        let lo = program.deps[u]
            .iter()
            .map(|&d| pos[d] + 1)
            .max()
            .unwrap_or(0);
        let hi = program.consumers[u]
            .iter()
            .map(|&c| pos[c] - 1)
            .min()
            .unwrap_or(n - 1);
        if hi <= lo {
            continue;
        }
        let mut j = lo + rng.below(hi - lo + 1);
        if j == i {
            j = if j < hi { j + 1 } else { j - 1 };
        }
        let (lo_p, hi_p) = (i.min(j), i.max(j));
        let repos = |order: &[usize], pos: &mut [usize]| {
            for (p, &w) in order.iter().enumerate().take(hi_p + 1).skip(lo_p) {
                pos[w] = p;
            }
        };
        if i < j {
            order[i..=j].rotate_left(1);
        } else {
            order[j..=i].rotate_right(1);
        }
        repos(&order, &mut pos);
        let (cost, w) = program.objective(&pos, &mut scratch);
        work += w;
        if cost < cost_c || rng.unit() < ((cost_c - cost) / t).exp() {
            cost_c = cost;
            accepted += 1;
            if cost < best_cost {
                best_work = work;
                best_cost = cost;
                best.copy_from_slice(&order);
                best_move = moves;
                if first_move == 0 {
                    first_move = moves;
                }
            }
        } else {
            if i < j {
                order[i..=j].rotate_right(1);
            } else {
                order[j..=i].rotate_left(1);
            }
            repos(&order, &mut pos);
        }
    }
    let note = format!(
        "order: {} components ({} merged), log2 proxy {start_cost:.1} to {best_cost:.1} \
         from move {first_move} to {best_move} of {moves} ({accepted} accepted), {} scored, {work} steps",
        n,
        program.units.iter().filter(|u| u.len() > 1).count(),
        program.scored.len()
    );
    (
        best.into_iter().map(|u| program.units[u].clone()).collect(),
        note,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::deps::tests::is_dependency_order;

    /// `src` ground with its derived atoms as written, each its own
    /// component to order (`GroundOptions::views` off).
    fn program(src: &str) -> GroundProgram {
        let p = crate::syntax::parse_program(src).unwrap();
        crate::ground::ground(
            &p,
            &crate::ground::GroundOptions {
                views: false,
                ..crate::ground::GroundOptions::default()
            },
        )
        .unwrap()
    }

    #[test]
    fn search_keeps_a_dependency_order() {
        let gp = program(
            "0.5::e(1,2). 0.5::e(2,3). 0.5::e(3,1). 0.5::e(1,4). 0.5::e(4,5).
             p(X,Y) :- e(X,Y). p(X,Y) :- e(X,Z), p(Z,Y).
             q(Y) :- p(1,Y). r :- q(5), \\+ q(2). query(r).",
        );
        let by_head = gp.rules_by_head();
        let sccs = deps::derived_sccs(&gp, &by_head);
        assert!(is_dependency_order(&gp, &sccs));
        let (order, note) = search(&gp, &by_head, sccs.clone());
        assert!(is_dependency_order(&gp, &order), "{note}");
        let mut a: Vec<AtomId> = order.iter().flatten().copied().collect();
        let mut b: Vec<AtomId> = sccs.iter().flatten().copied().collect();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b);
    }

    fn grid(n: usize) -> String {
        let mut src = String::new();
        for i in 0..n {
            for j in 0..n {
                if j + 1 < n {
                    src.push_str(&format!("0.5::edge(n{i}_{j},n{i}_{}).\n", j + 1));
                }
                if i + 1 < n {
                    src.push_str(&format!("0.5::edge(n{i}_{j},n{}_{j}).\n", i + 1));
                }
            }
        }
        src.push_str("path(X,Y) :- edge(X,Y).\npath(X,Y) :- edge(X,Z), path(Z,Y).\n");
        src.push_str(&format!("query(path(n0_0,n{}_{})).\n", n - 1, n - 1));
        src
    }

    /// `"... from move F to B of M (A accepted)"`.
    fn moves_run(note: &str) -> (usize, usize) {
        let tail = note.split("from move ").nth(1).expect(note);
        let mut it = tail.split_whitespace();
        let first: usize = it.next().unwrap().parse().unwrap();
        let moves: usize = it.nth(3).unwrap().parse().unwrap();
        (first, moves)
    }

    #[test]
    fn the_probe_stops_a_search_that_has_improved_nothing() {
        // A 6 by 6 grid is the smallest of that family whose proxy is flat
        // under insertion moves: the walk accepts most of what it tries and
        // never gets below where it began, so the probe returns the order
        // the search started from. A 4 by 4 grid improves at once and runs
        // its whole budget.
        for (n, expected) in [(6, PROBE_MOVES), (4, MAX_MOVES)] {
            let gp = program(&grid(n));
            let by_head = gp.rules_by_head();
            let sccs = deps::derived_sccs(&gp, &by_head);
            let (_, note) = search(&gp, &by_head, sccs);
            let (first, moves) = moves_run(&note);
            assert_eq!(moves, expected, "{n} by {n}: {note}");
            assert_eq!(first == 0, expected == PROBE_MOVES, "{n} by {n}: {note}");
        }
    }

    #[test]
    fn a_search_no_order_can_repay_is_skipped() {
        // Two facts bound each of the 599 relevant formulas at 8 nodes
        // whatever the order, below the probe's 200 evaluations of an
        // order of 596 components: the grounder's order stands. The 4 by 4 grid of
        // `the_probe_stops_a_search_that_has_improved_nothing`, 24 facts
        // under 16 components, is searched.
        let mut src = String::from("0.5::a. 0.5::b.\nx0 :- a.\ny0 :- b.\n");
        for i in 1..300 {
            src.push_str(&format!("x{i} :- x{j}, y{j}.\ny{i} :- x{j}.\n", j = i - 1));
        }
        src.push_str("query(x299).\n");
        let gp = program(&src);
        let by_head = gp.rules_by_head();
        let sccs = deps::derived_sccs(&gp, &by_head);
        let units = deps::components(&gp, &by_head, sccs.clone());
        let (order, note) = search(&gp, &by_head, sccs);
        assert!(
            note.starts_with("order: 596 components, 2 facts, at most 4776 nodes"),
            "{note}"
        );
        assert!(note.ends_with("search skipped"), "{note}");
        assert_eq!(order, units);
    }

    #[test]
    fn a_search_the_budget_cannot_pay_for_is_skipped() {
        // A chain of 1000 components, a fact each: an evaluation walks the
        // spans of the 32 deepest, about 1000 steps each, and the budget
        // pays for fewer than MIN_MOVES_PER_UNIT moves per component, so
        // the grounder's order stands. The 4 by 4 grid of
        // `the_probe_stops_a_search_that_has_improved_nothing` is searched.
        let mut src = String::from("0.5::f0. x0 :- f0.\n");
        for i in 1..1000 {
            src.push_str(&format!("0.5::f{i}. x{i} :- x{j}, f{i}.\n", j = i - 1));
        }
        src.push_str("query(x999).\n");
        let gp = program(&src);
        let by_head = gp.rules_by_head();
        let sccs = deps::derived_sccs(&gp, &by_head);
        let units = deps::components(&gp, &by_head, sccs.clone());
        let (order, note) = search(&gp, &by_head, sccs);
        assert!(note.ends_with("steps a move: search skipped"), "{note}");
        assert_eq!(order, units);
    }

    #[test]
    fn a_fact_one_component_reads_is_counted_without_being_visited() {
        // The merge joins the components that read a fact, so after it a
        // fact usually has one reader and the objective never walks it: it
        // adds the count its component owns and moves on. The split is a
        // partition of the facts either way.
        let gp = program(
            "0.5::a. 0.5::b. 0.5::c.
             p :- a. q :- b. r :- p, q, a, b. s :- r, c.
             query(s).",
        );
        let by_head = gp.rules_by_head();
        let sccs = deps::derived_sccs(&gp, &by_head);
        let program = Program::new(&gp, &by_head, deps::components(&gp, &by_head, sccs));
        let facts = program.fact_users.len();
        assert_eq!(
            program.solo_facts.iter().sum::<f64>() as usize + program.shared_facts.len(),
            facts
        );
        assert!(
            program.shared_facts.is_empty(),
            "the merge joined the readers of a and b"
        );
        assert!(
            program.facts_of.iter().all(|of| of.is_empty()),
            "nothing left for the scored loop to walk"
        );
        // Every fact is still counted where it enters, and an evaluation
        // leaves the buffers as it found them, so the same order scores the
        // same twice running.
        let n = program.units.len();
        let mut scratch = Scratch::default();
        let order: Vec<usize> = (0..n).collect();
        let mut pos = vec![0; n];
        for (p, &u) in order.iter().enumerate() {
            pos[u] = p;
        }
        let first = program.objective(&pos, &mut scratch);
        assert!(first.0 > 0.0, "the facts are counted where they enter");
        assert!(
            first.1 >= program.units.len() as u64,
            "the sweep over the components is charged"
        );
        assert_eq!(first, program.objective(&pos, &mut scratch));
    }

    #[test]
    fn a_chain_prefers_its_own_order() {
        // A chain x1 -> x2 -> ... with a fact each: any dependency order is
        // the chain itself, so the search returns it unchanged.
        let mut src = String::from("0.5::f1. x1 :- f1.\n");
        for i in 2..8 {
            src.push_str(&format!("0.5::f{i}. x{i} :- x{}, f{i}.\n", i - 1));
        }
        src.push_str("query(x7).");
        let gp = program(&src);
        let by_head = gp.rules_by_head();
        let sccs = deps::derived_sccs(&gp, &by_head);
        let (order, _) = search(&gp, &by_head, sccs.clone());
        assert_eq!(order, sccs);
    }
}
