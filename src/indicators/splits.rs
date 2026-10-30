//! Where a table is split on a parent ([`table_splits`]), and the units
//! that read each other's atoms through indicators ([`units`]).

use rustc_hash::{FxHashMap, FxHashSet};

use super::{PartId, WHOLE};
use crate::disjoint::{find, union};
use crate::ground::{AtomId, BITS_FROM, GroundProgram, OWN_VALUE_FROM};

/// A table is split where its rows squared are at least this many times
/// what the split table costs ([`table_splits`]).
const SPLIT_GAIN: usize = 2;

/// The parts of the split tables ([`table_splits`]): a tree whose root is
/// every table whole, in which a part split on a parent has a part for each
/// value of it, of the rules that read that value. The parts are numbered
/// depth first, each part's in the order their rules first read them.
pub(super) struct Parts {
    /// Per part the part it is of; the whole's is itself.
    pub(super) up: Vec<PartId>,
    /// Per part the exclusive set the part it is of is split on.
    set: Vec<usize>,
    /// Per part the value of that set its rules read.
    pub(super) value: Vec<AtomId>,
    /// Per part whether its rules read the copy of that value alone
    /// ([`OWN_VALUE_FROM`]).
    own_value: Vec<bool>,
    /// The part each rule is in, the smallest.
    pub(super) of_rule: Vec<PartId>,
}

impl Parts {
    /// The part whose copies of the set `s` a rule of the part `leaf` reads,
    /// and whether it reads its value's alone: the part split on `s` on the
    /// way up from `leaf`, if one is, and otherwise `leaf` itself.
    pub(super) fn reading(&self, leaf: PartId, s: usize) -> (PartId, bool) {
        let mut p = leaf;
        while p != WHOLE {
            if self.set[p as usize] == s {
                return (self.up[p as usize], self.own_value[p as usize]);
            }
            p = self.up[p as usize];
        }
        (leaf, false)
    }

    fn place(&mut self, plan: Plan, rules: &[usize], part: PartId) {
        let Some(split) = plan.split else {
            for &ri in rules {
                self.of_rule[ri] = part;
            }
            return;
        };
        for (value, rules, plan) in split.parts {
            let p = self.up.len() as PartId;
            self.up.push(part);
            self.set.push(split.set);
            self.value.push(value);
            self.own_value.push(split.own_value);
            self.place(plan, &rules, p);
        }
    }
}

/// How [`table_splits`] would split some rules of a unit, and what the
/// diagram of their tables costs so.
struct Plan {
    cost: usize,
    split: Option<Split>,
}

/// A split of some rules of a unit on a parent.
struct Split {
    /// The parent's exclusive set.
    set: usize,
    /// Whether the rules of a part read the parent's copy of their value
    /// alone ([`OWN_VALUE_FROM`]).
    own_value: bool,
    /// Per value the rules that read it and how they are split in turn.
    parts: Vec<(AtomId, Vec<usize>, Plan)>,
}

/// A parent `Unit::plan` weighs splitting some rules on: what the split
/// costs with its parts whole, the parent's exclusive set, whether the
/// parts read their values alone, what the split costs over its parts, per
/// value the rules that read it, and below a whole unit what each part
/// costs whole ([`Unit::whole`]).
struct Candidate {
    flat: usize,
    set: usize,
    own_value: bool,
    over: usize,
    values: Vec<(AtomId, Vec<usize>)>,
    wholes: Vec<usize>,
}

/// What a copy costs a part of a split table, in the pairs of
/// [`table_splits`]' model: its link is a formula of its own to compile and
/// conjoin, and its variable a level of the vtree.
const COPY_COST: usize = 64;

