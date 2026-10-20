//! Separator copies for `td`'s fold where every rule reads facts alone.
//!
//! The fold ([`super::td`]) lays a bag's introduced facts on a spine above
//! a balanced join of its children's subtrees, halving the list of children.
//! Its *units* are a bag's subtree and each join of two or more of a bag's
//! children short of all of them; each join of the fold splits a unit into
//! two halves. An atom whose rules read a fact inside a unit and one outside
//! it *crosses* the unit, and the outside facts the crossing atoms read are
//! the unit's *separator*. The evidence walk conjoins each observation at
//! the lowest vtree node that holds its variables, above the unit, and the
//! evidence's level at the unit's root holds a node per function of the
//! separator that an assignment of the unit leaves. Those can be many more
//! than the separator's points, and the join of two halves holds about the
//! product of theirs.
//!
//! A copy of a separator fact is a fact of probability one half laid on a
//! spine atop the unit, read in the fact's place by the atoms that cross
//! into the unit and tied to the fact by two observed clauses
//! ([`crate::ground::add_fact_copies`], which also takes the copies' weight
//! back out of `P(e)`). The crossing atoms then lie inside the unit, and
//! its root level holds at most a node per point of the copies. The right
//! half's copy of a fact the left half copied too is tied to the left
//! half's copy, so that the walk finds the two made equal at their join
//! rather than above it. A unit inside a copied unit copies what it reads
//! from outside, the outer unit's copies among them.
//!
//! The copies cost spines of their own, and a unit whose functions are no
//! more than its separator's points gains nothing from them, while copying
//! a wide separator lets the unit's root level grow to its points: on the
//! programs measured the functions at a unit's root fell well short of what
//! the structure allows. Which units are copied is a [`CopyRule`], read off
//! the decomposition and the rules before anything is compiled. Only the
//! halves of a join with two facts or more on each side are candidates:
//! elsewhere the join multiplies its level by two nodes at most.

use std::cmp::Reverse;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::ground::{AtomId, FactCopy, GroundProgram};

use super::forest::halves;
use super::td::Rooted;

/// Which candidate units get copies of their separator facts: every one
/// with a separator, under the bounds given. `S` is a unit's separator,
/// `I` the facts its crossing atoms read inside it, `K` the distinct parts
/// of their rules outside it, `N` the facts laid in it. `min(|I|, K)`
/// bounds the logarithm of the functions of `S` at the unit's root from
/// above.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CopyRule {
    /// `min(|I|, K)` at least this many times `|S|`.
    ratio: f64,
    /// `N` at most this many times `|S|`.
    most: f64,
}

impl CopyRule {
    /// Every candidate unit with a separator, which the tests copy to
    /// exercise the copies on every unit they can be made for.
    #[cfg(test)]
    pub(crate) const ALL: CopyRule = CopyRule {
        ratio: 0.0,
        most: f64::INFINITY,
    };
    /// The rule `td` lays out by: `min(|I|, K)` at least twice `|S|`, and
    /// `N` at most four times `|S|`. On 193 weighted CNFs under `auto` it
    /// answered every instance answered without copies and two more, in the
    /// same time on those both answer; copying every candidate unit lost
    /// nine. The bound on `N` leaves out long units, whose functions at the
    /// root fell far below what `I` and `K` allow.
    pub(crate) const ON: CopyRule = CopyRule {
        ratio: 2.0,
        most: 4.0,
    };

    /// Whether a unit with separator `s` facts, `inside` facts read by its
    /// crossing atoms inside, `parts` distinct outside parts and `facts`
    /// facts gets copies.
    fn takes(&self, s: usize, inside: usize, parts: usize, facts: usize) -> bool {
        self.may_take(s, s, inside.min(parts), facts)
    }

    /// Whether a unit of `facts` facts may get copies where its separator
    /// has `s_least ..= s_most` facts and the smaller of `I` and `K` is at
    /// most `bound`: false where [`CopyRule::takes`] would refuse it
    /// whatever the counts.
    fn may_take(&self, s_least: usize, s_most: usize, bound: usize, facts: usize) -> bool {
        s_most > 0
            && bound as f64 >= self.ratio * s_least as f64
            && facts as f64 <= self.most * s_most as f64
    }
}

