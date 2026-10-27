//! Network files read as programs (`tddlog::network`): each reads as the
//! program `scripts/gen_bn.py` writes for it, line for line, and the
//! command line answers it as ProbLog or enumeration does, a Markov
//! network's partition function on the `Z:` line.
//!
//! The programs are the fixtures gen_bn.py wrote: tests/regress/markov*.pl
//! and potts.pl, examples/bn_cancer.pl and bn_asia_evidence.pl, and
//! tests/networks/*.pl, whose comments say how; tests/test_gen_bn.py checks
//! that gen_bn.py still writes the last.
use std::process::{Command, Output};

use tddlog::network::{self, Evidence, Format, Queries, ReadOptions};

/// The program a network file reads as, with its queries, its evidence file
/// and whether its rows are rescaled.
fn read(path: &str, queries: Queries, evidence: Option<&str>, rescale: bool) -> String {
    let data = std::fs::read(path).unwrap();
    let evidence_data = evidence.map(|e| std::fs::read(e).unwrap());
    let options = ReadOptions {
        queries,
        evidence: evidence
            .map(|e| Evidence::of_path(e, evidence_data.as_deref().unwrap()).unwrap()),
        rescale,
    };
    let format = Format::of_path(path).unwrap();
    network::read(&data, format, &options)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .program
}

/// The lines of a program gen_bn.py wrote, without the comments a fixture
/// adds after its first line (the offset line, which gen_bn.py writes).
fn written(path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap();
    text.lines()
        .enumerate()
        .filter(|(i, l)| {
            (*i == 0 && l.starts_with("% log10 Z - log10 P(e) = ")) || !l.starts_with('%')
        })
        .map(|(_, l)| l.to_string())
        .collect()
}

#[test]
fn a_network_reads_as_the_program_gen_bn_py_writes() {
    let named = |names: &[&str]| Queries::Named(names.iter().map(|n| n.to_string()).collect());
    let cases: [(&str, Queries, Option<&str>, bool, &str); 12] = [
        (
            "tests/regress/markov.uai",
            Queries::All,
            None,
            false,
            "tests/regress/markov.pl",
        ),
        (
            "tests/regress/markov.uai",
            Queries::All,
            Some("tests/regress/markov.uai.evid"),
            false,
            "tests/regress/markov_ev.pl",
        ),
        (
            "tests/regress/markov_pr.uai",
            Queries::All,
            None,
            false,
            "tests/regress/markov_pr.pl",
        ),
        (
            "tests/regress/markov_pr.uai",
            Queries::All,
            Some("tests/regress/markov_pr.uai.evid"),
            false,
            "tests/regress/markov_pr_ev.pl",
        ),
        (
            "tests/regress/potts.uai",
            Queries::None,
            None,
            false,
            "tests/regress/potts.pl",
        ),
        (
            "examples/cancer.net",
            Queries::Leaves,
            None,
            false,
            "examples/bn_cancer.pl",
        ),
        (
            "examples/asia.net",
            Queries::All,
            Some("tests/networks/asia_leaves.inst"),
            false,
            "examples/bn_asia_evidence.pl",
        ),
        (
            "tests/networks/sprinkler.bif",
            named(&["WetGrass"]),
            None,
            false,
            "tests/networks/sprinkler_bif.pl",
        ),
        (
            "tests/networks/sprinkler.xdsl",
            Queries::Leaves,
            Some("tests/networks/sprinkler.inst"),
            false,
            "tests/networks/sprinkler_xdsl.pl",
        ),
        (
            "tests/networks/percent.hugin",
            Queries::All,
            None,
            true,
            "tests/networks/percent.pl",
        ),
        (
            "tests/networks/chain.uai",
            Queries::Leaves,
            None,
            false,
            "tests/networks/chain.pl",
        ),
        (
            "tests/networks/noisy.xdsl",
            Queries::All,
            None,
            false,
            "tests/networks/noisy.pl",
        ),
    ];
    for (path, queries, evidence, rescale, program) in cases {
        let read = read(path, queries, evidence, rescale);
        let got: Vec<&str> = read.lines().collect();
        assert_eq!(got, written(program), "{path} against {program}");
    }
}

