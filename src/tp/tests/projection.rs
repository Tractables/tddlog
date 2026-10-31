//! Projected vtrees: one vtree per component's ancestor cone.
//!
//! The shape the feature exists for is naive Bayes over many conditionally
//! independent observations — the dice digit-recognition program, 7840 facts
//! and no component whose cone is larger than 10 — where a structural TDD
//! spends nearly all of every formula on levels the function cannot depend on
//! (`docs/design.md`, "Compilation"). These tests check that projection
//! does not move an answer, against probabilities computed by hand, and that
//! it does shrink the formulas on that shape.

use std::sync::Arc;

use crate::tp::Formulation;
use crate::{Kind, Options, Projection, Strategy};
use tididi::vtree::VarId;

/// The options of a run under `strategy`, compiled under `projection`.
fn options(strategy: Strategy, projection: Projection) -> Options {
    Options {
        strategy,
        projection: Some(projection),
        ..Options::default()
    }
}

const PROJECTIONS: [Projection; 4] = [
    Projection::Off,
    Projection::Cones { max_share: 1.0 },
    Projection::Cones { max_share: 0.5 },
    Projection::Cones { max_share: 0.0 },
];

fn answers(source: &str, projection: Projection) -> Vec<(String, f64)> {
    let run = crate::solve::run_source(source, &options(Strategy::Trial(Kind::Closed), projection))
        .unwrap_or_else(|e| panic!("{projection}: {e}"));
    run.results
        .expect("evidence is consistent")
        .into_iter()
        .map(|r| {
            let sign = if r.positive { "" } else { "\\+" };
            (format!("{sign}{}", r.atom), r.probability)
        })
        .collect()
}

/// A class variable with two values and one observation conditioned on it,
/// with a second observation as evidence. Hand computed:
///
/// - `P(o1) = 0.5 * 0.8 + 0.5 * 0.3 = 0.55`
/// - `P(o1 ∧ o2) = 0.5 * 0.8 * 0.7 + 0.5 * 0.3 * 0.2 = 0.31`
/// - `P(o2 | o1) = 0.31 / 0.55 = 0.5636363...`
/// - `P(cls(a) | o1) = 0.4 / 0.55 = 0.7272727...`
///
/// ProbLog 2.2 on the same program prints `o2: 0.56363636` and
/// `cls(a): 0.72727273`.
#[test]
fn naive_bayes_with_evidence_is_the_hand_computed_answer() {
    let source = "\
0.5::cls(a); 0.5::cls(b).
0.8::o1 :- cls(a).
0.3::o1 :- cls(b).
0.7::o2 :- cls(a).
0.2::o2 :- cls(b).
evidence(o1, true).
query(o2).
query(cls(a)).
";
    for projection in PROJECTIONS {
        let got = answers(source, projection);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].0, "o2");
        assert!(
            (got[0].1 - 0.31 / 0.55).abs() < 1e-9,
            "{projection}: P(o2|o1) = {}",
            got[0].1
        );
        assert_eq!(got[1].0, "cls(a)");
        assert!(
            (got[1].1 - 0.4 / 0.55).abs() < 1e-9,
            "{projection}: P(cls(a)|o1) = {}",
            got[1].1
        );
    }
}

/// The digit-recognition shape in miniature: one class variable, `k`
/// observations conditioned on it, half of them observed, and padding facts
/// that nothing else touches so that the fact universe is much larger than
/// any one formula's cone.
fn naive_bayes(k: usize, pad: usize) -> String {
    let mut s = String::from("0.5::cls(v1); 0.5::cls(v2).\n");
    for i in 1..=k {
        let p = 0.1 + 0.8 * (i as f64 / k as f64);
        let q = 0.9 - 0.8 * (i as f64 / k as f64);
        s += &format!("{p:.4}::obs({i}) :- cls(v1).\n{q:.4}::obs({i}) :- cls(v2).\n");
    }
    for j in 1..=pad {
        s += &format!("0.25::pad({j}).\nquery(pad({j})).\n");
    }
    for i in 1..=k / 2 {
        s += &format!("evidence(obs({i}), true).\n");
    }
    for i in k / 2 + 1..=k {
        s += &format!("query(obs({i})).\n");
    }
    s
}

/// ProbLog 2.2's answers for `naive_bayes(12, 30)`, the program below: the
/// six unobserved features, given the six observed ones. The thirty padding
/// facts are independent of the evidence and keep their own 0.25. Produced by
/// running ProbLog, which these tests do not require.
const PROBLOG_NAIVE_BAYES: [(&str, f64); 6] = [
    ("obs(7)", 0.434_823_18),
    ("obs(8)", 0.369_744_08),
    ("obs(9)", 0.304_567_27),
    ("obs(10)", 0.239_390_45),
    ("obs(11)", 0.174_311_35),
    ("obs(12)", 0.109_134_53),
];

