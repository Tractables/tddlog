//! One small network written in several ways a user might write it: as
//! annotated disjunctions, and as rules, a two-valued variable one atom
//! read negatively for its other value, a variable of more values each
//! value a rule that reads the earlier values negatively
//! (src/ground/negation.rs). Each writing's answers against an enumeration
//! of the worlds, under the strategies, and what the grounder makes of the
//! rules.

use crate::common;

use common::{Rows, assert_answers};
use tddlog::ground::GLit;
use tddlog::{Kind, Projection, Strategy};

/// A variable: its name, number of values, parents, and a distribution
/// per row, the rows in the order of the parents' values, the last parent
/// varying fastest.
struct Var {
    name: &'static str,
    values: usize,
    parents: &'static [usize],
    rows: &'static [&'static [f64]],
}

/// `a` and `b` are roots, `c` reads both, with a row where `c(1)` has no
/// mass and one where `c(2)` has none; `d` reads `c`, and `e` reads `d`
/// and `a`, with a row where `e` is false.
const NET: [Var; 5] = [
    Var {
        name: "a",
        values: 2,
        parents: &[],
        rows: &[&[0.3, 0.7]],
    },
    Var {
        name: "b",
        values: 3,
        parents: &[],
        rows: &[&[0.2, 0.5, 0.3]],
    },
    Var {
        name: "c",
        values: 3,
        parents: &[0, 1],
        rows: &[
            &[0.2, 0.4, 0.4],
            &[0.0, 0.5, 0.5],
            &[0.5, 0.25, 0.25],
            &[0.2, 0.2, 0.6],
            &[0.5, 0.25, 0.25],
            &[0.3, 0.0, 0.7],
        ],
    },
    Var {
        name: "d",
        values: 2,
        parents: &[2],
        rows: &[&[0.9, 0.1], &[0.4, 0.6], &[0.7, 0.3]],
    },
    Var {
        name: "e",
        values: 2,
        parents: &[3, 0],
        rows: &[&[0.95, 0.05], &[0.6, 0.4], &[0.3, 0.7], &[0.0, 1.0]],
    },
];

/// The evidence and the queries, as (variable, value).
const EVIDENCE: [(usize, usize); 1] = [(4, 0)];
const QUERIES: [(usize, usize); 6] = [(1, 1), (2, 0), (2, 1), (2, 2), (3, 0), (3, 1)];

/// Each row's context: the values of the parents, in order.
fn contexts(v: &Var) -> Vec<Vec<usize>> {
    let mut out = vec![Vec::new()];
    for &p in v.parents {
        out = out
            .into_iter()
            .flat_map(|c| {
                (0..NET[p].values).map(move |u| {
                    let mut c = c.clone();
                    c.push(u);
                    c
                })
            })
            .collect();
    }
    out
}

/// The network as annotated disjunctions, `x(v)` the value `v` of `x`.
fn annotated() -> String {
    annotated_reversing(|_| false)
}

/// [`annotated`] with the heads of each row `r` for which `reversed(r)`
/// holds in the reverse order.
fn annotated_reversing(reversed: fn(usize) -> bool) -> String {
    let atom = |x: usize, u: usize| format!("{}({})", NET[x].name, u + 1);
    let mut s = String::new();
    for (x, v) in NET.iter().enumerate() {
        for (r, (row, context)) in v.rows.iter().zip(contexts(v)).enumerate() {
            let mut heads: Vec<String> = row
                .iter()
                .enumerate()
                .filter(|&(_, &p)| p > 0.0)
                .map(|(u, p)| format!("{p}::{}", atom(x, u)))
                .collect();
            if reversed(r) {
                heads.reverse();
            }
            let body: Vec<String> = v
                .parents
                .iter()
                .zip(&context)
                .map(|(&p, &u)| atom(p, u))
                .collect();
            s += &heads.join("; ");
            if !body.is_empty() {
                s += &format!(" :- {}", body.join(", "));
            }
            s += ".\n";
        }
    }
    for &(x, u) in &EVIDENCE {
        s += &format!("evidence({}).\n", atom(x, u));
    }
    for &(x, u) in &QUERIES {
        s += &format!("query({}).\n", atom(x, u));
    }
    s
}

