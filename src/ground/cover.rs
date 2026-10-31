//! Atoms whose rule bodies together are valid.
//!
//! A set of literals is *exhaustive* when one of them holds in every world.
//! The two literals of a fact are. So are the atoms of an exclusive set
//! ([`super::exclusive`]) whose rule bodies together are valid: each row's
//! switch chain selects one of its heads, and the rows' contexts cover every
//! value of the exhaustive sets they read. A derived atom whose rule bodies
//! together are valid is certain, and [`super::simplify::propagate_certain`]
//! takes it out of the bodies that read it.
//!
//! The case in point is a table whose rows all hold one distribution:
//! [`super::classes`] reads it through one class atom with a rule per
//! context, and the contexts cover the parents' values. The class atom is
//! certain, and the table has no parents left. bn_barley's aar_mod and
//! spndx are such tables; taking their parents away halves bn_barley_ev's
//! evidence.
//!
//! Validity is decided by merging: bodies equal but for one literal, those
//! literals being all of an exhaustive set, hold together exactly when the
//! rest holds, and the bodies are valid when merging reaches the empty body.
//! A stick-breaking chain of `n` heads merges in `n - 1` steps into its
//! context, so a table's rows reduce to their contexts and the contexts then
//! merge across the parents' values. The merged bodies themselves are not
//! kept as rules: a row whose context lost a parent reads fewer tables than
//! its siblings, and the clause encoding of tables
//! ([`crate::indicators::introduce`]) takes rows that all read one
//! atom of each of the same tables (bn_barley_ev runs out of memory so).

use crate::by_hash::ByHash;

use super::tables::Tables;
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, RulesByHead};

/// What the cover found: the atoms whose rule bodies are valid, and the
/// exclusive sets that are exhaustive.
pub(crate) struct Cover {
    /// Whether each atom is certain.
    pub(crate) certain: Vec<bool>,
    /// The exhaustive set of each atom, as its index into the tables'
    /// sets, `usize::MAX` for none.
    pub(crate) exhaustive: Vec<usize>,
}

impl Cover {
    /// The cover of `gp`, whose tables are `tables`, as described above.
    pub(crate) fn of(gp: &GroundProgram, tables: &Tables) -> Self {
        let n = gp.num_atoms();
        let by_head = tables.by_head();
        let sets = tables.sets();
        // In dependency order: an atom is decided with the atoms before it, a
        // set once its last atom is.
        let order = tables.order();
        let mut position = vec![usize::MAX; n];
        for (i, &a) in order.iter().enumerate() {
            position[a as usize] = i;
        }
        let mut closes: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (i, set) in sets.iter().enumerate() {
            let last = set
                .iter()
                .copied()
                .max_by_key(|&a| position[a as usize])
                .expect("a set has atoms");
            closes[last as usize].push(i);
        }
        let mut certain = vec![false; n];
        let mut set_of = vec![usize::MAX; n];
        // Each atom's exhaustive set and its size ([`group_of`]): `u32::MAX`
        // and 2 for a fact, its two literals; a size of 0 for an atom in no
        // set found exhaustive yet. Read from the atom's kind, its set and
        // the set's length, three places apart, for every literal the merge
        // weighs or counts.
        let mut group: Vec<(u32, u32)> = gp
            .kinds
            .iter()
            .map(|k| {
                if matches!(k, AtomKind::Fact(_)) {
                    (u32::MAX, 2)
                } else {
                    (0, 0)
                }
            })
            .collect();
        let mut merge = Merge::default();
        for &a in &order {
            certain[a as usize] = match Merge::settled(gp, &by_head[a as usize], &certain, |l| {
                group_of(n, &group, l)
            }) {
                Some(valid) => valid,
                None => {
                    merge.live_bodies(gp, by_head, &certain, &[a]);
                    merge.valid(|l| group_of(n, &group, l))
                }
            };
            for &i in &closes[a as usize] {
                merge.live_bodies(gp, by_head, &certain, &sets[i]);
                if merge.valid(|l| group_of(n, &group, l)) {
                    for &b in &sets[i] {
                        set_of[b as usize] = i;
                        group[b as usize] = (i as u32, sets[i].len() as u32);
                    }
                }
            }
        }
        Cover {
            certain,
            exhaustive: set_of,
        }
    }

