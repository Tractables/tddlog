//! Choices read under exclusive contexts: a fact per context.
//!
//! A choice is a table ([`Tables`]) whose atoms each have one rule over the
//! table's own facts alone, the facts that nothing else reads: the values
//! of an annotated disjunction without a body, as normalization writes them
//! ([`super::normalize`]), or one atom of a conjunction of such facts. Where
//! every rule that reads an atom of a choice reads one, positively, and the
//! rest of the bodies of every two such rules, their contexts, clash, each
//! of these rules reads a fact of its own in place of the atom, of the
//! atom's probability, and the choice goes:
//!
//! ```text
//! d(0) :- s0.  d(1) :- s1, \+s0.  d(2) :- s2, \+s0, \+s1.        0.03::f0.  0.06::f1.
//! e :- k(0), d(2).  e :- k(1), d(1).                        →    e :- k(0), f0.  e :- k(1), f1.
//! ```
//!
//! In every world at most one of the contexts holds. Where one does, its
//! rule holds where the atom it read holds, and that atom is a function of
//! the choice's facts alone, which decide nothing else; the fact read in its
//! place has the same probability and decides nothing else either. So every
//! atom of the program holds with the same probability given the facts
//! outside the choice, and the queries and the evidence with the same
//! probability. Contexts that do not clash would not do: two rules both
//! holding would read two facts independent of each other where they read
//! one choice.
//!
//! Dice's `iterate` draws a value each step and observes a function of it
//! and the state: [`data/dice/simpleCaesar.dice`](https://github.com/Tractables/tddlog/tree/baselines/data/dice/simpleCaesar.dice) draws one of 32 characters a
//! step and observes it added to a key of 32 values, a rule per value of the
//! key reading the character it takes. Under each value of the key a
//! step's formula is then one fact, not the chain of the switches before
//! the character's own, and switch sharing ([`super::share`]) makes the
//! facts of one probability one.
//!
//! A choice is read so where it saves literals at least as many as it adds
//! facts: the rules' atoms read through chains of switches, not a fact read
//! in many contexts.

use rustc_hash::FxHashSet;

use crate::syntax::{Atom, Term};

use super::tables::Tables;
use super::{AtomId, AtomKind, GLit, GroundProgram, Prob, simplify};

/// The name of the facts read in place of a choice's atoms.
const NAME: &str = "$cx";

/// The most rules reading a choice whose contexts are compared two by two;
/// above it, the contexts must each read a different atom of one exclusive
/// set.
const MOST_COMPARED: usize = 64;

