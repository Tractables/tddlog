//! The count of the conjunction of two diagrams, one on a projection of
//! the other's vtree, without building it ([`conjunction_count`]).

use std::cell::Cell;

use tididi::vtree::VarId;

use super::*;

/// The index of the node a child reference names (no level is summed out).
fn node(c: tididi::diagram::ChildRef) -> usize {
    match c {
        tididi::diagram::ChildRef::Node(n) => n.idx(),
        tididi::diagram::ChildRef::Value(_) => unreachable!("no level is summed out"),
    }
}

/// A level's nodes, each with its pairs as the slots of their children,
/// read from the level as they are walked, not decoded into vectors
/// per level first.
fn pairs_at(
    d: &tididi::Tdd,
    t: tididi::vtree::VtreeIdx,
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

type Poll<'a> = &'a mut dyn FnMut(u64) -> Result<(), tididi::OperationError>;

struct ConjunctionWalk<'a, V> {
    e: &'a tididi::Tdd,
    counted: &'a Counted<V>,
    spans: &'a [(u32, u32)],
    f: &'a tididi::Tdd,
    f_counted: &'a Counted<V>,
    image: Vec<Option<tididi::vtree::VtreeIdx>>,
    exact: Cell<bool>,
}

impl<V: Value> ConjunctionWalk<'_, V> {
    fn check(&self, b: bool) {
        if !b {
            self.exact.set(false);
        }
    }

    /// The product of two values, checked.
    fn times(&self, a: V, b: V) -> V {
        self.check(Range::of(a).times(Range::of(b)));
        let mut range = Range::NEW;
        let x = (a * b).settle(&mut range);
        self.check(range.finite());
        x
    }

    fn under(&self, a: tididi::vtree::VtreeIdx, t: tididi::vtree::VtreeIdx) -> bool {
        self.spans[t.idx()].0 <= self.spans[a.idx()].0
            && self.spans[a.idx()].0 <= self.spans[t.idx()].1
    }

    /// Each child's table lifted to the child of `t` on its side.
    fn lift(
        &self,
        mut table: Table<V>,
        from: tididi::vtree::VtreeIdx,
        to: tididi::vtree::VtreeIdx,
        poll: Poll,
    ) -> Result<Table<V>, tididi::OperationError> {
        if from == to {
            return Ok(table);
        }
        let ev = self.e.vtree();
        let up_e = &self.counted.up;
        let mut path = vec![to];
        while *path.last().unwrap() != from {
            let x = *path.last().unwrap();
            let (xl, xr) = ev.children(x);
            path.push(if self.under(from, xl) { xl } else { xr });
        }
        let cols = table.cols;
        for w in path.windows(2).rev() {
            let (x, on) = (w[0], w[1]);
            let (xl, xr) = ev.children(x);
            let (on_left, off) = if on == xl { (true, xr) } else { (false, xl) };
            poll((level_pairs(self.e, x) * cols) as u64)?;
            table.fit(self.counted.ranges[off.idx()], true);
            let source = &table;
            self.check(
                self.counted.exact[off.idx()] && self.counted.ranges[off.idx()].times(source.range),
            );
            let mut out = Table::new(
                up_e[x.idx()].len(),
                cols,
                self.counted.exps[off.idx()] + source.exp,
            )?;
            for (u, pairs) in pairs_at(self.e, x) {
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
    }

    /// The table of `f`'s level `s`, on its image, from its children's.
    fn product(
        &self,
        s: tididi::vtree::VtreeIdx,
        left: Table<V>,
        right: Table<V>,
        poll: Poll,
    ) -> Result<Table<V>, tididi::OperationError> {
        let (ev, fv) = (self.e.vtree(), self.f.vtree());
        let up_e = &self.counted.up;
        let t = self.image[s.idx()].expect("both sides have images");
        let (l, r) = ev.children(t);
        let (sl, sr) = fv.children(s);
        let image_of =
            |x: tididi::vtree::VtreeIdx| self.image[x.idx()].expect("both sides have images");
        let straight = self.under(image_of(sl), l);
        let (fa, fb) = if straight {
            (left, right)
        } else {
            (right, left)
        };
        let (sa, sb) = if straight { (sl, sr) } else { (sr, sl) };
        let mut a_table = self.lift(fa, image_of(sa), l, poll)?;
        let mut b_table = self.lift(fb, image_of(sb), r, poll)?;
        // `f`'s pairs as (node, side under `l`, side under `r`).
        let f_flat: Vec<(u32, u32, u32)> = collected(
            level_pairs(self.f, s),
            pairs_at(self.f, s).flat_map(|(v, pairs)| {
                pairs.map(move |(pl, pr)| {
                    let (x, y) = if straight { (pl, pr) } else { (pr, pl) };
                    (v as u32, x, y)
                })
            }),
        )?;
        let cols = self.f_counted.up[s.idx()].len();
        poll((level_pairs(self.e, t) * f_flat.len()) as u64)?;
        a_table.fit(b_table.range, false);
        b_table.fit(a_table.range, false);
        self.check(a_table.range.times(b_table.range));
        let mut out = Table::new(up_e[t.idx()].len(), cols, a_table.exp + b_table.exp)?;
        for (u, pairs) in pairs_at(self.e, t) {
            let row = &mut out.data[u * cols..(u + 1) * cols];
            for (pl, pr) in pairs {
                let (ra, rb) = (a_table.row(pl as usize), b_table.row(pr as usize));
                for &(v, x, y) in &f_flat {
                    row[v as usize] += ra[x as usize] * rb[y as usize];
                }
            }
        }
        Ok(out.settled())
    }

    /// The table of `f`'s level `s`, one side of which has no image: the
    /// other side's table, on the same image, weighted by the first side's
    /// count.
    fn absorb(
        &self,
        s: tididi::vtree::VtreeIdx,
        mut inner: Table<V>,
        poll: Poll,
    ) -> Result<Table<V>, tididi::OperationError> {
        let t = self.image[s.idx()].expect("one side has an image");
        let (sl, sr) = self.f.vtree().children(s);
        let outer_left = self.image[sl.idx()].is_none();
        let outer_level = if outer_left { sl } else { sr };
        let outer = &self.f_counted.up[outer_level.idx()];
        inner.fit(self.f_counted.ranges[outer_level.idx()], true);
        self.check(
            self.f_counted.exact[outer_level.idx()]
                && self.f_counted.ranges[outer_level.idx()].times(inner.range),
        );
        let (rows, cols) = (
            self.counted.up[t.idx()].len(),
            self.f_counted.up[s.idx()].len(),
        );
        poll((rows * level_pairs(self.f, s)) as u64)?;
        let mut out = Table::new(
            rows,
            cols,
            self.f_counted.exps[outer_level.idx()] + inner.exp,
        )?;
        for (v, pairs) in pairs_at(self.f, s) {
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
    }

    fn leaf_table(
        &self,
        x: tididi::vtree::VtreeIdx,
        poll: Poll,
    ) -> Result<Table<V>, tididi::OperationError> {
        use tididi::diagram::ONE_LEAF_IDX;
        let up_e = &self.counted.up;
        let t = self.image[x.idx()].expect("tested above");
        let (rows, cols) = (up_e[t.idx()].len(), self.f_counted.up[x.idx()].len());
        poll((rows * cols) as u64)?;
        // Each entry a count of `e`'s at `t`, or one of `f`'s at `x`
        // times `e`'s weight over `f`'s: under `t`'s power either way.
        let mut table = Table::new(rows, cols, self.counted.exps[t.idx()])?;
        let top = self.f_counted.top[x.idx()];
        let e_top = self.counted.top[t.idx()];
        // The variable's weight over both sides, in each diagram.
        let (e_one, f_one) = (
            up_e[t.idx()][ONE_LEAF_IDX.idx()],
            self.f_counted.up[x.idx()][ONE_LEAF_IDX.idx()],
        );
        self.check(
            self.counted.exact[t.idx()]
                && self.f_counted.exact[x.idx()]
                && self.f_counted.ranges[x.idx()].times(Range::of(e_one)),
        );
        for (u, row) in table.data.chunks_mut(cols.max(1)).enumerate() {
            for (w, entry) in row.iter_mut().enumerate() {
                *entry = if Some(w) == top {
                    up_e[t.idx()][u]
                } else if Some(u) == e_top {
                    let up = self.f_counted.up[x.idx()][w];
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
        Ok(table.settled())
    }

    /// Down from `f`'s output while its node is true on one side: the level
    /// to sum at, `f`'s node there, and the weight of the variables `e`
    /// does not hold on the sides where `f` is true above it.
    fn cone_root(&self) -> (tididi::vtree::VtreeIdx, usize, V, i64) {
        use tididi::diagram::{ChildRef, ONE_LEAF_IDX};
        let (f, fv, f_counted) = (self.f, self.f.vtree(), self.f_counted);
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
                (ChildRef::Node(a), ChildRef::Node(b))
                    if Some(a.idx()) == f_counted.top[l.idx()] =>
                {
                    (s, v) = (r, b.idx())
                }
                (ChildRef::Node(a), ChildRef::Node(b))
                    if Some(b.idx()) == f_counted.top[r.idx()] =>
                {
                    (s, v) = (l, a.idx())
                }
                _ => break,
            }
        }
        let (mut free, mut free_exp) = (V::ONE, 0);
        let mut stack = Vec::with_capacity(fv.num_nodes());
        stack.push(fv.root());
        while let Some(x) = stack.pop() {
            if x == s {
                continue;
            }
            if fv.node(x).is_leaf() {
                if self.image[x.idx()].is_none() {
                    self.check(f_counted.exact[x.idx()]);
                    free = self.times(free, f_counted.up[x.idx()][ONE_LEAF_IDX.idx()]);
                    free_exp += f_counted.exps[x.idx()];
                }
            } else {
                let (xl, xr) = fv.children(x);
                stack.extend([xl, xr]);
            }
        }
        (s, v, free, free_exp)
    }
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
    use tididi::vtree::VtreeIdx;
    if e.is_zero() || f.is_zero() {
        return Ok((Scaled::new(V::ZERO, 0), true));
    }
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
    let walk = ConjunctionWalk {
        e,
        counted,
        spans,
        f,
        f_counted,
        image,
        exact: Cell::new(true),
    };
    let (s, v, free, free_exp) = walk.cone_root();
    let Some(t_s) = walk.image[s.idx()] else {
        walk.check(counted.exact(e) && f_counted.exact[s.idx()]);
        let total = counted.total(e);
        let joint = walk.times(walk.times(total.value, f_counted.up[s.idx()][v]), free);
        return Ok((
            Scaled::new(joint, total.exp + f_counted.exps[s.idx()] + free_exp),
            walk.exact.get(),
        ));
    };
    // Bottom-up over `f`'s vtree under `s`, each child's table dropped once
    // its parent's is built. A level with no image has no table.
    let mut tables: Vec<Option<Table<V>>> = (0..fv.num_nodes()).map(|_| None).collect();
    let mut stack = Vec::with_capacity(fv.num_nodes());
    stack.push((s, false));
    while let Some((x, done)) = stack.pop() {
        if walk.image[x.idx()].is_none() {
            continue;
        }
        if fv.node(x).is_leaf() {
            tables[x.idx()] = Some(walk.leaf_table(x, &mut poll)?);
        } else if done {
            let (xl, xr) = fv.children(x);
            tables[x.idx()] = Some(match (tables[xl.idx()].take(), tables[xr.idx()].take()) {
                (Some(left), Some(right)) => walk.product(x, left, right, &mut poll)?,
                (Some(inner), None) | (None, Some(inner)) => walk.absorb(x, inner, &mut poll)?,
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
    walk.check(held && range.times(table.range));
    let mut sum = Range::NEW;
    let total = total.settle(&mut sum);
    walk.check(sum.finite());
    let joint = walk.times(total, free);
    Ok((
        Scaled::new(joint, exp + table.exp + free_exp),
        walk.exact.get(),
    ))
}
