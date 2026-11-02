//! The trace of a run (`tddlog::trace`, `tddlog --trace`): JSON whose last
//! stage holds the answers the run prints, which are ProbLog's; whose
//! compile stage counts the components, updates and fixpoint rounds the
//! compile counts; whose lists are bounded; and asking for which changes
//! neither an answer nor the work of the run.

use std::process::Command;

use tddlog::network::ReadOptions;
use tddlog::tp::trace::{ATOMS, COMPONENTS, UPDATES};
use tddlog::trace::{LINES, Trace};
use tddlog::{Format, Options, Progress, Strategy};

/// A JSON value, as far as the tests read one.
#[derive(Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn parse(text: &str) -> Json {
        let mut chars = text.trim().chars().peekable();
        let value = Self::value(&mut chars);
        assert!(chars.next().is_none(), "text after the JSON value");
        value
    }

    fn value(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Json {
        let skip = |chars: &mut std::iter::Peekable<std::str::Chars<'_>>| {
            while chars.peek().is_some_and(|c| c.is_whitespace()) {
                chars.next();
            }
        };
        skip(chars);
        let json = match chars.next().expect("a JSON value") {
            'n' => Self::word(chars, "ull", Json::Null),
            't' => Self::word(chars, "rue", Json::Bool(true)),
            'f' => Self::word(chars, "alse", Json::Bool(false)),
            '"' => Json::Text(Self::string(chars)),
            '[' => {
                let mut items = Vec::new();
                skip(chars);
                if chars.peek() == Some(&']') {
                    chars.next();
                } else {
                    loop {
                        items.push(Self::value(chars));
                        skip(chars);
                        match chars.next() {
                            Some(',') => {}
                            Some(']') => break,
                            other => panic!("{other:?} in an array"),
                        }
                    }
                }
                Json::Array(items)
            }
            '{' => {
                let mut fields = Vec::new();
                skip(chars);
                if chars.peek() == Some(&'}') {
                    chars.next();
                } else {
                    loop {
                        skip(chars);
                        assert_eq!(chars.next(), Some('"'), "a key");
                        let key = Self::string(chars);
                        skip(chars);
                        assert_eq!(chars.next(), Some(':'));
                        fields.push((key, Self::value(chars)));
                        skip(chars);
                        match chars.next() {
                            Some(',') => {}
                            Some('}') => break,
                            other => panic!("{other:?} in an object"),
                        }
                    }
                }
                Json::Object(fields)
            }
            c => {
                let mut number = c.to_string();
                while chars
                    .peek()
                    .is_some_and(|c| c.is_ascii_digit() || "+-.eE".contains(*c))
                {
                    number.push(chars.next().unwrap());
                }
                Json::Number(number.parse().unwrap_or_else(|_| panic!("{number}")))
            }
        };
        skip(chars);
        json
    }

    fn word(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, rest: &str, json: Json) -> Json {
        for c in rest.chars() {
            assert_eq!(chars.next(), Some(c));
        }
        json
    }

    fn string(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
        let mut out = String::new();
        loop {
            match chars.next().expect("an unterminated string") {
                '"' => return out,
                '\\' => match chars.next().unwrap() {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let hex: String = chars.by_ref().take(4).collect();
                        out.push(char::from_u32(u32::from_str_radix(&hex, 16).unwrap()).unwrap());
                    }
                    c => out.push(c),
                },
                c => {
                    assert!(c as u32 >= 0x20, "a control character in a string");
                    out.push(c);
                }
            }
        }
    }

    fn get(&self, key: &str) -> &Json {
        let Json::Object(fields) = self else {
            panic!("{key} of a value that is not an object: {self:?}")
        };
        fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("no {key} in {self:?}"))
    }

    fn number(&self) -> f64 {
        match self {
            Json::Number(x) => *x,
            other => panic!("{other:?} is not a number"),
        }
    }

    fn count(&self) -> usize {
        self.number() as usize
    }

    fn text(&self) -> &str {
        match self {
            Json::Text(s) => s,
            other => panic!("{other:?} is not a string"),
        }
    }

    fn items(&self) -> &[Json] {
        match self {
            Json::Array(items) => items,
            other => panic!("{other:?} is not an array"),
        }
    }
}

/// The stages of a trace named `name`.
fn stages<'a>(trace: &'a Json, name: &str) -> Vec<&'a Json> {
    trace
        .get("stages")
        .items()
        .iter()
        .filter(|s| s.get("stage").text() == name)
        .collect()
}

