//! The C API for tddlog: a ProbLog program goes in as text, and the
//! probabilities of its queries come out.
//!
//! `include/tddlog.h` is generated from this file with cbindgen, so the
//! documentation comments below are what a C caller reads. Building and
//! linking are covered in `bindings/c/README.md`.
//!
//! The options are the command line's. Nothing in this crate decides anything
//! about a program: it converts the arguments, calls `tddlog::solve`, and
//! keeps the answers for the accessors. Every entry point that runs the
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
/// The arguments broke a documented contract: a null pointer, text that is
/// not UTF-8, an unknown vtree strategy, or options that ask for nothing.
pub const TDDLOG_ERROR_INVALID_INPUT: i32 = 1;
/// The program is not in the language tddlog reads.
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
}

/// The answers to a program's queries, and `P(e)` where it was asked for.
/// `tddlog_solve` makes one and `tddlog_solution_free` releases it; the
/// accessors only read it, so threads may share one.
pub struct TddlogSolution {
    /// Each query's text and probability, in the program's order; `None`
    /// where the evidence is inconsistent.
    answers: Option<Vec<(CString, f64)>>,
    /// `P(e)` and its base-10 logarithm, where asked for.
    evidence: Option<(f64, f64)>,
}

impl TddlogSolution {
    /// The solution's values, read out of what `tddlog::solve` returned; the
    /// run that solution holds is freed with it.
    fn new(solution: tddlog::Solution) -> Self {
        let answers = solution.answers.as_deref().map(|answers| {
            answers
                .iter()
                .map(|a| (c_string(&a.to_string()), a.probability))
                .collect()
        });
        let evidence = solution
            .evidence_probability
            .map(|p| (p.to_f64(), p.log10()));
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

/// The defaults: `"auto"`, the queries answered, no `P(e)`.
#[unsafe(no_mangle)]
pub extern "C" fn tddlog_options_default() -> TddlogOptions {
    TddlogOptions {
        vtree: std::ptr::null(),
        evidence_probability: false,
        queries: true,
    }
}

/// Parse, ground, compile and count `program`, a ProbLog program as
/// NUL-terminated UTF-8 text, under `options` (null for the defaults).
///
/// On `TDDLOG_OK`, `*out` is a solution the caller releases with
/// `tddlog_solution_free`; on any other status `*out` is null.
///
/// # Safety
///
/// `program` must point to a NUL-terminated string, `options` must be null or
/// point to one `TddlogOptions` whose `vtree` is null or a NUL-terminated
/// string, and `out` must point to storage for one pointer.
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
    let Some(program) = (unsafe { text(program, "program") }) else {
        return TDDLOG_ERROR_INVALID_INPUT;
    };
    let strategy = if options.vtree.is_null() {
        tddlog::Strategy::Auto
    } else {
        let Some(vtree) = (unsafe { text(options.vtree, "options->vtree") }) else {
            return TDDLOG_ERROR_INVALID_INPUT;
        };
        match vtree.parse::<tddlog::Strategy>() {
            Ok(strategy) => strategy,
            Err(message) => return fail(TDDLOG_ERROR_INVALID_INPUT, &message),
        }
    };
    let options = tddlog::Options {
        strategy,
        evidence_probability: options.evidence_probability,
        queries: options.queries,
        ..tddlog::Options::default()
    };
    match catch_unwind(AssertUnwindSafe(|| {
        tddlog::solve(program, &options, &mut |_| {})
    })) {
        Ok(Ok(solution)) => {
            set_error("");
            unsafe { out.write(Box::into_raw(Box::new(TddlogSolution::new(solution)))) };
            TDDLOG_OK
        }
        Ok(Err(error)) => {
            let status = match error {
                tddlog::Error::Options(_) => TDDLOG_ERROR_INVALID_INPUT,
                tddlog::Error::Parse(_) => TDDLOG_ERROR_PARSE,
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
/// `-INFINITY`), and 0 too where it is under the least double. Returns false,
/// writing nothing, where it was not asked for. Either output may be null.
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

/// The UTF-8 text at `pointer`, recording why not where it is null or not
/// UTF-8; `what` names the argument.
///
/// # Safety
///
/// `pointer` must be null or point to a NUL-terminated string.
unsafe fn text<'a>(pointer: *const c_char, what: &str) -> Option<&'a str> {
    if pointer.is_null() {
        fail(
            TDDLOG_ERROR_INVALID_INPUT,
            &format!("{what} must not be null"),
        );
        return None;
    }
    match unsafe { CStr::from_ptr(pointer) }.to_str() {
        Ok(text) => Some(text),
        Err(e) => {
            fail(
                TDDLOG_ERROR_INVALID_INPUT,
                &format!("{what} is not UTF-8: {e}"),
            );
            None
        }
    }
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
