//! Queries on the values of a table's leaf variable answered through the
//! formula of its first value counted under other probabilities, against
//! the posteriors of the Bayesian network computed by summing its joint.
use crate::common;

use tddlog::Projection;
use tddlog::{Kind, Strategy};

/// `a` and `b` are parents of the leaves `c` and `e`; `d`, a child of `b`,
/// is observed. Each table is its rows, one per assignment to the parents
/// in order, each row the probabilities of the values. A row of `c` sums
/// to less than one, so `c` takes none of its values there.
const A: [f64; 2] = [0.35, 0.65];
const B: [[f64; 3]; 2] = [[0.2, 0.5, 0.3], [0.6, 0.15, 0.25]];
const C: [[f64; 4]; 6] = [
    [0.1, 0.2, 0.3, 0.4],
    [0.25, 0.25, 0.4, 0.1],
    [0.7, 0.1, 0.1, 0.1],
    [0.05, 0.15, 0.3, 0.4],
    [0.3, 0.3, 0.2, 0.2],
    [0.45, 0.05, 0.35, 0.15],
];
const D: [[f64; 2]; 3] = [[0.9, 0.1], [0.3, 0.7], [0.55, 0.45]];
const E: [[f64; 2]; 3] = [[0.15, 0.85], [0.6, 0.4], [0.8, 0.2]];

fn table(head: &str, values: usize, row: &[f64], body: &str) -> String {
    let choices: Vec<String> = (0..values)
        .map(|v| format!("{}::{head}(v{v})", row[v]))
        .collect();
    if body.is_empty() {
        format!("{}.\n", choices.join("; "))
    } else {
        format!("{} :- {body}.\n", choices.join("; "))
    }
}

/// The network as a program querying every value of `b`, `c` and `e`, with
/// `d(v0)` observed where `evidence` says.
fn program(evidence: bool) -> String {
    let mut source = table("a", 2, &A, "");
    for (i, row) in B.iter().enumerate() {
        source += &table("b", 3, row, &format!("a(v{i})"));
    }
    for (i, row) in C.iter().enumerate() {
        source += &table("c", 4, row, &format!("a(v{}), b(v{})", i / 3, i % 3));
    }
    for (j, row) in D.iter().enumerate() {
        source += &table("d", 2, row, &format!("b(v{j})"));
    }
    for (j, row) in E.iter().enumerate() {
        source += &table("e", 2, row, &format!("b(v{j})"));
    }
    if evidence {
        source += "evidence(d(v0)).\n";
    }
    for (head, values) in [("b", 3), ("c", 4), ("e", 2)] {
        for v in 0..values {
            source += &format!("query({head}(v{v})).\n");
        }
    }
    source
}

/// The posteriors of the queries, in the program's order, by summing the
/// joint over `a` and `b`.
fn posteriors(evidence: bool) -> Vec<f64> {
    let (mut total, mut b, mut c, mut e) = (0.0, [0.0; 3], [0.0; 4], [0.0; 2]);
    for i in 0..2 {
        for j in 0..3 {
            let w = A[i] * B[i][j] * if evidence { D[j][0] } else { 1.0 };
            total += w;
            b[j] += w;
            for v in 0..4 {
                c[v] += w * C[i * 3 + j][v];
            }
            for v in 0..2 {
                e[v] += w * E[j][v];
            }
        }
    }
    b.iter().chain(&c).chain(&e).map(|p| p / total).collect()
}

#[test]
fn the_values_of_a_leaf_are_counted_on_its_first_values_formula() {
    let strategies = [
        (Strategy::Auto, Projection::default()),
        (
            Strategy::Trial(Kind::Families { search: false }),
            Projection::default(),
        ),
        (Strategy::Trial(Kind::Closed), Projection::default()),
        (Strategy::Trial(Kind::Td), Projection::default()),
        (Strategy::Trial(Kind::Dynamic), Projection::Off),
    ];
    for evidence in [true, false] {
        let expected = posteriors(evidence);
        for (strategy, projection) in strategies {
            let what = format!("{strategy}, evidence {evidence}");
            let run =
                common::run(&program(evidence), &common::options(strategy, projection)).unwrap();
            // Values 1 to 3 of `c` and value 1 of `e`.
            assert_eq!(run.stats.templated, 4, "{what}: queries through a template");
            common::assert_probabilities(&what, &run.results.unwrap(), &expected, 1e-12);
        }
    }
}
