//! Formulas kept in skip form (`Options::skip_levels`, `--skip-levels`):
//! under `program`, `program:fanin` and `decision`, the answers and P(e)
//! of random circuits (by enumeration of their worlds, as in
//! tests/it/guards.rs), of the chains, grids and coupon collectors whose
//! cuts `program` names (a forward pass, tests/it/naming.rs), of a formula
//! narrowed to its support (tests/it/narrowed.rs) and of
//! examples/adder.pl, alea_dice.pl and equal_gates.pl (hand-computed and
//! ProbLog's answers) are the references' and the stored form's to 1e-12.
//! Under `auto`, whose trials go on in skip form only where their first
//! large formula keeps few pairs there, the answers are those of `auto`
//! without it. Every other strategy, a run without queries, and the
//! options that stop before the compile refuse skip form.

use std::process::Command;

use crate::common;
use crate::guards::Circuit;
use crate::naming::{Weather, coupon, coupon_exact, grid, grid_exact};
use crate::narrowed;

use tddlog::{Error, Kind, Options, Projection, Strategy};

/// The layouts skip form runs under.
const LAYOUTS: [Kind; 3] = [
    Kind::Program { fan_in: false },
    Kind::Program { fan_in: true },
    Kind::Decision,
];

/// The options of a run under `kind`, P(e) asked, in skip form where
/// `skip`.
fn options(kind: Kind, skip: bool) -> Options {
    Options {
        evidence_probability: true,
        skip_levels: skip,
        ..common::options(Strategy::Trial(kind), Projection::default())
    }
}

/// What [`check`] ran: the layouts, and the largest formula's pairs of
/// the stored runs and of the skip runs together.
#[derive(Default)]
struct Ran {
    layouts: usize,
    stored_pairs: usize,
    skip_pairs: usize,
}

/// Runs `source` under each layout, stored and in skip form, and asserts
/// that the skip run's answers and P(e) are the stored run's to 1e-12,
/// and its answers `expected` to `tolerance`, by name, with P(e)
/// `evidence` where it is given. A layout the stored form refuses
/// (`decision` on a program without guards) is refused in skip form too.
fn check(
    what: &str,
    source: &str,
    expected: &[(String, f64)],
    evidence: Option<f64>,
    tolerance: f64,
    ran: &mut Ran,
) {
    for kind in LAYOUTS {
        let what = format!("{what}, {kind}");
        let stored = common::run(source, &options(kind, false));
        let skip = common::run(source, &options(kind, true));
        let (stored, skip) = match (stored, skip) {
            (Ok(stored), Ok(skip)) => (stored, skip),
            (Err(Error::Vtree(a)), Err(Error::Vtree(b))) if kind == Kind::Decision => {
                assert_eq!(a, b, "{what}");
                continue;
            }
            (stored, skip) => panic!("{what}: stored {:?}, skip {:?}", stored.err(), skip.err()),
        };
        let (a, b) = (
            stored.results.as_ref().expect("consistent evidence"),
            skip.results.as_ref().expect("consistent evidence"),
        );
        assert_eq!(a.len(), b.len(), "{what}");
        for (a, b) in a.iter().zip(b) {
            assert_eq!(a.to_string(), b.to_string(), "{what}");
            assert!(
                (a.probability - b.probability).abs() < 1e-12,
                "{what}: {a} stored {} against skip {}",
                a.probability,
                b.probability
            );
        }
        for (q, p) in expected {
            let got = b
                .iter()
                .find(|r| r.to_string() == *q)
                .unwrap_or_else(|| panic!("{what}: no answer for {q}"));
            assert!(
                (got.probability - p).abs() < tolerance,
                "{what}: {q} = {} against {p}",
                got.probability
            );
        }
        let (ea, eb) = (
            stored.evidence_probability.expect("P(e)"),
            skip.evidence_probability.expect("P(e)"),
        );
        assert!((eb.ratio(ea) - 1.0).abs() < 1e-12, "{what}: P(e)");
        if let Some(e) = evidence {
            let got = eb.to_f64();
            assert!(
                (got - e).abs() < 1e-12 * e,
                "{what}: P(e) = {got} against {e}"
            );
        }
        ran.layouts += 1;
        ran.stored_pairs += stored.stats.max_pairs;
        ran.skip_pairs += skip.stats.max_pairs;
    }
}

