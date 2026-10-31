//! `--stats` counts every negation by what it was for
//! (`tddlog::tp::NegationSource`), in the compile and in the query phase.
use crate::common;

use common::Rows;
use tddlog::tp::{Formulation, NegationSource};
use tddlog::{Kind, Strategy};

/// `source` under `closed`, which compiles under the dual, or `dynamic`,
/// which compiles under the primal and, on programs this small, never
/// searches.
fn compile(source: &str, strategy: Strategy) -> tddlog::Run {
    let gp = common::ground(source, Rows::Classes, false);
    tddlog::solve_ground(gp, &common::options(strategy, tddlog::Projection::Off)).unwrap()
}

const BOTH: [Strategy; 2] = [
    Strategy::Trial(Kind::Closed),
    Strategy::Trial(Kind::Dynamic),
];

/// The calls of each source that negated anything, compile and query phase.
fn calls(run: &tddlog::Run) -> Vec<(NegationSource, usize)> {
    let of =
        |source| run.stats.negations.of(source).calls + run.query_sizes.negations.of(source).calls;
    NegationSource::ALL
        .iter()
        .map(|&s| (s, of(s)))
        .filter(|&(_, c)| c > 0)
        .collect()
}

#[test]
fn negations_are_counted_by_what_they_were_for() {
    // The dual stores μ = ¬λ, but λ for an atom of one rule, its body's
    // conjunction. p is read positively twice, through one cached λ_p,
    // and negatively once, as its μ_p; q, s and r are conjunctions, and q
    // observed false is its λ_q negated. The primal negates p's
    // ⋀_r ¬body_r and \+p, and q observed false is its formula negated.
    // Observed true, q would fix p and c (src/ground/condition.rs).
    let source = "0.3::a. 0.4::b. 0.5::c.
        p :- a.
        p :- b.
        q :- p, c.
        s :- p, b.
        r :- \\+p, c.
        evidence(q, false).
        query(r). query(s).";
    use NegationSource::*;
    for (strategy, expected) in [
        (
            Strategy::Trial(Kind::Closed),
            vec![(Lambda, 1), (Evidence, 1)],
        ),
        (
            Strategy::Trial(Kind::Dynamic),
            vec![(Primal, 2), (Evidence, 1)],
        ),
    ] {
        let run = compile(source, strategy);
        let formulation = run.kind.formulation();
        // p = a ∨ b and q = p ∧ c, P(¬q) = 1 - 0.58 * 0.5 = 0.71. r = ¬p ∧ c
        // implies ¬q: P(r, ¬q) = 0.42 * 0.5. s = p ∧ b = b: P(s, ¬q) =
        // P(b)P(¬c) = 0.4 * 0.5.
        common::assert_probabilities(
            &formulation.to_string(),
            run.results.as_ref().unwrap(),
            &[0.21 / 0.71, 0.2 / 0.71],
            1e-12,
        );
        assert_eq!(calls(&run), expected, "{formulation}");
        let compile = run.stats.negations.total();
        assert_eq!(
            compile.calls, run.stats.op_counts[4],
            "{formulation}: every counted negation has a source"
        );
        assert!(
            compile.pairs >= compile.calls && compile.work_units <= run.stats.work_units,
            "{formulation}: {compile:?}"
        );
    }
}

#[test]
fn the_well_founded_fixpoint_and_the_links_count_as_their_own() {
    for strategy in BOTH {
        let run = compile(
            &std::fs::read_to_string("examples/negcycle_sound.pl").unwrap(),
            strategy,
        );
        let formulation = run.kind.formulation();
        assert!(
            run.stats.negations.of(NegationSource::WellFounded).calls > 0,
            "{formulation}: {:?}",
            calls(&run)
        );
        // With indicators every read of an atom through a copy is a link,
        // observed true: under the dual, its μ negated in the query phase.
        // A table's clauses negate nothing, and a network's atoms are all
        // tables; dice_murder reads atoms that are not. Its evidence on
        // obs_1 would fix obs and withgun (src/ground/condition.rs), so
        // obs_1 has a second rule here.
        let dice_murder = "0.3::alicedunnit. 0.03::withgun_then. 0.8::withgun_else. 0.1::noise.
            withgun :- alicedunnit, withgun_then.
            withgun :- \\+alicedunnit, withgun_else.
            obs :- withgun.
            obs_1 :- obs.
            obs_1 :- noise.
            evidence(obs_1, true).
            query(alicedunnit).";
        // `withgun` and `obs`, read once, would be inlined into `obs_1`.
        let gp = common::ground_as_written(dice_murder, Rows::Classes, true);
        let run =
            tddlog::solve_ground(gp, &common::options(strategy, tddlog::Projection::Off)).unwrap();
        let links = run.stats.negations.of(NegationSource::LinkTerm).calls;
        let evidence = run.query_sizes.negations.of(NegationSource::Evidence).calls;
        assert!(links > 0, "{formulation}: {:?}", calls(&run));
        assert_eq!(
            evidence > 0,
            formulation == Formulation::Dual,
            "{formulation}: {:?}",
            calls(&run)
        );
    }
}

