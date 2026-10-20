//! The `td` layout: a tree decomposition of the interaction graph, under a
//! budget of charged work, folded into a fact order; and the same fold of
//! the min-fill elimination tree, `dynamic`'s start where every rule reads
//! facts alone.

use std::cmp::Reverse;
use std::time::Duration;

use crate::ground::deps::Deps;
use crate::ground::{AtomId, GroundProgram, add_fact_copies};

use super::TD_SEED;
use super::forest::Forest;
use super::graph::{
    elimination_budget, facts_by_use, interaction_edges, min_fill_cliques, minor_min_width,
};
use super::separators::{self, Atop, CopyRule};

/// The `td` strategy's search budget is charged graph work, not seconds.
///
/// goatd's decomposer is anytime against a deadline, and a deadline in wall
/// time makes the vtree — and so the answer's cost, and which programs finish
/// — depend on the machine and on its load. `goatd::meter::arm` replaces the
/// clock the decomposer reads with the work it has charged, so the same graph
/// and seed give the same decomposition everywhere. The library's own
/// calibration, `goatd::meter::UNITS_PER_MS`, is what converts a unit budget
/// into the `Duration` its API takes.
///
/// The ceiling, and what a large graph is given. goatd's hard cutoff is twice
/// its soft budget, so this is what the search spends, not what it may take.
/// A higher ceiling bought the graphs that reach it (the large pedigrees)
/// almost no narrower decomposition, and the programs `td` wins are given
/// less.
pub(crate) const MAX_TD_BUDGET_UNITS: u64 = 100_000_000;

/// The floor under a scaled budget.
///
/// goatd's portfolio decomposes at any budget, but on a small graph a
/// search below the floor returns a decomposition as narrow that lays out
/// worse. On bn_munin1's 187 groups and 330 edges the scaled budget, 12.9 M
/// units, found width 11 as the floor does, in 167 bags against 165, whose
/// layout is estimated twice as costly, and the searched trial's compile
/// did twenty times the work. A budget twice the scaled one kept munin1's
/// layout, but doubling every budget so cost 1npl's PR 1.7 times its time.
/// The floor, 50 nominal milliseconds, is not felt in `auto` where the
/// round stops at an earlier trial before `td` or the families search is
/// reached on such a program.
const MIN_TD_BUDGET_UNITS: u64 = 38_750_000;

/// The search budget for a graph of this size.
///
/// Past the floor the portfolio is anytime and has almost nothing left to
/// try: over a fortyfold range of budgets the treewidth does not move on any
/// program of the suite and the compiled formula barely does. A search that
/// only decides a vtree should not cost more than the compilation it decides
/// for. goatd's hard cutoff is twice the soft budget and the search takes all
/// of it; the floor covers everything small.
pub(super) fn scaled_budget_units(num_vertices: usize, num_edges: usize) -> u64 {
    ((num_vertices + num_edges) as u64 * 25_000).clamp(MIN_TD_BUDGET_UNITS, MAX_TD_BUDGET_UNITS)
}

/// The bag of `td` to root it at.
///
/// A structural TDD over a right-linear stretch of the vtree holds, at the
/// level under leaves `x_i..x_n`, one node per cofactor of the function with
/// those variables fixed, so it is the OBDD over the *reversed* leaf order.
/// The facts the compiler consumes last therefore belong nearest the root and the facts it consumes first deepest. Rank
/// the bags by the last-consumed fact they hold, breaking ties towards the
/// bag holding the fewest facts (a variable is introduced at the topmost bag
/// that holds it, so a fact-heavy root introduces everything at once and the
/// decomposition buys nothing). Ranking by the fewest facts alone leaves the
/// direction to chance, and the two directions of one spine can differ by
/// orders of magnitude. Every bag is ranked, and the head of the list is the
/// root the rule alone would pick; a caller that folds more than one of them
/// keeps the direction, since they all hold a late fact, and lets the fold
/// decide the rest.
fn root_bag(
    td: &goatd::TreeDecomposition,
    position: &[usize],
    leaf: &dyn Fn(AtomId) -> bool,
) -> Option<usize> {
    let bags = td.bags();
    (0..bags.len()).max_by_key(|&b| {
        let facts: Vec<usize> = bags[b]
            .vertices()
            .iter()
            .filter(|&&v| leaf(v as AtomId))
            .map(|&v| position[v as usize])
            .collect();
        let last = facts
            .iter()
            .copied()
            .filter(|&p| p != usize::MAX)
            .max()
            .map_or(0, |p| p + 1);
        (
            last,
            Reverse(facts.len()),
            Reverse(bags[b].vertices().len()),
        )
    })
}

