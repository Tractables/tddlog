//! That none of a set of cubes holds and at most one variable of each group
//! is true, built level by level ([`build`]).

use std::sync::Arc;

use rustc_hash::FxHashMap;
use tididi::diagram::{ChildPair, NEG_LEAF_IDX, NodeIdx, ONE_LEAF_IDX, POS_LEAF_IDX, TddNodeId};
use tididi::vtree::{VarId, VtreeIdx};
use tididi::{Engine, Literal, OperationError, Tdd, Vtree};

use crate::cubes::{Merged, merge_complements};

/// A class of assignments to a vtree node's variables: the residuals of
/// one of them, its representative, each with a cube that leaves it (whose
/// literals under the node all hold for the representative and some lie
/// outside), ascending by residual; the groups it sets a variable of; and
/// the representative's values, a bit per leaf under the node, left to
/// right, as ranges of its level's arenas, up to the last leaf a node
/// above may read ([`kept_bits`]); and its node.
///
/// A class that a lift passed up as it was ([`lift`]) keeps its values
/// from the leaf `lo` on, and those of the leaves before it, which the
/// lifts took in, are `defaults`' ([`Level::normalize`]). Its key's hash
/// is kept for the lifts, which look classes up on a level they extend,
/// and is [`UNHASHED`] for a leaf's.
#[derive(Clone, Copy)]
struct Class {
    residuals: (u32, u32),
    ones: (u32, u32),
    values: (u32, u32),
    lo: u32,
    hash: u64,
    node: NodeIdx,
    /// Along a run of lifts, the nearest leaf before the node's that a
    /// cube of its residuals has a literal at ([`NO_LEAF`] for none): the
    /// lift at that leaf is the one that takes the class out. Kept here,
    /// in the room the hash leaves, rather than on a list of the level's
    /// own, which made a [`Level`] too large to move without a call to
    /// `memcpy`.
    read: u32,
    /// False for a class a lift took out of its level.
    live: bool,
}

#[derive(Default)]
struct Level {
    classes: Vec<Class>,
    residuals: Vec<(u32, u32)>,
    ones: Vec<u32>,
    values: Vec<u64>,
    /// Whether a lift made the level: [`Class::read`] holds for every
    /// class, and a class may be taken out or keep values from past the
    /// node's first leaf.
    lifted: bool,
    /// The classes taken out.
    dead: usize,
}

/// What [`Class::read`] holds for a class none of whose cubes has a
/// literal before the node's leaves.
const NO_LEAF: u32 = u32::MAX;

/// The fewest classes of a level that a run of lifts begins on. Where the
/// run begins, a lift reads each class's cubes and keys, and where it
/// ends it writes each class's values out, which pairs of the leaf's one
/// or two classes with two or three of the level's do not repay: the
/// general join reads and writes as much for them.
const LIFT_FROM: usize = 4;

/// The hash a leaf's class keeps until [`Keys::rebuild`] needs it: only a
/// lift over the leaf's level does. A key whose hash is this is hashed
/// again, which gives the same.
const UNHASHED: u64 = u64::MAX;

impl Level {
    /// Emptied, keeping its arenas for the next level.
    fn cleared(mut self) -> Self {
        self.classes.clear();
        self.residuals.clear();
        self.ones.clear();
        self.values.clear();
        self.lifted = false;
        self.dead = 0;
        self
    }
    fn live(&self) -> usize {
        self.classes.len() - self.dead
    }
    /// A level that lifts made, as a level of the node whose leaves begin
    /// at `lo` and whose classes keep `bits` values ([`kept_bits`]), that
    /// holds its classes alone, each with its values from `lo`: the values
    /// of the leaves the lifts took in are read from `defaults`, a bit per
    /// leaf, and written out on `spare`'s arena, which takes the place of
    /// the level's own.
    fn normalize(&mut self, lo: u32, bits: u32, defaults: &[u64], spare: &mut Vec<u64>) {
        if !self.lifted {
            return;
        }
        spare.clear();
        let mut kept = 0;
        for k in 0..self.classes.len() {
            let c = self.classes[k];
            if !c.live {
                continue;
            }
            let v = spare.len();
            let lifted = (c.lo - lo).min(bits);
            let mut written = append_bits(spare, 0, defaults, lo as usize, lifted as usize);
            written = append_bits(
                spare,
                written,
                &self.values[c.values.0 as usize..c.values.1 as usize],
                0,
                (bits - lifted) as usize,
            );
            debug_assert_eq!(written, bits as usize);
            self.classes[kept] = Class {
                values: (v as u32, spare.len() as u32),
                lo,
                ..c
            };
            kept += 1;
        }
        self.classes.truncate(kept);
        std::mem::swap(&mut self.values, spare);
        self.lifted = false;
        self.dead = 0;
    }
    fn residuals(&self, c: &Class) -> &[(u32, u32)] {
        &self.residuals[c.residuals.0 as usize..c.residuals.1 as usize]
    }
    fn ones(&self, c: &Class) -> &[u32] {
        &self.ones[c.ones.0 as usize..c.ones.1 as usize]
    }
    fn values(&self, c: &Class) -> &[u64] {
        &self.values[c.values.0 as usize..c.values.1 as usize]
    }
    /// A class of the leaf numbered `leaf`, whose representative's value is
    /// `value`. Its key's hash is worked out where a lift asks for it
    /// ([`Keys::rebuild`]).
    fn add(
        &mut self,
        residuals: &[(u32, u32)],
        ones: &[u32],
        value: bool,
        leaf: u32,
        node: NodeIdx,
    ) {
        let at = self.residuals.len();
        self.residuals.extend_from_slice(residuals);
        let v = self.values.len();
        self.values.push(u64::from(value));
        self.push(at, ones, v, leaf, UNHASHED, node);
    }
    /// A class whose residuals are those on the arena from `at` and whose
    /// representative's values are the first `left_bits` bits of `left`,
    /// and where `right_bits` is not zero, clear bits up to bit `right_at`
    /// and then the first `right_bits` bits of `right`; each part's words
    /// from its first bit, and the class's words past its bits clear. The
    /// right's words go in shifted, each filling the word before and
    /// starting the next, rather than pushed and or-ed in a word at a time,
    /// part by part, which cost far more per class. Its values begin at the
    /// leaf `lo`, and `hash` is its key's.
    #[allow(clippy::too_many_arguments)]
    fn close(
        &mut self,
        at: usize,
        ones: &[u32],
        (left, left_bits): (&[u64], usize),
        (right, right_at, right_bits): (&[u64], usize, usize),
        lo: u32,
        hash: u64,
        node: NodeIdx,
    ) {
        let v = self.values.len();
        let words = if right_bits == 0 {
            left_bits
        } else {
            right_at + right_bits
        }
        .div_ceil(64);
        self.values.reserve(words);
        self.values
            .extend_from_slice(&left[..left_bits.div_ceil(64)]);
        if !left_bits.is_multiple_of(64) {
            *self
                .values
                .last_mut()
                .expect("a left part of bits has a word") &= (1 << (left_bits % 64)) - 1;
        }
        if right_bits > 0 {
            self.values.resize(v + right_at.div_ceil(64), 0);
            let n = right_bits.div_ceil(64);
            let last = if right_bits.is_multiple_of(64) {
                u64::MAX
            } else {
                (1 << (right_bits % 64)) - 1
            };
            let shift = right_at % 64;
            if shift == 0 {
                self.values.extend_from_slice(&right[..n]);
                *self
                    .values
                    .last_mut()
                    .expect("a right part of bits has a word") &= last;
            } else {
                let mut low = self
                    .values
                    .pop()
                    .expect("the word the right part begins in");
                let part = right[..n - 1]
                    .iter()
                    .copied()
                    .chain(std::iter::once(right[n - 1] & last));
                self.values.extend(part.map(|w| {
                    let word = low | w << shift;
                    low = w >> (64 - shift);
                    word
                }));
                if self.values.len() < v + words {
                    self.values.push(low);
                } else {
                    debug_assert_eq!(low, 0, "a part's bits past its own are clear");
                }
            }
        }
        debug_assert_eq!(self.values.len(), v + words);
        self.push(at, ones, v, lo, hash, node);
    }
    /// The class whose residuals are those on the arena from `at` and
    /// whose values those from `v`, with the groups `ones`.
    fn push(&mut self, at: usize, ones: &[u32], v: usize, lo: u32, hash: u64, node: NodeIdx) {
        let o = self.ones.len() as u32;
        self.ones.extend_from_slice(ones);
        self.classes.push(Class {
            residuals: (at as u32, self.residuals.len() as u32),
            ones: (o, self.ones.len() as u32),
            values: (v as u32, self.values.len() as u32),
            lo,
            hash,
            node,
            read: NO_LEAF,
            live: true,
        });
    }
}

