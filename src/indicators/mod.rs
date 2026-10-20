//! Indicator variables for derived atoms.
//!
//! Tp-compilation composes every derived atom away: the formula of an atom is
//! over the facts alone, and an atom read by two components has its whole
//! formula conjoined into both. Where the readers sit apart in the vtree that
//! formula is a function of facts on both sides of a cut, and the diagram
//! pays for every way the facts on one side can leave it open. A variable
//! standing for the atom cuts that: the readers read the variable, and one
//! constraint ties the variable to the formula.
//!
//! [`introduce`] rewrites a ground program so that every atom `k` a rule of
//! another component reads is read by each other unit (`units`) through a
//! fresh fact
//! `v_{k,u}` of its own, and adds for each a derived atom `d_{k,u}` with the
//! rules
//!
//! ```text
//! d_{k,u} :- v_{k,u}, k.
//! d_{k,u} :- \+v_{k,u}, \+k.
//! ```
//!
//! which the program then observes true. The atom `k` itself keeps its rules;
//! its formula is now over facts and the indicators of the atoms it reads.
//! In a world of the facts the observed links admit exactly one value of the
//! indicators, the atoms' truth values there, so with weight one on both
//! sides of an indicator the weighted count of `q ∧ e ∧ ⋀ d` is the
//! probability of `q ∧ e` in the program as grounded, and the query phase's
//! ratio of two counts is `P(q | e)` unchanged. A copy per reading unit
//! rather than one per atom lets the vtree put each copy next to its
//! reader, and the link between the copy and the atom's own family.
//!
//! Where the atom read is one of a set of which at most one is true in any
//! world (the exclusive sets of `ground::exclusive`, the values of a
//! Bayesian network variable), a
//! rule reading it positively also reads the other members' copies
//! negatively, which the links make redundant and which keeps the formula
//! of a table from separating rows that two values of one parent would
//! switch on together.
//!
//! A table whose every rule reads one value of a parent positively is split
//! on that parent where the split costs at most half the rows squared
//! (`table_splits`): the rules for each value read the other parents
//! through copies `v_{k,u,p}` of their own, and the links of an atom's
//! copies after the first part's tie each to the copy before, so that one
//! link holds the atom's formula. A part is split again on another parent
//! where that costs less, and a copy's name holds the values on the way to
//! its part. Where the parent the whole unit is split on has
//! `OWN_VALUE_FROM` values or more and the unit is one table, the rules
//! of a part read the parent's copy of their value alone, without the
//! others' negated.
//!
//! A set of `BITS_FROM` values or more that a unit or part reads only
//! positively, and not as its own value, it reads in binary (`Bits`):
//! through copies of ⌈log₂(k + 1)⌉ derived bits of the set's unit, each the
//! disjunction of the values whose code has it, instead of a copy per
//! value. A value is read as the cube of its code, which excludes the other
//! values without reading them, and the code 0 is none of them, as a row
//! of a table that does not sum to one leaves it. A bit's rules read the
//! sets they read one copy per value without the other copies negated:
//! the bit's formula holds the at-most-one of those copies instead.
//!
//! The link is between components and not inside one. The formula of an
//! atom of a cyclic component is the component's least fixpoint as a
//! function of what the component reads, which the indicators stand for
//! exactly; the atoms of one cycle keep reading each other's formulas, since
//! Clark's completion of a cycle admits models the least fixpoint does not.
//!
//! A table whose rows provably partition its inputs (`clause_tables`) is
//! tied to a reader's copies by its clauses instead. One link per table and reader has as rules the table's rows of
//! each value with the reader's copy of the value false, and the worlds
//! where the reader's copies are not exactly one, and the program observes
//! it false. Each reader's link also rules out the worlds where the
//! copies of one input's set are not exactly one, so a row reads a set's
//! copy of its value without the other copies negated, unless another rule
//! reads the set's copies. The compiler builds a
//! conjunction of clauses, one at a time and with no negation, and never
//! the formula of a table's atom, which reads its first reader's copy.
//!
//! The rewrite is a program like any other: an indicator is a fact and a
//! link a derived atom the program observes, and every answer follows from
//! the rules, the evidence and the weights ([`crate::eval::Probabilities`]).
//! The rewrite records what it made in the program's
//! [`Encoding`](crate::ground::Encoding), which the later stages read to
//! do better with it: the `families` layout places each copy by its reader,
//! the compile ties the readers' links of one table and holds sibling
//! groups in its loops of clauses, and the query phase reads a query of an
//! atom with a copy as a literal of the evidence.

mod splits;
mod tables;

use std::fmt::Write as _;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::cubes::{CubeMerge, Merged};
use crate::ground::{
    AtomId, AtomKind, BITS_FROM, Code, GLit, GroundProgram, GroundRule, Indicator, Prob,
};
use crate::syntax::{Atom, Name, Term};
use splits::{Parts, table_splits, units};
use tables::{clause_tables, not_one, outside_codes};

/// The name of atom `a` of `atoms` as a constant, written into `text` the
/// first time, rather than a string of its own per name, and kept in
/// `names`.
fn name_of(names: &mut [Option<Name>], text: &mut String, atoms: &[Atom], a: AtomId) -> Name {
    names[a as usize]
        .get_or_insert_with(|| {
            text.clear();
            write!(text, "{}", atoms[a as usize]).expect("a string takes any text");
            Name::from(text.as_str())
        })
        .clone()
}

/// The links of `gp` that hold a table's clauses (`clause_tables`): the
/// links it observes false.
pub fn clause_links(gp: &GroundProgram) -> FxHashSet<AtomId> {
    let links = gp.encoding.links();
    gp.evidence
        .iter()
        .filter_map(|&(_, a, value)| a.filter(|a| !value && links.contains(a)))
        .collect()
}

/// `atom` of `gp` given one copy for all its readers, which the caller has
/// them read: a fact `$v(atom)` of weight one on both sides and a link
/// `$d(atom)` tied to the atom ([`tie_copy`]), as [`name`] gives a copy of an
/// atom outside the exclusive sets to each unit that reads it. Returns the
/// copy. `decision`'s layout names its contexts, the atoms its tree's
/// readers outside it read and the atoms they read beside those so
/// ([`crate::vtree`]).
pub(crate) fn name_once(gp: &mut GroundProgram, atom: AtomId) -> AtomId {
    let arg = vec![Term::Const(Name::from(gp.atoms[atom as usize].to_string()))];
    let var = gp.atoms.len() as AtomId;
    gp.atoms.push(Atom::new("$v", arg.clone()));
    gp.kinds.push(AtomKind::Fact(Prob::HALF));
    let link = gp.atoms.len() as AtomId;
    gp.atoms.push(Atom::new("$d", arg));
    gp.kinds.push(AtomKind::Derived);
    tie_copy(gp, link, var, atom, std::iter::empty());
    gp.encoding.indicators.push(Indicator { atom, var, link });
    var
}

