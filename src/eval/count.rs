//! The two passes of a count over one diagram ([`count`], [`Counted`]),
//! and the count of the conjunction of two diagrams, one on a projection
//! of the other's vtree, without building it ([`conjunction_count`]).

use std::cell::{Cell, RefCell};

use tididi::diagram::LeafLabel;
use tididi::vtree::VarId;

use super::*;

/// The least pairs a level holds for the count passes to read it as a
/// [`Grid`] ([`grid_of`]). Fitting a level that stores its pairs costs about
/// a pass over them, and choosing a grid's orders, an implicit level's too,
/// a fixed amount (some 10^5 instructions), which a level of this size
/// repays; what a grid saves beyond decoding is cache misses, which a
/// smaller level mostly does not have. On the dense networks, most of the
/// evidence's pairs are on grids of this size or more.
const GRID_MIN_PAIRS: usize = 1 << 14;

/// The two passes of a weighted count over a diagram: `up[t][i] ·
/// 2^exps[t]` is the count of node `i` of level `t` (the leaf levels hold
/// the weights of their three nodes), and `Counted::down` the count of the
/// models through it, with the output's context one. A level's counts up,
/// and a column of counts down, are settled once complete.
///
/// The pass up keeps a level's counts under a power of two, `exps[t]`,
/// where a product of them with their sibling's would leave the normal
/// floats: before it computes the parent, each child is centred on one,
/// and the parent's power is the sum of its children's, the same for
/// every term of every node. A level's counts are scaled only where its
/// parent's products would leave the normal floats unscaled, or its sums
/// overflow, so a count that is exact in plain floats is that count.
pub struct Counted<V> {
    pub(crate) up: Vec<Vec<V>>,
    /// The power of two each level's counts up are kept under.
    exps: Vec<i64>,
    /// The range of each level's counts up.
    ranges: Vec<Range>,
    /// Whether each level's counts up are exact, and those of every level
    /// under it.
    exact: Vec<bool>,
    down: RefCell<Down<V>>,
    /// The node of each level that is true over the level's variables, if
    /// the level has one (`true_nodes`).
    pub(crate) top: Vec<Option<usize>>,
    /// The least pairs of a level the passes try as a [`Grid`].
    grid_min: usize,
    /// The pairs of the levels the pass up read as grids.
    grid_pairs: usize,
    /// The pairs the pass up took by each route.
    routes_up: [usize; ROUTES],
}

/// The routes a level's pass takes ([`Routes`]): a grid's vector kernels, a
/// grid's scalar loops in floats and in [`Count`]s, a grid's walk node by
/// node in floats and in `Count`s, and the loops over pairs in floats and
/// in `Count`s.
pub(crate) const ROUTES: usize = 7;

/// How a pass reads a level: a grid's plan through its kernel or its
/// scalar loops, a grid's walk node by node, or the loops over pairs.
#[derive(Clone, Copy)]
pub(super) enum Route {
    Kernel,
    Loops,
    Walk,
    Pairs,
}

impl Route {
    /// A grid's plan, through its kernel or not.
    fn planned(kernel: bool) -> Route {
        if kernel { Route::Kernel } else { Route::Loops }
    }

    /// The route's place among the [`ROUTES`] in `V`s.
    fn index<V: Value>(self) -> usize {
        let counts = usize::from(V::floats(&[]).is_none());
        match self {
            Route::Kernel => 0,
            Route::Loops => 1 + counts,
            Route::Walk => 3 + counts,
            Route::Pairs => 5 + counts,
        }
    }
}

/// The pairs a count's passes took by each route, up and down: a grid's
/// kernels, a grid's scalar loops in floats and in [`Count`]s, a grid's
/// walk node by node in floats and in `Count`s, and the loops over pairs
/// in floats and in `Count`s. Each pass down into a child counts the
/// level's pairs; on a grid's walk and over pairs, a node whose context is
/// zero is not walked, and its pairs not counted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Routes {
    pub up: [usize; ROUTES],
    pub down: [usize; ROUTES],
}

impl Routes {
    /// Both counts.
    #[must_use]
    pub fn plus(self, other: Routes) -> Routes {
        Routes {
            up: std::array::from_fn(|i| self.up[i] + other.up[i]),
            down: std::array::from_fn(|i| self.down[i] + other.down[i]),
        }
    }

    /// The pairs taken in [`Count`]s, up and down.
    #[must_use]
    pub fn in_counts(&self) -> (usize, usize) {
        let counts = [Route::Loops, Route::Walk, Route::Pairs].map(Route::index::<Count>);
        (
            counts.iter().map(|&i| self.up[i]).sum(),
            counts.iter().map(|&i| self.down[i]).sum(),
        )
    }
}

/// How the count passes read a level: not tried yet, over its pairs, or as
/// a [`Grid`].
#[derive(Clone, Debug)]
pub(super) enum View {
    Untried,
    Pairs,
    Grid(Box<Grid>),
}

/// The pass down, walked as far as it has been asked for. `columns[t]` is
/// complete where `has[t]` says: `t` is the output's level, or its parent
/// was spread into it ([`Down::spread`]); a column no walk has reached is
/// empty. A complete column is settled, its values noted in `ranges[t]`,
/// and `exact[t]` is whether it is exact: every column above it was, and
/// every product of a context and a count up it took. The count through
/// node `i` is `columns[t][i] · 2^exps[t]`. `opened[t]` says the column
/// was centred where its children's products need it, as it is once,
/// before the first child takes terms from it. `views` holds how the pass
/// up read each level, until the level is spread into both children.
///
/// A column whose own values span more than a float can hold under one
/// power is kept in [`Count`]s in `wide[t]` instead, its column in `V`s
/// left empty; it is exact, and its children are spread from it in
/// `Count`s ([`Down::spread_counted`]).
pub(super) struct Down<V> {
    columns: Vec<Vec<V>>,
    wide: Vec<Option<Vec<Count>>>,
    has: Vec<bool>,
    opened: Vec<bool>,
    ranges: Vec<Range>,
    exact: Vec<bool>,
    exps: Vec<i64>,
    views: Vec<View>,
    /// The pairs the passes into children took by each route.
    routes: [usize; ROUTES],
}

/// A level's counts up as the pass down reads them: values, the power of
/// two they are under, their range and whether they are exact, by level.
#[derive(Clone, Copy)]
pub(super) struct Ups<'a, V> {
    values: &'a [Vec<V>],
    exps: &'a [i64],
    ranges: &'a [Range],
    exact: &'a [bool],
}

impl<V: Value> Down<V> {
    /// No level walked, for a diagram of `n` levels, none of them viewed.
    fn new(n: usize) -> Self {
        Down::with_views(vec![View::Untried; n])
    }

    /// No level walked, the levels viewed as `views` has them.
    fn with_views(views: Vec<View>) -> Self {
        let n = views.len();
        Down {
            columns: vec![Vec::new(); n],
            wide: vec![None; n],
            has: vec![false; n],
            opened: vec![false; n],
            ranges: vec![Range::NEW; n],
            exact: vec![true; n],
            exps: vec![0; n],
            views,
            routes: [0; ROUTES],
        }
    }

    /// Walk the levels from the output's down to `t` that have no column
    /// yet, each spread from its parent ([`Down::spread`]), and give `t` a
    /// column.
    fn walk_to(
        &mut self,
        f: &tididi::Tdd,
        up: Ups<'_, V>,
        t: tididi::vtree::VtreeIdx,
    ) -> Result<(), tididi::OperationError> {
        let vtree = f.vtree();
        if !f.is_zero() {
            let output = f.output();
            // The levels on the way down to `t` that have no column, the
            // deepest first.
            let mut path = Vec::new();
            let mut x = t;
            let reached = loop {
                if x == output.vtree || self.has[x.idx()] {
                    break true;
                }
                match vtree.node(x).parent() {
                    Some(p) => {
                        path.push(x);
                        x = p;
                    }
                    None => break false,
                }
            };
            if reached {
                let at = output.vtree.idx();
                if !self.has[at] {
                    self.columns[at] = V::zeros(up.values[at].len())?;
                    self.columns[at][output.local.idx()] = V::ONE;
                    self.ranges[at] = Range::of(V::ONE);
                    self.has[at] = true;
                }
                for &c in path.iter().rev() {
                    self.spread(f, up, c)?;
                }
            }
        }
        if self.columns[t.idx()].is_empty() && self.wide[t.idx()].is_none() {
            self.columns[t.idx()] = V::zeros(up.values[t.idx()].len())?;
        }
        Ok(())
    }

