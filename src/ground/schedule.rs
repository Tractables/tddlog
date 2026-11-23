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
//! Der0)`, which shares the nonterminal `G` with every demand. An atom
//! with a variable bound at a key of its relation (a position at which no
//! two of its tuples agree) yields at most one tuple, as a lookup does, and
//! ranks with the lookups. From a new tuple of `f(P, K)` in `c(J, K1) :-
//! c(J, K), next(P, J), f(P, K), ...`, the successor fact `next(P, J)` binds
//! `J` as `P is J - 1` would, and `c(J, K)` is then a lookup; joined first,
//! the one written first of two atoms that share one bound variable each,
//! `c(J, K)` walked every tuple of `c` with that `K`. A switch
//! atom (a probabilistic choice), positive or negated, comes last, in
//! source order: it binds nothing and rejects no binding, so earlier it
//! would only make the walk create its atom for bindings that a later
//! literal rejects.
//! Every other literal is placed as early as its variables allow: a negative
//! literal or comparison when all its variables are bound, `X = t` with one
//! side an unbound variable and the other bound (it binds the variable), `V =
//! f(A1,...,An)` with `V` bound (it binds the `Ai`), `V is E` when `E` is bound
//! (it binds `V`), a list predicate when the arguments it reads are bound (it
//! binds the others; `memberchk/2` also its element where a literal written
//! before it binds it), a findall when the variables it shares with the clause
//! are bound, and its result where a literal written before it binds it (it
//! binds its result). Used by the grounder and by the magic-set rewrite,
//! which passes the bindings sideways in the same order but for the atoms
//! that wait ([`schedule_body`]), and in which a literal also waits for the
//! atoms written before it ([`Order::Source`]).

use crate::syntax::{Builtin, CmpOp, Literal, Term};
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
        Term::Compound(_) => t.vars().iter().all(|v| bound.contains(v)),
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
    schedule_keyed(body, bound, switch, |_| &[], order)
}

/// [`schedule_body`], where `keys` gives of the positive atom at a position
/// the argument positions that are keys of its relation: the grounder reads
/// them from a predicate's facts.
///
/// Each literal's variables are read once, and binding one updates only
/// the literals that read it, so that a body of `n` literals takes about
/// `n log n` steps: a test of a wide number bit by bit has a literal for
/// each bit, and ranking every atom left after each one placed took
/// `n^2`.
pub(crate) fn schedule_keyed<'k>(
    body: &[Literal],
    bound: &mut FxHashSet<String>,
    switch: impl Fn(usize) -> bool,
    keys: impl Fn(usize) -> &'k [usize],
    order: Order,
) -> Result<Vec<usize>, usize> {
    let binding = |i: usize| matches!(body[i], Literal::Pos(_)) && !switch(i);
    let atoms: Vec<usize> = (0..body.len()).filter(|&i| binding(i)).collect();
    let pending: Vec<usize> = (0..body.len())
        .filter(|&i| !binding(i) && !switch(i))
        .collect();
    // A body without variables, every body of a ground program: each
    // literal is ready as it stands and every atom is a lookup, so the
    // order below is the one the ranks and releases give, read off. In
    // the source order the literals stay as written; otherwise the other
    // literals come first, then the atoms in source order (`From`'s
    // first), then the switches. The schedule's tables for each of a
    // million ground rules cost more than grounding them.
    if body.iter().all(|l| literal_vars(l).is_empty()) {
        let mut out: Vec<usize> = match order {
            Order::Source => (0..body.len()).filter(|&i| !switch(i)).collect(),
            Order::From(first) if atoms.contains(&first) => pending
                .iter()
                .copied()
                .chain(std::iter::once(first))
                .chain(atoms.iter().copied().filter(|&l| l != first))
                .collect(),
            _ => pending.iter().chain(&atoms).copied().collect(),
        };
        out.extend((0..body.len()).filter(|&i| switch(i)));
        return Ok(out);
    }
    let mut s = Schedule::new(body, bound, &atoms, pending, keys);
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
    /// Of each atom, the variables at the keys of its relation, and
    /// whether a key holds a constant: with one of those bound, or with
    /// the constant, it yields at most one tuple.
    keyed: Vec<Vec<u32>>,
    fixed: Vec<bool>,
    /// The atoms by rank: a lookup, or an atom that yields at most one
    /// tuple, first, then the atom that shares the most bound variables,
    /// the first in source order among equals. One that shares none, after
    /// some are bound, would pair each of its tuples with each binding so
    /// far. Each change of a count pushes the new rank, and an entry that
    /// is no longer its atom's is passed over.
    ranks: BinaryHeap<(bool, usize, Reverse<usize>)>,
    /// The other literals, each placed as early as its variables allow, in
    /// source order; `waiting` holds, by place in `pending`, those whose
    /// variables were bound since they were last found not ready.
    pending: Vec<usize>,
    place_of: Vec<usize>,
    waiting: BTreeSet<usize>,
    /// Of each findall, the variables of its result that a literal written
    /// before it binds: ProbLog calls it with them bound, and stops where
    /// the result is then no list. It waits for them. So does `memberchk/2`
    /// for those of its element: called with a free element it binds the
    /// list's first item, and a join that reaches it before the literals
    /// that bind the element would keep only the bindings that agree with
    /// that item.
    waits: Vec<Vec<String>>,
    out: Vec<usize>,
}

