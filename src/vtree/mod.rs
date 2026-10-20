//! Vtree construction over the probabilistic facts of a ground program.
//!
//! [`Kind::Td`] and [`Kind::Dynamic`] read the program's
//! *interaction graph* — one vertex per ground atom, and one clique per ground
//! rule over its head and body atoms — and turn that structure into a vtree:
//!
//! - [`Kind::Td`] decomposes the graph with goatd and reads a fact
//!   order off the decomposition, which goes in a right-linear vtree; where
//!   every rule reads facts alone, the vtree is shaped like the bag tree,
//!   over the program with copies of separator facts (`separators`).
//! - [`Kind::Dynamic`] eliminates the graph in min-fill order, lays
//!   the facts out in a right-linear vtree in a pre-order of the elimination
//!   tree, and searches rotations of it during the compile.
//! - [`Kind::Closed`] reads the dependency graph instead, and is
//!   bushy only where the program's parts share nothing until a later rule
//!   reads them together.
//! - [`Kind::Families`] lays out a program that reads atoms through
//!   indicators ([`crate::indicators`]).
//! - [`Kind::Program`] reads no graph: it takes the facts in the
//!   order the program states them or, as `program:fanin`, in the fan-in
//!   order of the rules from the queries and the evidence, interleaved over
//!   them (`ground::deps::fan_in`), and names the derived atoms that
//!   cross a narrow cut of that order (`naming`).
//! - [`Kind::Decision`] reads no graph either: it lays out the
//!   decisions the grounder read the facts as guards in
//!   ([`crate::ground::Decisions`]), and names the contexts of their tree
//!   and the atoms of the tree that atoms outside it read (`decision`).
//!
//! Derived atoms take part in every construction and become no leaves: only
//! the probabilistic facts are variables of the diagram. Every strategy
//! answers the same queries with the same numbers; what differs is the size of
//! the diagrams on the way.

mod closed;
mod cost;
mod decision;
mod families;
mod forest;
mod graph;
mod naming;
mod nest;
pub(crate) mod order;
mod separators;
mod td;

pub(crate) use graph::{log2_add, min_fill_weight};

use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use tididi::Vtree;
use tididi::vtree::VarId;

use crate::ground::deps::Deps;
use crate::ground::shape::reads_facts_alone;
use crate::ground::{AtomId, GroundProgram};

use closed::closed_forest;
use families::families_forest;
use graph::facts_by_minfill;
use separators::CopyRule;
use td::{TdFold, metered, min_fill_forest, td_forest};

/// Seed handed to goatd. Fixed, so that two runs of the same program build the
/// same vtree.
const TD_SEED: u64 = 0;

