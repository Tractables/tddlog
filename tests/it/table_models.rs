//! Links of a network's tables built from their models (`src/tp/rows.rs`):
//! the loop that ties a table's classes to its parents' copies, or a
//! reader's copies to the classes, holds where each group of copies holds
//! one value and the output the value the inputs force, and tididi builds
//! that set of assignments in one pass. These tests check that such loops
//! are built so and that the answers are the networks', computed here by
//! enumerating their worlds.

use crate::common;

use common::{Rows, assert_answers};
use tddlog::Projection;
use tddlog::{Kind, Strategy};

/// The program's answers under the families vtree, and how many loops were
/// built from their models.
fn families(source: &str) -> (usize, Vec<tddlog::QueryResult>) {
    let clauses = common::ground(source, Rows::Classes, true);
    let run = tddlog::solve_ground(
        clauses,
        &common::options(
            Strategy::Trial(Kind::Families { search: true }),
            Projection::default(),
        ),
    )
    .unwrap_or_else(|e| panic!("families: {e}"));
    (
        run.stats.clause_memo.from_models,
        run.results.expect("evidence and queries"),
    )
}

/// A table `x` over parents `p` and `q`, read by `y` with a parent `r` of
/// its own, given `y(t)`.
#[test]
fn a_table_over_its_parents_copies_is_built_from_its_rows() {
    let p = [0.2, 0.5, 0.3];
    let q = [0.6, 0.4];
    let x = [
        [0.1, 0.3, 0.6],
        [0.4, 0.4, 0.2],
        [0.7, 0.2, 0.1],
        [0.2, 0.5, 0.3],
        [0.3, 0.3, 0.4],
        [0.5, 0.1, 0.4],
    ];
    let r = [0.35, 0.65];
    let y = [0.9, 0.4, 0.2, 0.3, 0.8, 0.5];
    let mut s = String::from(
        "0.2::p(a); 0.5::p(b); 0.3::p(c).\n0.6::q(a); 0.4::q(b).\n0.35::r(a); 0.65::r(b).\n",
    );
    for (i, pv) in ["a", "b", "c"].iter().enumerate() {
        for (j, qv) in ["a", "b"].iter().enumerate() {
            let [a, b, c] = x[2 * i + j];
            s += &format!("{a}::x(a); {b}::x(b); {c}::x(c) :- p({pv}), q({qv}).\n");
        }
    }
    for (k, xv) in ["a", "b", "c"].iter().enumerate() {
        for (a, rv) in ["a", "b"].iter().enumerate() {
            let t = y[2 * k + a];
            s += &format!("{t}::y(t); {}::y(f) :- x({xv}), r({rv}).\n", 1.0 - t);
        }
    }
    s += "evidence(y(t)).\nquery(x(a)). query(p(b)). query(q(b)).\n";
    let (mut e, mut xa, mut pb, mut qb) = (0.0, 0.0, 0.0, 0.0);
    for (i, pi) in p.iter().enumerate() {
        for (j, qj) in q.iter().enumerate() {
            for (k, xk) in x[2 * i + j].iter().enumerate() {
                for (a, ra) in r.iter().enumerate() {
                    let joint = pi * qj * xk * ra * y[2 * k + a];
                    e += joint;
                    xa += joint * f64::from(u8::from(k == 0));
                    pb += joint * f64::from(u8::from(i == 1));
                    qb += joint * f64::from(u8::from(j == 1));
                }
            }
        }
    }
    let (models, got) = families(&s);
    assert!(models > 0, "no loop built from its models");
    assert_answers(
        "families",
        &got,
        &[("x(a)", xa / e), ("p(b)", pb / e), ("q(b)", qb / e)],
        1e-12,
    );
}
