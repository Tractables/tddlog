//! The diagram operations the compiler needs, on tididi's TDDs; sizes are
//! pairs. Each takes or builds its formulas on a scope ([`super::scope`]).

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::sync::Arc;

use tididi::diagram::WeightStore;
use tididi::diagram::{
    ChildPair, EncodedChildRef, NEG_LEAF_IDX, NodeIdx, ONE_LEAF_IDX, POS_LEAF_IDX, TddNodeId,
};
use tididi::limits::StopAt;
use tididi::reduce::ReductionPlan;
use tididi::restructure::EmbedError;
use tididi::restructure::search::{PoolRestarts, PoolSearchConfig, PoolSearchStats};
use tididi::vtree::{VarId, VtreeIdx};
use tididi::{Engine, Literal, OperationError, SkipTdd, Tdd, Vtree};

use super::CompileError;
use super::projector::Projector;
use super::scope::VtreeScope;
use crate::eval::{Count, Probabilities, Weighing};

/// The form a backend keeps its diagrams in, fixed when it is made.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Form {
    /// tididi's stored form, a node at every level a function's paths
    /// cross.
    #[default]
    Stored,
    /// The skip form (`--skip-levels`), whose pairs skip a level where
    /// its other child is a single variable the function ignores: on a
    /// right-linear vtree the reduced ordered BDD, where the stored form
    /// is the quasi-reduced one. Always canonical, so it takes no
    /// reduction.
    Skip,
}

/// A diagram in its backend's [`Form`]. Every formula a backend keeps is
/// in its form, so the operands of an operation share it; a diagram a
/// builder makes is stored, and the backend converts it at birth
/// ([`TddBackend::born`]).
#[derive(Clone, Debug)]
pub(crate) enum Diagram {
    Stored(Tdd),
    Skip(SkipTdd),
}

impl Diagram {
    /// The vtree the diagram is decomposed along.
    pub(crate) fn vtree(&self) -> &Arc<Vtree> {
        match self {
            Diagram::Stored(f) => f.vtree(),
            Diagram::Skip(f) => f.vtree(),
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
        match self {
            Diagram::Stored(f) => f.is_zero(),
            Diagram::Skip(f) => f.is_zero(),
        }
    }

    /// The pairs held over all levels.
    pub(crate) fn pair_count(&self) -> usize {
        match self {
            Diagram::Stored(f) => f.pair_count(),
            Diagram::Skip(f) => f.pair_count(),
        }
    }

    /// The nodes held over all levels.
    pub(crate) fn node_count(&self) -> usize {
        match self {
            Diagram::Stored(f) => f.node_count(),
            Diagram::Skip(f) => f.node_count(),
        }
    }

    /// The attached weights.
    pub(crate) fn weights(&self) -> Option<&WeightStore> {
        match self {
            Diagram::Stored(f) => f.weights(),
            Diagram::Skip(f) => f.weights(),
        }
    }

    /// The stored diagram, for `what`, which reads stored levels: refused
    /// in skip form.
    pub(crate) fn stored(&self, what: &'static str) -> Result<&Tdd, CompileError> {
        match self {
            Diagram::Stored(f) => Ok(f),
            Diagram::Skip(_) => Err(CompileError::SkipForm(what)),
        }
    }

    /// [`Diagram::stored`], mutably.
    pub(crate) fn stored_mut(&mut self, what: &'static str) -> Result<&mut Tdd, CompileError> {
        match self {
            Diagram::Stored(f) => Ok(f),
            Diagram::Skip(_) => Err(CompileError::SkipForm(what)),
        }
    }

    /// The skip-form diagram of a skip backend's operation.
    fn skip(&self) -> &SkipTdd {
        match self {
            Diagram::Skip(f) => f,
            Diagram::Stored(_) => unreachable!("a skip backend's formulas are in skip form"),
        }
    }
}

impl From<Diagram> for Cow<'_, Diagram> {
    fn from(f: Diagram) -> Self {
        Cow::Owned(f)
    }
}

impl<'a> From<&'a Diagram> for Cow<'a, Diagram> {
    fn from(f: &'a Diagram) -> Self {
        Cow::Borrowed(f)
    }
}

/// A stored backend's operand, lent as tididi's stored operations take it.
fn stored_cow(f: Cow<'_, Diagram>) -> Cow<'_, Tdd> {
    match f {
        Cow::Owned(Diagram::Stored(f)) => Cow::Owned(f),
        Cow::Borrowed(Diagram::Stored(f)) => Cow::Borrowed(f),
        Cow::Owned(Diagram::Skip(_)) | Cow::Borrowed(Diagram::Skip(_)) => {
            unreachable!("a stored backend's formulas are stored")
        }
    }
}

/// A formula and the scope it lives on.
pub(crate) type Formula = (Diagram, VtreeScope);

/// A formula an operation reads: its own, which it consumes, or one a
/// store keeps ([`super::compile::Formulas`]), which a conjunction reads in
/// place, copying only the levels its result would take, and any other
/// operation copies.
pub(crate) struct Read {
    owned: Option<Diagram>,
    shared: Option<Arc<Diagram>>,
}

impl From<Diagram> for Read {
    fn from(f: Diagram) -> Self {
        Read {
            owned: Some(f),
            shared: None,
        }
    }
}

impl std::ops::Deref for Read {
    type Target = Diagram;

    fn deref(&self) -> &Diagram {
        match (&self.owned, &self.shared) {
            (Some(f), _) => f,
            (None, Some(f)) => f,
            (None, None) => unreachable!("a read is owned or shared"),
        }
    }
}

impl Read {
    /// A shared read of `f`.
    pub(crate) fn shared(f: Arc<Diagram>) -> Self {
        Read {
            owned: None,
            shared: Some(f),
        }
    }

    /// A stored formula taken from its store: owned where no read of it
    /// is outstanding.
    pub(crate) fn taken(f: Arc<Diagram>) -> Self {
        Arc::try_unwrap(f).map_or_else(Read::shared, Read::from)
    }

    /// The formula for an operation to consume: itself, or a copy of a
    /// shared one ([`TddBackend::copy`]).
    pub(crate) fn owned(self, backend: &TddBackend) -> Result<Diagram, CompileError> {
        match (self.owned, self.shared) {
            (Some(f), _) => Ok(f),
            (None, Some(f)) => Arc::try_unwrap(f).or_else(|f| backend.copy(&f)),
            (None, None) => unreachable!("a read is owned or shared"),
        }
    }

    /// Run `op` on the formula, owned where no other read holds it and
    /// borrowed otherwise, where [`owned`](Self::owned) would copy it.
    pub(crate) fn with_cow<R>(self, op: impl FnOnce(Cow<'_, Diagram>) -> R) -> R {
        match (self.owned, self.shared) {
            (Some(f), _) => op(Cow::Owned(f)),
            (None, Some(f)) => match Arc::try_unwrap(f) {
                Ok(f) => op(Cow::Owned(f)),
                Err(f) => op(Cow::Borrowed(&f)),
            },
            (None, None) => unreachable!("a read is owned or shared"),
        }
    }

    /// The formula as a conjunction takes it, with the store's hold on a
    /// shared one, which the borrow reads through ([`lend`]).
    fn split(self) -> (Option<Diagram>, Option<Arc<Diagram>>) {
        (self.owned, self.shared)
    }
}

/// An operand of a conjunction split from a [`Read`]: the owned formula,
/// or a borrow of the shared one.
fn lend(owned: Option<Diagram>, shared: &Option<Arc<Diagram>>) -> Cow<'_, Diagram> {
    match (owned, shared) {
        (Some(f), _) => Cow::Owned(f),
        (None, Some(f)) => Cow::Borrowed(f),
        (None, None) => unreachable!("a read is owned or shared"),
    }
}

/// The [`Read`] a refused conjunction hands back of an operand [`lend`]
/// made.
fn unlend(back: Cow<'_, Tdd>, shared: &Option<Arc<Diagram>>) -> Read {
    match back {
        Cow::Owned(f) => Read::from(Diagram::Stored(f)),
        Cow::Borrowed(_) => Read::shared(Arc::clone(
            shared.as_ref().expect("a borrow of a shared read"),
        )),
    }
}

/// A [`Read`] and the scope it lives on.
pub(crate) struct Part(pub(crate) Read, pub(crate) VtreeScope);

impl From<Formula> for Part {
    fn from((f, scope): Formula) -> Self {
        Part(Read::from(f), scope)
    }
}

impl Part {
    /// The formula for an operation to consume ([`Read::owned`]).
    pub(crate) fn owned(self, backend: &TddBackend) -> Result<Formula, CompileError> {
        Ok((self.0.owned(backend)?, self.1))
    }
}

/// How completely [`TddBackend::minimize`] reduces.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Reduce {
    /// Between operations, where a backend may reduce less than fully.
    Between,
    /// The formula an atom finishes with, which the compiler measures,
    /// stores and hands to `auto`'s watch. Always the full plan: a
    /// diagram carrying dead pairs would make the run report a size it does
    /// not have and the watch trip on pairs that are not there. Also a
    /// product that has outgrown its operands, whose dead pairs the next
    /// product would multiply.
    Settled,
    /// Between the clauses of one conjunction: inner-twin contraction
    /// alone, outside the prune schedule.
    Contract,
    /// The prune alone, without twin contraction: the evidence walk's
    /// joins between the full plans of its own `FullSchedule`.
    Prune,
}

/// tididi TDDs, run through an execution context's engine.
///
/// `vtree` is the global vtree, the one the strategy built until
/// `TddBackend::restructure` replaces it; a projected scope is a projection
/// of it and shares its execution context, so one engine serves every scope
/// (tididi `Context`: bind several vtrees to one context when their
/// operations should reuse the same scratch).
pub(crate) struct TddBackend<'a> {
    pub(crate) engine: &'a Engine,
    /// The form of every formula the backend keeps.
    form: Form,
    vtree: RefCell<Arc<Vtree>>,
    prune: Cell<FullSchedule>,
    /// Reused global-to-local renaming for scope projection. Only the
    /// selected entries are filled and cleared between projections.
    projector: RefCell<Projector>,
    /// The arenas of [`TddBackend::none_of`]'s builds, kept between them.
    none_of: RefCell<super::none_of::Scratch>,
    /// The work clock where the phase under way last kept a step
    /// ([`TddBackend::kept`]).
    kept_at: Cell<u64>,
    /// The work clock at which `auto`'s turn ends ([`TddBackend::pause`]),
    /// `None` outside a turn.
    turn_end: Cell<Option<u64>>,
    /// `auto`'s probe of the trial's form ([`TddBackend::probe`]), and
    /// the trial's clock: the work clock at the run's start, and what the
    /// trial had spent before the run.
    probe: Cell<Probe>,
    spent: Cell<(u64, u64)>,
}

/// `auto`'s probe of a trial's form: a stored trial's first large formula,
/// and where that keeps few pairs in skip form, the work of either form over
/// the same stretch of the compile, which decides whether the trial goes on
/// in skip form ([`TddBackend::probe`], [`TddBackend::fork`],
/// [`TddBackend::at_step`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Probe {
    /// No probe.
    #[default]
    Off,
    /// The first formula of `from` pairs or more is measured, and where it
    /// keeps under `below` of its pairs in skip form, the stored compile
    /// goes on over a stretch ([`Probe::Stretch`]).
    Armed { from: usize, below: f64 },
    /// The formula measured, which keeps most of its pairs in skip form:
    /// its pairs, stored and in skip form.
    Measured { pairs: usize, skip: usize },
    /// The formula measured kept few pairs, and the compile's copy in skip
    /// form did not fit the memory budget: the trial stays stored
    /// ([`TddBackend::fork`]).
    Refused,
    /// The formula measured kept few pairs in skip form: at its next step
    /// that holds only finished components the compile makes a copy of
    /// itself in skip form, for `converted` work ([`TddBackend::fork`]),
    /// and goes on stored from there, `start`, where it had finished that
    /// many components and the trial had spent that work, until the
    /// trial's work has doubled; then it stops with
    /// [`CompileError::Skips`].
    Stretch {
        pairs: usize,
        skip: usize,
        start: Option<(usize, u64)>,
        converted: u64,
    },
    /// The trial gone on with the copy in skip form, from the step with
    /// `from` components finished: its work from there, `at`, to the
    /// first step with `to`, and the copy's `converted`, is weighed against
    /// the stored compile's `stored` over the same components. Where the
    /// skip form spends as much, the compile stops with
    /// [`CompileError::Stored`].
    Duel {
        from: usize,
        to: usize,
        stored: u64,
        converted: u64,
        at: Option<u64>,
    },
    /// The skip form spent `skip` over the stretch, its conversion
    /// included, under the stored form's `stored`, and the trial goes on
    /// in it from where its clock read `at`.
    Weighed { skip: u64, stored: u64, at: u64 },
}

impl Probe {
    /// The probe at a step of the compile with `done` components finished,
    /// where the trial has spent `work`, and the stop of the compile there,
    /// if any. A stretch begins where the compile makes its copy in skip
    /// form ([`TddBackend::fork`]).
    pub(crate) fn at_step(self, done: usize, work: u64) -> (Probe, Option<CompileError>) {
        match self {
            Probe::Stretch {
                pairs,
                skip,
                start: Some((from, at)),
                converted,
            } if done > from && work >= at.saturating_mul(2) => (
                self,
                Some(CompileError::Skips {
                    pairs,
                    skip,
                    from,
                    to: done,
                    stored: work - at,
                    converted,
                }),
            ),
            Probe::Duel {
                from,
                to,
                stored,
                converted,
                at: None,
            } if done >= from => (
                Probe::Duel {
                    from,
                    to,
                    stored,
                    converted,
                    at: Some(work),
                },
                None,
            ),
            // The skip form loses as soon as it has spent what the stored
            // form spent over the whole stretch, its conversion included.
            Probe::Duel {
                to,
                stored,
                converted,
                at: Some(at),
                ..
            } if done >= to || work.saturating_sub(at).saturating_add(converted) >= stored => {
                debug_assert!(work >= at, "the trial's clock runs back");
                let skip = work.saturating_sub(at).saturating_add(converted);
                if skip < stored {
                    (
                        Probe::Weighed {
                            skip,
                            stored,
                            at: work,
                        },
                        None,
                    )
                } else {
                    (self, Some(CompileError::Stored { skip, stored }))
                }
            }
            p => (p, None),
        }
    }
}

/// The units of the work clock a skip operation is charged for each unit
/// the skip engine charges itself ([`TddBackend::skip_op`]), so that the
/// clock runs in the stored engine's units in either form. On the
/// circuits, the ratio of the two engines' rates in units a second is 10
/// at the median of `program`'s compiles of 49 circuits, 7 and 12.5 at the
/// quartiles, and 17 at the median of `program:fanin`'s of 24, where the
/// skip form keeps the fewest pairs. The larger is taken, so that a turn in
/// skip form takes no more time than one in the stored form, where
/// `auto`'s trials race by work, and so that the work of the two forms
/// weighed over a stretch ([`Probe`]) compares their times where the skip
/// form keeps few pairs.
const SKIP_WORK_SCALE: u64 = 16;

