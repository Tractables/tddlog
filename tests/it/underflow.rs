//! Counts below the least positive float and above the largest: a count
//! is taken in floats, and again in `eval::Count`s where one of its
//! products underflowed or overflowed (`eval::Range`). Evidence whose probability is under 2^-1074
//! is consistent, and the conditional probabilities are those of ProbLog
//! 2.3, which answers each program below in a few seconds.
use crate::common;

use common::{Rows, assert_probabilities};
use tddlog::Projection;
use tddlog::eval::{Count, Probabilities, Weighing};
use tddlog::ground::GroundOptions;
use tddlog::{Kind, Strategy};

/// `n` facts at 0.5, all observed true, so `P(e) = 2^-n`, and queries that
/// read none of them (`q`), one (`r`, `P = 0.3`) or all (`s`, `q` or `t`,
/// `1 - 0.7 · 0.4 = 0.72`), and one of them (`a5`). `s` reading every
/// observed fact keeps the evidence, which without it no query would
/// depend on and the grounder would drop (src/ground/detach.rs); with no
/// query it is dropped.
fn independent(n: usize, queries: bool) -> String {
    let mut s: String = (0..n).map(|i| format!("0.5::a{i}.\n")).collect();
    s += "0.3::q.\n0.6::t.\nr :- q, a0.\ns :- a0, q.\n";
    s += &(1..n)
        .map(|i| format!("s :- a{i}, t.\n"))
        .collect::<String>();
    s += &(0..n)
        .map(|i| format!("evidence(a{i},true).\n"))
        .collect::<String>();
    if queries {
        s += "query(q).\nquery(r).\nquery(s).\nquery(a5).\n";
    }
    s
}

/// [`independent`] with nothing queried and a cycle observed, `w` true
/// where `a0` is: the grounder leaves the evidence of a cyclic program,
/// which is then counted alone for its mass, `2^-n`. `v` reads two atoms
/// of the cycle, which keeps it from being written without recursion
/// (src/ground/eliminate.rs).
fn cyclic_unqueried(n: usize) -> String {
    independent(n, false) + "w :- a0.\nw :- v.\nv :- w, u.\nu :- w, a1.\nevidence(w,true).\n"
}

#[test]
fn evidence_on_independent_facts_below_the_least_float() {
    // ProbLog: q 0.3, r 0.3, s 0.72, a5 1. P(e) = 2^-1100.
    let source = independent(1100, true);
    for strategy in [Strategy::Trial(Kind::Closed), Strategy::Auto] {
        let run = common::run(&source, &common::options(strategy, Projection::default())).unwrap();
        let what = format!("{strategy:?}");
        // Under `closed` q, r and s are not literals of the evidence
        // formula: they are counted against it (`eval::conjunction_count`);
        // a5 is one. auto's trial holds q in the evidence's scope.
        if strategy != Strategy::Auto {
            assert_eq!(
                run.query_sizes.products, 3,
                "{what}: the queries counted against the evidence"
            );
        }
        assert_probabilities(
            &what,
            &run.results.expect("the evidence is consistent"),
            &[0.3, 0.3, 0.72, 1.0],
            1e-12,
        );
        // Conditioning fixes the 1 100 facts, and P(e) is what it took out.
        let pe = run.evidence_probability.expect("nothing detached").log10();
        assert!(
            (pe + 1100.0 * 2f64.log10()).abs() < 1e-9,
            "{what}: log10 P(e) = {pe}"
        );
    }
}

#[test]
fn evidence_below_the_least_float_that_no_query_reads_is_counted_where_kept() {
    // P(e) = 2^-1100 for both: w holds where a0 does. Ground for the
    // queries alone, observations no query reads may be dropped, and P(e)
    // is then not known.
    for source in [independent(1100, false), cyclic_unqueried(1100)] {
        let program = tddlog::syntax::parse_program(&source).unwrap();
        let kept = GroundOptions {
            detach: false,
            ..GroundOptions::default()
        };
        for (options, known) in [(kept, true), (GroundOptions::default(), false)] {
            let ground = tddlog::ground::ground(&program, &options).unwrap();
            let run = tddlog::solve_ground(
                ground,
                &common::options(Strategy::Trial(Kind::Closed), Projection::default()),
            )
            .unwrap();
            assert_eq!(
                run.results.map(|r| r.len()),
                Some(0),
                "no answers, and no inconsistency"
            );
            match run.evidence_probability {
                Some(pe) => {
                    assert_eq!(
                        run.ground.stats.detached, 0,
                        "P(e) without the dropped observations"
                    );
                    assert!(
                        (pe.log10() + 1100.0 * 2f64.log10()).abs() < 1e-9,
                        "log10 P(e) = {}",
                        pe.log10()
                    );
                }
                None => assert!(
                    !known && run.ground.stats.detached > 0,
                    "P(e) unknown with nothing dropped"
                ),
            }
        }
    }
}

