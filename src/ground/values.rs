//! Variables written over bits, written as tables over their values
//! ([`bits_as_values`]).
//!
//! A front end that holds a variable of several values as bits writes
//! each bit as rules, and its readers test the variable's values as cubes
//! over the bits. Dice draws a `discrete` bit by bit, a fact per ratio,
//! under a chain of conditionals on its parents' values, and a comparison
//! `x == k` is a conjunction of `x`'s bits; a network written with its
//! values as codes sets each bit from a chain of switches per row and
//! reads its parents' codes as cubes. The program then names no value:
//! the exclusive sets, the row classes, the cover and the families layout,
//! which read a network's values, find none, and the compile carries the
//! bits, the comparisons and the conditionals' guards as atoms of their
//! own. This pass finds such variables on the ground program, whatever
//! front end wrote it, and writes each as the network reader's tables are
//! ground: an atom per value, and each row of the table a chain of
//! switches over the values the row gives mass to, under the parents'
//! values. Each bit is then the disjunction of the values that set it, and
//! a reader's cube over a variable's bits the value it spells.
//!
//! The variables. A variable's bits are atoms that no query or observation
//! names and that every rule reading one of them reads all of, outside the
//! variable's own rules: a comparison or a row's context reads all of a
//! variable's bits. They are found from their readers: the atoms that
//! every rule reading an atom also reads, where at least two rules read it
//! and read them as two codes or more (each such set a *candidate*, the
//! largest first, none sharing an atom). A fact that only those rules read
//! is left out of the set: it owns nothing that would join it to the
//! others, and is a variable of its own (two flips that one comparison
//! reads together).
//! Each candidate, and each other atom with rules, *owns* the atoms that
//! only its own rules read, through their rules, from the readers down:
//! the private gates and the facts of its draw (its flips, its rows'
//! switches). A candidate's bits that the rules of what it owns do not
//! connect are separate variables (two parents that a child reads
//! together). A variable of two bits or more is written where it owns a
//! fact, so that its values are a draw of its own and not a function of
//! its inputs, and its readers test two of its values or more. An atom
//! that is not a candidate's and owns a fact is a variable of one bit,
//! written where its rules read a variable written before, so that its
//! rows read that variable's values.
//!
//! The rows. A variable's cone is what it owns and the views it reads:
//! the atoms that read only bits of variables, through such views. Its
//! inputs are what the cone reads besides: the facts it does not own and
//! the bits of other variables. The cone's rules are expanded over the
//! inputs as Shannon would: under each value of the first input that a
//! rule still reads, the rules that read it the other way are dropped,
//! the atoms left without rules are false and those with an empty body
//! true, and the rules the bits no longer read are dropped, until no rule
//! reads an input. Each such leaf is a row, its context the inputs fixed
//! on the way, and the same expansion over the variable's facts, weighted,
//! gives the row's distribution over the codes. A variable written before
//! is read through its values: the context's literals on its bits are the
//! value they spell, or a row per value where they leave several.
//!
//! A variable's draw is its own where each fact its cone reads that is no
//! variable's bit is read only by atoms that variables hold. Where an atom
//! no variable holds also reads it, the variable is a fragment of one the
//! pass did not find whole (the other bits of a `discrete` that a nested
//! branch reads apart from it, say), its rows would be over the draw of
//! the bits left out, and it is left as written.
//!
//! A variable one of whose atoms reads one of its bits both ways draws
//! its lower bits under the values of the higher ones, as a trie (the
//! index Alea.jl draws for a decision list). The trie's flips are the
//! guards that the `decision` layout conditions such circuits on
//! ([`super::guards`]), and written as values the circuits took up to
//! five times the work (Alea.jl's coin bias): such a variable is left as
//! written, with its group.
//!
//! The groups. The variables whose rules read each other's bits, directly
//! or through views, form a group, and a group is written whole or not at
//! all: where one of its variables is left as written, the rows written
//! for the others are taken back. A table over a variable's values whose
//! parents or children stay as bits holds the bits and the values both,
//! and costs the compile more than either writing alone.
//!
//! The bounds. The rows written hold at most [`GROWTH`] times the literals
//! of the cone's rules, and the expansion takes at most
//! [`WORK_PER_LITERAL`] steps per literal of the cone and [`WORK_FLOOR`]
//! more. A row's sum over the facts takes at most [`WORK_PER_LITERAL`]
//! steps per literal the row may write and [`WORK_FLOOR`] more: a row is a
//! draw over the variable's values, and a sum that takes longer infers its
//! distribution from a computation over the facts (an adder's sum over
//! thousands of flips), which the compile does better. The expansion stops
//! as soon as a bound is passed, and the variable is left as written; a
//! variable with no input is one row, and its rules are not expanded where
//! they alone pass that row's steps. The pass as a whole takes at most
//! [`WORK_PER_LITERAL`] steps per literal of the program and
//! [`WORK_FLOOR`] more, and writes no more groups once they are taken. A
//! variable that reads, through views, an atom that depends on it is left
//! as written: its rows would read their own values. The rules outside
//! the variables written that read a cube over a written variable's bits
//! that spells one value read that value instead.

use std::sync::OnceLock;

use rustc_hash::{FxHashMap, FxHashSet};

use super::program::Lists;
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, Prob, deps};
use crate::disjoint::{find, union};
use crate::syntax::{Atom, Term};

/// The rows a variable is written as hold at most this many times the
/// literals of its cone's rules.
const GROWTH: usize = 4;
/// Steps of a variable's expansion per literal of its cone's rules, and of
/// the whole pass per literal of the program: literals copied or read.
const WORK_PER_LITERAL: usize = 32;
/// Steps an expansion, and the pass, may take more, however few the
/// literals.
const WORK_FLOOR: usize = 1 << 16;
/// Bits a variable has at most.
const BITS: usize = 16;

/// No atom, unit or owner.
const NONE: u32 = u32::MAX;

/// The steps taken and the most the pass may take.
struct Work {
    done: usize,
    limit: usize,
}

impl Work {
    fn add(&mut self, steps: usize) -> bool {
        self.done += steps;
        self.done <= self.limit
    }

    fn left(&self) -> usize {
        self.limit.saturating_sub(self.done)
    }
}

/// The steps a row's sum may take: [`WORK_PER_LITERAL`] per literal the
/// row writes at most, a value of `bits` bits under a context of `context`
/// inputs each, and [`WORK_FLOOR`] more.
fn row_work(bits: usize, context: usize) -> usize {
    WORK_PER_LITERAL * ((context + 1) << bits) + WORK_FLOOR
}

