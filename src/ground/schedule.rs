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
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};

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
///
/// Each literal's variables are read once, and binding one updates only
/// the literals that read it, so that a body of `n` literals takes about
/// `n log n` steps: a test of a wide number bit by bit has a literal for
/// each bit, and ranking every atom left after each one placed took
/// `n^2`.
pub(crate) fn schedule_body(
    body: &[Literal],
    bound: &mut FxHashSet<String>,
    switch: impl Fn(usize) -> bool,
    order: Order,
) -> Result<Vec<usize>, usize> {
    let binding = |i: usize| matches!(body[i], Literal::Pos(_)) && !switch(i);
    let atoms: Vec<usize> = (0..body.len()).filter(|&i| binding(i)).collect();
    let pending: Vec<usize> = (0..body.len())
        .filter(|&i| !binding(i) && !switch(i))
        .collect();
    let mut s = Schedule::new(body, bound, &atoms, pending);
    // In the order of the source a literal waits for the atoms written
    // before it, as a call in ProbLog's resolution does: a demand it makes
    // reads them. Otherwise it goes as early as its variables allow. With
    // `k` atoms placed, the source order places `atoms[k]` next.
    let before = |k: usize| match order {
        Order::Source => atoms.get(k).copied().unwrap_or(usize::MAX),
        _ => usize::MAX,
    };
    s.release(bound, before(0));
    for (k, &next) in atoms.iter().enumerate() {
        let l = match order {
            Order::Source => next,
            Order::From(first) if s.atom[first] && !s.placed[first] => first,
            _ => s.best(),
        };
        s.place(l, bound);
        s.release(bound, before(k + 1));
    }
    if let Some(&stuck) = s.pending.iter().find(|&&l| !s.placed[l]) {
        return Err(stuck);
    }
    let mut out = s.out;
    for i in (0..body.len()).filter(|&i| switch(i)) {
        if literal_vars(&body[i]).iter().any(|v| !bound.contains(v)) {
            return Err(i);
        }
        out.push(i);
    }
    Ok(out)
}

/// A body as [`schedule_body`] orders it: its literals' variables,
/// numbered, and what binding each variable changes.
struct Schedule<'a> {
    body: &'a [Literal],
    /// Each literal's variables ([`literal_vars`]), and their numbers.
    vars: Vec<Vec<String>>,
    numbers: Vec<Vec<u32>>,
    /// Whether each variable, by number, is bound.
    is_bound: Vec<bool>,
    /// The literals that read each variable, by number.
    readers: Vec<Vec<usize>>,
    /// Whether each literal is an atom that binds, and whether it is placed.
    atom: Vec<bool>,
    placed: Vec<bool>,
    /// Of each atom, how many of its variables are bound.
    shared: Vec<usize>,
    /// The atoms by rank: a lookup first, then the atom that shares the
    /// most bound variables, the first in source order among equals. One
    /// that shares none, after some are bound, would pair each of its
    /// tuples with each binding so far. Each change of a count pushes the
    /// new rank, and an entry that is no longer its atom's is passed over.
    ranks: BinaryHeap<(bool, usize, Reverse<usize>)>,
    /// The other literals, each placed as early as its variables allow, in
    /// source order; `waiting` holds, by place in `pending`, those whose
    /// variables were bound since they were last found not ready.
    pending: Vec<usize>,
    place_of: Vec<usize>,
    waiting: BTreeSet<usize>,
    /// Of each findall, the variables of its result that an atom written
    /// before it binds: ProbLog calls it with them bound, and stops where
    /// the result is then no list. It waits for them.
    waits: Vec<Vec<String>>,
    out: Vec<usize>,
}

