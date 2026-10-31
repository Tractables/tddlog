//! A `families` group's block: the rules of the group as rows, laid out
//! over the copies they read.

use rustc_hash::FxHashMap;

use crate::disjoint::{find, union};
use crate::ground::AtomId;

use super::forest::Forest;

/// The rules of a group as [`nest`] lays them out, one row for each
/// distinct pair of the facts a rule reads (in the group's context order,
/// `rank`) and the input copies it reads, signed: its *context*. The
/// contexts are kept once each and a row names its own by number, so that
/// two rows' contexts compare as numbers: the rules of one row of a table,
/// one for each value, read one context.
///
/// Each list is kept end to end with the others of its kind, by range,
/// rather than a list per row and context.
#[derive(Default)]
pub(super) struct Rows {
    /// The facts each row reads, by `fact_ends`.
    pub(super) facts: Vec<AtomId>,
    pub(super) fact_ends: Vec<usize>,
    /// The context each row reads, an index into the contexts.
    pub(super) context: Vec<u32>,
    /// The heads of the rules that read each row, by `head_ends`.
    pub(super) heads: Vec<AtomId>,
    pub(super) head_ends: Vec<usize>,
    /// The distinct contexts, each sorted, by `context_ends`.
    pub(super) contexts: Vec<(AtomId, bool)>,
    pub(super) context_ends: Vec<usize>,
}

impl Rows {
    /// No rows, the lists' room kept.
    pub(super) fn clear(&mut self) {
        self.facts.clear();
        self.fact_ends.clear();
        self.context.clear();
        self.heads.clear();
        self.head_ends.clear();
        self.contexts.clear();
        self.context_ends.clear();
    }

    pub(super) fn len(&self) -> usize {
        self.context.len()
    }

    /// The facts row `i` reads.
    pub(super) fn facts(&self, i: usize) -> &[AtomId] {
        &self.facts[if i == 0 { 0 } else { self.fact_ends[i - 1] }..self.fact_ends[i]]
    }

    /// The heads of the rules that read row `i`.
    fn heads(&self, i: usize) -> &[AtomId] {
        &self.heads[if i == 0 { 0 } else { self.head_ends[i - 1] }..self.head_ends[i]]
    }

    /// The copies of context `x`, signed and sorted.
    pub(super) fn context_copies(&self, x: u32) -> &[(AtomId, bool)] {
        let x = x as usize;
        &self.contexts[if x == 0 { 0 } else { self.context_ends[x - 1] }..self.context_ends[x]]
    }

    pub(super) fn num_contexts(&self) -> usize {
        self.context_ends.len()
    }

    /// The input copies row `i` reads, signed and sorted.
    fn copies(&self, i: usize) -> &[(AtomId, bool)] {
        self.context_copies(self.context[i])
    }
}

/// A part of a group in [`nest`]: `Ok` with its rows, nested in a block of
/// their own, or `Err` with its facts and its rows, laid on the spine.
type Part = Result<Vec<usize>, (Vec<AtomId>, Vec<usize>)>;

/// The parts of a group's rows ([`parts_of`]), by the least of their
/// members: each part's members and its own copies.
type PartsByRoot = FxHashMap<usize, (Vec<usize>, Vec<AtomId>)>;