fn tddlog(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tddlog"))
        .args(args)
        .output()
        .unwrap()
}

/// The answer lines of a run, `atom:<tab>p`, and the base-10 logarithm on
/// its `label:` line, where it prints one.
fn answers(out: &Output, label: &str) -> (Vec<(String, f64)>, Option<f64>) {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = std::str::from_utf8(&out.stdout).unwrap();
    let mut answers = Vec::new();
    let mut log10 = None;
    for line in text.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields[0] == format!("{label}:") {
            log10 = Some(fields[2].strip_prefix("log10 ").unwrap().parse().unwrap());
        } else {
            let atom = fields[0].strip_suffix(':').unwrap().to_string();
            answers.push((atom, fields[1].parse().unwrap()));
        }
    }
    (answers, log10)
}

fn assert_close(what: &str, got: &[(String, f64)], expected: &[(&str, f64)], tolerance: f64) {
    let got: Vec<(&str, f64)> = got.iter().map(|(a, p)| (a.as_str(), *p)).collect();
    assert_eq!(got.len(), expected.len(), "{what}: {got:?}");
    for ((a, p), (b, q)) in got.iter().zip(expected) {
        assert_eq!(a, b, "{what}");
        assert!((p - q).abs() < tolerance, "{what}: {a} = {p} against {q}");
    }
}

