//! ProbLog's call modes ([`tests_unbound`]). ProbLog resolves a body from
//! left to right and makes each test on the terms as they stand when it
//! is reached: `X \= a` fails while `X` is unbound, `\+ p(X)` holds only
//! where no `p(_)` does, a findall collects the solutions of every value
//! of a variable still unbound, `var(X)` holds while `X` is unbound. The bottom-up routes place a test where
//! its variables are bound ([`super::schedule`]), which reads it on each
//! value a literal written after it binds. The two agree where the
//! queries' calls reach every test with its variables bound; a program
//! where some call does not is resolved top-down, as ProbLog resolves it
//! ([`super::engine::topdown`]). An arithmetic literal or a call of a
//! goal that ProbLog reaches with a variable free is an error there, and
//! is refused.

use rustc_hash::FxHashSet;

use super::engine::GroundError;
use super::normalize::{NormProgram, Sig};
use super::schedule::literal_vars;
use crate::syntax::{Atom, Builtin, Clause, CmpOp, Literal, Term};

/// What a call's argument holds when the call is made.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Arg {
    /// A free variable.
    Free,
    /// A term that may hold free variables, or a variable that may be
    /// bound to one.
    Partial,
    /// A ground term.
    Bound,
}

/// Whether a rule that the queries, the evidence or the query rules call,
/// called as ProbLog calls it, makes a test that a variable no literal
/// before it has bound reads otherwise than the bottom-up routes: a `\=`,
/// an identity `==` or `\==`, a comparison on the standard order (`@<`,
/// `compare/3`), a type test (`var/1`, `nonvar/1`, `is_list/1`), a
/// negation, or a findall whose goal shares the variable with the rest of
/// the rule. A call's pattern binds the arguments that
/// the call's literals before it have bound, as the magic-set rewrite's
/// do ([`super::magic`]), here in the order of the source.
///
/// An error where such a rule evaluates an arithmetic comparison or an
/// `is` over a variable that is free there, or calls a goal that is
/// ([`Builtin::Callable`]), which ProbLog stops on: a variable is free
/// where neither the call nor a literal before it can have bound it.
pub(crate) fn tests_unbound(norm: &NormProgram) -> Result<bool, GroundError> {
    let rules_of = norm.rules_by_sig();
    let none = Vars::default();
    let mut work: Vec<(Sig, Vec<Arg>)> = Vec::new();
    for (a, _) in norm.queries.iter().chain(&norm.evidence) {
        work.push((a.signature(), pattern(a, &none)));
    }
    for d in &norm.drivers {
        work.push((d.atom.signature(), pattern(&d.atom, &none)));
        let mut vars = Vars::default();
        vars.bound
            .extend(literal_vars(&Literal::Pos(d.atom.clone())));
        work.push((d.pattern.signature(), pattern(&d.pattern, &vars)));
    }
    let mut done: FxHashSet<(Sig, Vec<Arg>)> = FxHashSet::default();
    let mut fires = false;
    while let Some((sig, called)) = work.pop() {
        let Some(rules) = rules_of.get(&sig) else {
            continue;
        };
        if !done.insert((sig, called.clone())) {
            continue;
        }
        for &ri in rules {
            fires |= unbound_test(&norm.rules[ri], &called, &mut work)?;
        }
    }
    Ok(fires)
}

/// The variables of a rule that a literal of its body may find bound:
/// those it finds ground, and those that may hold a term.
#[derive(Default)]
struct Vars {
    bound: FxHashSet<String>,
    touched: FxHashSet<String>,
    /// The unifications that bound neither side, each tried again as
    /// literals after it bind.
    pending: Vec<(Term, Term)>,
}

impl Vars {
    fn ground(&self, t: &Term) -> bool {
        vars(t).iter().all(|v| self.bound.contains(v))
    }

    /// Whether the variable `v` is free: no literal before, nor the call,
    /// can have bound it.
    fn free(&self, v: &str) -> bool {
        !self.bound.contains(v) && !self.touched.contains(v)
    }
}