/// Random circuits of if-then-elses, guarded products and sums, against
/// the enumeration of their worlds; `decision` runs on those the grounder
/// conditions on their guards. Skip form holds fewer pairs than stored.
#[test]
fn circuits_in_skip_form_keep_their_answers() {
    let mut ran = Ran::default();
    for seed in 0..24 {
        let circuit = Circuit::random(seed);
        let what = format!("seed {seed}");
        check(
            &what,
            &circuit.source(),
            &circuit.answers(),
            None,
            1e-12,
            &mut ran,
        );
    }
    assert!(ran.layouts > 48, "{} layouts ran", ran.layouts);
    assert!(
        ran.skip_pairs < ran.stored_pairs,
        "skip form's largest formulas {} pairs against stored {}",
        ran.skip_pairs,
        ran.stored_pairs
    );
}

/// The weather factory, a link-failure grid and a coupon collector, with
/// and without evidence, against a forward pass.
#[test]
fn named_cuts_in_skip_form_keep_the_forward_pass() {
    let mut ran = Ran::default();
    for (factories, days) in [(2, 4), (3, 6), (3, 9)] {
        let weather = Weather { factories, days };
        let last = days - 1;
        let what = format!("weather {factories} {days}");
        let (done, _) = weather.exact(None);
        let source = weather.program(&format!("query(done_{last}).\n"));
        check(
            &what,
            &source,
            &[(format!("done_{last}"), done)],
            Some(1.0),
            1e-12,
            &mut ran,
        );
        let (both, e) = weather.exact(Some((2, false)));
        let source = weather.program(&format!("evidence(w_2, false).\nquery(done_{last}).\n"));
        check(
            &what,
            &source,
            &[(format!("done_{last}"), both / e)],
            Some(e),
            1e-12,
            &mut ran,
        );
    }
    for side in [3, 4, 5] {
        let last = side - 1;
        let want = grid_exact(side);
        let source = grid(
            side,
            &format!("evidence(c_{last}_{last}).\nquery(c_{last}_{last}).\n"),
        );
        let expected = [(format!("c_{last}_{last}"), 1.0)];
        check(
            &format!("grid {side}"),
            &source,
            &expected,
            Some(want),
            1e-12,
            &mut ran,
        );
    }
    for draws in [4, 6, 8] {
        let middle = draws / 2;
        let (both, e) = coupon_exact(draws, Some(middle));
        let source = coupon(draws, &format!("evidence(g_0_{middle}).\nquery(all).\n"));
        let expected = [("all".to_string(), both / e)];
        check(
            &format!("coupon {draws}"),
            &source,
            &expected,
            Some(e),
            1e-12,
            &mut ran,
        );
    }
    assert!(ran.layouts >= 2 * 14, "{} layouts ran", ran.layouts);
}

/// A formula that depends on fewer facts than its rules mention, with and
/// without evidence, against the hand-computed answers.
#[test]
fn a_narrowed_formula_in_skip_form_is_the_hand_computed_answer() {
    let mut ran = Ran::default();
    for evidence in [None, Some(true), Some(false)] {
        let what = format!("evidence {evidence:?}");
        let source = narrowed::program(evidence);
        check(
            &what,
            &source,
            &narrowed::exact(evidence),
            None,
            1e-12,
            &mut ran,
        );
    }
    assert!(ran.layouts >= 6, "{} layouts ran", ran.layouts);
}

