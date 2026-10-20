use std::sync::Arc;

use tididi::diagram::{ChildPair, NEG_LEAF_IDX, NodeIdx, POS_LEAF_IDX, TddBuilder, TddNodeId};
use tididi::vtree::{Vtree, VtreeIdx};
use tididi::{Engine, Tdd};

use super::*;

/// How [`table`] departs from a grid with affine offsets at level T.
#[derive(Clone, Copy, PartialEq)]
enum Flaw {
    None,
    /// Every node's last two pairs swapped: the offsets are a grid's
    /// list.
    Listed,
    /// One pair of node 5 names another node on the right.
    Moved,
    /// Node 9 lacks its last pair.
    Short,
}

/// The count passes give the bits of the passes over pairs, in floats and
/// in [`Count`]s under `w`, with each level of `f` that is a grid read as
/// one, in turn, in each way its passes may run (`Grid::variants`): every
/// count up, every column down and every literal's count. Panics where
/// they do not; returns the ways run.
fn grids_agree_with_pairs(f: &Tdd, w: &Probabilities) -> usize {
    fn passes<V: Value>(
        f: &Tdd,
        w: &Probabilities,
        view: Option<(VtreeIdx, &Grid)>,
    ) -> Vec<String> {
        let mut c = Counted::<V>::leaves(f, w, usize::MAX).unwrap();
        // Every level over its pairs, an implicit one's off its description,
        // but `t` as the grid `view`.
        c.down.get_mut().views.fill(View::Pairs);
        if let Some((t, grid)) = view {
            c.down.get_mut().views[t.idx()] = View::Grid(Box::new(grid.clone()));
        }
        c.up_from(f, 0, false);
        let mut out: Vec<String> =
            c.up.iter()
                .zip(&c.exps)
                .map(|(up, k)| format!("up {up:?} {k}"))
                .collect();
        for t in 0..f.vtree().num_nodes() {
            let column = c.down(f, VtreeIdx(t as u32)).unwrap();
            out.push(format!(
                "down {:?} {:?} {} {}",
                &*column.values, column.range, column.exact, column.exp
            ));
        }
        for (leaf, var) in f.vtree().leaf_bottomup() {
            out.push(format!(
                "{var:?} {:?}",
                c.with_literal(f, leaf, var, true, w).unwrap()
            ));
        }
        out
    }
    fn agree<V: Value>(f: &Tdd, w: &Probabilities, t: VtreeIdx, ways: &[Grid]) {
        let pairs = passes::<V>(f, w, None);
        for (i, grid) in ways.iter().enumerate() {
            for (a, b) in passes::<V>(f, w, Some((t, grid))).iter().zip(&pairs) {
                assert_eq!(a, b, "level {t:?}, way {i} of {}: {grid:?}", ways.len());
            }
        }
    }
    let mut runs = 0;
    if f.is_zero() {
        return runs;
    }
    for (t, l, r) in f.vtree().internal_bottomup() {
        let Some(grid) = grid_of(f, (t, l, r), 0) else {
            continue;
        };
        let ways = grid.variants();
        agree::<f64>(f, w, t, &ways);
        agree::<Count>(f, w, t, &ways);
        runs += ways.len();
    }
    runs
}

/// Every assignment of the variables under `t` as a node of level `t`,
/// each of one pair, in the order of the right child's nodes, then the
/// left's; a leaf's two literals.
fn complete(b: &mut TddBuilder, eng: &Engine, vtree: &Vtree, t: VtreeIdx) -> Vec<NodeIdx> {
    if vtree.node(t).is_leaf() {
        return vec![POS_LEAF_IDX, NEG_LEAF_IDX];
    }
    let (l, r) = vtree.children(t);
    let (ls, rs) = (complete(b, eng, vtree, l), complete(b, eng, vtree, r));
    rs.iter()
        .flat_map(|&y| ls.iter().map(move |&x| (x, y)))
        .map(|(x, y)| b.push(eng, t, &[ChildPair::new(x, y)]).unwrap())
        .collect()
}