    /// Spread the column of `c`'s parent `p` into `c`'s, each pair's
    /// context times the count of its other side, and settle it. A grid
    /// runs the pass into `c` alone ([`Grid::down_into`]), or walks its
    /// nodes for it ([`Grid::walk_into`]), and the sibling's waits for a
    /// walk to it: a query reads the column of its own level, and on a
    /// program whose queries are a few of its variables most levels beside
    /// the way down to one are under none. Any other level is spread over
    /// its pairs into both children at once ([`spread_pairs`]).
    ///
    /// Where a child's products, a context times its sibling's count up,
    /// would leave the normal floats, `p`'s column is centred on one in
    /// place ([`centring`]), once, before the first child takes terms from
    /// it; and where they still would, the child reads a centred copy of
    /// its sibling's counts. A child's column takes every term from `p`'s,
    /// under one power of two: the column's plus the sibling's, less the
    /// copy's shift.
    fn spread(
        &mut self,
        f: &tididi::Tdd,
        up: Ups<'_, V>,
        c: tididi::vtree::VtreeIdx,
    ) -> Result<(), tididi::OperationError> {
        use tididi::vtree::VtreeIdx;
        let p = f
            .vtree()
            .node(c)
            .parent()
            .expect("a level spread into has a parent");
        let (l, r) = f.vtree().children(p);
        let (ld, rd) = (f.level(l).child_decoder(), f.level(r).child_decoder());
        let above = self.exact[p.idx()];
        // An inline value is under no power: a child's column takes terms
        // under one only where its sibling, if marginal, is under none,
        // and no level is scaled here where either child is marginal.
        let uniform = |sibling: VtreeIdx| {
            !f.level(sibling).child_decoder().is_marginal() || up.exps[sibling.idx()] == 0
        };
        let movable = above && !ld.is_marginal() && !rd.is_marginal();
        let fails = |range: Range, sibling: VtreeIdx| {
            up.exact[sibling.idx()] && (centre_all() || !range.times(up.ranges[sibling.idx()]))
        };
        if !self.opened[p.idx()] {
            self.opened[p.idx()] = true;
            if movable
                && (fails(self.ranges[p.idx()], l) || fails(self.ranges[p.idx()], r))
                && let Some(k) = centring(self.ranges[p.idx()], Range::NEW)
                && k != 0
            {
                shift_all(&mut self.columns[p.idx()], k, &mut self.ranges[p.idx()]);
                self.exps[p.idx()] -= k;
            }
        }
        let range = self.ranges[p.idx()];
        // A copy of `sibling`'s counts up, its shift and its range, where
        // their products with the column fail the test, centred on one, or
        // could overflow, moved down as far as they stay normal
        // ([`capping`]).
        let copy =
            |sibling: VtreeIdx| -> Result<Option<(Vec<V>, i64, Range)>, tididi::OperationError> {
                let (values, from) = (&up.values[sibling.idx()], up.ranges[sibling.idx()]);
                let Some((low, high)) = from.extent().filter(|_| movable) else {
                    return Ok(None);
                };
                let centre = if fails(range, sibling) {
                    centring(from, Range::NEW).unwrap_or(0)
                } else {
                    0
                };
                let k =
                    (centre + capping(range, from.shifted(centre))).clamp(-1022 - low, 1023 - high);
                if k == 0 {
                    return Ok(None);
                }
                let mut copy = collected(values.len(), values.iter().copied())?;
                let mut copy_range = from;
                shift_all(&mut copy, k, &mut copy_range);
                Ok(Some((copy, k, copy_range)))
            };
        let exp = self.exps[p.idx()];
        // Whether a child's column, counted in `V`s as `values` in range
        // `got` from its sibling's counts read in range `read`, is exact;
        // and where it is not, whether one counted in [`Count`]s from the
        // same columns is, which [`fitted`] may then keep in `V`s under a
        // power of its own.
        let exact = |sibling: VtreeIdx, read: Range, values: &[V], got: Range| {
            above
                && uniform(sibling)
                && up.exact[sibling.idx()]
                && got.finite()
                && (range.times(read)
                    || exact_as_computed(values, got, lowest_product(range, read)))
        };
        let recountable = |sibling: VtreeIdx| above && up.exact[sibling.idx()];
        // The children spread into: `c` alone from a grid, both otherwise.
        let sibling = |x: VtreeIdx| if x == l { r } else { l };
        let kids = match self.views[p.idx()] {
            View::Grid(_) => [Some(c), None],
            View::Untried | View::Pairs => [Some(l), Some(r)],
        };
        // A child's column takes its terms from its parent's alone, all
        // under one power: none is there before.
        debug_assert!(
            kids.iter().flatten().all(|x| !self.has[x.idx()]),
            "a level takes terms from one column, under one power"
        );
        let [copy_0, copy_1] = kids.map(|x| x.map(|x| copy(sibling(x))).transpose());
        let copies = [copy_0?.flatten(), copy_1?.flatten()];
        // What each child reads of its sibling's counts, under the copy's
        // shift, and their range.
        let reads = [0, 1].map(|i| kids[i].map(|x| read_counts(&copies[i], up, sibling(x))));
        let mut fits = [None, None];
        if self.wide[p.idx()].is_some() {
            fits = self.spread_counted(f, up, p, c)?;
        }
        let mut floats = [None, None];
        if (0..2).any(|i| kids[i].is_some() && fits[i].is_none()) {
            let column = &self.columns[p.idx()];
            floats = match &self.views[p.idx()] {
                View::Grid(grid) => {
                    let (of_s, _, read) = reads[0].expect("a grid spreads into one child");
                    [
                        Some(grid_into(
                            grid,
                            c == l,
                            column,
                            of_s,
                            up.values[c.idx()].len(),
                            read.finite(),
                            &mut self.routes,
                        )?),
                        None,
                    ]
                }
                View::Untried | View::Pairs => {
                    let ((of_r, _, _), (of_l, _, _)) = (
                        reads[0].expect("the left child"),
                        reads[1].expect("the right child"),
                    );
                    let (dl, dr, walked) = spread_pairs(f, column, of_l, of_r, p)?;
                    self.routes[Route::Pairs.index::<V>()] += 2 * walked;
                    [Some(settled(dl)), Some(settled(dr))]
                }
            };
        }
        // Each child's column: in `V`s under the parent's power and its
        // sibling's less the shift of the copy it read, and counted again
        // in `Count`s where that is not exact and the counts it reads are.
        let mut recount = false;
        for i in 0..2 {
            let Some(x) = kids[i] else { continue };
            self.has[x.idx()] = true;
            match (fits[i].take(), floats[i].take()) {
                (Some(fit), _) => self.keep(x, fit),
                (None, Some((down, range))) => {
                    let (_, shift, read) = reads[i].expect("a child reads its sibling");
                    self.keep(
                        x,
                        Fit::Floats(down, range, exp + up.exps[sibling(x).idx()] - shift),
                    );
                    let held = exact(
                        sibling(x),
                        read,
                        &self.columns[x.idx()],
                        self.ranges[x.idx()],
                    );
                    self.exact[x.idx()] = held;
                    recount |= !held && recountable(sibling(x));
                }
                (None, None) => unreachable!("a child's column in `V`s or in `Count`s"),
            }
        }
        if recount {
            let again = self.spread_counted(f, up, p, c)?;
            for (x, fit) in kids.into_iter().zip(again) {
                if let Some((x, fit)) = x
                    .zip(fit)
                    .filter(|(x, _)| !self.exact[x.idx()] && recountable(sibling(*x)))
                {
                    self.keep(x, fit);
                }
            }
        }
        if self.has[l.idx()] && self.has[r.idx()] {
            self.views[p.idx()] = View::Pairs;
        }
        Ok(())
    }

    /// Level `p`'s column spread into its children as [`Down::spread`]
    /// spreads it, `c` alone from a grid, in [`Count`]s from the column and
    /// the siblings' counts up, each value times its power; each child's
    /// column kept in `V`s under a power of its own where it fits
    /// ([`fitted`]) and in `Count`s where it does not, in the order of
    /// [`Down::spread`]'s children.
    fn spread_counted(
        &mut self,
        f: &tididi::Tdd,
        up: Ups<'_, V>,
        p: tididi::vtree::VtreeIdx,
        c: tididi::vtree::VtreeIdx,
    ) -> Result<[Option<Fit<V>>; 2], tididi::OperationError> {
        let (l, r) = f.vtree().children(p);
        let owned;
        let column: &[Count] = match &self.wide[p.idx()] {
            Some(wide) => wide,
            None => {
                owned = in_counts(&self.columns[p.idx()], self.exps[p.idx()])?;
                &owned
            }
        };
        let counts = |x: tididi::vtree::VtreeIdx| in_counts(&up.values[x.idx()], up.exps[x.idx()]);
        // A sum of products of `Count`s is not normalized ([`Count`]'s
        // `Mul`): each column is settled before it is fitted or kept.
        let fit = |(down, _): (Vec<Count>, Range)| -> Result<Fit<V>, tididi::OperationError> {
            Ok(match fitted(&down)? {
                Some((values, power)) => {
                    let (values, range) = settled(values);
                    Fit::Floats(values, range, power)
                }
                None => Fit::Wide(down),
            })
        };
        match &self.views[p.idx()] {
            View::Grid(grid) => {
                let s = if c == l { r } else { l };
                let down = grid_into(
                    grid,
                    c == l,
                    column,
                    &counts(s)?,
                    up.values[c.idx()].len(),
                    true,
                    &mut self.routes,
                )?;
                Ok([Some(fit(down)?), None])
            }
            View::Untried | View::Pairs => {
                let (dl, dr, walked) = spread_pairs(f, column, &counts(l)?, &counts(r)?, p)?;
                self.routes[Route::Pairs.index::<Count>()] += 2 * walked;
                Ok([Some(fit(settled(dl))?), Some(fit(settled(dr))?)])
            }
        }
    }