/// Appends to `dst`, whose last `bits` bits are a class's values so far
/// (from a word's start), the `n` bits of `src` from bit `from`, and
/// returns how many the class has.
fn append_bits(dst: &mut Vec<u64>, mut bits: usize, src: &[u64], from: usize, n: usize) -> usize {
    let mut k = 0;
    while k < n {
        let take = (n - k).min(64);
        let (w, off) = ((from + k) / 64, (from + k) % 64);
        let mut word = src[w] >> off;
        if off != 0 && off + take > 64 {
            word |= src[w + 1] << (64 - off);
        }
        if take < 64 {
            word &= (1u64 << take) - 1;
        }
        let at = bits % 64;
        if at == 0 {
            dst.push(word);
        } else {
            *dst.last_mut().expect("a word begun") |= word << at;
            if at + take > 64 {
                dst.push(word >> (64 - at));
            }
        }
        bits += take;
        k += take;
    }
    bits
}

/// The hash of the key of the residuals `residuals` and the groups `ones`,
/// as the pairs of a level hash it while they merge the residuals.
fn key_hash(residuals: impl Iterator<Item = u32>, ones: &[u32]) -> u64 {
    let mut h = residuals.fold(0u64, mix);
    h = mix(h, SEP);
    ones.iter().fold(h, |h, &g| mix(h, g))
}

/// The value of the representative whose values are `values` at the leaf
/// `i` places into its node's.
fn value_at(values: &[u64], i: u32) -> bool {
    values[i as usize / 64] >> (i % 64) & 1 == 1
}

/// The cubes' literals, each by the number of its leaf, left to right,
/// twice it plus the sign, ascending, and a hash of each prefix of each
/// cube's. The leaves under a node are a range of numbers, so a cube's
/// literals outside it are a prefix and a suffix of its own, whose hash
/// takes constant time. Interning each residual as a list of its own
/// copies too much where bodies are long (bn_mildew's rules read the
/// siblings of a parent's value negated).
struct Cubes {
    codes: Vec<u32>,
    start: Vec<u32>,
    /// Cube `c`'s from `start[c] + c`, one more than it has literals.
    prefix: Vec<u64>,
    powers: Vec<u64>,
}

impl Cubes {
    const BASE: u64 = 0x9e37_79b9_7f4a_7c15;

    fn new(codes: Vec<u32>, start: Vec<u32>) -> Self {
        let mut prefix = Vec::with_capacity(codes.len() + start.len());
        let mut longest = 0;
        for w in start.windows(2) {
            let lits = &codes[w[0] as usize..w[1] as usize];
            longest = longest.max(lits.len());
            let mut h = 0u64;
            prefix.push(h);
            for &x in lits {
                // One more than the code, so that a leading zero counts.
                h = h.wrapping_mul(Self::BASE).wrapping_add(u64::from(x) + 1);
                prefix.push(h);
            }
        }
        let powers = std::iter::successors(Some(1u64), |p| Some(p.wrapping_mul(Self::BASE)))
            .take(longest + 1)
            .collect();
        Self {
            codes,
            start,
            prefix,
            powers,
        }
    }

    fn lits(&self, c: u32) -> &[u32] {
        &self.codes[self.start[c as usize] as usize..self.start[c as usize + 1] as usize]
    }

    /// The range of cube `c`'s literals under the node whose leaves are
    /// numbered `span`.
    fn under(&self, c: u32, (lo, hi): (u32, u32)) -> (usize, usize) {
        let l = self.lits(c);
        let i = l.partition_point(|&x| x / 2 < lo);
        (i, i + l[i..].partition_point(|&x| x / 2 <= hi))
    }

    /// Cube `c`'s literals but its `i..j`.
    fn without(&self, c: u32, i: usize, j: usize) -> impl Iterator<Item = u32> + '_ {
        let l = self.lits(c);
        l[..i].iter().chain(&l[j..]).copied()
    }

    /// Whether cube `c`'s literals but its `i..j` are cube `d`'s but its
    /// `a..b`, compared a slice at a time.
    fn same_without(&self, (c, i, j): (u32, usize, usize), (d, a, b): (u32, usize, usize)) -> bool {
        let (x, y) = (self.lits(c), self.lits(d));
        let (x1, x2, y1, y2) = (&x[..i], &x[j..], &y[..a], &y[b..]);
        if x1.len() + x2.len() != y1.len() + y2.len() {
            return false;
        }
        // The shorter prefix, then the rest of the longer against the
        // other's suffix.
        let ((p1, p2), (q1, q2)) = if x1.len() <= y1.len() {
            ((x1, x2), (y1, y2))
        } else {
            ((y1, y2), (x1, x2))
        };
        let k = q1.len() - p1.len();
        *p1 == q1[..p1.len()] && p2[..k] == q1[p1.len()..] && p2[k..] == *q2
    }

    /// The hash of [`without`](Self::without).
    fn hash_without(&self, c: u32, i: usize, j: usize) -> u64 {
        let base = self.start[c as usize] as usize + c as usize;
        let k = self.lits(c).len();
        let h = |m: usize| self.prefix[base + m];
        let p = self.powers[k - j];
        h(i).wrapping_mul(p)
            .wrapping_add(h(k).wrapping_sub(h(j).wrapping_mul(p)))
    }
}

/// The residual of each cube: its literals outside the node that last took
/// one of its literals in (`at`), interned (`id`) as the cube and range of
/// its literals left out that first made it (`made`), on a chain by hash
/// (`first`, `next`); and of each residual, the group whose variable it
/// forces true, where it is that variable's negation alone (`forced`,
/// `u32::MAX` for none), by the group of each leaf's variable (`group_of`,
/// by leaf number).
///
/// A residual that keeps a literal no other cube has is no other cube's,
/// and goes on no chain: of each cube, the first and last place of such a
/// literal in its own (`private`, `u32::MAX` for none). On the networks
/// nearly every residual is so: a table row's switch of its own value.
struct Residuals {
    made: Vec<(u32, u32, u32)>,
    first: FxHashMap<u64, u32>,
    next: Vec<u32>,
    id: Vec<u32>,
    at: Vec<u32>,
    forced: Vec<u32>,
    group_of: Vec<u32>,
    private: Vec<(u32, u32)>,
}

impl Residuals {
    /// The residual of cube `c` outside node `t`, under which lie its
    /// literals `i..j`, on both sides or at `t` itself where it is a leaf;
    /// `None` where none is outside.
    fn here(&mut self, cubes: &Cubes, c: u32, t: VtreeIdx, (i, j): (usize, usize)) -> Option<u32> {
        if self.at[c as usize] == t.0 {
            return Some(self.id[c as usize]);
        }
        let len = cubes.lits(c).len() - (j - i);
        if len == 0 {
            return None;
        }
        let n = self.made.len() as u32;
        let (p, q) = self.private[c as usize];
        let own = p != u32::MAX && ((p as usize) < i || (q as usize) >= j);
        let mut r = if own {
            n
        } else {
            *self.first.entry(cubes.hash_without(c, i, j)).or_insert(n)
        };
        let id = loop {
            if r == n {
                break n;
            }
            let (d, a, b) = self.made[r as usize];
            if cubes.same_without((c, i, j), (d, a as usize, b as usize)) {
                break r;
            }
            match self.next[r as usize] {
                u32::MAX => {
                    self.next[r as usize] = n;
                    r = n;
                }
                s => r = s,
            }
        };
        if id == n {
            self.made.push((c, i as u32, j as u32));
            self.next.push(u32::MAX);
            let forced = match (len, cubes.without(c, i, j).next()) {
                (1, Some(x)) if x % 2 == 0 => self.group_of[x as usize / 2],
                _ => u32::MAX,
            };
            self.forced.push(forced);
        }
        self.id[c as usize] = id;
        self.at[c as usize] = t.0;
        Some(id)
    }
}

/// Whether residuals, with the groups `ones` set, force two variables of a
/// group true, or one of a group already set: a residual that is a
/// variable's negation alone forces it true, and the groups allow one. The
/// rows of a table that the link of a reader after the first ties to the
/// reader's copies read the parents' copies without their exactly-one, so
/// below the reader's copies any number of rows can match, and each set of
/// the values they select would be a class (on bn_barley, hundreds of times
/// the table's pairs). The residuals come one at a time after `start`, each
/// once.
struct Forcing {
    /// The round in which each group was last forced.
    seen: Vec<u32>,
    round: u32,
}

