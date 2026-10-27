//! `tddlog`: answer ProbLog queries by Tp-compilation into TDDs.

// jemalloc does not build with MSVC; there the system allocator serves.
#[cfg(not(target_env = "msvc"))]
mod alloc;
mod stats;

use std::process::ExitCode;

use tddlog::network::{self, Evidence, Format, Queries, ReadOptions};
use tddlog::{Options, Progress, Strategy};

use stats::{ground_line, print_stats};

const USAGE: &str = "\
usage: tddlog [options] <program.pl | program.dice | network>

Prints one `atom:<tab>probability` line per query, like ProbLog. A network
file, BIF (.bif), Hugin (.net, .hugin), GeNIe (.xdsl) or UAI (.uai), is
read as a program of one annotated disjunction per row of its tables, a
Dice program (.dice) as the program its symbolic evaluation writes, and a
program that holds `:- begin_lpad.` as an LPAD written for cplint.

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
                                        compiled rather than dropped; for a UAI MARKOV network,
                                        the partition function given the evidence, as `Z:...`
  --no-queries                          leave out the program's queries and print P(e) alone, as
                                        for the partition function: the program is ground for its
                                        evidence alone, and what no observation depends on (in a
                                        Bayesian network, every variable that is not an ancestor
                                        of one) is not compiled; needs --evidence-probability
  --query <all | leaves | X,Y,...>      for a network: the variables queried, each at every value
                                        (default: all): every variable, those without children,
                                        or those named; an observed variable is left out
  --evidence <file>                     for a network: the observations of an Ace .inst file or,
                                        for a .uai network, of a UAI .evid file
  --rescale                             for a network: divide every row of every table by its
                                        sum, for a file that writes its tables in percent or as
                                        weights
  --program                             print the program a network, a Dice program or an LPAD
                                        is read as and exit
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
    /// The file's format where it is a network; `None` for a program.
    network: Option<Format>,
    /// Whether the file is a Dice program.
    dice: bool,
    /// How a network is read: its queries, its evidence file and whether
    /// its rows are rescaled.
    queries: Queries,
    evidence: Option<String>,
    rescale: bool,
    program_only: bool,
}

