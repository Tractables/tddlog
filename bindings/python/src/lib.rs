//! Python bindings for tddlog: a program goes in as text, in any format
//! tddlog reads, and the probabilities of its queries come out.
//!
//! The options are the command line's, under its names. Nothing in this crate
//! decides anything about a program: it converts the arguments, calls
//! `tddlog::read` and `tddlog::solve`, and converts the solution back.

use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

create_exception!(
    tddlog,
    Error,
    PyException,
    "A program tddlog could not answer: the base of ParseError, GroundingError \
     and CompilationError."
);
create_exception!(
    tddlog,
    ParseError,
    Error,
    "The program is not in the language or the format it is said to be in."
);
create_exception!(
    tddlog,
    GroundingError,
    Error,
    "The program parses but cannot be ground: an unsupported construct or a \
     rule that is not range restricted."
);
create_exception!(
    tddlog,
    CompilationError,
    Error,
    "The ground program could not be compiled and counted under the vtree \
     strategy asked for, or the well-founded model of some world leaves an \
     atom undefined."
);

/// The exception for each of tddlog's errors, carrying its message. Options
/// that ask for nothing are the caller's mistake, as an option the command
/// line refuses is, and raise `ValueError`.
fn py_error(error: tddlog::Error) -> PyErr {
    let message = error.to_string();
    match error {
        tddlog::Error::Options(_) => PyValueError::new_err(message),
        tddlog::Error::Parse(_) | tddlog::Error::Network(_) => ParseError::new_err(message),
        tddlog::Error::Ground(_) => GroundingError::new_err(message),
        tddlog::Error::Vtree(_) | tddlog::Error::Compile(_) | tddlog::Error::Trials { .. } => {
            CompilationError::new_err(message)
        }
    }
}

/// The answers to a program's queries, and the probability of its evidence
/// where ``solve`` was asked for it.
#[pyclass(module = "tddlog", frozen)]
pub struct Solution {
    /// Each query's text and probability, in the program's order; `None`
    /// where the evidence is inconsistent.
    answers: Option<Vec<(String, f64)>>,
    /// `P(e)` of the input and its base-10 logarithm, where asked for.
    evidence: Option<(f64, f64)>,
}

impl Solution {
    /// The solution's values, read out of what `tddlog::solve` returned for
    /// `input`; the run that solution holds is freed with it.
    fn new(solution: tddlog::Solution, input: &tddlog::Input) -> Self {
        let answers = solution.answers.as_deref().map(|answers| {
            answers
                .iter()
                .map(|a| (a.to_string(), a.probability))
                .collect()
        });
        let evidence = input.evidence_probability(&solution);
        Solution { answers, evidence }
    }
}

#[pymethods]
impl Solution {
    /// A dict from each query, as ProbLog prints it (``\+`` before a
    /// negative one), to its probability, in the program's order; ``None``
    /// where the evidence is inconsistent.
    #[getter]
    fn answers<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyDict>>> {
        let Some(answers) = &self.answers else {
            return Ok(None);
        };
        let dict = PyDict::new(py);
        for (query, probability) in answers {
            dict.set_item(query, probability)?;
        }
        Ok(Some(dict))
    }

    /// ``P(e)``, the probability of the evidence: 1.0 with none, 0.0 where
    /// it is inconsistent, and 0.0 too where it is under the least float,
    /// which ``log10_evidence_probability`` still gives. For a UAI
    /// ``MARKOV`` network, the partition function given the evidence, as
    /// the command line's ``Z:`` line. ``None`` unless ``solve`` was called
    /// with ``evidence_probability=True``.
    #[getter]
    fn evidence_probability(&self) -> Option<f64> {
        self.evidence.map(|(p, _)| p)
    }

