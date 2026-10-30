//! Right-linear recursion evaluated from its bound end.
//!
//! A predicate `p` whose rules are exit rules `p(X̄,Ȳ) :- E` and right-linear
//! rules `p(X̄,Ȳ) :- L, p(Z̄,Ȳ)`, in which the arguments `Ȳ` pass through the
//! recursive call unchanged and occur nowhere else in the rule, holds
//! `p(x̄,ȳ)` exactly when a chain of `L` steps leads from `x̄` to some `w̄`
//! with `E(w̄,ȳ)`. When every use of `p` outside its own rules has ground
//! arguments `c̄` in the other positions, the chains from those `c̄` are all
//! that is asked for:
//!
//! ```text
//! r(c̄,c̄).                  for each such c̄
//! r(S̄,Z̄) :- r(S̄,X̄), L.     for each right-linear rule
//! p(S̄,Ȳ) :- r(S̄,X̄), E.     for each exit rule
//! ```
//!
//! This is the factoring of Naughton, Ramakrishnan, Sagiv and Ullman
//! ("Argument reduction by factoring", VLDB 1989). It holds in every world,
//! the switches keeping their names and arguments, so every probability is
//! preserved. A transitive closure written right-recursively and queried
//! from one node, `path(1,Y)`, needs `path(X,Y)` for every node `X` on a path
//! to a queried `Y` when grounded as written; rewritten it needs the nodes
//! reachable from 1 and one `path(1,Y)` for each. The caller grounds both
//! programs and keeps the smaller.
//!
//! Only predicates that form a recursive component of their own qualify, and
//! only when no recursive component below them has internal negation, so the
//! steps `L` and exits `E` are two-valued in every world. A predicate whose
//! rules have no variables does not: it is ground as written, and its atoms
//! on the chains from the bound ends are the ones the relevance cut keeps of
//! it. Rewritten, they would be ground again once per bound end, never fewer
//! of them; and the chain from the bound end, evaluated against rules
//! written from the other end, takes a round of the grounder's semi-naive
//! evaluation per step, each round visiting every rule (Roulette's diamond
//! of 3 000 steps took 5 s).

use rustc_hash::{FxHashMap, FxHashSet};

use super::magic::ground_clause;
use super::normalize::{NormProgram, Sig};
use super::schedule::literal_vars;
use crate::syntax::{Args, Atom, Clause, Literal, Term};

/// Names the predicate of the chains from the bound arguments. Cannot clash
/// with a user predicate, whose name never contains `$`.
const REACH_PREFIX: &str = "$r";

/// The rewritten program with the predicates it rewrote, as `name/arity`
/// in name order, or `None` when no predicate qualifies.
pub(crate) fn rewrite(norm: &NormProgram) -> Option<(NormProgram, Vec<String>)> {
    let rules_of = norm.rules_by_sig();
    let plans: Vec<Plan> = rules_of
        .iter()
        .filter_map(|(sig, rules)| plan(norm, &rules_of, sig, rules))
        .collect();
    if plans.is_empty() {
        return None;
    }
    let mut out = norm.clone();
    let rewritten: FxHashSet<Sig> = plans.iter().map(|p| p.sig).collect();
    out.retain_rules(|_, c| !rewritten.contains(&c.heads[0].atom.signature()));
    for plan in &plans {
        plan.emit(norm, &mut out.rules);
    }
    let mut names: Vec<String> = plans
        .iter()
        .map(|p| format!("{}/{}", p.sig.0, p.sig.1))
        .collect();
    names.sort_unstable();
    Some((out, names))
}

/// What the rewrite of one predicate needs: which argument positions pass
/// through, and the ground tuples its uses bind the others to.
struct Plan<'a> {
    sig: Sig<'a>,
    rules: Vec<usize>,
    /// `pass[i]`: position `i` passes through the recursive call.
    pass: Vec<bool>,
    sources: Vec<Vec<Term>>,
}

