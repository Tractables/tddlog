//! Ground-program simplifications that preserve the distribution over the
//! remaining atoms.

use std::collections::VecDeque;
use std::sync::OnceLock;

use rustc_hash::{FxHashMap, FxHashSet};

use super::program::mix;
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, RulesByHead};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    Unknown,
    True,
    False,
}

/// Propagate atoms that are certainly true or certainly false in every world.
///
/// Facts with probability 1 are true and with probability 0 false. A derived
/// atom is true when one of its rules has an all-satisfied body, false when all
/// its rules are dead. Afterwards satisfied literals are removed from bodies,
/// dead rules are dropped, and a true atom keeps a single empty-body rule.
pub(crate) fn propagate_certain(gp: &mut GroundProgram) {
    let n = gp.atoms.len();
    let mut status = vec![Status::Unknown; n];
    let mut live_rules = vec![0usize; n];
    for r in &gp.rules {
        live_rules[r.head as usize] += 1;
    }
    let mut queue: VecDeque<AtomId> = VecDeque::new();
    for a in 0..n {
        match gp.kinds[a] {
            AtomKind::Fact(p) if p.not <= 0.0 => {
                status[a] = Status::True;
                queue.push_back(a as AtomId);
            }
            AtomKind::Fact(p) if p.p <= 0.0 => {
                status[a] = Status::False;
                queue.push_back(a as AtomId);
            }
            AtomKind::Derived if live_rules[a] == 0 => {
                status[a] = Status::False;
                queue.push_back(a as AtomId);
            }
            _ => {}
        }
    }
    for r in &gp.rules {
        if r.body.is_empty() && status[r.head as usize] == Status::Unknown {
            status[r.head as usize] = Status::True;
            queue.push_back(r.head);
        }
    }
    // With nothing certain, the program is left as it is.
    if queue.is_empty() {
        return;
    }
    // What the atoms certain so far make of each rule, in one pass: a rule
    // with a literal they falsify is dead. Where that leaves every other
    // atom undecided, no rule having all its literals satisfied and no
    // atom all its rules dead, nothing propagates past them, and these are
    // the rules the propagation would find dead: the index of the rules
    // reading each atom is built only where something does. A network's
    // switches of probability one, and the atoms the cover makes certain,
    // decide nothing further.
    let mut rule_dead = vec![false; gp.rules.len()];
    let mut live = live_rules.clone();
    let mut decides = false;
    for (ri, r) in gp.rules.iter().enumerate() {
        let h = r.head as usize;
        let mut unsatisfied = 0usize;
        for l in &r.body {
            match (l, status[l.atom() as usize]) {
                (_, Status::Unknown) => unsatisfied += 1,
                (GLit::Pos(_), Status::True) | (GLit::Neg(_), Status::False) => {}
                _ => {
                    rule_dead[ri] = true;
                    break;
                }
            }
        }
        if rule_dead[ri] {
            live[h] -= 1;
            decides |= live[h] == 0 && status[h] == Status::Unknown;
        } else {
            decides |= unsatisfied == 0 && status[h] == Status::Unknown;
        }
    }
    if decides {
        rule_dead.fill(false);
        propagate(gp, &mut status, &mut rule_dead, live_rules, queue);
    }
    // Rewrite, the bodies in place.
    let old = std::mem::take(&mut gp.rules);
    let mut rules = Vec::with_capacity(old.len());
    let mut has_true_rule = vec![false; n];
    for (i, mut r) in old.into_iter().enumerate() {
        if rule_dead[i] {
            continue;
        }
        let h = r.head as usize;
        if status[h] == Status::True {
            if !has_true_rule[h] {
                has_true_rule[h] = true;
                rules.push(GroundRule::new(r.head, Vec::new()));
            }
            continue;
        }
        r.body
            .retain(|l| status[l.atom() as usize] == Status::Unknown);
        rules.push(r);
    }
    // A fact that is certain is no longer a random variable.
    for a in 0..n {
        if let AtomKind::Fact(_) = gp.kinds[a] {
            match status[a] {
                Status::True => {
                    gp.kinds[a] = AtomKind::Derived;
                    if !has_true_rule[a] {
                        rules.push(GroundRule::new(a as AtomId, Vec::new()));
                    }
                }
                Status::False => gp.kinds[a] = AtomKind::Derived,
                Status::Unknown => {}
            }
        }
    }
    gp.rules = rules;
}

