//! The one answer path: parse, ground, the `decision` trial, the check
//! whether the queries are answered in parts, each part's run, and the
//! answers put back in the order of the program's queries ([`solve`]);
//! and one ground program's run ([`solve_ground`]).

mod split;
#[cfg(test)]
mod tests;

use std::time::Instant;

use crate::auto;
use crate::evaluation::Evaluation;
use crate::ground::{self, GroundOptions, GroundProgram};
use crate::strategy::{Prepared, Strategy};
use crate::syntax::{self, Program};
use crate::tp::{self, Projection};
use crate::vtree::Kind;
use crate::{Error, QueryResult, Run, eval};
use split::{Split, SplitBounds};

/// What a run is asked to do.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// `auto`'s trials, or one layout.
    pub strategy: Strategy,
    /// Whether `P(e)` is wanted ([`Solution::evidence_probability`]): the
    /// grounder then keeps the observations no query depends on, which it
    /// would drop for cancelling from the answers.
    pub evidence_probability: bool,
    /// Whether the program's queries are answered; without them the program
    /// is ground for its evidence alone, as for the partition function.
    pub queries: bool,
    /// For tests: the grounder's options, [`GroundOptions::default`] in a
    /// run; `detach` is off where `evidence_probability` is on.
    pub ground: GroundOptions,
    /// For tests: the projection, where not [`Projection::default`] as in a
    /// run. A dynamic layout compiles unprojected ([`Kind::projection`]),
    /// and is refused another.
    pub projection: Option<Projection>,
    /// For tests: the live pairs at which a dynamic layout searches first,
    /// where not `tp::DYNAMIC_FIRST_PAIRS`; with [`Kind::Dynamic`] alone.
    pub dynamic_first_pairs: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            strategy: Strategy::Auto,
            evidence_probability: false,
            queries: true,
            ground: GroundOptions::default(),
            projection: None,
            dynamic_first_pairs: None,
        }
    }
}

impl Options {
    /// The default options under `strategy`.
    pub fn with(strategy: Strategy) -> Self {
        Options {
            strategy,
            ..Options::default()
        }
    }

    /// The grounder's options for a run under these options, which
    /// [`solve_ground`] takes its program ground under.
    pub fn ground_options(&self) -> GroundOptions {
        self.strategy.ground_options(GroundOptions {
            detach: self.ground.detach && !self.evidence_probability,
            ..self.ground
        })
    }

    /// The projection asked for, failing where the strategy cannot
    /// compile under it, and the live pairs at which a dynamic layout
    /// searches first, failing where the strategy is not dynamic; failing
    /// too where neither the queries nor `P(e)` are asked for.
    fn checked(&self) -> Result<(Projection, usize), Error> {
        if !self.queries && !self.evidence_probability {
            return Err(Error::Options(
                "the queries are left out and P(e) is not asked for: nothing to answer".into(),
            ));
        }
        let dynamic = matches!(self.strategy, Strategy::Trial(kind) if kind.is_dynamic());
        if self.dynamic_first_pairs.is_some() && !dynamic {
            return Err(Error::Options(format!(
                "the first search's pairs are for a dynamic strategy, not `{}`",
                self.strategy
            )));
        }
        if dynamic && self.projection.is_some_and(|p| p != Projection::Off) {
            return Err(Error::Options(format!(
                "`{}` changes the one vtree every formula lives on, and needs projection off",
                self.strategy
            )));
        }
        Ok((
            self.projection.unwrap_or_default(),
            self.dynamic_first_pairs.unwrap_or(tp::DYNAMIC_FIRST_PAIRS),
        ))
    }
}