/// Reads each choice of `gp` that its readers read under exclusive
/// contexts through a fact per reading rule, as described above, and
/// drops the choice; `tables` are `gp`'s. Returns how many choices it read
/// so; where it read any, the atoms are compacted
/// ([`simplify::compact`]) and `tables` are `gp`'s no more.
pub(super) fn choices(gp: &mut GroundProgram, tables: &mut Tables) -> usize {
    let n = gp.num_atoms();
    assert!(
        gp.rules.iter().all(|r| !r.pairs),
        "choices in a program with rules of pairs"
    );
    let mut named = vec![false; n];
    for a in gp.named_atoms() {
        named[a as usize] = true;
    }
    // The rules reading each table's atoms, and whether a rule reads two
    // atoms of one table or one negatively.
    let nt = tables.sets().len() + n;
    let mut readers: Vec<Vec<usize>> = vec![Vec::new(); nt];
    let mut spoilt = vec![false; nt];
    for (ri, r) in gp.rules.iter().enumerate() {
        let mut seen: Vec<usize> = Vec::new();
        for l in &r.body {
            if gp.is_fact(l.atom()) {
                continue;
            }
            let t = tables.of(l.atom());
            if !l.is_positive() || seen.contains(&t) {
                spoilt[t] = true;
            }
            seen.push(t);
            readers[t].push(ri);
        }
    }
    // The choices are decided on the program as it is and written after:
    // a rule is rewritten for one choice at most, since another's contexts
    // may clash through the atom the first replaces.
    let mut taken = vec![false; gp.rules.len()];
    // Per rule rewritten, the place of the atom it reads and its probability.
    let mut rewrites: Vec<(usize, usize, Prob)> = Vec::new();
    let mut drop_rules = vec![false; gp.rules.len()];
    let mut read = 0;
    for t in (0..nt).filter(|&t| !spoilt[t] && !readers[t].is_empty()) {
        let rules = &readers[t];
        if rules.iter().any(|&ri| taken[ri]) {
            continue;
        }
        let Some(probabilities) = choice(gp, tables, t, rules, &named) else {
            continue;
        };
        let member = |ri: usize| {
            gp.rules[ri]
                .body
                .iter()
                .position(|l| !gp.is_fact(l.atom()) && tables.of(l.atom()) == t)
                .expect("a reader reads the table")
        };
        let contexts: Vec<Vec<GLit>> = rules
            .iter()
            .map(|&ri| {
                let k = member(ri);
                let mut c: Vec<GLit> = gp.rules[ri]
                    .body
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| j != k)
                    .map(|(_, &l)| l)
                    .collect();
                c.sort_unstable();
                c
            })
            .collect();
        if !exclusive(gp, tables, &contexts) {
            continue;
        }
        // The literals the atoms' rules held beyond the one fact each reader
        // reads, against the facts added beyond the table's own.
        let saved: usize = rules
            .iter()
            .map(|&ri| {
                gp.rules[tables.by_head()[gp.rules[ri].body[member(ri)].atom() as usize][0]]
                    .body
                    .len()
                    - 1
            })
            .sum();
        if saved < rules.len().saturating_sub(own_facts(gp, tables, t)) {
            continue;
        }
        for &ri in rules {
            let k = member(ri);
            taken[ri] = true;
            rewrites.push((ri, k, probabilities[&gp.rules[ri].body[k].atom()]));
        }
        for m in members(tables, t) {
            for &mr in &tables.by_head()[m as usize] {
                drop_rules[mr] = true;
            }
        }
        read += 1;
    }
    if read == 0 {
        return 0;
    }
    for (ri, k, p) in rewrites {
        if p.p == 0.0 {
            drop_rules[ri] = true;
        } else if p.not == 0.0 {
            gp.rules[ri].body.remove(k);
        } else {
            let f = gp.num_atoms() as AtomId;
            gp.atoms
                .push(Atom::new(NAME, vec![Term::Int(gp.stats.contexts as i64)]));
            gp.kinds.push(AtomKind::Fact(p));
            gp.stats.contexts += 1;
            gp.rules[ri].body[k] = GLit::Pos(f);
        }
    }
    let mut i = 0;
    gp.rules.retain(|_| {
        i += 1;
        !drop_rules[i - 1]
    });
    simplify::compact(gp);
    read
}

/// The atoms of table `t`: an exclusive set's, or the one atom of a table
/// of one ([`Tables::of`]).
fn members(tables: &Tables, t: usize) -> Vec<AtomId> {
    let sets = tables.sets();
    if t < sets.len() {
        sets[t].clone()
    } else {
        vec![(t - sets.len()) as AtomId]
    }
}

/// How many facts are table `t`'s own: what reading its atoms through
/// facts of their own frees.
fn own_facts(gp: &GroundProgram, tables: &Tables, t: usize) -> usize {
    let mut own: FxHashSet<AtomId> = FxHashSet::default();
    for m in members(tables, t) {
        for &mr in &tables.by_head()[m as usize] {
            own.extend(gp.rules[mr].body.iter().map(|l| l.atom()));
        }
    }
    own.len()
}

/// Where table `t` is a choice that `readers` read and no query or
/// evidence names (`named`), the weights of each atom they read: the
/// product over its one rule's literals, each of the table's own facts,
/// and its complement ([`Prob::cube`]).
fn choice(
    gp: &GroundProgram,
    tables: &Tables,
    t: usize,
    readers: &[usize],
    named: &[bool],
) -> Option<rustc_hash::FxHashMap<AtomId, Prob>> {
    let mut probabilities = rustc_hash::FxHashMap::default();
    for m in members(tables, t) {
        if named[m as usize] || gp.kinds[m as usize] != AtomKind::Derived {
            return None;
        }
    }
    for &ri in readers {
        for l in &gp.rules[ri].body {
            let a = l.atom();
            if gp.is_fact(a) || tables.of(a) != t || probabilities.contains_key(&a) {
                continue;
            }
            let &[mr] = &tables.by_head()[a as usize] else {
                return None;
            };
            let mut body = gp.rules[mr].body.clone();
            body.sort_unstable_by_key(|l| (l.atom(), l.is_positive()));
            body.dedup();
            let mut literals = Vec::with_capacity(body.len());
            let mut clash = false;
            for (j, &l) in body.iter().enumerate() {
                let s = l.atom();
                if tables.owner(gp, s) != Some(t) {
                    return None;
                }
                let AtomKind::Fact(q) = gp.kinds[s as usize] else {
                    return None;
                };
                clash |= body.get(j + 1).is_some_and(|m| m.atom() == s);
                literals.push(q.literal(l.is_positive()));
            }
            probabilities.insert(
                a,
                if clash {
                    Prob::FALSE
                } else {
                    Prob::cube(literals)
                },
            );
        }
    }
    Some(probabilities)
}

