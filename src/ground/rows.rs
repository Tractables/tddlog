//! The rows of a table that exclude each other share the parameters they
//! hold in common: a whole distribution, as a row class, or one switch.
//!
//! Normalization gives every row of an annotated disjunction its own chain
//! of switches ([`super::normalize`]). A row is the rules of a table with
//! one context, the literals that are not the table's own facts
//! ([`Tables::owner`]). Where two rows exclude each other, in a world where
//! the one's context holds the other's switches decide nothing, so a switch
//! of the one and a switch of the other with the same probability can be
//! one fact, which carries the weight of whichever decides: Theorem 1 of
//! flip-hoisting (Cheng et al., PROBPROG 2021), the contexts being the
//! inconsistent path conditions.
//!
//! ```text
//! x(v) :- a(1), s1.   x(w) :- a(1), \+s1.        x(v) :- a(1), s1.   x(w) :- a(1), \+s1.
//! x(v) :- a(2), s2.   x(w) :- a(2), \+s2.   →    x(v) :- a(2), s1.   x(w) :- a(2), \+s1.
//! ```
//!
//! where `s1` and `s2` have one probability. Rows whose chains are equal
//! share the whole chain; rows whose chains differ share the switches whose
//! probabilities coincide. One analysis per table finds both ([`share`]):
//!
//! - its rows, and whether every two of them exclude each other
//!   ([`Tables::exclusive`]);
//! - its *classes*: rows whose rules have the same heads over switches of
//!   the same probabilities in the same places, every two of them
//!   exclusive, grown first fit in rule order where the table's rows are
//!   not all exclusive;
//! - its shared switches: the `k`-th switch of probability `p` of each
//!   *unit*, in atom order, is one fact, the first of them. A unit is a
//!   class where the class is named with an atom (below), a row otherwise.
//!   The switches of one unit do not exclude each other, so a unit with two
//!   switches of one probability reads two facts, and a switch read under
//!   two contexts keeps its own fact.
//!
//! A table some atom of which depends on another is left as it is, so that
//! no context depends on a fact shared.
//!
//! What is written depends on the program's reader ([`Target`]). A weighted
//! CNF has no layout: every table whose rows exclude each other shares its
//! switches, the rows of a class all of theirs, and each shared fact is a
//! variable fewer, so that a factor's equal entries are one parameter. No
//! class is named, which would cost the CNF a variable and a clause per
//! row. The compile reads a table through its layout, which makes two
//! decisions, each taken here once: where a class is named with an atom of
//! its own ([`classes_of`]), and where the units share their switches
//! ([`compile_shares`]).
//!
//! # Classes named
//!
//! A table's diagram tells its parents' contexts apart one by one, where
//! many rows repeat a distribution (bn_pathfinder's f39 has a hundred rows
//! per distribution). A named class is an atom `c` with a rule
//! `c :- context` per row, and the table's rules for the class read `c` and
//! the switches of its first row:
//!
//! ```text
//! x(v) :- ctx_1, s.   x(v) :- ctx_2, s'.       c :- ctx_1.   c :- ctx_2.
//! x(w) :- ctx_1, \+s. x(w) :- ctx_2, \+s'.  →  x(v) :- c, s. x(w) :- c, \+s.
//! ```
//!
//! The class atoms of a table are a variable of their own, a deterministic
//! function of the parents, and the table reads it alone: the parents reach
//! the table's readers through one value per class instead of one per
//! context. A table's classes are named where it has fewer than
//! 1/[`CLASS_GAIN`] as many classes as rows and some class has switches: a
//! table whose every class is deterministic has a class per value, and its
//! class atoms would copy its values. A table of one class names one atom
//! whose contexts cover its parents' values: the cover finds it certain
//! ([`super::cover`]), and the table loses its parents (bn_barley's aar_mod
//! and spndx).
//!
//! Nor is a table of one parent, whose rows read atoms of one table only:
//! its classes only group the parent's values, which a split on the parent
//! reads one row at a time already ([`crate::indicators`]); named, such
//! tables (bn_pathfinder's of `fault` alone, dice_digitRecognition's of
//! `y`) cost more work.
//!
//! Nor is a table whose every row reads one fact at most, as a noisy-or
//! over facts does ([`super::noisy_or`]): each of its classes would be a
//! row, its class atom a copy of the fact, and the atoms that read facts
//! alone, in a component where they share facts, would read copies, each
//! in a component of its own. On the UAI 2022 or_chain networks, written
//! as noisy-ors, the class atoms (one per cause of each finding) would
//! turn a few derived atoms in one component into hundreds in many.
//!
//! A table's classes are named too where they, as many as its rows if need
//! be, stand for the rows of long spines. In the `families` vtree a
//! table's switches lie on a spine above its parents' copies, each level
//! with a node for every row still to come and value ([`crate::vtree`]),
//! about r² pairs for r rows; a split on a parent ([`crate::indicators`])
//! leaves a spine per part. No other layout of the switches does better
//! while a row is a conjunction of copies: a node over the switches of
//! several rows but not their copies has a class for every function from
//! those rows to the head's values. A class atom is one literal per row
//! instead, and the table over the class atoms reads one of them per row:
//! it is split on them, its parts read their class alone
//! ([`OWN_VALUE_FROM`]) and go side by side, each above its copy, and the
//! class atoms' own table has no switches. A table's classes are named so
//! where ([`on_spines`]):
//!
//! - it has [`OWN_VALUE_FROM`] classes or more, and its head fewer values
//!   than [`OWN_VALUE_FROM`], for its parts to read their class alone;
//! - a split on the largest parent of which every row reads a value leaves
//!   parts of [`SPINE_ROWS`] rows or more;
//! - its rows read three parents or more;
//! - it has [`MOST_SELECTORS`] classes at most, unless that largest parent
//!   is binary.
//!
//! A split on a parent of v values leaves v spines of r/v rows, about
//! r²/v pairs, and each part is split again ([`crate::indicators`]), so a
//! table with a parent of three values or more costs far less than r² as
//! it is, and its class atoms, nearly a row each where its classes are
//! many, cost more than they save. A split on a binary parent saves at
//! most half of the spine, which the split's pricing refuses: a table whose
//! rows all read a binary parent, and no larger one, keeps its r² spine,
//! and its classes cost about k log k for k classes, at most r log r,
//! however many they are (the UAI 2006 networks BN_115 to BN_125, a class
//! nearly every row). A table no parent of which every row reads is no
//! such spine, and keeps the bound: rows that share no parent, each over
//! facts no other row reads, make a class each, a copy of the row.
//!
//! So named, the evidence of bn_hailfinder, bn_hepar2 and bn_water
//! shrinks. Each bound rules out a loss: a head of many values that another
//! table reads, whose parts read every value's copy (bn_mildew); a table of
//! nearly as many classes as rows whose parents split it (bn_barley's
//! `tkv`); parts of fewer rows (bn_insurance, bn_diabetes, the munin
//! networks); tables of two parents (ObjectDetection_13 ran out of
//! memory).
//!
//! # Switches shared
//!
//! A shared fact changes where the layout puts a table's switches. The
//! parts of a table split on a parent of many values can go side by side,
//! each reading its own copy of the parent's value alone
//! ([`crate::indicators`]); a fact read in two parts keeps them on a spine
//! above the copies ([`crate::vtree`]). So the compile shares a table's
//! switches only where its units
//! - are the values of its one parent, every context one positive atom of
//!   one exclusive set, or its named classes, every two of its rows
//!   exclusive;
//! - are at least [`MIN_ROWS`];
//! - group the rows rather than select them: units that read no fact (a
//!   parent that is a function of other atoms, or classes whose contexts
//!   read none) have at least [`MIN_RULES_PER_ROW`] rules each on average;
//! - merge away at least one in [`MIN_MERGED`] of the table's switches.
//!
//! On tables of several parents whose classes are not named, of few rows,
//! where few switches merge and over selectors, which the layout splits on
//! to lay their rows side by side, sharing costs more than the switches it
//! saves.
//!
//! Choices read under exclusive contexts ([`super::contexts`]) are written
//! before: their facts are the reading table's own, and shared here.