/// The examples: the adder's 64 worlds summed by hand, the dice by
/// counting, and ProbLog 2.3.0's answers to equal_gates.pl, which it
/// prints to eight digits.
#[test]
fn examples_in_skip_form_keep_their_answers() {
    let mut ran = Ran::default();
    let e = 23.0 / 50.0;
    let read = |file: &str| {
        let data = std::fs::read(format!("examples/{file}")).unwrap();
        let input = tddlog::read(&data, tddlog::Format::of_path(file), &Default::default());
        input.unwrap().program.into_owned()
    };
    let named = |answers: &[(&str, f64)]| -> Vec<(String, f64)> {
        answers.iter().map(|&(q, p)| (q.to_string(), p)).collect()
    };
    let adder = named(&[
        ("sum(1)", 113.0 / 500.0 / e),
        ("sum(2)", 2857.0 / 12500.0 / e),
        ("at_least_8", 2909.0 / 12500.0 / e),
        ("x(0)", 7.0 / 25.0 / e),
    ]);
    check(
        "adder.pl",
        &read("adder.pl"),
        &adder,
        Some(e),
        1e-12,
        &mut ran,
    );
    let dice = named(&[("q1", 1.0 / 6.0), ("q2", 2.0 / 3.0)]);
    check(
        "alea_dice.pl",
        &read("alea_dice.pl"),
        &dice,
        None,
        1e-12,
        &mut ran,
    );
    let gates = named(&[
        ("s1", 0.57480653),
        ("t1", 0.57480653),
        ("differ", 0.0),
        ("both", 0.013542562),
        ("either", 0.4544282),
        ("s0", 0.54858126),
    ]);
    check(
        "equal_gates.pl",
        &read("equal_gates.pl"),
        &gates,
        None,
        1e-8,
        &mut ran,
    );
    assert!(ran.layouts >= 6, "{} layouts ran", ran.layouts);
}

/// Queries against evidence, against the enumeration of the worlds: facts
/// of the evidence's scope, each a literal the pass down reads; atoms whose
/// formulas the evidence's scope holds, walked against the counted
/// evidence; and atoms and facts it does not hold, each conjoined with the
/// evidence. Then evidence whose mass is under the floats' range, counted
/// again in `Count`s.
#[test]
fn queries_read_the_counted_evidence_in_skip_form() {
    let mut ran = Ran::default();
    let p = [0.3, 0.6, 0.45, 0.7, 0.2, 0.55];
    let names = ["a", "b", "c", "d", "f", "g"];
    let mut source: String = names
        .iter()
        .zip(p)
        .map(|(n, p)| format!("{p}::{n}.\n"))
        .collect();
    source.push_str(
        "x :- a, b.\nx :- \\+a, c.\ny :- x, d.\ny :- \\+x, \\+d, b.\n\
         z :- f, g.\nz :- \\+f, a.\nevidence(y).\n",
    );
    let queries = ["a", "\\+b", "c", "d", "x", "\\+x", "y", "z", "f", "\\+g"];
    for q in queries {
        source.push_str(&format!("query({q}).\n"));
    }
    // Per world, the facts and the atoms derived, by name.
    let world = |w: u32| -> Vec<(&str, bool)> {
        let at = |i: usize| w >> i & 1 == 1;
        let (a, b, c, d, f, g) = (at(0), at(1), at(2), at(3), at(4), at(5));
        let x = (a && b) || (!a && c);
        let y = (x && d) || (!x && !d && b);
        let z = (f && g) || (!f && a);
        vec![
            ("a", a),
            ("b", b),
            ("c", c),
            ("d", d),
            ("f", f),
            ("g", g),
            ("x", x),
            ("y", y),
            ("z", z),
        ]
    };
    let weight = |w: u32| -> f64 {
        (0..6)
            .map(|i| if w >> i & 1 == 1 { p[i] } else { 1.0 - p[i] })
            .product()
    };
    let value = |w: u32, q: &str| -> bool {
        let (name, positive) = match q.strip_prefix("\\+") {
            Some(n) => (n, false),
            None => (q, true),
        };
        world(w).iter().find(|(n, _)| *n == name).unwrap().1 == positive
    };
    let e: f64 = (0..64u32).filter(|&w| value(w, "y")).map(weight).sum();
    let expected: Vec<(String, f64)> = queries
        .iter()
        .map(|q| {
            let both: f64 = (0..64u32)
                .filter(|&w| value(w, "y") && value(w, q))
                .map(weight)
                .sum();
            (q.to_string(), both / e)
        })
        .collect();
    check("evidence y", &source, &expected, Some(e), 1e-12, &mut ran);

    // Forty facts of probability 1e-9 observed together: a mass of 1e-360.
    let mut tiny = String::from("0.3::a.\n0.6::b.\nu :- a.\nu :- b.\nw :- a, b.\n");
    for i in 1..=40 {
        tiny.push_str(&format!("1e-9::h{i}.\n"));
    }
    let body: Vec<String> = (1..=40).map(|i| format!("h{i}")).collect();
    tiny.push_str(&format!("e1 :- {}.\n", body.join(", ")));
    tiny.push_str("evidence(e1).\nevidence(u).\n");
    for q in ["a", "\\+b", "w", "h3", "\\+h7"] {
        tiny.push_str(&format!("query({q}).\n"));
    }
    let u = 1.0 - 0.7 * 0.4;
    let expected = [
        ("a".to_string(), 0.3 / u),
        ("\\+b".to_string(), 0.3 * 0.4 / u),
        ("w".to_string(), 0.18 / u),
        ("h3".to_string(), 1.0),
        ("\\+h7".to_string(), 0.0),
    ];
    check("a mass of 1e-360", &tiny, &expected, None, 1e-12, &mut ran);
    assert!(ran.layouts >= 4, "{} layouts ran", ran.layouts);
}

