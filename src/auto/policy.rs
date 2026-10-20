//! `auto`'s policy: the trials a program gets, in order ([`candidates`]),
//! their weights and first targets, and when the searched families layout
//! takes `families:0`'s place.

use crate::GroundProgram;
use crate::ground::{self, shape};
use crate::vtree::Kind;

use super::trial::Trial;

/// A trial's first watch on the pairs of its formulas. A layout whose
/// formulas outgrow the watch stops early, having spent little, and tries
/// again with a watch that holds them.
pub(super) const FIRST_PAIRS: usize = 1 << 20;

/// The engine work a trial's first turn gets for each pair of the first
/// watch, times its weight: 2^28 units, high enough that cheap compiles
/// (Win95pts, digit) finish there, low enough to cap what a layout that
/// does not suit the program costs before the next is tried.
pub(super) const WORK_PER_PAIR: u64 = 256;

/// The work a turn's target gives for each pair its watch may grow to: a
/// turn of 2^28 units may hold formulas of 2^24 pairs. A trial that needs
/// more ends its turn and goes on in its next, with twice the target.
pub(super) const WORK_PER_HELD_PAIR: u64 = 16;

/// The doublings past its target within which a trial's projected finish
/// ([`Trial::projected`]) moves it ahead in the turns ([`Trial::key`]); a
/// projection further out, or none, counts this many. A trial projected
/// to finish within its next turn so takes its turns this many doublings
/// ahead of the trials that make no projection, and one projected further
/// out fewer, by how far out. The projections hold within it: over the
/// circuits, a trial projected to finish within one doubling did so 71%
/// of the time, between one and two doublings out within two doublings
/// 71%, and between four and six within four 49%, where trials projected
/// past six doublings, or making no projection, finished within four
/// doublings 16 to 29% of the time. The bound is what a wrong projection
/// costs: no trial runs more than this many doublings ahead of another
/// by its projection.
pub(super) const PROJECTED_DOUBLINGS: f64 = 3.0;

/// The work a trial has spent from which its compile stops between
/// components where it is projected to finish more than
/// [`PROJECTED_DOUBLINGS`] doublings past its target (`tp::Horizon`): the
/// work whose watch is the first one, `first_pairs` ([`FIRST_PAIRS`] in a
/// run) times [`WORK_PER_HELD_PAIR`], 2^24 units, about a tenth of a
/// second. Before it the components done are too few to project from, and
/// a turn of less costs nothing that matters. `decision` on SPPL's markov
/// chains has finished 32 of their 24 k to 477 k components at 2^23 units,
/// and two more for each doubling past it, where the circuits it wins have
/// finished 1.3 to 16% of theirs at 2^23 and go on at about that pace or
/// faster.
pub(super) fn projected_from(first_pairs: usize) -> u64 {
    (first_pairs as u64).saturating_mul(WORK_PER_HELD_PAIR)
}

/// Where `program` goes first, its first turn and those of the two trials
/// after it (`program:fanin` and a families or `closed` trial) are this
/// much shorter than a first turn: the circuits whose views were inlined
/// are won by one of them, and the turns double, so the run costs about
/// twice the winner's work for each.
pub(super) const LEAD_SHARE: u64 = 256;

/// The share of the work `families:0` or a `decision` trial gets beside
/// each other trial, and the searched families trial once it takes
/// `families:0`'s place, whichever trial leads ([`super::weigh_searched`]).
/// Every benchmark
/// network that no trial finishes in its first attempt is won by a
/// families trial, and on those the other trials' work was most of the
/// run; where they spend past their estimates (the weather networks) they
/// lose the weight ([`WORK_PER_ESTIMATED_PAIR`]). The bet is on networks:
/// where `program` goes first, on the programs whose views the grounder
/// inlined ([`candidates`]), the families trials hold the weight only if
/// `families:0`'s rewrite ties some table by clauses, as a network's does
/// ([`weight`]). Without the weight `program` took up to five times main's
/// time beside it on the UAI mastermind networks (06_08_03_0014 1.0 s to
/// 5.0), which `families:0` wins; with it `families:0` spent eight shares
/// on circuits it does not win, which tie no table by clauses, its layouts
/// estimated at up to 1e14 pairs, which no overspending reaches (Alea's
/// pi_w12 8 s to 57, disease_small 66 s to 401, sorted_list 66 s to 102).
/// The searched trial otherwise weighs 1: its layout
/// estimated about as large as `families:0`'s, the bet is `families:0`'s,
/// and at the weight its first attempt spent 2.1 G units on GeNI's
/// weather_n7 and n8, which `program` wins.
pub(super) const FAMILIES_WEIGHT: u64 = 8;

