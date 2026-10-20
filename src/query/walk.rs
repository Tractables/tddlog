//! The evidence walk.
//!
//! Evidence formula: the conjunction of the observed atoms' formulas (or
//! negations). Each factor is put at the lowest node of the vtree holding
//! its variables, and the vtree is walked bottom up: a node conjoins what
//! its children built with the factors put there. A conjunction of the
//! factors in any other order has, at a node, factors that read variables
//! decided outside it without the factors deciding them, and a diagram of
//! it tells apart assignments the whole conjunction would rule out.
//!
//! The factors at one node, the children's results among them, are
//! conjoined pairwise as a balanced tree. A chain of thousands of
//! conjunctions (a naive Bayes classifier observes one atom per feature,
//! and every one of them is at the root when its cone is the program)
//! touches the growing conjunction once per atom, so it costs the square
//! of the evidence size; the tree touches each level's total once. `and`
//! does not minimize its output, so every result is reduced (the schedule
//! is in `EvidenceWalk::run`).
//!
//! The walk is itself a chain where the vtree is one: on a right-linear
//! vtree each step conjoins a factor of a few pairs into a large result,
//! which is the square of the evidence again. So a node whose items
//! other than the largest (its own factors, the other child's result,
//! what the children put off) total under 1/PUT_OFF_BELOW of the
//! largest's charge passes the largest up with the others put off. The
//! first node where that fails conjoins each child's put-off factors
//! among themselves and into the child's result, then takes the walk's
//! step; the root always does. A traversal of a large result then waits
//! until the factors put off reach a thirty-second of it, and each of
//! their pairs pays for at most thirty-two of its pairs. Putting off more
//! conjoins factors of different nodes without what lies between them,
//! and their products grow.
//!
//! A join also walks every level of the scope it is built on, the union
//! of its operands', whatever their pairs: a factor of a few pairs joined
//! into a result over hundreds of variables pays for all of them. The
//! test charges the largest item [`LEVEL_PAIRS`] pairs per node of its
//! scope's vtree on top of its own pairs. A spine of nodes, each with a
//! small factor beside the result coming up, then puts the factors off
//! and joins them among themselves before they meet the result, where
//! each met it at its own node.
//!
//! The factors a child put off are conjoined among themselves, a batch,
//! and their product meets the child's result, the batch's target. A node
//! that takes its step puts its own factors off into the batch of the
//! child it sets aside, as one more group, as a node that passes its
//! largest up puts its other items off: its own factors read variables
//! below it as much as put-off factors do. Factors that read variables the
//! target ties together then meet without the formula that ties them, and
//! their product can grow with each of them (several readers of one
//! variable, with the rows of their tables apart; data points observed
//! bit by bit, which read a shared number's lower bits that the target
//! decides). Each join of a batch is tried capped at the larger of
//! [`DEFER_GROWTH`] times its operands' pairs, the floor any capped join
//! has, and what the put-off test charges the target. Given up, the batch
//! would put each of the join's operands into the target instead, a join
//! that walks the target; a join of the batch that makes no more than
//! that costs no more than the join it saves. A batch whose joins stay
//! within their caps is finished however large its product: where the
//! target does not decide the factors, each of them put into the target
//! grows it, the chain the balanced tree avoids. Past the cap the batch
//! is given up: what its joins still to come would have read goes
//! into the target one at a time, in the order the factors were put off,
//! which keeps the factors of one node together, and what they make is
//! what their product with the target would have been. The cap reads the
//! pairs a conjunction has in flight, which pass it before the result
//! would.
//!
//! Where the batch given up is the largest child's, which the node sets
//! aside to meet its other items smallest first, those items are conjoined
//! first, and their product goes into the target before the batch's
//! operands: the chain is the node's last joins, and at the root the walk's,
//! whose result the watch does not hold.
//!
//! A result that meets a target is minimized in full off the schedule
//! (see `EvidenceWalk::run`) where that costs less than the join it
//! precedes: the last result of a given-up batch's chain, which stands for
//! the batch's product with the target, and a batch's product that holds
//! fewer pairs than its target. Each join's result is only pruned where
//! the schedule's full plans do not fall, and the twins left multiply in
//! the join with the target and in every join after it
//! (normal_mixture_theta under auto's `closed`: a batch's product pruned
//! to 475 745 pairs where the same factors' product came to 37 188, and
//! the target's join 17.8 M pairs against 5.5 M; clickGraph_w14: a
//! chain's 2.94 M pairs pruned against 1.04 M minimized, and the root's
//! last join 47.9 M pairs instead of 12.4 M). A full minimization costs as
//! much as its result is large, and a product larger than its target is
//! minimized on the schedule, as any join's result is (GeNI's weather_h400
//! under `program`: a root product of 202 M pairs, its target 23 M,
//! minimized in full to 200 pairs fewer for 143 M work units, 7% of the
//! walk).
//!
//! The batch conjoins its factors in groups. A node that passes its largest
//! item up puts off its children's put-off factors in their groups and its
//! other items as one more, and a group merges into the one before it while
//! it holds at least as many pairs, as a binary counter carries: the groups
//! hold fewer pairs from first to last. Each group is conjoined as a
//! balanced tree over its list, the rounds of all of them planned together,
//! round by round, and the groups' products meet two smallest first. A
//! balanced tree over the whole list pairs factors put off at nodes far
//! apart in the walk, of pairs far apart. The groups and the chain at the
//! end of a node's step make the walk's work vary less with the put-off
//! test's constants.
//!
//! The vtree's top can be a chain, a node whose one child is a leaf over
//! a node whose one child is a leaf, and so on (`decision`'s layout puts
//! the decisions all the parts read there, Alea.jl's means). Factors put
//! off on the chain read its leaves beside the parts below it, and the
//! nodes they were put off at are the chain's, one per leaf they reach up
//! to: a group per node holds the factors of one leaf, of every part. A
//! batch whose factors read the chain's leaves is therefore grouped by
//! part, where they read two parts or more: a factor's part is the lowest
//! node holding what it reads off the chain, and parts one above the other
//! are one, the higher; a group per part in the order the parts first
//! come, the factors that read only the chain one group, each in the order
//! the factors were put off. Factors of two parts share no variable off
//! the chain, they agree only through its leaves, and their product holds
//! every pair of what each keeps apart there; the factors of one part
//! meet first. altermu2 observes, bit by bit, each data point's draw plus
//! the means' sum, and the bits of different data points met first: at 15
//! bits the root's batch, given up, went into the target as products of
//! bits of several data points, and its third join made 2.2 G pairs (11.4
//! M minimized) where grouped by data point the walk's largest formula
//! holds 10.5 M. Where the factors off the chain lie in one part, grouping
//! only sets the factors on the chain apart from the others, and there is
//! no product of parts to avoid: a right-linear vtree is a chain but for
//! its last leaf, the one part, and a chain of one leaf over a tree that
//! factors read across (Roulette's networks, genes5 under `decision`) has
//! its parts one inside the other. Grouped there all the same, the
//! batches' products changed for nothing (GeNI's router_16 1.92x and
//! weather_n16 1.94x as long, and multimodal_w11's `closed` refused
//! memory at its root; `EvidenceWalk::by_part`).
//!
//! Each factor starts on its own scope and a pair is brought together on
//! the union of the two, which is the smallest vtree that can hold their
//! conjunction. Putting every factor on the union of all of them first
//! would pay the whole fact set at every leaf of the tree, which is the
//! cost the compiler just avoided.
//!
//! A node's last join below the root, of two operands of [`DEFER_FROM`]
//! pairs or more each, is tried with its result capped at
//! [`DEFER_GROWTH`] times their pairs. Past the cap the join waits: its
//! operands stay in their slots, and the walk goes on with the cap for
//! its result's size, which the result would exceed. Where the walk's
//! last join reads a waiting result with an operand smaller than both of
//! its own, that operand goes into the larger of the two first and their
//! product meets the smaller (pedigree6_10's root joins a few thousand
//! pairs with two operands whose product is tens of millions).
//! Anywhere else a waiting join is done before its result is read, and
//! before a node plans with its size other than to pass it up or set it
//! aside at the root, so that the walk decides as it would have on the
//! result. The cap keeps the attempt cheap where a join grows little, and
//! [`DEFER_FROM`] keeps small joins from waiting for nothing.

