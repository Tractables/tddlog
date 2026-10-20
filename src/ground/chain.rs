//! Doubly recursive transitive closures written linear.
//!
//! A predicate `p/2` whose rules are exits `p(s,t) :- B`, whose bodies do not
//! read `p`, and the chain rule `p(X,Y) :- p(X,Z), p(Z,Y)` holds in every
//! world the transitive closure `R⁺` of the relation `R` its exits define:
//! `R⁺` is the least relation that contains `R` and is closed under
//! composition with itself, and also the least that contains `R` and is
//! closed under composition with `R`. So the chain rule can give way, for
//! each exit, to the step
//!
//! ```text
//! p(s,W) :- B, p(t,W).        (right-linear, R⁺ = R ∪ R∘R⁺)
//! p(W,t) :- p(W,s), B.        (left-linear,  R⁺ = R ∪ R⁺∘R)
//! ```
//!
//! with `W` a variable of its own. The body keeps the exit's switches, so a
//! step reads the event that makes its exit hold, and every probability is
//! preserved. This is the linearization of a doubly recursive chain program
//! (Zhang, Yu and Troy, "Necessary and sufficient conditions to linearize
//! doubly recursive programs in logic databases", TODS 1990). The chain rule
//! leaves `p` to the compile's Tp iteration over every pair of nodes; written
//! linear, `p` is what [`super::linear`] evaluates from its bound end and
//! [`super::eliminate`] writes without recursion.
//!
//! The direction follows the uses of `p` outside its rules: left-linear when
//! every use binds the second argument to a ground term and some use leaves
//! the first unbound, right-linear otherwise, so that the factoring of
//! [`super::linear`] finds the bound end. Only a predicate that is a
//! recursive component of its own qualifies, with no recursive component
//! through negation below it, so that `R` is two-valued in every world.

use super::linear::own_component;
use super::normalize::{NormProgram, Sig};
use super::schedule::literal_vars;
use crate::syntax::{Atom, Clause, Literal, Term};

/// The program with every qualifying chain rule replaced by linear steps,
/// or `None` when no predicate qualifies.
pub(crate) fn linearize(norm: &NormProgram) -> Option<NormProgram> {
    let rules_of = norm.rules_by_sig();
    let mut chains: Vec<(usize, Vec<usize>, bool)> = Vec::new();
    for (sig, rules) in &rules_of {
        if sig.1 != 2 {
            continue;
        }
        let (chain, exits): (Vec<usize>, Vec<usize>) = rules
            .iter()
            .partition(|&&ri| is_chain(&norm.rules[ri], sig));
        if chain.is_empty() || exits.is_empty() {
            continue;
        }
        if exits.iter().any(|&ri| reads(&norm.rules[ri], sig))
            || !own_component(norm, &rules_of, sig)
        {
            continue;
        }
        let left = bound_second_only(norm, rules, sig);
        chains.extend(chain.into_iter().map(|ri| (ri, exits.clone(), left)));
    }
    if chains.is_empty() {
        return None;
    }
    let mut out = norm.clone();
    let mut replaced = vec![false; norm.rules.len()];
    for (ri, _, _) in &chains {
        replaced[*ri] = true;
    }
    out.rules = norm
        .rules
        .iter()
        .zip(&replaced)
        .filter(|(_, r)| !**r)
        .map(|(c, _)| c.clone())
        .collect();
    for (ri, exits, left) in &chains {
        // One set of steps for the predicate, however many chain rules it has.
        if chains.iter().any(|(rj, e, _)| rj < ri && e == exits) {
            continue;
        }
        for &ei in exits {
            out.rules.push(step(&norm.rules[ei], *left));
        }
    }
    Some(out)
}

/// Whether `c` is `p(X,Y) :- p(X,Z), p(Z,Y)`, its body in either order, with
/// `X`, `Y` and `Z` distinct variables.
fn is_chain(c: &Clause, sig: &Sig) -> bool {
    let head = &c.heads[0].atom;
    let [Term::Var(x), Term::Var(y)] = head.args.as_slice() else {
        return false;
    };
    if x == y
        || c.heads.len() != 1
        || c.heads[0].negated
        || c.heads[0].prob.is_some()
        || c.body.len() != 2
    {
        return false;
    }
    let pair = |l: &Literal| match l {
        Literal::Pos(a) if a.signature() == *sig => match a.args.as_slice() {
            [Term::Var(u), Term::Var(v)] => Some((u.clone(), v.clone())),
            _ => None,
        },
        _ => None,
    };
    let (Some((a0, a1)), Some((b0, b1))) = (pair(&c.body[0]), pair(&c.body[1])) else {
        return false;
    };
    let joins = |(f0, f1): (&String, &String), (s0, s1): (&String, &String)| {
        f0 == x && s1 == y && f1 == s0 && f1 != x && f1 != y
    };
    joins((&a0, &a1), (&b0, &b1)) || joins((&b0, &b1), (&a0, &a1))
}

/// Whether a body literal of `c` reads `sig`.
fn reads(c: &Clause, sig: &Sig) -> bool {
    c.body
        .iter()
        .any(|l| matches!(l, Literal::Pos(a) | Literal::Neg(a) if a.signature() == *sig))
}

/// Whether every use of `sig` outside `rules` binds its second argument to
/// a ground term and some use leaves its first unbound.
fn bound_second_only(norm: &NormProgram, rules: &[usize], sig: &Sig) -> bool {
    let mut uses: Vec<&Atom> = Vec::new();
    for (i, c) in norm.rules.iter().enumerate() {
        if rules.contains(&i) {
            continue;
        }
        for l in &c.body {
            if let Literal::Pos(a) | Literal::Neg(a) = l
                && a.signature() == *sig
            {
                uses.push(a);
            }
        }
    }
    uses.extend(
        norm.queries
            .iter()
            .chain(&norm.evidence)
            .map(|(a, _)| a)
            .filter(|a| a.signature() == *sig),
    );
    uses.extend(
        norm.query_drivers
            .iter()
            .map(|(_, a, _)| a)
            .filter(|a| a.signature() == *sig),
    );
    let ground = |t: &Term| !t.is_var();
    uses.iter().all(|a| ground(&a.args[1])) && uses.iter().any(|a| !ground(&a.args[0]))
}

