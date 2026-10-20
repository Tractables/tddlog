//! The memo of clause loops over facts alone (`ClauseMemo` in `src/tp`): a
//! loop over the same bodies in local variables on a scope of the same
//! shape is a copy of the first. These tests check that the copies are made
//! and that the answers are those of the program, computed here by
//! enumerating each block's worlds.

use crate::query::assert_answers;
use crate::tp::Formulation;
use crate::{Kind, Options, Projection, Strategy};

/// The options of a run under `strategy`, compiled under `projection`.
fn options(strategy: Strategy, projection: Projection) -> Options {
    Options {
        strategy,
        projection: Some(projection),
        ..Options::default()
    }
}

/// `source` ground and read through indicators ([`crate::indicators`]).
fn indicated(source: &str) -> crate::GroundProgram {
    let program = crate::syntax::parse_program(source).unwrap();
    crate::indicators::introduce(
        &crate::ground::ground(&program, &crate::GroundOptions::default()).unwrap(),
    )
}

const BLOCKS: usize = 5;
const FACTS: usize = 9;

fn prob(k: usize, j: usize) -> f64 {
    0.1 + 0.08 * j as f64 + 0.03 * k as f64
}

/// Whether body `j` (1 to 8) of block `k`'s chain holds in `world`, whose
/// bit `i - 1` is fact `i`: `x_j ∧ ¬x_{j+1}` for odd `j`, `x_j ∧ x_{j+1}`
/// for even `j`, the signs of `x_{j+1}` the other way round in block 4.
/// The chain is not symmetric under any renaming of its facts but the
/// identity, so a copy that read its variables in another order would have
/// another probability.
fn body(k: usize, world: u32, j: usize) -> bool {
    let x = |i: usize| world >> (i - 1) & 1 == 1;
    x(j) && (x(j + 1) == (j.is_multiple_of(2) != (k == 4)))
}

/// Block `k`'s head `h{k}` holds where some body of its chain holds, over
/// its own facts `x{k}_1` to `x{k}_9`. Block 3 lists its rules in reverse,
/// which orders its facts otherwise on `closed`'s vtree (by first use) and
/// puts its bodies on other variables of a scope of the same shape; block
/// 4's differ from block 0's in their signs alone. `both` is `h0 ∧ h1`,
/// `either` is `¬h2 ∧ h3`.
fn program() -> String {
    let mut s = String::new();
    for k in 0..BLOCKS {
        for j in 1..=FACTS {
            s += &format!("{}::x{k}_{j}.\n", prob(k, j));
        }
    }
    for k in 0..BLOCKS {
        let order: Vec<usize> = if k == 3 {
            (1..FACTS).rev().collect()
        } else {
            (1..FACTS).collect()
        };
        for j in order {
            let neg = if (j % 2 == 0) != (k == 4) { "" } else { "\\+" };
            s += &format!("h{k} :- x{k}_{j}, {neg}x{k}_{}.\n", j + 1);
        }
    }
    s += "both :- h0, h1.\neither :- \\+h2, h3.\n";
    for q in ["h0", "h1", "h2", "h3", "h4", "both", "either"] {
        s += &format!("query({q}).\n");
    }
    s
}

/// `P(h{k})`, summed over the 512 worlds of block `k`.
fn head(k: usize) -> f64 {
    (0..1u32 << FACTS)
        .filter(|&w| (1..FACTS).any(|j| body(k, w, j)))
        .map(|w| {
            (1..=FACTS)
                .map(|i| {
                    if w >> (i - 1) & 1 == 1 {
                        prob(k, i)
                    } else {
                        1.0 - prob(k, i)
                    }
                })
                .product::<f64>()
        })
        .sum()
}

fn expected() -> Vec<(&'static str, f64)> {
    let h: Vec<f64> = (0..BLOCKS).map(head).collect();
    vec![
        ("h0", h[0]),
        ("h1", h[1]),
        ("h2", h[2]),
        ("h3", h[3]),
        ("h4", h[4]),
        ("both", h[0] * h[1]),
        ("either", (1.0 - h[2]) * h[3]),
    ]
}

/// ProbLog 2.3 on `program()` prints h0: 0.93094177, h1: 0.94890906,
/// h2: 0.96373339, h3: 0.97568249, h4: 0.94683027, both: 0.88337908,
/// either: 0.035384695.
#[test]
fn copied_clause_loops_are_the_enumerated_answer() {
    let source = program();
    let want = expected();
    let run = crate::solve::run_source(
        &source,
        &options(Strategy::Trial(Kind::Closed), Projection::default()),
    )
    .unwrap();
    // h1 and h2 are copies of h0; h3's bodies are on other variables,
    // h4's of other signs.
    let memo = run.stats.clause_memo;
    assert_eq!((memo.lookups, memo.copies), (5, 2));
    assert_answers("closed", &run.results.expect("no evidence"), &want, 1e-9);
}

