//! `auto`'s trials, their order and their bounds, on small programs.

use super::policy::*;
use super::trial::Trial;
use super::*;
use crate::ground::deps::Deps;
use crate::ground::shape;

/// [`run_within`] from the first round's `first_pairs` and the live pairs
/// at which a dynamic trial searches first, `dynamic_first_pairs`.
fn run_with_watch(
    original: GroundProgram,
    trials: Vec<Kind>,
    projection: Projection,
    first_pairs: usize,
    dynamic_first_pairs: usize,
) -> Result<Run, Error> {
    run_within(
        Programs {
            ground: original,
            written: None,
            conditioned: None,
            decision: false,
        },
        trials,
        projection,
        Watch {
            first_pairs,
            dynamic_first_pairs,
            memory_bytes: None,
        },
        None,
    )
}

/// [`run_within`] with every operation held to `memory_bytes` ([`Watch`]):
/// the tests' way to make a trial's operations refused on a small program.
fn run_in_memory(
    original: GroundProgram,
    trials: Vec<Kind>,
    memory_bytes: u64,
) -> Result<Run, Error> {
    run_within(
        Programs {
            ground: original,
            written: None,
            conditioned: None,
            decision: false,
        },
        trials,
        Projection::default(),
        Watch {
            memory_bytes: Some(memory_bytes),
            ..Watch::default()
        },
        None,
    )
}

/// The watch and the trial of each bounded attempt in `notes`, in order.
fn attempts(notes: &[String]) -> Vec<(usize, String)> {
    notes
        .iter()
        .filter_map(|n| {
            let (trial, rest) = n.strip_prefix("trial: ")?.split_once(", watch=")?;
            Some((rest.split(',').next()?.parse().ok()?, trial.to_string()))
        })
        .collect()
}

/// Asserts that the bounded attempts in `notes` go in turns: the trials
/// take their first turns in the order of `trials`, and each trial's
/// watch only grows from attempt to attempt, by powers of two.
fn assert_turns(notes: &[String], trials: &[Kind]) {
    let name = |k: &Kind| format!("{k} indicators={}", k.indicators_name());
    let place = |t: &str| {
        trials
            .iter()
            .position(|k| name(k) == t)
            .unwrap_or_else(|| panic!("{t} in {notes:#?}"))
    };
    let attempts = attempts(notes);
    let mut firsts: Vec<usize> = Vec::new();
    let mut watch: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for (w, t) in &attempts {
        match watch.insert(t, *w) {
            None => firsts.push(place(t)),
            Some(before) => assert!(
                *w >= before && (w / before).is_power_of_two() && w % before == 0,
                "{t}: {before} then {w} in {notes:#?}"
            ),
        }
    }
    assert!(firsts.is_sorted(), "{firsts:?} in {notes:#?}");
}

#[test]
fn dynamic_is_the_one_trial_where_there_is_no_order_to_search() {
    // The programs as written: with their views inlined, a chain of
    // atoms each read once is one rule (`GroundOptions::views`).
    let options = crate::GroundOptions {
        views: false,
        ..crate::GroundOptions::default()
    };
    let ground = |src: &str| {
        crate::ground::ground(
            &crate::syntax::parse_program(src).expect("parses"),
            &options,
        )
        .expect("grounds")
    };
    let strategies = |gp: &GroundProgram| candidates(gp);
    // A chain of derived atoms, each reading the one before it and a
    // fact of its own: four components in a dependency order, which is
    // an order the search can move things in.
    let chain = |n: usize| {
        let mut src = String::new();
        for i in 0..n {
            let before = if i == 0 {
                String::new()
            } else {
                format!("a{}, ", i - 1)
            };
            src += &format!("0.5::f{i}. a{i} :- {before}f{i}.\n");
        }
        ground(&format!("{src}query(a{}).", n - 1))
    };
    let laid = Kind::Families { search: false };
    let searched = Kind::Families { search: true };
    assert_eq!(
        candidates(&chain(FEW_FACTS + 1)),
        vec![
            laid,
            Kind::Closed,
            Kind::Td,
            searched,
            Kind::Program { fan_in: false },
        ],
    );
    // With few facts but one derived atom per fact the families
    // layout still comes first.
    assert_eq!(candidates(&chain(4))[0], laid);
    // With few facts and many derived atoms per fact `closed` comes
    // first: the chain of four, read by a chain of 36 atoms of no fact.
    let mut src = String::new();
    for i in 0..4 {
        let before = if i == 0 {
            String::new()
        } else {
            format!("a{}, ", i - 1)
        };
        src += &format!("0.5::f{i}. a{i} :- {before}f{i}.\n");
    }
    src += "c0 :- a0, a1.\n";
    for j in 1..36 {
        src += &format!("c{j} :- c{}, a{}.\n", j - 1, j % 4);
    }
    let gp = ground(&format!("{src}query(c35)."));
    assert!(
        gp.num_derived() >= DERIVED_PER_FACT * gp.num_facts(),
        "{} over {}",
        gp.num_derived(),
        gp.num_facts()
    );
    assert_eq!(
        candidates(&gp),
        vec![
            Kind::Closed,
            laid,
            Kind::Td,
            searched,
            Kind::Program { fan_in: false },
        ],
    );
    // The smokers program is one recursive component: `smokes` over
    // three people, merged with nothing to be merged with. Queried for
    // everyone, as the benchmark programs are; a query for one person is
    // grounded as the chains of influence from that person
    // (`ground::linear`).
    let gp = ground(
        "0.3::stress(X) :- person(X).
                     0.2::influences(X,Y) :- person(X), person(Y).
                     person(p1). person(p2). person(p3).
                     friend(p1,p2). friend(p2,p1). friend(p2,p3). friend(p3,p2).
                     smokes(X) :- stress(X).
                     smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).
                     query(smokes(p1)). query(smokes(p2)). query(smokes(p3)).",
    );
    assert_eq!(strategies(&gp), vec![Kind::Dynamic]);
    // Transitive closure over a five-node graph, queried from several
    // sources: the shape of the random graphs, whose benchmark programs
    // query a path to every node. One component as well.
    let gp = ground(
        "0.5::e(1,2). 0.5::e(2,3). 0.5::e(3,4). 0.5::e(4,5). 0.5::e(5,1).
                     0.5::e(1,3). 0.5::e(2,5). 0.5::e(3,1). 0.5::e(4,2).
                     p(X,Y) :- e(X,Y). p(X,Y) :- e(X,Z), p(Z,Y).
                     query(p(1,2)). query(p(1,3)). query(p(1,4)). query(p(1,5)).
                     query(p(2,4)). query(p(3,5)). query(p(5,3)).",
    );
    assert_eq!(candidates(&gp), vec![Kind::Dynamic]);
}

#[test]
fn a_program_built_gate_by_gate_tries_its_own_orders_first() {
    // A mux and a gate of one rule read once, which the grounder
    // inlines as a view: `program` is the first trial and
    // `program:fanin` the second, and `td` is none. The same program
    // ground without inlining has no view, and `program` alone goes
    // last.
    let src = "0.5::f0. 0.5::f1. 0.5::f2. 0.5::f3. 0.5::f4. 0.5::f5.
               g1 :- f0, f1. g1 :- f2, \\+f0.
               g2 :- g1, f3.
               q :- g2, f4. q :- f5, \\+g1.
               query(q).";
    let program = crate::syntax::parse_program(src).expect("parses");
    let laid = Kind::Families { search: false };
    let searched = Kind::Families { search: true };
    let gp = crate::ground::ground(&program, &crate::GroundOptions::default()).expect("grounds");
    assert_eq!(gp.stats.inlined, 1);
    assert_eq!(
        candidates(&gp),
        vec![
            Kind::Program { fan_in: false },
            Kind::Program { fan_in: true },
            laid,
            Kind::Closed,
            searched,
        ],
    );
    let options = crate::GroundOptions {
        views: false,
        ..crate::GroundOptions::default()
    };
    let gp = crate::ground::ground(&program, &options).expect("grounds");
    assert_eq!(gp.stats.inlined, 0);
    assert_eq!(
        candidates(&gp).last(),
        Some(&Kind::Program { fan_in: false })
    );
    assert!(!candidates(&gp).contains(&Kind::Program { fan_in: true }));
    assert!(candidates(&gp).contains(&Kind::Td));
}

