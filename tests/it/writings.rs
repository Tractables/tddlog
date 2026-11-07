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
/// and `a`, with a row where `e` is false; `f`, of four values, reads `b`,
/// and `g` reads `f` and `a`; `h`, of two values, reads `c`, `f` and `a`,
/// and only `i`, of eight values, reads it.
const NET: [Var; 9] = [
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
    Var {
        name: "f",
        values: 4,
        parents: &[1],
        rows: &[
            &[0.1, 0.2, 0.3, 0.4],
            &[0.4, 0.1, 0.25, 0.25],
            &[0.25, 0.25, 0.1, 0.4],
        ],
    },
    Var {
        name: "g",
        values: 2,
        parents: &[5, 0],
        rows: &[
            &[0.9, 0.1],
            &[0.8, 0.2],
            &[0.3, 0.7],
            &[0.6, 0.4],
            &[0.5, 0.5],
            &[0.15, 0.85],
            &[0.35, 0.65],
            &[0.7, 0.3],
        ],
    },
    Var {
        name: "h",
        values: 2,
        parents: &[2, 5, 0],
        rows: &[
            &[0.01, 0.99],
            &[0.02, 0.98],
            &[0.03, 0.97],
            &[0.04, 0.96],
            &[0.05, 0.95],
            &[0.06, 0.94],
            &[0.07, 0.93],
            &[0.08, 0.92],
            &[0.09, 0.91],
            &[0.11, 0.89],
            &[0.12, 0.88],
            &[0.13, 0.87],
            &[0.14, 0.86],
            &[0.16, 0.84],
            &[0.17, 0.83],
            &[0.18, 0.82],
            &[0.19, 0.81],
            &[0.21, 0.79],
            &[0.22, 0.78],
            &[0.23, 0.77],
            &[0.24, 0.76],
            &[0.26, 0.74],
            &[0.27, 0.73],
            &[0.28, 0.72],
        ],
    },
    Var {
        name: "i",
        values: 8,
        parents: &[7],
        rows: &[
            &[0.1, 0.2, 0.1, 0.2, 0.15, 0.05, 0.12, 0.08],
            &[0.25, 0.05, 0.1, 0.15, 0.05, 0.2, 0.06, 0.14],
        ],
    },
];

/// The evidence and the queries, as (variable, value).
const EVIDENCE: [(usize, usize); 1] = [(4, 0)];
const QUERIES: [(usize, usize); 10] = [
    (1, 1),
    (2, 0),
    (2, 1),
    (2, 2),
    (3, 0),
    (3, 1),
    (5, 2),
    (6, 0),
    (8, 3),
    (8, 5),
];

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

/// The network as a Dice program, as a Dice user writes a table: each
/// variable an integer of the fewest bits that hold its values, each row a
/// `discrete` under a chain of conditionals on its parents' values, the
/// first parent outermost, its last value the final `else`; the evidence
/// an `observe` and the queries the components of the returned tuple.
fn dice() -> String {
    let width = |x: usize| (usize::BITS - (NET[x].values - 1).leading_zeros()).max(1);
    let value = |x: usize, u: usize| format!("int({}, {u})", width(x));
    fn table(x: usize, depth: usize, row: usize, value: &dyn Fn(usize, usize) -> String) -> String {
        let v = &NET[x];
        let Some(&p) = v.parents.get(depth) else {
            let row: Vec<String> = v.rows[row].iter().map(|q| format!("{q:?}")).collect();
            return format!("discrete({})", row.join(", "));
        };
        let name = NET[p].name.to_uppercase();
        let last = NET[p].values - 1;
        let mut s = table(x, depth + 1, row * NET[p].values + last, value);
        for u in (0..last).rev() {
            let then = table(x, depth + 1, row * NET[p].values + u, value);
            s = format!("if ({name} == {}) then ({then}) else ({s})", value(p, u));
        }
        s
    }
    let mut s = String::new();
    for (x, v) in NET.iter().enumerate() {
        s += &format!(
            "let {} = {} in\n",
            v.name.to_uppercase(),
            table(x, 0, 0, &value)
        );
    }
    for (k, &(x, u)) in EVIDENCE.iter().enumerate() {
        s += &format!(
            "let o{k} = observe ({} == {}) in\n",
            NET[x].name.to_uppercase(),
            value(x, u)
        );
    }
    let asked: Vec<String> = QUERIES
        .iter()
        .map(|&(x, u)| format!("{} == {}", NET[x].name.to_uppercase(), value(x, u)))
        .collect();
    let mut tuple = asked.last().unwrap().clone();
    for q in asked[..asked.len() - 1].iter().rev() {
        tuple = format!("({q}, {tuple})");
    }
    s + &tuple + "\n"
}