/// The parts of the tables of each unit.
///
/// The diagram of a table over its parents' copies pays, at each level of
/// the spine of switches above the copies, for every row still to come
/// (the families vtree, [`crate::vtree`]): r rows cost about r²/2 pairs.
/// Where every rule of a unit reads exactly one atom of an exclusive set P
/// positively, one value of a parent, the rules for each value a can read
/// the other parents through copies of their own. Its rows are then a table
/// of their own above those copies, and the table over P has a row per
/// value: |P|² + Σ_a r_a² against r², for |P| - 1 more copies of each other
/// parent's values. Where the parts read P's copies alone
/// ([`OWN_VALUE_FROM`]), they go side by side in a balanced tree instead,
/// and the table over P costs |P| log₂|P|: a table of one row of one
/// switch per value of P is then split too. A table that holds no switch,
/// or whose rows read P alone, is not split on a set of [`BITS_FROM`]
/// values or more that two units or more read ([`Unit::read_in_binary`]):
/// it reads the set in binary, as bn_pathfinder's row classes and tables
/// of `fault` alone read `fault`.
///
/// P is the set for which the split costs least with its parts whole, and a
/// unit is split where that is less than its rows squared by
/// [`SPLIT_GAIN`].
///
/// Each part is split again the same way, on another parent, where the
/// split, its own parts split or not, costs less than the part whole by
/// [`SPLIT_GAIN`]. A part's table costs its switches times its rows, which
/// is what its spine holds, and each copy it reads [`COPY_COST`]: priced at
/// its rows squared and nothing per copy, a part is split again for more
/// copies than it saves, and a cost per copy for whole units too keeps
/// too many copies. As it is, the parts of bn_barley's tables are split in
/// turn, and no other network of the bench is split below its units.
pub(super) fn table_splits(
    gp: &GroundProgram,
    unit_of: &[usize],
    exclusive: &[Vec<AtomId>],
    set_of: &[usize],
    is_chosen: &[bool],
) -> Parts {
    let observed: FxHashSet<AtomId> = gp.evidence.iter().filter_map(|&(_, a, _)| a).collect();
    // Each unit's rules in order, the units in the order of their first
    // rules, laid end to end: the `k`-th unit's are `unit_rules[from[k]..
    // from[k + 1]]`, not a map of a list per unit grown a rule at a time.
    let num_units = unit_of
        .iter()
        .filter(|&&u| u != usize::MAX)
        .map(|&u| u + 1)
        .max()
        .unwrap_or(0);
    let mut slot = vec![usize::MAX; num_units];
    let mut from: Vec<usize> = vec![0];
    for r in &gp.rules {
        let u = unit_of[r.head as usize];
        if u != usize::MAX {
            if slot[u] == usize::MAX {
                slot[u] = from.len() - 1;
                from.push(0);
            }
            from[slot[u] + 1] += 1;
        }
    }
    for k in 1..from.len() {
        from[k] += from[k - 1];
    }
    let mut next = from.clone();
    let mut unit_rules = vec![0; from[from.len() - 1]];
    for (ri, r) in gp.rules.iter().enumerate() {
        let u = unit_of[r.head as usize];
        if u != usize::MAX {
            unit_rules[next[slot[u]]] = ri;
            next[slot[u]] += 1;
        }
    }
    // What each rule reads positively: the atoms of exclusive sets, with
    // their sets, and the atoms of none it reads through copies; and
    // whether it reads each through a copy.
    let mut reads = Reads {
        all: Vec::new(),
        from: Vec::with_capacity(gp.rules.len() + 1),
    };
    reads.from.push(0);
    let mut read: Vec<(usize, AtomId, bool)> = Vec::new();
    for r in &gp.rules {
        let u = unit_of[r.head as usize];
        read.clear();
        read.extend(
            r.body
                .iter()
                .filter(|l| l.is_positive())
                .map(|l| {
                    let a = l.atom();
                    (
                        set_of[a as usize],
                        a,
                        is_chosen[a as usize] && unit_of[a as usize] != u,
                    )
                })
                .filter(|&(s, _, copied)| s != usize::MAX || copied),
        );
        read.sort_unstable();
        read.dedup();
        reads.all.extend_from_slice(&read);
        reads.from.push(reads.all.len());
    }
    let wide_and_shared = wide_and_shared(gp, unit_of, exclusive, &reads);
    let mut parts = Parts {
        up: vec![WHOLE],
        set: vec![usize::MAX],
        value: vec![AtomId::MAX],
        own_value: vec![false],
        of_rule: vec![WHOLE; gp.rules.len()],
    };
    let mut distinct = Distinct {
        stamp: vec![0; gp.atoms.len()],
        index: vec![0; gp.atoms.len()],
        now: 0,
    };
    let mut index = RowIndex {
        row_of: vec![u32::MAX; gp.rules.len()],
        ..RowIndex::default()
    };
    for k in 0..from.len() - 1 {
        let rules = &unit_rules[from[k]..from[k + 1]];
        let mut heads: Vec<AtomId> = rules.iter().map(|&ri| gp.rules[ri].head).collect();
        heads.sort_unstable();
        heads.dedup();
        // The tables of the unit that something outside it reads, by an
        // observation or through a copy of its own: a head's exclusive set,
        // or the head itself where it has none.
        let mut exposed: Vec<usize> = heads
            .iter()
            .filter(|&&h| is_chosen[h as usize] || observed.contains(&h))
            .map(|&h| {
                if set_of[h as usize] == usize::MAX {
                    usize::MAX - h as usize
                } else {
                    set_of[h as usize]
                }
            })
            .collect();
        exposed.sort_unstable();
        exposed.dedup();
        // The values of the largest table of the unit that another unit
        // reads through copies of its own ([`OWN_VALUE_FROM`]).
        let read_values = heads
            .iter()
            .filter(|&&h| is_chosen[h as usize])
            .map(|&h| {
                if set_of[h as usize] == usize::MAX {
                    1
                } else {
                    exclusive[set_of[h as usize]].len()
                }
            })
            .max()
            .unwrap_or(0);
        let mut unit = Unit {
            gp,
            exclusive,
            reads: &reads,
            heads: heads.len(),
            own_value: exposed.len() <= 1 && read_values < OWN_VALUE_FROM,
            distinct: &mut distinct,
            wide_and_shared: &wide_and_shared,
            switch_free: rules
                .iter()
                .all(|&ri| gp.rules[ri].body.iter().all(|l| !gp.is_fact(l.atom()))),
            rules,
            indexed: false,
            index: &mut index,
        };
        let plan = unit.plan(rules, &mut Vec::new(), None);
        parts.place(plan, rules, WHOLE);
    }
    parts
}