/// The network as rules: a two-valued `x` the atom `x`, its second value
/// `\+x` (named `x_2` where it is asked); a value `v` of a variable of
/// more values the rule `q::x(v) :- context, \+x(1), ..., \+x(v - 1)`,
/// `q` its mass over what the earlier values leave, and no rule where it
/// has no mass.
fn rules() -> String {
    let literal = |x: usize, u: usize| match (NET[x].values, u) {
        (2, 0) => NET[x].name.to_string(),
        (2, _) => format!("\\+{}", NET[x].name),
        _ => format!("{}({})", NET[x].name, u + 1),
    };
    let mut s = String::new();
    for (x, v) in NET.iter().enumerate() {
        for (row, context) in v.rows.iter().zip(contexts(v)) {
            let body: Vec<String> = v
                .parents
                .iter()
                .zip(&context)
                .map(|(&p, &u)| literal(p, u))
                .collect();
            let last = if v.values == 2 { 1 } else { v.values };
            for u in 0..last {
                let left: f64 = row[u..].iter().sum();
                if row[u] == 0.0 {
                    continue;
                }
                let q = row[u] / left;
                let mut body = body.clone();
                if v.values > 2 {
                    body.extend((0..u).map(|w| format!("\\+{}", literal(x, w))));
                }
                if q < 1.0 {
                    s += &format!("{q}::");
                }
                s += &literal(x, u);
                if !body.is_empty() {
                    s += &format!(" :- {}", body.join(", "));
                }
                s += ".\n";
            }
        }
    }
    let named = |x: usize, u: usize| match (NET[x].values, u) {
        (2, 1) => format!("{}_2", NET[x].name),
        _ => literal(x, u),
    };
    let mut asked: Vec<(usize, usize)> = EVIDENCE.to_vec();
    asked.extend(QUERIES);
    for &(x, u) in &asked {
        if NET[x].values == 2 && u == 1 && !s.contains(&format!("{} :-", named(x, u))) {
            s += &format!("{} :- \\+{}.\n", named(x, u), NET[x].name);
        }
    }
    for &(x, u) in &EVIDENCE {
        s += &format!("evidence({}).\n", named(x, u));
    }
    for &(x, u) in &QUERIES {
        s += &format!("query({}).\n", named(x, u));
    }
    s
}

/// The queries' probabilities given the evidence, summed over the worlds.
fn enumerated() -> Vec<f64> {
    let mut joint = vec![0.0; QUERIES.len()];
    let mut evidence = 0.0;
    let mut world = vec![0usize; NET.len()];
    loop {
        let w: f64 = NET
            .iter()
            .enumerate()
            .map(|(x, v)| {
                let row = v
                    .parents
                    .iter()
                    .fold(0, |r, &p| r * NET[p].values + world[p]);
                v.rows[row][world[x]]
            })
            .product();
        if EVIDENCE.iter().all(|&(x, u)| world[x] == u) {
            evidence += w;
            for (q, &(x, u)) in joint.iter_mut().zip(&QUERIES) {
                if world[x] == u {
                    *q += w;
                }
            }
        }
        // The next world, the last variable varying fastest.
        let Some(x) = (0..NET.len()).rev().find(|&x| world[x] + 1 < NET[x].values) else {
            break;
        };
        world[x] += 1;
        world[x + 1..].fill(0);
    }
    joint.iter().map(|q| q / evidence).collect()
}

