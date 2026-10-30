//! The tables whose readers' links hold clauses: which rows provably
//! partition their inputs, and the limits on how many they may be.

use rustc_hash::{FxBuildHasher, FxHashMap};
use std::hash::BuildHasher;

use super::{Bits, Copies, PartId, Sets};
use crate::by_hash::ByHash;
use crate::ground::{AtomId, GLit, GroundProgram, GroundRule};

/// A table whose readers' links hold clauses.
pub(super) struct ClauseTable {
    /// Its exclusive set.
    pub(super) set: usize,
    /// The rules of its atoms whose bodies can hold.
    pub(super) rules: Vec<usize>,
    /// Per reader unit that reads every atom of the set, the copy of each
    /// atom, in the set's order, that would otherwise be tied to the atom's
    /// formula, all of one part; a copy's index is its indicator's. A unit
    /// that reads the set in binary ([`Bits`]) is not a reader.
    pub(super) readers: Vec<Vec<usize>>,
}

/// Rules for `head` whose bodies are the worlds where two of `vars` hold,
/// one rule of pairs ([`GroundRule`]) where they are three or more, and,
/// if `exhaustive`, the world where none does.
pub(super) fn not_one(
    rules: &mut Vec<GroundRule>,
    head: AtomId,
    vars: &[AtomId],
    exhaustive: bool,
) {
    match vars {
        [a, b] => rules.push(GroundRule::new(head, vec![GLit::Pos(*a), GLit::Pos(*b)])),
        [_, _, _, ..] => rules.push(GroundRule {
            pairs: true,
            ..GroundRule::new(head, vars.iter().map(|&w| GLit::Pos(w)).collect())
        }),
        _ => {}
    }
    if exhaustive {
        rules.push(GroundRule::new(
            head,
            vars.iter().map(|&w| GLit::Neg(w)).collect(),
        ));
    }
}

/// Rules for `head` whose bodies are the codes over `bits` (bit j the j-th)
/// of no value of a set of `values` ([`Bits`]), and, unless `none`, the code
/// 0 of none of them: cubes over the high bits, each a run of codes that
/// starts at a multiple of its length.
pub(super) fn outside_codes(
    rules: &mut Vec<GroundRule>,
    head: AtomId,
    bits: &[AtomId],
    values: usize,
    none: bool,
) {
    let outside = |code: usize| code > values || (code == 0 && !none);
    // The codes `prefix << low` up to the next multiple of `1 << low`.
    fn cover(
        rules: &mut Vec<GroundRule>,
        head: AtomId,
        bits: &[AtomId],
        outside: &dyn Fn(usize) -> bool,
        prefix: usize,
        low: usize,
    ) {
        let codes = (prefix << low)..((prefix + 1) << low);
        if !codes.clone().any(outside) {
            return;
        }
        if low == 0 || codes.clone().all(outside) {
            let body = (low..bits.len())
                .map(|j| {
                    if prefix >> (j - low) & 1 == 1 {
                        GLit::Pos(bits[j])
                    } else {
                        GLit::Neg(bits[j])
                    }
                })
                .collect();
            rules.push(GroundRule::new(head, body));
            return;
        }
        cover(rules, head, bits, outside, prefix << 1, low - 1);
        cover(rules, head, bits, outside, prefix << 1 | 1, low - 1);
    }
    cover(rules, head, bits, &outside, 0, bits.len());
}