use std::collections::VecDeque;
use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use tididi::OperationError;
use tididi::limits::{StopAt, StopRules};
use tididi::vtree::{VarId, VtreeIdx};

use super::{Formula, PendingFormula, QuerySizes};
use crate::ground::AtomId;
use crate::tp::backend::FullSchedule;
use crate::tp::{CompileError, Compiled, NegationSource, Reduce, TddBackend, VtreeScope};

/// The evidence phase puts a node's smaller items off while they total under
/// 1/PUT_OFF_BELOW of its largest (see the module doc).
const PUT_OFF_BELOW: usize = 32;

/// The pairs the put-off test charges a node's largest item per node of its
/// scope's vtree, which every join with it walks (see the module
/// documentation).
const LEVEL_PAIRS: usize = 8;

/// Where the last full minimization freed nothing, one due on a result over
/// this many times the result the last one was due on is a prune instead
/// (see `EvidenceWalk::run`).
const FULL_GROWTH: usize = 16;

/// The pairs each operand of a node's last join reaches for the join to be
/// tried capped, and the cap, a multiple of their pairs (see the module
/// documentation).
const DEFER_FROM: usize = 40_000;
const DEFER_GROWTH: u64 = 4;

/// The most variables the factors a node conjoins among themselves may hold
/// for their joins to share the scope over all of them (see
/// [`EvidenceWalk::shared_scope`]).
const SHARED_UP_TO: usize = 16;

/// A conjunction the walk has planned: of two slots into a third, or of
/// several, whose two smallest are joined first when they all exist, then
/// the two smallest of what is left, into the last slot.
enum Join {
    Two(usize, usize, usize),
    Smallest(Vec<usize>, usize),
}

impl Join {
    /// The slot the join fills.
    fn out(&self) -> usize {
        match self {
            Join::Two(_, _, out) | Join::Smallest(_, out) => *out,
        }
    }

    /// Whether `slot` is one of the join's operands.
    fn reads(&self, slot: usize) -> bool {
        match self {
            Join::Two(a, b, _) => *a == slot || *b == slot,
            Join::Smallest(items, _) => items.contains(&slot),
        }
    }
}

/// A child's put-off factors conjoined among themselves into `product`,
/// which a later join of the node conjoins with `target`, the child's
/// result (see the module documentation).
struct Batch {
    target: usize,
    product: usize,
}

/// What a node built: its result, and where it passed that up, the factors
/// it put off, in the order they were put off, their pairs, and their
/// groups.
struct Built {
    main: usize,
    put_off: VecDeque<usize>,
    put_off_size: usize,
    groups: Vec<Group>,
}

/// A run of `len` put-off factors of `pairs` pairs in all, which a batch
/// conjoins among themselves before the other runs' products (see the
/// module documentation).
#[derive(Clone, Copy)]
struct Group {
    len: usize,
    pairs: usize,
}

/// Appends `group` to `groups`, the last merging into the one before it
/// while it holds as many pairs, as a binary counter carries: the groups
/// hold fewer pairs from first to last, each fewer than the one before.
fn push_group(groups: &mut Vec<Group>, group: Group) {
    if group.len == 0 {
        return;
    }
    groups.push(group);
    while let [.., before, last] = groups[..]
        && last.pairs >= before.pairs
    {
        groups.pop();
        let before = groups.last_mut().unwrap();
        before.len += last.len;
        before.pairs += last.pairs;
    }
}

/// `front` followed by `back`, the shorter moved into the longer. A
/// put-off list passes up a chain of nodes and gains a few factors at
/// each; copied whole at each node, it cost the square of its length:
/// two thirds of the run on Roulette's diamond at a million steps.
fn concat(mut front: VecDeque<usize>, mut back: VecDeque<usize>) -> VecDeque<usize> {
    if front.len() >= back.len() {
        front.append(&mut back);
        front
    } else {
        while let Some(f) = front.pop_back() {
            back.push_front(f);
        }
        back
    }
}

/// Per variable of `tree`, whether it is a leaf of its top chain: from the
/// root down, while a node has a leaf child, that leaf, and on into the
/// other child; and whether the chain ends over a node that is not a leaf.
fn top_chain(tree: &tididi::Vtree) -> (Vec<bool>, bool) {
    let mut chain = vec![false; tree.num_vars() as usize + 1];
    let mut n = tree.root();
    while !tree.node(n).is_leaf() {
        let (l, r) = tree.children(n);
        let (leaf, rest) = match (tree.node(l).is_leaf(), tree.node(r).is_leaf()) {
            (true, _) => (l, r),
            (false, true) => (r, l),
            (false, false) => return (chain, true),
        };
        chain[tree.leaf_var(leaf).0 as usize] = true;
        n = rest;
    }
    (chain, false)
}

/// A join's variables and its operands' renamings onto them, kept between
/// joins: the walk makes one per join, tens of thousands of them on the
/// small formulas of a large network.
#[derive(Default)]
struct Union {
    vars: Vec<VarId>,
    /// The local variable of `vars` each local variable of the first
    /// operand, and of the second, goes to.
    first: Vec<VarId>,
    second: Vec<VarId>,
}