    /// Level `x`'s column as `fit` has it, and exact: in `V`s under its
    /// power with its range, or in [`Count`]s.
    fn keep(&mut self, x: tididi::vtree::VtreeIdx, fit: Fit<V>) {
        self.exact[x.idx()] = true;
        match fit {
            Fit::Floats(down, range, power) => {
                self.columns[x.idx()] = down;
                self.ranges[x.idx()] = range;
                self.wide[x.idx()] = None;
                self.exps[x.idx()] = power;
            }
            Fit::Wide(down) => {
                self.columns[x.idx()] = Vec::new();
                self.ranges[x.idx()] = Range::NEW;
                self.wide[x.idx()] = Some(down);
                self.exps[x.idx()] = 0;
            }
        }
    }
}

/// The column of child `c` of grid level `grid`, the left child where
/// `left` says, from its parent's `column` and its sibling's counts
/// `other`, `n` values, settled, and the range they were noted in: the
/// grid walked node by node where few of its nodes have a context
/// ([`Grid::walk_into`]), and its plans run otherwise ([`Grid::down_into`]),
/// whose kernels need `finite` counts. The pairs it took added to their
/// route's in `routes`.
pub(super) fn grid_into<V: Value>(
    grid: &Grid,
    left: bool,
    column: &[V],
    other: &[V],
    n: usize,
    finite: bool,
    routes: &mut [usize; ROUTES],
) -> Result<(Vec<V>, Range), tididi::OperationError> {
    let mut down = V::zeros(n)?;
    match walk_contexts(grid, column) {
        Some(contexts) => {
            grid.walk_into(left, column, other, &mut down);
            routes[Route::Walk.index::<V>()] += contexts * grid.per_node();
            Ok(settled(down))
        }
        None => {
            let mut range = Range::NEW;
            let kernel = grid.down_into(left, column, other, &mut down, finite, &mut range);
            routes[Route::planned(kernel).index::<V>()] += grid.pairs();
            Ok((down, range))
        }
    }
}

/// `values` settled, and the range they were noted in.
pub(super) fn settled<V: Value>(mut values: Vec<V>) -> (Vec<V>, Range) {
    let mut range = Range::NEW;
    V::settle_all(&mut values, &mut range);
    (values, range)
}

/// The pass down over level `p`'s pairs from its `column`: each child's
/// column down, each pair's context times the count of its other side,
/// the left child's counts read from `of_l` and the right's from `of_r`,
/// and the pairs of the nodes of nonzero context. Each child's counts sit
/// beside its column down while the level is spread, so that a pair reads
/// a side's count and adds to its column in one place: on a level larger
/// than the caches, one miss a side rather than two. Whether each child's
/// sides are values or nodes is read once for the level
/// ([`spread_sides`]), not at every pair.
#[inline(never)]
pub(super) fn spread_pairs<V: Value>(
    f: &tididi::Tdd,
    column: &[V],
    of_l: &[V],
    of_r: &[V],
    p: tididi::vtree::VtreeIdx,
) -> Result<(Vec<V>, Vec<V>, usize), tididi::OperationError> {
    let (l, r) = f.vtree().children(p);
    match (
        f.level(l).child_decoder().is_marginal(),
        f.level(r).child_decoder().is_marginal(),
    ) {
        (false, false) => spread_sides::<V, false, false>(f, column, of_l, of_r, p),
        (false, true) => spread_sides::<V, false, true>(f, column, of_l, of_r, p),
        (true, false) => spread_sides::<V, true, false>(f, column, of_l, of_r, p),
        (true, true) => spread_sides::<V, true, true>(f, column, of_l, of_r, p),
    }
}

/// [`spread_pairs`], the left and the right child's sides values where
/// `LV` and `RV` say, and nodes otherwise.
pub(super) fn spread_sides<V: Value, const LV: bool, const RV: bool>(
    f: &tididi::Tdd,
    column: &[V],
    of_l: &[V],
    of_r: &[V],
    p: tididi::vtree::VtreeIdx,
) -> Result<(Vec<V>, Vec<V>, usize), tididi::OperationError> {
    use tididi::diagram::ChildDecoder;
    let decoder = |valued: bool| {
        if valued {
            ChildDecoder::marginal()
        } else {
            ChildDecoder::structural()
        }
    };
    let (ld, rd) = (decoder(LV), decoder(RV));
    let beside = |up: &[V]| collected(up.len(), up.iter().map(|&x| [x, V::ZERO]));
    let (mut lc, mut rc) = (beside(of_l)?, beside(of_r)?);
    let level = f.level(p);
    let mut walked = 0;
    // A stored level's nodes are read as slices of its pairs, in a loop
    // apart from an implicit level's generated ones.
    match level.stored() {
        Some(stored) => {
            for (i, node) in stored.nodes().iter().enumerate() {
                let context = column[i];
                if context.is_zero() {
                    continue;
                }
                let pairs = stored.of(node);
                walked += pairs.len();
                for &pair in pairs {
                    spread_pair(ld, rd, &mut lc, &mut rc, context, pair);
                }
            }
        }
        None => {
            for (i, pairs) in level.internal_inputs_iter() {
                let context = column[i];
                if context.is_zero() {
                    continue;
                }
                walked += pairs.len();
                pairs.for_each(|pair| spread_pair(ld, rd, &mut lc, &mut rc, context, pair));
            }
        }
    }
    let down = |both: Vec<[V; 2]>| both.into_iter().map(|[_, d]| d).collect::<Vec<V>>();
    Ok((down(lc), down(rc), walked))
}

/// One pair of the pass down ([`spread_pairs`]): the pair's context
/// times the count of each side added to the other side's column, where
/// that side is a node or a slot. Inlined into both loops, a stored
/// level's and an implicit one's, so that no pair makes a call.
#[inline(always)]
pub(super) fn spread_pair<V: Value>(
    ld: tididi::diagram::ChildDecoder,
    rd: tididi::diagram::ChildDecoder,
    lc: &mut [[V; 2]],
    rc: &mut [[V; 2]],
    context: V,
    pair: tididi::diagram::ChildPair,
) {
    let (a, ai) = match slot(ld, pair.left) {
        Ok(k) => (lc[k][0], Some(k)),
        Err(x) => (x, None),
    };
    let (b, bi) = match slot(rd, pair.right) {
        Ok(k) => (rc[k][0], Some(k)),
        Err(x) => (x, None),
    };
    if let Some(ai) = ai {
        lc[ai][1] += context * b;
    }
    if let Some(bi) = bi {
        rc[bi][1] += context * a;
    }
}

/// A child's column down: in `V`s, settled into its range, under a power
/// of two, or in [`Count`]s where it spans more than a float ([`Down`]).
pub(super) enum Fit<V> {
    Floats(Vec<V>, Range, i64),
    Wide(Vec<Count>),
}

/// What a child reads of its sibling's counts up in the pass down: the
/// centred `copy` of them, under its shift, where [`Down::spread`] made
/// one, and the counts of `sibling` in `up` otherwise; with their range.
pub(super) fn read_counts<'a, V: Value>(
    copy: &'a Option<(Vec<V>, i64, Range)>,
    up: Ups<'a, V>,
    sibling: tididi::vtree::VtreeIdx,
) -> (&'a [V], i64, Range) {
    match copy {
        Some((values, k, range)) => (values.as_slice(), *k, *range),
        None => (
            up.values[sibling.idx()].as_slice(),
            0,
            up.ranges[sibling.idx()],
        ),
    }
}

