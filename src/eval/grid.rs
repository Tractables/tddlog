//! A level read as a grid: every node has the same number of pairs, each
//! node's pairs are its first pair shifted by one list of offsets shared by
//! all nodes, and the first pairs are an affine function of the digits of
//! the node index in a mixed radix; the offsets are one too, of the digits
//! of their place, or are kept as a list. The large levels of a dense
//! network's evidence are such tables: the table over a cluster's
//! variables with one of them summed out.
//!
//! A grid's description is plain data ([`Grid::new`]): the slots of the
//! first pair, and for each digit, offset digits first, its radix and what
//! one unit of it adds to the left child's slot, the right child's and the
//! node's. A fitted level gives it ([`Grid::fit`]), and so does a level the
//! library keeps as such a description ([`Grid::implicit`]). A digit of
//! more places than a vector kernel's lanes is read as two where its radix
//! allows ([`split`]), so that a plan may hold some of its places in lanes
//! or sum over some of them and run the rest where it likes.
//!
//! The count passes over a grid are three contractions, each
//! `out[o] += x[a] · y[b]` over the grid's pairs with `o`, `a` and `b`
//! affine in the digits ([`Pass`]): the pass up, and the pass down into
//! each child. A digit that moves `o` is an output's, one that does not is
//! summed. Each output takes its terms in the order the passes over pairs
//! add them, so that the answers are theirs to the bit: up, a node's pairs
//! in their order, from the first product; down, a child's slot its terms
//! in the order of the parents' nodes and pairs, which holds where the slot
//! is a one-to-one function of the digits it depends on and the summed
//! digits keep their stored order among themselves. A pass down whose slot
//! is not one to one runs the digits stored first in a tile and the others
//! in stored order, the tile's outputs distinct slots.
//!
//! Each contraction has a plan ([`Plan`]): the digits whose places a tile
//! of outputs holds, the summed digits it runs inside, and the order of the
//! others, chosen against a model of the cache lines the order misses and
//! of the cycles its kernel spends. Where the arrays fit in the cache the
//! model gives them, every order misses the same lines, and the others run
//! in stored order without a search ([`Grid::order`]). In floats the plan
//! runs through a vector kernel where its outputs are lanes (`kernel`); in
//! [`super::Count`]s, or where no kernel fits, through scalar loops over
//! the same plan.
//!
//! A plan settles its outputs as it stores them ([`super::Value::settle`]),
//! each at its last store, where every summed place is in its sum: a
//! column many times the cache is not read again to note its range. Where
//! a pass down's slot is not one to one, a slot's last store is not known
//! at the place it is made, and the column is settled once it is complete.
//!
//! A pass down whose column of contexts is mostly zero walks the grid node
//! by node instead ([`Grid::walk_into`]), as the pass over pairs does, and
//! skips the nodes of zero context, where the model charges the walk of
//! the others less than the two plans ([`Grid::walks`]).
//!
//! A level is fitted once, by one pass in order over its pairs, which
//! compares each pair's two words with the grid's prediction. Nothing
//! depends on how the level's nodes are numbered beyond whether the
//! numbering is affine: a level whose numbering is not fails the check and
//! is counted over its pairs.

use tididi::diagram::{ChildDecoder, ChildPair, ChildRef, ImplicitLevel, TddLevel};

use super::kernel::{Kernel, LANES, LaneSet};
use super::{Range, Value};

/// The bytes of a cache line.
const LINE: usize = 64;

/// The cycles the model charges a pair of the walk down node by node
/// ([`Grid::walk_into`]), into both children: two counts up loaded, two
/// products, and two sums loaded and stored, at slots that no order of
/// the walk keeps near each other.
const WALK: f64 = 6.0;

/// The lines the dimensions an order puts innermost may touch for the
/// order to count on their staying in cache while the dimensions outside
/// them run: 1 MiB, about a core's second-level cache.
const BUDGET: f64 = ((1 << 20) / LINE) as f64;

/// The cycles the plan model charges a line it counts as missed, against
/// the cycles of a kernel's arithmetic.
const MISS: f64 = 20.0;

/// The most places the summed dimensions inside a tile may have: their
/// offsets are a table the tile walks.
const MOST_SUMMED: usize = 1 << 12;

/// One digit of a numbering: its radix, and what one unit of it adds on
/// each side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Digit {
    radix: usize,
    step: (isize, isize),
}

/// A grid's places for the walk node by node ([`Grid::walk`]).
#[derive(Clone, Debug)]
struct Walk {
    /// The node digits, by their steps of the node index from the least:
    /// what each adds to the slots of a node's first pair.
    nodes: Vec<Digit>,
    /// The offsets of a node's pairs from its first, in order.
    offsets: Vec<(isize, isize)>,
}

impl Walk {
    fn new(dims: &[Dim], list: &[(isize, isize)]) -> Walk {
        let mut nodes: Vec<&Dim> = dims.iter().filter(|d| d.node != 0).collect();
        nodes.sort_by_key(|d| d.node.unsigned_abs());
        debug_assert!(
            nodes
                .iter()
                .try_fold(1isize, |period, d| (d.node == period)
                    .then_some(period * d.radix as isize))
                .is_some(),
            "the node digits are a mixed radix of the node index"
        );
        let mut offsets = vec![(0isize, 0isize)];
        for d in dims.iter().filter(|d| d.node == 0) {
            let places: Vec<(isize, isize)> = if d.listed {
                list.to_vec()
            } else {
                (0..d.radix as isize)
                    .map(|c| (c * d.left, c * d.right))
                    .collect()
            };
            offsets = places
                .iter()
                .flat_map(|&(pl, pr)| offsets.iter().map(move |&(ol, or)| (ol + pl, or + pr)))
                .collect();
        }
        Walk {
            nodes: nodes
                .into_iter()
                .map(|d| Digit {
                    radix: d.radix,
                    step: (d.left, d.right),
                })
                .collect(),
            offsets,
        }
    }
}

/// One dimension of a grid's pairs, a digit of the node index or of the
/// offset's place: its radix, and what one unit of it adds to a pair's slot
/// in the left and the right child's columns and to its node's slot in the
/// level's. An offset digit adds nothing to the node's. A `listed`
/// dimension is the offsets' place where they are not affine in digits of
/// it: its places add the grid's `list`, and its steps are that list's
/// extent over its places on each side, zero where every offset is.
#[derive(Clone, Copy, Debug)]
pub(super) struct Dim {
    pub(super) radix: usize,
    pub(super) left: isize,
    pub(super) right: isize,
    pub(super) node: isize,
    pub(super) listed: bool,
}

impl Dim {
    /// A digit of the offset's place.
    pub(super) fn pair(radix: usize, left: isize, right: isize) -> Dim {
        Dim {
            radix,
            left,
            right,
            node: 0,
            listed: false,
        }
    }

    /// A digit of the node index, `node` what it adds to the index.
    pub(super) fn node(radix: usize, left: isize, right: isize, node: isize) -> Dim {
        Dim {
            radix,
            left,
            right,
            node,
            listed: false,
        }
    }
}

/// The three contractions of a grid's count passes, each `out[o] += x[a] ·
/// y[b]` over the grid's pairs: up, `out` the level's counts and `x`, `y`
/// the left and the right child's; down into the left child, `out` its
/// column down, `x` the level's column down and `y` the right child's
/// counts up; down into the right, the mirror.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Pass {
    Up,
    Left,
    Right,
}

impl Pass {
    const ALL: [Pass; 3] = [Pass::Up, Pass::Left, Pass::Right];

    /// What a step of `node`, `left` and `right` on the level's, the left
    /// and the right child's slots adds to `out`'s, `x`'s and `y`'s.
    pub(super) fn of(self, node: isize, left: isize, right: isize) -> [isize; 3] {
        match self {
            Pass::Up => [node, left, right],
            Pass::Left => [left, node, right],
            Pass::Right => [right, node, left],
        }
    }

