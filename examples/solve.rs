//! Answers the queries of a file, as `tddlog FILE` does, and reports the
//! probability of its evidence and which of `auto`'s trials answered. The
//! file's extension names its format: a ProbLog program, a Dice program
//! (`.dice`) or a network (`.bif`, `.net`, `.xdsl`, `.uai`).
//!
//! ```text
//! cargo run --release --example solve -- examples/alarm.pl
//! cargo run --release --example solve -- examples/asia.net
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/alarm.pl".to_string());
    let data = match std::fs::read(&path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    // The file read as the program tddlog answers, a network with every
    // variable queried and no evidence.
    let format = tddlog::Format::of_path(&path);
    let input = match tddlog::read(&data, format, &Default::default()) {
        Ok(input) => input,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let options = tddlog::Options {
        evidence_probability: true,
        ..tddlog::Options::default()
    };
    // A program whose queries share nothing is answered in parts, a run
    // each; every run reports the layout that answered it.
    let mut layouts = Vec::new();
    let solution = tddlog::solve(&input.program, &options, &mut |progress| match progress {
        tddlog::Progress::Grounded { seconds } => eprintln!("ground in {seconds:.3} s"),
        tddlog::Progress::Answered(run) => layouts.push(run.kind.to_string()),
        _ => {}
    });
    let solution = match solution {
        Ok(solution) => solution,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    match &solution.answers {
        Some(answers) => {
            for answer in answers {
                println!("{answer}:\t{:.6}", answer.probability);
            }
        }
        None => println!("the evidence is inconsistent"),
    }
    // A count keeps its exponent apart: `log10` is exact where the
    // probability is below the least float. For a Markov network, this is
    // the partition function.
    if let Some((p, log10)) = input.evidence_probability(&solution) {
        println!("P(e):\t{p:e}\tlog10 {log10:.6}");
    }
    eprintln!("answered by {}", layouts.join(", "));
    ExitCode::SUCCESS
}