/// Per exclusive set whether it has [`BITS_FROM`] values or more and two
/// units or more read it through copies (`reads`, [`table_splits`]), so
/// that some of its readers read it in binary ([`Unit::read_in_binary`]).
/// Where one unit alone reads such a set, as a table reads its row classes
/// ([`crate::ground::classes`]), the split on it still pays.
fn wide_and_shared(
    gp: &GroundProgram,
    unit_of: &[usize],
    exclusive: &[Vec<AtomId>],
    reads: &Reads,
) -> Vec<bool> {
    let mut first_reader = vec![usize::MAX; exclusive.len()];
    let mut shared = vec![false; exclusive.len()];
    for (ri, r) in gp.rules.iter().enumerate() {
        let u = unit_of[r.head as usize];
        for &(s, _, copied) in &reads[ri] {
            if s == usize::MAX || !copied || exclusive[s].len() < BITS_FROM {
                continue;
            }
            match first_reader[s] {
                usize::MAX => first_reader[s] = u,
                v if v != u => shared[s] = true,
                _ => {}
            }
        }
    }
    shared
}

/// What each rule reads positively ([`table_splits`]), sorted and
/// distinct: rule `ri`'s are `all[from[ri]..from[ri + 1]]`, the rules'
/// laid end to end, not a list allocated and grown per rule.
struct Reads {
    all: Vec<(usize, AtomId, bool)>,
    from: Vec<usize>,
}

impl std::ops::Index<usize> for Reads {
    type Output = [(usize, AtomId, bool)];

    fn index(&self, ri: usize) -> &Self::Output {
        &self.all[self.from[ri]..self.from[ri + 1]]
    }
}

/// A unit whose rules [`table_splits`] splits.
struct Unit<'a> {
    gp: &'a GroundProgram,
    exclusive: &'a [Vec<AtomId>],
    /// Per rule what it reads positively ([`table_splits`]), sorted.
    reads: &'a Reads,
    /// Its atoms, each of which has a rule per row.
    heads: usize,
    /// Whether its parts may read their values alone, as far as the unit
    /// goes ([`OWN_VALUE_FROM`]).
    own_value: bool,
    distinct: &'a mut Distinct,
    /// Per exclusive set whether it is wide and shared
    /// ([`wide_and_shared`]), and whether no rule of the unit reads a fact
    /// ([`Unit::read_in_binary`]).
    wide_and_shared: &'a [bool],
    switch_free: bool,
    /// Its rules, and whether its rows are in `index` yet.
    rules: &'a [usize],
    indexed: bool,
    index: &'a mut RowIndex,
}

