//! Annotated disjunctions whose heads hold masses under the rounding of
//! the heads before them (src/ground/normalize.rs, `switch_probabilities`;
//! src/ground/normalize.rs, `chain_weights`). A head's switch is true with
//! `p_i / S_i` and false with `S_{i+1} / S_i`, `S_i` the mass of heads `i`
//! on, each weight its own number, so that no head's mass is lost in any
//! order of the heads. Taken as `1 - p_i / S_i`, the weight false of the
//! switch of a head of nearly all of the remaining mass was zero, and the
//! heads after it were lost; so were heads after the remaining mass fell
//! under 1e-12. The answers against hand-computed ones, and on Markov
//! networks written as gen_bn.py writes them, whose priors hold values of
//! mass under 2^-53 that their factors make count, against enumeration.

use crate::common;

use common::Lcg;
use tddlog::Projection;
use tddlog::{Kind, Strategy};

/// `families` lays out indicators, which the Markov networks have and the
/// small programs do not.
const STRATEGIES: [Strategy; 3] = [
    Strategy::Trial(Kind::Closed),
    Strategy::Auto,
    Strategy::Trial(Kind::Td),
];
const WITH_FAMILIES: [Strategy; 4] = [
    Strategy::Trial(Kind::Closed),
    Strategy::Auto,
    Strategy::Trial(Kind::Td),
    Strategy::Trial(Kind::Families { search: false }),
];

/// The answers of `source` under `strategy` and `log10 P(e)`.
fn answers(source: &str, strategy: Strategy) -> (Vec<f64>, f64) {
    let run = common::run(source, &common::options(strategy, Projection::default())).unwrap();
    let results = run
        .results
        .unwrap_or_else(|| panic!("{strategy:?}: the evidence is consistent"));
    let pe = run.evidence_probability.map_or(0.0, |p| p.log10());
    (results.iter().map(|r| r.probability).collect(), pe)
}

fn assert_relative(what: &str, got: f64, want: f64, tolerance: f64) {
    assert!(
        (got - want).abs() <= tolerance * want.abs(),
        "{what}: {got:e} against {want:e}"
    );
}

/// The heads in every order: `0.7::a; 0.3::c; 1e-30::b`, which sum to one
/// in floats, so the distribution is full and divided by that sum. With
/// `a` false, `b` takes `1e-30 / 0.3` of what is left: the old switches
/// took `c` as certain once `a` was false (the remaining mass 0.3 within
/// 1e-9 of `c`'s), and `b` had probability zero. With `a` and `c` false,
/// P(e) is `b`'s mass, where the old switches found the evidence
/// inconsistent.
#[test]
fn a_tail_after_a_head_of_nearly_all_the_mass() {
    let heads = ["0.7::a", "0.3::c", "1.0e-30::b"];
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let ad = order
            .iter()
            .map(|&i| heads[i])
            .collect::<Vec<_>>()
            .join("; ");
        let one = format!("{ad}.\nevidence(a, false).\nquery(b).\nquery(c).\n");
        let both = format!("{ad}.\nevidence(a, false).\nevidence(c, false).\nquery(b).\n");
        for strategy in STRATEGIES {
            let what = format!("{ad} under {strategy:?}");
            let (p, pe) = answers(&one, strategy);
            assert_relative(
                &format!("{what}: P(b | not a)"),
                p[0],
                1e-30 / (0.3 + 1e-30),
                1e-12,
            );
            assert_relative(&format!("{what}: P(c | not a)"), p[1], 1.0, 1e-12);
            assert_relative(&format!("{what}: P(not a)"), pe, 0.3f64.log10(), 1e-12);
            let (p, pe) = answers(&both, strategy);
            assert_relative(&format!("{what}: P(b | not a, not c)"), p[0], 1.0, 1e-12);
            assert!(
                (pe + 30.0).abs() < 1e-12,
                "{what}: log10 P(not a, not c) = {pe} against -30"
            );
        }
    }
}

/// gen_bn.py's prior of a variable whose second value weighs 1e-40 of its
/// first, and a factor that makes it 1e50 times likelier: the tail holds
/// all but 1e-10 of the partition function, `1e-40 + 1e-50`. The old
/// switches dropped it: `x(v_1)` had probability zero and P(e) was 1e-50.
#[test]
fn a_tail_that_a_factor_outweighs() {
    let source = "1.0::x(v_0); 1.0e-40::x(v_1).\n1.0e-50::f(ok) :- x(v_0).\nf(ok) :- x(v_1).\nevidence(f(ok)).\nquery(x(v_1)).\n";
    for strategy in STRATEGIES {
        let (p, pe) = answers(source, strategy);
        assert_relative(
            &format!("{strategy:?}: x(v_1)"),
            p[0],
            1e-40 / (1e-40 + 1e-50),
            1e-12,
        );
        assert_relative(
            &format!("{strategy:?}: log10 P(e)"),
            pe,
            (1e-40f64 + 1e-50).log10(),
            1e-12,
        );
    }
}

