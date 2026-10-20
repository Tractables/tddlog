//! Helpers the integration tests share: grounding a program with or
//! without row classes and indicators, running one whole, checking a
//! run's answers, and fixed random numbers.

use tddlog::ground::{GroundOptions, GroundProgram};
use tddlog::{Error, Options, Projection, QueryResult, Run, Strategy};

/// How the rows of a table are grounded: apart (`GroundOptions::classes`
/// false), or reading a class atom per distribution (the default).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rows {
    Apart,
    Classes,
}

impl Rows {
    pub fn options(self) -> GroundOptions {
        GroundOptions {
            classes: self == Rows::Classes,
            ..GroundOptions::default()
        }
    }
}

/// `source` parsed, ground with its rows as `rows`, and where `indicators`
/// rewritten to read derived atoms through indicators
/// (`tddlog::indicators::introduce`), which a layout that does not read
/// them then compiles as they are.
pub fn ground(source: &str, rows: Rows, indicators: bool) -> GroundProgram {
    ground_with(source, rows.options(), indicators)
}

/// [`ground`] with the derived atoms as the program writes them, its views
/// not inlined (`GroundOptions::views`) nor read as facts
/// (`GroundOptions::contexts`): for a test of how one atom is read from
/// another.
pub fn ground_as_written(source: &str, rows: Rows, indicators: bool) -> GroundProgram {
    ground_with(
        source,
        GroundOptions {
            views: false,
            contexts: false,
            ..rows.options()
        },
        indicators,
    )
}

fn ground_with(source: &str, options: GroundOptions, indicators: bool) -> GroundProgram {
    let program = tddlog::syntax::parse_program(source).unwrap();
    let gp = tddlog::ground::ground(&program, &options).unwrap();
    if indicators {
        tddlog::indicators::introduce(&gp)
    } else {
        gp
    }
}

/// The options of a run under `strategy`, compiled under `projection`.
pub fn options(strategy: Strategy, projection: Projection) -> Options {
    Options {
        strategy,
        projection: Some(projection),
        ..Options::default()
    }
}

/// `source` parsed, ground under `options` and run whole
/// (`tddlog::solve_ground`): its queries on one program, where `tddlog::solve`
/// may answer them in parts.
pub fn run(source: &str, options: &Options) -> Result<Run, Error> {
    let program = tddlog::syntax::parse_program(source)?;
    let gp = tddlog::ground::ground(&program, &options.ground_options())?;
    tddlog::solve_ground(gp, options)
}

/// Asserts that `results` answer the queries `expected` names, in its
/// order, each within `tolerance`; `what` says which run failed.
pub fn assert_answers<S: AsRef<str>>(
    what: &str,
    results: &[QueryResult],
    expected: &[(S, f64)],
    tolerance: f64,
) {
    assert_eq!(results.len(), expected.len(), "{what}: number of answers");
    for (r, (atom, p)) in results.iter().zip(expected) {
        let got = r.to_string();
        assert_eq!(got, atom.as_ref(), "{what}: query order");
        assert!(
            (r.probability - p).abs() < tolerance,
            "{what}: {got} = {} against {p}",
            r.probability
        );
    }
}

/// Asserts that `results` are the probabilities `expected`, in order, each
/// within `tolerance`; `what` says which run failed.
pub fn assert_probabilities(what: &str, results: &[QueryResult], expected: &[f64], tolerance: f64) {
    assert_eq!(results.len(), expected.len(), "{what}: number of answers");
    for (r, p) in results.iter().zip(expected) {
        assert!(
            (r.probability - p).abs() < tolerance,
            "{what}: {r} = {} against {p}",
            r.probability
        );
    }
}

/// A fixed sequence of numbers in [0, 1), so that generated programs are
/// the same on every run.
pub struct Lcg(pub u64);

impl Lcg {
    pub fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// `k` thousandths, as written in the programs and read back.
pub fn milli(k: usize) -> f64 {
    k as f64 / 1000.0
}
