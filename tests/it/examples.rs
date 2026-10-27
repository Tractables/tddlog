//! The programs under `examples/` with the probabilities ProbLog gives for
//! them (or, for negcycle_sound.pl, the probabilities that follow from the
//! well-founded model of each world; ProbLog refuses that program). Every
//! vtree strategy, each under its own formulation, must agree.

use crate::common;

use common::{Rows, assert_answers};
use tddlog::Projection;
use tddlog::{Kind, Strategy};

fn check(file: &str, expected: &[(&str, f64)]) {
    let source = std::fs::read_to_string(format!("examples/{file}")).unwrap();
    for strategy in [
        Strategy::Trial(Kind::Closed),
        Strategy::Auto,
        Strategy::Trial(Kind::Dynamic),
    ] {
        one(file, &source, expected, strategy, Projection::Off);
    }
    // Projecting each component's vtree onto its ancestor cone must not move
    // an answer: the cone contains the support of the formula, and a
    // variable a formula does not mention marginalises to one in the
    // weighted model count. `max_share` 1.0 projects every component it can,
    // 0.0 sends every component to the global vtree (the fallback path), and
    // 0.5 is the default mix of the two.
    for (strategy, shares) in [
        (Strategy::Trial(Kind::Closed), &[1.0, 0.5, 0.0][..]),
        (Strategy::Auto, &[1.0][..]),
    ] {
        for &max_share in shares {
            one(
                file,
                &source,
                expected,
                strategy,
                Projection::Cones { max_share },
            );
        }
    }
    // Reading derived atoms through indicators must not move an answer
    // either: the links admit one value of the indicators in each world, and
    // their weights are one; where a table's rows partition its inputs, its
    // clauses admit the one value of its copies a link would.
    for strategy in [
        Strategy::Trial(Kind::Families { search: true }),
        Strategy::Trial(Kind::Closed),
    ] {
        indicated(file, &source, expected, true, strategy, Rows::Apart);
    }
    // Nor may row classes (the default): in each world at most one of the
    // rows that read a class holds (src/ground/classes.rs).
    for (indicators, strategy) in [
        (false, Strategy::Trial(Kind::Closed)),
        (false, Strategy::Auto),
        (true, Strategy::Trial(Kind::Families { search: true })),
    ] {
        indicated(file, &source, expected, indicators, strategy, Rows::Classes);
    }
    // `run` reads the program through every indicator for `families`, as
    // the binary does, written without linear recursion; a program where no
    // component reads another's atoms has none, and the layout refuses it,
    // under `--ground` too.
    for strategy in [
        Strategy::Trial(Kind::Families { search: true }),
        Strategy::Trial(Kind::Families { search: false }),
    ] {
        let options = common::options(strategy, Projection::default());
        let refused = |e: &tddlog::Error| matches!(e, tddlog::Error::Vtree(e) if e.contains("reads no atom through one"));
        match tddlog::rewritten(&source, &options) {
            Ok(gp) => {
                assert!(!gp.encoding.is_empty(), "{file} ({strategy:?})");
                one(file, &source, expected, strategy, Projection::default());
            }
            Err(e) if refused(&e) => match common::run(&source, &options) {
                Err(e) if refused(&e) => {}
                other => panic!(
                    "{file} ({strategy:?}): families without indicators gave {:?}",
                    other.map(|r| r.results)
                ),
            },
            Err(e) => panic!("{file} ({strategy:?}): {e}"),
        }
    }
    // The ground program as `--ground` prints it is a ProbLog program with
    // the same answers, also read through indicators, which it prints as
    // facts of probability 0.5 that the links observe: a constant factor in
    // the evidence and each query alike.
    for indicators in [false, true] {
        let printed = common::ground(&source, Rows::Classes, indicators).to_string();
        one(
            &format!("{file} printed ground (indicators {indicators})"),
            &printed,
            expected,
            Strategy::Auto,
            Projection::default(),
        );
    }
}

/// [`one`] on the program rewritten, where `indicators`, to read atoms
/// through indicators, and its rows grounded as `rows`.
fn indicated(
    file: &str,
    source: &str,
    expected: &[(&str, f64)],
    indicators: bool,
    strategy: Strategy,
    rows: Rows,
) {
    let ground = common::ground(source, rows, indicators);
    let what = format!("{file} (indicators {indicators}, {strategy:?}, rows {rows:?})");
    // A program the grounder left with no atom another component reads
    // is rewritten by the layout, without its linear recursion, and
    // `families` may find none there either, which it says (`check`
    // tells the two apart).
    let none =
        ground.encoding.is_empty() && matches!(strategy, Strategy::Trial(Kind::Families { .. }));
    let run = match tddlog::solve_ground(ground, &common::options(strategy, Projection::default()))
    {
        Err(tddlog::Error::Vtree(e)) if none && e.contains("reads no atom through one") => return,
        run => run.unwrap_or_else(|e| panic!("{what}: {e}")),
    };
    assert_answers(
        &what,
        &run.results.expect("evidence is consistent"),
        expected,
        1e-6,
    );
}

/// The answers of `source` under `strategy` and `projection` against
/// `expected`.
fn one(
    file: &str,
    source: &str,
    expected: &[(&str, f64)],
    strategy: Strategy,
    projection: Projection,
) {
    let what = format!("{file} ({strategy:?}, {projection})");
    let run = common::run(source, &common::options(strategy, projection))
        .unwrap_or_else(|e| panic!("{what}: {e}"));
    assert_answers(
        &what,
        &run.results.expect("evidence is consistent"),
        expected,
        1e-6,
    );
}

#[test]
fn alarm() {
    check(
        "alarm.pl",
        &[
            ("burglary", 0.357143),
            ("earthquake", 0.714286),
            ("calls(mary)", 0.4),
        ],
    );
}

#[test]
fn smokers() {
    check(
        "smokers.pl",
        &[
            ("smokes(p1)", 0.347880),
            ("smokes(p2)", 0.381480),
            ("asthma(p3)", 0.139152),
        ],
    );
}

/// Linear recursion over cycles, which every strategy compiles written
/// without recursion: ProbLog 2.3.0's answers, one query at a time.
#[test]
fn cycle_graph() {
    check(
        "cycle_graph.pl",
        &[
            ("path(1,2)", 0.632256),
            ("path(1,3)", 0.460356),
            ("path(1,4)", 0.399624),
            ("path(1,5)", 0.754432),
            ("path(1,6)", 0.5716992),
            ("path(1,7)", 0.199812),
        ],
    );
}

#[test]
fn gearwheels() {
    check(
        "gearwheels.pl",
        &[("turns1(0)", 0.0), ("turns1(1)", 0.72), ("turns2(1)", 0.72)],
    );
}

/// Two integers in binary added by a ripple-carry adder, the guide's worked
/// example: the 64 worlds summed by hand (P(e) = 23/50) and ProbLog 2.3.0
/// agree.
#[test]
fn adder() {
    let e = 23.0 / 50.0;
    check(
        "adder.pl",
        &[
            ("sum(1)", 113.0 / 500.0 / e),
            ("sum(2)", 2857.0 / 12500.0 / e),
            ("at_least_8", 2909.0 / 12500.0 / e),
            ("x(0)", 7.0 / 25.0 / e),
        ],
    );
}