/// When a minimization between operations runs tididi's full plan, and
/// when a partial one. `Reduce::Between` keeps one per backend; the evidence
/// walk keeps its own, which outlives the trial's backend.
///
/// `Reduce::Between` takes inner-twin contraction alone between full
/// plans. The prune walks the diagram to find what nothing reaches, so it
/// costs the diagram's size whether or not there is anything to remove, and
/// the programs differ on whether there is: the compile of a Bayesian
/// network or a grid spends much of its time on prunes that free nothing,
/// while UW-CSE's formulas carry many more pairs without them and every
/// later operation pays for those (`docs/design.md`, minimization). So its
/// schedule widens while
/// a full minimization frees at most [`PRUNE_KEPT`] of the pairs it was
/// given and snaps back to every call as soon as one frees more.
///
/// The evidence walk takes the prune alone (`Reduce::Prune`) between full
/// plans, and its schedule widens only while a full minimization frees
/// nothing at all; see `EvidenceWalk` in `query/walk.rs`.
///
/// The test is the whole minimization's yield rather than the skipped
/// pass's own, which keeps it to one pair count and one plan: a
/// minimization that freed almost nothing cannot have had a pass in it that
/// freed much.
#[derive(Clone, Copy, Default)]
pub(crate) struct FullSchedule {
    /// Minimizations since the last full one.
    since: u32,
    /// Minimizations to take with the partial plan before the next full one.
    stride: u32,
}

impl FullSchedule {
    /// Whether this minimization runs the full plan. A default schedule is
    /// due at once, so the first minimization always runs it.
    pub(crate) fn full_is_due(&self) -> bool {
        self.since >= self.stride
    }

    pub(crate) fn took_partial(&mut self) {
        self.since += 1;
    }

    /// What a full minimization that freed `freed` pairs does to the
    /// schedule, where freeing `kept` or fewer counts as almost nothing.
    pub(crate) fn observed(&mut self, freed: usize, kept: usize) {
        self.since = 0;
        self.stride = if freed <= kept {
            (self.stride * 2 + 1).min(STRIDE_CAP)
        } else {
            0
        };
    }
}

/// The widest a schedule goes. Sixteen leaves a full minimization in every
/// seventeenth call, which is often enough that a formula that starts
/// carrying dead pairs is caught before the watch reads it.
const STRIDE_CAP: u32 = 16;
/// The share of a diagram's pairs a full minimization between operations
/// must free to count as worth its walk, as a right shift: 1/32.
const PRUNE_KEPT: u32 = 5;

/// A backend's vtree with its projector, which a backend after it over the
/// same vtree takes over ([`TddBackend::resume`]): `auto` makes a backend
/// per attempt of a trial, which would otherwise project the trial's vtree
/// again.
pub(crate) struct KeptProjector {
    vtree: Arc<Vtree>,
    projector: Projector,
}

/// The run's stop and the end of its turn moved later
/// ([`TddBackend::stop_later`]), until dropped.
pub(crate) struct Later<'b> {
    _stop: Option<tididi::limits::LimitScope<'b>>,
    turn_end: &'b Cell<Option<u64>>,
    work: u64,
}

impl Drop for Later<'_> {
    fn drop(&mut self) {
        self.turn_end
            .set(self.turn_end.get().map(|at| at.saturating_sub(self.work)));
    }
}

impl<'a> TddBackend<'a> {
    pub(crate) fn new(engine: &'a Engine, vtree: Arc<Vtree>) -> Self {
        Self::resume(engine, vtree, None, Form::Stored)
    }

    /// A backend over `vtree` whose formulas are in `form`.
    pub(crate) fn in_form(engine: &'a Engine, vtree: Arc<Vtree>, form: Form) -> Self {
        TddBackend {
            form,
            ..Self::new(engine, vtree)
        }
    }

    /// A backend over `vtree` whose formulas are in `form`, with the
    /// projector of `kept` where that is over the same vtree.
    pub(crate) fn resume(
        engine: &'a Engine,
        vtree: Arc<Vtree>,
        kept: Option<KeptProjector>,
        form: Form,
    ) -> Self {
        let projector = match kept {
            Some(kept) if Arc::ptr_eq(&kept.vtree, &vtree) => kept.projector,
            _ => Projector::new(&vtree),
        };
        TddBackend {
            engine,
            form,
            vtree: RefCell::new(vtree),
            prune: Cell::new(FullSchedule::default()),
            projector: RefCell::new(projector),
            none_of: RefCell::default(),
            kept_at: Cell::new(engine.limits().work_units()),
            turn_end: Cell::new(None),
            probe: Cell::new(Probe::Off),
            spent: Cell::new((0, 0)),
        }
    }

    /// The vtree the backend ends on, with its projector, for the next
    /// backend over it.
    pub(crate) fn keep(self) -> KeptProjector {
        KeptProjector {
            vtree: self.vtree.into_inner(),
            projector: self.projector.into_inner(),
        }
    }

    /// The global vtree.
    #[must_use]
    pub(crate) fn vtree(&self) -> Arc<Vtree> {
        Arc::clone(&self.vtree.borrow())
    }

    /// The form of the backend's formulas.
    pub(crate) fn form(&self) -> Form {
        self.form
    }

    /// Arms `probe` ([`Self::probe`], [`Self::at_step`]), for a trial
    /// that had spent `spent` where the run's work clock read `start`. The
    /// engine's work clock starts again with each run, so the probe's
    /// clock is `spent` and the work since `start`, which runs on across
    /// the trial's turns.
    pub(crate) fn with_probe(self, probe: Probe, start: u64, spent: u64) -> Self {
        debug_assert!(match probe {
            Probe::Armed { .. } | Probe::Stretch { .. } => self.form == Form::Stored,
            Probe::Duel { .. } => self.form == Form::Skip,
            _ => true,
        });
        self.probe.set(probe);
        self.spent.set((start, spent));
        self
    }

    /// The probe as it stands.
    pub(crate) fn probed(&self) -> Probe {
        self.probe.get()
    }

    /// Measures `f`, an atom's settled formula of `pairs` pairs, where it
    /// is the first that reaches the armed probe: its pairs in skip form,
    /// by one conversion. Where they are under the probe's share of
    /// `pairs`, the compile goes on over a stretch ([`Probe::Stretch`]).
    pub(crate) fn probe(&self, f: &Diagram, pairs: usize) -> Result<(), CompileError> {
        let Probe::Armed { from, below } = self.probe.get() else {
            return Ok(());
        };
        if pairs < from {
            return Ok(());
        }
        let stored = f.stored("the skip probe")?;
        let skip = self.skip_op(|e| e.to_skip(stored))?.pair_count();
        self.probe.set(if (skip as f64) < below * pairs as f64 {
            Probe::Stretch {
                pairs,
                skip,
                start: None,
                converted: 0,
            }
        } else {
            Probe::Measured { pairs, skip }
        });
        Ok(())
    }

    /// The work the trial has spent: what it had spent before the run, and
    /// the work since ([`Self::with_probe`]).
    fn trial_work(&self) -> u64 {
        let (start, spent) = self.spent.get();
        spent + self.work_units().saturating_sub(start)
    }

    /// The compile's copy of itself in skip form, made by `copy` at a step
    /// with `done` components finished, where the probe's stretch is yet to
    /// begin ([`Probe::Stretch`]); `None` elsewhere. `copy` gives the copy
    /// and the pairs its conversion read ([`Self::skip_form_reads`]). The
    /// stretch begins after the copy, whose work, those pairs and what the
    /// clock counted, is set apart as the skip form's in the weighing. A
    /// copy that does not fit the memory budget leaves the trial stored
    /// ([`Probe::Refused`]).
    pub(crate) fn fork<T>(
        &self,
        done: usize,
        copy: impl FnOnce() -> Result<(T, u64), CompileError>,
    ) -> Result<Option<T>, CompileError> {
        let Probe::Stretch {
            pairs,
            skip,
            start: None,
            ..
        } = self.probe.get()
        else {
            return Ok(None);
        };
        let before = self.work_units();
        let (copy, read) = match copy() {
            Ok(copy) => copy,
            Err(CompileError::Tdd(
                tididi::OperationError::OverBudget | tididi::OperationError::IndexOverflow,
            )) => {
                self.probe.set(Probe::Refused);
                return Ok(None);
            }
            Err(e) => return Err(e),
        };
        self.probe.set(Probe::Stretch {
            pairs,
            skip,
            start: Some((done, self.trial_work())),
            converted: (self.work_units() - before).saturating_add(read),
        });
        Ok(Some(copy))
    }

    /// The compile at a step it has kept ([`Self::kept`]), with `done` of
    /// its components finished: the probe's stretch ends and its duel
    /// begins and ends at steps, and a stop there goes on later as from the
    /// end of a turn ([`Probe::at_step`]).
    pub(crate) fn at_step(&self, done: usize) -> Result<(), CompileError> {
        let (probe, stop) = self.probe.get().at_step(done, self.trial_work());
        self.probe.set(probe);
        stop.map_or(Ok(()), Err)
    }

    /// The skip engine's operation `op`, the work clock charged
    /// [`SKIP_WORK_SCALE`] times what it charged itself. The run's stop in
    /// work units, where it lies ahead, stops `op` once its own units, so
    /// scaled, reach it: `auto`'s guard cuts a skip operation where it cuts
    /// a stored one. A stop the charge reaches after `op` succeeded is the
    /// next operation's: `op`'s result is kept.
    fn skip_op<T, E>(&self, op: impl FnOnce(&Engine) -> Result<T, E>) -> Result<T, E> {
        let limits = self.engine.limits();
        let at = limits.mark();
        let now = limits.work_units();
        let rules = limits.armed().stop_rules();
        let scope = match rules.unconditional {
            Some(StopAt::WorkUnits(u)) if u > now => {
                let sooner = now + (u - now).div_ceil(SKIP_WORK_SCALE);
                Some(limits.edit(|c| {
                    c.with_stop_rules(rules.with_unconditional(Some(StopAt::WorkUnits(sooner))))
                }))
            }
            _ => None,
        };
        let result = op(self.engine);
        drop(scope);
        let more = limits.work_since(at).saturating_mul(SKIP_WORK_SCALE - 1);
        let _ = limits.poll_host_work(more);
        result
    }
}