impl Forcing {
    fn start(&mut self) {
        self.round = self.round.wrapping_add(1);
        if self.round == 0 {
            self.seen.iter_mut().for_each(|s| *s = 0);
            self.round = 1;
        }
    }
    /// False where a residual that forces a variable of group `g`
    /// (`u32::MAX` for none) clashes.
    fn admits(&mut self, g: u32, ones: &[u32]) -> bool {
        if g == u32::MAX {
            return true;
        }
        if self.seen[g as usize] == self.round || ones.binary_search(&g).is_ok() {
            return false;
        }
        self.seen[g as usize] = self.round;
        true
    }
}

/// Items by a key below some bound, in one arena, each key's in the order
/// they came.
struct Grouped<T> {
    start: Vec<u32>,
    items: Vec<T>,
}

impl<T: Copy + Default> Grouped<T> {
    /// The `items`, each with its key below `keys`, counted by key and
    /// then placed.
    fn new(keys: usize, items: impl Iterator<Item = (usize, T)> + Clone) -> Self {
        let mut start = vec![0u32; keys + 2];
        for (k, _) in items.clone() {
            start[k + 2] += 1;
        }
        for k in 2..start.len() {
            start[k] += start[k - 1];
        }
        let mut placed = vec![T::default(); start[keys + 1] as usize];
        for (k, x) in items {
            let at = &mut start[k + 1];
            placed[*at as usize] = x;
            *at += 1;
        }
        start.pop();
        Self {
            start,
            items: placed,
        }
    }

    fn of(&self, k: usize) -> &[T] {
        &self.items[self.start[k] as usize..self.start[k + 1] as usize]
    }
}

/// A step of the hash of a key, one word of it at a time.
fn mix(h: u64, x: u32) -> u64 {
    (h.rotate_left(5) ^ u64::from(x)).wrapping_mul(0x517c_c1b7_2722_0a95)
}

/// One level's classes by the hash of their key, their residuals and
/// groups: the first class of each hash, and after each class the next of
/// its hash. The keys are the level's own arenas, not copied into an arena
/// of their own.
#[derive(Default)]
struct Keys {
    first: FxHashMap<u64, u32>,
    next: Vec<u32>,
}

impl Keys {
    fn clear(&mut self) {
        self.first.clear();
        self.next.clear();
    }
    /// The number of the class of `level` whose residuals are those on its
    /// arena from `at` and whose groups are `ones`, the key of hash `h`;
    /// where there is none, the number the class of that key takes, which
    /// the caller adds next.
    fn find_or_insert(&mut self, level: &Level, at: usize, ones: &[u32], h: u64) -> u32 {
        let n = self.next.len() as u32;
        let mut k = *self.first.entry(h).or_insert(n);
        if k != n {
            let residuals = &level.residuals[at..];
            loop {
                let c = &level.classes[k as usize];
                let held = level.residuals(c);
                if c.live
                    && held.len() == residuals.len()
                    && held.iter().zip(residuals).all(|(x, y)| x.0 == y.0)
                    && level.ones(c) == ones
                {
                    return k;
                }
                match self.next[k as usize] {
                    u32::MAX => break,
                    j => k = j,
                }
            }
            self.next[k as usize] = n;
        }
        self.next.push(u32::MAX);
        n
    }
    /// The keys of `level`'s classes, by the hashes they keep, each class
    /// a key of its own; a leaf's class is hashed here.
    fn rebuild(&mut self, level: &mut Level) {
        self.first.clear();
        self.next.clear();
        self.next.resize(level.classes.len(), u32::MAX);
        for k in 0..level.classes.len() {
            let c = level.classes[k];
            if !c.live {
                continue;
            }
            let hash = match c.hash {
                UNHASHED => {
                    let hash =
                        key_hash(level.residuals(&c).iter().map(|&(r, _)| r), level.ones(&c));
                    level.classes[k].hash = hash;
                    hash
                }
                hash => hash,
            };
            let mut j = *self.first.entry(hash).or_insert(k as u32);
            if j != k as u32 {
                while self.next[j as usize] != u32::MAX {
                    j = self.next[j as usize];
                }
                self.next[j as usize] = k as u32;
            }
        }
    }
}

/// Where the key's residual cubes end and its groups begin.
const SEP: u32 = u32::MAX;

/// What `alone_of` holds for a left class whose class alone is not looked
/// up yet, and for one that has none.
const UNKNOWN: u32 = u32::MAX;
const REFUSED: u32 = u32::MAX - 1;

/// The arenas that grow with a build, which [`build`] keeps from one build
/// to the next, emptied, rather than growing them from empty in each build.
#[derive(Default)]
pub(crate) struct Scratch {
    levels: Vec<Level>,
    spare: Vec<Level>,
    key: Vec<u32>,
    here: Vec<(u32, u32)>,
    pos_key: Vec<u32>,
    pos_here: Vec<(u32, u32)>,
    pos_ones: Vec<u32>,
    pairs: Pairs,
    offsets: Vec<u32>,
    by_class: Vec<ChildPair>,
    /// The arena [`Level::normalize`] writes values on.
    values: Vec<u64>,
    /// Of each leaf a lift took in, the value of the representatives of the
    /// classes it passed up ([`Class`]).
    defaults: Vec<u64>,
}

impl Scratch {
    /// The most bytes the arenas keep between builds, of which a large
    /// build would hold its own through the rest of the run.
    const KEPT: usize = 1 << 22;

    /// Emptied, every level spare, and each arena past what is left of
    /// [`Self::KEPT`] released.
    fn trim(&mut self) {
        fn keep<T>(v: &mut Vec<T>, budget: &mut usize) {
            v.clear();
            let bytes = v.capacity() * std::mem::size_of::<T>();
            match budget.checked_sub(bytes) {
                Some(left) => *budget = left,
                None => *v = Vec::new(),
            }
        }
        let mut budget = Self::KEPT;
        let b = &mut budget;
        let Scratch {
            levels,
            spare,
            key,
            here,
            pos_key,
            pos_here,
            pos_ones,
            pairs,
            offsets,
            by_class,
            values,
            defaults,
        } = self;
        let Pairs {
            fresh,
            ones,
            span_of,
            pass,
            branches,
            left_split,
            right_split,
            emitted,
            alone_of,
        } = pairs;
        spare.append(levels);
        for level in spare.iter_mut() {
            keep(&mut level.classes, b);
            keep(&mut level.residuals, b);
            keep(&mut level.ones, b);
            keep(&mut level.values, b);
            level.lifted = false;
            level.dead = 0;
        }
        keep(key, b);
        keep(here, b);
        keep(fresh, b);
        keep(ones, b);
        keep(pos_key, b);
        keep(pos_here, b);
        keep(pos_ones, b);
        keep(span_of, b);
        keep(pass, b);
        keep(branches, b);
        keep(left_split, b);
        keep(right_split, b);
        keep(emitted, b);
        keep(offsets, b);
        keep(by_class, b);
        keep(alone_of, b);
        keep(values, b);
        keep(defaults, b);
    }
}

/// That none of `cubes` holds and that at most one variable of each group
/// is true, exactly one where the group says so, over `vtree`, whose
/// variables the literals name; `None` where the steps a level could take
/// pass what is left of `max_work`.
///
/// At a vtree node an assignment to its variables leaves a *residual*: of
/// each cube whose literals there all hold, the literals outside, and of
/// each group, whether a variable of it there is true. Two assignments
/// with the same residual cubes and groups extend alike, and are one class;
/// a pair of child classes makes the class of its union, or none where a
/// cube's literals all hold, two variables of a group are true, or a group
/// that must have one has none. The classes of a node are disjoint, and
/// each is a node of the diagram whose pairs are the pairs of child
/// classes that make it. A class keeps its residuals, each with one cube
/// that leaves it, and the values of one of its assignments: a residual
/// with literals on the right of the node above is kept there where the
/// right class's assignment makes them hold, and the cube gives what is
/// left as a residual of its own. Two cubes with the same literals outside
/// a node fare alike above it. Keyed by the cubes rather than their
/// residuals, the rows of a table that match below the level where their
/// context is read would stay apart, and a class would carry every cube
/// left alive, far more than their residuals.
///
/// The arenas that grow with the build are `scratch`'s, which it leaves
/// empty for the next. A vtree of one leaf is [`one_variable`]'s.
pub(crate) fn build(
    engine: &Engine,
    vtree: &Arc<Vtree>,
    cubes: &[Vec<Literal>],
    groups: &[(Vec<VarId>, bool)],
    max_work: u64,
    scratch: &mut Scratch,
) -> Result<Option<Tdd>, OperationError> {
    assert!(
        !vtree.node(vtree.root()).is_leaf(),
        "a vtree of one leaf is `one_variable`'s"
    );
    let built = build_on(engine, vtree, cubes, groups, max_work, scratch);
    scratch.trim();
    built
}

