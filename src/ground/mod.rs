//! Grounding: from a [`Program`] to a propositional [`GroundProgram`]
//! ([`ground`]). This module is the pipeline; the ground program and its
//! indexes are in `program`, and docs/design.md, "Grounding", gives each
//! pass's reasons. The passes in the order they run, each with what it
//! leaves for those after it:
//!
//! 1. The front end, from the program to a ground program:
//!    - `table`: a program each of whose clauses is one row of a table is
//!      ground from its rows, its noisy-or tables first written as
//!      noisy-ors (`noisy_or`), and goes on at `finish`. Any other program
//!      takes the next steps.
//!    - `normalize`: annotated disjunctions and probabilistic rules become
//!      *switch* facts and deterministic rules.
//!    - `chain`: doubly recursive transitive closures written linear.
//!    - `magic` and `schedule`: the magic-set rewrite, after which only
//!      what the queries and the evidence demand is ground.
//!    - `engine`: stratified semi-naive bottom-up evaluation with every
//!      switch assumed true, which emits every ground rule whose positive
//!      body atoms are all possible.
//!    - `linear`: the program with its right-linear recursion evaluated
//!      from its bound end is ground too, and kept where it is smaller.
//! 2. `finish`, after the front end and after each layout rewrite:
//!    - `simplify`: no certain atom is left, equal atoms are merged, every
//!      atom is relevant to a query or an observation, facts are read as
//!      variables, and the numbering is compact.
//!    - `condition::settle`: evidence that unit propagation settles
//!      without the exclusive sets, which leaves nothing for the next
//!      passes; they are skipped then.
//!    - Otherwise `exclusive` finds the program's structure (its rules by
//!      head, components and exclusive sets), which each of these passes
//!      reads and hands on, updated or found anew where the pass changed
//!      the program: `inline` (views, then atoms with equal rules merged,
//!      `simplify`), `contexts` (choices read through a fact per context),
//!      `classes` (row classes), `share` (switch sharing), `cover` (atoms
//!      certain by cover) and `condition` (what the evidence implies, with
//!      `preprocess`'s backbones, in an acyclic program). The program
//!      keeps the structure the last one leaves.
//!    - `detach`: evidence that no query shares a fact with, dropped where
//!      a world satisfies it, in an acyclic program.
//!    - `defined`: in a CNF over facts, implied clauses dropped and the
//!      facts an independent support leaves out eliminated by resolution.
//!
//!    The program's `weight` is the factor that the conditioning,
//!    `defined` and the separator copies take out of both `P(q, e)` and
//!    `P(e)`; its `stats` report what each pass did.
//! 3. The layout rewrites, each finished again: `without_linear_recursion`
//!    (`eliminate`) for every layout, `guarded` (`guards`, `steps`) for
//!    `decision`; `fact_copies` adds the separator copies of `td`'s layout
//!    (`add_fact_copies`), and `restricted` keeps some of the queries for
//!    the query split.

mod aig;
mod chain;
pub(crate) mod classes;
mod condition;
mod contexts;
mod cover;
pub(crate) mod deps;
mod detach;
pub(crate) mod eliminate;
mod encoding;
mod engine;
pub(crate) mod exclusive;
mod fact_copies;
pub(crate) mod guards;
mod inline;
mod linear;
mod magic;
mod noisy_or;
mod normalize;
mod preprocess;
mod program;
mod propagation;
mod renormalize;
mod schedule;
pub(crate) mod shape;
mod share;
mod simplify;
mod steps;
pub(crate) mod structure;
mod table;
pub(crate) mod tables;
pub(crate) mod terms;
mod unread;

pub use aig::SweepStats;
pub use condition::{Backbones, Conditioned};
pub(crate) use encoding::{BITS_FROM, OWN_VALUE_FROM};
pub use encoding::{Code, Encoding, Indicator};
pub use engine::GroundError;
pub(crate) use fact_copies::{FactCopy, add_fact_copies};
pub use guards::Decisions;
pub(crate) use guards::applies as guards_apply;
pub use preprocess::backbone::Search as BackboneSearch;
pub use preprocess::defined::Defined;
pub use preprocess::implied::ImpliedStats;
pub use preprocess::support::SupportStats;
pub(crate) use program::{AtomId, AtomKind, Prob, RulesByHead};
pub use program::{GLit, GroundProgram, GroundRule, GroundStats};

use std::sync::{Arc, OnceLock};