impl Plan<'_> {
    fn bound<'a>(&'a self, args: &'a [Term]) -> impl Iterator<Item = &'a Term> + 'a {
        args.iter()
            .zip(&self.pass)
            .filter(|(_, p)| !**p)
            .map(|(t, _)| t)
    }

    fn reach(&self, from: &[Term], to: impl Iterator<Item = Term>) -> Atom {
        let name = format!("{REACH_PREFIX}_{}_{}", self.sig.0, self.sig.1);
        Atom::new(name, from.iter().cloned().chain(to).collect::<Args>())
    }

    fn emit(&self, norm: &NormProgram, out: &mut Vec<Clause>) {
        let width = self.pass.iter().filter(|p| !**p).count();
        let starts: Vec<Term> = (0..width).map(|i| Term::Var(format!("$S{i}"))).collect();
        for c in &self.sources {
            out.push(Clause::rule(
                self.reach(c, c.iter().cloned()),
                Vec::new(),
                u32::MAX,
            ));
        }
        for &ri in &self.rules {
            let clause = &norm.rules[ri];
            let head = &clause.heads[0].atom;
            let from = Literal::Pos(self.reach(&starts, self.bound(&head.args).cloned()));
            match clause
                .body
                .iter()
                .position(|l| matches!(l, Literal::Pos(a) if a.signature() == self.sig))
            {
                Some(at) => {
                    let Literal::Pos(call) = &clause.body[at] else {
                        unreachable!()
                    };
                    let to = self.reach(&starts, self.bound(&call.args).cloned());
                    let mut body = vec![from];
                    body.extend(
                        clause
                            .body
                            .iter()
                            .enumerate()
                            .filter(|&(i, _)| i != at)
                            .map(|(_, l)| l.clone()),
                    );
                    out.push(Clause::rule(to, body, clause.group));
                }
                None => {
                    let mut start = starts.iter();
                    let args = head
                        .args
                        .iter()
                        .zip(&self.pass)
                        .map(|(t, &p)| {
                            if p {
                                t.clone()
                            } else {
                                start.next().unwrap().clone()
                            }
                        })
                        .collect::<Args>();
                    let mut body = vec![from];
                    body.extend(clause.body.iter().cloned());
                    out.push(Clause::rule(
                        Atom::new(head.name.clone(), args),
                        body,
                        clause.group,
                    ));
                }
            }
        }
    }
}

fn plan<'a>(
    norm: &'a NormProgram,
    rules_of: &FxHashMap<Sig<'a>, Vec<usize>>,
    sig: &Sig<'a>,
    rules: &[usize],
) -> Option<Plan<'a>> {
    // The rules: exits and right-linear rules, at least one of the latter.
    let mut recursive: Vec<(&Atom, &Atom, &Clause)> = Vec::new();
    for &ri in rules {
        let c = &norm.rules[ri];
        let mut calls = c
            .body
            .iter()
            .filter(|l| matches!(l, Literal::Pos(a) | Literal::Neg(a) if a.signature() == *sig));
        match (calls.next(), calls.next()) {
            (None, _) => {}
            (Some(Literal::Pos(call)), None) => recursive.push((&c.heads[0].atom, call, c)),
            _ => return None,
        }
    }
    if recursive.is_empty() {
        return None;
    }
    // Rules without variables are ground as written (the module's doc).
    if rules.iter().all(|&ri| ground_clause(&norm.rules[ri])) {
        return None;
    }
    // Pass-through positions: the same variable in the head and the call, in
    // no other argument of either and nowhere else in the rule.
    let pass: Vec<bool> = (0..sig.1)
        .map(|i| {
            recursive.iter().all(
                |&(head, call, clause)| match (&head.args[i], &call.args[i]) {
                    (Term::Var(v), Term::Var(w)) if v == w => {
                        let t = &head.args[i];
                        head.args.iter().filter(|a| *a == t).count() == 1
                            && call.args.iter().filter(|a| *a == t).count() == 1
                            && clause
                                .body
                                .iter()
                                .filter(|l| !matches!(l, Literal::Pos(a) if std::ptr::eq(a, call)))
                                .all(|l| !literal_vars(l).contains(v))
                    }
                    _ => false,
                },
            )
        })
        .collect();
    if pass.iter().all(|p| *p) {
        return None;
    }
    if !own_component(norm, rules_of, sig) {
        return None;
    }
    // Every other use binds the other positions to ground terms.
    let mut sources: Vec<Vec<Term>> = Vec::new();
    let mut used = |a: &Atom| -> bool {
        if a.signature() != *sig {
            return true;
        }
        let bound: Vec<Term> = a
            .args
            .iter()
            .zip(&pass)
            .filter(|(_, p)| !**p)
            .map(|(t, _)| t.clone())
            .collect();
        if bound.iter().any(Term::is_var) {
            return false;
        }
        if !sources.contains(&bound) {
            sources.push(bound);
        }
        true
    };
    for (i, c) in norm.rules.iter().enumerate() {
        if rules.contains(&i) {
            continue;
        }
        for l in &c.body {
            if let Literal::Pos(a) | Literal::Neg(a) = l
                && !used(a)
            {
                return None;
            }
        }
    }
    for (a, _) in norm.queries.iter().chain(&norm.evidence) {
        if !used(a) {
            return None;
        }
    }
    for pattern in norm.drivers.iter().map(|d| &d.pattern) {
        if !used(pattern) {
            return None;
        }
    }
    if sources.is_empty() {
        return None;
    }
    Some(Plan {
        sig: *sig,
        rules: rules.to_vec(),
        pass,
        sources,
    })
}

