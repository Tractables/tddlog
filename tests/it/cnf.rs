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
        assert_counts(file, source);
    }
}

/// The counts of `source`'s CNF, with each query's literal added and with
/// every observation kept, are tddlog's answers and its `P(e)`.
fn assert_counts(file: &str, source: &str) {
    let options = Options::default();
    let cnf = read(&cnf_text(source, &options));
    let solution = tddlog::solve(source, &options, &mut |_| {}).unwrap();
    let evidence = total(&cnf, file);
    let Some(answers) = solution.answers else {
        assert_eq!(evidence, 0.0, "{file}: the evidence is inconsistent");
        return;
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

/// A Markov network of one factor over three variables of three values
/// (tests/networks/local_structure_wide.uai): its entries are 8 (the
/// largest, twice), 0 (twice), 2 (twice), 1 (once), 3 (three times) and 4
/// (seventeen times), so that Z is their sum, 98, and given x2 = 0 the sum
/// of the nine entries of x2 = 0, 36. Each parameter is weighed (w, 1)
/// and defined by its rows, each way: 4's as false in the other eight
/// rows' contexts, 1's as implying its one context, and 2's and 3's as
/// implying one of their contexts, each a variable of its own (five).
#[test]
fn a_factors_parameters_are_defined_by_their_rows() {
    let file = "tests/networks/local_structure_wide.uai";
    let evid = format!("{file}.evid");
    for (evidence, z) in [(false, 98.0), (true, 36.0)] {
        let mut args = vec!["--cnf", "--evidence-probability", file];
        if evidence {
            args.extend(["--evidence", &evid]);
        }
        let out = tddlog_binary(&args);
        assert!(out.status.success(), "{args:?}");
        let cnf = read(&String::from_utf8(out.stdout).unwrap());
        let counted = total(&cnf, file);
        assert!(
            (counted - z).abs() < 1e-9 * z,
            "{args:?}: the CNF counts {counted}, Z is {z}"
        );
    }
    let data = std::fs::read(file).unwrap();
    let input = tddlog::read(&data, tddlog::Format::of_path(file), &Default::default()).unwrap();
    assert_counts(file, &input.program);
    // The facts are two switches of each variable and the four parameters,
    // the variables those, two values of each variable (the third is its
    // first switch) and the five contexts.
    let text = cnf_text(&input.program, &Options::default());
    let cnf = read(&text);
    let mut defined: Vec<f64> = cnf
        .weights
        .iter()
        .filter(|w| w[0] == 1.0 && w[1] != 1.0)
        .map(|w| w[1])
        .collect();
    defined.sort_by(f64::total_cmp);
    assert_eq!(defined, [0.125, 0.25, 0.375, 0.5], "{text}");
    assert_eq!((cnf.facts, cnf.weights.len() - 1), (10, 21), "{text}");
}

/// Two networks whose tables have zeros, entries equal to their table's
/// largest and equal entries (tests/networks/local_structure*.uai), each
/// with and without its evidence file: the counts of their CNFs are
/// tddlog's answers, and through the command line Z, or P(e).
#[test]
fn a_networks_cnf_shares_its_equal_entries_and_writes_its_zeros_as_clauses() {
    // By enumeration of the worlds: the Markov network's Z is 825, and 165
    // given x2 = 1; the Bayesian network's P(x2 = 1) is 0.46.
    for (file, evidence, z) in [
        ("tests/networks/local_structure.uai", false, 825.0),
        ("tests/networks/local_structure.uai", true, 165.0),
        ("tests/networks/local_structure_bayes.uai", false, 1.0),
        ("tests/networks/local_structure_bayes.uai", true, 0.46),
    ] {
        let evid = format!("{file}.evid");
        let (data, evid_data) = (std::fs::read(file).unwrap(), std::fs::read(&evid).unwrap());
        let format = tddlog::Format::of_path(file);
        let options = tddlog::network::ReadOptions {
            evidence: evidence.then(|| format.evidence(&evid_data)),
            ..Default::default()
        };
        let input = tddlog::read(&data, format, &options).unwrap();
        assert_counts(file, &input.program);
        let mut args = vec!["--cnf", "--evidence-probability", file];
        if evidence {
            args.extend(["--evidence", &evid]);
        }
        let out = tddlog_binary(&args);
        assert!(out.status.success(), "{args:?}");
        let cnf = read(&String::from_utf8(out.stdout).unwrap());
        let counted = total(&cnf, file);
        assert!(
            (counted - z).abs() < 1e-9 * z,
            "{args:?}: the CNF counts {counted}, Z is {z}"
        );
    }
    // Without evidence, the Markov network's facts are its variables'
    // switches (one each of x0, x2 and x3, two of x1) and a parameter for
    // each entry of a table other than a zero and its largest, the equal
    // entries of a table one: f0's 1/2 and 1/4, f1's 1/3 and f2's 1/2.
    // Its variables are those and two of x1's values (the third is its
    // switch); the factors and the rows have none, and each zero is a
    // clause. The Bayesian network's facts are x0's switch, x1's three
    // (its rows' switches 0.2, 1/2 and 1/2, 1/2, the first 1/2 of each
    // row one) and x2's two (its rows' 1/2 one, 0.9), and its variables
    // those, x1's three values and one for x2, its second value the
    // first's negation.
    for (file, facts, variables) in [
        ("tests/networks/local_structure.uai", 9, 11),
        ("tests/networks/local_structure_bayes.uai", 6, 10),
    ] {
        let data = std::fs::read(file).unwrap();
        let format = tddlog::Format::of_path(file);
        let input = tddlog::read(&data, format, &Default::default()).unwrap();
        let cnf = read(&cnf_text(&input.program, &Options::default()));
        assert_eq!(
            (cnf.facts, cnf.weights.len() - 1),
            (facts, variables),
            "{file}"
        );
    }
}

/// A table of two parents shares its switches in the CNF, where the
/// compile keeps them apart (src/ground/rows.rs): every two of its rows
/// read different values of `a` or `b`.
#[test]
fn a_table_of_two_parents_shares_its_switches_in_the_cnf() {
    // c's rows over (a, b): (1,1) and (2,2) hold [1/4, 1/4, 1/2], (1,2)
    // [1/4, 1/2, 1/4], (2,1) [1/2, 1/4, 1/4]. Their switches, 1/4 and 1/3,
    // 1/4 and 2/3, 1/2 and 1/2, 1/4 and 1/3, are five facts shared: one
    // 1/4, one 1/3, the 2/3, and (2,1)'s two halves, which one row reads
    // both of. With a's and b's switches, seven facts. By enumeration
    // P(c(2)) = 0.4 · 0.3 · 0.25 + 0.4 · 0.7 · 0.5 + 0.6 · 0.3 · 0.25
    // + 0.6 · 0.7 · 0.25 = 0.32.
    let source = "0.4::a(1); 0.6::a(2). 0.3::b(1); 0.7::b(2).
        0.25::c(1); 0.25::c(2); 0.5::c(3) :- a(1), b(1).
        0.25::c(1); 0.5::c(2); 0.25::c(3) :- a(1), b(2).
        0.5::c(1); 0.25::c(2); 0.25::c(3) :- a(2), b(1).
        0.25::c(1); 0.25::c(2); 0.5::c(3) :- a(2), b(2).
        query(c(2)).";
    assert_counts("two parents", source);
    let cnf = read(&cnf_text(source, &Options::default()));
    assert_eq!(cnf.facts, 7, "{}", cnf_text(source, &Options::default()));
    let (l, query) = &cnf.queries[0];
    assert_eq!(query, "c(2)");
    let p = count(&cnf, Some(*l)) / total(&cnf, "two parents");
    assert!((p - 0.32).abs() < 1e-12, "P(c(2)) counts {p}");
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
