//! The `auto` strategy: trials of vtrees over rewrites of the program.
//!
//! Each trial is a layout, a [`Kind`] ([`candidates`]), with the program
//! as that layout rewrites it. A trial wins only after its compilation,
//! evidence and all queries complete.
//!
//! The trials take turns. Each has a weight ([`policy::weight`]) and a
//! target, the engine work it may have spent by the end of its turn, at
//! first its weight times [`FIRST_PAIRS`] times [`WORK_PER_PAIR`] units.
//! Each attempt goes to the trial projected to finish its phase first for
//! its weight ([`Trial::key`]), the first in the candidates' order on a
//! tie, and may spend what the target leaves; once the trial has spent its
//! target, the target doubles. A trial's projection is the work at which
//! the pace of its last doubling finishes its phase, or the pace since its
//! query phase began where that phase is younger ([`Trial::projected`]),
//! counted from its target to [`PROJECTED_DOUBLINGS`] doublings past it;
//! one further out, or none, counts that many, so that without
//! projections the trials take equal shares for their weights, and a
//! trial projected to finish soon takes its turns ahead of the others,
//! by up to that many doublings. A compile notes its progress at half
//! its first turn's target ([`tp::Horizon`]), so that the pace of that
//! turn's second half projects it where it puts the compile's finish past
//! the trial's next target ([`Trial::ahead`]). The attempt's watch, the largest formula
//! it may complete, is a [`policy::WORK_PER_HELD_PAIR`]th of the target in
//! pairs, or more where a formula past the watch stopped an earlier
//! attempt: such a stop raises the watch to hold it ([`Trial::hold`]) and
//! leaves the target alone. Where `families:0`'s estimate
//! calls for it, the searched families layout is built once `families:0`
//! has spent what the search costs ([`policy::search_rent`]), and
//! otherwise at its own turn; either way it may take `families:0`'s place
//! and the weight ([`weigh_searched`]).
//! The trial resumes where it stopped: finished components survive a
//! stopped compile, and completed evidence conjunctions and query answers
//! a stopped query phase. The first trial
//! to finish answers. Memory goes to the trial that has spent the most:
//! refused it while other trials hold what their attempts built, and
//! having spent more work than each of them, a trial takes it, once: what
//! they hold is freed, each of them to begin its phase again at its next
//! turn ([`Trial::release`]), and the refused trial resumes at its next
//! turn. Refused otherwise (again, behind a trial that has spent as much,
//! while no other trial holds anything, or past tididi's 32-bit indices),
//! it is retired, and the run goes on with the others ([`Trial::retire`]).
//! So a refusal discards less work than the refused trial has spent, from
//! each other trial, and once a trial. A trial with no other left runs
//! without bounds ([`attempt`]). A trial retired while another held what
//! it built may have been refused for that memory alone: once no trial is
//! left, each such trial runs again, once, with nothing held beside it
//! ([`Trial::behind`]).
//!
//! So the run spends on each other trial at most its weight's share of
//! twice the winner's work, times two to the doublings by which the
//! projections set the two apart. The bounds are in diagram pairs and engine
//! work, so the winner does not depend on the machine. Vtree construction
//! and application-side walks are not preempted by the engine work clock.

mod policy;
#[cfg(test)]
mod tests;
mod trial;

use std::time::Instant;

use crate::evaluation::Answers;
use crate::strategy::Prepared;
use crate::vtree::Kind;
use crate::{
    Error, GroundOptions, GroundProgram, Projection, QueryResult, Run, ground, query, tp, vtree,
};
use tididi::limits::{LimitConfig, StopAt, StopRules};

use policy::{
    FAMILIES_WEIGHT, FIRST_PAIRS, LEAD_SHARE, PROJECTED_DOUBLINGS, SEARCHED_GAP, WORK_PER_PAIR,
    candidates, families_weighted, overspent_estimate, projected_from, search_due, search_rent,
    weight,
};
use trial::Trial;

/// Whether two layouts are the same vtree over the same fact order. Equal
/// numbered trees over different program facts are different layouts.
fn same_layout(a: &vtree::Layout, b: &vtree::Layout) -> bool {
    a.order == b.order && a.vtree.to_text() == b.vtree.to_text()
}

/// What the trials have cost so far, which the [`Run`] reports.
#[derive(Default)]
struct Ledger {
    attempts: usize,
    vtree_seconds: f64,
    rewrite_seconds: f64,
    /// Seconds in the trials' attempts.
    trial_seconds: f64,
    notes: Vec<String>,
}

/// `auto`'s `families` trial of `gp`, searched or not, built as the trial
/// builds it ([`prepare`]): the program without its linear recursion read
/// through every indicator, its rules by head and components and its
/// layout, which `run` takes as that trial's own; `None` where the rewrite
/// introduces no indicator, or the layout fails.
pub(crate) fn families(gp: &GroundProgram, search: bool) -> Option<Prepared> {
    let kind = Kind::Families { search };
    let start = Instant::now();
    let ground = kind.rewrite(gp.clone());
    if ground.encoding.is_empty() {
        return None;
    }
    Prepared::lay_out(ground, None, kind, start.elapsed().as_secs_f64()).ok()
}