/// The propagation of [`propagate_certain`] from the atoms in `queue`,
/// whose status is known, over the rules reading each atom: the status of
/// every atom it decides, and the rules it finds dead. `live_rules` is
/// the number of rules of each atom.
fn propagate(
    gp: &GroundProgram,
    status: &mut [Status],
    rule_dead: &mut [bool],
    mut live_rules: Vec<usize>,
    mut queue: VecDeque<AtomId>,
) {
    let readers = gp.readers(|ri, _| ri);
    let mut unsatisfied: Vec<usize> = gp.rules.iter().map(|r| r.body.len()).collect();
    while let Some(a) = queue.pop_front() {
        let st = status[a as usize];
        // A rule that reads `a` twice is listed twice, and its literals of
        // `a` are counted at its first listing.
        let mut previous = u32::MAX;
        for &ri in &readers[a as usize] {
            if ri == previous {
                continue;
            }
            previous = ri;
            let ri = ri as usize;
            if rule_dead[ri] {
                continue;
            }
            for l in &gp.rules[ri].body {
                if l.atom() != a {
                    continue;
                }
                let satisfied = matches!(
                    (l, st),
                    (GLit::Pos(_), Status::True) | (GLit::Neg(_), Status::False)
                );
                if satisfied {
                    unsatisfied[ri] -= 1;
                    let h = gp.rules[ri].head;
                    if unsatisfied[ri] == 0 && status[h as usize] == Status::Unknown {
                        status[h as usize] = Status::True;
                        queue.push_back(h);
                    }
                } else {
                    rule_dead[ri] = true;
                    let h = gp.rules[ri].head;
                    live_rules[h as usize] -= 1;
                    if live_rules[h as usize] == 0 && status[h as usize] == Status::Unknown {
                        status[h as usize] = Status::False;
                        queue.push_back(h);
                    }
                    break;
                }
            }
        }
    }
}

