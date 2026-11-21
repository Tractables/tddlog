//! A trial of `auto`: a layout, the program it compiles, its layout once
//! built, and what its attempts have built and spent.

use std::sync::Arc;

use crate::evaluation::Evaluation;
use crate::ground::GroundProgram;
use crate::ground::deps::Deps;
use crate::tp;
use crate::vtree::{self, Kind};

use super::policy::PROJECTED_DOUBLINGS;

/// The share of its phase left within which a trial cut short is
/// projected to finish in its next turn ([`Trial::projected`]).
pub(super) const NEAR_FINISHED: f64 = 1.0 / 128.0;

pub(super) struct Trial {
    pub(super) kind: Kind,
    pub(super) ground: Option<Arc<GroundProgram>>,
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
    /// The engine work its attempts have spent, and its last attempt.
    pub(super) spent: u64,
    pub(super) last_work: u64,
    /// The work its last attempt spent since its phase last kept a step
    /// ([`tp::TddBackend::lost`]): what a stop then lost.
    pub(super) lost: u64,
    /// Its target and the share of its phase finished after the last
    /// attempt at that target that left it in that phase
    /// ([`Self::projected`]).
    pub(super) progress: Vec<(u64, f64)>,
    /// The steps of its phase left after its last recorded attempt.
    pub(super) left: usize,
    /// Whether `progress` measures its query phase.
    pub(super) querying: bool,
    pub(super) evaluation: Evaluation,
    pub(super) disabled: bool,
    /// Whether it has taken an attempt ahead of every trial
    /// ([`Self::rushes`]).
    pub(super) rushed: bool,
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
            spent: 0,
            last_work: 0,
            lost: 0,
            progress: Vec::new(),
            left: 0,
            querying: false,
            evaluation: Evaluation::default(),
            disabled: false,
            rushed: false,
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

    /// Takes its weight away, once it has spent its bet on its layout's estimate
    /// ([`super::policy::spent_bet`]). A trial whose attempt the
    /// bet paused at a step it kept ([`Self::lost`] 0) resumes at its
    /// level, its target over its weight and past what it has spent; one
    /// whose operation the guard cut short, an operation that outgrew the
    /// whole bet, has nothing to resume and keeps its target, its weighted
    /// turn counted as spent.
    pub(super) fn lose_weight(&mut self) {
        if self.lost == 0 {
            self.target /= self.weight.max(1);
            while self.target <= self.spent {
                self.target = self.target.max(1).saturating_mul(2);
            }
        }
        self.weight = 1;
    }