/// `//` rounds down and `mod` takes the divisor's sign, as in ProbLog
/// 2.3.0, which gives these answers (q(7,-2,-3,1), euclidean division's
/// answer, it gives 0).
#[test]
fn integer_division_rounds_down() {
    let source = "0.5::p(7, 2). 0.6::p(-7, 2). 0.7::p(7, -2). 0.8::p(-7, -2).
        q(X, Y, D, M) :- p(X, Y), D is X // Y, M is X mod Y.
        query(q(7, 2, 3, 1)). query(q(-7, 2, -4, 1)). query(q(7, -2, -4, -1)).
        query(q(-7, -2, 3, -1)). query(q(7, -2, -3, 1)).";
    let run = common::run(source, &common::options(Strategy::Auto, Projection::Off)).unwrap();
    let expected = [
        ("q(7,2,3,1)", 0.5),
        ("q(-7,2,-4,1)", 0.6),
        ("q(7,-2,-4,-1)", 0.7),
        ("q(-7,-2,3,-1)", 0.8),
        ("q(7,-2,-3,1)", 0.0),
    ];
    assert_answers(
        "integer division",
        run.results.as_deref().unwrap(),
        &expected,
        1e-12,
    );
}

#[test]
fn win_move_on_a_dag() {
    check(
        "win.pl",
        &[("win(1)", 0.5625), ("win(2)", 0.25), ("win(3)", 0.5)],
    );
}

#[test]
fn negative_cycle_that_is_two_valued_everywhere() {
    check("negcycle_sound.pl", &[("a", 0.3), ("b", 0.7)]);
}

#[test]
fn negative_cycle_that_is_not_two_valued_is_rejected() {
    let source = std::fs::read_to_string("examples/unsound.pl").unwrap();
    // `closed` compiles under the dual, `dynamic` under the primal.
    for strategy in [
        Strategy::Trial(Kind::Closed),
        Strategy::Trial(Kind::Dynamic),
    ] {
        match common::run(&source, &common::options(strategy, Projection::Off)) {
            Err(tddlog::Error::Compile(tddlog::tp::CompileError::Unsound(_))) => {}
            other => panic!(
                "{strategy:?}: expected an unsoundness error, got {:?}",
                other.map(|r| r.results)
            ),
        }
    }
}

/// Regressions found by the differential fuzzer (`tests/fuzz_problog.py`),
/// and the programs of `auto`'s.
mod regress {
    use super::*;

    #[test]
    fn aliasing_keeps_queried_facts() {
        check(
            "../tests/regress/alias_query.pl",
            &[
                ("edge(2,1)", 0.4),
                ("path(2,1)", 0.4),
                ("\\+edge(2,1)", 0.6),
            ],
        );
    }

    /// Two atoms that read one fact alone are that fact, a query or
    /// evidence on one of them included.
    #[test]
    fn aliasing_reads_an_undirected_edge_as_one_fact() {
        let expected = |p: f64| [("path(1,4)", p), ("path(4,1)", p), ("conn(3,2)", 0.5)];
        check("../tests/regress/alias_undirected.pl", &expected(0.406));
        check("../tests/regress/alias_undirected_ev.pl", &expected(0.49));
    }

    #[test]
    fn aliasing_keeps_evidence_facts() {
        check(
            "../tests/regress/alias_evidence.pl",
            &[("asthma(p1)", 0.0), ("smokes(p2)", 1.0)],
        );
    }

    /// The programs of `src/auto/tests.rs`'s tests of a trial refused memory
    /// and of `closed` after `dynamic`; noisy_or_shared.pl is ProbLog
    /// 2.3.0's, the others are hand-computed in their comments.
    #[test]
    fn auto_regressions_agree_with_problog() {
        check(
            "../tests/regress/noisy_or_shared.pl",
            &[
                ("disease(d0)", 0.0171027036),
                ("disease(d1)", 0.0476664622),
                ("disease(d2)", 0.4864437251),
                ("disease(d3)", 0.0363359180),
                ("disease(d4)", 0.1841834972),
                ("disease(d5)", 0.3911484813),
                ("disease(d6)", 0.8255111439),
                ("disease(d7)", 0.0306481191),
            ],
        );
        check(
            "../tests/regress/growing_head.pl",
            &[("a0", 1.0 - 0.5f64.powi(7))],
        );
        check("../tests/regress/growing_negation.pl", &[("a0", 0.25)]);
    }

    /// Conditioning on what the evidence implies (src/ground/condition.rs):
    /// ProbLog 2.3.0's answers, hand-computed in the programs' comments. A
    /// cyclic program is left as it is, and evidence that propagation finds
    /// inconsistent stays detectable.
    #[test]
    fn conditioning_on_the_evidence() {
        check(
            "../tests/regress/condition_bn.pl",
            &[
                ("x(3)", 1.0),
                ("u(true)", 0.0),
                ("v(true)", 0.1),
                ("s", 0.25),
                ("t", 0.25),
            ],
        );
        check(
            "../tests/regress/condition_equal.pl",
            &[
                ("f(1,true)", 0.036 / 0.26),
                ("f(2,true)", 0.036 / 0.26),
                ("f(3,true)", 0.224 / 0.26),
                ("f(3,false)", 0.036 / 0.26),
                ("q", 0.018 / 0.26),
            ],
        );
        check(
            "../tests/regress/condition_copies.pl",
            &[
                ("x(1)", 0.162 / 0.33),
                ("x(3)", 0.074 / 0.33),
                ("x(4)", 0.212 / 0.33),
            ],
        );
        check(
            "../tests/regress/condition_cyclic.pl",
            &[("a", 0.12 / 0.68), ("sb", 0.0), ("eab", 0.28 / 0.68)],
        );
        check(
            "../tests/regress/condition_settled.pl",
            &[
                ("x(3)", 1.0),
                ("u(false)", 1.0),
                ("y(false)", 0.0),
                ("z(true)", 0.0),
            ],
        );
        // A backbone that the SAT search fixes and propagation does not
        // (src/ground/preprocess/backbone.rs), and the equality it leaves.
        check(
            "../tests/regress/backbone_clauses.pl",
            &[("a", 0.2 / 0.68), ("c", 1.0), ("d", 0.28 / 0.46)],
        );
        let conditioned = |file: &str| {
            let source = std::fs::read_to_string(format!("tests/regress/{file}")).unwrap();
            common::ground(&source, Rows::Classes, false)
                .stats
                .conditioned
        };
        let bn = conditioned("condition_bn.pl");
        assert!(bn.facts > 0 && bn.atoms > 0, "{bn:?}");
        let equal = conditioned("condition_equal.pl");
        assert_eq!((equal.merged, equal.into), (2, 1), "{equal:?}");
        assert!(!conditioned("condition_cyclic.pl").any());
    }

