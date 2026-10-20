//! A parent of many values that several units read (`read_in_binary` in
//! src/indicators/splits.rs): a table that holds no switch, or whose rows
//! read that parent alone, is not split on it and reads it in binary; a
//! table of switches over it and another parent is split on it.
//! The answers match an enumeration of the network's worlds.
//!
//! `x` has 40 values and `y` 4. `a` reads `x` alone, a row of switches per
//! value; `b` reads `x` and `y`, a row of switches per pair of values; `d`
//! reads `x` and `y` too, without switches: `d(k)` holds where
//! `(i + j) mod 3 + 1` is `k`. `e` reads `d`, so `b` and `d` are units of
//! their own, and `a(t)` and `e` are observed.

use crate::common;

use common::{Rows, assert_answers, milli};
use tddlog::Projection;
use tddlog::{Kind, Strategy};

const N: usize = 40;
const Y: [f64; 4] = [0.1, 0.2, 0.3, 0.4];
const E: [f64; 3] = [0.2, 0.5, 0.9];

/// The distribution of `x`.
fn x(i: usize) -> f64 {
    if i % 2 == 1 { milli(15) } else { milli(35) }
}

fn a(i: usize) -> [f64; 2] {
    let p = 5 + i;
    [milli(p), milli(1000 - p)]
}

/// The row of `b` over values `i` and `j`, numbered from 1: no two rows
/// have one distribution, so none is in a class.
fn b(i: usize, j: usize) -> [f64; 3] {
    let p = 100 + 4 * (i - 1) + j;
    [milli(p), milli(300), milli(700 - p)]
}

/// The value of `d` that `x(i)` and `y(j)` give.
fn d(i: usize, j: usize) -> usize {
    (i + j) % 3 + 1
}

fn program() -> String {
    let mut s = String::new();
    let xs: Vec<String> = (1..=N).map(|i| format!("{}::x({i})", x(i))).collect();
    let ys: Vec<String> = (1..=4).map(|j| format!("{}::y({j})", Y[j - 1])).collect();
    s += &format!("{}.\n{}.\n", xs.join("; "), ys.join("; "));
    for i in 1..=N {
        let [t, f] = a(i);
        s += &format!("{t}::a(t); {f}::a(f) :- x({i}).\n");
        for j in 1..=4 {
            let [b1, b2, b3] = b(i, j);
            s += &format!("{b1}::b(1); {b2}::b(2); {b3}::b(3) :- x({i}), y({j}).\n");
            s += &format!("d({}) :- x({i}), y({j}).\n", d(i, j));
        }
    }
    for (k, p) in E.iter().enumerate() {
        s += &format!("{p}::e :- d({}).\n", k + 1);
    }
    s += "evidence(a(t)).\nevidence(e).\n";
    for q in QUERIES {
        s += &format!("query({q}).\n");
    }
    s
}

const QUERIES: [&str; 8] = [
    "x(1)", "x(2)", "y(1)", "y(4)", "b(1)", "b(2)", "d(1)", "d(2)",
];

/// The queries' probabilities given `a(t)` and `e`, summed over the values
/// of `x` and `y`.
fn enumerated() -> Vec<(&'static str, f64)> {
    let mut joint = [0.0; QUERIES.len()];
    let mut evidence = 0.0;
    for i in 1..=N {
        for (j, y) in (1..=4).zip(Y) {
            let w = x(i) * y * a(i)[0] * E[d(i, j) - 1];
            evidence += w;
            let holds = [
                f64::from(u8::from(i == 1)),
                f64::from(u8::from(i == 2)),
                f64::from(u8::from(j == 1)),
                f64::from(u8::from(j == 4)),
                b(i, j)[0],
                b(i, j)[1],
                f64::from(u8::from(d(i, j) == 1)),
                f64::from(u8::from(d(i, j) == 2)),
            ];
            for (q, p) in joint.iter_mut().zip(holds) {
                *q += w * p;
            }
        }
    }
    QUERIES
        .iter()
        .zip(joint)
        .map(|(&q, p)| (q, p / evidence))
        .collect()
}

#[test]
fn a_parent_several_units_read_keeps_the_answers() {
    let source = program();
    let expected = enumerated();
    let check = |what: &str, run: Result<tddlog::Run, tddlog::Error>| {
        let run = run.unwrap_or_else(|e| panic!("{what}: {e}"));
        assert_answers(
            what,
            &run.results.expect("evidence is consistent"),
            &expected,
            1e-12,
        );
    };
    for strategy in [
        Strategy::Auto,
        Strategy::Trial(Kind::Closed),
        Strategy::Trial(Kind::Td),
    ] {
        check(
            &format!("{strategy:?}"),
            common::run(&source, &common::options(strategy, Projection::default())),
        );
    }
    for rows in [Rows::Apart, Rows::Classes] {
        for strategy in [
            Strategy::Trial(Kind::Families { search: true }),
            Strategy::Trial(Kind::Families { search: false }),
        ] {
            let ground = common::ground(&source, rows, true);
            check(
                &format!("{rows:?}, {strategy:?}"),
                tddlog::solve_ground(ground, &common::options(strategy, Projection::default())),
            );
        }
    }
}

/// The predicate of the unit an indicator's copy is read by: the second
/// argument of its name, `'$v'(atom, reader, part...)`, which a copy of an
/// atom that one unit reads leaves out.
fn reader(name: &str) -> &str {
    let second = name.split("','").nth(1).unwrap_or("");
    &second[..second.find('(').unwrap_or(second.len())]
}

#[test]
fn tables_without_switches_or_other_parents_read_the_parent_in_binary() {
    let source = program();
    for rows in [Rows::Apart, Rows::Classes] {
        let ground = common::ground(&source, rows, true);
        assert_eq!(
            ground.encoding.codes.len(),
            1,
            "{rows:?}: x is read in binary"
        );
        let name = |a: u32| ground.atoms[a as usize].to_string();
        let mut value_readers: Vec<String> = Vec::new();
        let mut bit_readers: Vec<String> = Vec::new();
        for ind in &ground.encoding.indicators {
            let atom = name(ind.atom);
            if atom.starts_with("x(") {
                value_readers.push(reader(&name(ind.var)).to_string());
            } else if atom.starts_with("'$b'(") && atom.contains("x(") {
                bit_readers.push(reader(&name(ind.var)).to_string());
            }
        }
        // `b` is split on `x`, a part per value, each reading its value
        // alone; `a` and `d` read the bits.
        assert_eq!(value_readers.len(), N, "{rows:?}: {value_readers:?}");
        assert!(
            value_readers.iter().all(|r| r == "b"),
            "{rows:?}: {value_readers:?}"
        );
        for unit in ["a", "d"] {
            assert!(
                bit_readers.iter().any(|r| r == unit),
                "{rows:?}: {unit} reads no bit of x: {bit_readers:?}"
            );
        }
        assert!(
            !bit_readers.iter().any(|r| r == "b"),
            "{rows:?}: {bit_readers:?}"
        );
    }
}
