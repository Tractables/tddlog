//! The evidence walk on a spine: a hidden Markov chain observed at every
//! step, against its forward pass. Each step's observation is a factor of
//! a few pairs at a node beside the chain's result coming up, which the
//! walk joins into the result or puts off while the result outweighs it
//! (`src/query/walk.rs`).
use crate::common;

use common::Rows;
use tddlog::ground::{GroundOptions, GroundProgram};
use tddlog::{Kind, Strategy};

const PRIOR: [f64; 3] = [0.5, 0.3, 0.2];
const MOVE: [[f64; 3]; 3] = [[0.7, 0.2, 0.1], [0.15, 0.6, 0.25], [0.2, 0.3, 0.5]];
/// P(o_i | h_i = v) for each value v.
const SEEN: [f64; 3] = [0.9, 0.4, 0.1];

/// Whether step `i` is observed true.
fn observed(i: usize) -> bool {
    i % 3 != 1
}

/// The chain of `steps` hidden states h_i of three values, each with an
/// observation o_i, every o_i observed, the last state queried.
fn chain(steps: usize) -> String {
    let heads = |i: usize, row: &[f64; 3]| {
        row.iter()
            .enumerate()
            .map(|(v, p)| format!("{p}::h_{i}(v{v})"))
            .collect::<Vec<_>>()
            .join("; ")
    };
    let mut source = format!("{}.\n", heads(0, &PRIOR));
    for i in 1..steps {
        for (v, row) in MOVE.iter().enumerate() {
            source += &format!("{} :- h_{}(v{v}).\n", heads(i, row), i - 1);
        }
    }
    for i in 0..steps {
        for (v, p) in SEEN.iter().enumerate() {
            source += &format!("{p}::o_{i} :- h_{i}(v{v}).\n");
        }
        source += &format!("evidence(o_{i}, {}).\n", observed(i));
    }
    for v in 0..3 {
        source += &format!("query(h_{}(v{v})).\n", steps - 1);
    }
    source
}

/// P(h_last = v, o) for each v, by the forward pass.
fn alpha(steps: usize) -> Vec<f64> {
    let seen = |i: usize, v: usize| if observed(i) { SEEN[v] } else { 1.0 - SEEN[v] };
    let mut alpha: Vec<f64> = (0..3).map(|v| PRIOR[v] * seen(0, v)).collect();
    for i in 1..steps {
        alpha = (0..3)
            .map(|w| (0..3).map(|v| alpha[v] * MOVE[v][w]).sum::<f64>() * seen(i, w))
            .collect();
    }
    alpha
}

/// P(h_last = v | o) for each v, by the forward pass.
fn forward(steps: usize) -> Vec<(String, f64)> {
    let alpha = alpha(steps);
    let total: f64 = alpha.iter().sum();
    alpha
        .iter()
        .enumerate()
        .map(|(v, a)| (format!("h_{}(v{v})", steps - 1), a / total))
        .collect()
}

/// Each strategy, and the steps of the chain it runs. Under closed and td
/// the evidence grows exponentially along the chain: closed's from 16 910
/// pairs at 16 steps to 4.16 M at 20, td's from 5 867 at 3 steps to
/// 3.65 M at 6, where auto's grows by 79 a step.
const RUNS: [(Strategy, usize); 6] = [
    (Strategy::Auto, 40),
    (Strategy::Trial(Kind::Families { search: true }), 40),
    (Strategy::Trial(Kind::Families { search: false }), 40),
    (Strategy::Trial(Kind::Dynamic), 40),
    (Strategy::Trial(Kind::Closed), 12),
    (Strategy::Trial(Kind::Td), 4),
];

/// The ground program `gp` of the chain of `steps` solved under
/// `strategy`.
fn solve(gp: GroundProgram, strategy: Strategy, steps: usize) -> tddlog::Run {
    // dynamic changes the one vtree every formula lives on.
    let projection = if strategy == Strategy::Trial(Kind::Dynamic) {
        tddlog::Projection::Off
    } else {
        tddlog::Projection::default()
    };
    tddlog::solve_ground(gp, &common::options(strategy, projection))
        .unwrap_or_else(|e| panic!("{strategy:?}, {steps} steps: {e}"))
}

#[test]
fn a_chain_observed_at_every_step() {
    use Kind::*;
    for (strategy, steps) in RUNS {
        let gp = common::ground(&chain(steps), Rows::Classes, false);
        let run = solve(gp, strategy, steps);
        common::assert_answers(
            &format!("{strategy:?}, {steps} steps"),
            run.results.as_ref().unwrap(),
            &forward(steps),
            1e-12,
        );
    }
    let gp = common::ground(&chain(40), Rows::Apart, false);
    let run = tddlog::solve_ground(
        gp,
        &common::options(
            Strategy::Trial(Families { search: true }),
            tddlog::Projection::default(),
        ),
    )
    .unwrap();
    common::assert_answers(
        "families, rows apart",
        run.results.as_ref().unwrap(),
        &forward(40),
        1e-12,
    );
}

#[test]
fn a_chain_without_queries_is_summed_out_as_it_is_walked() {
    // With no query, only P(o) is wanted: the walk's joins sum out the
    // steps no later join reads (src/query/sum_out.rs), and P(o) is the
    // last join's weighted value, against the forward pass's. Nothing is
    // detached, so that P(o) is counted (`GroundOptions::detach`).
    let mut summing = 0;
    for (strategy, steps) in RUNS {
        let program = tddlog::syntax::parse_program(&chain(steps))
            .unwrap()
            .without_queries();
        let options = GroundOptions {
            detach: false,
            ..Rows::Classes.options()
        };
        let gp = tddlog::ground::ground(&program, &options).unwrap();
        let run = solve(gp, strategy, steps);
        let p = run
            .evidence_probability
            .unwrap_or_else(|| panic!("{strategy:?}, {steps} steps: no P(o)"));
        let want: f64 = alpha(steps).iter().sum();
        assert!(
            (p.log10() - want.log10()).abs() < 1e-12,
            "{strategy:?}, {steps} steps: P(o) = {} against {want}",
            p.to_f64()
        );
        summing += run.query_sizes.summing_joins;
    }
    assert!(summing > 0, "no join summed out");
}