/// Whether the one variable of `cubes` and `groups` may be true, and
/// whether it may be false, where none of the cubes holds and each group
/// has at most one variable true, exactly one where it says so: a cube
/// rules out each value its literals all take, an empty cube both, and a
/// group that must have one rules out false.
pub(crate) fn one_variable(cubes: &[Vec<Literal>], groups: &[(Vec<VarId>, bool)]) -> (bool, bool) {
    let takes = |sign: bool| cubes.iter().any(|cube| cube.iter().all(|l| l.sign == sign));
    (
        !takes(true),
        !takes(false) && !groups.iter().any(|&(_, exactly)| exactly),
    )
}

/// The nodes of `vtree` in post-order, which the residuals' cache needs: a
/// node right after the subtrees under it, and after no other. It numbers
/// the leaves left to right, and the leaves under a node are the range of
/// numbers its span gives, by node; and the number of leaves.
fn post_order(vtree: &Vtree) -> (Vec<VtreeIdx>, Vec<(u32, u32)>, u32) {
    let n = vtree.num_nodes();
    let mut order = Vec::with_capacity(n);
    let mut spans = vec![(0u32, 0u32); n];
    let mut leaves = 0u32;
    let mut stack = Vec::with_capacity(n);
    stack.push((vtree.root(), false));
    while let Some((t, done)) = stack.pop() {
        if vtree.node(t).is_leaf() {
            spans[t.idx()] = (leaves, leaves);
            leaves += 1;
        } else if done {
            let (l, r) = vtree.children(t);
            spans[t.idx()] = (spans[l.idx()].0, spans[r.idx()].1);
        } else {
            let (l, r) = vtree.children(t);
            stack.extend([(t, true), (r, false), (l, false)]);
            continue;
        }
        order.push(t);
    }
    (order, spans, leaves)
}

