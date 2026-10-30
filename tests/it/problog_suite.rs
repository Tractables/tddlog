//! Run the ProbLog system-test programs under `tests/problog/`, and those
//! under `tests/regress/` that begin with an `%Expected outcome:` block, and
//! compare with the expected outcomes recorded in their comments: under
//! `dynamic`, `closed`, `program` and `auto`, each under its own
//! formulation, under `closed` with projected vtrees, and read through
//! indicators under `families` and `closed`.

use crate::common;

use std::path::Path;

use common::Rows;
use tddlog::{Kind, Projection, Strategy};

fn expected_outcomes(src: &str) -> Vec<(String, f64)> {
    let mut out = Vec::new();
    let mut reading = false;
    for line in src.lines() {
        let s = line.trim();
        if s.starts_with("%Expected outcome:") || s.starts_with("% Expected outcome:") {
            reading = true;
            continue;
        }
        if !reading {
            continue;
        }
        if !s.starts_with('%') {
            reading = false;
            continue;
        }
        let body = s.trim_start_matches('%').trim();
        if body.is_empty() {
            continue;
        }
        let Some((term, prob)) = body.rsplit_once(char::is_whitespace) else {
            continue;
        };
        let Ok(p) = prob.parse::<f64>() else {
            reading = false;
            continue;
        };
        out.push((normalize(term), p));
    }
    out
}

fn normalize(term: &str) -> String {
    term.chars().filter(|c| !c.is_whitespace()).collect()
}

fn check(path: &Path, strategy: Strategy, projection: Projection) -> Vec<String> {
    check_with(path, false, strategy, projection, Rows::Apart, None)
}

/// [`check`] on the program read, where `indicators`, through indicators
/// before `strategy` rewrites it as the binary does, its rows grounded as
/// `rows`, and with a dynamic strategy's first search at
/// `dynamic_first_pairs`.
fn check_with(
    path: &Path,
    indicators: bool,
    strategy: Strategy,
    projection: Projection,
    rows: Rows,
    dynamic_first_pairs: Option<usize>,
) -> Vec<String> {
    let src = std::fs::read_to_string(path).unwrap();
    let expected = expected_outcomes(&src);
    let options = rows.options();
    let ground = tddlog::consult(&src, path.to_str().expect("a path"))
        .and_then(|src| tddlog::syntax::parse_program(&src))
        .map_err(tddlog::Error::from)
        .and_then(|program| Ok(tddlog::ground::ground(&program, &options)?))
        .map(|ground| {
            if indicators {
                tddlog::indicators::introduce(&ground)
            } else {
                ground
            }
        });
    let ground = match ground {
        Ok(g) => g,
        Err(e) => {
            return vec![format!(
                "{}: error {e} (indicators {indicators})",
                path.display()
            )];
        }
    };
    // Families lays out indicators, and says so on a program without any.
    let families = matches!(strategy, Strategy::Trial(Kind::Families { .. }));
    let run = match tddlog::solve_ground(
        ground,
        &tddlog::Options {
            dynamic_first_pairs,
            ..common::options(strategy, projection)
        },
    ) {
        Err(tddlog::Error::Vtree(e)) if families && e.contains("reads no atom through one") => {
            return Vec::new();
        }
        Ok(r) if families && r.ground.encoding.is_empty() => {
            return vec![format!(
                "{}: families without indicators did not refuse (indicators {indicators})",
                path.display()
            )];
        }
        Ok(r) => r,
        Err(e) => {
            return vec![format!(
                "{}: error {e} (indicators {indicators}, {strategy:?})",
                path.display()
            )];
        }
    };
    let mut problems = Vec::new();
    let Some(results) = run.results else {
        return vec![format!(
            "{}: evidence reported inconsistent",
            path.display()
        )];
    };
    for (term, p) in expected {
        let got = results.iter().find(|r| normalize(&r.to_string()) == term);
        match got {
            None => problems.push(format!("{}: {term} missing", path.display())),
            Some(r) if (r.probability - p).abs() > 1e-6 => problems.push(format!(
                "{}: {term} expected {p}, got {}",
                path.display(),
                r.probability
            )),
            _ => {}
        }
    }
    problems
}

#[test]
fn problog_programs_all_configurations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let programs = |dir: &str| -> Vec<_> {
        let mut files: Vec<_> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "pl"))
            .collect();
        files.sort();
        files
    };
    let mut files = programs("tests/problog");
    assert!(
        files.len() > 30,
        "expected the copied suite, found {} files",
        files.len()
    );
    // Programs of our own that state their expected outcomes the same way.
    files.extend(programs("tests/regress").into_iter().filter(|p| {
        std::fs::read_to_string(p)
            .unwrap()
            .starts_with("%Expected outcome:")
    }));
    let mut problems = Vec::new();
    for f in &files {
        // Every layout answers the same questions; only the diagrams differ.
        for strategy in [
            Strategy::Trial(Kind::Dynamic),
            Strategy::Trial(Kind::Closed),
            Strategy::Auto,
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Trial(Kind::Program { fan_in: true }),
        ] {
            problems.extend(check(f, strategy, Projection::Off));
        }
        // Projection must not move an answer: a component's cone contains the
        // support of its formulas, and a variable a formula does not mention
        // marginalises to one in the weighted model count. `max_share` 1.0
        // projects every component a cone allows, 0.5 is the default mix with
        // the global-vtree fallback.
        for max_share in [1.0, 0.5] {
            problems.extend(check(
                f,
                Strategy::Trial(Kind::Closed),
                Projection::Cones { max_share },
            ));
        }
        // Nor may reading derived atoms through indicators: the links admit
        // one value of the indicators in each world, and their weights are
        // one; where a table's rows partition its inputs, its clauses admit
        // the one value of its copies a link would.
        for strategy in [
            Strategy::Trial(Kind::Families { search: true }),
            Strategy::Trial(Kind::Closed),
        ] {
            problems.extend(check_with(
                f,
                true,
                strategy,
                Projection::default(),
                Rows::Apart,
                None,
            ));
        }
        // Row classes (the default) must not move an answer: in each world
        // at most one of the rows that read a class holds.
        for (indicators, strategy) in [
            (false, Strategy::Trial(Kind::Closed)),
            (false, Strategy::Auto),
            (true, Strategy::Trial(Kind::Families { search: true })),
        ] {
            problems.extend(check_with(
                f,
                indicators,
                strategy,
                Projection::default(),
                Rows::Classes,
                None,
            ));
        }
        // A dynamic vtree changes how the formulas are represented, not what
        // they are: a search at every step, without indicators, as auto
        // compiles dynamic.
        problems.extend(check_with(
            f,
            false,
            Strategy::Trial(Kind::Dynamic),
            Projection::Off,
            Rows::Apart,
            Some(0),
        ));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
