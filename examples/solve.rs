//! Answers the queries of a program file, as `tddlog FILE` does, and
//! reports the probability of its evidence and which of `auto`'s trials
//! answered.
//!
//! ```text
//! cargo run --release --example solve -- examples/alarm.pl
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/alarm.pl".to_string());
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
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
    let solution = tddlog::solve(&source, &options, &mut |progress| match progress {
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
    if let Some(evidence) = solution.evidence_probability {
        // A count keeps its exponent apart: `log10` is exact where the
        // probability is below the least float.
        println!(
            "P(e):\t{:e}\tlog10 {:.6}",
            evidence.to_f64(),
            evidence.log10()
        );
    }
    eprintln!("answered by {}", layouts.join(", "));
    ExitCode::SUCCESS
}