    /// Makes the certain atoms certain in `gp`, the program the cover is
    /// of or one with its atoms and more after them (the conditioning's
    /// renormalized rows): each keeps one rule with an empty body.
    /// Returns how many there are. [`super::simplify::propagate_certain`]
    /// then takes them out of the bodies that read them.
    pub(crate) fn apply(&self, gp: &mut GroundProgram) -> usize {
        let count = self.certain.iter().filter(|&&c| c).count();
        if count == 0 {
            return 0;
        }
        let certain = &self.certain;
        gp.rules
            .retain(|r| !certain.get(r.head as usize).copied().unwrap_or(false));
        gp.rules.extend(
            (0..certain.len() as AtomId)
                .filter(|&a| certain[a as usize])
                .map(|head| GroundRule::new(head, Vec::new())),
        );
        count
    }
}

/// The exhaustive set a literal belongs to, as (group, size), if any: its
/// fact's two literals, or for a positive literal the exhaustive set of its
/// atom, numbered from `n`, the number of atoms. `group` holds each atom's
/// set ([`Cover::of`]); a fact is in none, so no literal belongs to two.
fn group_of(n: usize, group: &[(u32, u32)], l: GLit) -> Option<(usize, usize)> {
    let a = l.atom() as usize;
    match group[a] {
        (_, 0) => None,
        (u32::MAX, size) => Some((a, size as usize)),
        (set, size) => l.is_positive().then_some((n + set as usize, size as usize)),
    }
}

/// No body, for a group no body has read yet ([`Group`]).
const NONE: u32 = u32::MAX;

/// The bodies whose disjunction [`Merge::valid`] decides, and the terms
/// its search decides, in buffers reused from one call to the next rather
/// than a vector for every body given, every body derived and every index
/// entry.
#[derive(Default)]
struct Merge {
    /// The literals of the bodies and of the terms decided, end to end;
    /// each sorted and without repeats.
    lits: Vec<GLit>,
    /// The bodies given, as ranges of `lits`.
    bodies: Vec<(usize, usize)>,
    /// The terms decided, as ranges of `lits`, with whether merging
    /// derives each: the leaves of merging ([`Merge::valid`]), derived as
    /// given, and then the terms the search came to.
    terms: Vec<(usize, usize, bool)>,
    /// The terms by their hash, the exclusive or of their literals'
    /// ([`GLit::mixed`]).
    by_hash: ByHash,
    /// The leaves, as ranges of `order` with what each weighs
    /// ([`weight`]); and their literals, each with its group's key ([`key`]),
    /// each leaf's in the order of their keys.
    leaves: Vec<(u32, u32, f64)>,
    order: Vec<(u64, GLit)>,
    /// The groups the leaves read, numbered as they come ([`Group`]), and
    /// per group of [`group_of`] the call that numbered it and its number.
    groups: Vec<Group>,
    number: Vec<(u64, u32)>,
    /// The search's stacks ([`Merge::derived`]): the terms from the empty
    /// one to the term at hand, end to end; the leaves that hold each, by
    /// number in `leaves`, those of a term's terms with one literal more
    /// each together; where those of each term with one literal more end,
    /// and what they weigh; the literals of each group split on; and the
    /// groups a term is split on when the first fails.
    path: Vec<GLit>,
    holders: Vec<u32>,
    ends: Vec<(u32, f64)>,
    members: Vec<GLit>,
    others: Vec<(u64, u32)>,
    /// The holders of a literal of the group split on, with that literal's
    /// place among the group's, before they are put in order.
    tagged: Vec<(u32, u32)>,
    /// Per literal, twice its atom and 1 more if positive, the last split
    /// or count that met it, and its place among the literals of the group
    /// split on.
    lit_seen: Vec<(u64, u32)>,
    /// The calls, splits and counts so far.
    node: u64,
    /// Whether every leaf weighs a normal number ([`Merge::valid`]).
    weighed: bool,
}