/// Merge away the atoms the grounder introduced (a predicate name starting
/// with `$`) and no query or evidence names whose rules have the same
/// bodies as another atom's: in every world each is true exactly when one
/// of those bodies holds, so they are one function. The rules of an atom
/// merged away are dropped and its readers read the atom it was merged
/// into; merging repeats until the classes are stable, since readers of
/// merged atoms can become equal in turn.
///
/// A right-linear closure grounded from its bound end ([`super::linear`])
/// is the case in point: `path(1,Y)` and the chain atom `$r_path_2(1,Y)`
/// have the same bodies wherever `Y` is not 1, and without the merge both
/// are compiled. Atoms the program writes are left to
/// [`super::aig::reduce`], once the exclusive sets are known: on
/// bn_hailfinder, merging `scnrelplfcst(b) :- truescenario(b)` into
/// `scenrel3_4(b)` would split a network variable's values across two
/// others, and the layout's largest formula grows tenfold.
///
/// The two merges differ in when they run and how they reach a fixpoint.
/// This one runs on every program before its structure is known, the
/// settle path's programs among them, and repeats over the whole program,
/// which takes the grounder's atoms in cyclic components (the closure's
/// chain atoms) and needs few rounds for them. The other needs the
/// exclusive sets and the dependency order of the structure and builds
/// the acyclic atoms once each, as an and-inverter graph, where rounds
/// would take one for each level of copied gates: 4 096 on GPA_w19, each
/// over its 213 k rules.
pub(crate) fn merge_equal_atoms(gp: &mut GroundProgram) {
    let n = gp.atoms.len();
    // What a query or evidence names keeps its identity (a query driver's
    // `$query` atom among them), so the printed program defines it.
    let mut away: Vec<bool> = gp.atoms.iter().map(|a| a.name.starts_with('$')).collect();
    for a in gp.named_atoms() {
        away[a as usize] = false;
    }
    loop {
        // Atoms merge only where one is merged away: where no derived atom
        // with rules is, nothing is grouped or hashed.
        let mut ruled = vec![false; n];
        for r in &gp.rules {
            ruled[r.head as usize] = true;
        }
        let merged_away = |a: usize| away[a] && gp.kinds[a] == AtomKind::Derived && ruled[a];
        if !(0..n).any(merged_away) {
            return;
        }
        let by_head = gp.rules_by_head();
        // Only the atoms whose bodies may be those of an atom merged away,
        // by a hash of them, have their bodies compared. On a network,
        // whose atoms merged away are its certain switches, only the
        // switches and the atoms with one empty body need their bodies
        // sorted and kept.
        let hash = body_set_hashes(gp, &by_head);
        let shared: FxHashSet<u64> = (0..n)
            .filter(|&a| merged_away(a))
            .map(|a| hash[a])
            .collect();
        // Of these, keys are built only where two atoms' hashes meet: on a
        // program conditioned on its guards, whose pass made each of its
        // hundreds of thousands of atoms with bodies of its own, almost
        // nowhere.
        let candidate = |a: usize| {
            gp.kinds[a] == AtomKind::Derived && !by_head[a].is_empty() && shared.contains(&hash[a])
        };
        let mut met: FxHashMap<u64, bool> = FxHashMap::default();
        for a in (0..n).filter(|&a| candidate(a)) {
            met.entry(hash[a])
                .and_modify(|twice| *twice = true)
                .or_insert(false);
        }
        let mut class_of: FxHashMap<Vec<Vec<GLit>>, AtomId> = FxHashMap::default();
        let mut into: Vec<AtomId> = (0..n as AtomId).collect();
        let mut merged = false;
        for a in 0..n {
            if !candidate(a) || !met[&hash[a]] {
                continue;
            }
            let key = rules_key(gp, &by_head[a]);
            match class_of.get_mut(&key) {
                None => {
                    class_of.insert(key, a as AtomId);
                }
                Some(rep) if away[a] => {
                    into[a] = *rep;
                    merged = true;
                }
                Some(rep) if away[*rep as usize] => {
                    into[*rep as usize] = a as AtomId;
                    *rep = a as AtomId;
                    merged = true;
                }
                Some(_) => {}
            }
        }
        if !merged {
            return;
        }
        // An atom merged into one later merged in turn reads the last.
        let root = |mut a: AtomId| {
            while into[a as usize] != a {
                a = into[a as usize];
            }
            a
        };
        gp.rules.retain(|r| root(r.head) == r.head);
        // Where no body reads an atom merged away, the bodies stay as they
        // are, and another round would find the atoms left with the same
        // bodies as before, of which none is merged away or one alone.
        if !gp
            .rules
            .iter()
            .any(|r| r.body.iter().any(|l| into[l.atom() as usize] != l.atom()))
        {
            return;
        }
        for r in &mut gp.rules {
            for l in &mut r.body {
                *l = GLit::new(root(l.atom()), l.is_positive());
            }
        }
    }
}

/// The key by which [`merge_equal_atoms`] compares atoms: the set of the
/// bodies of `rules`, each the set of its literals. Two atoms with equal
/// keys are true in the same worlds.
fn rules_key(gp: &GroundProgram, rules: &[usize]) -> Vec<Vec<GLit>> {
    let mut key: Vec<Vec<GLit>> = rules
        .iter()
        .map(|&ri| {
            let mut body = gp.rules[ri].body.clone();
            body.sort_unstable();
            body.dedup();
            body
        })
        .collect();
    key.sort_unstable();
    key.dedup();
    key
}