    /// Which of `out`, `x` and `y` is indexed by the level's nodes.
    pub(super) fn node_side(self) -> usize {
        match self {
            Pass::Up => 0,
            Pass::Left | Pass::Right => 1,
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// How a contraction runs ([`Grid::plan`]). At each place of `outer`, in
/// that order innermost first, it walks the places of the summed
/// dimensions inside the tile in stored order, and at each the places of
/// the lanes' and the tile's output dimensions (`cells`), whose sums the
/// kernel and the scalar loops alike hold in registers.
#[derive(Clone, Debug)]
pub(super) struct Plan {
    pass: Pass,
    /// What a step of each dimension adds to `out`, `x` and `y`; zero for a
    /// listed one, whose places add the list.
    steps: Vec<[isize; 3]>,
    /// The places of the lanes' and the tile's dimensions, the lanes'
    /// first and the first fastest, as the offsets of their
    /// slots in `out`, `x` and `y`: the outputs whose sums the scalar loops
    /// hold together.
    cells: Vec<[isize; 3]>,
    /// The offsets the places of the summed dimensions inside the tile add
    /// to `x` and `y`, in stored order, the first fastest.
    offsets: Vec<[isize; 2]>,
    /// The other dimensions, innermost first.
    outer: Vec<usize>,
    /// The cycles the model charges the plan's arithmetic ([`Grid::work`]).
    work: f64,
    /// The kernel of floats, where one fits.
    kernel: Option<Kernel>,
}

/// A level read as a grid (module doc).
#[derive(Clone, Debug)]
pub(super) struct Grid {
    pairs: usize,
    /// The first pair of the first node, as child slots on each side.
    at: (isize, isize),
    /// The dimensions in the order the pairs are stored, fastest first: the
    /// offset's digits, then the node index's.
    dims: Vec<Dim>,
    /// The offsets of a node's pairs from its first, where they are not
    /// affine; empty otherwise.
    list: Vec<(isize, isize)>,
    /// The places of the walk node by node.
    walk: Walk,
    /// The plans of the pass up and of the pass down into each child.
    plans: [Plan; 3],
}

/// The values of a mixed-radix numbering, in order: the digits counted like
/// an odometer from the fastest.
struct Odometer<'a> {
    digits: &'a [Digit],
    count: Vec<usize>,
    at: (isize, isize),
}

impl<'a> Odometer<'a> {
    fn new(digits: &'a [Digit], at: (isize, isize)) -> Self {
        Odometer {
            digits,
            count: vec![0; digits.len()],
            at,
        }
    }

    /// The value at the current place, and a step to the next.
    #[inline]
    fn next(&mut self) -> (isize, isize) {
        let out = self.at;
        for (d, c) in self.digits.iter().zip(self.count.iter_mut()) {
            *c += 1;
            if *c < d.radix {
                self.at = (self.at.0 + d.step.0, self.at.1 + d.step.1);
                return out;
            }
            *c = 0;
            let back = (d.radix - 1) as isize;
            self.at = (self.at.0 - back * d.step.0, self.at.1 - back * d.step.1);
        }
        out
    }
}

impl Grid {
    /// The grid of the first pair `at`, the dimensions `dims`, offset digits
    /// first and each block fastest first, and the offsets `list` of a
    /// listed dimension, empty without one; with the plans of its passes.
    pub(super) fn new(at: (isize, isize), dims: Vec<Dim>, list: Vec<(isize, isize)>) -> Grid {
        Grid::planned(at, split(coalesced(dims)), list)
    }

    /// The grid of `dims` as they are given, with the plans of its passes.
    fn planned(at: (isize, isize), dims: Vec<Dim>, list: Vec<(isize, isize)>) -> Grid {
        let pairs = dims.iter().map(|d| d.radix).product();
        let empty = Plan {
            pass: Pass::Up,
            steps: Vec::new(),
            cells: Vec::new(),
            offsets: Vec::new(),
            outer: Vec::new(),
            work: 0.0,
            kernel: None,
        };
        let walk = Walk::new(&dims, &list);
        let mut grid = Grid {
            pairs,
            at,
            dims,
            list,
            walk,
            plans: [empty.clone(), empty.clone(), empty],
        };
        grid.plans = Pass::ALL.map(|pass| grid.plan(pass));
        grid
    }

    /// The grid of an implicit level of the library, off its description:
    /// the slots of its first pair, and its digits, the offset's first. The
    /// level's sides name nodes of its children.
    pub(super) fn implicit(level: &ImplicitLevel) -> Grid {
        let dims = level
            .digits()
            .iter()
            .map(|d| {
                let (radix, left, right) = (d.radix, d.left as isize, d.right as isize);
                if d.node == 0 {
                    Dim::pair(radix, left, right)
                } else {
                    Dim::node(radix, left, right, d.node as isize)
                }
            })
            .collect();
        let (l, r) = level.first();
        Grid::new((l as isize, r as isize), dims, Vec::new())
    }

    /// The pairs the grid holds.
    pub(super) fn pairs(&self) -> usize {
        self.pairs
    }

    /// The pairs of each node.
    pub(super) fn per_node(&self) -> usize {
        self.walk.offsets.len()
    }

    /// Whether the pass down walks the grid node by node: where the model
    /// charges the walk of the `contexts` nodes of nonzero context less
    /// than the plans of the passes into the two children.
    pub(super) fn walks(&self, contexts: usize) -> bool {
        (contexts * self.per_node()) as f64 * WALK
            < self.plans[Pass::Left.index()].work + self.plans[Pass::Right.index()].work
    }

    /// The pass down over the grid node by node into one child, the left
    /// where `left` says and the right otherwise, as the pass over pairs
    /// runs it: each node whose context in `column` is not zero, in order,
    /// adds its context times each of its pairs' count up on the other side
    /// (`other`) to that pair's slot in the child's column down (`down`),
    /// its pairs in order. The slots come off the description: each node's
    /// first pair by the node digits, in the order of their steps of the
    /// node index, and its pairs by the offsets.
    pub(super) fn walk_into<V: Value>(
        &self,
        left: bool,
        column: &[V],
        other: &[V],
        down: &mut [V],
    ) {
        if left {
            self.walk_side::<V, true>(column, other, down);
        } else {
            self.walk_side::<V, false>(column, other, down);
        }
    }

    /// [`Grid::walk_into`] into the left child where `LEFT` says, the
    /// right otherwise.
    fn walk_side<V: Value, const LEFT: bool>(&self, column: &[V], other: &[V], down: &mut [V]) {
        let mut firsts = Odometer::new(&self.walk.nodes, self.at);
        for &context in &column[..self.pairs / self.per_node()] {
            let (fl, fr) = firsts.next();
            if context.is_zero() {
                continue;
            }
            for &(ol, or) in &self.walk.offsets {
                let (a, b) = ((fl + ol) as usize, (fr + or) as usize);
                if LEFT {
                    down[a] += context * other[b];
                } else {
                    down[b] += context * other[a];
                }
            }
        }
    }

    /// Whether each pass runs through a vector kernel.
    #[cfg(test)]
    pub(super) fn kernels(&self) -> [bool; 3] {
        self.plans.each_ref().map(|p| p.kernel.is_some())
    }

    /// `level` as a grid, if it stores its pairs, at least `min_pairs` of
    /// them, and their sides name nodes of its children (`left`, `right`);
    /// `None` otherwise: for an implicit level, which [`Grid::implicit`]
    /// reads off its description, a marginal level or one over a marginal
    /// child.
    ///
    /// The digits come from a few nodes' first pairs; then each node's
    /// pairs are compared with the prediction as words, each pair's two
    /// slots one 64-bit word against its node's least slots plus an offset
    /// from them, which no carry crosses: every predicted slot is checked to
    /// lie in `0..2^31` first.
    pub(super) fn fit(
        level: &TddLevel,
        left: ChildDecoder,
        right: ChildDecoder,
        min_pairs: usize,
    ) -> Option<Grid> {
        if level.is_marginal() || left.is_marginal() || right.is_marginal() {
            return None;
        }
        let stored = level.stored()?;
        let nodes = level.slot_count();
        if nodes == 0 {
            return None;
        }
        let first = stored.of_idx(0);
        let k = first.len();
        if k == 0 || nodes.saturating_mul(k) < min_pairs {
            return None;
        }
        let pair = |p: &ChildPair| (slot(left, p.left), slot(right, p.right));
        let at = pair(&first[0]);
        let offsets: Vec<(isize, isize)> = first
            .iter()
            .map(|p| {
                let (a, b) = pair(p);
                (a - at.0, b - at.1)
            })
            .collect();
        let within = digits(k, |m| Some(offsets[m])).filter(|within| {
            let mut places = Odometer::new(within, (0, 0));
            offsets.iter().all(|&o| places.next() == o)
        });
        let across = digits(nodes, |i| stored.of_idx(i).first().map(pair))?;
        // The least offset on each side, and every predicted slot's range.
        let low = offsets
            .iter()
            .fold((0, 0), |(l, r), &(a, b)| (l.min(a), r.min(b)));
        let high = offsets
            .iter()
            .fold((0, 0), |(l, r), &(a, b)| (l.max(a), r.max(b)));
        let reach = |side: fn(&(isize, isize)) -> isize, sign: isize| -> isize {
            across
                .iter()
                .map(|d| (side(&d.step) * sign).max(0) * (d.radix as isize - 1))
                .sum()
        };
        let lows = (
            at.0 + low.0 - reach(|s| s.0, -1),
            at.1 + low.1 - reach(|s| s.1, -1),
        );
        let highs = (
            at.0 + high.0 + reach(|s| s.0, 1),
            at.1 + high.1 + reach(|s| s.1, 1),
        );
        if lows.0 < 0 || lows.1 < 0 || highs.0 >= 1 << 31 || highs.1 >= 1 << 31 {
            return None;
        }
        let word = |l: isize, r: isize| l as u64 | (r as u64) << 32;
        let pattern: Vec<u64> = offsets
            .iter()
            .map(|&(a, b)| word(a - low.0, b - low.1))
            .collect();
        // The fastest node digit steps the base word in the loop, signed
        // steps as wrapping words: every base lies in `0..2^31` on both
        // sides, so no step borrows across the halves.
        let (fast, slow) = across.split_first().map_or(
            (
                Digit {
                    radix: 1,
                    step: (0, 0),
                },
                &[][..],
            ),
            |(f, s)| (*f, s),
        );
        let step = (fast.step.0 as u64).wrapping_add((fast.step.1 as u64) << 32);
        let mut bases = Odometer::new(slow, (at.0 + low.0, at.1 + low.1));
        for run in stored.nodes().chunks_exact(fast.radix) {
            let (bl, br) = bases.next();
            let mut base = word(bl, br);
            for node in run {
                let pairs = stored.of(node);
                if pairs.len() != k || !same_words(pairs, base, &pattern) {
                    return None;
                }
                base = base.wrapping_add(step);
            }
        }
        let (mut dims, list) = match within {
            Some(within) => (
                within
                    .iter()
                    .map(|d| Dim::pair(d.radix, d.step.0, d.step.1))
                    .collect(),
                Vec::new(),
            ),
            None => {
                let extent = |side: fn(&(isize, isize)) -> isize| {
                    let (low, high) = offsets
                        .iter()
                        .map(side)
                        .fold((0, 0), |(l, h), x| (l.min(x), h.max(x)));
                    if high == low {
                        0
                    } else {
                        ((high - low) / (k as isize - 1)).max(1)
                    }
                };
                (
                    vec![Dim {
                        radix: k,
                        left: extent(|o| o.0),
                        right: extent(|o| o.1),
                        node: 0,
                        listed: true,
                    }],
                    offsets,
                )
            }
        };
        let mut period = 1usize;
        for d in &across {
            dims.push(Dim::node(d.radix, d.step.0, d.step.1, period as isize));
            period *= d.radix;
        }
        Some(Grid::new(at, dims, list))
    }

    /// The pass up over the grid: `out[i]` the count of node `i`, from the
    /// counts `l` and `r` of the children's slots, settled into `range`.
    /// Each node's sum starts from its first pair's product and adds the
    /// others in order, as the sum over its pairs does. Returns whether a
    /// vector kernel ran it.
    pub(super) fn up<V: Value>(&self, l: &[V], r: &[V], out: &mut [V], range: &mut Range) -> bool {
        self.run(&self.plans[Pass::Up.index()], out, l, r, false, range)
    }

    /// The pass down over the grid into one child, the left where `left`
    /// says and the right otherwise: each node's context in `column`,
    /// unless zero, times each of its pairs' count up on the other side
    /// (`other`), added to that pair's slot in the child's column down
    /// (`down`); each slot takes its terms in the order the pass over pairs
    /// adds them. A kernel of floats adds the product of a zero context
    /// too, which leaves a sum as it is where the count up is finite:
    /// `finite` says whether the other side's counts up are, and the
    /// scalar loops take the pass where they are not. The column is
    /// settled into `range`. Returns whether a vector kernel ran it.
    pub(super) fn down_into<V: Value>(
        &self,
        left: bool,
        column: &[V],
        other: &[V],
        down: &mut [V],
        finite: bool,
        range: &mut Range,
    ) -> bool {
        let pass = if left { Pass::Left } else { Pass::Right };
        self.run(
            &self.plans[pass.index()],
            down,
            column,
            other,
            !finite,
            range,
        )
    }

    /// Runs `plan` on the arrays: through its kernel where `V` is a float,
    /// one fits and the processor has it, unless `exact_skip`; through the
    /// scalar loops otherwise. A pass down (`plan.pass` not `Up`) skips the
    /// terms of a zero `x`. `out` is settled into `range`: each output at
    /// its last store where its slot is one to one, and once complete
    /// otherwise. Returns whether the kernel ran.
    fn run<V: Value>(
        &self,
        plan: &Plan,
        out: &mut [V],
        x: &[V],
        y: &[V],
        exact_skip: bool,
        range: &mut Range,
    ) -> bool {
        self.check(plan, out.len(), x.len(), y.len());
        let settles = self.one_to_one(plan.pass);
        let settle = settles.then_some(&mut *range);
        let mut kernel = false;
        if !exact_skip
            && let Some(k) = &plan.kernel
            && let (Some(o), Some(a), Some(b)) = (V::floats_mut(out), V::floats(x), V::floats(y))
            && k.ready()
        {
            // SAFETY: `check` put every slot the plan reads or writes inside
            // the arrays, and the kernel's lanes are places of the grid.
            unsafe { self.run_kernel(plan, k, o, a, b, settle) };
            kernel = true;
        } else if plan.pass == Pass::Up {
            self.run_scalar::<V, false>(plan, out, x, y, settle);
        } else {
            self.run_scalar::<V, true>(plan, out, x, y, settle);
        }
        if !settles {
            V::settle_all(out, range);
        }
        kernel
    }

    /// Panics unless every slot a plan's places name lies within arrays of
    /// these lengths: a fitted grid's pairs are the level's, and an unsafe
    /// kernel relies on it.
    fn check(&self, plan: &Plan, out: usize, x: usize, y: usize) {
        let mut low = Self::base(plan.pass, self.at);
        let mut high = low;
        for (d, s) in self.dims.iter().zip(&plan.steps) {
            if d.listed {
                for a in 0..3 {
                    let side = |&(l, r): &(isize, isize)| plan.pass.of(0, l, r)[a];
                    low[a] += self.list.iter().map(side).min().unwrap_or(0);
                    high[a] += self.list.iter().map(side).max().unwrap_or(0);
                }
            } else {
                for a in 0..3 {
                    let reach = s[a] * (d.radix as isize - 1);
                    low[a] += reach.min(0);
                    high[a] += reach.max(0);
                }
            }
        }
        let lens = [out, x, y];
        assert!(
            (0..3).all(|a| low[a] >= 0 && (high[a] as usize) < lens[a]),
            "a grid's slots lie outside its arrays"
        );
    }

    /// The slots of the first pair in `out`, `x` and `y`.
    fn base(pass: Pass, at: (isize, isize)) -> [isize; 3] {
        pass.of(0, at.0, at.1)
    }

    /// The scalar loops of `plan`: at each place of `outer`, the sums of
    /// the tile's outputs (`cells`) four at a time, then two and one, each
    /// over the summed places in stored order and held in registers, from
    /// its first product where the place starts it and from `out`
    /// otherwise; `SKIP`, the terms of a zero `x` left out. Each sum is
    /// settled into `settle`, where given, at the place that completes it.
    fn run_scalar<V: Value, const SKIP: bool>(
        &self,
        plan: &Plan,
        out: &mut [V],
        x: &[V],
        y: &[V],
        mut settle: Option<&mut Range>,
    ) {
        let (o, a, b) = (out.as_mut_ptr(), x.as_ptr(), y.as_ptr());
        let (fours, rest) = plan.cells.split_at(plan.cells.len() / 4 * 4);
        let (twos, one) = rest.split_at(rest.len() / 2 * 2);
        self.places(
            plan.pass,
            Self::base(plan.pass, self.at),
            &plan.steps,
            &plan.outer,
            |p, first, last| {
                let mut last = settle.as_deref_mut().filter(|_| last);
                // SAFETY: `check` put every slot the plan names inside the
                // arrays, and the cells are distinct slots of `out`, each held
                // in a register of its own.
                unsafe {
                    for cells in fours.as_chunks::<4>().0 {
                        sums::<V, 4, SKIP>(
                            cells,
                            &plan.offsets,
                            p,
                            first,
                            last.as_deref_mut(),
                            o,
                            a,
                            b,
                        );
                    }
                    if !twos.is_empty() {
                        sums::<V, 2, SKIP>(
                            twos,
                            &plan.offsets,
                            p,
                            first,
                            last.as_deref_mut(),
                            o,
                            a,
                            b,
                        );
                    }
                    if !one.is_empty() {
                        sums::<V, 1, SKIP>(one, &plan.offsets, p, first, last, o, a, b);
                    }
                }
            },
        );
    }

    /// The kernel of `plan` on floats, on copies of the sides it relays;
    /// each sum noted into `settle`, where given, at the place that
    /// completes it.
    ///
    /// # Safety
    ///
    /// Every slot `plan` names lies within the arrays ([`Grid::check`]),
    /// and the processor has what `k` needs ([`Kernel::ready`]).
    unsafe fn run_kernel(
        &self,
        plan: &Plan,
        k: &Kernel,
        out: &mut [f64],
        x: &[f64],
        y: &[f64],
        mut settle: Option<&mut Range>,
    ) {
        let stored = Self::base(plan.pass, self.at);
        let mut base = stored;
        let mut copies = [Vec::new(), Vec::new()];
        for (a, relay) in k.relays.iter().enumerate() {
            if let Some(relay) = relay {
                copies[a] = vec![0.0; relay.len()];
                // Up, `out` is written before it is read.
                if a == 1 || plan.pass != Pass::Up {
                    relay.gather(stored[a], if a == 0 { out } else { x }, &mut copies[a]);
                }
                base[a] = 0;
            }
        }
        let o = if k.relays[0].is_some() {
            copies[0].as_mut_ptr()
        } else {
            out.as_mut_ptr()
        };
        let a = if k.relays[1].is_some() {
            copies[1].as_ptr()
        } else {
            x.as_ptr()
        };
        let b = y.as_ptr();
        self.places(plan.pass, base, &k.steps, &plan.outer, |p, first, last| {
            // SAFETY: the caller's; a copy holds the slots of every place
            // from its first.
            unsafe {
                k.call(
                    o.offset(p[0]),
                    a.offset(p[1]),
                    b.offset(p[2]),
                    first,
                    settle.as_deref_mut().filter(|_| last),
                )
            };
        });
        if let Some(relay) = &k.relays[0] {
            relay.scatter(stored[0], &copies[0], out);
        }
    }

    /// What place `c` of dimension `d` adds to the slots of `out`, `x` and
    /// `y` under `steps`: for a listed dimension, the offset `c` of the
    /// grid's `list`; for any other, `c` of its steps.
    fn place(&self, pass: Pass, steps: &[[isize; 3]], d: usize, c: usize) -> [isize; 3] {
        if self.dims[d].listed {
            let (l, r) = self.list[c];
            pass.of(0, l, r)
        } else {
            steps[d].map(|s| s * c as isize)
        }
    }

    /// Calls `f` at every place of the dimensions `dims`, innermost first,
    /// as an odometer counts them from `base`, with the slots of the
    /// place's first pair in `out`, `x` and `y` (each dimension's `steps`),
    /// whether it starts the sums of its outputs, and whether it is the
    /// last place of their sums, every summed dimension among `dims` at its
    /// last place. A pass up starts them where every summed dimension is at
    /// zero; down, never, since a child's column is read from where it was
    /// zeroed.
    fn places(
        &self,
        pass: Pass,
        base: [isize; 3],
        steps: &[[isize; 3]],
        dims: &[usize],
        mut f: impl FnMut([isize; 3], bool, bool),
    ) {
        // Each dimension's radix, whether it is summed and moves, and what
        // stepping onto each place adds, the last entry the step back to
        // the first: affine dimensions add their steps, a listed one its
        // list.
        let wheels: Vec<(usize, bool, Vec<[isize; 3]>)> = dims
            .iter()
            .map(|&d| {
                let dim = &self.dims[d];
                let place = |c: usize| self.place(pass, steps, d, c);
                let delta = (1..=dim.radix).map(|c| {
                    let (to, from) = (place(c % dim.radix), place(c - 1));
                    [0, 1, 2].map(|a| to[a] - from[a])
                });
                (
                    dim.radix,
                    pass.of(dim.node, dim.left, dim.right)[0] == 0 && dim.radix > 1,
                    delta.collect(),
                )
            })
            .collect();
        let up = pass == Pass::Up;
        let mut count = vec![0usize; dims.len()];
        let mut at = base;
        // The summed dimensions among `dims` not at zero, and those not at
        // their last place.
        let mut raised = 0usize;
        let mut short = wheels.iter().filter(|w| w.1).count();
        loop {
            f(at, up && raised == 0, short == 0);
            let mut j = 0;
            loop {
                let Some((radix, summed, delta)) = wheels.get(j) else {
                    return;
                };
                let c = count[j];
                for (a, d) in at.iter_mut().zip(delta[c]) {
                    *a += d;
                }
                if c + 1 < *radix {
                    count[j] = c + 1;
                    raised += usize::from(*summed && c == 0);
                    short -= usize::from(*summed && c + 2 == *radix);
                    break;
                }
                count[j] = 0;
                raised -= usize::from(*summed);
                short += usize::from(*summed);
                j += 1;
            }
        }
    }
}

/// The sums of the `N` outputs `cells` at the place whose first pair's
/// slots are `p`: each over the summed places' `offsets` in order, in a
/// register, from the product at the first where `first` and from `out`
/// otherwise, then stored, settled into `last` where the place completes
/// them; `SKIP`, the terms of a zero `x` left out, which leaves the first
/// product of a sum up, whose pass does not skip.
///
/// # Safety
///
/// Every slot the cells name at every offset lies within the arrays, and
/// the cells name distinct slots of `out`.
#[allow(clippy::too_many_arguments)]
#[inline(always)]
unsafe fn sums<V: Value, const N: usize, const SKIP: bool>(
    cells: &[[isize; 3]],
    offsets: &[[isize; 2]],
    p: [isize; 3],
    first: bool,
    last: Option<&mut Range>,
    o: *mut V,
    x: *const V,
    y: *const V,
) {
    // SAFETY: the caller's.
    unsafe {
        let os: [*mut V; N] = std::array::from_fn(|i| o.offset(p[0] + cells[i][0]));
        let xs: [*const V; N] = std::array::from_fn(|i| x.offset(p[1] + cells[i][1]));
        let ys: [*const V; N] = std::array::from_fn(|i| y.offset(p[2] + cells[i][2]));
        let (mut acc, rest): ([V; N], &[[isize; 2]]) = if first {
            let [kx, ky] = offsets[0];
            (
                std::array::from_fn(|i| *xs[i].offset(kx) * *ys[i].offset(ky)),
                &offsets[1..],
            )
        } else {
            (std::array::from_fn(|i| *os[i]), offsets)
        };
        for &[kx, ky] in rest {
            for i in 0..N {
                let u = *xs[i].offset(kx);
                if SKIP && u.is_zero() {
                    continue;
                }
                acc[i] += u * *ys[i].offset(ky);
            }
        }
        match last {
            Some(range) => {
                for i in 0..N {
                    *os[i] = acc[i].settle(range);
                }
            }
            None => {
                for i in 0..N {
                    *os[i] = acc[i];
                }
            }
        }
    }
}

/// `dims` with each run of adjacent dimensions that continue each other
/// read as one: a dimension whose steps on every side are those of the one
/// before times its radix. The pairs and their order stay.
fn coalesced(dims: Vec<Dim>) -> Vec<Dim> {
    let mut out: Vec<Dim> = Vec::with_capacity(dims.len());
    for d in dims {
        if let Some(last) = out.last_mut()
            && !last.listed
            && !d.listed
            && [
                (d.left, last.left),
                (d.right, last.right),
                (d.node, last.node),
            ]
            .iter()
            .all(|&(a, b)| a == b * last.radix as isize)
        {
            last.radix *= d.radix;
            continue;
        }
        out.push(d);
    }
    out
}

/// `dims` with each affine dimension of more places than a kernel's lanes
/// ([`LANES`]) read as two where its places allow: its first places, as
/// many as the largest divisor of its radix the lanes hold, and the repeats
/// of those, the second split again. The pairs and their order stay; a plan
/// may then run a dimension's lanes inside and their repeats anywhere
/// outside, and a sum over a large dimension in parts.
fn split(dims: Vec<Dim>) -> Vec<Dim> {
    let mut out = Vec::with_capacity(dims.len());
    for mut d in dims {
        while !d.listed && d.radix > *LANES.end() {
            let Some(a) = LANES.rev().find(|&a| d.radix.is_multiple_of(a)) else {
                break;
            };
            out.push(Dim { radix: a, ..d });
            let s = a as isize;
            d = Dim {
                radix: d.radix / a,
                left: d.left * s,
                right: d.right * s,
                node: d.node * s,
                listed: false,
            };
        }
        out.push(d);
    }
    out
}

/// Whether each pair's two slots, as one word, are `base` plus the
/// pattern's word at its place; the slots are a structural child's node
/// indices, the pair's words themselves. One loop with no early exit, which
/// the compiler runs in vector registers.
fn same_words(pairs: &[ChildPair], base: u64, pattern: &[u64]) -> bool {
    let pattern = &pattern[..pairs.len()];
    let mut diff = 0u64;
    for i in 0..pairs.len() {
        diff |= pair_word(&pairs[i]) ^ base.wrapping_add(pattern[i]);
    }
    diff == 0
}

/// A pair's two slots as one word, the left in the low half.
#[inline(always)]
fn pair_word(p: &ChildPair) -> u64 {
    u64::from(p.left.raw()) | u64::from(p.right.raw()) << 32
}

/// One dimension as the plan model reads it: its radix and what a step
/// adds to the slots of the three arrays; a listed one, its extent.
#[derive(Clone, Copy, Debug)]
struct Axis {
    radix: usize,
    steps: [isize; 3],
}

/// One way to run a contraction, as the plan search weighs it: the lanes,
/// the tile's dimension, the summed dimensions that run inside, the first
/// of the pass's summed dimensions in stored order. It borrows its lanes
/// and its summed dimensions from the search, which prices hundreds.
#[derive(Clone, Copy, Debug)]
struct Candidate<'a> {
    lanes: Option<&'a LaneSet>,
    tile: Option<usize>,
    summed: &'a [usize],
}

impl Candidate<'_> {
    /// The dimensions inside the dimensions an order runs outside them:
    /// the lanes', the tile's, the summed ones'.
    fn prefix(&self) -> Vec<usize> {
        let mut prefix = self.lanes.map(|l| l.dims().to_vec()).unwrap_or_default();
        prefix.extend(self.tile);
        prefix.extend(self.summed);
        prefix
    }
}

