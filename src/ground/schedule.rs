//! Body-literal scheduling: order a rule body so that every literal has the
//! variables it needs bound when it is evaluated.
//!
//! Positive atoms over predicates with rules bind their variables and stay in
//! source order. Every other literal is placed as early as its variables allow:
//! a switch atom (probabilistic fact) when its arguments are bound, a negative
//! literal or comparison when all its variables are bound, `X = t` with exactly
//! one unbound plain-variable side (it binds that side), `V is E` when `E` is
//! bound (it binds `V`). Used by the grounder and by the magic-set rewrite, so
//! both see the same sideways information passing.

use crate::syntax::{CmpOp, Expr, Literal, Term};
use rustc_hash::FxHashSet;

fn expr_vars(e: &Expr, out: &mut Vec<String>) {
    let mut vs = Vec::new();
    e.vars(&mut vs);
    for v in vs {
        if let Term::Var(name) = v {
            out.push(name);
        }
    }
}

/// Variables of a literal.
pub(crate) fn literal_vars(l: &Literal) -> Vec<String> {
    let mut out = Vec::new();
    match l {
        Literal::Pos(a) | Literal::Neg(a) => {
            for t in &a.args {
                if let Term::Var(v) = t {
                    out.push(v.clone());
                }
            }
        }
        Literal::Cmp(_, a, b) => {
            expr_vars(a, &mut out);
            expr_vars(b, &mut out);
        }
        Literal::Is(v, e) => {
            if let Term::Var(name) = v {
                out.push(name.clone());
            }
            expr_vars(e, &mut out);
        }
    }
    out
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
                let unbound: Vec<String> = {
                    let mut u: Vec<String> = literal_vars(l)
                        .into_iter()
                        .filter(|v| !bound.contains(v))
                        .collect();
                    u.dedup();
                    u
                };
                let ready = match l {
                    Literal::Cmp(CmpOp::Eq, a, b) => {
                        let plain_unbound =
                            |e: &Expr| matches!(e, Expr::Term(Term::Var(v)) if !bound.contains(v));
                        if unbound.is_empty() {
                            true
                        } else if unbound.len() == 1 && (plain_unbound(a) || plain_unbound(b)) {
                            bound.insert(unbound[0].clone());
                            true
                        } else {
                            false
                        }
                    }
                    Literal::Is(v, e) => {
                        let mut ev = Vec::new();
                        expr_vars(e, &mut ev);
                        if ev.iter().all(|x| bound.contains(x)) {
                            if let Term::Var(name) = v {
                                bound.insert(name.clone());
                            }
                            true
                        } else {
                            false
                        }
                    }
                    _ => unbound.is_empty(),
                };
                if ready {
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