#[test]
fn a_network_written_as_rules_answers_as_written_as_annotated_disjunctions() {
    let expected = enumerated();
    for (writing, source) in [
        ("annotated", annotated()),
        (
            "annotated, odd rows reversed",
            annotated_reversing(|r| r % 2 == 1),
        ),
        ("rules", rules()),
    ] {
        for strategy in [
            Strategy::Trial(Kind::Closed),
            Strategy::Auto,
            Strategy::Trial(Kind::Families { search: false }),
        ] {
            for projection in [Projection::Off, Projection::default()] {
                let what = format!("{writing}, {strategy:?}, {projection}");
                let run = common::run(&source, &common::options(strategy, projection))
                    .unwrap_or_else(|e| panic!("{what}: {e}"));
                let results = run.results.expect("evidence is consistent");
                let expected: Vec<(String, f64)> = results
                    .iter()
                    .map(|r| r.to_string())
                    .zip(expected.iter().copied())
                    .collect();
                assert_answers(&what, &results, &expected, 1e-9);
            }
        }
    }
}

#[test]
fn a_table_written_as_rules_reads_its_switches_not_its_values() {
    // The values of `b` and `c` no longer read each other: each rule of
    // `c(2)` reads the switch of `c(1)`'s row, or nothing where `c(1)` has
    // no mass in the row; the rule of `c(3)` in the row where `c(2)` has
    // none reads `c(1)`'s switch alone.
    let gp = common::ground(&rules(), Rows::Apart, false);
    assert!(gp.stats.unfolded > 0, "{gp}");
    for r in &gp.rules {
        for l in &r.body {
            let atom = &gp.atoms[l.atom() as usize];
            assert!(
                !(matches!(l, GLit::Neg(_))
                    && matches!(&*atom.name, "b" | "c")
                    && !gp.is_fact(l.atom())),
                "{gp}"
            );
        }
    }
}

#[test]
fn rows_written_with_their_values_in_other_orders_are_ground_alike() {
    // The odd rows of each table list their values backwards; the heads
    // are put in the order the rows agree on before the chains of
    // switches are made, so that the rows of one distribution (`c`'s
    // third and fifth) read switches alike, as written in one order.
    for rows in [Rows::Apart, Rows::Classes] {
        let reversed = common::ground(&annotated_reversing(|r| r % 2 == 1), rows, false);
        let written = common::ground(&annotated(), rows, false);
        assert_eq!(reversed.to_string(), written.to_string());
    }
}

/// A diagnosis network written as rules, as `scripts/writings.py` writes
/// a noisy-or network by hand: roots `a`, `b` and `c` of probabilities
/// `NOISY_ROOTS`; `f` a noisy-or of the three with a leak, `g` one of `b`
/// and `c` without, and `h` a table of `a` and `b` that is no noisy-or,
/// each row a rule `p::x :- ...` that reads every parent, the rows where
/// `x` has no mass left out. `f` is observed true and `g` false.
const NOISY_ROOTS: [f64; 3] = [0.3, 0.6, 0.2];

/// The probability of each finding true given the roots' values.
fn noisy_finding(x: usize, u: [bool; 3]) -> f64 {
    let inhibit =
        |q: &[(usize, f64)]| -> f64 { q.iter().filter(|&&(i, _)| u[i]).map(|&(_, q)| q).product() };
    match x {
        0 => 1.0 - 0.9 * inhibit(&[(0, 0.4), (1, 0.5), (2, 0.7)]),
        1 => 1.0 - inhibit(&[(1, 0.2), (2, 0.6)]),
        _ => [[0.7, 0.2], [0.1, 0.9]][usize::from(u[0])][usize::from(u[1])],
    }
}

/// The findings' names and parents.
const NOISY_FINDINGS: [(&str, &[usize]); 3] = [("f", &[0, 1, 2]), ("g", &[1, 2]), ("h", &[0, 1])];