/// The most candidates whose orders the plan search completes, of those
/// whose kernels the model charges least.
const MOST_ORDERED: usize = 16;

impl Grid {
    /// What a step of each dimension adds to `out`, `x` and `y` in `pass`;
    /// for a listed one, the list's extent per place, for the model.
    fn axes(&self, pass: Pass) -> Vec<Axis> {
        self.dims
            .iter()
            .map(|d| Axis {
                radix: d.radix,
                steps: pass.of(d.node, d.left, d.right),
            })
            .collect()
    }

    /// The dimensions `pass` sums over, in stored order: those that do not
    /// move `out`.
    fn summed(&self, pass: Pass) -> Vec<usize> {
        self.axes(pass)
            .iter()
            .enumerate()
            .filter(|(_, a)| a.steps[0] == 0)
            .map(|(d, _)| d)
            .collect()
    }

    /// Whether a kernel of `pass` may relay `out`: down, where each slot of
    /// the child's column the pass writes is one place's and no listed
    /// dimension moves it.
    fn relays_out(&self, pass: Pass) -> bool {
        pass != Pass::Up
            && self.one_to_one(pass)
            && self
                .dims
                .iter()
                .zip(self.axes(pass))
                .all(|(d, a)| !d.listed || a.steps[0] == 0)
    }