    /// Facts of probability one half that the clauses define as an AND or
    /// an OR of other facts' literals become derived atoms with the gate's
    /// rules (src/ground/condition.rs, gates): ProbLog 2.3.0's answers, one
    /// query at a time, as counted by hand in the programs' comments. A
    /// gate nothing else reads goes, and the gates and facts only it read
    /// with it; a fact of another probability is left alone, and of two
    /// definitions through each other one is taken.
    #[test]
    fn gates_of_the_clauses() {
        check(
            "../tests/regress/gate_or_unread.pl",
            &[("x(1)", 0.1 / 0.22)],
        );
        check(
            "../tests/regress/gate_and_read.pl",
            &[("g", 0.3125), ("c", 0.78125)],
        );
        check(
            "../tests/regress/gate_nested.pl",
            &[("g", 0.2516 / 0.2836), ("h", 0.204 / 0.2836)],
        );
        check(
            "../tests/regress/gate_nested_unread.pl",
            &[("a", 0.2 / 0.44)],
        );
        check(
            "../tests/regress/gate_not_half.pl",
            &[("g", 0.1376 / 0.1952)],
        );
        check(
            "../tests/regress/gate_cyclic.pl",
            &[("g1", 0.18 / 0.355), ("g2", 0.15 / 0.355)],
        );
        check(
            "../tests/regress/gate_read_by_clause.pl",
            &[("c", 0.15 / 0.262), ("a", 0.03 / 0.262)],
        );
        check(
            "../tests/regress/gate_backbone.pl",
            &[("x(1)", 0.096 / 0.18), ("y", 0.12 / 0.18)],
        );
        // A gate is taken where nothing reads it but the definitions of the
        // other gates taken; one that a query, a rule or another clause
        // reads stays a fact with its clauses. In a CNF over its facts, one
        // that no query reads is then eliminated with the facts defined
        // otherwise (src/ground/preprocess/defined.rs): gate_read_by_clause's g.
        for (file, gates, facts) in [
            ("gate_or_unread.pl", 1, 2),
            ("gate_and_read.pl", 0, 4),
            ("gate_nested.pl", 0, 6),
            ("gate_nested_unread.pl", 2, 2),
            ("gate_not_half.pl", 0, 4),
            ("gate_cyclic.pl", 0, 4),
            ("gate_read_by_clause.pl", 0, 3),
            ("gate_backbone.pl", 1, 3),
        ] {
            let source = std::fs::read_to_string(format!("tests/regress/{file}")).unwrap();
            let ground = common::ground(&source, Rows::Classes, false);
            let c = ground.stats.conditioned;
            assert_eq!(
                (c.gates, ground.num_facts()),
                (gates, facts),
                "{file}: {c:?}"
            );
        }
        // The SAT search proves g a backbone and leaves it to the gate.
        let source = std::fs::read_to_string("tests/regress/gate_backbone.pl").unwrap();
        let searched = common::ground(&source, Rows::Classes, false)
            .stats
            .conditioned
            .backbones
            .expect("searched");
        assert_eq!(
            (searched.search.backbones, searched.fixed),
            (1, 0),
            "{searched:?}"
        );
    }

    /// A Markov network written by scripts/gen_bn.py, its factors
    /// observed (tests/test_gen_bn.py): the marginals by enumeration in
    /// the programs' comments.
    #[test]
    fn markov_network_marginals() {
        check(
            "../tests/regress/markov.pl",
            &[
                ("x0(v_0)", 17.0 / 65.0),
                ("x0(v_1)", 48.0 / 65.0),
                ("x1(v_0)", 6.0 / 13.0),
                ("x1(v_1)", 7.0 / 13.0),
                ("x2(v_0)", 33.0 / 65.0),
                ("x2(v_1)", 2.0 / 13.0),
                ("x2(v_2)", 22.0 / 65.0),
            ],
        );
        check(
            "../tests/regress/markov_ev.pl",
            &[
                ("x0(v_0)", 2.0 / 11.0),
                ("x0(v_1)", 9.0 / 11.0),
                ("x1(v_0)", 5.0 / 33.0),
                ("x1(v_1)", 28.0 / 33.0),
            ],
        );
    }

    /// Evidence no query shares a fact with (src/ground/detach.rs):
    /// ProbLog 2.3.0's answers. A part the search finds a world for is
    /// dropped, one it finds none for stays and the compile reports it
    /// inconsistent, and a program with no query answers nothing.
    #[test]
    fn dropping_detached_evidence() {
        check("../tests/regress/detach.pl", &[("q", 0.12 / 0.82)]);
        let ground = |file: &str| {
            let source = std::fs::read_to_string(format!("tests/regress/{file}")).unwrap();
            (
                common::ground(&source, Rows::Classes, false).stats.detached,
                source,
            )
        };
        let (detached, _) = ground("detach.pl");
        assert_eq!(detached, 3, "z, d and e are dropped, r is kept");
        let (detached, source) = ground("detach_inconsistent.pl");
        assert_eq!(detached, 0, "no world satisfies the four clauses");
        let run = common::run(
            &source,
            &common::options(Strategy::Trial(Kind::Closed), Projection::Off),
        )
        .unwrap();
        assert!(
            run.results.is_none(),
            "evidence must be reported inconsistent"
        );
        let (detached, source) = ground("detach_no_query.pl");
        assert_eq!(detached, 1);
        let run = common::run(
            &source,
            &common::options(Strategy::Trial(Kind::Closed), Projection::Off),
        )
        .unwrap();
        assert_eq!(run.results.map(|r| r.len()), Some(0));
    }
}

#[test]
fn uwcse() {
    check("uwcse.pl", &[("taught_by(c1,p1)", 0.092658)]);
}

/// Four students: 120 facts, the largest UW-CSE instance the reference
/// answers quickly. ProbLog: 0.094006751. Under `closed` the query formula
/// is 36 k pairs.
#[test]
fn uwcse_four_students() {
    let source = std::fs::read_to_string("examples/uwcse.pl")
        .unwrap()
        .replace("student(s2).", "student(s2).\nstudent(s3).\nstudent(s4).");
    let run = common::run(
        &source,
        &common::options(Strategy::Trial(Kind::Closed), Projection::Off),
    )
    .unwrap();
    let results = run.results.unwrap();
    assert!(
        (results[0].probability - 0.094007).abs() < 1e-6,
        "{}",
        results[0].probability
    );
    let largest = run.stats.final_max_pairs;
    assert!(largest < 100_000, "largest formula {largest} pairs");
}

/// Whether grounding `file` evaluated some right-linear recursion from its
/// bound end.
fn grounded_from_bound_end(file: &str) -> bool {
    let source = std::fs::read_to_string(format!("examples/{file}")).unwrap();
    !common::ground(&source, Rows::Classes, false)
        .stats
        .from_bound_end
        .is_empty()
}

/// The chain atoms of a closure grounded from its source merge into its
/// path atoms; two queried atoms with the same bodies stay apart.
/// ProbLog's answers.
#[test]
fn equal_atoms() {
    // Without the evidence, which fixes the chain atom of path(1,2)
    // (src/ground/condition.rs).
    let source = std::fs::read_to_string("examples/equal_atoms.pl")
        .unwrap()
        .replace("evidence(path(1,3), true).", "");
    let ground = common::ground(&source, Rows::Classes, false);
    assert_eq!(ground.stats.from_bound_end, ["path/2"]);
    let names: Vec<String> = ground.atoms.iter().map(|a| a.to_string()).collect();
    // A chain atom stays only where no path atom has its arguments.
    let chains: Vec<&str> = names
        .iter()
        .filter_map(|a| a.strip_prefix("'$r_path_2'"))
        .collect();
    assert!(
        !chains.is_empty()
            && chains
                .iter()
                .all(|args| !names.contains(&format!("path{args}"))),
        "{names:?}"
    );
    assert!(
        names.iter().any(|a| a == "p1") && names.iter().any(|a| a == "q1"),
        "{names:?}"
    );
    check(
        "equal_atoms.pl",
        &[
            ("path(1,1)", 0.4),
            ("path(1,5)", 0.395),
            ("path(1,4)", 0.79),
            ("p1", 0.6948),
            ("q1", 0.6948),
            ("t", 0.0),
        ],
    );
}

