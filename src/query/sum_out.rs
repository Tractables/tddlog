//! Summing out the evidence walk's finished subtrees, where only `P(e)` is
//! wanted.
//!
//! Where no query reads the evidence formula (`--no-queries`), the walk's
//! result is only counted, and a variable can be summed out of a join's
//! result once no later join can read it. A subtree `t` of the result's
//! scope is finished where the global node over its leaves, `g`, holds no
//! variable another slot holds: no later join can then put another
//! operand's variable under the image of `t`, and every later scope has
//! `g`'s variables of the result where this one has them. Slots hold the
//! variables of the observations conjoined into them, and a join's
//! operands hold their shared variables once more than its result does; so
//! `t` is finished where the holdings of the variables under `g`, summed,
//! are the result's own variables under `g`. A Fenwick tree over the
//! global vtree's leaves holds the holdings, and a sparse table over the
//! nodes that split adjacent leaves finds `g`.
//!
//! A join sums out every finished subtree of its scope as it builds it
//! (`Engine::and_marginalizing`, which folds each cell of such a subtree
//! into a weighted value before its pairs are kept). The weights are
//! scaled floats (`Arithmetic::ScaledFloat`): a subtree whose parent the
//! join also sums out keeps its values in columns of 8 bytes a value that
//! the parent reads and drops, and only the topmost finished subtrees reach
//! the result's weight store. The result carries the facts' weights; a
//! later join with it places it with the weights (`Engine::embed_over`)
//! and conjoins weighted, and the walk's last result is counted by its
//! weighted value.
//! On the model counting competition's random_013 under `td` the walk's
//! largest formula went from 437 M pairs to 7.2 M, and its work from 4.71 G
//! units to 379 M; the UAI 2022 PR protein networks 1bxv and 1jer held
//! 2.1 and 4.3 GB at their peak instead of 10.1 and 15.1 GB in signed logs
//! and maximal subtrees.

use std::borrow::Cow;

use num_rational::BigRational;
use tididi::diagram::{Arithmetic, LiteralWeights, RationalWeights, WeightStore};
use tididi::vtree::{VarId, VtreeIdx};
use tididi::{OperationError, Tdd};

use super::Formula;
use crate::eval::Probabilities;
use crate::tp::backend::Part;
use crate::tp::{CompileError, TddBackend, VtreeScope};

/// The variable that sets the sum-out: unset, joins sum out their
/// finished subtrees from [`SUM_OUT_FROM`] pairs; `off`, none do; a number
/// of pairs, joins whose operands hold that many pairs or more do.
pub const SUM_OUT_VAR: &str = "TDDLOG_SUM_OUT";

/// The least pairs a join's operands hold for it to sum out its finished
/// subtrees, where [`SUM_OUT_VAR`] is unset.
pub const SUM_OUT_FROM: usize = 0;

/// The least operand pairs [`SUM_OUT_VAR`] gives, `None` where it is
/// `off`; a value that does not parse is an error.
pub fn sum_out_from_env() -> Result<Option<usize>, String> {
    match std::env::var(SUM_OUT_VAR).as_deref() {
        Ok("off") => Ok(None),
        Ok(v) => v
            .parse()
            .map(Some)
            .map_err(|e| format!("{SUM_OUT_VAR}={v}: not `off` or a number of pairs: {e}")),
        Err(std::env::VarError::NotPresent) => Ok(Some(SUM_OUT_FROM)),
        Err(e) => Err(format!("{SUM_OUT_VAR}: {e}")),
    }
}

/// A join refused: the error, and both operands.
type Refused = Box<(CompileError, Part, Part)>;

/// The finished subtrees of the walk's joins, and the facts' weights to
/// sum them out under (see the module documentation).
pub(super) struct SumOut {
    /// Per slot, the program variables of the observations conjoined into
    /// it, ascending.
    support: Vec<Vec<VarId>>,
    /// The global vtree's leaves from left to right: each program
    /// variable's position, each node's positions `[lo, hi)`, each node's
    /// depth, and for position `k`, the node whose children split `k` from
    /// `k + 1`; `shallowest[j][k]` is the shallowest of those over `k ..
    /// k + 2^j`. The global node over positions `lo < hi` is the
    /// shallowest split between them.
    position: Vec<u32>,
    range: Vec<(u32, u32)>,
    depth: Vec<u32>,
    split: Vec<VtreeIdx>,
    leaf_at: Vec<VtreeIdx>,
    shallowest: Vec<Vec<u32>>,
    /// A Fenwick tree over positions: the slots holding each variable.
    held: Vec<i64>,
    /// The facts' weights, by program variable less one.
    weights: Vec<LiteralWeights<BigRational>>,
    /// The least pairs a join's operands hold for it to sum out.
    from: usize,
    /// The join begun: its result's variables, and the variables its
    /// operands share, whose holdings it dropped.
    merged: Vec<VarId>,
    shared: Vec<VarId>,
    /// The joins that summed out as they were built, and the other joins
    /// with an operand that carries weights.
    pub(super) summing: usize,
    pub(super) weighted: usize,
}

