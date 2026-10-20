//! The rows that the conditioning leaves fewer heads, renormalized over
//! the heads they keep, their mass on a fact their context requires
//! (src/ground/renormalize.rs). The answers and P(e) against
//! an enumeration of the worlds, under the strategies and groundings, with
//! the renormalization on (the default) and off, and what the rewrite
//! leaves.
//!
//! `a` (`K` values) is a root, `c` (three values) its child, `x` and `d`
//! children of `c`; `x(t)` is observed, and has probability zero where `c`
//! is 3, and `c(3)` is observed false. `c` is 3 wherever `a` is `K`, so
//! the conditioning makes `a(K)` false. The row of `a` keeps `K - 1` heads
//! and its mass, a constant; the rows of `c` under the other values of
//! `a` keep two heads and the masses `0.9 - i / 100`, each on a fact that
//! its context requires. With three values of `a` the masses are observed
//! through an atom of their own; with 32, a set read in binary, through
//! `c(3)`, the false head of each row, with the mass first in the chain of
//! the heads it keeps. `x(f)`, queried, is made false: the rows of `x`
//! keep `x(t)`, made true, and each reads one fact of `x(t)`'s probability
//! instead.

use crate::common;

use common::{Rows, assert_answers};
use tddlog::Projection;
use tddlog::ground::GroundOptions;
use tddlog::{Kind, Strategy};

/// P(x = t | c).
const X: [f64; 3] = [0.7, 0.4, 0.0];
/// P(d = 1 | c).
const D: [f64; 3] = [0.9, 0.4, 0.5];

/// The network with `k` values of `a`: its distributions and queries.
struct Net {
    k: usize,
    a: Vec<f64>,
    c: Vec<[f64; 3]>,
    queries: Vec<String>,
}

impl Net {
    fn new(k: usize) -> Net {
        let total = (k * (k + 1) / 2) as f64;
        let a = (0..k).map(|i| (i + 1) as f64 / total).collect();
        let c = (0..k)
            .map(|i| {
                if i + 1 == k {
                    [0.0, 0.0, 1.0]
                } else {
                    let (c1, c2) = ((60 - i) as f64 / 100.0, 0.3);
                    [c1, c2, 1.0 - c1 - c2]
                }
            })
            .collect();
        let queries = ["a(1)".to_string(), "a(2)".to_string(), format!("a({k})")]
            .into_iter()
            .chain(["c(1)", "c(2)", "c(3)", "d(1)", "x(f)"].map(String::from))
            .collect();
        Net { k, a, c, queries }
    }

    fn program(&self) -> String {
        let heads = |name: &str, ps: &[f64]| {
            let heads: Vec<String> = ps
                .iter()
                .enumerate()
                .filter(|&(_, &p)| p > 0.0)
                .map(|(v, p)| format!("{p}::{name}({})", v + 1))
                .collect();
            heads.join("; ")
        };
        let mut s = String::new();
        s += &format!("{}.\n", heads("a", &self.a));
        for (i, row) in self.c.iter().enumerate() {
            s += &format!("{} :- a({}).\n", heads("c", row), i + 1);
        }
        for (i, &t) in X.iter().enumerate() {
            let i = i + 1;
            s += &if t == 0.0 {
                format!("x(f) :- c({i}).\n")
            } else {
                format!("{t}::x(t); {}::x(f) :- c({i}).\n", 1.0 - t)
            };
            s += &format!("{}::d(1); {}::d(2) :- c({i}).\n", D[i - 1], 1.0 - D[i - 1]);
        }
        s += "evidence(x(t)).\nevidence(c(3), false).\n";
        for q in &self.queries {
            s += &format!("query({q}).\n");
        }
        s
    }