/// The pairs that `auto`'s `families` trial, searched or not, would lay
/// `gp` out to hold ([`families`]).
pub(crate) fn families_estimate(gp: &GroundProgram, search: bool) -> Option<f64> {
    families(gp, search)?.pairs()
}

/// Whether `auto`'s first trial on `gp` is `families:0` ([`candidates`]),
/// which [`run`] can be handed ready.
pub(crate) fn lays_families_first(gp: &GroundProgram) -> bool {
    candidates(gp).first() == Some(&Kind::Families { search: false })
}

/// `auto` on `original`, as grounded, its trials the [`candidates`], each
/// compiling its layout's rewrite of its program ([`Programs::of`]:
/// `original`, or `ready`'s `written` or `conditioned`, each eliminated
/// once for all of them, [`Kind::eliminated`]); the `families:0` trial
/// takes `ready`'s `laid` as its own, and where `ready`'s `conditioned`
/// [`decides`], `decision` is the first trial.
pub(crate) fn run(
    original: GroundProgram,
    projection: Projection,
    ready: Ready,
) -> Result<Run, Error> {
    let mut trials = candidates(&original);
    let decision = ready.decides(&original);
    if decision {
        trials.insert(0, Kind::Decision);
    }
    let Ready {
        written,
        conditioned,
        laid,
    } = ready;
    run_within(
        Programs {
            ground: original,
            written,
            conditioned,
            decision,
        },
        trials,
        projection,
        Watch::default(),
        laid,
    )
}

/// What [`run`] can be handed ready beside the program as ground, each
/// where it was built: the program ground with its views in place, and
/// that program conditioned on its guards ([`ready`]); and `families:0`'s
/// program and layout, built from the program as ground by [`families`]
/// (`solve::split_check`). Each is the whole program's: a part of a split
/// is handed none.
#[derive(Default)]
pub(crate) struct Ready {
    pub(crate) written: Option<GroundProgram>,
    pub(crate) conditioned: Option<GroundProgram>,
    pub(crate) laid: Option<Box<Prepared>>,
}

impl Ready {
    /// Whether `decision` is the first trial on `ground` ([`decides`]).
    pub(crate) fn decides(&self, ground: &GroundProgram) -> bool {
        self.conditioned
            .as_ref()
            .is_some_and(|c| decides(c, ground))
    }
}

/// The programs `auto` takes beside `ground` ([`Ready`]), from `kept`,
/// `ground`'s grounding kept before its views were inlined
/// ([`ground::ground_keeping`]), and how many atoms the conditioning on
/// guards made where it ran. `written` is `kept` finished with its views
/// in place under `options`, the options `ground` was ground under
/// ([`Kind::ground_options`]), where the grounding inlined any atom, a
/// view's or a chain's, or where [`may_decide`]; `conditioned` is
/// `written` rewritten for `decision` ([`Kind::rewrite`]) where `ground`
/// may decide, kept where the conditioning recorded decisions and made
/// enough atoms for a trial to take it ([`takes_conditioned`]). `closed`
/// takes less work on it: bb_altermu2_w11 2.29 G units against 14.4 G.
/// The full weekend circuit of the bit-blasting paper (98 303 flips) runs
/// out of memory under every other layout of the program as ground and
/// compiles under `decision` in 533 M work units.
pub(crate) fn ready(
    ground: &GroundProgram,
    kept: Option<ground::Simplified>,
    options: GroundOptions,
) -> (Ready, Option<usize>) {
    let Some(kept) = kept else {
        return (Ready::default(), None);
    };
    let decide = may_decide(ground);
    let written = (ground.stats.inlined + ground.stats.chained > 0 || decide)
        .then(|| ground::finish_kept(kept, &Kind::Decision.ground_options(options)));
    let Some(program) = written.as_ref().filter(|_| decide) else {
        return (
            Ready {
                written,
                ..Ready::default()
            },
            None,
        );
    };
    // `Kind::Decision.rewrite`, which `written`, read through no
    // indicator, takes in full, finished again only where a trial takes
    // it.
    let decided = ground::guarded_where(Kind::eliminated(program.clone()), |made| {
        takes_conditioned(made, ground)
    });
    let guarded = match &decided {
        Ok(gp) => gp.stats.guarded,
        Err(gp) => gp.stats.guarded,
    };
    let conditioned = decided.ok().filter(|gp| gp.stats.decisions.is_some());
    let ready = Ready {
        written,
        conditioned,
        laid: None,
    };
    (ready, Some(guarded))
}

/// Whether `decision` is a trial on `conditioned`, `ground` conditioned on
/// its guards: where the conditioning made at least as many atoms as
/// `ground` has derived ones ([`derived_unmerged`]). The full weekend
/// circuit of the bit-blasting paper makes 578 k for 160 k derived, and
/// compiles only under `decision`; the arithmetic circuits make fewer
/// (zeroone_w1: 960 for 7 238), and their `decision` compile does worse
/// than the other trials' or runs out of memory.
pub(crate) fn decides(conditioned: &GroundProgram, ground: &GroundProgram) -> bool {
    conditioned.stats.guarded >= derived_unmerged(ground)
}

