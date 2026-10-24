//! Exact probabilistic inference by compilation into tree decision diagrams.
//!
//! tddlog computes exact probabilities for probabilistic logic programs in
//! the Datalog fragment of ProbLog, for Bayesian and Markov networks, and
//! for discrete probabilistic programs. It grounds a program, compiles it
//! bottom up into tree decision diagrams (TDDs) with the
//! [tididi](https://github.com/Tractables/tididi) library, and counts each
//! query on the diagrams. The same solver is a command-line tool, and runs
//! from [Python](https://tractables.github.io/tddlog/python/),
//! [C](https://tractables.github.io/tddlog/c/) and
//! [the browser](https://tractables.github.io/tddlog/).
//!
//! # A first program
//!
//! [`solve`] takes a program's text and [`Options`], the command line's
//! options, and returns a [`Solution`]: the answers to the program's
//! queries, in their order, and `P(e)`, the probability of the evidence,
//! where it was asked for. Its third argument receives what `--stats`
//! prints as the run goes, which a caller may ignore.
//!
//! ```
//! let program = "
//!     0.1::burglary.
//!     0.2::earthquake.
//!     0.7::hears_alarm(john).
//!     alarm :- burglary.
//!     alarm :- earthquake.
//!     calls(X) :- alarm, hears_alarm(X).
//!     evidence(calls(john), true).
//!     query(burglary).
//! ";
//! let options = tddlog::Options { evidence_probability: true, ..tddlog::Options::default() };
//! let solution = tddlog::solve(program, &options, &mut |_| {})?;
//! // `answers` is `None` where the evidence is inconsistent.
//! for answer in solution.answers.as_deref().unwrap_or_default() {
//!     println!("{answer}: {}", answer.probability);
//! }
//! // P(alarm) = 1 - 0.9 * 0.8 = 0.28, P(e) = 0.28 * 0.7 = 0.196, and
//! // P(burglary | e) = 0.1 * 0.7 / 0.196.
//! let answers = solution.answers.as_deref().unwrap_or_default();
//! assert!((answers[0].probability - 0.1 * 0.7 / 0.196).abs() < 1e-12);
//! let evidence = solution.evidence_probability.expect("asked for");
//! assert!((evidence.to_f64() - 0.196).abs() < 1e-12);
//! # Ok::<(), tddlog::Error>(())
//! ```
//!
//! # Worked examples
//!
//! The [guide](https://tractables.github.io/tddlog/guide/) describes the
//! language tddlog reads and works through these programs, each in the
//! repository's [`examples/`](https://github.com/Tractables/tddlog/tree/main/examples)
//! and open in the playground:
//!
//! 1. [A Bayesian network with evidence](https://tractables.github.io/tddlog/guide/#bayesian-network):
//!    the asia network, each row of its tables an annotated disjunction.
//! 2. [Reachability in a graph with cycles](https://tractables.github.io/tddlog/guide/#reachability):
//!    the probability of a path, defined by recursion.
//! 3. [Integers in binary](https://tractables.github.io/tddlog/guide/#binary-integers):
//!    two random integers added bit by bit, as Dice compiles them.
//! 4. [Negation through recursion](https://tractables.github.io/tddlog/guide/#negation):
//!    the winning positions of a game, under the well-founded semantics.
//!
//! Two programs in `examples/` call this crate: `solve.rs` answers a
//! program file as the command line does and reports which of `auto`'s
//! trials answered, and `sprinkler.rs` writes a Bayesian network as a
//! program from Rust data and checks its posteriors against a sum over its
//! worlds (`cargo run --release --example sprinkler`).
//!
//! # How it works
//!
//! [`syntax`] parses a program; [`ground`] grounds the part of it that the
//! queries and the evidence depend on, and simplifies it; [`indicators`]
//! can rewrite it so that derived atoms are read through variables of
//! their own; [`vtree`] lays out the diagrams' variables; [`tp`] compiles
//! every derived atom to a TDD by a fixpoint of the immediate-consequence
//! operator, one strongly connected component at a time; and [`query`]
//! conjoins the evidence and counts each query against it, with the
//! weighted counts of [`eval`]. [`solve`] runs them under a [`Strategy`]:
//! `auto`, the default, runs trials of layouts ([`Kind`]) under bounds
//! that double until one finishes.
//! [docs/design.md](https://github.com/Tractables/tddlog/blob/main/docs/design.md)
//! describes each stage.

