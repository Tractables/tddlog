//! A trial of `auto`: a layout, the program it compiles, its layout once
//! built, and what its attempts have built and spent.

use std::sync::Arc;

use crate::evaluation::Evaluation;
use crate::ground::GroundProgram;
use crate::ground::deps::Deps;
use crate::tp;
use crate::vtree::{self, Kind};

use super::policy::{PROJECTED_DOUBLINGS, WORK_PER_HELD_PAIR};

pub(super) struct Trial {
    pub(super) kind: Kind,
    pub(super) ground: Option<GroundProgram>,
    /// The rules by head and components of `ground` ([`Deps`]), worked out
    /// for its first layout, and shared by the trials of its rewrite,
    /// their layouts and their compiles.
    pub(super) deps: Option<Arc<Deps>>,
    pub(super) built: Option<vtree::Layout>,
    /// What its layout is estimated to hold ([`vtree::build`]).
    pub(super) estimate: Option<vtree::Estimate>,
    /// The vtree its last attempt ended on, with that attempt's projector
    /// ([`tp::TddBackend::resume`]).
    pub(super) kept: Option<tp::KeptProjector>,
    /// Its share of the work beside the other trials ([`super::policy::weight`]).
    pub(super) weight: u64,
    /// The work it may have spent by the end of its next turn.
    pub(super) target: u64,
    /// The least watch that holds the formulas that stopped its attempts.
    pub(super) held: usize,
    /// The engine work its attempts have spent, and its last attempt.
    pub(super) spent: u64,
    pub(super) last_work: u64,
    /// Its target and the share of its phase finished after the last
    /// attempt at that target that left it in that phase
    /// ([`Self::projected`]).
    pub(super) progress: Vec<(u64, f64)>,
    /// Whether `progress` measures its query phase.
    pub(super) querying: bool,
    pub(super) evaluation: Evaluation,
    pub(super) disabled: bool,
    /// Whether the others' memory has been freed for it, which happens at
    /// most once a run ([`super::run`]).
    pub(super) freed_for: bool,
}

impl Trial {
    pub(super) fn new(kind: Kind) -> Self {
        Self {
            kind,
            ground: None,
            deps: None,
            built: None,
            estimate: None,
            kept: None,
            weight: 1,
            target: 0,
            held: 0,
            spent: 0,
            last_work: 0,
            progress: Vec::new(),
            querying: false,
            evaluation: Evaluation::default(),
            disabled: false,
            freed_for: false,
        }
    }

    /// Whether its layout may have added copies to its rewrite: a `td`
    /// trial's separator copies, or a `program` or `decision` trial's
    /// names, which give its rewrite indicators.
    pub(super) fn copied(&self) -> bool {
        self.built.is_some()
            && match self.kind {
                Kind::Td => true,
                Kind::Program { .. } | Kind::Decision => {
                    self.ground.as_ref().is_some_and(|g| !g.encoding.is_empty())
                }
                _ => false,
            }
    }

    /// The pairs of the largest formula its next attempt may complete: a
    /// [`WORK_PER_HELD_PAIR`]th of its target, and at least `first_pairs`
    /// and what holds the formulas that stopped it ([`Self::hold`]).
    pub(super) fn watch(&self, first_pairs: usize) -> usize {
        first_pairs
            .max(self.held)
            .max(usize::try_from(self.target / WORK_PER_HELD_PAIR).unwrap_or(usize::MAX))
    }

    /// Raises the watch, by powers of two, until it holds a formula of
    /// `pairs` that stopped an attempt.
    pub(super) fn hold(&mut self, pairs: usize, first_pairs: usize) {
        let mut watch = self.watch(first_pairs);
        while watch < pairs {
            watch = watch.saturating_mul(2);
        }
        self.held = watch;
    }

    /// Records the share of its phase finished after an attempt that did
    /// not finish it ([`Evaluation::progress`]), at the attempt's target,
    /// in place of an earlier record at that target. The first in its
    /// query phase drops those of its compile: the phase is projected from
    /// where it began ([`Self::began`]).
    pub(super) fn record_progress(&mut self) {
        self.record(self.evaluation.is_compiled(), self.evaluation.progress());
    }

    /// [`Self::record_progress`] after an attempt that left the trial with
    /// its compile finished or not, and `progress` of its phase.
    pub(super) fn record(&mut self, compiled: bool, progress: Option<(usize, usize)>) {
        if compiled && !self.querying {
            self.querying = true;
            self.progress.clear();
        }
        if let Some((done, all)) = progress
            && all > 0
        {
            if self.progress.last().is_some_and(|&(t, _)| t == self.target) {
                self.progress.pop();
            }
            self.progress.push((self.target, done as f64 / all as f64));
        }
    }

