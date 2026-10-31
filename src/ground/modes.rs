//! ProbLog's call modes ([`tests_unbound`]). ProbLog resolves a body from
//! left to right and makes each test on the terms as they stand when it
//! is reached: `X \= a` fails while `X` is unbound, `\+ p(X)` holds only
//! where no `p(_)` does, a findall collects the solutions of every value
//! of a variable still unbound. The bottom-up routes place a test where
//! its variables are bound ([`super::schedule`]), which reads it on each
//! value a literal written after it binds. The two agree where the
//! queries' calls reach every test with its variables bound; a program
//! where some call does not is resolved top-down, as ProbLog resolves it
//! ([`super::engine::topdown`]).

use rustc_hash::FxHashSet;

use super::normalize::{NormProgram, Sig};
use super::schedule::literal_vars;
use crate::syntax::{Atom, Builtin, Clause, CmpOp, Literal, Term};

/// Whether a rule that the queries, the evidence or the query rules call,
/// called as ProbLog calls it, makes a test that a variable no literal
/// before it has bound reads otherwise than the bottom-up routes: a `\=`,
/// an identity `==` or `\==`, a comparison on the standard order (`@<`,
/// `compare/3`), a negation, or a findall whose goal shares the variable
/// with the rest of the rule. A call's pattern binds the arguments that
/// the call's literals before it have bound, as the magic-set rewrite's
/// do ([`super::magic`]), here in the order of the source.
pub(crate) fn tests_unbound(norm: &NormProgram) -> bool {
    let rules_of = norm.rules_by_sig();
    let none = FxHashSet::default();
    let mut work: Vec<(Sig, Vec<bool>)> = Vec::new();
    for (a, _) in norm.queries.iter().chain(&norm.evidence) {
        work.push((a.signature(), pattern(a, &none)));
    }
    for d in &norm.drivers {
        work.push((d.atom.signature(), pattern(&d.atom, &none)));
        let bound = literal_vars(&Literal::Pos(d.atom.clone()))
            .into_iter()
            .collect();
        work.push((d.pattern.signature(), pattern(&d.pattern, &bound)));
    }
    let mut done: FxHashSet<(Sig, Vec<bool>)> = FxHashSet::default();
    while let Some((sig, called)) = work.pop() {
        let Some(rules) = rules_of.get(&sig) else {
            continue;
        };
        if !done.insert((sig, called.clone())) {
            continue;
        }
        for &ri in rules {
            if unbound_test(&norm.rules[ri], &called, &mut work) {
                return true;
            }
        }
    }
    false
}

/// Whether `rule`, its head called in the pattern `called`, makes a test
/// with a variable unbound ([`tests_unbound`]); the calls of its body, each
/// with its pattern, are pushed to `work`.
fn unbound_test<'a>(
    rule: &'a Clause,
    called: &[bool],
    work: &mut Vec<(Sig<'a>, Vec<bool>)>,
) -> bool {
    let head = &rule.heads[0].atom;
    let mut bound: FxHashSet<String> = FxHashSet::default();
    for (t, _) in head.args.iter().zip(called).filter(|(_, b)| **b) {
        bound.extend(vars(t));
    }
    // Whether a variable of the body's `i`th literal that is unbound there
    // is read elsewhere in the rule: a negation or a findall over a
    // variable of its own reads it alike on either route.
    let read_elsewhere = |i: usize, bound: &FxHashSet<String>| {
        literal_vars(&rule.body[i])
            .into_iter()
            .filter(|v| !bound.contains(v))
            .any(|v| {
                vars_of_atom(head).contains(&v)
                    || rule
                        .body
                        .iter()
                        .enumerate()
                        .any(|(j, l)| j != i && literal_vars(l).contains(&v))
            })
    };
    for (i, lit) in rule.body.iter().enumerate() {
        let unbound = |ts: &[&Term]| {
            ts.iter()
                .any(|t| vars(t).iter().any(|v| !bound.contains(v)))
        };
        let test = match lit {
            Literal::Cmp(
                CmpOp::Ne
                | CmpOp::Identical
                | CmpOp::NotIdentical
                | CmpOp::TermLt
                | CmpOp::TermLe
                | CmpOp::TermGt
                | CmpOp::TermGe,
                a,
                b,
            ) => unbound(&[a, b]),
            Literal::Builtin(Builtin::Compare, args, _) => unbound(&[&args[1], &args[2]]),
            Literal::Neg(_) | Literal::Builtin(_, _, true) => read_elsewhere(i, &bound),
            Literal::Collect(c) => unbound(&c.shared.iter().collect::<Vec<_>>()),
            _ => false,
        };
        if test {
            return true;
        }
        match lit {
            Literal::Pos(a) | Literal::Neg(a) => work.push((a.signature(), pattern(a, &bound))),
            Literal::Collect(c) => {
                let a = c.pattern();
                work.push(((&*c.aux, c.arity), pattern(&a, &bound)));
            }
            _ => {}
        }
        bind(lit, &mut bound);
    }
    false
}

/// Adds to `bound` the variables the literal `lit` binds where those in
/// `bound` are: every variable of an atom, whose answers are ground on the
/// routes this decides between; a side of `=` where the other is bound;
/// the value of an `is` whose expression is; the arguments of a built-in
/// that runs with those bound ([`Builtin::runs`]); a findall's result.
fn bind(lit: &Literal, bound: &mut FxHashSet<String>) {
    let all = |t: &Term, bound: &FxHashSet<String>| vars(t).iter().all(|v| bound.contains(v));
    let add: Vec<String> = match lit {
        Literal::Pos(a) => vars_of_atom(a),
        Literal::Cmp(CmpOp::Eq, a, b) if all(a, bound) => vars(b),
        Literal::Cmp(CmpOp::Eq, a, b) if all(b, bound) => vars(a),
        Literal::Is(v, e) if all(e, bound) => vars(v),
        Literal::Builtin(b, args, false) => {
            let given: Vec<bool> = args.iter().map(|t| all(t, bound)).collect();
            if b.runs(&given) {
                args.iter().flat_map(vars).collect()
            } else {
                Vec::new()
            }
        }
        Literal::Collect(c) => vars(&c.result),
        Literal::Findall(f) => vars(&f.result),
        _ => Vec::new(),
    };
    bound.extend(add);
}

/// The pattern of a call of `atom`: bound where an argument's variables
/// all are.
fn pattern(atom: &Atom, bound: &FxHashSet<String>) -> Vec<bool> {
    atom.args
        .iter()
        .map(|t| vars(t).iter().all(|v| bound.contains(v)))
        .collect()
}

fn vars(t: &Term) -> Vec<String> {
    let mut out = Vec::new();
    t.vars_into(&mut out);
    out
}

fn vars_of_atom(a: &Atom) -> Vec<String> {
    let mut out = Vec::new();
    a.vars_into(&mut out);
    out
}