/// Whether the parts `items` of the rows `members` go side by side
/// ([`nest`]): each part needs a copy that every one of its rows reads
/// positively and no row of another part does, and every copy of `bottom`
/// has to be read in one sign by every member.
///
/// Over a node that holds some of the parts, a formula of the table is
/// open on whether one of them holds and what it chose: the rows read the
/// copies below alike, and of the parts' own copies at most one holds.
/// Below a spine every row still to come is a class of its own. A copy of
/// `bottom` that one part reads positively and the others negated, its
/// value where the parts read the other values negated, keeps the parts on
/// the spine: placed beside its part, it made the walk's formulas larger on
/// tables of many rows of one switch each.
fn side_by_side(
    rows: &Rows,
    scratch: &mut NestScratch,
    members: &[usize],
    items: &[(u32, usize, Part)],
    bottom: &[AtomId],
) -> bool {
    const NONE: u32 = u32::MAX;
    const MANY: u32 = u32::MAX - 1;
    if items.len() < 2 {
        return false;
    }
    let part = |item: &Part| match item {
        Ok(part) | Err((_, part)) => part.len() as u32,
    };
    // Per copy: the rows that read it positively and negatively, and the
    // part that reads it positively, `MANY` for two or more; found by the
    // copy's mark, not looked up in a map.
    let mut read: Vec<(u32, u32, u32)> = Vec::new();
    scratch.read.next();
    for (k, (_, _, item)) in items.iter().enumerate() {
        let (Ok(rows_of) | Err((_, rows_of))) = item;
        for (x, n) in scratch.contexts_of(rows, rows_of) {
            for &(c, positive) in rows.context_copies(x) {
                let at = match scratch.read.get(c) {
                    Some(at) => at as usize,
                    None => {
                        scratch.read.set(c, read.len() as u32);
                        read.push((0, 0, NONE));
                        read.len() - 1
                    }
                };
                let e = &mut read[at];
                if positive {
                    e.0 += n;
                    e.2 = if e.2 == NONE || e.2 == k as u32 {
                        k as u32
                    } else {
                        MANY
                    };
                } else {
                    e.1 += n;
                }
            }
        }
    }
    let all = members.len() as u32;
    let entry = |c: AtomId| scratch.read.get(c).map(|at| read[at as usize]);
    let alike = |c: &AtomId| {
        entry(*c).is_some_and(|(positive, negative, _)| positive == all || negative == all)
    };
    let selects = |c: AtomId, k: usize| {
        entry(c).is_some_and(|(positive, _, by)| by == k as u32 && positive == part(&items[k].2))
    };
    bottom.iter().all(alike)
        && items.iter().enumerate().all(|(k, (_, _, item))| {
            let (Ok(rows_of) | Err((_, rows_of))) = item;
            rows.copies(rows_of[0])
                .iter()
                .any(|&(c, positive)| positive && selects(c, k))
        })
}

