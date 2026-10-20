//! Switch sharing (src/ground/share.rs) on a network whose tables repeat
//! parameters: the answers against an enumeration of its worlds under the
//! strategies and groundings `tests/it/examples.rs` runs, and which tables
//! share their switches.
//!
//! `a` has 40 values. `c` reads `a` alone and its rows exclude each other:
//! every odd row starts with two switches of 0.5, every even row with one
//! of 0.25 and one of 1/3, and each row ends with one of its own, so the
//! 120 switches of its rows are 44 facts shared. `e`, which `f` reads, has
//! a row per value of `a` too, each with one switch of 0.3 or 0.6: two
//! facts shared. `g` has three rows; `h` reads `a` and `x`; `k` has a row
//! per value of `a` whose switches merge one in 40; `m` has `c`'s rows
//! over `d`, a function of `a` and `b(1)` with one rule per value, and
//! only `m(1)` is asked, a switch per row: none of them shares. `f` is observed. ProbLog 2.3.0 gives the same answers to
//! its eight digits.

use crate::common;

use common::{Rows, assert_answers, milli};
use tddlog::Projection;
use tddlog::{Kind, Strategy};

const N: usize = 40;

fn a(i: usize) -> f64 {
    if i % 2 == 1 { milli(15) } else { milli(35) }
}

fn c(i: usize) -> [f64; 4] {
    if i % 2 == 1 {
        [0.5, 0.25, milli(3 * i), milli(250 - 3 * i)]
    } else {
        [0.25, 0.25, milli(4 * i), milli(500 - 4 * i)]
    }
}

fn e(i: usize) -> [f64; 2] {
    if i % 2 == 1 { [0.3, 0.7] } else { [0.6, 0.4] }
}

/// Every row of its own but the second, which repeats the first.
fn k(i: usize) -> [f64; 2] {
    let p = 100 + i.max(2) - 1;
    [milli(p), milli(1000 - p)]
}

const F: [f64; 2] = [0.9, 0.2];
const B: [f64; 3] = [0.2, 0.3, 0.5];
const G: [[f64; 3]; 3] = [[0.25, 0.25, 0.5], [0.25, 0.5, 0.25], [0.5, 0.25, 0.25]];
const X: f64 = 0.6;

fn h(i: usize, x: bool) -> [f64; 3] {
    let y = if x { 50 * i } else { 50 * i - 25 };
    [0.25, y as f64 / 10000.0, (7500 - y) as f64 / 10000.0]
}

fn heads(name: &str, ps: &[f64]) -> String {
    ps.iter()
        .enumerate()
        .map(|(v, p)| format!("{p}::{name}({})", v + 1))
        .collect::<Vec<_>>()
        .join("; ")
}

fn program() -> String {
    let mut s = String::new();
    s += &heads("a", &(1..=N).map(a).collect::<Vec<_>>());
    s += ".\n";
    s += &format!("{}.\n{X}::x.\n", heads("b", &B));
    for i in 1..=N {
        s += &format!("{} :- a({i}).\n", heads("c", &c(i)));
        s += &format!("{}::e(t); {}::e(f) :- a({i}).\n", e(i)[0], e(i)[1]);
        s += &format!("{} :- a({i}).\n", heads("k", &k(i)));
        s += &format!(
            "d({i}) :- a({i}), b(1).\n{} :- d({i}).\n",
            heads("m", &c(i))
        );
        s += &format!(
            "{} :- a({i}), x.\n{} :- a({i}), \\+x.\n",
            heads("h", &h(i, true)),
            heads("h", &h(i, false))
        );
    }
    s += &format!("{}::f :- e(t).\n{}::f :- e(f).\n", F[0], F[1]);
    for (j, g) in G.iter().enumerate() {
        s += &format!("{} :- b({}).\n", heads("g", g), j + 1);
    }
    s += "evidence(f).\n";
    for q in QUERIES {
        s += &format!("query({q}).\n");
    }
    s
}

const QUERIES: [&str; 14] = [
    "a(1)", "a(2)", "c(1)", "c(2)", "c(3)", "c(4)", "e(t)", "g(1)", "g(2)", "h(1)", "h(2)", "x",
    "k(1)", "m(1)",
];