fn build_on(
    engine: &Engine,
    vtree: &Arc<Vtree>,
    cubes: &[Vec<Literal>],
    groups: &[(Vec<VarId>, bool)],
    max_work: u64,
    scratch: &mut Scratch,
) -> Result<Option<Tdd>, OperationError> {
    let n = vtree.num_nodes();
    let root = vtree.root();
    let (order, spans, leaves) = post_order(vtree);
    // Each cube's literals, by the number of their leaf, twice it plus the
    // sign; a cube with a variable in both signs holds nowhere.
    let mut codes: Vec<u32> = Vec::with_capacity(cubes.iter().map(Vec::len).sum());
    let mut start: Vec<u32> = Vec::with_capacity(cubes.len() + 1);
    start.push(0);
    for cube in cubes {
        let at = codes.len();
        for l in cube {
            let leaf = vtree
                .leaf_of(l.var)
                .expect("a cube's variables are the vtree's");
            codes.push(2 * spans[leaf.idx()].0 + u32::from(l.sign));
        }
        codes[at..].sort_unstable();
        let mut kept = at;
        for k in at..codes.len() {
            if k == at || codes[k] != codes[kept - 1] {
                codes[kept] = codes[k];
                kept += 1;
            }
        }
        codes.truncate(kept);
        let lits = &codes[at..];
        if lits.is_empty() {
            return Ok(Some(engine.zero(vtree)));
        }
        if lits.windows(2).any(|w| w[0] / 2 == w[1] / 2) {
            codes.truncate(at);
            continue;
        }
        start.push(codes.len() as u32);
    }
    let mut occurs = vec![0u32; 2 * leaves as usize];
    for &x in &codes {
        occurs[x as usize] += 1;
    }
    match merge_complements(&mut codes, &mut start, &occurs, &mut Vec::new()) {
        Merged::No => {}
        Merged::Yes => {
            occurs.fill(0);
            for &x in &codes {
                occurs[x as usize] += 1;
            }
        }
        Merged::ToTrue => return Ok(Some(engine.zero(vtree))),
    }
    // The cubes with each literal and its place in the cube's, by code;
    // the groups with a variable at each leaf, by leaf number; the node
    // where each group's variables meet, and the groups that must have one
    // that meet at each node.
    let with_literal = by_literal(&codes, &start, leaves);
    let leaf_number = |v: VarId| {
        spans[vtree
            .leaf_of(v)
            .expect("a group's variables are the vtree's")
            .idx()]
        .0 as usize
    };
    let groups_at = Grouped::new(
        leaves as usize,
        groups
            .iter()
            .enumerate()
            .flat_map(|(g, (vars, _))| vars.iter().map(move |&v| (leaf_number(v), g as u32))),
    );
    let group_lca: Vec<VtreeIdx> = groups
        .iter()
        .map(|(vars, _)| {
            let leaf = |v: VarId| {
                vtree
                    .leaf_of(v)
                    .expect("a group's variables are the vtree's")
            };
            vars.iter()
                .map(|&v| leaf(v))
                .reduce(|m, l| vtree.lca(m, l))
                .expect("a group has a variable")
        })
        .collect();
    let exact_at = Grouped::new(
        n,
        groups
            .iter()
            .zip(&group_lca)
            .enumerate()
            .filter(|(_, ((_, exactly), _))| *exactly)
            .map(|(g, (_, meet))| (meet.idx(), g as u32)),
    );
    let count = start.len() - 1;
    let private: Vec<(u32, u32)> = start
        .windows(2)
        .map(|w| {
            let lits = &codes[w[0] as usize..w[1] as usize];
            let alone = |x: &u32| occurs[*x as usize] == 1;
            match (lits.iter().position(alone), lits.iter().rposition(alone)) {
                (Some(p), Some(q)) => (p as u32, q as u32),
                _ => (u32::MAX, u32::MAX),
            }
        })
        .collect();
    drop(occurs);
    let cubes = Cubes::new(codes, start);
    let kept = kept_bits(vtree, &spans, &order, &cubes, &with_literal, leaves);
    // The residual of each cube, interned by its literals, as of the last
    // node that took one of its literals in: a cube's residual changes only
    // where its literals meet, and a class reads it at a node only when the
    // cube has a literal under that node, which the post-order walk visited
    // last.
    let group_of: Vec<u32> = (0..leaves as usize)
        .map(|k| groups_at.of(k).first().copied().unwrap_or(u32::MAX))
        .collect();
    let mut res = Residuals {
        // Each literal makes a residual at its leaf, and a node where the
        // cube's literals meet one more; few go on the chains.
        made: Vec::with_capacity(cubes.codes.len()),
        first: FxHashMap::default(),
        next: Vec::with_capacity(cubes.codes.len()),
        id: vec![u32::MAX; count],
        at: vec![u32::MAX; count],
        forced: Vec::with_capacity(cubes.codes.len()),
        group_of,
        private,
    };
    let mut forcing = Forcing {
        seen: vec![0u32; groups.len()],
        round: 0,
    };
    let mut b = Tdd::builder(engine, vtree)?;
    // The levels of the nodes whose parent has not read them yet, which the
    // post-order leaves on a stack, and the levels a node has read,
    // emptied, whose arenas the next levels take instead of growing their
    // own. A leaf's key, residuals and groups, of its negative value and,
    // `pos_`, of its positive one.
    let Scratch {
        levels,
        spare,
        key,
        here,
        pos_key,
        pos_here,
        pos_ones,
        pairs,
        offsets,
        by_class,
        values: spare_values,
        defaults,
    } = scratch;
    pairs.span_of.resize(count, (u32::MAX, 0, 0, 0));
    defaults.clear();
    defaults.resize((leaves as usize).div_ceil(64), 0);
    let shape = Shape {
        spans: &spans,
        cubes: &cubes,
        group_lca: &group_lca,
        exact_at: &exact_at,
        kept: &kept,
    };
    let mut work = 0u64;
    let mut output: Option<NodeIdx> = None;
    let mut keys = Keys::default();
    // The node whose level's classes `keys` holds.
    let mut keys_for: Option<VtreeIdx> = None;
    for t in order {
        if vtree.node(t).is_leaf() {
            let leaf = spans[t.idx()].0;
            // The key and residuals of each value of the variable, or
            // `None` where it makes a cube hold or leaves a group without
            // its one.
            let mut value = |sign: bool,
                             key: &mut Vec<u32>,
                             here: &mut Vec<(u32, u32)>,
                             ones: &mut Vec<u32>|
             -> bool {
                key.clear();
                here.clear();
                ones.clear();
                for &(c, i) in with_literal.of(2 * leaf as usize + usize::from(sign)) {
                    let Some(r) = res.here(&cubes, c, t, (i as usize, i as usize + 1)) else {
                        return false;
                    };
                    here.push((r, c));
                }
                here.sort_unstable_by_key(|&(r, _)| r);
                here.dedup_by_key(|&mut (r, _)| r);
                key.extend(here.iter().map(|&(r, _)| r));
                key.push(SEP);
                for &g in groups_at.of(leaf as usize) {
                    if group_lca[g as usize] == t {
                        if groups[g as usize].1 && !sign {
                            return false;
                        }
                    } else if sign {
                        ones.push(g);
                    }
                }
                let residuals = key.len() - 1;
                forcing.start();
                if !key[..residuals]
                    .iter()
                    .all(|&r| forcing.admits(res.forced[r as usize], ones))
                {
                    return false;
                }
                key.extend_from_slice(ones);
                true
            };
            let mut level = spare.pop().unwrap_or_default();
            let pos = value(true, pos_key, pos_here, pos_ones);
            let neg = value(false, key, here, &mut pairs.ones);
            match (pos, neg) {
                (true, true) if pos_key == key => {
                    level.add(pos_here, pos_ones, true, leaf, ONE_LEAF_IDX)
                }
                _ => {
                    if pos {
                        level.add(pos_here, pos_ones, true, leaf, POS_LEAF_IDX);
                    }
                    if neg {
                        level.add(here, &pairs.ones, false, leaf, NEG_LEAF_IDX);
                    }
                }
            }
            if level.classes.is_empty() {
                b.abandon(engine);
                return Ok(Some(engine.zero(vtree)));
            }
            levels.push(level);
            continue;
        }
        let (l, r) = vtree.children(t);
        let right = levels.pop().expect("children first");
        let mut left = levels.pop().expect("children first");
        let (t_lo, t_hi) = spans[t.idx()];
        let level_work_before = work;
        let carried = |level: &Level| {
            level
                .classes
                .iter()
                .map(|c| level.residuals(c).len() as u64)
                .sum::<u64>()
        };
        // A leaf of no group on the left whose cubes all have a literal on
        // the right: a class of the right that no cube of the leaf's has a
        // residual of makes, with each class of the leaf's, the class of
        // its own residuals and groups. Such a class passes up as it is,
        // in its place on the right's level, which becomes the node's
        // ([`Level::lifted`]): its node is the leaf's classes paired with
        // its own, and the classes a cube of the leaf's reads are taken
        // out and paired anew. A table's switches lie so on a spine above
        // its parents' copies, where every class of a row still to come
        // passes each switch of the rows before. A run of lifts begins on
        // a level of [`LIFT_FROM`] classes or more.
        let lifts = (right.lifted || right.classes.len() >= LIFT_FROM)
            && vtree.node(l).is_leaf()
            && groups_at.of(t_lo as usize).is_empty()
            && [2 * t_lo, 2 * t_lo + 1].iter().all(|&x| {
                with_literal.of(x as usize).iter().all(|&(c, i)| {
                    cubes
                        .lits(c)
                        .get(i as usize + 1)
                        .is_some_and(|&y| y / 2 <= t_hi)
                })
            });
        if lifts {
            let mut level = right;
            let r_lo = spans[r.idx()].0;
            // A class passed up takes the first class of the leaf's value,
            // as its first pair would.
            let first = &left.classes[0];
            if left.values(first)[0] & 1 == 1 {
                defaults[t_lo as usize / 64] |= 1 << (t_lo % 64);
            }
            if !level.lifted {
                for k in 0..level.classes.len() {
                    level.classes[k].read =
                        nearest_read(&cubes, &res, level.residuals(&level.classes[k]), r_lo);
                }
                level.lifted = true;
            }
            if keys_for != Some(r) {
                keys.rebuild(&mut level);
            }
            // The classes a cube of the leaf's reads, taken out, each with
            // its values from the right's first leaf.
            let mut taken = spare.pop().unwrap_or_default();
            for k in 0..level.classes.len() {
                let c = level.classes[k];
                if !c.live || c.read != t_lo {
                    continue;
                }
                let at = taken.residuals.len();
                taken.residuals.extend_from_slice(level.residuals(&c));
                let v = taken.values.len();
                let lifted = (c.lo - r_lo).min(kept[r.idx()]);
                let bits = append_bits(
                    &mut taken.values,
                    0,
                    defaults,
                    r_lo as usize,
                    lifted as usize,
                );
                append_bits(
                    &mut taken.values,
                    bits,
                    level.values(&c),
                    0,
                    (kept[r.idx()] - lifted) as usize,
                );
                taken.push(at, level.ones(&c), v, r_lo, c.hash, c.node);
                level.classes[k].live = false;
                level.dead += 1;
            }
            let (nl, passed, nt) = (
                left.classes.len() as u64,
                level.live() as u64,
                taken.classes.len() as u64,
            );
            if work + nl * passed + nl * nt + nt * carried(&left) + nl * carried(&taken) > max_work
            {
                b.abandon(engine);
                return Ok(None);
            }
            pairs.emitted.clear();
            let before = level.classes.len();
            join(
                &shape,
                &mut res,
                &mut forcing,
                pairs,
                &mut keys,
                t,
                (l, r),
                &left,
                &taken,
                &mut level,
                &mut work,
            );
            for k in before..level.classes.len() {
                level.classes[k].read =
                    nearest_read(&cubes, &res, level.residuals(&level.classes[k]), t_lo);
            }
            keys_for = Some(t);
            // A pair a class passed up writes.
            work += nl * passed;
            if let Err(e) = engine.limits().poll_host_work(work - level_work_before) {
                b.abandon(engine);
                return Err(e);
            }
            if level.live() == 0 {
                b.abandon(engine);
                return Ok(Some(engine.zero(vtree)));
            }
            // The pairs by class: of a class passed up, the leaf's classes
            // with its own, then those the classes taken out made.
            offsets.clear();
            offsets.resize(level.classes.len() + 1, 0);
            for (k, c) in level.classes[..before].iter().enumerate() {
                if c.live {
                    offsets[k + 1] = nl as u32;
                }
            }
            for &(k, _) in pairs.emitted.iter() {
                debug_assert!(
                    level.classes[k as usize].live,
                    "a pair makes a class of the node"
                );
                offsets[k as usize + 1] += 1;
            }
            for k in 0..level.classes.len() {
                offsets[k + 1] += offsets[k];
            }
            by_class.clear();
            by_class.resize(
                offsets[level.classes.len()] as usize,
                ChildPair::new(NodeIdx(0), NodeIdx(0)),
            );
            for (k, c) in level.classes[..before].iter().enumerate() {
                if c.live {
                    for a in &left.classes {
                        by_class[offsets[k] as usize] = ChildPair::new(a.node, c.node);
                        offsets[k] += 1;
                    }
                }
            }
            for &(k, pair) in pairs.emitted.iter() {
                let at = &mut offsets[k as usize];
                by_class[*at as usize] = pair;
                *at += 1;
            }
            b.reserve(engine, t, level.live(), by_class.len())?;
            let mut from = 0;
            for (class, &to) in level.classes.iter_mut().zip(offsets.iter()) {
                let to = to as usize;
                if class.live {
                    class.node = b.push(engine, t, &by_class[from..to])?;
                }
                from = to;
            }
            // Taken out, the classes are dropped once they are as many as
            // those left.
            if level.dead > level.live() {
                let mut kept = 0;
                for k in 0..level.classes.len() {
                    if level.classes[k].live {
                        level.classes[kept] = level.classes[k];
                        kept += 1;
                    }
                }
                level.classes.truncate(kept);
                level.dead = 0;
                keys.rebuild(&mut level);
            }
            if t == root {
                debug_assert_eq!(level.live(), 1, "every cube and group closes at the root");
                output = Some(level.classes.iter().find(|c| c.live).expect("a class").node);
                spare.push(level.cleared());
            } else {
                levels.push(level);
            }
            spare.extend([left.cleared(), taken.cleared()]);
            continue;
        }
        let mut right = right;
        left.normalize(spans[l.idx()].0, kept[l.idx()], defaults, spare_values);
        right.normalize(spans[r.idx()].0, kept[r.idx()], defaults, spare_values);
        // A pair of classes costs one step, and one per residual the two
        // carry where it sets no group twice; where the level's pairs could
        // cost more than what is left, it gives up before trying any. A
        // build that gives up in the middle of a level has spent most of
        // what it wastes there, and on the networks no build that finishes
        // has a level whose bound passes what is left.
        let (nl, nr) = (left.classes.len() as u64, right.classes.len() as u64);
        if work + nl * nr + nr * carried(&left) + nl * carried(&right) > max_work {
            b.abandon(engine);
            return Ok(None);
        }
        keys.clear();
        pairs.emitted.clear();
        let mut level = spare.pop().unwrap_or_default();
        join(
            &shape,
            &mut res,
            &mut forcing,
            pairs,
            &mut keys,
            t,
            (l, r),
            &left,
            &right,
            &mut level,
            &mut work,
        );
        keys_for = Some(t);
        if let Err(e) = engine.limits().poll_host_work(work - level_work_before) {
            b.abandon(engine);
            return Err(e);
        }
        let made = level.classes.len();
        if made == 0 {
            b.abandon(engine);
            return Ok(Some(engine.zero(vtree)));
        }
        // The pairs by class, each class's in the order they were made.
        offsets.clear();
        offsets.resize(made + 1, 0);
        for &(k, _) in pairs.emitted.iter() {
            offsets[k as usize + 1] += 1;
        }
        for k in 0..made {
            offsets[k + 1] += offsets[k];
        }
        by_class.clear();
        by_class.resize(pairs.emitted.len(), ChildPair::new(NodeIdx(0), NodeIdx(0)));
        for &(k, pair) in pairs.emitted.iter() {
            let at = &mut offsets[k as usize];
            by_class[*at as usize] = pair;
            *at += 1;
        }
        b.reserve(engine, t, made, pairs.emitted.len())?;
        let mut from = 0;
        for (class, &to) in level.classes.iter_mut().zip(offsets.iter()) {
            let to = to as usize;
            let node = b.push(engine, t, &by_class[from..to])?;
            class.node = node;
            from = to;
        }
        if t == root {
            debug_assert_eq!(made, 1, "every cube and group closes at the root");
            output = Some(level.classes[0].node);
            spare.push(level.cleared());
        } else {
            levels.push(level);
        }
        spare.extend([left.cleared(), right.cleared()]);
    }
    let output = output.expect("the root is internal");
    let f = b
        .finish(TddNodeId {
            vtree: root,
            local: output,
        })
        .map_err(OperationError::from)?;
    Ok(Some(f))
}

