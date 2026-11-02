//! The weighted CNF of a program (`tddlog::weighted_cnf`, `tddlog --cnf`),
//! read back from its text and counted by enumeration: the weighted count
//! is `P(e)`, and with a query's literal added as a unit clause `P(q, e)`,
//! so that each ratio is tddlog's answer, and with every observation kept,
//! the count is tddlog's `P(e)`. Where no observation is a clause, the
//! `c count` line is the count.
use std::process::Command;

use tddlog::{Error, Options, Strategy};

/// A weighted CNF as its text states it: the clauses, each variable's
/// weights (false, true), how many variables are weighed (the facts), the
/// queries' literals and the `c count` line's count.
struct Dimacs {
    clauses: Vec<Vec<i32>>,
    weights: Vec<[f64; 2]>,
    facts: usize,
    queries: Vec<(i32, String)>,
    count: Option<f64>,
}

fn read(text: &str) -> Dimacs {
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("c t wmc"), "{text}");
    let header = lines.next().unwrap();
    let numbers: Vec<usize> = header
        .strip_prefix("p cnf ")
        .unwrap_or_else(|| panic!("{header}"))
        .split(' ')
        .map(|n| n.parse().unwrap())
        .collect();
    let (variables, declared) = (numbers[0], numbers[1]);
    let mut cnf = Dimacs {
        clauses: Vec::new(),
        weights: vec![[1.0, 1.0]; variables + 1],
        facts: 0,
        queries: Vec::new(),
        count: None,
    };
    for line in lines {
        if let Some(rest) = line.strip_prefix("c p weight ") {
            let words: Vec<&str> = rest.split(' ').collect();
            assert_eq!(words.len(), 3, "{line}");
            assert_eq!(words[2], "0", "{line}");
            let l: i32 = words[0].parse().unwrap();
            let w: f64 = words[1].parse().unwrap();
            cnf.weights[l.unsigned_abs() as usize][usize::from(l > 0)] = w;
            cnf.facts += usize::from(l > 0);
        } else if let Some(rest) = line.strip_prefix("c query ") {
            let (l, text) = rest.split_once(' ').unwrap();
            cnf.queries.push((l.parse().unwrap(), text.to_string()));
        } else if let Some(rest) = line.strip_prefix("c count ") {
            assert!(cnf.count.is_none(), "{text}");
            cnf.count = Some(rest.parse().unwrap());
        } else {
            assert!(!line.starts_with('c'), "{line}");
            let mut clause: Vec<i32> = line.split(' ').map(|l| l.parse().unwrap()).collect();
            assert_eq!(clause.pop(), Some(0), "{line}");
            assert!(
                clause
                    .iter()
                    .all(|l| l.unsigned_abs() as usize <= variables)
            );
            cnf.clauses.push(clause);
        }
    }
    assert_eq!(cnf.clauses.len(), declared, "{text}");
    cnf
}

/// The most facts a CNF counted by enumeration may weigh: the walk takes
/// a step per world of the facts, every other variable defined by them.
const MOST_FACTS: usize = 16;

/// The weighted count of `cnf` with `unit` as a clause too: a walk over
/// the assignments of the variables in their order, each clause checked
/// once its last variable has a value.
fn count(cnf: &Dimacs, unit: Option<i32>) -> f64 {
    assert!(cnf.facts <= MOST_FACTS, "{} facts", cnf.facts);
    let n = cnf.weights.len() - 1;
    let mut last: Vec<Vec<&[i32]>> = vec![Vec::new(); n + 1];
    let unit = unit.map(|l| [l]);
    for clause in cnf
        .clauses
        .iter()
        .map(Vec::as_slice)
        .chain(unit.as_ref().map(|u| &u[..]))
    {
        let at = clause
            .iter()
            .map(|l| l.unsigned_abs() as usize)
            .max()
            .unwrap_or(0);
        last[at].push(clause);
    }
    if !last[0].is_empty() {
        return 0.0;
    }
    fn walk(v: usize, value: &mut [bool], cnf: &Dimacs, last: &[Vec<&[i32]>]) -> f64 {
        if v == value.len() {
            return 1.0;
        }
        let mut sum = 0.0;
        for b in [false, true] {
            value[v] = b;
            let holds = last[v].iter().all(|c| {
                c.iter()
                    .any(|&l| value[l.unsigned_abs() as usize] == (l > 0))
            });
            if holds {
                sum += cnf.weights[v][usize::from(b)] * walk(v + 1, value, cnf, last);
            }
        }
        sum
    }
    walk(1, &mut vec![false; n + 1], cnf, &last)
}