/// The nodes of nonzero context in a grid level's `column` where the pass
/// down walks the grid node by node ([`Grid::walks`]), and `None` where it
/// runs the grid's plans. Counted a block at a time, so that a column with
/// too many for the walk is read only until it has them.
pub(super) fn walk_contexts<V: Value>(grid: &Grid, column: &[V]) -> Option<usize> {
    let mut contexts = 0;
    for block in column.chunks(1 << 12) {
        contexts += block.iter().filter(|x| !x.is_zero()).count();
        if !grid.walks(contexts) {
            return None;
        }
    }
    grid.walks(contexts).then_some(contexts)
}

/// A column of the pass down as a query reads it: its values, their
/// range, whether they are exact, and the power of two they are under;
/// or, where it spans more than a float, its values in [`Count`]s, and
/// `values` empty.
pub(super) struct Column<'a, V> {
    values: std::cell::Ref<'a, [V]>,
    wide: Option<std::cell::Ref<'a, [Count]>>,
    range: Range,
    exact: bool,
    exp: i64,
}

impl<V: Value> Counted<V> {
    /// The weighted count of the diagram.
    pub fn total(&self, f: &tididi::Tdd) -> Scaled<V> {
        if f.is_zero() {
            return Scaled::new(V::ZERO, 0);
        }
        let output = f.output();
        Scaled::new(
            self.up[output.vtree.idx()][output.local.idx()],
            self.exps[output.vtree.idx()],
        )
    }

    /// Whether the weighted count of `f`, the diagram counted, is exact:
    /// always, in [`Count`]s.
    pub fn exact(&self, f: &tididi::Tdd) -> bool {
        f.is_zero() || self.exact[f.output().vtree.idx()]
    }

    /// The count of the models through each node of level `t` of `f`, the
    /// diagram counted, with its range and whether it is exact. The pass
    /// down walks the levels on the way down to `t` the first time `t` or a
    /// level under it is asked for, and no other: a query reads the column
    /// of its cone's level, and on a program with few queries most levels
    /// are under none of them.
    fn down(
        &self,
        f: &tididi::Tdd,
        t: tididi::vtree::VtreeIdx,
    ) -> Result<Column<'_, V>, tididi::OperationError> {
        let up = Ups {
            values: &self.up,
            exps: &self.exps,
            ranges: &self.ranges,
            exact: &self.exact,
        };
        self.down.borrow_mut().walk_to(f, up, t)?;
        let d = self.down.borrow();
        let (range, exact, exp) = (d.ranges[t.idx()], d.exact[t.idx()], d.exps[t.idx()]);
        let wide =
            std::cell::Ref::filter_map(self.down.borrow(), |d| d.wide[t.idx()].as_deref()).ok();
        Ok(Column {
            values: std::cell::Ref::map(d, |d| d.columns[t.idx()].as_slice()),
            wide,
            range,
            exact,
            exp,
        })
    }

    /// The weighted count of `f ∧ x`, `x` the variable `var` at `leaf` or,
    /// where `positive` is false, its negation: the sum over the leaf nodes
    /// that assert it of the models through them; `None` where it is not
    /// exact.
    pub fn with_literal(
        &self,
        f: &tididi::Tdd,
        leaf: tididi::vtree::VtreeIdx,
        var: VarId,
        positive: bool,
        w: &Probabilities,
    ) -> Result<Option<Scaled<V>>, tididi::OperationError> {
        use tididi::diagram::{NEG_LEAF_IDX, ONE_LEAF_IDX, POS_LEAF_IDX};
        let column = self.down(f, leaf)?;
        let (asserting, label) = if positive {
            (POS_LEAF_IDX, LeafLabel::Pos)
        } else {
            (NEG_LEAF_IDX, LeafLabel::Neg)
        };
        if let Some(wide) = &column.wide {
            let with = ((wide[ONE_LEAF_IDX.idx()] + wide[asserting.idx()]).normalized()
                * Count::new(w.weight(var, label)))
            .normalized();
            return Ok(Some(Scaled::new(
                V::from_f64(with.m),
                i64::from(BLOCK) * i64::from(with.b),
            )));
        }
        let mut sum = Range::NEW;
        let one_or_lit =
            (column.values[ONE_LEAF_IDX.idx()] + column.values[asserting.idx()]).settle(&mut sum);
        let weight = V::from_f64(w.weight(var, label));
        let mut product = Range::NEW;
        let with = (one_or_lit * weight).settle(&mut product);
        Ok((column.exact
            && column.range.times(Range::of(weight))
            && sum.finite()
            && product.finite())
        .then_some(Scaled::new(with, column.exp)))
    }
}

/// What a pair's side is worth in a column: the value at the slot it
/// names, or the value it holds inline.
#[inline(always)]
pub(super) fn side<V: Value>(
    column: &[V],
    d: tididi::diagram::ChildDecoder,
    s: tididi::diagram::EncodedChildRef,
) -> V {
    match slot(d, s) {
        Ok(k) => column[k],
        Err(x) => x,
    }
}

/// The slot a pair's side names in its child's column, or the value it
/// holds inline.
pub(super) fn slot<V: Value>(
    d: tididi::diagram::ChildDecoder,
    s: tididi::diagram::EncodedChildRef,
) -> Result<usize, V> {
    use tididi::diagram::{ChildRef, ValueRef};
    match d.child(s) {
        ChildRef::Node(n) => Ok(n.idx()),
        ChildRef::Value(ValueRef::Slot(k)) => Ok(k as usize),
        ChildRef::Value(ValueRef::Inline(v)) => Err(V::from_f64(f64::from(v))),
    }
}

/// `n` copies of `x`, or `OverBudget` where the allocator refuses them. The
/// counts live outside the engine's byte budget, and a column of a large
/// level is gigabytes: `vec!` would abort the run where a refusal retires
/// the trial.
pub(super) fn filled<T: Clone>(n: usize, x: T) -> Result<Vec<T>, tididi::OperationError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| tididi::OperationError::OverBudget)?;
    v.resize(n, x);
    Ok(v)
}

/// The `n` items of `items`, as [`filled`] allocates.
pub(super) fn collected<T>(
    n: usize,
    items: impl Iterator<Item = T>,
) -> Result<Vec<T>, tididi::OperationError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| tididi::OperationError::OverBudget)?;
    v.extend(items);
    Ok(v)
}

/// The weighted count of `f` under `w`, in `V`s: the pass up the diagram
/// for each node's count, and the pass down for the count of the models
/// through each node as far as it is asked for (`Counted::down`).
pub fn count<V: Value>(
    f: &tididi::Tdd,
    w: &Probabilities,
) -> Result<Counted<V>, tididi::OperationError> {
    let mut counted = Counted::leaves(f, w, GRID_MIN_PAIRS)?;
    counted.up_from(f, 0, false);
    Ok(counted)
}

/// A count in floats while it is exact ([`Range`]), and in [`Count`]s once
/// it, or a query read against it, was not.
pub(crate) enum Counts {
    Plain(Counted<f64>),
    Wide(Counted<Count>),
}

impl Counts {
    /// A count in floats, whose counts up are exact ([`count_exact`]),
    /// taken over in [`Count`]s where a query read against it was not
    /// exact: the counts up as they are, each value times its level's
    /// power, and the pass down anew. The routes of the pass down it
    /// replaces are returned.
    pub(crate) fn widen(&mut self) -> Result<Routes, tididi::OperationError> {
        let Counts::Plain(plain) = self else {
            panic!("a count in `Count`s is exact")
        };
        let replaced = Routes {
            up: [0; ROUTES],
            down: plain.down.get_mut().routes,
        };
        let wide = plain.take_counts()?;
        *self = Counts::Wide(wide);
        Ok(replaced)
    }

    /// The pairs of the levels the pass up read as grids.
    pub(crate) fn grid_pairs(&self) -> usize {
        match self {
            Counts::Plain(c) => c.grid_pairs,
            Counts::Wide(c) => c.grid_pairs,
        }
    }

    /// The pairs the passes took by each route so far.
    pub(crate) fn routes(&self) -> Routes {
        match self {
            Counts::Plain(c) => c.routes(),
            Counts::Wide(c) => c.routes(),
        }
    }
}

/// The weighted count of `f` under `w` in floats where it is exact, each
/// level under its power of two. A level under `f`'s output whose counts
/// up are not exact is counted again in [`Count`]s from its children and
/// kept in floats where its values fit ([`Counted::recount_level`]);
/// where they do not, the count goes on in `Count`s from that level: the
/// levels the pass up settled before it are exact in floats, which is
/// what a count in `Count`s computes on them too, and are taken over as
/// they are, each value times its level's power. `recounts` counts the
/// switch.
pub(crate) fn count_exact(
    f: &tididi::Tdd,
    w: &Probabilities,
    recounts: &mut usize,
) -> Result<Counts, tididi::OperationError> {
    let mut plain = Counted::<f64>::leaves(f, w, GRID_MIN_PAIRS)?;
    let mut from = 0;
    let at = loop {
        let Some(at) = plain.up_from(f, from, true) else {
            return Ok(Counts::Plain(plain));
        };
        if !plain.recount_level(f, at)? {
            break at;
        }
        from = at + 1;
    };
    *recounts += 1;
    let mut wide = plain.take_counts()?;
    wide.up_from(f, at, false);
    Ok(Counts::Wide(wide))
}