/// The cubes with each literal, by code, each with the place of the
/// literal in the cube's.
fn by_literal(codes: &[u32], start: &[u32], leaves: u32) -> Grouped<(u32, u32)> {
    let cubes = start.windows(2).enumerate();
    Grouped::new(
        2 * leaves as usize,
        cubes.flat_map(|(c, w)| {
            codes[w[0] as usize..w[1] as usize]
                .iter()
                .enumerate()
                .map(move |(i, &x)| (x as usize, (c as u32, i as u32)))
        }),
    )
}

/// Of each node of `vtree`, by index, how many values its classes keep
/// from its first leaf: up to its last leaf at which some cube has a
/// literal and also one at a leaf before the node's, none where it has no
/// such leaf, and one at a leaf. [`join`] reads a right class's values
/// only at the literals on the right of a left class's residual cubes,
/// each of a cube with a literal on the left, so no node above reads a
/// node's leaves past that last one. A leaf stays so read at the nodes
/// above it while their leaves begin after the first literal of a cube
/// that reads it, its `reach`: the leaves a class keeps are among those
/// its children's classes keep, and those a lift took in are `defaults`'
/// ([`Level::normalize`]). With a bit for every leaf under its node, each
/// class of a right-linear vtree would write a bit for every leaf to its
/// right: quadratic on TPC-H's q18, whose cubes of a customer, one of its
/// orders and one of the order's lineitems are read only at the nodes whose
/// first leaf is the customer's.
///
/// The leaves are taken last first, and at each the nodes whose first leaf
/// it is, from below, each asking for the last leaf under it among those
/// whose reach is before its first: a leaf leaves that set at its reach,
/// and is linked to the leaf before it, so that the last leaf of the set at
/// or before a node's last is found along the links, halved as they are
/// walked. The leaves that leave at a leaf are read off the cubes whose
/// first literal is there (`with_literal`).
fn kept_bits(
    vtree: &Vtree,
    spans: &[(u32, u32)],
    order: &[VtreeIdx],
    cubes: &Cubes,
    with_literal: &Grouped<(u32, u32)>,
    leaves: u32,
) -> Vec<u32> {
    let mut reach: Vec<u32> = (0..leaves).collect();
    for c in 0..cubes.start.len() as u32 - 1 {
        let lits = cubes.lits(c);
        for &x in &lits[1..] {
            let r = &mut reach[x as usize / 2];
            *r = (*r).min(lits[0] / 2);
        }
    }
    // `up[x + 1]` leads to the last leaf at or before `x` in the set, and
    // `up[0]` stands for none.
    let mut up: Vec<u32> = (0..=leaves).collect();
    let mut kept = vec![1u32; vtree.num_nodes()];
    for &leaf in order.iter().rev().filter(|&&t| vtree.node(t).is_leaf()) {
        let j = spans[leaf.idx()].0;
        for &(c, i) in with_literal
            .of(2 * j as usize)
            .iter()
            .chain(with_literal.of(2 * j as usize + 1))
        {
            if i == 0 {
                for &y in &cubes.lits(c)[1..] {
                    if reach[y as usize / 2] == j {
                        up[y as usize / 2 + 1] = y / 2;
                    }
                }
            }
        }
        if reach[j as usize] == j {
            up[j as usize + 1] = j;
        }
        let mut u = leaf;
        while let Some(t) = vtree.node(u).parent().filter(|&t| vtree.children(t).0 == u) {
            let mut i = spans[t.idx()].1 + 1;
            while up[i as usize] != i {
                let p = up[up[i as usize] as usize];
                up[i as usize] = p;
                i = p;
            }
            kept[t.idx()] = i.saturating_sub(j);
            u = t;
        }
    }
    kept
}

/// The nearest leaf before `lo` at which a cube of `residuals` has a
/// literal, [`NO_LEAF`] for none: the residuals are a class's of a node
/// whose leaves begin at `lo`, so each cube's literals under the node are
/// those its residual left out, which a residual that its own cube made
/// keeps the place of.
fn nearest_read(cubes: &Cubes, res: &Residuals, residuals: &[(u32, u32)], lo: u32) -> u32 {
    let mut nearest = NO_LEAF;
    for &(rho, c) in residuals {
        let (d, a, _) = res.made[rho as usize];
        let lits = cubes.lits(c);
        let i = if d == c {
            a as usize
        } else {
            lits.partition_point(|&x| x / 2 < lo)
        };
        debug_assert_eq!(i, lits.partition_point(|&x| x / 2 < lo));
        if i > 0 {
            let leaf = lits[i - 1] / 2;
            if nearest == NO_LEAF || leaf > nearest {
                nearest = leaf;
            }
        }
    }
    nearest
}

/// What every node of a build reads: the leaves under each node, the
/// cubes, the node where each group's variables meet, and the groups that
/// must have one that meet at each node.
struct Shape<'a> {
    spans: &'a [(u32, u32)],
    cubes: &'a Cubes,
    group_lca: &'a [VtreeIdx],
    exact_at: &'a Grouped<u32>,
    kept: &'a [u32],
}

/// The arenas the pairs of a node take, emptied for the next.
///
/// Of each cube whose residual another cube made, the node it was last
/// asked at, the range of its literals under that node and where the
/// right child's begin (`span_of`).
///
/// The residuals of the children's classes that pass up as they are,
/// and the left's whose cubes have literals under both children, each
/// as the cube, the range of its literals on the right, as places in
/// the cubes' arena that a pair reads without the cube's start, and its
/// residual outside the node (`u32::MAX` for none); each class's range
/// of both. Each residual with the group it forces a variable of, read
/// here once per class rather than once per pair.
#[derive(Default)]
struct Pairs {
    fresh: Vec<(u32, u32, u32)>,
    ones: Vec<u32>,
    span_of: Vec<(u32, u32, u32, u32)>,
    pass: Vec<(u32, u32, u32)>,
    branches: Vec<(u32, u32, u32, u32, u32)>,
    left_split: Vec<(usize, usize, usize, usize)>,
    right_split: Vec<(usize, usize)>,
    emitted: Vec<(u32, ChildPair)>,
    alone_of: Vec<u32>,
}