/// The block over the rows `members` of a group ([`Rows`]); `placed` the
/// copies already below and the facts already above.
///
/// The copies every rule reads go in a balanced join at the bottom, and the
/// rules fall apart into parts that share no other fact or copy, on a
/// right-linear spine above, the first used deepest. A part whose rules all
/// read the copies below alike, one value of them, and read copies of their
/// own is a block of its own over those: nothing in it tells the values
/// below apart. The facts of any other part go on the spine itself, its own
/// copies at the bottom with the common ones. A table is thus its rows'
/// switches above its parents' copies, and a table split on a parent
/// ([`crate::indicators`]) a spine of its parts above that parent's copies,
/// each part its rows above its own copies of the others; where each part
/// has a copy that only its rows read positively, the parent's copy of its
/// value, and every copy below is read in one sign by every row, the parts
/// are side by side in a balanced tree above those copies
/// ([`side_by_side`]).
///
/// Where every head reads a part's facts under one context, as the
/// switches of one row of a table, the facts go in a balanced tree to the
/// right of what lies below them rather than on the spine: a node over the
/// tree has a class per value the row can take, where on the spine every
/// level between the row's first switch and its last holds a class for each
/// row still to come. A part whose heads read its facts under several
/// contexts, rows of a decision list each with a switch of its own, keeps
/// them on the spine: in a tree of their own the switches of k rows are 2^k
/// classes.
///
/// A fact that rules read under contexts that exclude each other, the
/// copies they read signed, is a switch shared between rows of a table
/// (as switch sharing makes them). It does not join parts;
/// read in two of them, it goes on a spine above them, the first used
/// deepest. At most one of its rows holds in any world, so below it a
/// node's classes are still one per row to come. Joined, the parts of a
/// split table would fall into one, its switches on one spine above all of
/// its copies.
///
/// What depends on the copies alone is worked out once per context among
/// the members rather than once per row.
pub(super) fn nest(
    forest: &mut Forest,
    rows: &Rows,
    members: Vec<usize>,
    scratch: &mut NestScratch,
) -> Option<usize> {
    let (copies, common) = copies_of(rows, scratch, &members);
    if copies.is_empty() {
        let facts = scratch.facts_of(rows, &members);
        return spine(forest, rows, scratch, &facts, &members, None);
    }
    scratch.common.next();
    for &c in &common {
        scratch.common.set(c, 0);
    }
    let any_shared = mark_shared(rows, scratch, &members);
    let (roots, parts) = parts_of(rows, scratch, &members);
    if common.is_empty() && parts.len() == 1 {
        let items: Vec<usize> = copies.iter().map(|&c| forest.leaf(c)).collect();
        let acc = forest.balanced(&items);
        for c in copies {
            scratch.place(c);
        }
        let facts = scratch.facts_of(rows, &members);
        return spine(forest, rows, scratch, &facts, &members, acc);
    }
    let hoisted = hoisted(rows, scratch, &members, &roots, any_shared);
    let mut bottom = common;
    let mut items = items_of(rows, scratch, parts, &mut bottom);
    // The parts of a split table ([`crate::indicators`]) go in the order of
    // their first rules, the order in which their copies of the other
    // parents are tied: a tie between parts apart in the layout is a class
    // at every node between them. Where each part has a copy that all its
    // rules read positively and no rule of another part does, the split
    // parent's value or a row's context, and every copy read below is read
    // alike by every rule, a node over some of the parts has a class for
    // each part under it rather than for each part to come, and the parts
    // go side by side in a balanced tree above the copies: as on
    // bn_pathfinder, where every table is split on `fault` and its parts
    // read their values alone, and in the parts of bn_barley's tables
    // split twice, above the copy of the value they were split on.
    if side_by_side(rows, scratch, &members, &items, &bottom) {
        let first_row = |item: &Part| match item {
            Ok(part) | Err((_, part)) => part.iter().copied().min().unwrap_or(usize::MAX),
        };
        let mut items: Vec<Part> = items.into_iter().map(|(_, _, item)| item).collect();
        items.sort_unstable_by_key(first_row);
        let leaves: Vec<usize> = bottom.iter().map(|&c| forest.leaf(c)).collect();
        let below = forest.balanced(&leaves);
        for c in bottom {
            scratch.place(c);
        }
        let mut blocks = Vec::with_capacity(items.len());
        for item in items {
            blocks.extend(match item {
                Ok(part) => nest(forest, rows, part, scratch),
                Err((facts, part)) => spine(forest, rows, scratch, &facts, &part, None),
            });
        }
        let acc = match (below, forest.balanced(&blocks)) {
            (Some(below), Some(parts)) => Some(forest.join(below, parts)),
            (below, parts) => below.or(parts),
        };
        return spine(forest, rows, scratch, &hoisted, &members, acc);
    }
    let leaves: Vec<usize> = bottom.iter().map(|&c| forest.leaf(c)).collect();
    let mut acc = forest.balanced(&leaves);
    for c in bottom {
        scratch.place(c);
    }
    items.sort_unstable_by_key(|&(r, first, _)| (r, first));
    for (_, _, item) in items {
        match item {
            Err((facts, part)) => acc = spine(forest, rows, scratch, &facts, &part, acc),
            Ok(part) => {
                if let Some(sub) = nest(forest, rows, part, scratch) {
                    acc = Some(match acc {
                        None => sub,
                        Some(rest) => forest.join(sub, rest),
                    });
                }
            }
        }
    }
    spine(forest, rows, scratch, &hoisted, &members, acc)
}