/// What a run reported of each part's compile: its work units, atom
/// updates and answers, and whether it recorded a trace.
#[derive(Debug, PartialEq)]
struct Part {
    work_units: u64,
    atom_updates: usize,
    query_work: u64,
    answers: Vec<(String, u64)>,
    traced: bool,
}

/// What [`solve`] returns: the parts' runs, the answers, `P(e)` and the
/// trace's JSON.
struct Solved {
    parts: Vec<Part>,
    answers: Vec<(String, f64)>,
    evidence: Option<f64>,
    trace: Option<Json>,
}

/// Solve `data`, read in `format` with `read`, under `strategy`, with a
/// trace if `traced`.
fn solve(
    data: &[u8],
    format: Format,
    read: &ReadOptions,
    (strategy, evidence_probability): (&str, bool),
    traced: bool,
) -> Solved {
    let input = tddlog::read(data, format, read).unwrap();
    let options = Options {
        strategy: strategy.parse::<Strategy>().unwrap(),
        evidence_probability,
        trace: traced,
        ..Options::default()
    };
    let mut trace = traced.then(|| Trace::new(format, data, &input, &options).unwrap());
    let mut parts = Vec::new();
    let solved = tddlog::solve(&input.program, &options, &mut |progress| {
        if let Some(t) = trace.as_mut() {
            t.observe(&progress);
        }
        if let Progress::Answered(run) = progress {
            parts.push(Part {
                work_units: run.stats.work_units,
                atom_updates: run.stats.atom_updates,
                query_work: run.query_sizes.work_units,
                answers: run
                    .results
                    .iter()
                    .flatten()
                    .map(|r| (r.to_string(), r.probability.to_bits()))
                    .collect(),
                traced: run.trace.is_some(),
            });
        }
    })
    .unwrap();
    let answers = solved
        .answers
        .iter()
        .flatten()
        .map(|r| (r.to_string(), r.probability))
        .collect();
    let evidence = input.evidence_probability(&solved).map(|(p, _)| p);
    let trace = trace.map(|mut t| {
        t.answered(&solved, &input);
        Json::parse(&t.to_string())
    });
    Solved {
        parts,
        answers,
        evidence,
        trace,
    }
}

fn file(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap()
}

/// The answers of a trace's last stage.
fn traced_answers(trace: &Json) -> Vec<(String, f64)> {
    let last = trace.get("stages").items().last().unwrap();
    assert_eq!(last.get("stage").text(), "answers");
    last.get("answers")
        .items()
        .iter()
        .map(|a| {
            (
                a.get("query").text().to_string(),
                a.get("probability").number(),
            )
        })
        .collect()
}

#[test]
fn the_trace_ends_with_the_answers_and_p_e_of_the_run() {
    // ProbLog 2.3: burglary 0.35714286, earthquake 0.71428571, calls(mary)
    // 0.4 (tests/it/examples.rs); P(e) of condition_bn 0.12
    // (tests/it/evidence_probability.rs).
    for (path, strategy, want) in [
        (
            "examples/alarm.pl",
            "auto",
            &[
                ("burglary", 0.35714286),
                ("earthquake", 0.71428571),
                ("calls(mary)", 0.4),
            ][..],
        ),
        ("examples/alarm.pl", "dynamic", &[][..]),
        ("tests/regress/condition_bn.pl", "auto", &[][..]),
    ] {
        let Solved {
            answers,
            evidence,
            trace,
            ..
        } = solve(
            &file(path),
            Format::Problog,
            &ReadOptions::default(),
            (strategy, true),
            true,
        );
        let trace = trace.unwrap();
        assert_eq!(traced_answers(&trace), answers, "{path} {strategy}");
        for ((q, p), (w, wp)) in answers.iter().zip(want) {
            assert_eq!(q, w);
            assert!((p - wp).abs() < 1e-8, "{path}: {q} {p} against {wp}");
        }
        let last = trace.get("stages").items().last().unwrap();
        let p = last.get("evidence").get("probability").number();
        assert_eq!(Some(p), evidence, "{path}");
        if path.ends_with("condition_bn.pl") {
            assert!((p - 0.12).abs() < 1e-12, "{path}: P(e) {p}");
        }
        // Each part's answers are among the run's.
        for part in stages(&trace, "queries") {
            for a in part.get("answers").items() {
                let q = a.get("query").text();
                let p = a.get("probability").number();
                assert!(answers.iter().any(|(aq, ap)| aq == q && *ap == p), "{q}");
            }
        }
        assert_eq!(trace.get("trace").count(), tddlog::trace::VERSION as usize);
    }
}