/// A layout: how to build the vtree. Each is a trial `auto` runs
/// ([`crate::Strategy`]), under the name its `Display` prints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Right-linear over a fact order read off a tree decomposition of the
    /// interaction graph (`td_forest`), or, where every rule reads facts
    /// alone, the decomposition itself folded into a vtree (`td_layout`). The
    /// decomposer's search budget scales with the size of the graph, up to
    /// `MAX_TD_BUDGET_UNITS`; it is charged graph work, not time. A graph
    /// that min-degree does not eliminate within its budget has no `td`
    /// layout (`td_forest`).
    Td,
    /// For a program that reads atoms through indicators: a tree
    /// decomposition of the indicators, with each group's private facts on
    /// a spine above its indicators (`families_forest`). With `search`
    /// the decomposer searches, under `td`'s budget, where min-fill leaves
    /// a wide decomposition; without it (named `families:0`) the min-fill
    /// decompositions are laid out alone.
    Families { search: bool },
    /// Bushy where the program's dependencies let it be, right-linear
    /// everywhere else (`closed_forest`): every subtree holds a set of
    /// facts that decides the atoms it feeds without reading any atom from
    /// outside it. The components are taken in a dependency order found by
    /// local search (`order::search`) on a proxy for the diagram
    /// sizes, which charges every fact inside a formula's span the
    /// assignments to the atoms alive across its cut.
    Closed,
    /// Right-linear over the facts in a pre-order of the min-fill
    /// elimination tree of the interaction graph (`facts_by_minfill`), the
    /// fact eliminated last first, or, where every rule reads facts alone,
    /// that tree itself folded as `td`'s decomposition is (`build`); and
    /// changed during the compile by a
    /// search of vtree rotations over every live formula at once whenever
    /// they have doubled in size (`tp::Compilation::dynamic`). The formulas
    /// all live on the one global vtree: a dynamic strategy compiles
    /// without projection. Named `dynamic:minfill`, after the vtree it
    /// starts from.
    Dynamic,
    /// Right-linear over the facts in the reverse of the order the program
    /// states them, so that the diagram reads them, as an OBDD would, in
    /// program order. A program that builds a circuit bit by bit and states
    /// its flips as it creates them (Alea's exported circuits) is laid out
    /// as its own BDD package laid it out. Named `program`. With `fan_in`,
    /// in the fan-in order of the rules from the evidence and the queries,
    /// interleaved over them, instead (`ground::deps::fan_in`): the bits of
    /// the operands a gate combines side by side, and those of an operand
    /// that several gates read beside each of the others', where a front
    /// end states each number's bits together. Named `program:fanin`. Either
    /// order names the derived atoms that cross its narrow cuts, read after
    /// them through copies, where it has any.
    Program { fan_in: bool },
    /// The tree of the decisions the grounder read facts as guards in
    /// ([`crate::ground::Decisions`]): each fact above the facts it decides
    /// between, with the facts the decisions leave out right-linear below
    /// the root's right, in the fan-in order of the shallowest roots first
    /// (`ground::deps::fan_in`), save those read beside one part of the
    /// tree only, which go above it. A circuit of nested if-then-elses is
    /// laid out as its own decision tree. Where atoms outside the tree read it,
    /// the contexts of the tree and the atoms the readers read are named,
    /// read through copies (`decision`). Named `decision`; a program in
    /// which the grounder read no fact as a guard has none, and the
    /// strategy fails.
    Decision,
}

/// Every layout's name, for the error message: the trials `auto` runs,
/// under the names its trial lines print, `program` and `decision`.
pub(crate) const KIND_NAMES: &str =
    "td, closed, families, families:0, dynamic[:minfill], program, program:fanin, decision";

impl Kind {
    /// Whether the compile searches for a better vtree as it goes.
    pub fn is_dynamic(self) -> bool {
        matches!(self, Kind::Dynamic)
    }
}

impl FromStr for Kind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "td" => Ok(Kind::Td),
            "closed" => Ok(Kind::Closed),
            "families" => Ok(Kind::Families { search: true }),
            "families:0" => Ok(Kind::Families { search: false }),
            "dynamic" | "dynamic:minfill" => Ok(Kind::Dynamic),
            "program" => Ok(Kind::Program { fan_in: false }),
            "program:fanin" => Ok(Kind::Program { fan_in: true }),
            "decision" => Ok(Kind::Decision),
            _ => Err(format!("unknown vtree strategy `{s}` (auto, {KIND_NAMES})")),
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Td => f.write_str("td"),
            Kind::Closed => f.write_str("closed"),
            Kind::Families { search: true } => f.write_str("families"),
            Kind::Families { search: false } => f.write_str("families:0"),
            Kind::Dynamic => f.write_str("dynamic:minfill"),
            Kind::Program { fan_in: false } => f.write_str("program"),
            Kind::Program { fan_in: true } => f.write_str("program:fanin"),
            Kind::Decision => f.write_str("decision"),
        }
    }
}

/// Fact atoms in program order.
pub fn facts_in_order(gp: &GroundProgram) -> Vec<AtomId> {
    gp.facts().map(|(a, _)| a).collect()
}

/// What a layout's diagrams are estimated to hold ([`build`]).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Estimate {
    /// The pairs.
    pub(crate) pairs: f64,
    /// The units goatd's searches would charge to lay the program out
    /// searched: twice the budget of each numbering's search (goatd's hard
    /// cutoff, which the search takes), none where no search would run
    /// ([`families::FAMILIES_SEARCH_FROM_WIDTH`], min-fill's width at the
    /// graph's lower bound, or min-fill charged more than the search may
    /// spend), and the searched layout is `families:0`'s.
    pub(crate) search_units: u64,
}