/// Whether a trial takes `ground` conditioned on its guards, the
/// conditioning having made `made` atoms ([`ready`]): where
/// it made at least a quarter as many as `ground` has derived ones
/// ([`derived_unmerged`]). `decision` takes it where it made as many
/// ([`decides`]), and `program`, `closed` and `td` otherwise
/// ([`Programs::of`]): `closed` wins bb_normal_mixture_theta_w9 (37 186
/// made for 49 158) on it. The zeroone circuits make an eighth
/// (bb_zeroone_w2_w25: 2 794 for 21 896), and the conditioned program,
/// which the merge of equal gates does not follow (`ground::guarded`),
/// holds 14 499 derived atoms where the program as ground holds 9 972;
/// `auto` takes 27% longer with it.
pub(crate) fn takes_conditioned(made: usize, ground: &GroundProgram) -> bool {
    made * 4 >= derived_unmerged(ground)
}

/// The derived atoms of `ground` with those the merge of equal gates took
/// out counted (`GroundStats::reduced`): the conditioning counts what it
/// makes on the program before any merge (`ground::guarded`), and a merge
/// that leaves fewer atoms does not make the program a different shape.
/// Counted after the merge, the and-inverter graph's (`ground::aig`) made
/// `decision` a trial on bb_normal_mixture_theta_w9 (36 522 derived for
/// 37 186 conditioned) and bb_normal_mixture_mu2_w7, where it took 45 s
/// and 39 s of runs that `closed` and `families:0` win.
pub(crate) fn derived_unmerged(ground: &GroundProgram) -> usize {
    ground.num_derived() + ground.stats.reduced
}

/// The programs `auto`'s trials rewrite: the program as ground, written
/// without its linear recursion; the program ground with its views in
/// place, where the grounding inlined any or `decision` may be a trial
/// ([`ready`]); the program conditioned on its guards, where
/// the conditioning records decisions; and whether `decision` is a trial
/// on it ([`decides`]).
struct Programs {
    ground: GroundProgram,
    written: Option<GroundProgram>,
    conditioned: Option<GroundProgram>,
    decision: bool,
}

impl Programs {
    /// The program trial `kind` rewrites. The conditioned program, where
    /// there is one ([`takes_conditioned`]), goes to `decision` where it is
    /// a trial; where it is not, it goes to `program`, `closed` and `td`,
    /// whose layouts the rules' text does not set: `program`'s order is
    /// the one the program states its facts in, the same in both
    /// programs, and `closed` and `td` search theirs on
    /// the program they get (`dynamic`, whose layout the text does not set
    /// either, is a trial only where nothing is left to order, and never
    /// beside a conditioned program). `program` and `program:fanin` rewrite
    /// the program with its views in place otherwise
    /// ([`Kind::ground_options`]), and the others the program as ground.
    /// `program:fanin`
    /// reads its order off the rules, a walk through the atoms they read,
    /// and the conditioning rewrites what they read: a test of a number's
    /// bits becomes the cube of the flips on its path, so the walk no
    /// longer meets each number's flips together. In CUDD's model the
    /// conditioned addFun_max_w17 under the order the walk reads off it
    /// takes 29 times the recursive calls that either program takes under
    /// the order it reads off the program as ground, while `program` takes
    /// about the same work on both (8.85 G units against 8.64 G).
    /// `families:0` takes more work on the conditioned program, its
    /// indicators on every atom the conditioning made
    /// (bb_normal_mixture_mu2_w7: 1.07 G units over its turns against
    /// 133 M). Beside `decision` the order layouts get
    /// further on the conditioned program and hold their diagrams between
    /// turns: at 25 GB a process they starve bb_coinBias_w16's `decision`
    /// of memory in its query phase, and the run runs out of time.
    fn of(&self, kind: Kind) -> &GroundProgram {
        let conditioned = if self.decision {
            kind == Kind::Decision
        } else {
            matches!(
                kind,
                Kind::Program { fan_in: false } | Kind::Closed | Kind::Td
            )
        };
        match (&self.conditioned, &self.written) {
            (Some(program), _) if conditioned => program,
            (_, Some(program)) if kind == (Kind::Program { fan_in: false }) => program,
            _ => &self.ground,
        }
    }
}

/// What a run of trials starts from: the first watch's pairs, from which
/// the first targets follow, the live pairs at which a dynamic trial
/// searches first, and tididi's soft byte budget for every operation,
/// which refuses an operation as the allocator does when the host's
/// memory runs out. A run starts from the default; the tests start from
/// smaller ones, to make a small program take several turns or have its
/// operations refused.
struct Watch {
    first_pairs: usize,
    dynamic_first_pairs: usize,
    memory_bytes: Option<u64>,
}

impl Default for Watch {
    fn default() -> Self {
        Watch {
            first_pairs: FIRST_PAIRS,
            dynamic_first_pairs: tp::DYNAMIC_FIRST_PAIRS,
            memory_bytes: None,
        }
    }
}