/// A diagram whose level T, the root's left child, is a table of 32
/// nodes of 4 pairs, over A, T's left child, whose 16 nodes are the
/// assignments of four variables, and B, T's right, whose 32 nodes are
/// those of five: node `n1 + 4 n0 + 16 n2` has the pairs
/// `(A n0 + 4 e, B n1 + 4 e + 16 n2)`, `e` from 0 to 3. The output, at
/// the root, has a pair with each node of T, the first four's with the
/// last variable's positive literal and the others' with its negative.
/// Each A node is the left of eight pairs and each B node the right of
/// four, so the order in which the pass down adds their terms shows in
/// the bits. Returns T too.
fn table(eng: &Engine, flaw: Flaw) -> (Tdd, VtreeIdx) {
    let text = "vtree 19\nL 0 1\nL 1 2\nI 2 0 1\nL 3 3\nL 4 4\nI 5 3 4\nI 6 2 5\nL 7 5\nL 8 6\nI 9 7 8\nL 10 7\nL 11 8\nI 12 10 11\nL 13 9\nI 14 12 13\nI 15 9 14\nI 16 6 15\nL 17 10\nI 18 16 17\n";
    let vtree = Arc::new(Vtree::from_text(text).unwrap());
    let root = vtree.root();
    let t = vtree.children(root).0;
    let (ta, tb) = vtree.children(t);
    let mut b = Tdd::builder(eng, &vtree).unwrap();
    let a = complete(&mut b, eng, &vtree, ta);
    let c = complete(&mut b, eng, &vtree, tb);
    assert_eq!((a.len(), c.len()), (16, 32));
    let nodes: Vec<NodeIdx> = (0..32)
        .map(|u| {
            let (n1, n0, n2) = (u % 4, u / 4 % 4, u / 16);
            let mut pairs: Vec<ChildPair> = (0..4)
                .map(|e| ChildPair::new(a[n0 + 4 * e], c[n1 + 4 * e + 16 * n2]))
                .collect();
            match flaw {
                Flaw::Listed => pairs.swap(2, 3),
                Flaw::Moved if u == 5 => {
                    pairs[2] = ChildPair::new(a[n0 + 8], c[(n1 + 9 + 16 * n2) % 32])
                }
                Flaw::Short if u == 9 => drop(pairs.pop()),
                _ => {}
            }
            b.push(eng, t, &pairs).unwrap()
        })
        .collect();
    let pairs: Vec<ChildPair> = nodes
        .iter()
        .enumerate()
        .map(|(k, &u)| ChildPair::new(u, if k < 4 { POS_LEAF_IDX } else { NEG_LEAF_IDX }))
        .collect();
    let out = b.push(eng, root, &pairs).unwrap();
    (
        b.finish(TddNodeId {
            vtree: root,
            local: out,
        })
        .unwrap(),
        t,
    )
}

/// The count passes over grids give the bits the passes over pairs
/// give: every count up, every column down and every literal's count,
/// in floats and in `Count`s, under weights that keep floats in range
/// and under weights whose products leave it. Every level of [`table`]
/// is a grid, level T's offsets affine or a list, and each level's
/// passes run in every way they may run, through every kernel.
#[test]
fn grids_count_to_the_bit_of_pairs() {
    let weights = |scale: f64| {
        let mut w = Probabilities {
            pos: vec![0.5; 11],
            neg: vec![0.5; 11],
            one: vec![1.0; 11],
        };
        for v in 1..=10 {
            let p = (v as f64 * 0.618_033_988_75).fract() * 0.9 + 0.05;
            let s = if (5..=9).contains(&v) { scale } else { 1.0 };
            (w.pos[v], w.neg[v], w.one[v]) = (p * s, (1.0 - p) * s * 0.37, s);
        }
        w
    };
    let eng = Engine::new();
    for flaw in [Flaw::None, Flaw::Listed] {
        let (f, t) = table(&eng, flaw);
        let (l, r) = f.vtree().children(t);
        let grid = grid_of(&f, (t, l, r), 0).unwrap();
        let ways = grid.variants();
        assert!(ways.len() > 50, "{} ways", ways.len());
        for pass in 0..3 {
            assert!(
                ways.iter().any(|g| g.kernels()[pass]),
                "some way runs pass {pass} of T through a kernel"
            );
        }
        for scale in [1.0, 1e-90] {
            assert!(grids_agree_with_pairs(&f, &weights(scale)) > ways.len());
        }
    }
}

/// Where the last variable's negative literal weighs nothing, all but
/// four of level T's contexts are zero ([`table`]), and its pass down
/// walks the grid node by node: the passes give the bits of the passes
/// over pairs, in floats and in `Count`s, in every way they may run.
#[test]
fn a_sparse_column_walks_its_grid() {
    let mut w = Probabilities {
        pos: vec![0.5; 11],
        neg: vec![0.5; 11],
        one: vec![1.0; 11],
    };
    for v in 1..=10 {
        let p = (v as f64 * 0.618_033_988_75).fract() * 0.9 + 0.05;
        (w.pos[v], w.neg[v]) = (p, 1.0 - p);
    }
    w.neg[10] = 0.0;
    let eng = Engine::new();
    let (f, t) = table(&eng, Flaw::None);
    assert!(grids_agree_with_pairs(&f, &w) > 0);
    let mut c = Counted::<f64>::leaves(&f, &w, 0).unwrap();
    c.up_from(&f, 0, false);
    assert_eq!(
        c.down(&f, t)
            .unwrap()
            .values
            .iter()
            .filter(|x| **x != 0.0)
            .count(),
        4
    );
    for k in 0..f.vtree().num_nodes() {
        c.down(&f, VtreeIdx(k as u32)).unwrap();
    }
    let routes = c.routes();
    assert!(
        routes.down[Route::Walk.index::<f64>()] >= 2 * 4 * 4,
        "{routes:?}"
    );
}

