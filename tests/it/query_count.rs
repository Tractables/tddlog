//! Queries counted against the evidence formula without building their
//! conjunction with it, where a query's formula reads variables the
//! evidence formula does not hold, against enumeration of every world.
use crate::common;

use common::Rows;
use tddlog::{Kind, Strategy};

/// The 3 × 3 grid with edges right and down, each edge a fact with a
/// probability of its own: its edges as (from, to, probability).
fn edges() -> Vec<(usize, usize, f64)> {
    let mut out = Vec::new();
    for i in 0..3 {
        for j in 0..3 {
            let k = i * 3 + j;
            let p = |down: usize| ((2 * k + down) * 37 % 89 + 5) as f64 / 100.0;
            if j < 2 {
                out.push((k, k + 1, p(0)));
            }
            if i < 2 {
                out.push((k, k + 3, p(1)));
            }
        }
    }
    out
}

fn name(k: usize) -> String {
    format!("n{}_{}", k / 3, k % 3)
}

/// The observed path and the queried ones. The first query reads edges
/// the evidence does not, the second shares some, the third none.
const OBSERVED: (usize, usize) = (0, 4);
const QUERIES: [(usize, usize); 3] = [(0, 8), (3, 7), (1, 2)];

/// `P(path(q) | path(OBSERVED))` for each query, by enumerating the
/// worlds of the edges.
fn enumerated() -> Vec<f64> {
    let edges = edges();
    let reaches = |world: usize, from: usize, to: usize| {
        let mut seen = 1usize << from;
        loop {
            let next = edges
                .iter()
                .enumerate()
                .filter(|&(e, &(a, _, _))| world >> e & 1 == 1 && seen >> a & 1 == 1)
                .fold(seen, |s, (_, &(_, b, _))| s | 1 << b);
            if next == seen {
                return seen >> to & 1 == 1;
            }
            seen = next;
        }
    };
    let (mut evidence, mut joint) = (0.0, vec![0.0; QUERIES.len()]);
    for world in 0..1usize << edges.len() {
        let weight: f64 = edges
            .iter()
            .enumerate()
            .map(|(e, &(_, _, p))| if world >> e & 1 == 1 { p } else { 1.0 - p })
            .product();
        if !reaches(world, OBSERVED.0, OBSERVED.1) {
            continue;
        }
        evidence += weight;
        for (j, &(from, to)) in QUERIES.iter().enumerate() {
            if reaches(world, from, to) {
                joint[j] += weight;
            }
        }
    }
    joint.iter().map(|j| j / evidence).collect()
}

#[test]
fn a_query_with_variables_the_evidence_lacks_is_counted_against_it() {
    let mut source = String::new();
    for (a, b, p) in edges() {
        source += &format!("{p}::edge({},{}).\n", name(a), name(b));
    }
    source += "path(X,Y) :- edge(X,Y).\npath(X,Y) :- edge(X,Z), path(Z,Y).\n";
    source += &format!(
        "evidence(path({},{})).\n",
        name(OBSERVED.0),
        name(OBSERVED.1)
    );
    for (a, b) in QUERIES {
        source += &format!("query(path({},{})).\n", name(a), name(b));
    }
    let expected = enumerated();
    let gp = common::ground(&source, Rows::Classes, true);
    let strategy = Strategy::Trial(Kind::Families { search: false });
    let run = tddlog::solve_ground(
        gp,
        &common::options(strategy, tddlog::Projection::default()),
    )
    .unwrap();
    assert_eq!(
        run.query_sizes.products,
        QUERIES.len(),
        "the queries are counted against the evidence"
    );
    common::assert_probabilities("families:0", &run.results.unwrap(), &expected, 1e-12);
}

/// Smokers on a chain of friends, one pair of them mutual (`p1` and `p2`),
/// `people` in all: only a mutual pair influences each other, so
/// `smokes(X)` is `stress(X)` but for the pair, where it is `s1 ∨ (i21 ∧
/// s2)`, of probability `1 - 0.7 · (1 - 0.2 · 0.3) = 0.342`.
fn smokers(people: usize, evidence: bool) -> (String, Vec<(String, f64)>) {
    let mut source = String::from(
        "0.3::stress(X) :- person(X).\n\
         0.2::influences(X,Y) :- friend(X,Y).\n\
         smokes(X) :- stress(X).\n\
         smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).\n\
         friend(p2,p1).\n",
    );
    for i in 1..=people {
        source += &format!("person(p{i}).\n");
        if i < people {
            source += &format!("friend(p{i},p{}).\n", i + 1);
        }
    }
    let mut expected = Vec::new();
    for i in 1..=people {
        source += &format!("query(smokes(p{i})).\n");
        let p = match i {
            1 | 2 => 0.342,
            5 if evidence => 1.0,
            7 if evidence => 0.0,
            _ => 0.3,
        };
        expected.push((format!("smokes(p{i})"), p));
    }
    source += "query(\\+smokes(p4)).\n";
    expected.push(("\\+smokes(p4)".to_string(), 0.7));
    if evidence {
        source += "evidence(stress(p5), true).\nevidence(stress(p7), false).\n";
    }
    (source, expected)
}

/// A query of a fact is the fact's literal on the fact's own scope, under
/// every projection: on the whole program, each query's literal had a
/// level for every variable to build and count, and the smokers of the
/// standard suite took a time quadratic in their people (30 000 queries,
/// 98 s).
#[test]
fn a_fact_query_is_counted_on_its_own_scope() {
    use Kind::*;
    let people = 300;
    let runs = [
        (Strategy::Auto, tddlog::Projection::default()),
        (Strategy::Trial(Dynamic), tddlog::Projection::Off),
        (Strategy::Trial(Closed), tddlog::Projection::Off),
        (Strategy::Trial(Td), tddlog::Projection::default()),
    ];
    for evidence in [false, true] {
        let (source, expected) = smokers(people, evidence);
        for (strategy, projection) in runs {
            let what = format!("{strategy:?}, {projection:?}, evidence {evidence}");
            let gp = common::ground(&source, Rows::Classes, false);
            let run = tddlog::solve_ground(gp, &common::options(strategy, projection))
                .unwrap_or_else(|e| panic!("{what}: {e}"));
            common::assert_answers(&what, run.results.as_ref().unwrap(), &expected, 1e-12);
            if !evidence {
                assert!(
                    run.query_sizes.largest < 16,
                    "{what}: largest formula {} pairs",
                    run.query_sizes.largest
                );
            }
        }
    }
}