fn noisy_rules() -> String {
    let names = ["a", "b", "c"];
    let mut s = String::new();
    for (name, p) in names.iter().zip(NOISY_ROOTS) {
        s += &format!("{p}::{name}.\n");
    }
    for (x, (name, parents)) in NOISY_FINDINGS.iter().enumerate() {
        for row in 0..1usize << parents.len() {
            let mut u = [false; 3];
            let mut body = Vec::new();
            for (k, &i) in parents.iter().enumerate() {
                u[i] = row >> k & 1 == 1;
                body.push(if u[i] {
                    names[i].to_string()
                } else {
                    format!("\\+{}", names[i])
                });
            }
            let p = noisy_finding(x, u);
            if p > 0.0 {
                s += &format!("{p}::{name} :- {}.\n", body.join(", "));
            }
        }
    }
    s + "evidence(f).\nevidence(g, false).\nquery(a).\nquery(b).\nquery(c).\nquery(h).\n"
}

/// P(e) and the queries' probabilities given the evidence, summed over the
/// worlds of the roots.
fn noisy_enumerated() -> (f64, Vec<f64>) {
    let (mut evidence, mut joint) = (0.0, [0.0; 4]);
    for w in 0..8usize {
        let u = [w & 1 == 1, w & 2 == 2, w & 4 == 4];
        let roots: f64 = (0..3)
            .map(|i| {
                if u[i] {
                    NOISY_ROOTS[i]
                } else {
                    1.0 - NOISY_ROOTS[i]
                }
            })
            .product();
        let e = roots * noisy_finding(0, u) * (1.0 - noisy_finding(1, u));
        evidence += e;
        for i in 0..3 {
            if u[i] {
                joint[i] += e;
            }
        }
        joint[3] += e * noisy_finding(2, u);
    }
    (evidence, joint.iter().map(|q| q / evidence).collect())
}

#[test]
fn a_noisy_or_written_as_rules_is_ground_as_its_noisy_or() {
    // f and g are written as their noisy-ors, h as written; P(e) and the
    // answers are the enumeration's, the rewrite on or off.
    let source = noisy_rules();
    let program = tddlog::syntax::parse_program(&source).unwrap();
    let (want_e, want) = noisy_enumerated();
    let expected: Vec<(&str, f64)> = ["a", "b", "c", "h"].into_iter().zip(want).collect();
    for noisy_or in [true, false] {
        let options = tddlog::ground::GroundOptions {
            noisy_or,
            detach: false,
            ..tddlog::ground::GroundOptions::default()
        };
        let gp = tddlog::ground::ground(&program, &options).unwrap();
        assert_eq!(gp.stats.noisy_or, if noisy_or { 2 } else { 0 }, "{gp}");
        for strategy in [
            Strategy::Trial(Kind::Closed),
            Strategy::Trial(Kind::Td),
            Strategy::Auto,
        ] {
            for projection in [Projection::Off, Projection::default()] {
                let what = format!("noisy_or {noisy_or}, {strategy:?}, {projection}");
                let run = tddlog::solve_ground(gp.clone(), &common::options(strategy, projection))
                    .unwrap_or_else(|e| panic!("{what}: {e}"));
                let p = run.evidence_probability.expect("nothing detached").to_f64();
                assert!(
                    (p - want_e).abs() < 1e-12 * want_e,
                    "{what}: P(e) = {p} against {want_e}"
                );
                assert_answers(
                    &what,
                    &run.results.expect("evidence is consistent"),
                    &expected,
                    1e-12,
                );
            }
        }
    }
}

#[test]
fn a_noisy_or_written_as_rules_reads_each_parent_once() {
    // Ground as its noisy-or, f's rules read one root each, g's too, and
    // h's rows read both of its parents.
    let gp = common::ground_as_written(&noisy_rules(), Rows::Apart, false);
    let name = |l: &GLit| gp.atoms[l.atom() as usize].name.to_string();
    for r in &gp.rules {
        let roots = r
            .body
            .iter()
            .filter(|l| matches!(&*name(l), "a" | "b" | "c"))
            .count();
        match &*gp.atoms[r.head as usize].name {
            "f" | "g" => assert!(roots <= 1, "{gp}"),
            "h" => assert_eq!(roots, 2, "{gp}"),
            _ => {}
        }
    }
}
