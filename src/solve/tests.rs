//! Queries answered in groups ([`split_check`]), each on the part of
//! the program that it and the evidence depend on: the answers, merged in
//! query order, are ProbLog's for the whole program (tests/it/examples.rs),
//! or by hand, and P(e) is the program's.

use super::split::{Split, SplitBounds, query_parts};
use super::{Answering, Options, Progress, Strategy, solve_ground, solve_split, split_check};
use crate::eval::Count;
use crate::ground::{GroundOptions, GroundProgram};
use crate::query::assert_answers;
use crate::{Kind, Projection, QueryResult, auto};

/// A network by hand: a, b given a, c given b (observed), d given a, e
/// given b and d, f given d. e and f head groups, a and b go with the first
/// (the evidence reads them) and d with e's. By enumeration of the 2^6
/// worlds, P(c) = 7/20 and P(a | c) = 12/25, P(b | c) = 4/7, P(d | c) =
/// 68/125, P(e | c) = 4443/8750, P(f | c) = 239/500.
const BY_HAND: &str = "0.3::a.
0.6::b :- a.
0.1::b :- \\+a.
0.8::c :- b.
0.2::c :- \\+b.
0.7::d :- a.
0.4::d :- \\+a.
0.9::e :- b, d.
0.5::e :- b, \\+d.
0.3::e :- \\+b, d.
0.1::e :- \\+b, \\+d.
0.25::f :- d.
0.75::f :- \\+d.
evidence(c).
query(a).
query(b).
query(d).
query(e).
query(f).
";

/// The answers of `source` with its queries split under `bounds`
/// ([`solve_split`]), each part run under `strategy`, in query order; the
/// number of parts; and P(e), asked for where nothing is detached.
fn split_run(
    source: &str,
    bounds: &SplitBounds,
    strategy: Strategy,
    detach: bool,
) -> (usize, Vec<QueryResult>, Option<Count>) {
    let options = Options {
        strategy,
        evidence_probability: !detach,
        ..Options::default()
    };
    let mut parts = 0;
    let solution = solve_split(source, &options, bounds, &mut |p| {
        if let Progress::Answered(_) = p {
            parts += 1;
        }
    })
    .unwrap();
    (
        parts,
        solution.answers.unwrap(),
        solution.evidence_probability,
    )
}

/// [`split_check`]'s parts of `gp` under `bounds` for a run under a
/// layout other than `auto`, which builds nothing ahead; `None` for the
/// program whole.
fn split_of(gp: &GroundProgram, bounds: &SplitBounds) -> Option<Split> {
    match split_check(gp, bounds, Strategy::Trial(Kind::Closed)) {
        Answering::Split(split) => Some(split),
        Answering::Whole(laid) => {
            assert!(laid.is_none());
            None
        }
    }
}