/// The tables whose links can hold clauses, in the order of their sets, and
/// which sets are tables whether read by a reader of every atom or not.
///
/// A reader `u` of the atoms `x_1 … x_m` of an exclusive set reads them
/// through copies `v_1 … v_m`, each tied to its atom's formula `f_i`, the
/// disjunction of the atom's rule bodies. Where in every world exactly one
/// of all the bodies holds, the ties are the clauses
///
/// ```text
/// ¬body ∨ v_i      for each rule of x_i,
/// ¬v_i ∨ ¬v_j      for i < j:
/// ```
///
/// the world's one body sets its value's copy, and at most one copy is
/// true. Every clause is a rule of the link, `link :- body, \+v_i` or
/// `link :- v_i, v_j`, and the program observes the links false: a
/// conjunction of clauses, which the compiler builds clause by clause
/// ([`crate::tp`]) with no negation, and no formula of the table's atoms.
///
/// The bodies are over switches and the copies of the table's inputs. A
/// world is one of the switches and, for each set and unit and part the
/// bodies read copies of, which of the copies is true: one of them, or
/// none unless the copies are of every atom of a set already found to be
/// a table here. A copy of an atom of no set is true or false. Worlds in
/// which two copies of one set are true need not be covered, since the
/// set's own ties rule them out. The bodies partition the worlds when no
/// two of them hold together and their measures, uniform on each input's
/// states and a half per switch literal, sum to one.
pub(super) fn clause_tables(
    gp: &GroundProgram,
    sets: &Sets,
    copies: &Copies,
) -> (Vec<ClauseTable>, Vec<bool>) {
    // The copy whose link holds each atom's formula, per reader unit, with
    // the part it is of.
    let mut first: FxHashMap<(AtomId, usize), (usize, PartId)> = FxHashMap::default();
    for (i, &(k, u, p)) in copies.keys.iter().enumerate() {
        if copies.tie[i] == k {
            first.insert((k, u), (i, p));
        }
    }
    let Sets {
        exclusive, set_of, ..
    } = *sets;
    // The rules of each set, in order, end to end, not a list per set
    // grown rule by rule.
    let mut rules_from = vec![0usize; exclusive.len() + 1];
    for r in &gp.rules {
        let s = set_of[r.head as usize];
        if s != usize::MAX {
            rules_from[s + 1] += 1;
        }
    }
    for s in 0..exclusive.len() {
        rules_from[s + 1] += rules_from[s];
    }
    let mut rules_of = vec![0usize; rules_from[exclusive.len()]];
    let mut next = rules_from.clone();
    for (ri, r) in gp.rules.iter().enumerate() {
        let s = set_of[r.head as usize];
        if s != usize::MAX {
            rules_of[next[s]] = ri;
            next[s] += 1;
        }
    }
    // A set's input states depend on whether the sets it reads are
    // tables, so the sets are checked until no more become tables.
    let mut table = vec![false; exclusive.len()];
    let mut firing: Vec<Vec<usize>> = vec![Vec::new(); exclusive.len()];
    let mut inputs = Inputs::new(gp.atoms.len(), sets, copies);
    let mut buffers = Buffers::default();
    loop {
        let mut more = false;
        for s in 0..exclusive.len() {
            let rules = &rules_of[rules_from[s]..rules_from[s + 1]];
            if !table[s]
                && let Some(rules) =
                    partition(gp, rules, exclusive, &table, &mut inputs, &mut buffers)
            {
                table[s] = true;
                firing[s] = rules;
                more = true;
            }
        }
        if !more {
            break;
        }
    }
    // The units and parts that read each atom through a copy whose link
    // holds its formula, with the copy, not a scan of all the copies per
    // table, which is quadratic in the tables.
    let mut readers_of: FxHashMap<AtomId, Vec<(usize, usize, PartId)>> = FxHashMap::default();
    for (&(k, u), &(i, p)) in &first {
        readers_of.entry(k).or_default().push((i, u, p));
    }
    let mut tables = Vec::new();
    for (s, set) in exclusive.iter().enumerate().filter(|&(s, _)| table[s]) {
        let mut units = readers_of.remove(&set[0]).unwrap_or_default();
        units.sort_unstable();
        let readers: Vec<Vec<usize>> = units
            .into_iter()
            .filter_map(|(_, u, p)| {
                set.iter()
                    .map(|&k| {
                        first
                            .get(&(k, u))
                            .filter(|&&(_, q)| q == p)
                            .map(|&(i, _)| i)
                    })
                    .collect()
            })
            .collect();
        let rules = firing[s].len();
        if !readers.is_empty()
            && (rules <= MAX_CLAUSE_RULES || rules * set.len() <= MAX_CLAUSE_CELLS)
        {
            tables.push(ClauseTable {
                set: s,
                rules: std::mem::take(&mut firing[s]),
                readers,
            });
        }
    }
    (tables, table)
}

