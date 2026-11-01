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
//! A query or an evidence of a built-in ([`Builtin`]) that the program
//! does not define asks for its answers: the program gets a clause of the
//! built-in's name that calls it.
//!
//! `call(G, A1, ..., An)` calls `G` with the arguments added. Where `G` is
//! written as an atom or a compound of a predicate or a built-in, it is
//! that call. Where it is a variable or a construct (a conjunction, a
//! disjunction, a negation, a comparison, `is`, a findall, `true`, `fail`;
//! [`construct`]), it is a call of `'$call<n>'(G, A1, ..., An)`, which has
//! a clause `'$call<n>'(p(B1, ..., Bk), A1, ..., An) :- p(B1, ..., Bk, A1,
//! ..., An)` (its first argument `p` where `k = 0`) for each predicate or
//! built-in `p/(k + n)` that the program names in an argument, the goals
//! a goal it builds can name. A construct named so has the clauses of the
//! goal it is ([`goal_literal`]): a conjunction `'$call0'((B1, B2)) :-
//! call(B1), call(B2)`, a disjunction one of each alternative, `true` a
//! fact, `fail` none. A variable written as a goal, in a body or in a
//! findall's or a negation's goal, is `call/1` of it. Each such clause is
//! read in every mode of the calls of `call/N`: one whose body is safe
//! only for the goal's own callers (a fact with a variable that only a
//! caller binds) makes the grounding refuse the program, as an unsafe
//! rule. The call is preceded by the test `'$callable'(G, n, Callable,
//! Unknown)` ([`Builtin::Callable`]), `Callable` the list of those goals'
//! names and arities less `n`: a goal bound to anything else has no
//! clause, which ProbLog stops on (its `UnknownClause`), or, where the
//! program says `:- unknown(fail).`, fails, unless it names a built-in,
//! which tddlog calls through a variable only where the program names it.

use std::cell::RefCell;
use std::sync::OnceLock;

use rustc_hash::{FxHashMap, FxHashSet};

use super::terms::{calls, calls_mut};
use crate::syntax::{
    Atom, Builtin, Clause, Compound, Literal, NIL, Name, Program, Term, construct, goal_literal,
};

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

'$findall_goal'((A, B)) :- '$findall_goal'(A), '$findall_goal'(B).
'$findall_goal'((A ; _)) :- '$findall_goal'(A).
'$findall_goal'((_ ; B)) :- '$findall_goal'(B).
'$findall_goal'(G) :- G \= (_, _), G \= (_ ; _), call(G).

enum_groups(Groups, Values, Group, GroupValues) :-
    '$lists_zip'(Groups, Values, Pairs), enum_groups(Pairs, Group, GroupValues).
'$lists_zip'([], [], []).
'$lists_zip'([G|Gs], [V|Vs], [(G,V)|Ps]) :- '$lists_zip'(Gs, Vs, Ps).
";

/// The predicates `library(aggregate)` writes as clauses, as it writes
/// them, for a program that loads it: `aggregate/5` and the functions of
/// its aggregates `f<V>`, which `aggregate/5` calls on the values of a
/// group.
const AGGREGATE: &str = r"
aggregate(AggFunc, Var, Group, Body, (Group, Result)) :-
    all([Group, Body, Var], Body, L),
    maplist(nth0(0), L, LG),
    maplist(nth0(2), L, LV),
    enum_groups(LG, LV, Group, Values),
    call(AggFunc, Values, Result).
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

/// The predicates `library(collect)` writes as clauses, as it writes them,
/// for a program that loads it. Its `CodeBlock => Collector`, which calls
/// `collect_<name>` of the collector's name, is read where it is written
/// ([`collector`]).
const COLLECT: &str = r"
collectgroup(CodeBlock, AggVar, GroupBy, Values) :-
    all_or_none((GroupBy, AggVar), CodeBlock, List1),
    enum_groups(List1, GroupBy, Values).
collect_list(CodeBlock, Y, Result) :- collect_list(CodeBlock, none, Y, Result).
collect_list(CodeBlock, GB, Y, Result) :- collectgroup(CodeBlock, Y, GB, Result).
";

