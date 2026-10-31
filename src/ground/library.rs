//! The predicates of ProbLog's libraries that its libraries write as
//! clauses, rather than built in ([`crate::syntax::Builtin`]), and
//! `call/N`, which they call.
//!
//! A program that calls a predicate of [`LIBRARY`] and does not define it
//! gets the library's clauses for it, and for those they call in turn:
//! `library(lists)`'s random selections and `library(apply)`'s maplist/2
//! to /5, foldl/4 to /6, include/3, exclude/3 and partition/4. The helpers
//! are named apart from the program's predicates.
//! A program that loads `library(aggregate)` gets those of [`AGGREGATE`]
//! too, the functions of its aggregates.
//!
//! `call(G, A1, ..., An)` calls `G` with the arguments added. Where `G` is
//! written as an atom or a compound, it is that predicate's call. Where it
//! is a variable, it is a call of `'$call<n>'(G, A1, ..., An)`, which has a
//! clause `'$call<n>'(p(B1, ..., Bk), A1, ..., An) :- p(B1, ..., Bk, A1,
//! ..., An)` (its first argument `p` where `k = 0`) for each predicate
//! `p/(k + n)` that the program defines and names in an argument, the
//! predicates a goal it builds can name. A goal bound to anything else has
//! no clause, and the call fails. Each such clause is read in every mode
//! of the calls of `call/N`: one whose body is safe only for the goal's own
//! callers (a fact with a variable that only a caller binds) makes the
//! grounding refuse the program, as an unsafe rule.

use std::cell::RefCell;
use std::sync::OnceLock;

use rustc_hash::{FxHashMap, FxHashSet};

use super::terms::{calls, calls_mut};
use crate::syntax::{Atom, Clause, Compound, Literal, Name, Program, Term};

/// The library's clauses. A random selection is made as ProbLog makes it,
/// of the same random variables: each item in turn is chosen with its
/// weight over those of the items left, by a probabilistic fact of the
/// selection's identifier, that weight, the weights left, the item and the
/// items after it.
const LIBRARY: &str = r"
select_uniform(ID, Values, Value, Rest) :-
    length(Values, Len), Len > 0, Weight is 1/Len,
    '$lists_make_list'(Len, Weight, Weights),
    select_weighted(ID, Weights, Values, Value, Rest).
select_weighted(ID, Weights, Values, Value, Rest) :-
    sum_list(Weights, Total), Total > 0,
    '$lists_sw'(ID, Total, Weights, Values, Value, Rest).
select_weighted(ID, WeightsValues, Value, Rest) :-
    '$lists_unzip'(WeightsValues, Weights, Values),
    select_weighted(ID, Weights, Values, Value, Rest).
P::'$lists_sw_p'(ID, P, _, _, _).
'$lists_sw'(_, _, [_|_], [X], X, []).
'$lists_sw'(ID, PW, [W|WT], [X|XT], X, XT) :-
    XT \= [], W1 is W/PW, '$lists_sw_p'(ID, W1, WT, X, XT).
'$lists_sw'(ID, PW, [W|WT], [X|XT], Y, [X|RT]) :-
    XT \= [], W1 is W/PW, \+ '$lists_sw_p'(ID, W1, WT, X, XT),
    PW1 is PW-W, '$lists_sw'(ID, PW1, WT, XT, Y, RT).
'$lists_make_list'(0, _, []).
'$lists_make_list'(Len, X, [X|L]) :-
    Len > 0, Len1 is Len-1, '$lists_make_list'(Len1, X, L).
'$lists_unzip'([], [], []).
'$lists_unzip'([(X,Y)|T], [X|R], [Y|S]) :- '$lists_unzip'(T, R, S).

include(_, [], []).
include(P, [X|T], [X|S]) :- call(P, X), include(P, T, S).
include(P, [X|T], S) :- \+ call(P, X), include(P, T, S).
exclude(_, [], []).
exclude(P, [X|T], [X|S]) :- \+ call(P, X), exclude(P, T, S).
exclude(P, [X|T], S) :- call(P, X), exclude(P, T, S).
partition(_, [], [], []).
partition(P, [X|R], [X|S], T) :- call(P, X), partition(P, R, S, T).
partition(P, [X|R], S, [X|T]) :- \+ call(P, X), partition(P, R, S, T).
maplist(_, []).
maplist(P, [A|S]) :- call(P, A), maplist(P, S).
maplist(_, [], []).
maplist(P, [A|S], [B|T]) :- call(P, A, B), maplist(P, S, T).
maplist(_, [], [], []).
maplist(P, [A|S], [B|T], [C|R]) :- call(P, A, B, C), maplist(P, S, T, R).
maplist(_, [], [], [], []).
maplist(P, [A|S], [B|T], [C|R], [D|U]) :-
    call(P, A, B, C, D), maplist(P, S, T, R, U).
