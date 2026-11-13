//! Queries answered in groups ([`split_check`]), each on the part of
//! the program that it and the evidence depend on: the answers, merged in
//! query order, are ProbLog's for the whole program (tests/it/examples.rs),
//! or by hand, and P(e) is the program's.

use super::split::{Answering, Split, SplitBounds, query_parts, split_check};
use super::windows::WindowBounds;
use super::{Options, Progress, Strategy, solve_ground, solve_split};
use crate::eval::Count;
use crate::ground::{GroundOptions, GroundProgram};
use crate::query::assert_answers;
use crate::{Kind, Projection, QueryResult, auto};
use std::ops::RangeBounds;

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

fn ground(source: &str, detach: bool) -> GroundProgram {
    let program = crate::syntax::parse_program(source).unwrap();
    let options = GroundOptions {
        detach,
        ..GroundOptions::default()
    };
    crate::ground::ground(&program, &options).unwrap()
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
    let gp = ground(BY_HAND, true);
    let split = query_parts(&gp, &SplitBounds::EVERY_GROUP, || true).expect("two groups");
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
    assert!(query_parts(&gp, &SplitBounds::default(), || true).is_none());
    assert!(split_of(&gp, &SplitBounds::default()).is_none());
    for source in [
        "0.5::a.\n0.5::c.\n0.5::b :- a.\n0.5::b :- c.\nevidence(b).\nquery(a).\nquery(c).\n",
        "0.5::a.\n0.5::b.\nquery(a).\n",
    ] {
        assert!(
            query_parts(&ground(source, true), &SplitBounds::EVERY_GROUP, || true).is_none(),
            "{source}"
        );
    }
    // A program read through indicators is not split.
    let indicated = crate::indicators::introduce(&gp).unwrap();
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
    let gp = ground(BY_HAND, true);
    let bounds = |layout_gain| SplitBounds {
        layout_gain,
        ..SplitBounds::EVERY_GROUP
    };
    let split = split_of(&gp, &bounds(1e-300)).expect("estimated and split");
    assert_eq!(split.parts.len(), 2);
    assert!(
        split.note.contains("layouts estimated at"),
        "{}",
        split.note
    );
    assert!(split_of(&gp, &bounds(1e300)).is_none());
}