/// Under `auto`, the random circuits and the examples answer as `auto`
/// does without skip trials; none of them holds a formula large enough to
/// be probed, so each run is the stored one.
#[test]
fn auto_with_skip_trials_keeps_its_answers() {
    let auto = |skip: bool| Options {
        evidence_probability: true,
        skip_levels: skip,
        ..common::options(Strategy::Auto, Projection::default())
    };
    let mut sources: Vec<(String, String)> = (0..8)
        .map(|seed| (format!("seed {seed}"), Circuit::random(seed).source()))
        .collect();
    for file in ["adder.pl", "alea_dice.pl", "equal_gates.pl"] {
        let data = std::fs::read(format!("examples/{file}")).unwrap();
        let input = tddlog::read(&data, tddlog::Format::of_path(file), &Default::default());
        sources.push((file.to_string(), input.unwrap().program.into_owned()));
    }
    for (what, source) in &sources {
        let stored = common::run(source, &auto(false)).unwrap();
        let skip = common::run(source, &auto(true)).unwrap();
        assert!(!skip.stats.skip_form, "{what}");
        assert_eq!(stored.kind, skip.kind, "{what}");
        let (a, b) = (stored.results.unwrap(), skip.results.unwrap());
        assert_eq!(a.len(), b.len(), "{what}");
        for (a, b) in a.iter().zip(&b) {
            assert_eq!(a.to_string(), b.to_string(), "{what}");
            assert_eq!(a.probability, b.probability, "{what}: {a}");
        }
        let (ea, eb) = (
            stored.evidence_probability.expect("P(e)"),
            skip.evidence_probability.expect("P(e)"),
        );
        assert_eq!(ea.to_f64(), eb.to_f64(), "{what}: P(e)");
    }
}