/// The queries' probabilities given `f`, summed over the values of `a`,
/// `b`, `x` and `e`.
fn enumerated() -> Vec<(&'static str, f64)> {
    let mut joint = [0.0; QUERIES.len()];
    let mut evidence = 0.0;
    for i in 1..=N {
        for (j, b) in B.iter().enumerate() {
            for x in [true, false] {
                for (t, pe) in e(i).iter().enumerate() {
                    let w = a(i) * b * if x { X } else { 1.0 - X } * pe * F[t];
                    evidence += w;
                    let holds = [
                        if i == 1 { 1.0 } else { 0.0 },
                        if i == 2 { 1.0 } else { 0.0 },
                        c(i)[0],
                        c(i)[1],
                        c(i)[2],
                        c(i)[3],
                        if t == 0 { 1.0 } else { 0.0 },
                        G[j][0],
                        G[j][1],
                        h(i, x)[0],
                        h(i, x)[1],
                        if x { 1.0 } else { 0.0 },
                        k(i)[0],
                        if j == 0 { c(i)[0] } else { 0.0 },
                    ];
                    for (q, p) in joint.iter_mut().zip(holds) {
                        *q += w * p;
                    }
                }
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
fn shared_switches_keep_the_answers() {
    let source = program();
    let expected = enumerated();
    let check = |what: &str, run: Result<tddlog::Run, tddlog::Error>| {
        let run = run.unwrap_or_else(|e| panic!("{what}: {e}"));
        assert_answers(
            what,
            &run.results.expect("evidence is consistent"),
            &expected,
            1e-9,
        );
    };
    for strategy in [Strategy::Trial(Kind::Closed), Strategy::Auto] {
        for projection in [Projection::Off, Projection::default()] {
            check(
                &format!("{strategy:?}, {projection}"),
                common::run(&source, &common::options(strategy, projection)),
            );
        }
    }
    for strategy in [
        Strategy::Trial(Kind::Families { search: true }),
        Strategy::Trial(Kind::Families { search: false }),
    ] {
        check(
            &format!("{strategy:?}"),
            common::run(&source, &common::options(strategy, Projection::default())),
        );
    }
    for rows in [Rows::Apart, Rows::Classes] {
        for (indicators, strategy) in [
            (false, Strategy::Trial(Kind::Closed)),
            (false, Strategy::Auto),
            (true, Strategy::Trial(Kind::Families { search: true })),
            (true, Strategy::Trial(Kind::Closed)),
        ] {
            let ground = common::ground(&source, rows, indicators);
            check(
                &format!("{rows:?}, indicators {indicators}, {strategy:?}"),
                tddlog::solve_ground(ground, &common::options(strategy, Projection::default())),
            );
        }
        // The ground program as `--ground` prints it, its shared facts
        // read under exclusive contexts, has the same answers.
        let printed = common::ground(&source, rows, false).to_string();
        check(
            &format!("{rows:?}, printed ground"),
            common::run(
                &printed,
                &common::options(Strategy::Auto, Projection::default()),
            ),
        );
    }
}

#[test]
fn a_table_of_one_parent_and_many_rows_shares_its_switches() {
    let source = program();
    for rows in [Rows::Apart, Rows::Classes] {
        let ground = common::ground(&source, rows, false);
        let names: Vec<String> = ground.atoms.iter().map(|a| a.to_string()).collect();
        let switches = |table: &str| {
            names
                .iter()
                .filter(|a| a.starts_with(&format!("'$sw{table}_")))
                .count()
        };
        assert_eq!(switches("c"), 44, "{rows:?}: {names:?}");
        assert_eq!(switches("e"), 2, "{rows:?}: {names:?}");
        assert_eq!(switches("g"), 6, "{rows:?}: {names:?}");
        assert_eq!(switches("h"), 4 * N, "{rows:?}: {names:?}");
        assert_eq!(switches("k"), N, "{rows:?}: {names:?}");
        assert_eq!(switches("m"), N, "{rows:?}: {names:?}");
    }
}