/// Where `decision` is no trial, the conditioned program goes to
/// `program`, `closed` and `td` and the program as ground to
/// `program:fanin` and the families trials, so `program` and
/// `program:fanin` share no rewrite; where it is a trial, it goes to
/// `decision` alone ([`Programs::of`], [`same_rewrite`]).
#[test]
fn the_conditioned_program_goes_to_the_layouts_its_rewrite_cannot_mislead() {
    let ground = |src: &str| {
        crate::ground::ground(
            &crate::syntax::parse_program(src).expect("parses"),
            &crate::GroundOptions::default(),
        )
        .expect("grounds")
    };
    let as_ground = ground("0.5::f0. 0.5::f1. q :- f0, f1. query(q).");
    let conditioned = ground("0.5::f0. 0.5::f1. q :- f0. q :- f1. query(q).");
    let program = Kind::Program { fan_in: false };
    let fan_in = Kind::Program { fan_in: true };
    let laid = Kind::Families { search: false };
    for decision in [false, true] {
        let programs = Programs {
            ground: as_ground.clone(),
            written: None,
            conditioned: Some(conditioned.clone()),
            decision,
        };
        let is_conditioned = |kind: Kind| {
            std::ptr::eq(
                programs.of(kind),
                programs.conditioned.as_ref().expect("conditioned"),
            )
        };
        for kind in [program, Kind::Closed, Kind::Td] {
            assert_eq!(
                is_conditioned(kind),
                !decision,
                "{kind} decision {decision}"
            );
        }
        for kind in [fan_in, laid, Kind::Families { search: true }] {
            assert!(!is_conditioned(kind), "{kind} decision {decision}");
        }
        assert_eq!(is_conditioned(Kind::Decision), decision);
        let (a, b) = (Trial::new(program), Trial::new(fan_in));
        assert_eq!(same_rewrite(&a, &b, &programs), decision);
    }
    // The program with its views in place goes to `program` and
    // `program:fanin`, where the conditioned program does not.
    let written = ground("0.5::f0. 0.5::f1. v :- f0, f1. q :- v. query(q).");
    for decision in [false, true] {
        let programs = Programs {
            ground: as_ground.clone(),
            written: Some(written.clone()),
            conditioned: Some(conditioned.clone()),
            decision,
        };
        let is_written = |kind: Kind| {
            std::ptr::eq(
                programs.of(kind),
                programs.written.as_ref().expect("written"),
            )
        };
        assert_eq!(is_written(program), decision, "decision {decision}");
        assert!(!is_written(fan_in), "decision {decision}");
        for kind in [laid, Kind::Closed, Kind::Td, Kind::Decision] {
            assert!(!is_written(kind), "{kind} decision {decision}");
        }
    }
}

