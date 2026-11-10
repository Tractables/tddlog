//! A compile stopped part way through a cyclic component resumes at the
//! atom update that stopped it, keeping the rounds done, and answers as
//! the worlds say.

use super::*;
use crate::ground::deps::Deps;
use crate::ground::{GroundOptions, ground};
use crate::query::query_probabilities;
use crate::syntax::parse_program;
use crate::vtree::{self, Kind};

const EDGES: [(u32, u32, f64); 13] = [
    (1, 2, 0.6),
    (2, 3, 0.5),
    (3, 1, 0.4),
    (3, 4, 0.7),
    (4, 2, 0.3),
    (4, 5, 0.8),
    (5, 6, 0.5),
    (6, 4, 0.6),
    (6, 7, 0.4),
    (7, 5, 0.7),
    (2, 6, 0.2),
    (7, 1, 0.5),
    (5, 3, 0.3),
];

/// P(path(1, n)) for n = 2..7, by enumerating the worlds of the edges.
fn exact() -> Vec<f64> {
    let mut p = [0.0; 8];
    for world in 0u32..1 << EDGES.len() {
        let mut weight = 1.0;
        for (i, &(_, _, q)) in EDGES.iter().enumerate() {
            weight *= if world >> i & 1 == 1 { q } else { 1.0 - q };
        }
        let mut reached = [false; 8];
        let mut stack = vec![1u32];
        while let Some(x) = stack.pop() {
            for (i, &(a, b, _)) in EDGES.iter().enumerate() {
                if a == x && world >> i & 1 == 1 && !reached[b as usize] {
                    reached[b as usize] = true;
                    stack.push(b);
                }
            }
        }
        for n in 2..8 {
            if reached[n] {
                p[n] += weight;
            }
        }
    }
    p[2..].to_vec()
}

/// The answers, the compile's atom updates, and how many runs stopped
/// part way through the cyclic component, compiling under `watches` in
/// turn until one run finishes, on the min-fill vtree `dynamic` starts
/// from, which stays put here.
fn compile(
    gp: &GroundProgram,
    formulation: Formulation,
    watches: &[Option<usize>],
) -> (Vec<f64>, usize, usize) {
    let deps = Deps::of(gp);
    let (vtree::Layout { vtree, order, .. }, _) =
        vtree::build(gp, &deps, Kind::Dynamic).expect("builds");
    vtree.context().run(|engine| {
        let backend = TddBackend::new(engine, vtree.clone());
        let var_of = fact_variables(gp, &order);
        let mut compilation =
            Compilation::new(gp, &deps, var_of, formulation, &backend, Projection::Off)
                .expect("maps every fact");
        let mut midway = 0;
        for &watch in watches {
            match compilation.run(gp, &backend, watch) {
                Ok(()) => break,
                Err(CompileError::Oversize { .. }) => {
                    midway += usize::from(
                        compilation
                            .fixpoint
                            .worklist
                            .as_ref()
                            .is_some_and(|w| w.midway),
                    )
                }
                Err(e) => panic!("{e}"),
            }
        }
        let mut compiled = compilation.finish(&backend).expect("finishes");
        let (results, _) = query_probabilities(gp, &backend, &mut compiled).expect("answers");
        (
            results
                .expect("no evidence")
                .iter()
                .map(|r| r.probability)
                .collect(),
            compiled.stats.atom_updates,
            midway,
        )
    })
}

#[test]
fn a_stopped_cyclic_component_resumes_where_it_stopped() {
    let mut source: String = EDGES
        .iter()
        .map(|(a, b, q)| format!("{q}::edge({a},{b}).\n"))
        .collect();
    source += "path(X,Y) :- edge(X,Y).\npath(X,Y) :- edge(X,Z), path(Z,Y).\n";
    source += &(2..8)
        .map(|n| format!("query(path(1,{n})).\n"))
        .collect::<String>();
    let gp = ground(
        &parse_program(&source).expect("parses"),
        &GroundOptions::default(),
    )
    .expect("grounds");
    let expected = exact();
    for formulation in [Formulation::Dual, Formulation::Primal] {
        let (_, once, _) = compile(&gp, formulation, &[None]);
        let watches: Vec<Option<usize>> = (2..16).map(|k| Some(1 << k)).chain([None]).collect();
        let (answers, updates, midway) = compile(&gp, formulation, &watches);
        for (a, e) in answers.iter().zip(&expected) {
            assert!(
                (a - e).abs() < 1e-12,
                "{formulation}: {answers:?} against {expected:?}"
            );
        }
        // A stop costs at most the update it stopped; starting the
        // component over would repeat every update before it.
        assert!(midway >= 2, "{formulation}: {midway} stops part way");
        assert!(
            updates <= once + midway,
            "{formulation}: {updates} updates against {once} and {midway} stops"
        );
    }
}

