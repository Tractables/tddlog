//! tddlog built for the browser: a program goes in, in any format tddlog
//! reads, and its answers and what the run did come out as JSON.
//!
//! Emscripten links a program, so `main` has to exist. It does nothing and the
//! page never runs it; `worker.js` calls [`tddlog_solve_json`] through
//! `runtime.js`. The run is [`tddlog::read`] and [`tddlog::solve`], as the
//! command line runs them.

use std::ffi::{CStr, CString, c_char};
use std::fmt::Write;
use std::panic::{AssertUnwindSafe, catch_unwind};

use tddlog::network::ReadOptions;
use tddlog::trace::{Trace, number, string};
use tddlog::{Format, Options, Progress, Strategy};

fn main() {}

/// Answer the queries of `program`, text in the format named `format`
/// (as `Format` parses it: `problog`, `dice`, `bif`, `hugin`, `xdsl` or
/// `uai`), a network's with `evidence` the text of its evidence file (null
/// for none), under the vtree strategy named `strategy` (as `--vtree`
/// takes it; `auto` is the default), with `P(e)` where
/// `evidence_probability` is 1, a ProbLog program's or an LPAD's
/// arguments the `num_arguments` strings at `arguments` (null for none;
/// each the text of a term that takes the places `$1`, `$2`, ... of the
/// program, as the command line takes the values after the program's
/// file), with the trace of the run ([`tddlog::trace::Trace`]) where
/// `trace` is 1, and return the JSON text of the result (`runtime.js`
/// documents its fields), or of `{"error": ...}` where the program or the
/// options are refused or the run fails. The caller releases the string
/// with [`tddlog_string_free`].
///
/// `evidence_probability` and `trace` are `u32`s so that they cross the
/// boundary as JavaScript numbers; a value other than 0 or 1 is an error
/// rather than read as true.
///
/// # Safety
///
/// `program`, `format` and `strategy` must point to NUL-terminated
/// strings, `evidence` must be null or point to one, and `arguments` must
/// be null or point to `num_arguments` pointers to NUL-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tddlog_solve_json(
    program: *const c_char,
    format: *const c_char,
    evidence: *const c_char,
    strategy: *const c_char,
    evidence_probability: u32,
    arguments: *const *const c_char,
    num_arguments: u32,
    trace: u32,
) -> *mut c_char {
    if program.is_null() || format.is_null() || strategy.is_null() {
        return into_c_string(error_json(
            "no program, no format or no strategy",
            &Partial::default(),
        ));
    }
    // SAFETY: the caller guarantees `num_arguments` pointers at `arguments`.
    let pointers = match (arguments.is_null(), num_arguments) {
        (_, 0) => &[][..],
        (true, _) => {
            return into_c_string(error_json(
                "no arguments, and num_arguments is not 0",
                &Partial::default(),
            ));
        }
        (false, n) => unsafe { std::slice::from_raw_parts(arguments, n as usize) },
    };
    let mut read = Vec::with_capacity(pointers.len());
    for &argument in pointers {
        // SAFETY: the caller guarantees NUL-terminated strings.
        let text = (!argument.is_null()).then(|| unsafe { CStr::from_ptr(argument) }.to_str());
        match text {
            Some(Ok(text)) => read.push(text.to_string()),
            _ => {
                return into_c_string(error_json(
                    "an argument is null or not UTF-8",
                    &Partial::default(),
                ));
            }
        }
    }
    // SAFETY: the caller guarantees NUL-terminated strings.
    let (program, format, strategy) = unsafe {
        (
            CStr::from_ptr(program).to_bytes(),
            CStr::from_ptr(format),
            CStr::from_ptr(strategy),
        )
    };
    let evidence = (!evidence.is_null()).then(|| unsafe { CStr::from_ptr(evidence) }.to_bytes());
    let (Ok(format), Ok(strategy)) = (format.to_str(), strategy.to_str()) else {
        return into_c_string(error_json(
            "the format or the strategy is not UTF-8",
            &Partial::default(),
        ));
    };
    // A panic reaching Emscripten aborts the module, and every later call on
    // it fails; the panic hook has written the details to the console.
    let json = catch_unwind(AssertUnwindSafe(|| {
        run(
            program,
            format,
            evidence,
            strategy,
            (evidence_probability, trace),
            read,
        )
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
/// last compile, a JSON object per part answered, and the trace where it
/// was asked for.
#[derive(Default)]
struct Partial {
    ground_seconds: f64,
    notes: Vec<String>,
    compiling: Option<String>,
    parts: Vec<String>,
    trace: Option<Trace>,
}

fn run(
    program: &[u8],
    format: &str,
    evidence: Option<&[u8]>,
    strategy: &str,
    (evidence_probability, trace): (u32, u32),
    arguments: Vec<String>,
) -> String {
    let mut partial = Partial::default();
    let format: Format = match format.parse() {
        Ok(f) => f,
        Err(e) => return error_json(&format!("options error: {e}"), &partial),
    };
    let strategy: Strategy = match strategy.parse() {
        Ok(s) => s,
        Err(e) => return error_json(&format!("options error: {e}"), &partial),
    };
    let flag = |name: &str, value: u32| match value {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(format!("options error: {name} is {other}, not 0 or 1")),
    };
    let (evidence_probability, traced) = match (
        flag("evidence_probability", evidence_probability),
        flag("trace", trace),
    ) {
        (Ok(e), Ok(t)) => (e, t),
        (Err(e), _) | (_, Err(e)) => return error_json(&e, &partial),
    };
    let read = ReadOptions {
        evidence: evidence.map(|e| format.evidence(e)),
        arguments,
        ..ReadOptions::default()
    };
    let input = match tddlog::read(program, format, &read) {
        Ok(input) => input,
        Err(e) => return error_json(&e.to_string(), &partial),
    };
    // An LPAD's example queries that its program leaves out.
    partial.notes.extend(input.left_out.iter().cloned());
    let options = Options {
        strategy,
        evidence_probability,
        trace: traced,
        ..Options::default()
    };
    if traced {
        match Trace::new(format, program, &input, &options) {
            Ok(trace) => partial.trace = Some(trace),
            Err(e) => return error_json(&e.to_string(), &partial),
        }
    }
    let solved = tddlog::solve(&input.program, &options, &mut |progress| {
        if let Some(trace) = partial.trace.as_mut() {
            trace.observe(&progress);
        }
        match progress {
            Progress::Parsed { .. } => {}
            Progress::Grounded { seconds, .. } => partial.ground_seconds = seconds,
            Progress::Note(line) => partial.notes.push(line),
            Progress::Compiling(ground) => partial.compiling = Some(ground_json(ground)),
            Progress::Answered(run) => partial.parts.push(part_json(run)),
        }
    });
    let solution = match solved {
        Ok(solution) => solution,
        Err(e) => {
            if let tddlog::Error::Trials { notes, .. } = &e {
                partial.notes.extend(notes.iter().cloned());
            }
            if let Some(trace) = partial.trace.as_mut() {
                trace.failed(&e);
            }
            return error_json(&e.to_string(), &partial);
        }
    };
    if let Some(trace) = partial.trace.as_mut() {
        trace.answered(&solution, &input);
    }
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
    match input.evidence_probability(&solution) {
        None => out.push_str("null"),
        Some((p, log10)) => {
            let _ = write!(
                out,
                "{{\"probability\":{},\"log10\":{},\"z\":{}}}",
                number(p),
                number(log10),
                input.log10_offset.is_some()
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
    push_strings(&mut out, &run.vtree_notes);
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
    push_strings(out, &partial.notes);
    out.push_str("],\"parts\":[");
    out.push_str(&partial.parts.join(","));
    out.push_str("],\"trace\":");
    match &partial.trace {
        Some(trace) => {
            let _ = write!(out, "{}", trace.to_string().trim_end());
        }
        None => out.push_str("null"),
    }
}

fn push_strings(out: &mut String, items: &[String]) {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&string(item));
    }
}

fn error_json(message: &str, partial: &Partial) -> String {
    let mut out = format!("{{\"error\":{}", string(message));
    push_partial(&mut out, partial);
    out.push('}');
    out
}

fn into_c_string(json: String) -> *mut c_char {
    // An atom or a message with a NUL inside would be a tddlog bug, so report
    // it as one rather than a truncated result.
    CString::new(json)
        .unwrap_or_else(|_| {
            c"{\"error\":\"the result contains a NUL byte\",\"notes\":[],\"parts\":[],\"trace\":null}"
                .to_owned()
        })
        .into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solve(program: &str, strategy: &str, evidence_probability: u32) -> String {
        solve_in(
            program.as_bytes(),
            "problog",
            None,
            strategy,
            evidence_probability,
        )
    }

    fn solve_in(
        program: &[u8],
        format: &str,
        evidence: Option<&[u8]>,
        strategy: &str,
        evidence_probability: u32,
    ) -> String {
        solve_with(
            program,
            format,
            evidence,
            strategy,
            evidence_probability,
            &[],
        )
    }

    /// [`solve_in`] of a program given `arguments`.
    fn solve_with(
        program: &[u8],
        format: &str,
        evidence: Option<&[u8]>,
        strategy: &str,
        evidence_probability: u32,
        arguments: &[&str],
    ) -> String {
        solve_traced(
            program,
            format,
            evidence,
            strategy,
            evidence_probability,
            arguments,
            0,
        )
    }

    /// [`solve_with`], with the trace where `trace` is 1.
    fn solve_traced(
        program: &[u8],
        format: &str,
        evidence: Option<&[u8]>,
        strategy: &str,
        evidence_probability: u32,
        arguments: &[&str],
        trace: u32,
    ) -> String {
        let program = CString::new(program).unwrap();
        let format = CString::new(format).unwrap();
        let evidence = evidence.map(|e| CString::new(e).unwrap());
        let strategy = CString::new(strategy).unwrap();
        let arguments: Vec<CString> = arguments
            .iter()
            .map(|a| CString::new(*a).unwrap())
            .collect();
        let pointers: Vec<*const c_char> = arguments.iter().map(|a| a.as_ptr()).collect();
        // SAFETY: each is NUL-terminated or null, and the result is freed
        // once read.
        unsafe {
            let out = tddlog_solve_json(
                program.as_ptr(),
                format.as_ptr(),
                evidence.as_ref().map_or(std::ptr::null(), |e| e.as_ptr()),
                strategy.as_ptr(),
                evidence_probability,
                pointers.as_ptr(),
                pointers.len() as u32,
                trace,
            );
            let text = CStr::from_ptr(out).to_str().unwrap().to_string();
            tddlog_string_free(out);
            text
        }
    }

    /// A file's bytes: a network file may be ISO-8859-1, as
    /// tests/networks/sprinkler.xdsl is.
    fn file(path: &str) -> Vec<u8> {
        std::fs::read(format!("../../{path}")).unwrap()
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
        assert!(out.contains("\"z\":false}"), "{out}");
        let out = solve("0.3::a. query(\\+a).", "auto", 0);
        assert!(
            out.starts_with("{\"answers\":[{\"query\":\"\\\\+a\",\"probability\":0.7"),
            "{out}"
        );
    }

    /// A Dice program against Dice's answers, a network with its evidence
    /// file against ProbLog's, and a Markov network's partition function,
    /// 165 by enumeration (tests/it/input.rs).
    #[test]
    fn other_formats() {
        let out = solve_in(&file("tests/dice/walk.dice"), "dice", None, "auto", 0);
        assert!(
            out.contains("{\"query\":\"x(1)\",\"probability\":0.2455567330"),
            "{out}"
        );
        let out = solve_in(
            &file("tests/networks/sprinkler.xdsl"),
            "xdsl",
            Some(&file("tests/networks/sprinkler.inst")),
            "auto",
            0,
        );
        assert!(
            out.contains("{\"query\":\"wetgrass(t)\",\"probability\":0.92"),
            "{out}"
        );
        let out = solve_in(
            &file("tests/regress/markov.uai"),
            "uai",
            Some(&file("tests/regress/markov.uai.evid")),
            "auto",
            1,
        );
        assert!(out.contains("\"evidence\":{\"probability\":16"), "{out}");
        assert!(out.contains("\"z\":true}"), "{out}");
        assert!(
            solve_in(b"0.5::a. query(a).", "prolog", None, "auto", 0)
                .starts_with("{\"error\":\"options error: unknown format")
        );
        assert!(
            solve_in(b"0.5::a. query(a).", "problog", Some(b""), "auto", 0)
                .starts_with("{\"error\":\"options error: an evidence file is for a network")
        );
    }

    /// examples/chain.pl, given a chain of 10 links of 0.9 and node 4,
    /// reaches it with probability 0.9^4; a place without an argument is
    /// an error.
    #[test]
    fn arguments_take_their_places() {
        let chain = file("examples/chain.pl");
        let out = solve_with(&chain, "problog", None, "auto", 0, &["10", "0.9", "4"]);
        assert!(
            out.starts_with("{\"answers\":[{\"query\":\"reach(4)\",\"probability\":0.6561"),
            "{out}"
        );
        let out = solve_with(&chain, "problog", None, "auto", 0, &["10", "0.9"]);
        assert!(out.contains("`$3` has no argument"), "{out}");
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
            out.starts_with(
                "{\"answers\":null,\"evidence\":{\"probability\":0.0,\"log10\":null,\"z\":false}"
            ),
            "{out}"
        );
    }

    /// The trace of a run is in its result where it is asked for, its
    /// answers the result's; a run that fails ends its trace with the
    /// error; and a flag other than 0 or 1 is refused.
    #[test]
    fn the_trace_comes_with_the_answers() {
        let alarm = file("examples/alarm.pl");
        let out = solve_with(&alarm, "problog", None, "auto", 0, &[]);
        assert!(out.ends_with(",\"trace\":null}"), "{out}");
        let out = solve_traced(&alarm, "problog", None, "auto", 0, &[], 1);
        let at = out.find(",\"trace\":{\"trace\":1,").expect("a trace");
        let answers = "\"answers\":[{\"query\":\"burglary\",\"probability\":0.35714";
        assert!(out.starts_with(&format!("{{{answers}")), "{out}");
        assert!(
            out[at..].contains(&format!("{{\"stage\":\"answers\",{answers}")),
            "{out}"
        );
        for stage in [
            "read", "parse", "ground", "rewrite", "layout", "compile", "evidence",
        ] {
            assert!(
                out[at..].contains(&format!("{{\"stage\":\"{stage}\",")),
                "{stage}: {out}"
            );
        }
        let out = solve_traced(b"0.5::a query(a).", "problog", None, "auto", 0, &[], 1);
        assert!(out.starts_with("{\"error\":\"parse error: "), "{out}");
        assert!(
            out.contains("{\"stage\":\"error\",\"message\":\"parse error: "),
            "{out}"
        );
        let out = solve_traced(&alarm, "problog", None, "auto", 0, &[], 2);
        assert!(
            out.starts_with("{\"error\":\"options error: trace is 2"),
            "{out}"
        );
    }
}
