//! Arguments no clause reads.
//!
//! An argument of a predicate that every clause of it leaves as a variable
//! the clause mentions nowhere else, as the `_Der` of cplint's left-corner
//! grammar `lc('S', a, _Der, rule('S', [a]))`, does not decide which atoms
//! hold: an atom holds with one term there exactly when it holds with any
//! other, in every world. The argument is dropped, from the predicate's
//! heads and from every literal that reads it, until no argument
//! qualifies: a clause whose variable only such an argument read leaves it
//! to its own head. The non-ground facts become ground ones, which
//! [`super::magic`] reads as they are, and a negation of them, the
//! grammar's `\+ lc(G, B, Der, rule(G, [B|_]))`, no longer depends on the
//! derivation `Der`, whose every value it would otherwise be asked for.
//!
//! A switch predicate, whose arguments name its choices, a predicate asked
//! or observed, and one that a findall reads keep their arguments; so does
//! a predicate whose name is in use with the arity it would get.

use super::normalize::{NormProgram, Sig};
use crate::syntax::{Atom, Clause, Literal, Name, Term};
use rustc_hash::{FxHashMap, FxHashSet};

/// `norm` with the arguments no clause reads dropped, or `None` where
/// there are none.
pub(crate) fn drop_unread(norm: &NormProgram) -> Option<NormProgram> {
    let mut out: Option<NormProgram> = None;
    loop {
        let cur = out.as_ref().unwrap_or(norm);
        let drops = unread(cur);
        if drops.is_empty() {
            return out;
        }
        let atom = |a: &Atom| match drops.get(&(a.name.clone(), a.args.len())) {
            Some(keep) => Atom::new(
                a.name.clone(),
                a.args
                    .iter()
                    .zip(keep)
                    .filter(|(_, k)| **k)
                    .map(|(t, _)| t.clone())
                    .collect::<Vec<Term>>(),
            ),
            None => a.clone(),
        };
        let mut next = cur.clone();
        for rule in &mut next.rules {
            for h in &mut rule.heads {
                h.atom = atom(&h.atom);
            }
            for l in &mut rule.body {
                if let Literal::Pos(a) | Literal::Neg(a) = l {
                    *a = atom(a);
                }
            }
        }
        out = Some(next);
    }
}

/// Whether argument `i` of the head of `rule`, its only head, is a
/// variable that the clause mentions nowhere else.
fn lone(rule: &Clause, i: usize) -> bool {
    let [head] = &rule.heads[..] else {
        return false;
    };
    let Term::Var(v) = &head.atom.args[i] else {
        return false;
    };
    let mut others = Vec::new();
    for (j, u) in head.atom.args.iter().enumerate() {
        if j != i {
            u.vars_into(&mut others);
        }
    }
    for l in &rule.body {
        l.vars_into(&mut others);
    }
    !head.negated && !others.contains(v)
}

/// The predicates of `norm` with an argument no clause reads, each with
/// the arguments it keeps.
fn unread(norm: &NormProgram) -> FxHashMap<(Name, usize), Vec<bool>> {
    let mut drops = FxHashMap::default();
    // Most programs have no such argument, and the facts of large ones are
    // ground: told without the predicates' rules listed.
    if !norm
        .rules
        .iter()
        .any(|r| (0..r.heads[0].atom.args.len()).any(|i| lone(r, i)))
    {
        return drops;
    }
    let mut fixed: FxHashSet<Sig> = norm.switches.keys().map(|(n, k)| (&**n, *k)).collect();
    let asked = norm.queries.iter().chain(&norm.evidence).map(|(a, _)| a);
    let drivers = norm.query_drivers.iter().flat_map(|(d, p, _)| [d, p]);
    fixed.extend(asked.chain(drivers).map(Atom::signature));
    fixed.extend(norm.counted.iter().map(|(n, k)| (&**n, *k)));
    for rule in &norm.rules {
        for l in &rule.body {
            if let Literal::Collect(c) = l {
                fixed.insert((&c.aux, c.arity));
            }
        }
    }
    // The names in use with each arity, and those the drops take.
    let mut used = signatures(norm);
    for (sig, rules) in norm.rules_by_sig() {
        if fixed.contains(&sig) {
            continue;
        }
        let keep: Vec<bool> = (0..sig.1)
            .map(|i| !rules.iter().all(|&ri| lone(&norm.rules[ri], i)))
            .collect();
        let arity = keep.iter().filter(|k| **k).count();
        if arity < sig.1 && used.insert((sig.0, arity)) {
            drops.insert((sig.0.into(), sig.1), keep);
        }
    }
    drops
}

/// Every predicate `norm` names.
fn signatures(norm: &NormProgram) -> FxHashSet<Sig<'_>> {
    let mut used: FxHashSet<Sig> = norm.switches.keys().map(|(n, k)| (&**n, *k)).collect();
    for rule in &norm.rules {
        used.extend(rule.heads.iter().map(|h| h.atom.signature()));
        for l in &rule.body {
            if let Literal::Pos(a) | Literal::Neg(a) = l {
                used.insert(a.signature());
            }
        }
    }
    let asked = norm.queries.iter().chain(&norm.evidence).map(|(a, _)| a);
    let drivers = norm.query_drivers.iter().flat_map(|(d, p, _)| [d, p]);
    used.extend(asked.chain(drivers).map(Atom::signature));
    used
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::normalize::normalize;
    use crate::syntax::parse_program;

    /// The derivation a fact leaves free is dropped, and with it the
    /// argument of the predicate whose clause only that fact read; a switch
    /// keeps the argument that names its choices, and a clause that reads
    /// the argument keeps it.
    #[test]
    fn an_argument_no_clause_reads_is_dropped() {
        let p = parse_program(
            "lc(s, a, _D, r(a)).\n\
             lc(s, b, _E, r(b)).\n\
             nt(G, B, D) :- lc(G, B, D, _).\n\
             0.5::first(s, D, a); 0.5::first(s, D, b) :- step(D).\n\
             step(0).\n\
             step(N) :- step(M), M < 2, N is M + 1, \\+ nt(s, a, N).\n\
             query(first(s, 1, a)).\n",
        )
        .unwrap();
        let norm = normalize(&p).unwrap();
        let dropped = drop_unread(&norm).unwrap();
        let text: Vec<String> = dropped.rules.iter().map(|r| r.to_string()).collect();
        assert!(text.contains(&"lc(s,a,r(a)).".to_string()), "{text:?}");
        assert!(
            text.iter().any(|r| r.starts_with("nt(G,B) :- lc(G,B,")),
            "{text:?}"
        );
        assert!(text.iter().any(|r| r.ends_with("\\+nt(s,a).")), "{text:?}");
        assert!(
            text.iter()
                .any(|r| r.starts_with("first(s,D,a) :- step(D)")),
            "{text:?}"
        );
        assert!(text.contains(&"step(0).".to_string()), "{text:?}");
        assert!(drop_unread(&dropped).is_none());
    }
}