/// [`LIBRARY`] read, with [`AGGREGATE`] and [`COLLECT`] where `program`
/// loads them.
fn library(program: &Program) -> &'static Program {
    static READ: [OnceLock<Program>; 4] = [const { OnceLock::new() }; 4];
    let (aggregate, collect) = (program.aggregate, program.collect);
    READ[usize::from(aggregate) + 2 * usize::from(collect)].get_or_init(|| {
        let mut text = LIBRARY.to_string();
        if aggregate {
            text.push_str(AGGREGATE);
        }
        if collect {
            text.push_str(COLLECT);
        }
        crate::syntax::parse_program(&text).expect("the library parses")
    })
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
    let collected = collectors(program);
    let program = collected.as_ref().unwrap_or(program);
    let defined: FxHashSet<(&str, usize)> = heads(program).collect();
    let library = library(program);
    let (wanted, any_call) = wanted_library_predicates(program, library, &defined);
    // The built-ins the queries and the evidence ask about.
    let mut asked: Vec<(&str, usize, Builtin)> = Vec::new();
    let patterns = program
        .queries
        .iter()
        .chain(&program.evidence)
        .map(|(a, _)| a);
    for a in patterns.chain(program.query_rules.iter().map(|r| &r.pattern)) {
        let (name, arity) = a.signature();
        if let Some(b) = Builtin::of(name, arity)
            && !defined.contains(&(name, arity))
            && !asked.iter().any(|e| e.2 == b)
        {
            asked.push((name, arity, b));
        }
    }
    if wanted.is_empty() && !any_call && asked.is_empty() {
        return collected;
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
    // A built-in asked about is defined by a clause that calls it.
    for (name, arity, b) in asked {
        let vars: Vec<Term> = (0..arity).map(|i| Term::Var(format!("A{i}"))).collect();
        let head = Atom::new(name, vars.clone());
        let body = vec![Literal::Builtin(b, vars, false)];
        with.clauses.push(Clause::rule(head, body, group));
        group += 1;
    }
    // The calls of call/N, and the numbers of arguments added to a goal that
    // is a variable or a construct.
    let dispatched =
        |name: &str, arity: usize| !defined.contains(&(name, arity)) && construct(name, arity);
    let added: RefCell<FxHashSet<usize>> = RefCell::default();
    let write = |a: &mut Atom| {
        if !is_call(a, &defined) {
            return;
        }
        let (goal, extra) = a.args.split_first().expect("call/N has a goal");
        let n = extra.len();
        let (name, mut args): (Name, Vec<Term>) = match goal {
            Term::Const(p) if !dispatched(p, n) => (p.clone(), Vec::new()),
            Term::Compound(c) if !dispatched(&c.name, c.args.len() + n) => {
                (c.name.clone(), c.args.to_vec())
            }
            Term::Var(_) | Term::Const(_) | Term::Compound(_) => {
                added.borrow_mut().insert(n);
                (call_name(n).into(), vec![goal.clone()])
            }
            // A number is no goal: the call is left to be refused.
            Term::Int(_) | Term::Float(_) => return,
        };
        args.extend(extra.iter().cloned());
        *a = Atom::new(name, args);
    };
    for_each_body(&mut with, |body| calls_mut(body, &write));
    if !added.borrow().is_empty() {
        let listed = build_call_clauses(&mut with, &defined, &added, &write, &mut group);
        guard_variable_calls(&mut with, &listed, program.unknown_fails);
    }
    Some(with)
}

type Sig = (&'static str, usize);

fn wanted_library_predicates(
    program: &Program,
    library: &'static Program,
    defined: &FxHashSet<(&str, usize)>,
) -> (FxHashSet<Sig>, bool) {
    // The library's predicates the program does not define, their arities
    // by name.
    let mut offered: FxHashMap<&'static str, Vec<usize>> = FxHashMap::default();
    for sig in heads(library).filter(|sig| !defined.contains(sig)) {
        offered.entry(sig.0).or_default().push(sig.1);
    }
    // The library's predicates the program reaches, and whether it calls
    // call/N.
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
            any_call |= is_call(a, defined);
            reach(a, &mut wanted, &mut stack);
        });
    }
    let clauses = |sig: Sig| {
        library
            .clauses
            .iter()
            .filter(move |c| c.heads[0].atom.signature() == sig)
    };
    let drain = |wanted: &mut FxHashSet<Sig>, stack: &mut Vec<Sig>| {
        while let Some(sig) = stack.pop() {
            for c in clauses(sig) {
                calls(&c.body, &mut |a| reach(a, wanted, stack));
            }
        }
    };
    drain(&mut wanted, &mut stack);
    // Where the program or the library's clauses it reaches call call/N, a
    // goal can name the library's predicates named in an argument.
    loop {
        let reached = || wanted.iter().flat_map(|&sig| clauses(sig));
        let mut calls_n = any_call;
        for c in reached() {
            calls(&c.body, &mut |a| calls_n |= is_call(a, defined));
        }
        if !calls_n {
            break;
        }
        let mut named = argument_names(program.clauses.iter(), Some(program));
        named.extend(argument_names(reached(), None));
        let named: FxHashSet<&str> = named.into_iter().map(|(p, _)| p).collect();
        let mut more: Vec<Sig> = offered
            .iter()
            .filter(|(name, _)| named.contains(*name))
            .flat_map(|(&name, arities)| arities.iter().map(move |&m| (name, m)))
            .filter(|sig| !wanted.contains(sig))
            .collect();
        if more.is_empty() {
            break;
        }
        more.sort_unstable();
        for sig in more {
            wanted.insert(sig);
            stack.push(sig);
        }
        drain(&mut wanted, &mut stack);
    }
    (wanted, any_call)
}

