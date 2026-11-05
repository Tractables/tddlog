//! Circulette netlists (tests/circulette, written for these tests) read
//! through the library, against an enumeration of every assignment of the
//! faults: the probability of the failure, `P(e)`, and the posterior given
//! it of every single, pair and triple of faults.

use std::collections::BTreeMap;

use crate::common;
use tddlog::network::ReadOptions;
use tddlog::{Format, Kind, Options, Projection, Strategy};

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

/// A `bits`-bit ripple-carry adder in the netlist format, its sum's bits
/// and the carry out concatenated low bit first or high bit first. The
/// failure is any bit that is one, so the order of the concatenation is a
/// writing of the circuit and not a different circuit.
fn ripple_adder(bits: usize, high_first: bool) -> String {
    let (a, b) = (|i: usize| 3 + i, |i: usize| 3 + bits + i);
    let mut lines = Vec::new();
    for i in 0..bits {
        lines.push(format!("  ({} (ws 0 (list {i})))", a(i)));
        lines.push(format!("  ({} (ws 1 (list {i})))", b(i)));
    }
    let mut next = 3 + 2 * bits;
    let mut wire = |lines: &mut Vec<String>, gate: String| {
        lines.push(format!("  ({next} ({gate}))"));
        next += 1;
        next - 1
    };
    let mut out = Vec::new();
    let mut carry = None;
    for i in 0..bits {
        let half = wire(&mut lines, format!("w^ {} {}", a(i), b(i)));
        let both = wire(&mut lines, format!("w& {} {}", a(i), b(i)));
        carry = Some(match carry {
            None => {
                out.push(half);
                both
            }
            Some(c) => {
                out.push(wire(&mut lines, format!("w^ {half} {c}")));
                let through = wire(&mut lines, format!("w& {half} {c}"));
                wire(&mut lines, format!("w|| {both} {through}"))
            }
        });
    }
    out.extend(carry);
    if high_first {
        out.reverse();
    }
    let out: Vec<String> = out.iter().map(|w| w.to_string()).collect();
    format!(
        "(def-netlist adder\n  #hash((0 . {bits}) (1 . {bits}))\n  #hash((2 . {}))\n  #hash()\n{}\n  (2 (wc (list {}))))\n\n\
         (define p-clean 0.99)\n(define expected-output (list{}))\n",
        bits + 1,
        lines.join("\n"),
        out.join(" "),
        " #f".repeat(bits + 1),
    )
}

/// The largest formula of `text`'s compile under `strategy`, and its
/// answers.
fn compiled(text: &str, strategy: Strategy) -> (usize, Vec<(String, f64)>) {
    let input = tddlog::read(text.as_bytes(), Format::Circulette, &ReadOptions::default()).unwrap();
    let options = common::options(strategy, Projection::default());
    let run = common::run(&input.program, &options).unwrap();
    let answers = (run.results.as_deref().expect("consistent evidence").iter())
        .map(|r| (r.to_string(), r.probability))
        .collect();
    (run.stats.max_pairs, answers)
}

/// One adder written with its sum high bit first and low bit first
/// compiles alike, and small. The pairs and triples of faults the queries
/// ask tied every two faults together, which made one bag of every fault in
/// `td`'s decomposition and one block in `closed`'s; and the queries of
/// the faults kept the gates' atoms that the aliasing removes, numbered in
/// the order the grounder met them. At six bits the largest formulas were
/// of 2203 to 8328 pairs, and `td`'s two writings differed 3.8 times; at
/// eight bits one writing took 1.5 s and the other over 300 s. The queried
/// cubes now shape no layout and keep no atom (`QueriedCubes` in
/// `src/ground/shape.rs`, and the aliasing). `td` still breaks the ties of
/// its elimination by the atoms' numbers, and its two writings differ 1.5
/// times here.
#[test]
fn an_adder_compiles_alike_whichever_way_its_sum_is_listed() {
    let bounds = [
        (Strategy::Auto, (6, 5)),
        (Strategy::Trial(Kind::Closed), (6, 5)),
        (Strategy::Trial(Kind::Td), (2, 1)),
    ];
    for (strategy, (num, den)) in bounds {
        let (low, low_answers) = compiled(&ripple_adder(6, false), strategy);
        let (high, high_answers) = compiled(&ripple_adder(6, true), strategy);
        assert_eq!(low_answers.len(), high_answers.len(), "{strategy:?}");
        for ((a, p), (b, q)) in low_answers.iter().zip(&high_answers) {
            assert_eq!(a, b, "{strategy:?}");
            assert!((p - q).abs() < 1e-9, "{strategy:?}: {a} = {p} against {q}");
        }
        let (least, most) = (low.min(high), low.max(high));
        assert!(
            most <= 2000 && most * den <= least * num,
            "{strategy:?}: largest formulas of {low} and {high} pairs"
        );
    }
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
