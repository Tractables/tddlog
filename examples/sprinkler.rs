//! A Bayesian network written as a program from Rust data, and its
//! posteriors under several observations, each checked against the sum
//! over the network's sixteen worlds.
//!
//! ```text
//! cargo run --release --example sprinkler
//! ```

/// Each variable with its parents and its probability of being true under
/// each assignment of the parents: all true first, the first parent's
/// value changing slowest.
const NETWORK: [(&str, &[&str], &[f64]); 4] = [
    ("cloudy", &[], &[0.5]),
    ("sprinkler", &["cloudy"], &[0.1, 0.5]),
    ("rain", &["cloudy"], &[0.8, 0.2]),
    ("wet", &["sprinkler", "rain"], &[0.99, 0.9, 0.9, 0.0]),
];

/// Whether parent `j` of `k` is true in the parents' assignment `row`.
fn parent_true(row: usize, j: usize, k: usize) -> bool {
    row >> (k - 1 - j) & 1 == 0
}

/// The network as a program: a probabilistic rule for each row of each
/// table, whose bodies exclude one another, so that the head's
/// probability under the row is the row's.
fn program(observed: &[(&str, bool)], query: &str) -> String {
    let mut text = String::new();
    for (var, parents, rows) in NETWORK {
        for (row, &p) in rows.iter().enumerate() {
            if p == 0.0 {
                continue;
            }
            let body: Vec<String> = parents
                .iter()
                .enumerate()
                .map(|(j, parent)| {
                    if parent_true(row, j, parents.len()) {
                        parent.to_string()
                    } else {
                        format!("\\+{parent}")
                    }
                })
                .collect();
            if body.is_empty() {
                text += &format!("{p}::{var}.\n");
            } else {
                text += &format!("{p}::{var} :- {}.\n", body.join(", "));
            }
        }
    }
    for (var, value) in observed {
        text += &format!("evidence({var}, {value}).\n");
    }
    text += &format!("query({query}).\n");
    text
}

/// `P(query | observed)` summed over the worlds, for the check.
fn enumerate(observed: &[(&str, bool)], query: &str) -> f64 {
    let index = |name: &str| NETWORK.iter().position(|(v, _, _)| *v == name).unwrap();
    let (mut joint, mut evidence) = (0.0, 0.0);
    for world in 0..1usize << NETWORK.len() {
        let value = |i: usize| world >> i & 1 == 1;
        if observed.iter().any(|&(var, v)| value(index(var)) != v) {
            continue;
        }
        let mut weight = 1.0;
        for (i, (_, parents, rows)) in NETWORK.iter().enumerate() {
            let row = (0..rows.len())
                .find(|&row| {
                    (0..parents.len())
                        .all(|j| parent_true(row, j, parents.len()) == value(index(parents[j])))
                })
                .unwrap();
            weight *= if value(i) { rows[row] } else { 1.0 - rows[row] };
        }
        evidence += weight;
        if value(index(query)) {
            joint += weight;
        }
    }
    joint / evidence
}

fn main() -> Result<(), tddlog::Error> {
    let cases: [(&[(&str, bool)], &str); 4] = [
        (&[], "wet"),
        (&[("wet", true)], "rain"),
        (&[("wet", true)], "sprinkler"),
        (&[("wet", true), ("sprinkler", false)], "cloudy"),
    ];
    for (observed, query) in cases {
        let solution = tddlog::solve(
            &program(observed, query),
            &tddlog::Options::default(),
            &mut |_| {},
        )?;
        let p = solution.answers.expect("consistent evidence")[0].probability;
        let expected = enumerate(observed, query);
        assert!(
            (p - expected).abs() < 1e-12,
            "{query}: {p} against {expected}"
        );
        let given: Vec<String> = observed
            .iter()
            .map(|(var, v)| {
                if *v {
                    var.to_string()
                } else {
                    format!("not {var}")
                }
            })
            .collect();
        if given.is_empty() {
            println!("P({query}) = {p:.6}");
        } else {
            println!("P({query} | {}) = {p:.6}", given.join(", "));
        }
    }
    Ok(())
}
