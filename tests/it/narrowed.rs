//! A formula that depends on fewer facts than its rules mention, on a scope
//! large enough to be narrowed: `big` is the disjunction of 1500 facts,
//! and `z` reads it through `x1` and `x2`, which cancel it, so `z` is
//! `(a, c) or (d, e)` on a cone of 1504 facts. Under `closed` the compile
//! moves `z`'s formula onto the scope of those four, and `w` and `u`,
//! which read it, start from that (src/tp/compile.rs,
//! `Compilation::narrow`). Under `program` nothing narrows: the boundary
//! after `big`'s facts is a narrow cut of the file's order, so `z` reads
//! `big` through a copy (src/vtree/naming.rs), its cone is five facts,
//! and the largest is the link's, `big`'s 1500 and the copy. Under
//! `closed`, `program` and `auto`, with and without evidence, the answers
//! equal hand-computed ones.

use crate::common;

use tddlog::{Kind, Projection, Strategy};

/// The facts `big` reads; beside them the six of `SMALL`, and 1600 more
/// that only the fillers read, so that `z`'s cone is under the global
/// limit.
const WIDE: usize = 1500;
const FILLERS: usize = 1600;
const SMALL: [(&str, f64); 6] = [
    ("a", 0.3),
    ("c", 0.7),
    ("d", 0.4),
    ("e", 0.5),
    ("g", 0.6),
    ("h", 0.2),
];

fn p(i: usize) -> f64 {
    0.001 * (1 + i % 7) as f64
}

fn program(evidence: Option<bool>) -> String {
    let mut src = String::new();
    for (x, q) in SMALL {
        src += &format!("{q}::{x}.\n");
    }
    for i in 0..WIDE {
        src += &format!("{}::b{i}.\nbig :- b{i}.\n", p(i));
    }
    for i in 0..FILLERS {
        src += &format!("0.5::f{i}.\n");
    }
    src += "x1 :- a, c, big.\nx2 :- a, c, \\+big.\nx3 :- d, e.\n\
            z :- x1.\nz :- x2.\nz :- x3.\n\
            w :- z, g.\nw :- \\+z, h.\n\
            u :- w, g.\n";
    for i in (0..FILLERS).step_by(2) {
        src += &format!("q{i} :- f{i}, f{}.\nquery(q{i}).\n", i + 1);
    }
    if let Some(value) = evidence {
        src += &format!("evidence(w, {value}).\n");
    }
    src += "query(z).\nquery(w).\nquery(u).\nquery(big).\n";
    src
}

/// P(z), P(w), P(u) and P(big), given `w` = `evidence` where there is one,
/// by enumeration of the six facts of `SMALL`; `big` cancels out of `z`.
fn exact(evidence: Option<bool>) -> Vec<(String, f64)> {
    let big = 1.0 - (0..WIDE).map(|i| 1.0 - p(i)).product::<f64>();
    let (mut total, mut sums) = (0.0, [0.0; 3]);
    for world in 0..1usize << SMALL.len() {
        let t = |i: usize| world >> i & 1 == 1;
        let q: f64 = (0..SMALL.len())
            .map(|i| if t(i) { SMALL[i].1 } else { 1.0 - SMALL[i].1 })
            .product();
        let (a, c, d, e, g, h) = (t(0), t(1), t(2), t(3), t(4), t(5));
        let z = (a && c) || (d && e);
        let w = (z && g) || (!z && h);
        let u = w && g;
        if evidence.is_some_and(|value| w != value) {
            continue;
        }
        total += q;
        for (s, holds) in sums.iter_mut().zip([z, w, u]) {
            if holds {
                *s += q;
            }
        }
    }
    let mut expected: Vec<(String, f64)> = (0..FILLERS)
        .step_by(2)
        .map(|i| (format!("q{i}"), 0.25))
        .collect();
    for (q, s) in ["z", "w", "u"].iter().zip(sums) {
        expected.push((q.to_string(), s / total));
    }
    expected.push(("big".to_string(), big));
    expected
}

#[test]
fn a_formula_narrowed_to_its_support_is_the_hand_computed_answer() {
    for evidence in [None, Some(true), Some(false)] {
        let source = program(evidence);
        let mut expected = exact(evidence);
        for strategy in [
            Strategy::Trial(Kind::Closed),
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Auto,
        ] {
            let what = format!("evidence {evidence:?}, {strategy:?}");
            let options = common::options(strategy, Projection::default());
            let run = common::run(&source, &options).unwrap_or_else(|e| panic!("{what}: {e}"));
            let results = run.results.as_ref().expect("consistent evidence");
            // The answers in the order tddlog gives them.
            expected.sort_by_key(|(q, _)| {
                results
                    .iter()
                    .position(|r| r.to_string() == *q)
                    .unwrap_or_else(|| panic!("{what}: no answer for {q}"))
            });
            common::assert_answers(&what, results, &expected, 1e-12);
            if strategy == Strategy::Trial(Kind::Closed) {
                let narrowed = run.stats.projection.map_or(0, |p| p.narrowed);
                assert!(
                    narrowed > 0,
                    "{what}: nothing narrowed: {:?}",
                    run.stats.projection
                );
            }
        }
    }
}
