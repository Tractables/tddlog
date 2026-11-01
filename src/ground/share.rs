//! Switch sharing: the rows of a table that exclude each other read one
//! fact for switches of one probability (local flip-hoisting).
//!
//! Normalization gives every row of an annotated disjunction its own chain
//! of stick-breaking switches ([`super::normalize`]). Where two rows exclude
//! each other, a switch of the one and a switch of the other with the same
//! probability can be one fact: in a world where one row's context holds,
//! the other's does not and its switch decides nothing, so the fact carries
//! the weight of whichever switch decides. This is Theorem 1 of
//! flip-hoisting (Cheng et al., PROBPROG 2021), with the rows' contexts as
//! the inconsistent path conditions.
//!
//! ```text
//! x(v) :- a(1), s1.   x(w) :- a(1), \+s1.        x(v) :- a(1), s1.   x(w) :- a(1), \+s1.
//! x(v) :- a(2), s2.   x(w) :- a(2), \+s2.   →    x(v) :- a(2), s1.   x(w) :- a(2), \+s1.
//! ```
//!
//! where `s1` and `s2` have one probability.
//!
//! A row is the rules of a table with one context, the literals that are not
//! the table's own facts ([`Tables::owner`]). A table shares its switches
//! where
//! - every context is one positive atom, and all of them are atoms of one
//!   exclusive set: the table has one parent, whose values, or whose row
//!   classes ([`super::classes`]), are the rows, and every two rows exclude
//!   each other;
//! - it has at least [`MIN_ROWS`] rows;
//! - where its parent is a function of other atoms, its rules reading no
//!   fact, the parent has at least [`MIN_RULES_PER_ROW`] rules per row of
//!   the table: row classes that group the rows of a table of several
//!   parents, not selectors of about one row each;
//! - at least one in [`MIN_MERGED`] of its switches is merged away;
//! - no atom of the table depends on an atom of the table, so the contexts
//!   do not depend on the facts shared.
//!
//! The `k`-th switch of probability `p` of each row, in atom order, becomes
//! one fact, the first of them: the switches of one row do not exclude each
//! other, so a row with two switches of one probability reads two facts. A
//! switch read under two contexts keeps its own fact.
//!
//! A shared fact also changes where the layout puts a table's switches.
//! The parts of a table split on a parent of many values can go side by
//! side, each reading its own copy of the parent's value alone
//! ([`crate::indicators`]); a fact read in two parts keeps them on a spine
//! above the copies ([`crate::vtree`]). On tables of several parents, on
//! tables of few rows, where few of a table's switches merge and on tables
//! over selectors, which the layout splits on to lay their rows side by
//! side, that cost more than the switches saved.

use rustc_hash::{FxHashMap, FxHashSet};

use super::tables::Tables;
use super::{AtomId, AtomKind, GLit, GroundProgram, simplify};

/// The fewest rows of a table that share switches.
const MIN_ROWS: usize = 32;

/// A table shares its switches where at least one in this many of them is
/// merged away.
const MIN_MERGED: usize = 4;

/// The fewest rules per row of the table that a parent which reads no fact
/// has where the table shares its switches.
const MIN_RULES_PER_ROW: usize = 2;

/// Whether the atoms `rows`, a table's parent, read no fact and have fewer
/// than [`MIN_RULES_PER_ROW`] rules per atom.
fn selectors(gp: &GroundProgram, tables: &Tables, rows: &[AtomId]) -> bool {
    let mut rules = 0;
    for &a in rows {
        for &ri in &tables.by_head()[a as usize] {
            if gp.rules[ri].body.iter().any(|l| gp.is_fact(l.atom())) {
                return false;
            }
            rules += 1;
        }
    }
    rules < MIN_RULES_PER_ROW * rows.len()
}

/// What a table's rules say of its rows: the one exclusive set every
/// context is an atom of, while there is one, and the atoms that are its
/// rows.
#[derive(Default)]
struct Shape {
    parent: Option<usize>,
    one_parent: bool,
    rows: Vec<AtomId>,
}