/// The states a rule body leaves an input in ([`partition`]): one, or all
/// but some.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Allowed {
    Only(u32),
    Except(Vec<u32>),
}

impl Allowed {
    fn count(&self, states: u32) -> u32 {
        match self {
            Allowed::Only(_) => 1,
            Allowed::Except(out) => states - out.len() as u32,
        }
    }

    /// Both, or `None` if no state is left.
    fn and(&self, other: &Allowed, states: u32) -> Option<Allowed> {
        let both = match (self, other) {
            (Allowed::Only(a), Allowed::Only(b)) => (a == b).then_some(Allowed::Only(*a))?,
            (Allowed::Only(a), Allowed::Except(out)) | (Allowed::Except(out), Allowed::Only(a)) => {
                (!out.contains(a)).then_some(Allowed::Only(*a))?
            }
            (Allowed::Except(x), Allowed::Except(y)) => {
                let mut out: Vec<u32> = x.iter().chain(y).copied().collect();
                out.sort_unstable();
                out.dedup();
                Allowed::Except(out)
            }
        };
        match both {
            Allowed::Except(out) if out.len() as u32 == states => None,
            Allowed::Except(out) if out.len() as u32 + 1 == states => Some(Allowed::Only(
                (0..states)
                    .find(|x| !out.contains(x))
                    .expect("one state is left"),
            )),
            other => Some(other),
        }
    }

    fn disjoint(&self, other: &Allowed, states: u32) -> bool {
        self.and(other, states).is_none()
    }
}

/// Rows of more than this many distinct inputs that are not each one
/// state of every input are not compared pairwise ([`partition`]).
const MAX_PAIRWISE_ROWS: usize = 2048;

/// A table of more than this many rules keeps links to its rules' formulas;
/// it is still a table to the tables that read it.
/// Each reader's link holds all of a table's rules as clauses, conjoined
/// one at a time, which on a large table read by several units costs
/// several times the formulas (bn_barley's aks_m2).
///
/// The limit is on rules, not rules times readers: a limit on rules times
/// readers holds bn_mildew's dm_3 and dm_4, tables of 100 values with one
/// reader each, as clauses, and dm_4's link is then one formula many times
/// the largest otherwise, whose conjunctions cost far more time than their
/// work units.
pub(super) const MAX_CLAUSE_RULES: usize = 4096;

/// A table of more rules than [`MAX_CLAUSE_RULES`] is held as clauses
/// still where its rules times its values are at most this many. Built
/// level by level, a class of assignments above the reader's copies keeps
/// the residual of each row of every value but the one it sets, so the
/// build grows with the rules times the values: it admits bn_barley's
/// aks_m2, of few values, and not bn_mildew's dm_4, of a hundred, whose
/// build would fall back to conjoining its clauses one at a time.
pub(super) const MAX_CLAUSE_CELLS: usize = 1 << 18;

