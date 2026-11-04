//! Circulette netlists (tests/circulette, written for these tests) read
//! through the library, against an enumeration of every assignment of the
//! faults: the probability of the failure, `P(e)`, and the posterior given
//! it of every single, pair and triple of faults.

use std::collections::BTreeMap;

use tddlog::network::ReadOptions;
use tddlog::{Format, Options};

/// A fault's probability, `1 - p-clean` in floating point.
const FAULT: f64 = 1.0 - 0.99;

/// tddlog's answers for the netlist at `path`, and its `P(e)`.
fn solve(path: &str) -> (BTreeMap<String, f64>, f64) {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let format = Format::of_path(path);
    assert_eq!(format, Format::Circulette);
    let input = tddlog::read(&data, format, &ReadOptions::default()).unwrap();
    let options = Options {
        evidence_probability: true,
        ..Options::default()
    };
    let solution = tddlog::solve(&input.program, &options, &mut |_| {}).unwrap();
    let answers = solution
        .answers
        .as_deref()
        .expect("consistent evidence")
        .iter()
        .map(|a| (a.to_string(), a.probability))
        .collect();
    (answers, input.evidence_probability(&solution).unwrap().0)
}

/// The posteriors of every single, pair and triple of the faults of
/// `gates` (in increasing order) given the failure, and `P(fail)`, by
/// enumeration: `fails` says whether the circuit fails where the faults
/// that are true are.
fn enumerate(gates: &[u64], fails: impl Fn(&[bool]) -> bool) -> (BTreeMap<String, f64>, f64) {
    let n = gates.len();
    let mut keys = Vec::new();
    for a in 0..n {
        keys.push(vec![a]);
        for b in a + 1..n {
            keys.push(vec![a, b]);
            for c in b + 1..n {
                keys.push(vec![a, b, c]);
            }
        }
    }
    let mut joint = vec![0.0; keys.len()];
    let mut p_fail = 0.0;
    for world in 0..1u32 << n {
        let f: Vec<bool> = (0..n).map(|i| world >> i & 1 == 1).collect();
        if !fails(&f) {
            continue;
        }
        let p: f64 = f
            .iter()
            .map(|&x| if x { FAULT } else { 1.0 - FAULT })
            .product();
        p_fail += p;
        for (key, j) in keys.iter().zip(&mut joint) {
            if key.iter().all(|&i| f[i]) {
                *j += p;
            }
        }
    }
    let answers = keys
        .iter()
        .zip(joint)
        .map(|(key, j)| {
            let ids: Vec<String> = key.iter().map(|&i| gates[i].to_string()).collect();
            let name = if key.len() == 1 { "fault" } else { "faults" };
            (format!("{name}({})", ids.join(",")), j / p_fail)
        })
        .collect();
    (answers, p_fail)
}

fn check(path: &str, gates: &[u64], fails: impl Fn(&[bool]) -> bool) {
    let (got, p) = solve(path);
    let (expected, q) = enumerate(gates, fails);
    assert!((p - q).abs() < 1e-12, "{path}: P(e) = {p} against {q}");
    let n = gates.len();
    assert_eq!(
        got.len(),
        n + n * (n - 1) / 2 + n * (n - 1) * (n - 2) / 6,
        "{path}"
    );
    assert_eq!(
        got.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>(),
        "{path}"
    );
    for (atom, q) in &expected {
        let p = got[atom];
        assert!((p - q).abs() < 1e-9, "{path}: {atom} = {p} against {q}");
    }
}

/// A majority voter: every gate's fault reaches the output through the
/// gates after it, the conjunctions' masking one another.
#[test]
fn a_majority_voter() {
    check("tests/circulette/voter.cql", &[4, 5, 6, 7, 8], |f| {
        f[4] ^ (((f[0] | f[1]) ^ f[3]) | f[2])
    });
}

/// A two-bit adder, its operands' bits selected from words and its sum
/// a concatenation of three bits.
#[test]
fn a_two_bit_adder() {
    check(
        "tests/circulette/adder2.cql",
        &[7, 8, 9, 10, 11, 12, 13],
        |f| {
            let (s0, c0, t) = (f[0], f[1], f[2]);
            let s1 = t ^ c0 ^ f[3];
            let c1 = (f[4] | ((t & c0) ^ f[5])) ^ f[6];
            s0 | s1 | c1
        },
    );
}

/// A counter: the register's next value, the count up where the enable
/// (gate 12) is false and the register where it is true, is observed; its
/// low bit is a negation of the register's false bit, which is one but
/// where its gate is faulty. Gate 11 reaches no output and is no fault.
#[test]
fn a_counter_with_a_register() {
    check("tests/circulette/counter.cql", &[5, 6, 9, 12], |f| {
        f[2] | (!f[3] & (!f[0] | f[1]))
    });
}