/// The rows of the units, made where [`Unit::whole`] first needs a unit's.
/// The rules of a unit that read the same atoms, of sets and through
/// copies, are a row; a split goes by those atoms, so every part is made
/// of whole rows. A row keeps the facts its rules read, each once: the
/// rules of a row of a table read its switches over and over, each value's
/// rule those of the values before it (the grounder's normalization), and
/// the parts' costs read every literal of every rule.
#[derive(Default)]
struct RowIndex {
    /// Per rule its row, where its unit's rows are made.
    row_of: Vec<u32>,
    /// Per row a rule of it, whose reads are the row's, and its number of
    /// rules.
    first: Vec<usize>,
    size: Vec<usize>,
    /// Per row the facts its rules read, each once, by `fact_ends`.
    facts: Vec<AtomId>,
    fact_ends: Vec<usize>,
    /// Per row the count it was last listed in, and the rows listed in the
    /// count at hand.
    seen: Vec<u32>,
    now: u32,
    part: Vec<u32>,
}

impl RowIndex {
    /// The facts row `r` reads.
    fn facts(&self, r: u32) -> &[AtomId] {
        let r = r as usize;
        &self.facts[if r == 0 { 0 } else { self.fact_ends[r - 1] }..self.fact_ends[r]]
    }
}

/// Counts distinct atoms by stamping them, one stamp per count, rather
/// than sorting the facts of each part to count them. Numbers
/// them too, in the order they first come, one stamp per numbering.
struct Distinct {
    stamp: Vec<u32>,
    index: Vec<u32>,
    now: u32,
}

impl Distinct {
    /// A new numbering of [`Distinct::number`].
    fn start(&mut self) {
        self.now += 1;
    }

    /// The number of `a` in the numbering at hand, `next` if it has none.
    fn number(&mut self, a: AtomId, next: usize) -> usize {
        if self.stamp[a as usize] != self.now {
            self.stamp[a as usize] = self.now;
            self.index[a as usize] = next as u32;
        }
        self.index[a as usize] as usize
    }

    /// Whether `a` is new to the count or numbering at hand, which it is
    /// then not.
    fn first(&mut self, a: AtomId) -> bool {
        let s = &mut self.stamp[a as usize];
        let new = *s != self.now;
        *s = self.now;
        new
    }

    fn count(&mut self, atoms: impl Iterator<Item = AtomId>) -> usize {
        self.now += 1;
        atoms.filter(|&a| self.first(a)).count()
    }
}