use rustc_hash::FxHashMap;

use super::OWN_VALUE_FROM;
use super::program::Lists;
use super::structure::Structure;
use super::tables::Tables;
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, simplify};
use crate::by_hash::ByHash;
use crate::syntax::{Args, Atom, Name, Term};

/// The compile names a table's classes where its rows hold fewer than one
/// class per this many rows.
const CLASS_GAIN: usize = 2;

/// The fewest rows per part of a table split on its largest parent for
/// which the compile names a table's classes for its spines (see above).
const SPINE_ROWS: usize = 12;

/// The most classes of a table named for its spines unless every row reads
/// a binary parent and no larger one (see above).
const MOST_SELECTORS: usize = 256;

/// The fewest units of a table whose switches the compile shares.
const MIN_ROWS: usize = 32;

/// The compile shares a table's switches where at least one in this many
/// of them is merged away.
const MIN_MERGED: usize = 4;

/// The fewest rules per unit, on average, of units that read no fact where
/// the compile shares a table's switches.
const MIN_RULES_PER_ROW: usize = 2;

/// The reader of the program whose tables [`share`] rewrites.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    /// The compile, through a layout: classes named where the layout gains
    /// and `classes` holds ([`crate::GroundOptions::classes`]), switches
    /// shared where the compile gains.
    Compile { classes: bool },
    /// A weighted CNF ([`crate::cnf`]), without a layout: no class named,
    /// and the switches of every table whose rows exclude each other
    /// shared.
    Cnf,
}