/// The rules of `link`, which ties the copy `var` to `tied`, and its
/// observation: the link is true where both are, with the copies that
/// `others` negates false, and where neither is.
fn tie_copy(
    gp: &mut GroundProgram,
    link: AtomId,
    var: AtomId,
    tied: AtomId,
    others: impl Iterator<Item = GLit>,
) {
    let mut positive = vec![GLit::Pos(var), GLit::Pos(tied)];
    positive.extend(others);
    gp.rules.push(GroundRule::new(link, positive));
    gp.rules
        .push(GroundRule::new(link, vec![GLit::Neg(var), GLit::Neg(tied)]));
    gp.evidence
        .push((gp.atoms[link as usize].clone(), Some(link), true));
}

/// `original` with every atom a rule of another component reads read
/// through indicators, their links observed after the program's own
/// evidence, in the order the atoms' components are compiled. Where the
/// rows of a table provably partition its inputs (`clause_tables`) and are
/// few (`MAX_CLAUSE_RULES`, `MAX_CLAUSE_CELLS`), the links of a reader's
/// copies of its values hold the table's clauses, observed false, and the
/// table's rules are not compiled; every other copy has a link true exactly
/// when the copy agrees with the value's formula, which the compiler builds
/// from the table's rules.
///
/// # Panics
///
/// On a program that already has indicators: a second rewrite would read
/// the first's links through indicators of their own (`Kind::rewrite`
/// compiles such a program as it is).
///
/// Clauses rather than links to every table's rules: the same answers,
/// and the networks (bn_link) run faster.
///
/// The rewrite's rules are written anew, each body from the rule's in
/// `original`, so a caller that keeps `original` (`auto`, which rewrites
/// the program as grounded for each of its trials) pays no copy.
pub fn introduce(original: &GroundProgram) -> GroundProgram {
    let structure = original.structure();
    name(original, units(&structure.base_of, &structure.sets)).0
}