#[test]
fn projection_is_problogs_answer_on_the_naive_bayes_shape() {
    let source = naive_bayes(12, 30);
    for projection in PROJECTIONS {
        let got = answers(&source, projection);
        assert_eq!(
            got.len(),
            36,
            "{projection}: thirty padding facts and six unobserved features"
        );
        let find = |name: &str| {
            got.iter()
                .find(|(a, _)| a == name)
                .unwrap_or_else(|| panic!("{projection}: no query {name}"))
                .1
        };
        for (name, want) in PROBLOG_NAIVE_BAYES {
            let p = find(name);
            assert!(
                (p - want).abs() < 1e-7,
                "{projection}: {name} = {p} but ProbLog says {want}"
            );
        }
        for j in 1..=30 {
            let name = format!("pad({j})");
            let p = find(&name);
            assert!(
                (p - 0.25).abs() < 1e-9,
                "{projection}: {name} = {p}, and padding is independent of the evidence"
            );
        }
    }
}

/// A hundred observations on a right-linear vtree, each at a node of its own
/// one above the other: the evidence phase puts the small factors off and
/// conjoins them into the growing conjunction in batches (`PUT_OFF_BELOW` in
/// `tp`). Hand computed from the program's own rounded weights:
/// `P(v1 | e) ∝ 0.5 Π p_i` and `P(v2 | e) ∝ 0.5 Π q_i` over the observed i,
/// and `P(obs(j) | e) = P(v1 | e) p_j + P(v2 | e) q_j`.
#[test]
fn many_observations_are_the_hand_computed_answer() {
    let k = 200;
    let source = naive_bayes(k, 0);
    let weight = |x: f64| format!("{x:.4}").parse::<f64>().unwrap();
    let p = |i: usize| weight(0.1 + 0.8 * (i as f64 / k as f64));
    let q = |i: usize| weight(0.9 - 0.8 * (i as f64 / k as f64));
    let (v1, v2) = (1..=k / 2).fold((0.5, 0.5), |(a, b), i| (a * p(i), b * q(i)));
    let (v1, v2) = (v1 / (v1 + v2), v2 / (v1 + v2));
    for projection in PROJECTIONS {
        let got = answers(&source, projection);
        assert_eq!(got.len(), k / 2, "{projection}: the unobserved features");
        for (j, (name, got)) in (k / 2 + 1..=k).zip(got) {
            assert_eq!(name, format!("obs({j})"), "{projection}: query order");
            let want = v1 * p(j) + v2 * q(j);
            assert!(
                (got - want).abs() < 1e-9,
                "{projection}: {name} = {got} but {want} by hand"
            );
        }
    }
}

/// Sizes in nodes and pairs are machine independent, so this is a size
/// assertion and not a timing one. The padding facts give every component a
/// cone far smaller than the fact set, which is exactly what projection
/// removes. The program is compiled under the dual on a right-linear vtree
/// over its facts in program order, and the final formulas are counted.
#[test]
fn projection_shrinks_the_formulas_it_is_for() {
    let source = naive_bayes(12, 30);
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(&source).unwrap(),
        &Default::default(),
    )
    .unwrap();
    let order = crate::vtree::facts_in_order(&gp);
    let vars: Vec<VarId> = (1..=order.len() as u32).map(VarId).collect();
    let vtree = Arc::new(tididi::Vtree::linear_from_order(&vars).unwrap());
    let total = |projection| {
        vtree.context().run(|engine| {
            let backend = crate::tp::TddBackend::new(engine, vtree.clone());
            let var_of = crate::tp::fact_variables(&gp, &order);
            let compiled =
                crate::tp::compile(&gp, var_of, Formulation::Dual, &backend, projection, None)
                    .unwrap();
            let nodes: usize = compiled
                .formulas
                .iter()
                .flatten()
                .map(|f| f.node_count())
                .sum();
            let largest = compiled
                .formulas
                .iter()
                .flatten()
                .map(|f| f.node_count())
                .max()
                .unwrap_or(0);
            (nodes, largest)
        })
    };
    let (flat_nodes, flat_largest) = total(Projection::Off);
    let (cone_nodes, cone_largest) = total(Projection::Cones { max_share: 1.0 });
    assert!(
        cone_nodes * 3 < flat_nodes,
        "projected store {cone_nodes} nodes against {flat_nodes} on one vtree, which is not the reduction the cones promise",
    );
    assert!(
        cone_largest <= flat_largest,
        "no formula grows: {cone_largest} against {flat_largest}"
    );
    // And the fallback really does turn it off again.
    let (off_nodes, _) = total(Projection::Cones { max_share: 0.0 });
    assert_eq!(
        off_nodes, flat_nodes,
        "every component over the threshold keeps the global vtree"
    );
}