/// A circuit's gates written twice are compiled once: each gate with an
/// earlier gate's rules, over the gates merged before it, is merged into
/// it (src/ground/aig.rs, `reduce`); a queried gate stays,
/// and a rule that the merge makes read a gate both ways is dropped.
/// ProbLog's answers (ProbLog 2.3.0, one query at a time).
#[test]
fn equal_gates() {
    let source = std::fs::read_to_string("examples/equal_gates.pl").unwrap();
    let ground = common::ground(&source, Rows::Classes, false);
    let names: Vec<String> = ground.atoms.iter().map(|a| a.to_string()).collect();
    for gone in ["d0", "t0", "k1", "t2"] {
        assert!(
            !names.iter().any(|a| a == gone),
            "{gone} is merged: {names:?}"
        );
    }
    for kept in ["c0", "s0", "h1", "s1", "s2", "t1", "differ"] {
        assert!(names.iter().any(|a| a == kept), "{kept} stays: {names:?}");
    }
    assert_eq!(ground.stats.merged, 4, "{names:?}");
    let differ = names.iter().position(|a| a == "differ").unwrap() as u32;
    assert_eq!(
        ground.rules.iter().filter(|r| r.head == differ).count(),
        2,
        "{ground}"
    );
    check(
        "equal_gates.pl",
        &[
            ("s1", 0.57480653),
            ("t1", 0.57480653),
            ("differ", 0.0),
            ("both", 0.013542562),
            ("either", 0.4544282),
            ("s0", 0.54858126),
        ],
    );
}

/// Gates equal in function but not in their rules are compiled once: the
/// sweep proposes them by simulation and proves them with the SAT solver,
/// and each is merged into the earlier gate, a complement through a
/// negated literal, a constant into its value (src/ground/aig.rs,
/// `reduce`). ProbLog's answers (ProbLog 2.3.0, one query at a time).
#[test]
fn equal_functions() {
    let source = std::fs::read_to_string("examples/equal_functions.pl").unwrap();
    let ground = common::ground(&source, Rows::Classes, false);
    let names: Vec<String> = ground.atoms.iter().map(|a| a.to_string()).collect();
    for gone in ["d1", "t1", "t2", "n0", "differ"] {
        assert!(
            !names.iter().any(|a| a == gone),
            "{gone} is merged: {names:?}"
        );
    }
    for kept in ["c1", "s1", "s2", "t0", "alarm"] {
        assert!(names.iter().any(|a| a == kept), "{kept} stays: {names:?}");
    }
    // Four proofs: once t1 is s1, `differ` reads s1 both ways and its
    // node is the constant false by structure alone.
    assert_eq!(ground.stats.swept.proved, 4, "{ground}");
    check(
        "equal_functions.pl",
        &[
            ("s1", 0.48861715),
            ("alarm", 0.0),
            ("low", 0.1092754),
            ("both", 0.048003122),
            ("t0", 0.37934175),
        ],
    );
}

/// A path relation written right-recursively, queried from two sources
/// with evidence on one of them and a negated step; ProbLog's answers.
#[test]
fn linear_path() {
    assert!(grounded_from_bound_end("linear_path.pl"));
    check(
        "linear_path.pl",
        &[
            ("path(1,1)", 0.15659724),
            ("path(1,4)", 0.65212599),
            ("path(1,6)", 0.23674718),
            ("path(1,8)", 0.39012124),
            ("\\+path(1,5)", 0.60638803),
            ("path(2,6)", 0.17713304),
        ],
    );
}

/// A closure written left-recursively and queried towards a bound target,
/// with a probabilistic exit rule; ProbLog's answers.
#[test]
fn linear_target() {
    assert!(grounded_from_bound_end("linear_target.pl"));
    check(
        "linear_target.pl",
        &[
            ("p(1,6)", 0.2412),
            ("p(2,6)", 0.493956),
            ("p(3,6)", 0.250002),
            ("p(4,6)", 0.27945),
            ("p(5,6)", 0.564408),
            ("p(6,6)", 0.11178),
        ],
    );
}

#[test]
fn dice() {
    check(
        "dice.pl",
        &[("on(0,1)", 1.0), ("on(1,1)", 1.0), ("on(2,1)", 0.166667)],
    );
}

/// A Bayesian network written by scripts/gen_bn.py (bnlearn's cancer
/// network, leaf marginals); ProbLog's answers.
#[test]
fn bn_cancer() {
    check(
        "bn_cancer.pl",
        &[
            ("xray(positive)", 0.208141),
            ("xray(negative)", 0.791859),
            ("dyspnoea(true)", 0.3040705),
            ("dyspnoea(false)", 0.6959295),
        ],
    );
}

/// The asia network with both leaves observed (scripts/gen_bn.py
/// --evidence); ProbLog's answers. The families layout reads the evidence
/// and every query through indicators.
#[test]
fn bn_asia_evidence() {
    check(
        "bn_asia_evidence.pl",
        &[
            ("asia(yes)", 0.01167842),
            ("asia(no)", 0.98832158),
            ("tub(yes)", 0.054021289),
            ("tub(no)", 0.94597871),
            ("smoke(yes)", 0.51320709),
            ("smoke(no)", 0.48679291),
            ("lung(yes)", 0.25229723),
            ("lung(no)", 0.74770277),
            ("bronc(yes)", 0.1932111),
            ("bronc(no)", 0.8067889),
            ("either(yes)", 0.30369463),
            ("either(no)", 0.69630537),
        ],
    );
}

/// c's table, nine rows over a and b, and e's over b and c are split on a
/// parent by the indicator rewrite, each part reading the other parent
/// through copies of its own; ProbLog's answers.
#[test]
fn bn_split() {
    let source = std::fs::read_to_string("examples/bn_split.pl").unwrap();
    let ground = common::ground(&source, Rows::Classes, true);
    let parts = ground
        .encoding
        .indicators
        .iter()
        .filter(|ind| ground.atoms[ind.var as usize].args.len() == 3)
        .count();
    assert!(parts > 0, "a table is split");
    check(
        "bn_split.pl",
        &[
            ("a(x)", 0.1395817),
            ("a(y)", 0.6352757),
            ("b(z)", 0.34526339),
            ("c(p)", 0.73280394),
            ("c(q)", 0.20266189),
            ("e(t)", 0.39609663),
        ],
    );
}

/// A query's formula next to evidence it does not read; ProbLog's answers.
#[test]
fn query_beside_evidence() {
    check("query_beside_evidence.pl", &[("w", 0.0), ("d", 0.29)]);
}

/// Dice programs translated by scripts/dice2problog.py; ProbLog and Dice
/// agree on both. evidence2 observes inside one branch of a conditional,
/// murderMystery inlines a function returning a tuple.
#[test]
fn dice_translations() {
    check("dice_evidence.pl", &[("result", 0.666667)]);
    check("dice_murder.pl", &[("alicedunnit", 0.015817)]);
}