#[test]
fn a_markov_network_prints_its_partition_function() {
    // Z by enumeration of the UAI files' worlds in exact fractions
    // (tests/it/evidence_probability.rs): 325 and, given markov.uai.evid,
    // 165; markov_pr.uai's 19947/25 and 11853/40; potts.uai's 225/2,
    // through the program that breaks its colour symmetry.
    for (args, z) in [
        (&["tests/regress/markov.uai"][..], 325.0),
        (
            &[
                "tests/regress/markov.uai",
                "--evidence",
                "tests/regress/markov.uai.evid",
            ][..],
            165.0,
        ),
        (
            &["tests/regress/markov_pr.uai", "--no-queries"][..],
            19947.0 / 25.0,
        ),
        (
            &[
                "tests/regress/markov_pr.uai",
                "--no-queries",
                "--evidence",
                "tests/regress/markov_pr.uai.evid",
            ][..],
            11853.0 / 40.0,
        ),
        (
            &["tests/regress/potts.uai", "--no-queries"][..],
            225.0 / 2.0,
        ),
    ] {
        let out = tddlog(&[args, &["--evidence-probability", "--precision", "13"]].concat());
        let (_, log10_z) = answers(&out, "Z");
        let log10_z = log10_z.unwrap_or_else(|| panic!("{args:?}: no Z line"));
        assert!(
            (log10_z - f64::log10(z)).abs() < 1e-12,
            "{args:?}: log10 Z = {log10_z} against {}",
            z.log10()
        );
    }
    let out = tddlog(&["tests/regress/markov.uai", "--evidence-probability"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.ends_with("\nZ:\t3.250000e2\tlog10 2.511883\n"),
        "{text}"
    );
}

#[test]
fn a_network_answers_as_problog_and_enumeration_do() {
    // markov.uai's marginals by enumeration (tests/regress/markov.pl).
    let out = tddlog(&["tests/regress/markov.uai", "--precision", "12"]);
    let markov = [
        ("x0(v_0)", 17.0 / 65.0),
        ("x0(v_1)", 48.0 / 65.0),
        ("x1(v_0)", 6.0 / 13.0),
        ("x1(v_1)", 7.0 / 13.0),
        ("x2(v_0)", 33.0 / 65.0),
        ("x2(v_1)", 2.0 / 13.0),
        ("x2(v_2)", 22.0 / 65.0),
    ];
    assert_close("markov.uai", &answers(&out, "Z").0, &markov, 1e-11);
    // ProbLog's answers on the programs gen_bn.py writes
    // (tests/it/examples.rs).
    let out = tddlog(&[
        "examples/cancer.net",
        "--query",
        "leaves",
        "--precision",
        "9",
    ]);
    let cancer = [
        ("xray(positive)", 0.208141),
        ("xray(negative)", 0.791859),
        ("dyspnoea(true)", 0.3040705),
        ("dyspnoea(false)", 0.6959295),
    ];
    assert_close("cancer.net", &answers(&out, "P(e)").0, &cancer, 1e-7);
    let out = tddlog(&[
        "examples/asia.net",
        "--evidence",
        "tests/networks/asia_leaves.inst",
        "--precision",
        "9",
    ]);
    let asia = [
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
    ];
    assert_close("asia.net", &answers(&out, "P(e)").0, &asia, 1e-7);
    // ProbLog's answers on gen_bn.py's programs for tests/networks/,
    // one query at a time.
    for (args, expected) in [
        (
            &["tests/networks/sprinkler.bif", "--query", "WetGrass"][..],
            &[("wetgrass(true)", 0.6471), ("wetgrass(false)", 0.3529)][..],
        ),
        (
            &[
                "tests/networks/sprinkler.xdsl",
                "--query",
                "leaves",
                "--evidence",
                "tests/networks/sprinkler.inst",
            ][..],
            &[("wetgrass(t)", 0.927), ("wetgrass(f)", 0.073)][..],
        ),
        (
            &["tests/networks/percent.hugin", "--rescale"][..],
            &[
                ("temp(v__12_)", 0.2),
                ("temp(v_12_20_)", 0.5),
                ("temp(v__20_)", 0.3),
                ("ice(yes)", 0.3075),
                ("ice(no)", 0.6925),
            ][..],
        ),
        (
            &["tests/networks/chain.uai", "--query", "leaves"][..],
            &[
                ("x2(v_0)", 0.25833517),
                ("x2(v_1)", 0.29227303),
                ("x2(v_2)", 0.31681688),
            ][..],
        ),
        (
            &["tests/networks/noisy.xdsl"][..],
            &[
                ("mantle(present)", 0.3),
                ("mantle(absent)", 0.7),
                ("f5(s0)", 0.0),
                ("f5(s1)", 0.99681353),
                ("f5(s2)", 0.002189654),
                ("f5(s3)", 0.000996812),
                ("fault(s0)", 0.0),
                ("fault(s1)", 0.99273708),
                ("fault(s2)", 0.0072629199),
            ][..],
        ),
    ] {
        let out = tddlog(&[args, &["--precision", "9"]].concat());
        assert_close(args[0], &answers(&out, "P(e)").0, expected, 1e-7);
    }
}

#[test]
fn program_prints_what_the_network_reads_as() {
    let out = tddlog(&["--program", "examples/cancer.net", "--query", "leaves"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = std::str::from_utf8(&out.stdout).unwrap();
    assert_eq!(
        text.lines().collect::<Vec<_>>(),
        written("examples/bn_cancer.pl")
    );
}

#[test]
fn network_options_fail_where_they_would_do_nothing() {
    for args in [
        &["examples/alarm.pl", "--query", "all"][..],
        &["examples/alarm.pl", "--rescale"][..],
        &["examples/alarm.pl", "--program"][..],
        &[
            "examples/alarm.pl",
            "--evidence",
            "tests/networks/asia_leaves.inst",
        ][..],
        &["examples/asia.net.gz"][..],
        &[
            "examples/asia.net",
            "--query",
            "all",
            "--no-queries",
            "--evidence-probability",
        ][..],
        &["examples/asia.net", "--program", "--stats"][..],
        &[
            "examples/asia.net",
            "--evidence",
            "tests/regress/markov.uai",
        ][..],
        &["examples/asia.net", "--query", ""][..],
    ] {
        let out = tddlog(args);
        assert_eq!(
            out.status.code(),
            Some(2),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    // A network the reader refuses, and UAI evidence for another format.
    for args in [
        &["tests/networks/sprinkler.bif", "--query", "Nothing"][..],
        &[
            "examples/asia.net",
            "--evidence",
            "tests/regress/markov.uai.evid",
        ][..],
    ] {
        let out = tddlog(args);
        assert_eq!(
            out.status.code(),
            Some(1),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
