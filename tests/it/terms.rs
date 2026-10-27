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
fn a_count_threaded_through_a_compound_grounds_for_the_counts_its_calls_give() {
    // A grammar S -> a (0.5) | S S (0.5) whose k-th expansion of S, in the
    // order of the derivation, chooses with rule(k, _): c(N, NN) passes the
    // count in and out. A demand of s(L, c(0, _)) binds the count only once
    // c(N, NN) is read as two arguments. Each parse tree of a word is a
    // sequence of choices of its own, so the trees are exclusive:
    // P(a) = 0.5, P(aa) = 0.5^3, P(aaa) = 0.5 (P(a) P(aa) + P(aa) P(a)).
    assert_answers(
        "0.5::rule(N, a); 0.5::rule(N, ss).
         s([a], c(N, NN)) :- rule(N, a), NN is N + 1.
         s(L, c(N, NN)) :- rule(N, ss), N1 is N + 1, split(L, L1, L2),
             s(L1, c(N1, N2)), s(L2, c(N2, NN)).
         split([A, B | C], [A], [B | C]).
         split([A, B | C], [A | D], E) :- split([B | C], D, E).
         word(L) :- s(L, c(0, _)).
         query(word([a])). query(word([a, a])). query(word([a, a, a])).",
        &[
            ("word([a])", 0.5),
            ("word([a, a])", 0.125),
            ("word([a, a, a])", 0.0625),
        ],
    );
}

#[test]
fn a_negation_with_variables_of_its_own_holds_where_no_instance_does() {
    // \+ p(X, _) holds where p(X, Y) holds for no Y: for a, neither of
    // p's two facts, 0.5 * 0.6; for b, always.
    assert_answers(
        "0.5::p(a, 1). 0.4::p(a, 2). q(a). q(b).
         r(X) :- q(X), \\+ p(X, _).
         query(r(a)). query(r(b)).",
        &[("r(a)", 0.5 * 0.6), ("r(b)", 1.0)],
    );
}

#[test]
fn a_left_corner_parser_over_its_derivation_so_far() {
    // cplint's plcg.pl (Sato, Kameya and Kurihara 2008) for S -> S S | a |
    // b: every choice is indexed by the derivation so far, and the third
    // clause of lc_call/6 and the second of attach_or_project/3 negate an
    // atom with variables of their own. A parse of n words chooses its n
    // words (0.5 each) and attach or project at each of its 2n - 1 rules
    // (0.5 each), and the parses, Catalan(n - 1) of them, are exclusive:
    // P(w) = Catalan(n - 1) / 2^(3n - 1).
    assert_answers(
        "plc(Ws) :- g_call(['S'], Ws, [], [], _Der).
         g_call([], L, L, Der, Der).
         g_call([G|R], [G|L], L2, Der0, Der) :- terminal(G), g_call(R, L, L2, Der0, Der).
         g_call([G|R], [Wd|L], L2, Der0, Der) :- \\+ terminal(G), first(G, Der0, Wd),
             lc_call(G, Wd, L, L1, [first(G, Wd)|Der0], Der1), g_call(R, L1, L2, Der1, Der).
         lc_call(G, B, L, L1, Der0, Der) :- lc(G, B, Der0, rule(G, [B|RHS2])),
             attach_or_project(G, Der0, attach),
             g_call(RHS2, L, L1, [lc(G, B, rule(G, [B|RHS2])), attach|Der0], Der).
         lc_call(G, B, L, L2, Der0, Der) :- lc(G, B, Der0, rule(A, [B|RHS2])),
             attach_or_project(G, Der0, project),
             g_call(RHS2, L, L1, [lc(G, B, rule(A, [B|RHS2])), project|Der0], Der1),
             lc_call(G, A, L1, L2, Der1, Der).
         lc_call(G, B, L, L2, Der0, Der) :- \\+ lc(G, B, Der0, rule(G, [B|_])),
             lc(G, B, Der0, rule(A, [B|RHS2])),
             g_call(RHS2, L, L1, [lc(G, B, rule(A, [B|RHS2]))|Der0], Der1),
             lc_call(G, A, L1, L2, Der1, Der).
         attach_or_project(A, Der, Op) :- lc(A, A, Der, _), attach(A, Der, Op).
         attach_or_project(A, Der, attach) :- \\+ lc(A, A, Der, _).
         lc('S', 'S', _Der, rule('S', ['S', 'S'])).
         lc('S', a, _Der, rule('S', [a])).
         lc('S', b, _Der, rule('S', [b])).
         0.5::first('S', Der, a); 0.5::first('S', Der, b).
         0.5::attach('S', Der, attach); 0.5::attach('S', Der, project).
         terminal(a). terminal(b).
         query(plc([a])). query(plc([a, b])). query(plc([b, a, a])). query(plc([a, b, b, a])).",
        &[
            ("plc([a])", 0.25),
            ("plc([a, b])", 1.0 / 32.0),
            ("plc([b, a, a])", 2.0 / 256.0),
            ("plc([a, b, b, a])", 5.0 / 2048.0),
        ],
    );
}