/// `count` of `cnf`, checked against its `c count` line where it has one.
fn total(cnf: &Dimacs, file: &str) -> f64 {
    let counted = count(cnf, None);
    if let Some(stated) = cnf.count {
        assert!(
            (counted - stated).abs() <= 1e-12 * stated,
            "{file}: the CNF counts {counted}, its `c count` line says {stated}"
        );
    }
    counted
}

/// The examples whose CNFs are counted, each of at most `MOST_FACTS`
/// facts.
const COUNTED: &[&str] = &[
    "adder.pl",
    "alarm.pl",
    "cycle_graph.pl",
    "dice.pl",
    "dice_evidence.pl",
    "equal_atoms.pl",
    "equal_functions.pl",
    "equal_gates.pl",
    "linear_path.pl",
    "cancer.net",
];

/// A recursion that is not linear, whose cycles the grounder leaves none
/// of: the CNF is the ground program's.
const NONLINEAR: &str = "0.5::e(a,b). 0.5::e(b,a). 0.5::e(b,c). 0.4::e(a,c).
    p(X,Y) :- e(X,Y). p(X,Y) :- p(X,Z), p(Z,Y).
    evidence(p(b,a)). query(p(a,c)). query(p(b,c)).";

fn source(file: &str) -> String {
    let data = std::fs::read(format!("examples/{file}")).unwrap();
    let format = tddlog::Format::of_path(file);
    tddlog::read(&data, format, &Default::default())
        .unwrap()
        .program
        .into_owned()
}

#[test]
fn the_counts_of_each_examples_cnf_are_tddlogs_answers() {
    let programs: Vec<(&str, String)> = COUNTED
        .iter()
        .map(|&file| (file, source(file)))
        .chain([("nonlinear", NONLINEAR.to_string())])
        .collect();
    let large: Vec<_> = programs
        .iter()
        .map(|(file, source)| (file, read(&cnf_text(source, &Options::default()))))
        .filter(|(_, cnf)| cnf.facts > MOST_FACTS)
        .map(|(file, cnf)| format!("{file} ({} facts)", cnf.facts))
        .collect();
    assert!(large.is_empty(), "too many facts to enumerate: {large:?}");
    for (file, source) in &programs {
        let options = Options::default();
        let cnf = read(&cnf_text(source, &options));
        let solution = tddlog::solve(source, &options, &mut |_| {}).unwrap();
        let evidence = total(&cnf, file);
        let Some(answers) = solution.answers else {
            assert_eq!(evidence, 0.0, "{file}: the evidence is inconsistent");
            continue;
        };
        assert_eq!(answers.len(), cnf.queries.len(), "{file}");
        for (answer, (l, text)) in answers.iter().zip(&cnf.queries) {
            assert_eq!(answer.to_string(), *text, "{file}");
            let p = count(&cnf, Some(*l)) / evidence;
            assert!(
                (p - answer.probability).abs() < 1e-9,
                "{file}: {text} counts {p}, tddlog answers {}",
                answer.probability
            );
        }
        // With every observation kept, the count is P(e).
        let options = Options {
            evidence_probability: true,
            ..Options::default()
        };
        let cnf = read(&cnf_text(source, &options));
        let solution = tddlog::solve(source, &options, &mut |_| {}).unwrap();
        let pe = solution.evidence_probability.unwrap().to_f64();
        let counted = total(&cnf, file);
        assert!(
            (counted - pe).abs() <= 1e-12 * pe.max(1e-300) + 1e-15,
            "{file}: the CNF counts {counted}, tddlog's P(e) is {pe}"
        );
    }
}

fn cnf_text(source: &str, options: &Options) -> String {
    tddlog::weighted_cnf(source, options).unwrap().to_string()
}