impl SumOut {
    /// On the global vtree `tree`, with the facts' weights `w`, summing out
    /// at joins whose operands hold `from` pairs or more.
    pub(super) fn new(tree: &tididi::Vtree, w: &Probabilities, from: usize) -> Self {
        let rational = |x: f64| BigRational::from_float(x).expect("a weight is finite");
        let n = tree.num_nodes();
        let mut range = vec![(0u32, 0u32); n];
        let mut depth = vec![0u32; n];
        let mut leaf_at = Vec::new();
        let mut position = vec![0u32; tree.num_vars() as usize + 1];
        // Left before right, each node's range closed after its children's.
        let mut stack = vec![(tree.root(), false)];
        while let Some((t, done)) = stack.pop() {
            let node = tree.node(t);
            if node.is_leaf() {
                let p = leaf_at.len() as u32;
                position[tree.leaf_var(t).0 as usize] = p;
                leaf_at.push(t);
                range[t.idx()] = (p, p + 1);
            } else if done {
                let (l, r) = tree.children(t);
                range[t.idx()] = (range[l.idx()].0, range[r.idx()].1);
            } else {
                let (l, r) = tree.children(t);
                depth[l.idx()] = depth[t.idx()] + 1;
                depth[r.idx()] = depth[t.idx()] + 1;
                stack.push((t, true));
                stack.push((r, false));
                stack.push((l, false));
            }
        }
        let leaves = leaf_at.len();
        let mut split = vec![tree.root(); leaves.saturating_sub(1)];
        for t in tree.bottomup() {
            if !tree.node(t).is_leaf() {
                let (l, _) = tree.children(t);
                split[range[l.idx()].1 as usize - 1] = t;
            }
        }
        let mut shallowest = vec![(0..split.len() as u32).collect::<Vec<u32>>()];
        let mut span = 1;
        while 2 * span <= split.len() {
            let below = shallowest.last().unwrap();
            let level: Vec<u32> = (0..=split.len() - 2 * span)
                .map(|k| {
                    let (x, y) = (below[k], below[k + span]);
                    if depth[split[y as usize].idx()] < depth[split[x as usize].idx()] {
                        y
                    } else {
                        x
                    }
                })
                .collect();
            shallowest.push(level);
            span *= 2;
        }
        let weights = (1..w.pos.len())
            .map(|v| LiteralWeights {
                negative: rational(w.neg[v]),
                positive: rational(w.pos[v]),
            })
            .collect();
        SumOut {
            support: Vec::new(),
            position,
            range,
            depth,
            split,
            leaf_at,
            shallowest,
            held: vec![0; leaves + 1],
            weights,
            from,
            merged: Vec::new(),
            shared: Vec::new(),
            summing: 0,
            weighted: 0,
        }
    }

    fn hold(&mut self, v: VarId, delta: i64) {
        let mut i = self.position[v.0 as usize] as usize + 1;
        while i < self.held.len() {
            self.held[i] += delta;
            i += i & i.wrapping_neg();
        }
    }

    /// The holdings of the variables at positions before `p`.
    fn held_before(&self, p: u32) -> i64 {
        let (mut i, mut s) = (p as usize, 0);
        while i > 0 {
            s += self.held[i];
            i &= i - 1;
        }
        s
    }

    /// The global node over positions `lo ..= hi`.
    fn over(&self, lo: u32, hi: u32) -> VtreeIdx {
        if lo == hi {
            return self.leaf_at[lo as usize];
        }
        let (lo, hi) = (lo as usize, hi as usize);
        let j = (usize::BITS - 1 - (hi - lo).leading_zeros()) as usize;
        let (x, y) = (self.shallowest[j][lo], self.shallowest[j][hi - (1 << j)]);
        let (x, y) = (self.split[x as usize], self.split[y as usize]);
        if self.depth[y.idx()] < self.depth[x.idx()] {
            y
        } else {
            x
        }
    }

    /// Slot `id` holds an observation on `scope`.
    pub(super) fn observe(&mut self, id: usize, scope: &VtreeScope) {
        if self.support.len() <= id {
            self.support.resize_with(id + 1, Vec::new);
        }
        let vars: Vec<VarId> = match scope.vars() {
            Some(v) => v.to_vec(),
            None => {
                let mut all: Vec<VarId> = scope.vtree().leaf_bottomup().map(|(_, v)| v).collect();
                all.sort_unstable();
                all
            }
        };
        for &v in &vars {
            self.hold(v, 1);
        }
        self.support[id] = vars;
    }