/// Where the processor lacks AVX2, every grid runs its plans through
/// the scalar loops and no kernel: the counts up and the columns down
/// of [`table`] are the bits of the passes over pairs, as they are
/// through the kernels.
#[test]
fn grids_fall_back_to_scalar_loops_without_avx2() {
    let mut w = Probabilities {
        pos: vec![0.5; 11],
        neg: vec![0.5; 11],
        one: vec![1.0; 11],
    };
    for v in 1..=10 {
        let p = (v as f64 * 0.618_033_988_75).fract() * 0.9 + 0.05;
        (w.pos[v], w.neg[v]) = (p, 1.0 - p);
    }
    let eng = Engine::new();
    let (f, _) = table(&eng, Flaw::None);
    let n = f.vtree().num_nodes();
    let counted = |grid_min: usize, no_avx2: bool| {
        kernel::NO_AVX2.set(no_avx2);
        let mut c = Counted::<f64>::leaves(&f, &w, grid_min).unwrap();
        if grid_min == usize::MAX {
            // Every level over its pairs, the implicit one too.
            c.down.get_mut().views.fill(View::Pairs);
        }
        c.up_from(&f, 0, false);
        let down: Vec<(Vec<u64>, i64)> = (0..n)
            .map(|k| {
                let column = c.down(&f, VtreeIdx(k as u32)).unwrap();
                (
                    column.values.iter().map(|x| x.to_bits()).collect(),
                    column.exp,
                )
            })
            .collect();
        kernel::NO_AVX2.set(false);
        let up: Vec<(Vec<u64>, i64)> =
            c.up.iter()
                .zip(&c.exps)
                .map(|(column, &k)| (column.iter().map(|x| x.to_bits()).collect(), k))
                .collect();
        (up, down, c.routes())
    };
    let (up, down, _) = counted(usize::MAX, false);
    let (scalar_up, scalar_down, scalar) = counted(0, true);
    assert_eq!((scalar_up, scalar_down), (up.clone(), down.clone()));
    let (loops, kernels) = (Route::Loops.index::<f64>(), Route::Kernel.index::<f64>());
    assert!(
        scalar.up[loops] > 0 && scalar.up[kernels] == 0 && scalar.down[kernels] == 0,
        "{scalar:?}"
    );
    #[cfg(target_arch = "x86_64")]
    if std::arch::is_x86_feature_detected!("avx2") {
        let (kernel_up, kernel_down, routes) = counted(0, false);
        assert_eq!((kernel_up, kernel_down), (up, down));
        assert!(routes.up[kernels] > 0, "{routes:?}");
    }
}

/// A level one of whose pairs is off the grid, or one of whose nodes
/// has fewer pairs, is read pair by pair. The library holds an affine
/// level as its description, which the fit leaves to
/// [`grid::Grid::implicit`]; it fits a level that lists its offsets.
#[test]
fn a_level_off_the_grid_is_not_fitted() {
    let eng = Engine::new();
    for flaw in [Flaw::None, Flaw::Listed, Flaw::Moved, Flaw::Short] {
        let (f, t) = table(&eng, flaw);
        let (l, r) = f.vtree().children(t);
        assert_eq!(f.level(t).implicit().is_some(), flaw == Flaw::None);
        let fit = grid::Grid::fit(
            f.level(t),
            f.level(l).child_decoder(),
            f.level(r).child_decoder(),
            0,
        );
        assert_eq!(fit.is_some(), flaw == Flaw::Listed);
        assert_eq!(
            grid_of(&f, (t, l, r), 0).is_some(),
            matches!(flaw, Flaw::None | Flaw::Listed)
        );
        let fit = grid::Grid::fit(
            f.level(t),
            f.level(l).child_decoder(),
            f.level(r).child_decoder(),
            129,
        );
        assert!(fit.is_none(), "fewer pairs than asked for");
    }
}

/// A count's bits: its significand and its binary exponent.
fn bits(c: Count) -> (u64, i64) {
    if c.is_zero() {
        return (0, 0);
    }
    (
        c.m.to_bits() & ((1 << 52) - 1),
        exponent(c.m) + i64::from(BLOCK) * i64::from(c.b),
    )
}

