//! The C API for tddlog: a program goes in as text, in any format tddlog
//! reads, and the probabilities of its queries come out.
//!
//! `include/tddlog.h` is generated from this file with cbindgen, so the
//! documentation comments below are what a C caller reads. Building and
//! linking are covered in `bindings/c/README.md`.
//!
//! The options are the command line's. Nothing in this crate decides anything
//! about a program: it converts the arguments, calls `tddlog::read` and
//! `tddlog::solve`, and keeps the answers for the accessors. Every entry point that runs the
//! solver catches a panic and reports it as `TDDLOG_ERROR_PANIC` rather than
//! unwinding into the caller.

use std::any::Any;
use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// What `tddlog_solve` returned: `TDDLOG_OK`, or one of the `TDDLOG_ERROR_`
/// values. `tddlog_last_error_message` describes the failure in words.
pub type TddlogStatus = i32;

/// The call succeeded.
pub const TDDLOG_OK: i32 = 0;
/// The arguments broke a documented contract: a null pointer, a format or
/// vtree strategy that is not UTF-8 or is unknown, evidence for a program
/// that is not a network, or options that ask for nothing.
pub const TDDLOG_ERROR_INVALID_INPUT: i32 = 1;
/// The program is not in the language or the format it is said to be in.
pub const TDDLOG_ERROR_PARSE: i32 = 2;
/// The program parses but cannot be ground: an unsupported construct or a
/// rule that is not range restricted.
pub const TDDLOG_ERROR_GROUND: i32 = 3;
/// The ground program could not be compiled and counted under the vtree
/// strategy asked for, or the well-founded model of some world leaves an atom
/// undefined.
pub const TDDLOG_ERROR_COMPILE: i32 = 4;
/// tddlog panicked. The panic did not cross into the caller; report it as a
/// bug.
pub const TDDLOG_ERROR_PANIC: i32 = 5;

/// What `tddlog_solve` is asked to do: the command line's options. Start from
/// `tddlog_options_default` and change what you need.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TddlogOptions {
    /// The vtree strategy, the command line's `--vtree`: `"auto"`, or one of
    /// the layouts `tddlog --help` lists. Null is `"auto"`.
    pub vtree: *const c_char,
    /// Also compute `P(e)`, the probability of the evidence, which
    /// `tddlog_solution_evidence_probability` then gives. The evidence no
    /// query depends on is then compiled rather than dropped.
    pub evidence_probability: bool,
    /// Answer the program's queries. `false` leaves them out, as the command
    /// line's `--no-queries` does, and needs `evidence_probability`.
    pub queries: bool,
    /// What the program's text is, as the command line tells it by the
    /// file's extension: `"problog"` (an LPAD where it holds
    /// `:- begin_lpad.`), `"dice"`, or a network's, `"bif"`, `"hugin"`
    /// (`.net`), `"xdsl"` or `"uai"`, with every variable queried. Null is
    /// `"problog"`.
    pub format: *const c_char,
    /// For a network, the NUL-terminated text of its evidence file, as the
    /// command line's `--evidence` reads it: an Ace `.inst` file, or a UAI
    /// `.evid` file for a UAI network. Null for none.
    pub evidence: *const c_char,
}

/// The answers to a program's queries, and `P(e)` where it was asked for.
/// `tddlog_solve` makes one and `tddlog_solution_free` releases it; the
/// accessors only read it, so threads may share one.
pub struct TddlogSolution {
    /// Each query's text and probability, in the program's order; `None`
    /// where the evidence is inconsistent.
    answers: Option<Vec<(CString, f64)>>,
    /// `P(e)` of the input and its base-10 logarithm, where asked for.
    evidence: Option<(f64, f64)>,
}

impl TddlogSolution {
    /// The solution's values, read out of what `tddlog::solve` returned for
    /// `input`; the run that solution holds is freed with it.
    fn new(solution: tddlog::Solution, input: &tddlog::Input) -> Self {
        let answers = solution.answers.as_deref().map(|answers| {
            answers
                .iter()
                .map(|a| (c_string(&a.to_string()), a.probability))
                .collect()
        });
        let evidence = input.evidence_probability(&solution);
        TddlogSolution { answers, evidence }
    }

    fn answer(&self, index: usize) -> Option<&(CString, f64)> {
        self.answers.as_ref()?.get(index)
    }
}

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::default());
}

/// The version of these bindings, which is the version of the tddlog release
/// they wrap. The string is static and outlives every other call.
#[unsafe(no_mangle)]
pub extern "C" fn tddlog_version() -> *const c_char {
    const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr().cast()
}