/// Whether `rule`, its head called in the pattern `called`, makes a test
/// with a variable unbound ([`tests_unbound`]); the calls of its body, each
/// with its pattern, are pushed to `work`.
fn unbound_test<'a>(
    rule: &'a Clause,
    called: &[Arg],
    work: &mut Vec<(Sig<'a>, Vec<Arg>)>,
) -> Result<bool, GroundError> {
    let head = &rule.heads[0].atom;
    let mut v = Vars::default();
    for (t, arg) in head.args.iter().zip(called) {
        match arg {
            Arg::Bound => v.bound.extend(vars(t)),
            Arg::Partial => v.touched.extend(vars(t)),
            Arg::Free => {}
        }
    }
    // Whether a variable of the body's `i`th literal that is unbound there
    // is read elsewhere in the rule: a negation or a findall over a
    // variable of its own reads it alike on either route.
    let read_elsewhere = |i: usize, v: &Vars| {
        literal_vars(&rule.body[i])
            .into_iter()
            .filter(|x| !v.bound.contains(x))
            .any(|x| {
                vars_of_atom(head).contains(&x)
                    || rule
                        .body
                        .iter()
                        .enumerate()
                        .any(|(j, l)| j != i && literal_vars(l).contains(&x))
            })
    };
    let mut fires = false;
    for (i, lit) in rule.body.iter().enumerate() {
        let unbound = |ts: &[&Term]| ts.iter().any(|t| !v.ground(t));
        let free = |ts: &[&Term]| ts.iter().flat_map(|t| vars(t)).find(|x| v.free(x));
        let stops = match lit {
            Literal::Cmp(op, a, b) if op.arithmetic() => free(&[a, b]),
            Literal::Is(_, e) => free(&[e]),
            Literal::Builtin(Builtin::Callable, args, _) => free(&[&args[0]]),
            _ => None,
        };
        if let Some(x) = stops {
            return Err(GroundError::Unsafe(format!(
                "{rule}: {x} is free where ProbLog reaches {lit}, which it stops on"
            )));
        }
        fires |= match lit {
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
            Literal::Builtin(Builtin::Var | Builtin::Nonvar | Builtin::IsList, args, _) => {
                unbound(&[&args[0]])
            }
            Literal::Neg(_) | Literal::Builtin(_, _, true) => read_elsewhere(i, &v),
            Literal::Collect(c) => unbound(&c.shared.iter().collect::<Vec<_>>()),
            _ => false,
        };
        match lit {
            Literal::Pos(a) | Literal::Neg(a) => work.push((a.signature(), pattern(a, &v))),
            Literal::Collect(c) => {
                let a = c.pattern();
                work.push(((&*c.aux, c.arity), pattern(&a, &v)));
            }
            _ => {}
        }
        bind(lit, &mut v);
    }
    Ok(fires)
}

/// Adds to `v` the variables the literal `lit` binds: every variable of an
/// atom, whose answers are ground on the routes this decides between; a
/// side of `=` where the other is bound, at once or once a later literal
/// binds it; the value of an `is` whose expression is bound; the arguments
/// of a built-in that runs with those bound ([`Builtin::runs`]); a
/// findall's result. A variable `=` makes a term, or that a built-in reads
/// but cannot bind, may hold a term.
fn bind(lit: &Literal, v: &mut Vars) {
    match lit {
        Literal::Pos(a) => v.bound.extend(vars_of_atom(a)),
        Literal::Cmp(CmpOp::Eq, a, b) => {
            for (x, t) in [(a, b), (b, a)] {
                if let Term::Var(x) = x
                    && (!matches!(t, Term::Var(_)) || !v.free(&vars(t)[0]))
                {
                    v.touched.insert(x.clone());
                }
            }
            v.pending.push((a.clone(), b.clone()));
        }
        Literal::Is(x, e) if v.ground(e) => v.bound.extend(vars(x)),
        Literal::Is(x, _) => v.touched.extend(vars(x)),
        Literal::Builtin(b, args, false) => {
            let given: Vec<bool> = args.iter().map(|t| v.ground(t)).collect();
            let all = args.iter().flat_map(vars);
            if b.runs(&given) {
                v.bound.extend(all);
            } else {
                v.touched.extend(all);
            }
        }
        Literal::Collect(c) => v.bound.extend(vars(&c.result)),
        Literal::Findall(f) => v.bound.extend(vars(&f.result)),
        _ => {}
    }
    // The unifications whose one side is now bound bind the other.
    loop {
        let before = v.pending.len();
        let pending = std::mem::take(&mut v.pending);
        for (a, b) in pending {
            if v.ground(&a) {
                v.bound.extend(vars(&b));
            } else if v.ground(&b) {
                v.bound.extend(vars(&a));
            } else {
                v.pending.push((a, b));
            }
        }
        if v.pending.len() == before {
            break;
        }
    }
}

/// The pattern of a call of `atom`: bound where an argument's variables
/// all are, free where it is a free variable.
fn pattern(atom: &Atom, v: &Vars) -> Vec<Arg> {
    atom.args
        .iter()
        .map(|t| match t {
            _ if v.ground(t) => Arg::Bound,
            Term::Var(x) if v.free(x) => Arg::Free,
            _ => Arg::Partial,
        })
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