    /// Records the share of its phase finished after an attempt that did
    /// not finish it ([`Evaluation::progress`]), at the attempt's target,
    /// in place of an earlier record at that target. The first in its query
    /// phase drops those of its compile: the phase is projected from where
    /// it began ([`Self::began`]).
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
            self.left = all.saturating_sub(done);
        }
    }

    /// The base-2 logarithm of the work it is projected to have spent once
    /// it has finished its phase: the share of the phase it finished over
    /// its last doubling of its target, by the latest record at half its
    /// last target or less, is taken to go on per doubling. A query phase
    /// with no such record is projected from where it began
    /// ([`Self::began`]): the share it finished per doubling of the work
    /// spent since. Infinite where that span finished no step of the
    /// phase; `None` before a compile has a record at half its target,
    /// or before a query phase has spent any work. The records are by target, a power
    /// of two times the first, rather than by the work spent, which an
    /// attempt stopped by work overruns by a little: by the work, a record
    /// at a hair past half the last one went unread, and the projection
    /// waited a doubling more.
    ///
    /// `None` too while its last attempt lost the operation under way at
    /// the guard ([`Self::lost`]), unless it holds the weight and the steps
    /// left ([`Self::left`]), each costing what that one lost, fit in twice
    /// its next turn: the step cut short cost more than the whole attempt, so
    /// the pace of the steps before it projects nothing of a rest that may
    /// hold more such steps, and the trial counts [`PROJECTED_DOUBLINGS`]
    /// past the target the loss raised. What it lost is only a floor on
    /// that step's cost, so the projection past it is a bet, which only the
    /// trial the run bets on takes ([`super::policy::FAMILIES_WEIGHT`]): at
    /// weight 1, `closed` on a weighted CNF benchmark, cut on the last of
    /// 77 steps, spent 2.3 G units in its next turn without finishing it
    /// where `td` finished in 0.2 G, and
    /// `families:0` on an HMM of ProbLog's tutorial, its weight lost to its
    /// estimate, spent 0.45 G units and 4 GiB where `closed` answers in
    /// 0.4 M. `families:0` on a loop of Caesar ciphers, cut at 2^29
    /// units at 81% of its query phase, was projected to finish at 2^30.8
    /// under its new target of 2^31 and took the next attempt ahead of an
    /// untried `closed`, spending 3.2 G units in one operation where
    /// `closed` answers in 3.1 M; on Dice's counter, cut with 11 of 48
    /// steps left, it was cut again at the same step and refused memory,
    /// where `closed` answers in 27 K. Where they fit, as on the last step
    /// of a phase, it keeps the projection: `families:0` on UAI's 1aie and
    /// 1f94, cut on the last of 5552 and 7959 steps, otherwise counted the
    /// most and gave `closed` and `td` 2.7 G and 5.4 G units of turns
    /// before it finished. The margin of twice the turn keeps that from
    /// turning on a few percent of the cost of the step lost: 1aie, with
    /// its query work charged 5% higher, lost the projection and gave
    /// `closed` and `td` their turns, 1.18 times the instructions. The
    /// next attempt that keeps a step projects again.
    ///
    /// A trial cut within the last [`NEAR_FINISHED`] of its phase is
    /// projected to finish in its next turn, whatever it lost and whatever
    /// its weight: pgrid18_17's `families:0`, cut at 885 of 888 query
    /// steps with 0.52 G units lost on the last, otherwise waited while
    /// `closed` and `td` took six turns of 2.7 G units in all, and then
    /// finished in 1.28 G. The share is a step count, so it holds where
    /// the work charged to the steps before it moves. The cuts above where
    /// a projection cost, on the last of 77 steps and with 11 of 48 left,
    /// are outside it.
    pub(super) fn projected(&self) -> Option<f64> {
        if self.near_finished() {
            return Some((self.target.max(1) as f64).log2());
        }
        if self.lost > 0
            && (self.weight <= 1
                || (self.left as u64).saturating_mul(self.lost)
                    > self.target.saturating_sub(self.spent).saturating_mul(2))
        {
            return None;
        }
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

    /// Whether its last attempt was cut within the last [`NEAR_FINISHED`]
    /// of its phase.
    pub(super) fn near_finished(&self) -> bool {
        self.lost > 0
            && self
                .progress
                .last()
                .is_some_and(|&(_, done)| done >= 1.0 - NEAR_FINISHED)
    }

    /// Whether it takes the next attempt ahead of every trial: cut near
    /// finished ([`Self::near_finished`]) and not yet rushed. Ranked by its
    /// key, the doubling its target took to fit twice the step it lost
    /// moved it behind trials a doubling short of it whenever that step
    /// cost a little more: UAI's 1aie, cut at 5551 of 5552 steps, with
    /// compile work charged 5% lower, gave `closed` and `td` their turns
    /// first, 1.21 times the instructions. Once only, so that a last step
    /// that no turn holds leaves the others their turns.
    pub(super) fn rushes(&self) -> bool {
        !self.rushed && self.near_finished()
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

    /// The doublings its projection puts its finish past its target, or
    /// past `earliest`, the earliest finish projected in the race
    /// ([`Self::finish`]), where that is later: so trials all projected
    /// past their targets take shares by their projections, not equal
    /// ones (arith_gcd_medium). At least none and at most
    /// [`PROJECTED_DOUBLINGS`], which is what a trial without a projection
    /// counts.
    pub(super) fn ahead(&self, earliest: f64) -> f64 {
        match self.projected() {
            Some(p) => {
                let target = (self.target.max(1) as f64).log2();
                (p - target.max(earliest)).clamp(0.0, PROJECTED_DOUBLINGS)
            }
            None => PROJECTED_DOUBLINGS,
        }
    }

    /// Its projected finish, for the race's earliest ([`Self::ahead`]):
    /// its projection, not before its target, or [`PROJECTED_DOUBLINGS`]
    /// past its target where it counts that many for want of one.
    pub(super) fn finish(&self) -> f64 {
        let target = (self.target.max(1) as f64).log2();
        match self.projected() {
            Some(p) if p.is_finite() => p.max(target),
            _ => target + PROJECTED_DOUBLINGS,
        }
    }

    /// Its place in the turns, the least first, in a race whose earliest
    /// projected finish is `earliest`: the base-2 logarithm of its target
    /// and the doublings it is projected past it ([`Self::ahead`]), for its
    /// weight. Without projections every trial counts the most and the
    /// trials take equal shares for their weights.
    pub(super) fn key(&self, earliest: f64) -> f64 {
        (self.target.max(1) as f64).log2() + self.ahead(earliest) - (self.weight as f64).log2()
    }

    /// Frees what its attempts built, and the scratch its engine kept, where
    /// it is retired ([`Self::retire`]). The layout stays, so that a later
    /// trial with the same one is not tried ([`super::prepare`]).
    pub(super) fn release(&mut self) {
        self.evaluation = Evaluation::new(self.evaluation.traced());
        self.kept = None;
        self.progress.clear();
        self.querying = false;
        if let Some(layout) = self.built.as_mut() {
            layout.vtree =
                Arc::new(tididi::Vtree::clone(&layout.vtree).with_context(Arc::default()));
        }
    }

    /// Ends the trial for the run, where an operation of its attempt was
    /// refused memory or outgrew tididi's 32-bit indices: an attempt in a
    /// later turn would resume at that operation with larger bounds and no
    /// less held, so the run goes on with the other trials, and fails with
    /// the refusal once none is left (on latin5 and gnb30 `families:0` is
    /// refused memory and `closed` finishes). What the trial built is freed
    /// ([`Self::release`]).
    pub(super) fn retire(&mut self) {
        self.release();
        self.disabled = true;
    }
}
