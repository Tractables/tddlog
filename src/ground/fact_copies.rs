//! Copies of facts that given atoms read in their place, each tied to what
//! it copies by observations.
//!
//! A copy is a fact of probability one half, `$sc(i)`, and its ties two
//! derived atoms, `$st(i, 0) :- $sc(i), \+t.` and `$st(i, 1) :- t,
//! \+$sc(i).`, which the evidence observes false. Under the ties every copy
//! equals the fact it stands for, so a formula that reads a copy where it
//! read the fact is the same function of the program's own facts, and the
//! worlds that satisfy the evidence are those of the program, each with its
//! weight times one half per copy. `P(e)` takes that factor back: the
//! program's [`GroundProgram::weight`] doubles with each copy. The answers
//! to queries, ratios over the same worlds, do not change.
//!
//! The ties are two clauses rather than one atom true where the copy and
//! what it copies differ. The evidence is the same function either way, and
//! minimized it is the same diagram; the walk, which leaves its last join
//! unreduced, does other joins and ends with other unreduced sizes. The two
//! clauses are what the layout's rule was measured with.
//!
//! The rewrite is for a vtree that places a copy elsewhere than the fact,
//! which the program as written would not let a formula read
//! ([`crate::vtree`]'s separator copies). It comes after the conditioning,
//! whose merge of facts it proves equal would take the copies back out.

use std::sync::OnceLock;

use crate::syntax::{Atom, Term};

use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, Prob};

/// The name of the copies' atoms.
const COPY_NAME: &str = "$sc";
/// The name of the ties' atoms, two per copy.
const TIE_NAME: &str = "$st";

/// One copy to make: of `replaces`, a fact or an earlier copy of the same
/// call, which the rules of the atoms `readers` read in its place; the
/// copy is observed equal to `equals`, the same fact or another copy of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FactCopy {
    pub(crate) replaces: AtomId,
    pub(crate) equals: AtomId,
    pub(crate) readers: Vec<AtomId>,
}