/// `values`, each times `2^exp`, as [`Count`]s.
pub(super) fn in_counts<V: Value>(
    values: &[V],
    exp: i64,
) -> Result<Vec<Count>, tididi::OperationError> {
    collected(
        values.len(),
        values.iter().map(|&x| x.to_count().shifted(exp)),
    )
}

/// The exact counts `column` as `V`s under a power of two of their own:
/// the values times `2^k`, `k` centring the nonzero ones on one, and
/// `-k`, the power they are under; `None` where the nonzero values do not
/// fit the normal floats together under one power.
pub(super) fn fitted<V: Value>(
    column: &[Count],
) -> Result<Option<(Vec<V>, i64)>, tididi::OperationError> {
    let mut extent: Option<(i64, i64)> = None;
    for c in column.iter().filter(|c| !c.is_zero()) {
        let e = exponent(c.m) + i64::from(BLOCK) * i64::from(c.b);
        extent = Some(extent.map_or((e, e), |(low, high)| (low.min(e), high.max(e))));
    }
    let k = extent.map_or(0, |(low, high)| -(low + high).div_euclid(2));
    if extent.is_some_and(|(low, high)| low + k < -1022 || high + k > 1023) {
        return Ok(None);
    }
    Ok(Some((
        collected(
            column.len(),
            column.iter().map(|c| V::from_count(c.shifted(k))),
        )?,
        -k,
    )))
}

impl Counted<f64> {
    /// The counts up taken over in [`Count`]s, each value times its
    /// level's power, with the levels' views and no column down; `self` is
    /// left without them. Where the counts up are exact, these are the
    /// counts a count in `Count`s from the leaves computes.
    fn take_counts(&mut self) -> Result<Counted<Count>, tididi::OperationError> {
        let up = std::mem::take(&mut self.up);
        Ok(Counted {
            up: up
                .into_iter()
                .zip(&self.exps)
                .map(|(column, &k)| in_counts(&column, k))
                .collect::<Result<_, _>>()?,
            exps: vec![0; self.exps.len()],
            ranges: vec![Range::NEW; self.ranges.len()],
            exact: vec![true; self.exact.len()],
            down: RefCell::new(Down::with_views(std::mem::take(
                &mut self.down.get_mut().views,
            ))),
            top: std::mem::take(&mut self.top),
            grid_min: self.grid_min,
            grid_pairs: self.grid_pairs,
            routes_up: self.routes_up,
        })
    }
}

impl<V: Value> Counted<V> {
    /// The `k`-th internal level in bottom-up order counted again in
    /// [`Count`]s from its children's counts, and kept in `V`s under a
    /// power of two of its own where its nonzero values fit the normal
    /// floats together ([`fitted`]); whether they did. The pass up's test
    /// on a level, [`Range::times`] on its children's ranges, takes the
    /// products of their extremes, which no pair of the level need take:
    /// a level whose own values span a few hundred powers of two can read
    /// children that span a thousand each.
    fn recount_level(&mut self, f: &tididi::Tdd, k: usize) -> Result<bool, tididi::OperationError> {
        let vtree = f.vtree();
        let (t, l, r) = vtree
            .internal_bottomup()
            .nth(k)
            .expect("a level of the pass up");
        let mut wide: Vec<Vec<Count>> = vec![Vec::new(); vtree.num_nodes()];
        for c in [l, r] {
            wide[c.idx()] = in_counts(&self.up[c.idx()], self.exps[c.idx()])?;
        }
        wide[t.idx()] = Count::zeros(self.up[t.idx()].len())?;
        let mut unnoted = Range::NEW;
        level_up(
            f,
            (t, l, r),
            &self.down.get_mut().views[t.idx()],
            &mut wide,
            &mut unnoted,
            &mut self.routes_up,
        );
        let Some((column, exp)) = fitted::<V>(&wide[t.idx()])? else {
            return Ok(false);
        };
        (self.up[t.idx()], self.ranges[t.idx()]) = settled(column);
        self.exps[t.idx()] = exp;
        self.exact[t.idx()] = true;
        Ok(true)
    }
}

impl<V: Value> Counted<V> {
    /// `f`'s levels with their slots, a slot per node reference, and the
    /// leaf levels' counts, the weights of their three nodes; the passes
    /// read a level of `grid_min` pairs or more as a grid where it is one.
    fn leaves(
        f: &tididi::Tdd,
        w: &Probabilities,
        grid_min: usize,
    ) -> Result<Self, tididi::OperationError> {
        use tididi::diagram::{NEG_LEAF_IDX, ONE_LEAF_IDX, POS_LEAF_IDX};
        use tididi::vtree::VtreeIdx;
        let vtree = f.vtree();
        let n = vtree.num_nodes();
        // Pushed into a vector of `n`: the fallible iterator gives no size.
        let mut up: Vec<Vec<V>> = Vec::with_capacity(n);
        for t in 0..n {
            up.push(V::zeros(f.reference_slot_count(VtreeIdx(t as u32)))?);
        }
        let mut ranges = vec![Range::NEW; n];
        let mut exact = vec![true; n];
        if !f.is_zero() {
            for (leaf, var) in vtree.leaf_bottomup() {
                let column = &mut up[leaf.idx()];
                let range = &mut ranges[leaf.idx()];
                for (slot, label) in [
                    (ONE_LEAF_IDX, LeafLabel::One),
                    (POS_LEAF_IDX, LeafLabel::Pos),
                    (NEG_LEAF_IDX, LeafLabel::Neg),
                ] {
                    column[slot.idx()] = V::from_f64(w.weight(var, label)).settle(range);
                }
                exact[leaf.idx()] = range.normal();
            }
        }
        Ok(Counted {
            up,
            exps: vec![0; n],
            ranges,
            exact,
            down: RefCell::new(Down::new(n)),
            top: true_nodes(f),
            grid_min,
            grid_pairs: 0,
            routes_up: [0; ROUTES],
        })
    }

    /// The pairs the passes took by each route so far.
    pub(crate) fn routes(&self) -> Routes {
        Routes {
            up: self.routes_up,
            down: self.down.borrow().routes,
        }
    }