/// The variables of `gp` written over bits written as tables over their
/// values, as the module says; how many were.
pub(super) fn bits_as_values(gp: &mut GroundProgram) -> usize {
    let n = gp.num_atoms();
    let literals: usize = gp.rules.iter().map(|r| r.body.len() + 1).sum();
    let mut work = Work {
        done: 0,
        limit: WORK_PER_LITERAL * literals + WORK_FLOOR,
    };
    let mut named = vec![false; n];
    for a in gp.named_atoms() {
        named[a as usize] = true;
    }
    let readers = gp.readers(|ri, _| ri);
    // The atoms that may be bits, where `gate` holds of the gates.
    let bits = |gate: &[bool]| -> Vec<bool> {
        (0..n)
            .map(|a| {
                !named[a]
                    && match gp.kinds[a] {
                        AtomKind::Fact(p) => normal(p),
                        AtomKind::Derived => gate[a],
                    }
            })
            .collect()
    };
    // Where no atom is read with others with every atom that has rules
    // taken for a gate, none is with the acyclic ones alone, which then
    // need not be found.
    let mut with_rules = vec![false; n];
    for r in &gp.rules {
        with_rules[r.head as usize] = true;
    }
    let mut any = Work {
        done: 0,
        limit: work.limit,
    };
    if co_read(
        gp,
        &readers,
        &bits(&with_rules),
        &with_rules,
        true,
        &mut any,
    )
    .is_some_and(|c| c.is_empty())
    {
        return 0;
    }
    let by_head = gp.rules_by_head();
    let sccs = deps::derived_sccs(gp, &by_head);
    // The gates, the acyclic atoms with rules, each with its place in
    // dependency order.
    let mut rank = vec![NONE; n];
    let mut gates = 0;
    for scc in &sccs {
        if let [a] = scc[..]
            && !by_head[a as usize].is_empty()
            && !deps::is_cyclic(gp, &by_head, scc)
        {
            rank[a as usize] = gates;
            gates += 1;
        }
    }
    if gates == 0 {
        return 0;
    }
    let candidates = {
        let gate: Vec<bool> = rank.iter().map(|&r| r != NONE).collect();
        match co_read(gp, &readers, &bits(&gate), &gate, false, &mut work) {
            Some(c) if !c.is_empty() => c,
            _ => return 0,
        }
    };
    let Some(units) = Units::new(
        (gp, &by_head, &readers),
        &sccs,
        rank,
        &named,
        candidates,
        &mut work,
    ) else {
        return 0;
    };
    let order = units.order();
    work.limit = work.done + GROWTH * WORK_PER_LITERAL * literals + WORK_FLOOR * units.bits.len();

    let mut written = Written::new(n);
    let mut cone = Cone::new(n);
    'groups: for group in units.groups(&order) {
        let mark = written.mark();
        for u in group {
            let i = &units.bits[&u];
            let members = units.members.get(u).expect("a unit has members");
            if i.len() > 1 && units.decides(members, i) {
                written.undo(mark);
                continue 'groups;
            }
            let mut unit = Unit {
                gp,
                units: &units,
                readers: &readers,
                cone: &mut cone,
                written: &mut written,
                work: &mut work,
            };
            let done = unit.write_whole(members, i);
            if work.left() == 0 {
                written.undo(mark);
                break 'groups;
            }
            if !done {
                written.undo(mark);
                continue 'groups;
            }
        }
    }
    written.finish(gp)
}

/// What writing a unit takes.
struct Unit<'a> {
    gp: &'a GroundProgram,
    units: &'a Units<'a>,
    readers: &'a Lists<u32>,
    cone: &'a mut Cone,
    written: &'a mut Written,
    work: &'a mut Work,
}

impl Unit<'_> {
    /// Writes the unit of `members` and `interface`, a variable of one bit
    /// only where it reads one written, through views; false where it is
    /// a fragment or its rows pass their bounds.
    fn write(&mut self, members: &[AtomId], interface: &[AtomId]) -> bool {
        let Unit {
            gp,
            units,
            readers,
            cone,
            written,
            work,
        } = self;
        let one = interface.len() == 1;
        if one && (written.values.is_empty() || !units.reads(members, &written.bit_of)) {
            return true;
        }
        if !cone.of(units, members, interface) || !work.add(cone.literals) {
            return false;
        }
        if one && !cone.inputs.iter().any(|x| written.bit_of.contains_key(x)) {
            return true;
        }
        if !units.draws_alone(readers, &cone.inputs) {
            return false;
        }
        let limit = (WORK_PER_LITERAL * cone.literals + WORK_FLOOR).min(work.left());
        let Some(leaves) = cone.rows(units, limit, work) else {
            return false;
        };
        let bound = GROWTH * cone.literals;
        written.write(gp, members, interface, &cone.inputs, &leaves, bound)
    }

    /// Writes the unit of `members` and `interface`, summing what it holds
    /// into its rows or, where those pass their bounds, with its private
    /// variables apart from it, each read through its values; whether it
    /// did.
    fn write_whole(&mut self, members: &[AtomId], interface: &[AtomId]) -> bool {
        if self.write(members, interface) {
            return true;
        }
        let parts = self
            .units
            .private(self.readers, members, interface, self.work);
        if parts.is_empty() {
            return false;
        }
        let mut apart: FxHashSet<AtomId> = FxHashSet::default();
        for part in &parts {
            if !self.write_whole(part, &part[..1]) {
                return false;
            }
            apart.extend(part);
        }
        let rest: Vec<AtomId> = members
            .iter()
            .copied()
            .filter(|a| !apart.contains(a))
            .collect();
        self.write(&rest, interface)
    }
}

/// The candidates of `gp`: for each atom of `bit` and `start` that two
/// rules or more read, the atoms of `bit` that every rule reading it reads,
/// with it, but the facts that only those rules read, where two atoms or
/// more are left and those rules read them as two codes or more; the
/// largest first, each kept where it shares no atom with one kept before;
/// with `any`, the first found alone. `None` where the steps run out.
fn co_read(
    gp: &GroundProgram,
    readers: &Lists<u32>,
    bit: &[bool],
    start: &[bool],
    any: bool,
    work: &mut Work,
) -> Option<Vec<Vec<AtomId>>> {
    // The sets found, one after the other, each by where it starts and its
    // length.
    let mut atoms: Vec<AtomId> = Vec::new();
    let mut found: Vec<(usize, usize)> = Vec::new();
    let mut set: Vec<AtomId> = Vec::new();
    let mut signs = vec![0u8; gp.num_atoms()];
    let distinct = |rs: &[u32]| rs.len() - rs.windows(2).filter(|w| w[0] == w[1]).count();
    let mut rs: Vec<u32> = Vec::new();
    for a in 0..gp.num_atoms() {
        if !bit[a] || !start[a] || readers[a].len() < 2 {
            continue;
        }
        // The rules reading `a` but those of atoms read with it: a trie's
        // lower bits, which read its higher ones.
        rs.clear();
        for &r in &readers[a] {
            let h = gp.rules[r as usize].head as usize;
            if !any && bit[h] {
                if !work.add(readers[h].len() + 1) {
                    return None;
                }
                if readers[h].iter().any(|r| readers[a].binary_search(r).is_ok()) {
                    continue;
                }
            }
            rs.push(r);
        }
        if rs.len() < 2 || rs.iter().all(|&r| r == rs[0]) {
            continue;
        }
        // From the shortest body that reads `a`, what every other reads.
        let first = *rs
            .iter()
            .min_by_key(|&&r| gp.rules[r as usize].body.len())
            .expect("readers");
        let body = &gp.rules[first as usize].body;
        if !work.add(rs.len() + body.len()) {
            return None;
        }
        set.clear();
        set.extend(
            body.iter()
                .map(|l| l.atom())
                .filter(|&x| x as usize != a && bit[x as usize]),
        );
        set.sort_unstable();
        set.dedup();
        for (k, &r) in rs.iter().enumerate() {
            if set.is_empty() {
                break;
            }
            if r == first || (k > 0 && rs[k - 1] == r) {
                continue;
            }
            if !work.add(set.len()) {
                return None;
            }
            set.retain(|&x| readers[x as usize].binary_search(&r).is_ok());
        }
        // A fact that only these rules read owns nothing that would join
        // it to the others: it is a variable of its own.
        let n = distinct(&rs);
        set.retain(|&x| {
            !matches!(gp.kinds[x as usize], AtomKind::Fact(_))
                || distinct(&readers[x as usize]) != n
        });
        if set.is_empty() {
            continue;
        }
        set.push(a as AtomId);
        set.sort_unstable();
        // The codes these rules read: some atom read both ways.
        if !work.add(rs.len() + set.len()) {
            return None;
        }
        for &x in &set {
            signs[x as usize] = 0;
        }
        for (k, &r) in rs.iter().enumerate() {
            if k > 0 && rs[k - 1] == r {
                continue;
            }
            let body = &gp.rules[r as usize].body;
            if !work.add(body.len()) {
                return None;
            }
            for l in body {
                let x = l.atom() as usize;
                if set.binary_search(&l.atom()).is_ok() {
                    signs[x] |= 1 << u8::from(l.is_positive());
                }
            }
        }
        if !set.iter().any(|&x| signs[x as usize] == 3) {
            continue;
        }
        if any {
            return Some(vec![set.clone()]);
        }
        found.push((atoms.len(), set.len()));
        atoms.extend_from_slice(&set);
    }
    found.sort_by_key(|&(_, len)| std::cmp::Reverse(len));
    let mut taken = vec![false; gp.num_atoms()];
    let mut kept = Vec::new();
    for (at, len) in found {
        let c = &atoms[at..at + len];
        if c.iter().any(|&x| taken[x as usize]) {
            continue;
        }
        for &x in c {
            taken[x as usize] = true;
        }
        kept.push(c.to_vec());
    }
    Some(kept)
}