/// Whether `auto` finishes `gp`'s grounding again for a `decision` trial
/// ([`ready`]): a program of at least [`DECISION_DERIVED`]
/// derived atoms to which the conditioning on guards applies and on which
/// it gets under way, as ground with its views inlined
/// ([`ground::guards_apply`]). The circuits a bit-blasting front end
/// builds are such programs; the small programs, which the other trials
/// finish before the conditioning would pay, are not. The grounding is
/// kept for it where it holds as many derived atoms before its views are
/// inlined ([`ground::ground_keeping`]).
pub(crate) fn may_decide(gp: &GroundProgram) -> bool {
    gp.num_derived() >= DECISION_DERIVED && ground::guards_apply(gp)
}

/// The derived atoms from which a program may have a `decision` trial
/// ([`may_decide`]).
pub(crate) const DECISION_DERIVED: usize = 4096;

/// Whether trials `a` and `b` compile one rewrite of one program: both
/// rewrite the program [`Programs::of`] gives them, and both or neither
/// read it through indicators. A `td` trial's layout may add separator
/// copies to its rewrite, and the layouts of `program` and `decision`
/// their names ([`Prepared::lay_out`], [`Trial::copied`]).
fn same_rewrite(a: &Trial, b: &Trial, programs: &Programs) -> bool {
    std::ptr::eq(programs.of(a.kind), programs.of(b.kind))
        && a.kind.indicators() == b.kind.indicators()
}

/// Gives trial `i` its rewrite of its program ([`Programs::of`]), written
/// without its linear recursion ([`Kind::rewrite_eliminated`]), taken from
/// an earlier trial of the same rewrite that compiles it as it is where
/// there is one. False where the trial is disabled, having nothing an
/// earlier trial lacks.
fn rewrite(trials: &mut [Trial], i: usize, programs: &Programs, ledger: &mut Ledger) -> bool {
    if trials[i].disabled {
        return false;
    }
    let program = programs.of(trials[i].kind);
    // A program consisting only of independent facts has no rule
    // interaction graph for td to improve. Closed already handles
    // its literal queries and evidence without that search.
    if program.rules.is_empty() && trials[i].kind == Kind::Td {
        trials[i].disabled = true;
        return false;
    }
    // An earlier trial of the same rewrite has its program, unless its
    // layout added copies to it.
    if trials[i].ground.is_none()
        && let Some((gp, deps)) = trials[..i]
            .iter()
            .find(|t| same_rewrite(t, &trials[i], programs) && !t.copied())
            .and_then(|t| Some((t.ground.clone()?, t.deps.clone())))
    {
        trials[i].ground = Some(gp);
        trials[i].deps = deps;
    }
    if trials[i].ground.is_none() {
        let start = Instant::now();
        let gp = trials[i].kind.rewrite_eliminated(program.clone());
        ledger.rewrite_seconds += start.elapsed().as_secs_f64();
        // No eligible boundary: the rewrite is the program as grounded.
        if trials[i].kind.indicators() && gp.encoding.is_empty() {
            trials[i].disabled = true;
            return false;
        }
        trials[i].ground = Some(gp);
    }
    true
}

/// Readies trial `i`: its rewrite ([`rewrite`]) and its layout. False
/// where the trial is disabled, having nothing an earlier trial lacks.
fn prepare(
    trials: &mut [Trial],
    i: usize,
    programs: &Programs,
    ledger: &mut Ledger,
) -> Result<bool, Error> {
    if !rewrite(trials, i, programs, ledger) {
        return Ok(false);
    }
    // Where `families:0`'s layout says no search would run on this program
    // ([`vtree::Estimate::search_units`]), the searched layout is
    // `families:0`'s, and the trial is disabled before its layout is built
    // rather than once it is found to be the same (below), which saves
    // building the same layout twice.
    if trials[i].built.is_none()
        && trials[i].kind == (Kind::Families { search: true })
        && trials[..i].iter().any(|t| {
            t.kind == (Kind::Families { search: false })
                && t.estimate.is_some_and(|e| e.search_units == 0)
        })
    {
        trials[i].disabled = true;
        return Ok(false);
    }
    if trials[i].built.is_none() {
        let t = &mut trials[i];
        // A layout that cannot be built, or cannot help (`td` where no
        // elimination finishes, [`vtree::build`]), disables its trial.
        let Prepared {
            ground,
            deps,
            mut layout,
            layout_seconds,
            ..
        } = match Prepared::lay_out(t.ground.take().unwrap(), t.deps.take(), t.kind, 0.0) {
            Err(Error::Vtree(why)) => {
                ledger.notes.push(format!(
                    "layout: {} indicators={} disabled: {why}",
                    t.kind,
                    t.kind.indicators_name()
                ));
                t.disabled = true;
                return Ok(false);
            }
            prepared => prepared?,
        };
        (t.ground, t.deps) = (Some(ground), Some(deps));
        book_layout(t, &mut layout, layout_seconds, ledger);
        // The searched families layout is the min-fill one where the
        // decomposer does not search, and the one-component trials
        // can agree. Comparing their rewrites too prevents confusing a
        // rewrite with the original just because their numbered trees match.
        if trials[..i].iter().any(|p| {
            same_rewrite(p, &trials[i], programs)
                && p.built.as_ref().is_some_and(|b| same_layout(b, &layout))
        }) {
            trials[i].disabled = true;
            return Ok(false);
        }
        trials[i].built = Some(layout);
    }
    Ok(true)
}