use crate::eval::Count;
use crate::syntax::Program;
use preprocess::defined;
use structure::Structure;
use tables::Tables;

/// Options controlling grounding.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundOptions {
    /// Let the rows of a table that hold one distribution and exclude each
    /// other read it through an atom of their own (`classes`). On by
    /// default.
    pub classes: bool,
    /// Drop the observations of each part of the program that no query
    /// shares a fact with, where a world satisfies them (`detach`): their
    /// count cancels from every conditional probability. On by default;
    /// off where the probability of the evidence is wanted, of which that
    /// count is a factor.
    pub detach: bool,
    /// Replace the views, derived atoms read once or of one rule over facts
    /// alone, by their bodies in their readers (`inline`). On by default;
    /// off where the program's derived atoms are wanted as written, as the
    /// tests of the compile and the layouts build them.
    pub views: bool,
    /// Write a table of two values whose one value is a noisy-or of its
    /// parents as that noisy-or, where nothing reads, asks or observes its
    /// other value (`noisy_or`). On by default; off where the program's
    /// tables are wanted as written.
    pub noisy_or: bool,
    /// Let the conditioning fix the facts that every model of the clauses
    /// its propagation leaves agrees on, which a SAT solver finds, and
    /// merge the facts that the clauses of two literals the solver learns
    /// make equal (`backbone`). On by default; it applies to every program
    /// that the conditioning leaves clauses over facts, and does nothing
    /// elsewhere.
    pub backbones: bool,
    /// Eliminate the facts of probability one half that an independent
    /// support of the clauses over facts, which a SAT solver finds,
    /// leaves out, by resolution, after the conditioning (`defined`). On
    /// by default; it applies to a program that is a CNF over its facts
    /// with such facts, and does nothing elsewhere.
    pub defined: bool,
    /// Read a choice that rules read under exclusive contexts through a
    /// fact per rule (`contexts`). On by default; off where the program's
    /// derived atoms are wanted as written.
    pub contexts: bool,
    /// Renormalize each row of a table that the conditioning leaves some
    /// heads false over the heads it keeps (`renormalize`). On
    /// by default; off where the program's tables are wanted as the
    /// conditioning's rewrite leaves them.
    pub renormalize: bool,
}

impl Default for GroundOptions {
    fn default() -> Self {
        GroundOptions {
            classes: true,
            detach: true,
            views: true,
            noisy_or: true,
            backbones: true,
            defined: true,
            contexts: true,
            renormalize: true,
        }
    }
}

/// Ground a program, and ground it again with its right-linear recursion
/// evaluated from the bound end (`linear`), keeping whichever ground
/// program is smaller.
pub fn ground(program: &Program, opts: &GroundOptions) -> Result<GroundProgram, GroundError> {
    Ok(ground_keeping(program, opts, usize::MAX)?.0)
}

/// [`ground`], and the program it kept as [`finish`] simplified it before
/// its views are inlined, where that holds at least `keep` derived atoms:
/// what [`finish_kept`] finishes under options that differ in
/// [`GroundOptions::views`] alone, as `auto`'s `decision` trial does
/// (`solve::decision_trial`), without grounding and simplifying the
/// program again. Where the bound-end grounding is kept, the kept program
/// is its own.
pub(crate) fn ground_keeping(
    program: &Program,
    opts: &GroundOptions,
    keep: usize,
) -> Result<(GroundProgram, Option<Simplified>), GroundError> {
    // Compound terms, list predicates and findalls are written as the
    // Datalog the rest reads, with three built-in literals ([`terms`]).
    let (program, counted) = terms::lower(program)?;
    let program: &Program = &program;
    // A table-shaped program, a Bayesian network as one annotated
    // disjunction per row, is ground as written and its grounding is known
    // without the generic steps ([`table`]). Its noisy-or tables are first
    // written as noisy-ors where nothing reads their other value
    // ([`noisy_or`]), which leaves the program table-shaped, its shape made
    // from the program's.
    if let Some(shape) = table::Shape::of(program)
        && let Some(order) = shape.evaluation_order()
    {
        let gp = match if opts.noisy_or {
            noisy_or::rewrite(&shape, &order, program)
        } else {
            None
        } {
            Some((written, n)) => {
                let shape = shape.rewritten(program, &written);
                let order = shape
                    .evaluation_order()
                    .expect("tables written as noisy-ors read no more than they did");
                let mut gp = shape.ground(program, &order);
                gp.stats.noisy_or = n;
                gp
            }
            None => shape.ground(program, &order),
        };
        return Ok(finish_keeping(gp, opts, keep));
    }
    let mut norm = normalize::normalize(program)?;
    norm.counted = counted;
    // An argument no clause reads, as a derivation a fact leaves free, is
    // dropped ([`unread`]).
    let norm = unread::drop_unread(&norm).unwrap_or(norm);
    // A transitive closure written with the chain rule holds what its linear
    // form holds in every world, and the linear form is what the bound-end
    // rewrite below and the compile's elimination of linear recursion take.
    let norm = chain::linearize(&norm).unwrap_or(norm);
    let (gp, ground_size, kept) = ground_norm(&norm, opts, None, keep)?;
    // The rewritten program can fail to ground where the original does not
    // (a step that binds its target only through the recursive call); the
    // original stands then. It is ground only while it stays within the
    // original's grounding before `finish`, which shrinks the two alike: on
    // the sickness programs it grew 13 to 138 times, and grounding and
    // finishing it took most of the run.
    let Some((linear, from_bound_end)) = linear::rewrite(&norm) else {
        return Ok((gp, kept));
    };
    Ok(match ground_norm(&linear, opts, Some(ground_size), keep) {
        Ok((mut alt, _, mut alt_kept)) if alt.size() < gp.size() => {
            if let Some(k) = &mut alt_kept {
                k.gp.stats.from_bound_end = from_bound_end.clone();
            }
            alt.stats.from_bound_end = from_bound_end;
            (alt, alt_kept)
        }
        _ => (gp, kept),
    })
}

