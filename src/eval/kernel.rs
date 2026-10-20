//! The vector kernels of a grid's contractions ([`super::grid`]). A kernel
//! runs `out[o] += x[a] · y[b]` over the places of a tile at each place of
//! the dimensions outside it: its lanes are outputs whose slots in `out`
//! are consecutive, four to a vector of floats, and the tile repeats them
//! along one more output dimension, so that a tile's outputs sit in
//! registers while the summed dimensions inside it run in stored order.
//! Each output takes its terms in that order, a product rounded and then
//! added, as the scalar loops take them: the bits are theirs.
//!
//! A lane reads `x` and `y` at consecutive slots (`UNIT`), all lanes at one
//! slot (`SAME`), or at slots of their own (`APART`, four loads), which the
//! tile's places share where its dimension moves neither (`FIXED`). Where
//! a lane's dimensions move the level's nodes, the kernel can read or
//! write the node side in an order of its own, the lanes' dimensions
//! first ([`Relay`]): a copy that turns a scattered side into a unit one.
//! A pass down can so relay `out` too, the child's column, where each of
//! its slots is one place's: lanes two slots apart in the column are then
//! consecutive in the copy.
//!
//! The kernels are AVX2 code, a multiply and an add per product, with no
//! fused multiply-add, which rounds once; they run where the processor has
//! AVX2, and the scalar loops elsewhere.
//!
//! A kernel called at its outputs' last place notes the sums it stores
//! there in a [`Range`]: the column is settled as it is written, not read
//! again after.

use super::Range;
use super::grid::{Dim, Pass};

/// How a vector reads an operand (module doc).
pub(super) type Read = u8;
pub(super) const UNIT: Read = 0;
pub(super) const SAME: Read = 1;
pub(super) const APART: Read = 2;
pub(super) const FIXED: Read = 3;

/// The lanes of a vector of floats.
const WIDTH: usize = 4;

/// The most lanes a tile's vectors hold for one place of its dimension.
const MOST_LANES: usize = 16;

/// The places a lane set of one dimension holds.
pub(super) const LANES: std::ops::RangeInclusive<usize> = WIDTH..=MOST_LANES;