/// The network with each variable's values in binary, as a user who
/// encodes them so writes it: value `u` of `x` the code `u + 1` on the
/// fewest bit atoms `x_bit(j)` that hold it, each row a chain of switches
/// setting the bits of its value, a row's context the cubes of its
/// parents' codes, and the values observed and asked atoms that read their
/// codes.
fn binary() -> String {
    let width = |x: usize| (usize::BITS - NET[x].values.leading_zeros()) as usize;
    let cube = |x: usize, u: usize| -> Vec<String> {
        (0..width(x))
            .map(|j| {
                let bit = format!("{}_bit({j})", NET[x].name);
                if (u + 1) >> j & 1 == 1 {
                    bit
                } else {
                    format!("\\+{bit}")
                }
            })
            .collect()
    };
    let mut s = String::new();
    for (x, v) in NET.iter().enumerate() {
        for (r, (row, context)) in v.rows.iter().zip(contexts(v)).enumerate() {
            let body: Vec<String> = v
                .parents
                .iter()
                .zip(&context)
                .flat_map(|(&p, &u)| cube(p, u))
                .collect();
            let given: Vec<usize> = (0..row.len()).filter(|&u| row[u] > 0.0).collect();
            let (mut done, mut left): (Vec<String>, f64) = (Vec::new(), 1.0);
            for (i, &u) in given.iter().enumerate() {
                let mut b = body.clone();
                if i + 1 < given.len() {
                    let switch = format!("s_{}_{r}_{i}", v.name);
                    s += &format!("{:?}::{switch}.\n", row[u] / left);
                    left -= row[u];
                    b.push(switch.clone());
                    b.extend(done.iter().cloned());
                    done.push(format!("\\+{switch}"));
                } else {
                    b.extend(done.iter().cloned());
                }
                for j in 0..width(x) {
                    if (u + 1) >> j & 1 == 1 {
                        s += &format!("{}_bit({j})", v.name);
                        if !b.is_empty() {
                            s += &format!(" :- {}", b.join(", "));
                        }
                        s += ".\n";
                    }
                }
            }
        }
    }
    let atom = |x: usize, u: usize| format!("{}({})", NET[x].name, u + 1);
    let mut named: Vec<(usize, usize)> = EVIDENCE.iter().chain(&QUERIES).copied().collect();
    named.sort_unstable();
    named.dedup();
    for (x, u) in named {
        s += &format!("{} :- {}.\n", atom(x, u), cube(x, u).join(", "));
    }
    for &(x, u) in &EVIDENCE {
        s += &format!("evidence({}).\n", atom(x, u));
    }
    for &(x, u) in &QUERIES {
        s += &format!("query({}).\n", atom(x, u));
    }
    s
}