/// What the analysis decided for one table: its classes where they are
/// named, and whether its units share their switches.
struct Plan {
    /// The table's place in [`Rows::tables`].
    k: usize,
    classes: Option<Vec<Vec<usize>>>,
    share: bool,
}

impl Plan {
    /// The row of each unit whose rules are kept: a named class's first row,
    /// or each row of the table.
    fn kept(&self, rows: &Rows) -> Vec<usize> {
        match &self.classes {
            Some(classes) => classes.iter().map(|class| class[0]).collect(),
            None => rows.of_table[self.k].to_vec(),
        }
    }
}

/// The tables' rows: each row's context, the literals of its rules outside
/// its table, end to end from `context_start`, and its rules, in order,
/// rather than two lists allocated per row: a program may have millions of
/// rows (mnist_sum256's rules are a row each). And the tables in the order
/// of their first rows, with their rows in order.
#[derive(Default)]
struct Rows {
    lits: Vec<GLit>,
    context_start: Vec<usize>,
    rules: Lists<usize>,
    tables: Vec<usize>,
    of_table: Lists<usize>,
}

impl Rows {
    fn len(&self) -> usize {
        self.context_start.len() - 1
    }

    fn context(&self, row: usize) -> &[GLit] {
        &self.lits[self.context_start[row]..self.context_start[row + 1]]
    }

    fn rules(&self, row: usize) -> &[usize] {
        &self.rules[row]
    }
}

/// Share the parameters of the rows of `gp`'s tables, `tables`, for
/// `target`, as described above, and compact the program. Returns the
/// tables of `gp` as it is then: `tables` where nothing was rewritten,
/// renumbered where only switches were shared, which leaves the exclusive
/// sets as they were, and found anew where classes were named.
pub(crate) fn share(gp: &mut GroundProgram, mut tables: Tables, target: Target) -> Tables {
    let n = gp.num_atoms();
    let own: Vec<usize> = (0..n)
        .map(|s| tables.owner(gp, s as AtomId).unwrap_or(usize::MAX))
        .collect();
    // The tables with a fact of their own. A table without one has no
    // switch to share, its classes are deterministic, and its rows are not
    // looked for (mnist_sum96's partial sums would be grown into classes
    // for nothing).
    let mut switched = vec![false; tables.sets().len() + n];
    for &t in &own {
        if t != usize::MAX {
            switched[t] = true;
        }
    }
    let rows = rows_of(gp, &tables, &own, &switched);
    let mut scratch = SignatureScratch::default();
    let plans: Vec<Plan> = (0..rows.tables.len())
        .filter_map(|k| plan(gp, &mut tables, &rows, &own, k, target, &mut scratch))
        .collect();
    let merged = merged_switches(gp, &rows, &own, &plans, target);
    let named: Vec<(usize, Vec<Vec<usize>>)> = plans
        .into_iter()
        .filter_map(|plan| Some((rows.tables[plan.k], plan.classes?)))
        .collect();
    if named.is_empty() && merged.is_empty() {
        return tables;
    }
    // The class atoms first, which renumbers the atoms, then the merged
    // switches, each body that reads one sorted again.
    let renumbered = (!named.is_empty()).then(|| rewrite_rows(gp, &rows, &own, &named));
    let new = |a: AtomId| renumbered.as_ref().map_or(a, |map| map[a as usize]);
    let rename: FxHashMap<AtomId, AtomId> = merged
        .into_iter()
        .map(|(from, to)| (new(from), new(to)))
        .collect();
    for r in &mut gp.rules {
        let mut changed = false;
        for l in &mut r.body {
            if let Some(&to) = rename.get(&l.atom()) {
                *l = GLit::new(to, l.is_positive());
                changed = true;
            }
        }
        if changed {
            r.body.sort_unstable();
        }
    }
    let compacted = simplify::compact(gp);
    if named.is_empty() {
        tables.renumbered(gp, &compacted)
    } else {
        Tables::of_structure(gp, Structure::of(gp))
    }
}

/// The analysis of the `k`-th table of `rows`: its classes where `target`
/// names them, and whether its units share their switches. `None` where
/// neither.
fn plan(
    gp: &GroundProgram,
    tables: &mut Tables,
    rows: &Rows,
    own: &[usize],
    k: usize,
    target: Target,
    scratch: &mut SignatureScratch,
) -> Option<Plan> {
    let t = rows.tables[k];
    let of_table = &rows.of_table[k];
    if of_table.len() < 2 {
        return None;
    }
    let (classes, share) = match target {
        Target::Cnf => (None, exclusive(tables, rows, of_table)),
        Target::Compile { classes } => {
            let named = classes
                .then(|| classes_of(gp, tables, rows, own, k, scratch))
                .flatten();
            let share = compile_shares(gp, tables, rows, of_table, named.as_ref());
            (named.map(|(classes, _)| classes), share)
        }
    };
    let share = share && tables.independent(gp, t);
    (share || classes.is_some()).then_some(Plan { k, classes, share })
}