/// The finished ground program of `norm`, its size as ground, before
/// [`finish`], and the program kept for [`finish_kept`] where `keep`
/// asks for it ([`ground_keeping`]); [`GroundError::OverBound`] once its
/// grounding outgrows `bound` ([`engine::ground_normalized`]).
fn ground_norm(
    norm: &normalize::NormProgram,
    opts: &GroundOptions,
    bound: Option<usize>,
    keep: usize,
) -> Result<(GroundProgram, usize, Option<Simplified>), GroundError> {
    // A program may be non-stratified, by itself or through the rewrite: a
    // recursive component with internal negation is then grounded as it is,
    // its negative literals read as unknown, and left to the compiler's
    // well-founded fixpoint. The rewritten program is preferred even then,
    // because the demand it carries can be what makes a rule safe (`X1 is
    // X-1` under a bound head) and what keeps the grounding finite. The plain
    // program is the fallback when the rewritten one cannot be grounded.
    // A program without variables grounds to itself, and the relevance cut
    // below is what the magic rewrite would give it: the rewrite would only
    // add the demand rules and a guard to every rule for the grounder to
    // evaluate. On bn_munin that was about three times the rules.
    // A program whose recursive strata all have rising levels, a loop over
    // a time index, is grounded level by level and not rewritten: its
    // demand runs down the levels (`T0 is T - 1`) and makes a chain one
    // stratum without them ([`engine::ground_by_levels`]). Where that
    // grounding fails (past the bound, or a rule that only the demand
    // makes safe), the rewrite is tried.
    let gp = if magic::is_ground(norm) {
        engine::ground_normalized(norm, bound)?
    } else if let Some(gp) = match engine::ground_by_levels(norm, bound) {
        Err(e @ (GroundError::OutOfMemory | GroundError::Refused(_))) => return Err(e),
        levelled => levelled.ok().flatten(),
    } {
        gp
    } else {
        match magic::rewrite(norm) {
            Some(magic) => match engine::ground_normalized(&magic, bound) {
                Ok(gp) => gp,
                Err(
                    e @ (GroundError::OverBound
                    | GroundError::OutOfMemory
                    | GroundError::Refused(_)),
                ) => return Err(e),
                Err(_) => engine::ground_normalized(norm, bound)?,
            },
            None => engine::ground_normalized(norm, bound)?,
        }
    };
    let size = gp.size();
    let (gp, kept) = finish_keeping(gp, opts, keep);
    Ok((gp, size, kept))
}

/// `gp` finished under `opts` ([`finish`]), and a copy of it as simplified
/// before its views are inlined where that holds at least `keep` derived
/// atoms ([`ground_keeping`]).
fn finish_keeping(
    gp: GroundProgram,
    opts: &GroundOptions,
    keep: usize,
) -> (GroundProgram, Option<Simplified>) {
    let simplified = simplified(gp, opts);
    let kept = (simplified.gp.num_derived() >= keep).then(|| simplified.clone());
    (finished(simplified, opts, opts.views), kept)
}