    /// Whether `out`'s slot in `pass` is a one-to-one function of the
    /// dimensions it depends on: always up, where it is the node.
    fn one_to_one(&self, pass: Pass) -> bool {
        if pass == Pass::Up {
            return true;
        }
        let side = |&(l, r): &(isize, isize)| pass.of(0, l, r)[0];
        one_to_one(self.dims.iter().filter_map(|d| {
            if !d.listed {
                let s = pass.of(d.node, d.left, d.right)[0].unsigned_abs();
                return (s != 0).then_some((s, (d.radix - 1) * s));
            }
            let mut v: Vec<isize> = self.list.iter().map(side).collect();
            v.sort_unstable();
            let (low, high) = (v[0], v[v.len() - 1]);
            let gap = v
                .windows(2)
                .map(|w| (w[1] - w[0]).unsigned_abs())
                .min()
                .unwrap_or(0);
            (high != low).then_some((gap, (high - low).unsigned_abs()))
        }))
    }

    /// The ways `pass` may run, of the lane sets `lanes` and the summed
    /// dimensions `summed` ([`Grid::summed`]). Where every output's slot
    /// is one to one, the summed dimensions keep their stored order, those
    /// inside a tile the first of them, and the others go anywhere; a tile
    /// of the scalar loops is one output dimension, whose sums they hold
    /// together.
    /// Otherwise a tile holds the dimensions stored first and the others
    /// keep their stored order: a slot's terms come from one place of the
    /// tile's outputs at each place outside it, in the summed places'
    /// order, and from the places outside it in theirs.
    fn candidates<'a>(
        &self,
        pass: Pass,
        steps: &[[isize; 3]],
        lanes: &'a [LaneSet],
        summed: &'a [usize],
    ) -> Vec<Candidate<'a>> {
        let n = self.dims.len();
        let output = |d: usize| steps[d][0] != 0 && !self.dims[d].listed;
        let places = |dims: &[usize]| dims.iter().map(|&d| self.dims[d].radix).product::<usize>();
        let mut out = Vec::new();
        if self.one_to_one(pass) {
            let most = (0..=summed.len())
                .take_while(|&t| places(&summed[..t]) <= MOST_SUMMED)
                .last()
                .unwrap_or(0);
            for lane in lanes.iter().map(Some).chain([None]) {
                let taken = lane.map_or(&[][..], LaneSet::dims);
                for tile in [None].into_iter().chain(
                    (0..n)
                        .filter(|&d| output(d) && !taken.contains(&d))
                        .map(Some),
                ) {
                    for t in 0..=most {
                        out.push(Candidate {
                            lanes: lane,
                            tile,
                            summed: &summed[..t],
                        });
                    }
                }
            }
            return out;
        }
        for m in 0..=n {
            let inside = &summed[..summed.partition_point(|&d| d < m)];
            if places(inside) > MOST_SUMMED {
                break;
            }
            let outputs: Vec<usize> = (0..m).filter(|d| !summed.contains(d)).collect();
            if outputs.is_empty() {
                out.push(Candidate {
                    lanes: None,
                    tile: None,
                    summed: inside,
                });
            }
            for l in lanes {
                let dims = l.dims();
                if !dims.iter().all(|d| outputs.contains(d)) {
                    continue;
                }
                let rest: Vec<usize> = outputs
                    .iter()
                    .copied()
                    .filter(|d| !dims.contains(d))
                    .collect();
                let tile = match rest[..] {
                    [] => None,
                    [j] if output(j) => {
                        let s = steps[j][0].unsigned_abs();
                        let lanes = places(dims);
                        if !one_to_one(
                            [(1, lanes - 1), (s, (self.dims[j].radix - 1) * s)].into_iter(),
                        ) {
                            continue;
                        }
                        Some(j)
                    }
                    _ => continue,
                };
                out.push(Candidate {
                    lanes: Some(l),
                    tile,
                    summed: inside,
                });
            }
        }
        out
    }

    /// The plan of `pass` (module doc): of the [`MOST_ORDERED`] candidates
    /// whose kernels the model charges least, the one whose order the
    /// model charges least with its misses ([`Grid::misses`]), in the
    /// order [`complete`] finds for it with its lookahead.
    fn plan(&self, pass: Pass) -> Plan {
        let steps = self.steps(pass);
        let summed = self.summed(pass);
        let lanes = LaneSet::options(&self.dims, &steps, pass, self.relays_out(pass));
        let mut candidates: Vec<(f64, usize, Candidate)> = self
            .candidates(pass, &steps, &lanes, &summed)
            .into_iter()
            .enumerate()
            .map(|(i, c)| (self.work(pass, &steps, c), i, c))
            .collect();
        // The least work first, ties in the candidates' order.
        let by_work = |a: &(f64, usize, Candidate), b: &(f64, usize, Candidate)| {
            a.0.total_cmp(&b.0).then(a.1.cmp(&b.1))
        };
        if candidates.len() > MOST_ORDERED {
            candidates.select_nth_unstable_by(MOST_ORDERED, by_work);
            candidates.truncate(MOST_ORDERED);
        }
        candidates.sort_unstable_by(by_work);
        let (_, c) = candidates
            .into_iter()
            .map(|(work, _, c)| (work + MISS * self.misses(pass, c), c))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .expect("the scalar loops are a candidate");
        let order = self.order(pass, c, true);
        self.assemble(pass, steps, c, &order)
    }

    /// What a step of each dimension adds to `out`, `x` and `y` in `pass`,
    /// zero for a listed one, whose places add the list.
    fn steps(&self, pass: Pass) -> Vec<[isize; 3]> {
        self.dims
            .iter()
            .map(|d| {
                if d.listed {
                    [0; 3]
                } else {
                    pass.of(d.node, d.left, d.right)
                }
            })
            .collect()
    }

    /// The model of `pass` for candidate `c`: its axes, with the sides its
    /// lanes relay relaid.
    fn model(&self, pass: Pass, c: Candidate) -> Model {
        let mut axes = self.axes(pass);
        if let Some(l) = c.lanes {
            let steps = l.steps(&self.dims, &self.steps(pass), c.tile);
            for a in (0..2).filter(|&a| l.relays()[a]) {
                for (axis, s) in axes.iter_mut().zip(&steps).filter(|(_, s)| s[a] != 0) {
                    axis.steps[a] = s[a];
                }
            }
        }
        Model::new(axes)
    }

    /// The order of the dimensions candidate `c` runs `pass` in, innermost
    /// first: its tile's, then the others, completed for few misses
    /// ([`complete`]) where every slot is one to one and the arrays
    /// outgrow the budget ([`BUDGET`]); in stored order otherwise, which
    /// is legal for every candidate and, within the budget, misses what
    /// every order does.
    fn order(&self, pass: Pass, c: Candidate, lookahead: bool) -> Vec<usize> {
        let n = self.dims.len();
        let prefix = c.prefix();
        if self.one_to_one(pass) {
            let model = self.model(pass, c);
            if model.lines(|_| true) > BUDGET {
                return complete(&model, &prefix, lookahead);
            }
        }
        prefix
            .iter()
            .copied()
            .chain((0..n).filter(|d| !prefix.contains(d)))
            .collect()
    }

    /// The lines the model counts candidate `c`'s order of `pass` as
    /// missing ([`Grid::order`], without its lookahead): within the budget,
    /// the lines of the whole arrays, whatever the order.
    fn misses(&self, pass: Pass, c: Candidate) -> f64 {
        let model = self.model(pass, c);
        let all = model.lines(|_| true);
        if all <= BUDGET {
            return all;
        }
        let prefix = c.prefix();
        if self.one_to_one(pass) {
            return model.priced(model.start(&prefix));
        }
        let mut order = model.start(&prefix);
        for d in (0..self.dims.len()).filter(|d| !prefix.contains(d)) {
            model.push(&mut order, d);
        }
        order.misses
    }

    /// The cycles the model charges candidate `c`'s arithmetic in `pass`:
    /// its kernel's ([`LaneSet::cycles`]) and its copies of the sides it
    /// relays, six a slot for each way, or the scalar loops'. Those take
    /// a summed place in an add's latency of four cycles for each sum
    /// they hold, and no fewer than a cycle and a quarter a pair for its
    /// loads, and add each sum's load and store and a call.
    fn work(&self, pass: Pass, steps: &[[isize; 3]], c: Candidate) -> f64 {
        let places = |dims: &[usize]| dims.iter().map(|&d| self.dims[d].radix).product::<usize>();
        let inside = (c.lanes.map_or(1, |l| places(l.dims()))
            * c.tile.map_or(1, |j| self.dims[j].radix)
            * places(c.summed)) as f64;
        let outer = self.pairs as f64 / inside;
        let summed = places(c.summed) as f64;
        match c.lanes {
            Some(l) => {
                let len = |a: usize| {
                    self.dims
                        .iter()
                        .zip(steps)
                        .filter(|(_, s)| s[a] != 0)
                        .map(|(d, _)| d.radix)
                        .product::<usize>() as f64
                };
                // Up, the node side is written back; down, read, and the
                // child's column read and written back.
                let ways = if pass == Pass::Up {
                    [1.0, 0.0]
                } else {
                    [2.0, 1.0]
                };
                let relay: f64 = (0..2)
                    .filter(|&a| l.relays()[a])
                    .map(|a| 6.0 * ways[a] * len(a))
                    .sum();
                l.cycles(&self.dims, steps, c.tile, summed, outer) + relay
            }
            None => {
                let cells = c.tile.map_or(1, |j| self.dims[j].radix);
                let block = |n: usize| summed * 4f64.max(1.25 * n as f64) + 2.0 * n as f64 + 6.0;
                let tiles = (cells / 4) as f64 * block(4)
                    + (cells % 4 / 2) as f64 * block(2)
                    + (cells % 2) as f64 * block(1);
                outer * (tiles + 8.0)
            }
        }
    }

    /// The plan of `pass` for candidate `c` in `order`, its kernel built
    /// where it has lanes.
    fn assemble(&self, pass: Pass, steps: Vec<[isize; 3]>, c: Candidate, order: &[usize]) -> Plan {
        let work = self.work(pass, &steps, c);
        let mut inner: Vec<usize> = c.lanes.map(|l| l.dims().to_vec()).unwrap_or_default();
        inner.extend(c.tile);
        let mut cells = vec![[0isize; 3]];
        for &d in &inner {
            let step = steps[d];
            cells = (0..self.dims[d].radix as isize)
                .flat_map(|k| {
                    cells
                        .iter()
                        .map(move |p| [0, 1, 2].map(|a| p[a] + k * step[a]))
                })
                .collect();
        }
        let outer: Vec<usize> = order[inner.len() + c.summed.len()..].to_vec();
        let offsets = self.summed_offsets(pass, &steps, c.summed);
        let kernel = c.lanes.and_then(|l| {
            Kernel::new(l, c.tile, &self.dims, &steps, |s| {
                self.summed_offsets(pass, s, c.summed)
            })
        });
        Plan {
            pass,
            steps,
            cells,
            offsets,
            outer,
            work,
            kernel,
        }
    }

    /// The offsets the places of the summed dimensions `summed` add to `x`
    /// and `y` under `steps`, in stored order, the first fastest.
    fn summed_offsets(
        &self,
        pass: Pass,
        steps: &[[isize; 3]],
        summed: &[usize],
    ) -> Vec<[isize; 2]> {
        let mut out = vec![[0isize; 2]];
        for &d in summed {
            let place = |c: usize| self.place(pass, steps, d, c);
            out = (0..self.dims[d].radix)
                .flat_map(|c| {
                    out.iter()
                        .map(move |o| [o[0] + place(c)[1], o[1] + place(c)[2]])
                })
                .collect();
        }
        out
    }

    /// Every way the passes may run, for the tests: the plan, and
    /// each candidate of each pass in each legal order of the dimensions
    /// outside its tile, or a completed one where they are more than four,
    /// through its kernel and through the scalar loops; the passes' ways
    /// paired in turn.
    #[cfg(test)]
    pub(super) fn variants(&self) -> Vec<Grid> {
        let ways: Vec<Vec<Plan>> = Pass::ALL
            .iter()
            .map(|&pass| {
                let steps = self.steps(pass);
                let (sums, lanes) = (
                    self.summed(pass),
                    LaneSet::options(&self.dims, &steps, pass, self.relays_out(pass)),
                );
                let mut plans = vec![self.plan(pass)];
                for c in self.candidates(pass, &steps, &lanes, &sums) {
                    let prefix = c.prefix();
                    let rest: Vec<usize> = (0..self.dims.len())
                        .filter(|d| !prefix.contains(d))
                        .collect();
                    let axes = self.axes(pass);
                    let summed: Vec<usize> = rest
                        .iter()
                        .copied()
                        .filter(|&d| axes[d].steps[0] == 0)
                        .collect();
                    let orders: Vec<Vec<usize>> = if !self.one_to_one(pass) {
                        vec![rest]
                    } else if rest.len() <= 4 {
                        permutations(&rest)
                            .into_iter()
                            .filter(|o| o.iter().filter(|d| summed.contains(d)).is_sorted())
                            .collect()
                    } else {
                        vec![self.order(pass, c, false)[prefix.len()..].to_vec()]
                    };
                    for o in orders {
                        let order: Vec<usize> = prefix.iter().copied().chain(o).collect();
                        let plan = self.assemble(pass, steps.clone(), c, &order);
                        if plan.kernel.is_some() {
                            plans.push(Plan {
                                kernel: None,
                                ..plan.clone()
                            });
                        }
                        plans.push(plan);
                    }
                }
                plans
            })
            .collect();
        let most = ways.iter().map(Vec::len).max().unwrap_or(0);
        (0..most)
            .map(|i| Grid {
                plans: [0, 1, 2].map(|p| ways[p][i % ways[p].len()].clone()),
                ..self.clone()
            })
            .collect()
    }
}