/// A vtree and the fact order it is over, as [`build`] lays them out.
pub(crate) struct Layout {
    /// The vtree.
    pub(crate) vtree: Arc<Vtree>,
    /// The facts in the order of the variables: fact `order[i]` is
    /// `VarId(i + 1)`.
    pub(crate) order: Vec<AtomId>,
    /// A diagnostic line for `--stats`: the search, when the strategy ran
    /// one (`closed`, `td`, `families`, `dynamic`), or the names it made
    /// (`program`, `decision`).
    pub(crate) note: Option<String>,
    /// The pairs the layout's diagrams are estimated to hold, where the
    /// strategy estimates them.
    pub(crate) estimate: Option<Estimate>,
}

impl Layout {
    /// The layout of `forest` from `root`.
    fn of(
        forest: &forest::Forest,
        root: usize,
        note: Option<String>,
        estimate: Option<Estimate>,
    ) -> Self {
        let (vtree, order) = forest.finish(root);
        Layout {
            vtree: Arc::new(vtree),
            order,
            note,
            estimate,
        }
    }

    /// A right-linear vtree over `order`, whose TDD is the quasi-reduced
    /// OBDD reading `order` from its last fact.
    fn right_linear(order: Vec<AtomId>, note: Option<String>) -> Self {
        let vtree = Vtree::linear_from_order(&variables(&order)).expect("nonempty order");
        Layout {
            vtree: Arc::new(vtree),
            order,
            note,
            estimate: None,
        }
    }

    /// A balanced vtree over `order`.
    fn balanced(order: Vec<AtomId>) -> Self {
        let vtree = Vtree::balanced_over(&variables(&order)).expect("nonempty order");
        Layout {
            vtree: Arc::new(vtree),
            order,
            note: None,
            estimate: None,
        }
    }
}

/// The variables of the facts `order`, `VarId(1)` on, and one where it is
/// empty: with no facts at all a single dummy variable keeps the library
/// happy.
fn variables(order: &[AtomId]) -> Vec<VarId> {
    (1..=order.len().max(1) as u32).map(VarId).collect()
}

/// Lay out the vtree over the facts, and the program it is over where
/// that is not `gp`: `td`'s with the separator copies its fold makes
/// ([`td_layout`]), `program`'s with the atoms crossing its narrow cuts
/// named (`naming`), `decision`'s with its tree's contexts and the atoms
/// its readers outside it read named (`decision`). `families` estimates
/// its layout's pairs by its layout cost
/// ([`cost::Interfaces::layout_cost`]), which is near the evidence's pairs
/// on the grids and uwcse and can be far over them on networks. A `td`,
/// `families`, `closed` or `decision` layout that finds no facts, or no
/// decomposition or tree covering one, lays the facts out balanced in
/// program order. `deps` are `gp`'s ([`Deps::of`]). `Err` where the layout
/// cannot be built, or cannot help: `decision` on a program the grounder
/// read no fact of as a guard, `td` where no elimination of the graph
/// finishes within its budget ([`td_forest`]).
pub(crate) fn build(
    gp: &GroundProgram,
    deps: &Deps,
    kind: Kind,
) -> Result<(Layout, Option<GroundProgram>), String> {
    let laid = match kind {
        Kind::Td => td_layout(gp, deps, Some(&CopyRule::ON))?,
        Kind::Families { search } => {
            families_forest(gp, deps, search)?.map(|(forest, root, note, estimate)| {
                (Layout::of(&forest, root, Some(note), Some(estimate)), None)
            })
        }
        Kind::Closed => closed_forest(gp, deps)
            .map(|(forest, root, note)| (Layout::of(&forest, root, Some(note), None), None)),
        Kind::Dynamic => Some((dynamic_layout(gp, deps), None)),
        Kind::Program { fan_in } => {
            // A right-linear vtree over x1 .. xn is read as an OBDD
            // reading xn first. Where the order has narrow cuts, the vtree
            // is over the program with the atoms crossing them named
            // ([`naming`]).
            let laid = naming::lay_out(gp, deps, fan_in);
            let (program, note) = laid.named.unzip();
            let order = laid.facts.into_iter().rev().collect();
            Some((Layout::right_linear(order, note), program))
        }
        Kind::Decision => decision_layout(gp, deps)?,
    };
    Ok(laid.unwrap_or_else(|| (Layout::balanced(facts_in_order(gp)), None)))
}