/// How [`count_exact`] counted, and the reads against the count.
#[derive(Debug, PartialEq)]
enum Way {
    /// In floats, every level and column under no power.
    Plain,
    /// In floats, some level or column under a power of two.
    Scaled,
    /// In floats, with `up` some level and with `down` some column
    /// counted again in `Count`s, and kept in floats under a power of
    /// its own; with `wide` some column kept in `Count`s.
    Recounted { up: bool, down: bool, wide: bool },
    /// In floats up, and in `Count`s down from the output once a
    /// column spans more than a float ([`Counts::widen`]).
    Widened,
    /// In `Count`s from a level partway up that spans more than a float.
    Switched,
}

/// Where a count in floats underflows partway up, `count_exact` keeps
/// the levels and columns under powers of two. Where a level's or a
/// column's children's extremes take products out of the floats and
/// its values do not show that none it took left them, it counts that
/// one in `Count`s and keeps it in floats where its own values fit.
/// Where a level's do not, it takes the levels under it over and goes
/// on in `Count`s from it; where a column's do not, the column stays in
/// `Count`s, and so do the columns under it that do not fit either.
/// Every way, the total and every literal's count are those of a count
/// in `Count`s from the leaves, bit for bit. Where nothing underflows
/// it is the count in floats, under no power. At 1e-9 the columns deep
/// in the cube or its negation span as much as the cube, 2 392 bits;
/// at 2^-49, 3 871, and a column in `Count`s spreads into columns in
/// `Count`s over levels whose products would leave the floats unless
/// each column is normalized.
#[test]
fn a_count_keeps_levels_under_powers_of_two_or_switches_to_counts() {
    let cube = |vtree: &Arc<Vtree>, vars: std::ops::RangeInclusive<i32>, sign: i32| {
        Tdd::cube(vtree, vars.map(|v| sign * v).collect::<Vec<_>>()).unwrap()
    };
    // Variables 1 to `k` weighing `p(v)` positive and `1 - p(v)`
    // negative.
    let weights = |k: usize, p: &dyn Fn(usize) -> f64| {
        let mut w = Probabilities {
            pos: vec![0.5; k + 1],
            neg: vec![0.5; k + 1],
            one: vec![1.0; k + 1],
        };
        for v in 1..=k {
            (w.pos[v], w.neg[v]) = (p(v), 1.0 - p(v));
        }
        w
    };
    // A cube of 80 variables, or the cube or its negation, and a clause
    // of three more. The root's children each hold a node of the cube,
    // p^41 and p^39, and one of its negation, near one: at p = 1e-9,
    // 1 226 and 1 166 bits apart, whose products span more than a
    // float, and the levels deep down have columns that span as much
    // as the cube; at p = 1e-16 the root's children themselves span
    // more.
    let n = 80;
    let v83 = Arc::new(Vtree::balanced(n as u32 + 3));
    let clause = Tdd::clause(&v83, [n + 1, -(n + 2), n + 3]).unwrap();
    let one = tididi::and(cube(&v83, 1..=n, 1), clause.clone()).unwrap();
    let both = tididi::and(
        tididi::or(cube(&v83, 1..=n, 1), cube(&v83, 1..=n, -1)).unwrap(),
        clause,
    )
    .unwrap();
    let cube_at = |p: f64| {
        weights(n as usize + 3, &move |v| {
            if v <= n as usize { p } else { 0.3 }
        })
    };
    // Two cubes of 40 variables crossed, each true where the other's
    // negation is: the root's children span 1 196 bits each at 1e-9,
    // their extremes' products 2 392, but the root's one node takes
    // the products of a cube and a negation only, near 2^-1196, and is
    // exact as computed.
    let v80 = Arc::new(Vtree::balanced(80));
    let crossed = tididi::or(
        tididi::and(cube(&v80, 1..=40, 1), cube(&v80, 41..=80, -1)).unwrap(),
        tididi::and(cube(&v80, 1..=40, -1), cube(&v80, 41..=80, 1)).unwrap(),
    )
    .unwrap();
    // Over a, b and c, 30, 30 and 60 variables whose cubes weigh about
    // 2^-500, 2^-1100 and 2^-1100: `(a ∧ ¬b ∧ c) ∨ (¬a ∧ b ∧ ¬c)`. The
    // level over a and b has the column 2^-1100 at `a ∧ ¬b` and about
    // one at `¬a ∧ b`, and b's counts up span 1 100 bits, but the
    // column's least value meets b's greatest only: a's column is exact
    // as computed.
    let v120 = Arc::new(Vtree::balanced(120));
    let (a, b, c) = (
        |s| cube(&v120, 1..=30, s),
        |s| cube(&v120, 31..=60, s),
        |s| cube(&v120, 61..=120, s),
    );
    let and3 = |x: Tdd, y: Tdd, z: Tdd| tididi::and(tididi::and(x, y).unwrap(), z).unwrap();
    let three = tididi::or(and3(a(1), b(-1), c(1)), and3(a(-1), b(1), c(-1))).unwrap();
    let three_w = weights(120, &|v| {
        f64::exp2(
            -(if v <= 30 {
                500.0 / 30.0
            } else if v <= 60 {
                1100.0 / 30.0
            } else {
                1100.0 / 60.0
            }),
        )
    });
    for (what, f, w, route) in [
        ("cube at 1/4", &one, cube_at(0.25), Way::Plain),
        ("cube at 1e-9", &one, cube_at(1e-9), Way::Scaled),
        (
            "two cubes crossed at 1e-9",
            &crossed,
            weights(80, &|_| 1e-9),
            Way::Scaled,
        ),
        ("three cubes", &three, three_w, Way::Scaled),
        (
            "cube or its negation at 1e-9",
            &both,
            cube_at(1e-9),
            Way::Recounted {
                up: false,
                down: true,
                wide: true,
            },
        ),
        (
            "cube or its negation at 2^-49",
            &both,
            cube_at(f64::exp2(-49.0)),
            Way::Recounted {
                up: true,
                down: true,
                wide: true,
            },
        ),
        (
            "cube or its negation at 1e-16",
            &both,
            cube_at(1e-16),
            Way::Switched,
        ),
    ] {
        let wide = count::<Count>(f, &w).unwrap();
        let mut recounts = 0;
        let mut counts = count_exact(f, &w, &mut recounts).unwrap();
        let switched = matches!(counts, Counts::Wide(_));
        // Each literal's count, read as a query reads it: against the
        // count in floats, and where that is not exact, against the
        // count taken over in `Count`s.
        let mut literals = Vec::new();
        let mut widened = false;
        for (leaf, var) in f.vtree().leaf_bottomup() {
            if let Counts::Plain(plain) = &counts
                && let Some(x) = plain.with_literal(f, leaf, var, true, &w).unwrap()
            {
                literals.push(Some(x.count()));
                continue;
            }
            if let Counts::Plain(_) = counts {
                counts.widen().unwrap();
                widened = true;
            }
            let Counts::Wide(wide) = &counts else {
                unreachable!("widened")
            };
            literals.push(
                wide.with_literal(f, leaf, var, true, &w)
                    .unwrap()
                    .map(Scaled::count),
            );
        }
        let total = match &counts {
            Counts::Plain(plain) => plain.total(f).count(),
            Counts::Wide(wide) => wide.total(f).count(),
        };
        let (up, down) = counts.routes().in_counts();
        let got = match &counts {
            _ if switched => {
                let all: usize = f
                    .vtree()
                    .internal_bottomup()
                    .map(|(t, _, _)| level_pairs(f, t))
                    .sum();
                assert!(
                    0 < up && up < all,
                    "{what}: {up} of {all} pairs up in counts"
                );
                Way::Switched
            }
            _ if widened => Way::Widened,
            Counts::Plain(plain) if up + down > 0 => {
                let wide = (0..plain.exps.len())
                    .any(|t| plain.down(f, VtreeIdx(t as u32)).unwrap().wide.is_some());
                Way::Recounted {
                    up: up > 0,
                    down: down > 0,
                    wide,
                }
            }
            Counts::Plain(plain) => {
                let scaled = plain.exps.iter().any(|&k| k != 0)
                    || (0..plain.exps.len())
                        .any(|t| plain.down(f, VtreeIdx(t as u32)).unwrap().exp != 0);
                if scaled { Way::Scaled } else { Way::Plain }
            }
            Counts::Wide(_) => unreachable!("a count in `Count`s is switched or widened"),
        };
        assert_eq!(got, route, "{what}");
        assert_eq!(recounts, usize::from(route == Way::Switched), "{what}");
        assert_eq!(
            bits(total),
            bits(wide.total(f).count()),
            "{what}: the total"
        );
        for ((leaf, var), a) in f.vtree().leaf_bottomup().zip(literals) {
            let b = wide
                .with_literal(f, leaf, var, true, &w)
                .unwrap()
                .map(Scaled::count);
            assert_eq!(a.map(bits), b.map(bits), "{what}: literal {var:?}");
        }
        if route == Way::Plain {
            let plain = count::<f64>(f, &w).unwrap();
            assert_eq!(total.to_f64(), plain.total(f).value, "{what}");
        }
    }
}