/// Books `layout`, built for trial `t` in `seconds`: its estimate is the
/// trial's, and its note and seconds go in the ledger.
fn book_layout(t: &mut Trial, layout: &mut vtree::Layout, seconds: f64, ledger: &mut Ledger) {
    t.estimate = layout.estimate;
    ledger.notes.extend(layout.note.take());
    ledger.vtree_seconds += seconds;
    ledger.notes.push(format!(
        "layout: {} indicators={}, {seconds:.3}s",
        t.kind,
        t.kind.indicators_name()
    ));
}

/// [`run`] over the given trials from `watch`.
fn run_within(
    programs: Programs,
    trials: Vec<Kind>,
    projection: Projection,
    watch: Watch,
    laid: Option<Box<Prepared>>,
) -> Result<Run, Error> {
    let Watch {
        first_pairs,
        dynamic_first_pairs,
        memory_bytes,
    } = watch;
    // The trials' rewrites start from the program written without its
    // linear recursion.
    let start = Instant::now();
    let programs = Programs {
        ground: Kind::eliminated(programs.ground),
        written: programs.written.map(Kind::eliminated),
        ..programs
    };
    let mut trials: Vec<_> = trials.into_iter().map(Trial::new).collect();
    let mut ledger = Ledger {
        rewrite_seconds: start.elapsed().as_secs_f64(),
        ..Ledger::default()
    };
    if let Some(Prepared {
        ground,
        deps,
        mut layout,
        rewrite_seconds,
        layout_seconds,
    }) = laid.map(|l| *l)
        && let Some(t) = trials
            .iter_mut()
            .find(|t| t.kind == (Kind::Families { search: false }))
    {
        ledger.rewrite_seconds += rewrite_seconds;
        book_layout(t, &mut layout, layout_seconds, &mut ledger);
        (t.ground, t.deps, t.built) = (Some(ground), Some(deps), Some(layout));
    }
    let first_work = (first_pairs.max(1) as u64).saturating_mul(WORK_PER_PAIR);
    let Some(lead) = trials.first().map(|t| t.kind) else {
        return Err(Error::Vtree("auto has no trial left".into()));
    };
    // Where `program` leads, whether the families trials hold their weight
    // turns on `families:0`'s rewrite ([`families_weighted`]). Its layout
    // waits for its first attempt, which a run that another trial answers
    // first never makes (dice_pmc1: 3.8 s for a layout of width 345, where
    // `program` answers in 0.3 s).
    let laid = trials
        .iter()
        .position(|t| t.kind == (Kind::Families { search: false }));
    if matches!(lead, Kind::Program { .. })
        && let Some(i) = laid
    {
        rewrite(&mut trials, i, &programs, &mut ledger);
    }
    let weighted = families_weighted(lead, laid.map(|i| &trials[i]).filter(|t| !t.disabled));
    for t in &mut trials {
        t.weight = weight(t.kind, weighted);
        t.target = first_work.saturating_mul(t.weight);
    }
    // Where `program` goes first, it and the two trials after it start
    // with short turns, so that none spends much more than its share.
    if matches!(lead, Kind::Program { .. }) {
        for t in trials.iter_mut().take(3) {
            t.target /= LEAD_SHARE;
        }
    }
    // Whether the searched families layout has been weighed against the
    // laid one ([`weigh_searched`]); once a run.
    let mut weighed = false;
    // The last refusal that retired a trial ([`Trial::retire`]), which the
    // run fails with once every trial is retired or disabled.
    let mut refused: Option<tididi::OperationError> = None;
    loop {
        // The next attempt goes to the trial projected to finish first, by
        // its target and its weight ([`Trial::key`]); the first in order on
        // a tie.
        let next = (0..trials.len())
            .filter(|&i| !trials[i].disabled)
            .min_by(|&i, &j| trials[i].key().total_cmp(&trials[j].key()));
        let Some(i) = next else {
            // Trials retired while another held memory, which is freed now,
            // run again, once each.
            let behind: Vec<usize> = (0..trials.len())
                .filter(|&j| trials[j].behind && !trials[j].revived)
                .collect();
            if !behind.is_empty() {
                for &j in &behind {
                    let t = &mut trials[j];
                    (t.disabled, t.revived) = (false, true);
                    ledger.notes.push(format!(
                        "revived: {} indicators={} ({} units), retired while another trial held memory",
                        t.kind,
                        t.kind.indicators_name(),
                        t.spent
                    ));
                }
                continue;
            }
            let error = refused.map_or_else(
                || Error::Vtree("auto has no trial left".into()),
                |e| Error::Trials {
                    error: tp::CompileError::Tdd(e),
                    notes: ledger.notes,
                },
            );
            return Err(error);
        };
        if !weighed && trials[i].kind == (Kind::Families { search: true }) {
            weighed = true;
            weigh_searched(&mut trials, &programs, &mut ledger)?;
            continue;
        }
        if !prepare(&mut trials, i, &programs, &mut ledger)? {
            continue;
        }
        let alone = trials.iter().enumerate().all(|(j, t)| j == i || t.disabled);
        let t = &mut trials[i];
        if let Some(projected) = t.projected().filter(|_| t.ahead() < PROJECTED_DOUBLINGS) {
            ledger.notes.push(format!(
                "turn: {} indicators={} projected to finish its {} at 2^{:.1} units, its target 2^{:.1}",
                t.kind,
                t.kind.indicators_name(),
                if t.querying { "query phase" } else { "compile" },
                projected,
                (t.target as f64).log2()
            ));
        }
        let left = t.target.saturating_sub(t.spent).max(1);
        let rent = if weighed { None } else { search_rent(t, left) };
        let bounds = Bounds {
            pairs: t.watch(first_pairs),
            work: rent.unwrap_or(left),
            projected_from: projected_from(first_pairs),
            horizon: (t.target.max(1) as f64).log2() + PROJECTED_DOUBLINGS,
        };
        let result = attempt(
            t,
            (!alone).then_some(bounds),
            projection,
            dynamic_first_pairs,
            memory_bytes,
            &mut ledger,
        );
        let error = match result {
            Ok((results, query_sizes)) => {
                let mut run = finish(t, results, query_sizes, ledger);
                run.spent = Spent(Some((trials, programs)));
                return Ok(run);
            }
            Err(error) => error,
        };
        t.record_progress();
        let Some(stop) = Stop::of(bounds, t.last_work, &error) else {
            if let tp::CompileError::Tdd(
                e @ (tididi::OperationError::OverBudget | tididi::OperationError::IndexOverflow),
            ) = error
            {
                // Memory refused goes to the trial refused it where it has
                // spent more than each trial that holds any, once: what
                // they built is freed, and it resumes at its next turn.
                let others: Vec<usize> = (0..trials.len())
                    .filter(|&j| j != i && trials[j].holds())
                    .collect();
                let leads = others.iter().all(|&j| trials[j].spent < trials[i].spent);
                if e == tididi::OperationError::OverBudget
                    && !others.is_empty()
                    && leads
                    && !trials[i].freed_for
                {
                    trials[i].freed_for = true;
                    let freed: Vec<String> = others
                        .iter()
                        .map(|&j| {
                            trials[j].release();
                            format!("{} ({} units)", trials[j].kind, trials[j].spent)
                        })
                        .collect();
                    ledger.notes.push(format!(
                        "released: {} for {} indicators={} ({} units), refused memory under a watch of {} pairs",
                        freed.join(", "),
                        trials[i].kind,
                        trials[i].kind.indicators_name(),
                        trials[i].spent,
                        bounds.pairs
                    ));
                    continue;
                }
                let t = &mut trials[i];
                ledger.notes.push(format!(
                    "retired: {} indicators={} under a watch of {} pairs: {e}",
                    t.kind,
                    t.kind.indicators_name(),
                    bounds.pairs
                ));
                t.retire();
                t.behind = e == tididi::OperationError::OverBudget && !others.is_empty();
                refused = Some(e);
                continue;
            }
            return Err(Error::Trials {
                error,
                notes: ledger.notes,
            });
        };
        match stop {
            // A formula past the watch: the trial's next attempt holds it.
            Stop::Size(pairs) => t.hold(pairs, first_pairs),
            // Its target spent, the trial's next doubles until it leaves
            // the trial more than twice the work the stop lost: the step
            // the stop cut short starts over, and each time it is cut
            // short its next try has at least twice the work (rus_20_40,
            // whose last join of 3.3 G units was otherwise stopped at
            // 2.1 G and given 2.1 G again).
            Stop::Work => {
                while t.target.saturating_sub(t.spent) <= t.lost.saturating_mul(2) {
                    t.target = t.target.saturating_mul(2);
                }
            }
            // Its compile projected past its horizon, the turn ends there as
            // one that spent its target.
            Stop::Projected => t.target = t.target.saturating_mul(2),
        }
        if t.weight > 1 && overspent_estimate(t) {
            ledger.notes.push(format!(
                "turn: {} indicators={} spent past its layout's estimate",
                t.kind,
                t.kind.indicators_name()
            ));
            t.weight = 1;
        }
        if !weighed && search_due(&trials[i]) {
            weighed = true;
            weigh_searched(&mut trials, &programs, &mut ledger)?;
        }
    }
}