/// Whether every two of the rows `members` exclude each other, their
/// contexts clashing ([`Tables::exclusive`]).
fn exclusive(tables: &Tables, rows: &Rows, members: &[usize]) -> bool {
    tables.exclusive(members.len(), |i| rows.context(members[i]).iter())
}

/// Whether the compile shares the switches of the table of the rows
/// `of_table`, whose classes are `named`, with whether its rows all
/// exclude each other, where they are named (see above).
fn compile_shares(
    gp: &GroundProgram,
    tables: &Tables,
    rows: &Rows,
    of_table: &[usize],
    named: Option<&(Vec<Vec<usize>>, bool)>,
) -> bool {
    match named {
        Some((classes, exclusive)) => {
            *exclusive
                && classes.len() >= MIN_ROWS
                && !selectors(
                    gp,
                    of_table.iter().map(|&row| rows.context(row)),
                    classes.len(),
                )
        }
        None => {
            let parent = |row: usize| match rows.context(row) {
                &[GLit::Pos(a)] => Some(a),
                _ => None,
            };
            let set = parent(of_table[0]).map(|a| tables.of(a));
            of_table.len() >= MIN_ROWS
                && set.is_some()
                && of_table
                    .iter()
                    .all(|&row| parent(row).map(|a| tables.of(a)) == set)
                && !selectors(
                    gp,
                    of_table.iter().flat_map(|&row| {
                        let a = parent(row).expect("a row of one parent");
                        tables.by_head()[a as usize]
                            .iter()
                            .map(|&ri| gp.rules[ri].body.as_slice())
                    }),
                    of_table.len(),
                )
        }
    }
}

/// Whether `units` units whose rules have the bodies `bodies` select their
/// rows rather than group them: no body reads a fact, and they are fewer
/// than [`MIN_RULES_PER_ROW`] per unit.
fn selectors<'a>(
    gp: &GroundProgram,
    bodies: impl Iterator<Item = &'a [GLit]>,
    units: usize,
) -> bool {
    let mut rules = 0;
    for body in bodies {
        if body.iter().any(|l| gp.is_fact(l.atom())) {
            return false;
        }
        rules += 1;
    }
    rules < MIN_RULES_PER_ROW * units
}

/// The switches of the tables that `plans` share, as described above: each
/// fact merged away and the fact it becomes, under the compile only where
/// at least one in [`MIN_MERGED`] of the table's switches is.
fn merged_switches(
    gp: &GroundProgram,
    rows: &Rows,
    own: &[usize],
    plans: &[Plan],
    target: Target,
) -> Vec<(AtomId, AtomId)> {
    const UNREAD: u32 = u32::MAX;
    const MIXED: u32 = u32::MAX - 1;
    // The unit each own fact of a table that shares is read in, MIXED where
    // two read it; a fact only the rows a class drops read is read in none.
    let mut unit_of = vec![UNREAD; gp.num_atoms()];
    let mut unit = 0;
    for plan in plans.iter().filter(|plan| plan.share) {
        let t = rows.tables[plan.k];
        for row in plan.kept(rows) {
            for &ri in rows.rules(row) {
                for l in gp.rules[ri].body.iter() {
                    let s = l.atom() as usize;
                    if own[s] == t {
                        unit_of[s] = if unit_of[s] == UNREAD || unit_of[s] == unit {
                            unit
                        } else {
                            MIXED
                        };
                    }
                }
            }
            unit += 1;
        }
    }
    // How many switches of each probability each unit has had, the fact
    // the k-th of them becomes, and the merges, by table.
    let mut seen: FxHashMap<(u32, (u64, u64)), usize> = FxHashMap::default();
    let mut first: FxHashMap<(usize, (u64, u64), usize), AtomId> = FxHashMap::default();
    let mut merges: FxHashMap<usize, (usize, Vec<(AtomId, AtomId)>)> = FxHashMap::default();
    for (s, &u) in unit_of.iter().enumerate() {
        let AtomKind::Fact(p) = gp.kinds[s] else {
            continue;
        };
        if u == UNREAD {
            continue;
        }
        let t = own[s];
        let (facts, merged) = merges.entry(t).or_default();
        *facts += 1;
        if u == MIXED {
            continue;
        }
        let k = seen.entry((u, p.bits())).or_default();
        let to = *first.entry((t, p.bits(), *k)).or_insert(s as AtomId);
        *k += 1;
        if to != s as AtomId {
            merged.push((s as AtomId, to));
        }
    }
    let mut merges: Vec<_> = merges.into_values().collect();
    merges.retain(|(facts, merged)| target == Target::Cnf || merged.len() * MIN_MERGED >= *facts);
    merges.into_iter().flat_map(|(_, merged)| merged).collect()
}