/// For each derived atom with rules, a hash of the set of its rules'
/// bodies, each taken as a set of literals: atoms whose rules have the same
/// bodies have the same hash. Zero for the other atoms.
fn body_set_hashes(gp: &GroundProgram, by_head: &RulesByHead) -> Vec<u64> {
    let (mut lits, mut bodies) = (Vec::new(), Vec::new());
    let mut hashes = vec![0; gp.atoms.len()];
    for (a, rules) in by_head.iter().enumerate() {
        if gp.kinds[a] == AtomKind::Derived && !rules.is_empty() {
            hashes[a] = rules_hash(gp, rules, &mut lits, &mut bodies);
        }
    }
    hashes
}

/// A hash of [`rules_key`]'s key, computed without building it (`lits`
/// and `bodies` are scratch): equal keys have equal hashes.
fn rules_hash(
    gp: &GroundProgram,
    rules: &[usize],
    lits: &mut Vec<GLit>,
    bodies: &mut Vec<u64>,
) -> u64 {
    bodies.clear();
    for &ri in rules {
        lits.clear();
        lits.extend_from_slice(&gp.rules[ri].body);
        // A strictly increasing body is a set already.
        if !lits.windows(2).all(|w| w[0] < w[1]) {
            lits.sort_unstable();
            lits.dedup();
        }
        bodies.push(lits.iter().fold(0u64, |h, &l| h.wrapping_add(l.mixed())));
    }
    bodies.sort_unstable();
    bodies.dedup();
    bodies.iter().fold(0u64, |h, &b| h.wrapping_add(mix(b)))
}

/// Keep only rules whose head the queries or evidence depend on, then
/// compact. Returns the new number of each atom ([`renumber`]).
pub(crate) fn restrict_to_relevant(gp: &mut GroundProgram) -> Vec<AtomId> {
    let by_head = gp.rules_by_head();
    let mut relevant = vec![false; gp.atoms.len()];
    let mut stack: Vec<AtomId> = Vec::new();
    for (_, id, _) in &gp.queries {
        if let Some(a) = id {
            stack.push(*a);
        }
    }
    for (_, id, _) in &gp.evidence {
        if let Some(a) = id {
            stack.push(*a);
        }
    }
    while let Some(a) = stack.pop() {
        if relevant[a as usize] {
            continue;
        }
        relevant[a as usize] = true;
        for &ri in &by_head[a as usize] {
            for l in &gp.rules[ri].body {
                if !relevant[l.atom() as usize] {
                    stack.push(l.atom());
                }
            }
        }
    }
    gp.rules.retain(|r| relevant[r.head as usize]);
    // The atoms relevant are the ones the rules kept, the queries and the
    // evidence name, those [`compact`] would mark.
    keep_used(gp, &relevant)
}

/// A derived atom whose only rule is `a :- f` for a fact `f` becomes the
/// random variable itself, where every use of `f` is such a rule: the
/// first of those atoms takes `f`'s place, and the others, true in exactly
/// the same worlds, are read as it.
///
/// An undirected edge written as two directed ones, `conn(X,Y) :-
/// edge(X,Y). conn(X,Y) :- edge(Y,X).`, is the case in point: `conn(a,b)`
/// and `conn(b,a)` both read `edge(a,b)` alone. Kept apart, a reachability
/// component over them reads no fact directly ([`crate::vtree`]'s
/// `nothing_to_order` then misses it), and its rewrite without recursion
/// ([`super::eliminate`]) builds each path label twice, once per
/// direction, over different atoms of the same function; read as one
/// atom, the two directions have the same bodies and
/// [`merge_equal_atoms`] keeps one. A fact also read otherwise (negated,
/// with other literals, or named by a query or evidence) is left alone,
/// so that a network variable read through `v(t) :- sw. v(f) :- \+sw.`
/// keeps its values as atoms.
pub(crate) fn alias_fact_atoms(gp: &mut GroundProgram) {
    // A pass makes facts of the atoms that read a fact alone; an atom that
    // reads one of those alone, `conn(a,b) :- edge(a,b)` over `edge(a,b) :-
    // sw`, becomes a fact in the next.
    while alias_pass(gp) {}
    compact(gp);
}

