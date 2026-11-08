//! A table of a distribution per row on long spines (src/ground/classes.rs).
//! Where its parents are binary, no split takes its spine apart, and it is
//! read through its row classes however many classes it has: a class atom
//! per row, the table over them split into parts side by side. Where a
//! parent has three values, a split on it takes the spine apart, and a
//! table of more classes than the bound keeps its rows, as does a table
//! whose rows share no parent. The answers match an enumeration of the
//! network's worlds either way.
//!
//! `c` has `parents` parents `p1`, `p2` and so on of `arity` values each,
//! and a probability of its own for each of their rows; `g` reads `c` and
//! is observed.

use crate::common;

use common::{Rows, assert_answers};
use tddlog::Projection;
use tddlog::{Kind, Strategy};

const G: [f64; 2] = [0.8, 0.3];

struct Network {
    arity: usize,
    parents: usize,
}

impl Network {
    fn rows(&self) -> usize {
        self.arity.pow(self.parents as u32)
    }

    /// The value of `p{i}`, `i` from 1, in `row`: its digit in base
    /// `arity`, `p1`'s the lowest.
    fn value(&self, row: usize, i: usize) -> usize {
        row / self.arity.pow(i as u32 - 1) % self.arity
    }

    fn name(&self, v: usize) -> String {
        match self.arity {
            2 => ["f", "t"][v].to_string(),
            _ => format!("v{v}"),
        }
    }

    /// The probability that `p{i}` takes the value `v`: for binary parents
    /// one of its own, otherwise a quarter, a quarter and a half, turned
    /// by `i`, which add up to one exactly.
    fn parent(&self, i: usize, v: usize) -> f64 {
        match self.arity {
            2 => {
                let t = 0.1 + 0.08 * i as f64;
                if v == 1 { t } else { 1.0 - t }
            }
            3 => [0.25, 0.25, 0.5][(v + i) % 3],
            _ => unreachable!("binary and ternary parents only"),
        }
    }

    /// The probability of `c(t)` in `row`: no two rows have one.
    fn c(&self, row: usize) -> f64 {
        (1 + row) as f64 / (2 * self.rows()) as f64 + 0.25
    }

    fn last(&self) -> String {
        self.name(self.arity - 1)
    }

    fn program(&self) -> String {
        let mut s = String::new();
        for i in 1..=self.parents {
            let values: Vec<String> = (0..self.arity)
                .map(|v| format!("{}::p{i}({})", self.parent(i, v), self.name(v)))
                .collect();
            s += &format!("{}.\n", values.join("; "));
        }
        for row in 0..self.rows() {
            let body: Vec<String> = (1..=self.parents)
                .map(|i| format!("p{i}({})", self.name(self.value(row, i))))
                .collect();
            let p = self.c(row);
            s += &format!("{p}::c(t); {}::c(f) :- {}.\n", 1.0 - p, body.join(", "));
        }
        s += &format!("{}::g :- c(t).\n{}::g :- c(f).\n", G[0], G[1]);
        s += &format!(
            "evidence(g).\nquery(c(t)).\nquery(p1({last})).\nquery(p{}({last})).\n",
            self.parents,
            last = self.last()
        );
        s
    }

    /// The queries' probabilities given `g`, summed over the parents'
    /// values and `c`'s.
    fn enumerated(&self) -> Vec<(String, f64)> {
        let (mut evidence, mut c_t, mut first, mut final_) = (0.0, 0.0, 0.0, 0.0);
        for row in 0..self.rows() {
            let prior: f64 = (1..=self.parents)
                .map(|i| self.parent(i, self.value(row, i)))
                .product();
            let t = prior * self.c(row) * G[0];
            let w = t + prior * (1.0 - self.c(row)) * G[1];
            evidence += w;
            c_t += t;
            if self.value(row, 1) == self.arity - 1 {
                first += w;
            }
            if self.value(row, self.parents) == self.arity - 1 {
                final_ += w;
            }
        }
        vec![
            ("c(t)".to_string(), c_t / evidence),
            (format!("p1({})", self.last()), first / evidence),
            (
                format!("p{}({})", self.parents, self.last()),
                final_ / evidence,
            ),
        ]
    }

    /// The class atoms of the program as grounded with row classes, and
    /// the answers under auto and under the families vtree, the rows
    /// apart and in classes, checked against the enumeration.
    fn check(&self) -> usize {
        let source = self.program();
        let classes = common::ground(&source, Rows::Classes, false);
        let class_atoms = classes
            .atoms
            .iter()
            .filter(|a| a.name.starts_with("$c_"))
            .count();
        let expected = self.enumerated();
        let check = |what: &str, run: Result<tddlog::Run, tddlog::Error>| {
            let run = run.unwrap_or_else(|e| panic!("{what}: {e}"));
            assert_answers(
                what,
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-12,
            );
        };
        check(
            "auto",
            common::run(
                &source,
                &common::options(Strategy::Auto, Projection::default()),
            ),
        );
        for rows in [Rows::Apart, Rows::Classes] {
            let ground = common::ground(&source, rows, true);
            check(
                &format!("{rows:?}"),
                tddlog::solve_ground(
                    ground,
                    &common::options(
                        Strategy::Trial(Kind::Families { search: false }),
                        Projection::default(),
                    ),
                ),
            );
        }
        class_atoms
    }
}

#[test]
fn a_table_on_binary_parents_reads_a_class_per_row_and_keeps_the_answers() {
    let network = Network {
        arity: 2,
        parents: 9,
    };
    assert_eq!(network.check(), network.rows(), "a class atom per row of c");
}

#[test]
fn a_table_split_on_a_parent_of_three_values_keeps_its_rows_past_the_bound() {
    let network = Network {
        arity: 3,
        parents: 6,
    };
    assert_eq!(network.check(), 0, "c's 729 rows kept as they are");
}

/// `s` reads `a0` and `q`, and `a{i}` and its own fact `t` for each other
/// `i` below `n`; `r` reads `q` and `a0` too, and `u` each other `a{i}`,
/// which are then not `s`'s own facts. No two of `s`'s rows clash, so
/// each would make a class of its own, a copy of the row, and no parent is
/// read by every row: past the bound the table keeps its rows.
#[test]
fn a_table_whose_rows_share_no_parent_keeps_its_rows_past_the_bound() {
    let n: i32 = 600;
    let mut source: String = (0..n).map(|i| format!("0.5::a{i}.\n")).collect();
    source += "0.3::q.\n0.6::t.\nr :- q, a0.\ns :- a0, q.\n";
    for i in 1..n {
        source += &format!("s :- a{i}, t.\nu :- a{i}.\n");
    }
    source += "query(r).\nquery(s).\nquery(u).\n";
    let classes = common::ground(&source, Rows::Classes, false);
    assert!(
        classes.atoms.iter().all(|a| !a.name.starts_with("$c_")),
        "s's {n} rows kept as they are"
    );
    // u: some a{i} past a0; s: r, or t and u.
    let u = 1.0 - 0.5f64.powi(n - 1);
    let s = 1.0 - 0.85 * (1.0 - 0.6 * u);
    let run = common::run(
        &source,
        &common::options(Strategy::Auto, Projection::default()),
    )
    .unwrap();
    assert_answers(
        "auto",
        &run.results.expect("no evidence"),
        &[
            ("r".to_string(), 0.15),
            ("s".to_string(), s),
            ("u".to_string(), u),
        ],
        1e-12,
    );
}
