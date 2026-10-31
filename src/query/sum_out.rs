//! Summing out the evidence walk's finished subtrees.
//!
//! The walk's result is counted, and the queries read only some of its
//! variables after it: a query's own variable, a fact's or an indicator
//! copy's, or those its formula depends on. Every other variable can be
//! summed out of a join's result once no later join can read it; the
//! queries' variables are held as a slot holds them, and never are
//! ([`SumOut::hold_for_queries`]). A subtree `t` of the result's
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
//! and conjoins weighted. The walk's last result is counted by its
//! weighted value where no query reads it, and otherwise by the count up
//! and down, which reads a summed level's values as its column
//! (`crate::eval::Counted`).
//! The UAI 2022 PR protein networks 1bxv and 1jer held 2.1 and 4.3 GB at
//! their peak instead of 10.1 and 15.1 GB in signed logs and maximal
//! subtrees.

use std::borrow::Cow;
use std::sync::Arc;

use num_rational::BigRational;
use tididi::diagram::{Arithmetic, LiteralWeights, RationalWeights, TddLevel, WeightStore};
use tididi::vtree::{VarId, VtreeIdx};
use tididi::{OperationError, Tdd};

use super::Formula;
use crate::eval::Probabilities;
use crate::tp::backend::Part;
use crate::tp::{CompileError, TddBackend, VtreeScope};

/// Where queries read variables after the walk, a finished subtree is
/// summed out only when the join's operands hold at least this many pairs
/// under it: smaller subtrees do not repay turning the walk's accumulator
/// weighted.
const MIN_QUERY_SUM_PAIRS: usize = 1_000_000;

/// A join refused: the error, and both operands.
type Refused = Box<(CompileError, Part, Part)>;

fn level_pairs(level: &TddLevel) -> usize {
    if level.is_marginal() {
        0
    } else if let Some(d) = level.implicit() {
        d.pairs()
    } else {
        (0..level.nodes().len())
            .map(|i| level.pair_count_at(i))
            .sum()
    }
}

fn pairs_at(f: &Tdd, nodes: &[VtreeIdx]) -> usize {
    nodes.iter().map(|&t| level_pairs(f.level(t))).sum()
}

/// The global vtree's leaves from left to right, built on the first join
/// that looks for finished subtrees: each program variable's position,
/// each node's positions `[lo, hi)`, each node's depth, and for position
/// `k`, the node whose children split `k` from `k + 1`; `shallowest[j][k]`
/// is the shallowest of those over `k .. k + 2^j`. The global node over
/// positions `lo < hi` is the shallowest split between them.
struct TreeIndex {
    position: Vec<u32>,
    range: Vec<(u32, u32)>,
    depth: Vec<u32>,
    split: Vec<VtreeIdx>,
    leaf_at: Vec<VtreeIdx>,
    shallowest: Vec<Vec<u32>>,
    /// A Fenwick tree over positions: the slots holding each variable.
    held: Vec<i64>,
    var_mask: Vec<u8>,
}

impl TreeIndex {
    fn new(tree: &tididi::Vtree, holders: &[u32]) -> Self {
        let n = tree.num_nodes();
        let mut range = vec![(0u32, 0u32); n];
        let mut depth = vec![0u32; n];
        let mut leaf_at = Vec::with_capacity(tree.num_vars() as usize);
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
        let mut held = vec![0i64; leaves + 1];
        for (v, &h) in holders.iter().enumerate().skip(1) {
            if h != 0 {
                held[position[v] as usize + 1] = i64::from(h);
            }
        }
        for i in 1..held.len() {
            let j = i + (i & i.wrapping_neg());
            if j < held.len() {
                held[j] += held[i];
            }
        }
        TreeIndex {
            position,
            range,
            depth,
            split,
            leaf_at,
            shallowest,
            held,
            var_mask: vec![0; tree.num_vars() as usize + 1],
        }
    }