/// The network's writings over bits, Dice's and [`binary`]'s.
fn over_bits() -> [(&'static str, String); 2] {
    [
        ("dice", tddlog::syntax::dice::read(&dice()).unwrap()),
        ("binary", binary()),
    ]
}

#[test]
fn a_network_written_over_bits_answers_as_the_enumeration() {
    // Written over their values (src/ground/values.rs).
    let expected = enumerated();
    for (writing, source) in over_bits() {
        for strategy in [
            Strategy::Trial(Kind::Closed),
            Strategy::Auto,
            Strategy::Trial(Kind::Families { search: false }),
        ] {
            let what = format!("{writing}, {strategy:?}");
            let run = common::run(&source, &common::options(strategy, Projection::default()))
                .unwrap_or_else(|e| panic!("{what}: {e}"));
            let results = run.results.expect("evidence is consistent");
            assert_eq!(results.len(), expected.len(), "{what}");
            for (r, &want) in results.iter().zip(&expected) {
                assert!(
                    (r.probability - want).abs() < 1e-9,
                    "{what}: {r} against {want}"
                );
            }
        }
    }
}

#[test]
fn a_network_written_over_bits_is_ground_over_its_values() {
    // Every variable is written over its values, each value an atom, but
    // a in Dice, one fact there; no rule reads a bit of b, c or f: their
    // readers read their values, g's those of f through the chain that
    // Dice reads f's low bit in once its first values are excluded; h,
    // whose table summed into i's would pass the rows' bound, is written
    // apart from it and i's rows read its values.
    for ((writing, source), written) in over_bits().into_iter().zip([8, 9]) {
        let gp = common::ground_as_written(&source, Rows::Classes, false);
        assert_eq!(gp.stats.values, written, "{writing}: {gp}");
        let bit = |name: &str| {
            !name.contains('$') && ["b_b", "c_b", "f_b"].iter().any(|b| name.starts_with(b))
        };
        for r in &gp.rules {
            for l in &r.body {
                let name = &gp.atoms[l.atom() as usize].name;
                assert!(
                    !bit(name),
                    "{writing}: {} reads {name}: {gp}",
                    gp.atoms[r.head as usize]
                );
            }
        }
    }
}

#[test]
fn a_network_written_over_its_values_is_left_as_written() {
    // The pass finds no variable held as bits where every variable is
    // written over its values, as annotated disjunctions or as rules.
    // `x` alone reads `c`, each value of `c` in three rules, and each
    // rule reads the first switch of its row too.
    let chain = "0.2::c(1); 0.3::c(2); 0.5::c(3).
        0.1::x(1); 0.6::x(2); 0.3::x(3) :- c(1).
        0.4::x(1); 0.4::x(2); 0.2::x(3) :- c(2).
        0.7::x(1); 0.2::x(2); 0.1::x(3) :- c(3).
        query(x(1)). query(x(2)).";
    for (writing, source) in [
        ("annotated", annotated()),
        ("rules", rules()),
        ("chain", chain.to_string()),
    ] {
        for rows in [Rows::Apart, Rows::Classes] {
            for gp in [
                common::ground(&source, rows, false),
                common::ground_as_written(&source, rows, false),
            ] {
                assert_eq!(gp.stats.values, 0, "{writing}: {gp}");
            }
        }
    }
}

/// A variable of two bits, each a function of the same sixteen flips
/// (a disjunction of cubes over them), read as values: it owns the flips,
/// and its one row's distribution is the sum over their worlds, which
/// takes more steps than the row could write. The pass leaves it as
/// written (src/ground/values.rs, the bounds), and it answers as the
/// enumeration of the flips.
#[test]
fn a_variable_computed_from_many_flips_is_left_as_written() {
    const FLIPS: usize = 16;
    let mut seed = 7u64;
    let mut next = |n: u64| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) % n
    };
    let ps: Vec<f64> = (0..FLIPS).map(|_| (1 + next(9)) as f64 / 10.0).collect();
    // Each bit's cubes, each three literals over distinct flips.
    let cubes: Vec<Vec<Vec<(usize, bool)>>> = (0..2)
        .map(|_| {
            (0..24)
                .map(|_| {
                    let mut c: Vec<(usize, bool)> = Vec::new();
                    while c.len() < 3 {
                        let f = next(FLIPS as u64) as usize;
                        if c.iter().all(|&(g, _)| g != f) {
                            c.push((f, next(2) == 1));
                        }
                    }
                    c
                })
                .collect()
        })
        .collect();
    let mut source = String::new();
    for (f, p) in ps.iter().enumerate() {
        source += &format!("{p}::f({f}).\n");
    }
    for (b, bit) in cubes.iter().enumerate() {
        for c in bit {
            let body: Vec<String> = c
                .iter()
                .map(|&(f, pos)| format!("{}f({f})", if pos { "" } else { "\\+" }))
                .collect();
            source += &format!("b{b} :- {}.\n", body.join(", "));
        }
    }
    source += "q(0) :- \\+b0, \\+b1.\nq(1) :- b0, \\+b1.\nq(2) :- b0, b1.\n";
    source += "query(q(0)). query(q(1)). query(q(2)).\n";
    let gp = common::ground_as_written(&source, Rows::Classes, false);
    assert_eq!(gp.stats.values, 0, "{gp}");
    let mut expected = [0.0; 3];
    for world in 0..1usize << FLIPS {
        let p: f64 = (0..FLIPS)
            .map(|f| {
                if world >> f & 1 == 1 {
                    ps[f]
                } else {
                    1.0 - ps[f]
                }
            })
            .product();
        let holds = |bit: &Vec<Vec<(usize, bool)>>| {
            bit.iter()
                .any(|c| c.iter().all(|&(f, pos)| (world >> f & 1 == 1) == pos))
        };
        match (holds(&cubes[0]), holds(&cubes[1])) {
            (false, false) => expected[0] += p,
            (true, false) => expected[1] += p,
            (true, true) => expected[2] += p,
            (false, true) => {}
        }
    }
    let run = common::run(
        &source,
        &common::options(Strategy::Auto, Projection::default()),
    )
    .expect("runs");
    let results = run.results.expect("no evidence");
    assert_eq!(results.len(), expected.len());
    for (r, &want) in results.iter().zip(&expected) {
        assert!((r.probability - want).abs() < 1e-9, "{r} against {want}");
    }
}