impl TddBackend<'_> {
    /// The lowest node of the program's vtree holding every variable of
    /// `scope`: the lowest common ancestor of its first and last leaves in
    /// pre-order, from the projector's table, rather than `Vtree::lca`
    /// folded over the leaves, which walks parent links per leaf.
    pub(crate) fn lowest_over(&self, scope: &VtreeScope) -> VtreeIdx {
        let vtree = self.vtree.borrow();
        match scope.vars() {
            None => vtree.root(),
            Some(vars) => self.projector.borrow().lowest_over(vars),
        }
    }

    /// The lowest node of the program's vtree holding every one of `vars`,
    /// which is not empty.
    pub(crate) fn lowest_over_vars(&self, vars: &[VarId]) -> VtreeIdx {
        self.projector.borrow().lowest_over(vars)
    }

    fn renamed(scope: &VtreeScope, lit: Literal) -> Result<Literal, CompileError> {
        match scope.local_of(lit.var) {
            Some(v) => Ok(Literal::new(v, lit.sign)),
            None => Err(CompileError::NotInScope(lit.var)),
        }
    }

    /// The variable of `into` that each local leaf of `from` goes to.
    ///
    /// Both scopes' variables are ascending, so each is looked up after the
    /// last one found, by a search that doubles its step from there: a
    /// search of all of `into`'s variables for each cost a scope a binary
    /// search per variable however close together they were.
    pub(crate) fn scope_map(
        from: &VtreeScope,
        into: &VtreeScope,
    ) -> Result<Vec<VarId>, CompileError> {
        // Local leaf i + 1 of `from` carries program variable from.vars[i].
        let Some(vars) = from.vars() else {
            return Err(CompileError::NotInScope(VarId(0)));
        };
        let Some(held) = into.vars() else {
            return Ok(vars.to_vec());
        };
        let mut map = Vec::with_capacity(vars.len());
        // `held[..at]` is below the next variable.
        let mut at = 0;
        for &v in vars {
            let (mut lo, mut step) = (at, 1);
            while lo + step <= held.len() && held[lo + step - 1] < v {
                lo += step;
                step *= 2;
            }
            let hi = (lo + step).min(held.len());
            match held[lo..hi].binary_search(&v) {
                Ok(i) => {
                    map.push(VarId((lo + i) as u32 + 1));
                    at = lo + i + 1;
                }
                Err(_) => return Err(CompileError::NotInScope(v)),
            }
        }
        Ok(map)
    }

    /// A copy of `f`, for an operation to consume while the original is
    /// kept. A copy the allocator cannot serve is refused, as an operation
    /// is (`OverBudget`), where `Clone` would abort the process.
    pub(crate) fn copy(&self, f: &Diagram) -> Result<Diagram, CompileError> {
        Ok(match f {
            Diagram::Stored(f) => Diagram::Stored(self.engine.try_clone(f)?),
            // Its levels are shared until written.
            Diagram::Skip(f) => Diagram::Skip(f.clone()),
        })
    }

    /// `f`, which a builder made, in the backend's form: converted at
    /// birth in skip form. A level held as its description (rule 9 of the
    /// stored form) is refused there, since the skip form has no
    /// description and each birth would write out its pairs.
    pub(crate) fn born(&self, f: Tdd) -> Result<Diagram, CompileError> {
        match self.form {
            Form::Stored => Ok(Diagram::Stored(f)),
            Form::Skip if implicit_pairs(&f) > 0 => Err(CompileError::SkipForm(
                "a built diagram with a level held as its description",
            )),
            Form::Skip => self.in_skip_form(&Diagram::Stored(f)),
        }
    }

    /// The pairs converting `f` into skip form reads: one pass over a
    /// stored diagram's pairs, those of a level held as its description
    /// among them (`tididi::Engine::to_skip`), which the engine's work
    /// clock does not count, and none for one in skip form.
    pub(crate) fn skip_form_reads(&self, f: &Diagram) -> u64 {
        match f {
            Diagram::Stored(f) => f.pair_count() as u64,
            Diagram::Skip(_) => 0,
        }
    }

    /// `f` in skip form, converted where it is stored
    /// ([`Self::skip_form_reads`]).
    pub(crate) fn in_skip_form(&self, f: &Diagram) -> Result<Diagram, CompileError> {
        match f {
            Diagram::Stored(f) => Ok(Diagram::Skip(self.skip_op(|e| e.to_skip(f))?)),
            Diagram::Skip(f) => Ok(Diagram::Skip(f.clone())),
        }
    }

    /// Embed a stored formula without first copying its source tree. The
    /// embedding itself copies levels into the destination tree; only an
    /// unchanged scope needs a copy to produce an owned operand.
    pub(crate) fn embed_ref(
        &self,
        f: &Diagram,
        from: &VtreeScope,
        into: &VtreeScope,
    ) -> Result<Diagram, CompileError> {
        if from.same(into) {
            return self.copy(f);
        }
        let map = Self::scope_map(from, into)?;
        let at = |v: VarId| map[v.0 as usize - 1];
        Ok(match f {
            Diagram::Stored(f) => Diagram::Stored(self.engine.embed(f, into.vtree(), at)?.0),
            Diagram::Skip(f) => Diagram::Skip(self.skip_op(|e| e.skip_embed(f, into.vtree(), at))?),
        })
    }

    /// `f` copied onto `into` with each variable of its vtree read as its
    /// image under `map`; `None` where `into` does not hold its shape under
    /// that renaming, or up to mirrors where `mirrored` says so.
    fn embed_mapped(
        &self,
        f: &Diagram,
        into: &VtreeScope,
        map: impl Fn(VarId) -> VarId,
        mirrored: bool,
    ) -> Result<Option<Diagram>, CompileError> {
        let embedded = match (f, mirrored) {
            (Diagram::Stored(f), true) => self
                .engine
                .embed_mirrored(f, into.vtree(), map)
                .map(|(f, _)| Diagram::Stored(f)),
            (Diagram::Stored(f), false) => self
                .engine
                .embed(f, into.vtree(), map)
                .map(|(f, _)| Diagram::Stored(f)),
            (Diagram::Skip(f), true) => self
                .engine
                .skip_embed_mirrored(f, into.vtree(), map)
                .map(Diagram::Skip),
            (Diagram::Skip(f), false) => self
                .engine
                .skip_embed(f, into.vtree(), map)
                .map(Diagram::Skip),
        };
        match embedded {
            Ok(embedded) => Ok(Some(embedded)),
            Err(EmbedError::NotIsomorphic { .. } | EmbedError::Vtree(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Embed a formula by moving its levels into the destination tree, and
    /// hand it back unchanged with the error when that is refused, boxed: a
    /// `Tdd` is 200 bytes and the error 48. The result may keep a few
    /// unread leaf wrappers until it is minimized.
    ///
    /// `map`, where the caller has it, is [`scope_map`](Self::scope_map)'s:
    /// the variable of `into` each local leaf of `from` goes to. The
    /// evidence walk has it from the merge that made `into`'s variables.
    pub(crate) fn embed_moving(
        &self,
        f: Diagram,
        from: &VtreeScope,
        into: &VtreeScope,
        map: Option<&[VarId]>,
    ) -> Result<Diagram, Box<(CompileError, Diagram)>> {
        if from.same(into) {
            return Ok(f);
        }
        let found;
        let map = match map {
            Some(map) => {
                debug_assert_eq!(
                    Self::scope_map(from, into).ok().as_deref(),
                    Some(map),
                    "the given renaming is the scopes'"
                );
                map
            }
            None => match Self::scope_map(from, into) {
                Ok(map) => {
                    found = map;
                    &found
                }
                Err(e) => return Err(Box::new((e, f))),
            },
        };
        let at = |v: VarId| map[v.0 as usize - 1];
        match f {
            Diagram::Stored(f) => match self.engine.embed_moving(f, into.vtree(), at) {
                Ok((embedded, _)) => Ok(Diagram::Stored(embedded)),
                Err(refused) => Err(Box::new((
                    refused.error.into(),
                    Diagram::Stored(refused.tdd),
                ))),
            },
            // A copy: the skip form has no move in place.
            Diagram::Skip(f) => match self.skip_op(|e| e.skip_embed(&f, into.vtree(), at)) {
                Ok(embedded) => Ok(Diagram::Skip(embedded)),
                Err(e) => Err(Box::new((e.into(), Diagram::Skip(f)))),
            },
        }
    }

    /// A skip backend's operand `part` on `scope`: borrowed where it is
    /// on it, else copied onto it, its variable `v` going to
    /// `map[v - 1]` ([`TddBackend::scope_map`]).
    fn skip_placed<'p>(
        &self,
        part: &'p Part,
        map: &[VarId],
        scope: &VtreeScope,
    ) -> Result<Cow<'p, SkipTdd>, CompileError> {
        let f = part.0.skip();
        if part.1.same(scope) {
            return Ok(Cow::Borrowed(f));
        }
        let at = |v: VarId| map[v.0 as usize - 1];
        Ok(Cow::Owned(
            self.skip_op(|e| e.skip_embed(f, scope.vtree(), at))?,
        ))
    }

    /// The conjunction of `fa` and `fb` on `scope`, each moved there as
    /// [`TddBackend::embed_moving`] moves it, with `first` and `second` its
    /// `map` where the caller has it, in one operation, `Engine::and_onto`:
    /// a move leaves unbuilt the levels it adds where none of the operand's
    /// variables is under them, and the conjunction takes the other
    /// operand's levels there without visiting them, so that what a join
    /// costs past the moves is the levels with variables of both operands
    /// under them. Refused, each operand comes back with the scope it was
    /// left on: `scope` once moved there, its own before.
    ///
    /// A shared operand is read in place: one already on `scope`
    /// unchanged, one that is not copied and the copy moved.
    pub(crate) fn and_onto<'m>(
        &self,
        fa: impl Into<Part>,
        first: Option<&'m [VarId]>,
        fb: impl Into<Part>,
        second: Option<&'m [VarId]>,
        scope: &VtreeScope,
    ) -> Result<Diagram, Box<(CompileError, Part, Part)>> {
        let (fa, fb) = (fa.into(), fb.into());
        let renaming = |own: &VtreeScope, map: Option<&'m [VarId]>| match (own.same(scope), map) {
            (true, _) => Ok(Cow::Borrowed(&[][..])),
            (false, Some(map)) => Ok(Cow::Borrowed(map)),
            (false, None) => Self::scope_map(own, scope).map(Cow::Owned),
        };
        let (map_a, map_b) = match (renaming(&fa.1, first), renaming(&fb.1, second)) {
            (Ok(a), Ok(b)) => (a, b),
            (Err(e), _) | (_, Err(e)) => return Err(Box::new((e, fa, fb))),
        };
        if self.form == Form::Skip {
            // Each operand copied onto `scope` and the copies conjoined:
            // the operands are borrowed, and a refused one comes back as
            // it was, on its own scope.
            let joined = self.skip_placed(&fa, &map_a, scope).and_then(|a| {
                let b = self.skip_placed(&fb, &map_b, scope)?;
                Ok(self.skip_op(|e| e.skip_and(&a, &b))?)
            });
            return joined.map(Diagram::Skip).map_err(|e| Box::new((e, fa, fb)));
        }
        let (Part(a, own_a), Part(b, own_b)) = (fa, fb);
        let ((a, shared_a), (b, shared_b)) = (a.split(), b.split());
        self.engine
            .and_onto(
                stored_cow(lend(a, &shared_a)),
                |v| map_a[v.0 as usize - 1],
                stored_cow(lend(b, &shared_b)),
                |v| map_b[v.0 as usize - 1],
                scope.vtree(),
            )
            .map(Diagram::Stored)
            .map_err(|refused| {
                let left_on = |f: Read, own: VtreeScope| match Arc::ptr_eq(f.vtree(), scope.vtree())
                {
                    true => Part(f, scope.clone()),
                    false => Part(f, own),
                };
                Box::new((
                    refused.error.into(),
                    left_on(unlend(refused.f, &shared_a), own_a),
                    left_on(unlend(refused.g, &shared_b), own_b),
                ))
            })
    }

    /// `and_onto` for a caller that keeps nothing it refuses: the
    /// conjunction of `fa` and `fb` on `scope`, which holds both of their
    /// scopes. Where neither is to be moved it is [`TddBackend::and`],
    /// whose sweep frees each operand's levels as it goes, which a sweep
    /// that may have to hand its operands back cannot (Roulette's
    /// programs, whose operands share their scopes, ran 0.6% slower).
    pub(crate) fn and_on(
        &self,
        fa: impl Into<Part>,
        fb: impl Into<Part>,
        scope: &VtreeScope,
    ) -> Result<Diagram, CompileError> {
        let (fa, fb) = (fa.into(), fb.into());
        if fa.1.same(scope) && fb.1.same(scope) {
            let ((a, shared_a), (b, shared_b)) = (fa.0.split(), fb.0.split());
            return self.and(lend(a, &shared_a), lend(b, &shared_b));
        }
        self.and_onto(fa, None, fb, None, scope)
            .map_err(|refused| refused.0)
    }

    /// The scope over every variable of the program.
    pub(crate) fn global(&self) -> VtreeScope {
        VtreeScope {
            vtree: self.vtree(),
            vars: None,
        }
    }

    /// The scope over `vars`, which must be a subset of the program's
    /// variables in ascending order.
    pub(crate) fn scope(&self, vars: &[VarId]) -> Result<VtreeScope, CompileError> {
        debug_assert!(
            vars.windows(2).all(|w| w[0] < w[1]),
            "a scope's variables are ascending and distinct"
        );
        let vtree = self.vtree();
        if vars.len() == vtree.num_leaves() as usize {
            return Ok(self.global());
        }
        if vars.is_empty() {
            return Err(CompileError::EmptyScope);
        }
        let projected = self.projector.borrow_mut().project(&vtree, vars)?;
        Ok(VtreeScope {
            vtree: Arc::new(projected),
            vars: Some(vars.into()),
        })
    }

    /// The scope over `vars`, a subset of `parent`'s variables in ascending
    /// order, built from `parent` rather than from the program: what a
    /// projection costs is the size of what it projects.
    pub(crate) fn subscope(
        &self,
        parent: &VtreeScope,
        vars: &[VarId],
    ) -> Result<VtreeScope, CompileError> {
        debug_assert!(
            vars.windows(2).all(|w| w[0] < w[1]),
            "a scope's variables are ascending and distinct"
        );
        match parent.vars() {
            Some(held) if vars.len() == held.len() => Ok(parent.clone()),
            // The projection of a projection is the projection.
            _ => self.scope(vars),
        }
    }

    /// `f`, which lives on `from`, as a formula on `into`.
    ///
    /// `from`'s variables must be a subset of `into`'s; the compiler gets that
    /// from the cone of a component containing the cone of everything it
    /// depends on. Nothing is applied: the levels are copied.
    pub(crate) fn embed(
        &self,
        f: Diagram,
        from: &VtreeScope,
        into: &VtreeScope,
    ) -> Result<Diagram, CompileError> {
        self.embed_moving(f, from, into, None)
            .map_err(|refused| refused.0)
    }

    /// [`embed`](Self::embed) of a read, a shared formula copied first
    /// ([`Read::owned`]): the copy charges no work, where copying onto
    /// `into` ([`embed_ref`](Self::embed_ref)) charges each level it
    /// builds, so a shared read is charged what an owned one is.
    pub(crate) fn embed_read(
        &self,
        f: Read,
        from: &VtreeScope,
        into: &VtreeScope,
    ) -> Result<Diagram, CompileError> {
        self.embed(f.owned(self)?, from, into)
    }

    /// `f`, which lives on `from` and depends on none of its variables but
    /// `vars`, ascending, on the scope over `vars`: `into`, where the caller
    /// holds that scope, and otherwise a new one, which it returns. The
    /// inverse of [`embed`](Self::embed): the levels are copied, those of
    /// the dropped variables left out.
    pub(crate) fn restrict(
        &self,
        f: &Diagram,
        from: &VtreeScope,
        vars: &[VarId],
        into: Option<&VtreeScope>,
    ) -> Result<(Diagram, VtreeScope), CompileError> {
        let local = |v: VarId| {
            let v = from.vars().map_or(v, |held| held[v.0 as usize - 1]);
            vars.binary_search(&v).ok().map(|i| VarId(i as u32 + 1))
        };
        let n = vars.len() as u32;
        let restricted = match f {
            Diagram::Stored(f) => Diagram::Stored(self.engine.project_to_vars(f, local, n)?),
            Diagram::Skip(f) => {
                Diagram::Skip(self.skip_op(|e| e.skip_project_to_vars(f, local, n))?)
            }
        };
        let own = VtreeScope {
            vtree: Arc::clone(restricted.vtree()),
            vars: Some(vars.into()),
        };
        match into {
            Some(into) => Ok((self.embed(restricted, &own, into)?, into.clone())),
            None => Ok((restricted, own)),
        }
    }

    /// `f`, which lives on `from`, with the program variable `renamed.0`
    /// read as `renamed.1`, as a formula on `into`; `None` where `into` does
    /// not hold `from`'s shape under that renaming. Nothing is applied: the
    /// levels are copied.
    pub(crate) fn embed_renamed(
        &self,
        f: &Diagram,
        from: &VtreeScope,
        into: &VtreeScope,
        (old, new): (VarId, VarId),
    ) -> Result<Option<Diagram>, CompileError> {
        let Some(vars) = from.vars() else {
            return Ok(None);
        };
        let mut map = Vec::with_capacity(vars.len());
        for &v in vars {
            let Some(local) = into.local_of(if v == old { new } else { v }) else {
                return Ok(None);
            };
            map.push(local);
        }
        self.embed_mapped(f, into, |v| map[v.0 as usize - 1], false)
    }

    /// `f`, which lives on `from`, as a formula on `to` with local variable
    /// `k + 1` of `from` read as `map[k]` of `to`, a local variable of it;
    /// `None` where the two scopes differ in shape, that is where `to`'s
    /// vtree is not `from`'s up to mirrors with each leaf renamed through
    /// `map`, and on the global scope. Nothing is applied: the levels are
    /// copied, a mirrored one with the sides of its pairs exchanged.
    ///
    /// The embed checks the shapes leaf for leaf, each level either as it
    /// is or mirrored.
    pub(crate) fn transport(
        &self,
        f: &Diagram,
        from: &VtreeScope,
        to: &VtreeScope,
        map: &[VarId],
    ) -> Result<Option<Diagram>, CompileError> {
        if from.vars().is_none() || to.vars().is_none() || from.len() != to.len() {
            return Ok(None);
        }
        assert_eq!(
            map.len(),
            from.len(),
            "a local variable of `to` for each of `from`"
        );
        self.embed_mapped(f, to, |v| map[v.0 as usize - 1], true)
    }

    pub(crate) fn zero(&self, scope: &VtreeScope) -> Result<Diagram, CompileError> {
        self.born(self.engine.zero(scope.vtree()))
    }
    pub(crate) fn one(&self, scope: &VtreeScope) -> Result<Diagram, CompileError> {
        self.born(self.engine.one(scope.vtree()))
    }
    /// `lit` names a variable of the program; the backend renames it into
    /// `scope`.
    pub(crate) fn literal(
        &self,
        scope: &VtreeScope,
        lit: Literal,
    ) -> Result<Diagram, CompileError> {
        self.born(
            self.engine
                .literal(scope.vtree(), Self::renamed(scope, lit)?)?,
        )
    }
    /// Conjunction of literals (`one` for none), built in one pass.
    pub(crate) fn cube(
        &self,
        scope: &VtreeScope,
        lits: &[Literal],
    ) -> Result<Diagram, CompileError> {
        let mut renamed = Vec::with_capacity(lits.len());
        for &l in lits {
            renamed.push(Self::renamed(scope, l)?);
        }
        self.born(self.engine.cube(scope.vtree(), renamed)?)
    }
    /// That exactly one of `lits` holds, or at most one, on `scope`, whose
    /// variables are those of `lits`, each once.
    ///
    /// Built level by level rather than applied: at each node of the
    /// scope's vtree, none of its leaves' literals holds where none holds on
    /// either side, and exactly one where exactly one holds on one side and
    /// none on the other. From the halves by `and` and `or`, each operation
    /// would pay for every level of its scope. Only
    /// the diagram asked for is built, and at most one is the root's pairs
    /// of both, which the nodes of one level, being disjoint, allow nowhere
    /// below.
    pub(crate) fn one_of(
        &self,
        scope: &VtreeScope,
        lits: &[Literal],
        exactly: bool,
    ) -> Result<Diagram, CompileError> {
        let vtree = scope.vtree();
        if let [l] = *lits {
            return if exactly {
                self.literal(scope, l)
            } else {
                self.one(scope)
            };
        }
        let mut sign: Vec<Option<bool>> = vec![None; vtree.num_nodes()];
        for &l in lits {
            let local = Self::renamed(scope, l)?;
            let leaf = vtree
                .leaf_of(local.var)
                .expect("a scope's variables are its leaves");
            debug_assert!(sign[leaf.idx()].is_none(), "each variable once");
            sign[leaf.idx()] = Some(local.sign);
        }
        let mut b = Tdd::builder(self.engine, vtree)?;
        // Per vtree node, its node for none and for exactly one.
        let mut none = vec![NEG_LEAF_IDX; vtree.num_nodes()];
        let mut one = vec![POS_LEAF_IDX; vtree.num_nodes()];
        let root = vtree.root();
        for t in vtree.bottomup() {
            if vtree.node(t).is_leaf() {
                if !sign[t.idx()].expect("every variable of the scope is one of the literals") {
                    (none[t.idx()], one[t.idx()]) = (POS_LEAF_IDX, NEG_LEAF_IDX);
                }
                continue;
            }
            let (l, r) = vtree.children(t);
            let exactly_one = [
                ChildPair::new(none[l.idx()], one[r.idx()]),
                ChildPair::new(one[l.idx()], none[r.idx()]),
            ];
            if t != root {
                none[t.idx()] = b.push(
                    self.engine,
                    t,
                    &[ChildPair::new(none[l.idx()], none[r.idx()])],
                )?;
                one[t.idx()] = b.push(self.engine, t, &exactly_one)?;
            } else if exactly {
                one[t.idx()] = b.push(self.engine, t, &exactly_one)?;
            } else {
                let [x, y] = exactly_one;
                one[t.idx()] = b.push(
                    self.engine,
                    t,
                    &[ChildPair::new(none[l.idx()], none[r.idx()]), x, y],
                )?;
            }
        }
        let output = TddNodeId {
            vtree: root,
            local: one[root.idx()],
        };
        let f = b.finish(output).map_err(OperationError::from)?;
        let mut f = self.born(f)?;
        self.minimize(&mut f, Reduce::Between)?;
        Ok(f)
    }
    /// That none of `cubes` holds, and at most one of each group's
    /// variables, exactly one where it says so, on `scope`, which holds
    /// `vars`, ascending and distinct: the cubes and groups read local
    /// variables, `k + 1` for `vars[k]`, and read each of `vars`. `None`
    /// where building it could take more than `max_work` steps, a step per
    /// pair of classes tried and per residual a pair carries.
    ///
    /// Built level by level ([`super::none_of::build`]) on the scope of
    /// the variables the cubes and groups read, and embedded; of one
    /// variable, a literal, true or false of `scope` itself
    /// ([`super::none_of::one_variable`]), without projecting a vtree of
    /// one leaf: a clause from one body of one fact is of one variable.
    pub(crate) fn none_of(
        &self,
        scope: &VtreeScope,
        vars: &[VarId],
        cubes: &[Vec<Literal>],
        groups: &[(Vec<VarId>, bool)],
        max_work: u64,
    ) -> Result<Option<Diagram>, CompileError> {
        if vars.is_empty() {
            return Ok(Some(if cubes.is_empty() {
                self.one(scope)?
            } else {
                self.zero(scope)?
            }));
        }
        if let [v] = *vars {
            return Ok(Some(match super::none_of::one_variable(cubes, groups) {
                (true, true) => self.one(scope)?,
                (true, false) => self.literal(scope, Literal::pos(v))?,
                (false, true) => self.literal(scope, Literal::neg(v))?,
                (false, false) => self.zero(scope)?,
            }));
        }
        let own = self.scope_of(scope, vars)?;
        let built = super::none_of::build(
            self.engine,
            own.vtree(),
            cubes,
            groups,
            max_work,
            &mut self.none_of.borrow_mut(),
        )?;
        let Some(mut f) = built else { return Ok(None) };
        // The classes are the canonical ones but where two residuals differ
        // and mean the same, which is rare; what the reduction frees is the
        // classes no pair above uses.
        self.engine.reduce(&mut f, ReductionPlan::Prune)?;
        Ok(Some(self.embed(self.born(f)?, &own, scope)?))
    }
    /// The diagram whose models are `models` on `scope`, which holds
    /// `vars`, ascending and distinct: each model is `vars.len().div_ceil(64)`
    /// words, bit `k` for `vars[k]`, and reads each of `vars`. Built on the
    /// scope of `vars` in one pass ([`tididi::Engine::from_models`]), the
    /// canonical diagram of that set, and embedded.
    pub(crate) fn of_models(
        &self,
        scope: &VtreeScope,
        vars: &[VarId],
        models: &[u64],
    ) -> Result<Diagram, CompileError> {
        let own = self.scope_of(scope, vars)?;
        let local: Vec<VarId> = (1..=vars.len() as u32).map(VarId).collect();
        let f = self.engine.from_models(own.vtree(), &local, models)?;
        self.embed(self.born(f)?, &own, scope)
    }
    /// The scope of `vars` within `scope`, ascending and distinct: `scope`
    /// itself where they are all its variables, whose local numbering is
    /// then theirs, and its projection to them otherwise.
    fn scope_of(&self, scope: &VtreeScope, vars: &[VarId]) -> Result<VtreeScope, CompileError> {
        match scope.vars() {
            Some(held) if held.len() == vars.len() => {
                match vars.iter().zip(held).find(|(v, h)| v != h) {
                    Some((&v, _)) => Err(CompileError::NotInScope(v)),
                    None => Ok(scope.clone()),
                }
            }
            _ => self.subscope(scope, vars),
        }
    }
    /// `v ∧ x ∨ ¬v ∧ y` for `x` and `y = ¬x` on `scope`, neither reading
    /// the program variable `v`, where `v`'s leaf is a child of the scope's
    /// root; `None` elsewhere, or where either is false.
    ///
    /// Built rather than applied: the root's two pairs, `v` and `x`, `¬v`
    /// and `y`, over `x`'s levels and the nodes of `y` that `x` lacks.
    /// `or` of `v ∧ x` and `¬v ∧ y` walks a product grid at every level for
    /// pairs that pair nothing.
    ///
    /// Each node of `y` is found among `x`'s by a scan of the level from
    /// where the last was found, then once a level from its start, and
    /// past that by its pairs hashed. The negation that made `y` from `x`
    /// keeps `x`'s nodes in their order, adds at each level one node, last,
    /// of the cells none covers, and drops the nodes the root no longer
    /// reaches: each node of `y` but its level's last is the next of `x`'s
    /// it reaches, so few nodes are looked up by their pairs hashed, which
    /// would hash every node of both. A node's pairs are written sorted.
    ///
    /// The build reads stored levels, so in skip form the selection does
    /// not apply.
    pub(crate) fn select_at_root(
        &self,
        scope: &VtreeScope,
        v: VarId,
        x: &Diagram,
        y: &Diagram,
    ) -> Result<Option<Diagram>, CompileError> {
        let (Diagram::Stored(x), Diagram::Stored(y)) = (x, y) else {
            return Ok(None);
        };
        let Some((c, v_left, xc, yc)) = root_selection(scope, v, x, y)? else {
            return Ok(None);
        };
        let vtree = scope.vtree();
        let root = vtree.root();
        // A unit per pair looked up, where the engine charges about a unit
        // per product pair: `auto`'s work budget sees the build.
        self.engine
            .limits()
            .poll_host_work((x.pair_count() + y.pair_count()) as u64)?;
        let mut b = Tdd::builder(self.engine, vtree)?;
        let key = |p: &ChildPair| (p.left.raw(), p.right.raw());
        // `to[t][j]` is where node `j` of level `t` of `y` went; `x`'s
        // nodes keep their places.
        let mut to: Vec<Vec<u32>> = vec![Vec::new(); vtree.num_nodes()];
        // A node's pairs are read as a slice: a stored level's own, an
        // implicit one's generated into `buf`.
        let (mut pairs, mut buf) = (Vec::new(), Vec::new());
        for t in vtree.bottomup() {
            if t == root || vtree.node(t).is_leaf() {
                continue;
            }
            let (tl, tr) = vtree.children(t);
            let at_x = x.level(t);
            let width = at_x.nodes().len();
            if (0..width).all(|i| at_x.pairs_read(i, &mut buf).is_sorted_by_key(key)) {
                b.replace_level(self.engine, t, x.level_view(t))?;
            } else {
                for i in 0..width {
                    pairs.clear();
                    pairs.extend_from_slice(at_x.pairs_read(i, &mut buf));
                    pairs.sort_unstable_by_key(key);
                    b.push(self.engine, t, &pairs)?;
                }
            }
            let moved = |ch: VtreeIdx, e: EncodedChildRef| match to[ch.idx()].as_slice() {
                [] => e,
                at => EncodedChildRef::from_raw(at[e.raw() as usize]),
            };
            let mut at = Vec::new();
            let (mut next, mut missed) = (0, 0);
            for (_, node) in y.level(t).internal_inputs_iter() {
                pairs.clear();
                pairs.extend(node.map(|p| ChildPair::new(moved(tl, p.left), moved(tr, p.right))));
                if !pairs.is_sorted_by_key(key) {
                    pairs.sort_unstable_by_key(key);
                }
                let level = b.level(t);
                let mut found = |from: usize, to: usize| {
                    (from..to).find(|&i| level.pairs_read(i, &mut buf) == pairs.as_slice())
                };
                let i = match found(next, width) {
                    Some(i) => {
                        next = i + 1;
                        i as u32
                    }
                    None if missed == 0 => {
                        missed += 1;
                        match found(0, next) {
                            Some(i) => i as u32,
                            None => b.push(self.engine, t, &pairs)?.0,
                        }
                    }
                    None => b.intern(self.engine, t, &pairs)?.0,
                };
                at.push(i);
            }
            to[t.idx()] = at;
        }
        let yc = to[c.idx()].get(yc as usize).copied().unwrap_or(yc);
        let node = b.push(self.engine, root, &root_selection_pairs(v_left, xc, yc))?;
        Ok(Some(Diagram::Stored(
            b.finish(TddNodeId {
                vtree: root,
                local: node,
            })
            .map_err(OperationError::from)?,
        )))
    }
    /// `a ∧ b`, each operand owned or borrowed: a borrowed one is read in
    /// place.
    pub(crate) fn and<'a, 'b>(
        &self,
        a: impl Into<Cow<'a, Diagram>>,
        b: impl Into<Cow<'b, Diagram>>,
    ) -> Result<Diagram, CompileError> {
        let (a, b) = (a.into(), b.into());
        Ok(match self.form {
            Form::Stored => Diagram::Stored(self.engine.and(stored_cow(a), stored_cow(b))?),
            Form::Skip => Diagram::Skip(self.skip_op(|e| e.skip_and(a.skip(), b.skip()))?),
        })
    }
    /// `a ∧ b` on `scope` ([`TddBackend::and_on`]), or both operands back,
    /// each on the scope it was left on, once it has spent `work` units
    /// without finishing; the stop as for [`TddBackend::within`].
    pub(crate) fn and_within(
        &self,
        a: Part,
        b: Part,
        scope: &VtreeScope,
        work: u64,
    ) -> Result<Result<Diagram, (Part, Part)>, CompileError> {
        let Some((_scope, at)) = self.stop_after(work) else {
            return Ok(Ok(self.and_on(a, b, scope)?));
        };
        match self.and_onto(a, None, b, None, scope) {
            Ok(r) => Ok(Ok(r)),
            Err(refused) => match *refused {
                (CompileError::Tdd(OperationError::Stopped), a, b) if self.work_units() >= at => {
                    Ok(Err((a, b)))
                }
                (e, _, _) => Err(e),
            },
        }
    }
    /// What `run` returns, or `None` once it has spent `work` units
    /// without finishing. Where the run's own limits stop sooner, or on a
    /// deadline, it runs under them alone, and a stop they make is an
    /// error, as for [`TddBackend::and`].
    pub(crate) fn within<T>(
        &self,
        work: u64,
        run: impl FnOnce() -> Result<T, CompileError>,
    ) -> Result<Option<T>, CompileError> {
        let Some((_scope, at)) = self.stop_after(work) else {
            return run().map(Some);
        };
        match run() {
            Ok(r) => Ok(Some(r)),
            Err(CompileError::Tdd(OperationError::Stopped)) if self.work_units() >= at => Ok(None),
            Err(e) => Err(e),
        }
    }
    /// A stop at `work` units from now, held while the scope lives, and
    /// the units it stops at; `None` where the run's own limits stop
    /// sooner or on a deadline.
    fn stop_after(&self, work: u64) -> Option<(tididi::limits::LimitScope<'_>, u64)> {
        let limits = self.engine.limits();
        let rules = limits.armed().stop_rules();
        let at = limits.work_units().saturating_add(work);
        match rules.unconditional {
            None => {}
            Some(StopAt::WorkUnits(u)) if u > at => {}
            Some(_) => return None,
        }
        Some((
            limits
                .edit(|c| c.with_stop_rules(rules.with_unconditional(Some(StopAt::WorkUnits(at))))),
            at,
        ))
    }
    /// The run's stop in work units, and the end of its turn
    /// ([`Self::pause`]), moved `work` units later while the result lives.
    pub(crate) fn stop_later(&self, work: u64) -> Later<'_> {
        let limits = self.engine.limits();
        let rules = limits.armed().stop_rules();
        let stop = match rules.unconditional {
            Some(StopAt::WorkUnits(u)) => Some(limits.edit(|c| {
                c.with_stop_rules(
                    rules.with_unconditional(Some(StopAt::WorkUnits(u.saturating_add(work)))),
                )
            })),
            _ => None,
        };
        self.turn_end
            .set(self.turn_end.get().map(|at| at.saturating_add(work)));
        Later {
            _stop: stop,
            turn_end: &self.turn_end,
            work,
        }
    }
    pub(crate) fn or(&self, a: Diagram, b: Diagram) -> Result<Diagram, CompileError> {
        Ok(match (a, b) {
            (Diagram::Stored(a), Diagram::Stored(b)) => Diagram::Stored(self.engine.or(a, b)?),
            (a, b) => Diagram::Skip(self.skip_op(|e| e.skip_or(a.skip(), b.skip()))?),
        })
    }
    /// `a ∧ (l_1 ∨ ... ∨ l_k)`, with `a` on `scope`.
    pub(crate) fn and_clause(
        &self,
        scope: &VtreeScope,
        a: Diagram,
        clause: &[Literal],
    ) -> Result<Diagram, CompileError> {
        let mut renamed = Vec::with_capacity(clause.len());
        for &l in clause {
            renamed.push(Self::renamed(scope, l)?);
        }
        Ok(match a {
            Diagram::Stored(a) => Diagram::Stored(self.engine.and_clause(a, &renamed)?),
            Diagram::Skip(a) => Diagram::Skip(self.skip_op(|e| e.skip_and_clause(&a, &renamed))?),
        })
    }
    pub(crate) fn negate<'f>(
        &self,
        a: impl Into<Cow<'f, Diagram>>,
    ) -> Result<Diagram, CompileError> {
        let a = a.into();
        if self.form == Form::Skip {
            return Ok(Diagram::Skip(self.skip_op(|e| e.skip_negate(a.skip()))?));
        }
        // The compiler minimizes every result it keeps, so the full reduction
        // `negate` ends with by default is paid twice. Pruning is the part
        // negation cannot skip: it removes the operand's dead cone, the nodes
        // the root complement orphans. Contraction is what the compiler
        // repeats, and tididi does not split `One` on the way in, so there
        // are next to no twins left for it to merge anyway.
        Ok(Diagram::Stored(
            self.engine
                .negate_with(stored_cow(a), ReductionPlan::Prune)?,
        ))
    }
    /// Reduce `f`. Between operations a backend may reduce less than fully,
    /// so the result is not necessarily the canonical form `equivalent`
    /// compares; `equivalent` reduces what it compares itself. A formula the
    /// compiler settles on is always reduced in full. A skip-form formula
    /// is always canonical and takes no reduction.
    pub(crate) fn minimize(&self, f: &mut Diagram, how: Reduce) -> Result<(), CompileError> {
        let Diagram::Stored(f) = f else {
            return Ok(());
        };
        // A settled formula is reduced in full and tells the schedule
        // nothing. It runs on a diagram that was reduced a moment ago, so it
        // almost always frees little, and letting it say so would widen the
        // schedule once per atom on the strength of a minimization that was
        // never on it (on bn_pathfinder that widening made the
        // intermediates dirty enough to trip the watch).
        match how {
            Reduce::Settled => return Ok(self.engine.minimize(f)?),
            Reduce::Contract => return Ok(self.engine.reduce(f, ReductionPlan::Contract)?),
            Reduce::Prune => return Ok(self.engine.reduce(f, ReductionPlan::Prune)?),
            Reduce::Between => {}
        }
        // See [`FullSchedule`]: the full plan runs on a schedule that
        // widens while it frees almost nothing, and between full plans the
        // diagram is contracted but not pruned. Contraction drains a
        // worklist, so it costs an empty check on a diagram with no twins,
        // where the prune costs a walk.
        let mut schedule = self.prune.get();
        if !schedule.full_is_due() {
            schedule.took_partial();
            self.prune.set(schedule);
            return Ok(self.engine.reduce(f, ReductionPlan::Contract)?);
        }
        let before = f.pair_count();
        self.engine.minimize(f)?;
        schedule.observed(before.saturating_sub(f.pair_count()), before >> PRUNE_KEPT);
        self.prune.set(schedule);
        Ok(())
    }
    pub(crate) fn equivalent(&self, a: &Diagram, b: &Diagram) -> Result<bool, CompileError> {
        Ok(match (a, b) {
            (Diagram::Stored(a), Diagram::Stored(b)) => self.engine.equivalent(a, b)?,
            (a, b) => self.engine.skip_equivalent(a.skip(), b.skip())?,
        })
    }
    pub(crate) fn is_zero(&self, f: &Diagram) -> bool {
        f.is_zero()
    }
    /// The size the backend pays for `f`.
    pub(crate) fn size(&self, f: &Diagram) -> usize {
        f.pair_count()
    }
    /// The pairs of `f` its levels hold as their description rather than
    /// written ([`tididi::diagram::TddLevel::implicit`]); the skip form
    /// holds none.
    pub(crate) fn implicit_size(&self, f: &Diagram) -> usize {
        match f {
            Diagram::Stored(f) => implicit_pairs(f),
            Diagram::Skip(_) => 0,
        }
    }
    /// The nodes of `f`, read in one step per level rather than per node:
    /// a proxy for [`TddBackend::size`] cheap enough to read after every
    /// clause.
    pub(crate) fn nodes(&self, f: &Diagram) -> usize {
        f.node_count()
    }
    /// The variables `f` depends on, local to its vtree, ascending.
    pub(crate) fn support(&self, f: &Diagram) -> Result<Vec<VarId>, CompileError> {
        Ok(match f {
            Diagram::Stored(f) => self.engine.support(f)?,
            Diagram::Skip(f) => self.engine.skip_support(f)?,
        })
    }
    /// `∃ vars. f ∧ g`, `vars` local to their vtree.
    pub(crate) fn and_exists(
        &self,
        f: Diagram,
        g: Diagram,
        vars: &[VarId],
    ) -> Result<Diagram, CompileError> {
        Ok(match (f, g) {
            (Diagram::Stored(f), Diagram::Stored(g)) => {
                Diagram::Stored(self.engine.and_exists(f, g, vars)?)
            }
            (f, g) => Diagram::Skip(self.skip_op(|e| e.skip_and_exists(f.skip(), g.skip(), vars))?),
        })
    }
    /// The weighted count of `f` under `w`, by the engine's evaluation in
    /// floats, which the engine's work clock is charged for; where that is
    /// not exact, in [`Count`]s, which a stored diagram takes by
    /// [`crate::eval::count`]'s pass up and a skip-form one by the
    /// engine's evaluation, and `recounts` counts it
    /// ([`crate::eval::mass`]).
    pub(crate) fn mass(
        &self,
        f: &Diagram,
        w: &Probabilities,
        recounts: &mut usize,
    ) -> Result<Count, CompileError> {
        let f = match f {
            Diagram::Stored(f) => return Ok(crate::eval::mass(self.engine, f, w, recounts)?),
            Diagram::Skip(f) => f,
        };
        let plain = Weighing::<f64>::new(w);
        let m = self.engine.skip_evaluate(f, &plain)?;
        if plain.exact() {
            return Ok(Count::new(m));
        }
        *recounts += 1;
        Ok(self.engine.skip_evaluate(f, &Weighing::<Count>::new(w))?)
    }

    /// The work clock the backend's operations advance, in its own units.
    pub(crate) fn work_units(&self) -> u64 {
        self.engine.limits().work_units()
    }

    /// Marks the work clock where the phase under way has kept a step: a
    /// component, an atom's update, an observation, a join or its putting
    /// off, an answer. A stop after the mark loses the work since
    /// ([`Self::lost`]): the step it cut short starts over.
    pub(crate) fn kept(&self) {
        self.kept_at.set(self.work_units());
    }

    /// The work since the backend began or last kept a step
    /// ([`Self::kept`]): what a stop now loses.
    pub(crate) fn lost(&self) -> u64 {
        self.work_units().saturating_sub(self.kept_at.get())
    }

    /// Ends `auto`'s turn at the first step kept once the work clock has
    /// reached `at` ([`Self::pause`]).
    pub(crate) fn end_turn_at(&self, at: u64) {
        self.turn_end.set(Some(at));
    }

    /// Whether the backend runs a turn with an end ([`Self::end_turn_at`]).
    pub(crate) fn in_turn(&self) -> bool {
        self.turn_end.get().is_some()
    }

    /// Whether the turn under way has reached its end.
    pub(crate) fn turn_over(&self) -> bool {
        self.turn_end
            .get()
            .is_some_and(|at| self.work_units() >= at)
    }

    /// A stop, where the turn under way has reached its end, at a step the
    /// phase has kept: between components, atom updates, the bodies of a
    /// loop of clauses, the evidence walk's observations and joins, and
    /// answers. The phase goes on from that step in the trial's next
    /// attempt, and the stop loses nothing ([`Self::lost`]). The run's stop
    /// in work units, past the turn's end, is the guard against one
    /// operation that runs far past it, and loses that operation.
    pub(crate) fn pause(&self) -> Result<(), CompileError> {
        if self.turn_over() {
            return Err(self.end_turn());
        }
        Ok(())
    }

    /// The stop of [`Self::pause`], where the phase has kept what it holds
    /// and the turn is over.
    pub(crate) fn end_turn(&self) -> CompileError {
        self.kept();
        CompileError::Tdd(OperationError::Stopped)
    }

    /// Search for a global vtree on which `members` are smaller, sweeping
    /// until a sweep keeps no move, then running `restarts`, their seed
    /// mixed with the members' live pairs so that each search draws its
    /// own. The members must be every live formula on the global scope;
    /// they keep their functions, and afterwards [`TddBackend::global`] is
    /// the scope they live on.
    pub(crate) fn restructure(
        &self,
        members: &mut [&mut Diagram],
        restarts: PoolRestarts,
    ) -> Result<PoolSearchStats, CompileError> {
        let mut stored = Vec::with_capacity(members.len());
        for m in members.iter_mut() {
            stored.push(m.stored_mut("a vtree search")?);
        }
        let members = &mut stored[..];
        let pairs: usize = members.iter().map(|m| m.pair_count()).sum();
        // No sweep or work limit: the library's default of four sweeps stops
        // the searches of the random graphs while they still gain, and the
        // compile's schedule, not a budget, bounds what searching costs.
        let config = PoolSearchConfig::default()
            .with_max_sweeps(usize::MAX)
            .with_max_work_units(u64::MAX)
            .with_restarts(restarts.with_seed(restarts.seed ^ pairs as u64));
        // A search that stops part way keeps the moves it made: the members
        // share the rotated vtree either way, and the backend follows them.
        let found = self.engine.pool_search(members, &config);
        if let Some(first) = members.first()
            && !Arc::ptr_eq(first.vtree(), &self.vtree.borrow())
        {
            *self.projector.borrow_mut() = Projector::new(first.vtree());
            *self.vtree.borrow_mut() = Arc::clone(first.vtree());
        }
        Ok(found?)
    }
}