/// Whether `rows`, distinct, each what it allows of the inputs it reads,
/// partition the product of the inputs they read.
///
/// Where every row fixes one input to a single state, as the rows of a
/// table split on a parent fix the parent ([`table_splits`]), each state
/// must have rows and those rows must partition the inputs they read: the
/// rows of one part leave the other parts' inputs free, and the product
/// over every part's inputs overflows (bn_pathfinder's tables split on
/// fault's 63 values). Otherwise the rows' measures must sum to the
/// product and no two rows may overlap: rows that each fix every input
/// are distinct points, and others are compared pairwise.
///
/// The inputs in `above` were split on by the calls above, and every row
/// fixes them: their entries are passed over, not copied out of each
/// part's rows. `states` gives an input's number of states, as the caller
/// works it out, rather than a map of every input the rows read.
///
/// [`table_splits`]: super::splits::table_splits
fn covers<I: Copy + Ord>(
    rows: &[&[(I, Allowed)]],
    above: &mut Vec<I>,
    states: &impl Fn(I) -> u32,
) -> bool {
    let mut read: Vec<I> = rows
        .iter()
        .flat_map(|row| open(row, above).map(|&(input, _)| input))
        .collect();
    read.sort_unstable();
    read.dedup();
    // Rows that each fix every input read are points of the product, which
    // they partition where they are as many as its points, without
    // copying each row into its part at every split input by input.
    if rows.iter().all(|row| {
        open(row, above).count() == read.len()
            && open(row, above).all(|(_, a)| matches!(a, Allowed::Only(_)))
    }) {
        return read
            .iter()
            .try_fold(1u128, |acc, input| {
                acc.checked_mul(u128::from(states(*input)))
            })
            .is_some_and(|total| total == rows.len() as u128);
    }
    let fixed = |row: &[(I, Allowed)], input: I| {
        row.binary_search_by(|(j, _)| j.cmp(&input))
            .ok()
            .and_then(|k| match row[k].1 {
                Allowed::Only(state) => Some(state),
                Allowed::Except(_) => None,
            })
    };
    if let Some(&split) = read
        .iter()
        .find(|&&input| rows.iter().all(|row| fixed(row, input).is_some()))
    {
        let mut parts: std::collections::BTreeMap<u32, Vec<&[(I, Allowed)]>> =
            std::collections::BTreeMap::new();
        for row in rows {
            let state = fixed(row, split).expect("every row fixes the input");
            parts.entry(state).or_default().push(row);
        }
        if parts.len() != states(split) as usize {
            return false;
        }
        above.push(split);
        let all = parts.values().all(|part| covers(part, above, states));
        above.pop();
        return all;
    }
    let Some(total) = read.iter().try_fold(1u128, |acc, input| {
        acc.checked_mul(u128::from(states(*input)))
    }) else {
        return false;
    };
    let mut sum = 0u128;
    let mut points = true;
    for row in rows {
        let mut measure = 1u128;
        for input in &read {
            let count = match row.binary_search_by(|(j, _)| j.cmp(input)) {
                Ok(k) => row[k].1.count(states(*input)),
                Err(_) => states(*input),
            };
            points &= count == 1;
            let Some(m) = measure.checked_mul(u128::from(count)) else {
                return false;
            };
            measure = m;
        }
        let Some(s) = sum.checked_add(measure) else {
            return false;
        };
        sum = s;
    }
    if sum != total {
        return false;
    }
    if points {
        return true;
    }
    if rows.len() > MAX_PAIRWISE_ROWS {
        return false;
    }
    rows.iter().enumerate().all(|(i, x)| {
        rows[i + 1..].iter().all(|y| {
            open(x, above).any(|(input, a)| {
                y.binary_search_by(|(j, _)| j.cmp(input))
                    .is_ok_and(|k| a.disjoint(&y[k].1, states(*input)))
            })
        })
    })
}

/// The entries of `row` but those of the inputs in `above` ([`covers`]).
fn open<'r, I: PartialEq>(
    row: &'r [(I, Allowed)],
    above: &'r [I],
) -> impl Iterator<Item = &'r (I, Allowed)> {
    row.iter().filter(move |(input, _)| !above.contains(input))
}