/// Whether every head reads `facts` under one context of copies, the
/// rules `readers`. Then a head's facts decide among its rows alone, and a
/// tree of them has a class per row. Each head's context is marked by the
/// head, `context_of`, not kept in a map made for each call.
fn one_context(
    rows: &Rows,
    mark: &mut AtomMarks,
    context_of: &mut AtomMarks,
    facts: &[AtomId],
    readers: &[usize],
) -> bool {
    mark.next();
    for &f in facts {
        mark.set(f, 0);
    }
    context_of.next();
    readers
        .iter()
        .filter(|&&i| rows.facts(i).iter().any(|&f| mark.get(f).is_some()))
        .all(|&i| {
            let x = rows.context[i];
            rows.heads(i).iter().all(|&h| match context_of.get(h) {
                Some(y) => y == x,
                None => {
                    context_of.set(h, x);
                    true
                }
            })
        })
}

/// `facts` above `acc`: where there is something below and every head of
/// the rules `readers` reads them under one context ([`one_context`]), in a
/// balanced tree to its right; else on a right-linear spine, the first
/// deepest.
fn spine(
    forest: &mut Forest,
    rows: &Rows,
    scratch: &mut NestScratch,
    facts: &[AtomId],
    readers: &[usize],
    mut acc: Option<usize>,
) -> Option<usize> {
    if let Some(below) = acc
        && !facts.is_empty()
        && one_context(rows, &mut scratch.mark, &mut scratch.seen, facts, readers)
    {
        let leaves: Vec<usize> = facts.iter().map(|&f| forest.leaf(f)).collect();
        let facts = forest.balanced(&leaves).unwrap();
        return Some(forest.join(below, facts));
    }
    for &f in facts {
        let leaf = forest.leaf(f);
        acc = Some(match acc {
            None => leaf,
            Some(rest) => forest.join(leaf, rest),
        });
    }
    acc
}

/// The copies the rows `members` read that are not placed yet, ascending,
/// and those of them every member reads, from the number of members
/// reading each copy (`scratch.count`), in one pass over their distinct
/// contexts rather than a scan of every row for each copy.
fn copies_of(
    rows: &Rows,
    scratch: &mut NestScratch,
    members: &[usize],
) -> (Vec<AtomId>, Vec<AtomId>) {
    // The distinct contexts of the members, in the order they first come,
    // each with the number of members that read it.
    let distinct = scratch.contexts_of(rows, members);
    scratch.count.next();
    let mut copies: Vec<AtomId> = Vec::new();
    for &(x, n) in &distinct {
        let mut last = None;
        for &(c, _) in rows.context_copies(x) {
            if last != Some(c) {
                let count = scratch.count.get(c);
                if count.is_none() && !scratch.placed(c) {
                    copies.push(c);
                }
                scratch.count.set(c, count.unwrap_or(0) + n);
                last = Some(c);
            }
        }
    }
    copies.sort_unstable();
    let common = copies
        .iter()
        .copied()
        .filter(|&c| scratch.count.get(c) == Some(members.len() as u32))
        .collect();
    (copies, common)
}