/// A search may spend the work the compile charged since the last one, on
/// the run's work clock. A run that resumes a stopped compile starts a
/// fresh clock, behind the last search's mark; its first search spends
/// what the run's limits allow rather than a difference across clocks.
#[test]
fn a_resumed_runs_first_search_spends_what_the_run_allows() {
    let mut dynamic = Dynamic {
        first_pairs: 0,
        next_at: 0,
        since: Some(400),
        resumed: false,
        debt: 0,
    };
    assert_eq!(dynamic.budget(650, 1, 10), 250);
    dynamic.resumed = true;
    assert_eq!(dynamic.budget(60, 1, 10), u64::MAX);
}

/// Near the end of the compile a search may spend no more than the
/// components left are likely to take: the work so far per finished
/// component times the components left.
#[test]
fn a_search_spends_no_more_than_the_components_left_take() {
    let dynamic = Dynamic {
        first_pairs: 0,
        next_at: 0,
        since: Some(400),
        resumed: false,
        debt: 0,
    };
    assert_eq!(dynamic.budget(1000, 9, 1), 111);
    assert_eq!(dynamic.budget(1000, 1, 9), 600);
    assert_eq!(
        dynamic.budget(1000, 0, 3),
        600,
        "no component finished counts as one"
    );
}

/// A search that spent past its budget holds the next until the compile
/// has spent the excess since it ended, or until the formulas have doubled
/// again past where the next was due; a run that resumes a stopped
/// compile, on a fresh clock, does not wait.
#[test]
fn a_search_past_its_budget_waits_for_the_compile_to_spend_the_excess() {
    let mut dynamic = Dynamic {
        first_pairs: 0,
        next_at: 2000,
        since: Some(400),
        resumed: false,
        debt: 300,
    };
    assert!(!dynamic.due(800, 1999));
    assert!(!dynamic.due(699, 3999));
    assert!(dynamic.due(699, 4000));
    assert!(dynamic.due(700, 2000));
    dynamic.resumed = true;
    assert!(dynamic.due(10, 2000));
}

/// The answers of `gp` compiled on the `closed` layout under the watches
/// in turn and then none, as `auto` resumes a trial round after round,
/// and the compile's clause memo counts, among them the loops the watch
/// refused and the runs kept (`clauses::Held`).
fn compile_retaking(
    gp: &GroundProgram,
    watches: impl Iterator<Item = usize>,
) -> (Vec<f64>, crate::tp::ClauseMemoStats) {
    let deps = Deps::of(gp);
    let (vtree::Layout { vtree, order, .. }, _) =
        vtree::build(gp, &deps, Kind::Closed).expect("builds");
    vtree.context().run(|engine| {
        let backend = TddBackend::new(engine, vtree.clone());
        let var_of = fact_variables(gp, &order);
        let mut compilation = Compilation::new(
            gp,
            &deps,
            var_of,
            Formulation::Dual,
            &backend,
            Projection::default(),
        )
        .expect("maps every fact");
        for watch in watches.map(Some).chain([None]) {
            match compilation.run(gp, &backend, watch) {
                Ok(()) => break,
                Err(CompileError::Oversize { .. }) => {}
                Err(e) => panic!("{e}"),
            }
        }
        let mut compiled = compilation.finish(&backend).expect("finishes");
        let (results, _) = query_probabilities(gp, &backend, &mut compiled).expect("answers");
        (
            results
                .expect("no evidence")
                .iter()
                .map(|r| r.probability)
                .collect(),
            compiled.stats.clause_memo,
        )
    })
}