/// The inputs of [`partition`], numbered once for every set: a set, or an
/// atom of none, with the unit and part of its copies, in that order. A
/// set's copies in a unit and part are its states in the set's order, and
/// none of them is one more state unless every atom of the set has a copy
/// there and the set is a table. A set read in binary there ([`Bits`]) has
/// a state per value, and none unless it is a table.
struct Inputs {
    /// Per atom, its input and state if it is a copy, or its place in the
    /// code if it is a copy of a bit; `u32::MAX` otherwise.
    of: Vec<(u32, u32)>,
    /// Per input its set, or `usize::MAX` for an atom of none, and how many
    /// of the set's atoms it has a copy or a code of.
    info: Vec<(usize, u32)>,
    /// Per input whether it is read in binary.
    binary: Vec<bool>,
    /// Per input where its states start in `marks`.
    base: Vec<u32>,
    /// Per state the stamp of the last rule to rule it out.
    marks: Vec<usize>,
    /// Per input the stamp of the last rule to read it and where that rule
    /// keeps it.
    slot: Vec<(usize, usize)>,
    /// The stamp of the rule at hand, one more for each rule read.
    stamp: usize,
}

impl Inputs {
    fn new(atoms: usize, sets: &Sets, copies: &Copies) -> Inputs {
        let exclusive = sets.exclusive;
        let copy = &copies.copy;
        let mut keys: Vec<(usize, usize, PartId)> = copies
            .keys
            .iter()
            .map(|&(b, u, p)| match sets.of(b) {
                None => (exclusive.len() + b as usize, u, p),
                Some(s) => (s, u, p),
            })
            .collect();
        keys.sort_unstable();
        keys.dedup();
        let mut of = vec![(u32::MAX, 0); atoms];
        let mut info = Vec::with_capacity(keys.len());
        let mut binary = vec![false; keys.len()];
        let mut base = Vec::with_capacity(keys.len());
        let mut size = 0u32;
        for (i, &(s, u, p)) in keys.iter().enumerate() {
            base.push(size);
            if s >= exclusive.len() {
                of[copy[&((s - exclusive.len()) as AtomId, u, p)] as usize] = (i as u32, 0);
                info.push((usize::MAX, 0));
                size += 2;
                continue;
            }
            if copies.in_binary(&sets.bit_atoms[s], u, p) {
                for (j, &b) in sets.bit_atoms[s].iter().enumerate() {
                    of[copy[&(b, u, p)] as usize] = (i as u32, j as u32);
                }
                binary[i] = true;
                info.push((s, exclusive[s].len() as u32));
                size += exclusive[s].len() as u32 + 1;
                continue;
            }
            let mut present = 0;
            for &c in &exclusive[s] {
                if let Some(&a) = copy.get(&(c, u, p)) {
                    of[a as usize] = (i as u32, present);
                    present += 1;
                }
            }
            info.push((s, present));
            size += present + 1;
        }
        Inputs {
            of,
            info,
            binary,
            base,
            marks: vec![0; size as usize],
            slot: vec![(0, 0); keys.len()],
            stamp: 0,
        }
    }
}

/// What a rule allows of one input it reads: the input, its states, the
/// one state a positive literal names, the states negative literals rule
/// out, and for an input read in binary the bits the rule fixes and their
/// values.
type InputRead = (u32, u32, Option<u32>, Vec<u32>, (u32, u32));

/// The buffers [`partition`] keeps from one call to the next rather than
/// allocating per call and growing rule by rule: the rows, the first
/// `rows_len` of `rows` the call's, each kept in a buffer an earlier call
/// left; the rows by hash; what a rule reads of the inputs and its row; its cube,
/// the cubes end to end with their rows; and the rules that can hold.
#[derive(Default)]
struct Buffers {
    rows: Vec<Vec<(u32, Allowed)>>,
    rows_len: usize,
    row_by: ByHash,
    read: Vec<InputRead>,
    row: Vec<(u32, Allowed)>,
    cube: Vec<GLit>,
    cubes: Vec<GLit>,
    cube_spans: Vec<(usize, usize, usize)>,
    firing: Vec<usize>,
}