/// Dice programs with loops, integer arithmetic and recursion
/// (tests/dice/, after the Dice paper's programs in
/// [`data/dice`](https://github.com/Tractables/tddlog/tree/baselines/data/dice) on the baselines branch),
/// translated by scripts/dice2problog.py beside them;
/// Dice's answers, which tests/test_dice2problog.py also checks against
/// an exact evaluation of the Dice program.
mod dice {
    use super::*;

    /// Integer arithmetic modulo four, shifts, uniform, comparisons.
    #[test]
    fn arith() {
        check(
            "../tests/dice/arith.pl",
            &[
                ("w(0)", 0.321428571429),
                ("w(1)", 0.353571428571),
                ("w(2)", 0.325),
                ("w(3)", 0.0),
            ],
        );
    }

    /// Comparisons between two integer variables, xor and iff.
    #[test]
    fn compare() {
        check(
            "../tests/dice/compare.pl",
            &[
                ("result_1", 0.112676056338),
                ("result_2_1(0)", 0.131455399061),
                ("result_2_1(1)", 0.112676056338),
                ("result_2_1(2)", 0.107981220657),
                ("result_2_1(3)", 0.056338028169),
                ("result_2_1(4)", 0.112676056338),
                ("result_2_1(5)", 0.140845070422),
                ("result_2_1(6)", 0.18779342723),
                ("result_2_1(7)", 0.150234741784),
                ("result_2_2(0)", 0.0),
                ("result_2_2(1)", 0.281690140845),
                ("result_2_2(2)", 0.154929577465),
                ("result_2_2(3)", 0.281690140845),
                ("result_2_2(4)", 0.093896713615),
                ("result_2_2(5)", 0.18779342723),
            ],
        );
    }

    /// A random walk in a loop (`iterate`), observed at the end.
    #[test]
    fn walk() {
        check(
            "../tests/dice/walk.pl",
            &[
                ("x(0)", 0.0),
                ("x(1)", 0.245556733026),
                ("x(2)", 0.0),
                ("x(3)", 0.362357126637),
                ("x(4)", 0.0),
                ("x(5)", 0.190068881992),
                ("x(6)", 0.0),
                ("x(7)", 0.202017258345),
            ],
        );
    }

    /// Recursion past Dice's limit of ten nested calls.
    #[test]
    fn geometric() {
        check(
            "../tests/dice/geometric.pl",
            &[
                ("result(0)", 0.500244259893),
                ("result(1)", 0.250122129946),
                ("result(2)", 0.125061064973),
                ("result(3)", 0.0625305324866),
                ("result(4)", 0.0312652662433),
                ("result(5)", 0.0156326331216),
                ("result(6)", 0.00781631656082),
                ("result(7)", 0.00390815828041),
                ("result(8)", 0.00195407914021),
                ("result(9)", 0.000977039570103),
                ("result(10)", 0.000488519785051),
            ],
        );
    }

    /// A loop over a tuple state with an observation in the step.
    #[test]
    fn counter() {
        check(
            "../tests/dice/counter.pl",
            &[
                ("n2(0)", 0.118949715135),
                ("n2(1)", 0.305870696061),
                ("n2(2)", 0.327718602923),
                ("n2(3)", 0.187267773099),
                ("n2(4)", 0.0601932127818),
                ("n2(5)", 0.0),
                ("n2(6)", 0.0),
                ("n2(7)", 0.0),
                ("result_2_2", 0.247460985881),
            ],
        );
    }

    /// Recursion that ends on a constant (Dice's resources/).
    #[test]
    fn recursion() {
        check("../tests/dice/recursion.pl", &[("result_else_1", 0.125)]);
    }

    /// The Caesar cipher of Dice's README as a recursive loop.
    #[test]
    fn caesar_recur() {
        check(
            "../tests/dice/caesar-recur.pl",
            &[
                ("key(0)", 0.0036496350365),
                ("key(1)", 0.0583941605839),
                ("key(2)", 0.934306569343),
                ("key(3)", 0.0036496350365),
            ],
        );
    }

    /// The diamond network of the Dice paper, 100 steps, a loop over the
    /// step index.
    #[test]
    fn diamond() {
        check("../tests/dice/diamond.pl", &[("result_2", 0.995012354812)]);
    }

    /// The ladder network of the Dice paper, 100 steps.
    #[test]
    fn ladder() {
        check(
            "../tests/dice/ladder.pl",
            &[
                ("result_1_1", 0.475370841777),
                ("result_2_1", 0.475370841777),
            ],
        );
    }

    /// A network written as nested `discrete`s, its rows of thirds
    /// summing to 0.999999 (tests/dice/nested.dice); the answers by hand,
    /// which ProbLog and tests/test_dice2problog.py's exact evaluation
    /// give too.
    #[test]
    fn nested() {
        check(
            "../tests/dice/nested.pl",
            &[
                ("c(0)", 73.0 / 300.0),
                ("c(1)", 134.0 / 300.0),
                ("c(2)", 93.0 / 300.0),
            ],
        );
    }
}

/// LPADs as cplint reads them (tests/lpad/NAME.cplint.pl), read by
/// tddlog's LPAD reader (`tddlog::syntax::lpad`) as the program
/// scripts/lpad2problog.py wrote for each before the reader replaced it
/// (tests/lpad/NAME.pl), with cplint's PITA answers, which ProbLog gives
/// on that program too.
mod lpad {
    use super::*;

    #[test]
    fn each_lpad_is_read_as_the_program_lpad2problog_wrote() {
        let mut read = 0;
        for entry in std::fs::read_dir("tests/lpad").unwrap() {
            let path = entry.unwrap().path();
            let name = path.to_str().unwrap();
            let Some(stem) = name.strip_suffix(".cplint.pl") else {
                continue;
            };
            let source = std::fs::read_to_string(&path).unwrap();
            let lpad = tddlog::syntax::lpad::read(&source)
                .expect("an LPAD")
                .unwrap();
            let written = std::fs::read_to_string(format!("{stem}.pl")).unwrap();
            assert_eq!(lpad.program + "\n", written, "{name}");
            read += 1;
        }
        assert_eq!(read, 2);
    }

    /// Annotated disjunctions, a head that takes what the others leave and
    /// a conjunction of evidence; a query under other evidence left out.
    #[test]
    fn coin() {
        check(
            "../tests/lpad/coin.cplint.pl",
            &[("heads(coin)", 0.6), ("tails(coin)", 0.4)],
        );
    }

    /// Database clauses outside the LPAD, called through db/1 and in a
    /// findall's goal.
    #[test]
    fn database() {
        check(
            "../tests/lpad/database.cplint.pl",
            &[
                ("trait(carl)", 0.196),
                ("sampled(carl)", 0.5),
                ("big_family", 0.9),
            ],
        );
    }
}

/// The circuits that scripts/alea2problog.jl writes for the Alea.jl
/// programs beside them (tests/test_alea2problog.py checks that it does),
/// with the answers computed by hand in those programs' comments, which
/// Alea gives too.
mod alea {
    use super::*;

    /// examples/alea_dice.pl: two dice, the first at least 4; the sum 7
    /// and the first showing more.
    #[test]
    fn dice() {
        check("alea_dice.pl", &[("q1", 1.0 / 6.0), ("q2", 2.0 / 3.0)]);
    }

