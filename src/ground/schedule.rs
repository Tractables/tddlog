//! Body-literal scheduling: order a rule body so that every literal has the
//! variables it needs bound when it is evaluated.
//!
//! Positive atoms over predicates with rules bind their variables and stay in
//! source order. Every other literal is placed as early as its variables allow:
//! a switch atom (probabilistic fact) when its arguments are bound, a negative
//! literal or comparison when all its variables are bound, `X = t` with one
//! side an unbound variable and the other bound (it binds the variable), `V =
//! f(A1,...,An)` with `V` bound (it binds the `Ai`), `V is E` when `E` is bound
//! (it binds `V`), a list predicate when the arguments it reads are bound (it
//! binds the others), a findall when the variables it shares with the clause
//! are bound (it binds its result). Used by the grounder and by the magic-set
//! rewrite, so both see the same sideways information passing.

use crate::syntax::{CmpOp, Literal, Term};
use rustc_hash::FxHashSet;

/// Variables of a literal, each once, in order of first occurrence.
pub(crate) fn literal_vars(l: &Literal) -> Vec<String> {
    let mut out = Vec::new();
    match l {
        // An atom's arguments, the common literal, are flat but in a
        // pattern: read without the walk of a term.
        Literal::Pos(a) | Literal::Neg(a) => {
            for t in &a.args {
                match t {
                    Term::Var(v) if !out.contains(v) => out.push(v.clone()),
                    Term::Compound(_) => t.vars_into(&mut out),
                    _ => {}
                }
            }
        }
        other => other.vars_into(&mut out),
    }
    out
}

/// Whether every variable of `t` is in `bound`.
fn bound_term(t: &Term, bound: &FxHashSet<String>) -> bool {
    match t {
        Term::Var(v) => bound.contains(v),
        Term::Compound(_) => {
            let mut vs = Vec::new();
            t.vars_into(&mut vs);
            vs.iter().all(|v| bound.contains(v))
        }
        _ => true,
    }
}

/// Whether `l`, pending, can be evaluated with `bound` bound; if so, the
/// variables it binds are added to `bound`. `unbound` holds the variables of
/// `l` not in `bound`.
fn ready(l: &Literal, unbound: &[String], bound: &mut FxHashSet<String>) -> bool {
    let bind_all = |bound: &mut FxHashSet<String>| {
        bound.extend(unbound.iter().cloned());
        true
    };
    match l {
        _ if unbound.is_empty() => true,
        Literal::Cmp(CmpOp::Eq, a, b) => {
            let free = |t: &Term| matches!(t, Term::Var(v) if !bound.contains(v));
            if (free(a) && bound_term(b, bound)) || (free(b) && bound_term(a, bound)) {
                // `X = t` with `t` bound binds `X`.
                bind_all(bound)
            } else if (bound_term(a, bound) && matches!(b, Term::Compound(_)))
                || (bound_term(b, bound) && matches!(a, Term::Compound(_)))
            {
                // `V = f(A..)` with `V` bound takes `V` apart.
                bind_all(bound)
            } else {
                false
            }
        }
        Literal::Is(v, e) => bound_term(e, bound) && v.is_var() && bind_all(bound),
        Literal::Builtin(b, args, false) => {
            let flags: Vec<bool> = args.iter().map(|t| bound_term(t, bound)).collect();
            b.runs(&flags) && bind_all(bound)
        }
        Literal::Collect(c) => c.shared.iter().all(|t| bound_term(t, bound)) && bind_all(bound),
        _ => false,
    }
}

/// Schedule `body` starting from `bound` variables. `binds` says of the
/// positive atom at a position of `body` whether its predicate has rules
/// (binds its variables) or is a switch (must be called bound). On success
/// returns the order, as positions in `body`, and updates `bound` with
/// everything the body binds. On failure gives the position of the stuck
/// literal.
pub(crate) fn schedule_body(
    body: &[Literal],
    bound: &mut FxHashSet<String>,
    binds: impl Fn(usize) -> bool,
) -> Result<Vec<usize>, usize> {
    let binding = |i: usize| matches!(body[i], Literal::Pos(_)) && binds(i);
    let mut pending: Vec<usize> = Vec::with_capacity(body.len());
    pending.extend((0..body.len()).filter(|&i| !binding(i)));
    let mut out: Vec<usize> = Vec::with_capacity(body.len());
    let release =
        |bound: &mut FxHashSet<String>, pending: &mut Vec<usize>, out: &mut Vec<usize>| loop {
            let mut progressed = false;
            let mut i = 0;
            while i < pending.len() {
                let l = &body[pending[i]];
                let unbound: Vec<String> = literal_vars(l)
                    .into_iter()
                    .filter(|v| !bound.contains(v))
                    .collect();
                if ready(l, &unbound, bound) {
                    out.push(pending.remove(i));
                    progressed = true;
                } else {
                    i += 1;
                }
            }
            if !progressed {
                break;
            }
        };
    release(bound, &mut pending, &mut out);
    for l in (0..body.len()).filter(|&i| binding(i)) {
        for v in literal_vars(&body[l]) {
            bound.insert(v);
        }
        out.push(l);
        release(bound, &mut pending, &mut out);
    }
    match pending.into_iter().next() {
        Some(stuck) => Err(stuck),
        None => Ok(out),
    }
}