/// A decomposition of the interaction graph rooted for its fold into a
/// vtree shape ([`Rooted::lay_out`]): its bags' parents and children, the
/// facts each introduces, and the order the fold lays them out in.
///
/// A bag *introduces* a fact that is in it and in no ancestor bag, which by
/// the connectivity of a tree decomposition is one the parent bag does not
/// hold; every fact is introduced by exactly one bag. A bag's subtree combines
/// its children's subtrees, balanced against each other, under a right-linear
/// spine of the facts it introduces. A bag introducing no fact and covering
/// none below it contributes nothing, and derived atoms are dropped wherever
/// they appear. The two choices this leaves — where to root, and how to lay a
/// bag out — are made below, each against what it measured.
///
/// The fold also says how many atoms the widest union of the bags on a root
/// path holds, which is what a right-linear reading of the result keeps
/// alive: reading a subtree leaves every atom of every ancestor bag alive,
/// where reading a path decomposition would leave one bag alive. The gap
/// between that union and the width is why a narrow decomposition can still
/// give a wide order.
pub(super) struct Rooted {
    /// Each bag's parent, `usize::MAX` at a root.
    pub(super) parent: Vec<usize>,
    /// The bags in a preorder.
    preorder: Vec<usize>,
    /// The roots, in the order the fold joins their subtrees.
    pub(super) roots: Vec<usize>,
    /// Each bag's children, in the order the fold joins their subtrees.
    pub(super) children: Vec<Vec<usize>>,
    /// The facts each bag introduces, in the order of its spine from the
    /// subtree's root down.
    pub(super) introduced: Vec<Vec<AtomId>>,
    /// The facts introduced in each bag's subtree.
    pub(super) facts_under: Vec<usize>,
    /// Whether a bag introduces the atom, by atom.
    placed: Vec<bool>,
    leaf: Vec<bool>,
}