/// Builds the searched families layout and weighs it against that of
/// `families:0`, once `families:0` has spent the search's charge
/// ([`search_due`]) or at the searched trial's own turn: where the
/// searched layout is estimated [`SEARCHED_GAP`] times smaller, its trial
/// takes `families:0`'s place in the order, just before it, with
/// [`FAMILIES_WEIGHT`] whichever trial leads, at the first target of that
/// weight, and so takes the next attempt, and `families:0` loses its
/// weight, the bet now being on the searched layout (pedigree51, where
/// `families:0` otherwise spent eight shares beside it; geni_router_24,
/// where it spent them until it was refused memory). Where `program`
/// leads the gap is the bet's evidence: on GeNI's routers, which `program`
/// leads and whose rewrite ties no table by clauses, the searched layout
/// wins, and at weight 1 the other trials' equal shares took router_20
/// from 12 s to 21 and router_24 past its memory.
fn weigh_searched(
    trials: &mut Vec<Trial>,
    programs: &Programs,
    ledger: &mut Ledger,
) -> Result<(), Error> {
    let laid = trials
        .iter()
        .position(|t| t.kind == (Kind::Families { search: false }) && !t.disabled);
    let searched = trials.iter().position(|t| {
        t.kind == (Kind::Families { search: true }) && !t.disabled && t.built.is_none()
    });
    let (Some(i), Some(j)) = (laid, searched) else {
        return Ok(());
    };
    if !prepare(trials, j, programs, ledger)? {
        return Ok(());
    }
    let pairs = |t: &Trial| t.estimate.map(|e| e.pairs);
    let (Some(laid), Some(searched)) = (pairs(&trials[i]), pairs(&trials[j])) else {
        return Ok(());
    };
    if searched * SEARCHED_GAP > laid {
        return Ok(());
    }
    ledger.notes.push(format!("turn: {} indicators={} takes the place of {}, its layout estimated at {searched:.3e} pairs against {laid:.3e}",
        trials[j].kind, trials[j].kind.indicators_name(), trials[i].kind));
    trials[i].weight = 1;
    let mut t = trials.remove(j);
    t.target = t.target / t.weight * FAMILIES_WEIGHT;
    t.weight = FAMILIES_WEIGHT;
    trials.insert(i, t);
    Ok(())
}