/// `original` with every derived atom that a rule of another unit reads
/// read through indicators, as [`introduce`] reads it with the program's
/// families for units; and the unit that reads each indicator, in the
/// order of `encoding.indicators`. `unit_of` is each derived atom's unit
/// (`usize::MAX` for a fact), and holds the atoms of a cyclic component in
/// one unit. The `program` layouts name the atoms that cross a narrow cut
/// of their order with the segments between the cuts for units
/// ([`crate::vtree`]'s naming).
pub(crate) fn name(original: &GroundProgram, unit_of: Vec<usize>) -> (GroundProgram, Vec<usize>) {
    assert!(
        original.encoding.is_empty(),
        "the program already reads atoms through indicators"
    );
    let n = original.num_atoms();
    let structure = original.structure();
    let (sccs, exclusive) = (&structure.sccs, &structure.sets);
    // The other units whose rules read each derived atom. A unit's rules
    // come one after another, and a reader the list ends with is not
    // listed again, rather than listed once per rule and sorted.
    let mut readers: Vec<Vec<usize>> = vec![Vec::new(); n];
    for r in &original.rules {
        let reader = unit_of[r.head as usize];
        for l in &r.body {
            let b = l.atom() as usize;
            if original.kinds[b] == AtomKind::Derived
                && unit_of[b] != reader
                && readers[b].last() != Some(&reader)
            {
                readers[b].push(reader);
            }
        }
    }
    for rs in &mut readers {
        rs.sort_unstable();
        rs.dedup();
    }
    let mut set_of = vec![usize::MAX; n];
    for (i, set) in exclusive.iter().enumerate() {
        for &k in set {
            set_of[k as usize] = i;
        }
    }
    let mut is_chosen: Vec<bool> = readers.iter().map(|rs| !rs.is_empty()).collect();
    let chosen: Vec<AtomId> = sccs
        .iter()
        .flatten()
        .copied()
        .filter(|&a| is_chosen[a as usize])
        .collect();
    // Which copy a rule reads an atom through: the reader's unit, and for a
    // split table ([`table_splits`]) the part the rule is in, except for a
    // split parent, which the part split on it reads through one copy.
    let parts = table_splits(original, &unit_of, exclusive, &set_of, &is_chosen);
    let mut unit_of = unit_of;
    // The sets a unit and part read in binary ([`Bits`]): their bit atoms
    // and the rules that define them, copies of the values' rules.
    let bits = Bits::choose(original, &unit_of, exclusive, &set_of, &is_chosen, &parts);
    let mut rank: Vec<(usize, usize)> = vec![(usize::MAX, 0); n];
    for (i, &k) in chosen.iter().enumerate() {
        rank[k as usize] = (i, 0);
    }
    let mut bit_atoms: Vec<Vec<AtomId>> = vec![Vec::new(); exclusive.len()];
    let mut place = vec![u32::MAX; n];
    for set in exclusive {
        for (i, &k) in set.iter().enumerate() {
            place[k as usize] = i as u32;
        }
    }
    // The rewrite, which has the program's atoms and gains the bits,
    // copies and links, and whose rules are written below.
    let mut gp = original.without_rules();
    // The set of each bit atom.
    let mut bit_of: Vec<usize> = vec![usize::MAX; n];
    let first_bit = gp.atoms.len();
    // The program's atoms' names, as the constants that the bits, copies
    // and links name them by, each written once rather than for each copy
    // and each part it is in.
    let mut names: Vec<Option<Name>> = vec![None; n];
    let mut text = String::new();
    for &s in &bits.sets {
        let set = &exclusive[s];
        for j in 0..Bits::width(set.len()) {
            let b = gp.atoms.len() as AtomId;
            let name = name_of(&mut names, &mut text, &gp.atoms, set[0]);
            gp.atoms.push(Atom::new(
                "$b",
                vec![Term::Const(name), Term::Int(j as i64)],
            ));
            gp.kinds.push(AtomKind::Derived);
            unit_of.push(unit_of[set[0] as usize]);
            set_of.push(usize::MAX);
            is_chosen.push(true);
            rank.push((rank[set[0] as usize].0, j + 1));
            bit_of.push(s);
            bit_atoms[s].push(b);
        }
        gp.encoding.codes.push(Code {
            values: set.clone(),
            bits: bit_atoms[s].clone(),
        });
    }
    // A bit's rules are the rows of the values whose code has it, read
    // through copies as their rows are and written after them, each bit's
    // by the rows' numbers here.
    let originals = original.rules.len();
    let (unit_of, set_of, is_chosen, bit_of) = (unit_of, set_of, is_chosen, bit_of);
    let coded = |head: AtomId| {
        set_of[head as usize] != usize::MAX && !bit_atoms[set_of[head as usize]].is_empty()
    };
    let mut rows_of_bit: Vec<Vec<usize>> = vec![Vec::new(); gp.atoms.len() - first_bit];
    for (ri, r) in original.rules.iter().enumerate() {
        if coded(r.head) {
            let code = place[r.head as usize] as usize + 1;
            for (j, &b) in bit_atoms[set_of[r.head as usize]].iter().enumerate() {
                if code >> j & 1 == 1 {
                    rows_of_bit[b as usize - first_bit].push(ri);
                }
            }
        }
    }
    let n = gp.num_atoms();
    // The unit and part reading `b` in binary for rule `ri`, if one does:
    // none where `b`'s set has no bits, as most have, told before the rule
    // is read.
    let binary = |ri: usize, b: AtomId| -> Option<(usize, usize, PartId)> {
        let s = set_of[b as usize];
        if s == usize::MAX || bit_atoms[s].is_empty() {
            return None;
        }
        let u = unit_of[original.rules[ri].head as usize];
        if unit_of[b as usize] == u {
            return None;
        }
        let key = (s, u, parts.reading(parts.of_rule[ri], s).0);
        bits.read.contains(&key).then_some(key)
    };
    let copy_key = |ri: usize, b: AtomId| -> (AtomId, usize, PartId) {
        (
            b,
            unit_of[original.rules[ri].head as usize],
            parts.reading(parts.of_rule[ri], set_of[b as usize]).0,
        )
    };
    // Each key once: a unit's rules read its parents' atoms over and over,
    // and no reading is sorted. The unit and part of the key last met for
    // each atom are kept beside the set,
    // and a key the same as that one is not looked up: the rules of a unit
    // come one after another and read the same atoms.
    let mut keys: Vec<(AtomId, usize, PartId)> = Vec::new();
    let mut seen_key: FxHashSet<(AtomId, usize, PartId)> = FxHashSet::default();
    let mut last_key: Vec<(usize, PartId)> = vec![(usize::MAX, WHOLE); n];
    for (ri, r) in original.rules.iter().enumerate() {
        for l in &r.body {
            let b = l.atom();
            if let Some((s, u, p)) = binary(ri, b) {
                // A set's bits are read all together or not at all.
                if seen_key.insert((bit_atoms[s][0], u, p)) {
                    keys.extend(bit_atoms[s].iter().map(|&bit| (bit, u, p)));
                }
            } else if is_chosen[b as usize] && unit_of[b as usize] != unit_of[r.head as usize] {
                let key = copy_key(ri, b);
                if last_key[b as usize] != (key.1, key.2) {
                    last_key[b as usize] = (key.1, key.2);
                    if seen_key.insert(key) {
                        keys.push(key);
                    }
                }
            }
        }
    }
    // The parts of a split table in their order, that in which its rules
    // first read them and the `families` vtree lays them out, and not by
    // the split parent's atoms: the copies of an atom in consecutive parts
    // are tied, and a chain across the layout keeps copies of many parts
    // open at once (bn_pathfinder's tables split on `fault`, with their
    // parts renumbered, ran out of memory so).
    // A bit ranks after its set's first value. Where no other unit reads
    // that value, the bits of two sets rank alike, and the atom decides.
    keys.sort_unstable_by_key(|&(k, u, p)| (rank[k as usize], u, p, k));
    // A unit's name, for the indicators it reads: its first atom's.
    let mut unit_name: FxHashMap<usize, Name> = FxHashMap::default();
    for &a in sccs.iter().flatten() {
        unit_name
            .entry(unit_of[a as usize])
            .or_insert_with(|| name_of(&mut names, &mut text, &gp.atoms, a));
    }
    let mut copy: FxHashMap<(AtomId, usize, PartId), AtomId> = FxHashMap::default();
    // The copies of a set's atoms that a unit and part read, in the set's
    // order, one list for each, through which the rules read the copies of
    // their parents' values, in place of a lookup of each copy read and
    // another of its siblings.
    let mut set_lists: Vec<Vec<Option<AtomId>>> = Vec::new();
    let mut set_list: FxHashMap<(usize, usize, PartId), usize> = FxHashMap::default();
    // The copies in order, each with the atom it stands for and its name's
    // arguments; their links are made once the tables are known.
    let mut copies: Vec<(AtomId, AtomId, Vec<Term>)> = Vec::with_capacity(keys.len());
    // What each link ties its copy to: the atom, or for the copies of one
    // atom in the parts of a split table after the first, the part before's
    // copy. The first part's link holds the atom's formula; the others are
    // an equality of two variables each, which the query phase conjoins
    // along the table's spine rather than a copy of that formula per part.
    let mut tie: Vec<AtomId> = Vec::with_capacity(keys.len());
    let mut last: Option<(AtomId, usize, AtomId)> = None;
    let var_name: Name = "$v".into();
    names.resize(gp.atoms.len(), None);
    // The values a part splits on, from the part up.
    let mut values: Vec<AtomId> = Vec::new();
    for &(k, u, p) in &keys {
        values.clear();
        let mut q = p;
        while q != WHOLE {
            values.push(parts.value[q as usize]);
            q = parts.up[q as usize];
        }
        // The atom, the reader's unit and the values from the top part down.
        let mut args = Vec::with_capacity(2 + values.len());
        args.push(Term::Const(name_of(&mut names, &mut text, &gp.atoms, k)));
        args.push(Term::Const(unit_name[&u].clone()));
        for &v in values.iter().rev() {
            args.push(Term::Const(name_of(&mut names, &mut text, &gp.atoms, v)));
        }
        let var = gp.atoms.len() as AtomId;
        assert_eq!(
            var as usize,
            n + copies.len(),
            "the copies are numbered after the program's atoms, in the keys' order"
        );
        gp.atoms.push(Atom::new(var_name.clone(), args.clone()));
        // The probability is never read: the weights of an indicator are one
        // and one.
        gp.kinds.push(AtomKind::Fact(Prob::HALF));
        copy.insert((k, u, p), var);
        let s = set_of[k as usize];
        if s != usize::MAX {
            let i = *set_list.entry((s, u, p)).or_insert_with(|| {
                set_lists.push(vec![None; exclusive[s].len()]);
                set_lists.len() - 1
            });
            set_lists[i][place[k as usize] as usize] = Some(var);
        }
        copies.push((k, var, args));
        tie.push(match last {
            Some((j, w, before)) if j == k && w == u => before,
            _ => k,
        });
        last = Some((k, u, var));
    }
    // A rule that reads one atom of an exclusive set positively reads, with
    // it, that the unit's copies of the set's other atoms are false. Where
    // the links hold this is the same function, since at most one atom of
    // the set is true. Where they do not, the rule no longer fires on two
    // copies of one set at once, which is what would make the formula of a
    // table tell apart the rows two values of one parent switch on
    // together. As literals of the body rather than a constraint beside it,
    // the negations are split on with the rest of the body ([`crate::tp`]'s
    // factoring), so that no intermediate formula of the table is built
    // without them. The copies of each set that a unit and part read are
    // recorded as a group of siblings. A bit's loops of clauses hold their
    // at-most-one instead ([`crate::tp`]), since only the bit's link reads
    // the bit, so a bit's rules read the positive literal alone (with the
    // negations, mnist_sum96's bits' rows would be about half negations).
    let mut bodies: Vec<Vec<GLit>> = Vec::with_capacity(original.rules.len());
    // Each list's copies negated, in order, end to end, and where each copy
    // is among them: a rule that reads a value reads the negations of the
    // others as the runs before and after its own, copied, rather than
    // filtered from the set's atoms one by one.
    let mut negated: Vec<GLit> = Vec::new();
    let mut negated_ends: Vec<usize> = Vec::with_capacity(set_lists.len() + 1);
    let mut negated_at: Vec<usize> = vec![usize::MAX; copies.len()];
    negated_ends.push(0);
    for list in &set_lists {
        for &var in list.iter().flatten() {
            negated_at[var as usize - n] = negated.len();
            negated.push(GLit::Neg(var));
        }
        negated_ends.push(negated.len());
    }
    // The same of each row of a value read in binary, as its bits' rules
    // read it, empty for the other rules.
    let mut bit_bodies: Vec<Vec<GLit>> = vec![Vec::new(); originals];
    let mut body: Vec<GLit> = Vec::new();
    // The body each literal was last kept for, which drops its repeats as
    // they come: a search of the kept literals for each would be quadratic
    // in a body that reads every copy of a parent's other values. A list's
    // copies are read through the list alone, so the copies a value's
    // read negates, most of the literals written, are distinct from the
    // body's other literals where the body reads no other value of the
    // list, and are kept without a look; a body that reads two values of
    // a list is looked through whole.
    let mut kept_for: Vec<usize> = vec![usize::MAX; 2 * gp.atoms.len()];
    let mut list_read: Vec<usize> = vec![usize::MAX; set_lists.len()];
    let mut written = 0;
    // Only a chosen atom has copies. A set's list and whether a rule reads
    // its own value of the set ([`Parts::reading`]) depend on the rule's
    // unit and part alone, and are found once in each run of rules of one
    // unit and part, stamped with the run's turn.
    let mut turn = 0u32;
    let mut context: Option<(usize, PartId)> = None;
    let mut read_at: Vec<(u32, Option<usize>, bool)> = vec![(0, None, false); exclusive.len()];
    // The lists a rule reads a value of positively, other than its own
    // value of a split parent: their copies are siblings.
    let mut sibling = vec![false; set_lists.len()];
    // The copies of a set's bits that a unit and part read, looked up once.
    let mut bit_copies: FxHashMap<(usize, usize, PartId), Vec<AtomId>> = FxHashMap::default();
    for (ri, r) in original.rules.iter().enumerate() {
        let u = unit_of[r.head as usize];
        if context != Some((u, parts.of_rule[ri])) {
            context = Some((u, parts.of_rule[ri]));
            turn += 1;
        }
        for as_bit in [false, true] {
            if as_bit && !coded(r.head) {
                break;
            }
            body.clear();
            written += 1;
            let mut twice = false;
            for &l in &r.body {
                let b = l.atom();
                if let Some(key) = binary(ri, b) {
                    let code = place[b as usize] as usize + 1;
                    let (s, u, p) = key;
                    let vars = bit_copies.entry(key).or_insert_with(|| {
                        bit_atoms[s].iter().map(|&bit| copy[&(bit, u, p)]).collect()
                    });
                    for (j, &var) in vars.iter().enumerate() {
                        keep_new(
                            &mut body,
                            &mut kept_for,
                            written,
                            if code >> j & 1 == 1 {
                                GLit::Pos(var)
                            } else {
                                GLit::Neg(var)
                            },
                        );
                    }
                    continue;
                }
                if !is_chosen[b as usize] {
                    keep_new(&mut body, &mut kept_for, written, l);
                    continue;
                }
                let s = set_of[b as usize];
                if s == usize::MAX {
                    let l = match copy.get(&copy_key(ri, b)) {
                        Some(&var) if l.is_positive() => GLit::Pos(var),
                        Some(&var) => GLit::Neg(var),
                        None => l,
                    };
                    keep_new(&mut body, &mut kept_for, written, l);
                    continue;
                }
                let at = &mut read_at[s];
                if at.0 != turn {
                    let (p, own_value) = parts.reading(parts.of_rule[ri], s);
                    *at = (turn, set_list.get(&(s, u, p)).copied(), own_value);
                }
                let (_, list, own_value) = *at;
                let Some((i, var)) =
                    list.and_then(|i| set_lists[i][place[b as usize] as usize].map(|var| (i, var)))
                else {
                    keep_new(&mut body, &mut kept_for, written, l);
                    continue;
                };
                twice |= std::mem::replace(&mut list_read[i], written) == written;
                if !l.is_positive() {
                    keep_new(&mut body, &mut kept_for, written, GLit::Neg(var));
                    continue;
                }
                keep_new(&mut body, &mut kept_for, written, GLit::Pos(var));
                if !own_value {
                    sibling[i] = true;
                    if !as_bit {
                        let at = negated_at[var as usize - n];
                        body.extend_from_slice(&negated[negated_ends[i]..at]);
                        body.extend_from_slice(&negated[at + 1..negated_ends[i + 1]]);
                    }
                }
            }
            if twice {
                written += 1;
                body.retain(|&l| {
                    std::mem::replace(
                        &mut kept_for[2 * l.atom() as usize + l.is_positive() as usize],
                        written,
                    ) != written
                });
            }
            let seen = body.clone();
            if as_bit {
                bit_bodies[ri] = seen;
            } else {
                bodies.push(seen);
            }
        }
    }
    gp.rules = original
        .rules
        .iter()
        .zip(bodies)
        .map(|(r, body)| GroundRule::new(r.head, body))
        .collect();
    // A bit's rules are the rows of the values whose code has it, and two
    // that differ only in one bit of the code of a value they read in
    // binary are one rule without it, merged as a clause loop merges its
    // cubes ([`crate::tp`]), before the layout and the compile read them
    // all (mnist_sum96's rules shrink twentyfold). Merged on any literal, two
    // rows of a table's last two values, whose switch chains differ in the
    // sign of the last switch alone, would lose it, and the families layout
    // reads the rows: bn_mildew's compile then costs more.
    if !bits.sets.is_empty() {
        let code: Vec<bool> = (0..gp.atoms.len())
            .map(|a| a >= n && bit_of[keys[a - n].0 as usize] != usize::MAX)
            .collect();
        let in_code = |a: AtomId| code[a as usize];
        let mut merge = BitMerge::new(gp.atoms.len());
        let mut merged: Vec<Option<Vec<Option<Vec<GLit>>>>> = rows_of_bit
            .iter()
            .map(|rows| merge.of(rows, &bit_bodies, &in_code))
            .collect();
        let mut next = vec![0usize; rows_of_bit.len()];
        for (ri, bit_body) in bit_bodies.iter().enumerate() {
            let head = gp.rules[ri].head;
            if !coded(head) {
                continue;
            }
            let code = place[head as usize] as usize + 1;
            for (j, &b) in bit_atoms[set_of[head as usize]].iter().enumerate() {
                if code >> j & 1 == 1 {
                    let i = b as usize - first_bit;
                    let k = next[i];
                    next[i] += 1;
                    match &mut merged[i] {
                        None => gp.rules.push(GroundRule::new(b, bit_body.clone())),
                        Some(at) => gp
                            .rules
                            .extend(at[k].take().map(|body| GroundRule::new(b, body))),
                    }
                }
            }
        }
    }
    gp.encoding.siblings = set_lists
        .iter()
        .zip(sibling)
        .filter(|&(_, read)| read)
        .map(|(members, _)| {
            let mut group: Vec<AtomId> = members.iter().flatten().copied().collect();
            group.sort_unstable();
            group
        })
        .filter(|group| group.len() >= 2)
        .collect();
    gp.encoding.siblings.sort_unstable();
    let copy_of = CopyOf {
        first: n,
        keys: &keys,
    };
    let sets = Sets {
        exclusive,
        set_of: &set_of,
        bit_of: &bit_of,
        bit_atoms: &bit_atoms,
    };
    let (tables, table) = clause_tables(&gp, &sets, &copy, &copy_of, &copies, &tie);
    // The copies a table's clauses define: the table, the reader and the
    // value's place in the set.
    let mut clause_copy: FxHashMap<usize, (usize, usize, usize)> = FxHashMap::default();
    let mut table_of = vec![usize::MAX; n];
    let mut fires = vec![false; gp.rules.len()];
    for (t, table) in tables.iter().enumerate() {
        for &k in &exclusive[table.set] {
            table_of[k as usize] = t;
        }
        for &ri in &table.rules {
            fires[ri] = true;
        }
        for (r, reader) in table.readers.iter().enumerate() {
            for (j, &i) in reader.iter().enumerate() {
                clause_copy.insert(i, (t, r, j));
            }
        }
    }
    // One link per copy, and one per table and reader for the copies its
    // clauses define.
    let mut reader_link: Vec<Vec<Option<AtomId>>> =
        tables.iter().map(|t| vec![None; t.readers.len()]).collect();
    let (clause_name, link_name): (Name, Name) = ("$c".into(), "$d".into());
    for (i, (k, var, args)) in copies.into_iter().enumerate() {
        let (name, slot) = match clause_copy.get(&i) {
            Some(&(t, r, _)) => (clause_name.clone(), Some((t, r))),
            None => (link_name.clone(), None),
        };
        let link = match slot.and_then(|(t, r)| reader_link[t][r]) {
            Some(link) => link,
            None => {
                let link = gp.atoms.len() as AtomId;
                gp.atoms.push(Atom::new(name, args));
                gp.kinds.push(AtomKind::Derived);
                if let Some((t, r)) = slot {
                    reader_link[t][r] = Some(link);
                }
                link
            }
        };
        gp.encoding
            .indicators
            .push(Indicator { atom: k, var, link });
    }
    // The inputs of each table: the sets, units and parts its rows read
    // copies of, each copy taken once per table: every row reads its
    // parents' copies, which would otherwise be sorted per row.
    let mut inputs: Vec<Vec<(usize, usize, PartId)>> = vec![Vec::new(); tables.len()];
    let mut taken_for = vec![usize::MAX; keys.len()];
    for (ri, rule) in gp.rules.iter().enumerate() {
        let t = table_of[rule.head as usize];
        if t != usize::MAX && fires[ri] {
            for l in &rule.body {
                if let Some(i) = (l.atom() as usize)
                    .checked_sub(n)
                    .filter(|&i| i < keys.len())
                    && std::mem::replace(&mut taken_for[i], t) != t
                {
                    let (b, u, p) = keys[i];
                    inputs[t].extend(sets.of(b).map(|s| (s, u, p)));
                }
            }
        }
    }
    for i in &mut inputs {
        i.sort_unstable();
        i.dedup();
    }
    // A table's rules become its readers' clauses: a row of a value with
    // the reader's copy of the value false is a world the clauses rule out,
    // and so is any world where the reader's copies are not exactly one.
    // Each reader's link also rules out the worlds where the copies of one
    // input's set are not exactly one, or more than one where the set is
    // not known to be a table or not every value is read. Both hold in every
    // world the links allow, and without them the evidence carries a region
    // of parent states no world has (bn_andes_ev's query phase grows many
    // times). One reader's would do, but with every reader's the readers'
    // loops are one set of clauses with the copies renamed, which the
    // compiler builds once and copies (the clause memo in `crate::tp`).
    // The exactly-one rules
    // come first: the rows alone leave a class for every set of copies false
    // where the copies are above the switches in the vtree, 2^63 for
    // bn_pathfinder's fault, and after them a class per value.
    //
    // A table's atom that a copy outside the clauses reads, that of a
    // reader that does not read every value, keeps its rules. Read through
    // the first reader's copy instead, its link would tie two copies in
    // different groups, and the query phase conjoins such ties among
    // themselves before the clauses that make them few (bn_pathfinder_ev's
    // readers of fault ran out of memory so).
    let mut read_by_link = vec![false; n];
    for (i, ind) in gp.encoding.indicators.iter().enumerate() {
        if !clause_copy.contains_key(&i) && tie[i] == ind.atom {
            read_by_link[ind.atom as usize] = true;
        }
    }
    // The sets whose copies a rule reads other than a table's row that
    // becomes its readers' clauses, a rule that reads a value's copy with
    // the other copies negated (the expansion above).
    // Each copy so read is marked first: a program's rules may read a few
    // thousand copies millions of times over (mnist_sum96).
    let mut copy_read = vec![false; keys.len()];
    for (ri, rule) in gp.rules.iter().enumerate() {
        if table_of[rule.head as usize] == usize::MAX
            || !fires[ri]
            || read_by_link[rule.head as usize]
        {
            for l in &rule.body {
                if let Some(i) = (l.atom() as usize)
                    .checked_sub(n)
                    .filter(|&i| i < keys.len())
                {
                    copy_read[i] = true;
                }
            }
        }
    }
    let read_elsewhere: FxHashSet<(usize, usize, PartId)> = keys
        .iter()
        .zip(&copy_read)
        .filter(|&(_, &read)| read)
        .map(|(&(b, u, p), _)| (set_of[b as usize], u, p))
        .collect();
    let in_binary = |s: usize, u: usize, p: PartId| {
        bit_atoms[s]
            .first()
            .is_some_and(|&b| copy.contains_key(&(b, u, p)))
    };
    // A row's copy of a value, by its set, unit and part, numbered, where
    // the links hold that set's at-most-one with `not_one` and no other
    // rule reads its copies, `u32::MAX` otherwise; worked out once per
    // copy, where the rows read each copy over and over.
    let mut held_number: FxHashMap<(usize, usize, PartId), u32> = FxHashMap::default();
    let held_copy: Vec<u32> = keys
        .iter()
        .map(|&(b, u, p)| match (set_of[b as usize], u, p) {
            (s, u, p)
                if s != usize::MAX
                    && !in_binary(s, u, p)
                    && !read_elsewhere.contains(&(s, u, p)) =>
            {
                let next = held_number.len() as u32;
                *held_number.entry((s, u, p)).or_insert(next)
            }
            _ => u32::MAX,
        })
        .collect();
    let held = |l: &GLit| {
        (l.atom() as usize)
            .checked_sub(n)
            .and_then(|i| held_copy.get(i).copied())
            .filter(|&k| k != u32::MAX)
    };
    let rules = std::mem::take(&mut gp.rules);
    let mut begun = vec![false; tables.len()];
    // Per set, unit and part held by the links, the last row to read a
    // copy of it, which drops the copies it negates; and the row without
    // them, for one row at a time.
    let mut read_by: Vec<usize> = vec![usize::MAX; held_number.len()];
    let mut row: Vec<GLit> = Vec::new();
    for (ri, rule) in rules.into_iter().enumerate() {
        let t = table_of[rule.head as usize];
        if t == usize::MAX {
            gp.rules.push(rule);
            continue;
        }
        if !fires[ri] {
            continue;
        }
        if !begun[t] {
            begun[t] = true;
            for (r, reader) in tables[t].readers.iter().enumerate() {
                let head = reader_link[t][r].expect("every reader has a link");
                let own: Vec<AtomId> = reader
                    .iter()
                    .map(|&c| gp.encoding.indicators[c].var)
                    .collect();
                not_one(&mut gp.rules, head, &own, true);
                for &(s, u, p) in &inputs[t] {
                    if in_binary(s, u, p) {
                        let own: Vec<AtomId> =
                            bit_atoms[s].iter().map(|&b| copy[&(b, u, p)]).collect();
                        outside_codes(&mut gp.rules, head, &own, exclusive[s].len(), !table[s]);
                        continue;
                    }
                    // The unit and part's copies of the set's atoms, in the
                    // set's order: its list, not a lookup per atom.
                    let members: Vec<AtomId> = set_lists[set_list[&(s, u, p)]]
                        .iter()
                        .flatten()
                        .copied()
                        .collect();
                    let exhaustive = members.len() == exclusive[s].len() && table[s];
                    not_one(&mut gp.rules, head, &members, exhaustive);
                }
            }
        }
        // The links hold the at-most-one of each set the row reads a copy
        // of, so a copy of a set the row reads a value of holds negated
        // wherever the link and the row's value do: the links read the row
        // without it, the same function. The clause loops leave such
        // literals out anyway; the families layout reads the rows, and a row
        // that reads its value's copy alone puts the copy by the row, which
        // shrinks the evidence (bn_barley, bn_diabetes). A set whose copies
        // another rule reads keeps the negations, which that rule reads too:
        // read one way by the links and the other by that rule, the copies
        // would be laid out apart (bn_water_ev).
        for k in rule
            .body
            .iter()
            .filter(|l| l.is_positive())
            .filter_map(held)
        {
            read_by[k as usize] = ri;
        }
        row.clear();
        row.extend(
            rule.body
                .iter()
                .copied()
                .filter(|l| l.is_positive() || held(l).is_none_or(|k| read_by[k as usize] != ri)),
        );
        let j = place[rule.head as usize] as usize;
        for reader in &tables[t].readers {
            let Indicator { var, link, .. } = gp.encoding.indicators[reader[j]];
            let mut body = Vec::with_capacity(row.len() + 1);
            body.extend_from_slice(&row);
            body.push(GLit::Neg(var));
            gp.rules.push(GroundRule::new(link, body));
        }
        if read_by_link[rule.head as usize] {
            gp.rules.push(rule);
        }
    }
    // A link from one part's copy to another's reads, as a rule does, that
    // the part's copies of the other atoms of the set are false: the
    // equalities alone, conjoined across the parts before the table is,
    // leave each value's copies free of the others', a class for every
    // subset of the values at every part.
    for (i, &tied) in tie.iter().enumerate() {
        let Indicator { atom, var, link } = gp.encoding.indicators[i];
        if let Some(&(_, _, j)) = clause_copy.get(&i) {
            if j == 0 {
                gp.evidence
                    .push((gp.atoms[link as usize].clone(), Some(link), false));
            }
            continue;
        }
        // A copy tied to another copy of an atom in a set reads the copies
        // of the set's other atoms false.
        let (set, u, p): (&[AtomId], usize, PartId) =
            if tied != atom && set_of[atom as usize] != usize::MAX {
                let &(_, u, p) = copy_of.get(var).expect("a copy");
                (&exclusive[set_of[atom as usize]], u, p)
            } else {
                (&[], usize::MAX, WHOLE)
            };
        let negated = set
            .iter()
            .filter(|&&c| c != atom)
            .filter_map(|&c| copy.get(&(c, u, p)))
            .map(|&w| GLit::Neg(w));
        tie_copy(&mut gp, link, var, tied, negated);
    }
    // What else reads a table's atoms, its queries and its evidence, reads
    // the first reader's copy, which the clauses make equal to the atom.
    for table in &tables {
        for (j, &k) in exclusive[table.set].iter().enumerate() {
            if !read_by_link[k as usize] {
                gp.rules.push(GroundRule::new(
                    k,
                    vec![GLit::Pos(gp.encoding.indicators[table.readers[0][j]].var)],
                ));
            }
        }
    }
    // The values of a set read in binary that no rule reads need no
    // formula of their own, and one that a query or the evidence names is
    // the cube of its code over a reader's copies of the bits, which the
    // links make equal to the bits: kept, its rules would read its
    // parents' copies (bn_mildew with all of dm_4's values queried ran out
    // of memory so).
    if !bits.sets.is_empty() {
        let mut read = vec![false; gp.atoms.len()];
        for l in gp.rules.iter().flat_map(|r| &r.body) {
            read[l.atom() as usize] = true;
        }
        let mut named = vec![false; gp.atoms.len()];
        for &(_, a, _) in gp.queries.iter().chain(&gp.evidence) {
            if let Some(a) = a {
                named[a as usize] = true;
            }
        }
        // The first reader's copies of each set's bits, in the order of
        // the readers' keys.
        let mut readers: Vec<(usize, usize, PartId)> = bits.read.iter().copied().collect();
        readers.sort_unstable();
        let mut bit_vars: FxHashMap<usize, Vec<AtomId>> = FxHashMap::default();
        for (s, u, p) in readers {
            if !bit_vars.contains_key(&s)
                && let Some(vars) = bit_atoms[s]
                    .iter()
                    .map(|&b| copy.get(&(b, u, p)).copied())
                    .collect::<Option<Vec<AtomId>>>()
            {
                bit_vars.insert(s, vars);
            }
        }
        gp.rules.retain(|r| {
            let s = set_of.get(r.head as usize).copied().unwrap_or(usize::MAX);
            s == usize::MAX
                || bit_atoms[s].is_empty()
                || read[r.head as usize]
                || (named[r.head as usize] && !bit_vars.contains_key(&s))
        });
        for &s in &bits.sets {
            let Some(vars) = bit_vars.get(&s) else {
                continue;
            };
            for (i, &k) in exclusive[s].iter().enumerate() {
                if named[k as usize] && !read[k as usize] {
                    let code = i + 1;
                    let body = vars
                        .iter()
                        .enumerate()
                        .map(|(j, &v)| {
                            if code >> j & 1 == 1 {
                                GLit::Pos(v)
                            } else {
                                GLit::Neg(v)
                            }
                        })
                        .collect();
                    gp.rules.push(GroundRule::new(k, body));
                }
            }
        }
    }
    (gp, keys.iter().map(|&(_, u, _)| u).collect())
}