/// The units of a program as the module says: the variables, of one bit
/// or more, each with what it owns, by the root of its class.
struct Units<'a> {
    gp: &'a GroundProgram,
    by_head: &'a Lists<usize>,
    rank: Vec<u32>,
    /// Per atom, its class: what one owner holds that its rules connect,
    /// by a root atom of it.
    class: Vec<u32>,
    /// The bits of each unit, by its class.
    bits: FxHashMap<u32, Vec<AtomId>>,
    /// Per atom, whether it is a unit's bit.
    is_bit: Vec<bool>,
    members: Members,
    /// The atoms that read only units' bits, through such atoms.
    view: Vec<bool>,
}

impl<'a> Units<'a> {
    /// The units of `gp` from its `candidates`; `None` where no variable of
    /// two bits or more is, or the steps run out. A candidate that is no
    /// variable owns nothing: where some are, the classes are found again
    /// from the variables alone.
    fn new(
        (gp, by_head, readers): (&'a GroundProgram, &'a Lists<usize>, &Lists<u32>),
        sccs: &[Vec<AtomId>],
        rank: Vec<u32>,
        named: &[bool],
        candidates: Vec<Vec<AtomId>>,
        work: &mut Work,
    ) -> Option<Self> {
        let n = gp.num_atoms();
        let mut candidates = candidates;
        candidates.sort_unstable_by_key(|c| c[0]);
        let mut reading = Reading::new(gp, by_head, readers, &rank, named);
        // The atoms of the variables not read as values, none of which is
        // a variable of one bit.
        let mut declined = vec![false; n];
        loop {
            if !work.add(4 * (n + gp.rules.len())) {
                return None;
            }
            let (owner, class) = classes(gp, readers, sccs, &rank, named, &candidates);
            // The units' bits: a candidate's, by class, and each other
            // gate that owns itself, each class's a run of `bits_of` from
            // `start`. A unit owns a fact of its draw, and has one bit, or
            // is read as values.
            let mut is_candidate = vec![false; n];
            let mut start = vec![0usize; n + 1];
            for c in &candidates {
                for &a in c {
                    is_candidate[a as usize] = true;
                    start[class[a as usize] as usize + 1] += 1;
                }
            }
            let alone = |a: usize| {
                owner[a] == a as u32 && rank[a] != NONE && !is_candidate[a] && !declined[a]
            };
            for a in 0..n {
                if alone(a) {
                    start[class[a] as usize + 1] += 1;
                }
            }
            for c in 0..n {
                start[c + 1] += start[c];
            }
            let mut bits_of = vec![0; start[n]];
            let mut next = start.clone();
            let mut place = |a: AtomId| {
                let c = class[a as usize] as usize;
                bits_of[next[c]] = a;
                next[c] += 1;
            };
            for c in &candidates {
                for &a in c {
                    place(a);
                }
            }
            for a in 0..n {
                if alone(a) {
                    place(a as AtomId);
                }
            }
            let of = |c: usize| &bits_of[start[c]..start[c + 1]];
            let mut random = vec![false; n];
            for (&c, kind) in class.iter().zip(&gp.kinds) {
                let c = c as usize;
                if !of(c).is_empty() && matches!(kind, AtomKind::Fact(_)) {
                    random[c] = true;
                }
            }
            // The candidates whose classes own a fact, two bits or more
            // each: the others are found again without those that draw
            // nothing (a comparison and a guard read together) before
            // what reads them is read.
            let mut drawn: Vec<Vec<AtomId>> = (0..n)
                .filter(|&c| random[c] && of(c).len() > 1)
                .map(|c| of(c).to_vec())
                .collect();
            if drawn.is_empty() {
                return None;
            }
            drawn.sort_unstable_by_key(|c| c[0]);
            if drawn != candidates {
                candidates = drawn;
                continue;
            }
            let mut kept = vec![false; n];
            let mut variables: Vec<Vec<AtomId>> = Vec::new();
            for c in 0..n {
                let i = of(c);
                kept[c] = random[c]
                    && match i.len() {
                        0 => false,
                        1 => !is_candidate[i[0] as usize],
                        k => {
                            k <= BITS && reading.as_values(&class, &is_candidate, c as u32, i, work)
                        }
                    };
                if kept[c] && i.len() > 1 {
                    variables.push(i.to_vec());
                } else if random[c] && i.len() > 1 {
                    for &a in i {
                        declined[a as usize] = true;
                    }
                }
            }
            if variables.is_empty() || work.left() == 0 || reading.left == 0 {
                return None;
            }
            variables.sort_unstable_by_key(|c| c[0]);
            if variables != candidates {
                candidates = variables;
                continue;
            }
            let bits: FxHashMap<u32, Vec<AtomId>> = (0..n)
                .filter(|&c| kept[c])
                .map(|c| (c as u32, of(c).to_vec()))
                .collect();
            let mut pairs: Vec<(u32, AtomId)> = (0..n as AtomId)
                .map(|a| (class[a as usize], a))
                .filter(|&(c, _)| kept[c as usize])
                .collect();
            pairs.sort_unstable();
            let members = Members::new(pairs);
            // The views, in dependency order, over the units' bits and the
            // facts that no one atom owns: the flips of variables of one
            // bit that several read, as their values.
            let free = |y: usize| {
                owner[y] == y as u32
                    && !named[y]
                    && matches!(gp.kinds[y], AtomKind::Fact(p) if normal(p))
            };
            let mut is_bit = vec![false; n];
            for i in bits.values() {
                for &a in i {
                    is_bit[a as usize] = true;
                }
            }
            let mut gates: Vec<AtomId> = (0..n as AtomId)
                .filter(|&a| rank[a as usize] != NONE)
                .collect();
            gates.sort_unstable_by_key(|&a| rank[a as usize]);
            let mut view = vec![false; n];
            for a in gates {
                view[a as usize] = !is_bit[a as usize]
                    && by_head[a as usize].iter().all(|&ri| {
                        gp.rules[ri].body.iter().all(|l| {
                            let y = l.atom() as usize;
                            is_bit[y] || view[y] || free(y)
                        })
                    });
            }
            return Some(Units {
                gp,
                by_head,
                rank,
                class,
                bits,
                is_bit,
                members,
                view,
            });
        }
    }

    fn is_gate(&self, a: AtomId) -> bool {
        self.rank[a as usize] != NONE
    }