#![doc(html_logo_url = "https://raw.githubusercontent.com/Tractables/tddlog/main/docs/logo.svg")]
#![doc(html_favicon_url = "https://tractables.github.io/tddlog/favicon.svg")]

mod auto;
mod cubes;
mod disjoint;
pub mod eval;
mod evaluation;
pub mod ground;
pub mod indicators;
mod multisets;
pub mod query;
mod solve;
mod strategy;
pub mod syntax;
pub mod tp;
pub mod vtree;

use std::sync::Arc;

pub use ground::{GroundOptions, GroundProgram};
pub use query::QueryResult;
pub use solve::{Options, Progress, Solution, rewritten, solve, solve_ground};
pub use strategy::Strategy;
pub use tp::Projection;
pub use vtree::Kind;

use tp::CompileStats;

/// Everything the driver needs to report.
pub struct Run {
    pub ground: GroundProgram,
    pub vtree: Arc<tididi::Vtree>,
    /// The layout the vtree came from (for `auto`, the trial that
    /// finished), which fixed the program's rewrite and the formulation.
    pub kind: Kind,
    /// Seconds spent building vtrees, which for a structure-aware strategy
    /// is a search and not a formality; for `auto`, over every trial built.
    pub vtree_seconds: f64,
    /// Compilations started, including the ones that stopped at an oversize
    /// formula or a cap; one for a single layout.
    pub attempts: usize,
    /// Diagnostic lines from the vtree searches that ran (`closed`, `td`,
    /// `families`).
    pub vtree_notes: Vec<String>,
    /// Seconds of the rewrites of the program (`Kind::rewrite`), for
    /// `auto` over every trial's, and of the first part's `decision`
    /// grounding.
    pub rewrite_seconds: f64,
    /// Seconds of the attempts that did not make the answers: the compiles
    /// and query phases of the trials that did not win, and what the
    /// winner's stopped attempts built and gave up.
    pub wasted_seconds: f64,
    /// Seconds spent conjoining the evidence and evaluating the queries.
    pub query_seconds: f64,
    /// What conjoining the evidence and the queries built: with indicators,
    /// the conjunction of their links is here.
    pub query_sizes: query::QuerySizes,
    pub stats: CompileStats,
    /// `None` when the evidence is inconsistent.
    pub results: Option<Vec<QueryResult>>,
    /// `P(e)`, the probability of the program's evidence: the count of the
    /// evidence formula times what the grounder's conditioning took out of
    /// it ([`ground::GroundProgram::weight`]); one with no evidence, zero
    /// with inconsistent evidence. `None` where the grounder dropped
    /// observations no query depends on ([`GroundOptions::detach`]),
    /// whose count it does not know.
    pub evidence_probability: Option<eval::Count>,
    /// What `auto` built and no longer reads (`auto`'s `Spent`), held
    /// for the caller to drop.
    pub(crate) spent: auto::Spent,
}

#[derive(Debug)]
pub enum Error {
    Parse(syntax::ParseError),
    Ground(ground::GroundError),
    Vtree(String),
    /// [`Options`] that ask for nothing, or for what the strategy cannot do.
    Options(String),
    Compile(tp::CompileError),
    /// `auto`'s trials ended without an answer: the error that ended the
    /// last of them, and the notes the trials left so far (the `trial:`,
    /// `turn:` and `retired:` lines of [`Run::vtree_notes`]), which `--stats`
    /// prints for a run that fails as for one that answers.
    Trials {
        error: tp::CompileError,
        notes: Vec<String>,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "parse error: {e}"),
            Error::Ground(e) => write!(f, "grounding error: {e}"),
            Error::Vtree(e) => write!(f, "vtree error: {e}"),
            Error::Options(e) => write!(f, "options error: {e}"),
            Error::Compile(e) | Error::Trials { error: e, .. } => {
                write!(f, "compilation error: {e}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<syntax::ParseError> for Error {
    fn from(e: syntax::ParseError) -> Self {
        Error::Parse(e)
    }
}
impl From<ground::GroundError> for Error {
    fn from(e: ground::GroundError) -> Self {
        Error::Ground(e)
    }
}
impl From<tp::CompileError> for Error {
    fn from(e: tp::CompileError) -> Self {
        Error::Compile(e)
    }
}