impl Rooted {
    /// `td` rooted at bag `first`, and every other tree of the forest at its
    /// first bag. `position` orders the facts as the compiler consumes them.
    /// `None` when the decomposition has no bags.
    fn of(
        num_atoms: usize,
        td: &goatd::TreeDecomposition,
        position: &[usize],
        first: usize,
        leaf: &dyn Fn(AtomId) -> bool,
    ) -> Option<Rooted> {
        let bags = td.bags();
        let adjacency = td.adjacency();
        if bags.is_empty() {
            return None;
        }
        let mut parent = vec![usize::MAX; bags.len()];
        let mut preorder = Vec::with_capacity(bags.len());
        let mut visited = vec![false; bags.len()];
        let mut roots = Vec::new();
        for start in std::iter::once(first).chain(0..bags.len()) {
            if visited[start] {
                continue;
            }
            visited[start] = true;
            roots.push(start);
            let mut stack = vec![start];
            while let Some(b) = stack.pop() {
                preorder.push(b);
                for &c in &adjacency[b] {
                    if !visited[c] {
                        visited[c] = true;
                        parent[c] = b;
                        stack.push(c);
                    }
                }
            }
        }

        // Top down, so that a fact lands in the highest bag that holds it.
        let mut introduced: Vec<Vec<AtomId>> = vec![Vec::new(); bags.len()];
        let mut placed = vec![false; num_atoms];
        for &b in &preorder {
            for &v in bags[b].vertices() {
                let atom = v as AtomId;
                if !leaf(atom) || placed[v as usize] {
                    continue;
                }
                if parent[b] != usize::MAX && bags[parent[b]].vertices().contains(&v) {
                    continue;
                }
                placed[v as usize] = true;
                introduced[b].push(atom);
            }
        }

        // Bottom up: a preorder reversed has every bag after its children.
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); bags.len()];
        for &b in &preorder {
            if parent[b] != usize::MAX {
                children[parent[b]].push(b);
            }
        }
        // The latest position in the compile order any fact of a bag's subtree
        // holds. A subtree whose facts are consumed late belongs leftmost, nearer
        // the root, for the same reason a bag's own spine is ordered that way.
        // Without it the left-to-right order of the sibling subtrees is whatever
        // the decomposer's adjacency lists give.
        let mut last: Vec<Option<usize>> = vec![None; bags.len()];
        let mut facts_under = vec![0usize; bags.len()];
        for &b in preorder.iter().rev() {
            last[b] = children[b]
                .iter()
                .filter_map(|&c| last[c])
                .chain(introduced[b].iter().map(|&a| position[a as usize]))
                .max();
            children[b].sort_by_key(|&c| Reverse(last[c]));
            // The variables this bag introduces sit above its children on a
            // right-linear spine, the one consumed last nearest this
            // subtree's root.
            introduced[b].sort_by_key(|&a| Reverse(position[a as usize]));
            facts_under[b] =
                introduced[b].len() + children[b].iter().map(|&c| facts_under[c]).sum::<usize>();
        }
        roots.sort_by_key(|&r| Reverse(last[r]));
        let leaf = (0..num_atoms).map(|a| leaf(a as AtomId)).collect();
        Some(Rooted {
            parent,
            preorder,
            roots,
            children,
            introduced,
            facts_under,
            placed,
            leaf,
        })
    }

    /// The fold's forest, its root and the widest root path union, with the
    /// atoms `atop` lays on spines above bags' subtrees and joins of their
    /// children, the first nearest the root. `None` when the decomposition
    /// covers no fact.
    pub(super) fn lay_out(
        &self,
        td: &goatd::TreeDecomposition,
        atop: &Atop,
    ) -> Option<(Forest, usize, usize)> {
        let Rooted {
            parent: _,
            preorder,
            roots,
            children,
            introduced,
            facts_under: _,
            placed,
            leaf,
        } = self;
        let bags = td.bags();
        let mut forest = Forest::default();
        let mut subtree: Vec<Option<usize>> = vec![None; bags.len()];
        let mut items = Vec::new();
        for &b in preorder.iter().rev() {
            items.clear();
            items.extend(children[b].iter().filter_map(|&c| subtree[c]));
            // The children are balanced against each other; the variables this bag
            // introduces sit above them on a right-linear spine, the one consumed
            // last nearest this subtree's root. Balancing the introduced variables
            // too costs several times the pairs: a bag's variables are tested one
            // after another, and a spine is what lets a diagram finish with one of
            // them before it reaches the next.
            // Atoms laid atop a join of some of the children go above it, and
            // those atop the subtree above its spine, the first topmost.
            let joined = forest.balanced_with(&items, &|lo, hi| atop.group(b, lo, hi));
            let spine = forest.spine(&introduced[b], joined);
            subtree[b] = forest.spine(atop.bag(b), spine);
        }

        let mut tops: Vec<usize> = roots.iter().filter_map(|&r| subtree[r]).collect();
        // Facts no bag introduced — an isolated fact occurring in no rule is a
        // vertex of the graph and so normally is in a bag of its own, but nothing
        // above depends on that — go in one balanced subtree at the root.
        let stray: Vec<usize> = (0..placed.len())
            .filter(|&a| !placed[a] && leaf[a])
            .map(|a| forest.leaf(a as AtomId))
            .collect();
        if let Some(s) = forest.balanced(&stray) {
            tops.push(s);
        }
        let root = forest.balanced(&tops)?;

        let mut union = 0usize;
        let mut on_path = vec![0u32; placed.len()];
        let mut live = 0usize;
        enum Walk {
            Enter(usize),
            Leave(usize),
        }
        let mut walk: Vec<Walk> = roots.iter().rev().map(|&r| Walk::Enter(r)).collect();
        while let Some(step) = walk.pop() {
            match step {
                Walk::Enter(b) => {
                    for &v in bags[b].vertices() {
                        if on_path[v as usize] == 0 {
                            live += 1;
                        }
                        on_path[v as usize] += 1;
                    }
                    union = union.max(live);
                    walk.push(Walk::Leave(b));
                    walk.extend(children[b].iter().rev().map(|&c| Walk::Enter(c)));
                }
                Walk::Leave(b) => {
                    for &v in bags[b].vertices() {
                        on_path[v as usize] -= 1;
                        if on_path[v as usize] == 0 {
                            live -= 1;
                        }
                    }
                }
            }
        }
        Some((forest, root, union))
    }
}