#[test]
fn rows_of_facts_alone_negate_nothing() {
    // p's rows share a and c, which the factoring took as one term
    // ¬(a ∧ c ∧ (b ∨ d)): two negations. As clauses, none. q, of one
    // rule, is the conjunction of its body, λ_p, which the dual negates
    // from μ_p; the primal negates ⋀_r ¬body_r of p.
    let source = "0.3::a. 0.4::b. 0.5::c. 0.6::d.
        p :- a, b, c.
        p :- a, c, d.
        q :- p.
        query(p). query(q).";
    // P(p) = P(a) P(c) (1 - (1 - P(b)) (1 - P(d))).
    let p = 0.3 * 0.5 * (1.0 - 0.6 * 0.4);
    use NegationSource::*;
    for (strategy, expected) in [
        (Strategy::Trial(Kind::Closed), vec![(Lambda, 1)]),
        (Strategy::Trial(Kind::Dynamic), vec![(Primal, 1)]),
    ] {
        let run = compile(source, strategy);
        let formulation = run.kind.formulation();
        common::assert_probabilities(
            &formulation.to_string(),
            run.results.as_ref().unwrap(),
            &[p, p],
            1e-12,
        );
        assert_eq!(calls(&run), expected, "{formulation}");
    }
}

#[test]
fn gates_of_one_rule_are_stored_as_their_conjunctions() {
    // x, y and z have one rule each, and both formulations store each as
    // its body's conjunction: x and y negate nothing, z reads x
    // negatively, one negation of λ_x. w's bodies are y and c: the dual
    // stores μ_w, and its body y reads μ_y, the other negation; the primal
    // negates ⋀_r ¬body_r of w as well. The queries are counted where the
    // compile drops their formulas, x, y and z positive and w negative
    // under the dual. y reads a, which x implies, so that z reads x and
    // not a alone: unfolded under b, \+x would be \+a
    // (src/ground/negation.rs).
    let source = "0.3::a. 0.6::b. 0.2::c.
        x :- a, b.
        y :- a, x, \\+c.
        z :- \\+x, b.
        w :- y.
        w :- c.
        query(x). query(y). query(z). query(w).";
    // x = a ∧ b, y = x ∧ ¬c, z = ¬x ∧ b = ¬a ∧ b, and w = y ∨ c, whose
    // disjuncts exclude each other.
    let (x, y, z) = (0.3 * 0.6, 0.3 * 0.6 * 0.8, 0.7 * 0.6);
    let w = y + 0.2;
    use NegationSource::*;
    for (strategy, expected) in [
        (Strategy::Trial(Kind::Closed), vec![(Primal, 2)]),
        (Strategy::Trial(Kind::Dynamic), vec![(Primal, 3)]),
    ] {
        let run = compile(source, strategy);
        let formulation = run.kind.formulation();
        common::assert_probabilities(
            &formulation.to_string(),
            run.results.as_ref().unwrap(),
            &[x, y, z, w],
            1e-12,
        );
        assert_eq!(calls(&run), expected, "{formulation}");
    }
    // Observed, w is read in the polarity it is not stored in under the
    // primal (false) and the dual (true): P(q | w) for each query.
    for (observed, expected) in [
        ("true", [x * 0.8 + 0.3 * 0.6 * 0.2, y, 0.7 * 0.6 * 0.2]),
        ("false", [0.0, 0.0, 0.7 * 0.6 * 0.8]),
    ] {
        let evidence = if observed == "true" { w } else { 1.0 - w };
        let expected: Vec<f64> = expected.iter().map(|p| p / evidence).collect();
        let source = source.replace("query(w).", &format!("evidence(w, {observed})."));
        for strategy in BOTH {
            let run = compile(&source, strategy);
            common::assert_probabilities(
                &format!("{} w={observed}", run.kind.formulation()),
                run.results.as_ref().unwrap(),
                &expected,
                1e-12,
            );
        }
    }
}