/// The work a families trial may spend for each pair of its layout's
/// estimate before the estimate is taken as wrong and the trial loses its
/// weight ([`FAMILIES_WEIGHT`]), which bets on the estimate. The families
/// trials that win, alone, have spent at most 7 units a pair on the
/// networks and up to 150 on the small programs they finish in their first
/// turn (mnist_sum96); those that lose have spent 300 and more (GeNI's
/// links and weather networks, 3 600 on weather_n7), or never finish
/// (Dice's simpleCaesar, estimated at 7.9e4 pairs, where `td` finishes).
/// Their formulas need not reach the watch, which grows with the target.
pub(super) const WORK_PER_ESTIMATED_PAIR: u64 = 64;

/// Whether `t`, a families trial, has spent past its layout's estimate
/// ([`WORK_PER_ESTIMATED_PAIR`]).
pub(super) fn overspent_estimate(t: &Trial) -> bool {
    matches!(t.kind, Kind::Families { .. })
        && t.estimate
            .is_some_and(|e| t.spent as f64 > e.pairs * WORK_PER_ESTIMATED_PAIR as f64)
}

/// How much smaller the searched `families` layout must be estimated than
/// `families:0`'s to take its place ([`search_due`]). The estimate decides
/// where the gap is wide (bn_munin1); where the gap is narrow it misleads
/// by less than it (bn_andes). Past what a turn may hold both estimates
/// run far over the formulas, but the gap still points the right way
/// (pedigree7_10, and GeNI's link_12 at 4.15, where the searched layout
/// wins at its weight and `program` takes 27 s).
pub(super) const SEARCHED_GAP: f64 = 4.0;

/// goatd's search units to a unit of engine work, by their times: the
/// families searches charge about twice as many units a second as the
/// trials spend units of work (the median of the runs with a search on
/// the network sets; quartiles 1.1 and 3.8).
pub(super) const SEARCH_UNITS_PER_WORK: u64 = 2;

/// The work past which `laid`, the `families:0` trial, has the searched
/// `families` layout built and weighed against its own
/// ([`SEARCHED_GAP`]) before `closed` and `td` have their turns: what the
/// search charges, in units of work ([`SEARCH_UNITS_PER_WORK`]), where
/// `laid`'s layout is estimated past what its turn may hold and its
/// rewrite ties tables by clauses. Otherwise `closed` and `td` spend their
/// turns before the searched layout is built and wins (bn_munin1). The
/// relational programs whose estimates are as large (cora500, uwcse10) tie
/// no table by clauses and are won by `closed` or `td`. Until then `laid`
/// may still finish for less than the search, as in renting until the
/// rent paid equals the price.
pub(super) fn search_charge(laid: &Trial) -> Option<u64> {
    let e = laid.estimate?;
    (laid.kind == (Kind::Families { search: false })
        && e.pairs > (laid.target / WORK_PER_HELD_PAIR) as f64
        && ties_tables_by_clauses(laid))
    .then_some(e.search_units / SEARCH_UNITS_PER_WORK)
}

/// The work `laid`'s next attempt may spend, `left` by its turn, where the
/// attempt stops at the search's charge ([`search_charge`]) for the
/// searched layout to be weighed: where it would otherwise spend past
/// twice the charge. Rent is paid in whole attempts up to that, so the
/// rent paid when the search is bought is at most twice its price: an
/// attempt stopped at the charge loses the operation under way, and the
/// rest of the turn may not hold it again (mastermind's single query
/// operation of 6.9e7 units, stopped at 6.6e7 and given 6.8e7 after).
pub(super) fn search_rent(laid: &Trial, left: u64) -> Option<u64> {
    search_charge(laid)
        .filter(|&charge| {
            laid.spent < charge && laid.spent.saturating_add(left) > charge.saturating_mul(2)
        })
        .map(|charge| charge - laid.spent)
}

/// Whether `laid` has spent its [`search_charge`].
pub(super) fn search_due(laid: &Trial) -> bool {
    search_charge(laid).is_some_and(|charge| laid.spent >= charge)
}