    fn add(&mut self, v: VarId, delta: i64) {
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
}

/// The finished subtrees of the walk's joins, and the facts' weights to
/// sum them out under (see the module documentation).
pub(super) struct SumOut {
    /// Per slot, the program variables of the observations conjoined into
    /// it, ascending.
    support: Vec<Vec<VarId>>,
    tree: Arc<tididi::Vtree>,
    index: Option<TreeIndex>,
    /// By program variable, the slots holding it.
    holders: Vec<u32>,
    /// The facts' weights, by program variable.
    pos: Vec<f64>,
    neg: Vec<f64>,
    rat: Vec<Option<LiteralWeights<BigRational>>>,
    /// Whether any query holds variables after the walk, and the least
    /// pairs a finished subtree holds for a join to sum it out.
    has_queries: bool,
    min_pairs: usize,
    /// `targets`' buffers, per node of the scope: the positions under it,
    /// and the result's variables under it where it is finished.
    ends: Vec<(u32, u32)>,
    own: Vec<Option<(u32, u8)>>,
    top_idx: Vec<u32>,
    f_seen: Vec<u32>,
    f_gen: u32,
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
    /// On the global vtree `tree`, with the facts' weights `w`.
    pub(super) fn new(tree: Arc<tididi::Vtree>, w: &Probabilities) -> Self {
        let holders = vec![0; tree.num_vars() as usize + 1];
        SumOut {
            support: Vec::new(),
            tree,
            index: None,
            holders,
            pos: w.pos.clone(),
            neg: w.neg.clone(),
            rat: vec![None; w.pos.len()],
            has_queries: false,
            min_pairs: MIN_QUERY_SUM_PAIRS,
            ends: Vec::new(),
            own: Vec::new(),
            top_idx: Vec::new(),
            f_seen: Vec::new(),
            f_gen: 0,
            merged: Vec::new(),
            shared: Vec::new(),
            summing: 0,
            weighted: 0,
        }
    }

    fn hold(&mut self, v: VarId, delta: i64) {
        let h = &mut self.holders[v.0 as usize];
        *h = h
            .checked_add_signed(delta as i32)
            .expect("a variable's holdings are never negative");
        if let Some(idx) = &mut self.index {
            idx.add(v, delta);
        }
    }

    /// The variables `vars` are read after the walk (by the queries): no
    /// subtree over one of them is ever finished.
    pub(super) fn hold_for_queries(&mut self, vars: &[VarId]) {
        self.has_queries |= !vars.is_empty();
        for &v in vars {
            self.hold(v, 1);
        }
    }

    #[cfg(test)]
    pub(super) fn set_min_pairs(&mut self, min: usize) {
        self.min_pairs = min;
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
        let (mut merged, mut shared) = (
            std::mem::take(&mut self.merged),
            std::mem::take(&mut self.shared),
        );
        merged.clear();
        shared.clear();
        merged.reserve(x.len() + y.len());
        let (mut i, mut j) = (0, 0);
        while i < x.len() && j < y.len() {
            if x[i] < y[j] {
                if x.len() - i > 64 {
                    let k = i + x[i..].partition_point(|&v| v < y[j]);
                    merged.extend_from_slice(&x[i..k]);
                    i = k;
                } else {
                    merged.push(x[i]);
                    i += 1;
                }
            } else if y[j] < x[i] {
                if y.len() - j > 64 {
                    let k = j + y[j..].partition_point(|&v| v < x[i]);
                    merged.extend_from_slice(&y[j..k]);
                    j = k;
                } else {
                    merged.push(y[j]);
                    j += 1;
                }
            } else {
                merged.push(x[i]);
                shared.push(x[i]);
                i += 1;
                j += 1;
            }
        }
        if i < x.len() {
            merged.extend_from_slice(&x[i..]);
        }
        if j < y.len() {
            merged.extend_from_slice(&y[j..]);
        }
        for &v in &shared {
            self.hold(v, -1);
        }
        self.merged = merged;
        self.shared = shared;
    }

    pub(super) fn undo(&mut self) {
        let mut shared = std::mem::take(&mut self.shared);
        for &v in &shared {
            self.hold(v, 1);
        }
        shared.clear();
        self.shared = shared;
        self.merged.clear();
    }

    /// The join begun is done, into slot `out`.
    pub(super) fn commit(&mut self, a: usize, b: usize, out: usize) {
        if self.support.len() <= out {
            self.support.resize_with(out + 1, Vec::new);
        }
        let reuse = if self.support[a].capacity() >= self.support[b].capacity() {
            self.support[b].clear();
            std::mem::take(&mut self.support[a])
        } else {
            self.support[a].clear();
            std::mem::take(&mut self.support[b])
        };
        self.support[out] = std::mem::replace(&mut self.merged, reuse);
        self.merged.clear();
        self.shared.clear();
    }