#[test]
fn asking_for_a_trace_changes_no_answer_and_no_work() {
    let hugin: Format = "hugin".parse().unwrap();
    let inst = file("tests/networks/asia_leaves.inst");
    let networks = ReadOptions {
        evidence: Some(hugin.evidence(&inst)),
        ..ReadOptions::default()
    };
    let cases: [(&str, Format, &ReadOptions, &str); 7] = [
        (
            "examples/alarm.pl",
            Format::Problog,
            &ReadOptions::default(),
            "auto",
        ),
        (
            "examples/cycle_graph.pl",
            Format::Problog,
            &ReadOptions::default(),
            "auto",
        ),
        (
            "examples/win.pl",
            Format::Problog,
            &ReadOptions::default(),
            "auto",
        ),
        (
            "examples/negcycle_sound.pl",
            Format::Problog,
            &ReadOptions::default(),
            "closed",
        ),
        ("examples/asia.net", hugin, &networks, "auto"),
        (
            "tests/dice/walk.dice",
            Format::Dice,
            &ReadOptions::default(),
            "auto",
        ),
        (
            "examples/alea_dice.pl",
            Format::Problog,
            &ReadOptions::default(),
            "auto",
        ),
    ];
    for (path, format, read, strategy) in cases {
        let data = file(path);
        let plain = solve(&data, format, read, (strategy, true), false);
        let traced = solve(&data, format, read, (strategy, true), true);
        assert!(plain.trace.is_none());
        assert!(
            plain.parts.iter().all(|p| !p.traced),
            "{path}: a record nobody asked for"
        );
        assert!(traced.parts.iter().all(|p| p.traced), "{path}: no record");
        let strip = |parts: Vec<Part>| -> Vec<Part> {
            parts
                .into_iter()
                .map(|p| Part { traced: false, ..p })
                .collect()
        };
        assert_eq!(plain.answers, traced.answers, "{path}");
        assert_eq!(plain.evidence, traced.evidence, "{path}");
        let trace = traced.trace.unwrap();
        assert_eq!(traced_answers(&trace), plain.answers, "{path}");
        assert_eq!(strip(plain.parts), strip(traced.parts), "{path} {strategy}");
        let read = &stages(&trace, "read")[0];
        assert_eq!(read.get("format").text(), format.to_string());
        assert_eq!(read.get("bytes").count(), data.len());
        // A network and a Dice program are read as a program of their own.
        assert_eq!(
            matches!(read.get("program"), Json::Null),
            format == Format::Problog,
            "{path}"
        );
    }
}

/// Whether a trace's compile stages agree with the totals they give, and
/// keep to their bounds.
fn check_compiles(path: &str, trace: &Json) {
    for compile in stages(trace, "compile") {
        let totals = compile.get("totals");
        let rest = compile.get("rest");
        let components = compile.get("components").items();
        assert!(components.len() <= COMPONENTS, "{path}");
        assert_eq!(
            components.len() + rest.get("components").count(),
            totals.get("components").count(),
            "{path}: components"
        );
        let cyclic = components
            .iter()
            .filter(|c| c.get("cyclic") == &Json::Bool(true))
            .count();
        assert_eq!(
            cyclic + rest.get("cyclic").count(),
            totals.get("cyclic").count(),
            "{path}: cyclic components"
        );
        let updates: usize = components.iter().map(|c| c.get("updates").count()).sum();
        assert_eq!(
            updates + rest.get("updates").count(),
            totals.get("updates").count(),
            "{path}: updates"
        );
        let rounds: usize = components
            .iter()
            .filter(|c| c.get("negation") == &Json::Bool(false))
            .map(|c| c.get("rounds").count())
            .sum();
        if rest.get("components").count() == 0 {
            assert_eq!(rounds, totals.get("iterations").count(), "{path}: rounds");
        }
        let largest = components
            .iter()
            .map(|c| c.get("largest").count())
            .chain([rest.get("largest").count()])
            .max()
            .unwrap();
        assert!(largest <= totals.get("largest").count(), "{path}: largest");
        let work: f64 = components
            .iter()
            .map(|c| c.get("workUnits").number())
            .sum::<f64>()
            + rest.get("workUnits").number();
        assert!(work <= totals.get("workUnits").number(), "{path}: work");
        for c in components {
            assert!(c.get("atoms").items().len() <= ATOMS.min(c.get("size").count()));
            let sizes = c.get("sizes").items();
            assert!(sizes.len() <= UPDATES.min(c.get("updates").count()));
            for s in sizes {
                assert_eq!(s.items().len(), 3, "{path}: [atom, pairs, round]");
                assert!(s.items()[2].count() <= c.get("rounds").count());
            }
        }
    }
}

