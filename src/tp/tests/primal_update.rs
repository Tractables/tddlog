//! The primal's update of an atom in a cyclic component conjoins the
//! bodies' terms into the negation of the atom's formula, factored where
//! the bodies share a derived literal: checked against enumeration of every
//! world, with and without a negated atom of a lower stratum in the bodies.
//! The recursion is linear, which a layout's rewrite would write without
//! recursion, so the program is compiled as grounded
//! (`crate::solve::run_as_given`).

use crate::tp::Projection;
use crate::vtree::Kind;
use crate::{GroundOptions, ground, syntax};

/// The facts: the start, `g`, and edges of two kinds as (kind, from, to,
/// probability). The bodies of `reach(x)` share `reach(y)` for each `y`
/// with both an `e` and an `f` edge into `x`. `reach(b)` also holds by
/// `g`, a rule its first update folds and no later one does, so every
/// later update of `b` folds the two bodies that share `reach(a)` alone and
/// needs `b`'s formula to keep `g`.
const START: f64 = 0.9;
const G: f64 = 0.25;
const EDGES: [(&str, &str, &str, f64); 7] = [
    ("e", "a", "b", 0.3),
    ("f", "a", "b", 0.6),
    ("e", "b", "c", 0.4),
    ("f", "b", "c", 0.2),
    ("e", "c", "a", 0.7),
    ("e", "b", "a", 0.5),
    ("f", "c", "d", 0.8),
];
const NODES: [&str; 4] = ["a", "b", "c", "d"];
/// The probability of each node's `c`, whose `cut` blocks the edges into
/// it where the rules read `\+cut(X)`.
const CUT: [f64; 4] = [0.2, 0.3, 0.1, 0.4];

/// `P(reach(n))` for each node, by enumerating the worlds of the facts,
/// with the edges into a cut node blocked if `cut`.
fn enumerated(cut: bool) -> Vec<f64> {
    let mut p = vec![0.0; NODES.len()];
    let node = |n: &str| NODES.iter().position(|&m| m == n).unwrap();
    let cuts = if cut { NODES.len() } else { 0 };
    for world in 0..1usize << (EDGES.len() + 2 + cuts) {
        let on = |i: usize| world >> i & 1 == 1;
        let (start, g) = (EDGES.len(), EDGES.len() + 1);
        let blocked = |n: usize| cut && on(EDGES.len() + 2 + n);
        let mut weight = if on(start) { START } else { 1.0 - START };
        weight *= if on(g) { G } else { 1.0 - G };
        for (i, &(_, _, _, q)) in EDGES.iter().enumerate() {
            weight *= if on(i) { q } else { 1.0 - q };
        }
        for (n, &q) in CUT.iter().enumerate().take(cuts) {
            weight *= if blocked(n) { q } else { 1.0 - q };
        }
        let mut reached = if on(start) { 1usize << node("a") } else { 0 };
        if on(g) {
            reached |= 1 << node("b");
        }
        loop {
            let next = EDGES
                .iter()
                .enumerate()
                .filter(|&(i, &(_, from, to, _))| {
                    on(i) && reached >> node(from) & 1 == 1 && !blocked(node(to))
                })
                .fold(reached, |r, (_, &(_, _, to, _))| r | 1 << node(to));
            if next == reached {
                break;
            }
            reached = next;
        }
        for (k, pk) in p.iter_mut().enumerate() {
            if reached >> k & 1 == 1 {
                *pk += weight;
            }
        }
    }
    p
}

/// The program, its edge rules reading `\+cut(X)` if `cut`.
fn program(cut: bool) -> String {
    let guard = if cut { ", \\+cut(X)" } else { "" };
    let mut source = format!(
        "{START}::s.\n{G}::g.\nreach(a) :- s.\nreach(b) :- g.\nreach(X) :- reach(Y), e(Y,X){guard}.\nreach(X) :- reach(Y), f(Y,X){guard}.\n"
    );
    for (kind, from, to, q) in EDGES {
        source += &format!("{q}::{kind}({from},{to}).\n");
    }
    if cut {
        source += "cut(X) :- c(X).\n";
        for (n, q) in NODES.iter().zip(CUT) {
            source += &format!("{q}::c({n}).\n");
        }
    }
    for n in NODES {
        source += &format!("query(reach({n})).\n");
    }
    source
}

#[test]
fn an_update_through_shared_derived_literals_matches_enumeration() {
    check(false);
}

#[test]
fn an_update_with_a_negated_lower_atom_matches_enumeration() {
    // The terms factored on the shared `reach(Y)` and `\+cut(X)`.
    check(true);
}

fn check(cut: bool) {
    let expected = enumerated(cut);
    let gp = ground::ground(
        &syntax::parse_program(&program(cut)).unwrap(),
        &GroundOptions::default(),
    )
    .unwrap();
    // `dynamic` compiles under the primal, `closed` under the dual; the
    // primal also with a search at every step, which changes the vtree
    // under its updates.
    for (kind, first) in [
        (Kind::Closed, None),
        (Kind::Dynamic, None),
        (Kind::Dynamic, Some(0)),
    ] {
        let run = crate::solve::run_as_given(
            gp.clone(),
            kind,
            Projection::Off,
            first.unwrap_or(crate::tp::DYNAMIC_FIRST_PAIRS),
        )
        .unwrap();
        let what = format!("{} {kind} {first:?}", kind.formulation());
        assert!(
            run.stats.cyclic_components > 0,
            "{what}: the component is cyclic"
        );
        let results = run.results.unwrap();
        assert_eq!(results.len(), expected.len(), "{what}: number of answers");
        for (r, p) in results.iter().zip(&expected) {
            assert!(
                (r.probability - p).abs() < 1e-12,
                "{what}: {} = {} against {p}",
                r.atom,
                r.probability
            );
        }
    }
}