/// A recursive component: every atom of a cycle shares one cone, so the
/// fixpoint's equivalence tests never leave their vtree. ProbLog 2.2 on the
/// same program prints `path(1,4): 0.174`, `path(1,3): 0.58` and
/// `elsewhere: 0.25`.
#[test]
fn transitive_closure_survives_projection() {
    let source = "\
0.6::edge(1,2). 0.5::edge(2,3). 0.4::edge(1,3). 0.3::edge(3,4).
path(X,Y) :- edge(X,Y).
path(X,Y) :- edge(X,Z), path(Z,Y).
0.25::elsewhere.
query(path(1,4)).
query(path(1,3)).
query(elsewhere).
";
    for projection in PROJECTIONS {
        let got = answers(source, projection);
        let want = [
            ("path(1,4)", 0.174),
            ("path(1,3)", 0.58),
            ("elsewhere", 0.25),
        ];
        assert_eq!(got.len(), want.len());
        for ((atom, p), (name, expected)) in got.iter().zip(want) {
            assert_eq!(atom, name, "{projection}: query order");
            assert!(
                (p - expected).abs() < 1e-9,
                "{projection}: {name} = {p} but ProbLog says {expected}"
            );
        }
    }
}

/// A program with no probabilistic fact at all. There is no variable to
/// project onto, every cone is empty, and every component keeps the vtree the
/// strategy built. The ProbLog system-test suite has such programs and they
/// panicked the first version of this: an empty cone was seeded with the
/// first variable, and there was no first variable.
#[test]
fn a_program_without_probabilistic_facts_survives_projection() {
    let source = "\
b.
a :- b.
query(a).
query(b).
";
    for projection in PROJECTIONS {
        let got = answers(source, projection);
        assert_eq!(got.len(), 2, "{projection}: two queries");
        for (atom, p) in &got {
            assert!(
                (p - 1.0).abs() < 1e-9,
                "{projection}: {atom} = {p}, and a certain atom has probability one"
            );
        }
    }
}

/// Two disjoint sub-programs `left_sub` and `right_sub` over private
/// probabilistic facts feed into `top :- left_sub, \+right_sub.`. Under cone
/// projection each sub-formula collapses to a single reweighed variable once
/// its internal dependencies die, shrinking `top`'s support to two variables
/// while keeping the exact hand-computed probabilities.
#[test]
fn private_closed_subcones_collapse_to_one_literal() {
    let source = "\
0.6::l1. 0.7::l2. 0.8::l3.
left_sub :- l1, l2.
left_sub :- l2, l3.
0.5::r1. 0.4::r2. 0.3::r3.
right_sub :- r1, r2.
right_sub :- \\+r2, r3.
top :- left_sub, \\+right_sub.
query(top).
query(left_sub).
query(right_sub).
";
    // P(left_sub) = P(l2) * (P(l1) + (1 - P(l1)) * P(l3))
    //             = 0.7 * (0.6 + 0.4 * 0.8) = 0.7 * 0.92 = 0.644
    // P(right_sub) = P(r1)*P(r2) + (1 - P(r2))*P(r3)
    //              = 0.5 * 0.4 + 0.6 * 0.3 = 0.20 + 0.18 = 0.38
    // P(top) = P(left_sub) * (1 - P(right_sub)) = 0.644 * 0.62 = 0.39928
    let want = [
        ("top", 0.644 * 0.62),
        ("left_sub", 0.644),
        ("right_sub", 0.38),
    ];
    for projection in PROJECTIONS {
        let got = answers(source, projection);
        assert_eq!(got.len(), want.len());
        for ((atom, p), (name, expected)) in got.iter().zip(want) {
            assert_eq!(atom, name, "{projection}: query order");
            assert!(
                (p - expected).abs() < 1e-9,
                "{projection}: {atom} = {p} but expected {expected}"
            );
        }
    }
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(source).unwrap(),
        &Default::default(),
    )
    .unwrap();
    let order = crate::vtree::facts_in_order(&gp);
    let vars: Vec<VarId> = (1..=order.len() as u32).map(VarId).collect();
    let vtree = Arc::new(tididi::Vtree::linear_from_order(&vars).unwrap());
    for formulation in [Formulation::Primal, Formulation::Dual] {
        for projection in [Projection::default(), Projection::Cones { max_share: 1.0 }] {
            vtree.context().run(|engine| {
                let backend = crate::tp::TddBackend::new(engine, vtree.clone());
                let var_of = crate::tp::fact_variables(&gp, &order);
                let compiled =
                    crate::tp::compile(&gp, var_of, formulation, &backend, projection, None)
                        .unwrap();
                assert_eq!(
                    compiled.for_queries.reweighed.len(),
                    2,
                    "{formulation:?} under {projection}: both closed sub-programs collapsed"
                );
            });
        }
    }
}