#[test]
fn a_private_variable_left_as_bits_reads_the_views_it_shares_with_its_reader() {
    // Q's rows with P summed in pass their bound, so P is apart from Q. P
    // reads only flips, no variable written, and stays as bits, reading the
    // comparison X1 && X2 that Q's rows have replaced. R makes the flips
    // read by more than P and Q.
    const X: [f64; 6] = [0.3, 0.6, 0.2, 0.7, 0.4, 0.55];
    const R: [f64; 6] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
    const Q: [[f64; 8]; 4] = [
        [0.1, 0.2, 0.1, 0.2, 0.15, 0.05, 0.12, 0.08],
        [0.25, 0.05, 0.1, 0.15, 0.05, 0.2, 0.06, 0.14],
        [0.05, 0.15, 0.3, 0.1, 0.1, 0.1, 0.05, 0.15],
        [0.2, 0.1, 0.05, 0.05, 0.3, 0.1, 0.1, 0.1],
    ];
    // P's table: its k-th row, X3 to X6 its bits and X1 && X2 the next.
    let p = |k: usize| 0.01 + 0.03 * k as f64;
    fn table(x: usize, k: usize, p: &dyn Fn(usize) -> f64) -> String {
        if x == 7 {
            return format!("flip {:?}", p(k));
        }
        format!(
            "if X{x} then ({}) else ({})",
            table(x + 1, 2 * k + 1, p),
            table(x + 1, 2 * k, p)
        )
    }
    let draw = |r: &[f64; 8]| {
        let weights: Vec<String> = r.iter().map(|q| format!("{q:?}")).collect();
        format!("discrete({})", weights.join(", "))
    };
    let mut source = String::new();
    for (k, q) in X.iter().enumerate() {
        source += &format!("let X{} = flip {q:?} in\n", k + 1);
    }
    let r: Vec<String> = R
        .iter()
        .enumerate()
        .map(|(k, q)| format!("(X{} && flip {q:?})", k + 1))
        .collect();
    source += &format!("let R = {} in\n", r.join(" || "));
    source += &format!(
        "let P = if (X1 && X2) then ({}) else ({}) in\n",
        table(3, 1, &p),
        table(3, 0, &p)
    );
    source += &format!(
        "let Q = if (X1 && X2) then (if P then {} else {}) else (if P then {} else {}) in\n",
        draw(&Q[0]),
        draw(&Q[1]),
        draw(&Q[2]),
        draw(&Q[3])
    );
    source += "(R, (Q == int(3, 3), Q == int(3, 5)))\n";
    let mut expected = [0.0; 3];
    for world in 0usize..1 << X.len() {
        let on = |k: usize| world >> k & 1 == 1;
        let weight: f64 = (0..X.len())
            .map(|k| if on(k) { X[k] } else { 1.0 - X[k] })
            .product();
        let not_r: f64 = (0..X.len())
            .map(|k| if on(k) { 1.0 - R[k] } else { 1.0 })
            .product();
        let both = on(0) && on(1);
        let row = (2..X.len()).fold(usize::from(both), |k, x| 2 * k + usize::from(on(x)));
        let (given, otherwise) = if both { (&Q[0], &Q[1]) } else { (&Q[2], &Q[3]) };
        expected[0] += weight * (1.0 - not_r);
        for (e, value) in expected[1..].iter_mut().zip([3, 5]) {
            *e += weight * (p(row) * given[value] + (1.0 - p(row)) * otherwise[value]);
        }
    }
    let source = tddlog::syntax::dice::read(&source).unwrap();
    let run = common::run(
        &source,
        &common::options(Strategy::Auto, Projection::default()),
    )
    .unwrap();
    let results = run.results.expect("no evidence");
    assert_eq!(results.len(), expected.len());
    for (r, &want) in results.iter().zip(&expected) {
        assert!((r.probability - want).abs() < 1e-9, "{r} against {want}");
    }
}