/// The rules of `rules` whose bodies can hold, if in every world exactly one
/// holds, as [`clause_tables`] describes; `None` if not, or if a body reads
/// an atom that is not a fact.
///
/// The bodies are grouped by what they read of the inputs, a row each, and
/// the rows must partition the inputs ([`covers`]). Within a row the switch
/// literals, a cube per rule, must clash pairwise and their measures sum to
/// one.
fn partition(
    gp: &GroundProgram,
    rules: &[usize],
    exclusive: &[Vec<AtomId>],
    table: &[bool],
    inputs: &mut Inputs,
    buffers: &mut Buffers,
) -> Option<Vec<usize>> {
    let Inputs {
        of,
        info,
        binary,
        base,
        marks,
        slot,
        stamp,
    } = inputs;
    let Buffers {
        rows,
        rows_len,
        row_by,
        read,
        row,
        cube,
        cubes,
        cube_spans,
        firing,
    } = buffers;
    let states_of = |i: u32| match info[i as usize] {
        (usize::MAX, _) => 2,
        (s, present) => present + u32::from(present < exclusive[s].len() as u32 || !table[s]),
    };
    // The rows, each kept once, by hash, rather than a map keyed by the
    // rows, which holds a second copy of each.
    *rows_len = 0;
    row_by.clear();
    firing.clear();
    // What the rule at hand allows of each input it reads, kept as the one
    // state a positive literal names and the states negative ones rule out,
    // each marked with the rule's stamp: a body reading one value of a set
    // reads the others' copies negatively, dm_4's a hundred of them. Per
    // input, `slot` holds the stamp of the last rule to read it and where in
    // `read` that rule keeps it. An input read in binary keeps the bits the
    // rule fixes and their values. The first `reading` entries of `read`
    // are the rule's; the others, and the buffers of the row and the cube,
    // are kept for the rules after it, and the cubes go end to end in one
    // buffer with their rows, rather than allocated per rule.
    let mut reading: usize;
    cubes.clear();
    cube_spans.clear();
    let mut last: Option<usize> = None;
    'rules: for &ri in rules {
        *stamp += 1;
        let stamp = *stamp;
        reading = 0;
        cube.clear();
        let body = &gp.rules[ri].body;
        let mut next = 0;
        while next < body.len() {
            let l = body[next];
            next += 1;
            let a = l.atom();
            // A copy is a fact; only another atom's kind is read.
            let (i, state) = of[a as usize];
            if i == u32::MAX {
                if !gp.is_fact(a) {
                    return None;
                }
                cube.push(l);
                continue;
            }
            // The literal names one state, and allows it or rules it out.
            let (state, positive) = match info[i as usize].0 {
                usize::MAX => (u32::from(!l.is_positive()), true),
                _ => (state, l.is_positive()),
            };
            if slot[i as usize].0 != stamp {
                slot[i as usize] = (stamp, reading);
                if reading == read.len() {
                    read.push((i, states_of(i), None, Vec::new(), (0, 0)));
                } else {
                    let entry = &mut read[reading];
                    (entry.0, entry.1, entry.2, entry.4) = (i, states_of(i), None, (0, 0));
                    entry.3.clear();
                }
                reading += 1;
            }
            let (_, states, only, out, (fixed, code)) = &mut read[slot[i as usize].1];
            if binary[i as usize] {
                let bit = 1u32 << state;
                if *fixed & bit != 0 && (*code & bit != 0) != l.is_positive() {
                    continue 'rules;
                }
                *fixed |= bit;
                if l.is_positive() {
                    *code |= bit;
                }
                continue;
            }
            let mark = &mut marks[base[i as usize] as usize + state as usize];
            if positive {
                if only.is_some_and(|o| o != state) || *mark == stamp {
                    continue 'rules;
                }
                *only = Some(state);
                // The literals right after it that read the input's other
                // states negatively, as `introduce` writes a row's value of
                // a parent, rule the rule out only where one negates this
                // state, and are passed over without the rest.
                if info[i as usize].0 != usize::MAX {
                    while let Some(&m) = body.get(next)
                        && !m.is_positive()
                        && of[m.atom() as usize].0 == i
                    {
                        if of[m.atom() as usize].1 == state {
                            continue 'rules;
                        }
                        next += 1;
                    }
                }
            } else if *only == Some(state) {
                continue 'rules;
            } else if only.is_none() {
                // Once a positive literal has named the input's state, the
                // states its negative literals rule out are not read: a
                // row reads a parent's value and then the other values
                // negated (the expansion in `introduce`).
                if *mark != stamp {
                    *mark = stamp;
                    out.push(state);
                    if out.len() as u32 == *states {
                        continue 'rules;
                    }
                }
            }
        }
        row.clear();
        for (i, states, only, out, (fixed, code)) in &mut read[..reading] {
            let allowed = if binary[*i as usize] {
                // A code names a value, none, or no state at all.
                let values = info[*i as usize].1;
                if *fixed != (1 << Bits::width(values as usize)) - 1 {
                    return None;
                }
                match *code {
                    0 if *states > values => Allowed::Only(values),
                    c if c >= 1 && c <= values => Allowed::Only(c - 1),
                    _ => continue 'rules,
                }
            } else {
                match *only {
                    Some(state) => Allowed::Only(state),
                    None if out.len() as u32 + 1 == *states => {
                        let at = base[*i as usize] as usize;
                        Allowed::Only(
                            (0..*states)
                                .find(|&x| marks[at + x as usize] != stamp)
                                .expect("one state is left"),
                        )
                    }
                    None => {
                        out.sort_unstable();
                        Allowed::Except(out.clone())
                    }
                }
            };
            row.push((*i, allowed));
        }
        row.sort_unstable_by_key(|x| x.0);
        cube.sort_unstable();
        cube.dedup();
        if cube
            .iter()
            .any(|l| l.is_positive() && cube.binary_search(&GLit::Neg(l.atom())).is_ok())
        {
            continue;
        }
        firing.push(ri);
        // The rules of a row come one after another: a rule that reads as
        // the rule before has its row, found without a lookup.
        let r = match last {
            Some(r) if rows[r] == *row => r,
            _ => {
                let (r, new) = row_by.find_or_push(FxBuildHasher.hash_one(row.as_slice()), |r| {
                    rows[r as usize] == *row
                });
                if new {
                    debug_assert_eq!(r as usize, *rows_len, "a row numbered in order");
                    // The row goes into a buffer of its own, and the row's
                    // buffer takes the one it leaves, cleared for the next
                    // rule.
                    if *rows_len == rows.len() {
                        rows.push(Vec::new());
                    }
                    std::mem::swap(&mut rows[*rows_len], row);
                    *rows_len += 1;
                }
                r as usize
            }
        };
        last = Some(r);
        cube_spans.push((r, cubes.len(), cubes.len() + cube.len()));
        cubes.extend_from_slice(cube);
    }
    let keys: Vec<&[(u32, Allowed)]> = rows[..*rows_len].iter().map(Vec::as_slice).collect();
    if !covers(&keys, &mut Vec::new(), &states_of) {
        return None;
    }
    // Each row's cubes, in the order of their rules.
    cube_spans.sort_unstable();
    for spans in cube_spans.chunk_by(|x, y| x.0 == y.0) {
        let cube = |&(_, start, end): &(usize, usize, usize)| &cubes[start..end];
        let longest = spans.iter().map(|c| cube(c).len()).max().unwrap_or(0);
        if longest > 120 {
            return None;
        }
        let mass: u128 = spans
            .iter()
            .map(|c| 1u128 << (longest - cube(c).len()))
            .sum();
        if mass != 1u128 << longest {
            return None;
        }
        for (i, x) in spans.iter().map(cube).enumerate() {
            for y in spans[i + 1..].iter().map(cube) {
                let clash = x.iter().any(|l| {
                    let opposite = if l.is_positive() {
                        GLit::Neg(l.atom())
                    } else {
                        GLit::Pos(l.atom())
                    };
                    y.binary_search(&opposite).is_ok()
                });
                if !clash {
                    return None;
                }
            }
        }
    }
    Some(firing.clone())
}