/// Runs `work` with goatd's meter armed ([`goatd::meter::arm`]): the
/// decomposer's clock reads the work charged while it runs, so a budget
/// stops after the same work on any machine (see [`MAX_TD_BUDGET_UNITS`]).
/// Returns what `work` returns, the units charged while it ran, and the
/// seconds it took.
pub(super) fn metered<T>(work: impl FnOnce() -> T) -> (T, u64, f64) {
    let spent_before = goatd::meter::units_spent();
    let started = std::time::Instant::now();
    let value = {
        let _meter = goatd::meter::arm(started);
        work()
    };
    (
        value,
        goatd::meter::units_spent().saturating_sub(spent_before),
        started.elapsed().as_secs_f64(),
    )
}

/// Decompose the interaction graph and read a fact order off it.
///
/// The decomposition is the narrowest of a min-degree elimination, a
/// min-fill one and goatd's portfolio, each run only where the ones before
/// it leave something to find. The atoms alive across an order's cuts do
/// not predict what the compilation pays, so neither the decompositions
/// nor their root bags are chosen by that score.
///
/// The eliminations share goatd's reductions of the graph
/// ([`goatd::elimination::Prepared`]). Min-degree, the cheapest, goes
/// first: where it does not finish within the elimination budget, nothing
/// that follows it can, and its decomposition is the vertices it left in
/// one bag, not a decomposition; `td` then cannot help, and says so with
/// `Err`. Where min-degree, or failing that min-fill, is as narrow as the
/// graph's lower bound ([`minor_min_width`]), its decomposition is taken:
/// nothing narrower exists. The portfolio runs only where its budget
/// (twice [`scaled_budget_units`], goatd's hard cutoff) is more than the
/// reductions and min-degree charged: below that its own eliminations, of
/// which min-degree is the cheapest, stop part way and leave their
/// vertices in one bag, as on Roulette's caesar at 20 000 steps, where
/// its decomposition was 1.3 M atoms wide. Its budget has a floor
/// ([`MIN_TD_BUDGET_UNITS`]) that a small program would pay in full where
/// `auto` tries `td` first, and it stops at a candidate as narrow as the
/// bound ([`portfolio_to_bound`]).
///
/// With `copies`, where the program's rules read facts alone, the children
/// the rule picks get copies of their separator facts
/// ([`super::separators`]), and the fold is over the program with them.
///
/// `Ok(None)` when the program has no facts or the decomposition covers
/// none.
pub(super) fn td_forest(
    gp: &GroundProgram,
    deps: &Deps,
    copies: Option<&CopyRule>,
) -> Result<Option<TdFold>, String> {
    use goatd::elimination::{Order, Prepared, RunConfig, RunOutcome};
    let num_atoms = gp.num_atoms();
    if num_atoms == 0 || gp.facts().next().is_none() {
        return Ok(None);
    }
    let edges = interaction_edges(gp);
    let num_edges = edges.len();
    let budget_units = scaled_budget_units(num_atoms, num_edges);
    // goatd takes its budget as a `Duration`, so the unit budget goes in
    // through the calibration its own meter reads the clock by.
    let budget = Duration::from_millis(goatd::meter::milliseconds_for_units(budget_units));
    let graph = goatd::Graph::new(num_atoms as u32, edges.clone());
    let failed = |e: goatd::Error| format!("td: goatd: {e:?}");
    let (found, units, seconds) = metered(|| -> Result<_, String> {
        let mut prepared = Prepared::new(&graph, Some(elimination_budget())).map_err(failed)?;
        let config = RunConfig::default().with_budgets(Some(elimination_budget()), None);
        // The reductions' and each elimination's units, and each
        // elimination's width, for `--stats`.
        let mut units = vec![prepared.preparation().work_units];
        let mut eliminated = |order| -> Result<(goatd::TreeDecomposition, bool), String> {
            let before = goatd::meter::units_spent();
            let outcome = prepared.run(order, TD_SEED, config).map_err(failed)?;
            units.push(goatd::meter::units_spent() - before);
            match outcome {
                RunOutcome::Completed(td) => Ok((td, true)),
                RunOutcome::CompletedAtDeadline(_, td) => Ok((td, false)),
                _ => Err("td: goatd returned no decomposition".to_string()),
            }
        };
        let mut widths = Vec::new();
        let (min_degree, finished) = eliminated(Order::MinDegree)?;
        if !finished {
            return Ok(None);
        }
        widths.push(min_degree.treewidth());
        let bound = minor_min_width(num_atoms, &edges);
        if min_degree.treewidth() as usize <= bound {
            return Ok(Some((min_degree, bound, false, 0, units, widths)));
        }
        let (min_fill, _) = eliminated(Order::MinFill)?;
        widths.push(min_fill.treewidth());
        if min_fill.treewidth() as usize <= bound {
            return Ok(Some((min_fill, bound, false, 0, units, widths)));
        }
        let narrower = if min_fill.treewidth() < min_degree.treewidth() {
            min_fill
        } else {
            min_degree
        };
        if units[0] + units[1] >= 2 * budget_units {
            return Ok(Some((narrower, bound, false, 0, units, widths)));
        }
        let (searched, pushed) = portfolio_to_bound(&graph, budget, budget_units, bound)
            .ok_or("td: goatd's portfolio failed")?;
        let td = if searched.treewidth() <= narrower.treewidth() {
            searched
        } else {
            narrower
        };
        Ok(Some((td, bound, true, pushed, units, widths)))
    });
    let Some((td, bound, searched, skipped, elimination_units, widths)) = found? else {
        return Err(format!(
            "td cannot help: min-degree, the cheapest elimination, does not finish within the elimination budget on {num_atoms} vertices \
             and {num_edges} edges, and leaves the vertices it has not eliminated in one bag"
        ));
    };
    let charged = units.saturating_sub(skipped);
    let Some((rooted, position)) = rooted(gp, deps, &td) else {
        return Ok(None);
    };
    let (copied, atop, copies_note) = match copies {
        None => (None, Atop::default(), String::new()),
        Some(rule) => {
            let plan = separators::plan(gp, &rooted, &position, rule);
            let mut copied = None;
            if !plan.copies.is_empty() {
                let mut program = gp.clone();
                add_fact_copies(&mut program, &plan.copies);
                copied = Some(program);
            }
            let note = format!(
                "; separator copies: {} copies atop {} of {} candidate units, {} read in full",
                plan.copies.len(),
                plan.copied,
                plan.candidates,
                plan.read
            );
            (copied, plan.atop, note)
        }
    };
    let Some((forest, root, union)) = rooted.lay_out(&td, &atop) else {
        return Ok(None);
    };
    let eliminations = ["reductions", "min-degree", "min-fill"]
        .iter()
        .zip(&elimination_units)
        .enumerate()
        .map(
            |(k, (name, u))| match k.checked_sub(1).and_then(|e| widths.get(e)) {
                Some(w) => format!("{name} {u} (width {w})"),
                None => format!("{name} {u}"),
            },
        )
        .collect::<Vec<_>>();
    let note = format!(
        "td: width {} (treewidth at least {bound}), {} bags, widest root path {union} atoms, {} vertices + {} edges, \
         eliminations {} units, {}, {charged} charged, {seconds:.3}s{copies_note}",
        td.treewidth(),
        td.bags().len(),
        num_atoms,
        num_edges,
        eliminations.join(", "),
        if searched {
            format!("{budget_units} units budgeted")
        } else {
            "no search".to_string()
        },
    );
    Ok(Some(TdFold {
        forest,
        root,
        note,
        copied,
    }))
}