/// A group the leaves of a call to [`Merge::valid`] read: how many leaves
/// read it, its size, the last body that read one of its literals, whether
/// the term at hand holds one, and what the last count of the groups
/// beyond a term counted of it ([`Merge::count`]): the count, the holders'
/// literals of it, and how many of its literals they read.
#[derive(Clone, Copy, Default)]
struct Group {
    reads: u32,
    size: u32,
    body: u32,
    held: bool,
    counted: u64,
    read: u32,
    members: u32,
}

/// A group's key, by which each leaf's literals are in order: the groups
/// more leaves read first, a table's parents, which every row reads,
/// before a row's switches, and then by number.
fn key(groups: &[Group], g: u32) -> u64 {
    u64::from(u32::MAX - groups[g as usize].reads) << 32 | u64::from(g)
}

/// The margin below 1 at which the holders of a term weigh too little for
/// merging to derive it ([`Merge::derived`]), far above the rounding of
/// their sum.
const SHORT: f64 = 1e-6;

impl Merge {
    /// Take the bodies of the rules of `atoms`, without the certain atoms
    /// they read and without the bodies that read one negatively.
    fn live_bodies(
        &mut self,
        gp: &GroundProgram,
        by_head: &RulesByHead,
        certain: &[bool],
        atoms: &[AtomId],
    ) {
        self.lits.clear();
        self.bodies.clear();
        for &a in atoms {
            for &ri in &by_head[a as usize] {
                let body = &gp.rules[ri].body;
                if body
                    .iter()
                    .any(|l| !l.is_positive() && certain[l.atom() as usize])
                {
                    continue;
                }
                self.push_body(body.iter().copied().filter(|l| !certain[l.atom() as usize]));
            }
        }
    }

    /// What [`Merge::valid`] says of the live bodies of `rules`, as
    /// [`Merge::live_bodies`] takes them, where that is decided before any
    /// merging: a body left empty, fewer than two bodies, or bodies whose
    /// measure is less than 1, as the rows of one value of a table. Each
    /// body is read where it is, as nearly every one is sorted and
    /// without repeats, not copied and sorted. `None` where a body is not
    /// so, or where the bodies must be merged.
    fn settled(
        gp: &GroundProgram,
        rules: &[usize],
        certain: &[bool],
        group_of: impl Fn(GLit) -> Option<(usize, usize)>,
    ) -> Option<bool> {
        let (mut bodies, mut measure, mut empty) = (0usize, 0.0f64, false);
        for &ri in rules {
            let body = &gp.rules[ri].body;
            if body
                .iter()
                .any(|l| !l.is_positive() && certain[l.atom() as usize])
            {
                continue;
            }
            if !body.is_sorted_by(|x, y| x < y) {
                return None;
            }
            let mut live = body
                .iter()
                .filter(|l| !certain[l.atom() as usize])
                .peekable();
            empty |= live.peek().is_none();
            measure += live.map(|&l| weight(&group_of, l)).product::<f64>();
            bodies += 1;
        }
        if empty {
            Some(true)
        } else if bodies < 2 || measure < 1.0 - 1e-9 {
            Some(false)
        } else {
            None
        }
    }

    /// Add a body, sorted and without repeats. Nearly every body comes so,
    /// and is only checked to be.
    fn push_body(&mut self, body: impl IntoIterator<Item = GLit>) {
        let start = self.lits.len();
        self.lits.extend(body);
        if !self.lits[start..].is_sorted_by(|x, y| x < y) {
            self.lits[start..].sort_unstable();
            let mut end = start;
            for i in start..self.lits.len() {
                if i == start || self.lits[i] != self.lits[end - 1] {
                    self.lits[end] = self.lits[i];
                    end += 1;
                }
            }
            self.lits.truncate(end);
        }
        self.bodies.push((start, self.lits.len()));
    }

