//! Programs that no demand makes safe, resolved top-down
//! (`src/ground/engine/topdown.rs`): queries of terms with variables in
//! them, clauses that answer with a variable, annotated disjunctions whose
//! heads have variables only their callers bind. The answers are worked out
//! by hand as ProbLog gives them; a grammar and a hidden Markov model are
//! under `tests/regress/` (`topdown_*.pl`), with their expected outcomes.
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

/// Asserts that `source` answers exactly the atoms of `expected`, in any
/// order, with their probabilities.
fn assert_answers(source: &str, expected: &[(&str, f64)]) {
    let mut got = answers(source);
    got.sort_by(|a, b| a.0.cmp(&b.0));
    let mut want: Vec<(&str, f64)> = expected.to_vec();
    want.sort_by(|a, b| a.0.cmp(b.0));
    assert_eq!(
        got.iter().map(|(a, _)| a.as_str()).collect::<Vec<_>>(),
        want.iter().map(|(a, _)| *a).collect::<Vec<_>>(),
        "{source}\n{got:?}"
    );
    for ((a, p), (_, q)) in got.iter().zip(&want) {
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
fn a_grammar_parses_words_of_a_given_length() {
    // x -> 0.6 :: c | 0.4 :: x x, its stack of symbols still to expand
    // carried with the word read so far. A word of n letters has as many
    // parse trees as the (n-1)th Catalan number, each of probability
    // 0.6^n 0.4^(n-1).
    assert_answers(
        "derive([], [x]).
         0.6::derive([c|W], S) ; 0.4::derive(W, [x,x|S]) :- derive(W, [x|S]).
         sentence(W) :- rev(W, [], R), derive(R, []).
         rev([], L, L).
         rev([H|T], A, L) :- rev(T, [H|A], L).
         query(sentence([_])).
         query(sentence([_,_])).
         query(sentence([_,_,_])).",
        &[
            ("sentence([c])", 0.6),
            ("sentence([c, c])", 0.6 * 0.6 * 0.4),
            ("sentence([c, c, c])", 2.0 * 0.216 * 0.16),
        ],
    );
}

#[test]
fn a_reversed_list_is_answered_with_its_variables() {
    // seen([X1, X2, X3]) rolls X3 after X2 after X1 = 1, the coin of each
    // roll chosen once per previous side: coin(_, 1) is one choice, so
    // that seen([1, 1, 2]) needs both of its sides and cannot hold.
    assert_answers(
        "1/2::coin(1,D); 1/2::coin(2,D) :- side(D).
         side(1). side(2).
         roll([1]).
         roll([H,F|L]) :- roll([F|L]), coin(H,F).
         seen(L) :- rev(L, [], R), roll(R).
         rev([], L, L).
         rev([H|T], A, L) :- rev(T, [H|A], L).
         query(seen([_,_,_])).",
        &[
            ("seen([1, 1, 1])", 0.5),
            ("seen([1, 1, 2])", 0.0),
            ("seen([1, 2, 1])", 0.25),
            ("seen([1, 2, 2])", 0.25),
        ],
    );
}

#[test]
fn a_choice_is_made_once_for_each_value_its_caller_gives() {
    // bit(T, _) is chosen at each T the walk reaches.
    assert_answers(
        "0.5::bit(T,0); 0.5::bit(T,1).
         step([X|_], T, X) :- bit(T, X).
         step([H|Tail], T, V) :- bit(T, X), X \\= H, TT is T+1, step(Tail, TT, V).
         coin(L, V) :- step(L, 1, V).
         query(coin([1,0], _)).",
        &[("coin([1, 0],1)", 0.5), ("coin([1, 0],0)", 0.25)],
    );
}

#[test]
fn a_query_rule_asks_for_lists_of_each_length() {
    assert_answers(
        "0.5::start(sun); 0.5::start(rain).
         0.6::trans(T,sun,sun); 0.4::trans(T,sun,rain).
         0.2::trans(T,rain,sun); 0.8::trans(T,rain,rain).
         seq([S|L]) :- start(S), more(S, L, 0).
         more(_, [], _).
         more(S, [S2|L], T) :- trans(T, S, S2), T1 is T + 1, more(S2, L, T1).
         query(seq(L)) :- between(1, 2, N), length(L, N).",
        &[
            ("seq([sun])", 0.5),
            ("seq([rain])", 0.5),
            ("seq([sun, sun])", 0.3),
            ("seq([sun, rain])", 0.2),
            ("seq([rain, sun])", 0.1),
            ("seq([rain, rain])", 0.4),
        ],
    );
}

#[test]
fn a_cycle_of_calls_is_completed() {
    // p(a, _) calls itself through b and c. Every path out of a starts
    // with e(a, b), and a is reached again through b or through c.
    assert_answers(
        "0.5::e(a,b). 0.5::e(b,c). 0.5::e(c,a). 0.5::e(b,a).
         p(X,Y) :- e(X,Y).
         p(X,Y) :- e(X,Z), p(Z,Y).
         pair(X, Y, [X, Y]).
         w(L) :- pair(X, Y, L), p(X, Y).
         query(w([a,_])).",
        &[
            ("w([a, a])", 0.5 * (1.0 - 0.5 * (1.0 - 0.25))),
            ("w([a, b])", 0.5),
            ("w([a, c])", 0.25),
        ],
    );
}

#[test]
fn an_answer_with_variables_is_an_atom_of_its_own() {
    // q(_) holds where f does, whatever a caller binds its argument to.
    assert_answers(
        "0.5::f. 0.4::g.
         q(X) :- f.
         q(b) :- g.
         pair(X, Y, [X, Y]).
         w(L) :- pair(a, Y, L), q(Y), Y = b.
         query(w(_)).",
        &[("w([a, b])", 1.0 - 0.5 * 0.6)],
    );
    // ProbLog prints a query's answer with variables by names of its own.
    let e = refusal(
        "0.5::f.
         q(X) :- f.
         query(q(_)).",
    );
    assert!(
        e.contains("a query answered by a term with variables"),
        "{e}"
    );
}

#[test]
fn identities_of_terms_with_variables_are_read_where_no_call_changes_them() {
    // A == B of one variable, and a list of two items \= [], hold however
    // a call binds the variables.
    assert_answers(
        "0.5::f.
         twin(X, [X, X]).
         w(V) :- twin(Y, L), L = [A, B], A == B, L \\= [], f, Y = V.
         query(w(c)).",
        &[("w(c)", 0.5)],
    );
    // A == unknown of a free A fails for the call fill(A) and holds for the
    // call fill(unknown), whose answers share the atom fill(unknown): ProbLog
    // answers q by the first call's proofs, 0.1, which one atom cannot say.
    let e = refusal(
        "0.1::p1.
         p(A) :- p1, A = unknown.
         fill(A) :- A == unknown.
         fill(A) :- p(A), fill(A).
         q :- fill(A).
         query(q).",
    );
    assert!(e.contains("of terms with variables"), "{e}");
}
