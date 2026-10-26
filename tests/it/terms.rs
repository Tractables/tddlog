//! Compound terms, lists and findall/3 (`src/ground/terms.rs`): answers
//! worked out by hand as ProbLog gives them, and the findalls tddlog
//! refuses rather than answer otherwise than ProbLog. The programs whose
//! answers ProbLog computed are under `tests/regress/` (`terms_*.pl`) and
//! `tests/problog/` (`findall*.pl`).
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
        assert!((p - q).abs() < 1e-9, "{a}: {p} against {q}");
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
fn findall_lists_solutions_in_the_order_of_resolution() {
    // Clauses in source order, goals left to right: q's two facts through
    // p's first clause, then p's fact.
    assert_answers(
        "p(X) :- q(X). p(b). q(a). q(c).
         s(L) :- findall(X, p(X), L).
         query(s(_)).",
        &[("s([a, c, b])", 1.0)],
    );
    assert_answers(
        "p(1). p(2). r(a). r(b).
         s(L) :- findall(X-Y, (p(X), r(Y)), L).
         query(s(_)).",
        &[("s([1-a, 1-b, 2-a, 2-b])", 1.0)],
    );
    // An empty goal gives the empty list, a negated goal filters.
    assert_answers(
        "bad(b). e(z) :- fail.
         s(L) :- findall(X, (member(X, [a, b, c]), \\+ bad(X)), L).
         t(L) :- findall(X, e(X), L).
         query(s(_)). query(t(_)).",
        &[("s([a, c])", 1.0), ("t([])", 1.0)],
    );
}

#[test]
fn findall_is_read_by_the_list_predicates() {
    assert_answers(
        "cost(a, 3). cost(b, 4). cost(c, 5).
         total(S) :- findall(C, cost(_, C), L), sum_list(L, S).
         n(N) :- findall(I, cost(I, _), L), length(L, N).
         most(M) :- findall(C, cost(_, C), L), max_list(L, M).
         least(M) :- findall(C, cost(_, C), L), min_list(L, M).
         kinds(K) :- findall(I, cost(I, _), L), sort(L, K).
         query(total(_)). query(n(_)). query(most(_)). query(least(_)). query(kinds(_)).",
        &[
            ("total(12)", 1.0),
            ("n(3)", 1.0),
            ("most(5)", 1.0),
            ("least(3)", 1.0),
            ("kinds([a, b, c])", 1.0),
        ],
    );
    // Shared variables bound before the findall: a list per person.
    assert_answers(
        "person(ann). person(bob). parent(ann, bob). parent(ann, cid).
         c(P, N) :- person(P), findall(C, parent(P, C), L), length(L, N).
         query(c(_, _)).",
        &[("c(ann,2)", 1.0), ("c(bob,0)", 1.0)],
    );
}

#[test]
fn the_guides_example_of_terms_and_lists() {
    // bindings/wasm/guide/index.html, "Terms, lists and findall".
    assert_answers(
        "0.4::item(a). 0.7::item(b). 0.5::item(c).
         cost(a, 3). cost(b, 4). cost(c, 5).
         first_two :- nth1(1, [a, b, c], X), nth1(2, [a, b, c], Y), item(X), item(Y).
         total(S) :- findall(C, cost(_, C), L), sum_list(L, S).
         query(first_two).
         query(total(_)).",
        &[("first_two", 0.4 * 0.7), ("total(12)", 1.0)],
    );
}

#[test]
fn findall_results_feed_probabilistic_rules() {
    // spend(S): the cost of the items other than the one bought.
    assert_answers(
        "cost(a, 3). cost(b, 4).
         0.3::buy(a). 0.6::buy(b).
         spend(S) :- buy(I), findall(C, (cost(J, C), J \\== I), L), sum_list(L, S).
         query(spend(_)).",
        &[("spend(3)", 0.6), ("spend(4)", 0.3)],
    );
}

#[test]
fn list_predicates_over_probabilistic_facts() {
    let (a, b, c) = (0.4, 0.7, 0.5);
    assert_answers(
        "0.4::item(a). 0.7::item(b). 0.5::item(c).
         some :- member(X, [a, b, c]), item(X).
         all([]).
         all([X|Xs]) :- item(X), all(Xs).
         second :- nth1(2, [a, b, c], X), item(X).
         last_one :- last([a, b, c], X), item(X).
         query(some). query(all([a, c])). query(second). query(last_one).",
        &[
            ("some", 1.0 - (1.0 - a) * (1.0 - b) * (1.0 - c)),
            ("all([a, c])", a * c),
            ("second", b),
            ("last_one", c),
        ],
    );
}

#[test]
fn equality_builds_terms_and_is_evaluates_them() {
    // `=` unifies terms: `3 - 1` is a term, which `is` and `=:=` evaluate.
    assert_answers(
        "t(X) :- Y = 3 - 1, X = Y.
         u(X) :- Y = 3 - 1, X is Y * 2.
         v :- Y = 3 - 1, Y =:= 2.
         w :- 3 - 1 = 2.
         query(t(_)). query(u(_)). query(v). query(w).",
        &[("t(3-1)", 1.0), ("u(4)", 1.0), ("v", 1.0), ("w", 0.0)],
    );
    // A head's compound is taken apart under a bound call and built
    // otherwise; a list in a head reads as one.
    assert_answers(
        "len([], 0).
         len([_|T], N) :- len(T, M), N is M + 1.
         pair(f(X, Y)) :- member(X, [1, 2]), Y is X * 10.
         query(len([a, b, c], _)). query(pair(_)).",
        &[
            ("len([a, b, c],3)", 1.0),
            ("pair(f(1,10))", 1.0),
            ("pair(f(2,20))", 1.0),
        ],
    );
}

