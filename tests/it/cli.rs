//! The command line: the answer lines, `--precision`, `--ground`,
//! `--evidence-probability`, the option errors, which exit with status 2
//! before the program is read, and the walk through `examples/alarm.pl` in
//! docs/design.md.
use std::process::{Command, Output};

fn tddlog(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tddlog"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> &str {
    std::str::from_utf8(&out.stdout).unwrap()
}

#[test]
fn one_tab_separated_line_per_query_in_query_order() {
    // ProbLog 2.3 prints burglary 0.35714286, earthquake 0.71428571 and
    // calls(mary) 0.4 (tests/it/examples.rs).
    let out = tddlog(&["examples/alarm.pl"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        stdout(&out),
        "burglary:\t0.357143\nearthquake:\t0.714286\ncalls(mary):\t0.400000\n"
    );
    let out = tddlog(&[
        "--precision",
        "2",
        "--vtree",
        "dynamic",
        "examples/alarm.pl",
    ]);
    assert_eq!(
        stdout(&out),
        "burglary:\t0.36\nearthquake:\t0.71\ncalls(mary):\t0.40\n"
    );
}

#[test]
fn an_answer_under_1e_4_is_written_in_scientific_notation() {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli_small_answers.pl");
    std::fs::write(
        &path,
        "0.0002::a.\n0.000012::b.\n0.0000000000000057406952::c.\nquery(a).\nquery(b).\nquery(c).\n",
    )
    .unwrap();
    let path = path.to_str().unwrap();
    let out = tddlog(&[path]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        stdout(&out),
        "a:\t0.000200\nb:\t1.200000e-5\nc:\t5.740695e-15\n"
    );
    // At 17 digits a fixed 5.7e-15 would show three significant digits.
    let out = tddlog(&["--precision", "17", path]);
    let lines: Vec<&str> = stdout(&out).lines().collect();
    assert_eq!(lines[0], "a:\t0.00020000000000000");
    for (line, want) in lines[1..].iter().zip([1.2e-5, 5.7406952e-15]) {
        let text = line.split('\t').nth(1).unwrap();
        let (mantissa, _) = text.split_once('e').expect(line);
        assert_eq!(mantissa.len(), "1.".len() + 17, "{line}");
        assert!(
            (text.parse::<f64>().unwrap() - want).abs() < 1e-12 * want,
            "{line}"
        );
    }
}

#[test]
fn every_strategy_the_usage_names_gives_the_same_lines() {
    // Not alarm.pl: its evidence fixes `alarm` (src/ground/condition.rs),
    // and then no atom is read through an indicator, which `families`
    // refuses.
    let plain = tddlog(&["examples/query_beside_evidence.pl"]);
    for strategy in [
        "auto",
        "td",
        "closed",
        "families",
        "families:0",
        "dynamic",
        "dynamic:minfill",
        "program",
        "program:fanin",
    ] {
        let out = tddlog(&["--vtree", strategy, "examples/query_beside_evidence.pl"]);
        assert!(
            out.status.success(),
            "{strategy}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(stdout(&out), stdout(&plain), "{strategy}");
    }
}

#[test]
fn stats_write_to_stderr_alone() {
    let plain = tddlog(&["examples/alarm.pl"]);
    let all = tddlog(&["--stats", "examples/alarm.pl"]);
    assert!(all.status.success());
    assert_eq!(stdout(&all), stdout(&plain));
    let stderr = String::from_utf8(all.stderr).unwrap();
    // Reading and parsing are timed apart from grounding.
    let size = std::fs::metadata("examples/alarm.pl").unwrap().len();
    assert!(
        stderr
            .lines()
            .any(|l| l.starts_with(&format!("read: {size} bytes, file ")) && l.ends_with('s')),
        "{stderr}"
    );
    assert!(
        stderr.lines().any(|l| l.starts_with("ground: ")),
        "{stderr}"
    );
    assert!(stderr.lines().any(|l| l.starts_with("trial: ")), "{stderr}");
    // The indicators' rewrite has seconds of its own, apart from what auto
    // discards.
    assert!(
        stderr
            .lines()
            .any(|l| l.starts_with("indicators: ") && l.ends_with('s')),
        "{stderr}"
    );
}

#[test]
fn ground_prints_the_ground_program_a_strategy_compiles() {
    let out = tddlog(&["--ground", "examples/dice_murder.pl"]);
    assert!(out.status.success());
    assert!(
        stdout(&out).lines().any(|l| l.starts_with("query(")),
        "{}",
        stdout(&out)
    );
    assert!(!stdout(&out).contains("'$v'"));
    // query_beside_evidence reads atoms another component reads: each
    // reader reads a copy, `'$v'(atom, reader)`, a fact of probability 0.5,
    // and the link `'$d'(atom, reader)` that ties it to the atom is
    // observed true. dice_murder's evidence fixes the atoms it read
    // (src/ground/condition.rs).
    let out = tddlog(&[
        "--ground",
        "--vtree",
        "families",
        "examples/query_beside_evidence.pl",
    ]);
    assert!(out.status.success());
    let printed = stdout(&out);
    assert!(
        printed.lines().any(|l| l.starts_with("0.5::'$v'(")),
        "{printed}"
    );
    assert!(
        printed
            .lines()
            .any(|l| l.starts_with("evidence('$d'(") && l.ends_with(",true).")),
        "{printed}"
    );
    // A strategy compiles linear recursion written without recursion
    // (src/ground/eliminate.rs), labels `'$e'(n)` and all; auto chooses its
    // trials from the program as grounded, which `--ground` prints.
    let out = tddlog(&["--ground", "--vtree", "td", "examples/cycle_graph.pl"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("'$e'("), "{}", stdout(&out));
    let out = tddlog(&["--ground", "examples/cycle_graph.pl"]);
    assert!(out.status.success());
    assert!(!stdout(&out).contains("'$e'("), "{}", stdout(&out));
    // With `--evidence-probability` the program `P(e)` is computed on:
    // detach.pl's observations that no query depends on are kept, and
    // with `--no-queries` the queries are left out.
    let observations = |args: &[&str]| {
        let out = tddlog(args);
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let printed = stdout(&out);
        (
            printed
                .lines()
                .filter(|l| l.starts_with("evidence("))
                .count(),
            printed.lines().filter(|l| l.starts_with("query(")).count(),
        )
    };
    let (dropped, queries) = observations(&["--ground", "tests/regress/detach.pl"]);
    let (kept, asked) = observations(&[
        "--ground",
        "--evidence-probability",
        "tests/regress/detach.pl",
    ]);
    assert!(
        kept > dropped && asked == queries && queries > 0,
        "{kept} against {dropped}"
    );
    assert_eq!(
        observations(&[
            "--ground",
            "--evidence-probability",
            "--no-queries",
            "tests/regress/detach.pl"
        ])
        .1,
        0
    );
}

#[test]
fn options_that_ask_for_nothing_fail() {
    // Without the queries P(e) is all there is to answer.
    for args in [
        &["--no-queries", "examples/alarm.pl"][..],
        &["--ground", "--no-queries", "examples/alarm.pl"],
    ] {
        let out = tddlog(args);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{args:?}: {stderr}");
        assert!(stderr.contains("nothing to answer"), "{args:?}: {stderr}");
        assert!(out.stdout.is_empty(), "{args:?}");
    }
}

#[test]
fn evidence_probability_prints_one_line_after_the_answers() {
    // P(e) by ProbLog: 0.12 for condition_bn, 0.3075 for detach.pl, whose
    // detached parts it counts; smokers has no evidence
    // (tests/it/evidence_probability.rs). The query lines are the ones
    // printed without the option.
    for (file, line) in [
        (
            "tests/regress/condition_bn.pl",
            "P(e):\t1.200000e-1\tlog10 -0.920819",
        ),
        (
            "tests/regress/detach.pl",
            "P(e):\t3.075000e-1\tlog10 -0.512155",
        ),
        ("examples/smokers.pl", "P(e):\t1.000000e0\tlog10 0.000000"),
    ] {
        let plain = tddlog(&[file]);
        let out = tddlog(&["--evidence-probability", file]);
        assert!(
            out.status.success(),
            "{file}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            stdout(&out),
            format!("{}{line}\n", stdout(&plain)),
            "{file}"
        );
    }
    let out = tddlog(&[
        "--evidence-probability",
        "--precision",
        "3",
        "--vtree",
        "dynamic",
        "tests/regress/condition_bn.pl",
    ]);
    assert!(
        stdout(&out).ends_with("\nP(e):\t1.200e-1\tlog10 -0.921\n"),
        "{}",
        stdout(&out)
    );
    let out = tddlog(&[
        "--evidence-probability",
        "tests/regress/condition_inconsistent.pl",
    ]);
    assert_eq!(
        stdout(&out),
        "evidence is inconsistent: probability zero\nP(e):\t0.000000e0\tlog10 -inf\n"
    );
    // 2^-1100, under the least float, is read off its logarithm.
    let path =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("evidence_probability_1100.pl");
    let facts: String = (0..1100)
        .map(|i| format!("0.5::a{i}.\nevidence(a{i}).\n"))
        .collect();
    std::fs::write(&path, facts + "query(a0).\n").unwrap();
    let out = tddlog(&["--evidence-probability", path.to_str().unwrap()]);
    assert_eq!(
        stdout(&out),
        "a0:\t1.000000\nP(e):\t7.362152e-332\tlog10 -331.132995\n"
    );
}

#[test]
fn no_queries_prints_the_evidence_probability_alone() {
    // The P(e) line the queries' run prints, and nothing before it
    // (tests/it/evidence_probability.rs checks the program ground for its
    // evidence alone).
    for file in [
        "tests/regress/condition_bn.pl",
        "examples/alarm.pl",
        "examples/smokers.pl",
        "tests/regress/condition_inconsistent.pl",
    ] {
        let with = tddlog(&["--evidence-probability", file]);
        let line = stdout(&with).lines().last().unwrap().to_string();
        let out = tddlog(&["--evidence-probability", "--no-queries", file]);
        assert!(
            out.status.success(),
            "{file}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let inconsistent = if file.ends_with("inconsistent.pl") {
            "evidence is inconsistent: probability zero\n"
        } else {
            ""
        };
        assert_eq!(stdout(&out), format!("{inconsistent}{line}\n"), "{file}");
    }
    let out = tddlog(&[
        "--no-queries",
        "--evidence-probability",
        "examples/alarm.pl",
    ]);
    assert_eq!(stdout(&out), "P(e):\t1.960000e-1\tlog10 -0.707744\n");
}

#[test]
fn help_prints_the_options_to_stdout() {
    let out = tddlog(&["--help"]);
    assert!(out.status.success());
    assert!(stdout(&out).starts_with("usage: tddlog"));
}

#[test]
fn version_prints_the_crate_version() {
    let out = tddlog(&["--version"]);
    assert!(out.status.success());
    assert_eq!(
        stdout(&out),
        format!("tddlog {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn bad_and_inert_options_exit_with_status_two() {
    let cases: &[(&[&str], &str)] = &[
        (&[], "usage"),
        (&["--vtree"], "--vtree needs a value"),
        (&["--precision"], "--precision needs a value"),
        (&["--precision", "x", "examples/alarm.pl"], "bad precision"),
        (
            &["--bogus", "examples/alarm.pl"],
            "unknown option `--bogus`",
        ),
        (
            &["examples/alarm.pl", "examples/dice.pl"],
            "only one program file",
        ),
        (
            &["--vtree", "closed", "--vtree", "td", "examples/alarm.pl"],
            "--vtree is given twice",
        ),
        (
            &["--precision", "3", "--precision", "4", "examples/alarm.pl"],
            "--precision is given twice",
        ),
        (
            &["--ground", "--stats", "examples/alarm.pl"],
            "so --stats would do nothing",
        ),
        (
            &["--vtree", "nope", "examples/alarm.pl"],
            "unknown vtree strategy `nope`",
        ),
        (
            &["--ground", "--precision", "3", "examples/alarm.pl"],
            "so --precision would do nothing",
        ),
        (
            &[
                "--evidence-probability",
                "--evidence-probability",
                "examples/alarm.pl",
            ],
            "--evidence-probability is given twice",
        ),
        (
            &["--ground", "--ground", "examples/alarm.pl"],
            "--ground is given twice",
        ),
        (
            &["--stats", "--stats", "examples/alarm.pl"],
            "--stats is given twice",
        ),
        (
            &[
                "--no-queries",
                "--no-queries",
                "--evidence-probability",
                "examples/alarm.pl",
            ],
            "--no-queries is given twice",
        ),
        (&["no/such/file.pl"], "cannot read no/such/file.pl"),
    ];
    for (args, message) in cases {
        let out = tddlog(args);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{args:?}: {stderr}");
        assert!(stderr.contains(message), "{args:?}: {stderr}");
        assert!(out.stdout.is_empty(), "{args:?}");
    }
}

#[test]
fn the_design_pages_walk_through_alarm_matches_the_binary() {
    // docs/design.md, "How a program is answered", prints the ground
    // program of examples/alarm.pl and says which trial answers it.
    let page = std::fs::read_to_string("docs/design.md").unwrap();
    let command = "$ tddlog --ground examples/alarm.pl";
    let start = page.find(command).expect("the page shows the command") + command.len();
    let end = start + page[start..].find("```").expect("the block closes");
    let shown: String = page[start..end]
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| format!("{line}\n"))
        .collect();
    let out = tddlog(&["--ground", "examples/alarm.pl"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), shown);

    let out = tddlog(&["--stats", "--evidence-probability", "examples/alarm.pl"]);
    let stats = String::from_utf8_lossy(&out.stderr);
    assert!(stats.contains("vtree td (from auto: 1 attempts"), "{stats}");
    assert!(stdout(&out).starts_with("burglary:\t0.357143\n"));
    assert!(
        stdout(&out).contains("P(e):\t1.960000e-1\t"),
        "{}",
        stdout(&out)
    );
}
