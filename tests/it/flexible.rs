//! Flexible probabilities, a head's probability that its clause's body
//! computes (`P::h(X) :- w(X, P).`): answers worked out by hand as ProbLog
//! gives them, each ground instance of the clause a choice of its own with
//! the probability its body binds, and the probabilities tddlog refuses.
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

/// The error `source` is refused with.
fn refusal(source: &str) -> String {
    match common::run(source, &Options::default()) {
        Err(e) => e.to_string(),
        Ok(_) => panic!("not refused: {source}"),
    }
}

#[test]
fn a_probability_bound_by_the_body() {
    assert_answers(
        "w(a, 0.3). w(b, 0.6).
         P::h(X) :- w(X, P).
         query(h(a)). query(h(b)).",
        &[("h(a)", 0.3), ("h(b)", 0.6)],
    );
}

#[test]
fn a_probability_computed_in_the_body_or_in_the_head() {
    // ProbLog's tutorial packs an item with probability one over its
    // weight.
    assert_answers(
        "weight(skis, 6). weight(boots, 4).
         P::pack(I) :- weight(I, W), P is 1.0 / W.
         1 / W::carry(I) :- weight(I, W).
         query(pack(skis)). query(pack(boots)). query(carry(skis)). query(carry(boots)).",
        &[
            ("pack(skis)", 1.0 / 6.0),
            ("pack(boots)", 0.25),
            ("carry(skis)", 1.0 / 6.0),
            ("carry(boots)", 0.25),
        ],
    );
}

#[test]
fn an_annotated_disjunction_of_flexible_probabilities() {
    // emit(T, 1) with T/4 and emit(T, 0) with the rest, for T in 1..3; and
    // two heads that leave 0.5 to none.
    assert_answers(
        "P1::emit(T, 1); P2::emit(T, 0) :- between(1, 3, T), P1 is T / 4, P2 is 1 - P1.
         v(0.2, 0.3).
         P::a; Q::b :- v(P, Q).
         query(emit(1, 1)). query(emit(2, 0)). query(emit(3, 0)). query(a). query(b).",
        &[
            ("emit(1,1)", 0.25),
            ("emit(2,0)", 0.5),
            ("emit(3,0)", 0.25),
            ("a", 0.2),
            ("b", 0.3),
        ],
    );
}

#[test]
fn each_instance_of_the_clause_is_a_choice_of_its_own() {
    // Two bodies for h(a), of probabilities 0.5 and 0.2, are independent
    // choices: h(a) fails with 0.5 · 0.8. A probability bound by a
    // probabilistic atom picks the instance with it: e is 0.5 · 0.2 +
    // 0.5 · 0.8.
    assert_answers(
        "p(a, 0.5). p(a, 0.2).
         P::h(X) :- p(X, P).
         0.5::c(0.2); 0.5::c(0.8).
         P::e :- c(P).
         query(h(a)). query(e).",
        &[("h(a)", 1.0 - 0.5 * 0.8), ("e", 0.5)],
    );
}

#[test]
fn a_probability_counted_with_findall() {
    // Vennekens et al.'s person drawn at random, from cplint's examples:
    // two males and three females.
    assert_answers(
        "male(john). male(david).
         female(anna). female(elen). female(cathy).
         M / P::male; F / P::female :-
             findall(X, male(X), LM), findall(Y, female(Y), LF),
             length(LM, M), length(LF, F), P is F + M.
         query(male). query(female).",
        &[("male", 0.4), ("female", 0.6)],
    );
}

#[test]
fn a_probability_the_call_binds() {
    // cplint's flexprob example: red(P) holds with probability P, drawn
    // from three red balls and one green.
    assert_answers(
        "P::red(P).
         draw_red(R, G) :- P is R / (R + G), red(P).
         query(draw_red(3, 1)).",
        &[("draw_red(3,1)", 0.75)],
    );
}

#[test]
fn a_probability_out_of_range_or_a_disjunction_over_one_is_refused() {
    for (source, error) in [
        ("w(2). P::h :- w(P). query(h).", "probability 2"),
        ("w(-0.5). P::h :- w(P). query(h).", "probability -0.5"),
        ("w(a). P::h :- w(P). query(h).", "not a number"),
        ("w(0.5). 0.7::a; P::b :- w(P). query(a).", "sum to 1.2 > 1"),
    ] {
        let e = refusal(source);
        assert!(e.contains(error), "{source}: {e}");
    }
}