/// Why the last `tddlog_solve` on this thread failed, as a NUL-terminated
/// string: the message the command line prints. The empty string where it
/// succeeded. Never null.
///
/// The message belongs to tddlog and is replaced by the next `tddlog_solve`
/// on the same thread; copy it to keep it.
#[unsafe(no_mangle)]
pub extern "C" fn tddlog_last_error_message() -> *const c_char {
    const EMPTY: &str = "\0";
    LAST_ERROR
        .try_with(|slot| slot.borrow().as_ptr())
        .unwrap_or_else(|_| EMPTY.as_ptr().cast())
}

/// The defaults: `"auto"`, the queries answered, no `P(e)`, a ProbLog
/// program without an evidence file.
#[unsafe(no_mangle)]
pub extern "C" fn tddlog_options_default() -> TddlogOptions {
    TddlogOptions {
        vtree: std::ptr::null(),
        evidence_probability: false,
        queries: true,
        format: std::ptr::null(),
        evidence: std::ptr::null(),
    }
}

/// Read `program`, NUL-terminated text in `options->format`, then ground,
/// compile and count it under `options` (null for the defaults). The text
/// is UTF-8, or for a network ISO-8859-1 too.
///
/// On `TDDLOG_OK`, `*out` is a solution the caller releases with
/// `tddlog_solution_free`; on any other status `*out` is null.
///
/// # Safety
///
/// `program` must point to a NUL-terminated string, `options` must be null or
/// point to one `TddlogOptions` whose `vtree`, `format` and `evidence` are
/// each null or a NUL-terminated string, and `out` must point to storage for
/// one pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tddlog_solve(
    program: *const c_char,
    options: *const TddlogOptions,
    out: *mut *mut TddlogSolution,
) -> TddlogStatus {
    if out.is_null() {
        return fail(TDDLOG_ERROR_INVALID_INPUT, "out must not be null");
    }
    unsafe { out.write(std::ptr::null_mut()) };
    let defaults = tddlog_options_default();
    let options = unsafe { options.as_ref() }.unwrap_or(&defaults);
    let Some(program) = (unsafe { bytes(program, "program") }) else {
        return TDDLOG_ERROR_INVALID_INPUT;
    };
    let Some(strategy) = (unsafe { named::<tddlog::Strategy>(options.vtree, "options->vtree") })
    else {
        return TDDLOG_ERROR_INVALID_INPUT;
    };
    let Some(format) = (unsafe { named::<tddlog::Format>(options.format, "options->format") })
    else {
        return TDDLOG_ERROR_INVALID_INPUT;
    };
    let evidence = match options.evidence.is_null() {
        true => None,
        false => Some(unsafe { CStr::from_ptr(options.evidence) }.to_bytes()),
    };
    let read = tddlog::network::ReadOptions {
        queries: match options.queries {
            true => tddlog::network::Queries::All,
            false => tddlog::network::Queries::None,
        },
        evidence: evidence.map(|e| format.evidence(e)),
        rescale: false,
    };
    let options = tddlog::Options {
        strategy,
        evidence_probability: options.evidence_probability,
        queries: options.queries,
        ..tddlog::Options::default()
    };
    match catch_unwind(AssertUnwindSafe(|| {
        let input = tddlog::read(program, format, &read)?;
        let solution = tddlog::solve(&input.program, &options, &mut |_| {})?;
        Ok::<_, tddlog::Error>(TddlogSolution::new(solution, &input))
    })) {
        Ok(Ok(solution)) => {
            set_error("");
            unsafe { out.write(Box::into_raw(Box::new(solution))) };
            TDDLOG_OK
        }
        Ok(Err(error)) => {
            let status = match error {
                tddlog::Error::Options(_) => TDDLOG_ERROR_INVALID_INPUT,
                tddlog::Error::Parse(_) | tddlog::Error::Network(_) => TDDLOG_ERROR_PARSE,
                tddlog::Error::Ground(_) => TDDLOG_ERROR_GROUND,
                tddlog::Error::Vtree(_)
                | tddlog::Error::Compile(_)
                | tddlog::Error::Trials { .. } => TDDLOG_ERROR_COMPILE,
            };
            fail(status, &error.to_string())
        }
        Err(payload) => fail(TDDLOG_ERROR_PANIC, &panic_message(&*payload)),
    }
}

/// Release a solution `tddlog_solve` made. Null is accepted.
///
/// # Safety
///
/// `solution` must be null or a solution from `tddlog_solve` that nothing has
/// freed since.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tddlog_solution_free(solution: *mut TddlogSolution) {
    if !solution.is_null() {
        drop(unsafe { Box::from_raw(solution) });
    }
}

/// Whether the evidence is consistent. Where it is not, the solution has no
/// answers, and `P(e)`, where asked for, is zero.
///
/// # Safety
///
/// `solution` must be a live solution from `tddlog_solve`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tddlog_solution_consistent(solution: *const TddlogSolution) -> bool {
    unsafe { solution.as_ref() }.is_some_and(|s| s.answers.is_some())
}