    /// Whether the rules of `members` read a bit of `written`, directly or
    /// through views.
    fn reads(&self, members: &[AtomId], written: &FxHashMap<AtomId, (usize, u32)>) -> bool {
        members.iter().any(|&a| {
            self.by_head[a as usize].iter().any(|&ri| {
                self.gp.rules[ri].body.iter().any(|l| {
                    let y = l.atom();
                    written.contains_key(&y) || self.view[y as usize]
                })
            })
        })
    }

    /// The units of `order` by group, each group's in their order there and
    /// the groups in the order of their first: a unit and the units whose
    /// bits its rules read, directly or through views, are of one group.
    fn groups(&self, order: &[u32]) -> Vec<Vec<u32>> {
        let gp = self.gp;
        let n = gp.num_atoms();
        let mut parent: Vec<usize> = (0..n).collect();
        // A unit by its class, a view by itself.
        let node = |a: usize| -> Option<usize> {
            if self.view[a] {
                Some(a)
            } else if self.is_bit[a] || self.bits.contains_key(&self.class[a]) {
                Some(self.class[a] as usize)
            } else {
                None
            }
        };
        for a in 0..n {
            let Some(from) = node(a) else {
                continue;
            };
            for &ri in &self.by_head[a] {
                for l in &gp.rules[ri].body {
                    let y = l.atom() as usize;
                    if self.is_bit[y] || self.view[y] {
                        let to = node(y).expect("a bit or a view");
                        union(&mut parent, from, to);
                    }
                }
            }
        }
        let mut at: FxHashMap<usize, usize> = FxHashMap::default();
        let mut groups: Vec<Vec<u32>> = Vec::new();
        for &u in order {
            let k = *at.entry(find(&mut parent, u as usize)).or_insert_with(|| {
                groups.push(Vec::new());
                groups.len() - 1
            });
            groups[k].push(u);
        }
        groups
    }

    /// Whether an atom of `members` chooses by a bit of `bits`, its rules
    /// reading the bit both ways.
    fn decides(&self, members: &[AtomId], bits: &[AtomId]) -> bool {
        let mut read: Vec<GLit> = Vec::new();
        members.iter().any(|&a| {
            read.clear();
            self.by_head[a as usize].iter().any(|&ri| {
                self.gp.rules[ri].body.iter().any(|&l| {
                    if !bits.contains(&l.atom()) {
                        return false;
                    }
                    if read.iter().any(|&m| m.atom() == l.atom() && m != l) {
                        return true;
                    }
                    read.push(l);
                    false
                })
            })
        })
    }

    /// The variables of one bit that the unit of `members` and `interface`
    /// sums into its rows: each gate of it, the readers first, that is
    /// not of `interface`, with the atoms of its class under it but the
    /// views (the gate first), where only they read those atoms, they hold
    /// a fact and they read a view or an atom outside the class. A gate
    /// that reads neither is a flip, which adds no row.
    fn private(
        &self,
        readers: &Lists<u32>,
        members: &[AtomId],
        interface: &[AtomId],
        work: &mut Work,
    ) -> Vec<Vec<AtomId>> {
        let gp = self.gp;
        let mut gates: Vec<AtomId> = members
            .iter()
            .copied()
            .filter(|&a| self.is_gate(a) && !self.view[a as usize] && !interface.contains(&a))
            .collect();
        gates.sort_unstable_by_key(|&a| std::cmp::Reverse(self.rank[a as usize]));
        let mut taken: FxHashSet<AtomId> = FxHashSet::default();
        let mut parts = Vec::new();
        for g in gates {
            if taken.contains(&g) {
                continue;
            }
            let mut part = vec![g];
            let mut seen: FxHashSet<AtomId> = FxHashSet::from_iter([g]);
            let mut outside = false;
            let mut steps = 0;
            let mut next = 0;
            while next < part.len() {
                for &ri in &self.by_head[part[next] as usize] {
                    let body = &gp.rules[ri].body;
                    steps += body.len();
                    for l in body {
                        let y = l.atom();
                        if self.class[y as usize] != self.class[g as usize] || self.view[y as usize] {
                            outside = true;
                        } else if seen.insert(y) {
                            part.push(y);
                        }
                    }
                }
                next += 1;
            }
            let private = outside
                && part[1..].iter().all(|&a| {
                    steps += readers[a as usize].len();
                    !interface.contains(&a)
                        && readers[a as usize]
                            .iter()
                            .all(|&ri| seen.contains(&gp.rules[ri as usize].head))
                })
                && part
                    .iter()
                    .any(|&a| matches!(gp.kinds[a as usize], AtomKind::Fact(_)));
            if !work.add(steps) {
                return Vec::new();
            }
            if private {
                taken.extend(&part);
                parts.push(part);
            }
        }
        parts
    }

    /// Whether the cone's draw is its own: each fact it reads that is no
    /// unit's bit is read only by atoms that units hold. A fact that an
    /// atom no unit holds also reads is a draw the unit shares with a bit
    /// left out of it.
    fn draws_alone(&self, readers: &Lists<u32>, inputs: &[AtomId]) -> bool {
        inputs.iter().all(|&x| {
            self.is_bit[x as usize]
                || !matches!(self.gp.kinds[x as usize], AtomKind::Fact(_))
                || readers[x as usize].iter().all(|&ri| {
                    let h = self.gp.rules[ri as usize].head as usize;
                    self.view[h] || self.bits.contains_key(&self.class[h])
                })
        })
    }

    /// The units in an order where each comes after the units it reads,
    /// through views, without those that read, through views, an atom
    /// that depends on them.
    fn order(&self) -> Vec<u32> {
        let gp = self.gp;
        let n = gp.num_atoms();
        let class = &self.class;
        let class_rules = Lists::new(n, || {
            gp.rules
                .iter()
                .enumerate()
                .map(|(i, r)| (class[r.head as usize] as usize, i))
        });
        let next = |v: usize, (rule, lit): &mut (usize, usize)| -> Option<usize> {
            let rules = &class_rules[v];
            while *rule < rules.len() {
                let body = &gp.rules[rules[*rule]].body;
                while *lit < body.len() {
                    let w = class[body[*lit].atom() as usize] as usize;
                    *lit += 1;
                    if w != v {
                        return Some(w);
                    }
                }
                *rule += 1;
                *lit = 0;
            }
            None
        };
        let mut order = Vec::new();
        deps::tarjan(
            n,
            (0..n).filter(|&a| class[a] as usize == a),
            next,
            |comp| {
                if let [u] = comp
                    && self.bits.contains_key(&(*u as u32))
                {
                    order.push(*u as u32);
                }
            },
        );
        order
    }
}

/// Each atom's class, as the module says: each atom's owner, from the
/// readers down (a candidate's atoms its first atom, an atom whose
/// readers' heads all have one owner that owner, any other atom itself),
/// and the atoms of one owner that its rules connect, by the least; each
/// atom's owner and its class.
fn classes(
    gp: &GroundProgram,
    readers: &Lists<u32>,
    sccs: &[Vec<AtomId>],
    rank: &[u32],
    named: &[bool],
    candidates: &[Vec<AtomId>],
) -> (Vec<u32>, Vec<u32>) {
    let n = gp.num_atoms();
    let mut owner = vec![NONE; n];
    for c in candidates {
        for &a in c {
            owner[a as usize] = c[0];
        }
    }
    let own = |a: AtomId, owner: &mut [u32]| {
        if owner[a as usize] != NONE {
            return;
        }
        let held = !named[a as usize]
            && match gp.kinds[a as usize] {
                AtomKind::Fact(p) => normal(p),
                AtomKind::Derived => rank[a as usize] != NONE,
            };
        let mut o = NONE;
        if held {
            for &ri in &readers[a as usize] {
                let h = owner[gp.rules[ri as usize].head as usize];
                if o == NONE {
                    o = h;
                } else if o != h {
                    o = NONE;
                    break;
                }
            }
        }
        owner[a as usize] = if o == NONE { a } else { o };
    };
    for scc in sccs.iter().rev() {
        for &a in scc {
            own(a, &mut owner);
        }
    }
    for a in 0..n as AtomId {
        own(a, &mut owner);
    }
    let mut parent: Vec<usize> = (0..n).collect();
    for r in &gp.rules {
        let o = owner[r.head as usize];
        for l in &r.body {
            let y = l.atom();
            if owner[y as usize] == o {
                union(&mut parent, r.head as usize, y as usize);
            }
        }
    }
    let class = (0..n).map(|a| find(&mut parent, a) as u32).collect();
    (owner, class)
}