#[test]
fn the_command_line_prints_the_answers() {
    // Evidence below the least float is not inconsistent evidence: the
    // answers are printed.
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("underflow_independent.pl");
    std::fs::write(&path, independent(1200, true)).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_tddlog"))
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        std::str::from_utf8(&out.stdout).unwrap(),
        "q:\t0.300000\nr:\t0.300000\ns:\t0.720000\na5:\t1.000000\n"
    );
}

/// A hidden cause `h` at 0.5 with `na` observed effects at 0.4 under `h`
/// and 0.2 under `\+h`, and `nb` at 0.2 under `h` and 0.4 under `\+h`; `x`
/// is one more effect of the first kind, not observed.
fn hidden(na: usize, nb: usize) -> String {
    let mut s = String::from("0.5::h.\n");
    for i in 0..na {
        s += &format!("0.4::o{i} :- h.\n0.2::o{i} :- \\+h.\nevidence(o{i},true).\n");
    }
    for i in 0..nb {
        s += &format!("0.2::p{i} :- h.\n0.4::p{i} :- \\+h.\nevidence(p{i},true).\n");
    }
    s + "0.4::x :- h.\n0.2::x :- \\+h.\nquery(h).\nquery(x).\n"
}

#[test]
fn a_hidden_cause_observed_many_times_for_and_against() {
    // With n + 1 observations for h and n against, P(e | h) = 0.4^(n+1) ·
    // 0.2^n and P(e | \+h) = 0.2^(n+1) · 0.4^n: P(h | e) = 2/3 and
    // P(x | e) = 2/3 · 0.4 + 1/3 · 0.2 = 1/3, whatever n. ProbLog: h
    // 0.66666667, x 0.33333333 for n = 290, 294, 319 and 2199. P(e) is
    // about 2^-1059 and 2^-1073, subnormal, and 2^-1164 and 2^-8014. In
    // floats alone tddlog answered 0.666660 and 0.333340 for n = 290, 1
    // and 0.5 for n = 294 under minfill, and the evidence inconsistent
    // for the other two.
    //
    // At the level of the closed vtree over the 2 200 observations for h
    // alone, the evidence formula has a node under each value of h, their
    // counts 2^2200 apart, and each carries half the total: no exponent
    // per level holds both, since a float spans 2^2098, and one that kept
    // the larger would answer 1. (A diagnostic over the levels of each
    // run found the span 2 200 bits; 320 for n = 319 under minfill.)
    //
    // In floats, each level and column under a power of two, the count is
    // exact up to n = 319; at n = 2199 it goes on in `Count`s from that
    // level.
    for n in [290, 294, 319, 2199] {
        let run = common::run(
            &hidden(n + 1, n),
            &common::options(Strategy::Trial(Kind::Closed), Projection::default()),
        )
        .unwrap();
        let what = format!("n = {n}");
        // h is a literal of the evidence formula, x is counted against it.
        let q = &run.query_sizes;
        assert_eq!(
            q.products, 1,
            "{what}: the queries counted against the evidence"
        );
        if n < 2199 {
            assert_eq!(
                (q.recounts, q.routes.in_counts()),
                (0, (0, 0)),
                "{what}: in floats"
            );
        } else {
            assert_eq!(q.recounts, 1, "{what}");
            assert!(
                0 < q.routes.in_counts().0 && q.routes.in_counts().0 < q.evidence,
                "{what}: {:?} of {} pairs in counts",
                q.routes.in_counts(),
                q.evidence
            );
        }
        assert_probabilities(
            &what,
            &run.results.expect("the evidence is consistent"),
            &[2.0 / 3.0, 1.0 / 3.0],
            1e-12,
        );
        // P(e) = 0.5 · (0.4 · 0.08^n + 0.2 · 0.08^n) = 0.3 · 0.08^n.
        let pe = run.evidence_probability.expect("nothing detached").log10();
        let want = 0.3f64.log10() + n as f64 * 0.08f64.log10();
        assert!(
            (pe - want).abs() < 1e-9,
            "{what}: log10 P(e) = {pe} against {want}"
        );
    }
}