/// Merge the switches that can be shared, as described above, and compact
/// the program. `tables` are `gp`'s. Returns the new number of each atom
/// where a fact was merged away ([`simplify::compact`]), for the tables to
/// be renumbered ([`Tables::renumbered`]); `gp` is unchanged if none was.
pub(crate) fn share_switches(gp: &mut GroundProgram, tables: &mut Tables) -> Option<Vec<AtomId>> {
    const NONE: usize = usize::MAX;
    const UNREAD: AtomId = AtomId::MAX;
    const MIXED: AtomId = AtomId::MAX - 1;
    let n = gp.num_atoms();
    let own: Vec<usize> = (0..n)
        .map(|s| tables.owner(gp, s as AtomId).unwrap_or(NONE))
        .collect();
    let mut shapes: FxHashMap<usize, Shape> = FxHashMap::default();
    for &t in own.iter().filter(|&&t| t != NONE) {
        shapes.entry(t).or_insert_with(|| Shape {
            one_parent: true,
            ..Shape::default()
        });
    }
    if shapes.is_empty() {
        return None;
    }
    // The context each own fact is read under, MIXED if under two.
    let mut row_of: Vec<AtomId> = vec![UNREAD; n];
    for r in &gp.rules {
        let t = tables.of(r.head);
        let Some(shape) = shapes.get_mut(&t) else {
            continue;
        };
        let mut context = r.body.iter().filter(|l| own[l.atom() as usize] != t);
        let (Some(&GLit::Pos(a)), None) = (context.next(), context.next()) else {
            shape.one_parent = false;
            continue;
        };
        let set = tables.of(a);
        if shape.parent.is_some_and(|p| p != set) {
            shape.one_parent = false;
        }
        shape.parent = Some(set);
        shape.rows.push(a);
        for l in r.body.iter().filter(|l| own[l.atom() as usize] == t) {
            let row = &mut row_of[l.atom() as usize];
            *row = if *row == UNREAD || *row == a {
                a
            } else {
                MIXED
            };
        }
    }
    // A table other than an exclusive set has one atom, so the rows of a
    // table of one parent and MIN_ROWS rows are values of one set, and
    // every two of them exclude each other.
    let mut shared: FxHashSet<usize> = FxHashSet::default();
    let mut tabs: Vec<usize> = shapes.keys().copied().collect();
    tabs.sort_unstable();
    for t in tabs {
        let shape = shapes.get_mut(&t).expect("a table with own facts");
        shape.rows.sort_unstable();
        shape.rows.dedup();
        if shape.one_parent
            && shape.rows.len() >= MIN_ROWS
            && !selectors(gp, tables, &shape.rows)
            && tables.independent(gp, t)
        {
            shared.insert(t);
        }
    }
    // How many switches of each probability each row has had, the fact
    // the k-th of them becomes, and the merges, by table.
    let mut seen: FxHashMap<(usize, AtomId, (u64, u64)), usize> = FxHashMap::default();
    let mut first: FxHashMap<(usize, (u64, u64), usize), AtomId> = FxHashMap::default();
    let mut merges: FxHashMap<usize, (usize, Vec<(AtomId, AtomId)>)> = FxHashMap::default();
    for s in 0..n {
        let (t, row) = (own[s], row_of[s]);
        let AtomKind::Fact(p) = gp.kinds[s] else {
            continue;
        };
        if !shared.contains(&t) {
            continue;
        }
        let (facts, merged) = merges.entry(t).or_default();
        *facts += 1;
        if row == MIXED || row == UNREAD {
            continue;
        }
        let k = seen.entry((t, row, p.bits())).or_default();
        let to = *first.entry((t, p.bits(), *k)).or_insert(s as AtomId);
        *k += 1;
        if to != s as AtomId {
            merged.push((s as AtomId, to));
        }
    }
    let rename: FxHashMap<AtomId, AtomId> = merges
        .into_values()
        .filter(|(facts, merged)| merged.len() * MIN_MERGED >= *facts)
        .flat_map(|(_, merged)| merged)
        .collect();
    if rename.is_empty() {
        return None;
    }
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
    Some(simplify::compact(gp))
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