/// What a query reads of `f`'s count in floats, as bits: the total and
/// each literal's count, `None` where not exact; whether some level or
/// column was under a power of two; and the pairs by route. Levels of
/// `grid_min` pairs or more are read as grids.
fn read_bits(
    f: &Tdd,
    w: &Probabilities,
    grid_min: usize,
) -> (Vec<Option<(u64, i64)>>, bool, Routes) {
    let mut c = Counted::<f64>::leaves(f, w, grid_min).unwrap();
    assert_eq!(c.up_from(f, 0, true), None, "exact in floats");
    let mut read = vec![Some(bits(c.total(f).count()))];
    read.extend(f.vtree().leaf_bottomup().map(|(leaf, var)| {
        c.with_literal(f, leaf, var, true, w)
            .unwrap()
            .map(|x| bits(x.count()))
    }));
    let scaled = c.exps.iter().any(|&k| k != 0)
        || (0..c.exps.len()).any(|t| c.down(f, VtreeIdx(t as u32)).unwrap().exp != 0);
    (read, scaled, c.routes())
}

/// Centred wherever a count in floats may centre, whether or not its
/// products need it (`CENTRE_ALL`), the count passes give the bits they
/// give unscaled, which are those of a count in `Count`s: the total and
/// every literal's count of [`table`], its levels read as grids through
/// their kernels, and of a cube and a clause over a balanced vtree,
/// read pair by pair; and the answers of programs whose queries are
/// counted against their evidence, centred tables and all.
#[test]
fn centring_every_level_keeps_the_bits() {
    let eng = Engine::new();
    let (table, _) = table(&eng, Flaw::None);
    let mut table_w = Probabilities {
        pos: vec![0.5; 11],
        neg: vec![0.5; 11],
        one: vec![1.0; 11],
    };
    for v in 1..=10 {
        let p = (v as f64 * 0.618_033_988_75).fract() * 0.9 + 0.05;
        (table_w.pos[v], table_w.neg[v], table_w.one[v]) = (p * 1e-20, (1.0 - p) * 0.37e-20, 1e-20);
    }
    let n = 80u32;
    let vtree = Arc::new(Vtree::balanced(n + 3));
    let cube = Tdd::cube(&vtree, (1..=n as i32).collect::<Vec<_>>()).unwrap();
    let clause = Tdd::clause(&vtree, [n as i32 + 1, -(n as i32 + 2), n as i32 + 3]).unwrap();
    let cube = tididi::and(cube, clause).unwrap();
    let k = n as usize + 4;
    let mut cube_w = Probabilities {
        pos: vec![1e-3; k],
        neg: vec![1.0 - 1e-3; k],
        one: vec![1.0; k],
    };
    for v in n as usize + 1..k {
        (cube_w.pos[v], cube_w.neg[v]) = (0.3, 0.7);
    }
    for (what, f, w, grid_min) in [
        ("table", &table, &table_w, 0),
        ("cube", &cube, &cube_w, usize::MAX),
    ] {
        let wide = count::<Count>(f, w).unwrap();
        let mut want = vec![Some(bits(wide.total(f).count()))];
        want.extend(f.vtree().leaf_bottomup().map(|(leaf, var)| {
            wide.with_literal(f, leaf, var, true, w)
                .unwrap()
                .map(|x| bits(x.count()))
        }));
        let (plain, scaled, _) = read_bits(f, w, grid_min);
        assert!(!scaled, "{what}: under no power where no product needs one");
        assert_eq!(plain, want, "{what}: in floats");
        CENTRE_ALL.set(true);
        let (centred, scaled, routes) = read_bits(f, w, grid_min);
        CENTRE_ALL.set(false);
        assert!(scaled, "{what}: centred");
        assert_eq!(centred, want, "{what}: centred");
        if grid_min == 0 {
            let kernel = Route::Kernel.index::<f64>();
            assert!(
                routes.up[kernel] > 0 && routes.down[kernel] > 0,
                "{what}: {routes:?}"
            );
        }
    }
    let mut products = 0;
    for name in [
        "alarm",
        "bn_asia_evidence",
        "dice_evidence",
        "query_beside_evidence",
    ] {
        let source =
            std::fs::read_to_string(format!("{}/examples/{name}.pl", env!("CARGO_MANIFEST_DIR")))
                .unwrap();
        let answers = || {
            let run = crate::solve::run_source(
                &source,
                &crate::Options::with(crate::Strategy::Trial(crate::Kind::Closed)),
            )
            .unwrap();
            let probabilities: Vec<u64> = run
                .results
                .expect("consistent evidence")
                .iter()
                .map(|q| q.probability.to_bits())
                .collect();
            (
                probabilities,
                run.evidence_probability.map(bits),
                run.query_sizes.products,
            )
        };
        let plain = answers();
        CENTRE_ALL.set(true);
        let centred = answers();
        CENTRE_ALL.set(false);
        assert_eq!(centred, plain, "{name}");
        products += plain.2;
    }
    assert!(products > 0, "some query counted against the evidence");
}