/// Every order of `dims`.
#[cfg(test)]
fn permutations(dims: &[usize]) -> Vec<Vec<usize>> {
    if dims.is_empty() {
        return vec![Vec::new()];
    }
    (0..dims.len())
        .flat_map(|i| {
            let mut rest = dims.to_vec();
            let d = rest.remove(i);
            permutations(&rest).into_iter().map(move |mut p| {
                p.insert(0, d);
                p
            })
        })
        .collect()
}

/// The plan model over a pass's dimensions as a candidate reads them
/// ([`Grid::model`]): with each array's dimensions that move it, by their
/// strides, for [`Model::lines`].
struct Model {
    axes: Vec<Axis>,
    by_stride: [Vec<usize>; 3],
}

/// An order of a pass's dimensions as the search grows it, innermost
/// first: which dimensions it holds, the places of those, and the lines
/// the model counts it as missing so far ([`Model::push`]).
#[derive(Clone)]
struct Partial {
    order: Vec<usize>,
    inside: Vec<bool>,
    places: f64,
    misses: f64,
}

impl Model {
    fn new(axes: Vec<Axis>) -> Model {
        let by_stride = std::array::from_fn(|a| {
            let mut dims: Vec<usize> = (0..axes.len()).filter(|&d| axes[d].steps[a] != 0).collect();
            dims.sort_by_key(|&d| axes[d].steps[a].unsigned_abs());
            dims
        });
        Model { axes, by_stride }
    }

