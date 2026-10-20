//! `tddlog`: answer ProbLog queries by Tp-compilation into TDDs.

// jemalloc does not build with MSVC; there the system allocator serves.
#[cfg(not(target_env = "msvc"))]
mod alloc;
mod stats;

use std::process::ExitCode;

use tddlog::eval::Count;
use tddlog::{Options, Progress, Strategy};

use stats::{ground_line, print_stats};

const USAGE: &str = "\
usage: tddlog [options] <program.pl>

Prints one `atom:<tab>probability` line per query, like ProbLog.

options:
  --vtree <strategy>                    vtree over the probabilistic facts (default: auto)
      auto                              trials of the layouts below over rewrites of the program,
                                        each compiled with its evidence and queries under bounds
                                        that double until one finishes
      td                                right-linear over a fact order from a tree decomposition;
                                        fails where no elimination of the graph finishes within
                                        its budget
      closed                            components in an order found by local search, each on a
                                        right-linear spine, bushy where its parts share nothing
      families, families:0              a tree decomposition of the program read through
                                        indicators (families:0 without the decomposer's search);
                                        fails for a program that reads no atom through one
      dynamic, dynamic:minfill          right-linear, changed during the compile by a search of
                                        rotations
      program                           right-linear in the order the program states its facts
      program:fanin                     right-linear in the fan-in order of the rules from the
                                        queries and evidence, interleaved over them
      decision                          the tree of the decisions the grounder read facts as
                                        guards in; fails for a program with none
  --ground                              print the relevant ground program and exit; under a
                                        strategy other than auto, as that strategy compiles it;
                                        with --evidence-probability, the program P(e) is
                                        computed on
  --evidence-probability                after the query lines, print P(e), the probability of the
                                        evidence, as `P(e):<tab>p<tab>log10 l`: p in scientific
                                        notation and l its base-10 logarithm, exact beyond a
                                        float's range (0 and -inf for inconsistent evidence, 1
                                        and 0 for none); the evidence no query depends on is then
                                        compiled rather than dropped
  --no-queries                          leave out the program's queries and print P(e) alone, as
                                        for the partition function: the program is ground for its
                                        evidence alone, and what no observation depends on (in a
                                        Bayesian network, every variable that is not an ancestor
                                        of one) is not compiled; needs --evidence-probability
  --stats                               print compilation statistics to stderr; a run that
                                        fails prints the ground program's line and auto's trials
                                        so far before the error
  --precision <n>                       digits after the decimal point (default: 6; for P(e),
                                        after the point of the mantissa and of the logarithm,
                                        and for an answer under 1e-4 that is not zero, after
                                        the point of its mantissa)
  -h, --help                            this text
  -V, --version                         the version of tddlog

Every strategy but auto is one of auto's trials, under the name its trial
lines in --stats print; docs/design.md describes each.";

/// What the command line asks for.
enum Request {
    Help,
    Version,
    Run(Args),
}

struct Args {
    file: String,
    strategy: Strategy,
    ground_only: bool,
    stats: bool,
    evidence_probability: bool,
    no_queries: bool,
    precision: usize,
}

fn parse_args() -> Result<Request, String> {
    let mut args = std::env::args().skip(1);
    let mut file = None;
    let mut strategy = None;
    let mut precision = None;
    let (mut ground_only, mut stats, mut evidence_probability, mut no_queries) =
        (None, None, None, None);
    while let Some(a) = args.next() {
        match a.as_str() {
            "-h" | "--help" => return Ok(Request::Help),
            "-V" | "--version" => return Ok(Request::Version),
            option @ "--vtree" => {
                let v = args.next().ok_or("--vtree needs a value")?;
                once(&mut strategy, v.parse::<Strategy>()?, option)?;
            }
            option @ "--ground" => once(&mut ground_only, (), option)?,
            option @ "--stats" => once(&mut stats, (), option)?,
            option @ "--evidence-probability" => once(&mut evidence_probability, (), option)?,
            option @ "--no-queries" => once(&mut no_queries, (), option)?,
            option @ "--precision" => {
                let v = args.next().ok_or("--precision needs a value")?;
                let digits = v.parse().map_err(|_| format!("bad precision `{v}`"))?;
                once(&mut precision, digits, option)?;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option `{other}`\n{USAGE}"));
            }
            other => {
                if file.is_some() {
                    return Err(format!("only one program file is accepted\n{USAGE}"));
                }
                file = Some(other.to_string());
            }
        }
    }
    let file = file.ok_or_else(|| USAGE.to_string())?;
    let ground_only = ground_only.is_some();
    let stats = stats.is_some();
    if ground_only {
        for (given, option) in [(stats, "--stats"), (precision.is_some(), "--precision")] {
            if given {
                return Err(format!(
                    "--ground stops before the compile, so {option} would do nothing"
                ));
            }
        }
    }
    let strategy = strategy.unwrap_or_default();
    let precision = precision.unwrap_or(6);
    Ok(Request::Run(Args {
        file,
        strategy,
        ground_only,
        stats,
        evidence_probability: evidence_probability.is_some(),
        no_queries: no_queries.is_some(),
        precision,
    }))
}