impl Unit<'_> {
    fn rows(&self, rules: usize) -> usize {
        rules.div_ceil(self.heads)
    }

    /// Make the unit's rows ([`RowIndex`]), once.
    fn index_rows(&mut self) {
        if std::mem::replace(&mut self.indexed, true) {
            return;
        }
        let (gp, reads, index) = (self.gp, self.reads, &mut *self.index);
        let start = index.first.len();
        // At most a row per rule, for which the map is allocated once.
        let mut row_by_reads: FxHashMap<&[(usize, AtomId, bool)], u32> =
            FxHashMap::with_capacity_and_hasher(self.rules.len(), Default::default());
        for &ri in self.rules {
            let next = index.first.len() as u32;
            let r = *row_by_reads.entry(&reads[ri]).or_insert(next);
            if r == next {
                index.first.push(ri);
                index.size.push(0);
            }
            index.size[r as usize] += 1;
            index.row_of[ri] = r;
        }
        // The rules of each row together, by a count of them, and each
        // row's facts from its rules'.
        let mut at: Vec<usize> = Vec::with_capacity(index.first.len() - start);
        let mut end = 0;
        for &n in &index.size[start..] {
            at.push(end);
            end += n;
        }
        let mut by_row = vec![0; self.rules.len()];
        for &ri in self.rules {
            let k = index.row_of[ri] as usize - start;
            by_row[at[k]] = ri;
            at[k] += 1;
        }
        let mut from = 0;
        for k in 0..index.first.len() - start {
            self.distinct.start();
            for &ri in &by_row[from..at[k]] {
                for l in &gp.rules[ri].body {
                    let f = l.atom();
                    if gp.is_fact(f) && self.distinct.first(f) {
                        index.facts.push(f);
                    }
                }
            }
            index.fact_ends.push(index.facts.len());
            from = at[k];
        }
        index.seen.resize(index.first.len(), 0);
    }

    /// Whether the rules of two values read one fact.
    fn shared(&self, values: &[(AtomId, Vec<usize>)]) -> bool {
        let mut value_of: FxHashMap<AtomId, AtomId> = FxHashMap::default();
        values.iter().any(|(a, rules)| {
            rules.iter().any(|&ri| {
                self.gp.rules[ri]
                    .body
                    .iter()
                    .map(|l| l.atom())
                    .filter(|&f| self.gp.is_fact(f))
                    .any(|f| *value_of.entry(f).or_insert(*a) != *a)
            })
        })
    }

    /// What the table of `rules` costs whole: its rows squared for a whole
    /// unit, and below that its switches times its rows and each copy it
    /// reads [`COPY_COST`].
    fn whole(&mut self, rules: &[usize], path: &[usize]) -> usize {
        let rows = self.rows(rules.len());
        if path.is_empty() {
            return rows * rows;
        }
        // The facts and the copies of other parents than those in `path`
        // that the rules read, from their rows.
        self.index_rows();
        let (reads, index) = (self.reads, &mut *self.index);
        index.now += 1;
        index.part.clear();
        for &ri in rules {
            let r = index.row_of[ri];
            if index.seen[r as usize] != index.now {
                index.seen[r as usize] = index.now;
                index.part.push(r);
            }
        }
        debug_assert_eq!(
            index
                .part
                .iter()
                .map(|&r| index.size[r as usize])
                .sum::<usize>(),
            rules.len(),
            "a part is made of whole rows"
        );
        let facts = self
            .distinct
            .count(index.part.iter().flat_map(|&r| index.facts(r)).copied());
        let copies = self.distinct.count(
            index
                .part
                .iter()
                .flat_map(|&r| reads[index.first[r as usize]].iter())
                .filter(|&&(s, _, copied)| copied && !path.contains(&s))
                .map(|&(_, a, _)| a),
        );
        facts.max(1) * rows + COPY_COST * copies
    }

    /// Whether `rules` are not split on the set `s` and read it in binary
    /// ([`super::Bits`]): a set of [`BITS_FROM`] values or more that two
    /// units or more read ([`wide_and_shared`]), where the unit holds no
    /// switch, as a unit of row classes does, or `rules` read nothing else
    /// through copies. Split on such a set, each part reads a copy of its
    /// value where the table whole reads the set's bits, and reads the other
    /// parents through copies of its own, each with a link that the compile
    /// builds and the evidence walk conjoins. A table whose rows hold
    /// switches and read other parents is still split on it: whole, its
    /// rows' switches would lie on one spine above the copies.
    fn read_in_binary(&self, s: usize, rules: &[usize]) -> bool {
        self.wide_and_shared[s]
            && (self.switch_free
                || rules.iter().all(|&ri| {
                    self.reads[ri]
                        .iter()
                        .all(|&(t, _, copied)| !copied || t == s)
                }))
    }

    /// How to split `rules`, which read one value of each set in `path`,
    /// given what they cost whole where that is known.
    fn plan(&mut self, rules: &[usize], path: &mut Vec<usize>, whole: Option<usize>) -> Plan {
        let top = path.is_empty();
        let whole = whole.unwrap_or_else(|| self.whole(rules, path));
        let mut candidates: Vec<usize> = self.reads[rules[0]]
            .iter()
            .filter(|&&(s, _, copied)| {
                s != usize::MAX && copied && !path.contains(&s) && !self.read_in_binary(s, rules)
            })
            .map(|&(s, _, _)| s)
            .collect();
        candidates.dedup();
        // The parent whose split costs least with its parts whole.
        let mut best: Option<Candidate> = None;
        let mut value_of: Vec<u32> = Vec::with_capacity(rules.len());
        for s in candidates {
            // The rules of each value, the values numbered as they come,
            // not a map from value to part looked up for each rule of each
            // candidate. Each rule's value is numbered first, and the
            // values' rules counted, and each value's list is made at its
            // size once every rule reads one value, not grown a rule at a
            // time for candidates a later rule rules out.
            let mut sizes: Vec<(AtomId, usize)> = Vec::new();
            value_of.clear();
            self.distinct.start();
            // A rule's reads are sorted, those of `s` one run, found by a
            // search and not a pass over all of them.
            let every = rules.iter().all(|&ri| {
                let read = &self.reads[ri];
                let at = read.partition_point(|&(t, _, _)| t < s);
                match (read.get(at), read.get(at + 1)) {
                    (Some(&(t, a, true)), next)
                        if t == s && next.is_none_or(|&(u, _, _)| u != s) =>
                    {
                        let j = self.distinct.number(a, sizes.len());
                        if j == sizes.len() {
                            sizes.push((a, 0));
                        }
                        sizes[j].1 += 1;
                        value_of.push(j as u32);
                        true
                    }
                    _ => false,
                }
            });
            if !every || sizes.len() < 2 {
                continue;
            }
            let mut values: Vec<(AtomId, Vec<usize>)> = sizes
                .iter()
                .map(|&(a, size)| (a, Vec::with_capacity(size)))
                .collect();
            for (&ri, &j) in rules.iter().zip(&value_of) {
                values[j as usize].1.push(ri);
            }
            let k = values.len();
            let own_value = top
                && self.own_value
                && self.exclusive[s].len() >= OWN_VALUE_FROM
                && !self.shared(&values);
            let mut over = if own_value {
                k * (usize::BITS - k.leading_zeros()) as usize
            } else {
                k * k
            };
            if !top {
                over += COPY_COST * k;
            }
            // The parts' costs, summed until they reach the best's: a
            // candidate that does loses, and the rest of its parts need
            // not be counted.
            // Below a whole unit, the parts' costs whole are kept for the
            // parts' own plans: the candidate kept summed them all, and
            // each would be worked out again there.
            path.push(s);
            let mut flat = over;
            let mut wholes = Vec::new();
            for (_, rules) in &values {
                if best.as_ref().is_some_and(|b| flat >= b.flat) {
                    break;
                }
                flat += if top {
                    self.rows(rules.len()).pow(2)
                } else {
                    let whole = self.whole(rules, path);
                    wholes.push(whole);
                    whole
                };
            }
            path.pop();
            if best.as_ref().is_none_or(|b| flat < b.flat) {
                best = Some(Candidate {
                    flat,
                    set: s,
                    own_value,
                    over,
                    values,
                    wholes,
                });
            }
        }
        let Some(Candidate {
            flat,
            set,
            own_value,
            over,
            values,
            wholes,
        }) = best
        else {
            return Plan {
                cost: whole,
                split: None,
            };
        };
        // A whole unit is split where the split pays with its parts whole.
        if top && SPLIT_GAIN * flat > whole {
            return Plan {
                cost: whole,
                split: None,
            };
        }
        debug_assert!(
            top || wholes.len() == values.len(),
            "the kept candidate summed every part"
        );
        path.push(set);
        let parts: Vec<(AtomId, Vec<usize>, Plan)> = values
            .into_iter()
            .enumerate()
            .map(|(k, (a, rules))| {
                let plan = self.plan(&rules, path, wholes.get(k).copied());
                (a, rules, plan)
            })
            .collect();
        path.pop();
        let cost = over + parts.iter().map(|(_, _, plan)| plan.cost).sum::<usize>();
        if top || SPLIT_GAIN * cost <= whole {
            Plan {
                cost,
                split: Some(Split {
                    set,
                    own_value,
                    parts,
                }),
            }
        } else {
            Plan {
                cost: whole,
                split: None,
            }
        }
    }
}