impl Union {
    /// The union of `x` and `y`, ascending lists of distinct variables, and
    /// where each of their variables lands in it.
    fn merge(&mut self, x: &[VarId], y: &[VarId]) {
        let Union {
            vars,
            first,
            second,
        } = self;
        vars.clear();
        first.clear();
        second.clear();
        let (mut i, mut j) = (0, 0);
        while i < x.len() && j < y.len() {
            let at = VarId(vars.len() as u32 + 1);
            match x[i].cmp(&y[j]) {
                std::cmp::Ordering::Less => {
                    vars.push(x[i]);
                    first.push(at);
                    i += 1;
                }
                std::cmp::Ordering::Greater => {
                    vars.push(y[j]);
                    second.push(at);
                    j += 1;
                }
                std::cmp::Ordering::Equal => {
                    vars.push(x[i]);
                    first.push(at);
                    second.push(at);
                    i += 1;
                    j += 1;
                }
            }
        }
        for (rest, map) in [(&x[i..], first), (&y[j..], second)] {
            for &v in rest {
                map.push(VarId(vars.len() as u32 + 1));
                vars.push(v);
            }
        }
    }
}

/// The bottom-up evidence walk, with diagram operands in slots.
/// Planning a node cannot stop; executing its conjunction queue can. A
/// successful conjunction replaces its two operands, so checkpoints retain
/// the frontier of the walk rather than its entire computation history.
pub(super) struct EvidenceWalk {
    observed: Vec<(AtomId, bool)>,
    next_observed: usize,
    at: Vec<Vec<usize>>,
    built: Vec<Option<Built>>,
    nodes: Vec<VtreeIdx>,
    next_node: usize,
    slots: Vec<Option<Formula>>,
    /// The pairs of each slot's formula as it was put there, or of a
    /// waiting join's cap. A count walks every node of the diagram, and a
    /// slot's is read when its join finishes and again when its vtree node
    /// plans the next joins.
    slot_pairs: Vec<usize>,
    joins: VecDeque<Join>,
    /// The slots a node below the root fills with its last join.
    last_joins: FxHashSet<usize>,
    /// The slots whose join waits, and the slots of its two operands.
    waiting: FxHashMap<usize, (usize, usize)>,
    /// The variables of the scope a planned join builds its result on, by
    /// the join's slot, where its factors share one.
    shared: FxHashMap<usize, Arc<[VarId]>>,
    /// The batches of put-off factors, and the batch each of their planned
    /// joins is in, by the join's slot.
    batches: Vec<Batch>,
    batch_of: FxHashMap<usize, usize>,
    union: Union,
    /// The leaves of the vtree's top chain, by variable, whether it ends
    /// over a node that is not a leaf, and the pre-order spans of the vtree's
    /// nodes, made where a batch first has two parts to tell apart (module
    /// documentation; `EvidenceWalk::by_part`).
    chain: Vec<bool>,
    over_subtree: bool,
    spans: Vec<(u32, u32)>,
    /// When a node's last join is tried capped, and the cap ([`DEFER_FROM`],
    /// [`DEFER_GROWTH`]).
    pub(super) defer_from: usize,
    pub(super) defer_growth: u64,
    pending: Option<PendingFormula>,
    /// When a join's result is minimized in full rather than pruned.
    full: FullSchedule,
    /// Whether the last full minimization freed a pair, and the pairs of
    /// the result the last one due was due on ([`FULL_GROWTH`]).
    full_freed: bool,
    full_due_on: usize,
    /// The slots whose results are minimized in full off the schedule: the
    /// last results of given-up batches' chains (`None`), and the batches'
    /// products where they hold fewer pairs than their target's slot
    /// (`Some(target)`; see the module documentation).
    in_full: FxHashMap<usize, Option<usize>>,
    pub(super) done: bool,
    /// The joins finished, each of which makes one formula of two: the
    /// walk makes one of the observed factors in one fewer.
    pub(super) completed_joins: usize,
    #[cfg(test)]
    pub(super) reassociated: usize,
    #[cfg(test)]
    pub(super) unbatched: usize,
    #[cfg(test)]
    pub(super) grouped: usize,
    /// Batches grouped by part.
    #[cfg(test)]
    pub(super) by_part: usize,
    #[cfg(test)]
    pub(super) ended_steps: usize,
    /// Every batch is given up at its first join.
    #[cfg(test)]
    pub(super) give_up_batches: bool,
}

impl EvidenceWalk {
    pub(super) fn new(observed: Vec<(AtomId, bool)>, tree: &tididi::Vtree) -> Self {
        let (chain, over_subtree) = top_chain(tree);
        Self {
            done: observed.is_empty(),
            observed,
            next_observed: 0,
            at: vec![Vec::new(); tree.num_nodes()],
            built: (0..tree.num_nodes()).map(|_| None).collect(),
            nodes: tree.bottomup().collect(),
            next_node: 0,
            slots: Vec::new(),
            slot_pairs: Vec::new(),
            joins: VecDeque::new(),
            last_joins: FxHashSet::default(),
            waiting: FxHashMap::default(),
            shared: FxHashMap::default(),
            batches: Vec::new(),
            batch_of: FxHashMap::default(),
            union: Union::default(),
            chain,
            over_subtree,
            spans: Vec::new(),
            defer_from: DEFER_FROM,
            defer_growth: DEFER_GROWTH,
            pending: None,
            full: FullSchedule::default(),
            full_freed: true,
            full_due_on: 0,
            in_full: FxHashMap::default(),
            completed_joins: 0,
            #[cfg(test)]
            reassociated: 0,
            #[cfg(test)]
            unbatched: 0,
            #[cfg(test)]
            grouped: 0,
            #[cfg(test)]
            by_part: 0,
            #[cfg(test)]
            ended_steps: 0,
            #[cfg(test)]
            give_up_batches: false,
        }
    }

    /// The joins finished, and all the walk makes: one fewer than the
    /// observed factors.
    pub(super) fn progress(&self) -> (usize, usize) {
        let joins = self.observed.len().saturating_sub(1);
        (
            if self.done {
                joins
            } else {
                self.completed_joins
            },
            joins,
        )
    }

    /// A new slot, empty or holding a formula of `pairs` pairs.
    fn slot(&mut self, formula: Option<Formula>, pairs: usize) -> usize {
        let id = self.slots.len();
        self.slots.push(formula);
        self.slot_pairs.push(pairs);
        id
    }

    /// The conjunction of `factors`, planned as joins: a balanced tree over
    /// their list, [`EvidenceWalk::conjoin_groups`] of one group.
    fn conjoin(&mut self, factors: Vec<usize>) -> Option<usize> {
        self.conjoin_groups(vec![factors])
    }