/// What [`td_forest`] lays out: the fold's forest and its root, the note
/// on the search, and the program with its separator copies where it made
/// any, which the fold is over.
pub(super) struct TdFold {
    pub(super) forest: Forest,
    pub(super) root: usize,
    pub(super) note: String,
    pub(super) copied: Option<GroundProgram>,
}

/// goatd's portfolio at `budget` (`budget_units` of charged work), stopped
/// once a candidate is as narrow as the graph's lower bound `bound`, with
/// the units charged to stop it, which are not work.
///
/// Nothing narrower than the bound exists, and the portfolio, anytime
/// against its deadline, would spend the rest of its budget finding no
/// better. The portfolio takes no target width, so its trace pushes the
/// decomposer's clock, which is charged work while the meter is armed, past
/// the hard deadline, twice the budget: the rest of the schedule finds its
/// window closed, and the best candidate so far, one at the bound, comes
/// back.
pub(super) fn portfolio_to_bound(
    graph: &goatd::Graph,
    budget: Duration,
    budget_units: u64,
    bound: usize,
) -> Option<(goatd::TreeDecomposition, u64)> {
    let weights = vec![1; graph.num_vertices() as usize];
    let config = goatd::portfolio::PortfolioConfig::standalone(Some(budget));
    let mut pushed = 0;
    let td = goatd::portfolio::decompose_traced(graph, &weights, TD_SEED, config, &mut |trace| {
        if let goatd::portfolio::CandidateOutcome::Produced { width, .. } = trace.outcome
            && width as usize <= bound
            && pushed == 0
        {
            pushed = 2 * budget_units + goatd::meter::UNITS_PER_MS;
            goatd::meter::charge(pushed);
        }
    })
    .ok()?;
    Some((td, pushed))
}