#[test]
fn a_network_observed_many_times_through_tables_and_indicators() {
    // h has three values; 551 children c_i observed t, P(t) 0.5, 0.25 and
    // 0.5 under h = a, b, c, and 550 children d_i observed t, P(t) 0.25,
    // 0.5 and 0.25. P(e | h) is 2^-1651, 2^-1652 and 2^-1651, so P(h | e)
    // is proportional to 0.2 · 2, 0.5 and 0.3 · 2: 4/15, 1/3 and 2/5.
    // ProbLog: 0.26666667, 0.33333333, 0.4.
    let mut source = String::from("0.2::h(a); 0.5::h(b); 0.3::h(c).\n");
    for (child, n, t) in [("c", 551, [0.5, 0.25, 0.5]), ("d", 550, [0.25, 0.5, 0.25])] {
        for i in 0..n {
            for (value, p) in ["a", "b", "c"].iter().zip(t) {
                source += &format!(
                    "{p}::{child}{i}(t); {}::{child}{i}(f) :- h({value}).\n",
                    1.0 - p
                );
            }
            source += &format!("evidence({child}{i}(t)).\n");
        }
    }
    source += "query(h(a)).\nquery(h(b)).\nquery(h(c)).\n";
    let expected = [4.0 / 15.0, 1.0 / 3.0, 2.0 / 5.0];
    // P(e) = (0.2 · 2 + 0.5 + 0.3 · 2) · 2^-1652 = 1.5 · 2^-1652.
    let pe = 1.5f64.log10() - 1652.0 * 2f64.log10();
    let check_pe = |what: &str, run: &tddlog::Run| {
        let got = run.evidence_probability.expect("nothing detached").log10();
        assert!(
            (got - pe).abs() < 1e-9,
            "{what}: log10 P(e) = {got} against {pe}"
        );
    };
    let read = common::ground(&source, Rows::Classes, true);
    // The children are one unit, which reads h's values through copies.
    assert_eq!(read.encoding.indicators.len(), 3, "h's copies");
    assert!(
        read.evidence
            .iter()
            .any(|(a, _, value)| !value && a.to_string().starts_with("'$c'")),
        "h's table is tied to its readers by clauses"
    );
    for strategy in [
        Strategy::Trial(Kind::Families { search: false }),
        Strategy::Trial(Kind::Closed),
    ] {
        let run = tddlog::solve_ground(
            read.clone(),
            &common::options(strategy, Projection::default()),
        )
        .unwrap();
        check_pe(&format!("indicators, {strategy:?}"), &run);
        assert_probabilities(
            &format!("indicators, {strategy:?}"),
            &run.results.expect("the evidence is consistent"),
            &expected,
            1e-12,
        );
    }
    let plain = common::ground(&source, Rows::Classes, false);
    let run = tddlog::solve_ground(
        plain,
        &common::options(Strategy::Trial(Kind::Closed), Projection::default()),
    )
    .unwrap();
    check_pe("no indicators, closed", &run);
    assert_probabilities(
        "no indicators, closed",
        &run.results.expect("the evidence is consistent"),
        &expected,
        1e-12,
    );
}

#[test]
fn indicator_weights_above_the_largest_float() {
    // An indicator weighs one on each side, two over both. The clause over
    // 1 100 indicators holds in 2^1100 - 1 of their assignments, which a
    // float holds as infinity, and its count with the first true is
    // 2^1099: the ratio is 1/2. A program cannot reach this: its evidence
    // ties every indicator to the facts, which leaves none free.
    let n = 1100u32;
    let vtree = std::sync::Arc::new(tididi::Vtree::balanced(n));
    let f = tididi::Tdd::clause(&vtree, (1..=n as i32).collect::<Vec<_>>()).unwrap();
    let size = n as usize + 1;
    let w = Probabilities {
        pos: vec![1.0; size],
        neg: vec![1.0; size],
        one: vec![2.0; size],
    };
    let half = Count::new(2f64.powi(550));
    let all = (half * half).normalized();
    let counted = tddlog::eval::count::<Count>(&f, &w).unwrap();
    let total = counted.total(&f).count();
    assert_eq!(total.to_f64(), f64::INFINITY);
    assert_eq!(total.ratio(all), 1.0, "2^1100 - 1 against 2^1100");
    let x1 = tididi::vtree::VarId(1);
    let leaf = vtree.leaf_of(x1).unwrap();
    let with_x1 = counted
        .with_literal(&f, leaf, x1, true, &w)
        .unwrap()
        .unwrap();
    assert_eq!(with_x1.ratio(total), 0.5);
    // In floats the root's sums overflow, and are taken again from its
    // children centred on one: the count is exact under a power of two,
    // 2^550, and the same.
    let floats = tddlog::eval::count::<f64>(&f, &w).unwrap();
    assert!(floats.exact(&f));
    assert_eq!(floats.total(&f).exp, 550);
    assert_eq!(floats.total(&f).count().ratio(total), 1.0);
    assert_eq!(
        floats
            .with_literal(&f, leaf, x1, true, &w)
            .unwrap()
            .unwrap()
            .ratio(total),
        0.5
    );
    // The engine's evaluation counts the same.
    assert_eq!(
        f.evaluate(&Weighing::<Count>::new(&w))
            .unwrap()
            .ratio(total),
        1.0
    );
    let plain = Weighing::<f64>::new(&w);
    assert_eq!(f.evaluate(&plain).unwrap(), f64::INFINITY);
    assert!(!plain.exact());
}