/// The rows of `gp`'s tables (`tables`) that have a fact of their own
/// (`switched`), `own` the table that owns each fact.
fn rows_of(gp: &GroundProgram, tables: &Tables, own: &[usize], switched: &[bool]) -> Rows {
    // Each table's rows, in the order of their first rules. A row is found
    // by the hash of its table and context ([`ByHash`]), rather than in a
    // map keyed by the table and the context, which costs a context per
    // rule.
    let mut rows = Rows {
        context_start: vec![0],
        ..Rows::default()
    };
    let mut row_of_rule: Vec<usize> = Vec::with_capacity(gp.rules.len());
    let mut row_table: Vec<usize> = Vec::new();
    let mut row_by_hash = ByHash::default();
    // The tables in the order of their first rows, and each table's place
    // in that order.
    let mut table_order: Vec<usize> = Vec::new();
    let mut table_number = vec![usize::MAX; switched.len()];
    let mut context: Vec<GLit> = Vec::new();
    // The table and row of the rule before, and its context as its body
    // reads it. A row's rules mostly follow one another and read their
    // context in one order, and a rule that reads the context of the rule
    // before so is of its row without sorting and looking it up.
    let mut previous = (usize::MAX, usize::MAX);
    let mut as_read: Vec<GLit> = Vec::new();
    for r in &gp.rules {
        use std::hash::BuildHasher;
        let t = tables.of(r.head);
        if !switched[t] {
            row_of_rule.push(usize::MAX);
            continue;
        }
        context.clear();
        context.extend(
            r.body
                .iter()
                .copied()
                .filter(|l| own[l.atom() as usize] != t),
        );
        if previous.0 == t && context == as_read {
            row_of_rule.push(previous.1);
            continue;
        }
        as_read.clone_from(&context);
        context.sort_unstable();
        context.dedup();
        let hash = rustc_hash::FxBuildHasher.hash_one((t, context.as_slice()));
        let (row, new) = row_by_hash.find_or_push(hash, |row| {
            row_table[row as usize] == t && rows.context(row as usize) == context.as_slice()
        });
        let row = row as usize;
        if new {
            rows.lits.extend_from_slice(&context);
            rows.context_start.push(rows.lits.len());
            row_table.push(t);
            if table_number[t] == usize::MAX {
                table_number[t] = table_order.len();
                table_order.push(t);
            }
        }
        previous = (t, row);
        row_of_rule.push(row);
    }
    // Each row's rules in their order.
    rows.rules = Lists::new(rows.len(), || {
        row_of_rule
            .iter()
            .enumerate()
            .filter(|&(_, &row)| row != usize::MAX)
            .map(|(ri, &row)| (row, ri))
    });
    // Each table's rows in their order.
    rows.of_table = Lists::new(table_order.len(), || {
        row_table
            .iter()
            .enumerate()
            .map(|(row, &t)| (table_number[t], row))
    });
    rows.tables = table_order;
    rows
}

/// The classes of the rows of the `k`-th table of `rows`, as described
/// above, where the compile names them, and whether every two of its rows
/// exclude each other; `own` the table that owns each fact.
fn classes_of(
    gp: &GroundProgram,
    tables: &mut Tables,
    rows: &Rows,
    own: &[usize],
    k: usize,
    scratch: &mut SignatureScratch,
) -> Option<(Vec<Vec<usize>>, bool)> {
    let t = rows.tables[k];
    let of_table = &rows.of_table[k];
    let mut parents = of_table
        .iter()
        .flat_map(|&row| rows.context(row).iter().map(|l| tables.of(l.atom())));
    let first = parents.next();
    if of_table.len() <= CLASS_GAIN
        || parents.all(|p| Some(p) == first)
        || !tables.independent(gp, t)
    {
        return None;
    }
    // Nor is a table whose every row reads one fact at most (below).
    if of_table.iter().all(|&row| match rows.context(row) {
        [] => true,
        [l] => gp.is_fact(l.atom()),
        _ => false,
    }) {
        return None;
    }
    // The most classes the table may have.
    let spines = on_spines(rows, of_table, t, tables);
    let few = (of_table.len() - 1) / CLASS_GAIN;
    let most = match spines {
        Spines::No => few,
        Spines::Split => few.max(MOST_SELECTORS),
        Spines::Whole => of_table.len(),
    };
    // A signature per group, of which there are at most `most + 1`
    // before the table is given up, and at most one a row.
    let mut by_signature: FxHashMap<Signature, usize> =
        FxHashMap::with_capacity_and_hasher((most + 1).min(of_table.len()), Default::default());
    let mut role_ids: FxHashMap<Role, u32> = FxHashMap::default();
    let mut groups: Vec<(bool, Vec<usize>)> = Vec::new();
    for &row in of_table {
        // Each group makes a class at least, so a table whose groups
        // come to the bound is given up before its other rows are
        // signed.
        if groups.len() > most {
            return None;
        }
        let Some(switches) = signature(gp, rows.rules(row), own, t, &mut role_ids, scratch) else {
            groups.push((true, vec![row]));
            continue;
        };
        let g = match by_signature.get(scratch.signature.as_slice()) {
            Some(&g) => g,
            None => {
                by_signature.insert(scratch.signature.clone(), groups.len());
                groups.push((switches, Vec::new()));
                groups.len() - 1
            }
        };
        groups[g].1.push(row);
    }
    // Each group makes a class at least, so the table is given up once
    // the classes made and the groups left come to the bound. Where every
    // two rows of the table exclude each other, so do those of each group.
    let all = exclusive(tables, rows, of_table);
    let mut classes: Vec<Vec<usize>> = Vec::new();
    let mut stochastic = false;
    let mut left = groups.len();
    for (switches, members) in groups {
        if classes.len() + left > most {
            return None;
        }
        left -= 1;
        // A group whose rows clash pairwise is one class, which the
        // first fit finds comparing each row with every row before it.
        if all || exclusive(tables, rows, &members) {
            classes.push(members);
            stochastic |= switches;
            continue;
        }
        let first = classes.len();
        for row in members {
            match classes[first..].iter_mut().find(|c| {
                c.iter()
                    .all(|&o| tables.clash(rows.context(row), rows.context(o)))
            }) {
                Some(class) => class.push(row),
                None => {
                    classes.push(vec![row]);
                    stochastic |= switches;
                }
            }
        }
    }
    // The first fit of a group whose rows do not clash pairwise may pass
    // `most` within the group.
    (stochastic
        && classes.len() <= most
        && (classes.len() <= few || (spines != Spines::No && classes.len() >= OWN_VALUE_FROM)))
        .then_some((classes, all))
}

