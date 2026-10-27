//! Body-literal scheduling: order a rule body so that every literal has the
//! variables it needs bound when it is evaluated.
//!
//! Positive atoms over predicates that are not switches bind their variables
//! and stay in source order. Where the grounder joins them, an atom whose
//! variables are all unbound after some are bound would pair each of its
//! tuples with each binding so far, and waits for the first atom that
//! shares a bound variable: in PSouffle's side-channel analyses,
//! `share(TO_E, FROM2_E)` after `xor_assign_right(TO, FROM2)` waits for
//! `equal_assign(FROM2, FROM2_E)`. A switch atom (a probabilistic choice),
//! positive or negated, comes last, in source order: it binds nothing and rejects no
//! binding, so earlier it would only make the walk create its atom for
//! bindings that a later literal rejects, and keep a semi-naive round from
//! passing over the literals before it (`narrow` in [`super::engine`]).
//! Every other literal is placed as early as its variables allow: a negative
//! literal or comparison when all its variables are bound, `X = t` with one
//! side an unbound variable and the other bound (it binds the variable), `V =
//! f(A1,...,An)` with `V` bound (it binds the `Ai`), `V is E` when `E` is bound
//! (it binds `V`), a list predicate when the arguments it reads are bound (it
//! binds the others), a findall when the variables it shares with the clause
//! are bound (it binds its result). Used by the grounder and by the magic-set
//! rewrite, which passes the bindings sideways in the same order but for the
//! atoms that wait ([`schedule_body`]).

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

/// Schedule `body` starting from `bound` variables. `switch` says of the
/// atom, positive or negated, at a position of `body` whether its predicate
/// is a switch (called bound, last). On success returns the order, as
/// positions in `body`, and updates `bound` with everything the body binds.
/// On failure gives the position of the stuck literal. With `join`, an atom
/// that shares no variable with those bound waits for one that does, as
/// the grounder joins; without, the atoms keep the order of the source, as
/// the magic rewrite passes the bindings sideways (a demand follows the
/// program's order, which ProbLog's resolution follows: `observe(Os) :-
/// start(S), walk(Os, 0, S)` asks `walk` with `S` bound, which its clause
/// `walk([], _, _)` needs).
pub(crate) fn schedule_body(
    body: &[Literal],
    bound: &mut FxHashSet<String>,
    switch: impl Fn(usize) -> bool,
    join: bool,
) -> Result<Vec<usize>, usize> {
    let binding = |i: usize| matches!(body[i], Literal::Pos(_)) && !switch(i);
    let mut pending: Vec<usize> = Vec::with_capacity(body.len());
    pending.extend((0..body.len()).filter(|&i| !binding(i) && !switch(i)));
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
    let mut atoms: Vec<usize> = (0..body.len()).filter(|&i| binding(i)).collect();
    while !atoms.is_empty() {
        // An atom whose variables are all unbound, after some are bound,
        // would pair each of its tuples with each binding so far: it waits
        // for the first atom that shares a bound variable, unless none is
        // left.
        let joins = |l: usize| {
            let vars = literal_vars(&body[l]);
            vars.is_empty() || vars.iter().any(|v| bound.contains(v))
        };
        let k = if !join || bound.is_empty() {
            0
        } else {
            atoms.iter().position(|&l| joins(l)).unwrap_or(0)
        };
        let l = atoms.remove(k);
        for v in literal_vars(&body[l]) {
            bound.insert(v);
        }
        out.push(l);
        release(bound, &mut pending, &mut out);
    }
    if let Some(&stuck) = pending.first() {
        return Err(stuck);
    }
    for i in (0..body.len()).filter(|&i| switch(i)) {
        if literal_vars(&body[i]).iter().any(|v| !bound.contains(v)) {
            return Err(i);
        }
        out.push(i);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::parse_program;

    /// The switches, positive or negated, come after every other literal,
    /// in source order; a negative literal that is no switch is placed as
    /// soon as its variables are bound.
    #[test]
    fn switches_come_last() {
        let p = parse_program("h(X,Y) :- s(X), q(X,Y), \\+ t(X), r(Y), \\+ w(X).\n").unwrap();
        let order = schedule_body(
            &p.clauses[0].body,
            &mut FxHashSet::default(),
            |i| matches!(i, 0 | 2),
            false,
        );
        assert_eq!(order, Ok(vec![1, 4, 3, 0, 2]));
    }

    /// An atom that shares no variable with those bound waits for one
    /// that does where the grounder joins them: `c(Y)` after `b(X, Y)`;
    /// the magic rewrite keeps the order of the source.
    #[test]
    fn an_atom_unbound_waits_for_one_that_joins() {
        let p = parse_program("h(X) :- a(X), c(Y), b(X, Y), d(Z).\n").unwrap();
        let order = schedule_body(
            &p.clauses[0].body,
            &mut FxHashSet::default(),
            |_| false,
            true,
        );
        assert_eq!(order, Ok(vec![0, 2, 1, 3]));
        let order = schedule_body(
            &p.clauses[0].body,
            &mut FxHashSet::default(),
            |_| false,
            false,
        );
        assert_eq!(order, Ok(vec![0, 1, 2, 3]));
    }
}