/// Sets an option's `value`, failing where the `option` is given twice.
fn once<T>(slot: &mut Option<T>, value: T, option: &str) -> Result<(), String> {
    match slot.replace(value) {
        Some(_) => Err(format!("{option} is given twice")),
        None => Ok(()),
    }
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(Request::Run(a)) => a,
        Ok(Request::Help) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Ok(Request::Version) => {
            println!("tddlog {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(2);
        }
    };
    let source = match std::fs::read_to_string(&args.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cannot read {}: {e}", args.file);
            return ExitCode::from(2);
        }
    };
    match run(&args, source) {
        Ok(()) => ExitCode::SUCCESS,
        // Options the library refuses are option errors, as above.
        Err(e @ tddlog::Error::Options(_)) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args, source: String) -> Result<(), tddlog::Error> {
    let options = Options {
        strategy: args.strategy,
        evidence_probability: args.evidence_probability,
        queries: !args.no_queries,
        ..Options::default()
    };
    if args.ground_only {
        print!("{}", tddlog::rewritten(&source, &options)?);
        return Ok(());
    }
    let mut ground_secs = 0.0;
    // A run that fails prints what `--stats` knows by then, before the
    // error: the program as handed to the compile (an answer's line counts
    // the program its compile ended on, with `auto`'s indicators) and,
    // under `auto`, the trials so far.
    let mut failed = None;
    let solved = tddlog::solve(&source, &options, &mut |progress| {
        if !args.stats {
            return;
        }
        match progress {
            Progress::Grounded { seconds } => ground_secs = seconds,
            Progress::Note(line) => eprintln!("{line}"),
            Progress::Compiling(ground) => failed = Some(ground_line(ground, ground_secs)),
            Progress::Answered(run) => print_stats(args, run, ground_secs),
        }
    });
    let solution = match solved {
        Ok(solution) => solution,
        Err(e) => {
            if let Some(line) = failed {
                eprintln!("{line}");
                if let tddlog::Error::Trials { notes, .. } = &e {
                    for note in notes {
                        eprintln!("{note}");
                    }
                }
            }
            return Err(e);
        }
    };
    match &solution.answers {
        None => println!("evidence is inconsistent: probability zero"),
        Some(answers) => {
            for r in answers {
                println!("{r}:\t{}", answer_text(r.probability, args.precision));
            }
        }
    }
    if args.evidence_probability {
        let p = solution
            .evidence_probability
            .expect("the grounder kept every observation");
        println!("{}", evidence_line(p, args.precision));
    }
    // The process ends once the answers are out, and what the solution
    // holds is left to the exit rather than freed piece by piece: freeing
    // the parsed, normalized and ground programs and the last run took up
    // to 1% of a large network's run.
    std::mem::forget(solution);
    Ok(())
}

/// The least answer written in fixed notation.
const LEAST_FIXED: f64 = 1e-4;

/// An answer with `digits` after the decimal point or, where it is not zero
/// and under [`LEAST_FIXED`], in scientific notation with `digits` after the
/// point of the mantissa, as ProbLog prints it: in fixed notation,
/// 5.7e-15 at 17 digits showed three significant digits.
fn answer_text(p: f64, digits: usize) -> String {
    if p > 0.0 && p < LEAST_FIXED {
        format!("{p:.digits$e}")
    } else {
        format!("{p:.digits$}")
    }
}

/// The `--evidence-probability` line, `P(e):<tab>p<tab>log10 l`: `P(e)` in
/// scientific notation and its base-10 logarithm, each with `digits` after
/// the point. A probability under the least normal float, which a
/// [`Count`] holds, has its mantissa and exponent read off the logarithm.
fn evidence_line(p: Count, digits: usize) -> String {
    let log10 = p.log10();
    let x = p.to_f64();
    let scientific = if x.is_normal() || log10 == f64::NEG_INFINITY {
        format!("{x:.digits$e}")
    } else {
        let mut exponent = log10.floor();
        let mut mantissa = format!("{:.digits$}", 10f64.powf(log10 - exponent));
        // A mantissa that rounds to ten is one at the next power.
        if mantissa.starts_with("10") {
            mantissa = format!("{:.digits$}", 1.0);
            exponent += 1.0;
        }
        format!("{mantissa}e{exponent}")
    };
    format!("P(e):\t{scientific}\tlog10 {log10:.digits$}")
}