impl<'a> Schedule<'a> {
    fn new<'k>(
        body: &'a [Literal],
        bound: &FxHashSet<String>,
        atoms: &[usize],
        pending: Vec<usize>,
        keys: impl Fn(usize) -> &'k [usize],
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
        let (mut keyed, mut fixed) = (vec![Vec::new(); body.len()], vec![false; body.len()]);
        for &l in atoms {
            atom[l] = true;
            let Literal::Pos(a) = &body[l] else {
                unreachable!("an atom that binds is positive")
            };
            for &k in keys(l) {
                match &a.args[k] {
                    Term::Var(v) => keyed[l].push(number[v.as_str()]),
                    Term::Compound(_) => {}
                    _ => fixed[l] = true,
                }
            }
        }
        let mut place_of = vec![usize::MAX; body.len()];
        for (p, &l) in pending.iter().enumerate() {
            place_of[l] = p;
        }
        let waits = (0..body.len())
            .map(|i| {
                let mut vs = Vec::new();
                match &body[i] {
                    Literal::Collect(c) => c.result.vars_into(&mut vs),
                    Literal::Builtin(Builtin::Memberchk, args, false) => args[0].vars_into(&mut vs),
                    _ => return vs,
                }
                vs.retain(|v| vars[..i].iter().any(|w| w.contains(v)));
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
            keyed,
            fixed,
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
        let one = shared == self.numbers[l].len()
            || self.fixed[l]
            || self.keyed[l].iter().any(|&v| self.is_bound[v as usize]);
        (one, shared, Reverse(l))
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

    /// A body without variables orders as one with them does where every
    /// literal is ready: the others first, the atoms in source order (from
    /// the one a round adds to), the switches last; the source order keeps
    /// the literals as written.
    #[test]
    fn a_ground_body_orders_as_its_literals_are_ready() {
        let p = parse_program("h :- s(a), q(a,b), \\+ t(a), r(b), \\+ w(a), u(c).\n").unwrap();
        let order = |order| {
            schedule_body(
                &p.clauses[0].body,
                &mut FxHashSet::default(),
                |i| matches!(i, 0 | 2),
                order,
            )
        };
        assert_eq!(order(Order::Join), Ok(vec![4, 1, 3, 5, 0, 2]));
        assert_eq!(order(Order::Source), Ok(vec![1, 3, 4, 5, 0, 2]));
        assert_eq!(order(Order::From(3)), Ok(vec![4, 3, 1, 5, 0, 2]));
        assert_eq!(order(Order::From(4)), Ok(vec![4, 1, 3, 5, 0, 2]));
    }

    /// An atom that shares no variable with those bound waits for one
    /// that does where the grounder joins them: `c(Y)` after `b(X, Y)`;
    /// the magic rewrite keeps the order of the source; a round whose new
    /// tuples are `b`'s joins from `b`.
    #[test]
    fn an_atom_unbound_waits_for_one_that_joins() {
        let p = parse_program("h(X) :- a(X), c(Y), b(X, Y), d(Z).\n").unwrap();
        let order = |o| schedule_body(&p.clauses[0].body, &mut FxHashSet::default(), |_| false, o);
        assert_eq!(order(Order::Join), Ok(vec![0, 2, 1, 3]));
        assert_eq!(order(Order::Source), Ok(vec![0, 1, 2, 3]));
        assert_eq!(order(Order::From(2)), Ok(vec![2, 0, 1, 3]));
    }

    /// `memberchk/2` waits for the element an atom written before it
    /// binds, though the list is bound first: a join from `r` would
    /// otherwise bind `X` to the list's first item.
    #[test]
    fn memberchk_waits_for_its_element() {
        let p = parse_program("h(X, L) :- m(X), memberchk(X, L), r(L).\n").unwrap();
        let mut body = p.clauses[0].body.clone();
        let Literal::Pos(a) = &body[1] else {
            unreachable!("an atom")
        };
        body[1] = Literal::Builtin(Builtin::Memberchk, a.args.to_vec(), false);
        let order = schedule_body(&body, &mut FxHashSet::default(), |_| false, Order::From(2));
        assert_eq!(order, Ok(vec![2, 0, 1]));
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

    /// An atom with a variable bound at a key of its relation ranks with
    /// the lookups: from the new tuples of `f(P, K)`, the successor fact
    /// `next(P, J)`, each of whose arguments is a key, comes before `c(J,
    /// K)`, which is written first and shares as many bound variables, and
    /// makes it a lookup.
    #[test]
    fn an_atom_bound_at_a_key_ranks_with_the_lookups() {
        let p = parse_program("c(J, K1) :- c(J, K), next(P, J), f(P, K), s(K, K1).\n").unwrap();
        let order = |next: &'static [usize]| {
            schedule_keyed(
                &p.clauses[0].body,
                &mut FxHashSet::default(),
                |_| false,
                |i| if i == 1 { next } else { &[] },
                Order::From(2),
            )
        };
        assert_eq!(order(&[]), Ok(vec![2, 0, 1, 3]));
        assert_eq!(order(&[0, 1]), Ok(vec![2, 1, 0, 3]));
    }
}