    /// The pass up over the internal levels from the `from`-th in
    /// bottom-up order. With `stop`, it stops at the first level under the
    /// output whose counts are not exact and returns its place in that
    /// order, to be counted again. A level is read as a grid the first time
    /// a pass reaches it ([`grid_of`]), and the grid kept for the pass down;
    /// any other level is read over its pairs.
    ///
    /// Where the children's products fail [`Range::times`] or could
    /// overflow ([`capping`]), each child is centred on one in place before
    /// the level is computed, and both moved down together where their
    /// products still could. A level whose children's products still fail
    /// the test is exact where its values are ([`exact_as_computed`]);
    /// where they came out too small, they are moved up and computed again
    /// ([`raising`]).
    fn up_from(&mut self, f: &tididi::Tdd, from: usize, stop: bool) -> Option<usize> {
        if f.is_zero() {
            return None;
        }
        let vtree = f.vtree();
        let output = f.output().vtree;
        let (up, exps, ranges, exact, views) = (
            &mut self.up,
            &mut self.exps,
            &mut self.ranges,
            &mut self.exact,
            &mut self.down.get_mut().views,
        );
        for (k, (t, l, r)) in vtree.internal_bottomup().enumerate().skip(from) {
            let (ld, rd) = (f.level(l).child_decoder(), f.level(r).child_decoder());
            if let View::Untried = views[t.idx()] {
                views[t.idx()] = match grid_of(f, (t, l, r), self.grid_min) {
                    Some(grid) => {
                        self.grid_pairs += grid.pairs();
                        View::Grid(Box::new(grid))
                    }
                    None => View::Pairs,
                };
            }
            let below = exact[l.idx()] && exact[r.idx()];
            // An inline value is under no power: a level reading a marginal
            // child takes its terms under one only where that child is under
            // none, and no child of it is scaled here.
            let uniform = (!ld.is_marginal() || exps[l.idx()] == 0)
                && (!rd.is_marginal() || exps[r.idx()] == 0);
            let movable = below && !ld.is_marginal() && !rd.is_marginal();
            // The children's counts times `2^k` and `2^m`, their powers less
            // the shifts.
            let shift =
                |up: &mut [Vec<V>], exps: &mut [i64], ranges: &mut [Range], (k, m): (i64, i64)| {
                    for (c, k) in [(l, k), (r, m)] {
                        if k != 0 {
                            shift_all(&mut up[c.idx()], k, &mut ranges[c.idx()]);
                            exps[c.idx()] -= k;
                        }
                    }
                };
            // Each child centred on one, and both moved down together
            // where their products could overflow.
            let centre = |up: &mut [Vec<V>], exps: &mut [i64], ranges: &mut [Range]| {
                let at = |c: tididi::vtree::VtreeIdx, ranges: &[Range]| {
                    centring(ranges[c.idx()], Range::NEW).unwrap_or(0)
                };
                let ks = (at(l, ranges), at(r, ranges));
                shift(up, exps, ranges, ks);
                if let Some(ks) = split_shift(
                    ranges[l.idx()],
                    ranges[r.idx()],
                    capping(ranges[l.idx()], ranges[r.idx()]),
                ) {
                    shift(up, exps, ranges, ks);
                }
            };
            if movable
                && (centre_all()
                    || !ranges[l.idx()].times(ranges[r.idx()])
                    || capping(ranges[l.idx()], ranges[r.idx()]) < 0)
            {
                centre(up, exps, ranges);
            }
            level_up(
                f,
                (t, l, r),
                &views[t.idx()],
                up,
                &mut ranges[t.idx()],
                &mut self.routes_up,
            );
            // Values that came out too small for the products not all
            // normal, moved up as far as they stay finite and computed
            // again.
            let products = ranges[l.idx()].times(ranges[r.idx()]);
            if movable
                && !products
                && !exact_as_computed(
                    &up[t.idx()],
                    ranges[t.idx()],
                    lowest_product(ranges[l.idx()], ranges[r.idx()]),
                )
                && raising(ranges[t.idx()]) > 0
                && let Some(ks) =
                    split_shift(ranges[l.idx()], ranges[r.idx()], raising(ranges[t.idx()]))
            {
                shift(up, exps, ranges, ks);
                level_up(
                    f,
                    (t, l, r),
                    &views[t.idx()],
                    up,
                    &mut ranges[t.idx()],
                    &mut self.routes_up,
                );
            }
            exps[t.idx()] = exps[l.idx()] + exps[r.idx()];
            exact[t.idx()] = below
                && uniform
                && ranges[t.idx()].finite()
                && (products
                    || exact_as_computed(
                        &up[t.idx()],
                        ranges[t.idx()],
                        lowest_product(ranges[l.idx()], ranges[r.idx()]),
                    ));
            if stop
                && !exact[t.idx()]
                && std::iter::successors(Some(t), |&x| vtree.node(x).parent()).any(|x| x == output)
            {
                return Some(k);
            }
        }
        None
    }
}

/// Level `t`'s counts up from its children's, `l` and `r`, read as `view`
/// has it, and settled into a new `range`, a grid's as its plan stores
/// them; the pairs it read added to their route's in `routes`.
pub(super) fn level_up<V: Value>(
    f: &tididi::Tdd,
    (t, l, r): (
        tididi::vtree::VtreeIdx,
        tididi::vtree::VtreeIdx,
        tididi::vtree::VtreeIdx,
    ),
    view: &View,
    up: &mut [Vec<V>],
    range: &mut Range,
    routes: &mut [usize; ROUTES],
) {
    *range = Range::NEW;
    match view {
        View::Grid(grid) => {
            let mut out = std::mem::take(&mut up[t.idx()]);
            let kernel = grid.up(&up[l.idx()], &up[r.idx()], &mut out, range);
            up[t.idx()] = out;
            routes[Route::planned(kernel).index::<V>()] += grid.pairs();
        }
        View::Untried | View::Pairs => {
            let (ld, rd) = (f.level(l).child_decoder(), f.level(r).child_decoder());
            let mut out = std::mem::take(&mut up[t.idx()]);
            let (lc, rc) = (&up[l.idx()], &up[r.idx()]);
            let value =
                |p: tididi::diagram::ChildPair| side(lc, ld, p.left) * side(rc, rd, p.right);
            let (level, mut walked) = (f.level(t), 0);
            // A stored level's nodes are read as slices of its pairs, in a
            // loop apart from an implicit level's generated ones.
            match level.stored() {
                Some(stored) => {
                    for (i, node) in stored.nodes().iter().enumerate() {
                        let pairs = stored.of(node);
                        walked += pairs.len();
                        out[i] = pairs.iter().map(|&p| value(p)).sum();
                    }
                }
                None => {
                    for (i, pairs) in level.internal_inputs_iter() {
                        walked += pairs.len();
                        out[i] = pairs.map(value).sum();
                    }
                }
            }
            V::settle_all(&mut out, range);
            up[t.idx()] = out;
            routes[Route::Pairs.index::<V>()] += walked;
        }
    }
}

/// The pairs of level `t` of `d`.
pub(super) fn level_pairs(d: &tididi::Tdd, t: tididi::vtree::VtreeIdx) -> usize {
    let level = d.level(t);
    (0..level.nodes().len())
        .map(|i| level.pair_count_at(i))
        .sum()
}

/// The node of each level of `f` that is true over the level's variables,
/// if the level has one: the `One` node of a leaf, and an internal node
/// with the one pair of its children's true nodes. Every level outside a
/// formula's cone has one, since embedding copies it there.
pub(super) fn true_nodes(f: &tididi::Tdd) -> Vec<Option<usize>> {
    use tididi::diagram::{ChildRef, ONE_LEAF_IDX};
    let vtree = f.vtree();
    let mut top: Vec<Option<usize>> = vec![None; vtree.num_nodes()];
    for (leaf, _) in vtree.leaf_bottomup() {
        top[leaf.idx()] = Some(ONE_LEAF_IDX.idx());
    }
    for (t, l, r) in vtree.internal_bottomup() {
        let (Some(tl), Some(tr)) = (top[l.idx()], top[r.idx()]) else {
            continue;
        };
        let (ld, rd) = (f.level(l).child_decoder(), f.level(r).child_decoder());
        for (i, mut pairs) in f.level(t).internal_inputs_iter() {
            let Some(p) = pairs.next() else { continue };
            if pairs.next().is_some() {
                continue;
            }
            if matches!(ld.child(p.left), ChildRef::Node(n) if n.idx() == tl)
                && matches!(rd.child(p.right), ChildRef::Node(n) if n.idx() == tr)
            {
                top[t.idx()] = Some(i);
                break;
            }
        }
    }
    top
}

/// The pre-order spans of a vtree: `spans[t] = (first, last)` with every
/// node under `t` numbered in `first..=last`, so that `a` lies under `t`
/// iff `spans[t].0 <= spans[a].0 <= spans[t].1`.
pub(crate) fn spans(vtree: &tididi::vtree::Vtree) -> Vec<(u32, u32)> {
    let mut out = vec![(0, 0); vtree.num_nodes()];
    let mut next = 0u32;
    let mut stack = vec![(vtree.root(), false)];
    while let Some((t, done)) = stack.pop() {
        if done {
            out[t.idx()].1 = next - 1;
            continue;
        }
        out[t.idx()].0 = next;
        next += 1;
        stack.push((t, true));
        if !vtree.node(t).is_leaf() {
            let (l, r) = vtree.children(t);
            stack.push((r, false));
            stack.push((l, false));
        }
    }
    out
}