    /// The finished internal nodes of `scope`'s vtree for the join begun,
    /// whose operands hold `pairs` pairs: none where they hold none. A node
    /// is finished where no slot but the result holds a variable under the
    /// global node over its leaves. Every finished node is a target, not
    /// only the maximal ones: each level under a target then folds its
    /// cells into values as it is built, instead of holding them as pairs
    /// until the target above it is folded.
    ///
    /// Only a node each of whose variables only the result holds, or none
    /// does, can be finished, and the global node over it is looked up only
    /// for those: a join whose scope is its operands' variables, fewer than
    /// two of them held only by the result, has no target at all.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn targets(
        &mut self,
        a: usize,
        b: usize,
        scope: &VtreeScope,
        fa: &Formula,
        fb: &Formula,
        pairs: usize,
        last: bool,
    ) -> Vec<VtreeIdx> {
        let (small, large, small_is_a) = if self.support[a].len() <= self.support[b].len() {
            (&self.support[a], &self.support[b], true)
        } else {
            (&self.support[b], &self.support[a], false)
        };
        let (small_weighted, large_weighted) = if small_is_a {
            (fa.0.weights().is_some(), fb.0.weights().is_some())
        } else {
            (fb.0.weights().is_some(), fa.0.weights().is_some())
        };
        let already_weighted = small_weighted || large_weighted;
        let min = if self.has_queries && !already_weighted {
            self.min_pairs
        } else {
            0
        };
        let tree = scope.vtree();
        let exact = scope
            .vars()
            .map_or(tree.num_vars() as usize, <[VarId]>::len)
            == self.merged.len();
        if pairs == 0
            || pairs < min
            || last && self.has_queries && min > 0
            || !self.merged.iter().any(|v| self.holders[v.0 as usize] == 1)
            || min > 0
                && (!small.iter().any(|v| self.holders[v.0 as usize] == 1)
                    || !large.iter().any(|v| self.holders[v.0 as usize] == 1))
            || (exact || min > 0)
                && self
                    .merged
                    .iter()
                    .filter(|v| self.holders[v.0 as usize] == 1)
                    .nth(1)
                    .is_none()
        {
            return Vec::new();
        }
        let idx = self
            .index
            .get_or_insert_with(|| TreeIndex::new(&self.tree, &self.holders));
        if !exact {
            for &v in large {
                idx.var_mask[v.0 as usize] = 2;
            }
        }
        for &v in small {
            idx.var_mask[v.0 as usize] = 1;
        }
        for &v in &self.shared {
            idx.var_mask[v.0 as usize] = 3;
        }
        let global = |local: VarId| match scope.vars() {
            Some(vars) => vars[local.0 as usize - 1],
            None => local,
        };
        let n = tree.num_nodes();
        let (mut ends, mut own) = (
            std::mem::take(&mut self.ends),
            std::mem::take(&mut self.own),
        );
        ends.clear();
        ends.resize(n, (0, 0));
        own.clear();
        own.resize(n, None);
        let can_finish = |mask: u8| -> bool {
            if min > 0 {
                mask == 3
            } else {
                !matches!(
                    (mask, small_weighted, large_weighted),
                    (1, true, _) | (2, _, true)
                )
            }
        };
        let mut found = Vec::new();
        for t in tree.bottomup() {
            if tree.node(t).is_leaf() {
                let v = global(tree.leaf_var(t)).0 as usize;
                let p = idx.position[v];
                ends[t.idx()] = (p, p);
                let m = idx.var_mask[v];
                let mine = u32::from(exact || m != 0);
                if self.holders[v] == mine {
                    own[t.idx()] = Some((
                        mine,
                        if m == 0 && !exact {
                            0
                        } else if m != 0 {
                            m
                        } else {
                            2
                        },
                    ));
                }
                continue;
            }
            let (l, r) = tree.children(t);
            let (lo, hi) = (
                ends[l.idx()].0.min(ends[r.idx()].0),
                ends[l.idx()].1.max(ends[r.idx()].1),
            );
            ends[t.idx()] = (lo, hi);
            let Some(((ml, kl), (mr, kr))) = own[l.idx()].zip(own[r.idx()]) else {
                continue;
            };
            let (mine, mask) = (ml + mr, kl | kr);
            own[t.idx()] = Some((mine, mask));
            if mine > 0 && can_finish(mask) {
                let (glo, ghi) = idx.range[idx.over(lo, hi).idx()];
                if idx.held_before(ghi) - idx.held_before(glo) == i64::from(mine) {
                    found.push(t);
                }
            }
        }
        if min > 0 && !found.is_empty() {
            let mut top_idx = std::mem::take(&mut self.top_idx);
            top_idx.clear();
            top_idx.resize(n, u32::MAX);
            let (a_bit, b_bit) = if small_is_a { (1u8, 2u8) } else { (2u8, 1u8) };
            let mut triggered = false;
            let mut group = Vec::new();
            let mut top_leaves = Vec::new();
            let mut stack = Vec::new();
            for &top in found.iter().rev() {
                if top_idx[top.idx()] != u32::MAX {
                    continue;
                }
                group.clear();
                top_leaves.clear();
                stack.push((top, false));
                while let Some((u, done)) = stack.pop() {
                    if tree.node(u).is_leaf() {
                        let v = global(tree.leaf_var(u));
                        let m = idx.var_mask[v.0 as usize];
                        let m = if m == 0 && !exact {
                            0
                        } else if m != 0 {
                            m
                        } else {
                            2
                        };
                        if m != 0 {
                            top_leaves.push((v, m));
                        }
                    } else if done {
                        if u == top {
                            group.push(u);
                        } else if let Some((mine, _)) = own[u.idx()]
                            && mine > 0
                        {
                            let (lo, hi) = ends[u.idx()];
                            let (glo, ghi) = idx.range[idx.over(lo, hi).idx()];
                            if idx.held_before(ghi) - idx.held_before(glo) == i64::from(mine) {
                                group.push(u);
                            }
                        }
                    } else {
                        top_idx[u.idx()] = 0;
                        let (l, r) = tree.children(u);
                        stack.push((u, true));
                        stack.push((r, false));
                        stack.push((l, false));
                    }
                }
                let mut sum = 0usize;
                for (f, bit) in [(fa, a_bit), (fb, b_bit)] {
                    if sum >= min {
                        break;
                    }
                    if f.1.same(scope) {
                        for &u in &group {
                            sum += level_pairs(f.0.level(u));
                            if sum >= min {
                                break;
                            }
                        }
                    } else {
                        let ftree = f.1.vtree();
                        let fvars = f.1.vars();
                        self.f_gen = self.f_gen.wrapping_add(1).max(1);
                        let stamp = self.f_gen;
                        if self.f_seen.len() < ftree.num_nodes() {
                            self.f_seen.resize(ftree.num_nodes(), 0);
                        }
                        let mut reached = 0usize;
                        for &(v, m) in &top_leaves {
                            if m & bit == 0 {
                                continue;
                            }
                            let lv = match fvars {
                                Some(vs) => match vs.binary_search(&v) {
                                    Ok(i) => VarId(i as u32 + 1),
                                    Err(_) => continue,
                                },
                                None => v,
                            };
                            let Some(mut cur) = ftree.leaf_of(lv) else {
                                continue;
                            };
                            while let Some(p) = ftree.node(cur).parent() {
                                if self.f_seen[p.idx()] != stamp {
                                    self.f_seen[p.idx()] = stamp;
                                    break;
                                }
                                reached += 1;
                                if sum < min {
                                    sum += level_pairs(f.0.level(p));
                                }
                                cur = p;
                            }
                        }
                        sum += 2 * group.len().saturating_sub(reached);
                    }
                }
                if sum >= min {
                    triggered = true;
                    break;
                }
            }
            found.clear();
            if triggered {
                for t in tree.bottomup() {
                    if !tree.node(t).is_leaf()
                        && let Some((mine, _)) = own[t.idx()]
                        && mine > 0
                    {
                        let (lo, hi) = ends[t.idx()];
                        let (glo, ghi) = idx.range[idx.over(lo, hi).idx()];
                        if idx.held_before(ghi) - idx.held_before(glo) == i64::from(mine) {
                            found.push(t);
                        }
                    }
                }
            }
            self.top_idx = top_idx;
        }
        if !exact {
            for &v in &self.merged {
                idx.var_mask[v.0 as usize] = 0;
            }
        } else {
            for &v in small {
                idx.var_mask[v.0 as usize] = 0;
            }
        }
        (self.ends, self.own) = (ends, own);
        found
    }

