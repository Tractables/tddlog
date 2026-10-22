//! Views inlined into their readers.
//!
//! A program over a database names parts of its queries: a selection of one
//! relation, the join of a pair, a subquery, each an intermediate predicate
//! (a view) that the query reads. Tp-compilation gives every derived atom a
//! formula of its own and composes it into its readers' formulas, so a view
//! costs a component, its formula and the formula's negation, and the
//! reader one term per body that reads it, on the reader's scope; and the
//! strategies see a program of many components where the query written
//! as one rule is one component of bodies over facts alone. On TPC-H's q18
//! through three views, the query's bodies, one per customer, each negated
//! on the query's whole scope, are nearly all of the compile.
//!
//! A derived atom `a` that no query or evidence names, of a predicate that
//! does not depend on itself, in no exclusive set, read only positively,
//! and whose rules read facts alone (a selection or a join of relations,
//! or a view over views already inlined), is replaced in its readers by
//! its rules' bodies: `h :- L, a` with the rules `a :- B_1; ...; a :- B_k`
//! becomes `h :- L, B_1; ...; h :- L, B_k`. In every world `a` holds
//! exactly where one of its bodies does, so each reader's head holds in
//! the same worlds, and no other atom's either. That is done where
//!
//! - `a` is read once, by one rule: its `k` bodies each take a copy of the
//!   reader's other literals `L`. The rule's other derived literals are then
//!   read `k` times, so a product of such views is unfolded one view deep;
//! - `a` has one rule: each reader takes a copy of the body in place of `a`.
//!
//! and the literals it adds are at most [`GROWTH`] times those of `a`'s
//! rules and its readers together. The atoms are taken in dependency order,
//! so that a view over views comes after them.
//!
//! An atom of one rule read once, by one rule, is replaced there by its
//! body whatever the body reads: no literal is copied and no atom is read
//! more often, and the program loses an atom and a literal. So is an
//! exclusive set of such atoms of several predicates, the branches of a
//! choice, as a whole. A loop unrolled step by step names each step's
//! intermediate values, each read once by the next value of the state,
//! two of them exclusive where a branch sets them (the Dice paper's
//! diamond, unrolled: four atoms a step, one once they are inlined;
//! ladder: eleven atoms, seven).
//!
//! Members of exclusive sets stay, but for the branches of a choice
//! (below): they are the values of a network variable, which the row
//! classes, the cover, the indicators and the families layout read as a
//! set (`aig::reduce` leaves a network variable's values
//! alone for the same reason). So do the atoms of a
//! recursive predicate, ground cyclic or not, some atom of which is read
//! in several bodies or negated: a path in a directed grid is read by the
//! paths into its first node, and each of them is a part that several
//! readers share; inlined, the grids' boundary paths make grid12's trials
//! many times larger and grid15's unfinished, and the paths read once
//! alone grid20's four times larger. A recursive predicate whose atoms
//! are each read once, positively, is a chain, such as a loop written as
//! recursion over a time index or a word's bits written as recursion
//! over their index, and its atoms are inlined as the same loop's
//! unrolled steps are, as long as their readers' bodies stay within
//! [`CHAIN_GROWTH`] times the longest body (bb_pi_w10 with bit/2 of its
//! words as recursion: 328 M units of work kept, 14 M inlined). The
//! atoms of the grounder's own predicates (`$`, the guards' for one)
//! stay: they are read where the grounder put them, and inlined,
//! bb_coinBias_w16's 4 081 guards made its compile slower.

use rustc_hash::FxHashSet;

use super::deps::recursive_predicates;
use super::structure::Structure;
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, simplify};

/// An atom is inlined only where the literals inlining adds are at most
/// this many times the literals of its rules and of the rules that read it.
/// TPC-H's queries written through views take at most 1.8 times their
/// views' literals written as one rule.
const GROWTH: usize = 2;

/// An atom of a chain is inlined only where each reader's body comes out
/// at most this many times the program's longest body. Unbounded, the
/// inlining turns a chain into one rule per step taken so far, and the
/// copies its compile reads grow with them: bb_fair_dt44_w16 with its
/// words as recursion takes 20 G instructions at twice the longest body,
/// 266 G at four times.
const CHAIN_GROWTH: usize = 2;

/// A rule as the inlining goes: its place among the rules is `key`, the
/// place of the rule it was made from followed by its own among those
/// made from it, so that the rules made from a reader stand where it did.
struct Rule {
    head: AtomId,
    body: Vec<GLit>,
    key: Vec<u32>,
    live: bool,
}