/// A bit's rules, where two differ in the sign of one literal of the code
/// alone, merged as `cubes::merge_complements` merges cubes, where
/// that leaves at most half of them: each rule left takes the place of the
/// first rule merged into it, with that rule's literals in their order,
/// since the order of the rules is that of the parts of a split table in
/// the families layout, and a rule the merge keeps stays as it is. The
/// atoms of the code that the rows read in both signs are the merge's
/// variables; a row's other literals, each sign of an atom its own, are
/// its context ([`CubeMerge`]). A bit whose rules read an atom of the code
/// in both signs within one rule keeps its rules.
///
/// The families layout reads the rows, and a row that reads all of a code
/// but one bit moves it: bn_mildew's few merges (dm_4's rows of a certain
/// value whose parent's codes differ in the low bit) cost its compile more
/// than they save.
struct BitMerge {
    /// Of each atom, the signs a bit's rows read it in, 1 positive and 2
    /// negative, and its variable's number where it is one.
    signs: Vec<u8>,
    var: Vec<u32>,
    atoms: Vec<AtomId>,
    context: Vec<u32>,
    cubes: CubeMerge,
}

impl BitMerge {
    fn new(num_atoms: usize) -> Self {
        BitMerge {
            signs: vec![0; num_atoms],
            var: vec![u32::MAX; num_atoms],
            atoms: Vec::new(),
            context: Vec::new(),
            cubes: CubeMerge::default(),
        }
    }