#[cfg(test)]
thread_local! {
    /// Whether [`avx2`] takes the processor to lack AVX2, for the tests of
    /// the scalar loops in its place.
    pub(super) static NO_AVX2: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the processor runs the vector loops here; where it does not,
/// their callers run scalar loops. The tests take a processor without
/// AVX2 where `NO_AVX2` says so.
fn avx2() -> bool {
    #[cfg(test)]
    if NO_AVX2.get() {
        return false;
    }
    #[cfg(target_arch = "x86_64")]
    {
        std::arch::is_x86_feature_detected!("avx2")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// The least nonzero float of `xs` and `low` and the greatest of `xs` and
/// `high`, as [`super::Range`] notes them (a zero counts as infinite for
/// the least, and a NaN for neither): eight floats a step, where the
/// processor has AVX2, and `None` where it has not.
pub(super) fn extremes(xs: &[f64], low: f64, high: f64) -> Option<(f64, f64)> {
    #[cfg(target_arch = "x86_64")]
    if avx2() {
        // SAFETY: the processor has AVX2.
        return Some(unsafe { avx2::extremes(xs, low, high) });
    }
    let _ = (xs, low, high);
    None
}

/// A kernel's entry: the tile's places at one place of the dimensions
/// outside it, `out`, `x` and `y` at that place's first pair; whether it
/// starts each output's sum; and where it completes them, the range their
/// values are noted in.
type Func = unsafe fn(&Kernel, *mut f64, *const f64, *const f64, bool, Option<&mut Range>);

/// One or two output dimensions whose places are a kernel's lanes, sorted
/// by their slots in `out`, which are consecutive; where there are more
/// than [`MOST_LANES`], in groups each the first shifted.
#[derive(Clone, Debug)]
pub(super) struct LaneSet {
    dims: Vec<usize>,
    /// Whether `out` and `x` are read in the order of [`relaid`], these
    /// dimensions first: the node side, which is `out` up and `x` down,
    /// and down `out`, the child's column.
    relay: [bool; 2],
    /// The lanes of the first group: what each adds to `out`, `x` and `y`
    /// from the place where the dimensions are zero.
    offsets: Vec<[isize; 3]>,
    groups: usize,
    group: [isize; 3],
    x: Read,
    y: Read,
}

impl LaneSet {
    /// The lane sets of `pass` over dimensions `dims`, of `steps`: each
    /// output dimension of four places or more, and each of two or three
    /// places with another output dimension, read as stored, with the node
    /// side relaid where they move the level's nodes, and where `relay_out`
    /// and their slots in `out` are not consecutive, with `out` relaid too.
    /// None reads both `x` and `y` same.
    pub(super) fn options(
        dims: &[Dim],
        steps: &[[isize; 3]],
        pass: Pass,
        relay_out: bool,
    ) -> Vec<LaneSet> {
        let n = dims.len();
        let lane = |d: usize| steps[d][0] != 0 && !dims[d].listed;
        let mut sets: Vec<Vec<usize>> = Vec::new();
        for v in (0..n).filter(|&v| lane(v)) {
            if dims[v].radix >= WIDTH {
                sets.push(vec![v]);
            } else {
                for w in
                    (0..n).filter(|&w| w != v && lane(w) && dims[v].radix * dims[w].radix >= WIDTH)
                {
                    sets.push(vec![v, w]);
                }
            }
        }
        let side = pass.node_side();
        let mut out = Vec::new();
        for set in sets {
            let mut ways = vec![[false; 2]];
            if set.iter().any(|&d| dims[d].node != 0) {
                ways.push(std::array::from_fn(|a| a == side));
            }
            if relay_out && !consecutive(&places(dims, steps, &set)) {
                ways.extend(ways.clone().into_iter().map(|[_, x]| [true, x]));
            }
            for relay in ways {
                out.extend(LaneSet::layout(
                    dims,
                    &relaid(dims, steps, &set, relay),
                    &set,
                    relay,
                ));
            }
        }
        out
    }

    /// The lane set of `set` under `steps`, if its places' slots in `out`
    /// are consecutive, in groups of [`MOST_LANES`] or fewer that repeat
    /// the first, and it reads `x` or `y` other than same.
    fn layout(
        dims: &[Dim],
        steps: &[[isize; 3]],
        set: &[usize],
        relay: [bool; 2],
    ) -> Option<LaneSet> {
        let places = places(dims, steps, set);
        let total = places.len();
        if total < WIDTH || !consecutive(&places) {
            return None;
        }
        let lanes = (WIDTH..=MOST_LANES.min(total)).rev().find(|&g| {
            total.is_multiple_of(g) && {
                let step = [0, 1, 2].map(|a| places.get(g).map_or(0, |p| p[a] - places[0][a]));
                places.iter().enumerate().all(|(i, p)| {
                    (0..3).all(|a| p[a] == places[i % g][a] + (i / g) as isize * step[a])
                })
            }
        })?;
        let offsets = places[..lanes].to_vec();
        let group = [0, 1, 2].map(|a| places.get(lanes).map_or(0, |p| p[a] - places[0][a]));
        let read = |a: usize| {
            if offsets
                .iter()
                .enumerate()
                .all(|(i, p)| p[a] == offsets[0][a] + i as isize)
            {
                UNIT
            } else if offsets.iter().all(|p| p[a] == offsets[0][a]) {
                SAME
            } else {
                APART
            }
        };
        let (x, y) = (read(1), read(2));
        if x == SAME && y == SAME {
            return None;
        }
        Some(LaneSet {
            dims: set.to_vec(),
            relay,
            offsets,
            groups: total / lanes,
            group,
            x,
            y,
        })
    }

    /// The dimensions whose places are the lanes.
    pub(super) fn dims(&self) -> &[usize] {
        &self.dims
    }

    /// Which of `out` and `x` the set reads relaid.
    pub(super) fn relays(&self) -> [bool; 2] {
        self.relay
    }

    /// `steps` with the sides the set relays relaid, the tile's dimension
    /// `tile` after its own.
    pub(super) fn steps(
        &self,
        dims: &[Dim],
        steps: &[[isize; 3]],
        tile: Option<usize>,
    ) -> Vec<[isize; 3]> {
        relaid(dims, steps, &self.first(tile), self.relay)
    }

    /// The dimensions the node side's order puts first: the lanes', then
    /// the tile's.
    fn first(&self, tile: Option<usize>) -> Vec<usize> {
        self.dims.iter().copied().chain(tile).collect()
    }

    /// How many vectors a place of the tile's dimension takes, and how
    /// many of its places a full tile takes.
    fn shape(&self) -> (usize, usize) {
        let vectors = self.offsets.len().div_ceil(WIDTH);
        (vectors, if vectors <= 2 { 4 } else { 2 })
    }

    /// The reads of `x` and `y` with the tile's dimension of `jstep`: one
    /// that reads apart and that the dimension does not move is fixed.
    fn reads(&self, jstep: Option<[isize; 3]>) -> (Read, Read) {
        let fixed = |r: Read, a: usize| {
            if r == APART && jstep.is_some_and(|s| s[a] == 0) {
                FIXED
            } else {
                r
            }
        };
        (fixed(self.x, 1), fixed(self.y, 2))
    }

    /// The cycles the model charges the kernel, with the tile's dimension
    /// `tile`, `summed` places inside it and `outer` places outside: a
    /// step of the summed places takes a cycle a product of vectors, and
    /// no fewer than an add's latency, nor than its loads at two a cycle
    /// or its shuffles at one; a tile adds the loads and stores of its
    /// sums and a call.
    pub(super) fn cycles(
        &self,
        dims: &[Dim],
        steps: &[[isize; 3]],
        tile: Option<usize>,
        summed: f64,
        outer: f64,
    ) -> f64 {
        let (vectors, full) = self.shape();
        let (x, y) = self.reads(tile.map(|j| steps[j]));
        let tile_cycles = |nj: usize| {
            let products = (vectors * nj) as f64;
            let mut loads = 1.0;
            let mut shuffles = 0.0;
            for r in [x, y] {
                let (l, s) = match r {
                    UNIT | SAME => (vectors * nj, 0),
                    APART => (4 * vectors * nj, 3 * vectors * nj),
                    _ => (4 * vectors, 3 * vectors),
                };
                loads += l as f64;
                shuffles += s as f64;
            }
            let step = products.max(4.0).max(loads / 2.0).max(shuffles);
            summed * step + 2.0 * products + 10.0
        };
        let radix = tile.map_or(1, |j| dims[j].radix);
        let mut tiles = (radix / full) as f64 * tile_cycles(full);
        let mut rest = radix % full;
        if full > 2 && rest >= 2 {
            tiles += tile_cycles(2);
            rest -= 2;
        }
        tiles += rest as f64 * tile_cycles(1);
        outer * (self.groups as f64 * tiles + 10.0)
    }
}

/// The places of the dimensions `set` under `steps`, as their offsets in
/// `out`, `x` and `y`, sorted by `out`'s.
fn places(dims: &[Dim], steps: &[[isize; 3]], set: &[usize]) -> Vec<[isize; 3]> {
    let mut places = vec![[0isize; 3]];
    for &d in set {
        places = (0..dims[d].radix as isize)
            .flat_map(|c| {
                places
                    .iter()
                    .map(move |p| [0, 1, 2].map(|a| p[a] + c * steps[d][a]))
            })
            .collect();
    }
    places.sort_by_key(|p| p[0]);
    places
}

/// Whether `places`, sorted by their offsets in `out`, are consecutive
/// there.
fn consecutive(places: &[[isize; 3]]) -> bool {
    places
        .iter()
        .enumerate()
        .all(|(i, p)| p[0] == places[0][0] + i as isize)
}

/// The dimensions that move side `a` (of `out`, `x`, `y`) under `steps`,
/// `first`'s first, in that order, and the others after them in stored
/// order, each with its stride in a numbering of the side's places in that
/// order.
fn relay_order(
    dims: &[Dim],
    steps: &[[isize; 3]],
    a: usize,
    first: &[usize],
) -> Vec<(usize, isize)> {
    let moves = |d: &usize| steps[*d][a] != 0;
    let order: Vec<usize> = first
        .iter()
        .copied()
        .filter(moves)
        .chain((0..steps.len()).filter(|d| moves(d) && !first.contains(d)))
        .collect();
    let mut stride = 1isize;
    order
        .into_iter()
        .map(|d| {
            let s = stride;
            stride *= dims[d].radix as isize;
            (d, s)
        })
        .collect()
}

/// `steps` with `out` and `x` relaid where `relay` says, `first`'s
/// dimensions first.
fn relaid(
    dims: &[Dim],
    steps: &[[isize; 3]],
    first: &[usize],
    relay: [bool; 2],
) -> Vec<[isize; 3]> {
    let mut out = steps.to_vec();
    for a in (0..2).filter(|&a| relay[a]) {
        for (d, s) in relay_order(dims, steps, a, first) {
            out[d][a] = s;
        }
    }
    out
}

/// A side's slots in an order of their own: `dims` the radix and the
/// stride in the stored order of each dimension that moves it, from the
/// fastest in the new order. Slot `new` of the copy is the stored slot
/// `origin` plus the strides of its digits.
#[derive(Clone, Debug)]
pub(super) struct Relay {
    dims: Vec<(usize, isize)>,
    len: usize,
}

impl Relay {
    fn new(dims: &[Dim], steps: &[[isize; 3]], a: usize, first: &[usize]) -> Relay {
        let order: Vec<(usize, isize)> = relay_order(dims, steps, a, first)
            .into_iter()
            .map(|(d, _)| (dims[d].radix, steps[d][a]))
            .collect();
        let len = order.iter().map(|&(r, _)| r).product();
        Relay { dims: order, len }
    }

    /// The slots of the copy.
    pub(super) fn len(&self) -> usize {
        self.len
    }

    /// `f(new, old)` for each slot, in the copy and in the stored order
    /// from `origin`, in the new order.
    fn each(&self, origin: isize, mut f: impl FnMut(usize, usize)) {
        let Some(&(r0, s0)) = self.dims.first() else {
            f(0, origin as usize);
            return;
        };
        let rest = &self.dims[1..];
        let mut count = vec![0usize; rest.len()];
        let (mut new, mut old) = (0usize, origin);
        loop {
            for c in 0..r0 {
                f(new + c, (old + c as isize * s0) as usize);
            }
            new += r0;
            let mut j = 0;
            loop {
                let Some(&(r, s)) = rest.get(j) else { return };
                count[j] += 1;
                old += s;
                if count[j] < r {
                    break;
                }
                old -= s * r as isize;
                count[j] = 0;
                j += 1;
            }
        }
    }

    /// `to`, in the new order, the values of `from`, in the stored one
    /// from `origin`.
    pub(super) fn gather(&self, origin: isize, from: &[f64], to: &mut [f64]) {
        self.each(origin, |new, old| to[new] = from[old]);
    }

    /// `to`, in the stored order from `origin`, the values of `from`, in
    /// the new one.
    pub(super) fn scatter(&self, origin: isize, from: &[f64], to: &mut [f64]) {
        self.each(origin, |new, old| to[old] = from[new]);
    }
}

/// A kernel of a contraction (module doc), with what it reads: the first
/// lane's offsets, and each vector's lanes from it; the tile's dimension;
/// the summed places' offsets. Only the AVX2 loops read most of it, so on
/// another processor (WebAssembly, ARM) they go unread.
#[derive(Clone, Debug)]
#[cfg_attr(not(target_arch = "x86_64"), allow(dead_code))]
pub(super) struct Kernel {
    /// What a step of each dimension adds to `out`, `x` and `y`, the
    /// sides `relays` copies relaid.
    pub(super) steps: Vec<[isize; 3]>,
    /// The copies of `out` and `x` the kernel reads in place of the
    /// stored sides.
    pub(super) relays: [Option<Relay>; 2],
    /// The offsets of the summed places inside the tile, in `x` and `y`.
    offsets: Vec<[isize; 2]>,
    /// The first lane of the first group in `out`, `x` and `y`. A vector
    /// read or written unit starts at four times its place from it, the
    /// last at `last`, which overlaps the one before where the lanes are
    /// not a multiple of four; a vector read same, at it. A vector read
    /// apart reads its lanes at `xl` or `yl` from it.
    lane: [isize; 3],
    last: isize,
    xl: [[isize; 4]; 4],
    yl: [[isize; 4]; 4],
    groups: usize,
    group: [isize; 3],
    /// The places of the tile's dimension and its step.
    places: usize,
    jstep: [isize; 3],
    func: Func,
}

impl Kernel {
    /// The kernel of `lanes` with the tile's dimension `tile`, over
    /// dimensions `dims` of `steps`, `offsets` giving the summed places'
    /// offsets under the kernel's steps; `None` where no kernel of its
    /// shape is compiled.
    pub(super) fn new(
        lanes: &LaneSet,
        tile: Option<usize>,
        dims: &[Dim],
        steps: &[[isize; 3]],
        offsets: impl Fn(&[[isize; 3]]) -> Vec<[isize; 2]>,
    ) -> Option<Kernel> {
        let first = lanes.first(tile);
        let relays = [0, 1].map(|a| lanes.relay[a].then(|| Relay::new(dims, steps, a, &first)));
        let steps = relaid(dims, steps, &first, lanes.relay);
        let jstep = tile.map(|j| steps[j]);
        let (vectors, _) = lanes.shape();
        let (x, y) = lanes.reads(jstep);
        let func = func(vectors, x, y)?;
        let g = lanes.offsets.len();
        let lane = lanes.offsets[0];
        let (mut xl, mut yl) = ([[0isize; 4]; 4], [[0isize; 4]; 4]);
        for v in 0..vectors {
            let s = (WIDTH * v).min(g - WIDTH);
            for i in 0..WIDTH {
                xl[v][i] = lanes.offsets[s + i][1] - lane[1];
                yl[v][i] = lanes.offsets[s + i][2] - lane[2];
            }
        }
        Some(Kernel {
            offsets: offsets(&steps),
            relays,
            steps,
            lane,
            last: (g - WIDTH) as isize,
            xl,
            yl,
            groups: lanes.groups,
            group: lanes.group,
            places: tile.map_or(1, |j| dims[j].radix),
            jstep: jstep.unwrap_or([0; 3]),
            func,
        })
    }

    /// Whether the processor runs the kernels; where it does not, a plan
    /// runs through its scalar loops ([`super::grid`]).
    pub(super) fn ready(&self) -> bool {
        avx2()
    }

    /// Runs the tile's places at one place of the dimensions outside it,
    /// noting the sums it stores in `range` where given, at the place that
    /// completes them.
    ///
    /// # Safety
    ///
    /// The processor has AVX2 ([`Kernel::ready`]), and `out`, `x` and `y`
    /// at that place's first pair are such that every slot the tile's
    /// places name lies within their arrays.
    #[inline]
    pub(super) unsafe fn call(
        &self,
        out: *mut f64,
        x: *const f64,
        y: *const f64,
        first: bool,
        range: Option<&mut Range>,
    ) {
        // SAFETY: the caller's.
        unsafe { (self.func)(self, out, x, y, first, range) }
    }
}

/// The kernel of `vectors` vectors a place reading `x` and `y` as given,
/// tiles of four places of the tile's dimension for one or two vectors and
/// of two for three or four.
fn func(vectors: usize, x: Read, y: Read) -> Option<Func> {
    #[cfg(target_arch = "x86_64")]
    {
        macro_rules! pick {
            ($nv:literal, $nj:literal) => {
                match (x, y) {
                    (UNIT, UNIT) => avx2::drive::<$nv, $nj, UNIT, UNIT> as Func,
                    (UNIT, SAME) => avx2::drive::<$nv, $nj, UNIT, SAME> as Func,
                    (SAME, UNIT) => avx2::drive::<$nv, $nj, SAME, UNIT> as Func,
                    (UNIT, APART) => avx2::drive::<$nv, $nj, UNIT, APART> as Func,
                    (APART, UNIT) => avx2::drive::<$nv, $nj, APART, UNIT> as Func,
                    (UNIT, FIXED) => avx2::drive::<$nv, $nj, UNIT, FIXED> as Func,
                    (FIXED, UNIT) => avx2::drive::<$nv, $nj, FIXED, UNIT> as Func,
                    (SAME, APART) => avx2::drive::<$nv, $nj, SAME, APART> as Func,
                    (APART, SAME) => avx2::drive::<$nv, $nj, APART, SAME> as Func,
                    (SAME, FIXED) => avx2::drive::<$nv, $nj, SAME, FIXED> as Func,
                    (FIXED, SAME) => avx2::drive::<$nv, $nj, FIXED, SAME> as Func,
                    (APART, APART) => avx2::drive::<$nv, $nj, APART, APART> as Func,
                    (APART, FIXED) => avx2::drive::<$nv, $nj, APART, FIXED> as Func,
                    (FIXED, APART) => avx2::drive::<$nv, $nj, FIXED, APART> as Func,
                    (FIXED, FIXED) => avx2::drive::<$nv, $nj, FIXED, FIXED> as Func,
                    _ => return None,
                }
            };
        }
        Some(match vectors {
            1 => pick!(1, 4),
            2 => pick!(2, 4),
            3 => pick!(3, 2),
            4 => pick!(4, 2),
            _ => return None,
        })
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (vectors, x, y);
        None
    }
}

#[cfg(target_arch = "x86_64")]
mod avx2 {
    use std::arch::x86_64::*;

    use super::{FIXED, Kernel, Range, Read, SAME, UNIT, WIDTH};

    /// Each group of lanes, and in it the tile's places, `NJ` at a time,
    /// then two and one; the sums stored noted in `range` where given, in
    /// lanes of their own while the tiles run.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn drive<const NV: usize, const NJ: usize, const X: Read, const Y: Read>(
        k: &Kernel,
        o: *mut f64,
        x: *const f64,
        y: *const f64,
        first: bool,
        range: Option<&mut Range>,
    ) {
        // SAFETY (all of this function and `tile`): the slots of every
        // group's and tile place's lanes lie within the arrays (`call`).
        unsafe {
            let mut lanes = range
                .as_ref()
                .map(|r| [_mm256_set1_pd(r.low), _mm256_set1_pd(r.high)]);
            let (o, x, y) = (
                o.offset(k.lane[0]),
                x.offset(k.lane[1]),
                y.offset(k.lane[2]),
            );
            for q in 0..k.groups as isize {
                let (o, x, y) = (
                    o.offset(q * k.group[0]),
                    x.offset(q * k.group[1]),
                    y.offset(q * k.group[2]),
                );
                let at = |j: usize| {
                    (
                        o.offset(j as isize * k.jstep[0]),
                        x.offset(j as isize * k.jstep[1]),
                        y.offset(j as isize * k.jstep[2]),
                    )
                };
                let mut j = 0;
                while j + NJ <= k.places {
                    let (o, x, y) = at(j);
                    tile::<NV, NJ, X, Y>(k, o, x, y, first, lanes.as_mut());
                    j += NJ;
                }
                if NJ > 2 && j + 2 <= k.places {
                    let (o, x, y) = at(j);
                    tile::<NV, 2, X, Y>(k, o, x, y, first, lanes.as_mut());
                    j += 2;
                }
                while j < k.places {
                    let (o, x, y) = at(j);
                    tile::<NV, 1, X, Y>(k, o, x, y, first, lanes.as_mut());
                    j += 1;
                }
            }
            if let (Some(range), Some([low, high])) = (range, lanes) {
                (range.low, range.high) = fold(low, high, range.low, range.high);
            }
        }
    }

    /// `NJ` places of the tile's dimension, `NV` vectors each, `out`, `x`
    /// and `y` at the first lane: the sums in registers, from `out` or,
    /// where `first`, from negative zero, which a first product's sum is
    /// that product; each summed place adds its products in order, a
    /// multiply and then an add. The sums are noted in the lanes `notes`
    /// where given, as they are stored.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn tile<const NV: usize, const NJ: usize, const X: Read, const Y: Read>(
        k: &Kernel,
        o: *mut f64,
        x: *const f64,
        y: *const f64,
        first: bool,
        notes: Option<&mut [__m256d; 2]>,
    ) {
        unsafe {
            let zero = _mm256_set1_pd(-0.0);
            let unit = |v: usize| {
                if v + 1 < NV {
                    (WIDTH * v) as isize
                } else {
                    k.last
                }
            };
            let mut acc = [[zero; NV]; NJ];
            if !first {
                for (j, acc) in acc.iter_mut().enumerate() {
                    for (v, a) in acc.iter_mut().enumerate() {
                        *a = _mm256_loadu_pd(o.offset(j as isize * k.jstep[0] + unit(v)));
                    }
                }
            }
            for &[kx, ky] in k.offsets.iter() {
                let (xk, yk) = (x.offset(kx), y.offset(ky));
                let mut xs = [zero; NV];
                let mut ys = [zero; NV];
                for (v, (xs, ys)) in xs.iter_mut().zip(&mut ys).enumerate() {
                    if X == FIXED {
                        *xs = apart(xk, &k.xl[v]);
                    }
                    if Y == FIXED {
                        *ys = apart(yk, &k.yl[v]);
                    }
                }
                for (j, acc) in acc.iter_mut().enumerate() {
                    let (xj, yj) = (
                        xk.offset(j as isize * k.jstep[1]),
                        yk.offset(j as isize * k.jstep[2]),
                    );
                    for (v, a) in acc.iter_mut().enumerate() {
                        let p = if X == FIXED {
                            xs[v]
                        } else {
                            operand::<X>(xj, unit(v), &k.xl[v])
                        };
                        let q = if Y == FIXED {
                            ys[v]
                        } else {
                            operand::<Y>(yj, unit(v), &k.yl[v])
                        };
                        *a = _mm256_add_pd(*a, _mm256_mul_pd(p, q));
                    }
                }
            }
            if let Some([low, high]) = notes {
                for a in acc.iter().flatten() {
                    note(*a, low, high);
                }
            }
            for (j, acc) in acc.iter().enumerate() {
                for (v, a) in acc.iter().enumerate() {
                    _mm256_storeu_pd(o.offset(j as isize * k.jstep[0] + unit(v)), *a);
                }
            }
        }
    }

    /// `x`'s four floats noted in the lanes of `low` and `high`, as a
    /// [`Range`] notes a float: a zero infinite for the least.
    /// `_mm256_min_pd(a, b)` is `a < b ? a : b` and `_mm256_max_pd(a, b)` is
    /// `a > b ? a : b`, a NaN in `a` keeping `b`, so the lanes keep what the
    /// scalar compares keep, in any order.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn note(x: __m256d, low: &mut __m256d, high: &mut __m256d) {
        let zero = _mm256_cmp_pd::<_CMP_EQ_OQ>(x, _mm256_setzero_pd());
        *low = _mm256_min_pd(
            _mm256_blendv_pd(x, _mm256_set1_pd(f64::INFINITY), zero),
            *low,
        );
        *high = _mm256_max_pd(x, *high);
    }

    /// [`super::extremes`] under AVX2: two vectors of four lanes each for
    /// the least and the greatest ([`note`]), the rest of `xs` past a
    /// multiple of eight one at a time.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn extremes(xs: &[f64], low: f64, high: f64) -> (f64, f64) {
        // SAFETY: each load reads four of a chunk's eight floats, and each
        // store writes four of an array's eight.
        unsafe {
            let (mut lo, mut hi) = ([_mm256_set1_pd(low); 2], [_mm256_set1_pd(high); 2]);
            let (chunks, remainder) = xs.as_chunks::<8>();
            for chunk in chunks {
                for (j, (lo, hi)) in lo.iter_mut().zip(&mut hi).enumerate() {
                    note(_mm256_loadu_pd(chunk.as_ptr().add(4 * j)), lo, hi);
                }
            }
            let (mut low, mut high) = fold(
                _mm256_min_pd(lo[0], lo[1]),
                _mm256_max_pd(hi[0], hi[1]),
                low,
                high,
            );
            for &x in remainder {
                let y = if x == 0.0 { f64::INFINITY } else { x };
                low = if y < low { y } else { low };
                high = if x > high { x } else { high };
            }
            (low, high)
        }
    }

    /// The least of `low` and the lanes of `lo`, and the greatest of `high`
    /// and the lanes of `hi`, lanes [`note`] kept, none of them a NaN.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn fold(lo: __m256d, hi: __m256d, low: f64, high: f64) -> (f64, f64) {
        let (lo, hi) = (
            _mm256_min_pd(lo, _mm256_permute2f128_pd::<1>(lo, lo)),
            _mm256_max_pd(hi, _mm256_permute2f128_pd::<1>(hi, hi)),
        );
        let l = _mm256_cvtsd_f64(_mm256_min_pd(lo, _mm256_permute_pd::<0b0101>(lo)));
        let h = _mm256_cvtsd_f64(_mm256_max_pd(hi, _mm256_permute_pd::<0b0101>(hi)));
        (
            if l < low { l } else { low },
            if h > high { h } else { high },
        )
    }

    /// A vector's operand: four floats from `at`, the first lane's float in
    /// every lane, or one float from each of `lanes`.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn operand<const M: Read>(p: *const f64, at: isize, lanes: &[isize; 4]) -> __m256d {
        unsafe {
            match M {
                UNIT => _mm256_loadu_pd(p.offset(at)),
                SAME => _mm256_broadcast_sd(&*p),
                _ => apart(p, lanes),
            }
        }
    }

    /// One float from each of `lanes`.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn apart(p: *const f64, lanes: &[isize; 4]) -> __m256d {
        unsafe {
            _mm256_setr_pd(
                *p.offset(lanes[0]),
                *p.offset(lanes[1]),
                *p.offset(lanes[2]),
                *p.offset(lanes[3]),
            )
        }
    }
}
