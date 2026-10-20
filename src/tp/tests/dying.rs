//! A formula is dropped once the last component whose rules read it is
//! finished; the evidence keeps its formula, and so do the queries of a
//! program with evidence. Without evidence a query is counted where its
//! formula is dropped.

use super::*;
use crate::ground::deps::Deps;
use crate::ground::{GroundOptions, ground};
use crate::query::query_probabilities;
use crate::syntax::parse_program;
use crate::vtree::{self, Kind};

/// `a` holds where `x` or `w` does, `c` where `a`, `y` and `z` do, `d`
/// where `y` or `z` does: `P(c | d) = (1 − 0.5·0.8)·0.6·0.7 / (1 − 0.4·0.3)
/// = 0.252 / 0.88`. (`a :- x.` alone would be grounded as a fact.)
const CHAIN: &str = "
    0.5::x. 0.6::y. 0.7::z. 0.2::w.
    a :- x.
    a :- w.
    b :- a, y.
    c :- b, z.
    d :- y.
    d :- z.
    evidence(d).
    query(c).
";

/// `source` ground with its derived atoms as written, each its own
/// component (`GroundOptions::views` off).
fn ground_as_written(source: &str) -> GroundProgram {
    ground(
        &parse_program(source).expect("parses"),
        &GroundOptions {
            views: false,
            ..GroundOptions::default()
        },
    )
    .expect("grounds")
}

fn atom(gp: &GroundProgram, name: &str) -> usize {
    gp.atoms
        .iter()
        .position(|a| a.to_string() == name)
        .unwrap_or_else(|| panic!("no atom {name}"))
}

#[test]
fn formulas_are_dropped_after_their_last_reader() {
    let gp = ground_as_written(CHAIN);
    let (a, b, c, d) = (
        atom(&gp, "a"),
        atom(&gp, "b"),
        atom(&gp, "c"),
        atom(&gp, "d"),
    );
    let deps = Deps::of(&gp);
    for first_pairs in [None, Some(0)] {
        let (vtree::Layout { vtree, order, .. }, _) =
            vtree::build(&gp, &deps, Kind::Dynamic).expect("builds");
        let (formulas, p) = vtree.context().run(|engine| {
            let backend = TddBackend::new(engine, vtree.clone());
            let mut compilation = Compilation::new(
                &gp,
                &deps,
                fact_variables(&gp, &order),
                Formulation::Dual,
                &backend,
                Projection::Off,
            )
            .expect("maps every fact");
            let comp = |x: usize| compilation.comp_of[x];
            assert_eq!(
                compilation.dying.dies_after[a],
                comp(b),
                "a lives until b is compiled"
            );
            assert_eq!(
                compilation.dying.dies_after[b],
                comp(c),
                "b lives until c is compiled"
            );
            assert_eq!(
                (
                    compilation.dying.dies_after[c],
                    compilation.dying.dies_after[d]
                ),
                (u32::MAX, u32::MAX),
                "the query and the evidence are kept"
            );
            if let Some(first) = first_pairs {
                compilation = compilation.dynamic(first);
            }
            compilation.run(&gp, &backend, None).expect("compiles");
            let compiled = compilation.finish(&backend).expect("finishes");
            let formulas: Vec<bool> = [a, b, c, d]
                .iter()
                .map(|&x| compiled.formulas[x].is_some())
                .collect();
            let (results, _) = query_probabilities(&gp, &backend, &compiled).expect("answers");
            (formulas, results.expect("evidence possible")[0].probability)
        });
        assert_eq!(formulas, [false, false, true, true], "{first_pairs:?}");
        assert!((p - 0.252 / 0.88).abs() < 1e-12, "{first_pairs:?}: {p}");
    }
}

/// `CHAIN` without its evidence, `a` queried while `b` still reads it:
/// `P(a) = 1 − 0.5·0.8 = 0.6`, `P(c) = 0.6·0.6·0.7 = 0.252` and
/// `P(\+b) = 1 − 0.6·0.6 = 0.64`.
const NO_EVIDENCE: &str = "
    0.5::x. 0.6::y. 0.7::z. 0.2::w.
    a :- x.
    a :- w.
    b :- a, y.
    c :- b, z.
    query(a).
    query(c).
    query(\\+b).
";