/// Make `copies` in `gp`, in order: copy `i` is atom `n + i`, `n` the atoms
/// before the call, so that a later copy can name an earlier one before
/// either exists. In every rule of a reader, a literal that reads what the
/// copy replaces, or reads a copy the conditioning made of it
/// ([`GroundProgram::copies_of_facts`]), reads the new copy instead, with
/// the sign that keeps the literal's value; a reader none of whose rules
/// reads it is an error of the caller. The ties come after the copies, two
/// per copy, and the evidence observes them false. Returns how many copies
/// were made.
pub(crate) fn add_fact_copies(gp: &mut GroundProgram, copies: &[FactCopy]) -> usize {
    if copies.is_empty() {
        return 0;
    }
    let n = gp.num_atoms() as AtomId;
    let copy_of = gp.copies_of_facts();
    let by_head = gp.rules_by_head();
    for i in 0..copies.len() {
        gp.atoms
            .push(Atom::new(COPY_NAME, vec![Term::Int(i as i64)]));
        gp.kinds.push(AtomKind::Fact(Prob::HALF));
    }
    for (i, copy) in copies.iter().enumerate() {
        let new = n + i as AtomId;
        for a in [copy.replaces, copy.equals] {
            assert!(
                a < new && gp.is_fact(a),
                "copy {i} is of a fact or a copy before it"
            );
        }
        for &reader in &copy.readers {
            let mut replaced = false;
            for &ri in &by_head[reader as usize] {
                let body = &mut gp.rules[ri].body;
                let mut changed = false;
                for l in body.iter_mut() {
                    let read = match copy_of.get(l.atom() as usize).copied().flatten() {
                        Some(f) if f.atom() == copy.replaces => {
                            Some(l.is_positive() == f.is_positive())
                        }
                        _ if l.atom() == copy.replaces => Some(l.is_positive()),
                        _ => None,
                    };
                    if let Some(positive) = read {
                        *l = GLit::new(new, positive);
                        changed = true;
                    }
                }
                if changed {
                    body.sort_unstable();
                    body.dedup();
                    replaced = true;
                }
            }
            assert!(
                replaced,
                "reader {reader} of copy {i} reads {}",
                copy.replaces
            );
        }
    }
    for (i, copy) in copies.iter().enumerate() {
        let new = n + i as AtomId;
        for (j, body) in [
            [GLit::Pos(new), GLit::Neg(copy.equals)],
            [GLit::Pos(copy.equals), GLit::Neg(new)],
        ]
        .into_iter()
        .enumerate()
        {
            let (tie, atom) = (
                gp.num_atoms() as AtomId,
                Atom::new(TIE_NAME, vec![Term::Int(i as i64), Term::Int(j as i64)]),
            );
            gp.atoms.push(atom.clone());
            gp.kinds.push(AtomKind::Derived);
            gp.rules.push(GroundRule::new(tie, body.to_vec()));
            gp.evidence.push((atom, Some(tie), false));
        }
    }
    let k = copies.len();
    gp.weight = gp.weight.shifted(k as i64);
    gp.structure = OnceLock::new();
    k
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A program over atoms named `names`, the first `facts` of them facts
    /// of probability `p`, with `rules` (head, body) and `observed` false.
    fn program(
        names: &[&str],
        facts: usize,
        rules: &[(usize, &[GLit])],
        observed: &[usize],
    ) -> GroundProgram {
        let atoms: Vec<Atom> = names.iter().map(|&n| Atom::new(n, Vec::new())).collect();
        let kinds = (0..names.len())
            .map(|i| {
                if i < facts {
                    AtomKind::Fact(crate::ground::Prob::new(0.3))
                } else {
                    AtomKind::Derived
                }
            })
            .collect();
        let rules = rules
            .iter()
            .map(|&(h, b)| GroundRule::new(h as AtomId, b.to_vec()))
            .collect();
        let evidence = observed
            .iter()
            .map(|&a| (atoms[a].clone(), Some(a as AtomId), false))
            .collect();
        GroundProgram {
            atoms,
            kinds,
            rules,
            evidence,
            ..GroundProgram::default()
        }
    }

    fn body(gp: &GroundProgram, h: AtomId) -> Vec<GLit> {
        gp.rules.iter().find(|r| r.head == h).unwrap().body.clone()
    }

    /// A copy replaces the target in its readers' rules alone, its ties
    /// are observed false, and the weight doubles.
    #[test]
    fn a_copy_replaces_its_target_in_its_readers() {
        use GLit::{Neg, Pos};
        // a b c; c1 :- a, \+b. c2 :- \+a, c.
        let mut gp = program(
            &["a", "b", "c", "c1", "c2"],
            3,
            &[(3, &[Pos(0), Neg(1)]), (4, &[Neg(0), Pos(2)])],
            &[3, 4],
        );
        let made = add_fact_copies(
            &mut gp,
            &[FactCopy {
                replaces: 0,
                equals: 0,
                readers: vec![4],
            }],
        );
        assert_eq!(made, 1);
        assert!(gp.is_fact(5));
        assert_eq!(gp.atoms[5].to_string(), "'$sc'(0)");
        assert_eq!(
            body(&gp, 3),
            vec![Pos(0), Neg(1)],
            "a reader not asked keeps the fact"
        );
        assert_eq!(body(&gp, 4), vec![Pos(2), Neg(5)]);
        assert_eq!(
            (body(&gp, 6), body(&gp, 7)),
            (vec![Pos(5), Neg(0)], vec![Pos(0), Neg(5)])
        );
        assert_eq!(gp.atoms[6].to_string(), "'$st'(0,0)");
        for tie in [6, 7] {
            assert!(gp.evidence.iter().any(|&(_, t, v)| t == Some(tie) && !v));
        }
        assert_eq!(gp.weight.to_f64(), 2.0);
    }

    /// A chain: the second copy is of the first, and a reader of a
    /// conditioning copy of the fact reads the new copy with the sign that
    /// keeps the literal's value.
    #[test]
    fn copies_chain_and_read_through_conditioning_copies() {
        use GLit::{Neg, Pos};
        // a b c; g :- \+a. c1 :- g, b. c2 :- \+g, c.
        let mut gp = program(
            &["a", "b", "c", "g", "c1", "c2"],
            3,
            &[
                (3, &[Neg(0)]),
                (4, &[Pos(3), Pos(1)]),
                (5, &[Neg(3), Pos(2)]),
            ],
            &[4, 5],
        );
        add_fact_copies(
            &mut gp,
            &[
                FactCopy {
                    replaces: 0,
                    equals: 0,
                    readers: vec![4, 5],
                },
                FactCopy {
                    replaces: 6,
                    equals: 6,
                    readers: vec![5],
                },
            ],
        );
        // c1 read g = \+a, so it reads \+$sc(0); c2 read \+g = a, so
        // $sc(0), and then $sc(1) in its place.
        assert_eq!(body(&gp, 4), vec![Pos(1), Neg(6)]);
        assert_eq!(body(&gp, 5), vec![Pos(2), Pos(7)]);
        assert_eq!(body(&gp, 3), vec![Neg(0)], "the conditioning copy stays");
        // The second copy's ties, after the first's two, are to the first.
        assert_eq!(
            (body(&gp, 10), body(&gp, 11)),
            (vec![Pos(7), Neg(6)], vec![Pos(6), Neg(7)])
        );
        assert_eq!(gp.weight.to_f64(), 4.0);
    }
}