    /// The facts' weights on `scope`, by its local variables.
    fn store_for(&mut self, scope: &VtreeScope) -> WeightStore {
        let n = scope.vars().map_or(self.pos.len() - 1, <[VarId]>::len);
        let mut w_neg = Vec::with_capacity(n);
        let mut w_pos = Vec::with_capacity(n);
        let mut push_var = |v: usize| {
            let lw = self.rat[v].get_or_insert_with(|| LiteralWeights {
                negative: BigRational::from_float(self.neg[v]).expect("a weight is finite"),
                positive: BigRational::from_float(self.pos[v]).expect("a weight is finite"),
            });
            w_neg.push(lw.negative.clone());
            w_pos.push(lw.positive.clone());
        };
        match scope.vars() {
            Some(vars) => {
                for v in vars {
                    push_var(v.0 as usize);
                }
            }
            None => {
                for v in 1..self.pos.len() {
                    push_var(v);
                }
            }
        }
        WeightStore::new(
            RationalWeights::from_polarities(LiteralWeights {
                negative: w_neg,
                positive: w_pos,
            })
            .expect("equal polarity column lengths"),
            Arithmetic::ScaledFloat,
        )
    }

    /// Both operands on `scope`: one that carries weights copied there with
    /// the facts' weights there (`Engine::embed_over`, which places
    /// marginal levels), the other moved. Refused, each comes back on the
    /// scope it was left on.
    #[allow(clippy::too_many_arguments)]
    fn place_both(
        &mut self,
        backend: &TddBackend,
        fa: Formula,
        first: Option<&[VarId]>,
        fb: Formula,
        second: Option<&[VarId]>,
        scope: &VtreeScope,
    ) -> Result<(Tdd, Tdd), Refused> {
        let store = if fa.0.weights().is_none() && fb.0.weights().is_none() {
            None
        } else if fa.1.same(scope)
            && let Some(w) = fa.0.weights()
        {
            Some(w.empty_like())
        } else if fb.1.same(scope)
            && let Some(w) = fb.0.weights()
        {
            Some(w.empty_like())
        } else {
            Some(self.store_for(scope))
        };
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
                        Some(store.as_ref().unwrap().empty_like()),
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
        let (a, b) = self.place_both(backend, fa, first, fb, second, scope)?;
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
        let (mut a, b) = self.place_both(backend, fa, first, fb, second, scope)?;
        let already_weighted = a.weights().is_some() || b.weights().is_some();
        let min = if self.has_queries && !already_weighted {
            self.min_pairs
        } else {
            0
        };
        let mut targets: Vec<VtreeIdx> = targets
            .iter()
            .copied()
            .filter(|&t| !a.level(t).is_marginal() && !b.level(t).is_marginal())
            .collect();
        if min > 0 && pairs_at(&a, &targets) + pairs_at(&b, &targets) < min {
            targets.clear();
        }
        if targets.is_empty() {
            if already_weighted {
                self.weighted += 1;
            }
            return backend.engine.and_restoring(a, b).map_err(|refused| {
                let (f, g) = (refused.f.into_owned(), refused.g.into_owned());
                Box::new((
                    refused.error.into(),
                    (f, scope.clone()).into(),
                    (g, scope.clone()).into(),
                ))
            });
        }
        self.summing += 1;
        let added_weights = !already_weighted;
        if added_weights {
            let store = self.store_for(scope);
            if let Err(e) = a.set_weights(store) {
                return Err(Box::new((
                    CompileError::Tdd(OperationError::from(e)),
                    (a, scope.clone()).into(),
                    (b, scope.clone()).into(),
                )));
            }
        }
        let armed = backend.engine.limits().armed();
        let can_resume =
            armed.stop_rules().unconditional.is_some() || armed.stop_rules().after_pairs.is_some();
        if !can_resume {
            return backend
                .engine
                .and_marginalizing(a, b, &targets)
                .map_err(|e| {
                    Box::new((
                        e.into(),
                        (backend.zero(scope), scope.clone()).into(),
                        (backend.zero(scope), scope.clone()).into(),
                    ))
                });
        }
        // Borrowed operands are read in place and left as they were, so a
        // refusal hands them back without a copy.
        backend
            .engine
            .and_marginalizing(&a, &b, &targets)
            .map_err(|e| {
                if added_weights {
                    let _ = a.take_weights();
                }
                Box::new((
                    e.into(),
                    (a, scope.clone()).into(),
                    (b, scope.clone()).into(),
                ))
            })
    }
}
