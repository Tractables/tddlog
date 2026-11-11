//! Network files read as programs (`tddlog::network`): each reads as the
//! program beside it, line for line, and the command line answers it as
//! ProbLog or enumeration does, a Markov network's partition function on
//! the `Z:` line.
//!
//! The programs are the ones the benchmarks' Python script `gen_bn.py`,
//! which the reader replaced, wrote: tests/regress/markov*.pl and
//! potts.pl, examples/bn_cancer.pl and bn_car_evidence.pl, and
//! tests/networks/*.pl, whose comments say how; the baselines branch
//! checks that the script still writes them.
use std::process::{Command, Output};

use crate::common::assert_close;
use tddlog::network::{self, Evidence, Format, Queries, ReadOptions};

/// The program a network file reads as, with its queries, its evidence file
/// and whether its rows are rescaled: every variable's rows, as `--program`
/// prints it.
fn read(path: &str, queries: Queries, evidence: Option<&str>, rescale: bool) -> String {
    let data = std::fs::read(path).unwrap();
    let evidence_data = evidence.map(|e| std::fs::read(e).unwrap());
    let options = ReadOptions {
        queries,
        evidence: evidence
            .map(|e| Evidence::of_path(e, evidence_data.as_deref().unwrap()).unwrap()),
        rescale,
        whole: true,
        ..ReadOptions::default()
    };
    let format = Format::of_path(path).unwrap();
    network::read(&data, format, &options)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .program
}