foldl(_, [], RR, RR).
foldl(P, [L1|LR], L0, RR) :- call(P, L1, L0, R), foldl(P, LR, R, RR).
foldl(_, [], [], RR, RR).
foldl(P, [L1|LR1], [L2|LR2], L0, RR) :-
    call(P, L1, L2, L0, R), foldl(P, LR1, LR2, R, RR).
foldl(_, [], [], [], RR, RR).
foldl(P, [L1|LR1], [L2|LR2], [L3|LR3], L0, RR) :-
    call(P, L1, L2, L3, L0, R), foldl(P, LR1, LR2, LR3, R, RR).
";

/// The predicates `library(aggregate)` writes as clauses, as it writes
/// them, for a program that loads it: the functions of its aggregates
/// `f<V>`, which `aggregate/5` calls on the values of a group.
const AGGREGATE: &str = r"
avg(L, Avg) :- sum(L, Sum), length(L, Count), Avg is Sum / Count.
sum(L, Sum) :- sum(L, 0, Sum).
min([H|T], Min) :- min(T, H, Min).
max([H|T], Max) :- max(T, H, Max).
sum([], Acc, Acc).
sum([X|T], Acc, Sum) :- Acc1 is Acc + X, sum(T, Acc1, Sum).
min([], Acc, Acc).
min([X|T], Acc, Min) :- X < Acc, min(T, X, Min).
min([X|T], Acc, Min) :- X >= Acc, min(T, Acc, Min).
max([], Acc, Acc).
max([X|T], Acc, Max) :- X >= Acc, max(T, X, Max).
max([X|T], Acc, Max) :- X < Acc, max(T, Acc, Max).
";

/// [`LIBRARY`] read, with [`AGGREGATE`] where `aggregate` says so.
fn library(aggregate: bool) -> &'static Program {
    static READ: OnceLock<Program> = OnceLock::new();
    static WITH_AGGREGATE: OnceLock<Program> = OnceLock::new();
    let parse = |text: &str| crate::syntax::parse_program(text).expect("the library parses");
    if aggregate {
        WITH_AGGREGATE.get_or_init(|| parse(&format!("{LIBRARY}{AGGREGATE}")))
    } else {
        READ.get_or_init(|| parse(LIBRARY))
    }
}

/// The signatures of the heads of `program`'s clauses.
fn heads(program: &Program) -> impl Iterator<Item = (&str, usize)> {
    program
        .clauses
        .iter()
        .flat_map(|c| &c.heads)
        .map(|h| h.atom.signature())
}

/// The bodies of `program`'s clauses and query rules.
fn bodies(program: &Program) -> impl Iterator<Item = &[Literal]> {
    program
        .clauses
        .iter()
        .map(|c| c.body.as_slice())
        .chain(program.query_rules.iter().map(|r| r.body.as_slice()))
}

/// Whether `a` is a call of `call/N` that the program does not define.
fn is_call(a: &Atom, defined: &FxHashSet<(&str, usize)>) -> bool {
    &*a.name == "call" && !a.args.is_empty() && !defined.contains(&a.signature())
}