#[test]
fn inconsistent_evidence_is_still_reported() {
    // x is a xor b and y is its negation, both observed true: no grounding
    // step decides it, and the count is zero. ProbLog 2.3 prints
    // `InconsistentEvidenceError: Inconsistent evidence detected during
    // compilation.` and exits 1. With the 1 100 observations of
    // `independent` beside it, the evidence is still inconsistent.
    let contradiction = "0.3::b.\n0.4::c.\nx :- b, \\+c.\nx :- \\+b, c.\ny :- b, c.\ny :- \\+b, \\+c.\nevidence(x,true).\nevidence(y,true).\nquery(b).\n";
    for source in [
        contradiction.to_string(),
        independent(1100, true) + contradiction,
    ] {
        for strategy in [Strategy::Trial(Kind::Closed), Strategy::Auto] {
            let run =
                common::run(&source, &common::options(strategy, Projection::default())).unwrap();
            assert!(
                run.results.is_none(),
                "{strategy:?}: the evidence must be reported inconsistent"
            );
        }
    }
}

#[test]
fn floats_count_unless_a_level_spans_more_than_a_float() {
    // An ordinary program is counted in floats alone.
    let alarm =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/alarm.pl")).unwrap();
    let mut too_small = String::from("0.5::e.\nevidence(e,true).\n");
    too_small += &(0..1100)
        .map(|i| format!("0.5::b{i}.\n"))
        .collect::<String>();
    // big reads e, which keeps the evidence (src/ground/detach.rs).
    too_small += &format!(
        "big :- e, {}.\nquery(big).\n",
        (0..1100)
            .map(|i| format!("b{i}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let without_evidence = too_small.replace("evidence(e,true).\n", "");
    let strategy = Strategy::Trial(Kind::Closed);
    let run = common::run(&alarm, &common::options(strategy, Projection::default())).unwrap();
    assert_eq!(
        (run.query_sizes.recounts, run.query_sizes.routes.in_counts()),
        (0, (0, 0)),
        "alarm, {strategy:?}"
    );
    // Evidence under 2^-1074: counted up and down for the queries in
    // floats, its levels and columns under powers of two, and up alone for
    // its mass, likewise; with no query and no cycle, dropped.
    for (what, source, recounts) in [
        ("independent", independent(1100, true), 0),
        ("cyclic, unqueried", cyclic_unqueried(1100), 0),
        ("independent, unqueried", independent(1100, false), 0),
    ] {
        let run = common::run(&source, &common::options(strategy, Projection::default())).unwrap();
        assert_eq!(
            (run.query_sizes.recounts, run.query_sizes.routes.in_counts()),
            (recounts, (0, 0)),
            "{what}, {strategy:?}"
        );
    }
    // Subnormal evidence, in floats under powers of two.
    let run = common::run(
        &hidden(291, 290),
        &common::options(strategy, Projection::default()),
    )
    .unwrap();
    assert_eq!(
        (run.query_sizes.recounts, run.query_sizes.routes.in_counts()),
        (0, (0, 0)),
        "hidden, {strategy:?}"
    );
    // big's formula, of one rule, is stored as itself, the conjunction of
    // e and the b's, 2^-1101: counted against the evidence, in floats
    // under powers of two. Without evidence, its mass by the engine's
    // evaluation, which underflows, and again in `Count`s; its
    // complement, 1 - m, needs no count of its own (`eval::complement`).
    // Stored as its negation, the mass 1 - 2^-1101 was counted twice,
    // and big's own formula then once more. ProbLog: 0.
    for (what, source, recounts) in [
        ("with evidence", &too_small, 0),
        ("without evidence", &without_evidence, 1),
    ] {
        let run = common::run(source, &common::options(strategy, Projection::default())).unwrap();
        assert_eq!(
            run.query_sizes.recounts, recounts,
            "big {what}, {strategy:?}"
        );
        assert_probabilities(
            &format!("big {what}, {strategy:?}"),
            &run.results.unwrap(),
            &[0.0],
            1e-12,
        );
    }
}

/// A cause `h` at 0.5 observed `n` times through effects at 0.1 under `h`
/// and 0.9 under `\+h`, so `P(h | e) = 1 / (1 + 9^n)`; `y` holds where `h`
/// does and `z` where it does not, and each is queried both ways.
fn unlikely_cause(n: usize) -> String {
    let mut s = String::from("0.5::h.\ny :- h.\nz :- \\+h.\n");
    for i in 0..n {
        s += &format!("0.1::o{i} :- h.\n0.9::o{i} :- \\+h.\nevidence(o{i},true).\n");
    }
    s + "query(h).\nquery(\\+h).\nquery(y).\nquery(\\+y).\nquery(z).\nquery(\\+z).\n"
}

#[test]
fn posteriors_far_below_one_keep_their_digits() {
    // An answer taken as one minus the count of the complement kept only
    // the count's absolute error, some 1e-14: a posterior of 2.8e-121
    // came out as 2.6e-14 (digitRecognition) and one of 1.3e-66 as zero.
    // Each answer is counted in the polarity asked, or as the complement's
    // count where one minus a count would be too small to keep its digits.
    for n in [20, 100, 300] {
        let tiny = 1.0 / (1.0 + 9f64.powi(n as i32));
        for strategy in [Strategy::Trial(Kind::Closed), Strategy::Auto] {
            let what = format!("n = {n}, {strategy:?}");
            let run = common::run(
                &unlikely_cause(n),
                &common::options(strategy, Projection::default()),
            )
            .unwrap();
            let results = run.results.expect("the evidence is consistent");
            for (r, want) in
                results
                    .iter()
                    .zip([tiny, 1.0 - tiny, tiny, 1.0 - tiny, 1.0 - tiny, tiny])
            {
                let relative = (r.probability - want).abs() / want;
                assert!(
                    relative < 1e-9,
                    "{what}: {}{} = {:e} against {want:e}",
                    if r.positive { "" } else { "\\+" },
                    r.atom,
                    r.probability
                );
            }
        }
    }
    // Without evidence the compile counts a query's formula where it drops
    // it: a conjunction of 300 facts at 0.1 and a disjunction of 300 at 0.9.
    let mut s: String = (0..300)
        .map(|i| format!("0.1::a{i}.\n0.9::b{i}.\nr :- b{i}.\n"))
        .collect();
    s += &format!(
        "q :- {}.\n",
        (0..300)
            .map(|i| format!("a{i}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    s += "query(q).\nquery(\\+q).\nquery(r).\nquery(\\+r).\n";
    let tiny = 0.1f64.powi(300);
    for strategy in [Strategy::Trial(Kind::Closed), Strategy::Auto] {
        let what = format!("no evidence, {strategy:?}");
        let run = common::run(&s, &common::options(strategy, Projection::default())).unwrap();
        let results = run.results.expect("nothing is observed");
        for (r, want) in results.iter().zip([tiny, 1.0, 1.0, tiny]) {
            let relative = (r.probability - want).abs() / want;
            assert!(
                relative < 1e-9,
                "{what}: {}{} = {:e} against {want:e}",
                if r.positive { "" } else { "\\+" },
                r.atom,
                r.probability
            );
        }
    }
    // The command line writes a probability that would show as zero in
    // scientific notation.
    let path =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("underflow_unlikely_cause.pl");
    std::fs::write(&path, unlikely_cause(100)).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_tddlog"))
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let tiny = format!("{:.6e}", 1.0 / (1.0 + 9f64.powi(100)));
    assert_eq!(
        std::str::from_utf8(&out.stdout).unwrap(),
        format!(
            "h:\t{tiny}\n\\+h:\t1.000000\ny:\t{tiny}\n\\+y:\t1.000000\nz:\t1.000000\n\\+z:\t{tiny}\n"
        )
    );
}
