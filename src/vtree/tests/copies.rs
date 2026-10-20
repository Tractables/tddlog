//! Separator copies on clause programs whose `td` fold copies separator
//! facts: `P(e)` and the answers under `td` with every candidate unit
//! copied ([`CopyRule::ALL`]), with the units [`CopyRule::ON`] picks and
//! with none, and under `auto`, whose `td` trial takes the copies too,
//! equal a count by enumeration.
use std::sync::Arc;

use super::*;
use crate::ground::deps::Deps;
use crate::strategy::Prepared;
use crate::vtree::{Kind, td_layout};
use crate::{GroundOptions, Run, Strategy};

/// A fixed sequence of numbers in `0..n`.
struct Numbers(u64);

impl Numbers {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % n as u64) as usize
    }
}

/// A clause program over facts in clusters of three, each cluster but the
/// first tied to an earlier one: two clauses of three literals within a
/// cluster and three across to the earlier one, each `violated(j)`
/// observed false. The interaction graph is a tree of small clusters, so
/// `td`'s fold is a tree of bags with siblings to copy for. The program
/// queries `x(0)` and `q`, which reads facts of two clusters.
struct Program {
    probabilities: Vec<f64>,
    clauses: Vec<Vec<(usize, bool)>>,
    q: Vec<(usize, bool)>,
}

impl Program {
    fn random(seed: u64, clusters: usize) -> Program {
        let mut numbers = Numbers(seed);
        let n = 3 * clusters;
        let probabilities = (0..n)
            .map(|_| [0.2, 0.3, 0.5, 0.6, 0.9][numbers.below(5)])
            .collect();
        let mut clauses = Vec::new();
        let mut clause = |numbers: &mut Numbers, vars: &[usize]| {
            let mut lits: Vec<(usize, bool)> = Vec::new();
            while lits.len() < 3 {
                let v = vars[numbers.below(vars.len())];
                if lits.iter().all(|&(w, _)| w != v) {
                    lits.push((v, numbers.below(2) == 0));
                }
            }
            clauses.push(lits);
        };
        for c in 0..clusters {
            let own = [3 * c, 3 * c + 1, 3 * c + 2];
            for _ in 0..2 {
                clause(&mut numbers, &own);
            }
            if c > 0 {
                let p = numbers.below(c);
                let both = [3 * c, 3 * c + 1, 3 * c + 2, 3 * p, 3 * p + 1, 3 * p + 2];
                for _ in 0..3 {
                    clause(&mut numbers, &both);
                }
            }
        }
        let q = vec![(1, true), (n - 1, false)];
        Program {
            probabilities,
            clauses,
            q,
        }
    }

    fn source(&self) -> String {
        let lit = |&(v, positive): &(usize, bool)| {
            if positive {
                format!("x({v})")
            } else {
                format!("\\+x({v})")
            }
        };
        let mut s = String::new();
        for (v, p) in self.probabilities.iter().enumerate() {
            s += &format!("{p}::x({v}).\n");
        }
        for (j, c) in self.clauses.iter().enumerate() {
            s += &format!(
                "violated({j}) :- {}.\nevidence(violated({j}), false).\n",
                c.iter().map(lit).collect::<Vec<_>>().join(", ")
            );
        }
        s += &format!(
            "q :- {}.\nquery(x(0)).\nquery(q).\n",
            self.q.iter().map(lit).collect::<Vec<_>>().join(", ")
        );
        s
    }

    /// `P(e)`, `P(x(0) | e)` and `P(q | e)` by enumeration.
    fn exact(&self) -> (f64, f64, f64) {
        let n = self.probabilities.len();
        let (mut e, mut x0, mut q) = (0.0, 0.0, 0.0);
        for world in 0u32..1 << n {
            let holds = |&(v, positive): &(usize, bool)| (world >> v & 1 == 1) == positive;
            if self.clauses.iter().any(|c| c.iter().all(holds)) {
                continue;
            }
            let w: f64 = (0..n)
                .map(|v| {
                    if world >> v & 1 == 1 {
                        self.probabilities[v]
                    } else {
                        1.0 - self.probabilities[v]
                    }
                })
                .product();
            e += w;
            if world & 1 == 1 {
                x0 += w;
            }
            if self.q.iter().all(holds) {
                q += w;
            }
        }
        (e, x0 / e, q / e)
    }
}

/// How [`answers`] runs a program: under `auto`, or under `td` with the
/// separator copies a rule makes, none where `None`.
#[derive(Clone, Copy, PartialEq)]
enum Under {
    Auto,
    Td(Option<CopyRule>),
}

/// `x(0)`, `q` and `log10 P(e)` of `program` run `under`, and the copies
/// its layout made.
fn answers(program: &Program, under: Under) -> ((f64, f64, f64), usize) {
    let parsed = crate::syntax::parse_program(&program.source()).expect("parses");
    let options = GroundOptions {
        detach: false,
        ..GroundOptions::default()
    };
    let ground = crate::ground::ground(&parsed, &options).expect("grounds");
    let run: Run = match under {
        Under::Auto => {
            crate::solve_ground(ground, &crate::Options::with(Strategy::Auto)).expect("answers")
        }
        Under::Td(rule) => {
            let deps = Deps::of(&ground);
            let (layout, copied) = td_layout(&ground, &deps, rule.as_ref())
                .expect("decomposes")
                .expect("a fold");
            let ground = copied.unwrap_or(ground);
            let prepared = Prepared {
                deps: Arc::new(Deps::of(&ground)),
                ground,
                layout,
                rewrite_seconds: 0.0,
                layout_seconds: 0.0,
            };
            crate::solve::run_prepared(
                prepared,
                Kind::Td,
                crate::Projection::default(),
                crate::tp::DYNAMIC_FIRST_PAIRS,
            )
            .expect("answers")
        }
    };
    let results = run.results.expect("consistent evidence");
    let p = |name: &str| {
        results
            .iter()
            .find(|r| r.atom.to_string() == name)
            .unwrap_or_else(|| panic!("no {name}"))
            .probability
    };
    let copies = run
        .vtree_notes
        .iter()
        .find_map(|n| n.split("separator copies: ").nth(1))
        .map_or(0, |n| n.split(' ').next().unwrap().parse().unwrap());
    (
        (
            p("x(0)"),
            p("q"),
            run.evidence_probability.expect("P(e)").log10(),
        ),
        copies,
    )
}

#[test]
fn copies_keep_the_evidence_probability_and_the_answers() {
    let mut copied = 0;
    for seed in 1..=6 {
        let program = Program::random(seed, 6);
        let (e, x0, q) = program.exact();
        let runs = [
            (Under::Td(Some(CopyRule::ALL)), "td, all"),
            (Under::Td(Some(CopyRule::ON)), "td"),
            (Under::Td(None), "td, none"),
            (Under::Auto, "auto"),
        ];
        for (under, name) in runs {
            let ((a, b, l), made) = answers(&program, under);
            if under == Under::Td(Some(CopyRule::ALL)) {
                copied += made;
            }
            assert!(
                (a - x0).abs() < 1e-8 && (b - q).abs() < 1e-8,
                "seed {seed}, {name}: {a} {b} against {x0} {q}"
            );
            assert!(
                (l - e.log10()).abs() < 1e-8,
                "seed {seed}, {name}: log10 P(e) {l} against {}",
                e.log10()
            );
        }
    }
    assert!(copied > 0, "no copies made");
}