/// What [`solve`] reports as it goes, for `--stats`.
pub enum Progress<'a> {
    /// The program is ground, in `seconds`.
    Grounded { seconds: f64 },
    /// A line on the run as a whole: the `decision` trial's grounding, the
    /// split of the queries, a part's start.
    Note(String),
    /// A part's program as it is handed to the compile.
    Compiling(&'a GroundProgram),
    /// A part's run, answered, before it is freed.
    Answered(&'a Run),
}

/// The answers to a program's queries.
pub struct Solution {
    /// The answers, in the order of the program's queries; `None` where the
    /// evidence is inconsistent.
    pub answers: Option<Vec<QueryResult>>,
    /// `P(e)` ([`Run::evidence_probability`]), where it was asked for.
    pub evidence_probability: Option<eval::Count>,
    /// What the run built and holds until the solution is dropped: the
    /// program as parsed and the last part's run. A caller that ends the
    /// process with the solution can leave it to the exit, as `tddlog`
    /// does: freeing them piece by piece took up to 1% of a large network's
    /// run.
    #[allow(dead_code)]
    held: (Program, Run),
}

/// Parse `source`, ground it, compile it and answer its queries under
/// `options`.
///
/// Under `auto`, a large circuit is ground again and conditioned on its
/// guards (`decision_trial`), for `decision`, the first trial, where it is
/// one (`auto::decides`), and for `program`, `closed` and `td` where it is
/// not (`auto::Programs`). The queries are answered in groups,
/// each on the part of the program it and the evidence depend on, where
/// the parts' layouts are estimated much smaller than the program's
/// (`split_check`); else all of them on the program, which under `auto`
/// may come with its `families:0` trial built by the check. A program with
/// a `decision` trial is answered whole without the check: on the full
/// weekend circuit the check built a `families:0` layout of width 5 143 in
/// 24 s and held it through a run that the `decision` trial, first of the
/// trials, finished. Each part's run is reported to `progress` and freed
/// before the next.
pub fn solve(
    source: &str,
    options: &Options,
    progress: &mut dyn FnMut(Progress<'_>),
) -> Result<Solution, Error> {
    solve_split(source, options, &SplitBounds::default(), progress)
}

/// [`solve`] with the queries answered in parts where `bounds` split them
/// ([`split_check`]).
fn solve_split(
    source: &str,
    options: &Options,
    bounds: &SplitBounds,
    progress: &mut dyn FnMut(Progress<'_>),
) -> Result<Solution, Error> {
    let (projection, first_pairs) = options.checked()?;
    let program = parsed(source, options)?;
    let start = Instant::now();
    let ground_options = options.ground_options();
    // `auto` keeps the program as simplified before its views are
    // inlined for its order layouts, which take it with its views in
    // place ([`Kind::ground_options`]).
    let keep = match options.strategy {
        Strategy::Auto => 0,
        Strategy::Trial(_) => usize::MAX,
    };
    let (ground, kept) = ground::ground_keeping(&program, &ground_options, keep)?;
    progress(Progress::Grounded {
        seconds: start.elapsed().as_secs_f64(),
    });
    // Its time is counted as the first part's rewrite's. It differs from
    // `ground` where the grounding inlined any atom, a view's or a chain's.
    let start = Instant::now();
    let mut written = kept
        .filter(|_| ground.stats.inlined + ground.stats.chained > 0 || auto::may_decide(&ground))
        .map(|kept| ground::finish_kept(kept, &Kind::Decision.ground_options(ground_options)));
    let decision = decision_trial(written.as_ref(), &ground);
    let decided_seconds = start.elapsed().as_secs_f64();
    let decides = decision
        .as_ref()
        .and_then(|d| d.program.as_ref())
        .is_some_and(|c| auto::decides(c, &ground));
    if let Some(d) = &decision {
        let tried = if decides { "a trial" } else { "no trial" };
        progress(Progress::Note(format!(
            "decision: finished again, {} atoms conditioned on their guards for {} derived, {tried}, {decided_seconds:.3}s",
            d.guarded,
            auto::derived_unmerged(&ground)
        )));
    }
    let mut conditioned = decision.and_then(|d| d.program);
    let answering = if decides {
        Answering::Whole(None)
    } else {
        split_check(&ground, bounds, options.strategy)
    };
    let (parts, places, mut laid) = match answering {
        Answering::Split(split) => {
            progress(Progress::Note(split.note));
            // The conditioned and the written programs are the whole
            // program's.
            conditioned = None;
            written = None;
            (split.parts, split.queries, None)
        }
        Answering::Whole(laid) => {
            let queries = (0..ground.num_queries()).collect();
            (vec![ground], vec![queries], laid)
        }
    };
    let mut answers: Vec<Option<QueryResult>> = (0..places.iter().map(Vec::len).sum())
        .map(|_| None)
        .collect();
    let (mut inconsistent, mut evidence_probability) = (false, None);
    let count = parts.len();
    let mut last = None;
    for (i, (part, places)) in parts.into_iter().zip(places).enumerate() {
        if count > 1 {
            progress(Progress::Note(format!(
                "part {} of {count}: {} queries",
                i + 1,
                places.len()
            )));
        }
        let mut run = match options.strategy {
            Strategy::Auto => {
                progress(Progress::Compiling(&part));
                auto::run(
                    part,
                    projection,
                    laid.take(),
                    conditioned.take(),
                    written.take(),
                )?
            }
            Strategy::Trial(kind) => {
                let prepared = kind.prepare(part)?;
                progress(Progress::Compiling(&prepared.ground));
                run_prepared(prepared, kind, projection, first_pairs)?
            }
        };
        if i == 0 {
            run.rewrite_seconds += decided_seconds;
            evidence_probability = run.evidence_probability;
        }
        progress(Progress::Answered(&run));
        match &run.results {
            None => inconsistent = true,
            Some(results) => {
                assert_eq!(
                    results.len(),
                    places.len(),
                    "an answer per query of the part"
                );
                for (r, &at) in results.iter().zip(&places) {
                    answers[at] = Some(r.clone());
                }
            }
        }
        // An earlier part's run is freed for the next.
        last = Some(run);
    }
    let answers = (!inconsistent).then(|| {
        answers
            .into_iter()
            .map(|r| r.expect("every part answered"))
            .collect()
    });
    let evidence_probability = if options.evidence_probability {
        evidence_probability
    } else {
        None
    };
    Ok(Solution {
        answers,
        evidence_probability,
        held: (program, last.expect("a program has a part")),
    })
}

/// The program [`solve`] hands the compile under `options`, as `--ground`
/// prints it: parsed and ground, and for a layout other than `auto`'s as
/// the layout compiles it (`Kind::prepare`), which lays it out: `td`'s
/// separator copies follow its layout.
pub fn rewritten(source: &str, options: &Options) -> Result<GroundProgram, Error> {
    options.checked()?;
    let ground = ground::ground(&parsed(source, options)?, &options.ground_options())?;
    Ok(match options.strategy {
        Strategy::Auto => ground,
        Strategy::Trial(kind) => kind.prepare(ground)?.ground,
    })
}

/// `source` parsed, without its queries where `options` leave them out.
fn parsed(source: &str, options: &Options) -> Result<Program, Error> {
    let program = syntax::parse_program(source)?;
    Ok(if options.queries {
        program
    } else {
        program.without_queries()
    })
}

/// Compile and answer the queries of `ground`, as grounded under
/// `options` ([`Options::ground`]), whole.
///
/// `auto` runs trials of layouts over rewrites of the program under bounds
/// that grow, and answers with the first to finish its compile, evidence
/// and queries (`src/auto/`); without the `decision` trial, which grounds
/// the program again. Any other strategy is one layout of its rewrite of
/// the program (`Kind::prepare`), compiled without bounds.
pub fn solve_ground(ground: GroundProgram, options: &Options) -> Result<Run, Error> {
    let (projection, first_pairs) = options.checked()?;
    match options.strategy {
        Strategy::Auto => auto::run(ground, projection, None, None, None),
        Strategy::Trial(kind) => run_prepared(kind.prepare(ground)?, kind, projection, first_pairs),
    }
}

/// The run of `kind`'s program and layout, `prepared`, compiled without
/// bounds under `projection`; a dynamic layout searches first at
/// `first_pairs` live pairs.
pub(crate) fn run_prepared(
    prepared: Prepared,
    kind: Kind,
    projection: Projection,
    first_pairs: usize,
) -> Result<Run, Error> {
    let Prepared {
        ground,
        deps,
        layout,
        rewrite_seconds,
        layout_seconds,
    } = prepared;
    let mut evaluation = Evaluation::default();
    let ((results, query_sizes), vtree) = layout.vtree.context().run(|engine| {
        let backend = tp::TddBackend::new(engine, layout.vtree.clone());
        let answers = evaluation.run(
            &ground,
            &deps,
            &layout.order,
            &backend,
            kind,
            projection,
            first_pairs,
            None,
            None,
        )?;
        Ok::<_, tp::CompileError>((answers, backend.vtree()))
    })?;
    let evidence_probability = evaluation.evidence_probability(&ground);
    Ok(Run {
        ground,
        vtree,
        kind,
        vtree_seconds: layout_seconds,
        attempts: 1,
        vtree_notes: layout.note.into_iter().collect(),
        rewrite_seconds,
        wasted_seconds: 0.0,
        query_seconds: evaluation.query_seconds,
        query_sizes,
        evidence_probability,
        stats: evaluation.into_stats(),
        results,
        spent: auto::Spent::default(),
    })
}

/// What [`split_check`] decided for a program: its queries in parts, or
/// the program whole, with `auto`'s `families:0` trial where the check
/// built it.
enum Answering {
    Split(Split),
    Whole(Option<Box<Prepared>>),
}

/// The program's queries in groups, each to be answered on its own part
/// ([`split::query_parts`]), for a run under `strategy`, where the parts'
/// layouts, as `auto`'s searched `families` trial lays them out, are
/// estimated to hold `bounds.layout_gain` times fewer pairs together, each
/// charged `bounds.part_pairs`, than the program's; the program whole
/// where they are not, the program's `families:0` layout is estimated
/// under `bounds.min_pairs` (the floor), or the weights rule a split out.
/// A part that `families` does not lay out, without indicators, is charged
/// 2 to its min-fill weight: the UAI network BN_15's 14th part, 3 atoms,
/// has none, and refused its split, its other 13 estimated at 9.8e11 pairs
/// together against 1.6e15 for the program, which is refused memory whole.
///
/// The floor is asked before the weights, which cost more where it
/// refuses the split. Under `auto`, where its first trial is `families:0`,
/// that trial's program and layout are built before the check, whose floor
/// is their estimate, and a program kept whole is handed them for its run
/// instead of building them again.
///
/// The min-fill weights over atoms are a filter and not the decision: they
/// mislead where the searched decomposition is much narrower than
/// min-fill's. pedigree7_10's parts weigh 2^50.4 against 2^61.0 and took
/// 368.8 M units of trial work against 259.3 M; their layouts are
/// estimated at 3.09e9 pairs against 2.53e9.
fn split_check(gp: &GroundProgram, bounds: &SplitBounds, strategy: Strategy) -> Answering {
    let first = strategy == Strategy::Auto && auto::lays_families_first(gp);
    let laid = if first {
        auto::families(gp, false).map(Box::new)
    } else {
        None
    };
    let floor = || {
        let estimate = if first {
            laid.as_deref().and_then(Prepared::pairs)
        } else {
            auto::families_estimate(gp, false)
        };
        bounds.layout_gain <= 0.0 || estimate.is_some_and(|e| e >= bounds.min_pairs)
    };
    let Some(mut split) = split::query_parts(gp, bounds, floor) else {
        return Answering::Whole(laid);
    };
    if bounds.layout_gain > 0.0 {
        let Some(whole) = auto::families_estimate(gp, true) else {
            return Answering::Whole(laid);
        };
        let mut parts = split.parts.len() as f64 * bounds.part_pairs;
        for part in &split.parts {
            if parts * bounds.layout_gain > whole {
                return Answering::Whole(laid);
            }
            parts += auto::families_estimate(part, true)
                .unwrap_or_else(|| crate::vtree::min_fill_weight(part).exp2());
        }
        if parts * bounds.layout_gain > whole {
            return Answering::Whole(laid);
        }
        split.note.push_str(&format!(
            ", layouts estimated at {parts:.3e} pairs together against {whole:.3e}"
        ));
    }
    Answering::Split(split)
}

/// A program finished again for `auto` ([`decision_trial`]): how many
/// atoms the conditioning on guards made, and the program conditioned on
/// its guards, where the conditioning recorded decisions.
struct DecisionTrial {
    guarded: usize,
    program: Option<GroundProgram>,
}

/// The program that `auto`'s `decision` trial compiles beside `ground`,
/// or, where `decision` is no trial, that `program`, `closed` and `td`
/// compile in its place (`auto::Programs`): `written`, `ground` as ground
/// with its views in place ([`ground::ground_keeping`],
/// [`Kind::ground_options`]), rewritten for `decision` ([`Kind::rewrite`]),
/// where `ground` is large and the conditioning on guards applies to it
/// and gets under way there (`auto::may_decide`), and kept where the
/// conditioning recorded decisions and made enough atoms for a trial to
/// take it (`auto::takes_conditioned`); `None` where the program is not
/// rewritten. `closed` takes less work on it: bb_altermu2_w11 2.29 G units
/// against 14.4 G. The full weekend circuit of the bit-blasting paper
/// (98 303 flips) runs out of memory under every other layout of the
/// program as ground and compiles under `decision` in 533 M work units.
fn decision_trial(
    written: Option<&GroundProgram>,
    ground: &GroundProgram,
) -> Option<DecisionTrial> {
    let written = written.filter(|_| auto::may_decide(ground))?;
    // `Kind::Decision.rewrite`, which `written`, read through no
    // indicator, takes in full, finished again only where a trial takes
    // it.
    let decided = ground::guarded_where(Kind::eliminated(written.clone()), |made| {
        auto::takes_conditioned(made, ground)
    });
    let guarded = match &decided {
        Ok(gp) => gp.stats.guarded,
        Err(gp) => gp.stats.guarded,
    };
    let program = decided.ok().filter(|gp| gp.stats.decisions.is_some());
    Some(DecisionTrial { guarded, program })
}

/// `ground` compiled as it is, without `kind`'s rewrite, under
/// `projection`, a dynamic layout searching first at `first_pairs` live
/// pairs: a test's run of a program the rewrite would change, such as a
/// linear recursion compiled as a cyclic component.
#[cfg(test)]
pub(crate) fn run_as_given(
    ground: GroundProgram,
    kind: Kind,
    projection: Projection,
    first_pairs: usize,
) -> Result<Run, Error> {
    run_prepared(
        Prepared::lay_out(ground, None, kind, 0.0)?,
        kind,
        projection,
        first_pairs,
    )
}

/// `source` parsed, ground under `options` and run whole
/// ([`solve_ground`]): a test's run of one program, whose run it reads.
#[cfg(test)]
pub(crate) fn run_source(source: &str, options: &Options) -> Result<Run, Error> {
    let ground = ground::ground(&syntax::parse_program(source)?, &options.ground_options())?;
    solve_ground(ground, options)
}