/// A program kept by [`ground_keeping`], finished under `opts`, which
/// differ from the options it was ground under in
/// [`GroundOptions::views`] alone: the program [`ground`] gives under
/// `opts`, but where the bound-end grounding would be kept under `opts`
/// and not under the first options, or the other way round.
pub(crate) fn finish_kept(kept: Simplified, opts: &GroundOptions) -> GroundProgram {
    debug_assert_eq!(
        GroundOptions {
            views: opts.views,
            ..kept.gp.options
        },
        *opts,
        "a kept program is finished again under options that differ in views alone"
    );
    finished(kept, opts, opts.views)
}

/// A ground program as [`finish`] simplified it, up to where it inlines
/// the views: what it does before is the same with the views inlined or
/// not, so that one simplification serves both ([`ground_keeping`]).
#[derive(Clone)]
pub(crate) struct Simplified {
    gp: GroundProgram,
    /// What the propagation settled of the evidence, with its factor,
    /// where it settled all of it; the program's structure, which the
    /// inlining and the conditioning read, where it did not.
    stage: Stage,
}

#[derive(Clone)]
enum Stage {
    Settled((Conditioned, Count)),
    Structured(Structure),
}

/// The grounder's output simplified, with `inline` its views inlined
/// ([`inline`]; [`GroundOptions::views`] but where the program is finished
/// again), its tables' rows read through class atoms where that pays,
/// conditioned on its evidence, its certain atoms by cover taken out,
/// with [`GroundOptions::detach`], the evidence no query depends on
/// dropped where it can hold ([`detach`]), and with
/// [`GroundOptions::defined`], where it is a CNF over its facts, the facts
/// of probability one half its clauses define eliminated ([`defined`]).
fn finish(gp: GroundProgram, opts: &GroundOptions, inline: bool) -> GroundProgram {
    finished(simplified(gp, opts), opts, inline)
}

/// [`finish`]'s simplifications before the views are inlined.
fn simplified(mut gp: GroundProgram, opts: &GroundOptions) -> Simplified {
    gp.options = *opts;
    simplify::propagate_certain(&mut gp);
    simplify::merge_equal_atoms(&mut gp);
    simplify::restrict_to_relevant(&mut gp);
    simplify::alias_fact_atoms(&mut gp);
    // Evidence that the propagation settles without the exclusive sets
    // leaves nothing for the passes that read them, which are then not
    // run.
    let stage = match condition::settle(&mut gp) {
        Some(settled) => {
            simplify::propagate_certain(&mut gp);
            condition::drop_settled_evidence(&mut gp);
            simplify::restrict_to_relevant(&mut gp);
            gp.structure = OnceLock::new();
            Stage::Settled(settled)
        }
        None => Stage::Structured(Structure::of(&gp)),
    };
    Simplified { gp, stage }
}

/// [`finish`] from where [`simplified`] left the program.
fn finished(simplified: Simplified, opts: &GroundOptions, inline: bool) -> GroundProgram {
    let Simplified { mut gp, stage } = simplified;
    gp.options = *opts;
    let (conditioned, weight) = match stage {
        Stage::Settled(settled) => settled,
        Stage::Structured(structure) => conditioned_on_structure(&mut gp, structure, opts, inline),
    };
    // A program finished again keeps what its earlier conditionings took
    // out of its counts; its report is the last conditioning's.
    gp.weight = (gp.weight * weight).normalized();
    gp.stats.conditioned = conditioned;
    gp.stats.defined = None;
    if opts.detach {
        drop_detached(&mut gp);
    }
    // After the detached parts go, which leave less to search.
    if opts.defined
        && let Some(defined) = defined::eliminate_defined(&mut gp)
    {
        if defined.eliminated > 0 || defined.implied.implied > 0 {
            condition::drop_settled_evidence(&mut gp);
            simplify::restrict_to_relevant(&mut gp);
            gp.structure = OnceLock::new();
            gp.weight = gp.weight.shifted(-(defined.eliminated as i64)).normalized();
        }
        gp.stats.defined = Some(defined);
    }
    gp
}