    /// The bodies of the rules the rows `rows` of `bodies` merge into, each
    /// at the place of its first row; `None` where they stay as they are.
    fn of(
        &mut self,
        rows: &[usize],
        bodies: &[Vec<GLit>],
        in_code: &impl Fn(AtomId) -> bool,
    ) -> Option<Vec<Option<Vec<GLit>>>> {
        let BitMerge {
            signs,
            var,
            atoms,
            context,
            cubes,
        } = self;
        atoms.clear();
        for l in rows.iter().flat_map(|&ri| &bodies[ri]) {
            if in_code(l.atom()) {
                let s = &mut signs[l.atom() as usize];
                if *s == 0 {
                    atoms.push(l.atom());
                }
                *s |= if l.is_positive() { 1 } else { 2 };
            }
        }
        let mut vars: Vec<AtomId> = atoms
            .iter()
            .copied()
            .filter(|&a| signs[a as usize] == 3)
            .collect();
        vars.sort_unstable();
        for (i, &a) in vars.iter().enumerate() {
            var[a as usize] = i as u32;
        }
        let out = Self::merged(rows, bodies, var, vars.len(), context, cubes);
        for &a in atoms.iter() {
            signs[a as usize] = 0;
            var[a as usize] = u32::MAX;
        }
        out
    }

