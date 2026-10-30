//! P(e), the probability of a program's evidence (`Run::evidence_probability`,
//! `--evidence-probability`), against ProbLog 2.3.0's: the program with its
//! evidence written as one rule `e :- l1, ..., ln` and `e` queried without
//! evidence, read from its Python API at 17 digits; or against P(e) by
//! hand. Conditioning on the evidence (src/ground/condition.rs) fixes facts
//! and merges equal ones out of the program, and what it takes out must
//! come back as `GroundProgram::weight`; detaching (src/ground/detach.rs)
//! drops observed parts no query reads, which `GroundOptions::detach` off
//! keeps in the count. Every strategy, with and without indicators, must
//! agree.

use crate::common;

use common::Rows;
use tddlog::eval::Count;
use tddlog::ground::GroundOptions;
use tddlog::{Kind, Projection, Strategy};

/// The strategies, projections and rewrites each program is run under:
/// each strategy's own (as the binary runs it), the cones of every
/// component and none, and the rows grounded apart under `families` and,
/// read through every indicator, under `closed`, which compiles the
/// indicators as they are.
fn configurations() -> Vec<(Strategy, Projection, bool, Rows)> {
    use Kind::*;
    let cones = Projection::default();
    let mut out: Vec<_> = [
        Strategy::Trial(Closed),
        Strategy::Auto,
        Strategy::Trial(Td),
        Strategy::Trial(Families { search: true }),
        Strategy::Trial(Families { search: false }),
    ]
    .into_iter()
    .map(|s| (s, cones, false, Rows::Classes))
    .collect();
    out.push((
        Strategy::Trial(Dynamic),
        Projection::Off,
        false,
        Rows::Classes,
    ));
    out.push((
        Strategy::Trial(Closed),
        Projection::Off,
        false,
        Rows::Classes,
    ));
    out.push((
        Strategy::Trial(Closed),
        Projection::Cones { max_share: 1.0 },
        false,
        Rows::Classes,
    ));
    out.push((
        Strategy::Trial(Families { search: true }),
        cones,
        false,
        Rows::Apart,
    ));
    out.push((Strategy::Trial(Closed), cones, true, Rows::Apart));
    out
}

/// P(e) of `source` under each configuration, as `--evidence-probability`
/// grounds it, nothing detached; `families` on a program that reads no
/// atom through an indicator is left out, which the layout refuses.
fn evidence_probabilities(source: &str) -> Vec<(String, Count, bool)> {
    evidence_probabilities_of(&tddlog::syntax::parse_program(source).unwrap())
}

/// [`evidence_probabilities`] of a parsed program.
fn evidence_probabilities_of(program: &tddlog::syntax::Program) -> Vec<(String, Count, bool)> {
    let defaults = GroundOptions::default();
    evidence_probabilities_with(
        program,
        defaults.noisy_or,
        defaults.backbones,
        defaults.defined,
    )
}

/// [`evidence_probabilities_of`] with the noisy-or tables written as
/// noisy-ors or not (`GroundOptions::noisy_or`), the conditioning's
/// search for backbones on or off (`GroundOptions::backbones`), and the
/// elimination of defined facts on or off (`GroundOptions::defined`).
fn evidence_probabilities_with(
    program: &tddlog::syntax::Program,
    noisy_or: bool,
    backbones: bool,
    defined: bool,
) -> Vec<(String, Count, bool)> {
    let mut out = Vec::new();
    for (strategy, projection, indicators, rows) in configurations() {
        let what = format!(
            "{strategy:?}, {projection}, indicators {indicators}, rows {rows:?}, noisy_or {noisy_or}, backbones {backbones}, defined {defined}"
        );
        let options = GroundOptions {
            detach: false,
            noisy_or,
            backbones,
            defined,
            ..rows.options()
        };
        let ground = tddlog::ground::ground(program, &options).unwrap();
        assert_eq!(ground.stats.detached, 0, "{what}: nothing detached");
        let ground = if indicators {
            tddlog::indicators::introduce(&ground)
        } else {
            ground
        };
        let run = match tddlog::solve_ground(ground, &common::options(strategy, projection)) {
            Err(tddlog::Error::Vtree(e)) if e.contains("reads no atom through one") => continue,
            other => other.unwrap_or_else(|e| panic!("{what}: {e}")),
        };
        let p = run
            .evidence_probability
            .unwrap_or_else(|| panic!("{what}: no P(e)"));
        out.push((what, p, run.results.is_some()));
    }
    assert!(out.len() >= 7, "every configuration but families ran");
    out
}