#[test]
fn a_formula_of_symbols_read_from_lists_and_evaluated() {
    // DeepProbLog's handwritten formulas (HWF), its parse/2 written for
    // each of its two modes, the result bound (parse) and free (value),
    // in place of var/1 and ground/1; the classifiers' distributions are
    // annotated disjunctions. Each sequence of symbols has one parse:
    // 2+3 = 5; 2*3 and 3+3 = 6; 2*0, 3*0 and 3-3 = 0; 2-3 = -1;
    // 2/3 within 1e-4 of 0.6667; 2/0 and 3/0 have none.
    let (n2, n3) = (0.6, 0.4);
    let (plus, minus, times, div) = (0.4, 0.1, 0.3, 0.2);
    let (m3, m0) = (0.7, 0.3);
    assert_answers(
        "0.6::detect_number(i1, 2); 0.4::detect_number(i1, 3).
         0.4::detect_operator(i2, +); 0.1::detect_operator(i2, -);
             0.3::detect_operator(i2, *); 0.2::detect_operator(i2, /).
         0.7::detect_number(i3, 3); 0.3::detect_number(i3, 0).
         detect_all([N], [N2]) :- detect_number(N, N2).
         detect_all([N, O|T], [N2, O2|T2]) :- detect_number(N, N2), detect_operator(O, O2),
             detect_all(T, T2).
         almost_equal(X, Y) :- abs(X - Y) < 0.0001.
         expression(Images, Result) :- detect_all(Images, Symbols), parse(Symbols, Result).
         parse([N], R) :- almost_equal(N, R).
         parse([N1, +|T], R) :- value(T, R2), almost_equal(N1 + R2, R).
         parse([N1, -|T], R) :- value([-1, *|T], R2), almost_equal(N1 + R2, R).
         parse([N1, *, N2|T], R) :- N3 is N1 * N2, parse([N3|T], R).
         parse([N1, /, N2|T], R) :- N2 \\== 0, N3 is N1 / N2, parse([N3|T], R).
         value([N], R) :- R is float(N).
         value([N1, +|T], R) :- value(T, R2), R is float(N1 + R2).
         value([N1, -|T], R) :- value([-1, *|T], R2), R is float(N1 + R2).
         value([N1, *, N2|T], R) :- N3 is N1 * N2, value([N3|T], R).
         value([N1, /, N2|T], R) :- N2 \\== 0, N3 is N1 / N2, value([N3|T], R).
         query(expression([i1, i2, i3], 5)). query(expression([i1, i2, i3], 6)).
         query(expression([i1, i2, i3], 0)). query(expression([i1, i2, i3], -1)).
         query(expression([i1, i2, i3], 0.6667)).",
        &[
            ("expression([i1, i2, i3],5)", n2 * plus * m3),
            (
                "expression([i1, i2, i3],6)",
                n2 * times * m3 + n3 * plus * m3,
            ),
            (
                "expression([i1, i2, i3],0)",
                (n2 + n3) * times * m0 + n3 * minus * m3,
            ),
            ("expression([i1, i2, i3],-1)", n2 * minus * m3),
            ("expression([i1, i2, i3],0.6667)", n2 * div * m3),
        ],
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
fn a_grounding_that_builds_ever_deeper_terms_is_refused() {
    let e = refusal("nat(0). nat(s(X)) :- nat(X). query(nat(_)).");
    assert!(e.contains("does not end"), "{e}");
    // Under a bound query the demand takes the term apart, and it ends.
    assert_answers(
        "nat(0). nat(s(X)) :- nat(X). query(nat(s(s(0)))).",
        &[("nat(s(s(0)))", 1.0)],
    );
}

#[test]
fn a_variable_the_query_binds_is_bound_and_one_nothing_binds_is_unsafe() {
    // ProbLog proves `p(1)` from the query down: its `X` is the query's 1.
    assert_answers("p(X) :- q(Y). q(1). query(p(1)).", &[("p(1)", 1.0)]);
    let e = refusal("p :- X > 1. query(p).");
    assert!(e.starts_with("grounding error: unsafe rule"), "{e}");
}