    fn merged(
        rows: &[usize],
        bodies: &[Vec<GLit>],
        var: &[u32],
        vars: usize,
        context: &mut Vec<u32>,
        cubes: &mut CubeMerge,
    ) -> Option<Vec<Option<Vec<GLit>>>> {
        if vars == 0 {
            return None;
        }
        // Two rules merge only where their contexts are the same, so the
        // rules left are at least as many as the contexts, unless two read
        // the code alone and merge into the empty rule: the rows are given
        // up once their contexts are more than half of them. bn_mildew's
        // rows each read their own switches. The contexts are counted as
        // they are numbered, not hashed and counted in a pass of their own.
        let outside = |l: &&GLit| var[l.atom() as usize] == u32::MAX;
        let mut alone = None;
        cubes.reset(vars);
        for &ri in rows {
            context.clear();
            context.extend(
                bodies[ri]
                    .iter()
                    .filter(outside)
                    .map(|l| 2 * l.atom() + u32::from(l.is_positive())),
            );
            context.sort_unstable();
            let k = cubes.context(context);
            if 2 * cubes.contexts() > rows.len()
                && *alone.get_or_insert_with(|| {
                    rows.iter()
                        .filter(|&&ri| !bodies[ri].iter().any(|l| outside(&l)))
                        .count()
                }) < 2
            {
                return None;
            }
            let lits = bodies[ri]
                .iter()
                .filter(|l| !outside(l))
                .map(|l| 2 * var[l.atom() as usize] + u32::from(l.is_positive()));
            cubes.push(k, lits)?;
        }
        match cubes.merge() {
            Merged::No => None,
            Merged::ToTrue => {
                let mut at = vec![None; rows.len()];
                at[0] = Some(Vec::new());
                Some(at)
            }
            Merged::Yes if 2 * cubes.kept().count() > rows.len() => None,
            Merged::Yes => {
                let mut at = vec![None; rows.len()];
                for (c, o) in cubes.kept() {
                    let body = &bodies[rows[o as usize]];
                    at[o as usize] = Some(
                        body.iter()
                            .copied()
                            .filter(|l| {
                                var[l.atom() as usize] == u32::MAX
                                    || cubes.reads(c, var[l.atom() as usize] as usize)
                            })
                            .collect(),
                    );
                }
                Some(at)
            }
        }
    }
}