/// Atoms laid atop units of the fold ([`super::td::Rooted::lay_out`]): a
/// bag's subtree, or the join of some of a bag's children.
#[derive(Default)]
pub(super) struct Atop {
    bags: FxHashMap<usize, Vec<AtomId>>,
    /// By the bag and the range `lo..hi` of its children with facts.
    groups: FxHashMap<(usize, usize, usize), Vec<AtomId>>,
}

impl Atop {
    /// The atoms above bag `b`'s subtree, the first topmost.
    pub(super) fn bag(&self, b: usize) -> &[AtomId] {
        self.bags.get(&b).map_or(&[], Vec::as_slice)
    }

    /// The atoms above the join of bag `b`'s children with facts `lo..hi`,
    /// the first topmost.
    pub(super) fn group(&self, b: usize, lo: usize, hi: usize) -> &[AtomId] {
        self.groups.get(&(b, lo, hi)).map_or(&[], Vec::as_slice)
    }
}

/// The copies a rule makes on a rooted decomposition ([`plan`]).
pub(super) struct Plan {
    /// The copies, in the order [`crate::ground::add_fact_copies`] makes
    /// them: copy `i` is atom `n + i` of the program of `n` atoms.
    pub(super) copies: Vec<FactCopy>,
    pub(super) atop: Atop,
    /// The candidate units, those copied, and those whose side was read in
    /// full rather than ruled out by its bounds.
    pub(super) candidates: usize,
    pub(super) copied: usize,
    pub(super) read: usize,
}

/// A unit of the fold: a bag's subtree, or the join of bag `b`'s children
/// with facts `lo..hi`, two or more of them and not all.
#[derive(Clone, Copy, Debug)]
enum Unit {
    Bag(usize),
    Group(usize, usize, usize),
}

/// The units of the fold as a tree: a bag's subtree holds the two halves
/// of the join of its children with facts (or the one child), and a group
/// the two halves of its own range, as [`super::forest::Forest::balanced`]
/// splits them. Bag `b` is unit `b`.
struct Units {
    unit: Vec<Unit>,
    parent: Vec<usize>,
    /// The units in preorder, children left to right, and each unit's place
    /// in it and the units in its subtree, itself among them.
    preorder: Vec<usize>,
    pre: Vec<usize>,
    size: Vec<usize>,
    /// The facts the fold lays in each unit.
    facts: Vec<usize>,
    /// The two halves of each split, in the preorder of what they split.
    splits: Vec<(usize, usize)>,
}

impl Units {
    fn of(rooted: &Rooted) -> Units {
        let bags = rooted.parent.len();
        let kids: Vec<Vec<usize>> = rooted
            .children
            .iter()
            .map(|k| {
                k.iter()
                    .copied()
                    .filter(|&c| rooted.facts_under[c] > 0)
                    .collect()
            })
            .collect();
        let mut units = Units {
            unit: (0..bags).map(Unit::Bag).collect(),
            parent: vec![usize::MAX; bags],
            preorder: Vec::with_capacity(2 * bags),
            pre: vec![usize::MAX; bags],
            size: vec![0; bags],
            facts: rooted.facts_under.clone(),
            splits: Vec::new(),
        };
        enum Step {
            Enter(usize),
            Leave(usize),
        }
        let mut stack: Vec<Step> = rooted.roots.iter().rev().map(|&r| Step::Enter(r)).collect();
        while let Some(step) = stack.pop() {
            let u = match step {
                Step::Leave(u) => {
                    units.size[u] = units.preorder.len() - units.pre[u];
                    continue;
                }
                Step::Enter(u) => u,
            };
            units.pre[u] = units.preorder.len();
            units.preorder.push(u);
            stack.push(Step::Leave(u));
            let (b, lo, hi) = match units.unit[u] {
                Unit::Bag(b) => (b, 0, kids[b].len()),
                Unit::Group(b, lo, hi) => (b, lo, hi),
            };
            let below: Vec<usize> = if hi - lo == 1 {
                vec![kids[b][lo]]
            } else if hi - lo >= 2 {
                let mid = halves(lo, hi);
                let halves = [(lo, mid), (mid, hi)].map(|(lo, hi)| {
                    if hi - lo == 1 {
                        return kids[b][lo];
                    }
                    units.unit.push(Unit::Group(b, lo, hi));
                    units.parent.push(usize::MAX);
                    units.pre.push(usize::MAX);
                    units.size.push(0);
                    units
                        .facts
                        .push(kids[b][lo..hi].iter().map(|&c| rooted.facts_under[c]).sum());
                    units.unit.len() - 1
                });
                units.splits.push((halves[0], halves[1]));
                halves.to_vec()
            } else {
                Vec::new()
            };
            for &v in below.iter().rev() {
                units.parent[v] = u;
                stack.push(Step::Enter(v));
            }
        }
        units
    }