/// `dynamic`'s first vtree. Where every rule reads facts alone, it starts
/// bushy, from the min-fill elimination tree folded as `td`'s
/// decomposition is: no class at a vtree node then reads atoms decided
/// outside it, and a node's size follows the facts its subtree shares with
/// the rest, where a right-linear order through the tree pays for every
/// fact alive across each cut. Otherwise it starts right-linear, in
/// min-fill order. The note gives the units the min-fill elimination and
/// the decomposition read from it charged to goatd's meter.
fn dynamic_layout(gp: &GroundProgram, deps: &Deps) -> Layout {
    let (layout, units, _) = metered(|| {
        match reads_facts_alone(gp)
            .then(|| min_fill_forest(gp, deps))
            .flatten()
        {
            Some((forest, root)) => Layout::of(&forest, root, None, None),
            None => Layout::right_linear(facts_by_minfill(gp), None),
        }
    });
    Layout {
        note: Some(format!("dynamic: min-fill {units} units charged")),
        ..layout
    }
}

/// `decision`'s layout of `gp`, the tree of its decisions, and the program
/// it is over where its tree's contexts and the atoms its readers outside
/// it read are named ([`decision::Tree::name`]). `Ok(None)` where the tree
/// covers no fact, as on a program the evidence left without facts, and
/// `Err` where the grounder read no fact of `gp` as a guard.
fn decision_layout(
    gp: &GroundProgram,
    deps: &Deps,
) -> Result<Option<(Layout, Option<GroundProgram>)>, String> {
    let decisions = gp
        .stats
        .decisions
        .as_ref()
        .ok_or("vtree `decision`: the grounder read no fact of this program as a guard")?;
    let tree = decision::Tree::of(decisions, gp, deps);
    let homes = tree.homes();
    let named = tree.name();
    let (program, above) = match &named {
        Some(named) => (&named.gp, named.above.as_slice()),
        None => (gp, &[][..]),
    };
    let Some((forest, root)) = decision::decision_forest(decisions, program, above, &homes) else {
        return Ok(None);
    };
    let (program, note) = named.map(|n| (n.gp, n.note)).unzip();
    Ok(Some((Layout::of(&forest, root, note, None), program)))
}