/// Marks in `scratch.shared` the facts of the rows `members` that are read
/// under two or more contexts that exclude each other, rows of one table
/// ([`nest`]), and whether there is one. A fact's first context is kept by
/// the row it came from, and the rest only for the facts that have more
/// than one.
fn mark_shared(rows: &Rows, scratch: &mut NestScratch, members: &[usize]) -> bool {
    scratch.first.next();
    let mut contexts: FxHashMap<AtomId, Vec<u32>> = FxHashMap::default();
    for &i in members {
        let x = rows.context[i];
        for &f in rows.facts(i) {
            if scratch.placed(f) {
                continue;
            }
            match scratch.first.get(f) {
                None => scratch.first.set(f, x),
                Some(y) if y == x => {}
                Some(y) => {
                    let seen = contexts.entry(f).or_insert_with(|| vec![y]);
                    if !seen.contains(&x) {
                        seen.push(x);
                    }
                }
            }
        }
    }
    let clash = |x: &[(AtomId, bool)], y: &[(AtomId, bool)]| {
        let (mut i, mut j) = (0, 0);
        while i < x.len() && j < y.len() {
            match x[i].0.cmp(&y[j].0) {
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal if x[i].1 != y[j].1 => return true,
                std::cmp::Ordering::Equal => (i, j) = (i + 1, j + 1),
            }
        }
        false
    };
    scratch.shared.next();
    let mut any_shared = false;
    for (&f, cs) in &contexts {
        let context = |x: u32| rows.context_copies(x);
        if cs
            .iter()
            .enumerate()
            .all(|(k, &x)| cs[k + 1..].iter().all(|&y| clash(context(x), context(y))))
        {
            scratch.shared.set(f, 0);
            any_shared = true;
        }
    }
    any_shared
}

/// The parts of the rows `members`: each member's part, by the least
/// member of it ([`union`]), and each part's members and own copies by
/// that member.
///
/// The rules are joined by a fact read in one context or a copy not all
/// of them read. The members of one context read the same such copies, so
/// a context's copies are joined at its first member, and its other
/// members to that one where it has any. Each such copy is read in one
/// part alone, one of the part's own copies, and is kept with the first
/// member to read it.
fn parts_of(
    rows: &Rows,
    scratch: &mut NestScratch,
    members: &[usize],
) -> (Vec<usize>, PartsByRoot) {
    let mut parent: Vec<usize> = (0..members.len()).collect();
    let mut own_copies: Vec<(AtomId, usize)> = Vec::new();
    scratch.first.next();
    scratch.context.next();
    for (j, &i) in members.iter().enumerate() {
        for &f in rows.facts(i) {
            if scratch.placed(f) || scratch.shared.get(f).is_some() {
                continue;
            }
            match scratch.first.get(f) {
                None => scratch.first.set(f, j as u32),
                Some(k) => {
                    union(&mut parent, j, k as usize);
                }
            }
        }
        let x = rows.context[i];
        match scratch.context.get(x) {
            Some(u32::MAX) => {}
            Some(k) => {
                union(&mut parent, j, k as usize);
            }
            None => {
                let mut own = false;
                for &(c, _) in rows.context_copies(x) {
                    if scratch.placed(c) || scratch.common.get(c).is_some() {
                        continue;
                    }
                    own = true;
                    match scratch.first.get(c) {
                        None => {
                            scratch.first.set(c, j as u32);
                            own_copies.push((c, j));
                        }
                        Some(k) => {
                            union(&mut parent, j, k as usize);
                        }
                    }
                }
                scratch
                    .context
                    .set(x, if own { j as u32 } else { u32::MAX });
            }
        }
    }
    // Each part's members, in a list sized by a count of them, and its own
    // copies. The parts then go into a map by root, in the order of their
    // roots, and come out in the map's order, which the copies at the
    // bottom follow: one insertion a part.
    let roots: Vec<usize> = (0..members.len()).map(|j| find(&mut parent, j)).collect();
    let mut size = vec![0usize; members.len()];
    for &root in &roots {
        size[root] += 1;
    }
    let mut lists: Vec<(usize, Vec<usize>, Vec<AtomId>)> = Vec::new();
    let mut list_of = vec![usize::MAX; members.len()];
    for (j, &m) in members.iter().enumerate() {
        let root = roots[j];
        if root == j {
            list_of[j] = lists.len();
            lists.push((j, Vec::with_capacity(size[j]), Vec::new()));
        }
        lists[list_of[root]].1.push(m);
    }
    for (c, j) in own_copies {
        lists[list_of[roots[j]]].2.push(c);
    }
    let mut parts = PartsByRoot::default();
    for (root, list, own) in lists {
        parts.insert(root, (list, own));
    }
    (roots, parts)
}