/// Inlines the views of `gp` whose structure is `structure`, as above,
/// counts them in its stats (the atoms of chains as `chained`, the others
/// as `inlined`) and returns how many atoms it inlined. The atoms inlined
/// are dropped
/// (`simplify::compact`), so the structure is the program's no more where
/// any was.
pub(super) fn views(gp: &mut GroundProgram, structure: &Structure) -> usize {
    let n = gp.num_atoms();
    let mut kept = vec![false; n];
    for a in gp.named_atoms() {
        kept[a as usize] = true;
    }
    let seen = seen(gp);
    // Members of an exclusive set, and whether its set goes whole: the
    // branches of a choice, of several predicates, each an atom of one rule
    // read once. The values of one predicate are a network variable's
    // (bn_munin's copies of a variable, each value read once by the next
    // copy, cost auto twice the work inlined).
    let (mut in_set, mut whole) = (vec![false; n], vec![false; n]);
    let predicate = |a: AtomId| (&*gp.atoms[a as usize].name, gp.atoms[a as usize].args.len());
    for set in &structure.sets {
        let one_predicate = set.iter().all(|&a| predicate(a) == predicate(set[0]));
        let branch = |a: AtomId| seen[a as usize].single() && !kept[a as usize];
        let all = !one_predicate && set.iter().all(|&a| branch(a));
        for &a in set {
            in_set[a as usize] = true;
            whole[a as usize] = all;
        }
    }
    let view = |a: usize| {
        let s = &seen[a];
        !kept[a]
            && gp.kinds[a] == AtomKind::Derived
            && s.read_positively()
            && ((s.over_facts && !in_set[a]) || (s.single() && (!in_set[a] || whole[a])))
    };
    if !(0..n).any(view) {
        return 0;
    }
    // The recursive predicates some atom of which is shared: read in
    // several bodies or negated.
    let shared: FxHashSet<(&str, usize)> = (0..n)
        .filter(|&a| seen[a].reads > 1 || seen[a].negated)
        .map(|a| gp.atoms[a].signature())
        .collect();
    let mut chain = vec![false; n];
    for (a, recursive) in recursive_predicates(gp).into_iter().enumerate() {
        kept[a] |= recursive
            && (shared.contains(&gp.atoms[a].signature()) || gp.atoms[a].name.starts_with('$'));
        chain[a] = recursive && !kept[a];
    }
    let bound = CHAIN_GROWTH * gp.rules.iter().map(|r| r.body.len()).max().unwrap_or(0);
    let mut rules: Vec<Rule> = gp
        .rules
        .iter()
        .enumerate()
        .map(|(i, r)| Rule {
            head: r.head,
            body: r.body.clone(),
            key: vec![i as u32],
            live: true,
        })
        .collect();
    let mut by_head: Vec<Vec<u32>> = vec![Vec::new(); n];
    // The rules each atom is read in, once per rule, dead ones among them.
    let mut read_in: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (i, r) in rules.iter().enumerate() {
        by_head[r.head as usize].push(i as u32);
        for l in &r.body {
            read_in[l.atom() as usize].push(i as u32);
        }
    }
    for list in &mut read_in {
        list.dedup();
    }
    let is_fact = |a: AtomId| matches!(gp.kinds[a as usize], AtomKind::Fact(_));
    let mut substitution = Substitution {
        mark: vec![0; n],
        sign: vec![false; n],
        stamp: 0,
    };
    let mut inlined = 0;
    let mut chained = 0;
    for &a in structure.sccs.iter().flatten() {
        if kept[a as usize] {
            continue;
        }
        let readers: Vec<u32> = read_in[a as usize]
            .iter()
            .copied()
            .filter(|&ri| rules[ri as usize].live)
            .collect();
        let (mut uses, mut negative) = (0, false);
        for &ri in &readers {
            for l in &rules[ri as usize].body {
                if l.atom() == a {
                    uses += 1;
                    negative |= !l.is_positive();
                }
            }
        }
        let own: Vec<u32> = by_head[a as usize]
            .iter()
            .copied()
            .filter(|&ri| rules[ri as usize].live)
            .collect();
        if negative || uses == 0 || own.is_empty() {
            continue;
        }
        let substituted = uses == 1 && own.len() == 1 && (!in_set[a as usize] || whole[a as usize]);
        if !substituted
            && (in_set[a as usize]
                || !own
                    .iter()
                    .all(|&ri| rules[ri as usize].body.iter().all(|l| is_fact(l.atom()))))
        {
            continue; // a value of a network variable, or it reads a derived atom that stays
        }
        let own_literals: usize = own.iter().map(|&ri| rules[ri as usize].body.len()).sum();
        let reader_literals: usize = readers
            .iter()
            .map(|&ri| rules[ri as usize].body.len())
            .sum();
        if chain[a as usize] {
            let own_max = own
                .iter()
                .map(|&ri| rules[ri as usize].body.len())
                .max()
                .unwrap_or(0);
            if readers
                .iter()
                .any(|&ri| rules[ri as usize].body.len() - 1 + own_max > bound)
            {
                continue;
            }
        }
        let added = if uses == 1 {
            // Each body takes a copy of the reader's other literals.
            (own.len() - 1) * (reader_literals - 1)
        } else if own.len() == 1 {
            // Each reader takes a copy of the body in place of the atom.
            (uses - 1) * own_literals.saturating_sub(1)
        } else {
            continue;
        };
        if added > GROWTH * (own_literals + reader_literals) {
            continue;
        }
        let bodies: Vec<Vec<GLit>> = own
            .iter()
            .map(|&ri| rules[ri as usize].body.clone())
            .collect();
        for &ri in &own {
            rules[ri as usize].live = false;
        }
        for &ri in &readers {
            rules[ri as usize].live = false;
            let (head, reader, key) = {
                let r = &rules[ri as usize];
                (r.head, r.body.clone(), r.key.clone())
            };
            for (j, body) in bodies.iter().enumerate() {
                let Some(lits) = substitution.of(&reader, a, body) else {
                    continue;
                };
                let id = rules.len() as u32;
                by_head[head as usize].push(id);
                for l in &lits {
                    read_in[l.atom() as usize].push(id);
                }
                let mut key = key.clone();
                key.push(j as u32);
                rules.push(Rule {
                    head,
                    body: lits,
                    key,
                    live: true,
                });
            }
        }
        inlined += 1;
        chained += usize::from(chain[a as usize]);
    }
    if inlined == 0 {
        return 0;
    }
    let mut live: Vec<Rule> = rules.into_iter().filter(|r| r.live).collect();
    live.sort_by(|x, y| x.key.cmp(&y.key));
    // Two readers of one view can come out the same rule.
    let mut seen: FxHashSet<(AtomId, Vec<GLit>)> = FxHashSet::default();
    gp.rules = live
        .into_iter()
        .filter(|r| {
            let mut body = r.body.clone();
            body.sort_unstable();
            seen.insert((r.head, body))
        })
        .map(|r| GroundRule::new(r.head, r.body))
        .collect();
    simplify::compact(gp);
    gp.stats.inlined += inlined - chained;
    gp.stats.chained += chained;
    inlined
}