    /// Before the join of slots `a` and `b`: their variables merged, and
    /// the holdings of those they share dropped to the result's one.
    /// [`SumOut::undo`] puts them back where the join does not happen.
    pub(super) fn begin(&mut self, a: usize, b: usize) {
        let need = a.max(b) + 1;
        if self.support.len() < need {
            self.support.resize_with(need, Vec::new);
        }
        let (x, y) = (&self.support[a], &self.support[b]);
        let (mut merged, mut shared) = (Vec::with_capacity(x.len() + y.len()), Vec::new());
        let (mut i, mut j) = (0, 0);
        while i < x.len() || j < y.len() {
            if j == y.len() || (i < x.len() && x[i] < y[j]) {
                merged.push(x[i]);
                i += 1;
            } else if i == x.len() || y[j] < x[i] {
                merged.push(y[j]);
                j += 1;
            } else {
                merged.push(x[i]);
                shared.push(x[i]);
                i += 1;
                j += 1;
            }
        }
        for &v in &shared {
            self.hold(v, -1);
        }
        self.merged = merged;
        self.shared = shared;
    }

    pub(super) fn undo(&mut self) {
        for v in std::mem::take(&mut self.shared) {
            self.hold(v, 1);
        }
        self.merged.clear();
    }

    /// The join begun is done, into slot `out`.
    pub(super) fn commit(&mut self, a: usize, b: usize, out: usize) {
        if self.support.len() <= out {
            self.support.resize_with(out + 1, Vec::new);
        }
        self.support[a] = Vec::new();
        self.support[b] = Vec::new();
        self.support[out] = std::mem::take(&mut self.merged);
        self.shared.clear();
    }

    /// The finished internal nodes of `scope`'s vtree for the join begun,
    /// whose operands hold `pairs` pairs: none under `from` pairs. A node
    /// is finished where no slot but the result holds a variable under the
    /// global node over its leaves. Every finished node is a target, not
    /// only the maximal ones: each level under a target then folds its
    /// cells into values as it is built, instead of holding them as pairs
    /// until the target above it is folded.
    pub(super) fn targets(&self, scope: &VtreeScope, pairs: usize) -> Vec<VtreeIdx> {
        if pairs < self.from.max(1) {
            return Vec::new();
        }
        let tree = scope.vtree();
        let global = |local: VarId| match scope.vars() {
            Some(vars) => vars[local.0 as usize - 1],
            None => local,
        };
        let n = tree.num_nodes();
        let mut ends = vec![(0u32, 0u32); n];
        let mut own = vec![0i64; n];
        let mut finished = vec![false; n];
        for t in tree.bottomup() {
            if tree.node(t).is_leaf() {
                let v = global(tree.leaf_var(t));
                let p = self.position[v.0 as usize];
                ends[t.idx()] = (p, p);
                own[t.idx()] = i64::from(self.merged.binary_search(&v).is_ok());
                continue;
            }
            let (l, r) = tree.children(t);
            let (lo, hi) = (
                ends[l.idx()].0.min(ends[r.idx()].0),
                ends[l.idx()].1.max(ends[r.idx()].1),
            );
            ends[t.idx()] = (lo, hi);
            own[t.idx()] = own[l.idx()] + own[r.idx()];
            let (glo, ghi) = self.range[self.over(lo, hi).idx()];
            finished[t.idx()] = self.held_before(ghi) - self.held_before(glo) == own[t.idx()];
        }
        tree.bottomup()
            .filter(|&t| finished[t.idx()] && !tree.node(t).is_leaf())
            .collect()
    }

    /// The facts' weights on `scope`, by its local variables.
    fn store_for(&self, scope: &VtreeScope) -> WeightStore {
        let table = match scope.vars() {
            Some(vars) => vars
                .iter()
                .map(|v| self.weights[v.0 as usize - 1].clone())
                .collect::<Vec<_>>(),
            None => self.weights.clone(),
        };
        WeightStore::new(
            RationalWeights::from_literals(&table),
            Arithmetic::ScaledFloat,
        )
    }