/// The shared facts ([`mark_shared`]) read in two parts of the rows
/// `members`, by the members' parts `roots`, in their rank order and
/// placed, to go above the parts; none where `any_shared` says no fact is
/// shared, as on the networks, whose members' facts are then not read for
/// it.
fn hoisted(
    rows: &Rows,
    scratch: &mut NestScratch,
    members: &[usize],
    roots: &[usize],
    any_shared: bool,
) -> Vec<AtomId> {
    scratch.first.next();
    let mut hoisted: Vec<AtomId> = Vec::new();
    if any_shared {
        for (j, &m) in members.iter().enumerate() {
            let root = roots[j] as u32;
            for &f in rows.facts(m) {
                if scratch.shared.get(f).is_none() {
                    continue;
                }
                match scratch.first.get(f) {
                    None => scratch.first.set(f, root),
                    Some(r) if r != root => hoisted.push(f),
                    Some(_) => {}
                }
            }
        }
    }
    hoisted.sort_unstable_by_key(|&f| scratch.rank.get(f).expect("a row's facts are its group's"));
    hoisted.dedup();
    for &f in &hoisted {
        scratch.place(f);
    }
    hoisted
}

/// Each part on the spine ([`nest`]), keyed by the rank of its first fact
/// and its first member: its rules in a block of their own, where they all
/// read the copies below alike and read copies of their own, or else its
/// facts, its own copies added to `bottom`. Only a part on the spine needs
/// its facts listed and sorted.
fn items_of(
    rows: &Rows,
    scratch: &mut NestScratch,
    parts: PartsByRoot,
    bottom: &mut Vec<AtomId>,
) -> Vec<(u32, usize, Part)> {
    let below = |scratch: &NestScratch, i: usize| {
        rows.copies(i)
            .iter()
            .copied()
            .filter(|&(c, _)| scratch.common.get(c).is_some())
            .collect::<Vec<_>>()
    };
    let mut items: Vec<(u32, usize, Part)> = Vec::new();
    for (part, mut own) in parts.into_values() {
        own.sort_unstable();
        let alike = |scratch: &mut NestScratch| {
            let first = below(scratch, part[0]);
            scratch.contexts_of(rows, &part).iter().all(|&(x, _)| {
                x == rows.context[part[0]] || {
                    let copies = rows.context_copies(x);
                    copies
                        .iter()
                        .filter(|&&(c, _)| scratch.common.get(c).is_some())
                        .eq(first.iter())
                }
            })
        };
        if !own.is_empty() && alike(scratch) {
            items.push((scratch.first_rank(rows, &part), part[0], Ok(part)));
        } else {
            let facts = scratch.facts_of(rows, &part);
            let first = facts.first().map_or(u32::MAX, |&f| {
                scratch.rank.get(f).expect("a row's facts are its group's")
            });
            bottom.extend(own);
            items.push((first, part[0], Err((facts, part))));
        }
    }
    items
}

/// Values by index (an atom, a context of [`Rows`], or a group of the
/// layout) for one use at a time: a use starts a round with [`AtomMarks::next`], and a value holds
/// only for the round it was set in, so a use costs what it touches.
/// [`nest`] runs on every group's rows again at each level of its
/// recursion, where hash maps and sets by atom would cost most of its time.
/// A slot keeps its round and its value side by side, so that a lookup
/// reads one place.
pub(super) struct AtomMarks {
    slots: Vec<(u32, u32)>,
    now: u32,
}

impl AtomMarks {
    pub(super) fn new(num_atoms: usize) -> Self {
        AtomMarks {
            slots: vec![(0, 0); num_atoms],
            now: 0,
        }
    }

    /// At least `n` slots.
    pub(super) fn reserve(&mut self, n: usize) {
        if self.slots.len() < n {
            self.slots.resize(n, (0, 0));
        }
    }