fn build_call_clauses(
    with: &mut Program,
    defined: &FxHashSet<(&str, usize)>,
    added: &RefCell<FxHashSet<usize>>,
    write: &impl Fn(&mut Atom),
    group: &mut u32,
) -> Vec<(usize, Vec<(Name, usize)>)> {
    let dispatched =
        |name: &str, arity: usize| !defined.contains(&(name, arity)) && construct(name, arity);
    let (goals, terms) = discover_callable_goals(with);
    // The clauses of '$call<n>', for each number n of arguments added,
    // and the goals each can be given, which its test lists.
    let var = |prefix: &str, i: usize| Term::Var(format!("{prefix}{i}"));
    let mut listed: Vec<(usize, Vec<(Name, usize)>)> = Vec::new();
    loop {
        let next = added
            .borrow()
            .iter()
            .copied()
            .filter(|n| listed.iter().all(|e| e.0 != *n))
            .min();
        let Some(n) = next else {
            break;
        };
        let mut callable = Vec::new();
        let others = terms.iter().map(|(p, k)| (p, k + n)).filter(|&(p, m)| {
            (Builtin::of(p, m).is_some() || construct(p, m)) && !defined.contains(&(&**p, m))
        });
        let preds = goals.iter().filter(|(_, m)| *m >= n).map(|(p, m)| (p, *m));
        for (p, m) in preds.chain(others) {
            let bound: Vec<Term> = (0..m - n).map(|i| var("B", i)).collect();
            let term = |p: &Name, args: Vec<Term>| {
                if args.is_empty() {
                    Term::Const(p.clone())
                } else {
                    Compound::term(p.clone(), args)
                }
            };
            let goal = term(p, bound.clone());
            let all: Vec<Term> = bound
                .into_iter()
                .chain((0..n).map(|i| var("A", i)))
                .collect();
            let bodies: Vec<Vec<Literal>> = if !dispatched(p, m) {
                vec![vec![Literal::Pos(Atom::new(p.clone(), all.clone()))]]
            } else {
                match (&**p, m) {
                    ("true", 0) => vec![Vec::new()],
                    ("fail" | "false", 0) => Vec::new(),
                    _ => match goal_literal(term(p, all.clone())) {
                        Some(l) => vec![vec![l]],
                        None => continue,
                    },
                }
            };
            let head = Atom::new(
                call_name(n),
                std::iter::once(goal)
                    .chain(all[all.len() - n..].iter().cloned())
                    .collect::<Vec<_>>(),
            );
            for mut body in bodies {
                calls_mut(&mut body, write);
                with.clauses.push(Clause::rule(head.clone(), body, *group));
                *group += 1;
            }
            callable.push((p.clone(), m - n));
        }
        listed.push((n, callable));
    }
    listed
}

type CallableGoals = (Vec<(Name, usize)>, Vec<(Name, usize)>);

/// The goals a goal it builds can name: the predicates it defines, named in
/// an argument, and the terms written in an argument, in the order of their
/// names, that are a built-in's or a construct's goal with the arguments
/// added.
fn discover_callable_goals(with: &Program) -> CallableGoals {
    let written = argument_names(with.clauses.iter(), Some(with));
    let named: FxHashSet<&str> = written.iter().map(|&(p, _)| p).collect();
    let mut goals: Vec<(Name, usize)> = Vec::new();
    let mut seen: FxHashSet<(Name, usize)> = FxHashSet::default();
    for h in with.clauses.iter().flat_map(|c| &c.heads) {
        let p = (h.atom.name.clone(), h.atom.args.len());
        if !p.0.starts_with('$') && named.contains(&*p.0) && seen.insert(p.clone()) {
            goals.push(p);
        }
    }
    let mut terms: Vec<(Name, usize)> = written
        .into_iter()
        .filter(|(p, _)| !p.starts_with('$'))
        .map(|(p, k)| (p.into(), k))
        .collect();
    terms.sort_unstable();
    (goals, terms)
}