/// `p`, `q` and `r` read each other, and `p` reads two of them in one body,
/// so that no rewrite takes their recursion away. By the worlds of `a`, `b`
/// and `c`: with `a`, `p` holds, and `q` and `r` hold with `b` or `c`;
/// without it all three hold with `b`. So `P(p) = 0.5 + 0.5 * 0.6 = 0.8` and
/// `P(q) = P(r) = 0.5 * (1 - 0.4 * 0.7) + 0.5 * 0.6 = 0.66`.
const NONLINEAR: &str = "0.5::a.\n0.6::b.\n0.3::c.\n\
    p :- a.\np :- q, r.\nq :- b.\nq :- r.\nr :- p, c.\nr :- q.\n\
    query(p).\nquery(q).\nquery(r).\n";

#[test]
fn the_compile_stage_counts_what_the_compile_counts() {
    // negcycle_sound.pl by the well-founded model of each world: with `f`,
    // `a` holds and `b` fails; without it, `b` holds (tests/it/examples.rs).
    let cases: [(&[u8], &str, &[f64]); 5] = [
        (NONLINEAR.as_bytes(), "closed", &[0.8, 0.66, 0.66]),
        (NONLINEAR.as_bytes(), "dynamic", &[0.8, 0.66, 0.66]),
        (&file("examples/negcycle_sound.pl"), "closed", &[0.3, 0.7]),
        (&file("examples/cycle_graph.pl"), "closed", &[]),
        (&file("examples/alarm.pl"), "dynamic", &[]),
    ];
    for (i, (data, strategy, want)) in cases.into_iter().enumerate() {
        let Solved { answers, trace, .. } = solve(
            data,
            Format::Problog,
            &ReadOptions::default(),
            (strategy, false),
            true,
        );
        let trace = trace.unwrap();
        let what = format!("case {i} {strategy}");
        check_compiles(&what, &trace);
        for ((q, p), w) in answers.iter().zip(want) {
            assert!((p - w).abs() < 1e-12, "{what}: {q} {p} against {w}");
        }
        let components: Vec<&Json> = stages(&trace, "compile")
            .iter()
            .flat_map(|c| c.get("components").items())
            .collect();
        match i {
            // The fixpoint: one component, iterated, each update named.
            0 | 1 => {
                let cyclic: Vec<_> = components
                    .iter()
                    .filter(|c| c.get("cyclic") == &Json::Bool(true))
                    .collect();
                assert_eq!(cyclic.len(), 1, "{what}");
                let c = cyclic[0];
                assert_eq!(c.get("size").count(), 3, "{what}");
                assert_eq!(c.get("negation"), &Json::Bool(false), "{what}");
                assert!(c.get("rounds").count() >= 2, "{what}: {c:?}");
            }
            // The well-founded model of a component read through negation.
            2 => assert!(
                components
                    .iter()
                    .any(|c| c.get("negation") == &Json::Bool(true)),
                "{what}"
            ),
            _ => {}
        }
    }
}

#[test]
fn a_large_programs_trace_keeps_to_its_bounds() {
    // A chain of 300 atoms, each with two rules so that no pass reads it
    // into the next: 300 components, of which the trace names the first
    // COMPONENTS, and a ground program of more than LINES lines.
    let mut source = String::new();
    for i in 1..=300 {
        source += &format!("0.5::f({i}).\n0.5::g({i}).\n");
        source += &format!("a({i}) :- g({i}).\n");
        if i > 1 {
            source += &format!("a({i}) :- a({}), f({i}).\n", i - 1);
        }
    }
    source += "query(a(300)).\n";
    let Solved { answers, trace, .. } = solve(
        source.as_bytes(),
        Format::Problog,
        &ReadOptions::default(),
        ("closed", false),
        true,
    );
    let trace = trace.unwrap();
    check_compiles("chain", &trace);
    // P(a(n)) = 1/2 + 1/2 * 1/2 * P(a(n - 1)), P(a(1)) = 1/2, which tends
    // to 2/3.
    assert!((answers[0].1 - 2.0 / 3.0).abs() < 1e-12, "{answers:?}");
    let compile = stages(&trace, "compile")[0];
    assert_eq!(compile.get("components").items().len(), COMPONENTS);
    assert!(compile.get("rest").get("components").count() > 0);
    let ground = stages(&trace, "ground")[0];
    let program = ground.get("program");
    assert_eq!(program.get("lines"), &Json::Null);
    assert_eq!(program.get("shown").count(), LINES);
    let read = stages(&trace, "read")[0].get("source");
    assert_eq!(read.get("lines").count(), source.lines().count());
    assert_eq!(read.get("shown").count(), LINES);
}