/// Wide integers read by comparisons with constants: uniforms of 16 and 20
/// bits, a flip a bit; a uniform of 16 bits over 40 000 values, which Dice
/// draws bit by bit from its masses; and a `discrete` of 300 values, 9 bits
/// drawn so, its masses 1 to 300 over their sum. Each stays as bits
/// (src/ground/values.rs), and answers as computed by hand.
#[test]
fn wide_integers_read_by_comparisons_stay_as_bits() {
    let total = 300.0 * 301.0 / 2.0;
    let masses: Vec<String> = (1..=300)
        .map(|k| format!("{:?}", k as f64 / total))
        .collect();
    let discrete = format!(
        "let x = discrete({}) in (x == int(9, 7), (x == int(9, 299), x < int(9, 150)))",
        masses.join(", ")
    );
    let uniform = |w: u32, hi: u64, small: u64| {
        format!(
            "let x = uniform({w}, 0, {hi}) in \
             (x == int({w}, 4242), (x == int({w}, 77), x < int({w}, {small})))"
        )
    };
    let cases = [
        (
            "uniform(16, 0, 65536)",
            uniform(16, 1 << 16, 30000),
            [1.0 / 65536.0, 1.0 / 65536.0, 30000.0 / 65536.0],
        ),
        (
            "uniform(20, 0, 1048576)",
            uniform(20, 1 << 20, 300000),
            [1.0 / 1048576.0, 1.0 / 1048576.0, 300000.0 / 1048576.0],
        ),
        (
            "uniform(16, 0, 40000)",
            uniform(16, 40000, 30000),
            [1.0 / 40000.0, 1.0 / 40000.0, 0.75],
        ),
        (
            "discrete of 300 values",
            discrete,
            [8.0 / total, 300.0 / total, 150.0 * 151.0 / 2.0 / total],
        ),
    ];
    for (what, dice, expected) in cases {
        let source = tddlog::syntax::dice::read(&dice).unwrap_or_else(|e| panic!("{what}: {e}"));
        let gp = common::ground_as_written(&source, Rows::Classes, false);
        assert_eq!(gp.stats.values, 0, "{what}: {gp}");
        let run = common::run(
            &source,
            &common::options(Strategy::Auto, Projection::default()),
        )
        .unwrap_or_else(|e| panic!("{what}: {e}"));
        let results = run.results.expect("no evidence");
        assert_eq!(results.len(), expected.len(), "{what}");
        for (r, &want) in results.iter().zip(&expected) {
            assert!(
                (r.probability - want).abs() < 1e-9,
                "{what}: {r} against {want}"
            );
        }
    }
}

/// A program `--ground` printed, read again with its lines in another
/// order, has the answers of the program it was printed from: the printed
/// program names the switches of its probabilistic rules, and the switches
/// it makes of its own (those of a predicate's facts beside its rules,
/// named after the clauses' places) take no name it already has. By hand:
/// `s(1)` holds with probability 0.4 × 0.6 = 0.24, and `s(2)` with 0.5 ×
/// (1 - 0.24) = 0.38.
#[test]
fn a_printed_ground_program_read_in_another_order_keeps_its_answers() {
    let source =
        "0.4::s(0).\n0.6::s(1) :- s(0).\n0.5::s(2) :- \\+s(1).\nquery(s(1)).\nquery(s(2)).\n";
    let options = common::options(Strategy::Auto, Projection::default());
    let printed = common::ground(source, Rows::Classes, false).to_string();
    assert!(printed.contains("'$sws_"), "{printed}");
    let lines: Vec<&str> = printed.lines().collect();
    for k in 0..lines.len() {
        let rotated = [&lines[k..], &lines[..k]].concat().join("\n");
        let run = common::run(&rotated, &options).unwrap_or_else(|e| panic!("{rotated}: {e}"));
        // The queries come in the order the rotation put them in.
        let mut got: Vec<(String, f64)> = run
            .results
            .expect("no evidence")
            .iter()
            .map(|r| (r.to_string(), r.probability))
            .collect();
        got.sort_by(|a, b| a.0.cmp(&b.0));
        common::assert_close(&rotated, &got, &[("s(1)", 0.24), ("s(2)", 0.38)], 1e-12);
    }
}