/// The number of answers: one per query of the program, in its order; zero
/// where the evidence is inconsistent or the queries were left out.
///
/// # Safety
///
/// `solution` must be a live solution from `tddlog_solve`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tddlog_solution_num_answers(solution: *const TddlogSolution) -> usize {
    unsafe { solution.as_ref() }
        .and_then(|s| s.answers.as_ref())
        .map_or(0, Vec::len)
}

/// The query of answer `index`, as ProbLog prints it (`\+p` for a
/// `query(\+p)`), or null where `index` is not below
/// `tddlog_solution_num_answers`. The string belongs to the solution and
/// lives until it is freed.
///
/// # Safety
///
/// `solution` must be a live solution from `tddlog_solve`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tddlog_solution_query(
    solution: *const TddlogSolution,
    index: usize,
) -> *const c_char {
    unsafe { solution.as_ref() }
        .and_then(|s| s.answer(index))
        .map_or(std::ptr::null(), |(query, _)| query.as_ptr())
}

/// The probability of answer `index`, or NaN where `index` is not below
/// `tddlog_solution_num_answers`.
///
/// # Safety
///
/// `solution` must be a live solution from `tddlog_solve`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tddlog_solution_probability(
    solution: *const TddlogSolution,
    index: usize,
) -> f64 {
    unsafe { solution.as_ref() }
        .and_then(|s| s.answer(index))
        .map_or(f64::NAN, |&(_, p)| p)
}

/// `P(e)`, the probability of the evidence, where `options.evidence_probability`
/// asked for it: writes it to `*probability` and its base-10 logarithm, exact
/// beyond a double's range, to `*log10_probability`, and returns true. `P(e)`
/// is 1 with no evidence, 0 with inconsistent evidence (its logarithm
/// `-INFINITY`), and 0 too where it is under the least double; for a Markov
/// network it is the partition function given the evidence, `Z`. Returns
/// false, writing nothing, where it was not asked for. Either output may be
/// null.
///
/// # Safety
///
/// `solution` must be a live solution from `tddlog_solve`, and each output
/// null or a pointer to storage for one double.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tddlog_solution_evidence_probability(
    solution: *const TddlogSolution,
    probability: *mut f64,
    log10_probability: *mut f64,
) -> bool {
    let Some((p, log10)) = unsafe { solution.as_ref() }.and_then(|s| s.evidence) else {
        return false;
    };
    if let Some(out) = unsafe { probability.as_mut() } {
        *out = p;
    }
    if let Some(out) = unsafe { log10_probability.as_mut() } {
        *out = log10;
    }
    true
}

/// The bytes of the string at `pointer`, recording why not where it is
/// null; `what` names the argument.
///
/// # Safety
///
/// `pointer` must be null or point to a NUL-terminated string.
unsafe fn bytes<'a>(pointer: *const c_char, what: &str) -> Option<&'a [u8]> {
    if pointer.is_null() {
        fail(
            TDDLOG_ERROR_INVALID_INPUT,
            &format!("{what} must not be null"),
        );
        return None;
    }
    Some(unsafe { CStr::from_ptr(pointer) }.to_bytes())
}

/// What the name at `pointer` names, the default where it is null,
/// recording why not where it is not UTF-8 or names nothing; `what` names
/// the argument.
///
/// # Safety
///
/// `pointer` must be null or point to a NUL-terminated string.
unsafe fn named<T>(pointer: *const c_char, what: &str) -> Option<T>
where
    T: std::str::FromStr<Err = String> + Default,
{
    if pointer.is_null() {
        return Some(T::default());
    }
    let parsed = match unsafe { CStr::from_ptr(pointer) }.to_str() {
        Ok(name) => name.parse(),
        Err(e) => Err(format!("{what} is not UTF-8: {e}")),
    };
    parsed
        .map_err(|message| fail(TDDLOG_ERROR_INVALID_INPUT, &message))
        .ok()
}

/// Record `message` as the thread's last error and return `status`.
fn fail(status: TddlogStatus, message: &str) -> TddlogStatus {
    set_error(message);
    status
}

fn set_error(message: &str) {
    let _ = LAST_ERROR.try_with(|slot| *slot.borrow_mut() = c_string(message));
}

/// `text` as a C string; an embedded NUL, which would cut it short, becomes a
/// space.
fn c_string(text: &str) -> CString {
    CString::new(text.replace('\0', " ")).unwrap_or_default()
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    let what = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str));
    match what {
        Some(what) => format!("tddlog panicked: {what}"),
        None => "tddlog panicked".to_string(),
    }
}