/// `td`'s layout of `gp`, the fold of its decomposition ([`td_forest`]),
/// with the program it is over where that is not `gp`. Where every rule
/// reads facts alone, the vtree is the fold itself, over `gp` with the
/// separator copies `copies` makes ([`separators`]), as `dynamic`'s start
/// is there ([`dynamic_layout`]): no class at a vtree node then reads an
/// atom decided outside it, and a node's size follows the facts its
/// subtree shares with the rest, at most a bag, where the cuts of a
/// right-linear order through the decomposition hold every fact alive on a
/// root path. Elsewhere the fold decides the order of the facts and
/// nothing else: the vtree over it is right-linear. `Ok(None)` where the
/// program has no facts or the decomposition covers none, and `Err` where
/// `td` cannot help ([`td_forest`]).
fn td_layout(
    gp: &GroundProgram,
    deps: &Deps,
    copies: Option<&CopyRule>,
) -> Result<Option<(Layout, Option<GroundProgram>)>, String> {
    let alone = reads_facts_alone(gp);
    let Some(TdFold {
        forest,
        root,
        note,
        copied,
    }) = td_forest(gp, deps, copies.filter(|_| alone))?
    else {
        return Ok(None);
    };
    let layout = Layout::of(&forest, root, Some(note), None);
    Ok(Some(if alone {
        (layout, copied)
    } else {
        (Layout::right_linear(layout.order, layout.note), None)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::{GroundOptions, ground};
    use crate::syntax::parse_program;
    use rustc_hash::FxHashSet;

    const SMOKERS: &str = "
        0.3::stress(X) :- person(X).
        0.2::influences(X,Y) :- person(X), person(Y).
        person(p1). person(p2). person(p3).
        friend(p1,p2). friend(p2,p1). friend(p2,p3). friend(p3,p2).
        smokes(X) :- stress(X).
        smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).
        query(smokes(p1)).
    ";

    pub(super) fn ground_smokers() -> GroundProgram {
        let program = parse_program(SMOKERS).expect("parses");
        ground(&program, &GroundOptions::default()).expect("grounds")
    }

    #[test]
    fn strategy_names_round_trip() {
        for name in [
            "td",
            "closed",
            "families",
            "families:0",
            "dynamic:minfill",
            "program",
            "program:fanin",
            "decision",
        ] {
            let s: Kind = name.parse().expect("a known name");
            assert_eq!(s.to_string(), name);
        }
        assert_eq!(
            "dynamic".parse::<Kind>().expect("the default base"),
            Kind::Dynamic
        );
        assert_eq!(
            "families:0".parse::<Kind>().expect("no search"),
            Kind::Families { search: false }
        );
        // Only the trials `auto` runs, `program` and `decision` have names.
        for name in ["auto", "nope"] {
            assert!(name.parse::<Kind>().is_err(), "{name}");
        }
    }

    /// Every strategy has to hand `tp` a numbering of exactly the facts.
    #[test]
    fn every_strategy_covers_the_facts_once() {
        let gp = ground_smokers();
        let facts: FxHashSet<AtomId> = gp.facts().map(|(a, _)| a).collect();
        assert!(!facts.is_empty());
        for strategy in [
            Kind::Td,
            Kind::Closed,
            Kind::Dynamic,
            Kind::Program { fan_in: false },
            Kind::Program { fan_in: true },
        ] {
            let (Layout { vtree, order, .. }, _) =
                build(&gp, &Deps::of(&gp), strategy).expect("builds");
            let seen: FxHashSet<AtomId> = order.iter().copied().collect();
            assert_eq!(seen, facts, "{strategy} misses or repeats a fact");
            assert_eq!(order.len(), seen.len(), "{strategy} repeats a fact");
            assert_eq!(
                vtree.num_leaves() as usize,
                order.len(),
                "{strategy} leaf count"
            );
            assert_eq!(
                vtree.num_vars() as usize,
                order.len(),
                "{strategy} variable space"
            );
        }
    }

    /// `program` reads the facts in program order, and `program:fanin` in
    /// the interleaved fan-in order: each vtree is right-linear over its
    /// order's reverse.
    #[test]
    fn program_lays_the_facts_out_right_linear_in_reverse() {
        let gp = ground_smokers();
        let met = crate::ground::deps::fan_in(&gp, true);
        for fan_in in [false, true] {
            let (Layout { vtree, order, .. }, _) =
                build(&gp, &Deps::of(&gp), Kind::Program { fan_in }).expect("builds");
            let mut facts = facts_in_order(&gp);
            if fan_in {
                facts.sort_by_key(|&a| met[a as usize]);
            }
            facts.reverse();
            assert_eq!(order, facts, "fan_in {fan_in}");
            assert!(!branches(&vtree));
        }
    }

    /// Whether some internal node of `vtree` has two internal children,
    /// which no right-linear vtree has.
    fn branches(vtree: &Vtree) -> bool {
        let text = vtree.to_text();
        let internal: Vec<Vec<&str>> = text
            .lines()
            .filter_map(|l| l.strip_prefix("I "))
            .map(|l| l.split_whitespace().collect())
            .collect();
        let is_internal = |id: &str| internal.iter().any(|n| n[0] == id);
        internal
            .iter()
            .any(|n| is_internal(n[1]) && is_internal(n[2]))
    }

    /// `td` keeps its decomposition's fold as the vtree where every rule
    /// reads facts alone, and lays its order out right-linear elsewhere.
    #[test]
    fn td_is_the_fold_where_every_rule_reads_facts_alone() {
        // Clauses observed true on a binary tree of 15 facts, each over a
        // fact and its two children, as a weighted CNF is written.
        let mut source = String::new();
        for i in 0..15 {
            source.push_str(&format!("0.5::x({i}).\n"));
        }
        for i in 0..7 {
            source.push_str(&format!(
                "c({i}) :- \\+x({i}), \\+x({}), \\+x({}).\nevidence(c({i}), false).\n",
                2 * i + 1,
                2 * i + 2
            ));
        }
        source.push_str("query(x(0)).\n");
        let gp = ground(
            &parse_program(&source).expect("parses"),
            &GroundOptions::default(),
        )
        .expect("grounds");
        assert!(reads_facts_alone(&gp));
        let (Layout { vtree, .. }, _) = build(&gp, &Deps::of(&gp), Kind::Td).expect("builds");
        assert!(branches(&vtree), "{}", vtree.to_text());
        let smokers = ground_smokers();
        assert!(!reads_facts_alone(&smokers));
        let (Layout { vtree, .. }, _) =
            build(&smokers, &Deps::of(&smokers), Kind::Td).expect("builds");
        assert!(!branches(&vtree), "{}", vtree.to_text());
    }
}