    /// The base-10 logarithm of ``P(e)``, exact beyond a float's range:
    /// ``-inf`` where the evidence is inconsistent. ``None`` unless
    /// ``solve`` was called with ``evidence_probability=True``.
    #[getter]
    fn log10_evidence_probability(&self) -> Option<f64> {
        self.evidence.map(|(_, log10)| log10)
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let answers = self.answers(py)?.into_pyobject(py)?.repr()?;
        let evidence = self.evidence_probability().into_pyobject(py)?.repr()?;
        Ok(format!(
            "Solution(answers={answers}, evidence_probability={evidence})"
        ))
    }
}

/// Read ``program``, text in ``format``, then ground, compile and count
/// it, and return the answers to its queries.
///
/// ``format`` names what the text is, as the command line tells it by the
/// file's extension: ``"problog"``, the default (an LPAD where it holds
/// ``:- begin_lpad.``), ``"dice"``, or a network's: ``"bif"``,
/// ``"hugin"`` (``.net``), ``"xdsl"`` or ``"uai"``. A network's
/// variables are each queried at every value, and ``evidence`` is the
/// text of its evidence file, as the command line's ``--evidence`` reads
/// it: a UAI ``.evid`` file for a UAI network, an Ace ``.inst`` file for
/// any other.
///
/// ``vtree`` is the command line's ``--vtree``: ``"auto"``, the default, or
/// one of the layouts it lists. ``evidence_probability=True`` also computes
/// ``P(e)``, compiling the evidence no query depends on rather than dropping
/// it. ``queries=False`` leaves out the program's queries, as
/// ``--no-queries`` does, and needs ``evidence_probability=True``.
///
/// An unknown ``format`` or ``vtree``, ``evidence`` for a program that is
/// not a network, or options that ask for nothing raise ``ValueError``; a
/// program tddlog cannot answer raises a subclass of ``tddlog.Error``. The
/// interpreter lock is released for the whole run.
#[pyfunction]
#[pyo3(signature = (program, *, format = "problog", evidence = None, vtree = "auto", evidence_probability = false, queries = true))]
fn solve(
    py: Python<'_>,
    program: &str,
    format: &str,
    evidence: Option<&str>,
    vtree: &str,
    evidence_probability: bool,
    queries: bool,
) -> PyResult<Solution> {
    let format = format
        .parse::<tddlog::Format>()
        .map_err(PyValueError::new_err)?;
    let strategy = vtree
        .parse::<tddlog::Strategy>()
        .map_err(PyValueError::new_err)?;
    let read = tddlog::network::ReadOptions {
        queries: match queries {
            true => tddlog::network::Queries::All,
            false => tddlog::network::Queries::None,
        },
        evidence: evidence.map(|e| format.evidence(e.as_bytes())),
        rescale: false,
    };
    let options = tddlog::Options {
        strategy,
        evidence_probability,
        queries,
        ..tddlog::Options::default()
    };
    py.detach(|| -> Result<Solution, tddlog::Error> {
        let input = tddlog::read(program.as_bytes(), format, &read)?;
        let solution = tddlog::solve(&input.program, &options, &mut |_| {})?;
        Ok(Solution::new(solution, &input))
    })
    .map_err(py_error)
}

// `pymodule` expands this function into a Rust module of the same name, so it
// cannot be called `tddlog` without shadowing the crate being wrapped. `name`
// is what Python imports and what the initialisation symbol is built from.
/// Exact probabilistic inference by compilation into tree decision
/// diagrams.
#[pymodule]
#[pyo3(name = "tddlog")]
fn python_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add("Error", py.get_type::<Error>())?;
    module.add("ParseError", py.get_type::<ParseError>())?;
    module.add("GroundingError", py.get_type::<GroundingError>())?;
    module.add("CompilationError", py.get_type::<CompilationError>())?;
    module.add_class::<Solution>()?;
    module.add_function(wrap_pyfunction!(solve, module)?)?;
    // The `__init__.py` maturin generates around the extension re-exports
    // exactly this list, `__version__` included.
    module.add(
        "__all__",
        vec![
            "CompilationError",
            "Error",
            "GroundingError",
            "ParseError",
            "Solution",
            "__version__",
            "solve",
        ],
    )
}