    /// Both operands on `scope`: one that carries weights copied there with
    /// the facts' weights there (`Engine::embed_over`, which places
    /// marginal levels), the other moved. Refused, each comes back on the
    /// scope it was left on.
    #[allow(clippy::too_many_arguments)]
    fn place_both(
        &self,
        backend: &TddBackend,
        fa: Formula,
        first: Option<&[VarId]>,
        fb: Formula,
        second: Option<&[VarId]>,
        scope: &VtreeScope,
        store: &WeightStore,
    ) -> Result<(Tdd, Tdd), Refused> {
        // The weighted operand first: a copy refused moves nothing.
        let swapped = fa.0.weights().is_none() && fb.0.weights().is_some();
        let (fa, fb, first, second) = if swapped {
            (fb, fa, second, first)
        } else {
            (fa, fb, first, second)
        };
        let place =
            |f: Formula, map: Option<&[VarId]>| -> Result<Formula, Box<(CompileError, Formula)>> {
                let (t, own) = f;
                if own.same(scope) {
                    return Ok((t, own));
                }
                let map: Cow<[VarId]> = match map {
                    Some(m) => Cow::Borrowed(m),
                    None => match TddBackend::scope_map(&own, scope) {
                        Ok(m) => Cow::Owned(m),
                        Err(e) => return Err(Box::new((e, (t, own)))),
                    },
                };
                if t.weights().is_some() {
                    match backend.engine.embed_over(
                        &t,
                        scope.vtree(),
                        |v| map[v.0 as usize - 1],
                        Some(store.empty_like()),
                    ) {
                        Ok((g, _)) => Ok((g, scope.clone())),
                        Err(e) => Err(Box::new((e.into(), (t, own)))),
                    }
                } else {
                    match backend
                        .engine
                        .embed_moving(t, scope.vtree(), |v| map[v.0 as usize - 1])
                    {
                        Ok((g, _)) => Ok((g, scope.clone())),
                        Err(refused) => Err(Box::new((refused.error.into(), (refused.tdd, own)))),
                    }
                }
            };
        let unswap = |x: Formula, y: Formula| if swapped { (y, x) } else { (x, y) };
        let fa = match place(fa, first) {
            Ok(f) => f,
            Err(refused) => {
                let (e, fa) = *refused;
                let (fa, fb) = unswap(fa, fb);
                return Err(Box::new((e, fa.into(), fb.into())));
            }
        };
        match place(fb, second) {
            Ok(fb) => {
                let (fa, fb) = unswap(fa, fb);
                Ok((fa.0, fb.0))
            }
            Err(refused) => {
                let (e, fb) = *refused;
                let (fa, fb) = unswap(fa, fb);
                Err(Box::new((e, fa.into(), fb.into())))
            }
        }
    }

    /// The walk's `and_onto` where an operand carries weights; refused
    /// after the placements, both come back on `scope`.
    pub(super) fn and_onto(
        &mut self,
        backend: &TddBackend,
        fa: Formula,
        first: Option<&[VarId]>,
        fb: Formula,
        second: Option<&[VarId]>,
        scope: &VtreeScope,
    ) -> Result<Tdd, Refused> {
        self.weighted += 1;
        let store = self.store_for(scope);
        let (a, b) = self.place_both(backend, fa, first, fb, second, scope, &store)?;
        backend.engine.and_restoring(a, b).map_err(|refused| {
            let (f, g) = (refused.f.into_owned(), refused.g.into_owned());
            Box::new((
                refused.error.into(),
                (f, scope.clone()).into(),
                (g, scope.clone()).into(),
            ))
        })
    }

    /// The conjunction on `scope` with `targets` summed out as it is built
    /// (`Engine::and_marginalizing`), under the facts' weights; refused,
    /// both operands come back on `scope` as they were placed.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn and_marginalizing(
        &mut self,
        backend: &TddBackend,
        fa: Formula,
        first: Option<&[VarId]>,
        fb: Formula,
        second: Option<&[VarId]>,
        scope: &VtreeScope,
        targets: &[VtreeIdx],
    ) -> Result<Tdd, Refused> {
        let store = self.store_for(scope);
        let (mut a, b) = self.place_both(backend, fa, first, fb, second, scope, &store)?;
        self.summing += 1;
        if a.weights().is_none()
            && b.weights().is_none()
            && let Err(e) = a.set_weights(store.empty_like())
        {
            return Err(Box::new((
                CompileError::Tdd(OperationError::from(e)),
                (a, scope.clone()).into(),
                (b, scope.clone()).into(),
            )));
        }
        let targets: Vec<VtreeIdx> = targets
            .iter()
            .copied()
            .filter(|&t| !a.level(t).is_marginal() && !b.level(t).is_marginal())
            .collect();
        // Borrowed operands are read in place and left as they were, so a
        // refusal hands them back without a copy.
        backend
            .engine
            .and_marginalizing(&a, &b, &targets)
            .map_err(|e| {
                Box::new((
                    e.into(),
                    (a, scope.clone()).into(),
                    (b, scope.clone()).into(),
                ))
            })
    }
}