/// Whether `t`'s rewrite holds some table's clauses
/// ([`crate::indicators::clause_links`]): rows that partition their
/// inputs, as a Bayesian network's do.
pub(super) fn ties_tables_by_clauses(t: &Trial) -> bool {
    t.ground
        .as_ref()
        .is_some_and(|gp| !crate::indicators::clause_links(gp).is_empty())
}

/// The weight of a trial of `kind`: [`FAMILIES_WEIGHT`] for `families:0`
/// and `decision` where the families trials are `weighted`
/// ([`families_weighted`]).
pub(super) fn weight(kind: Kind, weighted: bool) -> u64 {
    match kind {
        Kind::Families { search: false } | Kind::Decision if weighted => FAMILIES_WEIGHT,
        _ => 1,
    }
}

/// Whether the families trials of a run led by `lead`, the first in order,
/// hold [`FAMILIES_WEIGHT`]: unless `program` leads and `laid`, the
/// `families:0` trial with its rewrite, ties no table by clauses
/// ([`ties_tables_by_clauses`]) or is missing.
pub(super) fn families_weighted(lead: Kind, laid: Option<&Trial>) -> bool {
    !matches!(lead, Kind::Program { .. }) || laid.is_some_and(ties_tables_by_clauses)
}

/// Concrete layouts in trial order. The families entries use rewritten
/// ground programs. They come first, over the min-fill decompositions
/// alone, except where the program has few facts and many derived atoms
/// for each ([`FEW_FACTS`]): with `closed` first the networks would pay for
/// a `closed` trial that `families` then beats, and with the decomposer's
/// search in the first families trials the small programs would pay for a
/// search that lowers nothing. The searched layouts come last, so a program
/// pays for the search only once every cheaper layout has had its turn,
/// unless `families:0`'s estimate calls for it sooner ([`search_charge`]).
///
/// Each trial has its own budget, so a trial that never finishes first
/// changes no answer and only costs its work.
///
/// A program whose derived atoms are one component, or one recursive
/// component that reads most of the facts, has no boundary for an
/// indicator to cut and no component order for `closed` to search that
/// matters ([`shape::nothing_to_order`]): only an order of the
/// facts themselves can help, and its trial is `dynamic`: the min-fill
/// vtree, searched by rotations between the compile's steps
/// ([`tp::Compilation::dynamic`]), unprojected, since the component's cone
/// is most of the facts. The search beats the static min-fill vtree it
/// starts from on the larger random graphs; `td` does no better than
/// min-fill there, and its decomposer's least budget costs seconds on
/// programs that compile in milliseconds.
///
/// Where no derived atom depends on itself, the one component is acyclic
/// atoms merged because they read the same facts ([`crate::ground::deps::components`]),
/// and the atoms still have a direction: a formula compiled bottom up wants
/// its facts in reverse dependency order, which `closed` follows and
/// `minfill`, `dynamic`'s start, does not see (`docs/design.md`, vtrees).
/// There `closed` follows `dynamic`: a noisy-or network with every finding
/// observed is one such component, and so is the growing-head program,
/// whose heads share each annotated disjunction's switches; `dynamic`
/// finishes neither, and `closed` both.
///
/// `td` follows `closed` there: on the tuple-independent databases from
/// the literature ([`scripts/gen_pdb.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/gen_pdb.py)) both of the others stall, and
/// `td` alone finishes pdb40.
///
/// Where every rule reads facts alone, or copies of facts
/// ([`shape::reads_facts_alone`]), the atoms have no direction and `td`
/// comes first, then `dynamic` and `closed`. `td`'s vtree there is its
/// decomposition folded ([`vtree::build`]) and its compile is projected, so
/// the evidence walk conjoins each observation at the lowest vtree node
/// over its facts. `dynamic` starts from a fold too, but compiles
/// unprojected: every formula is on the global scope, and the walk conjoins
/// every observation at the root. A weighted CNF, its clauses observed
/// true, is such a program, and so are the literature's noisy-or networks
/// and databases and the TPC-H queries, where `td` first takes less trial
/// work.
///
/// Where the grounder inlined views (atoms of one rule read once,
/// `src/ground/inline.rs`), the program is built gate by gate, as the
/// circuits Alea, Dice and pluck export are: each gate is a derived atom
/// of one rule, and the program states its flips in the order its front
/// end created them. `program` comes first there, then `program:fanin`,
/// ahead of the families trials, and `program` alone comes last
/// elsewhere: on the programs with views, the circuits compile best in
/// the order their front end built them (triangle_small 24 s to 0.7) or
/// in the order their gates read the facts (`program:fanin`: an adder's
/// operand bits side by side, addFun_sum at 19 bits in 42 M units of work
/// against 4.3 G; pluck_hmm in 1 280 pairs in the file's order, out of
/// memory in the walk's), while first on every program `program` cost the
/// networks and grids, which have no views, 3 to 60 times. `td` is no
/// trial there: its order comes from a decomposition of the undirected
/// interaction graph, which does not see the gates' direction, where the
/// program's two orders and `closed`'s search over dependency orders
/// follow it. In the timed runs of the circuits, GeNI's networks and
/// Roulette's programs, all built with views, `td` won none, and its
/// turns took a fifth of the time the losing trials spent on the
/// circuits of a second or more.
///
/// Where the program is a CNF over its facts and there are several units
/// ([`shape::clauses_over_facts`]: every derived atom a copy of a fact or
/// observed false and read by nothing), the families trials are left out
/// and `td` comes before `closed`, as where every rule reads facts alone: a
/// clause reads facts or copies of facts. A weighted CNF is such a program,
/// its copies the facts the conditioning on the evidence makes equal, which
/// split its clauses into several units where without them they are one,
/// and `td` comes first above. The families trials spend their turns there
/// before another trial finishes (on weighted CNFs). `closed` lays each
/// copied fact out in a block below the clauses, away from where they read
/// it, and the conditioning's search for backbones and equal facts
/// (`src/ground/preprocess/backbone.rs`) makes more copies, so `td` comes
/// before it. A Markov network's factors, observed true, read their
/// variables' values as copies of facts too, and keep the families trials,
/// which they win.
///
/// Every trial compiles the program written without its linear recursion
/// (`Kind::rewrite`), but the trials are chosen from the program as
/// grounded: its rewrite is acyclic, a component per atom, where the
/// families layouts are first and fail on the random graphs. On the
/// rewrite `dynamic` takes less work than `closed` on the random graphs
/// and bidirectional grids, and stays the one trial of a recursive
/// component.
pub(super) fn candidates(gp: &GroundProgram) -> Vec<Kind> {
    if shape::nothing_to_order(gp) {
        let by_head = gp.rules_by_head();
        let recursive = ground::deps::derived_sccs(gp, &by_head)
            .iter()
            .any(|c| ground::deps::is_cyclic(gp, &by_head, c));
        if recursive {
            return vec![Kind::Dynamic];
        }
        if shape::reads_facts_alone(gp) {
            return vec![Kind::Td, Kind::Dynamic, Kind::Closed];
        }
        return vec![Kind::Dynamic, Kind::Closed, Kind::Td];
    }
    if shape::clauses_over_facts(gp) {
        return vec![Kind::Td, Kind::Closed];
    }
    let laid = Kind::Families { search: false };
    let searched = Kind::Families { search: true };
    let mut trials =
        if gp.num_facts() <= FEW_FACTS && gp.num_derived() >= DERIVED_PER_FACT * gp.num_facts() {
            vec![Kind::Closed, laid, Kind::Td, searched]
        } else {
            vec![laid, Kind::Closed, Kind::Td, searched]
        };
    if gp.stats.inlined > 0 {
        trials.retain(|&kind| kind != Kind::Td);
        trials.splice(
            0..0,
            [
                Kind::Program { fan_in: false },
                Kind::Program { fan_in: true },
            ],
        );
    } else {
        trials.push(Kind::Program { fan_in: false });
    }
    trials
}

/// Facts up to which, and derived atoms per fact from which, `closed`
/// comes before the families trials. With few facts and many derived
/// atoms for each, the families rewrite copies many atoms per fact, and
/// its rewrite and layout are paid for a trial that `closed` beats
/// (ace_fs_28, mastermind10_4_8). The bound on derived atoms per fact keeps
/// `families:0` first on the networks and grids with fewer, which it wins
/// (mnist_sum4, sickness6_10, grid6); the programs `closed` wins have 16 or
/// more.
pub(super) const FEW_FACTS: usize = 64;
pub(super) const DERIVED_PER_FACT: usize = 8;
