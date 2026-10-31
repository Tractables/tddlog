//! Every input format through the library (`tddlog::read`, then
//! `tddlog::solve`), as the bindings call it: a Dice program against
//! Dice's answers, a network with its evidence file against ProbLog's, a
//! Markov network's partition function against enumeration, an LPAD
//! against cplint's, a program that consults files (`tddlog::consult`),
//! and the refusals.

use std::borrow::Cow;

use tddlog::network::{Evidence, Queries, ReadOptions};
use tddlog::{Error, Format, Input, Options};

fn file(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The answers to `input`'s program, with its `P(e)` as the input means
/// it (a Markov network's `Z`).
fn answers(input: &Input) -> (Vec<(String, f64)>, (f64, f64)) {
    let options = Options {
        evidence_probability: true,
        ..Options::default()
    };
    let solution = tddlog::solve(&input.program, &options, &mut |_| {}).unwrap();
    let answers = solution
        .answers
        .as_deref()
        .expect("consistent evidence")
        .iter()
        .map(|a| (a.to_string(), a.probability))
        .collect();
    (answers, input.evidence_probability(&solution).unwrap())
}

fn assert_close(what: &str, got: &[(String, f64)], expected: &[(&str, f64)], tolerance: f64) {
    assert_eq!(got.len(), expected.len(), "{what}: {got:?}");
    for ((a, p), (b, q)) in got.iter().zip(expected) {
        assert_eq!(a, b, "{what}");
        assert!((p - q).abs() < tolerance, "{what}: {a} = {p} against {q}");
    }
}

/// tests/dice/walk.dice: a random walk absorbed at 0 and 7, observed not
/// at 0 after ten steps; Dice's answers (tests/it/examples.rs).
#[test]
fn a_dice_program() {
    let data = file("tests/dice/walk.dice");
    let input = tddlog::read(&data, Format::Dice, &ReadOptions::default()).unwrap();
    assert!(matches!(input.program, Cow::Owned(_)));
    let (got, _) = answers(&input);
    let dice = [
        ("x(0)", 0.0),
        ("x(1)", 0.245556733026),
        ("x(2)", 0.0),
        ("x(3)", 0.362357126637),
        ("x(4)", 0.0),
        ("x(5)", 0.190068881992),
        ("x(6)", 0.0),
        ("x(7)", 0.202017258345),
    ];
    assert_close("walk.dice", &got, &dice, 1e-9);
}

/// examples/asia.net with tests/networks/asia_leaves.inst, the evidence
/// file a Hugin network reads: ProbLog's answers and P(e) on the program
/// it is read as, examples/bn_asia_evidence.pl (tests/it/networks.rs and
/// the playground's asia example).
#[test]
fn a_network_and_its_evidence_file() {
    let format: Format = "hugin".parse().unwrap();
    let (data, inst) = (
        file("examples/asia.net"),
        file("tests/networks/asia_leaves.inst"),
    );
    let options = ReadOptions {
        evidence: Some(format.evidence(&inst)),
        ..ReadOptions::default()
    };
    let input = tddlog::read(&data, format, &options).unwrap();
    assert_eq!(input.log10_offset, None);
    let (got, (p, log10)) = answers(&input);
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
    assert_close("asia.net", &got, &asia, 1e-7);
    assert!((p - 0.0396199356).abs() < 1e-10, "P(e) = {p}");
    assert!((log10 - p.log10()).abs() < 1e-12);
}

/// tests/regress/markov.uai given markov.uai.evid, the evidence file a
/// UAI network reads: the partition function 165 and the marginals by
/// enumeration (tests/it/networks.rs).
#[test]
fn a_markov_networks_partition_function() {
    let format: Format = "uai".parse().unwrap();
    let (data, evid) = (
        file("tests/regress/markov.uai"),
        file("tests/regress/markov.uai.evid"),
    );
    assert!(matches!(format.evidence(&evid), Evidence::Uai(_)));
    let options = ReadOptions {
        evidence: Some(format.evidence(&evid)),
        ..ReadOptions::default()
    };
    let input = tddlog::read(&data, format, &options).unwrap();
    assert!(input.log10_offset.is_some());
    let (_, (z, log10_z)) = answers(&input);
    assert!((z - 165.0).abs() < 1e-9, "Z = {z}");
    assert!(
        (log10_z - 165f64.log10()).abs() < 1e-12,
        "log10 Z = {log10_z}"
    );
}

/// tests/lpad/coin.cplint.pl, read as a ProbLog input: cplint's answers
/// under the first example query's evidence, P(e) = P(biased) = 0.1, and
/// the query under other evidence named.
#[test]
fn an_lpad() {
    let data = file("tests/lpad/coin.cplint.pl");
    let input = tddlog::read(&data, Format::Problog, &ReadOptions::default()).unwrap();
    assert_eq!(
        input.program,
        String::from_utf8(file("tests/lpad/coin.pl")).unwrap()
    );
    assert_eq!(
        input.left_out,
        ["left out, asked under other evidence: prob(heads(coin), P)"]
    );
    let (got, (p, _)) = answers(&input);
    assert_close(
        "coin",
        &got,
        &[("heads(coin)", 0.6), ("tails(coin)", 0.4)],
        1e-12,
    );
    assert!((p - 0.1).abs() < 1e-12, "P(e) = {p}");
}

/// A ProbLog program is its own text, and its queries may be left out.
#[test]
fn a_problog_program_is_its_own_text() {
    let data = file("examples/alarm.pl");
    let options = ReadOptions {
        queries: Queries::None,
        ..ReadOptions::default()
    };
    let input = tddlog::read(&data, Format::Problog, &options).unwrap();
    assert!(matches!(input.program, Cow::Borrowed(_)));
    assert_eq!(input.program.as_bytes(), &data[..]);
    assert!(input.left_out.is_empty() && input.log10_offset.is_none());
}

#[test]
fn formats_by_name_and_by_extension() {
    for name in ["problog", "dice", "bif", "hugin", "xdsl", "uai"] {
        assert_eq!(name.parse::<Format>().unwrap().to_string(), name);
    }
    assert!("net".parse::<Format>().is_err());
    for (path, name) in [
        ("a.pl", "problog"),
        ("a.cplint.pl", "problog"),
        ("a.dice", "dice"),
        ("a.bif", "bif"),
        ("a.net", "hugin"),
        ("a.hugin", "hugin"),
        ("a.xdsl", "xdsl"),
        ("a.uai", "uai"),
    ] {
        assert_eq!(Format::of_path(path).to_string(), name, "{path}");
    }
}

/// A network's options for another input, text that is not UTF-8 and a
/// network file that does not read are errors.
#[test]
fn refusals() {
    let inst = file("tests/networks/sprinkler.inst");
    for (format, options) in [
        (
            Format::Problog,
            ReadOptions {
                evidence: Some(Evidence::Inst(&inst)),
                ..ReadOptions::default()
            },
        ),
        (
            Format::Dice,
            ReadOptions {
                rescale: true,
                ..ReadOptions::default()
            },
        ),
        (
            Format::Problog,
            ReadOptions {
                queries: Queries::Leaves,
                ..ReadOptions::default()
            },
        ),
    ] {
        let read = tddlog::read(b"0.5::a. query(a).", format, &options);
        assert!(matches!(read, Err(Error::Options(_))), "{format}");
    }
    let read = tddlog::read(
        b"0.5::a.\nquery(\xff).",
        Format::Problog,
        &Default::default(),
    );
    match read {
        Err(e @ Error::Parse(_)) => {
            assert_eq!(e.to_string(), "parse error: line 2: the text is not UTF-8")
        }
        other => panic!("{other:?}"),
    }
    let bif = Format::Network(tddlog::network::Format::Bif);
    let read = tddlog::read(b"network x {", bif, &Default::default());
    assert!(matches!(read, Err(Error::Network(_))), "{read:?}");
}

#[test]
fn a_program_with_the_files_it_consults() {
    // As ProbLog consults them: in the directive's place, each file once,
    // as named or with `.pl`, in the directory of the file that names it.
    // ProbLog's own tests consult.pl and consult_nested.pl are under
    // tests/problog/.
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("consult");
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    let write = |name: &str, text: &str| {
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        path.to_str().unwrap().to_string()
    };
    write(
        "sub/two.pl",
        "a(2).\n:- consult('../main.pl').\n:- consult(three).\n",
    );
    write("sub/three", "a(3).\n");
    let main = write(
        "main.pl",
        "a(1).\n:- consult('sub/two').\n:- consult(\n    'sub/three').\na(4).\n\
         q(L) :- findall(X, a(X), L).\nquery(q(_)).\n",
    );
    let text = std::fs::read_to_string(&main).unwrap();
    let program = tddlog::consult(&text, &main).unwrap();
    let solution = tddlog::solve(&program, &Options::default(), &mut |_| {}).unwrap();
    let answers: Vec<String> = solution
        .answers
        .unwrap()
        .iter()
        .map(|a| format!("{a} {}", a.probability))
        .collect();
    assert_eq!(answers, ["q([1, 2, 3, 4]) 1"]);
    // A program that consults nothing is its own text.
    assert!(matches!(
        tddlog::consult("a. query(a).", &main),
        Ok(Cow::Borrowed(_))
    ));
    let refused = |text: &str| tddlog::consult(text, &main).unwrap_err().to_string();
    let e = refused("a.\n:- consult(missing).\n");
    assert!(
        e.starts_with("line 2: consult(missing): cannot read"),
        "{e}"
    );
    let e = refused("a. :- consult('sub/three').\n");
    assert!(e.contains("must stand on lines of its own"), "{e}");
    write("sub/nested.pl", "a(5).\n:- consult(nothere).\n");
    let e = refused("\n:- consult('sub/nested').\n");
    assert!(
        e.starts_with("line 2: in ")
            && e.contains("nested.pl, which it consults: line 2: consult(nothere): cannot read"),
        "{e}"
    );
    // Given as text, the program has no file to consult from.
    let Err(e) = tddlog::solve(
        ":- consult(sub).\nquery(a).",
        &Options::default(),
        &mut |_| {},
    ) else {
        panic!("not refused");
    };
    assert!(e.to_string().contains("consult(sub) reads a file"), "{e}");
}