/// What each copy stands for, the atom, the reader's unit and the part,
/// by the copy's number: the copies are the atoms after the program's
/// own and the bits of the sets read in binary ([`Bits`]), numbered in the
/// order of their keys ([`introduce`]).
struct CopyOf<'a> {
    first: usize,
    keys: &'a [(AtomId, usize, PartId)],
}

impl CopyOf<'_> {
    fn get(&self, a: AtomId) -> Option<&(AtomId, usize, PartId)> {
        (a as usize)
            .checked_sub(self.first)
            .and_then(|i| self.keys.get(i))
    }
}

/// The sets of values that a unit and part read through bits rather than a
/// copy per value.
///
/// A set of k values is read as ⌈log₂(k + 1)⌉ bits, the code of value i
/// being i + 1 and the code 0 none of them. Its bit j is a derived atom of
/// the set's unit, `$b(x, j)`, whose rules are those of the values whose
/// code has bit j, and a reader reads value i as the cube of its code over
/// its copies of the bits, which exclude every other value without reading
/// their copies negated.
struct Bits {
    /// The sets some unit reads in binary, ascending.
    sets: Vec<usize>,
    /// The sets, units and parts that read in binary.
    read: FxHashSet<(usize, usize, PartId)>,
}

impl Bits {
    /// The bits of a set of `k` values: enough for k + 1 codes.
    fn width(k: usize) -> usize {
        (usize::BITS - k.leading_zeros()) as usize
    }