/// A program's name, its text, and its queries' answers.
type Case<'a> = (&'a str, String, Vec<(&'a str, f64)>);

#[test]
fn every_group_apart_gives_the_programs_answers() {
    // The values of tests/it/examples.rs, ProbLog's, and BY_HAND's; the number
    // of groups each splits into.
    // bn_cancer's by hand: P(cancer) = 0.01163, P(xray(positive)) = 0.9 ·
    // 0.01163 + 0.2 · 0.98837, P(dyspnoea(true)) = 0.65 · 0.01163 + 0.3 ·
    // 0.98837.
    let read = |file: &str| std::fs::read_to_string(format!("examples/{file}")).unwrap();
    let cancer = vec![
        ("xray(positive)", 0.208141),
        ("xray(negative)", 0.791859),
        ("dyspnoea(true)", 0.3040705),
        ("dyspnoea(false)", 0.6959295),
    ];
    let cases: Vec<Case> = vec![
        (
            "by hand",
            BY_HAND.to_string(),
            vec![
                ("a", 0.48),
                ("b", 4.0 / 7.0),
                ("d", 0.544),
                ("e", 4443.0 / 8750.0),
                ("f", 0.478),
            ],
        ),
        ("bn_cancer.pl", read("bn_cancer.pl"), cancer),
        (
            "smokers.pl",
            read("smokers.pl"),
            vec![
                ("smokes(p1)", 0.347880),
                ("smokes(p2)", 0.381480),
                ("asthma(p3)", 0.139152),
            ],
        ),
        (
            "alarm.pl",
            read("alarm.pl"),
            vec![
                ("burglary", 0.357143),
                ("earthquake", 0.714286),
                ("calls(mary)", 0.4),
            ],
        ),
        (
            "query_beside_evidence.pl",
            read("query_beside_evidence.pl"),
            vec![("w", 0.0), ("d", 0.29)],
        ),
        (
            "cover.pl",
            read("cover.pl"),
            vec![
                ("u(1)", 0.2),
                ("t(1)", 0.9),
                ("w(1)", 0.145),
                ("y", 0.87),
                ("z", 1.0),
                ("z2", 0.7),
            ],
        ),
    ];
    let mut split = 0;
    for (name, source, expected) in &cases {
        for strategy in [
            Strategy::Auto,
            Strategy::Trial(Kind::Closed),
            Strategy::Trial(Kind::Dynamic),
        ] {
            let (parts, answers, _) = split_run(source, &SplitBounds::EVERY_GROUP, strategy, true);
            assert_answers(
                &format!("{name} in {parts} parts ({strategy:?})"),
                &answers,
                expected,
                1e-6,
            );
            if strategy == Strategy::Auto && parts > 1 {
                split += 1;
            }
        }
    }
    assert!(split >= 3, "the programs split: {split}");
}

#[test]
fn a_part_keeps_every_observation_and_gives_the_programs_evidence_probability() {
    // With nothing detached each part counts all of the evidence: P(c) =
    // 7/20 by hand; ProbLog's 0.196 for alarm.
    for (source, want, parts) in [
        (BY_HAND.to_string(), 0.35, 2),
        (
            std::fs::read_to_string("examples/alarm.pl").unwrap(),
            0.196,
            1,
        ),
    ] {
        let (got_parts, _, p) =
            split_run(&source, &SplitBounds::EVERY_GROUP, Strategy::Auto, false);
        assert_eq!(got_parts, parts);
        let p = p.expect("nothing detached").to_f64();
        assert!((p - want).abs() < 1e-12, "P(e) = {p} against {want}");
    }
}

#[test]
fn a_part_is_what_its_queries_and_the_evidence_depend_on() {
    // BY_HAND's parts: e's reads a, b, c, d and e's facts and not f's; f's
    // reads d, not e's or b's rules but for the evidence's.
    let program = crate::syntax::parse_program(BY_HAND).unwrap();
    let ground = crate::ground::ground(&program, &GroundOptions::default()).unwrap();
    let split = query_parts(&ground, &SplitBounds::EVERY_GROUP, || true).expect("two groups");
    assert_eq!(split.queries, vec![vec![0, 1, 2, 3], vec![4]]);
    for (part, (has, lacks)) in split.parts.iter().zip([("e", "f"), ("f", "e")]) {
        let atoms: Vec<String> = part.atoms.iter().map(|a| a.to_string()).collect();
        assert!(
            atoms.iter().any(|a| a == has) && !atoms.iter().any(|a| a == lacks),
            "{atoms:?}"
        );
    }
    // With the default bounds a program this small is not split, nor one
    // whose queries the evidence reads, nor one with a single query.
    assert!(query_parts(&ground, &SplitBounds::default(), || true).is_none());
    assert!(split_of(&ground, &SplitBounds::default()).is_none());
    for source in [
        "0.5::a.\n0.5::c.\n0.5::b :- a.\n0.5::b :- c.\nevidence(b).\nquery(a).\nquery(c).\n",
        "0.5::a.\n0.5::b.\nquery(a).\n",
    ] {
        let program = crate::syntax::parse_program(source).unwrap();
        let ground = crate::ground::ground(&program, &GroundOptions::default()).unwrap();
        assert!(
            query_parts(&ground, &SplitBounds::EVERY_GROUP, || true).is_none(),
            "{source}"
        );
    }
    // A program read through indicators is not split.
    let indicated = crate::indicators::introduce(&ground);
    if !indicated.encoding.is_empty() {
        assert!(query_parts(&indicated, &SplitBounds::EVERY_GROUP, || true).is_none());
    }
}

#[test]
fn the_layouts_estimates_decide_the_split() {
    // BY_HAND's two parts, which the weights let through under
    // EVERY_GROUP, are split where their layouts' estimates may total any
    // multiple of the program's, and not where they must be smaller than
    // it by more than any estimate can be.
    let program = crate::syntax::parse_program(BY_HAND).unwrap();
    let ground = crate::ground::ground(&program, &GroundOptions::default()).unwrap();
    let bounds = |layout_gain| SplitBounds {
        layout_gain,
        ..SplitBounds::EVERY_GROUP
    };
    let split = split_of(&ground, &bounds(1e-300)).expect("estimated and split");
    assert_eq!(split.parts.len(), 2);
    assert!(
        split.note.contains("layouts estimated at"),
        "{}",
        split.note
    );
    assert!(split_of(&ground, &bounds(1e300)).is_none());
}

#[test]
fn the_further_check_is_asked_of_two_groups_before_the_weights() {
    // BY_HAND's two groups split under EVERY_GROUP where the check lets
    // them, and not where it does not; under the default weight they are
    // too light to split, and the check is asked before that is known. A
    // program with one query has one group, and the check is not asked.
    let parts = |source: &str, bounds: &SplitBounds, worth: bool| {
        let program = crate::syntax::parse_program(source).unwrap();
        let ground = crate::ground::ground(&program, &GroundOptions::default()).unwrap();
        let asked = std::cell::Cell::new(false);
        let split = query_parts(&ground, bounds, || {
            asked.set(true);
            worth
        });
        (split.map(|s| s.parts.len()), asked.get())
    };
    assert_eq!(
        parts(BY_HAND, &SplitBounds::EVERY_GROUP, true),
        (Some(2), true)
    );
    assert_eq!(
        parts(BY_HAND, &SplitBounds::EVERY_GROUP, false),
        (None, true)
    );
    assert_eq!(parts(BY_HAND, &SplitBounds::default(), true), (None, true));
    assert_eq!(
        parts(
            "0.5::a.\n0.5::b.\nquery(a).\n",
            &SplitBounds::EVERY_GROUP,
            true
        ),
        (None, false)
    );
}

#[test]
fn a_part_without_a_families_layout_does_not_refuse_the_split() {
    // x and y read the three values of the annotated disjunction and go
    // together; z reads one fact, its part has no indicators and no
    // `families` layout, and the split goes ahead all the same. The values
    // are of one predicate: those of several, each read once, would be
    // inlined (src/ground/inline.rs).
    let source = "0.2::v(a); 0.3::v(b); 0.5::v(c).\n0.4::z.\nx :- v(a).\ny :- v(b).\nquery(x).\nquery(y).\nquery(z).\n";
    let bounds = SplitBounds {
        layout_gain: 1e-300,
        ..SplitBounds::EVERY_GROUP
    };
    let (parts, answers, _) = split_run(source, &bounds, Strategy::Auto, true);
    assert_eq!(parts, 2);
    let probabilities: Vec<f64> = answers.iter().map(|r| r.probability).collect();
    for (got, want) in probabilities.iter().zip([0.2, 0.3, 0.4]) {
        assert!((got - want).abs() < 1e-12, "{probabilities:?}");
    }
}

#[test]
fn the_check_under_auto_splits_as_under_a_layout_and_runs_as_solve_ground() {
    // Under `auto`, whose first trial on BY_HAND is `families:0`, the check
    // builds that trial first and its floor is the trial's estimate; the
    // split is the one under any other layout for every bounds, and a
    // program kept whole runs with the trial as `solve_ground` runs it: the
    // same answers, P(e), attempts, sizes and notes, but for the seconds.
    let program = crate::syntax::parse_program(BY_HAND).unwrap();
    let ground = crate::ground::ground(
        &program,
        &GroundOptions {
            detach: false,
            ..GroundOptions::default()
        },
    )
    .unwrap();
    let floor_only = SplitBounds {
        min_pairs: 0.0,
        ..SplitBounds::default()
    };
    let estimates = SplitBounds {
        min_pairs: 0.0,
        min_weight: f64::NEG_INFINITY,
        gain: f64::NEG_INFINITY,
        ..SplitBounds::default()
    };
    // The notes with their seconds (every word such as "0.001s;") left out.
    let notes = |run: &crate::Run| -> Vec<String> {
        run.vtree_notes
            .iter()
            .map(|n| {
                n.split(' ')
                    .filter(|w| {
                        !w.trim_end_matches([',', ';'])
                            .strip_suffix('s')
                            .is_some_and(|x| x.contains('.') && x.parse::<f64>().is_ok())
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    };
    let mut laid_runs = 0;
    for bounds in [
        SplitBounds::default(),
        SplitBounds::EVERY_GROUP,
        floor_only,
        estimates,
    ] {
        let plain = split_of(&ground, &bounds).map(|s| s.queries);
        match split_check(&ground, &bounds, Strategy::Auto) {
            Answering::Split(split) => assert_eq!(Some(split.queries), plain, "{bounds:?}"),
            Answering::Whole(laid) => {
                assert_eq!(plain, None, "{bounds:?}");
                let laid = laid
                    .expect("BY_HAND's families:0 trial is auto's first, and the check builds it");
                let with = auto::run(
                    ground.clone(),
                    Projection::default(),
                    Some(laid),
                    None,
                    None,
                )
                .unwrap();
                let without = solve_ground(ground.clone(), &Options::default()).unwrap();
                assert_eq!(with.results, without.results, "{bounds:?}");
                assert_eq!(
                    format!("{:?}", with.evidence_probability),
                    format!("{:?}", without.evidence_probability)
                );
                assert_eq!((with.kind, with.attempts), (without.kind, without.attempts));
                assert_eq!(
                    (with.stats.max_pairs, with.stats.atom_updates),
                    (without.stats.max_pairs, without.stats.atom_updates)
                );
                assert_eq!(with.query_sizes.work_units, without.query_sizes.work_units);
                assert_eq!(notes(&with), notes(&without));
                laid_runs += 1;
            }
        }
    }
    assert_eq!(
        laid_runs, 3,
        "the defaults, the floor alone and the estimates keep BY_HAND whole"
    );
    // Under any other strategy the check builds nothing ahead.
    assert!(matches!(
        split_check(
            &ground,
            &SplitBounds::default(),
            Strategy::Trial(Kind::Closed)
        ),
        Answering::Whole(None)
    ));
}