/// Whether every two of `contexts`, sorted, clash ([`Tables::clash`]): at
/// once where each reads a different atom of one exclusive set positively,
/// two by two where there are at most [`MOST_COMPARED`] of them.
fn exclusive(gp: &GroundProgram, tables: &Tables, contexts: &[Vec<GLit>]) -> bool {
    let sets = tables.sets().len();
    let in_set = |l: &GLit| l.is_positive() && !gp.is_fact(l.atom()) && tables.of(l.atom()) < sets;
    let first: Vec<usize> = contexts[0]
        .iter()
        .filter(|l| in_set(l))
        .map(|l| tables.of(l.atom()))
        .collect();
    for s in first {
        let mut atoms: FxHashSet<AtomId> = FxHashSet::default();
        let distinct = contexts.iter().all(|c| {
            let mut of_s = c.iter().filter(|l| in_set(l) && tables.of(l.atom()) == s);
            matches!((of_s.next(), of_s.next()), (Some(l), None) if atoms.insert(l.atom()))
        });
        if distinct {
            return true;
        }
    }
    contexts.len() <= MOST_COMPARED
        && contexts
            .iter()
            .enumerate()
            .all(|(i, x)| contexts[i + 1..].iter().all(|y| tables.clash(x, y)))
}

#[cfg(test)]
mod tests {
    use crate::GroundOptions;

    fn ground(source: &str, contexts: bool) -> crate::GroundProgram {
        let program = crate::syntax::parse_program(source).expect("parses");
        crate::ground::ground(
            &program,
            &GroundOptions {
                contexts,
                ..GroundOptions::default()
            },
        )
        .expect("grounds")
    }

    /// A choice of four values read once per value of a key: one fact per
    /// value of the key, of the probability of the value it read.
    #[test]
    fn a_choice_under_a_key_is_a_fact_per_value() {
        let source = "0.25::k(0); 0.25::k(1); 0.25::k(2); 0.25::k(3).
            0.1::d(0); 0.2::d(1); 0.3::d(2); 0.4::d(3).
            e :- k(0), d(2). e :- k(1), d(1). e :- k(2), d(0). e :- k(3), d(3).
            evidence(e). query(k(0)). query(k(3)).";
        let gp = ground(source, true);
        assert_eq!(gp.stats.contexts, 4, "{gp}");
        assert!(gp.atoms.iter().all(|a| a.name.as_ref() != "d"), "{gp}");
        assert_eq!(ground(source, false).stats.contexts, 0);
    }

    /// The rows of a network's table read its parent under the rows' own
    /// switches, which do not clash: the parent stays.
    #[test]
    fn a_parent_read_by_rows_stays() {
        let source = "0.3::a(0); 0.7::a(1).
            0.2::c(0); 0.3::c(1); 0.5::c(2) :- a(0).
            0.6::c(0); 0.3::c(1); 0.1::c(2) :- a(1).
            evidence(c(1)). query(a(0)).";
        assert_eq!(ground(source, true).stats.contexts, 0);
    }

    /// Two heads reading one choice under one context would read it twice:
    /// the choice stays.
    #[test]
    fn a_choice_two_rules_read_under_one_context_stays() {
        let source = "0.5::k(0); 0.5::k(1).
            0.1::d(0); 0.2::d(1); 0.3::d(2); 0.4::d(3).
            e(0) :- k(0), d(1). e(1) :- k(0), d(2). e(0) :- k(1), d(3).
            query(e(0)). query(e(1)).";
        assert_eq!(ground(source, true).stats.contexts, 0);
    }
}