/// A loop of clauses over facts alone that the watch refused is kept,
/// and the next run, under a larger watch, takes it rather than build it
/// again: on the global scope, where TPC-H q18's one loop over the whole
/// lineage lies, and on a scope narrowed to the bodies' facts, which each
/// run projects anew. The answers are the worlds'.
#[test]
fn a_loop_the_watch_refused_is_taken_by_the_next_run() {
    let ground_as_written = |source: &str| {
        ground(
            &parse_program(source).expect("parses"),
            &GroundOptions {
                views: false,
                ..GroundOptions::default()
            },
        )
        .expect("grounds")
    };
    // q18's shape: a customer's order with a lineitem. Customer 1 has
    // orders 1 and 2, customer 2 order 3.
    let q18 = ground_as_written(
        "0.6::c(1). 0.7::c(2). 0.5::o(1,1). 0.4::o(2,1). 0.3::o(3,2).
         0.2::l(1,1). 0.9::l(1,2). 0.5::l(2,1). 0.6::l(3,1). 0.7::l(3,2).
         q :- c(C), o(O,C), l(O,_).
         query(q).",
    );
    let line = |ps: &[f64]| 1.0 - ps.iter().map(|p| 1.0 - p).product::<f64>();
    let first = 0.6 * line(&[0.5 * line(&[0.2, 0.9]), 0.4 * line(&[0.5])]);
    let second = 0.7 * (0.3 * line(&[0.6, 0.7]));
    let (answers, memo) = compile_retaking(&q18, (0..12).map(|k| 1 << k));
    assert!(
        (answers[0] - line(&[first, second])).abs() < 1e-12,
        "{answers:?}"
    );
    assert!(memo.refused >= 1, "no stop kept the loop");
    assert_eq!(
        memo.retaken, memo.refused,
        "a kept loop the next run built again"
    );
    assert_eq!(memo.built, 1, "the one loop was built {} times", memo.built);
    // `s` is in the most bodies, and its term's disjunction, over the a's
    // and b's, is built on `q`'s scope projected to them; `q`'s own scope
    // is its cone's, under half the facts with the eight r's sixteen
    // beside it, whose formulas are small.
    let a = [0.5, 0.4, 0.3, 0.2];
    let b = [0.7, 0.6, 0.5, 0.9];
    let z: Vec<f64> = (1..=16).map(|k| f64::from(k) / 20.0).collect();
    let mut source = String::from("0.6::s. 0.3::c(1). 0.4::c(2). 0.7::e. 0.2::f.\n");
    for (i, (pa, pb)) in a.iter().zip(&b).enumerate() {
        source += &format!("{pa}::a({i}). {pb}::b({i}).\n");
    }
    for (k, p) in z.iter().enumerate() {
        source += &format!("{p}::z({k}).\n");
    }
    source += "d :- e. d :- f.\nq :- s, a(I), b(I).\nq :- d, c(J).\nquery(q).\n";
    for k in 0..8 {
        source += &format!(
            "r({k}) :- z({}), z({}).\nquery(r({k})).\n",
            2 * k,
            2 * k + 1
        );
    }
    let narrowed = ground_as_written(&source);
    let ab: Vec<f64> = a.iter().zip(&b).map(|(a, b)| a * b).collect();
    let q = line(&[0.6 * line(&ab), line(&[0.7, 0.2]) * line(&[0.3, 0.4])]);
    let expected: Vec<f64> = std::iter::once(q)
        .chain(z.chunks(2).map(|z| z[0] * z[1]))
        .collect();
    let (answers, memo) = compile_retaking(&narrowed, (0..12).map(|k| 1 << k));
    assert_eq!(answers.len(), expected.len());
    for (answer, expected) in answers.iter().zip(&expected) {
        assert!(
            (answer - expected).abs() < 1e-12,
            "{answers:?} against {expected:?}"
        );
    }
    assert!(memo.refused >= 1, "no stop kept the loop");
    assert_eq!(
        memo.retaken, memo.refused,
        "a kept loop the next run built again"
    );
}