    /// The conjunction of each of `groups` as a balanced tree over its list,
    /// the rounds of all of them planned together, round by round, and then
    /// of their products, two smallest first as their sizes come out.
    //
    // Pairwise rounds pass an odd tail on to the next round, and the last
    // three are joined two smallest first: the tail, often the children's
    // result, is joined last where it is large and early where it is small.
    // Changing that order changes intermediate sizes substantially.
    fn conjoin_groups(&mut self, mut groups: Vec<Vec<usize>>) -> Option<usize> {
        let shared: Vec<_> = groups
            .iter()
            .map(|factors| self.shared_scope(factors))
            .collect();
        while groups.iter().any(|factors| factors.len() > 3) {
            for (factors, shared) in groups.iter_mut().zip(&shared) {
                if factors.len() <= 3 {
                    continue;
                }
                let mut next = Vec::with_capacity(factors.len().div_ceil(2));
                let mut it = std::mem::take(factors).into_iter();
                while let Some(a) = it.next() {
                    next.push(match it.next() {
                        Some(b) => {
                            let out = self.planned(shared.as_ref());
                            self.joins.push_back(Join::Two(a, b, out));
                            out
                        }
                        None => a,
                    });
                }
                *factors = next;
            }
        }
        let mut products = Vec::with_capacity(groups.len());
        for (factors, shared) in groups.into_iter().zip(&shared) {
            match factors[..] {
                // The last three, one of them passed up from an odd round,
                // two smallest first as their sizes come out.
                [_, _, _] => {
                    let out = self.planned(shared.as_ref());
                    self.joins.push_back(Join::Smallest(factors, out));
                    products.push(out);
                }
                [a, b] => {
                    let out = self.planned(shared.as_ref());
                    self.joins.push_back(Join::Two(a, b, out));
                    products.push(out);
                }
                _ => products.extend(factors),
            }
        }
        match products[..] {
            [a, b] => {
                let out = self.slot(None, 0);
                self.joins.push_back(Join::Two(a, b, out));
                Some(out)
            }
            [_, _, _, ..] => {
                let out = self.slot(None, 0);
                self.joins.push_back(Join::Smallest(products, out));
                Some(out)
            }
            _ => products.pop(),
        }
    }

    /// [`EvidenceWalk::conjoin_groups`] of a child's put-off factors in
    /// their `groups`, whose product meets `target`, the child's result, in
    /// a join the node plans after it: the joins planned here make a batch,
    /// each tried capped.
    fn conjoin_put_off(
        &mut self,
        factors: VecDeque<usize>,
        groups: &[Group],
        target: usize,
    ) -> Option<usize> {
        debug_assert_eq!(
            groups.iter().map(|group| group.len).sum::<usize>(),
            factors.len()
        );
        let mut factors = factors.into_iter();
        let groups: Vec<Vec<usize>> = groups
            .iter()
            .map(|group| factors.by_ref().take(group.len).collect())
            .collect();
        #[cfg(test)]
        {
            self.grouped += usize::from(groups.len() > 1);
        }
        let from = self.joins.len();
        let product = self.conjoin_groups(groups);
        if let Some(product) = product
            && self.joins.len() > from
        {
            let id = self.batches.len();
            self.batches.push(Batch { target, product });
            for join in self.joins.range(from..) {
                self.batch_of.insert(join.out(), id);
            }
            self.in_full.insert(product, Some(target));
        }
        product
    }

    /// `factors`, put off in `groups`, grouped by part where some of them
    /// read the top chain's leaves and what they read off it lies in two
    /// parts or more (module documentation): the parts are the lowest nodes
    /// holding what a factor reads off the chain that have none of those
    /// nodes above them, one group per part in the order the parts first
    /// come, and one for the factors that read only the chain. Otherwise
    /// `groups` stays, and at once at `node` where `node` is not on the
    /// chain or the chain ends on a leaf (a right-linear vtree).
    fn by_part(
        &mut self,
        node: VtreeIdx,
        factors: VecDeque<usize>,
        groups: Vec<Group>,
        backend: &TddBackend,
    ) -> (VecDeque<usize>, Vec<Group>) {
        // A factor that reads the chain was put off at a node of it, one
        // with a leaf of the chain for a child; where the chain ends on a
        // leaf, every factor off the chain reads it, one part.
        let tree = backend.vtree();
        let chain_leaf =
            |c: VtreeIdx| tree.node(c).is_leaf() && self.chain[tree.leaf_var(c).0 as usize];
        let of_chain = !tree.node(node).is_leaf() && {
            let (l, r) = tree.children(node);
            chain_leaf(l) || chain_leaf(r)
        };
        if !self.over_subtree || !of_chain {
            return (factors, groups);
        }
        let on_chain = |v: &VarId| self.chain.get(v.0 as usize).copied().unwrap_or(false);
        let vars = |f: usize| self.slots[f].as_ref().and_then(|(_, scope)| scope.vars());
        if !factors
            .iter()
            .any(|&f| vars(f).is_some_and(|vs| vs.iter().any(on_chain)))
        {
            return (factors, groups);
        }
        // The lowest node over what each factor reads off the chain, `None`
        // where it reads only the chain; a factor on the global scope reads
        // every part.
        let lowest: Vec<Option<VtreeIdx>> = factors
            .iter()
            .map(|&f| match vars(f) {
                None => Some(tree.root()),
                Some(vs) => {
                    let off: Vec<VarId> = vs.iter().copied().filter(|v| !on_chain(v)).collect();
                    (!off.is_empty()).then(|| backend.lowest_over_vars(&off))
                }
            })
            .collect();
        // The parts: those nodes with none of them above, each the part of
        // the nodes under it. Spans are nested or apart, and in pre-order a
        // node comes after the nodes above it: a node under the last part
        // found lies in it.
        let mut nodes: Vec<VtreeIdx> = lowest.iter().flatten().copied().collect();
        nodes.sort_unstable_by_key(|n| n.idx());
        nodes.dedup();
        if nodes.len() < 2 {
            return (factors, groups);
        }
        if self.spans.is_empty() {
            self.spans = crate::eval::spans(&tree);
        }
        nodes.sort_unstable_by_key(|n| self.spans[n.idx()].0);
        let mut part_of: FxHashMap<VtreeIdx, VtreeIdx> = FxHashMap::default();
        let mut parts = 0;
        let mut last: Option<VtreeIdx> = None;
        for n in nodes {
            let part = match last {
                Some(p) if self.spans[n.idx()].0 <= self.spans[p.idx()].1 => p,
                _ => {
                    parts += 1;
                    last = Some(n);
                    n
                }
            };
            part_of.insert(n, part);
        }
        if parts < 2 {
            return (factors, groups);
        }
        #[cfg(test)]
        {
            self.by_part += 1;
        }
        let mut rank: FxHashMap<Option<VtreeIdx>, usize> = FxHashMap::default();
        let ranks: Vec<usize> = lowest
            .iter()
            .map(|n| {
                let next = rank.len();
                *rank.entry(n.map(|n| part_of[&n])).or_insert(next)
            })
            .collect();
        let mut by: Vec<Vec<usize>> = vec![Vec::new(); rank.len()];
        for (&f, &k) in factors.iter().zip(&ranks) {
            by[k].push(f);
        }
        let groups = by
            .iter()
            .map(|part| Group {
                len: part.len(),
                pairs: part.iter().map(|&f| self.slot_pairs[f]).sum(),
            })
            .collect();
        (by.into_iter().flatten().collect(), groups)
    }

