//! Reachability on a directed grid under the `families` layout, whose
//! groups the grounder numbers a layer at a time, against the answer of a
//! frontier dynamic program ([`frontier`]).
use crate::common;

use common::Rows;
use tddlog::Projection;
use tddlog::{Kind, Strategy};

/// The probability of the edge out of node `k` of the grid, right (`down`
/// false) or down: a fixed sequence strictly between 0 and 1.
fn edge(k: usize, down: bool) -> f64 {
    ((2 * k + down as usize) * 37 % 89 + 5) as f64 / 100.0
}

/// The n × n grid with edges right and down, as bench/gridN.pl, and the
/// query that its far corner is reached from the near one.
fn grid(n: usize) -> String {
    let mut source = String::new();
    for i in 0..n {
        for j in 0..n {
            let k = i * n + j;
            if j + 1 < n {
                source += &format!("{}::edge(n{i}_{j},n{i}_{}).\n", edge(k, false), j + 1);
            }
            if i + 1 < n {
                source += &format!("{}::edge(n{i}_{j},n{}_{j}).\n", edge(k, true), i + 1);
            }
        }
    }
    source
        + &format!(
            "path(X,Y) :- edge(X,Y).\npath(X,Y) :- edge(X,Z), path(Z,Y).\nquery(path(n0_0,n{m}_{m})).\n",
            m = n - 1
        )
}

/// The answer of [`grid`] by a dynamic program over the nodes in row order,
/// whose state is the set of nodes reached among those whose edges are
/// still to come, one per column; and the number of states of positive
/// weight summed over the steps.
fn frontier(n: usize) -> (f64, usize) {
    let mut weight = vec![0.0; 1 << n];
    weight[1] = 1.0;
    let mut states = 0;
    for i in 0..n {
        for j in 0..n {
            if i == n - 1 && j == n - 1 {
                let p = weight
                    .iter()
                    .enumerate()
                    .filter(|&(s, _)| s >> j & 1 == 1)
                    .map(|(_, w)| w)
                    .sum();
                return (p, states);
            }
            let k = i * n + j;
            let (right, down) = (
                if j + 1 < n { edge(k, false) } else { 0.0 },
                if i + 1 < n { edge(k, true) } else { 0.0 },
            );
            let mut next = vec![0.0; 1 << n];
            for (s, &w) in weight.iter().enumerate().filter(|&(_, &w)| w > 0.0) {
                if s >> j & 1 == 0 {
                    next[s] += w;
                    continue;
                }
                for (r, pr) in [(true, right), (false, 1.0 - right)] {
                    for (d, pd) in [(true, down), (false, 1.0 - down)] {
                        let mut t = if d { s } else { s & !(1 << j) };
                        if r && j + 1 < n {
                            t |= 1 << (j + 1);
                        }
                        next[t] += w * pr * pd;
                    }
                }
            }
            weight = next;
            states += weight.iter().filter(|&&w| w > 0.0).count();
        }
    }
    unreachable!("the last node returns")
}

/// The run of [`grid`] on `families:0` with every indicator: its answer and
/// the evidence formula's pairs.
fn run(n: usize) -> (f64, usize) {
    let gp = common::ground(&grid(n), Rows::Classes, true);
    let strategy = Strategy::Trial(Kind::Families { search: false });
    let run = tddlog::solve_ground(gp, &common::options(strategy, Projection::default())).unwrap();
    (
        run.results.unwrap()[0].probability,
        run.query_sizes.evidence,
    )
}

#[test]
fn a_grid_keeps_its_answer() {
    for n in [4, 7, 10] {
        let (expected, _) = frontier(n);
        let (p, _) = run(n);
        assert!((p - expected).abs() < 1e-12, "{n}: {p} against {expected}");
    }
}

#[test]
fn a_grid_is_laid_out_a_layer_at_a_time() {
    // Laid out as the grounder numbers its groups, a layer at a time, the
    // grid has its side as width, and its evidence stays under the frontier
    // program's states summed over the steps; on min-fill's decomposition,
    // of width 23, bench/grid15.pl's took 24.1 M pairs against 6.5 M states.
    let (expected, states) = frontier(15);
    let (p, evidence) = run(15);
    assert!((p - expected).abs() < 1e-12, "{p} against {expected}");
    assert!(
        evidence < states,
        "evidence {evidence} pairs, {states} frontier states"
    );
}
