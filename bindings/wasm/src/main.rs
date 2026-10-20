//! tddlog built for the browser: a ProbLog program goes in, its answers and
//! what the run did come out as JSON.
//!
//! Emscripten links a program, so `main` has to exist. It does nothing and the
//! page never runs it; `worker.js` calls [`tddlog_solve_json`] through
//! `runtime.js`. The run is [`tddlog::solve`], as the command line runs it.

use std::ffi::{CStr, CString, c_char};
use std::fmt::Write;
use std::panic::{AssertUnwindSafe, catch_unwind};

use tddlog::{Options, Progress, Strategy};

fn main() {}

/// Answer the queries of the ProbLog program `program` under the vtree
/// strategy named `strategy` (as `--vtree` takes it; `auto` is the
/// default), with `P(e)` where `evidence_probability` is 1, and return
/// the JSON text of the result (`runtime.js` documents its fields), or of
/// `{"error": ...}` where the program or the options are refused or the
/// run fails. The caller releases the string with [`tddlog_string_free`].
///
/// `evidence_probability` is a `u32` so that it crosses the boundary as a
/// JavaScript number; a value other than 0 or 1 is an error rather than
/// read as true.
///
/// # Safety
///
/// `program` and `strategy` must point to NUL-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tddlog_solve_json(
    program: *const c_char,
    strategy: *const c_char,
    evidence_probability: u32,
) -> *mut c_char {
    if program.is_null() || strategy.is_null() {
        return into_c_string(error_json("no program or no strategy", &Partial::default()));
    }
    // SAFETY: the caller guarantees NUL-terminated strings.
    let (program, strategy) = unsafe { (CStr::from_ptr(program), CStr::from_ptr(strategy)) };
    let (Ok(program), Ok(strategy)) = (program.to_str(), strategy.to_str()) else {
        return into_c_string(error_json("the program is not UTF-8", &Partial::default()));
    };
    // A panic reaching Emscripten aborts the module, and every later call on
    // it fails; the panic hook has written the details to the console.
    let json = catch_unwind(AssertUnwindSafe(|| {
        run(program, strategy, evidence_probability)
    }))
    .unwrap_or_else(|payload| {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned());
        let message = format!(
            "tddlog panicked ({}); please report it as a bug",
            message.as_deref().unwrap_or("no message")
        );
        error_json(&message, &Partial::default())
    });
    into_c_string(json)
}

/// Release a string [`tddlog_solve_json`] returned.
///
/// # Safety
///
/// `text` must be null or a string [`tddlog_solve_json`] returned and nothing has
/// freed since.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tddlog_string_free(text: *mut c_char) {
    if !text.is_null() {
        // SAFETY: the pointer came from `CString::into_raw` in
        // `into_c_string`, and this is its only release point.
        drop(unsafe { CString::from_raw(text) });
    }
}

/// What the run reported before it answered or failed: the grounding's
/// seconds, the notes on the run as a whole, the program handed to the
/// last compile, and a JSON object per part answered.
#[derive(Default)]
struct Partial {
    ground_seconds: f64,
    notes: Vec<String>,
    compiling: Option<String>,
    parts: Vec<String>,
}

fn run(program: &str, strategy: &str, evidence_probability: u32) -> String {
    let mut partial = Partial::default();
    let strategy: Strategy = match strategy.parse() {
        Ok(s) => s,
        Err(e) => return error_json(&format!("options error: {e}"), &partial),
    };
    let evidence_probability = match evidence_probability {
        0 => false,
        1 => true,
        other => {
            return error_json(
                &format!("options error: evidence_probability is {other}, not 0 or 1"),
                &partial,
            );
        }
    };
    let options = Options {
        strategy,
        evidence_probability,
        ..Options::default()
    };
    let solved = tddlog::solve(program, &options, &mut |progress| match progress {
        Progress::Grounded { seconds } => partial.ground_seconds = seconds,
        Progress::Note(line) => partial.notes.push(line),
        Progress::Compiling(ground) => partial.compiling = Some(ground_json(ground)),
        Progress::Answered(run) => partial.parts.push(part_json(run)),
    });
    let solution = match solved {
        Ok(solution) => solution,
        Err(e) => {
            if let tddlog::Error::Trials { notes, .. } = &e {
                partial.notes.extend(notes.iter().cloned());
            }
            return error_json(&e.to_string(), &partial);
        }
    };
    let mut out = String::from("{\"answers\":");
    match &solution.answers {
        None => out.push_str("null"),
        Some(answers) => {
            out.push('[');
            for (i, r) in answers.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                let _ = write!(
                    out,
                    "{{\"query\":{},\"probability\":{}}}",
                    string(&r.to_string()),
                    number(r.probability)
                );
            }
            out.push(']');
        }
    }
    out.push_str(",\"evidence\":");
    match solution.evidence_probability {
        None => out.push_str("null"),
        Some(p) => {
            let _ = write!(
                out,
                "{{\"probability\":{},\"log10\":{}}}",
                number(p.to_f64()),
                number(p.log10())
            );
        }
    }
    push_partial(&mut out, &partial);
    out.push('}');
    out
}