#[test]
fn without_evidence_a_query_is_counted_where_its_formula_is_dropped() {
    let gp = ground_as_written(NO_EVIDENCE);
    let (a, b, c) = (atom(&gp, "a"), atom(&gp, "b"), atom(&gp, "c"));
    let deps = Deps::of(&gp);
    for formulation in [Formulation::Primal, Formulation::Dual] {
        for first_pairs in [None, Some(0)] {
            let (vtree::Layout { vtree, order, .. }, _) =
                vtree::build(&gp, &deps, Kind::Dynamic).expect("builds");
            let (kept, counted, ps) = vtree.context().run(|engine| {
                let backend = TddBackend::new(engine, vtree.clone());
                let mut compilation = Compilation::new(
                    &gp,
                    &deps,
                    fact_variables(&gp, &order),
                    formulation,
                    &backend,
                    Projection::Off,
                )
                .expect("maps every fact");
                assert_eq!(
                    compilation.dying.dies_after[a], compilation.comp_of[b],
                    "a lives until b is compiled"
                );
                if let Some(first) = first_pairs {
                    compilation = compilation.dynamic(first);
                }
                compilation.run(&gp, &backend, None).expect("compiles");
                let compiled = compilation.finish(&backend).expect("finishes");
                let kept: Vec<bool> = [a, b, c]
                    .iter()
                    .map(|&x| compiled.formulas[x].is_some())
                    .collect();
                let counted = compiled.stats.queries_counted;
                let (results, _) = query_probabilities(&gp, &backend, &compiled).expect("answers");
                (
                    kept,
                    counted,
                    results
                        .expect("no evidence")
                        .iter()
                        .map(|r| r.probability)
                        .collect::<Vec<_>>(),
                )
            });
            let at = format!("{formulation}, {first_pairs:?}");
            assert_eq!(kept, [false, false, false], "{at}");
            assert_eq!(counted, 3, "{at}");
            for (p, want) in ps.iter().zip([0.6, 0.252, 0.64]) {
                assert!((p - want).abs() < 1e-12, "{at}: {ps:?}");
            }
        }
    }
}

/// `a` read by `b` and by `c`, which `f` reads, and `a2` by both bodies
/// of `e`: `P(f) = (1 − 0.5·0.8)·(1 − 0.4·0.3) = 0.528` and `P(e) = (1 −
/// 0.5·0.7)·(1 − 0.4·0.3) = 0.572`. (Queried, `b` and `c` would be one
/// formula answered twice, through a template.)
const READ_TWICE: &str = "
    0.5::x. 0.6::y. 0.7::z. 0.2::w. 0.3::v.
    a :- x.
    a :- w.
    b :- a, y.
    c :- a, z.
    f :- b.
    f :- c.
    a2 :- x.
    a2 :- v.
    e :- a2, y.
    e :- a2, z.
    query(f).
    query(e).
";

#[test]
fn the_last_reader_takes_a_formula_in_a_run_without_a_watch() {
    let gp = ground_as_written(READ_TWICE);
    let deps = Deps::of(&gp);
    for formulation in [Formulation::Primal, Formulation::Dual] {
        for first_pairs in [None, Some(0)] {
            for watch in [None, Some(usize::MAX)] {
                let (vtree::Layout { vtree, order, .. }, _) =
                    vtree::build(&gp, &deps, Kind::Dynamic).expect("builds");
                let (taken, ps) = vtree.context().run(|engine| {
                    let backend = TddBackend::new(engine, vtree.clone());
                    let mut compilation = Compilation::new(
                        &gp,
                        &deps,
                        fact_variables(&gp, &order),
                        formulation,
                        &backend,
                        Projection::Off,
                    )
                    .expect("maps every fact");
                    if let Some(first) = first_pairs {
                        compilation = compilation.dynamic(first);
                    }
                    compilation.run(&gp, &backend, watch).expect("compiles");
                    let compiled = compilation.finish(&backend).expect("finishes");
                    let (results, _) =
                        query_probabilities(&gp, &backend, &compiled).expect("answers");
                    (
                        compiled.stats.taken,
                        results
                            .expect("no evidence")
                            .iter()
                            .map(|r| r.probability)
                            .collect::<Vec<_>>(),
                    )
                });
                let at = format!("{formulation}, {first_pairs:?}, {watch:?}");
                // `a` by the later of `b` and `c`, those two by `f`, and
                // `a2` by `e`; the queries are counted where they are
                // dropped, and nothing reads them.
                assert_eq!(taken, if watch.is_none() { 4 } else { 0 }, "{at}");
                for (p, want) in ps.iter().zip([0.528, 0.572]) {
                    assert!((p - want).abs() < 1e-12, "{at}: {ps:?}");
                }
            }
        }
    }
}