/// The lines of a program the script wrote, without the comments a
/// fixture adds after its first line (the offset line, which the program
/// states).
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
fn a_network_reads_as_the_program_beside_it() {
    let named = |names: &[&str]| Queries::Named(names.iter().map(|n| n.to_string()).collect());
    let cases: [(&str, Queries, Option<&str>, bool, &str); 13] = [
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
            "tests/networks/asia.net",
            Queries::All,
            Some("tests/networks/asia_leaves.inst"),
            false,
            "tests/networks/bn_asia_evidence.pl",
        ),
        (
            "examples/car.net",
            named(&["fuel", "plugs", "starter"]),
            Some("examples/car.inst"),
            false,
            "examples/bn_car_evidence.pl",
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

/// What tddlog answers is the part of a network's program that its queries
/// and evidence read: asia.net queried at `either` alone is answered
/// without the rows of `bronc`, `xray` and `dysp`, which `--program`
/// prints, and its answers are the whole program's, to the bit.
#[test]
fn a_network_is_answered_on_the_rows_its_queries_read() {
    let data = std::fs::read("tests/networks/asia.net").unwrap();
    let solved = |whole| {
        let options = ReadOptions {
            queries: Queries::Named(vec!["either".into()]),
            whole,
            ..ReadOptions::default()
        };
        let input = tddlog::read(&data, tddlog::Format::Network(Format::Hugin), &options).unwrap();
        let solution =
            tddlog::solve(&input.program, &tddlog::Options::default(), &mut |_| {}).unwrap();
        let answers: Vec<(String, u64)> = solution
            .answers
            .unwrap()
            .iter()
            .map(|a| (a.to_string(), a.probability.to_bits()))
            .collect();
        (input.program.into_owned(), answers)
    };
    let (part, answers) = solved(false);
    let (whole, whole_answers) = solved(true);
    for name in ["bronc", "xray", "dysp"] {
        assert!(!part.contains(&format!("{name}(")), "{part}");
        assert!(whole.contains(&format!("{name}(")), "{whole}");
    }
    assert!(part.lines().all(|l| whole.lines().any(|w| w == l)));
    assert_eq!(answers.len(), 2);
    assert_eq!(answers, whole_answers);
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

/// Z of a UAI file by enumerating its worlds: the product of its
/// functions' entries as written, each read with the scope's last
/// variable changing fastest, summed over every assignment that agrees
/// with `evidence`, a `.evid` file's text. The rows `rounded`, each a
/// function's index and a row's, of as many entries as the scope's last
/// variable has values, are divided by their sums.
fn enumerated_z(path: &str, evidence: Option<&str>, rounded: &[(usize, usize)]) -> f64 {
    fn int(tok: &mut std::str::SplitWhitespace) -> usize {
        tok.next().unwrap().parse().unwrap()
    }
    let text = std::fs::read_to_string(path).unwrap();
    let mut tok = text.split_whitespace();
    assert!(matches!(tok.next(), Some("MARKOV" | "BAYES")));
    let n = int(&mut tok);
    let cards: Vec<usize> = (0..n).map(|_| int(&mut tok)).collect();
    let scopes: Vec<Vec<usize>> = (0..int(&mut tok))
        .map(|_| {
            let k = int(&mut tok);
            (0..k).map(|_| int(&mut tok)).collect()
        })
        .collect();
    let mut tables: Vec<Vec<f64>> = scopes
        .iter()
        .map(|_| {
            let k = int(&mut tok);
            (0..k)
                .map(|_| tok.next().unwrap().parse().unwrap())
                .collect()
        })
        .collect();
    for &(f, r) in rounded {
        let k = cards[*scopes[f].last().unwrap()];
        let row = &mut tables[f][r * k..(r + 1) * k];
        let total: f64 = row.iter().sum();
        row.iter_mut().for_each(|p| *p /= total);
    }
    let observed: Vec<(usize, usize)> = evidence.map_or(Vec::new(), |e| {
        let mut tok = e.split_whitespace();
        (0..int(&mut tok))
            .map(|_| (int(&mut tok), int(&mut tok)))
            .collect()
    });
    let mut x = vec![0; n];
    let mut z = 0.0;
    for w in 0..cards.iter().product() {
        let mut rest = w;
        for v in (0..n).rev() {
            x[v] = rest % cards[v];
            rest /= cards[v];
        }
        if observed.iter().any(|&(v, value)| x[v] != value) {
            continue;
        }
        z += scopes
            .iter()
            .zip(&tables)
            .map(|(s, t)| t[s.iter().fold(0, |i, &v| i * cards[v] + x[v])])
            .product::<f64>();
    }
    z
}

#[test]
fn a_summed_out_variable_stays_while_a_scope_holds_it() {
    // tests/networks/restored_scope.uai, 18 binary variables and 8 random
    // tables, has a join stop after its moves and give its operands back on
    // the join's scope, which holds variables their factors do not read.
    // The evidence walk then dropped a summed-out variable that such a
    // scope still held, the next join's union brought it back, and the
    // check against counting its weights twice panicked. Z by enumerating
    // the worlds; variable elimination gives 642.190392938853 too.
    let path = "tests/networks/restored_scope.uai";
    let out = tddlog(&[
        path,
        "--no-queries",
        "--evidence-probability",
        "--precision",
        "13",
    ]);
    let (_, log10_z) = answers(&out, "Z");
    let got = log10_z.expect("a Z line");
    let want = enumerated_z(path, None, &[]).log10();
    assert!((got - want).abs() < 1e-12, "log10 Z = {got} against {want}");
}

#[test]
fn a_markov_file_of_rounded_tables_is_its_network() {
    // tests/networks/rounded.uai holds x0's table and x1's given x0, x1
    // first in its scope; x1's row at x0 = v_0, 0.160598 and 0.839403,
    // sums to 1.000001, which six decimals rounded up explain. The file is
    // read as the Bayesian network, the row divided by its sum: rounding
    // is not structure, so that no factor of x1's rows' sums over x0
    // remains, Z = 1 and P(x1 = v_0) = 0.2 * 0.160598 / 1.000001 + 0.8 *
    // 0.6.
    let out = tddlog(&[
        "tests/networks/rounded.uai",
        "--evidence-probability",
        "--precision",
        "15",
    ]);
    let (got, log10_z) = answers(&out, "Z");
    let p = 0.2 * 0.160598 / (0.160598 + 0.839403) + 0.8 * 0.6;
    let expected = [
        ("x0(v_0)", 0.2),
        ("x0(v_1)", 0.8),
        ("x1(v_0)", p),
        ("x1(v_1)", 1.0 - p),
    ];
    assert_close("rounded.uai", &got, &expected, 1e-12);
    let log10_z = log10_z.expect("a Z line");
    assert!(log10_z.abs() < 1e-12, "{log10_z}");
}

/// A UAI file's functions are read as written, whatever they sum to,
/// rounding aside: the PR task's `Z` is their product summed over the
/// worlds, a row that sums to one up to rounding taken as the distribution
/// it means.
#[test]
fn a_uai_network_is_the_product_of_its_functions_as_written() {
    // tests/networks/as_written.uai's functions are conditional tables
    // whose rows do not all sum to one: x0's, 0.333333 three times, to a
    // constant that stays, its one row's; x1's given x0 to 1.000001, which
    // two halves rounded up explain and which reads as one, 1 and
    // 1.000002, which rounding does not explain; x2's given x1 to 1 and
    // 0.8; and x3's given x1 and x2, which no question reads, to 1,
    // 1.000002, 1 and 0, a row of zeros. as_written_markov.uai is the same
    // file as MARKOV, whose rows over 1.000001 make it a Markov network
    // rather than a Bayesian one, its functions all as written. log10 Z of
    // each, alone and given x2 = v_0, is the enumeration's.
    let evid = "tests/networks/as_written.uai.evid";
    let evidence = std::fs::read_to_string(evid).unwrap();
    for (path, rounded) in [
        ("tests/networks/as_written.uai", &[(1, 0)][..]),
        ("tests/networks/as_written_markov.uai", &[][..]),
    ] {
        for observed in [None, Some(evidence.as_str())] {
            let mut args = vec![
                path,
                "--no-queries",
                "--evidence-probability",
                "--precision",
                "13",
            ];
            if observed.is_some() {
                args.extend(["--evidence", evid]);
            }
            let (_, log10_z) = answers(&tddlog(&args), "Z");
            let got = log10_z.unwrap_or_else(|| panic!("{args:?}: no Z line"));
            let want = enumerated_z(path, observed, rounded).log10();
            assert!(
                (got - want).abs() < 1e-12,
                "{args:?}: log10 Z = {got} against {want}"
            );
        }
    }
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
    // ProbLog's answers on the programs the networks read as
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
        "tests/networks/asia.net",
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
    // The car network of the examples given a cold morning's observations:
    // its marginals and P(e) by a sum over its 93,312 worlds, which
    // ProbLog's answers on examples/bn_car_evidence.pl agree with to their
    // eight digits; the queries move from their priors (an empty tank 0.1,
    // fouled plugs 0.2 in the cold, a faulty starter 0.03).
    let out = tddlog(&[
        "examples/car.net",
        "--evidence",
        "examples/car.inst",
        "--query",
        "fuel,plugs,starter",
        "--evidence-probability",
        "--precision",
        "12",
    ]);
    let car = [
        ("fuel(full)", 0.0872227771755762),
        ("fuel(low)", 0.62133250566198),
        ("fuel(empty)", 0.291444717162443),
        ("plugs(ok)", 0.533974993545689),
        ("plugs(fouled)", 0.466025006454311),
        ("starter(ok)", 0.901449655790024),
        ("starter(faulty)", 0.0985503442099751),
    ];
    let (got, log10_e) = answers(&out, "P(e)");
    assert_close("car.net", &got, &car, 1e-11);
    let log10_e = log10_e.expect("a P(e) line");
    assert!(
        (log10_e - 0.0226280992985857f64.log10()).abs() < 1e-11,
        "{log10_e}"
    );
    // chain.uai's leaves by hand (tests/networks/chain.pl): x0 uniform, x1's
    // rows divided by their sums, and x2's given x1 = v_1 by theirs, times
    // the factor 0.75 of their sum.
    let x1 = [
        (0.5 + 0.9 / 0.99 + 1e-5) / 3.0,
        0.75 * (0.5 + 0.09 / 0.99 + 0.99999) / 3.0,
    ];
    let z = x1[0] + x1[1];
    let chain = [
        ("x2(v_0)", x1[0] * 0.55 / z),
        ("x2(v_1)", (x1[0] * 0.34 + x1[1] / 3.0) / z),
        ("x2(v_2)", (x1[0] * 0.11 + x1[1] * 2.0 / 3.0) / z),
    ];
    // ProbLog's answers on the other programs of tests/networks/, one
    // query at a time.
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
            &chain[..],
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
fn a_row_that_sums_to_one_in_single_precision_is_a_distribution() {
    // single.uai's rows of x1 are two of diagnose_a's (the UAI classics),
    // written to eight decimals from single precision: they sum to
    // 1.00000004 and 1.00000005, more than the decimals explain and within
    // the 3 · 2^-24 of a row of two computed in single precision. Each row
    // is the distribution it means, divided by its sum, and no factor of
    // the sums over x0 remains: the program is the Bayesian network, of no
    // offset, and the marginals are the network's.
    let path = "tests/networks/single.uai";
    let out = tddlog(&[path, "--query", "leaves", "--precision", "15"]);
    let (a, b) = (0.99800204 + 0.001998, 0.99400605 + 0.005994);
    let p = 0.3 * 0.99800204 / a + 0.7 * 0.99400605 / b;
    let expected = [("x1(v_0)", p), ("x1(v_1)", 1.0 - p)];
    assert_close("single.uai", &answers(&out, "P(e)").0, &expected, 1e-13);
    let out = tddlog(&["--program", path]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success() && !text.contains("log10 Z"), "{text}");
    assert!(!text.contains("(ok)"), "{text}");
}

/// A factor's entry over its largest is a number, not a sum, and keeps its
/// gap from one however small: tests/networks/near_one.uai's factor over
/// x0 and x1 holds 1, 0.999999995, 0.5 and 1, so that Z = 3.499999995 and
/// P(x0 = v_0) = 1.999999995 / Z. The row of 0.999999995 of the factor's
/// observed variable was read as certain, within 1e-8 of one, and Z was
/// 3.5.
#[test]
fn a_factors_entry_close_to_one_keeps_its_number() {
    let path = "tests/networks/near_one.uai";
    let out = tddlog(&[path, "--evidence-probability", "--precision", "15"]);
    let (got, log10_z) = answers(&out, "Z");
    let z = 3.499999995;
    assert!((enumerated_z(path, None, &[]) - z).abs() < 1e-15);
    let expected = [
        ("x0(v_0)", 1.999999995 / z),
        ("x0(v_1)", 1.5 / z),
        ("x1(v_0)", 1.5 / z),
        ("x1(v_1)", 1.999999995 / z),
    ];
    assert_close("near_one.uai", &got, &expected, 1e-14);
    let log10_z = log10_z.expect("a Z line");
    assert!((log10_z - z.log10()).abs() < 1e-14, "{log10_z}");
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
        &["tests/networks/asia.net.gz"][..],
        &[
            "tests/networks/asia.net",
            "--query",
            "all",
            "--no-queries",
            "--evidence-probability",
        ][..],
        &["tests/networks/asia.net", "--program", "--stats"][..],
        &[
            "tests/networks/asia.net",
            "--evidence",
            "tests/regress/markov.uai",
        ][..],
        &["tests/networks/asia.net", "--query", ""][..],
    ] {
        let out = tddlog(args);
        assert_eq!(
            out.status.code(),
            Some(2),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    // A network the reader refuses, UAI evidence for another format, and
    // a UAI file's functions rescaled, which are read as written.
    for args in [
        &["tests/networks/sprinkler.bif", "--query", "Nothing"][..],
        &["tests/networks/chain.uai", "--rescale"][..],
        &[
            "tests/networks/asia.net",
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