/// `td` rooted where [`root_bag`] says, and the positions of the facts in
/// the order the compiler consumes them.
fn rooted(
    gp: &GroundProgram,
    deps: &Deps,
    td: &goatd::TreeDecomposition,
) -> Option<(Rooted, Vec<usize>)> {
    let num_atoms = gp.num_atoms();
    let mut position = vec![usize::MAX; num_atoms];
    for (i, a) in facts_by_use(gp, deps).into_iter().enumerate() {
        position[a as usize] = i;
    }
    let is_fact = |a: AtomId| gp.is_fact(a);
    let root_bag = root_bag(td, &position, &is_fact)?;
    Some((
        Rooted::of(num_atoms, td, &position, root_bag, &is_fact)?,
        position,
    ))
}

/// `td` [`rooted`] and folded ([`Rooted::lay_out`]).
fn fold(
    gp: &GroundProgram,
    deps: &Deps,
    td: &goatd::TreeDecomposition,
) -> Option<(Forest, usize, usize)> {
    rooted(gp, deps, td)?.0.lay_out(td, &Atop::default())
}

/// The min-fill elimination tree of the interaction graph, the one
/// `dynamic`'s right-linear start is read from (`facts_by_minfill`), folded
/// as `td`'s decomposition is: `dynamic`'s start on a program whose rules
/// read facts alone ([`super::build`]). `None` where the program has no
/// facts.
pub(super) fn min_fill_forest(gp: &GroundProgram, deps: &Deps) -> Option<(Forest, usize)> {
    let num_atoms = gp.num_atoms();
    gp.facts().next()?;
    let edges = interaction_edges(gp);
    let min_fill = min_fill_cliques(num_atoms, &edges);
    let td = min_fill.decomposition(&goatd::Graph::new(num_atoms as u32, edges))?;
    let (forest, root, _) = fold(gp, deps, &td)?;
    Some((forest, root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scaled_budget_holds_its_floor_and_ceiling() {
        // A small graph gets the floor, not a budget its size would imply:
        // below it the search lays a small graph out worse.
        assert_eq!(scaled_budget_units(14, 14), MIN_TD_BUDGET_UNITS);
        assert_eq!(scaled_budget_units(0, 0), MIN_TD_BUDGET_UNITS);
        // Past the floor it follows the graph, and stops at the ceiling.
        assert_eq!(scaled_budget_units(642, 1254), 47_400_000);
        assert_eq!(
            scaled_budget_units(1_000_000, 1_000_000),
            MAX_TD_BUDGET_UNITS
        );
        // The floor is 50 nominal milliseconds, and the ceiling stays well
        // inside the duration goatd's API can represent.
        assert_eq!(
            goatd::meter::milliseconds_for_units(MIN_TD_BUDGET_UNITS),
            50
        );
        assert_eq!(
            goatd::meter::milliseconds_for_units(MAX_TD_BUDGET_UNITS),
            129
        );
    }

    #[test]
    fn the_fold_lays_each_separator_copy_once_and_ties_it() {
        // Clusters of three facts on a binary tree, each tied to its parent
        // by clauses over both, observed false: the fold joins the subtrees
        // of a cluster's two children, whose clauses cross into the parent.
        let mut source = String::new();
        let clusters = 8;
        for i in 0..3 * clusters {
            source.push_str(&format!("0.{}::x({i}).\n", 2 + i % 7));
        }
        let mut clause = 0;
        let mut add = |source: &mut String, body: String| {
            source.push_str(&format!(
                "c({clause}) :- {body}.\nevidence(c({clause}), false).\n"
            ));
            clause += 1;
        };
        for c in 0..clusters {
            let (a, b, d) = (3 * c, 3 * c + 1, 3 * c + 2);
            add(&mut source, format!("x({a}), \\+x({b}), x({d})"));
            if c > 0 {
                let p = 3 * ((c - 1) / 2);
                add(
                    &mut source,
                    format!("x({a}), x({}), \\+x({})", p + 1, p + 2),
                );
                add(&mut source, format!("\\+x({b}), x({p}), x({})", p + 2));
                add(&mut source, format!("x({d}), \\+x({p}), x({b})"));
            }
        }
        source.push_str("query(x(0)).\n");
        let program = crate::syntax::parse_program(&source).expect("parses");
        let gp =
            crate::ground::ground(&program, &crate::GroundOptions::default()).expect("grounds");
        let fold = td_forest(&gp, &Deps::of(&gp), Some(&CopyRule::ALL))
            .expect("decomposes")
            .expect("a fold");
        let copied = fold.copied.expect("copies");
        let n = gp.num_atoms();
        let copies: Vec<AtomId> = (n..copied.num_atoms())
            .map(|a| a as AtomId)
            .filter(|&a| copied.is_fact(a))
            .collect();
        assert!(!copies.is_empty(), "{}", fold.note);
        assert!(
            fold.note
                .contains(&format!("separator copies: {} copies", copies.len())),
            "{}",
            fold.note
        );
        // Two ties per copy after the copies, each observed false.
        assert_eq!(copied.num_atoms(), n + 3 * copies.len());
        assert_eq!(copied.evidence.len(), gp.evidence.len() + 2 * copies.len());
        // The vtree holds facts alone, each once, every copy among them;
        // and an atom of the program reads each copy.
        let mut leaves = fold.forest.leaves_of(fold.root);
        assert!(leaves.iter().all(|&a| copied.is_fact(a)));
        leaves.sort_unstable();
        assert!(leaves.windows(2).all(|w| w[0] < w[1]));
        for &c in &copies {
            assert!(leaves.binary_search(&c).is_ok(), "copy {c} is not laid out");
            assert!(
                copied
                    .rules
                    .iter()
                    .any(|r| (r.head as usize) < n && r.body.iter().any(|l| l.atom() == c)),
                "nothing reads copy {c}"
            );
        }
        // Without copies, the same fold over the program's own facts.
        let plain = td_forest(&gp, &Deps::of(&gp), None)
            .expect("decomposes")
            .expect("a fold");
        assert!(plain.copied.is_none());
        assert_eq!(
            plain.forest.leaves_of(plain.root).len() + copies.len(),
            leaves.len()
        );
    }
}