#[test]
fn the_further_check_is_asked_of_two_groups_before_the_weights() {
    // BY_HAND's two groups split under EVERY_GROUP where the check lets
    // them, and not where it does not; under the default weight they are
    // too light to split, and the check is asked before that is known. A
    // program with one query has one group, and the check is not asked.
    let parts = |source: &str, bounds: &SplitBounds, worth: bool| {
        let gp = ground(source, true);
        let asked = std::cell::Cell::new(false);
        let split = query_parts(&gp, bounds, || {
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
    let ground = ground(BY_HAND, false);
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
                let ready = auto::Ready {
                    laid: Some(laid),
                    ..auto::Ready::default()
                };
                let with = auto::run(ground.clone(), Projection::default(), ready, false).unwrap();
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

/// `source` cut into windows of `rules` ground rules or more
/// ([`super::windows`]) under `options`: the runs answered (the windows,
/// and the whole program where it was run whole after them), the notes and
/// the solution.
fn run_windowed(
    source: &str,
    rules: usize,
    options: &Options,
) -> (usize, Vec<String>, super::Solution) {
    let bounds = SplitBounds {
        windows: WindowBounds {
            rules,
            ..WindowBounds::DEFAULT
        },
        ..SplitBounds::default()
    };
    let (mut runs, mut notes) = (0, Vec::new());
    let solution = solve_split(source, options, &bounds, &mut |p| match p {
        Progress::Answered(_) => runs += 1,
        Progress::Note(note) => notes.push(note),
        _ => {}
    })
    .unwrap();
    (runs, notes, solution)
}

/// The answers of `source` cut into windows of `rules` ground rules or
/// more under `strategy`, the number of windows, and P(e), asked for.
fn windowed(source: &str, rules: usize, strategy: Strategy) -> (usize, Vec<QueryResult>, Count) {
    let options = Options {
        strategy,
        evidence_probability: true,
        ..Options::default()
    };
    let (windows, _, solution) = run_windowed(source, rules, &options);
    (
        windows,
        solution.answers.unwrap(),
        solution.evidence_probability.unwrap(),
    )
}

/// [`windowed`] under the default options: P(e) not asked for, so the
/// observations no query depends on are dropped ([`GroundOptions::detach`]),
/// and a lone query without evidence observed (`observed_query`). The
/// number of windows and the answers.
fn windowed_default(
    source: &str,
    rules: usize,
    strategy: Strategy,
) -> (usize, Option<Vec<QueryResult>>) {
    let options = Options {
        strategy,
        ..Options::default()
    };
    let (windows, _, solution) = run_windowed(source, rules, &options);
    (windows, solution.answers)
}

/// `source`'s answers under the default options cut into windows of
/// `rules` rules, under `auto` and `closed`, against the whole program's
/// to 1e-9 and against `expected`, computed by hand; and the windows, a
/// number in `windows`.
fn windows_agree_by_default(
    source: &str,
    rules: usize,
    expected: &[(&str, f64)],
    windows: impl RangeBounds<usize>,
) {
    for strategy in [Strategy::Auto, Strategy::Trial(Kind::Closed)] {
        let (one, whole) = windowed_default(source, usize::MAX, strategy);
        let (count, cut) = windowed_default(source, rules, strategy);
        assert_eq!(one, 1, "{strategy}: the whole program is one window");
        assert!(windows.contains(&count), "{strategy}: {count} windows");
        let what = format!("{strategy}, {count} windows");
        let (whole, cut) = (whole.unwrap(), cut.unwrap());
        let at_whole: Vec<(String, f64)> = whole
            .iter()
            .map(|r| (r.to_string(), r.probability))
            .collect();
        let at_whole: Vec<(&str, f64)> = at_whole.iter().map(|(a, p)| (a.as_str(), *p)).collect();
        assert_answers(&what, &cut, &at_whole, 1e-9);
        assert_answers(&what, &cut, expected, 1e-9);
    }
}

/// `source`'s answers and P(e) cut into windows of `rules` rules, under
/// `auto` and `closed`, against the whole program's to 1e-9 and against
/// `expected`, computed by hand; and the windows, more than two.
fn windows_agree(source: &str, rules: usize, expected: &[(&str, f64)], evidence: f64) {
    windows_agree_in(source, rules, expected, evidence, 3..);
}

/// [`windows_agree`] with the number of windows in `windows`.
fn windows_agree_in(
    source: &str,
    rules: usize,
    expected: &[(&str, f64)],
    evidence: f64,
    windows: impl RangeBounds<usize>,
) {
    for strategy in [Strategy::Auto, Strategy::Trial(Kind::Closed)] {
        let (one, whole, z) = windowed(source, usize::MAX, strategy);
        let (count, cut, z_cut) = windowed(source, rules, strategy);
        assert_eq!(one, 1, "{strategy}: the whole program is one window");
        assert!(windows.contains(&count), "{strategy}: {count} windows");
        let what = format!("{strategy}, {count} windows");
        let at_whole: Vec<(String, f64)> = whole
            .iter()
            .map(|r| (r.to_string(), r.probability))
            .collect();
        let at_whole: Vec<(&str, f64)> = at_whole.iter().map(|(a, p)| (a.as_str(), *p)).collect();
        assert_answers(&what, &cut, &at_whole, 1e-9);
        assert_answers(&what, &cut, expected, 1e-9);
        for z in [z, z_cut] {
            assert!(
                (z.to_f64() - evidence).abs() < 1e-9,
                "{what}: P(e) {z:?} against {evidence}"
            );
        }
    }
}

/// A Markov chain of `n` steps, `s(0)` with probability 0.9 and each step
/// keeping `s` with 0.95 and making it with 0.05, written out; then
/// `rest`. P(s(i)) = 0.5 + 0.4 * 0.9^i, and P(s(j) | s(i)) = 0.5 + 0.5 *
/// 0.9^(j - i) for i < j.
fn chain(n: usize, rest: &str) -> String {
    let mut s = String::from("0.9::s(0).\n");
    for i in 1..=n {
        s += &format!(
            "0.95::s({i}) :- s({}).\n0.05::s({i}) :- \\+s({}).\n",
            i - 1,
            i - 1
        );
    }
    s + rest
}

fn marginal(i: usize) -> f64 {
    0.5 + 0.4 * 0.9f64.powi(i as i32)
}

fn given(i: usize, j: usize) -> f64 {
    0.5 + 0.5 * 0.9f64.powi((j - i) as i32)
}

#[test]
fn a_chain_cut_into_windows_gives_the_programs_answers() {
    let n = 60;
    windows_agree(
        &chain(n, "query(s(60)).\nquery(\\+s(30)).\n"),
        12,
        &[("s(60)", marginal(60)), ("\\+s(30)", 1.0 - marginal(30))],
        1.0,
    );
}

#[test]
fn evidence_in_the_middle_is_passed_on_by_the_windows() {
    // Given s(30), s(60) is a chain of 30 steps from a certain start.
    windows_agree(
        &chain(60, "evidence(s(30)).\nquery(s(60)).\nquery(s(45)).\n"),
        12,
        &[("s(60)", given(30, 60)), ("s(45)", given(30, 45))],
        marginal(30),
    );
}

#[test]
fn a_query_before_later_evidence_is_carried_to_it() {
    // P(s(15) | s(60)) = P(s(15)) P(s(60) | s(15)) / P(s(60)).
    let (q, e) = (15, 60);
    windows_agree(
        &chain(e, "query(s(15)).\nevidence(s(60)).\n"),
        12,
        &[("s(15)", marginal(q) * given(q, e) / marginal(e))],
        marginal(e),
    );
}

/// A ladder of `n` steps written as a loop: a(i) and b(i) each read a(i - 1)
/// and b(i - 1), through switches of their own, so that given the step
/// before they are independent; then `rest`.
fn ladder(n: usize, rest: &str) -> String {
    format!(
        "steps({n}).
step(1).
step(T1) :- step(T), steps(N), T1 is T + 1, T1 =< N.
0.6::a(0).
0.3::b(0).
0.7::a(T) :- step(T), T0 is T - 1, a(T0), b(T0).
0.4::a(T) :- step(T), T0 is T - 1, \\+a(T0).
0.5::b(T) :- step(T), T0 is T - 1, a(T0).
0.2::b(T) :- step(T), T0 is T - 1, \\+b(T0).
{rest}"
    )
}

#[test]
fn a_ladder_passes_the_joint_of_two_atoms() {
    // The joint of a(i) and b(i) is carried forward over the four states.
    let n = 40;
    let source = ladder(
        n,
        "evidence(b(20)).\nquery(a(40)).\nquery(b(40)).\nquery(a(10)).\n",
    );
    // P(a' | a, b) and P(b' | a, b): one minus the product of the rules
    // that hold failing.
    let next = |a: bool, b: bool| {
        let pa = 1.0 - (1.0 - if a && b { 0.7 } else { 0.0 }) * (1.0 - if a { 0.0 } else { 0.4 });
        let pb = 1.0 - (1.0 - if a { 0.5 } else { 0.0 }) * (1.0 - if b { 0.0 } else { 0.2 });
        (pa, pb)
    };
    // The joint over (a, b), state 2a + b, forward from step `from` to
    // step `to`.
    let forward = |mut joint: [f64; 4], from: usize, to: usize| {
        for _ in from..to {
            let mut out = [0.0; 4];
            for (s, &p) in joint.iter().enumerate() {
                let (pa, pb) = next(s >= 2, s % 2 == 1);
                for (t, o) in out.iter_mut().enumerate() {
                    let fa = if t >= 2 { pa } else { 1.0 - pa };
                    let fb = if t % 2 == 1 { pb } else { 1.0 - pb };
                    *o += p * fa * fb;
                }
            }
            joint = out;
        }
        joint
    };
    let start = [0.4 * 0.7, 0.4 * 0.3, 0.6 * 0.7, 0.6 * 0.3];
    let at20 = forward(start, 0, 20);
    let evidence = at20[1] + at20[3];
    let observed = [0.0, at20[1] / evidence, 0.0, at20[3] / evidence];
    let end = forward(observed, 20, n);
    // P(a(10) | b(20)): forward to 10, split on a(10), forward each to 20.
    let at10 = forward(start, 0, 10);
    let b20 = |j: [f64; 4]| {
        let f = forward(j, 10, 20);
        f[1] + f[3]
    };
    let a10 = b20([0.0, 0.0, at10[2], at10[3]]) / evidence;
    windows_agree(
        &source,
        24,
        &[
            ("a(40)", end[2] + end[3]),
            ("b(40)", end[1] + end[3]),
            ("a(10)", a10),
        ],
        evidence,
    );
}

/// P(s(j) | \+s(i)) in [`chain`], for i < j.
fn given_not(i: usize, j: usize) -> f64 {
    0.5 - 0.5 * 0.9f64.powi((j - i) as i32)
}

#[test]
fn a_lone_query_observed_keeps_what_a_window_before_drops() {
    // The lone query is observed and its answer is P(e): no window may
    // drop an observation, whose probability is a factor of it. q holds
    // with s(60), c and one of a and b: 0.8 * 0.75.
    windows_agree_by_default(
        &chain(
            60,
            "0.5::a.\n0.5::b.\n0.8::c.\nq :- s(60), a, c.\nq :- s(60), b, c.\nquery(q).\n",
        ),
        12,
        &[("q", 0.6 * marginal(60))],
        2..,
    );
}

#[test]
fn a_lone_query_observed_without_a_window_dropping() {
    windows_agree_by_default(
        &chain(60, "0.8::c.\nq :- s(60), c.\nquery(q).\n"),
        12,
        &[("q", 0.8 * marginal(60))],
        2..,
    );
}

#[test]
fn a_message_row_below_the_least_float_runs_the_program_whole() {
    // A hidden cause h = u, v at 0.25, with 2 200 observed effects at 0.4
    // under h and 0.2 under \+h, and 2 199 at 0.2 under h and 0.4 under
    // \+h: P(e | h) = 2 P(e | \+h), P(h | e) = 0.5 / 1.25 = 0.4, and x, one
    // more effect of the first kind, 0.4 * 0.4 + 0.6 * 0.2 = 0.28. P(e) =
    // 0.25 * 0.08^2199. A cut after the first thousand observations
    // leaves P(\+h | e so far) below the least float, not zero.
    let mut source = String::from("0.5::u.\n0.5::v.\nh :- u, v.\n");
    for i in 0..2200 {
        source += &format!("0.4::o{i} :- h.\n0.2::o{i} :- \\+h.\nevidence(o{i},true).\n");
    }
    for i in 0..2199 {
        source += &format!("0.2::p{i} :- h.\n0.4::p{i} :- \\+h.\nevidence(p{i},true).\n");
    }
    source += "0.4::x :- h.\n0.2::x :- \\+h.\nquery(h).\nquery(x).\n";
    let log10_e = 0.25f64.log10() + 2199.0 * 0.08f64.log10();
    for strategy in [Strategy::Auto, Strategy::Trial(Kind::Closed)] {
        let options = Options {
            strategy,
            evidence_probability: true,
            ..Options::default()
        };
        let (_, _, whole) = run_windowed(&source, usize::MAX, &options);
        let (runs, notes, cut) = run_windowed(&source, 3000, &options);
        let what = format!("{strategy}, {runs} runs");
        assert!(
            notes.iter().any(|n| n.starts_with("windows: ")),
            "{what}: cut"
        );
        let expected = [("h", 0.4), ("x", 0.28)];
        for solution in [&whole, &cut] {
            assert_answers(&what, solution.answers.as_ref().unwrap(), &expected, 1e-9);
            let z = solution.evidence_probability.unwrap().log10();
            assert!(
                (z - log10_e).abs() < 1e-9 * log10_e.abs(),
                "{what}: log10 P(e) {z} against {log10_e}"
            );
        }
        // The first window's row \+h is the one under the least float.
        assert!(
            notes
                .iter()
                .any(|n| n.contains("after window 1 may") && n.contains("run whole")),
            "{what}: {notes:?}"
        );
    }
}

#[test]
fn a_helper_of_each_step_read_by_the_next() {
    // h(i) = c(i) or d(i), 0.75; s(i) after s(i - 1) 0.75, after \+s(i - 1)
    // 0.15: s(i) = 0.375 + 0.525 * 0.6^i. Every h(i) ranks before s(2)
    // and crosses the boundaries before it: the program is cut only near
    // its end.
    let mut source = String::from("0.9::s(0).\n");
    for i in 1..=60 {
        source += &format!(
            "0.5::c({i}).\n0.5::d({i}).\n0.2::k({i}).\nh({i}) :- c({i}).\nh({i}) :- d({i}).\ns({i}) :- s({j}), h({i}).\ns({i}) :- \\+s({j}), h({i}), k({i}).\n",
            j = i - 1
        );
    }
    source += "query(s(7)).\nquery(s(60)).\n";
    let s = |i: i32| 0.375 + 0.525 * 0.6f64.powi(i);
    windows_agree_in(&source, 12, &[("s(7)", s(7)), ("s(60)", s(60))], 1.0, 2..);
}

#[test]
fn a_choice_across_cuts_with_later_evidence() {
    // m1 makes s(10) true, m2 s(50); s(60) observed.
    let z = 0.4 * given(10, 60) + 0.6 * given(50, 60);
    windows_agree(
        &chain(
            60,
            "0.4::m1; 0.6::m2.\ns(10) :- m1.\ns(50) :- m2.\nevidence(s(60)).\nquery(m1).\nquery(s(30)).\n",
        ),
        12,
        &[
            ("m1", 0.4 * given(10, 60) / z),
            (
                "s(30)",
                (0.4 * given(10, 30) * given(30, 60) + 0.6 * marginal(30) * given(50, 60)) / z,
            ),
        ],
        z,
    );
}

#[test]
fn a_cycle_in_each_step_is_not_cut() {
    // u(i) and v(i) read each other: u(i) = s(i - 1) and (f(i) or h(i)),
    // 0.95, so s(i) is the chain's step.
    let mut source = String::from("0.9::s(0).\n");
    for i in 1..=60 {
        source += &format!(
            "0.9::f({i}).\n0.5::g({i}).\n0.5::h({i}).\n0.05::k({i}).\nu({i}) :- s({j}), f({i}).\nu({i}) :- v({i}).\nv({i}) :- u({i}), g({i}).\nv({i}) :- s({j}), h({i}).\ns({i}) :- u({i}).\ns({i}) :- \\+s({j}), k({i}).\n",
            j = i - 1
        );
    }
    source += "evidence(s(30)).\nquery(s(60)).\nquery(s(45)).\n";
    windows_agree(
        &source,
        24,
        &[("s(60)", given(30, 60)), ("s(45)", given(30, 45))],
        marginal(30),
    );
}

#[test]
fn evidence_true_and_false_with_queries_on_both_sides() {
    let a = given(10, 30) * (1.0 - given(30, 50));
    let b = (1.0 - given(10, 30)) * (1.0 - given_not(30, 50));
    windows_agree(
        &chain(
            60,
            "evidence(s(10)).\nevidence(s(50),false).\nquery(s(5)).\nquery(s(30)).\nquery(s(60)).\nquery(s(10)).\n",
        ),
        12,
        &[
            ("s(5)", marginal(5) * given(5, 10) / marginal(10)),
            ("s(30)", a / (a + b)),
            ("s(60)", given_not(50, 60)),
            ("s(10)", 1.0),
        ],
        marginal(10) * (1.0 - given(10, 50)),
    );
}

#[test]
fn atoms_named_as_the_windows_name_theirs() {
    windows_agree(
        &chain(
            60,
            "0.5::'$wq'(0,0).\n0.5::'$wr'(1,0).\n0.5::'$ws'(1,0).\ny :- s(2), '$wq'(0,0).\nz :- s(60), '$wr'(1,0), '$ws'(1,0).\nquery(y).\nquery('$wq'(0,0)).\nquery(z).\n",
        ),
        12,
        &[
            ("y", 0.5 * marginal(2)),
            ("'$wq'(0,0)", 0.5),
            ("z", 0.25 * marginal(60)),
        ],
        1.0,
    );
}

#[test]
fn inconsistent_evidence_in_two_windows() {
    let source = chain(
        60,
        "0.5::a.\ne1 :- s(20), a.\ne2 :- s(40), \\+a.\nevidence(e1).\nevidence(e2).\nquery(s(60)).\n",
    );
    for strategy in [Strategy::Auto, Strategy::Trial(Kind::Closed)] {
        for rules in [usize::MAX, 12] {
            let (count, answers) = windowed_default(&source, rules, strategy);
            assert!(answers.is_none(), "{strategy}, {count} windows");
            assert_eq!(count > 1, rules == 12, "{strategy}, {count} windows");
        }
    }
}

#[test]
fn queries_held_to_late_evidence_wider_than_a_cut() {
    // Six queries held to s(60) cross every boundary: no cut is narrow.
    let mut source = chain(60, "evidence(s(60)).\n");
    let mut expected = Vec::new();
    for i in 1..=6 {
        source += &format!("query(s({i})).\n");
        expected.push((format!("s({i})"), marginal(i) * given(i, 60) / marginal(60)));
    }
    let expected: Vec<(&str, f64)> = expected.iter().map(|(a, p)| (a.as_str(), *p)).collect();
    windows_agree_in(&source, 12, &expected, marginal(60), 1..=1);
}