/// Geni's weather factory written first-order (the suites' weather.pl, two
/// factories over three days), with `sun/1` renamed into the names the
/// switches of its probabilistic rules take (`$sw`, the head's name, the
/// clause's place, the head's place), keeps its answers whatever the
/// place in the name: under the earlier switch names, the switch of
/// `sunny`'s first rule took the renamed predicate's name and merged the
/// two.
#[test]
fn the_weather_factory_keeps_its_answers_with_a_predicate_named_as_a_switch() {
    let source = "\
factory(1).
factory(2).
0.3::strikes(1, T, on, sun) :- step(T).
0.5::strikes(1, T, on, rain) :- step(T).
0.1::strikes(1, T, off, sun) :- step(T).
0.4::strikes(1, T, off, rain) :- step(T).
0.6::strikes(2, T, on, sun) :- step(T).
0.8::strikes(2, T, on, rain) :- step(T).
0.2::strikes(2, T, off, sun) :- step(T).
0.7::strikes(2, T, off, rain) :- step(T).
days(3).
step(0) :- days(H), H > 0.
step(T1) :- step(T), days(H), T1 is T + 1, T1 < H.
time(0).
time(T1) :- step(T), T1 is T + 1.
sun(0).
0.7::sunny(T, sun) :- step(T).
0.6::sunny(T, rain) :- step(T).
sky(T, sun) :- sun(T).
sky(T, rain) :- time(T), \\+sun(T).
sun(T1) :- step(T), T1 is T + 1, sky(T, W), sunny(T, W).
state(I, T, on) :- strike(I, T).
state(I, T, off) :- factory(I), time(T), \\+strike(I, T).
strike(I, T1) :- step(T), T1 is T + 1, state(I, T, S), sky(T, W), strikes(I, T, S, W).
some_off(T) :- state(_, T, off).
reached :- time(T), \\+some_off(T).
query(reached).
query(strike(1, 3)).
";
    let options = common::options(Strategy::Auto, Projection::default());
    let answers = |text: &str| {
        let run = common::run(text, &options).unwrap_or_else(|e| panic!("{text}: {e}"));
        let mut got: Vec<(String, f64)> = run
            .results
            .expect("no evidence")
            .iter()
            .map(|r| (r.to_string(), r.probability))
            .collect();
        got.sort_by(|a, b| a.0.cmp(&b.0));
        got
    };
    let original = answers(source);
    let expected: Vec<(&str, f64)> = original.iter().map(|(q, p)| (q.as_str(), *p)).collect();
    for place in 0..source.lines().count() {
        let renamed = source.replace("sun(", &format!("'$swsunny_{place}_0'("));
        common::assert_close(&renamed, &answers(&renamed), &expected, 1e-12);
    }
}

/// The ground program of a ladder of `n` steps, as its printed writing
/// lists it (rl_ladder's): the facts, then the rules of each step, `o1`
/// and `o2` its two states, `b(j)` a fair coin and `breaks(j)` a rare one;
/// with `seed`, the facts and rules in an order drawn from it. The queries
/// come last.
fn ladder(n: usize, seed: Option<u64>) -> String {
    let mut lines: Vec<String> = (0..n)
        .map(|j| format!("0.001::breaks({j})."))
        .chain((0..n).map(|j| format!("0.5::b({j}).")))
        .collect();
    lines.push("o2(1) :- \\+breaks(0), \\+b(0).".into());
    lines.push("o1(1) :- b(0), \\+breaks(0).".into());
    for k in 2..=n {
        let j = k - 1;
        lines.push(format!("o2({k}) :- o2({j}), \\+b({j}), \\+o1({j})."));
        lines.push(format!("o2({k}) :- o1({j}), \\+breaks({j}), \\+b({j})."));
        lines.push(format!("o1({k}) :- b({j}), o2({j})."));
        lines.push(format!("o1({k}) :- b({j}), o1({j}), \\+breaks({j})."));
    }
    if let Some(mut x) = seed {
        for i in (1..lines.len()).rev() {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            lines.swap(i, (x >> 33) as usize % (i + 1));
        }
    }
    lines.push(format!("query(o1({n}))."));
    lines.push(format!("query(o2({n}))."));
    lines.join("\n")
}

