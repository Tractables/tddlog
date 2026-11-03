//! A program that asks one query and observes nothing is answered by the
//! probability of the query observed (`GroundProgram::observed_query`)
//! where observing it fixes or merges a fact: `P(q) = P(e)` for the
//! evidence `e` that observes `q`, so that the conditioning on the evidence
//! propagates the query's value. Its answers against enumeration of the
//! worlds or a hand count, under every layout, through `solve` and
//! `solve_ground`; a program with evidence, or with two queries, or whose
//! query's value fixes no fact, keeps its queries.

use crate::common;

use tddlog::{Kind, Options, Projection, Strategy};

/// The layouts each program is run under, each with its own projection.
fn strategies() -> Vec<(Strategy, Projection)> {
    use Kind::*;
    let cones = Projection::default();
    vec![
        (Strategy::Auto, cones),
        (Strategy::Trial(Closed), cones),
        (Strategy::Trial(Closed), Projection::Off),
        (Strategy::Trial(Td), cones),
        (Strategy::Trial(Families { search: false }), cones),
        (Strategy::Trial(Program { fan_in: false }), cones),
        (Strategy::Trial(Dynamic), Projection::Off),
    ]
}

/// `source`'s answers under every layout, through `solve` and through
/// `solve_ground`, each checked against `expected` within `tolerance`
/// relative to it; `families`, where the program reads no atom through an
/// indicator, refuses and is left out. The runs `solve_ground` made.
fn check(source: &str, expected: &[(&str, f64)], tolerance: f64) -> Vec<tddlog::Run> {
    let mut runs = Vec::new();
    for (strategy, projection) in strategies() {
        let options = common::options(strategy, projection);
        let what = format!("{source} ({strategy:?}, {projection:?})");
        let refused = |e: &tddlog::Error| matches!(e, tddlog::Error::Vtree(e) if e.contains("reads no atom through one"));
        let close = |results: &[tddlog::QueryResult], how: &str| {
            assert_eq!(results.len(), expected.len(), "{what} {how}");
            for (r, (atom, p)) in results.iter().zip(expected) {
                assert_eq!(r.to_string(), *atom, "{what} {how}");
                let err = (r.probability - p).abs();
                assert!(
                    err <= tolerance * p.abs().max(f64::MIN_POSITIVE) || (*p == 0.0 && err == 0.0),
                    "{what} {how}: {r} = {:e} against {p:e}",
                    r.probability
                );
            }
        };
        match tddlog::solve(source, &options, &mut |_| {}) {
            Ok(solution) => close(
                solution.answers.as_deref().expect("answered"),
                "through solve",
            ),
            Err(e) if refused(&e) => continue,
            Err(e) => panic!("{what}: {e}"),
        }
        let run = common::run(source, &options).unwrap_or_else(|e| panic!("{what}: {e}"));
        close(
            run.results.as_deref().expect("answered"),
            "through solve_ground",
        );
        runs.push(run);
    }
    runs
}

/// A chain of gates over `n` facts, as a circuit's front end writes a
/// test of one value: `g_i :- g_{i-1}, f_i` with every third literal
/// negated, beside gates that read the facts in pairs, which the query
/// reads too. The query holds in one assignment of the facts, of
/// probability `2^-n`-like, and observed it fixes every fact it reads and
/// leaves no derived atom.
#[test]
fn a_rare_query_is_answered_by_the_probability_of_its_observation() {
    let n = 30;
    let p = |i: usize| 0.3 + 0.4 * (i as f64 / n as f64);
    let lit = |i: usize| {
        if i.is_multiple_of(3) {
            format!("\\+f{i}")
        } else {
            format!("f{i}")
        }
    };
    let mut source = String::new();
    let mut want = 1.0;
    for i in 1..=n {
        let p = common::milli((p(i) * 1000.0).round() as usize);
        source.push_str(&format!("{p}::f{i}.\n"));
        want *= if i.is_multiple_of(3) { 1.0 - p } else { p };
    }
    source.push_str(&format!("g1 :- {}.\n", lit(1)));
    for i in 2..=n {
        source.push_str(&format!("g{i} :- g{}, {}.\n", i - 1, lit(i)));
        source.push_str(&format!(
            "h{i} :- f{}, f{i}.\nh{i} :- \\+f{}, \\+f{i}.\n",
            i - 1,
            i - 1
        ));
    }
    let pairs: Vec<String> = (2..=n).map(|i| format!("\\+h{i}")).collect();
    source.push_str(&format!("odd :- {}.\n", pairs[..4].join(", ")));
    source.push_str(&format!("q :- g{n}.\nq :- odd, g{n}, f1.\nquery(q).\n"));
    for run in check(&source, &[("q", want)], 1e-12) {
        assert_eq!(run.ground.num_derived(), 0, "{:?}", run.kind);
        assert_eq!(run.ground.num_queries(), 1, "{:?}", run.kind);
    }
}