/// The sizes of a ground program.
fn ground_json(g: &tddlog::GroundProgram) -> String {
    format!(
        "{{\"atoms\":{},\"facts\":{},\"derived\":{},\"rules\":{}}}",
        g.num_atoms(),
        g.num_facts(),
        g.num_derived(),
        g.rules.len()
    )
}

/// What a part's run did: the layout it answered with (for `auto`, the
/// trial that won), its compilations, the formulas it built, in pairs, and
/// its seconds; and `auto`'s notes on its trials.
fn part_json(run: &tddlog::Run) -> String {
    let (s, q) = (&run.stats, &run.query_sizes);
    let mut out = format!(
        "{{\"trial\":{},\"attempts\":{},\"ground\":{},\"leaves\":{},\"components\":{},\"cyclic\":{},\"largestPairs\":{},\"finalPairs\":{},\"evidencePairs\":{},\"queryPairs\":{},\"rewriteSeconds\":{},\"vtreeSeconds\":{},\"compileSeconds\":{},\"querySeconds\":{},\"discardedSeconds\":{},\"trials\":[",
        string(&run.kind.to_string()),
        run.attempts,
        ground_json(&run.ground),
        run.vtree.num_leaves(),
        s.components,
        s.cyclic_components,
        s.max_pairs,
        s.final_max_pairs,
        q.evidence,
        q.largest,
        number(run.rewrite_seconds),
        number(run.vtree_seconds),
        number(s.seconds),
        number(run.query_seconds),
        number(run.wasted_seconds),
    );
    for (i, note) in run.vtree_notes.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&string(note));
    }
    out.push_str("]}");
    out
}

/// The fields every result has, answered or not.
fn push_partial(out: &mut String, partial: &Partial) {
    let _ = write!(
        out,
        ",\"groundSeconds\":{},\"compiling\":{},\"notes\":[",
        number(partial.ground_seconds),
        partial.compiling.as_deref().unwrap_or("null")
    );
    for (i, note) in partial.notes.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&string(note));
    }
    out.push_str("],\"parts\":[");
    out.push_str(&partial.parts.join(","));
    out.push(']');
}

fn error_json(message: &str, partial: &Partial) -> String {
    let mut out = format!("{{\"error\":{}", string(message));
    push_partial(&mut out, partial);
    out.push('}');
    out
}

/// A JSON number; `null` for a value JSON cannot hold, as the logarithm of
/// a zero `P(e)`.
fn number(x: f64) -> String {
    if x.is_finite() {
        format!("{x:?}")
    } else {
        "null".to_string()
    }
}

/// A JSON string.
fn string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn into_c_string(json: String) -> *mut c_char {
    // An atom or a message with a NUL inside would be a tddlog bug, so report
    // it as one rather than a truncated result.
    CString::new(json)
        .unwrap_or_else(|_| {
            c"{\"error\":\"the result contains a NUL byte\",\"notes\":[],\"parts\":[]}".to_owned()
        })
        .into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solve(program: &str, strategy: &str, evidence_probability: u32) -> String {
        let program = CString::new(program).unwrap();
        let strategy = CString::new(strategy).unwrap();
        // SAFETY: both are NUL-terminated, and the result is freed once read.
        unsafe {
            let out = tddlog_solve_json(program.as_ptr(), strategy.as_ptr(), evidence_probability);
            let text = CStr::from_ptr(out).to_str().unwrap().to_string();
            tddlog_string_free(out);
            text
        }
    }

    #[test]
    fn answers_and_the_probability_of_the_evidence() {
        let out = solve(
            &std::fs::read_to_string("../../examples/alarm.pl").unwrap(),
            "auto",
            1,
        );
        assert!(
            out.starts_with("{\"answers\":[{\"query\":\"burglary\",\"probability\":0.35714"),
            "{out}"
        );
        assert!(
            out.contains("{\"query\":\"calls(mary)\",\"probability\":0.4"),
            "{out}"
        );
        assert!(out.contains("\"evidence\":{\"probability\":0.196"), "{out}");
        let out = solve("0.3::a. query(\\+a).", "auto", 0);
        assert!(
            out.starts_with("{\"answers\":[{\"query\":\"\\\\+a\",\"probability\":0.7"),
            "{out}"
        );
    }

    #[test]
    fn refused_options_and_programs_are_errors() {
        assert!(
            solve("0.5::a. query(a).", "nonesuch", 0).starts_with("{\"error\":\"options error: ")
        );
        assert!(
            solve("0.5::a. query(a).", "auto", 2)
                .starts_with("{\"error\":\"options error: evidence_probability is 2")
        );
        assert!(solve("0.5::a query(a).", "auto", 0).starts_with("{\"error\":\"parse error: "));
    }

    #[test]
    fn inconsistent_evidence_has_no_answers() {
        let out = solve(
            "0.5::a. b :- a. evidence(a,true). evidence(b,false). query(a).",
            "auto",
            1,
        );
        assert!(
            out.starts_with("{\"answers\":null,\"evidence\":{\"probability\":0.0,\"log10\":null}"),
            "{out}"
        );
    }

    #[test]
    fn strings_are_escaped() {
        assert_eq!(string("a\"b\\c\n\u{1}"), "\"a\\\"b\\\\c\\n\\u0001\"");
        assert_eq!(number(f64::NEG_INFINITY), "null");
        assert_eq!(number(1e-300), "1e-300");
    }
}
