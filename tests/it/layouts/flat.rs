//! The Boolean queries of a tuple-independent database (as
//! `scripts/gen_pdb.py` writes them), whose rules read facts alone, under
//! `dynamic`: against the answers of enumerating every world, and the
//! size the bushy start keeps the compile to.
use crate::common;

use common::Rows;
use tddlog::Projection;
use tddlog::{Kind, Strategy};

/// A probability strictly between 0 and 1 for the `k`-th fact.
fn prob(k: usize) -> f64 {
    ((k * 37) % 89 + 5) as f64 / 100.0
}

/// The S-tuples over a domain of `n` constants: each constant related to
/// two others, fixed by `n` alone.
fn tuples(n: usize) -> Vec<(usize, usize)> {
    let mut s: Vec<(usize, usize)> = (0..n)
        .flat_map(|x| [(x, (7 * x + 3) % n), (x, (11 * x + 5) % n)])
        .collect();
    s.sort_unstable();
    s.dedup();
    s
}

/// The program over a domain of `n` constants: R and T over the domain,
/// S over [`tuples`], and the queries `h0`, `h1` and `safe` of gen_pdb.py.
fn program(n: usize) -> String {
    let mut source = String::new();
    for x in 0..n {
        source += &format!("{}::r(c{x}).\n{}::t(c{x}).\n", prob(2 * x), prob(2 * x + 1));
    }
    for (k, &(x, y)) in tuples(n).iter().enumerate() {
        source += &format!("{}::s(c{x},c{y}).\n", prob(2 * n + k));
    }
    source
        + "h0 :- r(X), s(X,Y), t(Y).\nh1 :- r(X), s(X,Y).\nh1 :- s(X,Y), t(Y).\nsafe :- r(X), s(X,Y).\n\
              query(h0).\nquery(h1).\nquery(safe).\n"
}

/// The answers of [`program`] by enumerating its worlds.
fn enumerate(n: usize) -> [f64; 3] {
    let s = tuples(n);
    let facts = 2 * n + s.len();
    assert!(facts <= 20, "{facts} facts to enumerate");
    let mut answers = [0.0; 3];
    for world in 0u32..1 << facts {
        let holds = |k: usize| world >> k & 1 == 1;
        let weight: f64 = (0..facts)
            .map(|k| if holds(k) { prob(k) } else { 1.0 - prob(k) })
            .product();
        let (r, t) = (|x: usize| holds(2 * x), |y: usize| holds(2 * y + 1));
        let present: Vec<(usize, usize)> = s
            .iter()
            .enumerate()
            .filter(|&(k, _)| holds(2 * n + k))
            .map(|(_, &e)| e)
            .collect();
        let h0 = present.iter().any(|&(x, y)| r(x) && t(y));
        let h1 = present.iter().any(|&(x, y)| r(x) || t(y));
        let safe = present.iter().any(|&(x, _)| r(x));
        for (a, q) in answers.iter_mut().zip([h0, h1, safe]) {
            if q {
                *a += weight;
            }
        }
    }
    answers
}

/// The run of [`program`] under `dynamic`: its answers and the largest
/// formula the compile held.
fn run(n: usize) -> (Vec<tddlog::QueryResult>, usize) {
    let gp = common::ground(&program(n), Rows::Classes, false);
    let run = tddlog::solve_ground(
        gp,
        &common::options(Strategy::Trial(Kind::Dynamic), Projection::Off),
    )
    .unwrap();
    (run.results.unwrap(), run.stats.max_pairs)
}

#[test]
fn a_database_query_keeps_its_answers() {
    for n in [3, 4, 5] {
        let (results, _) = run(n);
        let [h0, h1, safe] = enumerate(n);
        common::assert_answers(
            &format!("domain {n}"),
            &results,
            &[("h0", h0), ("h1", h1), ("safe", safe)],
            1e-12,
        );
    }
}

#[test]
fn a_database_query_starts_from_the_elimination_tree() {
    // Every rule reads facts alone, so `dynamic` starts from the min-fill
    // elimination tree folded bushy rather than from a right-linear order
    // through it, from which this compile held 19 811 pairs before the
    // search lowered them.
    let (_, pairs) = run(40);
    assert!(pairs < 3_000, "{pairs} pairs");
}