/// Whether `sig` is a recursive component of its own, with no recursive
/// component through negation anywhere below it.
pub(super) fn own_component<'a>(
    norm: &'a NormProgram,
    rules_of: &FxHashMap<Sig<'a>, Vec<usize>>,
    sig: &Sig<'a>,
) -> bool {
    // Each predicate's calls, distinct: the rules of a predicate repeat the
    // same few calls, and a walk below each repeat was quadratic in them.
    let calls = |s: &Sig<'a>| -> Vec<(Sig<'a>, bool)> {
        let mut calls: Vec<(Sig<'a>, bool)> = rules_of.get(s).map_or(Vec::new(), |rules| {
            rules
                .iter()
                .flat_map(|&ri| norm.rules[ri].body.iter())
                .filter_map(|l| match l {
                    Literal::Pos(a) => Some((a.signature(), false)),
                    Literal::Neg(a) => Some((a.signature(), true)),
                    _ => None,
                })
                .collect()
        });
        calls.sort_unstable();
        calls.dedup();
        calls
    };
    let below = |from: &Sig<'a>| -> FxHashSet<Sig<'a>> {
        let mut seen: FxHashSet<Sig> = FxHashSet::default();
        let mut stack: Vec<Sig> = calls(from).into_iter().map(|(s, _)| s).collect();
        while let Some(s) = stack.pop() {
            if seen.insert(s) {
                stack.extend(calls(&s).into_iter().map(|(s, _)| s));
            }
        }
        seen
    };
    let cone: FxHashSet<Sig> = calls(sig)
        .into_iter()
        .map(|(s, _)| s)
        .filter(|s| s != sig)
        .flat_map(|s| {
            let mut b = below(&s);
            b.insert(s);
            b
        })
        .collect();
    if cone.contains(sig) {
        return false;
    }
    // A negative call from `s` to `t` inside one component: `t` reaches `s`.
    cone.iter().all(|s| {
        calls(s)
            .into_iter()
            .all(|(t, negative)| !negative || !below(&t).contains(s) && t != *s)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::normalize::normalize;
    use crate::syntax::parse_program;

    fn rewritten(src: &str) -> Option<Vec<String>> {
        let n = normalize(&parse_program(src).unwrap(), Default::default()).unwrap();
        rewrite(&n).map(|(m, _)| m.rules.iter().map(|r| r.to_string()).collect())
    }

    #[test]
    fn closure_from_a_source() {
        let text = rewritten(
            "0.5::e(1,2). 0.5::e(2,3).\npath(X,Y) :- e(X,Y).\npath(X,Y) :- e(X,Z), path(Z,Y).\nquery(path(1,Y)).\n",
        )
        .unwrap();
        assert!(text.contains(&"'$r_path_2'(1,1).".to_string()), "{text:?}");
        assert!(
            text.contains(&"'$r_path_2'($S0,Z) :- '$r_path_2'($S0,X), e(X,Z).".to_string()),
            "{text:?}"
        );
        assert!(
            text.contains(&"path($S0,Y) :- '$r_path_2'($S0,X), e(X,Y).".to_string()),
            "{text:?}"
        );
        assert!(!text.iter().any(|t| t.contains("path(Z,Y)")), "{text:?}");
    }

    #[test]
    fn closure_to_a_target() {
        let text = rewritten(
            "0.5::e(1,2).\np(X,Y) :- e(X,Y).\np(X,Y) :- p(X,Z), e(Z,Y).\nquery(p(X,2)).\n",
        )
        .unwrap();
        assert!(text.contains(&"'$r_p_2'(2,2).".to_string()), "{text:?}");
        assert!(
            text.contains(&"'$r_p_2'($S0,Z) :- '$r_p_2'($S0,Y), e(Z,Y).".to_string()),
            "{text:?}"
        );
        assert!(
            text.contains(&"p(X,$S0) :- '$r_p_2'($S0,Y), e(X,Y).".to_string()),
            "{text:?}"
        );
    }

    #[test]
    fn declines() {
        let closure = "0.5::e(1,2).\npath(X,Y) :- e(X,Y).\npath(X,Y) :- e(X,Z), path(Z,Y).\n";
        // A query with the source unbound.
        assert!(rewritten(&format!("{closure}query(path(X,2)).\n")).is_none());
        // A use elsewhere with the source unbound.
        assert!(rewritten(&format!("{closure}q(X) :- path(X,2).\nquery(q(1)).\n")).is_none());
        // Two recursive calls.
        assert!(
            rewritten(
                "0.5::e(1,2).\np(X,Y) :- e(X,Y).\np(X,Y) :- p(X,Z), p(Z,Y).\nquery(p(1,Y)).\n"
            )
            .is_none()
        );
        // Mutual recursion.
        assert!(rewritten("0.5::e(1,2).\np(X,Y) :- e(X,Y).\np(X,Y) :- e(X,Z), q(Z,Y).\nq(X,Y) :- p(X,Y).\nquery(p(1,Y)).\n").is_none());
        // The passed argument read by a step.
        assert!(rewritten("0.5::e(1,2).\np(X,Y) :- e(X,Y).\np(X,Y) :- e(X,Z), e(Z,Y), p(Z,Y).\nquery(p(1,Y)).\n").is_none());
        // A chain of ground rules, ground as written.
        assert!(rewritten("0.5::r(1). 0.5::r(2).\ns(0).\ns(1) :- s(0), r(1).\ns(2) :- s(1), r(2).\nquery(s(2)).\n").is_none());
    }

    #[test]
    fn the_rewrite_grounds_within_a_bound() {
        use crate::ground::engine::{GroundError, ground_normalized};
        let n = normalize(
            &parse_program("0.5::e(1,2). 0.5::e(2,3). 0.5::e(3,1).\npath(X,Y) :- e(X,Y).\npath(X,Y) :- e(X,Z), path(Z,Y).\nquery(path(1,Y)).\n")
                .unwrap(),
            Default::default(),
        )
        .unwrap();
        let (linear, _) = rewrite(&n).unwrap();
        let size = ground_normalized(&linear, None).unwrap().size();
        assert!(ground_normalized(&linear, Some(size)).is_ok());
        assert!(matches!(
            ground_normalized(&linear, Some(size - 1)),
            Err(GroundError::OverBound)
        ));
    }
}