/// A run whose turn ends inside a loop of clauses holds the loop before
/// its next body, and the next run goes on with it from there rather than
/// conjoin its bodies again (`clauses::Held`): the runs together spend
/// about the work of one, and answer as the worlds say. `q`'s loop is of
/// bodies that read a derived atom each, a disjunction no factoring
/// shares, and each run's turn ends at the first step it keeps.
#[test]
fn a_loop_at_the_end_of_its_turn_goes_on_in_the_next_run() {
    let n = 12;
    let p = |i: usize, k: usize| 0.05 + 0.9 * ((i * 7 + k * 3) % 17) as f64 / 17.0;
    let mut source = String::new();
    for i in 0..n {
        source += &format!(
            "{}::g({i}). {}::h({i}). {}::f({i}).\nr({i}) :- g({i}).\nr({i}) :- h({i}).\n",
            p(i, 0),
            p(i, 1),
            p(i, 2)
        );
    }
    source += "q :- r(I), f(I).\nquery(q).\n";
    let gp = ground(
        &parse_program(&source).expect("parses"),
        &GroundOptions {
            views: false,
            ..GroundOptions::default()
        },
    )
    .expect("grounds");
    let r = |i: usize| 1.0 - (1.0 - p(i, 0)) * (1.0 - p(i, 1));
    let want = 1.0 - (0..n).map(|i| 1.0 - r(i) * p(i, 2)).product::<f64>();
    let deps = Deps::of(&gp);
    let (vtree::Layout { vtree, order, .. }, _) =
        vtree::build(&gp, &deps, Kind::Closed).expect("builds");
    // The answer, the compile's work and its clause memo counts, in runs
    // whose turns end after `turn` work units, or in one run.
    let compile = |turn: Option<u64>| {
        vtree.context().run(|engine| {
            let backend = TddBackend::new(engine, vtree.clone());
            let var_of = fact_variables(&gp, &order);
            let mut compilation = Compilation::new(
                &gp,
                &deps,
                var_of,
                Formulation::Dual,
                &backend,
                Projection::default(),
            )
            .expect("maps every fact");
            loop {
                if let Some(turn) = turn {
                    backend.end_turn_at(backend.work_units() + turn);
                }
                match compilation.run(&gp, &backend, Some(usize::MAX)) {
                    Ok(()) => break,
                    Err(CompileError::Tdd(tididi::OperationError::Stopped)) if turn.is_some() => {}
                    Err(e) => panic!("{e}"),
                }
            }
            let mut compiled = compilation.finish(&backend).expect("finishes");
            backend.end_turn_at(u64::MAX);
            let (results, _) = query_probabilities(&gp, &backend, &mut compiled).expect("answers");
            (
                results.expect("no evidence")[0].probability,
                compiled.stats.work_units,
                compiled.stats.clause_memo,
            )
        })
    };
    let (once, work_once, _) = compile(None);
    let (paused, work_paused, memo) = compile(Some(1));
    for p in [once, paused] {
        assert!((p - want).abs() < 1e-12, "{p} against {want}");
    }
    assert!(memo.paused >= n / 2, "{} loops paused", memo.paused);
    assert_eq!(
        memo.resumed, memo.paused,
        "a paused loop the next run began again"
    );
    assert!(
        work_paused <= work_once + work_once / 10,
        "{work_paused} units in turns against {work_once} in one run"
    );
}

#[test]
fn a_projected_stop_reads_the_rules_of_the_components_finished() {
    // `big`, the first component, holds eight of the program's ten rules,
    // and the queries `s1` and `s2` read it (one of them may be answered
    // through a template rather than compiled). A horizon that stops the
    // run at its first read stops it once `big` is finished: one component,
    // eight rules, and the projection is the work spent times all the
    // rules over eight, not times the components over one.
    let mut source: String = (1..=8).map(|i| format!("0.5::f{i}.\n")).collect();
    source += "0.5::g1.\n0.5::g2.\n";
    source += &(1..=8)
        .map(|i| format!("big :- f{i}.\n"))
        .collect::<String>();
    source += "s1 :- big, g1.\ns2 :- big, g1, g2.\nquery(s1).\nquery(s2).\n";
    let gp = ground(
        &parse_program(&source).expect("parses"),
        &GroundOptions::default(),
    )
    .expect("grounds");
    let deps = Deps::of(&gp);
    let (vtree::Layout { vtree, order, .. }, _) =
        vtree::build(&gp, &deps, Kind::Closed).expect("builds");
    vtree.context().run(|engine| {
        let backend = TddBackend::new(engine, vtree.clone());
        let var_of = fact_variables(&gp, &order);
        let mut compilation = Compilation::new(
            &gp,
            &deps,
            var_of,
            Formulation::Dual,
            &backend,
            Projection::Off,
        )
        .expect("maps every fact");
        let (_, all) = compilation.progress();
        assert!(all >= 9, "{all}");
        compilation.horizon = Some(Horizon {
            origin: 0,
            from: 0,
            limit: f64::NEG_INFINITY,
            mark: 0,
        });
        let stopped = compilation.run(&gp, &backend, Some(usize::MAX));
        let Err(CompileError::Projected { work }) = stopped else {
            panic!("{stopped:?}");
        };
        let (finished, components) = compilation.components();
        assert!(
            finished == 1 && components >= 2,
            "{finished} of {components}"
        );
        assert_eq!(compilation.progress(), (8, all));
        let spent = backend.work_units() as f64;
        let projected = (spent * all as f64 / 8.0).log2();
        assert!(
            (work - projected).abs() < 1e-9,
            "{work} against {projected}"
        );
    });
}