/// P(o1(n)) and P(o2(n)) of [`ladder`], from the joint of the two states
/// step by step: `o1` true before the first.
fn ladder_answers(n: usize) -> [f64; 2] {
    let (coin, breaks) = (0.5, 0.001);
    let mut p = [[0.0; 2]; 2];
    p[1][0] = 1.0;
    for _ in 0..n {
        let mut q = [[0.0; 2]; 2];
        for (o1, o2, b, br) in (0..16).map(|w| (w & 1 == 1, w & 2 == 2, w & 4 == 4, w & 8 == 8)) {
            let w = p[o1 as usize][o2 as usize]
                * if b { coin } else { 1.0 - coin }
                * if br { breaks } else { 1.0 - breaks };
            let next2 = !b && ((o2 && !o1) || (o1 && !br));
            let next1 = b && (o2 || (o1 && !br));
            q[next1 as usize][next2 as usize] += w;
        }
        p = q;
    }
    [p[1][0] + p[1][1], p[0][1] + p[1][1]]
}

/// The parity of the flips `f(j, B)` of each of two bits over `n` steps,
/// the loop unrolled: each step's rules carry its number, which leaves
/// the bit `B` the last argument of each atom.
fn unrolled_parity(n: usize) -> String {
    let mut s = String::from("w(0). w(1).\non(0, B) :- w(B).\n");
    for k in 1..=n {
        let j = k - 1;
        s += &format!("on({k}, B) :- on({j}, B), \\+f({j}, B).\n");
        s += &format!("on({k}, B) :- off({j}, B), f({j}, B).\n");
        s += &format!("off({k}, B) :- off({j}, B), \\+f({j}, B).\n");
        s += &format!("off({k}, B) :- on({j}, B), f({j}, B).\n");
    }
    for j in 0..n {
        s += &format!("0.1::f({j}, 0). 0.1::f({j}, 1).\n");
    }
    s + &format!("query(on({n}, 0)).\nquery(off({n}, 1)).\n")
}

/// A chain written step by step, each step's number in its rules: the
/// ground program of a ladder, as printed and with its clauses shuffled,
/// and a loop unrolled. Each grounds in walks linear in its steps: a
/// round of the grounder adds a step's few atoms and visits the rules
/// that read them, not every rule of their predicates. A ladder of 8 000
/// steps took 116 s to ground shuffled, against 0.26 s as printed, and a
/// loop of 3 000 steps 34 s unrolled, against 0.39 s as a recursion.
#[test]
fn a_chain_written_step_by_step_grounds_in_walks_linear_in_its_steps() {
    let n = 600;
    let walked: Vec<(&str, u64)> = [
        ("the ladder as printed", ladder(n, None)),
        ("the ladder shuffled", ladder(n, Some(7))),
        ("the parity unrolled", unrolled_parity(n)),
    ]
    .into_iter()
    .map(|(writing, source)| {
        (
            writing,
            common::ground(&source, Rows::Apart, false).stats.walked,
        )
    })
    .collect();
    assert!(
        walked.iter().all(|&(_, w)| w <= 100 * n as u64),
        "body literals walked: {walked:?}"
    );
}

#[test]
fn a_chain_written_step_by_step_answers_as_its_recurrence() {
    let n = 20;
    let [o1, o2] = ladder_answers(n);
    let states = [(format!("o1({n})"), o1), (format!("o2({n})"), o2)];
    let even = (1.0 + 0.8f64.powi(n as i32)) / 2.0;
    let parities = [
        (format!("on({n},0)"), even),
        (format!("off({n},1)"), 1.0 - even),
    ];
    for (writing, source, expected) in [
        ("the ladder as printed", ladder(n, None), &states),
        ("the ladder shuffled", ladder(n, Some(7)), &states),
        ("the parity unrolled", unrolled_parity(n), &parities),
    ] {
        for strategy in [Strategy::Auto, Strategy::Trial(Kind::Closed)] {
            let what = format!("{writing}, {strategy:?}");
            let run = common::run(&source, &common::options(strategy, Projection::default()))
                .unwrap_or_else(|e| panic!("{what}: {e}"));
            let results = run.results.expect("no evidence");
            assert_answers(&what, &results, expected, 1e-9);
        }
    }
}