    /// P(e) and the queries' probabilities given the evidence, summed over
    /// `a` and `c`.
    fn enumerated(&self) -> (f64, Vec<(String, f64)>) {
        let n = self.queries.len();
        let mut joint = vec![0.0; n];
        let mut evidence = 0.0;
        for (i, pa) in self.a.iter().enumerate() {
            for (j, pc) in self.c[i].iter().enumerate() {
                if j == 2 {
                    continue;
                }
                let w = pa * pc * X[j];
                evidence += w;
                let mut holds = vec![0.0; n];
                if i == 0 {
                    holds[0] = 1.0;
                }
                if i == 1 {
                    holds[1] = 1.0;
                }
                if i + 1 == self.k {
                    holds[2] = 1.0;
                }
                holds[3 + j] = 1.0;
                holds[6] = D[j];
                for (q, p) in joint.iter_mut().zip(holds) {
                    *q += w * p;
                }
            }
        }
        (
            evidence,
            self.queries
                .iter()
                .cloned()
                .zip(joint)
                .map(|(q, p)| (q, p / evidence))
                .collect(),
        )
    }
}

#[test]
fn the_rows_renormalized_keep_the_answers_and_the_evidence_probability() {
    for k in [3, 32] {
        let net = Net::new(k);
        let source = net.program();
        let (pe, expected) = net.enumerated();
        // P(e) where the run grounds the program itself: a ground program
        // printed leaves out the factor its conditioning took out.
        let check = |what: &str, run: Result<tddlog::Run, tddlog::Error>, count: bool| {
            let run = run.unwrap_or_else(|e| panic!("{k} values, {what}: {e}"));
            assert_answers(
                &format!("{k} values, {what}"),
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-9,
            );
            if count && let Some(got) = run.evidence_probability {
                let got = got.to_f64();
                assert!(
                    (got - pe).abs() <= 1e-12 * pe,
                    "{k} values, {what}: P(e) = {got} against {pe}"
                );
            }
        };
        for strategy in [
            Strategy::Trial(Kind::Closed),
            Strategy::Auto,
            Strategy::Trial(Kind::Families { search: false }),
        ] {
            for projection in [Projection::Off, Projection::default()] {
                check(
                    &format!("{strategy:?}, {projection}"),
                    common::run(&source, &common::options(strategy, projection)),
                    true,
                );
            }
        }
        let program = tddlog::syntax::parse_program(&source).unwrap();
        for rows in [Rows::Apart, Rows::Classes] {
            for (indicators, strategy) in [
                (false, Strategy::Trial(Kind::Closed)),
                (true, Strategy::Trial(Kind::Families { search: true })),
            ] {
                let ground = common::ground(&source, rows, indicators);
                check(
                    &format!("{rows:?}, indicators {indicators}, {strategy:?}"),
                    tddlog::solve_ground(ground, &common::options(strategy, Projection::default())),
                    true,
                );
                // P(e), nothing detached, with the renormalization and
                // without.
                for renormalize in [true, false] {
                    let options = GroundOptions {
                        detach: false,
                        renormalize,
                        ..rows.options()
                    };
                    let ground = tddlog::ground::ground(&program, &options).unwrap();
                    assert_eq!(
                        ground.stats.conditioned.renormalized > 0,
                        renormalize,
                        "{k} values, {rows:?}, renormalize {renormalize}: {} rows renormalized",
                        ground.stats.conditioned.renormalized
                    );
                    let run = tddlog::solve_ground(
                        ground,
                        &common::options(strategy, Projection::default()),
                    )
                    .unwrap();
                    assert_answers(
                        &format!(
                            "{k} values, {rows:?}, indicators {indicators}, {strategy:?}, renormalize {renormalize}"
                        ),
                        &run.results.expect("evidence is consistent"),
                        &expected,
                        1e-9,
                    );
                    let got = run.evidence_probability.expect("P(e)").to_f64();
                    assert!(
                        (got - pe).abs() <= 1e-12 * pe,
                        "{k} values, {rows:?}, indicators {indicators}, {strategy:?}, renormalize {renormalize}: P(e) = {got} against {pe}"
                    );
                }
            }
            let printed = common::ground(&source, rows, false).to_string();
            check(
                &format!("{rows:?}, printed ground"),
                common::run(
                    &printed,
                    &common::options(Strategy::Auto, Projection::default()),
                ),
                false,
            );
        }
    }
}

