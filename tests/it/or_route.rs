//! A body of two derived parts goes into its head's formula `acc` by the
//! or-route, `(acc ∧ ¬a) ∨ (acc ∧ ¬b)`, where `acc` is small against the
//! parts or their product runs long (`src/tp/clauses.rs`). On a random
//! graph whose compile under `auto` takes that route, the answers are those
//! of enumerating every world of its edges.

use crate::common;

use tddlog::Projection;
use tddlog::Strategy;

/// The edges of [`scripts/gen_graph.py 9 18 --seed 2`](https://github.com/Tractables/tddlog/blob/baselines/scripts/gen_graph.py), as (from, to,
/// probability).
const EDGES: [(usize, usize, f64); 18] = [
    (1, 2, 0.69),
    (1, 3, 0.83),
    (2, 6, 0.71),
    (3, 4, 0.73),
    (3, 5, 0.38),
    (3, 6, 0.88),
    (3, 9, 0.87),
    (4, 1, 0.23),
    (5, 4, 0.7),
    (6, 7, 0.67),
    (6, 8, 0.47),
    (6, 9, 0.52),
    (7, 9, 0.49),
    (8, 7, 0.84),
    (9, 3, 0.5),
    (9, 5, 0.77),
    (9, 6, 0.38),
    (9, 8, 0.81),
];

const NODES: usize = 9;

/// `P(path(1, k))` for `k` from 2 to [`NODES`], by enumerating the worlds
/// of the edges.
fn enumerated() -> Vec<f64> {
    let mut reach = vec![0.0; NODES + 1];
    for world in 0..1usize << EDGES.len() {
        let weight: f64 = EDGES
            .iter()
            .enumerate()
            .map(|(e, &(_, _, p))| if world >> e & 1 == 1 { p } else { 1.0 - p })
            .product();
        // Node 1 reaches itself only through a cycle, as `path(1,1)` would.
        let mut seen = 0usize;
        let mut frontier = 1usize << 1;
        while frontier != 0 {
            let next = EDGES
                .iter()
                .enumerate()
                .filter(|&(e, &(a, _, _))| world >> e & 1 == 1 && frontier >> a & 1 == 1)
                .fold(0, |s, (_, &(_, b, _))| s | 1 << b);
            frontier = next & !seen;
            seen |= next;
        }
        for (k, r) in reach.iter_mut().enumerate().skip(2) {
            if seen >> k & 1 == 1 {
                *r += weight;
            }
        }
    }
    reach.split_off(2)
}

#[test]
fn a_graph_compiled_by_the_or_route_answers_as_its_worlds() {
    let mut source = String::new();
    for (a, b, p) in EDGES {
        source += &format!("{p}::edge({a},{b}).\n");
    }
    source += "path(X,Y) :- edge(X,Y).\npath(X,Y) :- edge(X,Z), path(Z,Y).\n";
    for k in 2..=NODES {
        source += &format!("query(path(1,{k})).\n");
    }
    let expected: Vec<(String, f64)> = (2..=NODES)
        .map(|k| format!("path(1,{k})"))
        .zip(enumerated())
        .collect();
    let run = common::run(
        &source,
        &common::options(Strategy::Auto, Projection::default()),
    )
    .unwrap();
    assert!(
        run.stats.clause_memo.or_routes > 0,
        "the compile takes no body by the or-route"
    );
    common::assert_answers("auto", &run.results.expect("no evidence"), &expected, 1e-12);
}