/// A disjunction of seven heads whose four tails, of 5e-12 to 5e-11, come
/// after a head of all but 1e-10 of the mass left, and one of three heads
/// under a body with a tail of 1e-11 after a head of nearly all the mass
/// left: the evidence leaves the tails a tenth of the mass. The answers
/// are enumerated by hand in exact arithmetic. The old switches took
/// `c(5)` as certain once `c(1)` and `c(3)` were false and gave `c(1)`
/// probability one and every tail zero. ProbLog 2.3.0 answers as they did:
/// it reads a probability of 1e-10 or less as zero (`1.0e-10::c.
/// query(c).` gives 0), so it cannot check these.
#[test]
fn tails_after_heads_of_nearly_all_the_mass_left() {
    let source = "0.6::c(1); 0.25::c(3); 0.1499999999::c(5); 5.0e-11::c(2); 3.0e-11::c(4); 1.5e-11::c(6); 5.0e-12::c(8).
1.0e-9::t.
0.6::w.
obs :- c(1), t.
obs :- c(2).
obs :- c(4), w.
obs :- c(6).
obs :- c(8), \\+w.
0.7::d(a); 0.29999999999::d(c); 1.0e-11::d(b) :- c(2).
seen :- d(b).
seen :- d(c), w.
evidence(obs).
query(c(1)).
query(c(2)).
query(c(4)).
query(c(6)).
query(c(8)).
query(w).
query(d(b)).
query(seen).
";
    // P(e) = 0.6e-9 + 5e-11 + 0.6 * 3e-11 + 1.5e-11 + 0.4 * 5e-12.
    let pe = 6.85e-10;
    let want = [
        6e-10 / pe,
        5e-11 / pe,
        1.8e-11 / pe,
        1.5e-11 / pe,
        2e-12 / pe,
        4.17e-10 / pe,
        5e-11 / pe * 1e-11,
        5e-11 / pe * (0.29999999999 * 0.6 + 1e-11),
    ];
    for strategy in STRATEGIES {
        let (p, got) = answers(source, strategy);
        for (i, (&p, &w)) in p.iter().zip(&want).enumerate() {
            assert_relative(&format!("{strategy:?}: query {i}"), p, w, 1e-12);
        }
        assert_relative(&format!("{strategy:?}: log10 P(e)"), got, pe.log10(), 1e-12);
    }
}

/// A disjunction of 24 heads of masses 2^-1 to 2^-24, which leave 2^-24
/// to no head, and two of eight heads under a body and its negation,
/// against ProbLog 2.3.0 (`problog -k sdd`, one query at a time, eight
/// digits), which tddlog answered alike before the switches changed.
#[test]
fn a_long_disjunction_against_problog() {
    let heads: Vec<String> = (1..=24)
        .map(|i| format!("{:?}::c({i})", 2f64.powi(-i)))
        .collect();
    let mut source = heads.join("; ") + ".\n0.3::w.\nbig :- c(1).\nbig :- c(2).\nbig :- c(3), w.\n";
    for i in (1..=24).step_by(2) {
        source += &format!("odd :- c({i}).\n");
    }
    source += "0.4::d(1); 0.3::d(2); 0.2::d(3); 0.05::d(4); 0.03::d(5); 0.015::d(6); 0.004::d(7); 0.001::d(8) :- odd.
0.1::d(1); 0.1::d(2); 0.1::d(3); 0.1::d(4); 0.1::d(5); 0.1::d(6); 0.1::d(7); 0.1::d(8) :- \\+odd.
late :- d(7).
late :- d(8).
late :- d(6), w.
evidence(big, false).
evidence(late).
query(c(3)).
query(c(4)).
query(c(20)).
query(c(24)).
query(odd).
query(w).
query(d(8)).
query(d(6)).
";
    let problog = [
        0.02187499,
        0.71874969,
        1.096725e-05,
        6.8545312e-07,
        0.04166663,
        0.3875,
        0.42312501,
        0.13437499,
    ];
    for strategy in STRATEGIES {
        let (p, _) = answers(&source, strategy);
        for (i, (&p, &w)) in p.iter().zip(&problog).enumerate() {
            assert_relative(&format!("{strategy:?}: query {i}"), p, w, 1e-7);
        }
    }
}

/// A head of probability zero has a switch of weight zero and the heads
/// after it keep theirs; a head after the whole mass has none.
#[test]
fn exact_zeros_still_go() {
    let source = "0.4::a; 0.0::b; 0.6::c.\n0.5::d; 0.5::e; 0.0::f.\nquery(a).\nquery(b).\nquery(c).\nquery(f).\n";
    for strategy in STRATEGIES {
        let (p, _) = answers(source, strategy);
        assert_eq!(p[1], 0.0, "{strategy:?}: b");
        assert_eq!(p[3], 0.0, "{strategy:?}: f");
        assert_relative(&format!("{strategy:?}: a"), p[0], 0.4, 1e-15);
        assert_relative(&format!("{strategy:?}: c"), p[2], 0.6, 1e-15);
    }
}