/// The pairs of `left`'s classes with `right`'s at `t`, whose children
/// are `l` and `r`, on `pairs.emitted`, each with the number of the class
/// of `level` it makes: a class new to `level` is added to it, looked up
/// in `keys`, which holds `level`'s classes. `left` and `right` are
/// levels of `l` and `r` whose classes keep their values from the
/// children's first leaves.
#[allow(clippy::too_many_arguments)]
fn join(
    shape: &Shape,
    res: &mut Residuals,
    forcing: &mut Forcing,
    pairs: &mut Pairs,
    keys: &mut Keys,
    t: VtreeIdx,
    (l, r): (VtreeIdx, VtreeIdx),
    left: &Level,
    right: &Level,
    level: &mut Level,
    work: &mut u64,
) {
    let Shape {
        spans,
        cubes,
        group_lca,
        exact_at,
        kept,
    } = *shape;
    let Pairs {
        fresh,
        ones,
        span_of,
        pass,
        branches,
        left_split,
        right_split,
        emitted,
        alone_of,
    } = pairs;
    let right_lo = spans[r.idx()].0;
    // A class made here keeps its values up to the node's last leaf a node
    // above may read: those of the left's leaves its left class keeps, and
    // where it keeps a leaf on the right, clear bits up to the right's
    // first and those of the right's leaves it keeps, which its right
    // class keeps too ([`kept_bits`]).
    let bits = kept[t.idx()] as usize;
    let (left_bits, right_at) = (
        bits.min(kept[l.idx()] as usize),
        (right_lo - spans[l.idx()].0) as usize,
    );
    let right_bits = bits.saturating_sub(right_at);
    let t_lo = spans[t.idx()].0;
    // Each residual of one side whose cube has no literal on the other
    // passes up as it is. One of the left whose cube has literals on
    // the right stays where the right's representative makes those all
    // hold, with them taken out, where some remain. Its cube's
    // literals on the right are the residual's there, whichever cube
    // it came with, so the right's residuals of such cubes, which the
    // left's cover, are passed over. Which kind a residual is depends
    // on the node alone, so it is sorted out once per class, not per
    // pair. A residual its cube made left out that cube's literals
    // under the child it comes from, and the literals on the other side
    // follow or precede them: whether some do is a look at the literal
    // next to them, and the range of a left one's on the right a
    // search of that side alone, not of the whole cube; ranges kept per
    // cube and node mostly missed.
    {
        let t_hi = spans[t.idx()].1;
        // The range of cube `c`'s literals under the node and where
        // those on the right begin, for a residual another cube made:
        // a search of the whole cube, kept for the cube's next.
        let mut searched = |c: u32| {
            let e = &mut span_of[c as usize];
            if e.0 != t.0 {
                let (i, j) = cubes.under(c, (t_lo, t_hi));
                let p = i + cubes.lits(c)[i..j].partition_point(|&x| x / 2 < right_lo);
                *e = (t.0, i as u32, p as u32, j as u32);
            }
            (e.1 as usize, e.2 as usize, e.3 as usize)
        };
        pass.clear();
        branches.clear();
        left_split.clear();
        right_split.clear();
        for ca in &left.classes {
            let (p0, b0) = (pass.len(), branches.len());
            for &(rho, c) in left.residuals(ca) {
                let (d, a, b) = res.made[rho as usize];
                let (i, p, j) = if d == c {
                    let lits = cubes.lits(c);
                    let p = b as usize;
                    let j = if lits.get(p).is_some_and(|&x| x / 2 <= t_hi) {
                        p + lits[p..].partition_point(|&x| x / 2 <= t_hi)
                    } else {
                        p
                    };
                    (a as usize, p, j)
                } else {
                    searched(c)
                };
                debug_assert_eq!((i, j), cubes.under(c, (t_lo, t_hi)));
                debug_assert!(i < p, "a left class's cube has literals on the left");
                if p < j {
                    let id = res.here(cubes, c, t, (i, j)).unwrap_or(u32::MAX);
                    let at = cubes.start[c as usize];
                    branches.push((
                        c,
                        at + p as u32,
                        at + j as u32,
                        id,
                        res.forced.get(id as usize).copied().unwrap_or(u32::MAX),
                    ));
                } else {
                    pass.push((rho, c, res.forced[rho as usize]));
                }
            }
            left_split.push((p0, pass.len(), b0, branches.len()));
        }
        for cb in &right.classes {
            let p0 = pass.len();
            for &(rho, c) in right.residuals(cb) {
                let (d, a, _) = res.made[rho as usize];
                let on_left = if d == c {
                    a > 0 && cubes.lits(c)[a as usize - 1] / 2 >= t_lo
                } else {
                    let (i, p, _) = searched(c);
                    i < p
                };
                debug_assert_eq!(
                    on_left,
                    cubes.under(c, (t_lo, t_hi)).0 < cubes.under(c, (right_lo, t_hi)).0
                );
                if !on_left {
                    pass.push((rho, c, res.forced[rho as usize]));
                }
            }
            right_split.push((p0, pass.len()));
        }
    }
    let exact = exact_at.of(t.idx());
    // A pair whose right class passes no residual up and sets no group,
    // and where no cube of the left class's residuals with literals on
    // the right holds there, makes the class of the left class's
    // residuals that pass and its groups, or none where those force a
    // group twice, whatever the right class: it is looked up once per
    // left class (`alone_of`). On the networks such pairs are mostly a
    // table's context paired with the two classes of a row's switches.
    alone_of.clear();
    alone_of.resize(left.classes.len(), UNKNOWN);
    for (cb, &(bp0, bp1)) in right.classes.iter().zip(right_split.iter()) {
        let (b_here, b_ones, b_values) = (right.residuals(cb), right.ones(cb), right.values(cb));
        'pair: for (ia, (ca, &(ap0, ap1, ab0, ab1))) in
            left.classes.iter().zip(left_split.iter()).enumerate()
        {
            let (a_here, a_ones) = (left.residuals(ca), left.ones(ca));
            *work += 1;
            // The groups: none set on both sides, each that closes here
            // with one where it must have one.
            ones.clear();
            let (mut i, mut j) = (0, 0);
            while i < a_ones.len() || j < b_ones.len() {
                let g = match (a_ones.get(i), b_ones.get(j)) {
                    (Some(&x), Some(&y)) if x == y => continue 'pair,
                    (Some(&x), Some(&y)) if x < y => {
                        i += 1;
                        x
                    }
                    (Some(&x), None) => {
                        i += 1;
                        x
                    }
                    (_, Some(&y)) => {
                        j += 1;
                        y
                    }
                    (None, None) => unreachable!(),
                };
                ones.push(g);
            }
            for &g in exact {
                if ones.binary_search(&g).is_err() {
                    continue 'pair;
                }
            }
            ones.retain(|&g| group_lca[g as usize] != t);
            *work += (a_here.len() + b_here.len()) as u64;
            fresh.clear();
            for &(c, p, j, id, g) in &branches[ab0..ab1] {
                let on_right = &cubes.codes[p as usize..j as usize];
                debug_assert!(
                    on_right.iter().all(|&x| x / 2 - right_lo < kept[r.idx()]),
                    "a right class keeps the values a left residual reads"
                );
                if on_right
                    .iter()
                    .all(|&x| value_at(b_values, x / 2 - right_lo) == (x % 2 == 1))
                {
                    if id == u32::MAX {
                        continue 'pair;
                    }
                    fresh.push((id, c, g));
                }
            }
            fresh.sort_unstable_by_key(|&(r, _, _)| r);
            fresh.dedup_by_key(|&mut (r, _, _)| r);
            let lone = fresh.is_empty() && bp0 == bp1 && b_ones.is_empty();
            match alone_of[ia] {
                _ if !lone => {}
                UNKNOWN => {}
                REFUSED => continue 'pair,
                k => {
                    emitted.push((k, ChildPair::new(ca.node, cb.node)));
                    continue 'pair;
                }
            }
            // The residuals that pass up from each side, ascending as
            // their classes keep them, and the new ones, merged onto the
            // level's arena, where a new class keeps them, and hashed
            // as they come.
            let (from_a, from_b) = (&pass[ap0..ap1], &pass[bp0..bp1]);
            let (mut x, mut y, mut z) = (0, 0, 0);
            let at = level.residuals.len();
            forcing.start();
            let mut h = 0u64;
            loop {
                let (u, v, w) = (
                    from_a.get(x).map_or(u32::MAX, |e| e.0),
                    from_b.get(y).map_or(u32::MAX, |e| e.0),
                    fresh.get(z).map_or(u32::MAX, |e| e.0),
                );
                let r = u.min(v).min(w);
                if r == u32::MAX {
                    break;
                }
                let (_, c, g) = if u == r {
                    from_a[x]
                } else if v == r {
                    from_b[y]
                } else {
                    fresh[z]
                };
                level.residuals.push((r, c));
                x += usize::from(u == r);
                y += usize::from(v == r);
                z += usize::from(w == r);
                if !forcing.admits(g, ones) {
                    level.residuals.truncate(at);
                    if lone {
                        alone_of[ia] = REFUSED;
                    }
                    continue 'pair;
                }
                h = mix(h, r);
            }
            h = mix(h, SEP);
            for &g in ones.iter() {
                h = mix(h, g);
            }
            let k = keys.find_or_insert(level, at, ones, h);
            if lone {
                alone_of[ia] = k;
            }
            if k as usize == level.classes.len() {
                level.close(
                    at,
                    ones,
                    (left.values(ca), left_bits),
                    (b_values, right_at, right_bits),
                    t_lo,
                    h,
                    NodeIdx(0),
                );
            } else {
                level.residuals.truncate(at);
            }
            emitted.push((k, ChildPair::new(ca.node, cb.node)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cubes, Level, append_bits, by_literal, kept_bits, post_order, value_at};
    use tididi::Vtree;
    use tididi::diagram::NodeIdx;

    /// Runs of bits appended one after another, from anywhere in their
    /// sources, read back as they were, and the words past them clear.
    #[test]
    fn appended_runs_of_bits_read_back() {
        let mut rng = 0x2545_f491_4f6c_dd1du64;
        let mut next = |m: u64| {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng % m
        };
        for _ in 0..1000 {
            let mut dst = Vec::new();
            let mut bits = 0;
            let mut expected = Vec::new();
            for _ in 0..1 + next(4) {
                let src: Vec<u64> = (0..1 + next(4)).map(|_| next(u64::MAX)).collect();
                let from = next(64 * src.len() as u64) as usize;
                let n = next((64 * src.len() - from) as u64 + 1) as usize;
                expected.extend((from..from + n).map(|i| value_at(&src, i as u32)));
                bits = append_bits(&mut dst, bits, &src, from, n);
            }
            assert_eq!(bits, expected.len());
            assert_eq!(dst.len(), bits.div_ceil(64));
            for (i, &b) in expected.iter().enumerate() {
                assert_eq!(value_at(&dst, i as u32), b, "bit {i}");
            }
            for i in bits..64 * dst.len() {
                assert!(!value_at(&dst, i as u32), "bit {i} past the runs");
            }
        }
    }

    /// A class's values are a prefix of the left part's bits, and where it
    /// takes some of the right's, clear bits up to where they begin and a
    /// prefix of the right's, wherever the widths fall on words, and its
    /// bits past them clear; each part keeps bits past those taken.
    #[test]
    fn a_class_joins_its_parts_bits() {
        let mut rng = 0x2545_f491_4f6c_dd1du64;
        let mut next = |m: u64| {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng % m
        };
        for round in 0..4000 {
            let (left_kept, right_kept) = (1 + next(200) as usize, 1 + next(200) as usize);
            let left_bits = if round % 2 == 0 {
                left_kept
            } else {
                next(left_kept as u64 + 1) as usize
            };
            let right_at = left_bits
                + if round % 3 == 0 {
                    0
                } else {
                    next(140) as usize
                };
            let right_bits = match round % 5 {
                0 => 0,
                1 | 2 => right_kept,
                _ => next(right_kept as u64 + 1) as usize,
            };
            let mut words = |bits: usize| -> Vec<u64> {
                let mut w: Vec<u64> = (0..bits.div_ceil(64)).map(|_| next(u64::MAX)).collect();
                if !bits.is_multiple_of(64) {
                    *w.last_mut().unwrap() &= (1u64 << (bits % 64)) - 1;
                }
                w
            };
            let (left, right) = (words(left_kept), words(right_kept));
            let mut level = Level::default();
            level.add(&[], &[], true, 0, NodeIdx(0));
            level.close(
                level.residuals.len(),
                &[],
                (&left, left_bits),
                (&right, right_at, right_bits),
                0,
                0,
                NodeIdx(0),
            );
            let class = level.classes[1];
            let values = level.values(&class);
            let bits = if right_bits == 0 {
                left_bits
            } else {
                right_at + right_bits
            };
            assert_eq!(values.len(), bits.div_ceil(64), "round {round}");
            for i in 0..64 * values.len() as u32 {
                let expected = match i as usize {
                    k if k < left_bits => value_at(&left, i),
                    k if k < right_at => false,
                    k if k < right_at + right_bits => value_at(&right, i - right_at as u32),
                    _ => false,
                };
                assert_eq!(
                    value_at(values, i),
                    expected,
                    "round {round}: bit {i} of {left_bits}, {right_at} and {right_bits}"
                );
            }
        }
    }

    #[test]
    fn cubes_but_a_gap_compare_as_their_literals() {
        let mut rng = 0x2545_f491_4f6c_dd1du64;
        let mut next = |m: u64| {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng % m
        };
        for _ in 0..200 {
            // Few codes, so that equal literals outside a gap are common.
            let mut codes = Vec::new();
            let mut start = vec![0];
            for _ in 0..12 {
                let mut lits: Vec<u32> = (0..1 + next(6)).map(|_| next(8) as u32).collect();
                lits.sort_unstable();
                lits.dedup();
                codes.extend(lits);
                start.push(codes.len() as u32);
            }
            let cubes = Cubes::new(codes, start);
            let gap = |c: u32, next: &mut dyn FnMut(u64) -> u64| {
                let k = cubes.lits(c).len() as u64;
                let i = next(k + 1) as usize;
                (c, i, i + next(k + 1 - i as u64) as usize)
            };
            for _ in 0..50 {
                let (x, y) = (
                    gap(next(12) as u32, &mut next),
                    gap(next(12) as u32, &mut next),
                );
                let expected = cubes
                    .without(x.0, x.1, x.2)
                    .eq(cubes.without(y.0, y.1, y.2));
                assert_eq!(
                    cubes.same_without(x, y),
                    expected,
                    "{x:?} {y:?}: {:?} {:?}",
                    cubes.lits(x.0),
                    cubes.lits(y.0)
                );
                if expected {
                    assert_eq!(
                        cubes.hash_without(x.0, x.1, x.2),
                        cubes.hash_without(y.0, y.1, y.2)
                    );
                }
            }
        }
    }

    /// Of each node, the values its classes keep run to its last leaf that
    /// a cube reads together with a leaf before the node's, against a scan
    /// of every cube at every node, on random vtrees with cubes of random
    /// leaves and of leaves near each other.
    #[test]
    fn a_node_keeps_the_values_up_to_its_last_leaf_read_from_before() {
        let mut rng = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = |m: u64| {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng % m
        };
        for seed in 0..300u64 {
            let n = 2 + next(150) as u32;
            let vtree = Vtree::random(n, seed);
            let (order, spans, leaves) = post_order(&vtree);
            assert_eq!(leaves, n);
            let (mut codes, mut start) = (Vec::new(), vec![0u32]);
            for _ in 0..next(3 * u64::from(n)) {
                let base = next(u64::from(n)) as u32;
                let mut lits: Vec<u32> = (0..1 + next(4))
                    .map(|_| {
                        let leaf = if seed % 2 == 0 {
                            next(u64::from(n)) as u32
                        } else {
                            (base + next(6) as u32).min(n - 1)
                        };
                        2 * leaf + next(2) as u32
                    })
                    .collect();
                lits.sort_unstable();
                lits.dedup_by_key(|x| *x / 2);
                codes.extend(lits);
                start.push(codes.len() as u32);
            }
            let cubes = Cubes::new(codes, start);
            let with_literal = by_literal(&cubes.codes, &cubes.start, leaves);
            let kept = kept_bits(&vtree, &spans, &order, &cubes, &with_literal, leaves);
            for &t in &order {
                let (lo, hi) = spans[t.idx()];
                let expected = if vtree.node(t).is_leaf() {
                    1
                } else {
                    let read = |x: u32| {
                        (0..cubes.start.len() as u32 - 1).any(|c| {
                            cubes.lits(c)[0] / 2 < lo && cubes.lits(c).iter().any(|&y| y / 2 == x)
                        })
                    };
                    (lo..=hi).rev().find(|&x| read(x)).map_or(0, |x| x - lo + 1)
                };
                assert_eq!(
                    kept[t.idx()],
                    expected,
                    "seed {seed}: node {} over {lo}..={hi}",
                    t.0
                );
            }
        }
    }
}