/// What reads a variable's bits ([`Reading::as_values`]), with scratch
/// kept from one variable to the next: by its token, the variable's bits,
/// the rules read, and what each atom found depends on.
struct Reading<'a> {
    gp: &'a GroundProgram,
    by_head: &'a Lists<usize>,
    readers: &'a Lists<u32>,
    rank: &'a [u32],
    named: &'a [bool],
    /// The steps the walks through gates may still take, over all the
    /// variables: one per literal of the program, and [`WORK_FLOOR`] more.
    left: usize,
    /// The lowest place in dependency order of an atom that may read the
    /// current variable.
    floor: u32,
    token: u32,
    in_set: Vec<u32>,
    rule_seen: Vec<u32>,
    found: Vec<u32>,
    entered: Vec<u32>,
    /// Per atom found, the variable's bits it depends on, as a mask, and
    /// [`ALONE`] where it reads nothing else.
    depends: Vec<u32>,
}

/// In [`Reading::depends`], of an atom that reads, through gates, the
/// variable's bits and nothing else.
const ALONE: u32 = 1 << 31;

impl<'a> Reading<'a> {
    fn new(
        gp: &'a GroundProgram,
        by_head: &'a Lists<usize>,
        readers: &'a Lists<u32>,
        rank: &'a [u32],
        named: &'a [bool],
    ) -> Self {
        let n = gp.num_atoms();
        Reading {
            gp,
            by_head,
            readers,
            rank,
            named,
            left: gp.rules.iter().map(|r| r.body.len() + 1).sum::<usize>() + WORK_FLOOR,
            floor: 0,
            token: 0,
            in_set: vec![0; n],
            rule_seen: vec![0; gp.rules.len()],
            found: vec![0; n],
            entered: vec![0; n],
            depends: vec![0; n],
        }
    }

    /// Whether the variable of `bits`, of class `c`, is read as values:
    /// each rule outside its class that reads one of its bits reads them
    /// all, directly or through what the rest of its body depends on (a
    /// decision chain's guard that fixes the bits the rule does not
    /// read), or reads nothing but the variable, through gates, and the
    /// rules reading its head read the variable so in turn; and those
    /// rules read two codes or more of the bits. What a literal depends
    /// on is found through gates down to the variables' atoms, which
    /// `variable` marks, and no lower than the bits' first readers, in the
    /// steps [`Reading::left`]; false once they have run out.
    fn as_values(
        &mut self,
        class: &[u32],
        variable: &[bool],
        c: u32,
        bits: &[AtomId],
        work: &mut Work,
    ) -> bool {
        if self.left == 0 {
            return false;
        }
        let (gp, readers) = (self.gp, self.readers);
        self.token += 1;
        let token = self.token;
        self.floor = bits
            .iter()
            .map(|&a| match self.rank[a as usize] {
                NONE => readers[a as usize]
                    .iter()
                    .map(|&ri| self.rank[gp.rules[ri as usize].head as usize])
                    .min()
                    .unwrap_or(NONE),
                place => place,
            })
            .min()
            .unwrap_or(NONE);
        for (k, &a) in bits.iter().enumerate() {
            self.in_set[a as usize] = token;
            self.found[a as usize] = token;
            self.depends[a as usize] = 1 << k | ALONE;
        }
        let all = (1u32 << bits.len()) - 1;
        let start = self.left;
        let mut left = start;
        let mut tested: Vec<(u32, u32)> = Vec::new();
        let mut read: Vec<AtomId> = bits.to_vec();
        let mut ok = true;
        'read: while let Some(a) = read.pop() {
            for &ri in &readers[a as usize] {
                if self.rule_seen[ri as usize] == token {
                    continue;
                }
                self.rule_seen[ri as usize] = token;
                let r = &gp.rules[ri as usize];
                if class[r.head as usize] == c {
                    continue;
                }
                if !work.add(r.body.len()) {
                    return false;
                }
                let mut mask = 0u32;
                let mut code = 0u32;
                for l in &r.body {
                    if self.in_set[l.atom() as usize] == token {
                        let k = bits.iter().position(|&b| b == l.atom()).expect("a bit");
                        mask |= 1 << k;
                        code |= u32::from(l.is_positive()) << k;
                    }
                }
                if mask != 0 && !tested.contains(&(mask, code)) {
                    tested.push((mask, code));
                }
                let mut depends = mask;
                for l in &r.body {
                    if depends & all == all {
                        break;
                    }
                    match self.depends(variable, l.atom(), all & !depends, &mut left) {
                        Some(d) => depends |= d,
                        None => {
                            left = 0;
                            ok = false;
                            break 'read;
                        }
                    }
                }
                if depends & all == all {
                    continue;
                }
                match self.depends(variable, r.head, ALONE, &mut left) {
                    Some(d) if d & ALONE != 0 => read.push(r.head),
                    Some(_) => {
                        ok = false;
                        break 'read;
                    }
                    None => {
                        left = 0;
                        ok = false;
                        break 'read;
                    }
                }
            }
        }
        self.left = left;
        work.add(start - left) && ok && tested.len() >= 2
    }

    /// What `a` depends on, through gates down to the variables' atoms:
    /// the current variable's bits, as a mask, with [`ALONE`] where it
    /// reads nothing else and no query or observation names it. The walk
    /// stops once it has found the bits of `want`, or, where `want` is
    /// [`ALONE`], an atom that is not alone, and gives what it found then;
    /// `None` where the steps `left` run out.
    fn depends(
        &mut self,
        variable: &[bool],
        a: AtomId,
        want: u32,
        left: &mut usize,
    ) -> Option<u32> {
        let (gp, by_head) = (self.gp, self.by_head);
        let token = self.token;
        let mut stack = vec![a];
        let mut reached = 0;
        let mut alone = true;
        while let Some(&x) = stack.last() {
            let x = x as usize;
            if self.found[x] != token {
                if variable[x] || self.rank[x] == NONE || self.rank[x] < self.floor {
                    self.depends[x] = 0;
                } else if self.entered[x] != token {
                    // A gate: its body's atoms first, then itself.
                    self.entered[x] = token;
                    for &ri in &by_head[x] {
                        let body = &gp.rules[ri].body;
                        *left = left.checked_sub(body.len() + 1)?;
                        stack.extend(
                            body.iter()
                                .map(|l| l.atom())
                                .filter(|&y| self.found[y as usize] != token),
                        );
                    }
                    continue;
                } else {
                    let mut mask = 0;
                    let mut alone = !self.named[x];
                    for &ri in &by_head[x] {
                        for l in &gp.rules[ri].body {
                            let d = self.depends[l.atom() as usize];
                            mask |= d & !ALONE;
                            alone &= d & ALONE != 0;
                        }
                    }
                    self.depends[x] = mask | if alone { ALONE } else { 0 };
                }
                self.found[x] = token;
            }
            stack.pop();
            let d = self.depends[x];
            reached |= d & !ALONE;
            alone &= d & ALONE != 0;
            if (want == ALONE && !alone) || (want != ALONE && reached & want == want) {
                // The gates left half read are read again on another walk.
                for &y in &stack {
                    if self.found[y as usize] != token {
                        self.entered[y as usize] = 0;
                    }
                }
                return Some(reached);
            }
        }
        Some(self.depends[a as usize])
    }
}