    /// The cache lines a run over the dimensions `inside` touches in the
    /// three arrays of floats, about: in each, the strides taken from the
    /// smallest, a stride within a line of the extent so far extends it,
    /// and a larger one repeats it. It depends on the dimensions only, not
    /// on their order.
    fn lines(&self, inside: impl Fn(usize) -> bool) -> f64 {
        let per_line = LINE / std::mem::size_of::<f64>();
        (0..3)
            .map(|a| {
                let (mut extent, mut repeats) = (1usize, 1f64);
                for &d in self.by_stride[a].iter().filter(|&&d| inside(d)) {
                    let (radix, stride) =
                        (self.axes[d].radix, self.axes[d].steps[a].unsigned_abs());
                    if stride <= extent + per_line {
                        extent += (radix - 1) * stride;
                    } else {
                        repeats *= radix as f64;
                    }
                }
                repeats * ((extent - 1) as f64 / per_line as f64 + 1.0)
            })
            .sum()
    }

    /// The order `prefix`, grown from none.
    fn start(&self, prefix: &[usize]) -> Partial {
        let mut p = Partial {
            order: Vec::with_capacity(self.axes.len()),
            inside: vec![false; self.axes.len()],
            places: 1.0,
            misses: 0.0,
        };
        for &d in prefix {
            self.push(&mut p, d);
        }
        p
    }

    /// Places dimension `d` outside the order `p`. The order misses the
    /// lines its dimensions touch once while those fit in [`BUDGET`]; the
    /// dimensions placed after they first outgrow it each multiply its
    /// misses by their radix, the run inside missing all of its lines
    /// again on each of their places.
    fn push(&self, p: &mut Partial, d: usize) {
        let radix = self.axes[d].radix as f64;
        p.order.push(d);
        p.inside[d] = true;
        p.places *= radix;
        p.misses = if p.misses > BUDGET {
            p.misses * radix
        } else {
            self.lines(|j| p.inside[j])
        };
    }

    /// The dimensions that may come next after the order `p`: any output
    /// dimension, and the first summed one in stored order not placed, so
    /// that the summed dimensions keep their stored order. `p` holds the
    /// first summed dimensions, if any.
    fn next<'a>(&'a self, p: &'a Partial) -> impl Iterator<Item = usize> + 'a {
        let summed = |d: usize| self.axes[d].steps[0] == 0;
        let first = (0..self.axes.len()).find(|&d| summed(d) && !p.inside[d]);
        (0..self.axes.len()).filter(move |&d| !p.inside[d] && (!summed(d) || Some(d) == first))
    }