#[test]
fn the_rows_keep_the_heads_left_and_their_masses() {
    for k in [3, 32] {
        let net = Net::new(k);
        let source = net.program();
        // With 32 values of `a`, the masses go first in each row's chain
        // and are observed through `c(3)`.
        let dead_head = k >= 32;
        for rows in [Rows::Apart, Rows::Classes] {
            let ground = common::ground(&source, rows, false);
            assert!(
                ground.stats.conditioned.renormalized >= 2,
                "{k} values, {rows:?}: {} rows renormalized:\n{ground}",
                ground.stats.conditioned.renormalized
            );
            let name = |a: u32| ground.atoms[a as usize].to_string();
            // A rule for a ruled-out atom: with the masses on a false head,
            // the dead head of a row, `c(3) :- a(i), \+w`.
            let dead = |r: &tddlog::ground::GroundRule| {
                r.body
                    .iter()
                    .any(|l| !l.is_positive() && name(l.atom()).starts_with("'$rn_w'"))
            };
            let defines = |atom: &str| {
                ground
                    .rules
                    .iter()
                    .any(|r| name(r.head) == atom && !(dead_head && dead(r)))
            };
            let reads = |atom: &str| {
                ground
                    .rules
                    .iter()
                    .any(|r| r.body.iter().any(|l| name(l.atom()) == atom))
            };
            for ruled_out in [format!("a({k})"), "c(3)".to_string()] {
                assert!(
                    !defines(&ruled_out) && !reads(&ruled_out),
                    "{k} values, {rows:?}: {ruled_out} is still in the program:\n{ground}"
                );
            }
            let own_atom = ground
                .atoms
                .iter()
                .any(|a| a.to_string().starts_with("'$rn_bad'"));
            if dead_head {
                assert!(
                    ground
                        .rules
                        .iter()
                        .any(|r| name(r.head) == "c(3)" && dead(r)),
                    "{k} values, {rows:?}: no dead head:\n{ground}"
                );
                assert!(
                    !own_atom,
                    "{k} values, {rows:?}: an atom of the masses' own:\n{ground}"
                );
            } else {
                assert!(
                    own_atom,
                    "{k} values, {rows:?}: no atom of the masses' own:\n{ground}"
                );
            }
            // The masses of c's rows, each a fact its context requires.
            let printed = ground.to_string();
            let masses: Vec<f64> = printed
                .lines()
                .filter_map(|l| l.split_once("::'$rn_w'"))
                .map(|(p, _)| p.parse().unwrap())
                .collect();
            assert_eq!(
                masses.len(),
                k - 1,
                "{k} values, {rows:?}: the masses' facts:\n{ground}"
            );
            for i in 0..k - 1 {
                let m = 1.0 - net.c[i][2];
                assert!(
                    masses.iter().any(|&w| (w - m).abs() < 1e-12),
                    "{k} values, {rows:?}: no mass {m} in {masses:?}:\n{ground}"
                );
            }
            // The rows of `x` keep `x(t)`, observed: each reads one fact of
            // `x(t)`'s probability, and `x(f)` goes.
            let mut known: Vec<f64> = printed
                .lines()
                .filter_map(|l| l.split_once("::'$rn_t'"))
                .map(|(p, _)| p.parse().unwrap())
                .collect();
            known.sort_by(f64::total_cmp);
            assert_eq!(
                known,
                [X[1], X[0]],
                "{k} values, {rows:?}: the facts of x(t):\n{ground}"
            );
            assert!(
                !defines("x(f)") && !reads("x(f)"),
                "{k} values, {rows:?}: x(f) is still in the program:\n{ground}"
            );
        }
    }
}