fn tddlog(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tddlog"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn the_command_line_writes_the_trace_of_what_it_prints() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("trace_alarm.json");
    let path = path.to_str().unwrap();
    let plain = tddlog(&["--evidence-probability", "examples/alarm.pl"]);
    let out = tddlog(&[
        "--trace",
        path,
        "--evidence-probability",
        "examples/alarm.pl",
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, plain.stdout);
    let printed = String::from_utf8(out.stdout).unwrap();
    let trace = Json::parse(&std::fs::read_to_string(path).unwrap());
    let answers = traced_answers(&trace);
    let mut lines = printed.lines();
    for (q, p) in &answers {
        assert_eq!(lines.next().unwrap(), format!("{q}:\t{p:.6}"));
    }
    let last = trace.get("stages").items().last().unwrap();
    let p = last.get("evidence").get("probability").number();
    assert!(
        lines
            .next()
            .unwrap()
            .starts_with(&format!("P(e):\t{p:.6e}"))
    );
    let names: Vec<&str> = trace
        .get("stages")
        .items()
        .iter()
        .map(|s| s.get("stage").text())
        .collect();
    assert_eq!(names[..3], ["read", "parse", "ground"]);
    for stage in ["rewrite", "layout", "compile", "evidence", "queries"] {
        assert!(names.contains(&stage), "{stage} in {names:?}");
    }
    assert_eq!(trace.get("strategy").text(), "auto");
    assert_eq!(trace.get("evidenceProbability"), &Json::Bool(true));

    // A run that fails ends its trace with the error.
    let broken = dir.join("trace_broken.pl");
    std::fs::write(&broken, "0.5::a.\nb :- a\nquery(b).\n").unwrap();
    let out = tddlog(&["--trace", path, broken.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    let trace = Json::parse(&std::fs::read_to_string(path).unwrap());
    let last = trace.get("stages").items().last().unwrap();
    assert_eq!(last.get("stage").text(), "error");
    assert!(!last.get("message").text().is_empty());
    // One that cannot read its program leaves no trace.
    let out = tddlog(&["--trace", path, "tests/no_such_file.pl"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(!std::path::Path::new(path).exists());
}

#[test]
fn a_trace_it_cannot_serve_or_write_is_refused() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("trace_refused.json");
    let path = path.to_str().unwrap();
    for (args, message) in [
        (
            &["examples/alarm.pl", "--trace"][..],
            "--trace needs a file",
        ),
        (
            &["--trace", path, "--trace", path, "examples/alarm.pl"][..],
            "--trace",
        ),
        (
            &["--ground", "--trace", path, "examples/alarm.pl"][..],
            "so --trace would do nothing",
        ),
        (
            &["--program", "--trace", path, "examples/asia.net"][..],
            "--trace",
        ),
        (
            &["--cnf", "--trace", path, "examples/alarm.pl"][..],
            "--trace",
        ),
        (
            &[
                "--trace",
                "tests/no_such_dir/trace.json",
                "examples/alarm.pl",
            ][..],
            "cannot write tests/no_such_dir/trace.json",
        ),
    ] {
        let out = tddlog(args);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{args:?}: {stderr}");
        assert!(stderr.contains(message), "{args:?}: {stderr}");
        assert!(out.stdout.is_empty(), "{args:?}");
        assert!(!std::path::Path::new(path).exists(), "{args:?}");
    }
    // The library refuses a trace of a run that does not record its
    // compile.
    let input = tddlog::read(
        b"0.5::a.\nquery(a).\n",
        Format::Problog,
        &ReadOptions::default(),
    )
    .unwrap();
    let refused = Trace::new(Format::Problog, b"", &input, &Options::default());
    assert!(matches!(refused, Err(tddlog::Error::Options(_))));
}
