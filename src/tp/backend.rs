//! The diagram operations the compiler needs, on tididi's TDDs; sizes are
//! pairs. Each takes or builds its formulas on a scope ([`super::scope`]).

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::sync::Arc;

use tididi::diagram::{
    ChildPair, EncodedChildRef, NEG_LEAF_IDX, NodeIdx, ONE_LEAF_IDX, POS_LEAF_IDX, TddNodeId,
};
use tididi::limits::StopAt;
use tididi::reduce::ReductionPlan;
use tididi::restructure::EmbedError;
use tididi::restructure::search::{PoolRestarts, PoolSearchConfig, PoolSearchStats};
use tididi::vtree::{VarId, VtreeIdx};
use tididi::{Engine, Literal, OperationError, Tdd, Vtree};

use super::CompileError;
use super::projector::Projector;
use super::scope::VtreeScope;

/// A formula and the scope it lives on.
pub(crate) type Formula = (Tdd, VtreeScope);

/// A formula an operation reads: its own, which it consumes, or one a
/// store keeps ([`super::compile::Formulas`]), which a conjunction reads in
/// place, copying only the levels its result would take, and any other
/// operation copies.
#[allow(
    clippy::large_enum_variant,
    reason = "an owned read moves into the conjunction as `Cow` holds it"
)]
pub(crate) enum Read {
    Owned(Tdd),
    Shared(Arc<Tdd>),
}

impl std::ops::Deref for Read {
    type Target = Tdd;

    fn deref(&self) -> &Tdd {
        match self {
            Read::Owned(f) => f,
            Read::Shared(f) => f,
        }
    }
}

impl Read {
    /// A stored formula taken from its store: owned where no read of it
    /// is outstanding.
    pub(crate) fn taken(f: Arc<Tdd>) -> Self {
        Arc::try_unwrap(f).map_or_else(Read::Shared, Read::Owned)
    }

    /// The formula for an operation to consume: itself, or a copy of a
    /// shared one ([`TddBackend::copy`]).
    pub(crate) fn owned(self, backend: &TddBackend) -> Result<Tdd, CompileError> {
        match self {
            Read::Owned(f) => Ok(f),
            Read::Shared(f) => Arc::try_unwrap(f).or_else(|f| backend.copy(&f)),
        }
    }

    /// Run `op` on the formula, owned where no other read holds it and
    /// borrowed otherwise, where [`owned`](Self::owned) would copy it.
    pub(crate) fn with_cow<R>(self, op: impl FnOnce(Cow<'_, Tdd>) -> R) -> R {
        match self {
            Read::Owned(f) => op(Cow::Owned(f)),
            Read::Shared(f) => match Arc::try_unwrap(f) {
                Ok(f) => op(Cow::Owned(f)),
                Err(f) => op(Cow::Borrowed(&f)),
            },
        }
    }

    /// The formula as a conjunction takes it, with the store's hold on a
    /// shared one, which the borrow reads through ([`lend`]).
    fn split(self) -> (Option<Tdd>, Option<Arc<Tdd>>) {
        match self {
            Read::Owned(f) => (Some(f), None),
            Read::Shared(f) => (None, Some(f)),
        }
    }
}

/// An operand of a conjunction split from a [`Read`]: the owned formula,
/// or a borrow of the shared one.
fn lend(owned: Option<Tdd>, shared: &Option<Arc<Tdd>>) -> Cow<'_, Tdd> {
    match (owned, shared) {
        (Some(f), _) => Cow::Owned(f),
        (None, Some(f)) => Cow::Borrowed(f),
        (None, None) => unreachable!("a read is owned or shared"),
    }
}

/// The [`Read`] a refused conjunction hands back of an operand [`lend`]
/// made.
fn unlend(back: Cow<'_, Tdd>, shared: &Option<Arc<Tdd>>) -> Read {
    match back {
        Cow::Owned(f) => Read::Owned(f),
        Cow::Borrowed(_) => Read::Shared(Arc::clone(
            shared.as_ref().expect("a borrow of a shared read"),
        )),
    }
}