/// The pairs of `f` its levels hold as their description.
fn implicit_pairs(f: &Tdd) -> usize {
    f.vtree()
        .internal_bottomup()
        .filter_map(|(t, _, _)| f.level(t).implicit())
        .map(|d| d.pairs())
        .sum()
}

/// Where `scope`'s root has `v`'s leaf as a child and neither `x` nor `y`
/// is zero or reads `v`: the other child `c` of the root, whether `v`'s
/// leaf is on the left, and the node of `c` that each operand's root pair
/// points to.
fn root_selection(
    scope: &VtreeScope,
    v: VarId,
    x: &Tdd,
    y: &Tdd,
) -> Result<Option<(VtreeIdx, bool, u32, u32)>, CompileError> {
    let vtree = scope.vtree();
    let root = vtree.root();
    if vtree.node(root).is_leaf() || x.is_zero() || y.is_zero() {
        return Ok(None);
    }
    let leaf = vtree
        .leaf_of(TddBackend::renamed(scope, Literal::pos(v))?.var)
        .expect("a scope's variables are its leaves");
    let (l, r) = vtree.children(root);
    let (c, v_left) = match leaf {
        _ if leaf == l => (r, true),
        _ if leaf == r => (l, false),
        _ => return Ok(None),
    };
    // Neither reads `v`: its root has one pair, `One` on `v`'s side.
    let at_c = |f: &Tdd| -> Option<u32> {
        let mut pairs = f.level(root).pairs_iter_of_idx(f.output().local.idx());
        let (1, Some(p)) = (pairs.len(), pairs.next()) else {
            return None;
        };
        let (at_v, at_c) = if v_left {
            (p.left, p.right)
        } else {
            (p.right, p.left)
        };
        (at_v.raw() == ONE_LEAF_IDX.0).then_some(at_c.raw())
    };
    let (Some(xc), Some(yc)) = (at_c(x), at_c(y)) else {
        return Ok(None);
    };
    Ok(Some((c, v_left, xc, yc)))
}

