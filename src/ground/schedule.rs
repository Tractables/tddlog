//! Body-literal scheduling: order a rule body so that every literal has the
//! variables it needs bound when it is evaluated.
//!
//! Positive atoms over predicates that are not switches bind their variables
//! and stay in source order. Where the grounder joins them, the next atom
//! is one whose variables are all bound, a lookup, else the one that shares
//! the most bound variables, the first in source order among equals. An
//! atom whose variables are all unbound after some are bound would pair
//! each of its tuples with each binding so far: in PSouffle's side-channel
//! analyses, `share(TO_E, FROM2_E)` after `xor_assign_right(TO, FROM2)`
//! waits for `equal_assign(FROM2, FROM2_E)`. One that shares fewer may
//! share only a variable that most tuples bind alike: in cplint's
//! left-corner grammar, a semi-naive round of the grounder, which starts
//! its join from the atom that reads the round's new tuples
//! ([`Order::From`]), joins `lc_call(G, A, L1, L2, Der1, Der)` to
//! `g_call(RHS2, L, L1, [..|Der0], Der1)`, which shares the rest of the
//! word and the derivation, rather than to the demand of `lc_call(G, B, L,
//! Der0)`, which shares the nonterminal `G` with every demand. A switch
//! atom (a probabilistic choice), positive or negated, comes last, in
//! source order: it binds nothing and rejects no binding, so earlier it
//! would only make the walk create its atom for bindings that a later
//! literal rejects.
//! Every other literal is placed as early as its variables allow: a negative
//! literal or comparison when all its variables are bound, `X = t` with one
//! side an unbound variable and the other bound (it binds the variable), `V =
//! f(A1,...,An)` with `V` bound (it binds the `Ai`), `V is E` when `E` is bound
//! (it binds `V`), a list predicate when the arguments it reads are bound (it
//! binds the others), a findall when the variables it shares with the clause
//! are bound, and its result where an atom written before it binds it (it
//! binds its result). Used by the grounder and by the magic-set rewrite,
//! which passes the bindings sideways in the same order but for the atoms
//! that wait ([`schedule_body`]), and in which a literal also waits for the
//! atoms written before it ([`Order::Source`]).

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

/// How [`schedule_body`] orders the positive atoms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Order {
    /// In the order of the source, as the magic rewrite passes the
    /// bindings sideways: a demand follows the program's order, which
    /// ProbLog's resolution follows (`observe(Os) :- start(S), walk(Os, 0,
    /// S)` asks `walk` with `S` bound, which its clause `walk([], _, _)`
    /// needs). Every other literal waits for the positive atoms written
    /// before it, so that the demand it makes reads them: `\+ b(P, B)`
    /// after `hop(K), P is K - 1` is demanded at the levels `hop` holds, not
    /// at every level below the one asked, without end.
    Source,
    /// As the grounder joins them: an atom whose variables are all bound
    /// first, else the one that shares the most bound variables.
    Join,
    /// The atom at this position first, then as `Join`: a semi-naive round
    /// joins the rest of the body to the tuples it adds to that atom's
    /// relation, rather than walking every binding of the atoms before it
    /// to find the few that meet a new tuple.
    From(usize),
}