/// `gp` with the rows of each table of `rewrite` read through their
/// classes' atoms, as described above, not yet compacted; `own` the table
/// that owns each fact. Returns the new number of each atom.
fn rewrite_rows(
    gp: &mut GroundProgram,
    rows: &Rows,
    own: &[usize],
    rewrite: &[(usize, Vec<Vec<usize>>)],
) -> Vec<AtomId> {
    let n = gp.num_atoms();
    // The class atoms go before the first atom of their table, and their
    // rules and the table's before the table's first rule.
    let mut before_atom: FxHashMap<AtomId, Vec<Atom>> = FxHashMap::default();
    let mut before_rule: FxHashMap<usize, Vec<GroundRule>> = FxHashMap::default();
    let mut dropped = vec![false; gp.rules.len()];
    let mut fresh = n as AtomId;
    for (seq, &(t, ref classes)) in rewrite.iter().enumerate() {
        // The table's first atom and first rule, found without listing
        // its rules.
        let (mut first_atom, mut first_rule) = (AtomId::MAX, usize::MAX);
        for &row in classes.iter().flatten() {
            for &ri in rows.rules(row) {
                first_atom = first_atom.min(gp.rules[ri].head);
                first_rule = first_rule.min(ri);
            }
        }
        let name: Name = format!("$c_{}_{seq}", gp.atoms[first_atom as usize].name).into();
        let atoms = before_atom.entry(first_atom).or_default();
        let emitted = before_rule.entry(first_rule).or_default();
        // A rule per row, and per rule of each class's first row.
        emitted.reserve(
            classes
                .iter()
                .map(|class| class.len() + rows.rules(class[0]).len())
                .sum(),
        );
        for (k, class) in classes.iter().enumerate() {
            atoms.push(Atom::new(name.clone(), Args::One(Term::Int(k as i64))));
            let c = fresh;
            fresh += 1;
            // The new rules are written in the bodies of the rules they
            // replace, which held their literals and more: the context of
            // a row other than the first in the body of its first rule,
            // and the class and switches of each rule of the first row in
            // that rule's body, rather than allocating a context for each
            // row and freeing the replaced rules' bodies.
            for &row in class {
                let body = match rows.rules(row) {
                    &[ri, ..] if row != class[0] => {
                        let mut body = std::mem::take(&mut gp.rules[ri].body);
                        body.clear();
                        body.extend_from_slice(rows.context(row));
                        body
                    }
                    _ => rows.context(row).to_vec(),
                };
                emitted.push(GroundRule::new(c, body));
                for &ri in rows.rules(row) {
                    dropped[ri] = true;
                }
            }
            for &ri in rows.rules(class[0]) {
                let r = &mut gp.rules[ri];
                let mut body = std::mem::take(&mut r.body);
                body.retain(|l| own[l.atom() as usize] == t);
                body.insert(0, GLit::Pos(c));
                emitted.push(GroundRule::new(r.head, body));
            }
        }
    }
    // Renumber: the atoms in order with the class atoms placed, the fresh
    // ids in the order they were handed out.
    let mut order = Vec::with_capacity(fresh as usize);
    for a in 0..n as AtomId {
        if let Some(placed) = before_atom.remove(&a) {
            for atom in placed {
                order.push(gp.atoms.len() as AtomId);
                gp.atoms.push(atom);
                gp.kinds.push(AtomKind::Derived);
            }
        }
        order.push(a);
    }
    gp.splice_rules(before_rule, &dropped);
    simplify::renumber(gp, &order)
}