/// A 3-CNF over ten facts written as a Dice program's chain of
/// conjunctions, `x_k :- x_{k-1}, c_k` with `c_k` an atom of three rules
/// of one literal each, its one query the last `x`, positive and
/// negative: against the sum over the worlds that satisfy every clause.
#[test]
fn the_query_of_a_chain_of_clauses_against_enumeration() {
    let (n, m) = (10, 30);
    let mut rng = common::Lcg(7);
    let probs: Vec<f64> = (0..n)
        .map(|_| common::milli(100 + (rng.next() * 800.0) as usize))
        .collect();
    let clauses: Vec<[(usize, bool); 3]> = (0..m)
        .map(|_| {
            let mut c = [(0, true); 3];
            for l in &mut c {
                *l = ((rng.next() * n as f64) as usize, rng.next() < 0.5);
            }
            c
        })
        .collect();
    let mut source = String::new();
    for (i, p) in probs.iter().enumerate() {
        source.push_str(&format!("{p}::v{i}.\n"));
    }
    for (k, c) in clauses.iter().enumerate() {
        for &(v, positive) in c {
            let sign = if positive { "" } else { "\\+" };
            source.push_str(&format!("c{k} :- {sign}v{v}.\n"));
        }
        if k == 0 {
            source.push_str("x0 :- c0.\n");
        } else {
            source.push_str(&format!("x{k} :- x{}, c{k}.\n", k - 1));
        }
    }
    let mut want = 0.0;
    for world in 0..1u32 << n {
        let holds = |v: usize| world >> v & 1 == 1;
        if clauses
            .iter()
            .all(|c| c.iter().any(|&(v, positive)| holds(v) == positive))
        {
            want += (0..n)
                .map(|v| if holds(v) { probs[v] } else { 1.0 - probs[v] })
                .product::<f64>();
        }
    }
    assert!(want > 0.0 && want < 0.1, "a rare query: {want}");
    let last = format!("x{}", m - 1);
    check(
        &format!("{source}query({last}).\n"),
        &[(last.as_str(), want)],
        1e-10,
    );
    check(
        &format!("{source}query(\\+{last}).\n"),
        &[(format!("\\+{last}").as_str(), 1.0 - want)],
        1e-12,
    );
}

/// The lone query of an annotated disjunction's choice, which the
/// observation leaves some heads of false (`renormalize`), and of a cycle,
/// which the conditioning leaves alone; and of an atom no world derives,
/// zero, and its negation, one. By hand: `P(q) = 0.2 0.4 + 0.5 0.7`, and
/// `p(1, 3)` holds where `e(1, 2)` and `e(2, 3)` do.
#[test]
fn the_lone_query_of_a_table_a_cycle_and_an_impossible_atom() {
    let table = "0.2::x(1); 0.5::x(2); 0.3::x(3).\n\
                 0.4::y :- x(1).\n0.7::y :- x(2).\n\
                 q :- y, \\+x(3).\n";
    check(&format!("{table}query(q).\n"), &[("q", 0.43)], 1e-12);
    check(&format!("{table}query(\\+q).\n"), &[("\\+q", 0.57)], 1e-12);
    let cycle = "0.5::e(1,2). 0.6::e(2,1). 0.7::e(2,3). 0.8::e(3,1).\n\
                 p(X,Y) :- e(X,Y).\np(X,Y) :- e(X,Z), p(Z,Y).\n";
    check(
        &format!("{cycle}query(p(1,3)).\n"),
        &[("p(1,3)", 0.35)],
        1e-12,
    );
    let never = "0.3::a.\nr :- a, \\+a.\n";
    check(&format!("{never}query(r).\n"), &[("r", 0.0)], 0.0);
    check(&format!("{never}query(\\+r).\n"), &[("\\+r", 1.0)], 1e-15);
}

/// The evidence a lone query is observed as is the program's own: its
/// `P(e)` is one, as for any program that observes nothing. A program
/// with evidence, or with two queries, or whose query fixes no fact (a
/// chain `x_k = if x_{k-1} then 0.2 else 0.3`, `P(x_k) = 0.3 - 0.1
/// P(x_{k-1})` from `P(x_0) = 0.1`), keeps its queries, and the program
/// `--ground` prints keeps a lone query a query.
#[test]
fn a_lone_query_leaves_p_e_one_and_other_programs_their_queries() {
    let source = "0.3::a. 0.6::b.\nq :- a, b.\nquery(q).\n";
    let options = Options {
        evidence_probability: true,
        ..Options::default()
    };
    let solution = tddlog::solve(source, &options, &mut |_| {}).unwrap();
    common::assert_answers(
        source,
        solution.answers.as_deref().unwrap(),
        &[("q", 0.18)],
        1e-15,
    );
    let pe = solution.evidence_probability.expect("asked for");
    assert!((pe.to_f64() - 1.0).abs() < 1e-15, "P(e) {}", pe.to_f64());
    let printed = tddlog::rewritten(source, &Options::default()).unwrap();
    assert_eq!(
        (printed.num_queries(), printed.evidence.len()),
        (1, 0),
        "--ground keeps the query"
    );
    for (other, expected) in [
        (
            "0.3::a. 0.6::b.\nq :- a, b.\nevidence(b, true).\nquery(q).\n",
            vec![("q", 0.3)],
        ),
        (
            "0.3::a. 0.6::b.\nq :- a, b.\nquery(q).\nquery(a).\n",
            vec![("q", 0.18), ("a", 0.3)],
        ),
        (
            "0.1::x(0).\n0.2::a(1). 0.3::b(1). 0.2::a(2). 0.3::b(2). 0.2::a(3). 0.3::b(3).\n\
             x(1) :- x(0), a(1).\nx(1) :- \\+x(0), b(1).\n\
             x(2) :- x(1), a(2).\nx(2) :- \\+x(1), b(2).\n\
             x(3) :- x(2), a(3).\nx(3) :- \\+x(2), b(3).\nquery(x(3)).\n",
            vec![("x(3)", 0.2729)],
        ),
    ] {
        let run = common::run(other, &Options::default()).unwrap();
        common::assert_answers(other, run.results.as_deref().unwrap(), &expected, 1e-14);
        // As the grounder leaves it, before a layout names atoms through
        // observed copies.
        let program = tddlog::syntax::parse_program(other).unwrap();
        let ground = tddlog::ground::ground(&program, &Options::default().ground).unwrap();
        if !other.contains("evidence(b") {
            assert_eq!(ground.evidence.len(), 0, "{other}: observations");
        }
    }
}