/// Schedule `body` starting from `bound` variables. `switch` says of the
/// atom, positive or negated, at a position of `body` whether its predicate
/// is a switch (called bound, last); `order` how the other atoms are
/// ordered. On success returns the order, as positions in `body`, and
/// updates `bound` with everything the body binds. On failure gives the
/// position of the stuck literal.
pub(crate) fn schedule_body(
    body: &[Literal],
    bound: &mut FxHashSet<String>,
    switch: impl Fn(usize) -> bool,
    order: Order,
) -> Result<Vec<usize>, usize> {
    let binding = |i: usize| matches!(body[i], Literal::Pos(_)) && !switch(i);
    let mut pending: Vec<usize> = Vec::with_capacity(body.len());
    pending.extend((0..body.len()).filter(|&i| !binding(i) && !switch(i)));
    // A findall waits for the variables of its result that an atom written
    // before it binds: ProbLog calls it with them bound, and stops where
    // the result is then no list.
    let waits: Vec<Vec<String>> = (0..body.len())
        .map(|i| {
            let Literal::Collect(c) = &body[i] else {
                return Vec::new();
            };
            let mut vs = Vec::new();
            c.result.vars_into(&mut vs);
            vs.retain(|v| (0..i).any(|j| binding(j) && literal_vars(&body[j]).contains(v)));
            vs
        })
        .collect();
    let mut out: Vec<usize> = Vec::with_capacity(body.len());
    // The literals before position `before`, in source order, that their
    // variables allow.
    let release = |bound: &mut FxHashSet<String>,
                   pending: &mut Vec<usize>,
                   out: &mut Vec<usize>,
                   before: usize| loop {
        let mut progressed = false;
        let mut i = 0;
        while i < pending.len() && pending[i] < before {
            let l = &body[pending[i]];
            let unbound: Vec<String> = literal_vars(l)
                .into_iter()
                .filter(|v| !bound.contains(v))
                .collect();
            if waits[pending[i]].iter().all(|v| bound.contains(v)) && ready(l, &unbound, bound) {
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
    let mut atoms: Vec<usize> = (0..body.len()).filter(|&i| binding(i)).collect();
    // In the order of the source a literal waits for the atoms written
    // before it, as a call in ProbLog's resolution does: a demand it makes
    // reads them. Otherwise it goes as early as its variables allow.
    let before = |atoms: &[usize]| match order {
        Order::Source => atoms.first().copied().unwrap_or(usize::MAX),
        _ => usize::MAX,
    };
    release(bound, &mut pending, &mut out, before(&atoms));
    while !atoms.is_empty() {
        // A lookup first, then the atom that shares the most bound
        // variables, the first in source order among equals: one that
        // shares none, after some are bound, would pair each of its tuples
        // with each binding so far.
        let rank = |l: usize| {
            let vars = literal_vars(&body[l]);
            let shared = vars.iter().filter(|&v| bound.contains(v)).count();
            (shared == vars.len(), shared)
        };
        let first = match order {
            Order::From(first) => atoms.iter().position(|&l| l == first),
            _ => None,
        };
        let k = match (order, first) {
            (_, Some(k)) => k,
            (Order::Source, _) => 0,
            _ => (0..atoms.len())
                .rev()
                .max_by_key(|&k| rank(atoms[k]))
                .expect("an atom left"),
        };
        let l = atoms.remove(k);
        for v in literal_vars(&body[l]) {
            bound.insert(v);
        }
        out.push(l);
        release(bound, &mut pending, &mut out, before(&atoms));
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
    /// soon as its variables are bound, and in the order of the source
    /// after the atoms written before it.
    #[test]
    fn switches_come_last() {
        let p = parse_program("h(X,Y) :- s(X), q(X,Y), \\+ t(X), r(Y), \\+ w(X).\n").unwrap();
        let order = |order| {
            schedule_body(
                &p.clauses[0].body,
                &mut FxHashSet::default(),
                |i| matches!(i, 0 | 2),
                order,
            )
        };
        assert_eq!(order(Order::Join), Ok(vec![1, 4, 3, 0, 2]));
        assert_eq!(order(Order::Source), Ok(vec![1, 3, 4, 0, 2]));
    }

    /// An atom that shares no variable with those bound waits for one
    /// that does where the grounder joins them: `c(Y)` after `b(X, Y)`;
    /// the magic rewrite keeps the order of the source; a round whose new
    /// tuples are `b`'s joins from `b`.
    #[test]
    fn an_atom_unbound_waits_for_one_that_joins() {
        let p = parse_program("h(X) :- a(X), c(Y), b(X, Y), d(Z).\n").unwrap();
        let order = schedule_body(
            &p.clauses[0].body,
            &mut FxHashSet::default(),
            |_| false,
            Order::Join,
        );
        assert_eq!(order, Ok(vec![0, 2, 1, 3]));
        let order = schedule_body(
            &p.clauses[0].body,
            &mut FxHashSet::default(),
            |_| false,
            Order::Source,
        );
        assert_eq!(order, Ok(vec![0, 1, 2, 3]));
        let order = schedule_body(
            &p.clauses[0].body,
            &mut FxHashSet::default(),
            |_| false,
            Order::From(2),
        );
        assert_eq!(order, Ok(vec![2, 0, 1, 3]));
    }

    /// The atom that shares the most bound variables comes next: from the
    /// new tuples of `l`, `g` (sharing `L1` and `D1`) before the demand `m`
    /// and `a`, which share only `G`; then `m` (`G` and `L`), then `a`, a
    /// lookup.
    #[test]
    fn the_atom_sharing_most_comes_next() {
        let p = parse_program(
            "l(G, B, L) :- m(G, B, L, D), a(G, D), g(R, L, L1, D1), l(G, A, L1, D1).\n",
        )
        .unwrap();
        let order = schedule_body(
            &p.clauses[0].body,
            &mut FxHashSet::default(),
            |_| false,
            Order::From(3),
        );
        assert_eq!(order, Ok(vec![3, 2, 0, 1]));
    }
}