/// Whether a fact's weights sum to one.
fn normal(p: Prob) -> bool {
    (p.p + p.not - 1.0).abs() <= 1e-12
}

/// Each unit's members: what its class holds.
struct Members {
    units: Vec<u32>,
    lists: Vec<Vec<AtomId>>,
    at: FxHashMap<u32, usize>,
}

impl Members {
    fn new(pairs: Vec<(u32, AtomId)>) -> Self {
        let mut members = Members {
            units: Vec::new(),
            lists: Vec::new(),
            at: FxHashMap::default(),
        };
        for (u, a) in pairs {
            if members.units.last() != Some(&u) {
                members.at.insert(u, members.units.len());
                members.units.push(u);
                members.lists.push(Vec::new());
            }
            members.lists.last_mut().expect("a unit was pushed").push(a);
        }
        members
    }

    fn get(&self, u: u32) -> Option<&[AtomId]> {
        self.at.get(&u).map(|&k| self.lists[k].as_slice())
    }
}

/// A literal of a cone's rule: of an atom of the cone, an input, or one of
/// the unit's facts, by its number among them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Of {
    Atom(u32),
    Input(u32),
    Fact(u32),
}

#[derive(Clone, Copy, Debug)]
struct Lit {
    of: Of,
    positive: bool,
}

#[derive(Clone, Debug)]
struct Rule {
    head: u32,
    body: Vec<Lit>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    False,
    True,
    Open,
}

/// A row: the inputs fixed on its way, and its distribution over the
/// codes.
struct Leaf {
    cube: Vec<(u32, bool)>,
    dist: Distribution,
}

/// Codes, each with its probability, in the order of the codes.
type Distribution = Vec<(u32, f64)>;

/// A unit's cone ([`Cone::of`]), with scratch kept from one unit to the
/// next.
struct Cone {
    stamp: Vec<u32>,
    local: Vec<u32>,
    role: Vec<u8>,
    current: u32,
    atoms: Vec<AtomId>,
    inputs: Vec<AtomId>,
    facts: Vec<AtomId>,
    bits: Vec<Of>,
    weights: Vec<Prob>,
    literals: usize,
}

const ATOM: u8 = 0;
const INPUT: u8 = 1;
const FACT: u8 = 2;

impl Cone {
    fn new(n: usize) -> Self {
        Cone {
            stamp: vec![0; n],
            local: vec![0; n],
            role: vec![0; n],
            current: 0,
            atoms: Vec::new(),
            inputs: Vec::new(),
            facts: Vec::new(),
            bits: Vec::new(),
            weights: Vec::new(),
            literals: 0,
        }
    }

    /// The cone of the unit of `members` and `interface`: its atoms, in
    /// dependency order, its inputs and its facts; false where a fact it
    /// sums over has weights that do not sum to one. Its rules are read
    /// over them as [`Cone::rows`] expands them.
    fn of(&mut self, units: &Units, members: &[AtomId], interface: &[AtomId]) -> bool {
        let (gp, by_head) = (units.gp, units.by_head);
        self.current += 1;
        let current = self.current;
        self.atoms.clear();
        self.inputs.clear();
        self.facts.clear();
        let mut stack: Vec<AtomId> = Vec::new();
        for &a in members {
            self.stamp[a as usize] = current;
            if units.is_gate(a) {
                self.role[a as usize] = ATOM;
                stack.push(a);
            } else {
                self.role[a as usize] = FACT;
                self.facts.push(a);
            }
        }
        self.literals = 0;
        while let Some(a) = stack.pop() {
            self.atoms.push(a);
            for &ri in &by_head[a as usize] {
                self.literals += gp.rules[ri].body.len() + 1;
                for l in &gp.rules[ri].body {
                    let y = l.atom();
                    if self.stamp[y as usize] == current {
                        continue;
                    }
                    self.stamp[y as usize] = current;
                    if units.view[y as usize] {
                        self.role[y as usize] = ATOM;
                        stack.push(y);
                    } else {
                        self.role[y as usize] = INPUT;
                        self.inputs.push(y);
                    }
                }
            }
        }
        self.atoms.sort_unstable_by_key(|&a| units.rank[a as usize]);
        self.inputs.sort_unstable();
        self.facts.sort_unstable();
        for list in [&self.atoms, &self.inputs, &self.facts] {
            for (k, &a) in list.iter().enumerate() {
                self.local[a as usize] = k as u32;
            }
        }
        self.weights.clear();
        for &f in &self.facts {
            match gp.kinds[f as usize] {
                AtomKind::Fact(p) if normal(p) => self.weights.push(p),
                _ => return false,
            }
        }
        self.bits = interface.iter().map(|&a| self.place(a)).collect();
        true
    }

    /// The atom `y` of the cone as a literal reads it.
    fn place(&self, y: AtomId) -> Of {
        let k = self.local[y as usize];
        match self.role[y as usize] {
            ATOM => Of::Atom(k),
            INPUT => Of::Input(k),
            _ => Of::Fact(k),
        }
    }

    /// The cone's rows, or `None` where `limit` steps or the rows' bounds
    /// are passed; the steps taken added to `work`. Where the cone has no
    /// input, its rules are its one row's, and they are not read where
    /// they alone pass that row's steps.
    fn rows(&mut self, units: &Units, limit: usize, work: &mut Work) -> Option<Vec<Leaf>> {
        if self.inputs.is_empty() && self.literals > row_work(self.bits.len(), 0) {
            return None;
        }
        let (gp, by_head) = (units.gp, units.by_head);
        let mut rules = Vec::new();
        for (k, &a) in self.atoms.iter().enumerate() {
            for &ri in &by_head[a as usize] {
                rules.push(Rule {
                    head: k as u32,
                    body: gp.rules[ri]
                        .body
                        .iter()
                        .map(|l| Lit {
                            of: self.place(l.atom()),
                            positive: l.is_positive(),
                        })
                        .collect(),
                });
            }
        }
        let mut expansion = Expansion {
            bits: &self.bits,
            weights: &self.weights,
            leaves: Vec::new(),
            work: 0,
            work_limit: limit,
            written: 0,
            written_limit: GROWTH * self.literals,
            status: vec![Status::False; self.atoms.len()],
            reached: vec![false; self.atoms.len()],
            fixed: vec![None; self.facts.len()],
        };
        let mut cube = Vec::new();
        let ok = expansion.expand(rules, &mut cube);
        work.add(expansion.work);
        ok.then_some(expansion.leaves)
    }
}

/// The expansion of a cone's rules over its inputs and its facts
/// ([`Expansion::expand`]).
struct Expansion<'a> {
    /// The interface, a bit each.
    bits: &'a [Of],
    /// The facts' weights, by their number.
    weights: &'a [Prob],
    leaves: Vec<Leaf>,
    work: usize,
    work_limit: usize,
    /// The literals the rows found so far write at least.
    written: usize,
    written_limit: usize,
    /// Per atom of the cone, scratch for [`Expansion::simplified`].
    status: Vec<Status>,
    reached: Vec<bool>,
    /// Each fact's value on the way to the current code.
    fixed: Vec<Option<bool>>,
}