    /// The sets of [`BITS_FROM`] values or more read in another unit, and
    /// the units and parts that read them: those whose every read of the
    /// set is positive and not of an own-value part ([`table_splits`]),
    /// which reads a single value through a copy of its own.
    fn choose(
        gp: &GroundProgram,
        unit_of: &[usize],
        exclusive: &[Vec<AtomId>],
        set_of: &[usize],
        is_chosen: &[bool],
        parts: &Parts,
    ) -> Bits {
        let wanted = |s: usize| exclusive[s].len() >= BITS_FROM;
        let mut read: FxHashSet<(usize, usize, PartId)> = FxHashSet::default();
        let mut refused: FxHashSet<(usize, usize, PartId)> = FxHashSet::default();
        for (ri, r) in gp.rules.iter().enumerate() {
            let u = unit_of[r.head as usize];
            for l in &r.body {
                let b = l.atom() as usize;
                let s = set_of[b];
                if s == usize::MAX || !is_chosen[b] || unit_of[b] == u || !wanted(s) {
                    continue;
                }
                let (p, own) = parts.reading(parts.of_rule[ri], s);
                read.insert((s, u, p));
                if own || !l.is_positive() {
                    refused.insert((s, u, p));
                }
            }
        }
        read.retain(|key| !refused.contains(key));
        let mut sets: Vec<usize> = read.iter().map(|&(s, _, _)| s).collect();
        sets.sort_unstable();
        sets.dedup();
        Bits { sets, read }
    }
}

/// A part of a split table ([`Parts`]).
type PartId = u32;

/// The part that is every table whole, whose copies a unit reads where it
/// is not split.
const WHOLE: PartId = 0;

/// `l` pushed onto `body` unless `kept_for` marks it kept for the body at
/// hand, numbered `written`, and marked so ([`introduce`]).
fn keep_new(body: &mut Vec<GLit>, kept_for: &mut [usize], written: usize, l: GLit) {
    let kept = &mut kept_for[2 * l.atom() as usize + l.is_positive() as usize];
    if *kept != written {
        *kept = written;
        body.push(l);
    }
}

/// The exclusive sets, and the bits of those read in binary ([`Bits`]).
struct Sets<'a> {
    exclusive: &'a [Vec<AtomId>],
    /// Per atom its set, `usize::MAX` for none.
    set_of: &'a [usize],
    /// Per atom, for a bit atom, its set; `usize::MAX` otherwise.
    bit_of: &'a [usize],
    /// Per set its bit atoms, none where no unit reads it in binary.
    bit_atoms: &'a [Vec<AtomId>],
}

impl Sets<'_> {
    /// The set whose value `a` or a bit of it stands for.
    fn of(&self, a: AtomId) -> Option<usize> {
        match (self.set_of[a as usize], self.bit_of[a as usize]) {
            (usize::MAX, usize::MAX) => None,
            (usize::MAX, s) | (s, _) => Some(s),
        }
    }
}