#[test]
fn alarms_cnf_counts_the_hand_computed_probabilities() {
    // P(e) = P(alarm) P(hears_alarm(john)) = (1 - 0.9 · 0.8) · 0.7 = 0.196,
    // and P(burglary | e) = 0.1 · 0.7 / 0.196.
    let options = Options {
        evidence_probability: true,
        ..Options::default()
    };
    let text = cnf_text(&source("alarm.pl"), &options);
    let cnf = read(&text);
    assert!(cnf.count.is_none(), "{text}");
    assert!((count(&cnf, None) - 0.196).abs() < 1e-15, "{text}");
    let (l, query) = &cnf.queries[0];
    assert_eq!(query, "burglary");
    assert!((count(&cnf, Some(*l)) - 0.07).abs() < 1e-15, "{text}");
}

#[test]
fn a_cnf_without_observations_states_its_count() {
    // P(c) = 1 - 0.7 · 0.4 = 0.72, and the count of the clauses is one.
    let text = cnf_text(
        "0.3::a. 0.6::b. c :- a. c :- b. query(c).",
        &Options::default(),
    );
    let cnf = read(&text);
    assert_eq!(cnf.count, Some(1.0), "{text}");
    assert!((total(&cnf, "c") - 1.0).abs() < 1e-15, "{text}");
    assert!(
        (count(&cnf, Some(cnf.queries[0].0)) - 0.72).abs() < 1e-15,
        "{text}"
    );
}

#[test]
fn a_markov_networks_cnf_counts_its_partition_function() {
    // Z by enumeration of the file's worlds, as in tests/it/networks.rs:
    // 325, and 165 given the evidence file.
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
    ] {
        let out = tddlog_binary(&[&["--cnf", "--evidence-probability"], args].concat());
        assert!(out.status.success(), "{args:?}");
        let cnf = read(&String::from_utf8(out.stdout).unwrap());
        let counted = total(&cnf, args[0]);
        assert!(
            (counted - z).abs() < 1e-9 * z,
            "{args:?}: the CNF counts {counted}, Z is {z}"
        );
    }
}

#[test]
fn a_program_that_keeps_a_cycle_has_no_cnf() {
    // Negation through a cycle: two-valued in every world, and undefined.
    for name in ["negcycle_sound.pl", "unsound.pl"] {
        match tddlog::weighted_cnf(&source(name), &Options::default()) {
            Err(Error::Cnf(e)) => assert!(e.contains("cyclic component"), "{name}: {e}"),
            Err(e) => panic!("{name}: {e}"),
            Ok(_) => panic!("{name}: a CNF of a cyclic program"),
        }
    }
    // A layout's strategy would do nothing.
    let options = Options::with(Strategy::Trial(tddlog::Kind::Closed));
    assert!(matches!(
        tddlog::weighted_cnf(&source("alarm.pl"), &options),
        Err(Error::Options(_))
    ));
    // Nor would a projection.
    let options = Options {
        projection: Some(tddlog::Projection::Off),
        ..Options::default()
    };
    assert!(matches!(
        tddlog::weighted_cnf(&source("alarm.pl"), &options),
        Err(Error::Options(_))
    ));
}

fn tddlog_binary(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tddlog"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn the_command_line_prints_the_cnf_and_refuses_what_it_would_not_read() {
    let out = tddlog_binary(&["--cnf", "examples/alarm.pl"]);
    assert!(out.status.success());
    let options = Options::default();
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        cnf_text(&source("alarm.pl"), &options)
    );
    // A Markov network's count is its partition function; cancer.net is a
    // Bayesian network, whose CNF is its program's.
    let out = tddlog_binary(&["--cnf", "examples/cancer.net"]);
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        cnf_text(&source("cancer.net"), &options)
    );
    let out = tddlog_binary(&["--cnf", "examples/negcycle_sound.pl"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("weighted CNF error"));
    for (args, message) in [
        (
            &["--cnf", "--vtree", "td"][..],
            "so --vtree would do nothing",
        ),
        (&["--cnf", "--stats"], "so --stats would do nothing"),
        (&["--cnf", "--ground"], "so --ground would do nothing"),
        (&["--cnf", "--program"], "so --program would do nothing"),
        (
            &["--cnf", "--precision", "3"],
            "so --precision would do nothing",
        ),
        (&["--cnf", "--cnf"], "--cnf is given twice"),
    ] {
        let mut args = args.to_vec();
        args.push("examples/alarm.pl");
        let out = tddlog_binary(&args);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{args:?}: {stderr}");
        assert!(stderr.contains(message), "{args:?}: {stderr}");
        assert!(out.stdout.is_empty(), "{args:?}");
    }
}