/// A Dice diamond loop of `steps` steps, unrolled as Dice programs were
/// translated before a loop became recursion over a step index: each
/// step drops the signal with probability 1e-4 on one of its two routes,
/// so that the query keeps it with probability (1 - 0.5e-4)^steps.
fn diamond_loop(steps: usize) -> GroundProgram {
    let mut src = String::from(
        "0.5::route.\n0.0001::drop.\nresult :- \\+route, \\+drop.\nresult_1 :- route.\nresult_1 :- result.\n",
    );
    for i in 1..steps {
        let (prev, mid, next) = (2 * i - 1, 2 * i, 2 * i + 1);
        src += &format!(
            "0.5::route_{i}.\ns2_{i} :- route_{i}, result_{prev}.\ns3_{i} :- \\+route_{i}, result_{prev}.\n0.0001::drop_{i}.\n\
             result_{mid} :- s3_{i}, \\+drop_{i}.\nresult_{next} :- s2_{i}.\nresult_{next} :- result_{mid}.\n"
        );
    }
    src += &format!("query(result_{}).\n", 2 * steps - 1);
    crate::ground::ground(
        &crate::syntax::parse_program(&src).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap()
}

#[test]
fn where_program_leads_it_and_the_next_trials_take_turns() {
    // Each step of the diamond loop reads every earlier one through the
    // step before. Without names every layout without indicators compiled
    // it in the square of the steps (24 M units under `program` at 400
    // steps); `program` now names each step's result at a cut of width one
    // (src/vtree/naming.rs), as `families:0` cuts each step at an
    // indicator, and both are linear. `program` leads (the grounder
    // inlines each step's views), `program:fanin` lays out the same named
    // order and is dropped, and `program` and `families:0` take turns of
    // doubling work from 2^15 units (a first watch of 2^15 pairs): one of
    // them finishes (`program`, projected to finish its compile within
    // two doublings, takes its turns ahead), and neither has spent more
    // than the other's work times two to the doublings a projection may
    // move it ahead, and one more for its turn.
    let steps = 400;
    let gp = diamond_loop(steps);
    assert!(gp.stats.inlined > 0);
    assert_eq!(
        candidates(&gp)[..3],
        [
            Kind::Program { fan_in: false },
            Kind::Program { fan_in: true },
            Kind::Families { search: false }
        ]
    );
    let run = run_with_watch(
        gp.clone(),
        candidates(&gp),
        Projection::default(),
        1 << 15,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    let notes = &run.vtree_notes;
    assert!(
        [
            Kind::Program { fan_in: false },
            Kind::Families { search: false }
        ]
        .contains(&run.kind),
        "{notes:#?}"
    );
    assert!(
        notes
            .iter()
            .any(|n| n.starts_with("naming: ") && n.ends_with("widest 1, 399 copies of 399 atoms")),
        "{notes:#?}"
    );
    let work = |t: &str| -> u64 {
        notes
            .iter()
            .filter_map(|n| {
                n.strip_prefix(&format!("trial: {t} indicators="))?
                    .split(", work=")
                    .nth(1)?
                    .split(',')
                    .next()?
                    .parse::<u64>()
                    .ok()
            })
            .sum()
    };
    let turns: Vec<_> = attempts(notes).into_iter().map(|(_, t)| t).collect();
    assert!(
        turns
            .iter()
            .all(|t| t.starts_with("program") || t.starts_with("families:0")),
        "{notes:#?}"
    );
    assert_eq!(work("program:fanin"), 0, "{notes:#?}");
    let ahead = 2u64 << super::policy::PROJECTED_DOUBLINGS as u32;
    assert!(
        work("program") <= ahead * work("families:0") + (1 << 15),
        "{notes:#?}"
    );
    assert!(
        work("families:0") <= ahead * work("program") + (1 << 15),
        "{notes:#?}"
    );
    let p = run.results.unwrap()[0].probability;
    assert!(
        (p - (1.0 - 0.5e-4f64).powi(steps as i32)).abs() < 1e-12,
        "{p}"
    );
}

#[test]
fn a_turn_projected_past_its_horizon_ends_between_components() {
    // At a first watch of 2^4 pairs the first turns are of 2^4 units where
    // `program` leads, and the projection is read from 2^8 units on. The
    // diamond loop's 400 steps are a component each, and `program`'s
    // compile, at the pace of its first components, is projected past its
    // target's three doublings: the turn ends between components, and the
    // trials go on in later turns to the same answer.
    let steps = 400;
    let gp = diamond_loop(steps);
    let run = run_with_watch(
        gp.clone(),
        candidates(&gp),
        Projection::default(),
        1 << 4,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    let notes = &run.vtree_notes;
    let projected: Vec<_> = notes
        .iter()
        .filter(|n| n.starts_with("trial: ") && n.contains(", projected, "))
        .collect();
    assert!(!projected.is_empty(), "{notes:#?}");
    // Each stopped where the work its trial had spent, for the components
    // left, went past its horizon, and after the projection's first read.
    for n in &projected {
        let reach: f64 = n
            .split(", reach 2^")
            .nth(1)
            .and_then(|r| r.split(',').next())
            .and_then(|r| r.parse().ok())
            .unwrap_or_else(|| panic!("{n}"));
        assert!(reach > 8.0 + PROJECTED_DOUBLINGS, "{n}");
    }
    let p = run.results.unwrap()[0].probability;
    assert!(
        (p - (1.0 - 0.5e-4f64).powi(steps as i32)).abs() < 1e-12,
        "{p}"
    );
}

#[test]
fn settling_past_its_budget_is_the_trials_work() {
    // At 120 steps `program`'s first attempt compiles and stops while it
    // settles its names, past the budget, the compile's work: past it each
    // cut still reads its readers before it keeps its names. Uncharged,
    // that work tripped the run's stop before the trial had spent its
    // turn, and the stop read as one by size: the watch doubled and the
    // work did not, and each attempt settled a few cuts more (watches of
    // 2^16 and 2^17 pairs, where no formula holds 2^15). Charged to the
    // trial, the stop is one by work, and no trial's watch moves.
    let steps = 120;
    let gp = diamond_loop(steps);
    let run = run_with_watch(
        gp.clone(),
        candidates(&gp),
        Projection::default(),
        1 << 15,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    let notes = &run.vtree_notes;
    assert!(
        notes
            .iter()
            .any(|n| n.starts_with("trial: program indicators=off,")
                && n.contains(", phase=query, stopped,")),
        "{notes:#?}"
    );
    assert!(
        attempts(notes).iter().all(|&(watch, _)| watch == 1 << 15),
        "{notes:#?}"
    );
    let p = run.results.unwrap()[0].probability;
    assert!(
        (p - (1.0 - 0.5e-4f64).powi(steps as i32)).abs() < 1e-12,
        "{p}"
    );
}

#[test]
fn a_cnf_of_several_units_with_copies_takes_no_families_trial() {
    // Two facts made equal, one a copy of the other that a clause
    // reads, and a clause sharing no fact with the rest, kept as P(e)
    // keeps it.
    let program = crate::syntax::parse_program(include_str!(
        "../../tests/regress/condition_copies_units.pl"
    ))
    .unwrap();
    let gp = crate::ground::ground(
        &program,
        &crate::GroundOptions {
            detach: false,
            ..crate::GroundOptions::default()
        },
    )
    .unwrap();
    assert_eq!(gp.stats.conditioned.merged, 1);
    assert!(!shape::nothing_to_order(&gp) && shape::clauses_over_facts(&gp));
    assert_eq!(candidates(&gp), vec![Kind::Td, Kind::Closed]);
}

/// `closed` with `td` after it, for a program of facts alone, so that
/// `closed` runs within the rounds' bounds: `td` is left out of a
/// program with no rules, and `candidates` puts it first for one, which
/// leaves `closed` alone and run without bounds.
fn closed_then_td() -> Vec<Kind> {
    vec![Kind::Closed, Kind::Td]
}

#[test]
fn a_small_compile_cannot_win_before_its_evidence_fits() {
    let mut source = String::new();
    // Each observed fact queried, so that the evidence is not dropped
    // as detached (src/ground/detach.rs).
    for i in 0..13 {
        source.push_str(&format!("0.5::f({i}).\nquery(f({i})).\n"));
        if i < 12 {
            source.push_str(&format!("evidence(f({i}),true).\n"));
        }
    }
    source.push_str("query(\\+f(12)).\n");
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(&source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    let run = run_with_watch(
        gp.clone(),
        closed_then_td(),
        Projection::default(),
        2,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    assert!(run.attempts > 1);
    assert!(
        run.vtree_notes
            .iter()
            .any(|n| n.contains("phase=query") && !n.contains("finished"))
    );
    assert!(!run.kind.indicators());
    let answers = run.results.unwrap();
    assert_eq!(answers.len(), 14);
    for (i, r) in answers.iter().enumerate() {
        let expected = if i < 12 { 1.0 } else { 0.5 };
        assert!((r.probability - expected).abs() < 1e-12, "{r:?}");
    }
}

#[test]
fn competing_rewrites_preserve_the_problog_posteriors() {
    let source = include_str!("../../examples/bn_split.pl");
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    let expected = [
        ("a(x)", 0.1395817),
        ("a(y)", 0.6352757),
        ("b(z)", 0.34526339),
        ("c(p)", 0.73280394),
        ("c(q)", 0.20266189),
        ("e(t)", 0.39609663),
    ];
    let run = run_with_watch(
        gp.clone(),
        candidates(&gp),
        Projection::default(),
        2,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    assert!(
        run.attempts > 4,
        "the program must exercise multiple bounded trials"
    );
    assert_turns(&run.vtree_notes, &candidates(&gp));
    let answers = run.results.unwrap();
    assert_eq!(answers.len(), expected.len());
    for (name, p) in expected {
        let got = answers
            .iter()
            .find(|r| r.atom.to_string() == name)
            .unwrap()
            .probability;
        assert!((got - p).abs() < 1e-7, "{name}: {got} versus ProbLog {p}");
    }
}

#[test]
fn a_trial_with_indicators_ties_tables_by_clauses() {
    let source = include_str!("../../examples/bn_split.pl");
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    let rewritten: Vec<_> = candidates(&gp)
        .into_iter()
        .filter(|k| k.indicators())
        .collect();
    assert!(!rewritten.is_empty());
    let expected = [
        ("a(x)", 0.1395817),
        ("a(y)", 0.6352757),
        ("b(z)", 0.34526339),
        ("c(p)", 0.73280394),
        ("c(q)", 0.20266189),
        ("e(t)", 0.39609663),
    ];
    let run = run_with_watch(
        gp.clone(),
        rewritten.clone(),
        Projection::default(),
        4,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    assert!(run.kind.indicators());
    // Some reader's link is observed false as clauses.
    let links = run.ground.encoding.links();
    assert!(
        run.ground
            .evidence
            .iter()
            .any(|&(_, a, value)| !value && a.is_some_and(|a| links.contains(&a)))
    );
    let answers = run.results.unwrap();
    for (name, p) in expected {
        let got = answers
            .iter()
            .find(|r| r.atom.to_string() == name)
            .unwrap()
            .probability;
        assert!((got - p).abs() < 1e-7, "{name}: {got} versus ProbLog {p}");
    }
}

/// Trial `strategy` on `gp` written without its linear recursion, in
/// attempts held to the bounds a trial gets beside others, round after
/// round from two pairs. A trial alone runs unbounded, so this is how a
/// test makes one resume.
fn bounded_alone(gp: &GroundProgram, strategy: Kind) -> Run {
    let programs = Programs {
        ground: ground::without_linear_recursion(gp.clone()),
        written: None,
        conditioned: None,
        decision: false,
    };
    let mut trials = vec![Trial::new(strategy)];
    let mut ledger = Ledger::default();
    assert!(prepare(&mut trials, 0, &programs, &mut ledger).unwrap());
    let mut bounds = Bounds {
        pairs: 2,
        work: 2 * WORK_PER_PAIR,
        projected_from: u64::MAX,
        horizon: f64::INFINITY,
    };
    loop {
        match attempt(
            &mut trials[0],
            Some(bounds),
            Projection::default(),
            0,
            None,
            &mut ledger,
        ) {
            Ok((results, sizes)) => return finish(&mut trials[0], results, sizes, ledger),
            Err(e) => match Stop::of(bounds, trials[0].last_work, &e) {
                Some(Stop::Work) => bounds.work *= 2,
                Some(Stop::Size(pairs)) => bounds.pairs = pairs.next_power_of_two(),
                Some(Stop::Projected) | None => panic!("{strategy}: {e:?}"),
            },
        }
    }
}

#[test]
fn a_program_with_nothing_to_order_matches_problog_round_after_round() {
    // One component, its recursion linear: the one trial is dynamic,
    // and it compiles the program written without recursion; so does
    // `closed`. Each is held to the bounds of a trial that shares the
    // rounds ([`bounded_alone`]), from two pairs up, and fails attempt
    // after attempt before it fits. `dynamic`, searching from the
    // first step, resumes each attempt on the vtree the one before it
    // left, and only there are its formulas what they were; `closed`
    // resumes the components the one before finished.
    let source = "0.6::edge(1,2). 0.5::edge(2,3). 0.7::edge(3,1). 0.4::edge(3,4). 0.8::edge(4,5).
        0.3::edge(5,3). 0.6::edge(5,6). 0.5::edge(6,4). 0.4::edge(2,6). 0.3::edge(6,1).
        0.5::edge(4,7). 0.6::edge(7,2). 0.4::edge(7,5). 0.7::edge(1,5).
        path(X,Y) :- edge(X,Y).
        path(X,Y) :- edge(X,Z), path(Z,Y).
        query(path(1,2)). query(path(1,3)). query(path(1,4)). query(path(1,5)). query(path(1,6)). query(path(1,7)).";
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    assert_eq!(candidates(&gp), vec![Kind::Dynamic]);
    // ProbLog 2.3.0.
    let expected = [0.632256, 0.460356, 0.399624, 0.754432, 0.5716992, 0.199812];
    for trial in [Kind::Dynamic, Kind::Closed] {
        let run = bounded_alone(&gp, trial);
        assert!(run.attempts > 1, "{trial}: one attempt");
        assert!(
            run.ground.atoms.iter().any(|a| a.name.starts_with("$e")),
            "{trial}: no label atoms"
        );
        if trial.is_dynamic() {
            let search = run.stats.search.as_ref().unwrap();
            assert!(search.searches > 0, "{trial}: {search:?}");
        }
        let answers = run.results.unwrap();
        assert_eq!(answers.len(), expected.len());
        for (r, p) in answers.iter().zip(expected) {
            assert!(
                (r.probability - p).abs() < 1e-7,
                "{trial}: {r:?} versus ProbLog {p}"
            );
        }
    }
}

#[test]
fn a_trial_alone_runs_in_one_unbounded_attempt() {
    // The program above, whose one trial is dynamic: under a first
    // bound of two pairs it finishes in one attempt, without a watch,
    // with ProbLog's answers.
    let source = "0.6::edge(1,2). 0.5::edge(2,3). 0.7::edge(3,1). 0.4::edge(3,4). 0.8::edge(4,5).
        0.3::edge(5,3). 0.6::edge(5,6). 0.5::edge(6,4). 0.4::edge(2,6). 0.3::edge(6,1).
        0.5::edge(4,7). 0.6::edge(7,2). 0.4::edge(7,5). 0.7::edge(1,5).
        path(X,Y) :- edge(X,Y).
        path(X,Y) :- edge(X,Z), path(Z,Y).
        query(path(1,2)). query(path(1,3)). query(path(1,4)). query(path(1,5)). query(path(1,6)). query(path(1,7)).";
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    let run = run_with_watch(gp.clone(), candidates(&gp), Projection::default(), 2, 0).unwrap();
    assert_eq!(run.attempts, 1, "{:?}", run.vtree_notes);
    assert!(
        run.vtree_notes
            .iter()
            .any(|n| n.starts_with("trial: dynamic") && n.contains("watch=none")),
        "{:?}",
        run.vtree_notes
    );
    // ProbLog 2.3.0.
    let expected = [0.632256, 0.460356, 0.399624, 0.754432, 0.5716992, 0.199812];
    for (r, p) in run.results.unwrap().iter().zip(expected) {
        assert!((r.probability - p).abs() < 1e-7, "{r:?} versus ProbLog {p}");
    }
}

#[test]
fn a_dynamic_trial_compiles_primal_and_the_others_dual() {
    // The programs as written: the chain below, its views inlined, is
    // one rule (`GroundOptions::views`).
    let options = crate::GroundOptions {
        views: false,
        ..crate::GroundOptions::default()
    };
    let run = |source: &str| {
        let gp = crate::ground::ground(&crate::syntax::parse_program(source).unwrap(), &options)
            .unwrap();
        crate::solve_ground(gp, &crate::Options::default()).unwrap()
    };
    // Reachability over a graph with cycles: one component, one trial,
    // dynamic.
    let graph = "0.6::edge(1,2). 0.5::edge(2,3). 0.7::edge(3,1). 0.4::edge(3,4). 0.8::edge(4,5).
        0.3::edge(5,3). 0.6::edge(5,6). 0.5::edge(6,4). 0.4::edge(2,6). 0.3::edge(6,1).
        path(X,Y) :- edge(X,Y).
        path(X,Y) :- edge(X,Z), path(Z,Y).
        query(path(1,3)). query(path(1,5)). query(path(1,6)).";
    // ProbLog 2.3.0.
    let expected = [0.3144, 0.1728, 0.27456];
    let r = run(graph);
    assert!(r.kind.is_dynamic(), "{}", r.kind);
    assert_eq!(r.kind.formulation(), tp::Formulation::Primal);
    for (q, p) in r.results.unwrap().iter().zip(expected) {
        assert!((q.probability - p).abs() < 1e-7, "{q:?} versus ProbLog {p}");
    }
    // A chain of components has an order to search, and its trials
    // compile under the dual.
    let r = run("0.5::a. 0.5::b. 0.5::c. p :- a. q :- p, b. r :- q, c. query(r).");
    assert!(!r.kind.is_dynamic(), "{}", r.kind);
    assert_eq!(r.kind.formulation(), tp::Formulation::Dual);
    assert!((r.results.unwrap()[0].probability - 0.125).abs() < 1e-12);
}

#[test]
fn an_operation_lost_in_flight_is_built_again_in_a_later_turn() {
    // A first turn of 256 units holds a watch of 16 pairs and caps every
    // operation at 32 nodes, so the conjunction of 40 observations stops
    // in flight; the trial resumes in a later turn, under a larger cap,
    // and answers. Each observed fact queried, so that the evidence is
    // not dropped as detached (src/ground/detach.rs).
    let mut source = String::new();
    for i in 0..41 {
        source.push_str(&format!("0.5::f({i}).\nquery(f({i})).\n"));
        if i < 40 {
            source.push_str(&format!("evidence(f({i}),true).\n"));
        }
    }
    source.push_str("query(\\+f(40)).\n");
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(&source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    let run = run_with_watch(
        gp.clone(),
        closed_then_td(),
        Projection::default(),
        1,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    let trials: Vec<&String> = run
        .vtree_notes
        .iter()
        .filter(|n| n.starts_with("trial: "))
        .collect();
    let name = |n: &str| n.split(", watch=").next().unwrap().to_string();
    let lost = trials
        .iter()
        .position(|n| n.contains(", stopped,"))
        .unwrap_or_else(|| panic!("{trials:#?}"));
    assert!(
        trials[lost + 1..]
            .iter()
            .any(|n| name(n) == name(trials[lost])),
        "{trials:#?}"
    );
    let answers = run.results.unwrap();
    assert_eq!(answers.len(), 42);
    for (r, expected) in answers
        .iter()
        .zip([[1.0; 40].as_slice(), &[0.5, 0.5]].concat())
    {
        assert!((r.probability - expected).abs() < 1e-12, "{r:?}");
    }
}

/// A Boolean query of TPC-H q18's shape ([`scripts/gen_tpch.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/gen_tpch.py)): one
/// rule per lineitem of an order of a customer, every body over facts
/// alone, one component; and its probability by hand, the query being
/// hierarchical, read once.
fn lineage_of_orders(customers: usize, orders: usize, lines: usize) -> (GroundProgram, f64) {
    let mut src = String::new();
    let mut k = 0;
    let mut p = || {
        k += 1;
        0.05 + 0.9 * ((k * 37) % 101) as f64 / 101.0
    };
    let mut none = 1.0;
    for c in 0..customers {
        let pc = p();
        src += &format!("{pc}::customer({c}).\n");
        let mut no_order = 1.0;
        for o in 0..orders {
            let id = c * orders + o;
            let po = p();
            src += &format!("{po}::orders({id},{c}).\n");
            let mut no_line = 1.0;
            for l in 0..lines {
                let pl = p();
                src += &format!("{pl}::lineitem({id},{l}).\n");
                no_line *= 1.0 - pl;
            }
            no_order *= 1.0 - po * (1.0 - no_line);
        }
        none *= 1.0 - pc * (1.0 - no_order);
    }
    src += "q :- customer(C), orders(O,C), lineitem(O,_).\nquery(q).\n";
    (regress(&src), 1.0 - none)
}

#[test]
fn a_loop_the_watch_refused_is_taken_in_the_trials_next_attempt() {
    // Under a first watch of 512 pairs (a first turn of 2^13 units)
    // every trial's formula, one loop of clauses over the whole lineage,
    // is refused whole, as TPC-H q18's are. `td`, first, keeps its loop,
    // and its next attempt, under a watch that holds it, takes the loop
    // rather than build it again; `td` finishes.
    let (gp, p) = lineage_of_orders(10, 4, 5);
    let td = Kind::Td;
    assert_eq!(candidates(&gp), vec![td, Kind::Dynamic, Kind::Closed]);
    let run = run_with_watch(
        gp.clone(),
        candidates(&gp),
        Projection::default(),
        32,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    assert_eq!(run.kind, td);
    let trials: Vec<&String> = run
        .vtree_notes
        .iter()
        .filter(|n| n.starts_with("trial: td "))
        .collect();
    assert!(trials.len() >= 2, "{:#?}", run.vtree_notes);
    assert!(trials[0].contains(", components 0/1, size,"), "{trials:#?}");
    assert!(trials[1].contains(", components 1/1, "), "{trials:#?}");
    assert_eq!(run.stats.clause_memo.retaken, 1);
    assert_eq!(run.stats.clause_memo.built, 1, "the loop was built twice");
    let answers = run.results.unwrap();
    assert!(
        (answers[0].probability - p).abs() < 1e-12,
        "{answers:?} against {p}"
    );
}

#[test]
fn a_families_trial_that_spends_past_its_estimate_has_overspent_it() {
    let estimated = |kind, spent| {
        let mut t = Trial::new(kind);
        t.estimate = Some(vtree::Estimate {
            pairs: 100.0,
            search_units: 0,
        });
        t.spent = spent;
        overspent_estimate(&t)
    };
    let laid = Kind::Families { search: false };
    assert!(!estimated(laid, 100 * WORK_PER_ESTIMATED_PAIR));
    assert!(estimated(laid, 100 * WORK_PER_ESTIMATED_PAIR + 1));
    assert!(estimated(
        Kind::Families { search: true },
        100 * WORK_PER_ESTIMATED_PAIR + 1
    ));
    assert!(!estimated(Kind::Closed, 100 * WORK_PER_ESTIMATED_PAIR + 1));
    assert!(!overspent_estimate(&Trial {
        spent: u64::MAX,
        ..Trial::new(laid)
    }));
}

#[test]
fn a_searched_layout_that_would_not_search_is_not_built() {
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(include_str!("../../examples/bn_split.pl")).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    let programs = Programs {
        ground: ground::without_linear_recursion(gp),
        written: None,
        conditioned: None,
        decision: false,
    };
    let mut trials = vec![
        Trial::new(Kind::Families { search: false }),
        Trial::new(Kind::Families { search: true }),
    ];
    let mut ledger = Ledger::default();
    assert!(prepare(&mut trials, 0, &programs, &mut ledger).unwrap());
    assert!(trials[0].estimate.is_some_and(|e| e.search_units == 0));
    let notes = ledger.notes.len();
    assert!(!prepare(&mut trials, 1, &programs, &mut ledger).unwrap());
    assert!(trials[1].disabled && trials[1].built.is_none());
    assert_eq!(ledger.notes.len(), notes);
    // The layout it would have built is `families:0`'s.
    let (searched, _) = vtree::build(
        trials[1].ground.as_ref().unwrap(),
        trials[1].deps.as_deref().unwrap(),
        trials[1].kind,
    )
    .unwrap();
    assert!(same_layout(trials[0].built.as_ref().unwrap(), &searched));
}

#[test]
fn a_grid_answers_round_after_round() {
    // Reachability across a 4 × 4 grid, as tests/it/layouts/grid.rs builds
    // it, under a first round of two pairs. ProbLog 2.3.0 gives
    // 0.13566659, and the frontier program of tests/it/layouts/grid.rs
    // 0.13566659244461016.
    let n = 4;
    let edge = |k: usize, down: bool| ((2 * k + down as usize) * 37 % 89 + 5) as f64 / 100.0;
    let mut source = String::new();
    for i in 0..n {
        for j in 0..n {
            let k = i * n + j;
            if j + 1 < n {
                source += &format!("{}::edge(n{i}_{j},n{i}_{}).\n", edge(k, false), j + 1);
            }
            if i + 1 < n {
                source += &format!("{}::edge(n{i}_{j},n{}_{j}).\n", edge(k, true), i + 1);
            }
        }
    }
    source +=
        "path(X,Y) :- edge(X,Y).\npath(X,Y) :- edge(X,Z), path(Z,Y).\nquery(path(n0_0,n3_3)).\n";
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(&source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    let run = run_with_watch(
        gp.clone(),
        candidates(&gp),
        Projection::default(),
        2,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    assert!(run.attempts > 1);
    assert_turns(&run.vtree_notes, &candidates(&gp));
    let answers = run.results.unwrap();
    assert!(
        (answers[0].probability - 0.13566659244461016).abs() < 1e-12,
        "{:?}",
        answers[0]
    );
}

#[test]
fn a_markov_network_answers_round_after_round() {
    // A Markov network over six binary x and six binary y with a factor
    // on each pair (x_i, y_j), written as the network reader writes one,
    // under a first round of two pairs: `families:0`'s evidence takes
    // 2048.
    let n = 6;
    let mut source = String::new();
    for t in 0..2 * n {
        let v = if t < n {
            format!("x{t}")
        } else {
            format!("y{}", t - n)
        };
        let k = 10 + 8 * ((7 * t + 3) % 11);
        source += &format!("0.{k:02}::{v}(v_0); 0.{:02}::{v}(v_1).\n", 100 - k);
    }
    for i in 0..n {
        for j in 0..n {
            let f = i * n + j;
            let w = 5 + 7 * ((5 * i + 3 * j + 1) % 13);
            for (a, b) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                let weight = if (a == b) == ((i + j) % 2 == 0) {
                    format!("0.{w:02}::")
                } else {
                    String::new()
                };
                source += &format!("{weight}f{f}(ok) :- x{i}(v_{a}), y{j}(v_{b}).\n");
            }
            source += &format!("evidence(f{f}(ok)).\n");
        }
    }
    for j in 0..n {
        source += &format!("query(y{j}(v_1)).\n");
    }
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(&source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    let run = run_with_watch(
        gp.clone(),
        candidates(&gp),
        Projection::default(),
        2,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    assert!(run.attempts > 1);
    assert_turns(&run.vtree_notes, &candidates(&gp));
    // By enumeration of the 4096 worlds; ProbLog 2.3.0 agrees to the
    // eight digits it prints, one query at a time.
    let expected = [
        0.4655495472923504,
        0.5317696857456016,
        0.47636259024250643,
        0.6998651074612225,
        0.45981911579427037,
        0.5368963839653353,
    ];
    let answers = run.results.unwrap();
    assert_eq!(answers.len(), n);
    for (j, (r, p)) in answers.iter().zip(expected).enumerate() {
        assert_eq!(r.atom.to_string(), format!("y{j}(v_1)"));
        assert!((r.probability - p).abs() < 1e-12, "{r:?} against {p}");
    }
}

#[test]
fn the_shared_noisy_or_network_answers_round_after_round() {
    // Under a first round of eight pairs, the trials listed so that `td`
    // does not come first. ProbLog 2.3.0.
    let gp = regress(include_str!("../../tests/regress/noisy_or_shared.pl"));
    let trials = vec![Kind::Dynamic, Kind::Closed, Kind::Td];
    let run = run_with_watch(
        gp,
        trials.clone(),
        Projection::default(),
        8,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    assert!(run.attempts > 1);
    assert_turns(&run.vtree_notes, &trials);
    let noisy_or = [
        0.017102703623876402,
        0.047666462243650835,
        0.4864437251327412,
        0.036335917991856474,
        0.18418349718033797,
        0.3911484812873504,
        0.825511143943469,
        0.030648119100867094,
    ];
    let answers = run.results.unwrap();
    assert_eq!(answers.len(), noisy_or.len());
    for (i, (r, p)) in answers.iter().zip(noisy_or).enumerate() {
        assert_eq!(r.atom.to_string(), format!("disease(d{i})"));
        assert!((r.probability - p).abs() < 1e-8, "{r:?} against {p}");
    }
}

#[test]
fn a_noisy_or_network_from_the_literature_answers_round_after_round() {
    // noisyor15_22 of the programs from the literature under a first
    // round of 256 pairs. [`scripts/lit_exact.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/lit_exact.py); ProbLog 2.3.0 agrees to
    // the eight digits it prints, one query at a time.
    let gp = regress(include_str!("../../tests/regress/noisyor15_22.pl"));
    let run = run_with_watch(
        gp.clone(),
        candidates(&gp),
        Projection::default(),
        256,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    assert!(run.attempts > 1);
    assert_turns(&run.vtree_notes, &candidates(&gp));
    let expected = [
        0.02061970492365235,
        0.9064593852653364,
        0.4826818953335637,
        0.16321971769033045,
        0.22367375497784964,
        0.04255030926874283,
        0.4484375961547964,
        0.01151275859156349,
        0.8716836788951766,
        0.9782458408739202,
        0.7306036455854469,
        0.11661510033660244,
        0.7432533855001254,
        0.00696468820956061,
        0.05062260203705159,
    ];
    let answers = run.results.unwrap();
    assert_eq!(answers.len(), expected.len());
    for (i, (r, p)) in answers.iter().zip(expected).enumerate() {
        assert_eq!(r.atom.to_string(), format!("disease(d{i})"));
        assert!((r.probability - p).abs() < 1e-9, "{r:?} against {p}");
    }
}

#[test]
fn mastermind_answers_round_after_round() {
    // mastermind3_4_3 under a first round of 256 pairs.
    // [`scripts/lit_exact.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/lit_exact.py); ProbLog 2.3.0 gives the same, one query at a
    // time.
    let gp = regress(include_str!("../../tests/regress/mastermind3_4_3.pl"));
    let run = run_with_watch(
        gp.clone(),
        candidates(&gp),
        Projection::default(),
        256,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    assert!(run.attempts > 1);
    assert_turns(&run.vtree_notes, &candidates(&gp));
    let expected = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.5, 0.5, 0.0, 0.5, 0.5, 0.0];
    let answers = run.results.unwrap();
    assert_eq!(answers.len(), expected.len());
    for (r, p) in answers.iter().zip(expected) {
        assert!((r.probability - p).abs() < 1e-9, "{r:?} against {p}");
    }
}

#[test]
fn a_lost_query_operation_is_not_lost_again_under_the_same_bounds() {
    // UW-CSE with four students under a first round of 128 pairs: the
    // compiled `families:0` trial loses evidence conjunctions in flight
    // to the node cap and to the round's work. No attempt repeats a
    // stopped one at the same watch after the same work, which would lose
    // the same operation twice: every round doubles both. ProbLog:
    // 0.094006751.
    let source = include_str!("../../examples/uwcse.pl")
        .replace("student(s2).", "student(s2).\nstudent(s3).\nstudent(s4).");
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(&source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    let run = run_with_watch(
        gp.clone(),
        candidates(&gp),
        Projection::default(),
        128,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    let stopped: Vec<String> = run
        .vtree_notes
        .iter()
        .filter(|n| {
            n.starts_with("trial: ")
                && n.contains(", phase=query, ")
                && n.contains(", stopped, ")
                && !n.contains(", work=0,")
        })
        .map(|n| n.rsplit_once(", ").unwrap().0.to_string())
        .collect();
    for (k, a) in stopped.iter().enumerate() {
        assert!(
            !stopped[..k].contains(a),
            "{a} twice in {:#?}",
            run.vtree_notes
        );
    }
    let answers = run.results.unwrap();
    assert!(
        (answers[0].probability - 0.094006751).abs() < 1e-8,
        "{:?}",
        answers[0]
    );
}

#[test]
fn a_dynamic_trial_resumes_on_the_vtree_it_reached() {
    // The graph above plus `edge(6,5)`, with `path(6,7)` observed
    // false, two components. Searching from the min-fill vtree, an
    // attempt gets past its first step, a round or a component,
    // searches and stops, and the next resumes on the vtree the search
    // reached. A search stopped part way keeps its moves, and the
    // backend must follow them, or the next attempt fails with a vtree
    // mismatch. The extra edge gives the search moves to accept within
    // its budget (with `edge(3,6)` instead it keeps none since the pool
    // search charges its reductions). The answers enumerate the 2^15
    // worlds and are ProbLog's, one query at a time.
    let source = "0.6::edge(1,2). 0.5::edge(2,3). 0.7::edge(3,1). 0.4::edge(3,4). 0.8::edge(4,5).
        0.3::edge(5,3). 0.6::edge(5,6). 0.5::edge(6,4). 0.4::edge(2,6). 0.3::edge(6,1).
        0.5::edge(4,7). 0.6::edge(7,2). 0.4::edge(7,5). 0.7::edge(1,5). 0.5::edge(6,5).
        path(X,Y) :- edge(X,Y).
        path(X,Y) :- edge(X,Z), path(Z,Y).
        evidence(path(6,7), false).
        query(path(1,2)). query(path(1,3)). query(path(1,4)). query(path(1,5)). query(path(1,6)). query(path(1,7)).";
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    let expected = [
        0.598734046294,
        0.438719152721,
        0.308003226941,
        0.763568996546,
        0.567766038516,
        0.031710899199,
    ];
    let run = bounded_alone(&gp, Kind::Dynamic);
    let search = run.stats.search.as_ref().unwrap();
    assert!(search.accepts > 0, "{search:?}");
    let resumed = run.vtree_notes.iter().any(|n| {
        let stopped = n.contains(", size,") || n.contains(", stopped,");
        let past_first = n
            .split("components ")
            .nth(1)
            .is_some_and(|c| !c.starts_with("0/"))
            || n.split(" at round ")
                .nth(1)
                .and_then(|r| r.split(',').next()?.parse::<usize>().ok())
                .is_some_and(|r| r >= 2);
        stopped && (n.contains("phase=query") || past_first)
    });
    assert!(
        resumed,
        "no attempt stopped past its first step: {:?}",
        run.vtree_notes
    );
    let answers = run.results.unwrap();
    assert_eq!(answers.len(), expected.len());
    for (r, p) in answers.iter().zip(expected) {
        assert!((r.probability - p).abs() < 1e-9, "{r:?} versus {p}");
    }
}

#[test]
fn query_counting_obeys_the_batch_work_limit() {
    // The compile counts `q`; the query phase counts the literal of `a`.
    let source = "0.5::a. 0.4::b. q :- a,b. query(q). query(a).";
    let gp = crate::ground::ground(
        &crate::syntax::parse_program(source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap();
    let (
        vtree::Layout {
            vtree: tree, order, ..
        },
        _,
    ) = vtree::build(&gp, &Deps::of(&gp), Kind::Closed).unwrap();
    tree.context().run(|engine| {
        let backend = tp::TddBackend::new(engine, tree.clone());
        let compiled = tp::compile(
            &gp,
            tp::fact_variables(&gp, &order),
            tp::Formulation::Dual,
            &backend,
            Projection::default(),
            None,
        )
        .unwrap();
        let stopped = LimitConfig::none().with_stop_rules(StopRules {
            unconditional: Some(StopAt::WorkUnits(engine.limits().work_units())),
            after_pairs: None,
        });
        {
            let _guard = engine.limits().scope(stopped);
            assert!(matches!(
                query::query_probabilities(&gp, &backend, &compiled),
                Err(tp::CompileError::Tdd(tididi::OperationError::Stopped))
            ));
        }
        let (answers, _) = query::query_probabilities(&gp, &backend, &compiled).unwrap();
        let answers = answers.unwrap();
        assert!((answers[0].probability - 0.2).abs() < 1e-12);
        assert!((answers[1].probability - 0.5).abs() < 1e-12);
    });
}

/// The ground program of a regression program in `tests/regress/`.
fn regress(source: &str) -> GroundProgram {
    crate::ground::ground(
        &crate::syntax::parse_program(source).unwrap(),
        &crate::GroundOptions::default(),
    )
    .unwrap()
}

fn refused(result: &Result<Run, Error>) -> bool {
    matches!(
        result,
        Err(Error::Trials {
            error: tp::CompileError::Tdd(tididi::OperationError::OverBudget),
            ..
        })
    )
}

#[test]
fn a_trial_refused_memory_is_retired_and_the_next_answers() {
    // As gnb30's `families:0` is refused memory in its query phase,
    // where `closed`, the next trial, answers. A byte budget of 6 KiB per
    // operation does the same to examples/query_beside_evidence.pl: `families:0`
    // needs 7 KiB and `closed` 5 KiB (tididi's soft budget, counted in
    // reservations, which a level array the engine recycles does not
    // make).
    let gp = regress(include_str!("../../examples/query_beside_evidence.pl"));
    let laid = Kind::Families { search: false };
    let trials = vec![laid, Kind::Closed];
    let run = run_in_memory(gp.clone(), trials.clone(), 6 << 10).unwrap();
    assert!(
        run.vtree_notes
            .iter()
            .any(|n| n.starts_with("retired: families:0 indicators=all ")),
        "{:#?}",
        run.vtree_notes
    );
    assert_eq!(run.kind, Kind::Closed);
    // ProbLog's answers (tests/it/examples.rs).
    let answers = run.results.unwrap();
    assert!(
        (answers[0].probability - 0.0).abs() < 1e-9 && (answers[1].probability - 0.29).abs() < 1e-9,
        "{answers:?}"
    );
    // Where every trial is refused, the run fails with the refusal
    // rather than raising the bounds of no trial for ever, and with the
    // trials' notes, which `--stats` prints before the error.
    let failed = run_in_memory(gp, trials, 1 << 10);
    assert!(refused(&failed));
    let Err(Error::Trials { notes, .. }) = failed else {
        unreachable!()
    };
    for trial in ["families:0 indicators=all", "closed indicators=off"] {
        assert!(
            notes
                .iter()
                .any(|n| n.starts_with(&format!("trial: {trial},"))),
            "{notes:#?}"
        );
        assert!(
            notes
                .iter()
                .any(|n| n.starts_with(&format!("retired: {trial} "))),
            "{notes:#?}"
        );
    }
}

#[test]
fn a_trial_refused_memory_takes_what_the_others_hold() {
    // `closed` goes first and stops at its first targets, holding what it
    // built; `families:0`, refused memory in its query phase (as above),
    // takes it: `closed`'s formulas are freed rather than `families:0`
    // retired, and `families:0` resumes at its next turn and answers
    // (geni_router_24, where `families` was refused at its last step while
    // four stopped trials held their formulas).
    let gp = regress(include_str!("../../examples/query_beside_evidence.pl"));
    let trials = vec![Kind::Closed, Kind::Families { search: false }];
    let run = run_within(
        Programs {
            ground: gp,
            written: None,
            conditioned: None,
            decision: false,
        },
        trials,
        Projection::default(),
        Watch {
            first_pairs: 1,
            memory_bytes: Some(6 << 10),
            ..Watch::default()
        },
        None,
    )
    .unwrap();
    let notes = &run.vtree_notes;
    assert!(
        notes.iter().any(|n| n.starts_with("released: closed (")
            && n.contains(" for families:0 indicators=all ("))
            && !notes.iter().any(|n| n.starts_with("retired: ")),
        "{notes:#?}"
    );
    assert_eq!(run.kind, Kind::Families { search: false });
    let answers = run.results.unwrap();
    assert!(
        (answers[0].probability - 0.0).abs() < 1e-9 && (answers[1].probability - 0.29).abs() < 1e-9,
        "{answers:?}"
    );
}

#[test]
fn memory_goes_to_the_trial_that_has_spent_the_most_once() {
    // Where no trial fits the budget, the memory others hold is freed for
    // a refused trial only where it has spent more work than each of them,
    // and once: refused again, or behind a trial that has spent as much,
    // it is retired, and the run fails with the refusal. So a refusal
    // discards less work than the refused trial has spent, and the trials
    // cannot free each other for ever. At 3 KiB, `program` and `closed`
    // are freed for `families:0`, retired at its next refusal, and
    // `closed` for `program` (on bb_spacex_w17, freed for `families:0`,
    // which had spent less, `program` did its 2 G units again).
    let gp = regress(include_str!("../../examples/query_beside_evidence.pl"));
    let failed = run_within(
        Programs {
            ground: gp,
            written: None,
            conditioned: None,
            decision: false,
        },
        vec![
            Kind::Program { fan_in: false },
            Kind::Closed,
            Kind::Families { search: false },
        ],
        Projection::default(),
        Watch {
            first_pairs: 1,
            memory_bytes: Some(3 << 10),
            ..Watch::default()
        },
        None,
    );
    assert!(refused(&failed));
    let Err(Error::Trials { notes, .. }) = failed else {
        unreachable!()
    };
    let units = |s: &str| -> Vec<u64> {
        s.split(" units)")
            .filter_map(|p| p.rsplit('(').next()?.parse().ok())
            .collect()
    };
    let released: Vec<&String> = notes
        .iter()
        .filter(|n| n.starts_with("released: "))
        .collect();
    assert!(released.len() >= 2, "{notes:#?}");
    for note in &released {
        let (freed, refused) = note.split_once(" for ").unwrap();
        let (freed, refused) = (units(freed), units(refused));
        assert!(
            !freed.is_empty() && refused.len() == 1 && freed.iter().all(|&f| f < refused[0]),
            "{notes:#?}"
        );
    }
    for trial in [
        "families:0 indicators=all",
        "closed indicators=off",
        "program indicators=off",
    ] {
        let freed = released
            .iter()
            .filter(|n| n.contains(&format!(" for {trial} (")))
            .count();
        assert!(freed <= 1, "{notes:#?}");
        assert!(
            notes
                .iter()
                .any(|n| n.starts_with(&format!("retired: {trial} "))),
            "{notes:#?}"
        );
    }
}

#[test]
fn closed_follows_dynamic_where_nothing_to_order_has_no_recursion() {
    // One component of acyclic atoms merged by the facts they share,
    // as gh20 is. A byte budget per operation that the min-fill vtree
    // outgrows and `closed`'s does not stands in for the host that
    // refused the `dynamic` trial memory: with `dynamic` alone the run
    // fails, and with `closed` after it `closed` answers.
    let growing_head = [("a0", 1.0 - 0.5f64.powi(7))];
    let cases = [
        // Hand-computed (the program's comment); `closed` needs 128 KiB,
        // `dynamic` 1 MiB.
        (
            include_str!("../../tests/regress/growing_head.pl"),
            &growing_head[..],
            1 << 18,
        ),
    ];
    for (source, expected, budget) in cases {
        let gp = regress(source);
        assert_eq!(candidates(&gp), vec![Kind::Dynamic, Kind::Closed, Kind::Td]);
        let alone = run_in_memory(gp.clone(), vec![Kind::Dynamic], budget);
        assert!(refused(&alone), "{:?}", alone.map(|r| r.kind));
        let run = run_in_memory(gp.clone(), candidates(&gp), budget).unwrap();
        assert_eq!(run.kind, Kind::Closed);
        assert!(
            run.vtree_notes
                .iter()
                .any(|n| n.starts_with("retired: dynamic:minfill ")),
            "{:#?}",
            run.vtree_notes
        );
        let answers = run.results.unwrap();
        assert_eq!(answers.len(), expected.len());
        for (r, (name, p)) in answers.iter().zip(expected) {
            assert_eq!(r.atom.to_string(), *name);
            assert!((r.probability - p).abs() < 1e-8, "{r:?} against {p}");
        }
    }
}

#[test]
fn td_comes_first_where_the_rules_read_copies_of_facts() {
    // Clauses observed true, two of which make x(1) and x(2) equal: the
    // conditioning makes one a copy of the other, which a clause of
    // three literals reads. The copy's formula is a literal of a fact,
    // so every rule reads facts alone, and `td` is first and answers.
    // By hand in the program's comment; ProbLog 2.3.0 agrees to the
    // eight digits it prints.
    let gp = regress(include_str!("../../tests/regress/condition_copies.pl"));
    assert_eq!(
        (gp.stats.conditioned.merged, gp.stats.conditioned.into),
        (1, 1)
    );
    assert!(
        gp.rules
            .iter()
            .any(|r| r.body.iter().any(|l| !gp.is_fact(l.atom()))),
        "a rule reads the copy"
    );
    let trials = candidates(&gp);
    assert_eq!(trials, vec![Kind::Td, Kind::Dynamic, Kind::Closed]);
    let run = run_with_watch(
        gp,
        trials,
        Projection::default(),
        FIRST_PAIRS,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    assert_eq!(run.kind, Kind::Td);
    let expected = [
        ("x(1)", 0.162 / 0.33),
        ("x(3)", 0.074 / 0.33),
        ("x(4)", 0.212 / 0.33),
    ];
    let answers = run.results.unwrap();
    assert_eq!(answers.len(), expected.len());
    for (r, (name, p)) in answers.iter().zip(expected) {
        assert_eq!(r.atom.to_string(), name);
        assert!((r.probability - p).abs() < 1e-12, "{r:?} against {p}");
    }
}

#[test]
fn dynamic_answers_a_database_query_from_its_elimination_tree() {
    // A tuple-independent database ([`scripts/gen_pdb.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/gen_pdb.py), domain 40):
    // three acyclic components over the facts r, s and t, nothing to
    // order. Every rule reads facts alone, so `dynamic` starts from the
    // min-fill elimination tree folded bushy and finishes in the first
    // round. `td`, first there, keeps its decomposition's fold too, and
    // finishes in the first round as well. [`scripts/lit_exact.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/lit_exact.py).
    let gp = regress(include_str!("../../tests/regress/pdb40.pl"));
    assert_eq!(candidates(&gp), vec![Kind::Td, Kind::Dynamic, Kind::Closed]);
    let expected = [
        ("h0", 0.389617084615),
        ("h1", 0.795336660308),
        ("safe", 0.656349231681),
    ];
    for (trials, strategy) in [
        (candidates(&gp), Kind::Td),
        (vec![Kind::Dynamic], Kind::Dynamic),
    ] {
        let run = run_with_watch(
            gp.clone(),
            trials,
            Projection::default(),
            FIRST_PAIRS,
            tp::DYNAMIC_FIRST_PAIRS,
        )
        .unwrap();
        assert_eq!(run.kind, strategy);
        assert_eq!(run.attempts, 1);
        let answers = run.results.unwrap();
        assert_eq!(answers.len(), expected.len());
        for (r, (name, p)) in answers.iter().zip(expected) {
            assert_eq!(r.atom.to_string(), name);
            assert!((r.probability - p).abs() < 1e-9, "{r:?} against {p}");
        }
    }
}

#[test]
fn dynamic_answers_a_noisy_or_network() {
    // A noisy-or diagnosis network ([`scripts/gen_noisyor.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/gen_noisyor.py), 20
    // diseases, 30 findings, every finding observed). The findings
    // observed absent tie no causes together in the interaction graph,
    // and `dynamic` reads the facts in a pre-order of the elimination
    // tree, each disease's links beside it, and answers.
    // [`scripts/lit_exact.py`](https://github.com/Tractables/tddlog/blob/baselines/scripts/lit_exact.py). Every rule reads facts alone, and `td` comes
    // first there; `dynamic` alone is the run below.
    let gp = regress(include_str!("../../tests/regress/noisyor20_30.pl"));
    assert_eq!(candidates(&gp)[0], Kind::Td);
    let run = run_with_watch(
        gp,
        vec![Kind::Dynamic],
        Projection::default(),
        FIRST_PAIRS,
        tp::DYNAMIC_FIRST_PAIRS,
    )
    .unwrap();
    assert_eq!(run.kind, Kind::Dynamic);
    let expected = [
        0.021397961040,
        0.983240786654,
        0.403124776690,
        0.114000000000,
        0.120597397565,
        0.007920903326,
        0.087476070496,
        0.101989917348,
        0.016707165150,
        0.013995918503,
        0.047772994887,
        0.068645554783,
        0.058151457812,
        0.000164305970,
        0.055739131943,
        0.586646654509,
        0.041429134770,
        0.993691231689,
        0.997931977411,
        0.006331263015,
    ];
    let answers = run.results.unwrap();
    assert_eq!(answers.len(), expected.len());
    for (i, (r, p)) in answers.iter().zip(expected).enumerate() {
        assert_eq!(r.atom.to_string(), format!("disease(d{i})"));
        assert!((r.probability - p).abs() < 1e-9, "{r:?} against {p}");
    }
    // Under a byte budget of 1 MiB per operation it answers alone.
    // ProbLog 2.3.0.
    let noisy_or = [
        ("disease(d0)", 0.017102703623876402),
        ("disease(d1)", 0.047666462243650835),
        ("disease(d2)", 0.4864437251327412),
        ("disease(d3)", 0.036335917991856474),
        ("disease(d4)", 0.18418349718033797),
        ("disease(d5)", 0.3911484812873504),
        ("disease(d6)", 0.825511143943469),
        ("disease(d7)", 0.030648119100867094),
    ];
    let gp = regress(include_str!("../../tests/regress/noisy_or_shared.pl"));
    let run = run_in_memory(gp, vec![Kind::Dynamic], 1 << 20).unwrap();
    let answers = run.results.unwrap();
    assert_eq!(answers.len(), noisy_or.len());
    for (r, (name, p)) in answers.iter().zip(noisy_or) {
        assert_eq!(r.atom.to_string(), name);
        assert!((r.probability - p).abs() < 1e-8, "{r:?} against {p}");
    }
}

#[test]
fn auto_compiles_an_already_rewritten_program_as_it_is() {
    // `h` read as written, not inlined (`GroundOptions::views`) nor
    // read as a fact (`contexts`). As under one layout
    // (`Kind::rewrite`), every trial compiles the program with its
    // indicators.
    let source = "0.5::a. 0.5::b. 0.5::d. h :- a,b. q :- h,d. query(q).";
    let options = crate::GroundOptions {
        views: false,
        contexts: false,
        ..crate::GroundOptions::default()
    };
    let gp =
        crate::ground::ground(&crate::syntax::parse_program(source).unwrap(), &options).unwrap();
    let gp = crate::indicators::introduce(&gp);
    assert!(!gp.encoding.is_empty());
    let run = run(gp, Projection::default(), None, None, None).unwrap();
    assert!(!run.ground.encoding.is_empty());
    let answers = run.results.unwrap();
    assert!(
        (answers[0].probability - 0.125).abs() < 1e-12,
        "{answers:?}"
    );
}

#[test]
fn a_trial_is_projected_from_its_last_doubling() {
    // A share of the components per doubling of its target, over the last
    // doubling: 0.2 from a target of 2^21 units to 2^22, so the 0.6 left
    // takes three more doublings, to 2^25 units.
    let close = |p: Option<f64>, want: f64| p.is_some_and(|p| (p - want).abs() < 1e-9);
    let mut t = Trial::new(Kind::Closed);
    assert_eq!(t.projected(), None);
    t.progress = vec![(1 << 20, 0.1), (1 << 21, 0.2), (1 << 22, 0.4)];
    assert!(close(t.projected(), 25.0), "{:?}", t.projected());
    // Over two doublings the pace is per doubling.
    t.progress = vec![(1 << 20, 0.2), (3 << 20, 0.3), (1 << 22, 0.6)];
    assert!(close(t.projected(), 24.0), "{:?}", t.projected());
    // A doubling that finished no component projects no finish.
    t.progress = vec![(1 << 20, 0.5), (1 << 21, 0.5)];
    assert_eq!(t.projected(), Some(f64::INFINITY));
    // Nor does a trial that has not doubled its work in its compile.
    t.progress = vec![(3 << 20, 0.5), (1 << 22, 0.6)];
    assert_eq!(t.projected(), None);
    // A trial's place in the turns is its projected finish, from its
    // target to `PROJECTED_DOUBLINGS` past it, for its weight.
    let d = super::policy::PROJECTED_DOUBLINGS;
    t.progress = vec![(1 << 21, 0.5), (1 << 22, 0.9)];
    assert!(close(t.projected(), 22.25), "{:?}", t.projected());
    t.target = 1 << 23;
    assert_eq!(t.ahead(), 0.0);
    assert!(close(Some(t.key()), 23.0));
    t.progress = vec![(1 << 20, 0.1), (1 << 21, 0.2), (1 << 22, 0.4)];
    assert!(close(Some(t.ahead()), 2.0));
    t.weight = 8;
    assert!(close(Some(t.key()), 22.0));
    t.weight = 1;
    t.target = 1 << 25;
    assert_eq!(t.ahead(), 0.0);
    // A doubling that finished no component and a trial that has not
    // doubled its target count the most, whatever its target, and so does
    // a projection further out.
    t.progress = vec![(1 << 20, 0.5), (1 << 21, 0.5)];
    assert_eq!(t.ahead(), d);
    t.progress = vec![(3 << 20, 0.5), (1 << 22, 0.6)];
    assert_eq!(t.ahead(), d);
    t.progress = vec![(1 << 20, 0.01), (1 << 21, 0.02)];
    assert_eq!(t.ahead(), d);
    assert!(close(Some(t.key()), 25.0 + d));
}

#[test]
fn a_query_phase_is_projected_from_where_it_began() {
    // The compile, recorded at targets of 2^20 and 2^21 units, finishes
    // within the attempt at 2^22, whose query phase is a quarter done.
    let close = |p: Option<f64>, want: f64| p.is_some_and(|p| (p - want).abs() < 1e-9);
    let compile_then_query = |query_work: u64| {
        let mut t = Trial::new(Kind::Program { fan_in: false });
        for (target, compiled, progress) in [
            (1 << 20, false, (1, 4)),
            (1 << 21, false, (3, 4)),
            (1 << 22, true, (1, 4)),
        ] {
            t.target = target;
            t.spent = target;
            t.record(compiled, Some(progress));
        }
        t.evaluation.query_work = query_work;
        t
    };
    // Begun at 2^21 units, the phase did a quarter of its steps in a
    // doubling of the work: three doublings more.
    let t = compile_then_query(1 << 21);
    assert_eq!(t.progress, vec![(1 << 22, 0.25)]);
    assert_eq!(t.began(), 1 << 21);
    assert!(close(t.projected(), 25.0), "{:?}", t.projected());
    // Begun at 2^22 - 2^18 units, late in the attempt, it did as much in
    // a sixteenth of the work: projected within the next turn.
    let mut t = compile_then_query(1 << 18);
    let want = 22.0 + 3.0 * (16.0f64 / 15.0).log2();
    assert!(close(t.projected(), want), "{:?}", t.projected());
    t.target = 1 << 23;
    assert_eq!(t.ahead(), 0.0);
    // A phase that has spent nothing is not projected.
    let t = compile_then_query(0);
    assert_eq!(t.projected(), None);
    // Once the phase has a record at half its target, the pace is that of
    // its last doubling, as in a compile.
    let mut t = compile_then_query(1 << 18);
    t.target = 1 << 23;
    t.spent = 1 << 23;
    t.record(true, Some((1, 2)));
    assert!(close(t.projected(), 25.0), "{:?}", t.projected());
    // Where the compile finished in the trial's first attempt, the phase
    // is projected from where it began just the same.
    let mut t = Trial::new(Kind::Program { fan_in: false });
    t.target = 1 << 20;
    t.spent = 1 << 20;
    t.record(true, Some((1, 4)));
    t.evaluation.query_work = 1 << 19;
    assert!(close(t.projected(), 23.0), "{:?}", t.projected());
}