    /// Of the dimensions `next`, the one with which the order `p` touches
    /// the fewest cache lines per pair ([`Model::lines`]), preferring those
    /// whose lines stay within [`BUDGET`]; ties go to the dimension stored
    /// first.
    fn grown(&self, p: &Partial, next: impl Iterator<Item = usize>) -> usize {
        let cost = |d: usize| -> (bool, f64) {
            let lines = self.lines(|j| p.inside[j] || j == d);
            (
                lines > BUDGET,
                lines / (p.places * self.axes[d].radix as f64),
            )
        };
        next.map(|d| (cost(d), d))
            .min_by(|(a, _), (b, _)| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)))
            .map(|(_, d)| d)
            .expect("a dimension completes a legal order")
    }

    /// The misses of the order the greedy step ([`Model::grown`])
    /// completes from `p`. Once they are past [`BUDGET`], every dimension
    /// placed after multiplies them by its radix, in whatever order: the
    /// greedy step runs only until then.
    fn priced(&self, mut p: Partial) -> f64 {
        while p.misses <= BUDGET && p.order.len() < self.axes.len() {
            let d = self.grown(&p, self.next(&p));
            self.push(&mut p, d);
        }
        (0..self.axes.len())
            .filter(|&d| !p.inside[d])
            .fold(p.misses, |m, d| m * self.axes[d].radix as f64)
    }
}

/// The dimensions in the order a pass runs them, innermost first: `prefix`,
/// then the others, the summed ones in stored order ([`Model::next`]). Each
/// place takes the dimension with which the dimensions placed so far touch
/// the fewest lines per pair ([`Model::grown`]); with `lookahead`, of the
/// dimensions with which the order the greedy step completes misses the
/// fewest lines ([`Model::priced`]) and those within 1% of the fewest, the
/// one the greedy step takes. The greedy step alone weighs the dimensions
/// placed so far, and can put inside a dimension that spoils the reuse a
/// dimension outside would have had. Once the order placed misses more
/// than [`BUDGET`], every completion misses the same: the lookahead stops.
fn complete(model: &Model, prefix: &[usize], lookahead: bool) -> Vec<usize> {
    let n = model.axes.len();
    let mut out = model.start(prefix);
    while out.order.len() < n {
        let d = if lookahead && out.misses <= BUDGET {
            let scored: Vec<(f64, usize)> = model
                .next(&out)
                .map(|d| {
                    let mut full = out.clone();
                    model.push(&mut full, d);
                    (model.priced(full), d)
                })
                .collect();
            let least = scored.iter().map(|s| s.0).fold(f64::INFINITY, f64::min);
            model.grown(
                &out,
                scored.iter().filter(|s| s.0 <= least * 1.01).map(|s| s.1),
            )
        } else {
            model.grown(&out, model.next(&out))
        };
        model.push(&mut out, d);
    }
    out.order
}

/// Whether a slot is a one-to-one function of the dimensions it depends
/// on, each given by the least gap between two of its places' values and
/// the span of them: the gaps, from the least, each beyond the spans of
/// the dimensions before it. A dimension two of whose places share a value
/// has no gap, and fails.
fn one_to_one(dims: impl Iterator<Item = (usize, usize)>) -> bool {
    let mut dims: Vec<(usize, usize)> = dims.collect();
    dims.sort_unstable();
    let mut reach = 0usize;
    for (gap, span) in dims {
        if gap <= reach {
            return false;
        }
        reach += span;
    }
    true
}

/// The slot a side names in a structural child.
#[inline]
fn slot(d: ChildDecoder, s: tididi::diagram::EncodedChildRef) -> isize {
    match d.child(s) {
        ChildRef::Node(n) => n.idx() as isize,
        ChildRef::Value(_) => unreachable!("a structural child's side names a node"),
    }
}

/// The digits of `base` over `0..n`, if it is affine in some mixed radix:
/// each digit's step read off the first place past the digits before it,
/// its radix the longest run of that step, cut to a divisor of what the
/// digits before it leave. `None` where a place has no value or no radix
/// fits; the caller checks every place against the digits found.
fn digits(n: usize, base: impl Fn(usize) -> Option<(isize, isize)>) -> Option<Vec<Digit>> {
    let v0 = base(0)?;
    let mut digits = Vec::new();
    let mut period = 1usize;
    while period < n {
        if !n.is_multiple_of(period) {
            return None;
        }
        let rest = n / period;
        let v1 = base(period)?;
        let step = (v1.0 - v0.0, v1.1 - v0.1);
        let mut run = 2usize;
        while run < rest
            && base(period * run)? == (v0.0 + run as isize * step.0, v0.1 + run as isize * step.1)
        {
            run += 1;
        }
        let mut radix = run.min(rest);
        while radix > 1 && !rest.is_multiple_of(radix) {
            radix -= 1;
        }
        if radix < 2 {
            return None;
        }
        digits.push(Digit { radix, step });
        period *= radix;
    }
    Some(digits)
}

#[cfg(test)]
mod tests {
    use super::super::Count;
    use super::*;

    /// The passes over the pairs of a grid `dims` from `at`, as the passes
    /// over a level's pairs run them, node by node and each node's pairs in
    /// order: the counts up from `l` and `r`, and the columns down into the
    /// two children from `column`, a zero context's pairs left out.
    fn by_pairs<V: Value>(
        dims: &[Dim],
        at: (isize, isize),
        l: &[V],
        r: &[V],
        column: &[V],
    ) -> [Vec<V>; 3] {
        let mut pairs: Vec<(isize, usize, isize, isize)> = vec![(0, 0, at.0, at.1)];
        let mut period = 1;
        for d in dims {
            let within = if d.node == 0 { period } else { 0 };
            pairs = (0..d.radix as isize)
                .flat_map(|c| {
                    pairs.iter().map(move |&(n, k, a, b)| {
                        (
                            n + c * d.node,
                            k + c as usize * within,
                            a + c * d.left,
                            b + c * d.right,
                        )
                    })
                })
                .collect();
            if d.node == 0 {
                period *= d.radix;
            }
        }
        pairs.sort_by_key(|&(n, k, ..)| (n, k));
        let nodes = pairs.last().map_or(0, |p| p.0 as usize + 1);
        let (mut up, mut dl, mut dr) = (
            vec![V::ZERO; nodes],
            vec![V::ZERO; l.len()],
            vec![V::ZERO; r.len()],
        );
        for &(n, k, a, b) in &pairs {
            let (n, a, b) = (n as usize, a as usize, b as usize);
            up[n] = if k == 0 {
                l[a] * r[b]
            } else {
                up[n] + l[a] * r[b]
            };
            if !column[n].is_zero() {
                dl[a] += column[n] * r[b];
                dr[b] += column[n] * l[a];
            }
        }
        [up, dl, dr]
    }