impl Expansion<'_> {
    /// The rows of `rules` under `cube`, into `leaves`; false where a bound
    /// is passed.
    fn expand(&mut self, rules: Vec<Rule>, cube: &mut Vec<(u32, bool)>) -> bool {
        let rules = self.simplified(rules);
        let next = rules
            .iter()
            .flat_map(|r| &r.body)
            .filter_map(|l| match l.of {
                Of::Input(x) => Some(x),
                _ => None,
            })
            .min();
        if let Some(x) = next {
            for value in [false, true] {
                let under = self.restricted(&rules, Of::Input(x), value);
                if self.work > self.work_limit {
                    return false;
                }
                cube.push((x, value));
                let ok = self.expand(under, cube);
                cube.pop();
                if !ok {
                    return false;
                }
            }
            return true;
        }
        let mut dist = Vec::new();
        let limit = self.work_limit;
        self.work_limit = limit.min(self.work + row_work(self.bits.len(), cube.len()));
        let ok = self.sum(rules, 1.0, &mut dist);
        self.work_limit = limit;
        if !ok {
            return false;
        }
        dist.sort_unstable_by_key(|&(c, _)| c);
        self.written += dist.len() * (cube.len() + 1);
        if self.written > self.written_limit {
            return false;
        }
        self.leaves.push(Leaf {
            cube: cube.clone(),
            dist,
        });
        true
    }

    /// The distribution over the codes of `rules`, simplified, which read
    /// no input, times `weight`, added into `dist`; false where a bound is
    /// passed.
    fn sum(&mut self, rules: Vec<Rule>, weight: f64, dist: &mut Vec<(u32, f64)>) -> bool {
        let read = rules
            .iter()
            .flat_map(|r| &r.body)
            .filter_map(|l| match l.of {
                Of::Fact(f) => Some(f),
                _ => None,
            })
            .min();
        let unset = self
            .bits
            .iter()
            .filter_map(|&b| match b {
                Of::Fact(f) if self.fixed[f as usize].is_none() => Some(f),
                _ => None,
            })
            .min();
        let next = match (read, unset) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let Some(f) = next else {
            self.work += self.bits.len() + 1;
            let code = self.bits.iter().enumerate().fold(0u32, |c, (k, &b)| {
                let set = match b {
                    Of::Atom(j) => rules.iter().any(|r| r.head == j),
                    Of::Fact(f) => self.fixed[f as usize] == Some(true),
                    Of::Input(_) => unreachable!("a bit is the unit's"),
                };
                c | u32::from(set) << k
            });
            match dist.iter_mut().find(|(c, _)| *c == code) {
                Some((_, p)) => *p += weight,
                None => dist.push((code, weight)),
            }
            return self.work <= self.work_limit;
        };
        for value in [false, true] {
            let w = weight * self.weights[f as usize].of(value);
            if w == 0.0 {
                continue;
            }
            let under = self.restricted(&rules, Of::Fact(f), value);
            if self.work > self.work_limit {
                return false;
            }
            let under = self.simplified(under);
            self.fixed[f as usize] = Some(value);
            let ok = self.sum(under, w, dist);
            self.fixed[f as usize] = None;
            if !ok {
                return false;
            }
        }
        true
    }

    /// `rules` under `of` set to `value`: those that read it the other way
    /// dropped, and it dropped from the others.
    fn restricted(&mut self, rules: &[Rule], of: Of, value: bool) -> Vec<Rule> {
        let mut under = Vec::with_capacity(rules.len());
        for r in rules {
            if r.body.iter().any(|l| l.of == of && l.positive != value) {
                continue;
            }
            let body: Vec<Lit> = r.body.iter().copied().filter(|l| l.of != of).collect();
            self.work += body.len() + 1;
            under.push(Rule { head: r.head, body });
        }
        under
    }

    /// `rules` with the atoms they leave constant folded into the bodies
    /// that read them, a true atom kept as one rule of an empty body, and
    /// the rules of atoms the interface no longer reads dropped. The rules
    /// come by head in dependency order, and stay so.
    fn simplified(&mut self, rules: Vec<Rule>) -> Vec<Rule> {
        let mut out: Vec<Rule> = Vec::with_capacity(rules.len());
        let mut touched: Vec<u32> = Vec::new();
        let mut i = 0;
        while i < rules.len() {
            let head = rules[i].head;
            let start = out.len();
            let mut certain = false;
            while i < rules.len() && rules[i].head == head {
                let r = &rules[i];
                i += 1;
                if certain {
                    continue;
                }
                self.work += r.body.len() + 1;
                let mut dead = false;
                let mut body = Vec::with_capacity(r.body.len());
                for &l in &r.body {
                    match l.of {
                        Of::Atom(j) => match self.status[j as usize] {
                            Status::Open => body.push(l),
                            Status::True => dead = !l.positive,
                            Status::False => dead = l.positive,
                        },
                        _ => body.push(l),
                    }
                    if dead {
                        break;
                    }
                }
                if dead {
                    continue;
                }
                if body.is_empty() {
                    certain = true;
                    out.truncate(start);
                }
                out.push(Rule { head, body });
            }
            if out.len() > start {
                self.status[head as usize] = if certain { Status::True } else { Status::Open };
                touched.push(head);
            }
        }
        // The rules of the atoms the interface reads, through the rules.
        for &b in self.bits {
            if let Of::Atom(j) = b {
                self.reached[j as usize] = true;
            }
        }
        let mut kept = vec![false; out.len()];
        for (k, r) in out.iter().enumerate().rev() {
            if self.reached[r.head as usize] {
                kept[k] = true;
                for l in &r.body {
                    if let Of::Atom(j) = l.of {
                        self.reached[j as usize] = true;
                    }
                }
            }
        }
        for &h in &touched {
            self.status[h as usize] = Status::False;
            self.reached[h as usize] = false;
        }
        for &b in self.bits {
            if let Of::Atom(j) = b {
                self.reached[j as usize] = false;
            }
        }
        let mut kept = kept.into_iter();
        out.retain(|_| kept.next() == Some(true));
        out
    }
}

/// The units written so far and what they write: atoms numbered after the
/// program's, and rules.
struct Written {
    /// The program's atoms, which the new atoms are numbered after.
    first: usize,
    atoms: Vec<Atom>,
    kinds: Vec<AtomKind>,
    /// Each written unit's codes and the atom of each code's value.
    values: Vec<(Vec<u32>, Vec<AtomId>)>,
    /// The written unit and bit of each interface atom.
    bit_of: FxHashMap<AtomId, (usize, u32)>,
    /// The atoms the units written hold, whose rules the rows replace.
    replaced: Vec<AtomId>,
    /// The facts of their interfaces, which their values now define.
    facts: Vec<AtomId>,
    rules: Vec<GroundRule>,
}

impl Written {
    fn new(first: usize) -> Self {
        Written {
            first,
            atoms: Vec::new(),
            kinds: Vec::new(),
            values: Vec::new(),
            bit_of: FxHashMap::default(),
            replaced: Vec::new(),
            facts: Vec::new(),
            rules: Vec::new(),
        }
    }

    /// A new atom: `base`'s predicate with `suffix`, `base`'s arguments and
    /// `k`.
    fn atom(&mut self, base: &Atom, suffix: &str, k: usize, kind: AtomKind) -> AtomId {
        let id = (self.first + self.atoms.len()) as AtomId;
        let mut args = base.args.as_slice().to_vec();
        args.push(Term::Int(k as i64));
        self.atoms
            .push(Atom::new(format!("{}${suffix}", base.name), args));
        self.kinds.push(kind);
        id
    }