/// A [`Read`] and the scope it lives on.
pub(crate) struct Part(pub(crate) Read, pub(crate) VtreeScope);

impl From<Formula> for Part {
    fn from((f, scope): Formula) -> Self {
        Part(Read::Owned(f), scope)
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
}

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

impl<'a> TddBackend<'a> {
    pub(crate) fn new(engine: &'a Engine, vtree: Arc<Vtree>) -> Self {
        Self::resume(engine, vtree, None)
    }

    /// A backend over `vtree`, with the projector of `kept` where that is
    /// over the same vtree.
    pub(crate) fn resume(
        engine: &'a Engine,
        vtree: Arc<Vtree>,
        kept: Option<KeptProjector>,
    ) -> Self {
        let projector = match kept {
            Some(kept) if Arc::ptr_eq(&kept.vtree, &vtree) => kept.projector,
            _ => Projector::new(&vtree),
        };
        TddBackend {
            engine,
            vtree: RefCell::new(vtree),
            prune: Cell::new(FullSchedule::default()),
            projector: RefCell::new(projector),
            none_of: RefCell::default(),
            kept_at: Cell::new(engine.limits().work_units()),
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
    pub(crate) fn copy(&self, f: &Tdd) -> Result<Tdd, CompileError> {
        Ok(self.engine.try_clone(f)?)
    }

    /// Embed a stored formula without first copying its source tree. The
    /// embedding itself copies levels into the destination tree; only an
    /// unchanged scope needs a copy to produce an owned operand.
    pub(crate) fn embed_ref(
        &self,
        f: &Tdd,
        from: &VtreeScope,
        into: &VtreeScope,
    ) -> Result<Tdd, CompileError> {
        if from.same(into) {
            return self.copy(f);
        }
        let map = Self::scope_map(from, into)?;
        let (embedded, _) = self
            .engine
            .embed(f, into.vtree(), |v| map[v.0 as usize - 1])?;
        Ok(embedded)
    }

    /// `f` copied onto `into` with each variable of its vtree read as its
    /// image under `map`; `None` where `into` does not hold its shape under
    /// that renaming, or up to mirrors where `mirrored` says so.
    fn embed_mapped(
        &self,
        f: &Tdd,
        into: &VtreeScope,
        map: impl Fn(VarId) -> VarId,
        mirrored: bool,
    ) -> Result<Option<Tdd>, CompileError> {
        let embedded = if mirrored {
            self.engine.embed_mirrored(f, into.vtree(), map)
        } else {
            self.engine.embed(f, into.vtree(), map)
        };
        match embedded {
            Ok((embedded, _)) => Ok(Some(embedded)),
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
        f: Tdd,
        from: &VtreeScope,
        into: &VtreeScope,
        map: Option<&[VarId]>,
    ) -> Result<Tdd, Box<(CompileError, Tdd)>> {
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
        match self
            .engine
            .embed_moving(f, into.vtree(), |v| map[v.0 as usize - 1])
        {
            Ok((embedded, _)) => Ok(embedded),
            Err(refused) => Err(Box::new((refused.error.into(), refused.tdd))),
        }
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
    ) -> Result<Tdd, Box<(CompileError, Part, Part)>> {
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
        let (Part(a, own_a), Part(b, own_b)) = (fa, fb);
        let ((a, shared_a), (b, shared_b)) = (a.split(), b.split());
        self.engine
            .and_onto(
                lend(a, &shared_a),
                |v| map_a[v.0 as usize - 1],
                lend(b, &shared_b),
                |v| map_b[v.0 as usize - 1],
                scope.vtree(),
            )
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
    ) -> Result<Tdd, CompileError> {
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
        f: Tdd,
        from: &VtreeScope,
        into: &VtreeScope,
    ) -> Result<Tdd, CompileError> {
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
    ) -> Result<Tdd, CompileError> {
        self.embed(f.owned(self)?, from, into)
    }

    /// `f`, which lives on `from` and depends on none of its variables but
    /// `vars`, ascending, on the scope over `vars`: `into`, where the caller
    /// holds that scope, and otherwise a new one, which it returns. The
    /// inverse of [`embed`](Self::embed): the levels are copied, those of
    /// the dropped variables left out.
    pub(crate) fn restrict(
        &self,
        f: &Tdd,
        from: &VtreeScope,
        vars: &[VarId],
        into: Option<&VtreeScope>,
    ) -> Result<(Tdd, VtreeScope), CompileError> {
        let local = |v: VarId| {
            let v = from.vars().map_or(v, |held| held[v.0 as usize - 1]);
            vars.binary_search(&v).ok().map(|i| VarId(i as u32 + 1))
        };
        let restricted = self.engine.project_to_vars(f, local, vars.len() as u32)?;
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
        f: &Tdd,
        from: &VtreeScope,
        into: &VtreeScope,
        (old, new): (VarId, VarId),
    ) -> Result<Option<Tdd>, CompileError> {
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
        f: &Tdd,
        from: &VtreeScope,
        to: &VtreeScope,
        map: &[VarId],
    ) -> Result<Option<Tdd>, CompileError> {
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

    pub(crate) fn zero(&self, scope: &VtreeScope) -> Tdd {
        self.engine.zero(scope.vtree())
    }
    pub(crate) fn one(&self, scope: &VtreeScope) -> Tdd {
        self.engine.one(scope.vtree())
    }
    /// `lit` names a variable of the program; the backend renames it into
    /// `scope`.
    pub(crate) fn literal(&self, scope: &VtreeScope, lit: Literal) -> Result<Tdd, CompileError> {
        Ok(self
            .engine
            .literal(scope.vtree(), Self::renamed(scope, lit)?)?)
    }
    /// Conjunction of literals (`one` for none), built in one pass.
    pub(crate) fn cube(&self, scope: &VtreeScope, lits: &[Literal]) -> Result<Tdd, CompileError> {
        let mut renamed = Vec::with_capacity(lits.len());
        for &l in lits {
            renamed.push(Self::renamed(scope, l)?);
        }
        Ok(self.engine.cube(scope.vtree(), renamed)?)
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
    ) -> Result<Tdd, CompileError> {
        let vtree = scope.vtree();
        if let [l] = *lits {
            return if exactly {
                self.literal(scope, l)
            } else {
                Ok(self.one(scope))
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
        let mut f = b.finish(output).map_err(OperationError::from)?;
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
    ) -> Result<Option<Tdd>, CompileError> {
        if vars.is_empty() {
            return Ok(Some(if cubes.is_empty() {
                self.one(scope)
            } else {
                self.zero(scope)
            }));
        }
        if let [v] = *vars {
            return Ok(Some(match super::none_of::one_variable(cubes, groups) {
                (true, true) => self.one(scope),
                (true, false) => self.literal(scope, Literal::pos(v))?,
                (false, true) => self.literal(scope, Literal::neg(v))?,
                (false, false) => self.zero(scope),
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
        Ok(Some(self.embed(f, &own, scope)?))
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
    ) -> Result<Tdd, CompileError> {
        let own = self.scope_of(scope, vars)?;
        let local: Vec<VarId> = (1..=vars.len() as u32).map(VarId).collect();
        let f = self.engine.from_models(own.vtree(), &local, models)?;
        self.embed(f, &own, scope)
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
    pub(crate) fn select_at_root(
        &self,
        scope: &VtreeScope,
        v: VarId,
        x: &Tdd,
        y: &Tdd,
    ) -> Result<Option<Tdd>, CompileError> {
        let vtree = scope.vtree();
        let root = vtree.root();
        if vtree.node(root).is_leaf() || x.is_zero() || y.is_zero() {
            return Ok(None);
        }
        let leaf = vtree
            .leaf_of(Self::renamed(scope, Literal::pos(v))?.var)
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
        let (xc, yc) = (NodeIdx(xc), NodeIdx(yc));
        let pairs = if v_left {
            [
                ChildPair::new(POS_LEAF_IDX, xc),
                ChildPair::new(NEG_LEAF_IDX, yc),
            ]
        } else {
            [
                ChildPair::new(xc, POS_LEAF_IDX),
                ChildPair::new(yc, NEG_LEAF_IDX),
            ]
        };
        let node = b.push(self.engine, root, &pairs)?;
        Ok(Some(
            b.finish(TddNodeId {
                vtree: root,
                local: node,
            })
            .map_err(OperationError::from)?,
        ))
    }
    /// `a ∧ b`, each operand owned or borrowed: a borrowed one is read in
    /// place.
    pub(crate) fn and<'a, 'b>(
        &self,
        a: impl Into<Cow<'a, Tdd>>,
        b: impl Into<Cow<'b, Tdd>>,
    ) -> Result<Tdd, CompileError> {
        Ok(self.engine.and(a, b)?)
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
    ) -> Result<Result<Tdd, (Part, Part)>, CompileError> {
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
    /// The run's stop in work units moved `work` units later, held while
    /// the scope lives; `None` where the run has no such stop.
    pub(crate) fn stop_later(&self, work: u64) -> Option<tididi::limits::LimitScope<'_>> {
        let limits = self.engine.limits();
        let rules = limits.armed().stop_rules();
        let Some(StopAt::WorkUnits(u)) = rules.unconditional else {
            return None;
        };
        Some(limits.edit(|c| {
            c.with_stop_rules(
                rules.with_unconditional(Some(StopAt::WorkUnits(u.saturating_add(work)))),
            )
        }))
    }
    pub(crate) fn or(&self, a: Tdd, b: Tdd) -> Result<Tdd, CompileError> {
        Ok(self.engine.or(a, b)?)
    }
    /// `a ∧ (l_1 ∨ ... ∨ l_k)`, with `a` on `scope`.
    pub(crate) fn and_clause(
        &self,
        scope: &VtreeScope,
        a: Tdd,
        clause: &[Literal],
    ) -> Result<Tdd, CompileError> {
        let mut renamed = Vec::with_capacity(clause.len());
        for &l in clause {
            renamed.push(Self::renamed(scope, l)?);
        }
        Ok(self.engine.and_clause(a, &renamed)?)
    }
    pub(crate) fn negate<'f>(&self, a: impl Into<Cow<'f, Tdd>>) -> Result<Tdd, CompileError> {
        // The compiler minimizes every result it keeps, so the full reduction
        // `negate` ends with by default is paid twice. Pruning is the part
        // negation cannot skip: it removes the operand's dead cone, the nodes
        // the root complement orphans. Contraction is what the compiler
        // repeats, and tididi does not split `One` on the way in, so there
        // are next to no twins left for it to merge anyway.
        Ok(self.engine.negate_with(a, ReductionPlan::Prune)?)
    }
    /// Reduce `f`. Between operations a backend may reduce less than fully,
    /// so the result is not necessarily the canonical form `equivalent`
    /// compares; `equivalent` reduces what it compares itself. A formula the
    /// compiler settles on is always reduced in full.
    pub(crate) fn minimize(&self, f: &mut Tdd, how: Reduce) -> Result<(), CompileError> {
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
    pub(crate) fn equivalent(&self, a: &Tdd, b: &Tdd) -> Result<bool, CompileError> {
        Ok(self.engine.equivalent(a, b)?)
    }
    pub(crate) fn is_zero(&self, f: &Tdd) -> bool {
        f.is_zero()
    }
    /// The size the backend pays for `f`.
    pub(crate) fn size(&self, f: &Tdd) -> usize {
        f.pair_count()
    }
    /// The pairs of `f` its levels hold as their description rather than
    /// written ([`tididi::diagram::TddLevel::implicit`]).
    pub(crate) fn implicit_size(&self, f: &Tdd) -> usize {
        f.vtree()
            .internal_bottomup()
            .filter_map(|(t, _, _)| f.level(t).implicit())
            .map(|d| d.pairs())
            .sum()
    }
    /// The nodes of `f`, read in one step per level rather than per node:
    /// a proxy for [`TddBackend::size`] cheap enough to read after every
    /// clause.
    pub(crate) fn nodes(&self, f: &Tdd) -> usize {
        f.node_count()
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

    /// Search for a global vtree on which `members` are smaller, sweeping
    /// until a sweep keeps no move, then running `restarts`, their seed
    /// mixed with the members' live pairs so that each search draws its
    /// own. The members must be every live formula on the global scope;
    /// they keep their functions, and afterwards [`TddBackend::global`] is
    /// the scope they live on.
    pub(crate) fn restructure(
        &self,
        members: &mut [&mut Tdd],
        restarts: PoolRestarts,
    ) -> Result<PoolSearchStats, CompileError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tp::scope::scope_shape;
    use tididi::vtree::VtreeNode;

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
        let (engine, vtree) = (backend.engine, scope.vtree());
        let root = vtree.root();
        if vtree.node(root).is_leaf() || x.is_zero() || y.is_zero() {
            return None;
        }
        let leaf = vtree
            .leaf_of(TddBackend::renamed(scope, Literal::pos(v)).unwrap().var)
            .unwrap();
        let (l, r) = vtree.children(root);
        let (c, v_left) = match leaf {
            _ if leaf == l => (r, true),
            _ if leaf == r => (l, false),
            _ => return None,
        };
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
        let (xc, yc) = (at_c(x)?, at_c(y)?);
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
        let (xc, yc) = (NodeIdx(side(0, xc)), NodeIdx(side(1, yc)));
        let pairs = if v_left {
            [
                ChildPair::new(POS_LEAF_IDX, xc),
                ChildPair::new(NEG_LEAF_IDX, yc),
            ]
        } else {
            [
                ChildPair::new(xc, POS_LEAF_IDX),
                ChildPair::new(yc, NEG_LEAF_IDX),
            ]
        };
        let node = b.push(engine, root, &pairs).unwrap();
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
                    let mut x = backend.zero(&own);
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
                    let other = reversed(engine, &y);
                    if x.is_zero() || y.is_zero() {
                        assert!(backend.select_at_root(&scope, v, &x, &y).unwrap().is_none());
                        return;
                    }
                    let built = backend
                        .select_at_root(&scope, v, &x, &y)
                        .unwrap()
                        .expect("v is a child of the root");
                    let interned = select_by_interning(&backend, &scope, v, &x, &y)
                        .expect("v is a child of the root");
                    assert!(
                        same_levels(&built, &interned),
                        "seed {seed}: not the interned diagram"
                    );
                    let built = backend
                        .select_at_root(&scope, v, &x, &other)
                        .unwrap()
                        .expect("v is a child of the root");
                    let interned = select_by_interning(&backend, &scope, v, &x, &other)
                        .expect("v is a child of the root");
                    assert!(
                        same_levels(&built, &interned),
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
                let mut exactly = backend.zero(&scope);
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
            let mut groups: Vec<(Vec<Literal>, bool)> = Vec::new();
            let mut v = 1;
            while v < n && next(3) > 0 {
                let size = (2 + next(3) as u32).min(n + 1 - v);
                groups.push((
                    (v..v + size).map(|x| Literal::pos(VarId(x))).collect(),
                    next(2) == 0,
                ));
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
                let mut applied = backend.one(&scope);
                for cube in &cubes {
                    let clause: Vec<Literal> = cube.iter().map(|l| l.negated()).collect();
                    applied = backend.and_clause(&scope, applied, &clause).unwrap();
                }
                for (g, exactly) in &groups {
                    for (i, &a) in g.iter().enumerate() {
                        for &b in &g[i + 1..] {
                            applied = backend
                                .and_clause(&scope, applied, &[a.negated(), b.negated()])
                                .unwrap();
                        }
                    }
                    if *exactly {
                        applied = backend.and_clause(&scope, applied, g).unwrap();
                    }
                }
                let mut vars: Vec<VarId> = cubes
                    .iter()
                    .flatten()
                    .chain(groups.iter().flat_map(|(g, _)| g))
                    .map(|l| l.var)
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
                    .map(|(g, exactly)| (g.iter().map(|l| local(l.var)).collect(), *exactly))
                    .collect();
                let built = backend
                    .none_of(&scope, &vars, &local_cubes, &local_groups, u64::MAX)
                    .unwrap()
                    .expect("no bound");
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
                let mut applied = backend.one(&scope);
                for cube in &cubes {
                    let clause: Vec<Literal> = cube.iter().map(|l| l.negated()).collect();
                    applied = backend.and_clause(&scope, applied, &clause).unwrap();
                }
                for (g, exactly) in &groups {
                    for (i, &a) in g.iter().enumerate() {
                        for &b in &g[i + 1..] {
                            applied = backend
                                .and_clause(&scope, applied, &[Literal::neg(a), Literal::neg(b)])
                                .unwrap();
                        }
                    }
                    if *exactly {
                        let clause: Vec<Literal> = g.iter().map(|&v| Literal::pos(v)).collect();
                        applied = backend.and_clause(&scope, applied, &clause).unwrap();
                    }
                }
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
                        let mut applied = backend.one(&scope);
                        for signs in &picked {
                            let clause: Vec<Literal> =
                                signs.iter().map(|&s| Literal::new(x, !s)).collect();
                            applied = if clause.is_empty() {
                                backend.zero(&scope)
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
                let mut applied = backend.one(&scope);
                for cube in &cubes {
                    let clause: Vec<Literal> = cube.iter().map(|l| l.negated()).collect();
                    applied = backend.and_clause(&scope, applied, &clause).unwrap();
                }
                for (g, exactly) in &groups {
                    for (i, &a) in g.iter().enumerate() {
                        for &b in &g[i + 1..] {
                            applied = backend
                                .and_clause(&scope, applied, &[Literal::neg(a), Literal::neg(b)])
                                .unwrap();
                        }
                    }
                    if *exactly {
                        let clause: Vec<Literal> = g.iter().map(|&v| Literal::pos(v)).collect();
                        applied = backend.and_clause(&scope, applied, &clause).unwrap();
                    }
                }
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
                let built = backend
                    .none_of(&scope, &vars, &local_cubes, &local_groups, u64::MAX)
                    .unwrap()
                    .expect("no bound");
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
                let mut applied = backend.one(&scope);
                for cube in &cubes {
                    let clause: Vec<Literal> = cube.iter().map(|l| l.negated()).collect();
                    applied = backend.and_clause(&scope, applied, &clause).unwrap();
                }
                for (g, exactly) in &groups {
                    for (i, &a) in g.iter().enumerate() {
                        for &b in &g[i + 1..] {
                            applied = backend
                                .and_clause(&scope, applied, &[Literal::neg(a), Literal::neg(b)])
                                .unwrap();
                        }
                    }
                    if *exactly {
                        let clause: Vec<Literal> = g.iter().map(|&v| Literal::pos(v)).collect();
                        applied = backend.and_clause(&scope, applied, &clause).unwrap();
                    }
                }
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
            .transport(&f, &scope(from, 1), &scope(to, n + 1), &map)
            .unwrap();
        if let Some(copied) = &copied {
            let renamed = backend
                .engine
                .cube(to, (0..n as usize).map(|k| lit(k, map[k])))
                .unwrap();
            assert!(
                backend.engine.equivalent(copied, &renamed).unwrap(),
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
}