fn root_selection_pairs(v_left: bool, xc: u32, yc: u32) -> [ChildPair; 2] {
    let (xc, yc) = (NodeIdx(xc), NodeIdx(yc));
    if v_left {
        [
            ChildPair::new(POS_LEAF_IDX, xc),
            ChildPair::new(NEG_LEAF_IDX, yc),
        ]
    } else {
        [
            ChildPair::new(xc, POS_LEAF_IDX),
            ChildPair::new(yc, NEG_LEAF_IDX),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tp::scope::scope_shape;
    use tididi::vtree::VtreeNode;

    /// A stored backend's diagram as tididi's.
    fn st(f: &Diagram) -> &Tdd {
        f.stored("a test of stored diagrams").unwrap()
    }

    /// Xorshift draws from `rng`: each call a number below its argument.
    fn draws(mut rng: u64) -> impl FnMut(u64) -> u64 {
        move |m| {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng % m
        }
    }

    /// `v ∧ x ∨ ¬v ∧ y` for `x` and `y = ¬x` as `select_at_root` built it
    /// before it scanned `x`'s levels for `y`'s nodes: the levels of both
    /// interned side by side, each node's pairs sorted.
    fn select_by_interning(
        backend: &TddBackend,
        scope: &VtreeScope,
        v: VarId,
        x: &Tdd,
        y: &Tdd,
    ) -> Option<Tdd> {
        let (c, v_left, xc, yc) = root_selection(scope, v, x, y).unwrap()?;
        let (engine, vtree) = (backend.engine, scope.vtree());
        let root = vtree.root();
        let mut b = Tdd::builder(engine, vtree).unwrap();
        let mut to: [Vec<Vec<u32>>; 2] = [
            vec![Vec::new(); vtree.num_nodes()],
            vec![Vec::new(); vtree.num_nodes()],
        ];
        for t in vtree.bottomup() {
            if t == root || vtree.node(t).is_leaf() {
                continue;
            }
            let (tl, tr) = vtree.children(t);
            for (k, f) in [x, y].into_iter().enumerate() {
                let moved = |ch: VtreeIdx, e: EncodedChildRef| match to[k][ch.idx()].as_slice() {
                    [] => e,
                    at => EncodedChildRef::from_raw(at[e.raw() as usize]),
                };
                let mut at = Vec::new();
                for (_, node) in f.level(t).internal_inputs_iter() {
                    let mut pairs: Vec<ChildPair> = node
                        .map(|p| ChildPair::new(moved(tl, p.left), moved(tr, p.right)))
                        .collect();
                    pairs.sort_unstable_by_key(|p| (p.left.raw(), p.right.raw()));
                    at.push(b.intern(engine, t, &pairs).unwrap().0);
                }
                to[k][t.idx()] = at;
            }
        }
        let side = |k: usize, i: u32| to[k][c.idx()].get(i as usize).copied().unwrap_or(i);
        let node = b
            .push(
                engine,
                root,
                &root_selection_pairs(v_left, side(0, xc), side(1, yc)),
            )
            .unwrap();
        Some(
            b.finish(TddNodeId {
                vtree: root,
                local: node,
            })
            .unwrap(),
        )
    }

    /// `f` with the nodes of each level in the reverse order.
    fn reversed(engine: &Engine, f: &Tdd) -> Tdd {
        if f.is_zero() {
            return f.clone();
        }
        let vtree = f.vtree().clone();
        let mut b = Tdd::builder(engine, &vtree).unwrap();
        let mut to: Vec<Vec<u32>> = vec![Vec::new(); vtree.num_nodes()];
        for t in vtree.bottomup() {
            if vtree.node(t).is_leaf() {
                continue;
            }
            let (tl, tr) = vtree.children(t);
            let moved = |ch: VtreeIdx, e: EncodedChildRef| match to[ch.idx()].as_slice() {
                [] => e,
                at => EncodedChildRef::from_raw(at[e.raw() as usize]),
            };
            let level = f.level(t);
            let n = level.internal_inputs_iter().count();
            let mut at = vec![0u32; n];
            for i in (0..n).rev() {
                let pairs: Vec<ChildPair> = level
                    .pairs_iter_of_idx(i)
                    .map(|p| ChildPair::new(moved(tl, p.left), moved(tr, p.right)))
                    .collect();
                at[i] = b.push(engine, t, &pairs).unwrap().0;
            }
            to[t.idx()] = at;
        }
        let out = f.output();
        b.finish(TddNodeId {
            vtree: out.vtree,
            local: NodeIdx(to[out.vtree.idx()][out.local.idx()]),
        })
        .unwrap()
    }

    /// Whether `a` and `b` are the same diagram node for node: the same
    /// output, and on every level the same nodes in the same order with
    /// the same pairs in the same order.
    fn same_levels(a: &Tdd, b: &Tdd) -> bool {
        a.output() == b.output()
            && a.vtree().bottomup().all(|t| {
                let (la, lb) = (a.level(t), b.level(t));
                la.internal_inputs_iter().count() == lb.internal_inputs_iter().count()
                    && la
                        .internal_inputs_iter()
                        .all(|(i, _)| la.pairs_iter_of_idx(i).eq(lb.pairs_iter_of_idx(i)))
            })
    }

    /// `select_at_root` builds the canonical diagram of `v ∧ x ∨ ¬v ∧ ¬x`,
    /// the one `or` of the two conjunctions reduces to, with `v` on either
    /// side of the root and random formulas over random trees below it,
    /// and declines where `v` is deeper. It is node for node the diagram
    /// the levels of `x` and of `¬x` interned side by side make, with `¬x`
    /// the negation of `x` and that with each level's nodes reversed.
    #[test]
    fn a_selection_at_the_root_is_the_applied_one() {
        for seed in 0..60u64 {
            let n = 3 + (seed % 9) as u32;
            let below = Vtree::random(n - 1, seed);
            let v = VarId(n);
            let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            let mut next = move || {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state
            };
            // A disjunction of a few random cubes over the variables below.
            let mut cubes: Vec<Vec<Literal>> = Vec::new();
            for _ in 0..1 + next() % 4 {
                let mut cube = Vec::new();
                for w in 1..n {
                    let r = next();
                    if r % 3 == 0 {
                        cube.push(Literal::new(VarId(w), (r >> 8) % 2 == 0));
                    }
                }
                cubes.push(cube);
            }
            let leaf = Vtree::leaf(v);
            for tree in [
                Vtree::join(&leaf, &below).unwrap(),
                Vtree::join(&below, &leaf).unwrap(),
            ] {
                let tree = Arc::new(tree);
                tree.context().run(|engine| {
                    let backend = TddBackend::new(engine, tree.clone());
                    let scope = backend.global();
                    // Both operands settled on the scope below `v` and
                    // embedded, as the compiler hands them over.
                    let vars: Vec<VarId> = (1..n).map(VarId).collect();
                    let own = backend.scope(&vars).unwrap();
                    let mut x = backend.zero(&own).unwrap();
                    for cube in &cubes {
                        x = backend.or(x, backend.cube(&own, cube).unwrap()).unwrap();
                    }
                    backend.minimize(&mut x, Reduce::Settled).unwrap();
                    let y = backend.negate(x.clone()).unwrap();
                    let (x, y) = (
                        backend.embed(x, &own, &scope).unwrap(),
                        backend.embed(y, &own, &scope).unwrap(),
                    );
                    // The negation with its nodes in another order than `x`'s.
                    let other = Diagram::Stored(reversed(engine, st(&y)));
                    if x.is_zero() || y.is_zero() {
                        assert!(backend.select_at_root(&scope, v, &x, &y).unwrap().is_none());
                        return;
                    }
                    let built = backend
                        .select_at_root(&scope, v, &x, &y)
                        .unwrap()
                        .expect("v is a child of the root");
                    let interned = select_by_interning(&backend, &scope, v, st(&x), st(&y))
                        .expect("v is a child of the root");
                    assert!(
                        same_levels(st(&built), &interned),
                        "seed {seed}: not the interned diagram"
                    );
                    let built = backend
                        .select_at_root(&scope, v, &x, &other)
                        .unwrap()
                        .expect("v is a child of the root");
                    let interned = select_by_interning(&backend, &scope, v, st(&x), st(&other))
                        .expect("v is a child of the root");
                    assert!(
                        same_levels(st(&built), &interned),
                        "seed {seed}: not the interned diagram of the other negation"
                    );
                    let yes = backend
                        .and_clause(&scope, x.clone(), &[Literal::pos(v)])
                        .unwrap();
                    let no = backend
                        .and_clause(&scope, y.clone(), &[Literal::neg(v)])
                        .unwrap();
                    let mut applied = backend.or(yes, no).unwrap();
                    backend.minimize(&mut applied, Reduce::Settled).unwrap();
                    assert!(backend.equivalent(&built, &applied).unwrap(), "seed {seed}");
                    assert_eq!(
                        built.pair_count(),
                        applied.pair_count(),
                        "seed {seed}: not reduced"
                    );
                    let mut settled = built.clone();
                    backend.minimize(&mut settled, Reduce::Settled).unwrap();
                    assert_eq!(
                        settled.pair_count(),
                        built.pair_count(),
                        "seed {seed}: settling changed it"
                    );
                });
            }
            // Under a balanced tree of all of them `v` is no child of the root.
            let tree = Arc::new(Vtree::balanced(n.max(4)));
            tree.context().run(|engine| {
                let backend = TddBackend::new(engine, tree.clone());
                let scope = backend.global();
                let x = backend.literal(&scope, Literal::pos(VarId(1))).unwrap();
                let y = backend.literal(&scope, Literal::neg(VarId(1))).unwrap();
                assert!(
                    backend
                        .select_at_root(&scope, VarId(2), &x, &y)
                        .unwrap()
                        .is_none()
                );
            });
        }
    }

    /// `one_of` builds the diagram of exactly one and of at most one of
    /// literals of either sign that `or` of their cubes reduces to, on
    /// random trees.
    #[test]
    fn one_of_is_the_applied_disjunction_of_cubes() {
        for seed in 0..80u64 {
            let n = 1 + (seed % 12) as u32;
            let tree = Arc::new(Vtree::random(n, seed));
            let lits: Vec<Literal> = (1..=n)
                .map(|v| Literal::new(VarId(v), (seed >> (v % 7)) & 1 == 0))
                .collect();
            tree.context().run(|engine| {
                let backend = TddBackend::new(engine, tree.clone());
                let scope = backend.global();
                let mut exactly = backend.zero(&scope).unwrap();
                for i in 0..lits.len() {
                    let cube: Vec<Literal> = lits
                        .iter()
                        .enumerate()
                        .map(|(j, &l)| if i == j { l } else { l.negated() })
                        .collect();
                    exactly = backend
                        .or(exactly, backend.cube(&scope, &cube).unwrap())
                        .unwrap();
                }
                let none: Vec<Literal> = lits.iter().map(|l| l.negated()).collect();
                let mut most = backend
                    .or(exactly.clone(), backend.cube(&scope, &none).unwrap())
                    .unwrap();
                backend.minimize(&mut exactly, Reduce::Settled).unwrap();
                backend.minimize(&mut most, Reduce::Settled).unwrap();
                for (applied, flag) in [(exactly, true), (most, false)] {
                    let built = backend.one_of(&scope, &lits, flag).unwrap();
                    assert!(
                        backend.equivalent(&built, &applied).unwrap(),
                        "seed {seed} exactly {flag}"
                    );
                    let mut settled = built.clone();
                    backend.minimize(&mut settled, Reduce::Settled).unwrap();
                    assert_eq!(
                        settled.pair_count(),
                        applied.pair_count(),
                        "seed {seed} exactly {flag}"
                    );
                }
            });
        }
    }

    fn applied_none_of(
        backend: &TddBackend,
        scope: &VtreeScope,
        cubes: &[Vec<Literal>],
        groups: &[(Vec<VarId>, bool)],
    ) -> Diagram {
        let mut applied = backend.one(scope).unwrap();
        for cube in cubes {
            let clause: Vec<Literal> = cube.iter().map(|l| l.negated()).collect();
            applied = backend.and_clause(scope, applied, &clause).unwrap();
        }
        for (g, exactly) in groups {
            for (i, &a) in g.iter().enumerate() {
                for &b in &g[i + 1..] {
                    applied = backend
                        .and_clause(scope, applied, &[Literal::neg(a), Literal::neg(b)])
                        .unwrap();
                }
            }
            if *exactly {
                let clause: Vec<Literal> = g.iter().map(|&v| Literal::pos(v)).collect();
                applied = backend.and_clause(scope, applied, &clause).unwrap();
            }
        }
        applied
    }

    fn local_none_of(
        backend: &TddBackend,
        scope: &VtreeScope,
        cubes: &[Vec<Literal>],
        groups: &[(Vec<VarId>, bool)],
    ) -> Diagram {
        let mut vars: Vec<VarId> = cubes
            .iter()
            .flatten()
            .map(|l| l.var)
            .chain(groups.iter().flat_map(|(g, _)| g.iter().copied()))
            .collect();
        vars.sort_unstable();
        vars.dedup();
        let local = |v: VarId| VarId(vars.binary_search(&v).unwrap() as u32 + 1);
        let local_cubes: Vec<Vec<Literal>> = cubes
            .iter()
            .map(|c| {
                c.iter()
                    .map(|l| Literal::new(local(l.var), l.sign))
                    .collect()
            })
            .collect();
        let local_groups: Vec<(Vec<VarId>, bool)> = groups
            .iter()
            .map(|(g, exactly)| (g.iter().map(|&v| local(v)).collect(), *exactly))
            .collect();
        backend
            .none_of(scope, &vars, &local_cubes, &local_groups, u64::MAX)
            .unwrap()
            .expect("no bound")
    }

    /// The conjunction of the cubes' negations and the groups' clauses, one
    /// clause at a time, against the diagram built level by level, on
    /// random vtrees with groups whose variables the cubes read in either
    /// sign, and with cubes the builder merges (`cubes::merge_complements`).
    #[test]
    fn none_of_is_the_conjunction_of_its_clauses() {
        for seed in 0..2000u64 {
            let mut next = draws(
                seed.wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407),
            );
            let n = 2 + next(9) as u32;
            let tree = Arc::new(Vtree::random(n, seed));
            let mut groups: Vec<(Vec<VarId>, bool)> = Vec::new();
            let mut v = 1;
            while v < n && next(3) > 0 {
                let size = (2 + next(3) as u32).min(n + 1 - v);
                groups.push(((v..v + size).map(VarId).collect(), next(2) == 0));
                v += size;
            }
            // Random cubes, or on odd seeds the codes of random values over
            // the first variables, some with a literal of another, as a set
            // read in binary names its values, which merge.
            let bits = 1 + next(u64::from(n.min(4))) as u32;
            let cubes: Vec<Vec<Literal>> = (0..next(if seed % 2 == 0 { 8 } else { 14 }))
                .map(|_| {
                    if seed % 2 == 0 {
                        return (0..1 + next(4))
                            .map(|_| Literal::new(VarId(1 + next(n as u64) as u32), next(3) > 0))
                            .collect();
                    }
                    let value = next(1 << bits) as u32;
                    let mut cube: Vec<Literal> = (0..bits)
                        .map(|b| Literal::new(VarId(1 + b), value >> b & 1 == 1))
                        .collect();
                    if bits < n && next(2) == 0 {
                        cube.push(Literal::new(
                            VarId(1 + bits + next(u64::from(n - bits)) as u32),
                            next(3) > 0,
                        ));
                    }
                    cube
                })
                .collect();
            tree.context().run(|engine| {
                let backend = TddBackend::new(engine, tree.clone());
                let scope = backend.global();
                let applied = applied_none_of(&backend, &scope, &cubes, &groups);
                let built = local_none_of(&backend, &scope, &cubes, &groups);
                assert!(
                    backend.equivalent(&built, &applied).unwrap(),
                    "seed {seed}: cubes {cubes:?} groups {groups:?}"
                );
            });
        }
    }

    /// A backend's builds share their arenas ([`super::super::none_of::Scratch`]):
    /// after builds of more cubes and of fewer, and builds that gave up
    /// part way, each build is still the conjunction of its clauses.
    #[test]
    fn none_of_builds_in_a_row_are_each_their_clauses() {
        let n = 10u32;
        let tree = Arc::new(Vtree::random(n, 7));
        let mut next = draws(0x9e37_79b9_7f4a_7c15);
        tree.context().run(|engine| {
            let backend = TddBackend::new(engine, tree.clone());
            let scope = backend.global();
            let vars: Vec<VarId> = (1..=n).map(VarId).collect();
            let (mut built, mut gave_up) = (0, 0);
            for round in 0..300 {
                let count = if round % 3 == 0 { 30 } else { 1 + next(6) };
                let cubes: Vec<Vec<Literal>> = (0..count)
                    .map(|_| {
                        (0..1 + next(4))
                            .map(|_| {
                                Literal::new(VarId(1 + next(u64::from(n)) as u32), next(3) > 0)
                            })
                            .collect()
                    })
                    .collect();
                let groups: Vec<(Vec<VarId>, bool)> = if round % 2 == 0 {
                    Vec::new()
                } else {
                    vec![((2..=4).map(VarId).collect(), next(2) == 0)]
                };
                let max_work = if round % 4 == 3 {
                    4 + next(40)
                } else {
                    u64::MAX
                };
                let Some(f) = backend
                    .none_of(&scope, &vars, &cubes, &groups, max_work)
                    .unwrap()
                else {
                    assert!(
                        max_work != u64::MAX,
                        "round {round}: gave up without a bound"
                    );
                    gave_up += 1;
                    continue;
                };
                built += 1;
                let applied = applied_none_of(&backend, &scope, &cubes, &groups);
                assert!(
                    backend.equivalent(&f, &applied).unwrap(),
                    "round {round}: cubes {cubes:?} groups {groups:?}"
                );
            }
            assert!(built > 0 && gave_up > 0, "{built} built, {gave_up} gave up");
        });
    }

    /// Of one variable, every set of cubes over it, empty, in either sign
    /// and in both, and every group of it, against the conjunction of the
    /// clauses, on the global scope and on a scope of other variables too.
    #[test]
    fn none_of_one_variable_is_the_conjunction_of_its_clauses() {
        let tree = Arc::new(Vtree::random(6, 3));
        let x = VarId(4);
        let shapes: [&[bool]; 5] = [&[], &[true], &[false], &[true, false], &[true, true]];
        tree.context().run(|engine| {
            let backend = TddBackend::new(engine, tree.clone());
            for scope in [
                backend.global(),
                backend.scope(&[VarId(2), x, VarId(5)]).unwrap(),
            ] {
                for set in 0..1u32 << shapes.len() {
                    let picked: Vec<&[bool]> = (0..shapes.len())
                        .filter(|&k| set >> k & 1 == 1)
                        .map(|k| shapes[k])
                        .collect();
                    for group in [None, Some(false), Some(true)] {
                        let cubes: Vec<Vec<Literal>> = picked
                            .iter()
                            .map(|signs| signs.iter().map(|&s| Literal::new(VarId(1), s)).collect())
                            .collect();
                        let groups: Vec<(Vec<VarId>, bool)> = group
                            .map(|exactly| (vec![VarId(1)], exactly))
                            .into_iter()
                            .collect();
                        let built = backend
                            .none_of(&scope, &[x], &cubes, &groups, u64::MAX)
                            .unwrap()
                            .expect("no bound");
                        let mut applied = backend.one(&scope).unwrap();
                        for signs in &picked {
                            let clause: Vec<Literal> =
                                signs.iter().map(|&s| Literal::new(x, !s)).collect();
                            applied = if clause.is_empty() {
                                backend.zero(&scope).unwrap()
                            } else {
                                backend.and_clause(&scope, applied, &clause).unwrap()
                            };
                        }
                        if group == Some(true) {
                            applied = backend
                                .and_clause(&scope, applied, &[Literal::pos(x)])
                                .unwrap();
                        }
                        assert!(
                            backend.equivalent(&built, &applied).unwrap(),
                            "cubes {picked:?} group {group:?}"
                        );
                    }
                }
            }
        });
    }

    /// The conjunction of the clauses against the diagram built level by
    /// level over a few hundred variables, where a class's values run over
    /// several words and are cut past the last leaf a node above reads
    /// (`none_of::kept_bits`): the lineage of a hierarchical query, each
    /// cube a customer, one of its orders and one of the order's lineitems,
    /// on the right-linear vtree of `closed`'s spine (each order after its
    /// lineitems, a customer after its orders), on its mirror and on trees
    /// split at random places of that order; and cubes of nearby variables
    /// in either sign, with groups.
    #[test]
    fn none_of_over_many_leaves_is_the_conjunction_of_its_clauses() {
        for seed in 0..48u64 {
            let mut next = draws(
                seed.wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407),
            );
            let mut cubes: Vec<Vec<Literal>> = Vec::new();
            let mut groups: Vec<(Vec<VarId>, bool)> = Vec::new();
            let mut leaves: Vec<VarId> = Vec::new();
            if seed % 2 == 0 {
                let mut n = 0u32;
                let mut fresh = || {
                    n += 1;
                    VarId(n)
                };
                for _ in 0..6 + next(8) {
                    let c = fresh();
                    for _ in 0..1 + next(5) {
                        let o = fresh();
                        let sign = seed % 4 == 0 || next(4) > 0;
                        for _ in 0..1 + next(6) {
                            let l = fresh();
                            cubes.push(vec![
                                Literal::pos(c),
                                Literal::new(o, sign),
                                Literal::pos(l),
                            ]);
                            leaves.push(l);
                        }
                        leaves.push(o);
                    }
                    leaves.push(c);
                }
            } else {
                let n = 150 + next(150) as u32;
                for _ in 0..n {
                    let base = 1 + next(u64::from(n)) as u32;
                    let cube: Vec<Literal> = (0..1 + next(4))
                        .map(|_| Literal::new(VarId((base + next(12) as u32).min(n)), next(3) > 0))
                        .collect();
                    cubes.push(cube);
                }
                let mut v = 1;
                while v + 3 <= n {
                    if next(8) == 0 {
                        groups.push((
                            (v..v + 2 + next(2) as u32).map(VarId).collect(),
                            next(2) == 0,
                        ));
                    }
                    v += 4;
                }
                leaves = (1..=n).map(VarId).collect();
            }
            // A tree split at random places of the order, whose widths the
            // cubes' nearness bounds, where a random vtree's are not.
            fn split(leaves: &[VarId], next: &mut dyn FnMut(u64) -> u64) -> Vtree {
                if leaves.len() == 1 {
                    return Vtree::leaf(leaves[0]);
                }
                let k = 1 + next(leaves.len() as u64 - 1) as usize;
                Vtree::join(&split(&leaves[..k], next), &split(&leaves[k..], next)).unwrap()
            }
            let n = leaves.len() as u32;
            let tree = match seed / 2 % 3 {
                2 => split(&leaves, &mut next),
                k => {
                    if k == 1 {
                        leaves.reverse();
                    }
                    let mut spine = Vtree::leaf(*leaves.last().unwrap());
                    for &v in leaves.iter().rev().skip(1) {
                        spine = Vtree::join(&Vtree::leaf(v), &spine).unwrap();
                    }
                    spine
                }
            };
            let tree = Arc::new(tree);
            tree.context().run(|engine| {
                let backend = TddBackend::new(engine, tree.clone());
                let scope = backend.global();
                let applied = applied_none_of(&backend, &scope, &cubes, &groups);
                let built = local_none_of(&backend, &scope, &cubes, &groups);
                assert!(
                    backend.equivalent(&built, &applied).unwrap(),
                    "seed {seed}: {n} variables"
                );
            });
        }
    }

    /// Tables as a reader's link holds them ([`crate::indicators`]): each
    /// context, a value of each parent's copies, a decision list of
    /// switches of its own, each row the context, its switch literals and
    /// the reader's copy of its value negated, some contexts with values
    /// left out or one alone; at most or exactly one of each parent's
    /// copies, and exactly one of the reader's. On the families layout's
    /// vtree ([`crate::vtree`]), the switches on a spine above the parents'
    /// copies, the reader's copies beside it, where the spine's classes
    /// pass up as they are (`none_of::build`'s lifts), and on random
    /// vtrees; against the conjunction of the clauses.
    #[test]
    fn a_table_on_a_spine_of_its_switches_is_its_clauses() {
        for seed in 0..400u64 {
            let mut next = draws(
                seed.wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407),
            );
            let mut n = 0u32;
            let mut fresh = |k: u32| -> Vec<VarId> {
                n += k;
                (n + 1 - k..=n).map(VarId).collect()
            };
            // Up to sixteen contexts: one or two parents of up to four
            // values, or three of two.
            let parents: Vec<Vec<VarId>> = match next(3) {
                0 => (0..3).map(|_| fresh(2)).collect(),
                k => (0..k).map(|_| fresh(2 + next(3) as u32)).collect(),
            };
            let values = 2 + next(3) as u32;
            let ys = fresh(values);
            let mut contexts: Vec<Vec<VarId>> = vec![Vec::new()];
            for p in &parents {
                contexts = contexts
                    .iter()
                    .flat_map(|c| p.iter().map(move |&x| [c.as_slice(), &[x]].concat()))
                    .collect();
            }
            let mut cubes: Vec<Vec<Literal>> = Vec::new();
            let mut switches: Vec<VarId> = Vec::new();
            for ctx in &contexts {
                let mut order: Vec<u32> = (0..values).collect();
                for k in (1..order.len()).rev() {
                    order.swap(k, next(k as u64 + 1) as usize);
                }
                order.truncate(match next(4) {
                    0 => 1,
                    1 => 1 + next(u64::from(values)) as usize,
                    _ => values as usize,
                });
                let w = fresh(order.len() as u32 - 1);
                for (k, &y) in order.iter().enumerate() {
                    let mut cube: Vec<Literal> = ctx.iter().map(|&x| Literal::pos(x)).collect();
                    cube.extend(w.get(k).map(|&s| Literal::pos(s)));
                    cube.extend(w[..k].iter().map(|&s| Literal::neg(s)));
                    cube.push(Literal::neg(ys[y as usize]));
                    cubes.push(cube);
                }
                // The first used deepest, or the last.
                if seed % 2 == 0 {
                    switches.extend(w.iter().rev());
                } else {
                    switches.extend(w);
                }
            }
            let mut groups: Vec<(Vec<VarId>, bool)> =
                parents.iter().map(|p| (p.clone(), next(2) == 0)).collect();
            groups.push((ys.clone(), true));
            let tree = if seed % 4 == 3 {
                Vtree::random(n, seed)
            } else {
                let mut spine = Vtree::balanced_over(&parents.concat()).unwrap();
                for &s in &switches {
                    spine = Vtree::join(&Vtree::leaf(s), &spine).unwrap();
                }
                Vtree::join(&Vtree::balanced_over(&ys).unwrap(), &spine).unwrap()
            };
            let tree = Arc::new(tree);
            tree.context().run(|engine| {
                let backend = TddBackend::new(engine, tree.clone());
                let scope = backend.global();
                let vars: Vec<VarId> = (1..=n).map(VarId).collect();
                let built = backend
                    .none_of(&scope, &vars, &cubes, &groups, u64::MAX)
                    .unwrap()
                    .expect("no bound");
                let applied = applied_none_of(&backend, &scope, &cubes, &groups);
                assert!(
                    backend.equivalent(&built, &applied).unwrap(),
                    "seed {seed}: cubes {cubes:?} groups {groups:?}"
                );
            });
        }
    }

    /// The schedule widens while the full plan frees almost nothing and
    /// snaps back to every call as soon as one frees more: for
    /// `Reduce::Between` more than a thirty-second of what it was given,
    /// for `Reduce::Join` anything at all.
    #[test]
    fn the_full_schedule_widens_on_nothing_and_resets_on_something() {
        let mut s = FullSchedule::default();
        assert!(s.full_is_due(), "the first minimization runs the full plan");

        // A diagram of 1000 pairs that a full minimization leaves alone.
        let kept = 1000 >> PRUNE_KEPT;
        let mut widths = Vec::new();
        for _ in 0..8 {
            s.observed(0, kept);
            widths.push(s.stride);
            let mut skipped = 0;
            while !s.full_is_due() {
                s.took_partial();
                skipped += 1;
            }
            assert_eq!(skipped, s.stride, "the stride is what it skips");
        }
        assert_eq!(widths, vec![1, 3, 7, 15, 16, 16, 16, 16]);

        // One that frees more than 1000 >> 5 = 31 of them.
        s.observed(32, kept);
        assert_eq!(s.stride, 0);
        assert!(s.full_is_due(), "and the next minimization is full again");

        // Exactly the threshold still counts as almost nothing.
        s.observed(31, kept);
        assert_eq!(s.stride, 1);

        // The walk's schedule counts a single pair freed.
        s.observed(0, 0);
        assert_eq!(s.stride, 3);
        s.observed(1, 0);
        assert_eq!(s.stride, 0);
    }

    /// A permutation of `0..n` drawn from `seed`.
    fn shuffled(n: u32, seed: u64) -> Vec<u32> {
        let mut p: Vec<u32> = (0..n).collect();
        let mut x = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
        for i in (1..p.len()).rev() {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            p.swap(i, (x % (i as u64 + 1)) as usize);
        }
        p
    }

    /// The subtree of `tree` under `t` with leaf `v` renamed to
    /// `rename[v - 1]` and its children exchanged at each internal node
    /// `mirror` names.
    fn rebuilt(
        tree: &Vtree,
        t: VtreeIdx,
        rename: &[VarId],
        mirror: &dyn Fn(VtreeIdx) -> bool,
    ) -> Vtree {
        match tree.node(t) {
            VtreeNode::Leaf { var, .. } => Vtree::leaf(rename[var.0 as usize - 1]),
            VtreeNode::Internal { left, right, .. } => {
                let (l, r) = (
                    rebuilt(tree, *left, rename, mirror),
                    rebuilt(tree, *right, rename, mirror),
                );
                let joined = if mirror(t) {
                    Vtree::join(&r, &l)
                } else {
                    Vtree::join(&l, &r)
                };
                joined.unwrap()
            }
        }
    }

    /// Whether [`TddBackend::transport`] copies a formula on `from`, whose
    /// local variable `k + 1` is in slot `from_slots[k]`, onto `to`, whose
    /// local variable `j + 1` is in slot `to_slots[j]`, by the map the
    /// clause memo makes of the slots; checked against the formula built on
    /// `to` where it does.
    fn transports(
        backend: &TddBackend,
        from: &Arc<Vtree>,
        from_slots: &[u32],
        to: &Arc<Vtree>,
        to_slots: &[u32],
    ) -> bool {
        let n = from.num_leaves();
        let scope = |vtree: &Arc<Vtree>, first: u32| VtreeScope {
            vtree: vtree.clone(),
            vars: Some((first..first + n).map(VarId).collect()),
        };
        let mut local = vec![VarId(0); to_slots.len()];
        for (j, &s) in to_slots.iter().enumerate() {
            local[s as usize] = VarId(j as u32 + 1);
        }
        let map: Vec<VarId> = from_slots.iter().map(|&s| local[s as usize]).collect();
        let lit = |k: usize, v: VarId| {
            if k % 3 == 1 {
                -(v.0 as i32)
            } else {
                v.0 as i32
            }
        };
        let f = backend
            .engine
            .cube(from, (0..n as usize).map(|k| lit(k, VarId(k as u32 + 1))))
            .unwrap();
        let copied = backend
            .transport(
                &Diagram::Stored(f),
                &scope(from, 1),
                &scope(to, n + 1),
                &map,
            )
            .unwrap();
        if let Some(copied) = &copied {
            let renamed = backend
                .engine
                .cube(to, (0..n as usize).map(|k| lit(k, map[k])))
                .unwrap();
            assert!(
                backend.engine.equivalent(st(copied), &renamed).unwrap(),
                "the copy is the formula renamed"
            );
        }
        copied.is_some()
    }

    /// A scope's tree renamed, its slots following the renaming, and
    /// mirrored at no node, at the root, at each internal node alone, at
    /// every one, and at sets of them, every set on small trees: the
    /// transport takes each, and each hashes as the source does.
    #[test]
    fn every_scope_the_transport_takes_hashes_alike() {
        let engine = Engine::new();
        let mut hashes = Vec::new();
        for seed in 0..48u64 {
            let n = 1 + (seed % 12) as u32;
            let from = Arc::new(Vtree::random(n, seed));
            let backend = TddBackend::new(&engine, from.clone());
            let from_slots = shuffled(n, seed);
            let rename: Vec<VarId> = shuffled(n, seed + 1000)
                .into_iter()
                .map(|v| VarId(v + 1))
                .collect();
            let mut to_slots = vec![0; n as usize];
            for (k, &v) in rename.iter().enumerate() {
                to_slots[v.0 as usize - 1] = from_slots[k];
            }
            let internal: Vec<VtreeIdx> = from.internal_bottomup().map(|(t, _, _)| t).collect();
            let mut sets: Vec<Vec<VtreeIdx>> = vec![vec![], vec![from.root()], internal.clone()];
            sets.extend(internal.iter().map(|&t| vec![t]));
            match internal.len() {
                0..=6 => sets.extend((0..1u32 << internal.len()).map(|bits| {
                    internal
                        .iter()
                        .enumerate()
                        .filter(|&(i, _)| bits >> i & 1 == 1)
                        .map(|(_, &t)| t)
                        .collect()
                })),
                _ => sets.extend((0..16u64).map(|k| {
                    internal
                        .iter()
                        .copied()
                        .filter(|t| {
                            (seed ^ k).wrapping_mul(0x9e37_79b9_7f4a_7c15) >> (t.idx() % 64) & 1
                                == 1
                        })
                        .collect()
                })),
            }
            let here = scope_shape(&from, &from_slots, &mut hashes);
            for set in &sets {
                let to = Arc::new(rebuilt(&from, from.root(), &rename, &|t| set.contains(&t)));
                assert!(
                    transports(&backend, &from, &from_slots, &to, &to_slots),
                    "seed {seed}: mirrored at {set:?}, the transport takes it"
                );
                assert_eq!(
                    scope_shape(&to, &to_slots, &mut hashes),
                    here,
                    "seed {seed}: mirrored at {set:?}"
                );
            }
        }
    }

    /// On small random trees and slots, two scopes hash the same exactly
    /// where the transport takes one onto the other.
    #[test]
    fn scopes_hash_alike_where_the_transport_takes_them() {
        let engine = Engine::new();
        let mut hashes = Vec::new();
        let (mut taken, mut refused) = (0, 0);
        for seed in 0..400u64 {
            let n = 1 + (seed % 6) as u32;
            let from = Arc::new(Vtree::random(n, seed));
            let to = Arc::new(Vtree::random(n, seed / 6 + 7919));
            let backend = TddBackend::new(&engine, from.clone());
            let (from_slots, to_slots) = (shuffled(n, seed), shuffled(n, seed / 3));
            let took = transports(&backend, &from, &from_slots, &to, &to_slots);
            let alike = scope_shape(&from, &from_slots, &mut hashes)
                == scope_shape(&to, &to_slots, &mut hashes);
            assert_eq!(alike, took, "seed {seed}");
            if took { taken += 1 } else { refused += 1 }
        }
        assert!(
            taken > 50 && refused > 50,
            "both kinds are tried: {taken} taken, {refused} refused"
        );
    }

    /// An operation of [`every_skip_arm_is_the_stored_arm_in_skip_form`]:
    /// each adds a formula on the global scope to the pool from earlier
    /// ones, an index taken modulo the pool's length.
    #[derive(Debug)]
    enum Op {
        Cube(Vec<Literal>),
        /// Exactly one, or at most one, of the literals, built on the
        /// scope of their variables and embedded.
        OneOf(Vec<Literal>, bool),
        NoneOf(Vec<Vec<Literal>>),
        And(usize, usize),
        Or(usize, usize),
        Negate(usize),
        Clause(usize, Vec<Literal>),
        AndExists(usize, usize, Vec<VarId>),
        /// A cube on the scope of its variables, conjoined with a pool
        /// formula on the global scope, onto which it is moved.
        Moved(Vec<Literal>, usize),
        /// A pool formula restricted to its support and embedded back.
        Narrowed(usize),
    }

    /// `op` on `backend`, over `pool`.
    fn apply(backend: &TddBackend, pool: &[Diagram], op: &Op) -> Result<Diagram, CompileError> {
        let scope = backend.global();
        let at = |i: usize| &pool[i % pool.len()];
        let vars_of = |lits: &mut dyn Iterator<Item = VarId>| {
            let mut vars: Vec<VarId> = lits.collect();
            vars.sort_unstable();
            vars.dedup();
            vars
        };
        Ok(match op {
            Op::Cube(lits) => backend.cube(&scope, lits)?,
            Op::OneOf(lits, exactly) => {
                let own = backend.scope(&vars_of(&mut lits.iter().map(|l| l.var)))?;
                let f = backend.one_of(&own, lits, *exactly)?;
                backend.embed(f, &own, &scope)?
            }
            Op::NoneOf(cubes) => {
                let vars = vars_of(&mut cubes.iter().flatten().map(|l| l.var));
                let local = |v: VarId| VarId(vars.binary_search(&v).unwrap() as u32 + 1);
                let cubes: Vec<Vec<Literal>> = cubes
                    .iter()
                    .map(|c| {
                        c.iter()
                            .map(|l| Literal::new(local(l.var), l.sign))
                            .collect()
                    })
                    .collect();
                backend
                    .none_of(&scope, &vars, &cubes, &[], u64::MAX)?
                    .expect("no bound")
            }
            Op::And(i, j) => backend.and(at(*i), at(*j))?,
            Op::Or(i, j) => backend.or(at(*i).clone(), at(*j).clone())?,
            Op::Negate(i) => backend.negate(at(*i))?,
            Op::Clause(i, clause) => backend.and_clause(&scope, at(*i).clone(), clause)?,
            Op::AndExists(i, j, vars) => {
                backend.and_exists(at(*i).clone(), at(*j).clone(), vars)?
            }
            Op::Moved(lits, i) => {
                let own = backend.scope(&vars_of(&mut lits.iter().map(|l| l.var)))?;
                let f = backend.cube(&own, lits)?;
                backend.and_on((f, own), (at(*i).clone(), scope.clone()), &scope)?
            }
            // A constant keeps no variable, and the compile does not narrow it.
            Op::Narrowed(i) => match backend.support(at(*i))? {
                support if support.is_empty() => at(*i).clone(),
                support => backend.restrict(at(*i), &scope, &support, Some(&scope))?.0,
            },
        })
    }

    /// Random operations over `n` variables, from `next`.
    fn random_ops(n: u32, next: &mut impl FnMut(u64) -> u64) -> Vec<Op> {
        let lits = |next: &mut dyn FnMut(u64) -> u64, at_most: u64| -> Vec<Literal> {
            let mut vars: Vec<u32> = (1..=n).collect();
            let k = 1 + next(at_most.min(u64::from(n)));
            (0..k)
                .map(|_| {
                    let v = vars.swap_remove(next(vars.len() as u64) as usize);
                    Literal::new(VarId(v), next(2) == 0)
                })
                .collect()
        };
        (0..24)
            .map(|_| {
                let (i, j) = (next(64) as usize, next(64) as usize);
                match next(10) {
                    0 | 1 => Op::Cube(lits(next, 4)),
                    2 => Op::OneOf(lits(next, 5), next(2) == 0),
                    3 => Op::NoneOf((0..1 + next(4)).map(|_| lits(next, 3)).collect()),
                    4 => Op::And(i, j),
                    5 => Op::Or(i, j),
                    6 => Op::Negate(i),
                    7 => Op::Clause(i, lits(next, 3)),
                    8 if next(2) == 0 => Op::AndExists(i, j, {
                        let mut vars: Vec<VarId> =
                            lits(next, 3).into_iter().map(|l| l.var).collect();
                        vars.sort_unstable();
                        vars
                    }),
                    8 => Op::Narrowed(i),
                    _ => Op::Moved(lits(next, 4), i),
                }
            })
            .collect()
    }

    /// Each skip arm of the backend gives the skip form of what the stored
    /// arm gives: random operations on random and right-linear vtrees, run
    /// alike on a stored backend and a skip one, each result equivalent to
    /// `to_skip` of the stored one and as large (both forms are
    /// canonical), with the same support, the same weighted count to
    /// 1e-12 and the same equivalences with the pool. A builder whose
    /// stored diagram holds a level as its description is refused in skip
    /// form, and only that.
    #[test]
    fn every_skip_arm_is_the_stored_arm_in_skip_form() {
        let (mut checked, mut refused) = (0, 0);
        for seed in 0..300u64 {
            let mut next = draws(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let n = 2 + next(9) as u32;
            let tree = Arc::new(if seed % 3 == 0 {
                Vtree::linear(n)
            } else {
                Vtree::random(n, seed)
            });
            let ops = random_ops(n, &mut next);
            // Facts, and every fifth variable an indicator, which weighs
            // two where a formula ignores it.
            let mut w = Probabilities {
                pos: vec![0.5; n as usize + 1],
                neg: vec![0.5; n as usize + 1],
                one: vec![1.0; n as usize + 1],
            };
            for v in 1..=n as usize {
                if v % 5 == 0 {
                    (w.pos[v], w.neg[v], w.one[v]) = (1.0, 1.0, 2.0);
                } else {
                    let p = (1 + next(999)) as f64 / 1000.0;
                    (w.pos[v], w.neg[v]) = (p, 1.0 - p);
                }
            }
            tree.context().run(|engine| {
                let stored = TddBackend::new(engine, tree.clone());
                let skip = TddBackend::in_form(engine, tree.clone(), Form::Skip);
                let scope = stored.global();
                let mut sp = vec![stored.one(&scope).unwrap()];
                let mut kp = vec![skip.one(&scope).unwrap()];
                for op in &ops {
                    let s = apply(&stored, &sp, op).unwrap();
                    let k = match apply(&skip, &kp, op) {
                        Err(CompileError::SkipForm(_)) => {
                            assert!(stored.implicit_size(&s) > 0, "seed {seed}: {op:?} refused");
                            refused += 1;
                            continue;
                        }
                        k => k.unwrap(),
                    };
                    let want = engine.to_skip(st(&s)).unwrap();
                    assert!(
                        engine.skip_equivalent(k.skip(), &want).unwrap(),
                        "seed {seed}: {op:?}"
                    );
                    assert_eq!(k.pair_count(), want.pair_count(), "seed {seed}: {op:?}");
                    assert_eq!(
                        stored.support(&s).unwrap(),
                        skip.support(&k).unwrap(),
                        "seed {seed}: {op:?}"
                    );
                    let (ms, mk) = (
                        stored.mass(&s, &w, &mut 0).unwrap(),
                        skip.mass(&k, &w, &mut 0).unwrap(),
                    );
                    assert_eq!(ms.is_zero(), mk.is_zero(), "seed {seed}: {op:?}");
                    if !ms.is_zero() {
                        assert!((mk.ratio(ms) - 1.0).abs() < 1e-12, "seed {seed}: {op:?}");
                    }
                    for (a, b) in sp.iter().zip(&kp) {
                        assert_eq!(
                            stored.equivalent(&s, a).unwrap(),
                            skip.equivalent(&k, b).unwrap(),
                            "seed {seed}: {op:?}"
                        );
                    }
                    checked += 1;
                    sp.push(s);
                    kp.push(k);
                }
            });
        }
        assert!(
            checked > 3000,
            "{checked} results checked, {refused} refused"
        );
    }

    /// The compile's copy in skip form begins the probe's stretch where
    /// none has begun: the stretch starts after it, with the pairs the
    /// conversion read, which the engine's clock does not count, charged to
    /// the copy. A copy over the memory budget leaves the trial stored, and
    /// a probe in any other state makes none.
    #[test]
    fn the_compile_copies_itself_where_the_stretch_begins() {
        let tree = Arc::new(Vtree::linear(4));
        let stretch = Probe::Stretch {
            pairs: 1000,
            skip: 10,
            start: None,
            converted: 0,
        };
        tree.context().run(|engine| {
            let start = engine.limits().work_units();
            let stored = TddBackend::new(engine, tree.clone()).with_probe(stretch, start, 40);
            let copy = stored.fork(5, || Ok(("copy", 7))).unwrap();
            assert_eq!(copy, Some("copy"));
            assert_eq!(
                stored.probed(),
                Probe::Stretch {
                    pairs: 1000,
                    skip: 10,
                    start: Some((5, 40)),
                    converted: 7,
                }
            );
            // The stretch has begun: no second copy.
            assert_eq!(stored.fork(6, || Ok(("again", 1))).unwrap(), None);
            let stored = TddBackend::new(engine, tree.clone()).with_probe(stretch, start, 40);
            let over = stored.fork::<&str>(5, || {
                Err(CompileError::Tdd(tididi::OperationError::OverBudget))
            });
            assert_eq!(over.unwrap(), None);
            assert_eq!(stored.probed(), Probe::Refused);
            let stopped = TddBackend::new(engine, tree.clone()).with_probe(stretch, start, 40);
            assert!(matches!(
                stopped.fork::<&str>(5, || Err(CompileError::Tdd(
                    tididi::OperationError::Stopped
                ))),
                Err(CompileError::Tdd(tididi::OperationError::Stopped))
            ));
            assert_eq!(
                stopped.probed(),
                stretch,
                "a stop makes the copy again later"
            );
        });
    }

    #[test]
    fn the_probe_weighs_the_skip_form_over_the_stored_forms_stretch() {
        // The stretch begins where the compile makes its copy in skip form
        // (`TddBackend::fork`), here at component 3 with the trial's work
        // at 100 after a copy of 7 units, and ends at the first later step
        // where the trial's work has doubled.
        let stretch = Probe::Stretch {
            pairs: 1000,
            skip: 10,
            start: None,
            converted: 0,
        };
        let (p, stop) = stretch.at_step(3, 100);
        assert_eq!((p, stop.is_none()), (stretch, true), "no copy yet");
        let p = Probe::Stretch {
            pairs: 1000,
            skip: 10,
            start: Some((3, 100)),
            converted: 7,
        };
        let (p, stop) = p.at_step(3, 500);
        assert!(stop.is_none(), "the stretch ends past its first step");
        let (p, stop) = p.at_step(4, 150);
        assert!(stop.is_none());
        let (_, stop) = p.at_step(5, 200);
        assert!(matches!(
            stop,
            Some(CompileError::Skips {
                pairs: 1000,
                skip: 10,
                from: 3,
                to: 5,
                stored: 100,
                converted: 7,
            })
        ));
        // The skip form's work is read from the step with as many
        // components finished, its copy's included, and wins where it
        // spends less over them.
        let duel = Probe::Duel {
            from: 3,
            to: 5,
            stored: 100,
            converted: 7,
            at: None,
        };
        let (p, _) = duel.at_step(2, 1000);
        assert_eq!(p, duel);
        let (p, _) = p.at_step(3, 1050);
        let (p, stop) = p.at_step(4, 1090);
        assert!(stop.is_none());
        let (won, stop) = p.at_step(5, 1142);
        assert!(stop.is_none());
        assert_eq!(
            won,
            Probe::Weighed {
                skip: 99,
                stored: 100,
                at: 1142
            }
        );
        // It loses once it has spent the stored form's work, at the end of
        // the stretch or before.
        let (_, stop) = p.at_step(4, 1143);
        assert!(matches!(
            stop,
            Some(CompileError::Stored {
                skip: 100,
                stored: 100
            })
        ));
        let (_, stop) = p.at_step(5, 1143);
        assert!(matches!(stop, Some(CompileError::Stored { .. })));
    }

    /// The engine's work clock starts again with each run, and a trial's
    /// turns are runs: the probe's clock is what the trial spent before the
    /// run and the work since, so a duel begun in one turn is weighed in
    /// the next over the work of both.
    #[test]
    fn the_probes_clock_runs_on_across_a_trials_turns() {
        let n = 30u32;
        let tree = Arc::new(Vtree::random(n, 11));
        let work = |backend: &TddBackend, seed: u64| {
            let mut next = draws(seed);
            let scope = backend.global();
            let mut f = backend.one(&scope).unwrap();
            for _ in 0..12 {
                let clause: Vec<Literal> = (0..3)
                    .map(|_| Literal::new(VarId(1 + next(u64::from(n)) as u32), next(2) == 0))
                    .collect();
                f = backend.and_clause(&scope, f, &clause).unwrap();
            }
        };
        let duel = Probe::Duel {
            from: 1,
            to: 3,
            stored: u64::MAX,
            converted: 0,
            at: None,
        };
        // The first turn: the duel begins at component 1, after some work,
        // and the turn ends with more.
        let (probe, spent, at) = tree.context().run(|engine| {
            let start = engine.limits().work_units();
            let skip =
                TddBackend::in_form(engine, tree.clone(), Form::Skip).with_probe(duel, start, 0);
            work(&skip, 1);
            let at = skip.work_units() - start;
            skip.at_step(1).unwrap();
            work(&skip, 2);
            (skip.probed(), skip.work_units() - start, at)
        });
        assert!(at > 0 && spent > at);
        assert!(
            matches!(probe, Probe::Duel { at: Some(a), .. } if a == at),
            "{probe:?}"
        );
        // The next turn reads the clock from what the first spent.
        let (probe, more) = tree.context().run(|engine| {
            let start = engine.limits().work_units();
            assert!(start < at, "the engine's clock starts again");
            let skip = TddBackend::in_form(engine, tree.clone(), Form::Skip)
                .with_probe(probe, start, spent);
            skip.at_step(2).unwrap();
            work(&skip, 3);
            skip.at_step(3).unwrap();
            (skip.probed(), skip.work_units() - start)
        });
        let skip = spent + more - at;
        assert!(
            matches!(probe, Probe::Weighed { skip: s, at: a, .. } if s == skip && a == spent + more),
            "{probe:?}, {skip} units"
        );
    }

    /// The run's stop in work units cuts a skip operation once its own
    /// units, scaled by `SKIP_WORK_SCALE`, reach the stop, as it cuts a
    /// stored one: a conjunction of two random 3-CNFs in skip form, grown
    /// until it takes at least 2^18 units of its own, stops when given half
    /// the scaled work it takes, with the clock at or past that half. Its
    /// own units alone would not have reached the stop.
    #[test]
    fn a_stop_in_work_units_cuts_a_skip_operation_at_its_scaled_work() {
        let n = 40u32;
        let tree = Arc::new(Vtree::random(n, 7));
        let cnf = |backend: &TddBackend, seed: u64, clauses: usize| {
            let mut next = draws(seed);
            let scope = backend.global();
            let mut f = backend.one(&scope).unwrap();
            for _ in 0..clauses {
                let clause: Vec<Literal> = (0..3)
                    .map(|_| Literal::new(VarId(1 + next(u64::from(n)) as u32), next(2) == 0))
                    .collect();
                f = backend.and_clause(&scope, f, &clause).unwrap();
            }
            f
        };
        let run = |clauses: usize, work: Option<u64>| -> (bool, u64) {
            tree.context().run(|engine| {
                let skip = TddBackend::in_form(engine, tree.clone(), Form::Skip);
                let f = cnf(&skip, 0x2545_F491, clauses);
                let g = cnf(&skip, 0x9E37_79B9, clauses);
                let start = skip.work_units();
                let done = match work {
                    None => skip.and(&f, &g).map(|_| true).unwrap(),
                    Some(w) => skip.within(w, || skip.and(&f, &g)).unwrap().is_some(),
                };
                (done, skip.work_units() - start)
            })
        };
        let (clauses, all) = (20..=60)
            .step_by(4)
            .map(|k| (k, run(k, None).1))
            .find(|&(_, all)| all >= SKIP_WORK_SCALE << 18)
            .expect("a conjunction large enough");
        let (done, spent) = run(clauses, Some(all / 2));
        assert!(
            !done && spent >= all / 2,
            "{clauses} clauses: {spent} of {all} units"
        );
    }
}
