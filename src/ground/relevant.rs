//! The predicates the queries and the evidence read.
//!
//! A predicate that no query, observation or query rule reads, through any
//! chain of rule bodies, holds no atom that an answer depends on. Its
//! rules and its facts are dropped before anything is ground. The ground
//! program keeps only the atoms the queries and the evidence reach in any
//! case ([`super::simplify::restrict_to_relevant`]), but the routes that
//! ground the whole program, level by level
//! ([`super::engine::ground_by_levels`]) or as written, would ground the
//! other predicates' atoms first. A loop over a frame of many variables
//! that is asked about one of them grounds the whole frame at every step:
//! noDice's brp asks whether an acknowledgement is received, which reads
//! about 10 of the frame's 280 predicates at each step.
//!
//! A body reads its positive and negated atoms, the goals of its findalls
//! and the predicates its findalls collect. An annotated disjunction's
//! heads are rules of their own over its chain of switches
//! ([`super::normalize`]): a head that is read reads the switches of the
//! heads before it, and the heads after it are dropped where nothing reads
//! them, as the ground program would drop them.

use super::normalize::{NormProgram, Sig};
use super::terms::calls;
use crate::syntax::Literal;
use rustc_hash::FxHashSet;

/// Drops from `norm` the rules and the facts of the predicates that its
/// queries, its observations and its query rules do not read; the number of
/// rules dropped.
pub(crate) fn keep_relevant(norm: &mut NormProgram) -> usize {
    let (keep, keep_facts) = {
        let rules_of = norm.rules_by_sig();
        let mut read: FxHashSet<Sig> = FxHashSet::default();
        let mut stack: Vec<Sig> = Vec::new();
        let asked = norm.queries.iter().chain(&norm.evidence).map(|(a, _)| a);
        let drivers = norm.drivers.iter().flat_map(|d| [&d.atom, &d.pattern]);
        for a in asked.chain(drivers) {
            note(a.signature(), &mut read, &mut stack);
        }
        while let Some(sig) = stack.pop() {
            for &ri in rules_of.get(&sig).into_iter().flatten() {
                let body = &norm.rules[ri].body;
                calls(body, &mut |a| note(a.signature(), &mut read, &mut stack));
                for l in body {
                    if let Literal::Collect(c) = l {
                        note((&*c.aux, c.arity), &mut read, &mut stack);
                    }
                }
            }
        }
        let keep: Vec<bool> = norm
            .rules
            .iter()
            .map(|c| read.contains(&c.heads[0].atom.signature()))
            .collect();
        let keep_facts: Vec<bool> = norm
            .facts
            .iter()
            .map(|(_, f)| read.contains(&(&*f.name, f.arity)))
            .collect();
        (keep, keep_facts)
    };
    let dropped = keep.iter().filter(|k| !**k).count();
    if dropped > 0 {
        norm.retain_rules(|i, _| keep[i]);
    }
    if keep_facts.contains(&false) {
        let mut kept = keep_facts.into_iter();
        norm.facts
            .retain(|_| kept.next().expect("a flag per predicate"));
    }
    dropped
}

/// Marks `sig` read, to be followed where it is new.
fn note<'a>(sig: Sig<'a>, read: &mut FxHashSet<Sig<'a>>, stack: &mut Vec<Sig<'a>>) {
    if read.insert(sig) {
        stack.push(sig);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::normalize::normalize;
    use crate::ground::terms::lower;
    use crate::syntax::parse_program;

    /// The predicates the query reads through a negation, a findall and an
    /// annotated disjunction's first head keep their rules and facts; a
    /// predicate nothing asked reads, the facts only it reads and the
    /// disjunction's second head are dropped.
    #[test]
    fn what_no_query_reads_is_dropped() {
        let p = parse_program(
            "0.3::a(1). 0.4::a(2).\n\
             0.5::b(1).\n\
             c(X) :- a(X), \\+ d(X).\n\
             d(X) :- b(X).\n\
             n(N) :- findall(X, a(X), L), length(L, N).\n\
             0.2::h(X); 0.3::k(X) :- a(X).\n\
             unread(X) :- b(X), e(X).\n\
             e(1).\n\
             q :- c(1), n(2), h(2).\n\
             query(q).\n",
        )
        .unwrap();
        let (lowered, counted) = lower(&p).unwrap();
        let mut norm = normalize(&lowered, counted).unwrap();
        let before = norm.rules.len();
        let dropped = keep_relevant(&mut norm);
        let mut named: FxHashSet<String> = norm
            .rules
            .iter()
            .map(|r| r.heads[0].atom.name.to_string())
            .collect();
        named.extend(norm.facts.iter().map(|(_, f)| f.name.to_string()));
        for kept in ["q", "c", "d", "n", "h", "a", "b"] {
            assert!(named.contains(kept), "{kept} not in {named:?}");
        }
        for gone in ["unread", "k", "e"] {
            assert!(!named.contains(gone), "{gone} in {named:?}");
        }
        assert_eq!(dropped, before - norm.rules.len());
        assert!(dropped >= 2);
        assert_eq!(keep_relevant(&mut norm), 0);
    }
}