    /// Gives up batch `id`, one of whose joins went past its cap: what the
    /// batch's joins still to come would have read, the two operands of
    /// that join first, goes into the batch's target one at a time, in the
    /// order the joins read it, which is the order the factors were put off
    /// in, and fills the slot of the join that was to read the batch's
    /// product with the target. Where that join also read the node's other
    /// items, their product goes into the target first, so that the chain
    /// ends the node's step. The chain's last result is minimized in full
    /// (see the module documentation).
    fn unbatch(&mut self, id: usize) {
        let Batch { target, product } = self.batches[id];
        self.in_full.remove(&product);
        let (mut operands, mut outs) = (Vec::new(), Vec::new());
        let batch_of = &self.batch_of;
        self.joins.retain(|join| {
            if batch_of.get(&join.out()) != Some(&id) {
                return true;
            }
            match join {
                Join::Two(a, b, _) => operands.extend([*a, *b]),
                Join::Smallest(items, _) => operands.extend_from_slice(items),
            }
            outs.push(join.out());
            false
        });
        for out in &outs {
            self.batch_of.remove(out);
            self.shared.remove(out);
        }
        operands.retain(|slot| !outs.contains(slot));
        let at = self
            .joins
            .iter()
            .position(|join| join.reads(product))
            .expect("a batch's product is read by a join planned after it");
        debug_assert!(
            self.joins[at].reads(target),
            "the join that reads a batch's product reads its target"
        );
        let mut chain = Vec::with_capacity(operands.len() + 1);
        let into = match self
            .joins
            .remove(at)
            .expect("the join reading the product is planned")
        {
            Join::Two(_, _, out) => out,
            Join::Smallest(items, out) => {
                let others: Vec<usize> = items
                    .into_iter()
                    .filter(|&slot| slot != target && slot != product)
                    .collect();
                #[cfg(test)]
                {
                    self.ended_steps += 1;
                }
                match others[..] {
                    [] => {}
                    [other] => operands.insert(0, other),
                    [a, b] => {
                        let made = self.slot(None, 0);
                        chain.push(Join::Two(a, b, made));
                        operands.insert(0, made);
                    }
                    _ => {
                        let made = self.slot(None, 0);
                        chain.push(Join::Smallest(others, made));
                        operands.insert(0, made);
                    }
                }
                out
            }
        };
        self.in_full.insert(into, None);
        let mut acc = target;
        let count = operands.len();
        for (i, operand) in operands.into_iter().enumerate() {
            let next = if i + 1 == count {
                into
            } else {
                self.slot(None, 0)
            };
            chain.push(Join::Two(acc, operand, next));
            acc = next;
        }
        // In the node's step the other items can be made by joins planned
        // before it, which the chain then follows.
        for (i, join) in chain.into_iter().enumerate() {
            self.joins.insert(at + i, join);
        }
        #[cfg(test)]
        {
            self.unbatched += 1;
        }
    }

    /// An empty slot for a join's result, built on the scope over `shared`
    /// where that is given.
    fn planned(&mut self, shared: Option<&Arc<[VarId]>>) -> usize {
        let out = self.slot(None, 0);
        if let Some(vars) = shared {
            self.shared.insert(out, Arc::clone(vars));
        }
        out
    }

    /// The variables of all of `factors`, where there are more than two of
    /// them, each holds a formula on a scope of its own, and together they
    /// hold at most [`SHARED_UP_TO`] variables. Their joins are then built on
    /// the scope over all of them: it is projected once and each factor
    /// embedded once, where joining two at a time on the union of the two
    /// projects a scope and embeds both operands at every join. The order of
    /// the joins is the same, and so are the functions they build.
    fn shared_scope(&self, factors: &[usize]) -> Option<Arc<[VarId]>> {
        if factors.len() <= 2 {
            return None;
        }
        let mut vars: Vec<VarId> = Vec::new();
        for &id in factors {
            let own = self.slots[id].as_ref()?.1.vars()?;
            vars.extend_from_slice(own);
            if vars.len() > SHARED_UP_TO * factors.len() {
                return None;
            }
        }
        vars.sort_unstable();
        vars.dedup();
        (vars.len() <= SHARED_UP_TO).then(|| vars.into())
    }

    /// What the put-off test charges slot `id`: its pairs, and
    /// [`LEVEL_PAIRS`] per node of its scope's vtree.
    fn charged(&self, id: usize) -> usize {
        self.slot_pairs[id] + LEVEL_PAIRS * self.scope_nodes(id)
    }

    /// The nodes of the vtree of slot `id`'s scope, or for a waiting join's
    /// result, of the larger of its operands' scopes.
    fn scope_nodes(&self, id: usize) -> usize {
        let of = |s: usize| {
            self.slots[s]
                .as_ref()
                .map_or(0, |(_, scope)| scope.vtree().num_nodes())
        };
        match self.waiting.get(&id) {
            Some(&(a, b)) => of(a).max(of(b)),
            None => of(id),
        }
    }

    /// The waiting results among `node`'s children's that its plan reads
    /// other than as its largest item passed up or, at the root, its
    /// largest child set aside: a waiting result's size is its cap, less
    /// than the result's, and the plan must decide as it would on the
    /// result.
    fn to_join_first(&self, tree: &tididi::Vtree, node: VtreeIdx) -> Vec<usize> {
        let (l, r) = tree.children(node);
        let children: Vec<&Built> = [l, r]
            .into_iter()
            .filter_map(|c| self.built[c.idx()].as_ref())
            .collect();
        let waiting: Vec<usize> = children
            .iter()
            .map(|b| b.main)
            .filter(|m| self.waiting.contains_key(m))
            .collect();
        let [main] = waiting[..] else { return waiting };
        let cap = self.slot_pairs[main];
        let own = self.at[node.idx()].iter().map(|&id| self.slot_pairs[id]);
        let others: Vec<usize> = own
            .clone()
            .chain(
                children
                    .iter()
                    .filter(|b| b.main != main)
                    .map(|b| self.slot_pairs[b.main]),
            )
            .collect();
        let put_off = children
            .iter()
            .find(|b| b.main == main)
            .map_or(0, |b| b.put_off_size);
        let kept = others.iter().all(|&z| z < cap)
            && if node == tree.root() {
                put_off == 0 || own.sum::<usize>() <= put_off
            } else {
                let rest = others.iter().sum::<usize>()
                    + children.iter().map(|b| b.put_off_size).sum::<usize>();
                PUT_OFF_BELOW * rest < self.charged(main)
            };
        if kept { Vec::new() } else { waiting }
    }

