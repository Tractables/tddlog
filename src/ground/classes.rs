//! Row classes: the rows of a table that hold one distribution read it
//! through an atom of their own.
//!
//! Normalization gives every row of an annotated disjunction its own chain
//! of switches ([`super::normalize`]), and a table's diagram then tells its
//! parents' contexts apart one by one, where many rows repeat a
//! distribution (bn_pathfinder's f39 has a hundred rows per distribution).
//! Where the rows of one class exclude each other, the class is an atom
//! `c` with a rule `c :- context` per row, and the table's rules for the
//! class read `c` and the switches of its first row:
//!
//! ```text
//! x(v) :- ctx_1, s.   x(v) :- ctx_2, s'.       c :- ctx_1.   c :- ctx_2.
//! x(w) :- ctx_1, \+s. x(w) :- ctx_2, \+s'.  →  x(v) :- c, s. x(w) :- c, \+s.
//! ```
//!
//! In a world where one row's context holds, no other row of its class
//! holds, so the one chain decides as the row's own did and every atom's
//! distribution is unchanged (the argument for one switch shared by rows
//! that exclude each other, for a whole chain). The class atoms of a table are a variable of their own, a
//! deterministic function of the parents, and the table reads it alone: the
//! parents reach the table's readers through one value per class instead of
//! one per context.
//!
//! A row is the rules of a table with one context, the literals that are
//! not the table's own facts ([`Tables::owner`]). Rows are of one class when
//! their rules have the same heads over switches of the same probabilities
//! in the same places, and their contexts pairwise clash; the classes are
//! grown first fit in rule order. A table is rewritten where no atom of it
//! depends on another, it has fewer than 1/[`CLASS_GAIN`] as many classes
//! as rows, and some class has switches: a table whose every class is
//! deterministic has a class per value, and its class atoms would copy its
//! values.
//!
//! Nor is a table of one parent, whose rows read atoms of one table only:
//! its classes only group the parent's values, which a split on the parent
//! reads one row at a time already ([`crate::indicators`]); rewritten,
//! such tables (bn_pathfinder's of `fault` alone, dice_digitRecognition's
//! of `y`) cost more work.
//!
//! Nor is a table whose every row reads one fact at most, as a noisy-or
//! over facts does ([`super::noisy_or`]): each of its classes would be a
//! row, its class atom a copy of the fact, and the atoms that read facts
//! alone, in a component where they share facts, would read copies, each
//! in a component of its own. On the UAI 2022 or_chain networks, written
//! as noisy-ors, the class atoms (one per cause of each finding) would
//! turn a few derived atoms in one component into hundreds in many.
//!
//! A table is rewritten too where its classes, as many as its rows if need
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
//! class atoms' own table has no switches. A table is rewritten so where
//! ([`on_spines`]):
//!
//! - it has [`OWN_VALUE_FROM`] classes or more, and at most
//!   [`MOST_SELECTORS`], and its head fewer values than
//!   [`OWN_VALUE_FROM`], for its parts to read their class alone;
//! - a split on the largest parent of which every row reads a value leaves
//!   parts of [`SPINE_ROWS`] rows or more;
//! - its rows read three parents or more.
//!
//! So rewritten, the evidence of bn_hailfinder, bn_hepar2 and bn_water
//! shrinks. Each bound rules out a loss: a head of many values that another
//! table reads, whose parts read every value's copy (bn_mildew); a table of
//! nearly as many classes as rows (bn_barley's `tkv`); parts of fewer rows
//! (bn_insurance, bn_diabetes, the munin networks); tables of two parents
//! (ObjectDetection_13 ran out of memory).

use rustc_hash::FxHashMap;

use super::OWN_VALUE_FROM;
use super::program::Lists;
use super::tables::Tables;
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, simplify};
use crate::by_hash::ByHash;
use crate::syntax::{Args, Atom, Name, Term};

/// A table is rewritten where its rows hold fewer than one class per this
/// many rows.
const CLASS_GAIN: usize = 2;

/// The fewest rows per part of a table split on its largest parent for
/// which a table is rewritten for its spines (see above).
const SPINE_ROWS: usize = 12;

/// The most classes of a table rewritten for its spines (see above).
const MOST_SELECTORS: usize = 256;

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