/// Without projection every loop is on the global scope, which is not
/// looked up.
#[test]
fn the_global_scope_is_not_looked_up() {
    let run = crate::solve::run_source(
        &program(),
        &options(Strategy::Trial(Kind::Closed), Projection::Off),
    )
    .unwrap();
    assert_eq!(run.stats.clause_memo.lookups, 0);
    assert_answers(
        "closed, unprojected",
        &run.results.expect("no evidence"),
        &expected(),
        1e-9,
    );
}

/// A table `x` over parents `p` and `q`, read by `y`, `z` and `w`, each
/// with a parent of its own: `x`'s clauses tie it to each reader's copies,
/// one loop per reader ([`crate::indicators`], `Tables::Clauses`).
/// ProbLog 2.3 prints p(b): 0.48823052, w(t): 0.40056412,
/// x(a): 0.47605519, z(t): 0.5661247.
fn three_readers() -> (String, [(&'static str, f64); 4]) {
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
    let root = [0.35, 0.65];
    let readers = [
        ("y", "r", [0.9, 0.4, 0.2, 0.3, 0.8, 0.5]),
        ("z", "s", [0.3, 0.8, 0.5, 0.6, 0.1, 0.7]),
        ("w", "u", [0.6, 0.1, 0.7, 0.2, 0.9, 0.4]),
    ];
    let mut s = String::from("0.2::p(a); 0.5::p(b); 0.3::p(c).\n0.6::q(a); 0.4::q(b).\n");
    for (i, pv) in ["a", "b", "c"].iter().enumerate() {
        for (j, qv) in ["a", "b"].iter().enumerate() {
            let [a, b, c] = x[2 * i + j];
            s += &format!("{a}::x(a); {b}::x(b); {c}::x(c) :- p({pv}), q({qv}).\n");
        }
    }
    for (name, parent, t) in &readers {
        s += &format!("0.35::{parent}(a); 0.65::{parent}(b).\n");
        for (i, xv) in ["a", "b", "c"].iter().enumerate() {
            for (j, rv) in ["a", "b"].iter().enumerate() {
                let pt = t[2 * i + j];
                s += &format!(
                    "{pt}::{name}(t); {}::{name}(f) :- x({xv}), {parent}({rv}).\n",
                    1.0 - pt
                );
            }
        }
    }
    s += "evidence(y(t)).\nquery(z(t)). query(w(t)). query(x(a)). query(p(b)).\n";
    // Every world of p, q, x and the readers' parents, given y(t).
    let (mut e, mut z, mut w, mut xa, mut pb) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (i, pi) in p.iter().enumerate() {
        for (j, qj) in q.iter().enumerate() {
            for (k, xk) in x[2 * i + j].iter().enumerate() {
                for (a, ra) in root.iter().enumerate() {
                    for (b, sb) in root.iter().enumerate() {
                        for (c, uc) in root.iter().enumerate() {
                            let joint = pi * qj * xk * ra * sb * uc * readers[0].2[2 * k + a];
                            e += joint;
                            z += joint * readers[1].2[2 * k + b];
                            w += joint * readers[2].2[2 * k + c];
                            xa += joint * f64::from(u8::from(k == 0));
                            pb += joint * f64::from(u8::from(i == 1));
                        }
                    }
                }
            }
        }
    }
    (
        s,
        [
            ("z(t)", z / e),
            ("w(t)", w / e),
            ("x(a)", xa / e),
            ("p(b)", pb / e),
        ],
    )
}

/// The vtree `(L, (X, R))`: `x`'s switches and its copies of `p` and `q`
/// right-linear in `X`, `z`'s copies of `x` right-linear first in `L`, `y`'s
/// and `w`'s first in `R`, the other facts after them. Each reader's loop
/// is on the projection to `X` and its copies: `z`'s has its copies on the
/// left of `X`, `y`'s and `w`'s on the right, the same shape up to that
/// mirror, and `z`'s copies come first in the variables' order where the
/// others' come last.
///
/// With `inside`, the vtree is `((X1, (L, X2)), R)` instead, `X` split
/// after its eighth variable: `z`'s loop has its copies among `x`'s inputs,
/// a shape no mirror of the others' makes.
fn readers_vtree(gp: &crate::ground::GroundProgram, inside: bool) -> String {
    let facts = crate::vtree::facts_in_order(gp);
    let name = |a: u32| gp.atoms[a as usize].to_string();
    let copy_of_x = |a: u32, reader: &str| {
        name(a).starts_with("'$v'('x(") && name(a).ends_with(&format!("'{reader}(t)')"))
    };
    let in_x = |a: u32| {
        name(a).starts_with("'$swx_")
            || (name(a).starts_with("'$v'(") && name(a).ends_with("'x(a)')"))
    };
    // Variable `i + 1` is `facts[i]`.
    let vars = |keep: &dyn Fn(u32) -> bool| {
        facts
            .iter()
            .enumerate()
            .filter(|&(_, &a)| keep(a))
            .map(|(i, _)| i + 1)
            .collect::<Vec<_>>()
    };
    let left = vars(&|a| copy_of_x(a, "z"));
    let middle = vars(&|a| in_x(a));
    let right: Vec<usize> = vars(&|a| copy_of_x(a, "y"))
        .into_iter()
        .chain(vars(&|a| copy_of_x(a, "w")))
        .chain(vars(&|a| {
            !in_x(a) && !["y", "z", "w"].iter().any(|r| copy_of_x(a, r))
        }))
        .collect();
    assert_eq!(
        (
            left.len(),
            middle.len(),
            left.len() + middle.len() + right.len()
        ),
        (3, 17, facts.len())
    );
    let mut lines: Vec<String> = Vec::new();
    let mut node = |line: String| {
        lines.push(line);
        lines.len() - 1
    };
    let right_linear = |vars: &[usize], node: &mut dyn FnMut(String) -> usize| {
        let leaves: Vec<usize> = vars.iter().map(|v| node(format!("L {{}} {v}"))).collect();
        leaves
            .into_iter()
            .rev()
            .reduce(|below, leaf| node(format!("I {{}} {leaf} {below}")))
            .unwrap()
    };
    let l = right_linear(&left, &mut node);
    let r = right_linear(&right, &mut node);
    if inside {
        let x1 = right_linear(&middle[..8], &mut node);
        let x2 = right_linear(&middle[8..], &mut node);
        let lx2 = node(format!("I {{}} {l} {x2}"));
        let x = node(format!("I {{}} {x1} {lx2}"));
        node(format!("I {{}} {x} {r}"));
    } else {
        let m = right_linear(&middle, &mut node);
        let mr = node(format!("I {{}} {m} {r}"));
        node(format!("I {{}} {l} {mr}"));
    }
    let body: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(id, line)| line.replacen("{}", &id.to_string(), 1))
        .collect();
    format!("vtree {}\n{}\n", body.len(), body.join("\n"))
}

/// `gp` compiled on the vtree `text`, whose variable `i + 1` is the `i`-th
/// fact in program order, under the dual with projection, as a static
/// vtree is compiled: the clause memo's counts and the answers.
fn on_vtree(
    gp: &crate::ground::GroundProgram,
    text: &str,
) -> (crate::tp::ClauseMemoStats, Vec<crate::QueryResult>) {
    let vtree = std::sync::Arc::new(tididi::Vtree::from_text(text).unwrap());
    let order = crate::vtree::facts_in_order(gp);
    vtree.context().run(|engine| {
        let backend = crate::tp::TddBackend::new(engine, vtree.clone());
        let var_of = crate::tp::fact_variables(gp, &order);
        let compiled = crate::tp::compile(
            gp,
            var_of,
            Formulation::Dual,
            &backend,
            Projection::default(),
            None,
        )
        .unwrap();
        let (results, _) = crate::query::query_probabilities(gp, &backend, &compiled).unwrap();
        (
            compiled.stats.clause_memo,
            results.expect("evidence and queries"),
        )
    })
}

/// The three readers' loops are one set of clauses with the copies
/// renamed, each link stating that the copies of each of `x`'s parents are
/// exactly one. `z`'s copies are numbered first among the facts and sit on
/// the other side of `x`'s inputs, `y`'s and `w`'s last: the first loop is
/// built and the other two are copies of it, `z`'s mirrored. The answers
/// are the network's.
///
/// Loops of two bodies and more are looked up, so the counts hold the
/// program's smaller loops too: the readers' own tables, six bodies each,
/// one built and two copied, but on the mirrored vtree `z`'s is of another
/// shape and built; a loop of four bodies built and copied three times;
/// and one of seven. Those are 8 lookups and 5 copies (4 on the mirrored
/// vtree) besides the readers' 3 and 2.
#[test]
fn the_readers_of_one_table_copy_one_loop_on_either_side() {
    let (source, want) = three_readers();
    let clauses = indicated(&source);
    let vtree = readers_vtree(&clauses, false);
    let families = Strategy::Trial(Kind::Families { search: true });
    let run = crate::solve_ground(clauses.clone(), &options(families, Projection::default()))
        .unwrap_or_else(|e| panic!("families: {e}"));
    let runs = [
        ("mirrored", on_vtree(&clauses, &vtree), 2 + 4),
        (
            "families",
            (
                run.stats.clause_memo,
                run.results.expect("evidence and queries"),
            ),
            2 + 5,
        ),
    ];
    for (vtree, (memo, got), copies) in runs {
        assert_eq!((memo.lookups, memo.copies), (3 + 8, copies), "{vtree}");
        assert_answers(vtree, &got, &want, 1e-12);
    }
}

/// `z`'s copies among `x`'s inputs: one loop is built, one is copied onto
/// the same shape, and the one of the other shape is built and kept. The
/// smaller loops add 8 lookups, 4 copies, `z`'s own table built and kept,
/// and the three loops built (see above). The answers are the network's.
#[test]
fn a_reader_of_another_shape_builds_its_loop() {
    let (source, want) = three_readers();
    let clauses = indicated(&source);
    let (memo, got) = on_vtree(&clauses, &readers_vtree(&clauses, true));
    assert_eq!(
        (memo.lookups, memo.copies, memo.kept),
        (3 + 8, 1 + 4, 2 + 4)
    );
    assert_answers("inside", &got, &want, 1e-12);
}