#[test]
fn division_is_in_floating_point_and_floats_are_not_integers() {
    assert_answers(
        "t(X) :- X is 7 / 2.
         u(X) :- X is 4 / 2.
         v :- X is 4 / 2, X == 2.
         w :- X is 4 / 2, X =:= 2.
         z(X, Y) :- X is -7 // 2, Y is -7 mod 2.
         query(t(_)). query(u(_)). query(v). query(w). query(z(_, _)).",
        &[
            ("t(3.5)", 1.0),
            ("u(2.0)", 1.0),
            ("v", 0.0),
            ("w", 1.0),
            ("z(-4,1)", 1.0),
        ],
    );
}

#[test]
fn floats_are_rounded_and_numbers_sorted_as_problog_does() {
    // ProbLog rounds a float it binds to 15 decimal places; a comparison
    // reads the number computed. Its standard order puts numbers of
    // different values in the order of their text.
    assert_answers(
        "t(X) :- X is 0.1 + 0.2.
         u :- 0.1 + 0.2 =:= 0.3.
         v :- X is 0.1 + 0.2, X =:= 0.3.
         s(S) :- sum_list([0.1, 0.2, 0.3], S).
         o(L) :- sort([3, 20, 100, 2.5, b, a, 20], L).
         w :- 10 @< 2.
         query(t(_)). query(u). query(v). query(s(_)). query(o(_)). query(w).",
        &[
            ("t(0.3)", 1.0),
            ("u", 0.0),
            ("v", 1.0),
            ("s(0.6)", 1.0),
            ("o([100, 2.5, 20, 3, a, b])", 1.0),
            ("w", 1.0),
        ],
    );
}

#[test]
fn a_findall_reads_a_nested_findall() {
    assert_answers(
        "q(L) :- findall((X, Ys), (member(X, [1, 2]), findall(Y, between(1, X, Y), Ys)), L).
         query(q(_)).",
        &[("q([(1, [1]), (2, [1, 2])])", 1.0)],
    );
}

#[test]
fn a_solution_with_more_than_one_proof_is_refused() {
    // ProbLog lists a solution once per proof: `[a, b, a]` for the first.
    for source in [
        "r(a). r(b). r(a). s(L) :- findall(X, r(X), L). query(s(_)).",
        "q(a) :- r1. q(a) :- r2. r1. r2. s(L) :- findall(X, q(X), L). query(s(_)).",
        "s(L) :- findall(X, member(X, [a, b, a]), L). query(s(_)).",
        "q(X) :- e(X, _). e(a, 1). e(a, 2). s(L) :- findall(X, q(X), L). query(s(_)).",
        "q(X) :- e(X), between(1, 3, _). e(a). s(L) :- findall(X, q(X), L). query(s(_)).",
    ] {
        let e = refusal(source);
        assert!(e.contains("more than one proof"), "{source}: {e}");
    }
}

#[test]
fn a_findall_over_a_probabilistic_goal_is_refused() {
    // Its answer is a distribution over lists, as ProbLog's findall2.pl.
    let e = refusal(
        "interested(a1, i1). interested(a2, i1).
         0.1::appears(i1, t1).
         bids(A, I, T) :- appears(I, T), interested(A, I).
         bidding(I, T, As) :- findall(A, bids(A, I, T), As).
         query(bidding(I, T, As)) :- interested(_, I), T = t1.",
    );
    assert!(e.contains("probabilistic fact"), "{e}");
}

#[test]
fn a_findall_tddlog_cannot_list_as_prolog_does_is_refused() {
    for (source, why) in [
        // Through its own clause, as ProbLog's cycle_over_findall.pl.
        (
            "a(1). a(2). a(L) :- findall(X, a(X), L). query(a(_)).",
            "recursive",
        ),
        // A disjunction after another literal of the goal.
        (
            "r(1). r(2). p(1). q(2).
             s(L) :- findall(X, (r(X), (p(X) ; q(X))), L). query(s(_)).",
            "disjunction",
        ),
        // A shared variable that only a later literal binds: ProbLog calls
        // the goal with it unbound.
        (
            "p(1, a). p(2, b). r(a).
             q(L) :- findall(X, p(X, Y), L), r(Y). query(q(_)).",
            "bound after the findall",
        ),
    ] {
        let e = refusal(source);
        assert!(e.contains(why), "{source}: {e}");
    }
}

#[test]
fn a_builtin_of_problogs_that_tddlog_does_not_evaluate_is_refused() {
    // Read as predicates without clauses, they would be false.
    for source in [
        "q. p :- call(q). query(p).",
        "s(X) :- plus(1, 2, X). query(s(_)).",
        "t(L) :- select(a, [a, b], L). query(t(_)).",
        "u :- findall(X, (member(X, [1, 2]), integer(X)), _). query(u).",
        // Asked directly, as ProbLog's append.pl and clause_pred.pl ask.
        "query(append([[1], [2]], [1, 2])).",
        "a :- b(X). query(clause(a, b(Y))).",
    ] {
        let e = refusal(source);
        assert!(e.contains("does not evaluate"), "{source}: {e}");
    }
    // A program's own predicate of that name is its own.
    assert_answers(
        "select(X, [X|T], T). t(L) :- select(a, [a, b], L). query(t(_)).",
        &[("t([b])", 1.0)],
    );
}

#[test]
fn a_grounding_that_builds_ever_deeper_terms_is_refused() {
    let e = refusal("nat(0). nat(s(X)) :- nat(X). query(nat(_)).");
    assert!(e.contains("does not end"), "{e}");
    // Under a bound query the demand takes the term apart, and it ends.
    assert_answers(
        "nat(0). nat(s(X)) :- nat(X). query(nat(s(s(0)))).",
        &[("nat(s(s(0)))", 1.0)],
    );
}