/// The unit of each derived atom: the merged components of
/// [`crate::ground::deps::components`], which join the components that share a
/// fact and then the ones with the same dependencies and consumers, joined
/// again where they hold atoms of one exclusive set. A unit reads another's
/// atom through an indicator of its own ([`introduce`]). The unit is meant
/// to be a Bayesian network variable: the values of a table with switches
/// share them, and the values of a deterministic table, which share no fact
/// and may feed different children, are exclusive. Reading the parents
/// through one copy for the whole table is what keeps the table's switches
/// above that copy and nothing else in the vtree; a variable split in two
/// would read its parents through two copies, and link each to them
/// twice. Facts have no unit.
///
/// [`introduce`]: super::introduce
pub(super) fn units(base_of: &[usize], exclusive: &[Vec<AtomId>]) -> Vec<usize> {
    let num_base = base_of
        .iter()
        .filter(|&&u| u != usize::MAX)
        .map(|&u| u + 1)
        .max()
        .unwrap_or(0);
    let mut parent: Vec<usize> = (0..num_base).collect();
    for set in exclusive {
        let first = base_of[set[0] as usize];
        for &a in &set[1..] {
            union(&mut parent, first, base_of[a as usize]);
        }
    }
    base_of
        .iter()
        .map(|&u| {
            if u == usize::MAX {
                u
            } else {
                find(&mut parent, u)
            }
        })
        .collect()
}