/// One pass of [`alias_fact_atoms`]: whether it made a fact of an atom.
fn alias_pass(gp: &mut GroundProgram) -> bool {
    let n = gp.atoms.len();
    let by_head = gp.rules_by_head();
    let mut uses = vec![0usize; n];
    for r in &gp.rules {
        for l in &r.body {
            uses[l.atom() as usize] += 1;
        }
    }
    // A query or evidence directive is a use too: the fact must keep its
    // identity if anything outside the rules names it.
    let mut named = vec![false; n];
    for a in gp.named_atoms() {
        uses[a as usize] += 1;
        named[a as usize] = true;
    }
    // The atoms whose only rule reads one fact, positively, by the fact,
    // each fact's in the order of their numbers.
    let mut aliases: FxHashMap<AtomId, Vec<(AtomId, usize)>> = FxHashMap::default();
    for (a, heads) in by_head.iter().enumerate() {
        if gp.kinds[a] != AtomKind::Derived || heads.len() != 1 {
            continue;
        }
        let ri = heads[0];
        if let [GLit::Pos(f)] = gp.rules[ri].body[..]
            && let AtomKind::Fact(_) = gp.kinds[f as usize]
        {
            aliases.entry(f).or_default().push((a as AtomId, ri));
        }
    }
    let mut drop_rule = vec![false; gp.rules.len()];
    let mut into: Vec<AtomId> = (0..n as AtomId).collect();
    let (mut aliased, mut renamed) = (false, false);
    for (f, atoms) in aliases {
        if atoms.len() != uses[f as usize] {
            continue;
        }
        let AtomKind::Fact(p) = gp.kinds[f as usize] else {
            unreachable!("aliases read facts")
        };
        let (first, ri) = atoms[0];
        gp.kinds[first as usize] = AtomKind::Fact(p);
        gp.kinds[f as usize] = AtomKind::Derived; // now unreferenced
        drop_rule[ri] = true;
        aliased = true;
        for &(a, ri) in &atoms[1..] {
            if named[a as usize] {
                // A query or evidence names it: it keeps a rule, `a :- first`.
                gp.rules[ri].body[0] = GLit::Pos(first);
            } else {
                drop_rule[ri] = true;
            }
            into[a as usize] = first;
            renamed = true;
        }
    }
    if !aliased {
        return false;
    }
    let mut i = 0;
    gp.rules.retain(|_| {
        let keep = !drop_rule[i];
        i += 1;
        keep
    });
    if renamed {
        for r in &mut gp.rules {
            for l in &mut r.body {
                *l = GLit::new(into[l.atom() as usize], l.is_positive());
            }
        }
        gp.structure = OnceLock::new();
    }
    true
}

/// Renumber atoms so that only those referenced by rules, queries or evidence
/// remain, preserving relative order. Returns the new number of each atom
/// ([`renumber`]).
pub(crate) fn compact(gp: &mut GroundProgram) -> Vec<AtomId> {
    let n = gp.atoms.len();
    let mut used = vec![false; n];
    for r in &gp.rules {
        used[r.head as usize] = true;
        for l in &r.body {
            used[l.atom() as usize] = true;
        }
    }
    for a in gp.named_atoms() {
        used[a as usize] = true;
    }
    keep_used(gp, &used)
}

/// Keeps the atoms `used` marks, in their order, and drops the others
/// ([`renumber`]). Returns the new number of each atom.
fn keep_used(gp: &mut GroundProgram, used: &[bool]) -> Vec<AtomId> {
    let n = gp.atoms.len();
    let order: Vec<AtomId> = (0..n as AtomId).filter(|&a| used[a as usize]).collect();
    // Every atom used keeps its number, and the rules are left as they are
    // rather than rewritten literal by literal to themselves.
    if order.len() == n {
        return order;
    }
    renumber(gp, &order)
}