    /// A Boolean, and the distribution of a three-bit number over the
    /// worlds of its bits, under an observation.
    #[test]
    fn tiny_pr() {
        let other = 0.105 / 0.72;
        check(
            "../tests/alea/tiny_pr.pl",
            &[
                ("q1", 0.12 / 0.72),
                ("q2(0)", 0.0),
                ("q2(1)", other),
                ("q2(2)", 0.405 / 0.72),
                ("q2(3)", other),
                ("q2(4)", other),
                ("q2(5)", 0.0),
                ("q2(6)", 0.0),
                ("q2(7)", 0.0),
            ],
        );
    }

    /// The bits of a signed four-bit number uniform on -3, -2, -1, 1, 2:
    /// each is true in three of the five values.
    #[test]
    fn tiny_expectation() {
        check(
            "../tests/alea/tiny_expectation.pl",
            &[
                ("e1(1)", 0.6),
                ("e1(2)", 0.6),
                ("e1(3)", 0.6),
                ("e1(4)", 0.6),
            ],
        );
    }
}

/// The scaling programs of Roulette's evaluation, small, in
/// tests/roulette/, as the baselines branch's [`scripts/gen_roulette.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/gen_roulette.py)
/// writes them; that script's reference answers (a dynamic program over
/// each model's states), which its tests there check against an exact
/// enumeration of the Dice programs it writes and against Roulette's
/// answers.
mod roulette {
    use super::*;

    /// The Caesar cipher with one observed letter: the key's posterior.
    #[test]
    fn caesar() {
        check(
            "../tests/roulette/caesar-1.pl",
            &[
                ("key(0)", 0.0241312850374),
                ("key(1)", 0.0100015812331),
                ("key(2)", 0.0282004761475),
                ("key(3)", 0.0925918470297),
                ("key(4)", 0.0646902828605),
                ("key(5)", 0.0612140894499),
                ("key(6)", 0.000973702463819),
                ("key(7)", 0.0197246986845),
                ("key(8)", 0.076754718815),
                ("key(9)", 0.069004852329),
                ("key(10)", 0.0246015935577),
                ("key(11)", 0.0411544086511),
                ("key(12)", 0.0789324517457),
                ("key(13)", 0.0156452834762),
                ("key(14)", 0.0712234816528),
                ("key(15)", 0.0623080679644),
                ("key(16)", 0.0206039711355),
                ("key(17)", 0.0227817040662),
                ("key(18)", 0.0447635153391),
                ("key(19)", 0.0434855030558),
                ("key(20)", 0.0284458545058),
                ("key(21)", 0.0152567677421),
                ("key(22)", 0.0835026236708),
                ("key(23)", 2.41312850374e-06),
                ("key(24)", 2.41312850374e-06),
                ("key(25)", 2.41312925664e-06),
            ],
        );
    }

    /// The same with two letters, under `auto` and `closed`. The dynamic
    /// strategy is left out: on this program it grows past 20 GB.
    #[test]
    fn caesar_two_letters() {
        let file = "tests/roulette/caesar-2.pl";
        let source = std::fs::read_to_string(file).unwrap();
        let expected = [
            ("key(0)", 0.0424471725797),
            ("key(1)", 0.000280890177767),
            ("key(2)", 0.015985973189),
            ("key(3)", 0.204215841909),
            ("key(4)", 0.128272226805),
            ("key(5)", 0.0432783515262),
            ("key(6)", 0.00115152237828),
            ("key(7)", 0.0447380435651),
            ("key(8)", 0.0345129490155),
            ("key(9)", 0.141226454641),
            ("key(10)", 0.0440477760457),
            ("key(11)", 0.0243689084225),
            ("key(12)", 0.0516776915026),
            ("key(13)", 0.0201249223785),
            ("key(14)", 0.0890010946345),
            ("key(15)", 0.0509343185288),
            ("key(16)", 0.00903461472261),
            ("key(17)", 0.0546633480843),
            ("key(18)", 7.87394727604e-06),
            ("key(19)", 7.6491436327e-06),
            ("key(20)", 5.00365435789e-06),
            ("key(21)", 2.68368075862e-06),
            ("key(22)", 1.46881953129e-05),
            ("key(23)", 4.24471725797e-10),
            ("key(24)", 4.24471725797e-10),
            ("key(25)", 4.24471858232e-10),
        ];
        for strategy in [Strategy::Auto, Strategy::Trial(Kind::Closed)] {
            one(file, &source, &expected, strategy, Projection::default());
        }
    }

    /// Three diamonds in a row.
    #[test]
    fn diamond() {
        check("../tests/roulette/diamond-3.pl", &[("s(3)", 0.9998500075)]);
    }

    /// The perturbed-memory interpreter on five units of fuel.
    #[test]
    fn hardware() {
        check(
            "../tests/roulette/hardware-5.pl",
            &[
                ("res(0)", 0.000100019998999),
                ("res(1)", 9.98900709734e-05),
                ("res(2)", 9.99000509846e-05),
                ("res(3)", 0.999700069975),
                ("res(4)", 1.19904035492e-07),
            ],
        );
    }

    /// The hidden Markov model, 3 steps: filtering joint.
    #[test]
    fn hmm_filtering_joint() {
        check(
            "../tests/roulette/hmm-filtering-joint-3.pl",
            &[("q", 0.124552678953)],
        );
    }

    /// The hidden Markov model, 3 steps: filtering marginal.
    #[test]
    fn hmm_filtering_marginal() {
        check(
            "../tests/roulette/hmm-filtering-marginal-3.pl",
            &[("q", 0.626965576917)],
        );
    }

    /// The hidden Markov model, 3 steps: marginal likelihood.
    #[test]
    fn hmm_marginal_likelihood() {
        check(
            "../tests/roulette/hmm-marginal-likelihood-3.pl",
            &[("q", 0.0942590625)],
        );
    }

    /// The hidden Markov model, 4 steps: smoothing joint.
    #[test]
    fn hmm_smoothing_joint() {
        check(
            "../tests/roulette/hmm-smoothing-joint-4.pl",
            &[("q", 0.194085985912)],
        );
    }

    /// The hidden Markov model, 4 steps: smoothing marginal.
    #[test]
    fn hmm_smoothing_marginal() {
        check(
            "../tests/roulette/hmm-smoothing-marginal-4.pl",
            &[("q", 0.451015571209)],
        );
    }

    /// A ladder of three rungs, the two last wires' joint as two marginals.
    #[test]
    fn ladder() {
        check(
            "../tests/roulette/ladder-3.pl",
            &[("o1(3)", 0.499000624875), ("o2(3)", 0.499000624875)],
        );
    }

    /// The Markov chain of the paper's Figure 2, three steps.
    #[test]
    fn motivating() {
        check("../tests/roulette/motivating-3.pl", &[("x(3)", 0.2729)]);
    }

    /// A 4-bit message through two noisy diamonds: its distribution and loss.
    #[test]
    fn network() {
        check(
            "../tests/roulette/network-2.pl",
            &[
                ("m(2,0)", 0.0),
                ("m(2,1)", 0.0),
                ("m(2,2)", 0.0),
                ("m(2,3)", 3.125e-06),
                ("m(2,4)", 0.0),
                ("m(2,5)", 3.125e-06),
                ("m(2,6)", 3.125e-06),
                ("m(2,7)", 0.002475),
                ("m(2,8)", 0.0),
                ("m(2,9)", 3.125e-06),
                ("m(2,10)", 3.125e-06),
                ("m(2,11)", 0.002475),
                ("m(2,12)", 3.125e-06),
                ("m(2,13)", 0.002475),
                ("m(2,14)", 0.002475),
                ("m(2,15)", 0.98010625),
                ("m(2,16)", 0.009975),
            ],
        );
    }
}