    /// Whether the disjunction of the bodies taken is valid: whether
    /// merging them reaches the empty body. Two terms merge into what they
    /// share where each is it with one literal more, the literals being
    /// all of a group; a body is valid where merging takes every literal
    /// away. Such merging is a tree whose leaves are bodies, which the
    /// search builds from the empty body down ([`Merge::derived`]). Merged
    /// from the bodies up, every pair of terms that could merge would, and
    /// the empty body would come last: a table of a row for each of the 2^k
    /// values of k parents of two values makes all 3^k terms of the cube's
    /// faces first (bn_win95pts's pc2prt).
    fn valid(&mut self, group_of: impl Fn(GLit) -> Option<(usize, usize)>) -> bool {
        // One body is valid only when it is empty, and most atoms have one.
        if self.bodies.iter().any(|&(start, end)| start == end) {
            return true;
        }
        if self.bodies.len() < 2 {
            return false;
        }
        // Merging holds in the worlds where every fact is free and every
        // exhaustive set has one true atom, so bodies it finds valid cover
        // all of those, and their measures sum to 1 or more when each fact
        // is true with probability 1/2 and each atom of a set of `s` with
        // 1/s, a literal of neither counted as 1. The rows of one value of
        // a table sum to less, and are settled without merging.
        let (lits, bodies) = (&self.lits, &self.bodies);
        let measure: f64 = bodies
            .iter()
            .map(|&(start, end)| {
                lits[start..end]
                    .iter()
                    .map(|&l| weight(&group_of, l))
                    .product::<f64>()
            })
            .sum();
        if measure < 1.0 - 1e-9 {
            return false;
        }
        self.terms.clear();
        self.by_hash.clear();
        self.leaves.clear();
        self.order.clear();
        self.groups.clear();
        self.path.clear();
        self.holders.clear();
        self.ends.clear();
        self.members.clear();
        self.others.clear();
        // The leaves: the distinct bodies whose literals are in groups,
        // each in its own. Merging takes only literals of groups away, one
        // group at a time, so another body is under no merging that
        // reaches the empty body.
        self.node += 1;
        let call = self.node;
        for k in 0..self.bodies.len() {
            let (start, end) = self.bodies[k];
            let at = self.order.len();
            let (mut leaf, mut weight) = (true, 1.0);
            for &l in &self.lits[start..end] {
                let Some((g, size)) = group_of(l) else {
                    leaf = false;
                    break;
                };
                if self.number.len() <= g {
                    self.number.resize(g + 1, (0, 0));
                }
                if self.number[g].0 != call {
                    self.number[g] = (call, self.groups.len() as u32);
                    self.groups.push(Group {
                        size: size as u32,
                        body: NONE,
                        ..Group::default()
                    });
                }
                let i = self.number[g].1;
                let group = &mut self.groups[i as usize];
                if group.body == k as u32 {
                    leaf = false;
                    break;
                }
                group.body = k as u32;
                weight /= size as f64;
                self.order.push((u64::from(i), l));
                if self.lit_seen.len() <= code(l) {
                    self.lit_seen.resize(code(l) + 1, (0, 0));
                }
            }
            let body = &self.lits[start..end];
            let hash = body.iter().fold(0, |h, &l| h ^ l.mixed());
            if !leaf || self.find(body, hash).is_some() {
                self.order.truncate(at);
                continue;
            }
            for &(i, _) in &self.order[at..] {
                self.groups[i as usize].reads += 1;
            }
            self.add(start, end, hash, true);
            self.holders.push(self.leaves.len() as u32);
            self.leaves
                .push((at as u32, self.order.len() as u32, weight));
        }
        let mut sum = 0.0;
        for &(start, end, weight) in &self.leaves {
            let order = &mut self.order[start as usize..end as usize];
            for (k, _) in order.iter_mut() {
                *k = key(&self.groups, *k as u32);
            }
            order.sort_unstable_by_key(|&(k, _)| k);
            sum += weight;
        }
        // A term weighs no less than a leaf that holds it, so where every
        // leaf weighs a normal number, every weight the search takes is a
        // product of normal numbers and close to its value, and so is
        // their sum. A leaf of over a thousand literals weighs less, and
        // the search then splits a term whatever its holders weigh.
        self.weighed = self
            .leaves
            .iter()
            .all(|&(_, _, weight)| weight >= f64::MIN_POSITIVE);
        let all = self.holders.len();
        self.derived(0, 0, 0, all, sum, 1.0)
    }

    /// The term `term` of hash `hash`, if it has been decided.
    fn find(&self, term: &[GLit], hash: u64) -> Option<u32> {
        self.by_hash.find(hash, |t| {
            let (start, end, _) = self.terms[t as usize];
            self.lits[start..end] == *term
        })
    }