/// Rewrite the tables whose rows fall into few classes, as described above,
/// and compact the program. `tables` are `gp`'s. Returns whether a table
/// was rewritten, for the tables to be found again; `gp` is unchanged if
/// none was.
pub(crate) fn row_classes(gp: &mut GroundProgram, tables: &mut Tables) -> bool {
    let n = gp.num_atoms();
    let own: Vec<usize> = (0..n)
        .map(|s| tables.owner(gp, s as AtomId).unwrap_or(usize::MAX))
        .collect();
    // The tables with a fact of their own. A table without one reads no
    // switch, so its classes are deterministic and it is not rewritten,
    // and its rows are not looked for (mnist_sum96's partial sums would be
    // grown into classes for nothing).
    let mut switched = vec![false; tables.sets().len() + n];
    for &t in &own {
        if t != usize::MAX {
            switched[t] = true;
        }
    }
    let rows = rows_of(gp, tables, &own, &switched);
    // The rows' classes, per table rewritten.
    let mut scratch = SignatureScratch::default();
    let mut selectors: Vec<AtomId> = Vec::new();
    let rewrite: Vec<(usize, Vec<Vec<usize>>)> = (0..rows.tables.len())
        .filter_map(|k| {
            let classes = classes_of(gp, tables, &rows, &own, k, &mut scratch, &mut selectors)?;
            Some((rows.tables[k], classes))
        })
        .collect();
    if rewrite.is_empty() {
        return false;
    }
    rewrite_rows(gp, &rows, &own, &rewrite);
    true
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
/// above, where the table is rewritten; `own` the table that owns each fact.
fn classes_of(
    gp: &GroundProgram,
    tables: &mut Tables,
    rows: &Rows,
    own: &[usize],
    k: usize,
    scratch: &mut SignatureScratch,
    selectors: &mut Vec<AtomId>,
) -> Option<Vec<Vec<usize>>> {
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
    let most = if spines { few.max(MOST_SELECTORS) } else { few };
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
    // the classes made and the groups left come to the bound.
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
        if clash_pairwise(rows, &members, tables, selectors) {
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
    (stochastic
        && (classes.len() <= few
            || (spines && (OWN_VALUE_FROM..=MOST_SELECTORS).contains(&classes.len()))))
    .then_some(classes)
}

/// `gp` with the rows of each table of `rewrite` read through their
/// classes' atoms, as described above, and compacted; `own` the table that
/// owns each fact.
fn rewrite_rows(
    gp: &mut GroundProgram,
    rows: &Rows,
    own: &[usize],
    rewrite: &[(usize, Vec<Vec<usize>>)],
) {
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
    let old = std::mem::take(&mut gp.rules);
    let mut rules = Vec::with_capacity(old.len());
    for (ri, r) in old.into_iter().enumerate() {
        if let Some(emitted) = before_rule.remove(&ri) {
            rules.extend(emitted);
        }
        if !dropped[ri] {
            rules.push(r);
        }
    }
    gp.rules = rules;
    simplify::renumber(gp, &order);
    simplify::compact(gp);
}

/// Whether a table may be rewritten for its spines, as described above,
/// whatever its classes: its head has fewer than [`OWN_VALUE_FROM`]
/// values, its rows read three parents or more, and a split on the largest
/// parent of which every row reads a value positively leaves parts of
/// [`SPINE_ROWS`] rows or more. `of_table` are its rows.
fn on_spines(rows: &Rows, of_table: &[usize], t: usize, tables: &Tables) -> bool {
    let sets = tables.sets();
    if of_table.len() < OWN_VALUE_FROM.max(SPINE_ROWS)
        || sets.get(t).is_some_and(|s| s.len() >= OWN_VALUE_FROM)
    {
        return false;
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
        return false;
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
    of_table.len() / largest >= SPINE_ROWS
}

/// Whether the contexts of the rows `members` clash pairwise, as
/// [`Tables::clash`] tells, because each reads one atom of every one of the
/// same exclusive sets, positively, and no two read the same atoms of them.
/// One row clashes with none and makes a class of its own. `atoms` is a
/// buffer, the atoms each row reads of the sets in their order.
fn clash_pairwise(
    rows: &Rows,
    members: &[usize],
    tables: &Tables,
    atoms: &mut Vec<AtomId>,
) -> bool {
    if members.len() == 1 {
        return true;
    }
    let num_sets = tables.sets().len();
    let read = |row: usize| {
        rows.context(row)
            .iter()
            .filter(|l| l.is_positive() && tables.of(l.atom()) < num_sets)
            .map(|l| (tables.of(l.atom()), l.atom()))
    };
    // The sets the first row reads, sorted, which every row must read one
    // atom each of.
    let mut sets: Vec<usize> = read(members[0]).map(|(s, _)| s).collect();
    sets.sort_unstable();
    let w = sets.len();
    if w == 0 || sets.windows(2).any(|p| p[0] == p[1]) {
        return false;
    }
    atoms.clear();
    // Whether each row's context is the atoms it reads of the sets alone:
    // the rows of a table have distinct contexts, and then distinct atoms.
    let mut only_sets = true;
    for &row in members {
        let start = atoms.len();
        atoms.resize(start + w, AtomId::MAX);
        for (s, a) in read(row) {
            match sets.binary_search(&s) {
                Ok(k) if atoms[start + k] == AtomId::MAX => atoms[start + k] = a,
                _ => return false,
            }
        }
        if atoms[start..].contains(&AtomId::MAX) {
            return false;
        }
        only_sets &= rows.context(row).len() == w;
    }
    // Otherwise two rows may read the same atoms, which a sort tells; the
    // case above spares most groups of rows the sort.
    if only_sets {
        return true;
    }
    let mut order: Vec<usize> = (0..members.len()).collect();
    order.sort_unstable_by(|&x, &y| atoms[x * w..(x + 1) * w].cmp(&atoms[y * w..(y + 1) * w]));
    order
        .windows(2)
        .all(|p| atoms[p[0] * w..(p[0] + 1) * w] != atoms[p[1] * w..(p[1] + 1) * w])
}

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