/// The weighted count of `e ∧ f`, without building the conjunction.
///
/// `e` is counted (`counted`, both passes) on a vtree whose pre-order
/// `spans` these are; `f` is counted on its own vtree, a projection of
/// `e`'s: `leaf_in_e` maps each of its variables to that variable's leaf
/// in `e`'s vtree. Every internal node of `f`'s vtree then has an image in
/// `e`'s, the lowest common ancestor of its leaves' images, and `f`
/// embedded onto `e`'s vtree would hold, at every level between an image
/// and its parent's, a node with one pair passing its node through on the
/// side the image lies and true on the other side, and true nodes
/// everywhere else. The count reads the two diagrams as they are, with
/// that embedding read off the images instead of built.
///
/// The count starts at the image of the level of `f`'s cone, below which
/// `f` is not one pair with a true side. It is the sum, over `e`'s nodes
/// `u` there, of the models through `u` (`down`) times the count of
/// `u ∧ v` under the level, `v` `f`'s node. Those counts are computed
/// bottom-up over `f`'s vtree under the cone, one dense table per level of
/// `f`: a row per node of `e` at the level's image, a column per node of
/// `f`. At a leaf the literals agree or contradict; at an internal level
/// an entry is the sum over the pairs of both nodes of the products of the
/// children's entries. A child's table is on its own image, which can lie
/// below the image's child on its side: there `f` passes its nodes
/// through, and the table is lifted level by level up that path, each
/// node of `e` on it summing its pairs' rows on the path side weighted by
/// the count of their other side.
///
/// Every pair of nodes of a level is computed, reached from the cone or
/// not: a memoized walk from the cone reaches nearly all of them, and a
/// hash entry per pair costs far more than the dense loop.
///
/// A variable of `f` that `e` does not hold (`leaf_in_e` gives `None`) is
/// summed inside `f`, since `e` does not read it. A subtree of `f`'s
/// vtree over such variables alone has no image and enters as its own
/// count, and a node with one on a side takes the image of its other side
/// and that side's table weighted by it. Embedding `e` onto a scope that
/// holds them would copy the whole evidence formula for a query's own few
/// variables.
///
/// The walk is outside the engine's operations, so it reports its own work:
/// `poll` is called with the multiply-adds of each table before it is
/// computed, and an error from it stops the walk.
///
/// The count comes under a power of two, the sum of those of the levels
/// and the column it read, and with whether it is exact: every count of
/// the two diagrams it read, and every product it took ([`Range`]).
pub(crate) fn conjunction_count<V: Value>(
    e: &tididi::Tdd,
    counted: &Counted<V>,
    spans: &[(u32, u32)],
    f: &tididi::Tdd,
    f_counted: &Counted<V>,
    leaf_in_e: impl Fn(VarId) -> Option<tididi::vtree::VtreeIdx>,
    mut poll: impl FnMut(u64) -> Result<(), tididi::OperationError>,
) -> Result<(Scaled<V>, bool), tididi::OperationError> {
    use tididi::diagram::{ChildRef, ONE_LEAF_IDX};
    use tididi::vtree::VtreeIdx;
    if e.is_zero() || f.is_zero() {
        return Ok((Scaled::new(V::ZERO, 0), true));
    }
    // Whether the counts read and the products taken so far are exact.
    let exact = Cell::new(true);
    let check = |b: bool| {
        if !b {
            exact.set(false);
        }
    };
    // The product of two values, checked.
    let times = |a: V, b: V| {
        check(Range::of(a).times(Range::of(b)));
        let mut range = Range::NEW;
        let x = (a * b).settle(&mut range);
        check(range.finite());
        x
    };
    let (ev, fv) = (e.vtree(), f.vtree());
    // The image in `e`'s vtree of each node of `f`'s, `None` over
    // variables `e` does not hold.
    let mut image: Vec<Option<VtreeIdx>> = vec![None; fv.num_nodes()];
    for (leaf, var) in fv.leaf_bottomup() {
        image[leaf.idx()] = leaf_in_e(var);
    }
    for (t, l, r) in fv.internal_bottomup() {
        image[t.idx()] = match (image[l.idx()], image[r.idx()]) {
            (Some(a), Some(b)) => Some(ev.lca(a, b)),
            (a, b) => a.or(b),
        };
    }
    let under = |a: VtreeIdx, t: VtreeIdx| {
        spans[t.idx()].0 <= spans[a.idx()].0 && spans[a.idx()].0 <= spans[t.idx()].1
    };
    // Down from `f`'s output while its node is true on one side: the
    // level to sum at and `f`'s node there.
    let (mut s, mut v) = (f.output().vtree, f.output().local.idx());
    loop {
        if fv.node(s).is_leaf() {
            break;
        }
        let (l, r) = fv.children(s);
        let (ld, rd) = (f.level(l).child_decoder(), f.level(r).child_decoder());
        let mut pairs = f.level(s).pairs_iter_of_idx(v);
        let Some(p) = pairs.next() else { break };
        if pairs.next().is_some() {
            break;
        }
        match (ld.child(p.left), rd.child(p.right)) {
            (ChildRef::Node(a), ChildRef::Node(b)) if Some(a.idx()) == f_counted.top[l.idx()] => {
                (s, v) = (r, b.idx())
            }
            (ChildRef::Node(a), ChildRef::Node(b)) if Some(b.idx()) == f_counted.top[r.idx()] => {
                (s, v) = (l, a.idx())
            }
            _ => break,
        }
    }
    // The variables `e` does not hold on the sides where `f` is true above
    // `s`, which neither diagram constrains.
    let (mut free, mut free_exp) = (V::ONE, 0);
    // Each walk's stack is allocated for every level of `f`'s vtree, more
    // than it holds.
    let mut stack = Vec::with_capacity(fv.num_nodes());
    stack.push(fv.root());
    while let Some(x) = stack.pop() {
        if x == s {
            continue;
        }
        if fv.node(x).is_leaf() {
            if image[x.idx()].is_none() {
                check(f_counted.exact[x.idx()]);
                free = times(free, f_counted.up[x.idx()][ONE_LEAF_IDX.idx()]);
                free_exp += f_counted.exps[x.idx()];
            }
        } else {
            let (xl, xr) = fv.children(x);
            stack.extend([xl, xr]);
        }
    }
    let Some(t_s) = image[s.idx()] else {
        check(counted.exact(e) && f_counted.exact[s.idx()]);
        let total = counted.total(e);
        let joint = times(times(total.value, f_counted.up[s.idx()][v]), free);
        return Ok((
            Scaled::new(joint, total.exp + f_counted.exps[s.idx()] + free_exp),
            exact.get(),
        ));
    };
    fn node(c: ChildRef) -> usize {
        match c {
            ChildRef::Node(n) => n.idx(),
            ChildRef::Value(_) => unreachable!("no level is summed out"),
        }
    }
    /// A level's nodes, each with its pairs as the slots of their children,
    /// read from the level as they are walked, not decoded into vectors
    /// per level first.
    fn pairs_at(
        d: &tididi::Tdd,
        t: VtreeIdx,
    ) -> impl Iterator<Item = (usize, impl Iterator<Item = (u32, u32)> + '_)> + '_ {
        let (l, r) = d.vtree().children(t);
        let (ld, rd) = (d.level(l).child_decoder(), d.level(r).child_decoder());
        d.level(t).internal_inputs_iter().map(move |(i, ps)| {
            (
                i,
                ps.map(move |p| {
                    (
                        node(ld.child(p.left)) as u32,
                        node(rd.child(p.right)) as u32,
                    )
                }),
            )
        })
    }
    /// A dense table: a row per node of `e` at a level, a column per node
    /// of `f` at another, every entry under the power of two `exp`, the
    /// sum of those of the levels whose counts its terms multiply. A table
    /// is settled once it is built, its entries noted in `range`.
    struct Table<V> {
        cols: usize,
        data: Vec<V>,
        range: Range,
        exp: i64,
    }
    impl<V: Value> Table<V> {
        fn new(rows: usize, cols: usize, exp: i64) -> Result<Self, tididi::OperationError> {
            Ok(Table {
                cols,
                data: V::zeros(rows * cols)?,
                range: Range::NEW,
                exp,
            })
        }
        fn row(&self, u: usize) -> &[V] {
            &self.data[u * self.cols..(u + 1) * self.cols]
        }
        fn settled(mut self) -> Self {
            V::settle_all(&mut self.data, &mut self.range);
            self
        }
        /// Where the products of the entries with values of range `range`
        /// fail [`Range::times`], the entries times the power of two that
        /// centres on one the entries alone, or with `joint` their products
        /// with those values, if they stay normal.
        fn fit(&mut self, range: Range, joint: bool) {
            if (centre_all() || !range.times(self.range))
                && let Some(k) = centring(self.range, if joint { range } else { Range::NEW })
                && k != 0
            {
                shift_all(&mut self.data, k, &mut self.range);
                self.exp -= k;
            }
        }
    }
    let up_e = &counted.up;
    // The table of `f`'s level `s`, on its image, from its children's.
    type Poll<'a> = &'a mut dyn FnMut(u64) -> Result<(), tididi::OperationError>;
    let product = |s: VtreeIdx,
                   left: Table<V>,
                   right: Table<V>,
                   poll: Poll|
     -> Result<Table<V>, tididi::OperationError> {
        let t = image[s.idx()].expect("both sides have images");
        let (l, r) = ev.children(t);
        let (sl, sr) = fv.children(s);
        let image_of = |x: VtreeIdx| image[x.idx()].expect("both sides have images");
        let straight = under(image_of(sl), l);
        // Each child's table lifted to the child of `t` on its side.
        let mut lift = |mut table: Table<V>,
                        from: VtreeIdx,
                        to: VtreeIdx|
         -> Result<Table<V>, tididi::OperationError> {
            if from == to {
                return Ok(table);
            }
            let mut path = vec![to];
            while *path.last().unwrap() != from {
                let x = *path.last().unwrap();
                let (xl, xr) = ev.children(x);
                path.push(if under(from, xl) { xl } else { xr });
            }
            let cols = table.cols;
            for w in path.windows(2).rev() {
                let (x, on) = (w[0], w[1]);
                let (xl, xr) = ev.children(x);
                let (on_left, off) = if on == xl { (true, xr) } else { (false, xl) };
                poll((level_pairs(e, x) * cols) as u64)?;
                table.fit(counted.ranges[off.idx()], true);
                let source = &table;
                check(counted.exact[off.idx()] && counted.ranges[off.idx()].times(source.range));
                let mut out = Table::new(
                    up_e[x.idx()].len(),
                    cols,
                    counted.exps[off.idx()] + source.exp,
                )?;
                for (u, pairs) in pairs_at(e, x) {
                    let row = &mut out.data[u * cols..(u + 1) * cols];
                    for (pl, pr) in pairs {
                        let (a, b) = if on_left { (pl, pr) } else { (pr, pl) };
                        let weight = up_e[off.idx()][b as usize];
                        if weight.is_zero() {
                            continue;
                        }
                        for (o, &x) in row.iter_mut().zip(source.row(a as usize)) {
                            *o += weight * x;
                        }
                    }
                }
                table = out.settled();
            }
            Ok(table)
        };
        let (fa, fb) = if straight {
            (left, right)
        } else {
            (right, left)
        };
        let (sa, sb) = if straight { (sl, sr) } else { (sr, sl) };
        let mut a_table = lift(fa, image_of(sa), l)?;
        let mut b_table = lift(fb, image_of(sb), r)?;
        // `f`'s pairs as (node, side under `l`, side under `r`).
        let f_flat: Vec<(u32, u32, u32)> = collected(
            level_pairs(f, s),
            pairs_at(f, s).flat_map(|(v, pairs)| {
                pairs.map(move |(pl, pr)| {
                    let (x, y) = if straight { (pl, pr) } else { (pr, pl) };
                    (v as u32, x, y)
                })
            }),
        )?;
        let cols = f_counted.up[s.idx()].len();
        poll((level_pairs(e, t) * f_flat.len()) as u64)?;
        a_table.fit(b_table.range, false);
        b_table.fit(a_table.range, false);
        check(a_table.range.times(b_table.range));
        let mut out = Table::new(up_e[t.idx()].len(), cols, a_table.exp + b_table.exp)?;
        for (u, pairs) in pairs_at(e, t) {
            let row = &mut out.data[u * cols..(u + 1) * cols];
            for (pl, pr) in pairs {
                let (ra, rb) = (a_table.row(pl as usize), b_table.row(pr as usize));
                for &(v, x, y) in &f_flat {
                    row[v as usize] += ra[x as usize] * rb[y as usize];
                }
            }
        }
        Ok(out.settled())
    };
    // The table of `f`'s level `s`, one side of which has no image: the
    // other side's table, on the same image, weighted by the first side's
    // count.
    let absorb = |s: VtreeIdx,
                  mut inner: Table<V>,
                  poll: Poll|
     -> Result<Table<V>, tididi::OperationError> {
        let t = image[s.idx()].expect("one side has an image");
        let (sl, sr) = fv.children(s);
        let outer_left = image[sl.idx()].is_none();
        let outer_level = if outer_left { sl } else { sr };
        let outer = &f_counted.up[outer_level.idx()];
        inner.fit(f_counted.ranges[outer_level.idx()], true);
        check(
            f_counted.exact[outer_level.idx()]
                && f_counted.ranges[outer_level.idx()].times(inner.range),
        );
        let (rows, cols) = (up_e[t.idx()].len(), f_counted.up[s.idx()].len());
        poll((rows * level_pairs(f, s)) as u64)?;
        let mut out = Table::new(rows, cols, f_counted.exps[outer_level.idx()] + inner.exp)?;
        for (v, pairs) in pairs_at(f, s) {
            for (pl, pr) in pairs {
                let (a, b) = if outer_left { (pl, pr) } else { (pr, pl) };
                let weight = outer[a as usize];
                if weight.is_zero() {
                    continue;
                }
                for u in 0..rows {
                    out.data[u * cols + v] += weight * inner.data[u * inner.cols + b as usize];
                }
            }
        }
        Ok(out.settled())
    };
    // Bottom-up over `f`'s vtree under `s`, each child's table dropped once
    // its parent's is built. A level with no image has no table.
    let mut tables: Vec<Option<Table<V>>> = (0..fv.num_nodes()).map(|_| None).collect();
    let mut stack = Vec::with_capacity(fv.num_nodes());
    stack.push((s, false));
    while let Some((x, done)) = stack.pop() {
        if image[x.idx()].is_none() {
            continue;
        }
        if fv.node(x).is_leaf() {
            let t = image[x.idx()].expect("tested above");
            let (rows, cols) = (up_e[t.idx()].len(), f_counted.up[x.idx()].len());
            poll((rows * cols) as u64)?;
            // Each entry a count of `e`'s at `t`, or one of `f`'s at `x`
            // times `e`'s weight over `f`'s: under `t`'s power either way.
            let mut table = Table::new(rows, cols, counted.exps[t.idx()])?;
            let top = f_counted.top[x.idx()];
            let e_top = counted.top[t.idx()];
            // The variable's weight over both sides, in each diagram.
            let (e_one, f_one) = (
                up_e[t.idx()][ONE_LEAF_IDX.idx()],
                f_counted.up[x.idx()][ONE_LEAF_IDX.idx()],
            );
            check(
                counted.exact[t.idx()]
                    && f_counted.exact[x.idx()]
                    && f_counted.ranges[x.idx()].times(Range::of(e_one)),
            );
            for (u, row) in table.data.chunks_mut(cols.max(1)).enumerate() {
                for (w, entry) in row.iter_mut().enumerate() {
                    *entry = if Some(w) == top {
                        up_e[t.idx()][u]
                    } else if Some(u) == e_top {
                        let up = f_counted.up[x.idx()][w];
                        if up.is_zero() {
                            V::ZERO
                        } else {
                            up * e_one / f_one
                        }
                    } else if u == w {
                        up_e[t.idx()][u]
                    } else {
                        V::ZERO
                    };
                }
            }
            tables[x.idx()] = Some(table.settled());
        } else if done {
            let (xl, xr) = fv.children(x);
            tables[x.idx()] = Some(match (tables[xl.idx()].take(), tables[xr.idx()].take()) {
                (Some(left), Some(right)) => product(x, left, right, &mut poll)?,
                (Some(inner), None) | (None, Some(inner)) => absorb(x, inner, &mut poll)?,
                (None, None) => unreachable!("a level with an image has a side with one"),
            });
        } else {
            let (xl, xr) = fv.children(x);
            stack.push((x, true));
            stack.push((xr, false));
            stack.push((xl, false));
        }
    }
    let mut table = tables[s.idx()].take().unwrap();
    let mut total = V::ZERO;
    let column = counted.down(e, t_s)?;
    // A column in `Count`s is read in `V`s where it fits under one power.
    let fit;
    let (values, range, held, exp): (&[V], Range, bool, i64) = match &column.wide {
        Some(wide) => match fitted::<V>(wide)? {
            Some((values, power)) => {
                let range;
                (fit, range) = settled(values);
                (&fit, range, true, power)
            }
            None => return Ok((Scaled::new(V::ZERO, 0), false)),
        },
        None => (&column.values, column.range, column.exact, column.exp),
    };
    table.fit(range, true);
    for (u, &context) in values.iter().enumerate() {
        if !context.is_zero() {
            total += context * table.row(u)[v];
        }
    }
    check(held && range.times(table.range));
    let mut sum = Range::NEW;
    let total = total.settle(&mut sum);
    check(sum.finite());
    let joint = times(total, free);
    Ok((Scaled::new(joint, exp + table.exp + free_exp), exact.get()))
}

/// Level `t` of `f`, over `l` and `r`, as a grid, if it holds `grid_min`
/// pairs or more and its sides name nodes of its children: an implicit
/// level off its description ([`Grid::implicit`]), a level that stores its
/// pairs where they fit one ([`Grid::fit`]).
pub(super) fn grid_of(
    f: &tididi::Tdd,
    (t, l, r): (
        tididi::vtree::VtreeIdx,
        tididi::vtree::VtreeIdx,
        tididi::vtree::VtreeIdx,
    ),
    grid_min: usize,
) -> Option<Grid> {
    let (level, ld, rd) = (
        f.level(t),
        f.level(l).child_decoder(),
        f.level(r).child_decoder(),
    );
    match level.implicit() {
        Some(d) => (d.pairs() >= grid_min
            && !(level.is_marginal() || ld.is_marginal() || rd.is_marginal()))
        .then(|| Grid::implicit(d)),
        None => Grid::fit(level, ld, rd, grid_min),
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