fn guard_variable_calls(
    with: &mut Program,
    listed: &[(usize, Vec<(Name, usize)>)],
    unknown_fails: bool,
) {
    // The test before each call of a goal that is a variable.
    let unknown = if unknown_fails { "fail" } else { "error" };
    let unknown = Term::Const(unknown.into());
    let tests: FxHashMap<String, Literal> = listed
        .iter()
        .map(|&(n, ref goals)| {
            let callable = Term::list(
                goals.iter().map(|(p, k)| {
                    Compound::term(
                        "/".into(),
                        vec![Term::Const(p.clone()), Term::Int(*k as i64)],
                    )
                }),
                Term::Const(NIL.into()),
            );
            let test = Literal::Builtin(
                Builtin::Callable,
                vec![
                    Term::Var(String::new()),
                    Term::Int(n as i64),
                    callable,
                    unknown.clone(),
                ],
                false,
            );
            (call_name(n), test)
        })
        .collect();
    let test = |a: &Atom| {
        let mut t = tests.get(&*a.name)?.clone();
        if let Literal::Builtin(_, args, _) = &mut t {
            args[0] = a.args[0].clone();
        }
        Some(t)
    };
    for_each_body(with, |body| guard(body, &test));
}

fn for_each_body(program: &mut Program, mut f: impl FnMut(&mut Vec<Literal>)) {
    for c in &mut program.clauses {
        f(&mut c.body);
    }
    for r in &mut program.query_rules {
        f(&mut r.body);
    }
}

/// `program` with library(collect)'s calls `CodeBlock => Collector` read
/// as the calls its clauses of `'=>'/2` build: `collect_c(CodeBlock, G,
/// A1, ..., An)` for a collector `G/c(A1, ..., An)`, `collect_c(CodeBlock,
/// none, A1, ..., An)` for a collector `c(A1, ..., An)`; `None` where the
/// program does not load the library or makes no such call. A collector
/// that is a variable is left a call of `'=>'/2`, which has no clauses.
fn collectors(program: &Program) -> Option<Program> {
    if !program.collect || heads(program).any(|sig| sig == ("=>", 2)) {
        return None;
    }
    let changed = std::cell::Cell::new(false);
    let collector = |a: &mut Atom| {
        let [block, collector] = &*a.args else {
            return;
        };
        if &*a.name != "=>" {
            return;
        }
        let (group, c) = match collector.as_compound() {
            Some((slash, [g, c])) if &**slash == "/" => (g.clone(), c),
            _ => (Term::Const("none".into()), collector),
        };
        let (name, args) = match c {
            Term::Const(name) => (name, &[][..]),
            Term::Compound(c) => (&c.name, &c.args[..]),
            _ => return,
        };
        let args: Vec<Term> = [block.clone(), group]
            .into_iter()
            .chain(args.iter().cloned())
            .collect();
        *a = Atom::new(format!("collect_{name}"), args);
        changed.set(true);
    };
    let mut with = program.clone();
    for_each_body(&mut with, |body| calls_mut(body, &collector));
    changed.get().then_some(with)
}

/// `body` with the `test` of each call it makes, in it or in a findall's
/// goal, put before the call.
fn guard(body: &mut Vec<Literal>, test: &impl Fn(&Atom) -> Option<Literal>) {
    let mut i = 0;
    while i < body.len() {
        if let Literal::Findall(fa) = &mut body[i] {
            for conj in &mut fa.goal {
                guard(conj, test);
            }
        }
        let t = match &body[i] {
            Literal::Pos(a) | Literal::Neg(a) => test(a),
            _ => None,
        };
        if let Some(t) = t {
            body.insert(i, t);
            i += 1;
        }
        i += 1;
    }
}

/// The name of the auxiliary predicate of `call/(n + 1)`.
fn call_name(n: usize) -> String {
    format!("$call{n}")
}

/// Whether `name` is that of the auxiliary predicate of a `call/N`.
pub(super) fn is_call_name(name: &str) -> bool {
    name.strip_prefix("$call")
        .is_some_and(|n| n.parse::<usize>().is_ok())
}

/// The names and arities of the constants and compounds in the arguments
/// of the atoms, comparisons and findalls of `clauses` and, where it is
/// given, of `program`'s queries, evidence and query rules.
fn argument_names<'p>(
    clauses: impl Iterator<Item = &'p Clause>,
    program: Option<&'p Program>,
) -> FxHashSet<(&'p str, usize)> {
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
    for c in clauses {
        for h in &c.heads {
            terms.extend(h.atom.args.iter());
        }
        literals(&c.body, &mut terms);
    }
    if let Some(program) = program {
        for r in &program.query_rules {
            terms.extend(r.pattern.args.iter());
            literals(&r.body, &mut terms);
        }
        for (a, _) in program.queries.iter().chain(&program.evidence) {
            terms.extend(a.args.iter());
        }
    }
    let mut names = FxHashSet::default();
    while let Some(t) = terms.pop() {
        match t {
            Term::Const(c) => {
                names.insert((&**c, 0));
            }
            Term::Compound(c) => {
                names.insert((&*c.name, c.args.len()));
                terms.extend(c.args.iter());
            }
            Term::Int(_) | Term::Var(_) | Term::Float(_) => {}
        }
    }
    names
}
