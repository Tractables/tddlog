//! UW-CSE under the `families` layout, whose hubs and branches keep the
//! students apart: every student's advising reads the professor's
//! position, and the professor's teaching and publications are
//! disjunctions over the students. Against ProbLog's answer, and for more
//! students than ProbLog answers quickly, against `closed`.
use crate::common;

use common::Rows;
use tddlog::Projection;
use tddlog::{Kind, Strategy};

/// examples/uwcse.pl with `extra` students besides its two.
fn uwcse(extra: usize) -> String {
    let more: String = (3..3 + extra)
        .map(|i| format!("\nstudent(s{i})."))
        .collect();
    std::fs::read_to_string("examples/uwcse.pl")
        .unwrap()
        .replace("student(s2).", &format!("student(s2).{more}"))
}

/// The run of `source` on `families:0` with every indicator: its answer and
/// the evidence formula's pairs.
fn families(source: &str) -> (f64, usize) {
    let gp = common::ground(source, Rows::Classes, true);
    let run = tddlog::solve_ground(
        gp,
        &common::options(
            Strategy::Trial(Kind::Families { search: false }),
            Projection::default(),
        ),
    )
    .unwrap();
    (
        run.results.unwrap()[0].probability,
        run.query_sizes.evidence,
    )
}

#[test]
fn four_students_against_problog() {
    // ProbLog: 0.094006751. With the position in the advising families'
    // group, every reader of any student's advising would read all four,
    // and the evidence would hold about 181 k pairs.
    let (p, evidence) = families(&uwcse(2));
    assert!((p - 0.094006751).abs() < 1e-8, "{p}");
    assert!(evidence < 20_000, "evidence {evidence} pairs");
}

#[test]
fn six_students_against_closed() {
    // With the students' families joined, the evidence would hold about
    // 11.6 M pairs.
    let source = uwcse(4);
    let (p, evidence) = families(&source);
    let closed = common::run(
        &source,
        &common::options(Strategy::Trial(Kind::Closed), Projection::Off),
    )
    .unwrap()
    .results
    .unwrap()[0]
        .probability;
    assert!((p - closed).abs() < 1e-12, "{p} against {closed}");
    assert!(evidence < 60_000, "evidence {evidence} pairs");
}