/// The vector loop notes a set of floats as the scalar one does, bit
/// for bit: zeros of either sign, subnormals, infinities and NaNs, at
/// every length around a multiple of eight, from a fresh range and from
/// one already noted.
#[test]
fn settling_with_vectors_notes_what_scalars_do() {
    let pool = [
        0.0,
        -0.0,
        1.0,
        0.5,
        3.0e300,
        2.0e-310,
        f64::MIN_POSITIVE,
        7.25,
        f64::INFINITY,
        f64::NAN,
        1e-300,
        0.0,
    ];
    let mut xs: Vec<f64> = Vec::new();
    for n in 0..40usize {
        xs.clear();
        xs.extend(
            (0..n)
                .map(|i| pool[(i * 7 + n * 3) % pool.len()] * if i % 5 == 0 { 1.0 } else { 0.75 }),
        );
        let noted = Range::of(0.125f64);
        for start in [Range::NEW, noted] {
            let settle = |no_avx2: bool| {
                kernel::NO_AVX2.set(no_avx2);
                let mut range = start;
                <f64 as Value>::settle_all(&mut xs.clone(), &mut range);
                kernel::NO_AVX2.set(false);
                (range.low.to_bits(), range.high.to_bits())
            };
            assert_eq!(settle(false), settle(true), "{n} floats from {start:?}");
        }
    }
}