/// The bounds on an attempt beside other trials: the pairs of a formula it
/// completes (its watch; a formula past it stops the attempt by size),
/// which also bound its operations in flight ([`attempt`]), the engine
/// work it may spend, and the base-2 logarithm of the work past which a
/// projected finish stops its compile between components, once its trial
/// has spent `projected_from` ([`tp::Horizon`]).
#[derive(Clone, Copy)]
struct Bounds {
    pairs: usize,
    work: u64,
    projected_from: u64,
    horizon: f64,
}

/// Why an attempt stopped.
enum Stop {
    /// It spent its work.
    Work,
    /// Its compile was projected, between components, to finish past
    /// [`PROJECTED_DOUBLINGS`] doublings of its target ([`tp::Horizon`]).
    Projected,
    /// It needs a watch of these pairs: a formula it completed was past
    /// the watch, or an operation in flight past its nodes or pairs.
    Size(usize),
}

impl Stop {
    /// Why an attempt within `bounds` that spent `work` stopped with
    /// `error`, `None` where `error` is no stop.
    fn of(bounds: Bounds, work: u64, error: &tp::CompileError) -> Option<Stop> {
        match error {
            tp::CompileError::Oversize { pairs, .. }
            | tp::CompileError::QueryOversize { pairs } => Some(Stop::Size(*pairs)),
            tp::CompileError::Tdd(tididi::OperationError::Stopped) if work >= bounds.work => {
                Some(Stop::Work)
            }
            tp::CompileError::Projected { .. } => Some(Stop::Projected),
            tp::CompileError::Tdd(
                tididi::OperationError::OutputCap | tididi::OperationError::Stopped,
            ) => Some(Stop::Size(bounds.pairs.saturating_add(1))),
            _ => None,
        }
    }
}