impl<'a> Schedule<'a> {
    fn new(
        body: &'a [Literal],
        bound: &FxHashSet<String>,
        atoms: &[usize],
        pending: Vec<usize>,
    ) -> Self {
        let vars: Vec<Vec<String>> = body.iter().map(literal_vars).collect();
        let mut number: FxHashMap<&str, u32> = FxHashMap::default();
        let mut is_bound = Vec::new();
        let mut numbers = Vec::with_capacity(body.len());
        for vs in &vars {
            let mut row = Vec::with_capacity(vs.len());
            for v in vs {
                let n = *number.entry(v.as_str()).or_insert_with(|| {
                    is_bound.push(bound.contains(v));
                    is_bound.len() as u32 - 1
                });
                row.push(n);
            }
            numbers.push(row);
        }
        let mut readers = vec![Vec::new(); is_bound.len()];
        for (l, row) in numbers.iter().enumerate() {
            for &v in row {
                readers[v as usize].push(l);
            }
        }
        let shared = numbers
            .iter()
            .map(|row| row.iter().filter(|&&v| is_bound[v as usize]).count())
            .collect();
        let mut atom = vec![false; body.len()];
        for &l in atoms {
            atom[l] = true;
        }
        let mut place_of = vec![usize::MAX; body.len()];
        for (p, &l) in pending.iter().enumerate() {
            place_of[l] = p;
        }
        let waits = (0..body.len())
            .map(|i| {
                let Literal::Collect(c) = &body[i] else {
                    return Vec::new();
                };
                let mut vs = Vec::new();
                c.result.vars_into(&mut vs);
                vs.retain(|v| (0..i).any(|j| atom[j] && vars[j].contains(v)));
                vs
            })
            .collect();
        let mut s = Schedule {
            body,
            vars,
            numbers,
            is_bound,
            readers,
            atom,
            placed: vec![false; body.len()],
            shared,
            ranks: BinaryHeap::new(),
            waiting: (0..pending.len()).collect(),
            pending,
            place_of,
            waits,
            out: Vec::with_capacity(body.len()),
        };
        s.ranks = atoms.iter().map(|&l| s.rank(l)).collect();
        s
    }

    fn rank(&self, l: usize) -> (bool, usize, Reverse<usize>) {
        let shared = self.shared[l];
        (shared == self.numbers[l].len(), shared, Reverse(l))
    }

    /// The atom of the highest rank not placed.
    fn best(&mut self) -> usize {
        while let Some((_, shared, Reverse(l))) = self.ranks.pop() {
            if !self.placed[l] && shared == self.shared[l] {
                return l;
            }
        }
        unreachable!("an atom left")
    }

    /// Places literal `l`, whose variables are then all bound.
    fn place(&mut self, l: usize, bound: &mut FxHashSet<String>) {
        self.placed[l] = true;
        self.out.push(l);
        for k in 0..self.numbers[l].len() {
            let v = self.numbers[l][k] as usize;
            if self.is_bound[v] {
                continue;
            }
            self.is_bound[v] = true;
            bound.insert(self.vars[l][k].clone());
            for r in 0..self.readers[v].len() {
                let r = self.readers[v][r];
                if self.placed[r] {
                    continue;
                }
                if self.atom[r] {
                    self.shared[r] += 1;
                    self.ranks.push(self.rank(r));
                } else if self.place_of[r] != usize::MAX {
                    self.waiting.insert(self.place_of[r]);
                }
            }
        }
    }

    /// Places every pending literal before position `before` that is
    /// ready, in source order, until none is: those after the one placed in
    /// the same pass, those before it in the next. The others stay waiting.
    fn release(&mut self, bound: &mut FxHashSet<String>, before: usize) {
        let end = self.pending.partition_point(|&l| l < before);
        while self.waiting.range(..end).next().is_some() {
            let mut from = 0;
            while let Some(&p) = self.waiting.range(from..end).next() {
                self.waiting.remove(&p);
                from = p + 1;
                let l = self.pending[p];
                if !self.waits[l].iter().all(|v| bound.contains(v)) {
                    continue;
                }
                let unbound: Vec<String> = self.vars[l]
                    .iter()
                    .filter(|v| !bound.contains(*v))
                    .cloned()
                    .collect();
                if ready(&self.body[l], &unbound, bound) {
                    self.place(l, bound);
                }
            }
        }
    }
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