    /// Decide the term `lits[start..end]` of hash `hash`.
    fn add(&mut self, start: usize, end: usize, hash: u64, derived: bool) {
        self.by_hash.push(hash);
        self.terms.push((start, end, derived));
    }

    /// Whether merging derives the term `path[at..]`, of hash `hash`, which
    /// weighs `weight` ([`weight`]) and which the leaves `holders[from..to]`
    /// hold, weighing `sum` together: whether it is a leaf, or for a group
    /// none of whose literals it holds, merging derives it with each
    /// literal of the group added. Each term is decided once: two parents
    /// of a row are split on in either order. A term merging derives is
    /// held by the leaves under it, which weigh 1 together beyond it, so a
    /// term whose holders weigh less is not derived. The group split on
    /// first is the first beyond the term in the first holder's order
    /// ([`key`]): along a table's rows, its parents and then a row's
    /// switches, each read after the term by every holder. Where that
    /// fails, every other group whose literals the holders all read beyond
    /// the term is split on, those more holders read first. Counted at
    /// every term, the holders' literals would be read once per literal
    /// before them along a table's rows.
    fn derived(
        &mut self,
        at: usize,
        hash: u64,
        from: usize,
        to: usize,
        sum: f64,
        weight: f64,
    ) -> bool {
        if let Some(t) = self.find(&self.path[at..], hash) {
            return self.terms[t as usize].2;
        }
        let mut derived = false;
        if from < to && (!self.weighed || sum >= weight * (1.0 - SHORT)) {
            let (start, end, _) = self.leaves[self.holders[from] as usize];
            // A holder with no literal beyond the term is the term, a leaf.
            let first = self.order[start as usize..end as usize]
                .iter()
                .map(|&(k, _)| k as u32)
                .find(|&g| !self.groups[g as usize].held)
                .expect("a leaf is decided");
            derived = self.split_on(at, hash, from, to, first, weight);
            if !derived {
                let others = self.others.len();
                self.count(from, to, first);
                let mut k = others;
                while !derived && k < self.others.len() {
                    derived = self.split_on(at, hash, from, to, self.others[k].1, weight);
                    k += 1;
                }
                self.others.truncate(others);
            }
        }
        let start = self.lits.len();
        self.lits.extend_from_slice(&self.path[at..]);
        self.add(start, self.lits.len(), hash, derived);
        derived
    }

    /// Puts on `others` the groups other than `first` whose literals the
    /// leaves `holders[from..to]` all read beyond the term at hand, those
    /// more of them read first.
    fn count(&mut self, from: usize, to: usize, first: u32) {
        self.node += 1;
        let node = self.node;
        let here = self.others.len();
        for &h in &self.holders[from..to] {
            let (start, end, _) = self.leaves[h as usize];
            for &(k, l) in &self.order[start as usize..end as usize] {
                let g = k as u32;
                let group = &mut self.groups[g as usize];
                if group.held {
                    continue;
                }
                if group.counted != node {
                    (group.counted, group.read, group.members) = (node, 0, 0);
                    self.others.push((0, g));
                }
                group.read += 1;
                let seen = &mut self.lit_seen[code(l)].0;
                if *seen != node {
                    *seen = node;
                    group.members += 1;
                }
            }
        }
        let mut kept = here;
        for k in here..self.others.len() {
            let g = self.others[k].1;
            let group = self.groups[g as usize];
            if g != first && group.members == group.size {
                self.others[kept] = (u64::from(u32::MAX - group.read) << 32 | u64::from(g), g);
                kept += 1;
            }
        }
        self.others.truncate(kept);
        self.others[here..].sort_unstable();
    }