/// Skip form is refused, naming the conflict, under a dynamic layout,
/// `td`, `families` and `closed`, without the queries, and for the ground
/// program, which comes before the compile.
#[test]
fn skip_form_is_refused_where_it_does_not_run() {
    let source = Circuit::random(1).source();
    let refused = |options: &Options, message: &str| match common::run(&source, options) {
        Err(Error::Options(e)) => assert!(e.contains(message), "{e}"),
        other => panic!("{message}: {:?}", other.map(|_| ())),
    };
    let skip = |strategy: Strategy| Options {
        skip_levels: true,
        ..Options::with(strategy)
    };
    refused(
        &skip(Strategy::Trial(Kind::Dynamic)),
        "its vtree search reads stored levels",
    );
    for kind in [
        Kind::Td,
        Kind::Families { search: false },
        Kind::Families { search: true },
        Kind::Closed,
    ] {
        refused(
            &skip(Strategy::Trial(kind)),
            &format!("not `{kind}`: its tables are built in the stored form"),
        );
    }
    let without_queries = Options {
        queries: false,
        evidence_probability: true,
        ..skip(Strategy::Trial(Kind::Program { fan_in: false }))
    };
    refused(&without_queries, "skip levels need the queries");
    let auto_without_queries = Options {
        queries: false,
        evidence_probability: true,
        ..skip(Strategy::Auto)
    };
    refused(&auto_without_queries, "skip levels need the queries");
    match tddlog::rewritten(&source, &skip(Strategy::Trial(Kind::Decision))) {
        Err(Error::Options(e)) => assert!(e.contains("skip levels would do nothing"), "{e}"),
        other => panic!("the ground program in skip form: {:?}", other.map(|_| ())),
    }
}

/// The command line: `--skip-levels` answers as the stored form does,
/// says so in `--stats`, answers under `auto` as `auto` does, and is
/// refused as an options error (exit code 2) under `closed`, without the
/// queries and with the options that stop before the compile.
#[test]
fn the_command_line_takes_skip_levels_under_its_layouts() {
    let tddlog = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_tddlog"))
            .args(args)
            .output()
            .unwrap()
    };
    let program = ["--vtree", "program", "examples/adder.pl"];
    let stored = tddlog(&program);
    let skip = tddlog(&[&["--skip-levels", "--stats"][..], &program].concat());
    assert!(
        skip.status.success(),
        "{}",
        String::from_utf8_lossy(&skip.stderr)
    );
    assert_eq!(stored.stdout, skip.stdout);
    let stats = String::from_utf8_lossy(&skip.stderr);
    assert!(stats.contains("vtree program in skip form over"), "{stats}");
    let auto = tddlog(&["examples/adder.pl"]);
    let auto_skip = tddlog(&["--skip-levels", "examples/adder.pl"]);
    assert!(
        auto_skip.status.success(),
        "{}",
        String::from_utf8_lossy(&auto_skip.stderr)
    );
    assert_eq!(auto.stdout, auto_skip.stdout);
    for (args, message) in [
        (
            &["--skip-levels", "--vtree", "closed", "examples/adder.pl"][..],
            "not `closed`",
        ),
        (
            &[
                "--skip-levels",
                "--vtree",
                "program",
                "--no-queries",
                "--evidence-probability",
                "examples/adder.pl",
            ],
            "skip levels need the queries",
        ),
        (
            &["--skip-levels", "--ground", "examples/adder.pl"],
            "--ground stops before the compile, so --skip-levels would do nothing",
        ),
        (
            &["--skip-levels", "--program", "examples/adder.pl"],
            "--program prints the program and stops, so --skip-levels would do nothing",
        ),
        (
            &["--skip-levels", "--cnf", "examples/adder.pl"],
            "--cnf prints the weighted CNF and stops, so --skip-levels would do nothing",
        ),
        (
            &["--skip-levels", "--skip-levels", "examples/adder.pl"],
            "--skip-levels is given twice",
        ),
    ] {
        let out = tddlog(args);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{args:?}: {stderr}");
        assert!(stderr.contains(message), "{args:?}: {stderr}");
    }
}