/// The linear step of the exit `p(s,t) :- B`.
fn step(exit: &Clause, left: bool) -> Clause {
    let head = &exit.heads[0].atom;
    let (s, t) = (head.args[0].clone(), head.args[1].clone());
    let mut name = String::from("$W");
    let taken = |n: &str| {
        let v = Term::Var(n.to_string());
        head.args.contains(&v)
            || exit
                .body
                .iter()
                .any(|l| literal_vars(l).contains(&n.to_string()))
    };
    while taken(&name) {
        name.push('W');
    }
    let w = Term::Var(name);
    let p = |a: Term, b: Term| Atom::new(head.name.clone(), vec![a, b]);
    let (new_head, call) = if left {
        (p(w.clone(), t), p(w, s))
    } else {
        (p(s, w.clone()), p(t, w))
    };
    let mut body = exit.body.clone();
    if left {
        body.insert(0, Literal::Pos(call));
    } else {
        body.push(Literal::Pos(call));
    }
    Clause::rule(new_head, body, exit.group)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::normalize::normalize;
    use crate::syntax::parse_program;

    fn linearized(src: &str) -> Option<Vec<String>> {
        let n = normalize(&parse_program(src).unwrap()).unwrap();
        linearize(&n).map(|m| m.rules.iter().map(|r| r.to_string()).collect())
    }

    const CLOSURE: &str =
        "0.5::e(1,2). 0.5::e(2,3). 0.5::e(3,1).\np(X,Y) :- e(X,Y).\np(X,Y) :- p(X,Z), p(Z,Y).\n";

    #[test]
    fn a_chain_rule_from_a_source_goes_right_linear() {
        let text = linearized(&format!("{CLOSURE}query(p(1,Y)).\n")).unwrap();
        assert!(
            text.contains(&"p(X,$W) :- e(X,Y), p(Y,$W).".to_string()),
            "{text:?}"
        );
        assert!(text.contains(&"p(X,Y) :- e(X,Y).".to_string()), "{text:?}");
        assert!(
            !text.iter().any(|t| t.contains("p(X,Z), p(Z,Y)")),
            "{text:?}"
        );
    }

    #[test]
    fn a_chain_rule_to_a_target_goes_left_linear() {
        let text = linearized(&format!("{CLOSURE}query(p(X,2)).\n")).unwrap();
        assert!(
            text.contains(&"p($W,Y) :- p($W,X), e(X,Y).".to_string()),
            "{text:?}"
        );
    }

    #[test]
    fn the_body_order_of_the_chain_does_not_matter() {
        let text = linearized(
            "0.5::e(1,2).\np(A,B) :- e(A,B).\np(A,B) :- p(C,B), p(A,C).\nquery(p(1,B)).\n",
        )
        .unwrap();
        assert!(
            text.contains(&"p(A,$W) :- e(A,B), p(B,$W).".to_string()),
            "{text:?}"
        );
    }

    #[test]
    fn every_exit_gives_a_step_and_a_probabilistic_exit_keeps_its_switch() {
        let text = linearized(
            "0.5::e(1,2). 0.5::f(2,3).\np(X,Y) :- e(X,Y).\n0.3::p(X,Y) :- f(Y,X).\np(a,b).\np(X,Y) :- p(X,Z), p(Z,Y).\nquery(p(1,Y)).\n",
        )
        .unwrap();
        assert!(
            text.contains(&"p(X,$W) :- e(X,Y), p(Y,$W).".to_string()),
            "{text:?}"
        );
        assert!(
            text.contains(&"p(a,$W) :- p(b,$W).".to_string()),
            "{text:?}"
        );
        let switched = text
            .iter()
            .find(|t| t.starts_with("p(X,$W) :- f(Y,X)"))
            .expect("the probabilistic exit's step");
        assert!(switched.contains("sw"), "{switched}");
    }

    #[test]
    fn declines() {
        // Another rule reads p besides the chain rule.
        assert!(linearized("0.5::e(1,2).\np(X,Y) :- e(X,Y).\np(X,Y) :- e(X,Z), p(Z,Y).\np(X,Y) :- p(X,Z), p(Z,Y).\nquery(p(1,Y)).\n").is_none());
        // The chain rule with another literal.
        assert!(linearized("0.5::e(1,2). 0.5::g(2).\np(X,Y) :- e(X,Y).\np(X,Y) :- p(X,Z), g(Z), p(Z,Y).\nquery(p(1,Y)).\n").is_none());
        // Not a chain: the join is on the wrong argument.
        assert!(
            linearized(
                "0.5::e(1,2).\np(X,Y) :- e(X,Y).\np(X,Y) :- p(X,Z), p(Y,Z).\nquery(p(1,Y)).\n"
            )
            .is_none()
        );
        // Mutual recursion through an exit.
        assert!(linearized("0.5::e(1,2).\np(X,Y) :- q(X,Y).\nq(X,Y) :- e(X,Y).\nq(X,Y) :- p(Y,X).\np(X,Y) :- p(X,Z), p(Z,Y).\nquery(p(1,Y)).\n").is_none());
        // No exit.
        assert!(linearized("p(X,Y) :- p(X,Z), p(Z,Y).\nquery(p(1,2)).\n").is_none());
    }
}
