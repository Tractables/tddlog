//! The block of a group in the `families` vtree (`nest` in
//! `src/vtree/nest.rs`): the switches of one row of a table go in a tree to
//! the right of its parents' copies, and the switches of a decision list,
//! read under exclusive contexts, stay on the spine.
use crate::common;

use common::Rows;
use tddlog::{Kind, Strategy};

/// Each answer of `source` under `families:0` against `expected`; the
/// largest formula of the compile and of the query phase.
fn run(source: &str, expected: &[(String, f64)]) -> (usize, usize) {
    let gp = common::ground(source, Rows::Classes, true);
    let strategy = Strategy::Trial(Kind::Families { search: false });
    let run = tddlog::solve_ground(
        gp,
        &common::options(strategy, tddlog::Projection::default()),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    common::assert_answers("families:0", &run.results.unwrap(), expected, 1e-9);
    (run.stats.max_pairs, run.query_sizes.largest)
}

#[test]
fn a_rows_switches_go_in_a_tree_beside_the_parents_copies() {
    // c has twelve values and a row for each of r's twelve. On a spine of
    // switches above r's copies, every level held a class for each row
    // still to come: the compile's largest formula took 2 141 pairs and
    // the evidence 1 408. With each row's switches in a tree, 821 and 641;
    // the walk leaves the evidence formula as its last conjunction made
    // it, 1 452 pairs.
    let (rows, values) = (12, 12);
    let weight: Vec<f64> = (0..rows).map(|i| ((i * 5) % 7 + 1) as f64).collect();
    let total: f64 = weight.iter().sum();
    let prior: Vec<f64> = weight.iter().map(|w| w / total).collect();
    let mut source = (0..rows)
        .map(|i| format!("{}::r(v{i})", prior[i]))
        .collect::<Vec<_>>()
        .join("; ")
        + ".\n";
    let mut marginal = vec![0.0; values];
    for (i, p) in prior.iter().enumerate() {
        let w: Vec<f64> = (0..values)
            .map(|k| ((i * 7 + k * 3) % 11 + 1) as f64)
            .collect();
        let s: f64 = w.iter().sum();
        source += &((0..values)
            .map(|k| format!("{}::c({k})", w[k] / s))
            .collect::<Vec<_>>()
            .join("; ")
            + &format!(" :- r(v{i}).\n"));
        for k in 0..values {
            marginal[k] += p * w[k] / s;
        }
    }
    source += "evidence(c(3), false).\n";
    let mut expected = Vec::new();
    for (k, m) in marginal.iter().enumerate() {
        source += &format!("query(c({k})).\n");
        expected.push((
            format!("c({k})"),
            if k == 3 { 0.0 } else { m / (1.0 - marginal[3]) },
        ));
    }
    let (compile, query) = run(&source, &expected);
    assert!(
        compile <= 1_000 && query <= 1_600,
        "compile {compile} pairs, query phase {query}"
    );
}

#[test]
fn a_decision_lists_switches_stay_on_the_spine() {
    // shared_publication(A, B) of uwcse: seven rules for one head, each
    // with a switch of its own, under exclusive contexts that share
    // parents across the split on the first, so they form one part. In a
    // tree of their own the seven switches are a class for each subset of
    // the rows, 779 pairs; on the spine 102.
    let x = [0.6, 0.35, 0.7, 0.45, 0.55];
    let y = [0.2, 0.5, 0.1, 0.3, 0.4];
    let mut source = String::new();
    for i in 0..5 {
        source += &format!(
            "{}::x({i}).\n{}::y({i}).\na({i}) :- x({i}).\na({i}) :- y({i}).\n",
            x[i], y[i]
        );
    }
    let rows: [(f64, &[(usize, bool)]); 7] = [
        (0.2, &[(0, true), (1, true)]),
        (0.3, &[(0, true), (2, true), (3, false), (1, false)]),
        (0.4, &[(0, true), (2, false), (1, false)]),
        (0.5, &[(0, false), (2, true), (3, false)]),
        (0.6, &[(0, false), (4, true), (3, true), (2, false)]),
        (0.7, &[(0, false), (3, true), (2, false), (4, false)]),
        (0.8, &[(0, false), (2, false), (3, false)]),
    ];
    let a: Vec<f64> = (0..5).map(|i| 1.0 - (1.0 - x[i]) * (1.0 - y[i])).collect();
    let mut p = 0.0;
    for (q, context) in rows {
        let body: Vec<String> = context
            .iter()
            .map(|&(i, positive)| format!("{}a({i})", if positive { "" } else { "\\+" }))
            .collect();
        source += &format!("{q}::h :- {}.\n", body.join(", "));
        p += q * context
            .iter()
            .map(|&(i, positive)| if positive { a[i] } else { 1.0 - a[i] })
            .product::<f64>();
    }
    source += "query(h).\n";
    let (compile, _) = run(&source, &[("h".to_string(), p)]);
    assert!(compile <= 200, "compile {compile} pairs");
}
