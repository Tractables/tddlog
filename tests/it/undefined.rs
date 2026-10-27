//! Predicates without clauses (`src/ground/terms.rs`): a query, an
//! observation or a call of one is refused where ProbLog 2.3 refuses it
//! (its `UnknownClause`, "No clauses found"), and where ProbLog answers,
//! tddlog gives its answers. tddlog would otherwise read such a
//! predicate as false, and a misspelt name would give a wrong
//! probability.
use crate::common;

use tddlog::Options;

/// The error `source` is refused with.
fn refusal(source: &str) -> String {
    match common::run(source, &Options::default()) {
        Err(e) => e.to_string(),
        Ok(_) => panic!("not refused: {source}"),
    }
}

fn assert_answers(source: &str, expected: &[(&str, f64)]) {
    let run = common::run(source, &Options::default()).unwrap_or_else(|e| panic!("{source}: {e}"));
    common::assert_answers(
        source,
        &run.results.expect("consistent evidence"),
        expected,
        1e-9,
    );
}

#[test]
fn a_query_or_an_observation_of_a_predicate_without_clauses_is_refused() {
    for (source, missing) in [
        ("0.3::c. query(c). query(d).", "d/0"),
        ("0.3::c. query(\\+d).", "d/0"),
        ("0.3::c. query(p(X)).", "p/1"),
        ("0.3::c. evidence(d, true). query(c).", "d/0"),
        ("0.3::c. evidence(d, false). query(c).", "d/0"),
        ("0.3::c. evidence(\\+d). query(c).", "d/0"),
        // Clauses of another arity do not define it.
        ("0.3::p(1). query(p).", "p/0"),
    ] {
        let e = refusal(source);
        assert!(
            e.contains(&format!("no clauses for {missing}")),
            "{source}: {e}"
        );
    }
}

#[test]
fn a_call_the_queries_or_the_evidence_reach_is_refused() {
    for (source, missing) in [
        ("0.3::c. a :- c, b. query(a).", "b/0"),
        ("0.3::c. a :- c, \\+ b. query(a).", "b/0"),
        ("0.3::p(1). a :- p. query(a).", "p/0"),
        // Through a negation, a findall, the evidence and a query rule.
        ("0.3::c. a :- c, \\+ b. b :- d. query(a).", "d/0"),
        (
            "0.3::c. a :- c, findall(X, b(X), L), length(L, 0). query(a).",
            "b/1",
        ),
        ("0.3::c. a :- c, b. evidence(a, false). query(c).", "b/0"),
        ("0.3::c(1). p(X) :- c(X). query(p(X)) :- q(X).", "q/1"),
    ] {
        let e = refusal(source);
        assert!(
            e.contains(&format!("no clauses for {missing}")),
            "{source}: {e}"
        );
    }
}

#[test]
fn a_builtin_of_problogs_that_tddlog_does_not_evaluate_is_refused() {
    // ProbLog evaluates these, tddlog does not: read as predicates without
    // clauses, they would be false.
    for source in [
        "0.3::c. a :- c, write(hello). query(a).",
        "q. p :- call(q). query(p).",
        "s(X) :- plus(1, 2, X). query(s(_)).",
        "t(L) :- select(a, [a, b], L). query(t(_)).",
        "u :- findall(X, (member(X, [1, 2]), integer(X)), _). query(u).",
        // Asked directly, as ProbLog's append.pl and clause_pred.pl ask.
        "query(append([[1], [2]], [1, 2])).",
        "a :- b(X). query(clause(a, b(Y))).",
    ] {
        let e = refusal(source);
        assert!(e.contains("no clauses for"), "{source}: {e}");
    }
    // A query of a list predicate tddlog evaluates in a body: ProbLog
    // answers it, tddlog refuses it rather than answer 0.
    let e = refusal(":- use_module(library(lists)). query(member(1, [1, 2])).");
    assert!(
        e.contains("no clauses for member/2") && e.contains("in a clause's body only"),
        "{e}"
    );
    // A program's own predicate of that name is its own.
    assert_answers(
        "select(X, [X|T], T). t(L) :- select(a, [a, b], L). query(t(_)).",
        &[("t([b])", 1.0)],
    );
}

#[test]
fn a_call_nothing_asked_reaches_is_answered_as_problog_answers_it() {
    // ProbLog's answers: its resolution never calls `d`.
    assert_answers("0.3::c. b :- d. query(c).", &[("c", 0.3)]);
    assert_answers("0.3::c. 0.4::d. a :- d, b. query(c).", &[("c", 0.3)]);
    // A fact of probability zero is a clause.
    assert_answers("0::b. 0.3::c. a :- c, b. query(a).", &[("a", 0.0)]);
    assert_answers("0.3::x; 0.5::y. a :- y. query(a).", &[("a", 0.5)]);
}
