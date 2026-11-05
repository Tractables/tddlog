//! `subquery/2,3`, an inference of its own inside the program: answers
//! worked out by hand as ProbLog gives them, and the calls tddlog refuses.
use tddlog::Options;

/// The answers of `source`, each atom as printed with its probability.
fn answers(source: &str) -> Vec<(String, f64)> {
    tddlog::solve(source, &Options::default(), &mut |_| {})
        .unwrap()
        .answers
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
    match tddlog::solve(source, &Options::default(), &mut |_| {}) {
        Err(e) => e.to_string(),
        Ok(_) => panic!("not refused: {source}"),
    }
}

#[test]
fn a_conditional_probability_as_a_certain_answer() {
    // P(b) = 0.3 * 0.6 + 0.7 * 0.2 = 0.32, P(a | b) = 0.18 / 0.32.
    assert_answers(
        "0.3::a. 0.6::b :- a. 0.2::b :- \\+a.
         c(P) :- subquery(a, P, [b]).
         d(P) :- subquery(b, P).
         e(P) :- subquery(a, P, [\\+b]).
         query(c(_)). query(d(_)). query(e(_)).",
        &[
            ("c(0.5625)", 1.0),
            ("d(0.32)", 1.0),
            ("e(0.176470588235294)", 1.0),
        ],
    );
}

#[test]
fn each_instance_of_the_goal_an_answer() {
    assert_answers(
        "0.2::p(1). 0.7::p(2).
         q(X, P) :- subquery(p(X), P).
         query(q(_, _)).",
        &[("q(1,0.2)", 1.0), ("q(2,0.7)", 1.0)],
    );
}

#[test]
fn a_probability_drawn_from_a_nested_inference() {
    // Two players draw 0, 1 or 2 and win on a sum of 3. The second sees
    // the first's number and believes a sum over 1 wins: given that, it
    // draws 2 after a 0, 1 or 2 after a 1, and any after a 2. The pair
    // (A, B) is played with that probability of B, so a win needs A = 1
    // (B = 2, 1/2) or A = 2 (B = 1, 1/3): P(win) = (1/2 + 1/3) / 3, and
    // P(first = 1 | win) = 3/5.
    assert_answers(
        "1/3::first(0); 1/3::first(1); 1/3::first(2).
         1/3::second(0); 1/3::second(1); 1/3::second(2).
         believed(A) :- second(C), A + C > 1.
         played(A, B) :- first(A), subquery(second(B), P, [believed(A)]), draw(A, B, P).
         P::draw(_, _, P).
         won :- played(A, B), A + B =:= 3.
         query(first(_)). evidence(won).",
        &[("first(0)", 0.0), ("first(1)", 0.6), ("first(2)", 0.4)],
    );
}

#[test]
fn a_call_that_another_calls_answers_bind() {
    // P(x | y) = 1, so the second call is made, and P(y) = 0.2.
    assert_answers(
        "0.5::x. 0.4::y :- x.
         s(P) :- subquery(x, P, [y]).
         t(Q) :- s(P), P > 0.5, subquery(y, Q).
         query(t(_)).",
        &[("t(0.2)", 1.0)],
    );
}

#[test]
fn calls_it_does_not_read() {
    for (source, what) in [
        ("0.3::a. b :- subquery(a, 0.5). query(b).", "bound before"),
        ("0.3::a. b :- \\+ subquery(a, _). query(b).", "negation"),
        (
            "0.3::a. b(P) :- subquery(a, P, [], x). query(b(_)).",
            "semiring",
        ),
        (
            "0.3::a. b(P) :- subquery(a, P, [E]). query(b(_)).",
            "evidence has a variable",
        ),
        (
            "0.3::a. b(P) :- subquery(a, P, [a, \\+a]). query(b(_)).",
            "probability zero",
        ),
        (
            "0.3::a. b(P) :- subquery((a, a), P). query(b(_)).",
            "not an atom",
        ),
    ] {
        let e = refusal(source);
        assert!(e.contains(what), "{source}: {e}");
    }
}