    pub(super) fn next(&mut self) {
        self.now += 1;
        if self.now == u32::MAX {
            self.slots.fill((0, 0));
            self.now = 1;
        }
    }

    pub(super) fn get(&self, a: AtomId) -> Option<u32> {
        let (round, value) = self.slots[a as usize];
        (round == self.now).then_some(value)
    }

    pub(super) fn set(&mut self, a: AtomId, value: u32) {
        self.slots[a as usize] = (self.now, value);
    }
}

/// What [`nest`] keeps by atom across one group: the rank of each of the
/// group's facts, the atoms placed so far, and marks for one step at a time,
/// by atom and by context.
pub(super) struct NestScratch {
    pub(super) rank: AtomMarks,
    placed: AtomMarks,
    count: AtomMarks,
    first: AtomMarks,
    pub(super) mark: AtomMarks,
    seen: AtomMarks,
    common: AtomMarks,
    shared: AtomMarks,
    read: AtomMarks,
    pub(super) context: AtomMarks,
}

impl NestScratch {
    pub(super) fn new(num_atoms: usize) -> Self {
        NestScratch {
            rank: AtomMarks::new(num_atoms),
            placed: AtomMarks::new(num_atoms),
            count: AtomMarks::new(num_atoms),
            first: AtomMarks::new(num_atoms),
            mark: AtomMarks::new(num_atoms),
            seen: AtomMarks::new(num_atoms),
            common: AtomMarks::new(num_atoms),
            shared: AtomMarks::new(num_atoms),
            read: AtomMarks::new(num_atoms),
            context: AtomMarks::new(0),
        }
    }

    /// A new group whose facts in rank order are `facts`, nothing placed.
    pub(super) fn start(&mut self, facts: &[AtomId]) {
        self.rank.next();
        for (i, &f) in facts.iter().enumerate() {
            self.rank.set(f, i as u32);
        }
        self.placed.next();
    }

    fn placed(&self, a: AtomId) -> bool {
        self.placed.get(a).is_some()
    }

    fn place(&mut self, a: AtomId) {
        self.placed.set(a, 0);
    }

    /// The distinct contexts of `members`, in the order they first come,
    /// each with the number of members that read it: at most one a member,
    /// for which the list is allocated.
    fn contexts_of(&mut self, rows: &Rows, members: &[usize]) -> Vec<(u32, u32)> {
        self.context.next();
        let mut distinct: Vec<(u32, u32)> = Vec::with_capacity(members.len());
        for &i in members {
            let x = rows.context[i];
            match self.context.get(x) {
                None => {
                    self.context.set(x, distinct.len() as u32);
                    distinct.push((x, 1));
                }
                Some(k) => distinct[k as usize].1 += 1,
            }
        }
        distinct
    }

    /// The facts the rows `members` read that are not placed yet, in rank
    /// order.
    fn facts_of(&mut self, rows: &Rows, members: &[usize]) -> Vec<AtomId> {
        self.seen.next();
        let mut facts: Vec<(u32, AtomId)> = Vec::new();
        for &i in members {
            for &f in rows.facts(i) {
                if !self.placed(f) && self.seen.get(f).is_none() {
                    self.seen.set(f, 0);
                    facts.push((self.rank.get(f).expect("a row's facts are its group's"), f));
                }
            }
        }
        facts.sort_unstable();
        facts.into_iter().map(|(_, f)| f).collect()
    }

    /// The rank of the first of [`NestScratch::facts_of`], `u32::MAX` for
    /// none.
    fn first_rank(&self, rows: &Rows, members: &[usize]) -> u32 {
        let mut first = u32::MAX;
        for &i in members {
            for &f in rows.facts(i) {
                if !self.placed(f) {
                    first = first.min(self.rank.get(f).expect("a row's facts are its group's"));
                }
            }
        }
        first
    }
}