/// A vtree over `left + right + 1` variables from 1, in the text format:
/// the root's left child has a balanced subtree over the first `left`
/// variables on its left and one over the next `right` on its right, and
/// the root's right child is the last variable.
fn split_vtree(left: u32, right: u32) -> String {
    fn subtree(first: u32, n: u32, lines: &mut Vec<String>) -> usize {
        if n == 1 {
            lines.push(format!("L {} {first}", lines.len()));
            return lines.len() - 1;
        }
        let l = subtree(first, n / 2, lines);
        let r = subtree(first + n / 2, n - n / 2, lines);
        lines.push(format!("I {} {l} {r}", lines.len()));
        lines.len() - 1
    }
    let mut lines = Vec::new();
    let a = subtree(1, left, &mut lines);
    let b = subtree(1 + left, right, &mut lines);
    lines.push(format!("I {} {a} {b}", lines.len()));
    let t = lines.len() - 1;
    lines.push(format!("L {} {}", lines.len(), left + right + 1));
    lines.push(format!("I {} {t} {}", lines.len(), lines.len() - 1));
    format!("vtree {}\n{}\n", lines.len(), lines.join("\n"))
}

/// A table level T over two complete levels A and B, the children of T,
/// whose nodes are every assignment of six variables each; the root, over
/// T and one more variable, has a pair with each node of T. T's node `u`,
/// of digits `n` in the radices of `nodes` (the first fastest), has a pair
/// for each place `e` of `pairs`: `(A[a], B[b])`, `a` and `b` the base plus
/// each digit times its steps. `swap` exchanges the last two pairs of
/// every node, which makes the offsets a list.
struct Table {
    base: (isize, isize),
    nodes: Vec<(usize, isize, isize)>,
    pairs: Vec<(usize, isize, isize)>,
    swap: bool,
}

impl Table {
    fn build(&self, eng: &Engine) -> Tdd {
        let places = |digits: &[(usize, isize, isize)]| -> Vec<(isize, isize)> {
            let mut out = vec![(0, 0)];
            for &(radix, a, b) in digits {
                out = (0..radix as isize)
                    .flat_map(|c| out.iter().map(move |&(x, y)| (x + c * a, y + c * b)))
                    .collect();
            }
            out
        };
        let vtree = Arc::new(Vtree::from_text(&split_vtree(6, 6)).unwrap());
        let root = vtree.root();
        let t = vtree.children(root).0;
        let (ta, tb) = vtree.children(t);
        let mut b = Tdd::builder(eng, &vtree).unwrap();
        let a = complete(&mut b, eng, &vtree, ta);
        let c = complete(&mut b, eng, &vtree, tb);
        let offsets = places(&self.pairs);
        let nodes: Vec<NodeIdx> = places(&self.nodes)
            .into_iter()
            .map(|(x, y)| {
                let at = |o: isize, base: isize| {
                    usize::try_from(base + o).expect("a table's slot lies in its child")
                };
                let mut pairs: Vec<ChildPair> = offsets
                    .iter()
                    .map(|&(ox, oy)| {
                        ChildPair::new(a[at(x + ox, self.base.0)], c[at(y + oy, self.base.1)])
                    })
                    .collect();
                if self.swap {
                    let k = pairs.len();
                    pairs.swap(k - 2, k - 1);
                }
                b.push(eng, t, &pairs).unwrap()
            })
            .collect();
        let pairs: Vec<ChildPair> = nodes
            .iter()
            .map(|&u| ChildPair::new(u, POS_LEAF_IDX))
            .collect();
        let out = b.push(eng, root, &pairs).unwrap();
        b.finish(TddNodeId {
            vtree: root,
            local: out,
        })
        .unwrap()
    }
}