/// Asserts that every configuration gives `source` the P(e) `want`, within
/// 1e-12 relative.
fn check(name: &str, source: &str, want: f64) {
    check_program(name, &tddlog::syntax::parse_program(source).unwrap(), want);
}

/// [`check`] of a parsed program.
fn check_program(name: &str, program: &tddlog::syntax::Program, want: f64) {
    for (what, p, consistent) in evidence_probabilities_of(program) {
        let got = p.log10();
        let close = if want == 0.0 {
            got == f64::NEG_INFINITY
        } else {
            (got - want.log10()).abs() < 1e-12 / std::f64::consts::LN_10
        };
        assert!(
            close,
            "{name} ({what}): P(e) = {} against {want}",
            p.to_f64()
        );
        assert_eq!(
            consistent,
            want > 0.0,
            "{name} ({what}): answers given exactly when P(e) > 0"
        );
    }
}

fn read(path: &str) -> String {
    std::fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn examples_against_problog() {
    for (file, want) in [
        ("alarm.pl", 0.196),
        ("bn_split.pl", 0.44705),
        ("shared_rows.pl", 0.45),
        ("bn_asia_evidence.pl", 0.0396199356),
        ("dice.pl", 1.0 / 36.0),
        ("equal_atoms.pl", 0.3),
        ("row_classes.pl", 0.572),
        ("dice_murder.pl", 0.569),
        ("linear_path.pl", 0.7150368),
        ("dice_evidence.pl", 0.75),
        ("query_beside_evidence.pl", 0.1),
    ] {
        check(file, &read(&format!("examples/{file}")), want);
    }
    check(
        "noisy_or_shared.pl",
        &read("tests/regress/noisy_or_shared.pl"),
        3.2900206604829484e-05,
    );
}

#[test]
fn noisy_or_tables_are_counted_as_noisy_ors() {
    // tests/regress/noisy_or_chain.pl: d1 to d6, x1, x2 and y1 are
    // noisy-ors whose absent value nothing reads or asks, and f1, f2 and
    // f4, observed present, are noisy-ors whose parents are written too.
    // So is f3, observed absent: observed at its present value, false.
    // Thirteen in all. Written as noisy-ors or as tables, P(e) and the
    // answers are ProbLog's.
    let source = read("tests/regress/noisy_or_chain.pl");
    let program = tddlog::syntax::parse_program(&source).unwrap();
    let ground = tddlog::ground::ground(&program, &GroundOptions::default()).unwrap();
    assert_eq!(ground.stats.noisy_or, 13, "tables written as noisy-ors");
    let off = GroundOptions {
        noisy_or: false,
        ..GroundOptions::default()
    };
    assert_eq!(
        tddlog::ground::ground(&program, &off)
            .unwrap()
            .stats
            .noisy_or,
        0,
        "none with the rewrite off"
    );
    let want: f64 = 0.0048892872971448;
    for noisy_or in [true, false] {
        for (what, p, _) in evidence_probabilities_with(
            &program,
            noisy_or,
            GroundOptions::default().backbones,
            GroundOptions::default().defined,
        ) {
            assert!(
                (p.log10() - want.log10()).abs() < 1e-12,
                "noisy_or_chain.pl ({what}): P(e) = {} against {want}",
                p.to_f64()
            );
        }
        let options = GroundOptions {
            noisy_or,
            ..GroundOptions::default()
        };
        let auto = Strategy::Auto;
        let ground = tddlog::ground::ground(&program, &options).unwrap();
        let run =
            tddlog::solve_ground(ground, &common::options(auto, Projection::default())).unwrap();
        let want = [
            ("d2(v_1)", 0.35459094930658464),
            ("d4(v_1)", 0.5876331336451859),
            ("d5(v_1)", 0.19447122066033104),
        ];
        common::assert_answers(
            &format!("auto, noisy_or {noisy_or}"),
            &run.results.unwrap(),
            &want,
            1e-12,
        );
    }
}

#[test]
fn without_queries_the_evidence_alone_is_ground_and_counted() {
    // `--no-queries`: the program ground for its evidence alone has the
    // same P(e), ProbLog's as above. On alarm, calls(mary) and
    // hears_alarm(mary), which only the query reads, are left out; on
    // bn_asia_evidence nothing is (both leaves are observed); on
    // query_beside_evidence the queried parts, on detach.pl q, and on
    // condition_bn everything, the conditioning having settled every
    // observation into its weight. by_query_alone below is a
    // network whose unobserved child c only the query reads: P(b) = 0.3 ·
    // 0.6 + 0.7 · 0.1 = 0.25 by hand.
    let by_query_alone = "0.3::a.\n0.6::b :- a.\n0.1::b :- \\+a.\n0.8::c :- b.\n0.2::c :- \\+b.\nevidence(b).\nquery(c).\n";
    for (file, want, fewer) in [
        ("examples/alarm.pl", 0.196, true),
        ("examples/bn_asia_evidence.pl", 0.0396199356, false),
        ("examples/dice_murder.pl", 0.569, false),
        ("examples/query_beside_evidence.pl", 0.1, true),
        ("tests/regress/condition_bn.pl", 0.12, true),
        ("tests/regress/detach.pl", 0.82 * 0.5 * 0.75, true),
        ("by_query_alone", 0.25, true),
    ] {
        let source = if file == "by_query_alone" {
            by_query_alone.to_string()
        } else {
            read(file)
        };
        let program = tddlog::syntax::parse_program(&source).unwrap();
        let options = GroundOptions {
            detach: false,
            ..GroundOptions::default()
        };
        let atoms = tddlog::ground::ground(&program, &options)
            .unwrap()
            .num_atoms();
        let program = program.without_queries();
        let ground = tddlog::ground::ground(&program, &options).unwrap();
        assert!(
            ground.to_string().lines().all(|l| !l.starts_with("query(")),
            "{file}: no query left"
        );
        assert_eq!(
            ground.num_atoms() < atoms,
            fewer,
            "{file}: {} atoms against {atoms} with the queries",
            ground.num_atoms()
        );
        check_program(file, &program, want);
    }
}

#[test]
fn what_conditioning_takes_out_comes_back() {
    // condition_bn fixes facts and condition_equal merges equal facts:
    // ProbLog 0.12 and 0.26 (by hand in its comment, 0.036 + 0.224);
    // condition_copies merges two facts of a CNF and a clause reads the
    // copy, 0.33 by hand in its comment; condition_copies_units does so in
    // one of two units, 0.2672 by hand.
    // condition_cyclic, which conditioning leaves as it is, 0.68 by hand
    // and by ProbLog; alias_evidence, ProbLog 0.25; condition_settled,
    // which unit propagation alone settles, 0.12 by hand in its comment.
    for (file, want, fixed) in [
        ("condition_bn.pl", 0.12, true),
        ("condition_settled.pl", 0.12, true),
        ("condition_equal.pl", 0.26, true),
        ("condition_copies.pl", 0.33, true),
        ("condition_copies_units.pl", 0.2672, true),
        ("condition_cyclic.pl", 0.68, false),
        ("alias_evidence.pl", 0.25, false),
    ] {
        let source = read(&format!("tests/regress/{file}"));
        if fixed {
            let program = tddlog::syntax::parse_program(&source).unwrap();
            let ground = tddlog::ground::ground(&program, &GroundOptions::default()).unwrap();
            assert!(
                ground.stats.conditioned.any() && ground.weight.to_f64() < 1.0,
                "{file}: conditioning takes mass out"
            );
        }
        check(file, &source, want);
    }
}

#[test]
fn a_gate_takes_out_one_half() {
    // Facts of probability one half that the clauses define as an AND or
    // an OR of other facts' literals become derived atoms, and summing each
    // out leaves one half, which goes into `GroundProgram::weight`
    // (src/ground/condition.rs, gates). ProbLog 2.3.0's P(e), as counted by
    // hand in the programs' comments: gate_not_half's fact is left alone,
    // and so are the gates something else reads (gate_and_read,
    // gate_nested, gate_cyclic, gate_read_by_clause).
    for (file, want) in [
        ("gate_or_unread.pl", 0.22),
        ("gate_and_read.pl", 0.192),
        ("gate_nested.pl", 0.0709),
        ("gate_nested_unread.pl", 0.11),
        ("gate_not_half.pl", 0.1952),
        ("gate_cyclic.pl", 0.355),
        ("gate_read_by_clause.pl", 0.262),
    ] {
        check(file, &read(&format!("tests/regress/{file}")), want);
    }
}

#[test]
fn the_backbone_search_keeps_the_count() {
    // backbone_clauses.pl: c holds in every world the clauses allow, which
    // unit propagation does not find, and with c fixed d and e are equal.
    // ProbLog 2.3.0's P(e) 0.09384, by hand in its comment.
    let source = read("tests/regress/backbone_clauses.pl");
    check("backbone_clauses.pl", &source, 0.09384);
    let program = tddlog::syntax::parse_program(&source).unwrap();
    let conditioned = |backbones: bool| {
        tddlog::ground::ground(
            &program,
            &GroundOptions {
                detach: false,
                backbones,
                ..GroundOptions::default()
            },
        )
        .unwrap()
        .stats
        .conditioned
    };
    let (on, off) = (conditioned(true), conditioned(false));
    assert_eq!(
        (on.facts, on.merged, on.backbones.map(|b| b.fixed)),
        (1, 1, Some(1)),
        "{on:?}"
    );
    assert_eq!(
        (off.facts, off.merged, off.backbones),
        (0, 0, None),
        "{off:?}"
    );
    for (what, p, _) in evidence_probabilities_with(
        &program,
        GroundOptions::default().noisy_or,
        false,
        GroundOptions::default().defined,
    ) {
        assert!(
            (p.log10() - 0.09384f64.log10()).abs() < 1e-12,
            "{what}: P(e) = {}",
            p.to_f64()
        );
    }
}

#[test]
fn weighted_cnfs_count_the_same_with_and_without_backbones() {
    // Random weighted CNFs as programs (each variable a fact, each clause j
    // a rule `violated(j)` observed false), each with a planted backbone
    // that unit propagation does not find (a or b, a implies c, b implies
    // c), an equality that holds once c does (d equals e where c holds) and
    // clauses that c then shortens: P(e) by enumeration of the worlds,
    // against every configuration with the backbone search on and off.
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let probabilities = [0.5, 0.3, 0.7, 0.2, 0.9, 0.5];
    let (mut fixed, mut merged) = (0, 0);
    for round in 0..24 {
        let n = 10;
        let p: Vec<f64> = (0..n)
            .map(|_| probabilities[(next() % 6) as usize])
            .collect();
        let lit = |v: u64, positive: bool| (v as i64 + 1) * if positive { 1 } else { -1 };
        let r = next() % n;
        let (a, b, c, d, e) = (r, (r + 1) % n, (r + 2) % n, (r + 3) % n, (r + 4) % n);
        let mut clauses: Vec<Vec<i64>> = vec![
            vec![lit(a, true), lit(b, true)],
            vec![lit(a, false), lit(c, true)],
            vec![lit(b, false), lit(c, true)],
            vec![lit(d, true), lit(e, false), lit(c, false)],
            vec![lit(d, false), lit(e, true), lit(c, false)],
        ];
        let m = 8 + next() % 10;
        for k in 0..m {
            let width = 2 + next() % 2;
            let mut clause: Vec<i64> = (0..width)
                .map(|_| lit(next() % n, next() & 1 == 1))
                .collect();
            if k % 3 == 0 {
                // Shortened to two literals once c is fixed true.
                clause.push(lit(c, false));
            }
            clauses.push(clause);
        }
        let holds = |w: u64, x: i64| (w >> (x.unsigned_abs() - 1) & 1 == 1) == (x > 0);
        let z: f64 = (0..1u64 << n)
            .filter(|&w| clauses.iter().all(|cl| cl.iter().any(|&x| holds(w, x))))
            .map(|w| {
                (0..n as usize)
                    .map(|v| if w >> v & 1 == 1 { p[v] } else { 1.0 - p[v] })
                    .product::<f64>()
            })
            .sum();
        let mut source = String::new();
        for (v, q) in p.iter().enumerate() {
            source.push_str(&format!("{q}::x({}).\n", v + 1));
        }
        for (j, cl) in clauses.iter().enumerate() {
            let body: Vec<String> = cl
                .iter()
                .map(|&x| {
                    if x > 0 {
                        format!("\\+x({x})")
                    } else {
                        format!("x({})", -x)
                    }
                })
                .collect();
            source.push_str(&format!(
                "violated({j}) :- {}.\nevidence(violated({j}), false).\n",
                body.join(", ")
            ));
        }
        let program = tddlog::syntax::parse_program(&source).unwrap();
        let conditioned = |backbones: bool| {
            tddlog::ground::ground(
                &program,
                &GroundOptions {
                    detach: false,
                    backbones,
                    ..GroundOptions::default()
                },
            )
            .unwrap()
            .stats
            .conditioned
        };
        let (on, off) = (conditioned(true), conditioned(false));
        fixed += on.backbones.map_or(0, |b| b.fixed);
        merged += usize::from(on.merged > 0 && off.merged == 0);
        for backbones in [true, false] {
            for (what, got, consistent) in evidence_probabilities_with(
                &program,
                GroundOptions::default().noisy_or,
                backbones,
                GroundOptions::default().defined,
            ) {
                if z == 0.0 {
                    assert!(
                        !consistent && got.to_f64() == 0.0,
                        "round {round} ({what}): P(e) = {} against 0\n{source}",
                        got.to_f64()
                    );
                } else {
                    assert!(
                        (got.log10() - z.log10()).abs() < 1e-10,
                        "round {round} ({what}): P(e) = {} against {z}\n{source}",
                        got.to_f64()
                    );
                }
            }
        }
    }
    assert!(
        fixed > 0 && merged > 0,
        "the search fixed {fixed} facts, and made the conditioning merge facts in {merged} rounds where it merged none"
    );
}

#[test]
fn defined_facts_take_out_one_half_each() {
    // tests/regress/defined_facts.pl: c is the xor of a and b and d the
    // and of c and f, both of probability one half and read by other
    // clauses; ProbLog 2.3.0's P(e) 0.14 and answers a 0.45, c 0.57857143,
    // g 0.69285714, by hand in its comment. Without the queries, an
    // independent support leaves c and d out and both are eliminated by
    // resolution; with them, the query keeps c, and d alone goes. P(e) and
    // the answers are the same with the elimination off.
    let source = read("tests/regress/defined_facts.pl");
    let program = tddlog::syntax::parse_program(&source).unwrap();
    let alone = program.clone().without_queries();
    let options = |defined: bool| GroundOptions {
        detach: false,
        defined,
        ..GroundOptions::default()
    };
    for (prog, eliminated, clauses) in [(&alone, 2, 3), (&program, 1, 6)] {
        let ground = tddlog::ground::ground(prog, &options(true)).unwrap();
        let defined = ground.stats.defined.expect("a CNF over facts");
        assert_eq!(
            (defined.eliminated, defined.clauses, defined.left),
            (eliminated, 9, clauses),
            "{defined:?}"
        );
        assert_eq!(ground.num_facts(), 6 - eliminated);
        assert_eq!(
            tddlog::ground::ground(prog, &options(false))
                .unwrap()
                .stats
                .defined,
            None
        );
        for defined in [true, false] {
            for (what, p, _) in evidence_probabilities_with(
                prog,
                GroundOptions::default().noisy_or,
                GroundOptions::default().backbones,
                defined,
            ) {
                assert!(
                    (p.log10() - 0.14f64.log10()).abs() < 1e-12,
                    "defined_facts.pl ({what}): P(e) = {}",
                    p.to_f64()
                );
            }
        }
    }
    let want = [("a", 0.45), ("c", 0.57857143), ("g", 0.69285714)];
    for strategy in [
        Strategy::Auto,
        Strategy::Trial(Kind::Td),
        Strategy::Trial(Kind::Closed),
    ] {
        for defined in [true, false] {
            let ground = tddlog::ground::ground(
                &program,
                &GroundOptions {
                    defined,
                    ..GroundOptions::default()
                },
            )
            .unwrap();
            let run =
                tddlog::solve_ground(ground, &common::options(strategy, Projection::default()))
                    .unwrap();
            common::assert_answers(
                &format!("{strategy:?}, defined {defined}"),
                &run.results.unwrap(),
                &want,
                1e-7,
            );
        }
    }
}

#[test]
fn a_clause_the_others_imply_leaves_before_the_support() {
    // tests/regress/implied_clause.pl: k(1) and k(2) imply k(3), a clause
    // of four literals, which leaves (src/ground/preprocess/implied.rs), and e and d
    // with it where nothing else reads them; no fact is eliminated. P(e)
    // 0.42 by hand in its comment, and ProbLog 2.3.0's answers c
    // 0.85714286, d 0.8. P(e) and the answers are the same with the stage
    // off.
    let source = read("tests/regress/implied_clause.pl");
    let program = tddlog::syntax::parse_program(&source).unwrap();
    let alone = program.clone().without_queries();
    let options = |defined: bool| GroundOptions {
        detach: false,
        defined,
        ..GroundOptions::default()
    };
    for (prog, facts) in [(&alone, 3), (&program, 4)] {
        let ground = tddlog::ground::ground(prog, &options(true)).unwrap();
        let defined = ground.stats.defined.expect("a CNF over facts");
        assert_eq!(
            (
                defined.eliminated,
                defined.implied.tried,
                defined.implied.implied,
                defined.clauses,
                defined.left
            ),
            (0, 1, 1, 3, 2),
            "{defined:?}"
        );
        assert_eq!(ground.num_facts(), facts);
        for defined in [true, false] {
            for (what, p, _) in evidence_probabilities_with(
                prog,
                GroundOptions::default().noisy_or,
                GroundOptions::default().backbones,
                defined,
            ) {
                assert!(
                    (p.log10() - 0.42f64.log10()).abs() < 1e-12,
                    "implied_clause.pl ({what}): P(e) = {}",
                    p.to_f64()
                );
            }
        }
    }
    let want = [("c", 0.85714286), ("d", 0.8)];
    for strategy in [
        Strategy::Auto,
        Strategy::Trial(Kind::Td),
        Strategy::Trial(Kind::Closed),
    ] {
        for defined in [true, false] {
            let ground = tddlog::ground::ground(
                &program,
                &GroundOptions {
                    defined,
                    ..GroundOptions::default()
                },
            )
            .unwrap();
            let run =
                tddlog::solve_ground(ground, &common::options(strategy, Projection::default()))
                    .unwrap();
            common::assert_answers(
                &format!("{strategy:?}, defined {defined}"),
                &run.results.unwrap(),
                &want,
                1e-7,
            );
        }
    }
}

#[test]
fn weighted_cnfs_count_the_same_with_and_without_eliminating_defined_facts() {
    // Random weighted CNFs as programs (each variable a fact, each clause j
    // a rule `violated(j)` observed false), with facts of probability one
    // half defined as the and, or or xor of two others and read by other
    // clauses besides, which the conditioning's gates leave alone: P(e) by
    // enumeration of the worlds, against every configuration with the
    // elimination of defined facts on and off.
    let mut state = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let probabilities = [0.3, 0.7, 0.2, 0.9, 0.5, 0.5];
    let mut eliminated = 0;
    for round in 0..24 {
        let n = 10u64;
        let mut p: Vec<f64> = (0..n)
            .map(|_| probabilities[(next() % 6) as usize])
            .collect();
        let lit = |v: u64, positive: bool| (v as i64 + 1) * if positive { 1 } else { -1 };
        let mut clauses: Vec<Vec<i64>> = Vec::new();
        for g in 0..3 {
            // Fact `y` of one half defined over `a` and `b`, after it.
            let y = 3 * g;
            let (a, b) = (
                y + 1 + next() % (n - y - 1),
                (y + 2 + next() % (n - y - 2)) % n,
            );
            if a == b {
                continue;
            }
            p[y as usize] = 0.5;
            let (yt, yf, at, af, bt, bf) = (
                lit(y, true),
                lit(y, false),
                lit(a, true),
                lit(a, false),
                lit(b, true),
                lit(b, false),
            );
            clauses.extend(match next() % 3 {
                0 => vec![vec![yf, at], vec![yf, bt], vec![yt, af, bf]],
                1 => vec![vec![yt, af], vec![yt, bf], vec![yf, at, bt]],
                _ => vec![
                    vec![yf, at, bt],
                    vec![yf, af, bf],
                    vec![yt, af, bt],
                    vec![yt, at, bf],
                ],
            });
        }
        for _ in 0..4 + next() % 6 {
            let width = 2 + next() % 2;
            clauses.push(
                (0..width)
                    .map(|_| lit(next() % n, next() & 1 == 1))
                    .collect(),
            );
        }
        let holds = |w: u64, x: i64| (w >> (x.unsigned_abs() - 1) & 1 == 1) == (x > 0);
        let z: f64 = (0..1u64 << n)
            .filter(|&w| clauses.iter().all(|cl| cl.iter().any(|&x| holds(w, x))))
            .map(|w| {
                (0..n as usize)
                    .map(|v| if w >> v & 1 == 1 { p[v] } else { 1.0 - p[v] })
                    .product::<f64>()
            })
            .sum();
        let mut source = String::new();
        for (v, q) in p.iter().enumerate() {
            source.push_str(&format!("{q}::x({}).\n", v + 1));
        }
        for (j, cl) in clauses.iter().enumerate() {
            let body: Vec<String> = cl
                .iter()
                .map(|&x| {
                    if x > 0 {
                        format!("\\+x({x})")
                    } else {
                        format!("x({})", -x)
                    }
                })
                .collect();
            source.push_str(&format!(
                "violated({j}) :- {}.\nevidence(violated({j}), false).\n",
                body.join(", ")
            ));
        }
        let program = tddlog::syntax::parse_program(&source).unwrap();
        let ground = tddlog::ground::ground(
            &program,
            &GroundOptions {
                detach: false,
                ..GroundOptions::default()
            },
        )
        .unwrap();
        eliminated += ground.stats.defined.map_or(0, |d| d.eliminated);
        for defined in [true, false] {
            for (what, got, consistent) in evidence_probabilities_with(
                &program,
                GroundOptions::default().noisy_or,
                GroundOptions::default().backbones,
                defined,
            ) {
                if z == 0.0 {
                    assert!(
                        !consistent && got.to_f64() == 0.0,
                        "round {round} ({what}): P(e) = {} against 0\n{source}",
                        got.to_f64()
                    );
                } else {
                    assert!(
                        (got.log10() - z.log10()).abs() < 1e-10,
                        "round {round} ({what}): P(e) = {} against {z}\n{source}",
                        got.to_f64()
                    );
                }
            }
        }
    }
    assert!(
        eliminated >= 20,
        "facts eliminated over the rounds: {eliminated}"
    );
}

#[test]
fn weighted_cnfs_count_the_same_with_and_without_implied_clauses() {
    // Random weighted CNFs as programs (each variable a fact, each clause j
    // a rule `violated(j)` observed false), with facts of probability one
    // half, clauses of four and five literals and planted ones that two
    // others imply (their resolvent widened to four literals or more),
    // which the stage of defined facts drops before the search for a
    // support (src/ground/preprocess/implied.rs): P(e) by enumeration of
    // the worlds, against every configuration with the stage on and off.
    let mut state = 0x6a09_e667_f3bc_c909u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let probabilities = [0.3, 0.7, 0.2, 0.5, 0.5, 0.5];
    let mut implied = 0;
    for round in 0..24 {
        let n = 10u64;
        let p: Vec<f64> = (0..n)
            .map(|_| probabilities[(next() % 6) as usize])
            .collect();
        let lit = |v: u64, positive: bool| (v as i64 + 1) * if positive { 1 } else { -1 };
        let mut clauses: Vec<Vec<i64>> = Vec::new();
        for _ in 0..6 + next() % 6 {
            let width = 2 + next() % 4;
            let mut clause: Vec<i64> = Vec::new();
            for _ in 0..width {
                let x = lit(next() % n, next() & 1 == 1);
                if !clause.iter().any(|&y: &i64| y.abs() == x.abs()) {
                    clause.push(x);
                }
            }
            clauses.push(clause);
        }
        for _ in 0..2 {
            // b or x and not x or c imply b or c, widened with two more.
            let x = next() % n;
            let (b, c, d, e) = ((x + 1) % n, (x + 2) % n, (x + 3) % n, (x + 4) % n);
            clauses.push(vec![lit(b, true), lit(x, true)]);
            clauses.push(vec![lit(x, false), lit(c, next() & 1 == 1)]);
            clauses.push(vec![
                lit(b, true),
                lit(c, clauses.last().unwrap()[1] > 0),
                lit(d, next() & 1 == 1),
                lit(e, next() & 1 == 1),
            ]);
        }
        let holds = |w: u64, x: i64| (w >> (x.unsigned_abs() - 1) & 1 == 1) == (x > 0);
        let z: f64 = (0..1u64 << n)
            .filter(|&w| clauses.iter().all(|cl| cl.iter().any(|&x| holds(w, x))))
            .map(|w| {
                (0..n as usize)
                    .map(|v| if w >> v & 1 == 1 { p[v] } else { 1.0 - p[v] })
                    .product::<f64>()
            })
            .sum();
        let mut source = String::new();
        for (v, q) in p.iter().enumerate() {
            source.push_str(&format!("{q}::x({}).\n", v + 1));
        }
        for (j, cl) in clauses.iter().enumerate() {
            let body: Vec<String> = cl
                .iter()
                .map(|&x| {
                    if x > 0 {
                        format!("\\+x({x})")
                    } else {
                        format!("x({})", -x)
                    }
                })
                .collect();
            source.push_str(&format!(
                "violated({j}) :- {}.\nevidence(violated({j}), false).\n",
                body.join(", ")
            ));
        }
        let program = tddlog::syntax::parse_program(&source).unwrap();
        let ground = tddlog::ground::ground(
            &program,
            &GroundOptions {
                detach: false,
                ..GroundOptions::default()
            },
        )
        .unwrap();
        implied += ground
            .stats
            .defined
            .map_or(0, |d| d.implied.implied + d.implied_after.implied);
        for defined in [true, false] {
            for (what, got, consistent) in evidence_probabilities_with(
                &program,
                GroundOptions::default().noisy_or,
                GroundOptions::default().backbones,
                defined,
            ) {
                if z == 0.0 {
                    assert!(
                        !consistent && got.to_f64() == 0.0,
                        "round {round} ({what}): P(e) = {} against 0\n{source}",
                        got.to_f64()
                    );
                } else {
                    assert!(
                        (got.log10() - z.log10()).abs() < 1e-10,
                        "round {round} ({what}): P(e) = {} against {z}\n{source}",
                        got.to_f64()
                    );
                }
            }
        }
    }
    assert!(
        implied >= 20,
        "implied clauses dropped over the rounds: {implied}"
    );
}

#[test]
fn detached_parts_are_counted_where_they_are_kept() {
    // detach.pl: q's part, r false, 1 - 0.3 · 0.6 = 0.82; z's, 0.2 + 0.5 ·
    // 0.6 = 0.5; d's, six of the eight worlds of c1..c3, 0.75. ProbLog
    // 0.3075. detach_no_query.pl is z's part alone, 0.5.
    for (file, want) in [
        ("detach.pl", 0.82 * 0.5 * 0.75),
        ("detach_no_query.pl", 0.5),
    ] {
        let source = read(&format!("tests/regress/{file}"));
        check(file, &source, want);
        // Ground as for the queries alone, the parts are dropped and P(e)
        // is not known.
        let run = common::run(
            &source,
            &common::options(Strategy::Trial(Kind::Closed), Projection::default()),
        )
        .unwrap();
        assert!(run.ground.stats.detached > 0, "{file}: parts detached");
        assert!(
            run.evidence_probability.is_none(),
            "{file}: no P(e) once parts are dropped"
        );
    }
}

#[test]
fn without_evidence_it_is_one_and_inconsistent_evidence_zero() {
    check("smokers.pl", &read("examples/smokers.pl"), 1.0);
    for file in [
        "condition_inconsistent.pl",
        "detach_inconsistent.pl",
        "alias_evidence_inconsistent.pl",
    ] {
        check(file, &read(&format!("tests/regress/{file}")), 0.0);
    }
}

#[test]
fn markov_networks_and_weighted_cnfs_give_their_partition_function() {
    // The programs of tests/regress/markov.uai and markov_pr.uai, without
    // and with their .evid files: the first line states log10 Z -
    // log10 P(e), and Z by enumeration of the UAI file's worlds is 325,
    // 165, 19947/25 and 11853/40 (in exact fractions, the programs'
    // comments). The programs of weighted CNFs in tests/lit (wcnf.pl: its
    // weighted model count is 153/200 by hand; wcnf_factors_bn.pl and
    // wcnf_factors_mn.pl: a Bayesian and a Markov network's weighted CNFs
    // with clauses over other variables besides, 45/128 and 499/20 by
    // enumeration of their models, in exact fractions). The program of
    // tests/regress/potts.uai under --no-queries, a Markov network of
    // Potts tables whose colour symmetry the program breaks (a third of Z,
    // the offset holding log10 3 besides): Z = 225/2 by enumeration (in
    // exact fractions, the program's comment).
    for (file, z) in [
        ("tests/regress/markov.pl", 325.0),
        ("tests/regress/potts.pl", 225.0 / 2.0),
        ("tests/regress/markov_ev.pl", 165.0),
        ("tests/regress/markov_pr.pl", 19947.0 / 25.0),
        ("tests/regress/markov_pr_ev.pl", 11853.0 / 40.0),
        ("tests/lit/wcnf.pl", 153.0 / 200.0),
        ("tests/lit/wcnf_factors_bn.pl", 45.0 / 128.0),
        ("tests/lit/wcnf_factors_mn.pl", 499.0 / 20.0),
    ] {
        let source = read(file);
        let offset: f64 = source
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("% log10 Z - log10 P(e) = "))
            .unwrap_or_else(|| panic!("{file}: no offset line"))
            .parse()
            .unwrap();
        for (what, p, _) in evidence_probabilities(&source) {
            let log10_z = p.log10() + offset;
            assert!(
                (log10_z - f64::log10(z)).abs() < 1e-12,
                "{file} ({what}): log10 Z = {log10_z} against {}",
                f64::log10(z)
            );
        }
    }
}