/// The literature families (generators and notes on the baselines branch),
/// each on a small program in tests/lit/; ProbLog's answers (ProbLog 2.3.0, `-k
/// ddnnf`, one query at a time), which [`scripts/lit_exact.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/lit_exact.py) also gives
/// (its tests/test_lit.py, on the same branch).
mod lit {
    use super::*;

    /// The sum of two digit classifications, after DeepProbLog and Scallop.
    #[test]
    fn mnist_sum() {
        check(
            "../tests/lit/mnist_sum.pl",
            &[
                ("sum(0)", 0.00140544),
                ("sum(1)", 0.00374336),
                ("sum(2)", 0.022827),
                ("sum(3)", 0.03859655),
                ("sum(4)", 0.00591345),
                ("sum(5)", 0.02648234),
                ("sum(6)", 0.04219473),
                ("sum(7)", 0.02116928),
                ("sum(8)", 0.05089386),
                ("sum(9)", 0.08413355),
                ("sum(10)", 0.04095259),
                ("sum(11)", 0.50704523),
                ("sum(12)", 0.02676586),
                ("sum(13)", 0.02359843),
                ("sum(14)", 0.02757976),
                ("sum(15)", 0.03501714),
                ("sum(16)", 0.00668589),
                ("sum(17)", 0.00121972),
                ("sum(18)", 0.03377582),
            ],
        );
    }

    /// cplint's blood type rules on a random pedigree of three generations
    /// of three, some blood types observed. Not under `check`: the
    /// right-linear orders of td and dynamic exceed the byte budget on its
    /// tables.
    #[test]
    fn pedigree() {
        let source = std::fs::read_to_string("tests/lit/pedigree.pl").unwrap();
        let expected = &[
            ("bloodtype(i2_0,a)", 0.31673708),
            ("bloodtype(i2_0,b)", 0.2366961),
            ("bloodtype(i2_0,ab)", 0.17176306),
            ("bloodtype(i2_0,null)", 0.19319018),
            ("bloodtype(i2_1,a)", 0.31798337),
            ("bloodtype(i2_1,b)", 0.23088535),
            ("bloodtype(i2_1,ab)", 0.18444069),
            ("bloodtype(i2_1,null)", 0.17794811),
            ("bloodtype(i2_2,a)", 0.31673708),
            ("bloodtype(i2_2,b)", 0.2366961),
            ("bloodtype(i2_2,ab)", 0.17176306),
            ("bloodtype(i2_2,null)", 0.19319018),
        ];
        for strategy in [Strategy::Trial(Kind::Closed), Strategy::Auto] {
            one("pedigree.pl", &source, expected, strategy, Projection::Off);
            one(
                "pedigree.pl",
                &source,
                expected,
                strategy,
                Projection::Cones { max_share: 1.0 },
            );
        }
        indicated(
            "pedigree.pl",
            &source,
            expected,
            true,
            Strategy::Trial(Kind::Families { search: true }),
            Rows::Classes,
        );
    }

    /// The dynamic sickness domain of the Tp-compilation paper over four
    /// steps, friendships cyclic within a step.
    #[test]
    fn sickness() {
        check(
            "../tests/lit/sickness.pl",
            &[
                ("sick(p0,3)", 0.8272),
                ("sick(p1,3)", 0.89632),
                ("sick(p2,3)", 0.712),
                ("sick(p3,3)", 0.52),
            ],
        );
    }

    /// A two-layer noisy-or diagnosis network, every finding observed.
    #[test]
    fn noisyor() {
        check(
            "../tests/lit/noisyor.pl",
            &[
                ("disease(d0)", 0.00028898761),
                ("disease(d1)", 0.23203622),
                ("disease(d2)", 0.21170946),
                ("disease(d3)", 0.00032129522),
                ("disease(d4)", 0.22811529),
                ("disease(d5)", 0.11132415),
            ],
        );
    }

    /// Mastermind: the code given the black and white pegs of a guess.
    #[test]
    fn mastermind() {
        check(
            "../tests/lit/mastermind.pl",
            &[
                ("code(1,c0)", 0.0),
                ("code(1,c1)", 0.35714286),
                ("code(1,c2)", 0.35714286),
                ("code(1,c3)", 0.28571429),
                ("code(2,c0)", 0.42857143),
                ("code(2,c1)", 0.28571429),
                ("code(2,c2)", 0.28571429),
                ("code(2,c3)", 0.0),
                ("code(3,c0)", 0.42857143),
                ("code(3,c1)", 0.28571429),
                ("code(3,c2)", 0.28571429),
                ("code(3,c3)", 0.0),
            ],
        );
    }

    /// The unsafe queries h0 and h1 and a safe one over a tuple-independent
    /// database.
    #[test]
    fn pdb() {
        check(
            "../tests/lit/pdb.pl",
            &[("h0", 0.2511066), ("h1", 0.64132759), ("safe", 0.50430046)],
        );
    }

    /// Boolean TPC-H queries over a tuple-independent database at SF
    /// 0.0004, as [`gen_tpch.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/gen_tpch.py) writes them: joins, selections and
    /// comparisons. q2 and q7 are the unsafe chain queries, on data small
    /// enough that their lineage still decomposes.
    #[test]
    fn tpch() {
        check(
            "../tests/lit/tpch.pl",
            &[
                ("q2", 0.017751203),
                ("q3", 0.02461413),
                ("q4", 0.46913878),
                ("q6", 0.61112731),
                ("q7", 0.0155477),
                ("q10", 0.048477216),
                ("q11", 0.080382831),
                ("q12", 0.17013583),
                ("q14", 0.33707206),
                ("q15", 0.58742125),
                ("q17", 0.38035305),
            ],
        );
    }

    /// The same queries written through views ([`gen_tpch.py --views`](https://github.com/Tractables/tddlog/blob/baselines/scripts/gen_tpch.py)): a
    /// selection, a join or a subquery as an intermediate predicate. The
    /// answers are the flat program's, and the grounder inlines the views
    /// (src/ground/inline.rs).
    #[test]
    fn tpch_views() {
        check(
            "../tests/lit/tpch_views.pl",
            &[
                ("q2", 0.017751203),
                ("q3", 0.02461413),
                ("q4", 0.46913878),
                ("q6", 0.61112731),
                ("q7", 0.0155477),
                ("q10", 0.048477216),
                ("q11", 0.080382831),
                ("q12", 0.17013583),
                ("q14", 0.33707206),
                ("q15", 0.58742125),
                ("q17", 0.38035305),
            ],
        );
    }