/// Whether the compile names a table's classes for its spines ([`on_spines`]).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Spines {
    /// It is not.
    No,
    /// It is, with [`MOST_SELECTORS`] classes at most: a split on its
    /// largest parent, of three values or more, takes its spine apart, or no
    /// parent is read by every row.
    Split,
    /// It is, whatever its classes: every row reads a binary parent and no
    /// larger one, and no split takes its spine apart.
    Whole,
}

/// Whether a table may be rewritten for its spines, as described above,
/// whatever its classes: its head has fewer than [`OWN_VALUE_FROM`]
/// values, its rows read three parents or more, and a split on the largest
/// parent of which every row reads a value positively leaves parts of
/// [`SPINE_ROWS`] rows or more; and whether that parent, binary, leaves its
/// spine whole ([`Spines::Whole`]). `of_table` are its rows.
fn on_spines(rows: &Rows, of_table: &[usize], t: usize, tables: &Tables) -> Spines {
    let sets = tables.sets();
    if of_table.len() < OWN_VALUE_FROM.max(SPINE_ROWS)
        || sets.get(t).is_some_and(|s| s.len() >= OWN_VALUE_FROM)
    {
        return Spines::No;
    }
    // Three parents or more: the first three distinct tables the rows
    // read, rather than every parent of every row listed, sorted and made
    // distinct.
    let mut parents = [usize::MAX; 3];
    let mut found = 0;
    for p in of_table
        .iter()
        .flat_map(|&row| rows.context(row).iter().map(|l| tables.of(l.atom())))
    {
        if !parents[..found].contains(&p) {
            parents[found] = p;
            found += 1;
            if found == parents.len() {
                break;
            }
        }
    }
    if found < parents.len() {
        return Spines::No;
    }
    let read = |row: usize| {
        rows.context(row)
            .iter()
            .filter(|l| l.is_positive())
            .map(|l| tables.of(l.atom()))
            .filter(|&p| p < sets.len())
    };
    let largest = read(of_table[0])
        .filter(|&s| of_table[1..].iter().all(|&row| read(row).any(|p| p == s)))
        .map(|s| sets[s].len())
        .max()
        .unwrap_or(1);
    if of_table.len() / largest < SPINE_ROWS {
        Spines::No
    } else if largest == 2 {
        Spines::Whole
    } else {
        Spines::Split
    }
}

/// A switch's place in its row: its probability and the heads of the rules
/// that read it, with the sign they read it with, sorted; written end to
/// end, the probability's bits and then `head << 1 | positive` for each.
type Role = Vec<u64>;

/// A row's rules as heads over the roles of their switches, each role by
/// its number among the roles of the table's rows, the rules sorted by
/// head and then by their switches, each rule's switches sorted; written
/// end to end, for each rule its head, its number of switches and then
/// `role << 1 | positive` for each.
type Signature = Vec<u64>;

/// The buffers [`signature`] keeps from one row to the next.
#[derive(Default)]
struct SignatureScratch {
    /// Each atom's number among the row's switches, `u32::MAX` for an atom
    /// that is none, and the switches in the order the row's rules first
    /// read them.
    slot: Vec<u32>,
    of_row: Vec<AtomId>,
    /// The literals of each switch, `head << 1 | positive` for each rule
    /// that reads it, end to end from `read_start`, and where each one's
    /// next goes.
    reads: Vec<u64>,
    read_start: Vec<usize>,
    at: Vec<usize>,
    role: Role,
    /// Each switch's role number, by its number in the row, and the
    /// numbers sorted.
    ids: Vec<u32>,
    numbers: Vec<u32>,
    /// Each rule's switches, and per rule its head and their range.
    switches: Vec<(bool, u32)>,
    rules: Vec<(AtomId, usize, usize)>,
    /// The row's signature.
    signature: Signature,
}

