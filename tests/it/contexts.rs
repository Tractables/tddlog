//! Choices read under exclusive contexts (src/ground/contexts.rs) on a
//! Caesar cipher: the answers against the posterior computed by hand, with
//! the choices read through a fact per context and as written, and which
//! choices are read so.
//!
//! A key `k` of eight values is drawn once; each of twelve steps draws a
//! character `d_i` and observes `(k + d_i) mod 8`, written as one rule per
//! value of the key reading the character it takes. The posterior of the
//! key is its prior times the probability of each step's character under
//! that key, normalized. Step 5's character is also asked, so its choice
//! stays as written.

use crate::common;

use common::assert_answers;
use tddlog::Projection;
use tddlog::ground::GroundOptions;
use tddlog::{Kind, Strategy};

const N: usize = 8;
const KEY: [f64; N] = [0.05, 0.2, 0.1, 0.15, 0.1, 0.25, 0.05, 0.1];
const CHAR: [f64; N] = [0.3, 0.1, 0.1, 0.1, 0.2, 0.05, 0.05, 0.1];
const OBSERVED: [usize; 12] = [3, 7, 3, 4, 0, 3, 5, 3, 2, 3, 7, 4];
const ASKED: usize = 5;

fn heads(name: &str, ps: &[f64]) -> String {
    ps.iter()
        .enumerate()
        .map(|(v, p)| format!("{p}::{name}({v})"))
        .collect::<Vec<_>>()
        .join("; ")
}

fn program() -> String {
    let mut s = format!("{}.\n", heads("k", &KEY));
    for (i, &o) in OBSERVED.iter().enumerate() {
        s += &format!("{}.\n", heads(&format!("d{i}"), &CHAR));
        for k in 0..N {
            s += &format!("e{i} :- k({k}), d{i}({}).\n", (o + N - k) % N);
        }
        s += &format!("evidence(e{i}).\n");
    }
    for k in 0..N {
        s += &format!("query(k({k})).\n");
    }
    s += &format!("query(d{ASKED}(0)).\n");
    s
}

/// The posterior of each value of the key, and of step `ASKED`'s character
/// being 0.
fn by_hand() -> Vec<(String, f64)> {
    let joint: Vec<f64> = (0..N)
        .map(|k| {
            KEY[k]
                * OBSERVED
                    .iter()
                    .map(|&o| CHAR[(o + N - k) % N])
                    .product::<f64>()
        })
        .collect();
    let z: f64 = joint.iter().sum();
    let mut answers: Vec<(String, f64)> =
        (0..N).map(|k| (format!("k({k})"), joint[k] / z)).collect();
    // d_ASKED is 0 exactly where the key is OBSERVED[ASKED].
    answers.push((format!("d{ASKED}(0)"), joint[OBSERVED[ASKED]] / z));
    answers
}

fn ground(contexts: bool) -> tddlog::GroundProgram {
    let program = tddlog::syntax::parse_program(&program()).unwrap();
    tddlog::ground::ground(
        &program,
        &GroundOptions {
            contexts,
            ..GroundOptions::default()
        },
    )
    .unwrap()
}

#[test]
fn a_cipher_read_through_a_fact_per_key_keeps_its_posterior() {
    let source = program();
    let expected = by_hand();
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
        Strategy::Trial(Kind::Closed),
        Strategy::Auto,
        Strategy::Trial(Kind::Families { search: true }),
    ] {
        check(
            &format!("{strategy:?}"),
            common::run(&source, &common::options(strategy, Projection::default())),
        );
    }
    for contexts in [true, false] {
        for strategy in [
            Strategy::Trial(Kind::Closed),
            Strategy::Trial(Kind::Families { search: true }),
        ] {
            check(
                &format!("contexts {contexts}, {strategy:?}"),
                tddlog::solve_ground(
                    ground(contexts),
                    &common::options(strategy, Projection::default()),
                ),
            );
        }
    }
    // The ground program as `--ground` prints it, with the facts made in
    // place of the choices, has the same answers.
    let printed = ground(true).to_string();
    check(
        "printed ground",
        common::run(
            &printed,
            &common::options(Strategy::Auto, Projection::default()),
        ),
    );
}

#[test]
fn every_choice_but_the_asked_one_is_a_fact_per_key() {
    let gp = ground(true);
    assert_eq!(gp.stats.contexts, (OBSERVED.len() - 1) * N, "{gp}");
    let names: Vec<String> = gp.atoms.iter().map(|a| a.to_string()).collect();
    assert!(
        names.iter().any(|a| a.starts_with(&format!("d{ASKED}("))),
        "{names:?}"
    );
    assert!(!names.iter().any(|a| a.starts_with("d0(")), "{names:?}");
    assert_eq!(ground(false).stats.contexts, 0);
}