    /// Connection to a disease in a made-up graph in Biomine's format, as
    /// [`gen_biomine.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/gen_biomine.py) writes it: the union of the three best paths to each
    /// gene, and the whole component. Undirected and cyclic.
    #[test]
    fn biomine() {
        check(
            "../tests/lit/biomine3.pl",
            &[
                ("path(phenotype_mim_104300,gene_entrezgene_351)", 0.8622505),
                ("path(phenotype_mim_104300,gene_entrezgene_323)", 0.9107231),
                ("path(phenotype_mim_104300,gene_entrezgene_590)", 0.489378),
                ("path(phenotype_mim_104300,gene_entrezgene_5126)", 0.4204916),
            ],
        );
        check(
            "../tests/lit/biomine.pl",
            &[
                ("path(phenotype_mim_104300,gene_entrezgene_351)", 0.89274736),
                ("path(phenotype_mim_104300,gene_entrezgene_323)", 0.93024004),
                ("path(phenotype_mim_104300,gene_entrezgene_590)", 0.57842804),
                (
                    "path(phenotype_mim_104300,gene_entrezgene_5126)",
                    0.46925437,
                ),
            ],
        );
    }
}

/// Rows of tables with equal parameters, some excluding each other and some
/// not; ProbLog's answers (ProbLog 2.3.0).
#[test]
fn shared_rows() {
    let expected = [
        ("a(1)", 0.25),
        ("a(2)", 0.375),
        ("a(3)", 0.375),
        ("b(1)", 0.28666667),
        ("b(2)", 0.11333333),
        ("b(3)", 0.6),
        ("d(1)", 0.18333333),
        ("d(3)", 0.33333333),
        ("d(4)", 0.48333333),
        ("y", 0.3),
        ("z", 0.3276),
        ("x", 0.5),
    ];
    check("shared_rows.pl", &expected);
}

/// The rows of a table that hold one distribution read a class atom where
/// they exclude each other and read more than one parent, and only there;
/// ProbLog's answers (ProbLog 2.3.0).
#[test]
fn row_classes() {
    let expected = [
        ("a(1)", 0.15384615),
        ("a(2)", 0.46153846),
        ("a(3)", 0.38461538),
        ("b(t)", 0.60839161),
        ("c(x)", 0.4965035),
        ("c(y)", 0.37762238),
        ("c(z)", 0.12587413),
        ("w", 0.33332452),
        ("d", 0.5),
    ];
    check("row_classes.pl", &expected);
    let source = std::fs::read_to_string("examples/row_classes.pl").unwrap();
    let ground = common::ground(&source, Rows::Classes, false);
    let names: Vec<String> = ground.atoms.iter().map(|a| a.to_string()).collect();
    // c's six rows fall into two classes; e's classes are its values, w's
    // rows can hold together and d's read one parent, so none of them is
    // rewritten.
    assert_eq!(
        names.iter().filter(|a| a.starts_with("'$c_")).count(),
        2,
        "{names:?}"
    );
    assert!(
        names
            .iter()
            .filter(|a| a.starts_with("'$c_"))
            .all(|a| a.starts_with("'$c_c_")),
        "{names:?}"
    );
    assert_eq!(
        names.iter().filter(|a| a.starts_with("'$swc_")).count(),
        3,
        "{names:?}"
    );
    assert_eq!(
        names.iter().filter(|a| a.starts_with("'$sww_")).count(),
        4,
        "{names:?}"
    );
    assert_eq!(
        names.iter().filter(|a| a.starts_with("'$swd_")).count(),
        5,
        "{names:?}"
    );
}

/// Views over facts alone, read once or of one rule, are replaced by their
/// bodies in their readers (src/ground/inline.rs), and so is an atom of one
/// rule read once whatever it reads; queried atoms, atoms read negatively,
/// values of an annotated disjunction, recursive atoms and a view whose
/// bodies would take too many copies stay. ProbLog's answers (ProbLog
/// 2.3.0, one query at a time).
#[test]
fn views() {
    check(
        "views.pl",
        &[
            ("q", 0.56323379),
            ("big(3,b)", 0.476),
            ("quiet(a)", 0.549312),
            ("quiet(b)", 0.45),
            ("quiet(c)", 0.7),
            ("second_only", 0.56),
            ("warm", 0.18),
            ("cold", 0.25),
            ("looped", 0.105),
            ("wide", 0.01562265),
        ],
    );
    let source = std::fs::read_to_string("examples/views.pl").unwrap();
    let ground = common::ground(&source, Rows::Classes, false);
    let names: Vec<String> = ground.atoms.iter().map(|a| a.to_string()).collect();
    for gone in [
        "ordered(",
        "home_supplier(",
        "local_order(",
        "shipped(",
        "big(1,a)",
        "big(2,a)",
        "buyer(a)",
        "buyer(b)",
    ] {
        assert!(
            !names.iter().any(|a| a.starts_with(gone)),
            "{gone} is inlined: {names:?}"
        );
    }
    for kept in [
        "big(3,b)",
        "busy(a)",
        "busy(b)",
        "colour(red)",
        "colour(blue)",
        "many",
    ] {
        assert!(names.iter().any(|a| a == kept), "{kept} stays: {names:?}");
    }
    // `looped`'s demand grounds `reach` from its bound end
    // (src/ground/linear.rs), the smaller grounding since the chain's base
    // case, true in every world, keeps one rule. One atom of the chain
    // stays, and the views inlined are 14.
    assert!(
        names.iter().any(|a| a == "'$r_reach_1'(1,3)"),
        "reach stays: {names:?}"
    );
    assert_eq!(ground.stats.inlined, 14, "{names:?}");
    // The body of `shipped(1)` that reads `item(1,1)` meets its complement
    // in `second_only`'s rule and is dropped.
    let second = names.iter().position(|a| a == "second_only").unwrap() as u32;
    let rules: Vec<_> = ground.rules.iter().filter(|r| r.head == second).collect();
    assert_eq!(rules.len(), 1, "{}", ground);
}

/// A derived atom whose rule bodies together are valid is certain
/// (src/ground/cover.rs). The answers are ProbLog's.
#[test]
fn cover() {
    check(
        "cover.pl",
        &[
            ("u(1)", 0.2),
            ("t(1)", 0.9),
            ("w(1)", 0.145),
            ("y", 0.87),
            ("z", 1.0),
            ("z2", 0.7),
        ],
    );
    let source = std::fs::read_to_string("examples/cover.pl").unwrap();
    let ground = common::ground(&source, Rows::Classes, false);
    let names: Vec<String> = ground.atoms.iter().map(|a| a.to_string()).collect();
    // u's class atom is certain, and then t's: both tables read their
    // switches alone. w's class atom stays, and y and z2 are not certain.
    assert!(names.iter().any(|a| a.starts_with("'$c_w_")), "{names:?}");
    assert!(
        names
            .iter()
            .filter(|a| a.starts_with("'$c_"))
            .all(|a| a.starts_with("'$c_w_")),
        "{names:?}"
    );
    for r in &ground.rules {
        let head = &names[r.head as usize];
        if head.starts_with("u(") || head.starts_with("t(") {
            assert!(
                r.body.iter().all(|l| ground.is_fact(l.atom())),
                "{head} reads its switches alone"
            );
        }
        if head == "y" || head == "z2" {
            assert!(!r.body.is_empty(), "{head} is not certain");
        }
    }
}

/// examples/README.md, the index a reader finds the examples by, links
/// every file in `examples/`.
#[test]
fn the_index_lists_every_example() {
    let index = std::fs::read_to_string("examples/README.md").unwrap();
    for entry in std::fs::read_dir("examples").unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        assert!(
            name == "README.md" || index.contains(&format!("[{name}]({name})")),
            "examples/README.md does not list {name}"
        );
    }
}