/// The count passes over grids, in every way each level's passes may run
/// and through every kernel, give the bits of the passes over pairs, in
/// floats and in `Count`s, on tables of each shape the kernels take: a
/// matrix product; digits of radix two, which the lanes merge; radices
/// that leave a vector part empty, and more lanes than a vector group
/// holds; steps that run backwards; offsets kept as a list; a child slot
/// two node digits share, which the pass down runs in stored order; a
/// pair digit that moves one side only; a child's slots two apart, which
/// a pass down writes through a copy of its column (DBN_13's levels); and
/// lanes that read both sides apart (Grids_16's). Under weights that keep
/// floats in range, and under weights whose products overflow, where a
/// pass down reads the counts up of a side that is not finite.
#[test]
fn grid_kernels_count_every_shape_to_the_bit_of_pairs() {
    let tables = [
        // A matrix product: up[i, j] = Σ_k A[i + 8k] B[k + 8j].
        Table {
            base: (0, 0),
            nodes: vec![(8, 1, 0), (8, 0, 8)],
            pairs: vec![(8, 8, 1)],
            swap: false,
        },
        // Radix two throughout.
        Table {
            base: (0, 0),
            nodes: vec![(2, 1, 0), (2, 0, 1), (2, 2, 0), (2, 0, 2), (2, 4, 0)],
            pairs: vec![(2, 8, 4), (2, 16, 8)],
            swap: false,
        },
        // Radices of five, six and three, and thirty-two lanes.
        Table {
            base: (0, 0),
            nodes: vec![(5, 1, 0), (6, 0, 1), (2, 5, 6)],
            pairs: vec![(3, 10, 12)],
            swap: false,
        },
        Table {
            base: (0, 0),
            nodes: vec![(32, 1, 0), (2, 0, 1)],
            pairs: vec![(2, 32, 2), (3, 0, 4)],
            swap: false,
        },
        // Backwards steps.
        Table {
            base: (63, 3),
            nodes: vec![(4, -1, 0), (4, 0, 1), (3, -4, 0)],
            pairs: vec![(4, -16, 4), (2, 0, 16)],
            swap: false,
        },
        // Offsets as a list.
        Table {
            base: (0, 0),
            nodes: vec![(4, 1, 0), (4, 0, 1), (2, 4, 0)],
            pairs: vec![(4, 8, 4)],
            swap: true,
        },
        // A slot two node digits share, and a pair digit that moves B only.
        Table {
            base: (0, 0),
            nodes: vec![(4, 1, 0), (4, 1, 4), (2, 0, 16)],
            pairs: vec![(4, 8, 1)],
            swap: false,
        },
        Table {
            base: (0, 0),
            nodes: vec![(4, 1, 0), (4, 4, 0), (2, 0, 32)],
            pairs: vec![(4, 0, 1), (2, 16, 4)],
            swap: false,
        },
        // A child's slots two apart, a pair digit stepping back on A.
        Table {
            base: (1, 0),
            nodes: vec![(4, 2, 0), (4, 0, 2), (2, 8, 0), (2, 0, 8)],
            pairs: vec![(2, -1, 1)],
            swap: false,
        },
        // Every output digit moving both sides.
        Table {
            base: (4, 0),
            nodes: vec![(2, 1, 0), (4, 0, 1), (4, 0, 4), (2, 2, 16)],
            pairs: vec![(2, -4, 32)],
            swap: false,
        },
    ];
    let weights = |scale: f64| {
        let n = 14;
        let mut w = Probabilities {
            pos: vec![0.5; n],
            neg: vec![0.5; n],
            one: vec![1.0; n],
        };
        for v in 1..n {
            let p = (v as f64 * 0.618_033_988_75).fract() * 0.9 + 0.05;
            (w.pos[v], w.neg[v], w.one[v]) = (p * scale, (1.0 - p) * scale * 0.37, scale);
        }
        w
    };
    let eng = Engine::new();
    let mut ways = 0;
    for table in &tables {
        let f = table.build(&eng);
        for scale in [1.0, 1e60] {
            ways += grids_agree_with_pairs(&f, &weights(scale));
        }
    }
    assert!(ways > 1000, "{ways} ways");
}