/// `program` with the library's clauses for the predicates it calls and
/// does not define, and its calls of `call/N` written as the module says;
/// `None` where it has neither.
pub(super) fn added(program: &Program) -> Option<Program> {
    let defined: FxHashSet<(&str, usize)> = heads(program).collect();
    let library = library(program.aggregate);
    // The library's predicates the program does not define, their arities
    // by name.
    let mut offered: FxHashMap<&'static str, Vec<usize>> = FxHashMap::default();
    for sig in heads(library).filter(|sig| !defined.contains(sig)) {
        offered.entry(sig.0).or_default().push(sig.1);
    }
    // The library's predicates the program reaches, and whether it calls
    // call/N.
    type Sig = (&'static str, usize);
    let mut wanted: FxHashSet<Sig> = FxHashSet::default();
    let mut stack: Vec<Sig> = Vec::new();
    let mut any_call = false;
    let reach = |a: &Atom, wanted: &mut FxHashSet<Sig>, stack: &mut Vec<Sig>| {
        let (name, arity) = a.signature();
        if let Some((&name, arities)) = offered.get_key_value(name)
            && arities.contains(&arity)
            && wanted.insert((name, arity))
        {
            stack.push((name, arity));
        }
    };
    for body in bodies(program) {
        calls(body, &mut |a| {
            any_call |= is_call(a, &defined);
            reach(a, &mut wanted, &mut stack);
        });
    }
    while let Some(sig) = stack.pop() {
        for c in library
            .clauses
            .iter()
            .filter(|c| c.heads[0].atom.signature() == sig)
        {
            calls(&c.body, &mut |a| reach(a, &mut wanted, &mut stack));
        }
    }
    if wanted.is_empty() && !any_call {
        return None;
    }
    // The clauses added in groups of their own, which share no switch with
    // the program's.
    let mut group = program
        .clauses
        .iter()
        .map(|c| c.group + 1)
        .max()
        .unwrap_or(0);
    let mut with = program.clone();
    let after = group;
    for c in &library.clauses {
        if wanted.contains(&c.heads[0].atom.signature()) {
            let mut c = c.clone();
            c.group += after;
            group = group.max(c.group + 1);
            with.clauses.push(c);
        }
    }
    // The calls of call/N, and the numbers of arguments added to a goal that
    // is a variable.
    let added: RefCell<FxHashSet<usize>> = RefCell::default();
    let write = |a: &mut Atom| {
        if !is_call(a, &defined) {
            return;
        }
        let (goal, extra) = a.args.split_first().expect("call/N has a goal");
        let n = extra.len();
        let (name, mut args): (Name, Vec<Term>) = match goal {
            Term::Const(p) => (p.clone(), Vec::new()),
            Term::Compound(c) => (c.name.clone(), c.args.to_vec()),
            Term::Var(_) => {
                added.borrow_mut().insert(n);
                (call_name(n).into(), vec![goal.clone()])
            }
            // A number is no goal: the call is left to be refused.
            Term::Int(_) | Term::Float(_) => return,
        };
        args.extend(extra.iter().cloned());
        *a = Atom::new(name, args);
    };
    for c in &mut with.clauses {
        calls_mut(&mut c.body, &write);
    }
    for r in &mut with.query_rules {
        calls_mut(&mut r.body, &write);
    }
    let mut added: Vec<usize> = added.into_inner().into_iter().collect();
    added.sort_unstable();
    if !added.is_empty() {
        let named = argument_names(&with);
        let mut preds: Vec<(Name, usize)> = Vec::new();
        let mut seen: FxHashSet<(Name, usize)> = FxHashSet::default();
        for h in with.clauses.iter().flat_map(|c| &c.heads) {
            let p = (h.atom.name.clone(), h.atom.args.len());
            if !p.0.starts_with('$') && named.contains(&*p.0) && seen.insert(p.clone()) {
                preds.push(p);
            }
        }
        let var = |prefix: &str, i: usize| Term::Var(format!("{prefix}{i}"));
        for n in added {
            for (p, m) in preds.iter().filter(|(_, m)| *m >= n) {
                let bound: Vec<Term> = (0..m - n).map(|i| var("B", i)).collect();
                let goal = if bound.is_empty() {
                    Term::Const(p.clone())
                } else {
                    Compound::term(p.clone(), bound.clone())
                };
                let extra = (0..n).map(|i| var("A", i));
                let head = Atom::new(
                    call_name(n),
                    std::iter::once(goal)
                        .chain(extra.clone())
                        .collect::<Vec<_>>(),
                );
                let body = Atom::new(
                    p.clone(),
                    bound.into_iter().chain(extra).collect::<Vec<_>>(),
                );
                with.clauses
                    .push(Clause::rule(head, vec![Literal::Pos(body)], group));
                group += 1;
            }
        }
    }
    Some(with)
}

/// The name of the auxiliary predicate of `call/(n + 1)`.
fn call_name(n: usize) -> String {
    format!("$call{n}")
}

/// The names of the constants and compounds in the arguments of
/// `program`'s atoms, comparisons and findalls.
fn argument_names(program: &Program) -> FxHashSet<&str> {
    let mut terms: Vec<&Term> = Vec::new();
    fn literals<'p>(body: &'p [Literal], terms: &mut Vec<&'p Term>) {
        for l in body {
            match l {
                Literal::Pos(a) | Literal::Neg(a) => terms.extend(a.args.iter()),
                Literal::Cmp(_, x, y) | Literal::Is(x, y) => terms.extend([x, y]),
                Literal::Builtin(_, args, _) => terms.extend(args),
                Literal::Findall(fa) => {
                    terms.extend([&fa.template, &fa.result]);
                    for conj in &fa.goal {
                        literals(conj, terms);
                    }
                }
                Literal::Collect(_) => {}
            }
        }
    }
    for c in &program.clauses {
        for h in &c.heads {
            terms.extend(h.atom.args.iter());
        }
        literals(&c.body, &mut terms);
    }
    for r in &program.query_rules {
        terms.extend(r.pattern.args.iter());
        literals(&r.body, &mut terms);
    }
    for (a, _) in program.queries.iter().chain(&program.evidence) {
        terms.extend(a.args.iter());
    }
    let mut names = FxHashSet::default();
    while let Some(t) = terms.pop() {
        match t {
            Term::Const(c) => {
                names.insert(&**c);
            }
            Term::Compound(c) => {
                names.insert(&*c.name);
                terms.extend(c.args.iter());
            }
            Term::Int(_) | Term::Var(_) | Term::Float(_) => {}
        }
    }
    names
}