/// One attempt of trial `t` within `bounds`: it compiles what is not
/// compiled yet, then runs the query phase, each resuming where its last
/// attempt stopped, and may spend the bounds' work, besides what settling
/// the names of `program` and `decision` spends (`tp::settle`). A formula
/// it completes past the watch stops it by size; an operation in flight past
/// twice the watch in nodes, or sixteen times it in pairs, is lost and
/// built again in a later turn. Its compile stops between components where
/// it is projected to finish more than [`PROJECTED_DOUBLINGS`] doublings
/// past its target ([`Bounds`], [`tp::Horizon`]): the turn cannot finish
/// it, and the other trials take theirs before it goes on. Records the
/// attempt in `ledger`.
///
/// Without bounds the attempt runs to its end, held only to
/// `memory_bytes`: the trial left alone, whose stops would only make it
/// build again the component, or the query operation, under way.
fn attempt(
    t: &mut Trial,
    bounds: Option<Bounds>,
    projection: Projection,
    dynamic_first_pairs: usize,
    memory_bytes: Option<u64>,
    ledger: &mut Ledger,
) -> Result<Answers, tp::CompileError> {
    let layout = t.built.as_ref().unwrap();
    let (gp, deps) = (
        t.ground.as_ref().unwrap(),
        t.deps
            .as_deref()
            .expect("a laid out trial has its program's deps"),
    );
    let (tree, order) = (layout.vtree.clone(), &layout.order);
    // A dynamic trial leaves its formulas on the vtree the search reached,
    // which its next attempt builds on.
    let mut reached = None;
    let kept = t.kept.take();
    let settled = t.evaluation.settled_work();
    let started = Instant::now();
    ledger.attempts += 1;
    let mut work = 0;
    let result = tree.context().run(|engine| {
        let start_work = engine.limits().work_units();
        let mut limits = LimitConfig::none().with_memory_budget_bytes(memory_bytes);
        if let Some(Bounds { pairs, work, .. }) = bounds {
            let stop = StopRules::default()
                .with_unconditional(Some(StopAt::WorkUnits(start_work.saturating_add(work))))
                .with_after_pairs(Some((
                    (pairs as u64).saturating_mul(16),
                    StopAt::WorkUnits(0),
                )));
            limits = limits
                .with_output_node_cap(Some((pairs as u64).saturating_mul(2)))
                .with_stop_rules(stop);
        }
        let _guard = engine.limits().scope(limits);
        let horizon = bounds.map(|b| tp::Horizon {
            origin: start_work.saturating_sub(t.spent),
            from: b.projected_from,
            limit: b.horizon,
            mark: t.target / 2,
        });
        let backend = tp::TddBackend::resume(engine, tree.clone(), kept);
        let evaluated = t.evaluation.run(
            gp,
            deps,
            order,
            &backend,
            t.kind,
            projection,
            dynamic_first_pairs,
            bounds.map(|b| b.pairs),
            horizon,
        );
        work = engine.limits().work_units().saturating_sub(start_work);
        // Settling within its budget is not charged to the trial's turn
        // (`tp::settle`).
        let charged = work - (t.evaluation.settled_work() - settled);
        t.spent = t.spent.saturating_add(charged);
        t.last_work = charged;
        t.lost = backend.lost();
        reached = Some(backend.vtree());
        t.kept = Some(backend.keep());
        evaluated
    });
    if let Some(reached) = reached {
        t.built.as_mut().unwrap().vtree = reached;
    }
    let seconds = started.elapsed().as_secs_f64();
    ledger.trial_seconds += seconds;
    let outcome = match &result {
        Ok(_) => "finished",
        Err(tp::CompileError::QueryOversize { .. } | tp::CompileError::Oversize { .. }) => "size",
        Err(tp::CompileError::Projected { .. }) => "projected",
        Err(_) => "stopped",
    };
    // The farthest finish the compile's horizon projected in the attempt.
    let reach = t
        .evaluation
        .compilation
        .as_ref()
        .and_then(|c| c.reach)
        .map(|r| format!(", reach 2^{r:.1}"))
        .unwrap_or_default();
    let phase = if t.evaluation.is_compiled() {
        "query"
    } else {
        "compile"
    };
    // A finished query phase has handed its answers over.
    let progress = t
        .evaluation
        .progress()
        .filter(|_| result.is_err())
        .map(|(done, all)| match t.evaluation.compilation.as_ref() {
            Some(c) => {
                let round = c
                    .rounds()
                    .map(|r| format!(" at round {r}"))
                    .unwrap_or_default();
                format!(", components {done}/{all}{round}")
            }
            None => format!(", steps {done}/{all}"),
        });
    let watch = bounds.map_or("none".to_string(), |b| b.pairs.to_string());
    ledger.notes.push(format!("trial: {} indicators={}, watch={watch}, work={work}, phase={phase}{}{reach}, {outcome}, {seconds:.3}s",
        t.kind, t.kind.indicators_name(), progress.unwrap_or_default()));
    result
}

/// What a run of trials built and no longer reads once it has its
/// answers: the trials, the winner's compiled formulas among them, and the
/// programs they rewrote. The [`Run`] holds it rather than freeing it piece
/// by piece as the run ends, so that a caller that ends the process with
/// the run can leave it to the exit, as `tddlog` does: freeing it is a
/// measurable part of a small program's run.
#[derive(Default)]
pub(crate) struct Spent(#[allow(dead_code)] Option<(Vec<Trial>, Programs)>);

/// The [`Run`] of trial `t`, whose last attempt finished with `results` and
/// `query_sizes`. The trial keeps its compiled formulas.
fn finish(
    t: &mut Trial,
    results: Option<Vec<QueryResult>>,
    query_sizes: query::QuerySizes,
    ledger: Ledger,
) -> Run {
    let compiled = t.evaluation.compiled.as_ref().unwrap();
    let ground = t.ground.take().unwrap();
    Run {
        evidence_probability: t.evaluation.evidence_probability(&ground),
        ground,
        vtree: t.built.as_ref().unwrap().vtree.clone(),
        kind: t.kind,
        vtree_seconds: ledger.vtree_seconds,
        attempts: ledger.attempts,
        vtree_notes: ledger.notes,
        rewrite_seconds: ledger.rewrite_seconds,
        wasted_seconds: (ledger.trial_seconds
            - compiled.stats.seconds
            - t.evaluation.query_seconds)
            .max(0.0),
        query_seconds: t.evaluation.query_seconds,
        query_sizes,
        stats: compiled.stats.clone(),
        results,
        spent: Spent::default(),
    }
}