/// Whether a row reads switches, its [`Signature`] in `scratch`; `None`
/// where two of its switches have one role, and the row's switches cannot
/// be told apart by their places. `role_ids` numbers the roles of the
/// table's rows as they are first met: two signatures of numbers are equal
/// exactly when their signatures of roles are, whatever order the roles
/// are met in, which spares copying a role for every literal of a switch
/// and allocating per row, per switch and per rule.
fn signature(
    gp: &GroundProgram,
    row_rules: &[usize],
    own: &[usize],
    t: usize,
    role_ids: &mut FxHashMap<Role, u32>,
    scratch: &mut SignatureScratch,
) -> Option<bool> {
    let SignatureScratch {
        slot,
        of_row,
        reads,
        read_start,
        at,
        role,
        ids,
        numbers,
        switches,
        rules,
        signature,
    } = scratch;
    if slot.len() < gp.num_atoms() {
        slot.resize(gp.num_atoms(), u32::MAX);
    }
    // The row's switches numbered as its rules first read them, and their
    // literals by switch: a counting sort.
    of_row.clear();
    read_start.clear();
    read_start.push(0);
    for &ri in row_rules {
        for l in gp.rules[ri]
            .body
            .iter()
            .filter(|l| own[l.atom() as usize] == t)
        {
            let s = l.atom() as usize;
            if slot[s] == u32::MAX {
                slot[s] = of_row.len() as u32;
                of_row.push(l.atom());
                read_start.push(0);
            }
            read_start[slot[s] as usize + 1] += 1;
        }
    }
    for k in 1..read_start.len() {
        read_start[k] += read_start[k - 1];
    }
    reads.clear();
    reads.resize(read_start[of_row.len()], 0);
    at.clear();
    at.extend_from_slice(&read_start[..of_row.len()]);
    for &ri in row_rules {
        let r = &gp.rules[ri];
        for l in r.body.iter().filter(|l| own[l.atom() as usize] == t) {
            let k = slot[l.atom() as usize] as usize;
            reads[at[k]] = u64::from(r.head) << 1 | u64::from(l.is_positive());
            at[k] += 1;
        }
    }
    // Each switch's literals, sorted, are its role. Two switches have one
    // role exactly when they get one number.
    ids.clear();
    for (k, &s) in of_row.iter().enumerate() {
        let AtomKind::Fact(p) = gp.kinds[s as usize] else {
            unreachable!("a table's own atom is a fact")
        };
        let run = &mut reads[read_start[k]..read_start[k + 1]];
        run.sort_unstable();
        role.clear();
        role.push(p.p.to_bits());
        role.push(p.not.to_bits());
        role.extend_from_slice(run);
        let id = match role_ids.get(role.as_slice()) {
            Some(&id) => id,
            None => {
                let id = role_ids.len() as u32;
                role_ids.insert(role.clone(), id);
                id
            }
        };
        ids.push(id);
    }
    numbers.clear();
    numbers.extend_from_slice(ids);
    numbers.sort_unstable();
    let distinct = numbers.windows(2).all(|w| w[0] != w[1]);
    switches.clear();
    rules.clear();
    if distinct {
        for &ri in row_rules {
            let r = &gp.rules[ri];
            let start = switches.len();
            switches.extend(
                r.body
                    .iter()
                    .filter(|l| own[l.atom() as usize] == t)
                    .map(|l| (l.is_positive(), ids[slot[l.atom() as usize] as usize])),
            );
            switches[start..].sort_unstable();
            rules.push((r.head, start, switches.len()));
        }
    }
    for &s in of_row.iter() {
        slot[s as usize] = u32::MAX;
    }
    if !distinct {
        return None;
    }
    rules.sort_unstable_by(|x, y| (x.0, &switches[x.1..x.2]).cmp(&(y.0, &switches[y.1..y.2])));
    signature.clear();
    for &(head, start, end) in rules.iter() {
        signature.push(u64::from(head));
        signature.push((end - start) as u64);
        signature.extend(
            switches[start..end]
                .iter()
                .map(|&(positive, id)| u64::from(id) << 1 | u64::from(positive)),
        );
    }
    Some(!reads.is_empty())
}

#[cfg(test)]
mod tests {
    use super::super::structure::Structure;
    use super::*;

    #[test]
    fn the_structure_after_sharing_is_the_structure_found_anew() {
        // x has a row for each of the 40 values of a, of three
        // distributions between them, so that its switches merge to six
        // facts and the others are dropped; y reads x. The rows are kept
        // apart: as classes they would be three, too few to share.
        let rows = MIN_ROWS + 8;
        let mut source = (1..=rows)
            .map(|i| format!("{}::a({i})", 1.0 / rows as f64))
            .collect::<Vec<_>>()
            .join("; ")
            + ".\n";
        for i in 1..=rows {
            let [u, v, w] = [[0.2, 0.3, 0.5], [0.1, 0.6, 0.3], [0.4, 0.4, 0.2]][i % 3];
            source += &format!("{u}::x(u); {v}::x(v); {w}::x(w) :- a({i}).\n");
        }
        source += "0.2::y :- x(u). 0.7::y :- x(v). query(y). query(x(u)).";
        let program = crate::syntax::parse_program(&source).expect("parses");
        let gp = crate::ground::ground(
            &program,
            &crate::GroundOptions {
                classes: false,
                ..crate::GroundOptions::default()
            },
        )
        .expect("grounds");
        let facts = gp.facts().count();
        assert!(
            facts < 2 * rows,
            "{facts} facts: the switches of x were not shared"
        );
        let kept = gp
            .structure
            .get()
            .expect("the grounder keeps the structure");
        assert!(
            **kept == Structure::of(&gp),
            "the structure kept is not the structure found anew"
        );
    }
}