fn parse_args() -> Result<Request, String> {
    let mut args = std::env::args().skip(1);
    let mut file = None;
    let mut strategy = None;
    let mut precision = None;
    let (mut ground_only, mut stats, mut evidence_probability, mut no_queries) =
        (None, None, None, None);
    let (mut queries, mut evidence, mut rescale, mut program_only) = (None, None, None, None);
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
            option @ "--query" => {
                let v = args.next().ok_or("--query needs a value")?;
                once(&mut queries, v.parse::<Queries>()?, option)?;
            }
            option @ "--evidence" => {
                let v = args.next().ok_or("--evidence needs a file")?;
                once(&mut evidence, v, option)?;
            }
            option @ "--rescale" => once(&mut rescale, (), option)?,
            option @ "--program" => once(&mut program_only, (), option)?,
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
    if file.ends_with(".gz") {
        return Err(format!("{file} is compressed: gunzip it first"));
    }
    let network = Format::of_path(&file);
    let dice = file.ends_with(".dice");
    if network.is_none() {
        for (given, option) in [
            (queries.is_some(), "--query"),
            (evidence.is_some(), "--evidence"),
            (rescale.is_some(), "--rescale"),
        ] {
            if given {
                return Err(format!(
                    "{option} is for a network file (.bif, .net, .hugin, .xdsl, .uai), \
                     and {file} is a program"
                ));
            }
        }
    }
    if queries.is_some() && no_queries.is_some() {
        return Err(
            "--query chooses the queries and --no-queries leaves them out: one of the two".into(),
        );
    }
    let program_only = program_only.is_some();
    if program_only {
        for (given, option) in [
            (ground_only, "--ground"),
            (stats, "--stats"),
            (precision.is_some(), "--precision"),
            (strategy.is_some(), "--vtree"),
            (evidence_probability.is_some(), "--evidence-probability"),
        ] {
            if given {
                return Err(format!(
                    "--program prints the program and stops, so {option} would do nothing"
                ));
            }
        }
    }
    // Without queries, the network is read as the program for P(e) alone.
    let queries = match (queries, no_queries.is_some()) {
        (_, true) => Queries::None,
        (q, false) => q.unwrap_or(Queries::All),
    };
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
        network,
        dice,
        queries,
        evidence,
        rescale: rescale.is_some(),
        program_only,
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
    let (source, log10_offset) = match read_source(&args) {
        Ok(Ok(read)) => read,
        Ok(Err(e)) => {
            eprintln!("{}: {e}", args.file);
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    // An LPAD's example queries that its program leaves out are named, and
    // `--program` prints a network's, a Dice program's or an LPAD's program.
    let lpad = match (args.network, args.dice) {
        (None, false) => tddlog::syntax::lpad::read(&source),
        _ => None,
    };
    let lpad = match lpad.transpose() {
        Ok(lpad) => lpad,
        Err(e) => {
            eprintln!("{}: parse error: {e}", args.file);
            return ExitCode::FAILURE;
        }
    };
    for note in lpad.iter().flat_map(|l| &l.left_out) {
        eprintln!("{}: {note}", args.file);
    }
    if args.program_only {
        match (&lpad, args.network.is_some() || args.dice) {
            (Some(lpad), _) => println!("{}", lpad.program),
            (None, true) => print!("{source}"),
            (None, false) => {
                eprintln!(
                    "--program is for a network file (.bif, .net, .hugin, .xdsl, .uai), a Dice \
                     program (.dice) or an LPAD (a program that holds `:- begin_lpad.`), and {} is \
                     none of them",
                    args.file
                );
                return ExitCode::from(2);
            }
        }
        return ExitCode::SUCCESS;
    }
    match run(&args, source, log10_offset) {
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

/// The program's text, and for a Markov network `log10 Z - log10 P(e)`.
/// The outer error is a file that cannot be read, the inner one a network
/// or a Dice program that cannot be read as a program.
#[allow(clippy::type_complexity)]
fn read_source(args: &Args) -> Result<Result<(String, Option<f64>), String>, String> {
    let read = |path: &str| std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"));
    let Some(format) = args.network else {
        let source = std::fs::read_to_string(&args.file)
            .map_err(|e| format!("cannot read {}: {e}", args.file))?;
        if args.dice {
            return Ok(tddlog::syntax::dice::read(&source)
                .map(|program| (program, None))
                .map_err(|e| format!("parse error: {e}")));
        }
        return Ok(Ok((source, None)));
    };
    let data = read(&args.file)?;
    let evidence = match &args.evidence {
        Some(path) => Some((path, read(path)?)),
        None => None,
    };
    let evidence = match &evidence {
        Some((path, data)) => Some(Evidence::of_path(path, data)?),
        None => None,
    };
    let options = ReadOptions {
        queries: args.queries.clone(),
        evidence,
        rescale: args.rescale,
    };
    Ok(network::read(&data, format, &options).map(|p| (p.program, p.log10_offset)))
}

fn run(args: &Args, source: String, log10_offset: Option<f64>) -> Result<(), tddlog::Error> {
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
        let line = match log10_offset {
            None => magnitude_line("P(e)", p.to_f64(), p.log10(), args.precision),
            // Z given the evidence is P(e) times 10 to the offset.
            Some(c) => magnitude_line(
                "Z",
                p.to_f64() * 10f64.powf(c),
                p.log10() + c,
                args.precision,
            ),
        };
        println!("{line}");
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

/// The `--evidence-probability` line, `label:<tab>x<tab>log10 l`: `x`, of
/// base-10 logarithm `log10`, in scientific notation and its logarithm,
/// each with `digits` after the point. A number outside a float's normal
/// range, a `P(e)` that a count holds or a partition function, has its
/// mantissa and exponent read off the logarithm.
fn magnitude_line(label: &str, x: f64, log10: f64, digits: usize) -> String {
    let scientific = if log10 == f64::NEG_INFINITY {
        format!("{:.digits$e}", 0.0)
    } else if x.is_normal() {
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
    format!("{label}:\t{scientific}\tlog10 {log10:.digits$}")
}
