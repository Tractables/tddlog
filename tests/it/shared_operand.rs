//! Two ripple-carry sums that share an operand, as conjugate_gaussians
//! adds one mean to two noises: `s = a + b` observed bit by bit and
//! `t = a + c` queried bit by bit, the bits modulo `2^WIDTH`. Under
//! `program`, under `program:fanin`, which lays the three operands' bits
//! out interleaved (src/ground/deps.rs, `fan_in`), and under `auto`, the
//! answers and P(e) equal an enumeration of the operands' values.

use crate::common;

use tddlog::{Kind, Options, Projection, Strategy};

const WIDTH: usize = 4;

/// The strategies the program is run under.
const STRATEGIES: [Strategy; 3] = [
    Strategy::Trial(Kind::Program { fan_in: false }),
    Strategy::Trial(Kind::Program { fan_in: true }),
    Strategy::Auto,
];

/// Each operand's bits' probabilities, `a`, `b` and `c` in turn, low bit
/// first, in thousandths.
fn weights() -> [[usize; WIDTH]; 3] {
    let mut rng = common::Lcg(17);
    let mut w = [[0; WIDTH]; 3];
    for x in w.iter_mut().flatten() {
        *x = 100 + (rng.next() * 800.0) as usize;
    }
    w
}

/// The program: the facts, the two sums' rules and carries, the evidence
/// that `s` is `observed` and a query of each bit of `t`.
fn program(w: &[[usize; WIDTH]; 3], observed: usize) -> String {
    let mut src = String::new();
    for (x, bits) in ["a", "b", "c"].iter().zip(w) {
        for (i, &k) in bits.iter().enumerate() {
            src += &format!("{}::{x}{i}.\n", common::milli(k));
        }
    }
    for (s, k, y) in [("s", "k", "b"), ("t", "j", "c")] {
        src += &format!("{s}0 :- a0, \\+{y}0. {s}0 :- \\+a0, {y}0. {k}0 :- a0, {y}0.\n");
        for i in 1..WIDTH {
            let p = i - 1;
            src += &format!(
                "{s}{i} :- a{i}, \\+{y}{i}, \\+{k}{p}. {s}{i} :- \\+a{i}, {y}{i}, \\+{k}{p}.\n\
                 {s}{i} :- \\+a{i}, \\+{y}{i}, {k}{p}. {s}{i} :- a{i}, {y}{i}, {k}{p}.\n\
                 {k}{i} :- a{i}, {y}{i}. {k}{i} :- a{i}, {k}{p}. {k}{i} :- {y}{i}, {k}{p}.\n"
            );
        }
    }
    for i in 0..WIDTH {
        let v = if observed >> i & 1 == 1 {
            "true"
        } else {
            "false"
        };
        src += &format!("evidence(s{i}, {v}).\n");
    }
    for i in 0..WIDTH {
        src += &format!("query(t{i}).\n");
    }
    src
}

/// P(t_i | s = observed) for each bit, and P(s = observed), by
/// enumeration.
fn exact(w: &[[usize; WIDTH]; 3], observed: usize) -> (Vec<f64>, f64) {
    let values = 1 << WIDTH;
    let p = |x: usize, v: usize| -> f64 {
        (0..WIDTH)
            .map(|i| {
                let q = common::milli(w[x][i]);
                if v >> i & 1 == 1 { q } else { 1.0 - q }
            })
            .product()
    };
    let (mut evidence, mut bits) = (0.0, vec![0.0; WIDTH]);
    for a in 0..values {
        for b in 0..values {
            if (a + b) % values != observed {
                continue;
            }
            for c in 0..values {
                let q = p(0, a) * p(1, b) * p(2, c);
                evidence += q;
                let t = (a + c) % values;
                for (i, bit) in bits.iter_mut().enumerate() {
                    if t >> i & 1 == 1 {
                        *bit += q;
                    }
                }
            }
        }
    }
    (bits.iter().map(|b| b / evidence).collect(), evidence)
}

#[test]
fn two_sums_sharing_an_operand_are_the_enumeration() {
    let w = weights();
    for observed in [0, 5, 11] {
        let source = program(&w, observed);
        let (want, want_evidence) = exact(&w, observed);
        let expected: Vec<(String, f64)> = want
            .iter()
            .enumerate()
            .map(|(i, &p)| (format!("t{i}"), p))
            .collect();
        for strategy in STRATEGIES {
            let what = format!("s = {observed}, {strategy:?}");
            let options = common::options(strategy, Projection::default());
            let run = common::run(&source, &options).unwrap_or_else(|e| panic!("{what}: {e}"));
            let results = run.results.as_ref().expect("consistent evidence");
            common::assert_answers(&what, results, &expected, 1e-12);
            let options = Options {
                evidence_probability: true,
                ..options
            };
            let run = common::run(&source, &options).unwrap_or_else(|e| panic!("{what}: {e}"));
            let p = run.evidence_probability.expect("P(e)").to_f64();
            assert!(
                (p - want_evidence).abs() < 1e-12 * want_evidence,
                "{what}: P(e) = {p} against {want_evidence}"
            );
        }
    }
}