    /// Whether unit `x` is in unit `u`'s subtree.
    fn within(&self, x: usize, u: usize) -> bool {
        x != usize::MAX
            && self.pre[x] != usize::MAX
            && self.pre[u] <= self.pre[x]
            && self.pre[x] < self.pre[u] + self.size[u]
    }
}

/// What one unit shares with the rest, at the time it is planned.
#[derive(Default)]
struct Side {
    /// Each atom read outside the unit, with the crossing atoms that read
    /// it.
    outside: FxHashMap<AtomId, Vec<AtomId>>,
    /// The atoms the crossing atoms read inside the unit.
    inside: FxHashSet<AtomId>,
    /// The distinct literals over outside atoms of the crossing atoms'
    /// rules.
    parts: FxHashSet<Vec<(AtomId, bool)>>,
}

/// The separator copies `rule` makes on `rooted`, a decomposition of `gp`'s
/// interaction graph rooted for its fold, `position` the facts in the order
/// the compiler consumes them. The candidates are the two halves of each
/// join of the fold that holds two facts or more on either side: a bag's
/// subtree, or a join of some of a bag's children. They are planned in the
/// fold's preorder, the left half of a join before the right, so that a
/// unit sees what was copied above it and beside it. A unit's copies go
/// atop it in the order of the facts they copy, the one consumed last
/// topmost, as on a bag's spine; a right half's copy of what the left half
/// copied is tied to the left half's copy, which the walk then finds made
/// equal at their join.
pub(super) fn plan(
    gp: &GroundProgram,
    rooted: &Rooted,
    position: &[usize],
    rule: &CopyRule,
) -> Plan {
    let n = gp.num_atoms();
    let units = Units::of(rooted);
    let count = units.unit.len();
    let mut candidate = vec![false; count];
    for &(l, r) in &units.splits {
        if units.facts[l] >= 2 && units.facts[r] >= 2 {
            candidate[l] = true;
            candidate[r] = true;
        }
    }
    let candidates = candidate.iter().filter(|&&c| c).count();
    // The unit each fact is laid in, and each copy, after the facts.
    let mut at = vec![usize::MAX; n];
    for (b, facts) in rooted.introduced.iter().enumerate() {
        for &a in facts {
            at[a as usize] = b;
        }
    }
    // The nearest candidate at or above each unit.
    let mut up = vec![usize::MAX; count];
    for &u in &units.preorder {
        up[u] = if candidate[u] {
            u
        } else if units.parent[u] != usize::MAX {
            up[units.parent[u]]
        } else {
            usize::MAX
        };
    }

    // The atoms the walk conjoins, every derived atom but the copies of
    // facts the conditioning made, and the facts each reads, through those
    // copies. `reads[start[h]..start[h + 1]]` are atom `h`'s, each with the
    // atom it reads the fact through now.
    let copy_of = gp.copies_of_facts();
    let by_head = gp.rules_by_head();
    let resolve = |a: AtomId| copy_of[a as usize].map_or(a, |l| l.atom());
    let mut start = vec![0usize; n + 1];
    let mut reads: Vec<(AtomId, AtomId)> = Vec::new();
    let mut seen = vec![usize::MAX; n];
    for h in 0..n {
        start[h] = reads.len();
        if gp.is_fact(h as AtomId) || copy_of[h].is_some() {
            continue;
        }
        for &ri in &by_head[h] {
            for l in &gp.rules[ri].body {
                let f = resolve(l.atom());
                if gp.is_fact(f) && seen[f as usize] != h {
                    seen[f as usize] = h;
                    reads.push((f, f));
                }
            }
        }
    }
    start[n] = reads.len();
    // Each atom's reads by the preorder place of the unit they lie in,
    // ascending (none where a read lies in no unit), so that the reads
    // inside a unit, a range of the preorder, are counted by two searches.
    let place = |a: AtomId, at: &[usize]| match at[a as usize] {
        usize::MAX => usize::MAX,
        x => units.pre[x],
    };
    let mut places: Vec<usize> = reads.iter().map(|&(_, a)| place(a, &at)).collect();
    for h in 0..n {
        places[start[h]..start[h + 1]].sort_unstable();
    }
    // The candidates each atom crosses: those on the way up from a unit of
    // one of its facts that do not hold all of them.
    let mut crossing: Vec<Vec<AtomId>> = vec![Vec::new(); count];
    let mut mark = vec![usize::MAX; count];
    for h in 0..n {
        let facts = &reads[start[h]..start[h + 1]];
        if facts.len() < 2 {
            continue;
        }
        let stray = facts.iter().any(|&(f, _)| at[f as usize] == usize::MAX);
        let places = || {
            facts
                .iter()
                .filter_map(|&(f, _)| units.pre.get(at[f as usize]).copied())
        };
        let (lo, hi) = (places().min().unwrap_or(0), places().max().unwrap_or(0));
        for &(f, _) in facts {
            let mut y = match at[f as usize] {
                usize::MAX => usize::MAX,
                x => up[x],
            };
            while y != usize::MAX {
                if !stray && units.pre[y] <= lo && hi < units.pre[y] + units.size[y] {
                    break;
                }
                if mark[y] != h {
                    mark[y] = h;
                    crossing[y].push(h as AtomId);
                }
                y = match units.parent[y] {
                    usize::MAX => usize::MAX,
                    p => up[p],
                };
            }
        }
    }

    // Bounds on what [`Side`] counts for unit `u`, from the reads of each
    // crossing atom inside it: `|S|` is at least the most reads one atom
    // makes outside and at most their sum, `I` at most the reads inside, and
    // `K` at most the crossing atoms' rules. A program whose one atom reads
    // every fact crosses every unit, and its side is the program.
    let bounds = |u: usize, places: &[usize]| {
        let (lo, hi) = (units.pre[u], units.pre[u] + units.size[u]);
        let (mut s_least, mut s_most, mut inside, mut parts) = (0, 0, 0, 0);
        for &h in &crossing[u] {
            let p = &places[start[h as usize]..start[h as usize + 1]];
            let i = p.partition_point(|&x| x < hi) - p.partition_point(|&x| x < lo);
            if i == 0 || i == p.len() {
                continue;
            }
            s_least = s_least.max(p.len() - i);
            s_most += p.len() - i;
            inside += i;
            parts += by_head[h as usize].len();
        }
        (s_least, s_most, inside.min(parts))
    };
    let side = |u: usize, reads: &[(AtomId, AtomId)], at: &[usize]| {
        let mut side = Side::default();
        for &h in &crossing[u] {
            let entries = &reads[start[h as usize]..start[h as usize + 1]];
            let is_in = |a: AtomId| units.within(at[a as usize], u);
            if !entries.iter().any(|&(_, a)| is_in(a)) || entries.iter().all(|&(_, a)| is_in(a)) {
                continue;
            }
            // The atom each fact is read through now.
            let through: FxHashMap<AtomId, AtomId> = entries.iter().copied().collect();
            for &(_, a) in entries {
                if is_in(a) {
                    side.inside.insert(a);
                } else {
                    side.outside.entry(a).or_default().push(h);
                }
            }
            for &ri in &by_head[h as usize] {
                let mut part: Vec<(AtomId, bool)> = Vec::new();
                for l in &gp.rules[ri].body {
                    let (f, positive) = match copy_of[l.atom() as usize] {
                        Some(g) => (g.atom(), l.is_positive() == g.is_positive()),
                        None => (l.atom(), l.is_positive()),
                    };
                    if let Some(&a) = through.get(&f)
                        && !is_in(a)
                    {
                        part.push((a, positive));
                    }
                }
                if !part.is_empty() {
                    part.sort_unstable();
                    part.dedup();
                    side.parts.insert(part);
                }
            }
        }
        side
    };

    let mut copies: Vec<FactCopy> = Vec::new();
    // The fact each copy is of.
    let mut origin: Vec<AtomId> = Vec::new();
    let mut atop: Vec<Vec<AtomId>> = vec![Vec::new(); count];
    let (mut copied, mut read) = (0, 0);
    for &(l, r) in &units.splits {
        if !candidate[l] {
            continue;
        }
        // The left half's copies, by what they copy.
        let mut left: FxHashMap<AtomId, AtomId> = FxHashMap::default();
        for (k, u) in [l, r].into_iter().enumerate() {
            // The right half is planned after the left half's copies.
            let (s_least, s_most, bound) = bounds(u, &places);
            if !rule.may_take(s_least, s_most, bound, units.facts[u]) {
                continue;
            }
            read += 1;
            let s = side(u, &reads, &at);
            if !rule.takes(
                s.outside.len(),
                s.inside.len(),
                s.parts.len(),
                units.facts[u],
            ) {
                continue;
            }
            copied += 1;
            let original = |a: AtomId, origin: &[AtomId]| {
                if (a as usize) < n {
                    a
                } else {
                    origin[a as usize - n]
                }
            };
            let mut separator: Vec<(AtomId, Vec<AtomId>)> = s.outside.into_iter().collect();
            separator.sort_by_key(|&(a, _)| (Reverse(position[original(a, &origin) as usize]), a));
            let mut moved: Vec<AtomId> = Vec::new();
            for (a, readers) in separator {
                let new = (n + copies.len()) as AtomId;
                let equals = if k == 0 {
                    a
                } else {
                    left.get(&a).copied().unwrap_or(a)
                };
                if k == 0 {
                    left.insert(a, new);
                }
                for &h in &readers {
                    for e in &mut reads[start[h as usize]..start[h as usize + 1]] {
                        if e.1 == a {
                            e.1 = new;
                        }
                    }
                }
                moved.extend_from_slice(&readers);
                origin.push(original(a, &origin));
                at.push(u);
                atop[u].push(new);
                copies.push(FactCopy {
                    replaces: a,
                    equals,
                    readers,
                });
            }
            moved.sort_unstable();
            moved.dedup();
            for h in moved {
                let (from, to) = (start[h as usize], start[h as usize + 1]);
                for j in from..to {
                    places[j] = place(reads[j].1, &at);
                }
                places[from..to].sort_unstable();
            }
        }
    }
    let mut laid = Atop::default();
    for (u, atoms) in atop.into_iter().enumerate() {
        if atoms.is_empty() {
            continue;
        }
        match units.unit[u] {
            Unit::Bag(b) => laid.bags.insert(b, atoms),
            Unit::Group(b, lo, hi) => laid.groups.insert((b, lo, hi), atoms),
        };
    }
    Plan {
        copies,
        atop: laid,
        candidates,
        copied,
        read,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bounds_decide_a_unit() {
        // No separator, nothing to copy; under `all`, any other unit.
        assert!(!CopyRule::ALL.takes(0, 9, 9, 1));
        assert!(CopyRule::ALL.takes(30, 1, 1, 1000));
        // `on`: the smaller of I and K at least twice S, N at most four
        // times S.
        let on = CopyRule::ON;
        assert!(on.takes(5, 10, 12, 20));
        assert!(!on.takes(5, 9, 12, 20));
        assert!(!on.takes(5, 12, 9, 20));
        assert!(!on.takes(5, 10, 12, 21));
    }
}

#[cfg(test)]
#[path = "tests/copies.rs"]
mod copies;