    pub(super) fn run(
        &mut self,
        backend: &TddBackend,
        compiled: &Compiled,
        own_scope: &impl Fn(AtomId) -> Result<VtreeScope, CompileError>,
        scope_over: &impl Fn(&[VarId]) -> Result<VtreeScope, CompileError>,
        sizes: &mut QuerySizes,
        watch: Option<usize>,
    ) -> Result<Option<Formula>, CompileError> {
        let tree = &backend.vtree();
        while self.next_observed < self.observed.len() {
            let (a, value) = self.observed[self.next_observed];
            // A link tied to another reader's copies goes where the link
            // would have, and weighs what the link would: the plan below
            // reads its items' pairs, and the readers' links are one loop
            // with the copies renamed. Weighed by their own pairs, ties
            // would be passed up where the links are joined, and conjoined
            // among themselves without the links they stand on
            // (`crate::tp::tie`).
            let (own, node, formula, pairs) = match compiled.for_queries.ties.get(&a) {
                Some(tie) => {
                    debug_assert!(!value, "a clause table's link is observed false");
                    let mut vars = tie.vars();
                    vars.dedup();
                    let own = scope_over(&vars)?;
                    let formula = tie.formula(backend, &own)?;
                    let pairs = compiled.formulas[tie.reference as usize]
                        .as_ref()
                        .map_or(0, |f| backend.size(f))
                        .max(backend.size(&formula));
                    (own, tie.node(backend), formula, pairs)
                }
                None => {
                    let own = own_scope(a)?;
                    let node = backend.lowest_over(&own);
                    let formula = compiled.literal_formula(
                        backend,
                        &own,
                        a,
                        value,
                        (&mut sizes.negations, NegationSource::Evidence),
                    )?;
                    let pairs = backend.size(&formula);
                    (own, node, formula, pairs)
                }
            };
            let id = self.slot(Some((formula, own)), pairs);
            self.at[node.idx()].push(id);
            self.next_observed += 1;
        }
        loop {
            while let Some(join) = self.joins.front_mut() {
                let (a, b, out) = match join {
                    Join::Two(a, b, out) => (*a, *b, *out),
                    Join::Smallest(items, out) => {
                        let out = *out;
                        let mut items = std::mem::take(items);
                        self.joins.pop_front();
                        items.sort_by_key(|&id| self.slot_pairs[id]);
                        // A waiting result about to meet a partial product
                        // is joined first, and the order taken on its size.
                        if items.len() > 2
                            && let Some(&wait) =
                                items[..2].iter().find(|id| self.waiting.contains_key(id))
                        {
                            let (x, y) = self.waiting.remove(&wait).unwrap();
                            self.joins.push_front(Join::Smallest(items, out));
                            self.joins.push_front(Join::Two(x, y, wait));
                            continue;
                        }
                        let (a, b) = (items[0], items[1]);
                        let into = if items.len() == 2 {
                            out
                        } else {
                            let shared = self.shared.get(&out).cloned();
                            let into = self.planned(shared.as_ref());
                            if let Some(&id) = self.batch_of.get(&out) {
                                self.batch_of.insert(into, id);
                            }
                            into
                        };
                        if items.len() > 2 {
                            items.splice(..2, [into]);
                            self.joins.push_front(Join::Smallest(items, out));
                        }
                        self.joins.push_front(Join::Two(a, b, into));
                        continue;
                    }
                };
                if self.pending.is_none() {
                    let last = self.joins.len() == 1 && self.next_node == self.nodes.len();
                    if let Some((wait, other)) = [(a, b), (b, a)]
                        .into_iter()
                        .find(|(w, _)| self.waiting.contains_key(w))
                    {
                        let (x, y) = self.waiting.remove(&wait).unwrap();
                        let (zx, zy, zo) = (
                            self.slot_pairs[x],
                            self.slot_pairs[y],
                            self.slot_pairs[other],
                        );
                        if last && !self.waiting.contains_key(&other) && zo < zx && zo < zy {
                            let (larger, smaller) = if zx >= zy { (x, y) } else { (y, x) };
                            let t = self.slot(None, 0);
                            self.joins.pop_front();
                            self.joins.push_front(Join::Two(t, smaller, out));
                            self.joins.push_front(Join::Two(other, larger, t));
                            #[cfg(test)]
                            {
                                self.reassociated += 1;
                            }
                        } else {
                            self.joins.push_front(Join::Two(x, y, wait));
                        }
                        continue;
                    }
                    // A cap under the stop the attempt runs under already: on
                    // a node's last join, or on a join of a batch whose
                    // target is there to take its operands instead, at the
                    // larger of the floor and what a join with the target
                    // is charged.
                    let floor = self
                        .defer_growth
                        .saturating_mul((self.slot_pairs[a] + self.slot_pairs[b]) as u64);
                    let batch = self
                        .batch_of
                        .get(&out)
                        .copied()
                        .filter(|&id| self.slots[self.batches[id].target].is_some());
                    let floor = match batch {
                        #[cfg(test)]
                        Some(id) if self.give_up_batches => {
                            self.unbatch(id);
                            continue;
                        }
                        Some(id) => floor.max(self.charged(self.batches[id].target) as u64),
                        None => floor,
                    };
                    let cap = (!last
                        && (batch.is_some()
                            || self.slot_pairs[a] >= self.defer_from
                                && self.slot_pairs[b] >= self.defer_from
                                && self.last_joins.contains(&out)))
                    .then(|| backend.engine.limits().armed().stop_rules())
                    .filter(|outer| outer.after_pairs.is_none_or(|(pairs, _)| pairs > floor));
                    let (_, sa) = self.slots[a].as_ref().unwrap();
                    let (_, sb) = self.slots[b].as_ref().unwrap();
                    let shared = self.shared.get(&out).filter(|_| !sa.same(sb));
                    // Both are ascending and distinct: merged, not sorted, and
                    // each operand's renaming onto the union read off the
                    // merge.
                    let merged = !sa.same(sb)
                        && shared.is_none()
                        && match (sa.vars(), sb.vars()) {
                            (Some(x), Some(y)) => {
                                self.union.merge(x, y);
                                true
                            }
                            _ => false,
                        };
                    let scope = if sa.same(sb) {
                        sa.clone()
                    } else if let Some(vars) = shared {
                        scope_over(vars)?
                    } else if merged {
                        scope_over(&self.union.vars)?
                    } else {
                        backend.global()
                    };
                    let (first, second) = match merged {
                        true => (Some(&self.union.first[..]), Some(&self.union.second[..])),
                        false => (None, None),
                    };
                    // The walk keeps its operands until a result exists.
                    // `and_onto` moves them onto the join's scope and gives
                    // them back when stopped, each on the scope it was left
                    // on; a pair refused after its moves goes back on the
                    // join's scope, which the retry then needs none for. The
                    // cap's pair floor is for the conjunction: the moves put
                    // no pair in flight, so it cannot stop them.
                    let (fa, fb) = (self.slots[a].take().unwrap(), self.slots[b].take().unwrap());
                    let attempt = match cap {
                        Some(outer) => {
                            let _cap = backend.engine.limits().edit(|c| {
                                c.with_stop_rules(StopRules {
                                    unconditional: outer.unconditional,
                                    after_pairs: Some((floor, StopAt::WorkUnits(0))),
                                })
                            });
                            backend.and_onto(fa, first, fb, second, &scope)
                        }
                        None => backend.and_onto(fa, first, fb, second, &scope),
                    };
                    let formula = match attempt {
                        Ok(formula) => formula,
                        Err(refused)
                            if matches!(refused.0, CompileError::Tdd(OperationError::Stopped))
                                && refused.1.1.same(&scope)
                                && refused.2.1.same(&scope)
                                && cap.is_some_and(|outer| {
                                    !reached(outer.unconditional, backend)
                                }) =>
                        {
                            // Past the cap: the operands go into the batch's
                            // target, or wait on the join's scope.
                            let (_, fa, fb) = *refused;
                            self.slots[a] = Some(fa);
                            self.slots[b] = Some(fb);
                            if let Some(id) = batch {
                                self.unbatch(id);
                                continue;
                            }
                            self.slot_pairs[out] = usize::try_from(floor).unwrap_or(usize::MAX);
                            self.last_joins.remove(&out);
                            self.waiting.insert(out, (a, b));
                            self.joins.pop_front();
                            continue;
                        }
                        Err(refused) => {
                            let (e, fa, fb) = *refused;
                            self.slots[a] = Some(fa);
                            self.slots[b] = Some(fb);
                            return Err(e);
                        }
                    };
                    self.pending = Some(PendingFormula {
                        formula: (formula, scope),
                        reduced: false,
                    });
                    self.slots[a] = None;
                    self.slots[b] = None;
                }
                let pending = self.pending.as_mut().unwrap();
                // The walk's last join makes the evidence formula, which only
                // counting reads, and counting needs no reduced diagram: it
                // stays as the conjunction left it (a full reduction there
                // can spend the rest of the round's work). Nor is it held to
                // the watch: what is left of the attempt is counting it, which
                // costs less than the reduction a stop would have paid for.
                //
                // The other joins' results are pruned after every join and
                // minimized in full on the walk's own schedule
                // (`FullSchedule`, kept here since a trial's backend does not
                // outlive the trial): the stride widens while the full plan
                // frees nothing at all, and it runs again at once wherever it
                // frees a pair. The full plan after every join costs over half
                // the conjunctions' work for a few freed pairs; the prune alone lets
                // twins compound (dice_digitRecognition, where most joins
                // leave some).
                //
                // The schedule counts joins, and the full plan's twin search
                // costs as much as its result is large: a seventeenth join
                // can be the walk's largest. Where the full plan last freed
                // nothing, a result over [`FULL_GROWTH`] times the one it
                // was last due on is pruned instead, and the schedule goes
                // on as after a full plan that freed nothing.
                //
                // A result over the watch is measured as the schedule left
                // it: a full plan there frees next to nothing and never
                // brings a result under the watch.
                //
                // The last result of a given-up batch's chain, and a batch's
                // product that holds fewer pairs than its target, are
                // minimized in full off the schedule (see the module
                // documentation).
                let last = self.joins.len() == 1 && self.next_node == self.nodes.len();
                // A count visits every node of every level: the result's
                // size is counted once, after its reduction, which the full
                // plan's yield reads too.
                let mut counted = None;
                let settle = match self.in_full.remove(&out) {
                    Some(None) => true,
                    Some(Some(target)) => {
                        backend.size(&pending.formula.0) < self.slot_pairs[target]
                    }
                    None => false,
                };
                if !pending.reduced && !last {
                    let f = &mut pending.formula.0;
                    if settle {
                        backend.minimize(f, Reduce::Settled)?;
                        counted = Some(backend.size(f));
                    } else {
                        match self.full.full_is_due().then(|| backend.size(f)) {
                            // Passed over, the full plan counts as one
                            // that freed nothing.
                            Some(before)
                                if !self.full_freed && before > FULL_GROWTH * self.full_due_on =>
                            {
                                self.full_due_on = before;
                                backend.minimize(f, Reduce::Prune)?;
                                self.full.observed(0, 0);
                            }
                            Some(before) => {
                                self.full_due_on = before;
                                backend.minimize(f, Reduce::Settled)?;
                                let after = backend.size(f);
                                self.full.observed(before - after, 0);
                                self.full_freed = after < before;
                                counted = Some(after);
                            }
                            None => {
                                backend.minimize(f, Reduce::Prune)?;
                                self.full.took_partial();
                            }
                        }
                    }
                }
                pending.reduced = true;
                let size = counted.unwrap_or_else(|| backend.size(&pending.formula.0));
                self.slots[out] = Some(self.pending.take().unwrap().formula);
                self.slot_pairs[out] = size;
                self.shared.remove(&out);
                self.batch_of.remove(&out);
                self.joins.pop_front();
                self.completed_joins += 1;
                // Commit even an oversized result before yielding. The next
                // call checks the saved high-water mark before using it.
                sizes.record(size, if last { None } else { watch })?;
            }
            let Some(&node) = self.nodes.get(self.next_node) else {
                let main = self.built[tree.root().idx()]
                    .as_ref()
                    .expect("observations reach the root")
                    .main;
                if let Some((x, y)) = self.waiting.remove(&main) {
                    self.joins.push_back(Join::Two(x, y, main));
                    continue;
                }
                self.done = true;
                self.built[tree.root().idx()] = None;
                return Ok(self.slots[main].take());
            };
            if !tree.node(node).is_leaf() {
                let first = self.to_join_first(tree, node);
                if !first.is_empty() {
                    for main in first {
                        let (x, y) = self.waiting.remove(&main).unwrap();
                        self.joins.push_back(Join::Two(x, y, main));
                    }
                    continue;
                }
            }
            self.next_node += 1;
            let fresh = self.slots.len();
            let mut items: Vec<(usize, usize)> = std::mem::take(&mut self.at[node.idx()])
                .into_iter()
                .map(|id| (id, self.slot_pairs[id]))
                .collect();
            let own = items.len();
            let mut put_off = Vec::new();
            if !tree.node(node).is_leaf() {
                let (l, r) = tree.children(node);
                for c in [l, r] {
                    if let Some(b) = self.built[c.idx()].take() {
                        items.push((b.main, self.slot_pairs[b.main]));
                        put_off.push((b.put_off, b.put_off_size, b.groups));
                    }
                }
            }
            let Some((largest, size)) = items
                .iter()
                .enumerate()
                .map(|(i, &(_, z))| (i, z))
                .rev()
                .max_by_key(|&(_, z)| z)
            else {
                continue;
            };
            let rest = items.iter().map(|&(_, z)| z).sum::<usize>() - size
                + put_off.iter().map(|&(_, z, _)| z).sum::<usize>();
            if node != tree.root() && PUT_OFF_BELOW * rest < self.charged(items[largest].0) {
                let (main, _) = items.remove(largest);
                let mut groups = Vec::new();
                for &group in put_off.iter().flat_map(|(_, _, groups)| groups) {
                    push_group(&mut groups, group);
                }
                push_group(
                    &mut groups,
                    Group {
                        len: items.len(),
                        pairs: items.iter().map(|&(_, z)| z).sum(),
                    },
                );
                let mut rest_factors = VecDeque::new();
                for (factors, _, _) in put_off {
                    rest_factors = concat(rest_factors, factors);
                }
                rest_factors.extend(items.into_iter().map(|(f, _)| f));
                self.built[node.idx()] = Some(Built {
                    main,
                    put_off: rest_factors,
                    put_off_size: rest,
                    groups,
                });
                continue;
            }
            // Below the root, the node's own factors are conjoined with the
            // largest child's put-off factors, a batch of that child, and
            // what the node built besides the largest is joined into it
            // once; in the balanced pairing it met a partial product at
            // every round. At the root the own factors may span the whole
            // vtree, and their product alone grows past what the child
            // constrains. So the root sets its largest child aside only
            // where the child's put-offs weigh at least the own factors, or
            // where it has none.
            let own_size: usize = items[..own].iter().map(|&(_, z)| z).sum();
            let largest_child = (own..items.len()).max_by_key(|&i| items[i].1);
            let aside = if node == tree.root() {
                largest_child.filter(|&i| own_size <= put_off[i - own].1 || put_off[i - own].1 == 0)
            } else {
                largest_child
            };
            let mut step = Vec::with_capacity(items.len());
            let mut items = items.into_iter().map(|(f, _)| f).enumerate();
            let mut mine: Vec<usize> = items.by_ref().take(own).map(|(_, f)| f).collect();
            if aside.is_none() {
                step.append(&mut mine);
            }
            let mut largest = None;
            for ((i, main), (mut factors, _, mut groups)) in items.zip(put_off) {
                if Some(i) == aside {
                    // The own factors are put off into the batch of the
                    // child set aside, one more group, as a node that
                    // passes its largest up puts its other items off.
                    largest = Some(main);
                    push_group(
                        &mut groups,
                        Group {
                            len: mine.len(),
                            pairs: own_size,
                        },
                    );
                    factors.extend(std::mem::take(&mut mine));
                    let (factors, groups) = self.by_part(node, factors, groups, backend);
                    step.extend(self.conjoin_put_off(factors, &groups, main));
                } else if factors.is_empty() {
                    step.push(main);
                } else {
                    let mut batch = vec![main];
                    let (factors, groups) = self.by_part(node, factors, groups, backend);
                    batch.extend(self.conjoin_put_off(factors, &groups, main));
                    step.extend(self.conjoin(batch));
                }
            }
            // The largest set aside meets what the node built besides it
            // smallest first, as their sizes come out, rather than after
            // their product.
            if let Some(main) = largest {
                step.insert(0, main);
                if step.len() > 2 {
                    let out = self.slot(None, 0);
                    self.joins.push_back(Join::Smallest(step, out));
                    step = vec![out];
                }
            }
            let main = self.conjoin(step);
            if let Some(main) = main
                && main >= fresh
                && node != tree.root()
            {
                self.last_joins.insert(main);
            }
            self.built[node.idx()] = main.map(|main| Built {
                main,
                put_off: VecDeque::new(),
                put_off_size: 0,
                groups: Vec::new(),
            });
        }
    }
}