/// Numbers the atoms of `order` by their place in it and drops the others,
/// which no rule, query or evidence may name. Returns the new number of
/// each atom, `AtomId::MAX` for one dropped.
pub(crate) fn renumber(gp: &mut GroundProgram, order: &[AtomId]) -> Vec<AtomId> {
    let mut map = vec![u32::MAX; gp.atoms.len()];
    let mut atoms = Vec::with_capacity(order.len());
    let mut kinds = Vec::with_capacity(order.len());
    for &a in order {
        map[a as usize] = atoms.len() as AtomId;
        atoms.push(std::mem::take(&mut gp.atoms[a as usize]));
        kinds.push(gp.kinds[a as usize]);
    }
    let m = |a: AtomId| map[a as usize];
    for r in &mut gp.rules {
        r.head = m(r.head);
        for l in &mut r.body {
            *l = GLit::new(m(l.atom()), l.is_positive());
        }
    }
    for (_, id, _) in &mut gp.queries {
        *id = id.map(m);
    }
    for (_, id, _) in &mut gp.evidence {
        *id = id.map(m);
    }
    gp.atoms = atoms;
    gp.kinds = kinds;
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::Prob;
    use crate::ground::program::test_program;
    use crate::syntax::Atom;
    use GLit::{Neg, Pos};

    /// A program of facts of probabilities `facts`, atoms 0 on, and
    /// `derived` derived atoms after them, with `rules`.
    fn program(facts: &[f64], derived: usize, rules: &[(AtomId, &[GLit])]) -> GroundProgram {
        test_program(facts, derived, rules, &[])
    }

    fn rules_of(gp: &GroundProgram) -> Vec<(AtomId, Vec<GLit>)> {
        gp.rules.iter().map(|r| (r.head, r.body.clone())).collect()
    }

    #[test]
    fn certain_atoms_that_decide_no_other_drop_out_of_the_bodies() {
        // 0 is certain, 1 free and 2 impossible: 3 and 4 keep a rule each.
        let mut gp = program(
            &[1.0, 0.5, 0.0],
            2,
            &[
                (3, &[Pos(0), Pos(1)]),
                (3, &[Pos(2)]),
                (4, &[Neg(2), Pos(1)]),
                (4, &[Neg(1)]),
            ],
        );
        propagate_certain(&mut gp);
        assert_eq!(
            rules_of(&gp),
            [
                (3, vec![Pos(1)]),
                (4, vec![Pos(1)]),
                (4, vec![Neg(1)]),
                (0, vec![])
            ]
        );
        assert_eq!(
            gp.kinds[..3],
            [
                AtomKind::Derived,
                AtomKind::Fact(Prob::new(0.5)),
                AtomKind::Derived
            ]
        );
    }

    #[test]
    fn certain_atoms_decide_the_atoms_that_read_them_in_turn() {
        // 3 :- 0 is certain, and with it 7 :- 3; 5 :- 2 is impossible.
        let rules: &[(AtomId, &[GLit])] = &[
            (3, &[Pos(0)]),
            (4, &[Pos(3), Pos(1)]),
            (5, &[Pos(2)]),
            (6, &[Neg(5), Pos(1)]),
            (7, &[Pos(3)]),
            (7, &[Pos(1)]),
        ];
        let mut gp = program(&[1.0, 0.5, 0.0], 5, rules);
        propagate_certain(&mut gp);
        assert_eq!(
            rules_of(&gp),
            [
                (3, vec![]),
                (4, vec![Pos(1)]),
                (6, vec![Pos(1)]),
                (7, vec![]),
                (0, vec![])
            ]
        );
    }

    /// Atoms 1 and 2 read fact 0 alone, as `conn(a,b)` and `conn(b,a)`
    /// read `edge(a,b)`: 1 becomes the fact and 3 reads it twice. Atom 6
    /// reads fact 4 alone, as does 7, but 8 reads 4 negated: 4 stays.
    #[test]
    fn atoms_that_read_one_fact_alone_are_that_fact() {
        let rules: &[(AtomId, &[GLit])] = &[
            (1, &[Pos(0)]),
            (2, &[Pos(0)]),
            (3, &[Pos(1), Pos(5)]),
            (3, &[Pos(2)]),
            (6, &[Pos(4)]),
            (7, &[Pos(4)]),
            (8, &[Neg(4), Pos(6), Pos(7)]),
        ];
        let mut gp = program(&[0.5, 0.0, 0.0, 0.0, 0.3, 0.2], 3, rules);
        for a in 1..4 {
            gp.kinds[a] = AtomKind::Derived;
        }
        gp.queries = vec![
            (Atom::default(), Some(3), true),
            (Atom::default(), Some(8), true),
        ];
        alias_fact_atoms(&mut gp);
        // 0 and 2 go; 1 is the fact of probability 0.5.
        assert_eq!(
            gp.kinds[..4],
            [
                AtomKind::Fact(Prob::new(0.5)),
                AtomKind::Derived,
                AtomKind::Fact(Prob::new(0.3)),
                AtomKind::Fact(Prob::new(0.2))
            ]
        );
        assert_eq!(
            rules_of(&gp),
            [
                (1, vec![Pos(0), Pos(3)]),
                (1, vec![Pos(0)]),
                (4, vec![Pos(2)]),
                (5, vec![Pos(2)]),
                (6, vec![Neg(2), Pos(4), Pos(5)])
            ]
        );
    }

    /// A chain `conn :- edge :- sw` aliases in two passes, and an alias a
    /// query names keeps a rule that reads the fact.
    #[test]
    fn aliases_chain_and_named_ones_keep_a_rule() {
        let rules: &[(AtomId, &[GLit])] = &[
            (1, &[Pos(0)]),
            (2, &[Pos(1)]),
            (3, &[Pos(1)]),
            (4, &[Pos(2), Pos(3)]),
        ];
        let mut gp = program(&[0.5], 4, rules);
        gp.queries = vec![
            (Atom::default(), Some(3), true),
            (Atom::default(), Some(4), true),
        ];
        alias_fact_atoms(&mut gp);
        // 1 is the fact first, then 2 is, and 3 keeps `3 :- 2`.
        assert_eq!(
            gp.kinds[..],
            [
                AtomKind::Fact(Prob::new(0.5)),
                AtomKind::Derived,
                AtomKind::Derived
            ]
        );
        assert_eq!(
            rules_of(&gp),
            [(1, vec![Pos(0)]), (2, vec![Pos(0), Pos(0)])]
        );
        assert_eq!(
            gp.queries.iter().map(|q| q.1).collect::<Vec<_>>(),
            [Some(1), Some(2)]
        );
    }

    /// A body that reads certain fact 0 twice, as an alias leaves
    /// `conn(a,b), conn(b,a)`, still waits on 1: 2 is not certain.
    #[test]
    fn a_certain_atom_read_twice_satisfies_its_literals_once() {
        let mut gp = program(
            &[1.0, 0.5],
            2,
            &[(2, &[Pos(0), Pos(0), Pos(1)]), (3, &[Pos(0)])],
        );
        propagate_certain(&mut gp);
        assert_eq!(rules_of(&gp), [(2, vec![Pos(1)]), (3, vec![]), (0, vec![])]);
    }

    #[test]
    fn a_program_without_certain_atoms_is_left_as_it_is() {
        let mut gp = program(
            &[0.5, 0.5],
            2,
            &[(2, &[Pos(0), Neg(1)]), (3, &[Pos(2)]), (3, &[Neg(0)])],
        );
        let before = rules_of(&gp);
        propagate_certain(&mut gp);
        assert_eq!(rules_of(&gp), before);
    }
}