/// A Markov network of `n` binary variables as gen_bn.py writes one: each
/// variable's prior an annotated disjunction over its two values, and
/// each factor an atom `f<k>(ok)`, observed, with a rule per row of its
/// table, the row's entry over the table's largest its probability. Some
/// priors put a mass of 1e-20 to 1e-45 on one value, first or last, and a
/// factor over that variable alone makes the same value up to 1e-30 times
/// likelier than the other, so that the tail holds much of the partition
/// function; the pairwise factors' entries are 1e-6 to 1. Returns the
/// program and, by enumeration, P(e) and each `x<i>(v_1)`'s probability
/// given the evidence.
fn network(n: usize, rng: &mut Lcg) -> (String, f64, Vec<f64>) {
    let mut s = String::new();
    let mut prior = Vec::with_capacity(n);
    for i in 0..n {
        let p = if rng.next() < 0.6 {
            let tail = 10f64.powf(-20.0 - 25.0 * rng.next());
            let t = (rng.next() < 0.5) as usize;
            let mut p = [1.0 / (1.0 + tail); 2];
            p[t] = tail / (1.0 + tail);
            p
        } else {
            let a = 0.05 + 0.9 * rng.next();
            [a, 1.0 - a]
        };
        s += &format!("{:?}::x{i}(v_0); {:?}::x{i}(v_1).\n", p[0], p[1]);
        prior.push(p);
    }
    // Factors: (scope, entries in the order of the scope's values).
    let mut factors: Vec<(Vec<usize>, Vec<f64>)> = Vec::new();
    for (i, p) in prior.iter().enumerate() {
        if p[0].min(p[1]) < 1e-15 {
            let t = (p[1] < p[0]) as usize;
            let mut e = vec![10f64.powf(-30.0 * rng.next()); 2];
            e[t] = 1.0;
            factors.push((vec![i], e));
        }
    }
    for _ in 0..2 * n {
        let a = (rng.next() * n as f64) as usize;
        let b = (rng.next() * n as f64) as usize;
        if a == b {
            continue;
        }
        let mut e: Vec<f64> = (0..4).map(|_| 10f64.powf(-6.0 * rng.next())).collect();
        let top = e.iter().cloned().fold(0.0, f64::max);
        e.iter_mut().for_each(|x| *x /= top);
        factors.push((vec![a, b], e));
    }
    for (k, (scope, e)) in factors.iter().enumerate() {
        for (row, &w) in e.iter().enumerate() {
            let body: Vec<String> = scope
                .iter()
                .enumerate()
                .map(|(j, &v)| format!("x{v}(v_{})", row >> (scope.len() - 1 - j) & 1))
                .collect();
            let head = if w < 1.0 {
                format!("{w:?}::f{k}(ok)")
            } else {
                format!("f{k}(ok)")
            };
            s += &format!("{head} :- {}.\n", body.join(", "));
        }
        s += &format!("evidence(f{k}(ok)).\n");
    }
    for i in 0..n {
        s += &format!("query(x{i}(v_1)).\n");
    }
    let mut z = 0.0;
    let mut one = vec![0.0; n];
    for world in 0..1usize << n {
        let x = |v: usize| world >> v & 1;
        let mut w: f64 = (0..n).map(|v| prior[v][x(v)]).product();
        for (scope, e) in &factors {
            let row = scope.iter().fold(0, |r, &v| r << 1 | x(v));
            w *= e[row];
        }
        z += w;
        for (v, o) in one.iter_mut().enumerate() {
            if x(v) == 1 {
                *o += w;
            }
        }
    }
    (s, z, one.iter().map(|o| o / z).collect())
}

/// Random networks of [`network`]: P(e) to 1e-9 relative and each
/// marginal to 1e-9, under each strategy.
#[test]
fn markov_priors_with_tails_against_enumeration() {
    let mut rng = Lcg(91);
    let mut mattered = 0;
    for case in 0..24 {
        let (source, z, marginals) = network(5 + case % 4, &mut rng);
        // Where a tail value holds a marginal of a tenth or more.
        mattered += usize::from(
            source.lines().filter(|l| l.contains("e-")).count() > 0
                && marginals.iter().any(|&m| m > 0.1 && m < 0.9),
        );
        for strategy in WITH_FAMILIES {
            let what = format!("case {case} under {strategy:?}");
            let (p, pe) = answers(&source, strategy);
            assert!(
                (pe - z.log10()).abs() < 1e-9 / std::f64::consts::LN_10,
                "{what}: log10 P(e) {pe} against {}\n{source}",
                z.log10()
            );
            for (i, (&got, &want)) in p.iter().zip(&marginals).enumerate() {
                assert!(
                    (got - want).abs() < 1e-9,
                    "{what}: x{i}(v_1) {got} against {want}\n{source}"
                );
            }
        }
    }
    assert!(mattered > 6, "networks whose tails count: {mattered} of 24");
}