    /// Whether merging derives the term `path[at..]`, of hash `hash` and
    /// weight `weight`, from the terms with each literal of the group `g`
    /// added: whether the leaves `holders[from..to]` read every literal of
    /// it beyond the term, and [`Merge::derived`] of each such term, its
    /// holders put together in one pass over them.
    fn split_on(
        &mut self,
        at: usize,
        hash: u64,
        from: usize,
        to: usize,
        g: u32,
        weight: f64,
    ) -> bool {
        self.node += 1;
        let node = self.node;
        let depth = self.path.len() - at;
        let key = key(&self.groups, g);
        let members = self.members.len();
        self.tagged.clear();
        for &h in &self.holders[from..to] {
            let (start, end, _) = self.leaves[h as usize];
            let order = &self.order[start as usize..end as usize];
            // Where the term's literals come first in the holder's order,
            // as along a table's rows, the literal is the one after them.
            let l = match order.get(depth) {
                Some(&(k, l)) if k == key => l,
                _ => match order.binary_search_by_key(&key, |&(k, _)| k) {
                    Ok(i) => order[i].1,
                    Err(_) => continue,
                },
            };
            let seen = &mut self.lit_seen[code(l)];
            if seen.0 != node {
                *seen = (node, (self.members.len() - members) as u32);
                self.members.push(l);
            }
            self.tagged.push((seen.1, h));
        }
        let size = self.groups[g as usize].size as usize;
        let mut derived = self.members.len() - members == size;
        if derived {
            // The holders of each term with one literal more, together and
            // in the order they were in, and what they weigh.
            let below = self.holders.len();
            let ends = self.ends.len();
            self.ends.resize(ends + size, (0, 0.0));
            for &(i, h) in &self.tagged {
                let end = &mut self.ends[ends + i as usize];
                end.0 += 1;
                end.1 += self.leaves[h as usize].2;
            }
            let mut total = 0;
            for end in &mut self.ends[ends..] {
                (end.0, total) = (total, total + end.0);
            }
            self.holders.resize(below + self.tagged.len(), 0);
            for &(i, h) in &self.tagged {
                let place = &mut self.ends[ends + i as usize].0;
                self.holders[below + *place as usize] = h;
                *place += 1;
            }
            self.groups[g as usize].held = true;
            for i in 0..size {
                let start = if i == 0 {
                    below
                } else {
                    below + self.ends[ends + i - 1].0 as usize
                };
                let (end, sum) = self.ends[ends + i];
                let m = self.members[members + i];
                // The term with `m`, after the term on the path.
                let child = self.path.len();
                let place = at + self.path[at..].partition_point(|&l| l < m);
                self.path.extend_from_within(at..place);
                self.path.push(m);
                self.path.extend_from_within(place..child);
                derived = self.derived(
                    child,
                    hash ^ m.mixed(),
                    start,
                    below + end as usize,
                    sum,
                    weight / size as f64,
                );
                self.path.truncate(child);
                if !derived {
                    break;
                }
            }
            self.groups[g as usize].held = false;
            self.ends.truncate(ends);
            self.holders.truncate(below);
        }
        self.members.truncate(members);
        derived
    }
}

/// A literal's place in [`Merge::lit_seen`]: twice its atom, and 1 more if
/// it is positive.
fn code(l: GLit) -> usize {
    2 * l.atom() as usize + usize::from(l.is_positive())
}