/// A body under construction: each atom's sign where `stamp` marks it.
struct Substitution {
    mark: Vec<u32>,
    sign: Vec<bool>,
    stamp: u32,
}

impl Substitution {
    /// The body `reader` with `body` in place of the literal `a`, each
    /// literal once; `None` where it holds a literal and its complement,
    /// and so never holds.
    fn of(&mut self, reader: &[GLit], a: AtomId, body: &[GLit]) -> Option<Vec<GLit>> {
        self.stamp += 1;
        let mut lits = Vec::with_capacity(reader.len() + body.len());
        let mut clash = false;
        for &l in reader.iter().flat_map(|l| {
            if *l == GLit::Pos(a) {
                body
            } else {
                std::slice::from_ref(l)
            }
        }) {
            let x = l.atom() as usize;
            if self.mark[x] == self.stamp {
                clash |= self.sign[x] != l.is_positive();
                continue;
            }
            self.mark[x] = self.stamp;
            self.sign[x] = l.is_positive();
            lits.push(l);
        }
        (!clash).then_some(lits)
    }
}

/// What one pass over the rules of a program sees of an atom: how often
/// it is read positively, whether it is read negatively, how many rules
/// it has, and whether they read facts alone. Asked first, it tells
/// [`views`] whether there is a view to inline, the first atom it inlines
/// being one with the rules of the program until then, and so spares a
/// program without views the copy of every rule and the index of their
/// readers (TPC-H's q1 reads its tuples in a rule each of a query).
#[derive(Clone, Copy)]
struct Seen {
    reads: u32,
    negated: bool,
    rules: u32,
    over_facts: bool,
}

impl Seen {
    /// Read, and only positively, and with rules.
    fn read_positively(&self) -> bool {
        self.reads > 0 && !self.negated && self.rules > 0
    }

    /// One rule, read once, positively.
    fn single(&self) -> bool {
        self.reads == 1 && !self.negated && self.rules == 1
    }
}

fn seen(gp: &GroundProgram) -> Vec<Seen> {
    let mut seen = vec![
        Seen {
            reads: 0,
            negated: false,
            rules: 0,
            over_facts: true
        };
        gp.num_atoms()
    ];
    for r in &gp.rules {
        let mut over_facts = true;
        for l in &r.body {
            let a = l.atom() as usize;
            match l.is_positive() {
                true => seen[a].reads = seen[a].reads.saturating_add(1),
                false => seen[a].negated = true,
            }
            over_facts &= matches!(gp.kinds[a], AtomKind::Fact(_));
        }
        let head = &mut seen[r.head as usize];
        head.rules = head.rules.saturating_add(1);
        head.over_facts &= over_facts;
    }
    seen
}