/// Whether the bound `at` has fallen.
fn reached(at: Option<StopAt>, backend: &TddBackend) -> bool {
    match at {
        Some(StopAt::WorkUnits(w)) => backend.engine.limits().work_units() >= w,
        Some(StopAt::Time(t)) => std::time::Instant::now() >= t,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Put-off lists are joined in the order they were put off, whichever
    /// is moved into the other.
    #[test]
    fn concat_keeps_the_order_whichever_side_is_longer() {
        let d = |v: &[usize]| v.iter().copied().collect::<VecDeque<_>>();
        assert_eq!(concat(d(&[1, 2, 3]), d(&[4])), d(&[1, 2, 3, 4]));
        assert_eq!(concat(d(&[1]), d(&[2, 3, 4])), d(&[1, 2, 3, 4]));
        assert_eq!(concat(d(&[1, 2]), d(&[3, 4, 5])), d(&[1, 2, 3, 4, 5]));
        assert_eq!(concat(d(&[]), d(&[1])), d(&[1]));
        assert_eq!(concat(d(&[1]), d(&[])), d(&[1]));
    }

    /// A join's union is the ascending list of the variables of both
    /// operands, and each operand's renaming sends its `i`th variable to
    /// that variable's place in the union, counted from one: the renaming
    /// an embed onto the union's scope looks up. One `Union` serves every
    /// pair in turn, as it does in the walk.
    #[test]
    fn a_union_renames_each_operand_onto_its_variables() {
        let mut union = Union::default();
        let mut state = 0x2545_F491_4F6C_DD1Du64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for round in 0..300 {
            let n = 1 + round % 40;
            let pick = |bits: u64| -> Vec<VarId> {
                (1..=n)
                    .filter(|v| (bits >> (v % 64)) & 1 == 1)
                    .map(VarId)
                    .collect()
            };
            let (x, y) = (pick(next()), pick(next()));
            union.merge(&x, &y);
            let mut expected: Vec<VarId> = x.iter().chain(&y).copied().collect();
            expected.sort_unstable();
            expected.dedup();
            assert_eq!(union.vars, expected, "x {x:?} y {y:?}");
            for (operand, map) in [(&x, &union.first), (&y, &union.second)] {
                let places: Vec<VarId> = operand
                    .iter()
                    .map(|v| VarId(expected.binary_search(v).unwrap() as u32 + 1))
                    .collect();
                assert_eq!(*map, places, "x {x:?} y {y:?}");
            }
        }
    }
}