    /// Every way each pass may run over shapes of the census's levels, cut
    /// down (DBN_13's, Grids_16's and CSP_11's), and the walk down node by
    /// node, give the bits of the passes over pairs, in floats and in
    /// `Count`s, and the plans settle their outputs as settling them after
    /// does: the range noted, bit for bit, and each `Count` normalized, its
    /// sum out of the band. Every pass has a kernel among its ways, and
    /// DBN_13's passes down relay their output.
    #[test]
    fn census_shapes_run_through_kernels_to_the_bit() {
        let shapes: [((isize, isize), Vec<Dim>); 3] = [
            (
                (1, 0),
                vec![
                    Dim::pair(2, -1, 1),
                    Dim::node(4, 2, 0, 1),
                    Dim::node(4, 0, 2, 4),
                    Dim::node(2, 8, 0, 16),
                    Dim::node(16, 0, 8, 32),
                ],
            ),
            (
                (4, 0),
                vec![
                    Dim::pair(2, -4, 512),
                    Dim::node(2, 1, 0, 1),
                    Dim::node(16, 0, 1, 2),
                    Dim::node(16, 0, 16, 32),
                    Dim::node(2, 2, 256, 512),
                ],
            ),
            (
                (0, 31),
                vec![
                    Dim::pair(4, 1, -8),
                    Dim::node(2, 4, 0, 1),
                    Dim::node(2, 64, -1, 2),
                    Dim::node(4, 8, 32, 4),
                    Dim::node(4, 256, -2, 16),
                ],
            ),
        ];
        let value = |i: usize, salt: usize| {
            ((i * 2_654_435_761 + salt * 40_503) % 1009) as f64 / 1009.0 + 0.003
        };
        for (s, (at, dims)) in shapes.into_iter().enumerate() {
            let grid = Grid::new(at, dims.clone(), Vec::new());
            let reach = |side: fn(&Dim) -> isize, base: isize| {
                base + dims
                    .iter()
                    .map(|d| side(d).max(0) * (d.radix as isize - 1))
                    .sum::<isize>()
                    + 1
            };
            let (nl, nr, nodes) = (
                reach(|d| d.left, at.0) as usize,
                reach(|d| d.right, at.1) as usize,
                reach(|d| d.node, 0) as usize,
            );
            let l: Vec<f64> = (0..nl).map(|i| value(i, 1)).collect();
            let r: Vec<f64> = (0..nr).map(|i| value(i, 2)).collect();
            let column: Vec<f64> = (0..nodes)
                .map(|i| if i % 7 == 3 { 0.0 } else { value(i, 3) })
                .collect();
            let want = by_pairs(&dims, at, &l, &r, &column);
            let noted = |xs: &[f64]| {
                let mut range = Range::NEW;
                <f64 as Value>::settle_all(&mut xs.to_vec(), &mut range);
                (range.low.to_bits(), range.high.to_bits())
            };
            let want_ranges = want.each_ref().map(|w| noted(w));
            let counts = |xs: &[f64]| {
                xs.iter()
                    .map(|&x| Count::new(x * 2f64.powi(200)))
                    .collect::<Vec<Count>>()
            };
            let mut unnoted = Range::NEW;
            let want_counts =
                by_pairs(&dims, at, &counts(&l), &counts(&r), &counts(&column)).map(|mut w| {
                    Count::settle_all(&mut w, &mut unnoted);
                    w
                });
            let mut walked = [vec![0.0; nl], vec![0.0; nr]];
            let [dl, dr] = &mut walked;
            grid.walk_into(true, &column, &r, dl);
            grid.walk_into(false, &column, &l, dr);
            assert!(
                walked
                    .iter()
                    .zip(&want[1..])
                    .all(|(g, w)| g.iter().zip(w).all(|(a, b)| a.to_bits() == b.to_bits())),
                "shape {s} walked in floats"
            );
            let mut walked = [vec![Count::ZERO; nl], vec![Count::ZERO; nr]];
            let [dl, dr] = &mut walked;
            grid.walk_into(true, &counts(&column), &counts(&r), dl);
            grid.walk_into(false, &counts(&column), &counts(&l), dr);
            walked
                .iter_mut()
                .for_each(|w| Count::settle_all(w, &mut unnoted));
            assert!(
                walked.iter().zip(&want_counts[1..]).all(|(g, w)| g
                    .iter()
                    .zip(w)
                    .all(|(a, b)| format!("{a:?}") == format!("{b:?}"))),
                "shape {s} walked in counts"
            );
            let ways = grid.variants();
            let mut kernels = [false; 3];
            let mut relays_out = [false; 3];
            for g in &ways {
                let mut got = [vec![0.0; nodes], vec![0.0; nl], vec![0.0; nr]];
                let mut ranges = [Range::NEW; 3];
                let [up, dl, dr] = &mut got;
                let [ru, rl, rr] = &mut ranges;
                g.up(&l, &r, up, ru);
                g.down_into(true, &column, &r, dl, true, rl);
                g.down_into(false, &column, &l, dr, true, rr);
                for (p, (g, w)) in got.iter().zip(&want).enumerate() {
                    assert!(
                        g.iter().zip(w).all(|(a, b)| a.to_bits() == b.to_bits()),
                        "shape {s} pass {p} in floats"
                    );
                    assert_eq!(
                        (ranges[p].low.to_bits(), ranges[p].high.to_bits()),
                        want_ranges[p],
                        "shape {s} pass {p} settled"
                    );
                }
                let mut got: [Vec<Count>; 3] = [
                    vec![Count::ZERO; nodes],
                    vec![Count::ZERO; nl],
                    vec![Count::ZERO; nr],
                ];
                let [up, dl, dr] = &mut got;
                g.up(&counts(&l), &counts(&r), up, &mut unnoted);
                g.down_into(true, &counts(&column), &counts(&r), dl, true, &mut unnoted);
                g.down_into(false, &counts(&column), &counts(&l), dr, true, &mut unnoted);
                for (p, (g, w)) in got.iter().zip(&want_counts).enumerate() {
                    assert!(
                        g.iter()
                            .zip(w)
                            .all(|(a, b)| format!("{a:?}") == format!("{b:?}")),
                        "shape {s} pass {p} in counts"
                    );
                }
                for p in 0..3 {
                    kernels[p] |= g.kernels()[p];
                    relays_out[p] |= g.plans[p]
                        .kernel
                        .as_ref()
                        .is_some_and(|k| k.relays[0].is_some());
                }
            }
            assert_eq!(kernels, [true; 3], "shape {s}: {} ways", ways.len());
            if s == 0 {
                assert!(
                    relays_out[1],
                    "DBN_13's pass into the left child relays its output"
                );
            }
        }
    }

    /// A pass down whose slot is not one to one, two digits stepping the
    /// left child's slot alike, settles its column once complete: in every
    /// way it runs, the column and its range are those of the pass over
    /// pairs settled after.
    #[test]
    fn a_slot_not_one_to_one_is_settled_once_complete() {
        let (at, dims) = (
            (0, 0),
            vec![
                Dim::pair(2, 1, 0),
                Dim::node(4, 1, 1, 1),
                Dim::node(8, 0, 4, 4),
            ],
        );
        let grid = Grid::new(at, dims.clone(), Vec::new());
        assert!(!grid.one_to_one(Pass::Left));
        let value = |i: usize| ((i * 2_654_435_761) % 1009) as f64 / 1009.0 + 0.003;
        let (l, r, column): (Vec<f64>, Vec<f64>, Vec<f64>) = (
            (0..5).map(value).collect(),
            (32..64).map(value).collect(),
            (64..96).map(value).collect(),
        );
        let mut want = by_pairs(&dims, at, &l, &r, &column)[1].clone();
        let mut noted = Range::NEW;
        <f64 as Value>::settle_all(&mut want, &mut noted);
        for g in grid.variants() {
            let (mut dl, mut range) = (vec![0.0; l.len()], Range::NEW);
            g.down_into(true, &column, &r, &mut dl, true, &mut range);
            assert_eq!(
                dl.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                want.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
            );
            assert_eq!(
                (range.low.to_bits(), range.high.to_bits()),
                (noted.low.to_bits(), noted.high.to_bits())
            );
        }
    }

    /// A mixed radix read back from the sequence it numbers, digits that
    /// continue each other merged; a sequence no radix numbers refused.
    #[test]
    fn digits_read_a_mixed_radix_back() {
        let of = |radices: &[usize], steps: &[(isize, isize)]| -> Vec<(isize, isize)> {
            let n: usize = radices.iter().product();
            (0..n)
                .map(|mut i| {
                    let mut v = (7, 3);
                    for (&r, &s) in radices.iter().zip(steps) {
                        let d = (i % r) as isize;
                        i /= r;
                        v = (v.0 + d * s.0, v.1 + d * s.1);
                    }
                    v
                })
                .collect()
        };
        let v = of(&[3, 5, 2], &[(1, 0), (0, 4), (30, 30)]);
        let d = digits(v.len(), |i| Some(v[i])).unwrap();
        assert_eq!(
            d,
            vec![
                Digit {
                    radix: 3,
                    step: (1, 0)
                },
                Digit {
                    radix: 5,
                    step: (0, 4)
                },
                Digit {
                    radix: 2,
                    step: (30, 30)
                }
            ]
        );
        let mut places = Odometer::new(&d, (7, 3));
        assert!(v.iter().all(|&x| places.next() == x));
        let v = of(&[3, 4], &[(1, 0), (3, 0)]);
        assert_eq!(
            digits(v.len(), |i| Some(v[i])).unwrap(),
            vec![Digit {
                radix: 12,
                step: (1, 0)
            }]
        );
        let v: Vec<(isize, isize)> = (0..6).map(|i| (i * i, 0)).collect();
        assert!(digits(v.len(), |i| Some(v[i])).is_none());
    }

    /// Adjacent digits whose steps continue each other on every side are
    /// read as one; an offset digit and a node digit never are.
    #[test]
    fn coalesced_merges_digits_that_continue_each_other() {
        let dims = vec![
            Dim::pair(2, 1, 0),
            Dim::pair(4, 2, 0),
            Dim::pair(2, 0, 1),
            Dim::node(2, 0, 2, 1),
            Dim::node(3, 0, 4, 2),
            Dim::node(2, 1, 0, 6),
        ];
        let got: Vec<(usize, isize, isize, isize)> = coalesced(dims)
            .iter()
            .map(|d| (d.radix, d.left, d.right, d.node))
            .collect();
        assert_eq!(
            got,
            vec![(8, 1, 0, 0), (2, 0, 1, 0), (6, 0, 2, 1), (2, 1, 0, 6)]
        );
    }

    /// A slot that two places of the dimensions it depends on share is not
    /// one to one; gaps each past the spans of the smaller are.
    #[test]
    fn one_to_one_reads_the_gaps() {
        let digit = |radix: usize, stride: usize| (stride, (radix - 1) * stride);
        assert!(one_to_one([digit(4, 1), digit(3, 4)].into_iter()));
        assert!(one_to_one(
            [digit(4, 12), digit(3, 4), digit(2, 1)].into_iter()
        ));
        assert!(!one_to_one([digit(4, 1), digit(3, 3)].into_iter()));
        // A list of values 0, 2, 3: gap 1, span 3; past it, a gap of 4.
        assert!(one_to_one([(1, 3), digit(2, 4)].into_iter()));
        assert!(!one_to_one([(0, 3), digit(2, 4)].into_iter()));
    }
}