/// A literal's measure in [`Merge::valid`]: a half for a fact's, one over
/// its size for an atom of an exhaustive set, 1 for any other.
fn weight(group_of: &impl Fn(GLit) -> Option<(usize, usize)>, l: GLit) -> f64 {
    group_of(l).map_or(1.0, |(_, size)| 1.0 / size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use GLit::{Neg, Pos};

    /// Atoms 0 to 2 are facts; 3 to 5 are one exhaustive set.
    fn group_of(l: GLit) -> Option<(usize, usize)> {
        match l.atom() {
            a @ 0..=2 => Some((a as usize, 2)),
            _ if l.is_positive() => Some((10, 3)),
            _ => None,
        }
    }

    fn valid_of(bodies: &[&[GLit]]) -> bool {
        let mut merge = Merge::default();
        for body in bodies {
            merge.push_body(body.iter().copied());
        }
        merge.valid(group_of)
    }

    #[test]
    fn bodies_are_valid_when_merging_reaches_the_empty_body() {
        assert!(valid_of(&[&[Pos(0)], &[Neg(0)]]));
        assert!(valid_of(&[&[Pos(0), Pos(1)], &[Pos(0), Neg(1)], &[Neg(0)]]));
        assert!(!valid_of(&[&[Pos(0), Pos(1)], &[Neg(0)]]));
        assert!(valid_of(&[
            &[Pos(3)],
            &[Pos(4), Pos(1)],
            &[Pos(4), Neg(1)],
            &[Pos(5)]
        ]));
        assert!(!valid_of(&[&[Pos(3)], &[Pos(4)]]));
        // Atom 1 is read most, so the first body's order splits on it
        // first, which fails; splitting on atom 0 derives the empty body.
        assert!(valid_of(&[
            &[Pos(0), Pos(1)],
            &[Pos(0), Neg(1)],
            &[Neg(0)],
            &[Neg(1), Pos(3)],
            &[Neg(1), Pos(4)]
        ]));
    }

    /// The search decides as merging every two terms that merge until no
    /// more do, from the bodies up: on random bodies over three facts, a
    /// set of three atoms and the atoms' negations, which are in no group.
    #[test]
    fn the_search_decides_as_merging_from_the_bodies_up() {
        use std::collections::BTreeSet;
        let groups: [&[GLit]; 4] = [
            &[Pos(0), Neg(0)],
            &[Pos(1), Neg(1)],
            &[Pos(2), Neg(2)],
            &[Pos(3), Pos(4), Pos(5)],
        ];
        let group = |l: GLit| groups.iter().position(|g| g.contains(&l));
        let merged = |bodies: &[Vec<GLit>]| -> bool {
            let mut terms: BTreeSet<Vec<GLit>> = bodies.iter().cloned().collect();
            loop {
                let mut more = Vec::new();
                for t in &terms {
                    for (k, &l) in t.iter().enumerate() {
                        let Some(g) = group(l) else { continue };
                        let rest: Vec<GLit> = t
                            .iter()
                            .enumerate()
                            .filter(|&(j, _)| j != k)
                            .map(|(_, &m)| m)
                            .collect();
                        let with = |m: GLit| {
                            let mut term = rest.clone();
                            term.push(m);
                            term.sort_unstable();
                            terms.contains(&term)
                        };
                        if rest.iter().all(|m| !groups[g].contains(m))
                            && groups[g].iter().all(|&m| with(m))
                            && !terms.contains(&rest)
                        {
                            more.push(rest);
                        }
                    }
                }
                if more.is_empty() {
                    return terms.contains(&Vec::new());
                }
                terms.extend(more);
            }
        };
        let literals = [
            Pos(0),
            Neg(0),
            Pos(1),
            Neg(1),
            Pos(2),
            Neg(2),
            Pos(3),
            Pos(4),
            Pos(5),
            Neg(3),
            Neg(4),
        ];
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = |m: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % m
        };
        let (mut valid, mut cases) = (0, 0);
        while cases < 20_000 {
            let bodies: Vec<Vec<GLit>> = (0..2 + next(12))
                .map(|_| {
                    let mut body: Vec<GLit> = (0..1 + next(4))
                        .map(|_| literals[next(literals.len() as u64) as usize])
                        .collect();
                    body.sort_unstable();
                    body.dedup();
                    body
                })
                .collect();
            let refs: Vec<&[GLit]> = bodies.iter().map(Vec::as_slice).collect();
            let measure: f64 = bodies
                .iter()
                .map(|b| b.iter().map(|&l| weight(&group_of, l)).product::<f64>())
                .sum();
            if measure < 1.0 {
                continue;
            }
            let expected = merged(&bodies);
            assert_eq!(valid_of(&refs), expected, "{bodies:?}");
            valid += usize::from(expected);
            cases += 1;
        }
        assert!(
            valid > 1_000 && valid < 19_000,
            "{valid} of the cases valid"
        );
    }

    /// A chain of 1100 switches: a body for each, true after those before
    /// it false, and one with all of them false. The deepest bodies weigh
    /// less than any normal number, and the search splits the terms they
    /// hold whatever their holders weigh.
    #[test]
    fn a_chain_too_long_to_weigh_is_valid() {
        let n = 1100;
        let mut merge = Merge::default();
        for i in 0..=n {
            merge.push_body((0..i).map(Neg).chain((i < n).then_some(Pos(i))));
        }
        assert!(merge.valid(|l| Some((l.atom() as usize, 2))));
    }

    #[test]
    fn a_repeated_literal_counts_once_in_the_measure() {
        // Counted twice, the first body would weigh 1/4 and the bodies 3/4.
        assert!(valid_of(&[&[Pos(0), Pos(0)], &[Neg(0)]]));
    }
}