/// In `gp`, whose structure is `structure`, the views inlined
/// ([`inline::views`]) and the atoms with an earlier atom's rules merged
/// into it ([`aig::reduce`]; not again after the guards' conditioning),
/// the choices read through a fact per context ([`contexts::choices`]),
/// the row classes, switch sharing and the cover applied, and `gp`
/// conditioned on its evidence on the exclusive sets these read
/// ([`condition::on_evidence`]); what the conditioning took out and its
/// factor, and `gp` left with its structure.
fn conditioned_on_structure(
    gp: &mut GroundProgram,
    mut structure: Structure,
    opts: &GroundOptions,
    inline: bool,
) -> (Conditioned, Count) {
    if inline {
        if inline::views(gp, &structure) > 0 {
            structure = Structure::of(gp);
        }
        // Not on a program conditioned on its guards (`guarded`): the
        // conditioning put each gate under the decisions that select it,
        // and which of two equal gates survives decides whether
        // `decision` compiles (bb_altermu2_w11: 103 gates merged there,
        // 181 M work units with g39582 kept, out of 8 GB with g12148,
        // 27 000 atoms earlier, kept). The structural merge before it
        // found none to merge on those programs.
        if gp.stats.decisions.is_none() && aig::reduce(gp, &structure) {
            structure = Structure::of(gp);
        }
    }
    // The row classes, switch sharing, the cover and the conditioning read
    // the exclusive sets; each reads them anew where the one before
    // rewrote the program, and the program keeps them where the cover and
    // the conditioning left it as it was. After the sharing, which renames
    // facts alone, they are the sets before, renumbered
    // ([`Structure::renumbered`]).
    let mut tables = Tables::of_structure(gp, structure);
    if opts.contexts && contexts::choices(gp, &mut tables) > 0 {
        tables = Tables::of_structure(gp, Structure::of(gp));
    }
    if opts.classes && classes::row_classes(gp, &mut tables) {
        tables = Tables::of_structure(gp, Structure::of(gp));
    }
    if let Some(map) = share::share_switches(gp, &mut tables) {
        tables = tables.renumbered(gp, &map);
    }
    let cover = cover::Cover::of(gp, &tables);
    let (conditioned, weight) =
        condition::on_evidence(gp, &tables, &cover, opts.backbones, opts.renormalize);
    if cover.apply(gp) > 0 || conditioned.any() {
        simplify::propagate_certain(gp);
        if conditioned.any() {
            condition::drop_settled_evidence(gp);
        }
        simplify::restrict_to_relevant(gp);
        gp.structure = OnceLock::new();
    } else {
        gp.structure = OnceLock::from(Arc::new(tables.into_structure()));
    }
    (conditioned, weight)
}

/// The program for the queries at `queries` of `gp.queries` alone: what
/// they and the evidence depend on, finished as the grounder finishes a
/// program, the observations no query of theirs depends on dropped where
/// `gp` was ground to drop them ([`GroundOptions::detach`]).
pub(crate) fn restricted(gp: &GroundProgram, queries: &[usize]) -> GroundProgram {
    let mut part = gp.clone();
    part.queries = queries.iter().map(|&i| gp.queries[i].clone()).collect();
    simplify::restrict_to_relevant(&mut part);
    part.structure = OnceLock::new();
    if part.options.detach {
        drop_detached(&mut part);
    }
    part
}

/// The observations no query depends on dropped where they can hold
/// ([`detach`]), and the rules only they read.
fn drop_detached(gp: &mut GroundProgram) {
    let detached = detach::drop_detached(gp);
    if detached > 0 {
        gp.structure = OnceLock::new();
        simplify::restrict_to_relevant(gp);
        gp.stats.detached += detached;
    }
}

/// The program with its linear recursion written without recursion
/// (`eliminate`) and finished again, under the options it was ground
/// with: the passes that leave a cyclic program alone then apply, the
/// conditioning on the evidence among them. sickness22_40's evidence then
/// fixes 34 facts and 520 derived atoms. The views are not inlined again:
/// the elimination's atoms are labels it keeps apart on purpose
/// (`eliminate`), which inlining would multiply out.
pub(crate) fn without_linear_recursion(mut gp: GroundProgram) -> GroundProgram {
    if eliminate::linear_recursion(&mut gp) == 0 {
        return gp;
    }
    let opts = gp.options;
    finish(gp, &opts, false)
}

