//! A clause table's readers tied to another reader's copies in place of
//! their links (`src/tp/tie.rs`): the links are not built, and the answers
//! are the network's, computed here by enumerating its worlds.

use crate::common;

use common::{Rows, assert_answers};
use tddlog::Projection;
use tddlog::{Kind, Strategy};

const VALUES: [&str; 4] = ["a", "b", "c", "d"];
const READERS: [&str; 4] = ["y", "z", "w", "v"];

/// Row `(i, j)` of `x`, given `p` = value i and `q` = value j: twentieths
/// that sum to one, no two rows alike.
fn row(i: usize, j: usize) -> [f64; 4] {
    let a = 1 + (i + 2 * j) % 5;
    let b = 1 + (3 * i + j) % 6;
    let c = 1 + (2 * i + 3 * j) % 4;
    [a, b, c, 20 - a - b - c].map(|n| n as f64 / 20.0)
}

/// Reader `k`'s probability of `t` given `x` = value i, `r` = value j and
/// its own parent `s{k}` = value m.
fn reads(k: usize, i: usize, j: usize, m: usize) -> f64 {
    (1 + (5 * k + 3 * i + 7 * j + 2 * m) % 9) as f64 / 10.0
}

/// Parent `s{k}`'s probability of value m.
fn own(k: usize, m: usize) -> f64 {
    [0.1, 0.2, 0.3, 0.4][(k + m) % 4]
}

/// A table `x` of four values over parents `p` and `q` of four values
/// each, 16 rows, read by four tables `y`, `z`, `w`, `v`, each over `x`,
/// `r` and a parent `s{k}` of its own, 32 rows: each of `x`'s readers'
/// links holds 106 clauses, among them a clause per row and value, and
/// each of `r`'s 4, which are built whatever their nodes. The readers'
/// own parents keep each of `x`'s links on under half the facts, which
/// projection needs.
fn program() -> String {
    let p = [0.1, 0.2, 0.3, 0.4];
    let q = [0.4, 0.3, 0.2, 0.1];
    let choice = |name: &str, probs: &[f64]| {
        probs
            .iter()
            .zip(VALUES)
            .map(|(pr, v)| format!("{pr}::{name}({v})"))
            .collect::<Vec<_>>()
            .join("; ")
    };
    let mut s = format!(
        "{}.\n{}.\n0.35::r(a); 0.65::r(b).\n",
        choice("p", &p),
        choice("q", &q)
    );
    for (i, pv) in VALUES.iter().enumerate() {
        for (j, qv) in VALUES.iter().enumerate() {
            s += &format!("{} :- p({pv}), q({qv}).\n", choice("x", &row(i, j)));
        }
    }
    for (k, name) in READERS.iter().enumerate() {
        s += &format!(
            "{}.\n",
            choice(
                &format!("s{k}"),
                &(0..4).map(|m| own(k, m)).collect::<Vec<_>>()
            )
        );
        for (i, xv) in VALUES.iter().enumerate() {
            for (j, rv) in ["a", "b"].iter().enumerate() {
                for (m, sv) in VALUES.iter().enumerate() {
                    let t = reads(k, i, j, m);
                    s += &format!(
                        "{t}::{name}(t); {}::{name}(f) :- x({xv}), r({rv}), s{k}({sv}).\n",
                        ((1.0 - t) * 1e9).round() / 1e9
                    );
                }
            }
        }
    }
    s += "evidence(y(t)).\nquery(z(t)). query(w(t)). query(v(t)). query(x(a)). query(p(b)).\n";
    s
}

/// The answers, summed over every world of `p`, `q`, `x`, `r` and the
/// readers' own parents, with the readers' tables in closed form.
fn expected() -> Vec<(&'static str, f64)> {
    let p = [0.1, 0.2, 0.3, 0.4];
    let q = [0.4, 0.3, 0.2, 0.1];
    let r = [0.35, 0.65];
    // Reader k's probability of t given x = value i and r = value j, its
    // own parent summed out.
    let given =
        |k: usize, i: usize, j: usize| (0..4).map(|m| own(k, m) * reads(k, i, j, m)).sum::<f64>();
    let (mut e, mut z, mut w, mut v, mut xa, mut pb) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for (i, pi) in p.iter().enumerate() {
        for (j, qj) in q.iter().enumerate() {
            for (k, xk) in row(i, j).iter().enumerate() {
                for (l, rl) in r.iter().enumerate() {
                    let joint = pi * qj * xk * rl * given(0, k, l);
                    e += joint;
                    z += joint * given(1, k, l);
                    w += joint * given(2, k, l);
                    v += joint * given(3, k, l);
                    xa += joint * f64::from(u8::from(k == 0));
                    pb += joint * f64::from(u8::from(i == 1));
                }
            }
        }
    }
    vec![
        ("z(t)", z / e),
        ("w(t)", w / e),
        ("v(t)", v / e),
        ("x(a)", xa / e),
        ("p(b)", pb / e),
    ]
}

#[test]
fn the_rows_of_x_are_apart() {
    let mut rows: Vec<String> = (0..16)
        .map(|n| format!("{:?}", row(n / 4, n % 4)))
        .collect();
    rows.sort();
    rows.dedup();
    assert_eq!(rows.len(), 16);
    assert!((0..16).all(|n| row(n / 4, n % 4).iter().all(|&pr| pr > 0.0)));
}

/// On the families layouts some of `x`'s readers are tied to another's
/// copies, `r`'s readers never; the answers are the network's either way.
#[test]
fn readers_tied_to_another_reader_s_copies_keep_the_answers() {
    let clauses = common::ground(&program(), Rows::Classes, true);
    let want = expected();
    for (strategy, tied) in [
        (Strategy::Trial(Kind::Families { search: false }), 3),
        (Strategy::Trial(Kind::Families { search: true }), 3),
    ] {
        let run = tddlog::solve_ground(
            clauses.clone(),
            &common::options(strategy, Projection::default()),
        )
        .unwrap_or_else(|e| panic!("{strategy}: {e}"));
        assert_eq!(run.stats.tied, tied, "{strategy}");
        assert_answers(
            &strategy.to_string(),
            &run.results.expect("evidence and queries"),
            &want,
            1e-12,
        );
    }
}

/// Without projection every link is at the root, and none is tied.
#[test]
fn without_projection_no_reader_is_tied() {
    let clauses = common::ground(&program(), Rows::Classes, true);
    let run = tddlog::solve_ground(
        clauses,
        &common::options(
            Strategy::Trial(Kind::Families { search: false }),
            Projection::Off,
        ),
    )
    .unwrap();
    assert_eq!(run.stats.tied, 0);
    assert_answers(
        "unprojected",
        &run.results.expect("evidence and queries"),
        &expected(),
        1e-12,
    );
}