    /// The literals of `literals` on no written unit's bits, and for each
    /// written unit they read bits of, the values whose codes those bits
    /// spell; `None` where they spell none.
    fn read(&self, literals: &[GLit]) -> Option<(Vec<GLit>, Vec<Vec<AtomId>>)> {
        let mut kept: Vec<GLit> = Vec::new();
        // Per written unit, the bits fixed and their values.
        let mut fixed: Vec<(usize, u32, u32)> = Vec::new();
        for &l in literals {
            let Some(&(w, bit)) = self.bit_of.get(&l.atom()) else {
                kept.push(l);
                continue;
            };
            let value = u32::from(l.is_positive()) << bit;
            match fixed.iter_mut().find(|f| f.0 == w) {
                Some(f) => {
                    if f.1 & 1 << bit != 0 && f.2 & 1 << bit != value {
                        return None;
                    }
                    f.1 |= 1 << bit;
                    f.2 |= value;
                }
                None => fixed.push((w, 1 << bit, value)),
            }
        }
        let mut values = Vec::with_capacity(fixed.len());
        for &(w, mask, value) in &fixed {
            let (codes, atoms) = &self.values[w];
            let matching: Vec<AtomId> = codes
                .iter()
                .zip(atoms)
                .filter(|&(&c, _)| c & mask == value)
                .map(|(_, &v)| v)
                .collect();
            if matching.is_empty() {
                return None;
            }
            values.push(matching);
        }
        Some((kept, values))
    }

    /// What is written so far, for [`Written::undo`].
    fn mark(&self) -> [usize; 5] {
        [
            self.atoms.len(),
            self.values.len(),
            self.replaced.len(),
            self.facts.len(),
            self.rules.len(),
        ]
    }

    /// Takes back what was written since `mark`.
    fn undo(&mut self, [atoms, values, replaced, facts, rules]: [usize; 5]) {
        self.atoms.truncate(atoms);
        self.kinds.truncate(atoms);
        self.values.truncate(values);
        self.replaced.truncate(replaced);
        self.facts.truncate(facts);
        self.rules.truncate(rules);
        self.bit_of.retain(|_, &mut (w, _)| w < values);
    }

    /// Writes the unit of `members` and `interface` as the rows `leaves`
    /// over its `inputs`, where they hold at most `limit` literals;
    /// whether they do.
    fn write(
        &mut self,
        gp: &GroundProgram,
        members: &[AtomId],
        interface: &[AtomId],
        inputs: &[AtomId],
        leaves: &[Leaf],
        limit: usize,
    ) -> bool {
        // Each row's contexts, read through the written units' values.
        let mut rows: Vec<(Vec<GLit>, &Distribution)> = Vec::new();
        let mut size = 0;
        for leaf in leaves {
            let literals: Vec<GLit> = leaf
                .cube
                .iter()
                .map(|&(x, v)| GLit::new(inputs[x as usize], v))
                .collect();
            let Some((kept, values)) = self.read(&literals) else {
                continue;
            };
            let mut bodies: Vec<Vec<GLit>> = vec![kept];
            for matching in values {
                if bodies.len() * matching.len() > limit {
                    return false;
                }
                bodies = bodies
                    .iter()
                    .flat_map(|b| {
                        matching.iter().map(move |&v| {
                            let mut b = b.clone();
                            b.push(GLit::Pos(v));
                            b
                        })
                    })
                    .collect();
            }
            let k = leaf.dist.len();
            for body in bodies {
                size += k * body.len() + k * (k + 1) / 2;
                rows.push((body, &leaf.dist));
            }
            if size > limit {
                return false;
            }
        }
        let mut codes: Vec<u32> = rows
            .iter()
            .flat_map(|(_, d)| d.iter().map(|&(c, _)| c))
            .collect();
        codes.sort_unstable();
        codes.dedup();
        let base = &gp.atoms[interface[0] as usize];
        let values: Vec<AtomId> = codes
            .iter()
            .map(|&c| self.atom(base, "value", c as usize, AtomKind::Derived))
            .collect();
        let mut switch = 0;
        for (body, dist) in rows {
            // Each switch true with its code's share of what is left, false
            // with the rest; the last code takes what is left.
            let mut left: Vec<f64> = vec![0.0; dist.len() + 1];
            for k in (0..dist.len()).rev() {
                left[k] = left[k + 1] + dist[k].1;
            }
            let mut chain: Vec<GLit> = Vec::new();
            for (k, &(c, p)) in dist.iter().enumerate() {
                let mut b = body.clone();
                b.extend_from_slice(&chain);
                if k + 1 < dist.len() {
                    let weights = Prob {
                        p: p / left[k],
                        not: left[k + 1] / left[k],
                    };
                    let s = self.atom(base, "switch", switch, AtomKind::Fact(weights));
                    switch += 1;
                    b.push(GLit::Pos(s));
                    chain.push(GLit::Neg(s));
                }
                let v = values[codes.binary_search(&c).expect("a code of the unit")];
                self.rules.push(GroundRule::new(v, b));
            }
        }
        let w = self.values.len();
        for (bit, &i) in interface.iter().enumerate() {
            for (&c, &v) in codes.iter().zip(&values) {
                if c >> bit & 1 == 1 {
                    self.rules.push(GroundRule::new(i, vec![GLit::Pos(v)]));
                }
            }
            self.bit_of.insert(i, (w, bit as u32));
            if matches!(gp.kinds[i as usize], AtomKind::Fact(_)) {
                self.facts.push(i);
            }
        }
        self.replaced.extend_from_slice(members);
        self.values.push((codes, values));
        true
    }

    /// `gp` with the rules of the atoms the units hold replaced by the rows
    /// written, and the other rules that read a value of a written unit
    /// through its bits reading it; how many units were written.
    fn finish(self, gp: &mut GroundProgram) -> usize {
        if self.values.is_empty() {
            return 0;
        }
        let mut replaced = vec![false; gp.num_atoms()];
        for &a in &self.replaced {
            replaced[a as usize] = true;
        }
        let old = std::mem::take(&mut gp.rules);
        let mut rules = Vec::with_capacity(old.len() + self.rules.len());
        for mut r in old {
            if replaced[r.head as usize] {
                continue;
            }
            if r.body.iter().any(|l| self.bit_of.contains_key(&l.atom())) {
                let Some((mut kept, values)) = self.read(&r.body) else {
                    continue;
                };
                if values.iter().all(|m| m.len() == 1) {
                    kept.extend(values.iter().map(|m| GLit::Pos(m[0])));
                    r.body = kept;
                }
            }
            rules.push(r);
        }
        rules.extend(self.rules);
        gp.rules = rules;
        for &f in &self.facts {
            gp.kinds[f as usize] = AtomKind::Derived;
        }
        gp.atoms.extend(self.atoms);
        gp.kinds.extend(self.kinds);
        gp.structure = OnceLock::new();
        self.values.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::GroundOptions;

    /// What the pass writes it leaves as it is: a program written over its
    /// values, simplified again as the conditioning on guards finishes it
    /// again, holds no variable as bits.
    #[test]
    fn a_variable_written_over_its_values_is_not_written_again() {
        let source = crate::syntax::dice::read(
            "let B = discrete(0.2, 0.3, 0.5) in
             let C = if (B == int(2, 0)) then (discrete(0.1, 0.6, 0.3))
                 else (if (B == int(2, 1)) then (discrete(0.4, 0.4, 0.2))
                 else (discrete(0.7, 0.2, 0.1))) in
             (C == int(2, 0), C == int(2, 1))",
        )
        .expect("reads");
        let program = crate::syntax::parse_program(&source).expect("parses");
        let mut gp = crate::ground::ground(&program, &GroundOptions::default()).expect("grounds");
        assert!(gp.stats.values > 0, "{gp}");
        assert_eq!(super::bits_as_values(&mut gp), 0, "{gp}");
    }
}
