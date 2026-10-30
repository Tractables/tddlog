//! Evidence rules, `evidence(p(X)) :- body.`: every instance of the pattern
//! under a grounding of the body is observed, as the evidence directives of
//! those instances would observe it. Answers worked out by hand.
use crate::common;

use tddlog::Options;

/// The answers of `source`, each atom as printed with its probability.
fn answers(source: &str) -> Vec<(String, f64)> {
    let run = common::run(source, &Options::default()).unwrap();
    run.results
        .expect("consistent evidence")
        .iter()
        .map(|r| (r.to_string(), r.probability))
        .collect()
}

fn assert_answers(source: &str, expected: &[(&str, f64)]) {
    let got = answers(source);
    assert_eq!(
        got.iter().map(|(a, _)| a.as_str()).collect::<Vec<_>>(),
        expected.iter().map(|(a, _)| *a).collect::<Vec<_>>(),
        "{source}"
    );
    for ((a, p), (_, q)) in got.iter().zip(expected) {
        assert!((p - q).abs() < 1e-12, "{a}: {p} against {q}");
    }
}

const PROGRAM: &str = "0.5::c(1). 0.5::c(2). 0.4::d. p(X) :- c(X). p(X) :- d, obs(X).
     obs(1). obs(2). query(d). query(c(1)).";

#[test]
fn an_evidence_rule_observes_each_instance_its_body_finds() {
    // p(1) and p(2) observed: e = (c1 | d)(c2 | d), P(e) = 0.4 + 0.6 * 0.25
    // = 0.55; P(d, e) = 0.4 and P(c1, e) = 0.5 * (1 - 0.5 * 0.6) = 0.35.
    let expected = [("d", 0.4 / 0.55), ("c(1)", 0.35 / 0.55)];
    assert_answers(&format!("{PROGRAM} evidence(p(X)) :- obs(X)."), &expected);
    assert_answers(
        &format!("{PROGRAM} evidence(p(1)). evidence(p(2), true)."),
        &expected,
    );
    // c(2) observed false as well: e = (c1 | d) d !c2, so d holds and c(1)
    // keeps its prior.
    let expected = [("d", 1.0), ("c(1)", 0.5)];
    assert_answers(
        &format!("{PROGRAM} evidence(p(X)) :- obs(X). evidence(c(X), false) :- obs(X), X > 1."),
        &expected,
    );
    assert_answers(
        &format!("{PROGRAM} evidence(p(X)) :- obs(X). evidence(\\+c(X)) :- obs(X), X > 1."),
        &expected,
    );
}