    /// The base-2 logarithm of the work it is projected to have spent once
    /// it has finished its phase: the share of the phase it finished over
    /// its last doubling of its target, by the latest record at half its
    /// last target or less, is taken to go on per doubling. A query phase
    /// with no such record is projected from where it began
    /// ([`Self::began`]): the share it finished per doubling of the work
    /// spent since. Infinite where that span finished no step of the
    /// phase; `None` before a compile's target has doubled, or before a
    /// query phase has spent any work. The records are by target, a power
    /// of two times the first, rather than by the work spent, which an
    /// attempt stopped by work overruns by a little: by the work, a record
    /// at a hair past half the last one went unread, and the projection
    /// waited a doubling more.
    pub(super) fn projected(&self) -> Option<f64> {
        let &(target, done) = self.progress.last()?;
        let before = self.progress.iter().rev().find(|&&(t, _)| t <= target / 2);
        let (at, doublings, finished) = match before {
            Some(&(before, done_before)) => (
                target as f64,
                (target as f64 / before as f64).log2(),
                done - done_before,
            ),
            None if self.querying && self.began() > 0 && self.began() < self.spent => (
                self.spent as f64,
                (self.spent as f64 / self.began() as f64).log2(),
                done,
            ),
            None => return None,
        };
        let per_doubling = finished / doublings;
        Some(if per_doubling > 0.0 {
            at.log2() + (1.0 - done) / per_doubling
        } else {
            f64::INFINITY
        })
    }

    /// The work it had spent when its query phase began: what it has spent
    /// but for the query phase's work ([`Evaluation::query_work`]). The
    /// phase begins where the compile finishes, within an attempt, so the
    /// target of the attempt before is no measure of it: spacex_p16's
    /// `program` finished its compile late in its turn of 2^33 units and
    /// three of its nineteen queries in what was left; measured from
    /// 2^32, that pace was projected 5.3 doublings out, and `closed` took
    /// two turns of 104 s before the few seconds `program` needed.
    pub(super) fn began(&self) -> u64 {
        self.spent.saturating_sub(self.evaluation.query_work)
    }

    /// The doublings its projection puts its finish past its target
    /// ([`Self::projected`]), at least none and at most
    /// [`PROJECTED_DOUBLINGS`], which is what a trial without a projection
    /// counts.
    pub(super) fn ahead(&self) -> f64 {
        let target = (self.target.max(1) as f64).log2();
        self.projected().map_or(PROJECTED_DOUBLINGS, |p| {
            (p - target).clamp(0.0, PROJECTED_DOUBLINGS)
        })
    }

    /// Its place in the turns, the least first: the base-2 logarithm of the
    /// work it is projected to have spent once it has finished its phase,
    /// for its weight, between its target and [`PROJECTED_DOUBLINGS`]
    /// doublings past it ([`Self::ahead`]). Without projections every
    /// trial counts the most and the trials take equal shares for their
    /// weights.
    pub(super) fn key(&self) -> f64 {
        (self.target.max(1) as f64).log2() + self.ahead() - (self.weight as f64).log2()
    }

    /// Whether its attempts left anything built: the formulas of its phase,
    /// or the vtree its engine kept between attempts.
    pub(super) fn holds(&self) -> bool {
        self.kept.is_some() || self.evaluation.is_compiled() || self.evaluation.progress().is_some()
    }

    /// Frees what its attempts built, and the scratch its engine kept: its
    /// next attempt begins its phase again, on its layout, with the target,
    /// the watch and the work spent it has (`super::run_within`, where
    /// another trial is refused memory). The layout stays, so that a later
    /// trial with the same one is not tried ([`super::prepare`]).
    pub(super) fn release(&mut self) {
        self.evaluation = Evaluation::default();
        self.kept = None;
        self.progress.clear();
        self.querying = false;
        if let Some(layout) = self.built.as_mut() {
            layout.vtree =
                Arc::new(tididi::Vtree::clone(&layout.vtree).with_context(Arc::default()));
        }
    }

    /// Ends the trial for the run, where an operation of its attempt was
    /// refused memory while no other trial held any, or outgrew tididi's
    /// 32-bit indices: an attempt in a later turn would resume at that
    /// operation with larger bounds and no less held, so the run goes on
    /// with the other trials, and fails with the refusal once none is left
    /// (on latin5 and gnb30 `families:0` is refused memory and `closed`
    /// finishes). What the trial built is freed ([`Self::release`]).
    pub(super) fn retire(&mut self) {
        self.release();
        self.disabled = true;
    }
}