/// The program with its derived atoms conditioned on the fact literals
/// that guard their reads (`guards`) and its chains of steps written as
/// balanced trees of their compositions (`steps`), and finished again
/// under the options it was ground with and its views inlined
/// ([`GroundOptions::views`]), which propagates the evidence through what
/// these made. Both read the program best ground with its views in place
/// (`Kind::ground_options`); the program as it was where neither applies
/// and its views are inlined. The layout `decision` compiles it, laid out
/// from the decisions the conditioning recorded (`gp.stats.decisions`). It
/// is not the grounder's default: on uwcse it takes the tables' structure
/// apart, and every layout's work grows a hundredfold and more. A program
/// already conditioned, its decisions recorded, is returned as it is:
/// `auto` hands one to every trial (`solve::decision_trial`).
pub(crate) fn guarded(mut gp: GroundProgram) -> GroundProgram {
    if gp.stats.decisions.is_some() {
        return gp;
    }
    let conditioned = guards::condition(&mut gp).is_some();
    let stepped = steps::balance(&mut gp) > 0;
    if !conditioned && !stepped && gp.options.views {
        return gp;
    }
    let opts = GroundOptions {
        views: true,
        ..gp.options
    };
    finish(gp, &opts, true)
}

#[cfg(test)]
mod tests {
    use super::{GroundOptions, Stage, finish_kept, ground, ground_keeping};

    /// A program kept as simplified and finished with its views in place
    /// is the program ground with them in place: on a program whose view
    /// the first finish inlines, on one whose evidence the propagation
    /// settles, and on one ground from its bound end.
    #[test]
    fn a_kept_grounding_finished_again_is_the_grounding_under_its_options() {
        let sources = [
            "0.3::s. 0.6::a. 0.2::b. 0.4::c.
             v :- a, c.
             m :- s, v. m :- \\+s, b.
             t :- m, s. t :- \\+m, \\+s.
             query(t).",
            "0.3::a. 0.6::b.
             x :- a. y :- x, b. z :- \\+y.
             evidence(a, true). evidence(b, true).
             query(z).",
            include_str!("../../examples/linear_path.pl"),
        ];
        let first = GroundOptions::default();
        let again = GroundOptions {
            views: false,
            ..first
        };
        for (i, source) in sources.into_iter().enumerate() {
            let program = crate::syntax::parse_program(source).expect("parses");
            let (gp, kept) = ground_keeping(&program, &first, 0).expect("grounds");
            let plain = ground(&program, &first).expect("grounds");
            assert_eq!(gp.to_string(), plain.to_string());
            let kept = kept.expect("kept");
            match i {
                0 => assert!(plain.stats.inlined > 0),
                1 => assert!(matches!(kept.stage, Stage::Settled(_))),
                _ => assert_eq!(plain.stats.from_bound_end, ["path/2"]),
            }
            let finished = finish_kept(kept, &again);
            let expected = ground(&program, &again).expect("grounds");
            assert_eq!(finished.to_string(), expected.to_string(), "{source}");
            assert_eq!(
                format!("{:?} {:?}", finished.weight, finished.stats),
                format!("{:?} {:?}", expected.weight, expected.stats),
                "{source}"
            );
            assert_eq!(finished.options, again);
            assert_eq!(finished.stats.inlined, 0);
            let (_, none) = ground_keeping(&program, &first, usize::MAX).expect("grounds");
            assert!(none.is_none());
        }
    }

    /// A loop over a time index written as recursion is grounded level by
    /// level, which proves atoms constant as their levels complete
    /// (tests/regress/levels.pl, against ProbLog), and one whose level
    /// rises through a successor relation given by its facts
    /// (tests/regress/levels_successor.pl, by hand); a recursion with no
    /// rising level is not, nor one whose relation has a clause that does
    /// not rise.
    #[test]
    fn a_rising_level_is_grounded_level_by_level() {
        let opts = GroundOptions::default();
        let source = include_str!("../../tests/regress/levels.pl");
        let program = crate::syntax::parse_program(source).expect("parses");
        let gp = ground(&program, &opts).expect("grounds");
        assert!(gp.stats.levels > 0 && gp.stats.constants > 0);
        let source = include_str!("../../tests/regress/levels_successor.pl");
        let program = crate::syntax::parse_program(source).expect("parses");
        assert!(ground(&program, &opts).expect("grounds").stats.levels > 0);
        let flat = format!("{source}\nnext(4,4).");
        let program = crate::syntax::parse_program(&flat).expect("parses");
        assert_eq!(ground(&program, &opts).expect("grounds").stats.levels, 0);
        let source = include_str!("../../examples/linear_path.pl");
        let program = crate::syntax::parse_program(source).expect("parses");
        assert_eq!(ground(&program, &opts).expect("grounds").stats.levels, 0);
    }
}
