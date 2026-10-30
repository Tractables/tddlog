//! Top-down grounding: a program whose demand is not Datalog, resolved
//! from its queries as ProbLog resolves it.
//!
//! The magic rewrite ([`super::super::magic`]) demands each argument of a
//! call bound or free. A call whose argument holds variables inside a
//! term (`word([_, _])`, a list whose length a query fixes and whose
//! items the grammar binds), or an answer that leaves a variable unbound
//! and shared between its arguments (`reverse([], L, L)`, whose `L` the
//! caller binds later), is neither: no demand makes such a rule safe, in
//! the rewritten program or in the plain one. Such a program is resolved
//! here. SLD resolution runs in the order of the source, every call is
//! tabled by its variant, and every probabilistic choice is taken as
//! possible. Each clause instance proved under ground answers is a ground
//! rule, and the engine grounds those rules as it grounds any program
//! without variables ([`ground`]).
//!
//! - A table holds the answers of a call. A call that is a variant of one
//!   still being resolved reads the answers found so far. The first call
//!   of such a cycle then resolves its clauses again, and so do the calls
//!   of the cycle it reaches, until no table gains an answer (linear
//!   tabling). Its tables are then complete.
//! - An answer is an atom of the ground program, an answer with variables
//!   too, its variables named, as ProbLog gives it a node of its own: it
//!   holds for every value of them. A query answered so is refused, since
//!   ProbLog prints its variables by names of its own.
//! - The calls of a predicate share the atoms of their answers, which is
//!   sound where an answer is proved alike whatever the call bound. An
//!   identity or a `\=` of terms that unify and are not identical breaks
//!   that: it compares otherwise once a call binds their variables, and
//!   ProbLog answers by what each call proved. It is refused.
//! - A negation is of a ground call whose table is complete.
//! - `is/2`, the comparisons and the list built-ins are the engine's
//!   ([`super::solutions`]), on the values of ground terms. Prolog's
//!   `length/2` also makes a list of fresh variables.
//! - A findall calls its goal with its shared variables bound to ground
//!   terms, and lists the solutions of the complete table as the engine
//!   does ([`lists`]): each clause instance of the predicates it reads is
//!   recorded as a derivation, its clause's group and what each body
//!   literal reads.
//!
//! The answers, the rules and the derivations count against the
//! grounding's bound, and a ground term deeper than
//! [`MAX_DEPTH`](super::MAX_DEPTH), or calls or a term with variables
//! nested deeper than [`MAX_NESTING`], give the grounding up with the goal
//! that reached them.

use std::rc::Rc;
use std::sync::{Arc, LazyLock};

use rustc_hash::{FxHashMap, FxHashSet};

use super::lists::{self, Step};
use super::{GroundError, Grounder, Prob, Val, Weights, compare, ground_normalized, solutions};
use crate::ground::GroundProgram;
use crate::ground::normalize::{Driver, NormProgram};
use crate::syntax::{Args, Atom, Builtin, Clause, CmpOp, Compound, Literal, Name, Term};

/// The deepest nesting of calls resolved, and of a term with variables:
/// each level takes a few frames of the stack. A ground term is bound by
/// [`MAX_DEPTH`](super::MAX_DEPTH).
const MAX_NESTING: usize = 1_000;

/// A term being resolved: a ground term's value, a variable, or a compound
/// term with a variable in it.
#[derive(Clone, Debug)]
enum T {
    G(Val),
    X(u32),
    F(u32, Rc<[T]>),
}

/// A term of a clause, its variables numbered from 0 in the clause.
#[derive(Clone, Debug)]
enum Tpl {
    G(Val),
    X(u32),
    F(u32, Box<[Tpl]>),
}

/// An item of a call's or an answer's arguments written in prefix order,
/// the variables numbered by their first places: two calls are variants
/// where their keys are equal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum K {
    G(Val),
    X(u32),
    F(u32, u32),
}

type Key = Rc<[K]>;

/// A body literal of a clause.
enum Lit {
    /// A call of a predicate, negated where the flag says so.
    Call(u32, Box<[Tpl]>, bool),
    Unify(Tpl, Tpl),
    Cmp(CmpOp, Tpl, Tpl),
    Is(Tpl, Tpl),
    Builtin(Builtin, Box<[Tpl]>, bool),
    Collect(Box<Collect>),
}

/// A findall: its goal's predicate, whose arguments are the shared terms,
/// the template and the goal's other variables.
struct Collect {
    aux: u32,
    shared: Box<[Tpl]>,
    result: Tpl,
    all: bool,
    text: Arc<str>,
}

struct Rule {
    args: Box<[Tpl]>,
    body: Box<[Lit]>,
    vars: u32,
    /// The rule's place in the program, for the errors that name it.
    at: usize,
    /// The group of the clause, for the order of a findall's solutions.
    group: u32,
}

/// What defines a predicate.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Rules,
    /// The facts of the program's `facts` at this place.
    Facts(usize),
    Switch,
    Undefined,
}

struct Pred {
    name: Name,
    arity: usize,
    kind: Kind,
    rules: Vec<u32>,
    /// A query rule's driver, whose atoms the ground program leaves out.
    driver: bool,
    /// Read by a findall: the derivations of its atoms are recorded.
    counted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    /// Being resolved: its frame is on the stack.
    Active,
    /// Resolved once in a cycle whose first call is still being resolved.
    Open,
    Complete,
}

/// The facts of a predicate with a value at a place, by that value.
type FactIndex = Rc<FxHashMap<Val, Vec<u32>>>;

/// A ground rule: its head and its body's literals, each atom with its sign.
type GroundRule = (u32, Box<[(bool, u32)]>);

/// The lists a findall gives ([`lists::collect`]).
type Lists = Rc<[lists::Listed]>;

struct Table {
    pred: u32,
    call: Key,
    answers: Vec<(Key, u32)>,
    known: FxHashSet<Key>,
    state: State,
    /// The first table of its cycle, where it is open.
    low: u32,
    /// The pass it was last resolved in.
    pass: u32,
}

/// A table being resolved: the first table of a cycle it reached, and
/// whether it read a table that was not complete.
struct Frame {
    table: u32,
    low: u32,
    looped: bool,
}

/// Whether the terms `a` and `b`, their bound variables replaced
/// ([`TopDown::full`]), are the same term, variables and all.
fn identical(a: &T, b: &T) -> bool {
    match (a, b) {
        (T::G(x), T::G(y)) => x == y,
        (T::X(x), T::X(y)) => x == y,
        (T::F(m, xs), T::F(n, ys)) => {
            m == n && xs.len() == ys.len() && xs.iter().zip(ys.iter()).all(|(x, y)| identical(x, y))
        }
        _ => false,
    }
}

/// The ground program of `norm`, resolved from its queries, its evidence
/// and its query rules; [`GroundError::OverBound`] once its answers and
/// rules outgrow `bound` terms.
pub(crate) fn ground(
    norm: &NormProgram,
    bound: Option<usize>,
) -> Result<GroundProgram, GroundError> {
    let mut td = TopDown::new(norm, bound)?;
    for (a, _) in norm.queries.iter().chain(&norm.evidence) {
        td.top(a)?;
    }
    let mut queries = norm.queries.clone();
    let mut evidence = norm.evidence.clone();
    for d in &norm.drivers {
        td.driver(d, &mut queries, &mut evidence)?;
    }
    let out = td.program(norm, queries, evidence);
    ground_normalized(&out, bound)
}

struct TopDown<'a> {
    /// The terms' values and the built-ins.
    g: Grounder<'a>,
    norm: &'a NormProgram,
    preds: Vec<Pred>,
    pred_ix: FxHashMap<(Name, usize), u32>,
    rules: Rc<[Rule]>,
    /// The variables' bindings, and the variables bound, in order.
    binds: Vec<Option<T>>,
    trail: Vec<u32>,
    tables: Vec<Table>,
    table_ix: FxHashMap<(u32, Key), u32>,
    /// The tables that are not complete, in the order made.
    open: Vec<u32>,
    frames: Vec<Frame>,
    pass: u32,
    /// The answers found so far, all tables'.
    added: u64,
    /// The facts' values, by predicate, and their indexes by a place.
    facts: FxHashMap<u32, Rc<[Val]>>,
    fact_ix: FxHashMap<(u32, usize), FactIndex>,
    /// The ground atoms, by predicate and values, and the ground rules.
    atoms: Vec<(u32, Box<[Val]>)>,
    atom_ix: FxHashMap<(u32, Box<[Val]>), u32>,
    ground: Vec<GroundRule>,
    ground_ix: FxHashSet<GroundRule>,
    /// The derivations of the atoms of counted predicates and of lists, by
    /// atom: each its clause's group and what its body reads.
    derivations: FxHashMap<u32, Vec<(u32, Vec<Step>)>>,
    /// The proofs of the atoms findalls read ([`lists::proofs`]).
    proofs: FxHashMap<u32, Rc<[lists::Proof]>>,
    /// The lists of each findall's goal and shared values.
    collected: FxHashMap<(u32, Box<[Val]>), Lists>,
    /// The predicate of the lists' atoms, and the next one's number.
    states: Option<(u32, i64)>,
    /// The answers, the rules' terms and the derivations' steps, against
    /// `bound`.
    size: usize,
    bound: usize,
}

impl<'a> TopDown<'a> {
    fn new(norm: &'a NormProgram, bound: Option<usize>) -> Result<Self, GroundError> {
        let mut td = TopDown {
            g: Grounder::new(&[], 64, None),
            norm,
            preds: Vec::new(),
            pred_ix: FxHashMap::default(),
            rules: Rc::new([]),
            binds: Vec::new(),
            trail: Vec::new(),
            tables: Vec::new(),
            table_ix: FxHashMap::default(),
            open: Vec::new(),
            frames: Vec::new(),
            pass: 0,
            added: 0,
            facts: FxHashMap::default(),
            fact_ix: FxHashMap::default(),
            atoms: Vec::new(),
            atom_ix: FxHashMap::default(),
            ground: Vec::new(),
            ground_ix: FxHashSet::default(),
            derivations: FxHashMap::default(),
            proofs: FxHashMap::default(),
            collected: FxHashMap::default(),
            states: None,
            size: 0,
            bound: bound.unwrap_or(usize::MAX),
        };
        for (name, arity) in &norm.counted {
            let p = td.pred(name, *arity);
            td.preds[p as usize].counted = true;
        }
        for (name, arity) in norm.switches.keys() {
            let p = td.pred(name, *arity);
            td.preds[p as usize].kind = Kind::Switch;
        }
        for (k, (_, f)) in norm.facts.iter().enumerate() {
            let p = td.pred(&f.name, f.arity);
            td.preds[p as usize].kind = Kind::Facts(k);
        }
        for d in &norm.drivers {
            let p = td.pred(&d.atom.name, d.atom.args.len());
            td.preds[p as usize].driver = true;
        }
        let mut rules = Vec::with_capacity(norm.rules.len());
        for (at, c) in norm.rules.iter().enumerate() {
            let head = &c.heads[0].atom;
            let p = td.pred(&head.name, head.args.len());
            if td.preds[p as usize].kind != Kind::Undefined
                && td.preds[p as usize].kind != Kind::Rules
            {
                return Err(GroundError::Unsupported(format!(
                    "rule head {head} is a probabilistic switch predicate"
                )));
            }
            td.preds[p as usize].kind = Kind::Rules;
            td.preds[p as usize].rules.push(rules.len() as u32);
            let rule = td.rule(c, at)?;
            rules.push(rule);
        }
        td.rules = rules.into();
        Ok(td)
    }

    /// The predicate `name/arity`, numbered where it is new.
    fn pred(&mut self, name: &Name, arity: usize) -> u32 {
        let next = self.preds.len() as u32;
        let p = *self.pred_ix.entry((name.clone(), arity)).or_insert(next);
        if p == next {
            self.preds.push(Pred {
                name: name.clone(),
                arity,
                kind: Kind::Undefined,
                rules: Vec::new(),
                driver: false,
                counted: false,
            });
        }
        p
    }

    /// The clause `c`, at place `at` of the program, compiled.
    fn rule(&mut self, c: &Clause, at: usize) -> Result<Rule, GroundError> {
        let mut vars: Vec<String> = Vec::new();
        let head = &c.heads[0].atom;
        let args = head.args.iter().map(|t| self.tpl(t, &mut vars)).collect();
        let mut body = Vec::with_capacity(c.body.len());
        for l in &c.body {
            body.push(match l {
                Literal::Pos(a) | Literal::Neg(a) => {
                    let p = self.pred(&a.name, a.args.len());
                    let args = a.args.iter().map(|t| self.tpl(t, &mut vars)).collect();
                    Lit::Call(p, args, matches!(l, Literal::Neg(_)))
                }
                Literal::Cmp(CmpOp::Eq, a, b) => {
                    Lit::Unify(self.tpl(a, &mut vars), self.tpl(b, &mut vars))
                }
                Literal::Cmp(op, a, b) => {
                    Lit::Cmp(*op, self.tpl(a, &mut vars), self.tpl(b, &mut vars))
                }
                Literal::Is(v, e) => Lit::Is(self.tpl(v, &mut vars), self.tpl(e, &mut vars)),
                Literal::Builtin(b, args, negated) => {
                    let args = args.iter().map(|t| self.tpl(t, &mut vars)).collect();
                    Lit::Builtin(*b, args, *negated)
                }
                Literal::Collect(k) => Lit::Collect(Box::new(Collect {
                    aux: self.pred(&k.aux, k.arity),
                    shared: k.shared.iter().map(|t| self.tpl(t, &mut vars)).collect(),
                    result: self.tpl(&k.result, &mut vars),
                    all: k.all,
                    text: k.text.clone(),
                })),
                Literal::Findall(_) => unreachable!("a findall is lowered before grounding"),
            });
        }
        Ok(Rule {
            args,
            body: body.into(),
            vars: vars.len() as u32,
            at,
            group: c.group,
        })
    }

    /// The term `t` of a clause whose variables are `vars`.
    fn tpl(&mut self, t: &Term, vars: &mut Vec<String>) -> Tpl {
        match t {
            Term::Var(v) => Tpl::X(match vars.iter().position(|w| w == v) {
                Some(i) => i as u32,
                None => {
                    vars.push(v.clone());
                    vars.len() as u32 - 1
                }
            }),
            Term::Compound(c) if !t.is_ground() => {
                let name = self.g.syms.intern(&c.name);
                Tpl::F(name, c.args.iter().map(|a| self.tpl(a, vars)).collect())
            }
            t => Tpl::G(self.g.val(t)),
        }
    }

    /// The term `t` of a goal, its variables named in `vars`, made where
    /// they are new.
    fn term(&mut self, t: &Term, vars: &mut Vec<(String, T)>) -> T {
        match t {
            Term::Var(v) => match vars.iter().find(|(w, _)| w == v) {
                Some((_, x)) => x.clone(),
                None => {
                    let x = self.fresh_var();
                    vars.push((v.clone(), x.clone()));
                    x
                }
            },
            Term::Compound(c) if !t.is_ground() => {
                let name = self.g.syms.intern(&c.name);
                T::F(name, c.args.iter().map(|a| self.term(a, vars)).collect())
            }
            t => T::G(self.g.val(t)),
        }
    }

    fn fresh_var(&mut self) -> T {
        self.binds.push(None);
        T::X(self.binds.len() as u32 - 1)
    }

    /// The term of a clause's `t` whose variables start at `base`.
    fn inst(t: &Tpl, base: u32) -> T {
        match t {
            Tpl::G(v) => T::G(*v),
            Tpl::X(i) => T::X(base + i),
            Tpl::F(name, args) => T::F(*name, args.iter().map(|a| Self::inst(a, base)).collect()),
        }
    }

    /// `t` with the bindings of the variables it is followed through.
    fn deref(&self, t: &T) -> T {
        let mut t = t.clone();
        while let T::X(x) = t {
            match &self.binds[x as usize] {
                Some(b) => t = b.clone(),
                None => break,
            }
        }
        t
    }

    fn mark(&self) -> (usize, usize) {
        (self.binds.len(), self.trail.len())
    }

    /// Undoes the bindings made, and drops the variables made, since `mark`.
    fn undo(&mut self, (binds, trail): (usize, usize)) {
        for x in self.trail.drain(trail..) {
            self.binds[x as usize] = None;
        }
        self.binds.truncate(binds);
    }

    fn unify(&mut self, a: &T, b: &T) -> bool {
        let (a, b) = (self.deref(a), self.deref(b));
        match (&a, &b) {
            (T::X(x), T::X(y)) if x == y => true,
            (T::X(x), _) => {
                self.binds[*x as usize] = Some(b.clone());
                self.trail.push(*x);
                true
            }
            (_, T::X(y)) => {
                self.binds[*y as usize] = Some(a.clone());
                self.trail.push(*y);
                true
            }
            (T::G(x), T::G(y)) => x == y,
            (T::G(Val::Fn(id)), T::F(name, args)) | (T::F(name, args), T::G(Val::Fn(id))) => {
                let key = self.g.terms.keys.list(*id as usize);
                if key.len() != args.len() + 1 || key[0] != Val::Sym(*name) {
                    return false;
                }
                let vals = key[1..].to_vec();
                args.iter().zip(vals).all(|(a, v)| self.unify(a, &T::G(v)))
            }
            (T::F(m, xs), T::F(n, ys)) => {
                m == n
                    && xs.len() == ys.len()
                    && xs.iter().zip(ys.iter()).all(|(x, y)| self.unify(x, y))
            }
            _ => false,
        }
    }

    fn unify_all(&mut self, xs: &[T], ys: &[T]) -> bool {
        xs.iter().zip(ys).all(|(x, y)| self.unify(x, y))
    }

    /// Whether `a` and `b` unify, binding nothing.
    fn unifiable(&mut self, a: &[T], b: &[T]) -> bool {
        let mark = self.mark();
        let ok = self.unify_all(a, b);
        self.undo(mark);
        ok
    }

    /// `t` with every bound variable replaced, its ground compound terms
    /// interned; the term-depth bound's error where one is too deep.
    fn full(&mut self, t: &T) -> Result<T, GroundError> {
        match self.deref(t) {
            T::F(name, args) => {
                let args: Vec<T> = args
                    .iter()
                    .map(|a| self.full(a))
                    .collect::<Result<_, _>>()?;
                if !args.iter().all(|a| matches!(a, T::G(_))) {
                    return Ok(T::F(name, args.into()));
                }
                let vals: Vec<Val> = args
                    .iter()
                    .map(|a| match a {
                        T::G(v) => *v,
                        _ => unreachable!("ground"),
                    })
                    .collect();
                let v = self.g.intern_fn(name, &vals, true);
                if let Some(e) = self.g.error.take() {
                    return Err(e);
                }
                if self.g.refused {
                    return Err(GroundError::OutOfMemory);
                }
                Ok(T::G(v))
            }
            t => Ok(t),
        }
    }

    /// The values of `args` (made [`TopDown::full`]) with each variable a
    /// symbol of its own, `_0`, `_1` and on in the order of first
    /// occurrence: the atom of an answer with variables.
    fn named(&mut self, args: &[T]) -> Result<Box<[Val]>, GroundError> {
        let mut seen: Vec<u32> = Vec::new();
        let mut out = Vec::with_capacity(args.len());
        for a in args {
            let t = self.vars_named(a, &mut seen);
            match self.full(&t)? {
                T::G(v) => out.push(v),
                _ => unreachable!("every variable named"),
            }
        }
        Ok(out.into())
    }

    fn vars_named(&mut self, t: &T, seen: &mut Vec<u32>) -> T {
        match self.deref(t) {
            T::X(v) => {
                let k = seen.iter().position(|&w| w == v).unwrap_or_else(|| {
                    seen.push(v);
                    seen.len() - 1
                });
                T::G(Val::Sym(self.g.syms.intern(&format!("_{k}").into())))
            }
            T::F(name, args) => {
                let args: Vec<T> = args.iter().map(|a| self.vars_named(a, seen)).collect();
                T::F(name, args.into())
            }
            g => g,
        }
    }

    fn full_all(&mut self, ts: &[T]) -> Result<Vec<T>, GroundError> {
        ts.iter().map(|t| self.full(t)).collect()
    }

    /// The error of a `what` (a call, an answer) of `p` whose term with
    /// variables is nested deeper than [`MAX_NESTING`].
    fn too_deep(&self, p: u32, what: &str) -> GroundError {
        let pred = &self.preds[p as usize];
        GroundError::Unsupported(format!(
            "the top-down grounding reached a {what} of {}/{} with a term with variables \
             nested deeper than {MAX_NESTING}",
            pred.name, pred.arity
        ))
    }

    /// The key of the arguments `args`, every variable in them unbound
    /// ([`Self::full`]); `None` where a term with variables is nested deeper
    /// than [`MAX_NESTING`].
    fn key(args: &[T]) -> Option<Key> {
        fn walk(t: &T, depth: u32, out: &mut Vec<K>, vars: &mut Vec<u32>) -> Option<()> {
            match t {
                T::G(v) => out.push(K::G(*v)),
                T::X(x) => {
                    let i = match vars.iter().position(|y| y == x) {
                        Some(i) => i,
                        None => {
                            vars.push(*x);
                            vars.len() - 1
                        }
                    };
                    out.push(K::X(i as u32));
                }
                T::F(name, args) => {
                    if depth as usize > MAX_NESTING {
                        return None;
                    }
                    out.push(K::F(*name, args.len() as u32));
                    for a in args.iter() {
                        walk(a, depth + 1, out, vars)?;
                    }
                }
            }
            Some(())
        }
        let (mut out, mut vars) = (Vec::with_capacity(args.len()), Vec::new());
        for a in args {
            walk(a, 0, &mut out, &mut vars)?;
        }
        Some(out.into())
    }

    /// The `arity` terms of `key`, its variables made fresh.
    fn unkey(&mut self, key: &[K], arity: usize) -> Vec<T> {
        fn read(key: &[K], at: &mut usize, base: u32) -> T {
            let k = key[*at];
            *at += 1;
            match k {
                K::G(v) => T::G(v),
                K::X(i) => T::X(base + i),
                K::F(name, n) => T::F(name, (0..n).map(|_| read(key, at, base)).collect()),
            }
        }
        let vars = key
            .iter()
            .filter_map(|k| match k {
                K::X(i) => Some(i + 1),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        let base = self.binds.len() as u32;
        self.binds.resize(self.binds.len() + vars as usize, None);
        let mut at = 0;
        (0..arity).map(|_| read(key, &mut at, base)).collect()
    }

    /// The term `t` as the program writes it, a variable named by its number.
    fn show_term(&mut self, t: &T) -> Term {
        match self.deref(t) {
            T::G(v) => self.g.term_of(v),
            T::X(x) => Term::Var(format!("_{x}")),
            T::F(name, args) => {
                let args = args.iter().map(|a| self.show_term(a)).collect();
                Compound::term(self.g.syms.names[name as usize].clone(), args)
            }
        }
    }

    fn show(&mut self, p: u32, args: &[T]) -> Atom {
        let args: Args = args.iter().map(|a| self.show_term(a)).collect();
        Atom::new(self.preds[p as usize].name.clone(), args)
    }

    /// The atom of `p` with the values `vals`, numbered where it is new.
    fn atom(&mut self, p: u32, vals: Box<[Val]>) -> Result<u32, GroundError> {
        if let Some(&a) = self.atom_ix.get(&(p, vals.clone())) {
            return Ok(a);
        }
        if self.atoms.try_reserve(1).is_err() {
            return Err(GroundError::OutOfMemory);
        }
        let a = self.atoms.len() as u32;
        self.atoms.push((p, vals.clone()));
        self.atom_ix.insert((p, vals), a);
        Ok(a)
    }

    /// Counts `n` more terms against the bound.
    fn grow(&mut self, n: usize) -> Result<(), GroundError> {
        self.size += n;
        if self.size > self.bound {
            return Err(GroundError::OverBound);
        }
        Ok(())
    }

    /// The table of the call of `p` with `args`, resolved: complete, or
    /// holding the answers found so far where it is part of a cycle still
    /// being resolved.
    fn call(&mut self, p: u32, args: &[T]) -> Result<u32, GroundError> {
        let args = self.full_all(args)?;
        let Some(key) = Self::key(&args) else {
            return Err(self.too_deep(p, "call"));
        };
        if let Some(&t) = self.table_ix.get(&(p, key.clone())) {
            match self.tables[t as usize].state {
                State::Complete => {}
                State::Active => self.reads(t),
                State::Open => {
                    if self.tables[t as usize].pass != self.pass {
                        self.resolve_table(t)?;
                    }
                    if self.tables[t as usize].state == State::Open {
                        self.reads(self.tables[t as usize].low);
                    }
                }
            }
            return Ok(t);
        }
        if self.tables.try_reserve(1).is_err() {
            return Err(GroundError::OutOfMemory);
        }
        self.grow(key.len())?;
        let t = self.tables.len() as u32;
        self.tables.push(Table {
            pred: p,
            call: key.clone(),
            answers: Vec::new(),
            known: FxHashSet::default(),
            state: State::Active,
            low: t,
            pass: self.pass,
        });
        self.table_ix.insert((p, key), t);
        self.open.push(t);
        self.resolve_table(t)?;
        Ok(t)
    }

    /// The table being resolved read the table that is not complete of a
    /// cycle whose first table is `low`.
    fn reads(&mut self, low: u32) {
        if let Some(f) = self.frames.last_mut() {
            f.low = f.low.min(low);
            f.looped = true;
        }
    }

    /// Resolves the clauses of table `t`'s call, again while it is the
    /// first of a cycle that gained answers.
    fn resolve_table(&mut self, t: u32) -> Result<(), GroundError> {
        if self.frames.len() >= MAX_NESTING {
            let key = self.tables[t as usize].call.clone();
            let p = self.tables[t as usize].pred;
            let mark = self.mark();
            let args = self.unkey(&key, self.preds[p as usize].arity);
            let goal = self.show(p, &args);
            self.undo(mark);
            return Err(GroundError::Unsupported(format!(
                "the top-down grounding nests calls deeper than {MAX_NESTING}, at {goal}"
            )));
        }
        self.frames.push(Frame {
            table: t,
            low: t,
            looped: false,
        });
        self.tables[t as usize].state = State::Active;
        loop {
            let before = self.added;
            self.tables[t as usize].pass = self.pass;
            self.clauses(t)?;
            let f = self.frames.last_mut().expect("a frame");
            if f.low < t {
                self.tables[t as usize].state = State::Open;
                self.tables[t as usize].low = f.low;
                break;
            }
            if !f.looped || self.added == before {
                while let Some(&u) = self.open.last()
                    && u >= t
                {
                    self.open.pop();
                    self.tables[u as usize].state = State::Complete;
                }
                break;
            }
            f.looped = false;
            self.pass += 1;
        }
        let f = self.frames.pop().expect("a frame");
        if f.low < f.table
            && let Some(parent) = self.frames.last_mut()
        {
            parent.low = parent.low.min(f.low);
            parent.looped = true;
        }
        Ok(())
    }

    /// Resolves the call of table `t` with each clause of its predicate.
    fn clauses(&mut self, t: u32) -> Result<(), GroundError> {
        let p = self.tables[t as usize].pred;
        let rules = self.rules.clone();
        for k in 0..self.preds[p as usize].rules.len() {
            let rule = &rules[self.preds[p as usize].rules[k] as usize];
            let mark = self.mark();
            let key = self.tables[t as usize].call.clone();
            let call = self.unkey(&key, rule.args.len());
            let base = self.binds.len() as u32;
            self.binds
                .resize(self.binds.len() + rule.vars as usize, None);
            let head: Vec<T> = rule.args.iter().map(|a| Self::inst(a, base)).collect();
            if self.unify_all(&head, &call) {
                self.body(t, rule, base, 0, &call, &mut Vec::new())?;
            }
            self.undo(mark);
        }
        Ok(())
    }

    /// Resolves the literals of `rule` from the `i`th on, its variables
    /// from `base`, for table `t`'s call `call`; `steps` holds what the
    /// literals before it read: atoms, negated or not, and the ranks of
    /// built-ins' solutions.
    fn body(
        &mut self,
        t: u32,
        rule: &Rule,
        base: u32,
        i: usize,
        call: &[T],
        steps: &mut Vec<Step>,
    ) -> Result<(), GroundError> {
        let Some(lit) = rule.body.get(i) else {
            return self.answer(t, call, rule.group, steps);
        };
        let next = |me: &mut Self, step: Option<Step>, steps: &mut Vec<Step>| {
            steps.extend(step);
            let r = me.body(t, rule, base, i + 1, call, steps);
            if step.is_some() {
                steps.pop();
            }
            r
        };
        match lit {
            Lit::Call(p, args, false) => {
                let args: Vec<T> = args.iter().map(|a| Self::inst(a, base)).collect();
                let p = *p;
                match self.preds[p as usize].kind {
                    Kind::Rules => {
                        let u = self.call(p, &args)?;
                        let arity = args.len();
                        let mut k = 0;
                        while let Some((key, a)) = self.tables[u as usize].answers.get(k).cloned() {
                            k += 1;
                            let mark = self.mark();
                            let answer = self.unkey(&key, arity);
                            if self.unify_all(&args, &answer) {
                                next(self, Some(Step::Pos(a)), steps)?;
                            }
                            self.undo(mark);
                        }
                    }
                    Kind::Facts(f) => {
                        for vals in self.fact_matches(p, f, &args) {
                            let mark = self.mark();
                            let tuple: Vec<T> = vals.iter().map(|v| T::G(*v)).collect();
                            if self.unify_all(&args, &tuple) {
                                let a = self.atom(p, vals)?;
                                next(self, Some(Step::Pos(a)), steps)?;
                            }
                            self.undo(mark);
                        }
                    }
                    Kind::Switch => {
                        let vals = self.values(p, &args, rule, "a probabilistic choice")?;
                        let a = self.atom(p, vals)?;
                        next(self, Some(Step::Pos(a)), steps)?;
                    }
                    Kind::Undefined => {}
                }
            }
            Lit::Call(p, args, true) => {
                let args: Vec<T> = args.iter().map(|a| Self::inst(a, base)).collect();
                let p = *p;
                let vals = self.values(p, &args, rule, "a negation")?;
                match self.preds[p as usize].kind {
                    Kind::Rules => {
                        let u = self.call(p, &args)?;
                        if self.tables[u as usize].state != State::Complete {
                            let goal = self.show(p, &args);
                            return Err(GroundError::Unsupported(format!(
                                "\\+{goal}: a negation in a cycle of calls, in {}",
                                self.norm.rules[rule.at]
                            )));
                        }
                        // The call is ground, and so is its one answer.
                        match self.tables[u as usize].answers.first() {
                            None => next(self, None, steps)?,
                            Some(&(_, a)) => next(self, Some(Step::Neg(a)), steps)?,
                        }
                    }
                    Kind::Facts(f) => {
                        if self.fact_matches(p, f, &args).is_empty() {
                            next(self, None, steps)?;
                        } else {
                            let a = self.atom(p, vals)?;
                            next(self, Some(Step::Neg(a)), steps)?;
                        }
                    }
                    Kind::Switch => {
                        let a = self.atom(p, vals)?;
                        next(self, Some(Step::Neg(a)), steps)?;
                    }
                    Kind::Undefined => next(self, None, steps)?,
                }
            }
            Lit::Unify(a, b) => {
                let (a, b) = (Self::inst(a, base), Self::inst(b, base));
                let mark = self.mark();
                if self.unify(&a, &b) {
                    next(self, None, steps)?;
                }
                self.undo(mark);
            }
            Lit::Cmp(op, a, b) => {
                let a = Self::inst(a, base);
                let b = Self::inst(b, base);
                if self.compare(*op, &a, &b, rule)? {
                    next(self, None, steps)?;
                }
            }
            Lit::Is(v, e) => {
                let e = Self::inst(e, base);
                let T::G(x) = self.full(&e)? else {
                    return Err(self.unbound(rule, i));
                };
                let Some(x) = self.g.num(x).map(|n| n.rounded().val()) else {
                    return Ok(());
                };
                let v = Self::inst(v, base);
                let mark = self.mark();
                if self.unify(&v, &T::G(x)) {
                    next(self, None, steps)?;
                }
                self.undo(mark);
            }
            Lit::Builtin(b, args, negated) => {
                let args: Vec<T> = args.iter().map(|a| Self::inst(a, base)).collect();
                let args = self.full_all(&args)?;
                let sols = self.builtin(*b, &args, *negated, rule, i)?;
                for (rank, sol) in sols.into_iter().enumerate() {
                    let mark = self.mark();
                    if self.unify_all(&args, &sol) {
                        let step = (!negated).then_some(Step::Rank(rank as u32));
                        next(self, step, steps)?;
                    }
                    self.undo(mark);
                }
            }
            Lit::Collect(c) => {
                for &(list, cond) in self.collect(c, base, rule)?.iter() {
                    let mark = self.mark();
                    if self.unify(&Self::inst(&c.result, base), &T::G(list)) {
                        let step = cond.map(|(positive, a)| match positive {
                            true => Step::Pos(a),
                            false => Step::Neg(a),
                        });
                        next(self, step, steps)?;
                    }
                    self.undo(mark);
                }
            }
        }
        Ok(())
    }

    /// The lists of the findall `c` of `rule`, its variables from `base`,
    /// each with the literal of the worlds where it is the list: its goal
    /// called with the shared variables' terms, which must be ground, and
    /// its table complete.
    fn collect(&mut self, c: &Collect, base: u32, rule: &Rule) -> Result<Lists, GroundError> {
        let shared: Vec<T> = c.shared.iter().map(|a| Self::inst(a, base)).collect();
        let vals = self.values(c.aux, &shared, rule, "a findall")?;
        if let Some(l) = self.collected.get(&(c.aux, vals.clone())) {
            return Ok(l.clone());
        }
        let mut answers: Vec<(Val, u32)> = Vec::new();
        if self.preds[c.aux as usize].kind == Kind::Rules {
            let arity = self.preds[c.aux as usize].arity;
            let mut args = shared;
            while args.len() < arity {
                let x = self.fresh_var();
                args.push(x);
            }
            let u = self.call(c.aux, &args)?;
            if self.tables[u as usize].state != State::Complete {
                return Err(GroundError::Unsupported(format!(
                    "{}: a findall in a cycle of calls, in {}",
                    c.text, self.norm.rules[rule.at]
                )));
            }
            let k = vals.len();
            for (key, a) in self.tables[u as usize].answers.clone() {
                let mark = self.mark();
                let answer = self.unkey(&key, arity);
                let template = self.full(&answer[k])?;
                let T::G(v) = template else {
                    let goal = self.show(c.aux, &answer);
                    self.undo(mark);
                    return Err(GroundError::Unsupported(format!(
                        "{}: a solution with variables, {goal}",
                        c.text
                    )));
                };
                self.undo(mark);
                answers.push((v, a));
            }
        }
        let l: Rc<[_]> = lists::collect(self, &answers, c.all, &c.text)?.into();
        if let Some(e) = self.g.error.take() {
            return Err(e);
        }
        if self.g.refused {
            return Err(GroundError::OutOfMemory);
        }
        self.grow(l.len())?;
        self.collected.insert((c.aux, vals), l.clone());
        Ok(l)
    }

    /// The error of `rule`'s `i`th literal, called with a variable it
    /// needs bound.
    fn unbound(&self, rule: &Rule, i: usize) -> GroundError {
        let c = &self.norm.rules[rule.at];
        GroundError::Unsafe(format!(
            "{c}: literal {} has variables not bound when it is called",
            c.body[i]
        ))
    }

    /// The values of the ground arguments `args` of a call of `p` in
    /// `rule`, which `what` (a probabilistic choice, a negation) needs.
    fn values(
        &mut self,
        p: u32,
        args: &[T],
        rule: &Rule,
        what: &str,
    ) -> Result<Box<[Val]>, GroundError> {
        let full = self.full_all(args)?;
        if let Some(vals) = full
            .iter()
            .map(|t| match t {
                T::G(v) => Some(*v),
                _ => None,
            })
            .collect::<Option<Box<[Val]>>>()
        {
            return Ok(vals);
        }
        let goal = self.show(p, &full);
        Err(GroundError::Unsafe(format!(
            "{}: {what} called with variables, {goal}",
            self.norm.rules[rule.at]
        )))
    }

    /// The tuples of the facts of `p`, the facts at place `f` of the
    /// program, that may match `args`: those of the value of its first
    /// ground argument, or all.
    fn fact_matches(&mut self, p: u32, f: usize, args: &[T]) -> Vec<Box<[Val]>> {
        let arity = args.len();
        if !self.facts.contains_key(&p) {
            let facts = &self.norm.facts[f].1;
            let vals: Vec<Val> = facts.args.iter().map(|t| self.g.val(t)).collect();
            self.facts.insert(p, vals.into());
        }
        let vals = self.facts[&p].clone();
        let tuples = vals
            .len()
            .checked_div(arity)
            .unwrap_or(self.norm.facts[f].1.probs.len());
        let tuple = |k: usize| -> Box<[Val]> { vals[k * arity..(k + 1) * arity].into() };
        let at = args.iter().position(|a| matches!(self.deref(a), T::G(_)));
        let Some(at) = at else {
            return (0..tuples).map(tuple).collect();
        };
        let T::G(v) = self.deref(&args[at]) else {
            unreachable!("ground")
        };
        let index = self
            .fact_ix
            .entry((p, at))
            .or_insert_with(|| {
                let mut ix: FxHashMap<Val, Vec<u32>> = FxHashMap::default();
                for k in 0..tuples {
                    ix.entry(vals[k * arity + at]).or_default().push(k as u32);
                }
                Rc::new(ix)
            })
            .clone();
        index
            .get(&v)
            .map(|ks| ks.iter().map(|&k| tuple(k as usize)).collect())
            .unwrap_or_default()
    }

    /// Whether `a op b` holds, both sides needed bound but for `=`'s
    /// negation and the identities, which read variables as they are.
    fn compare(&mut self, op: CmpOp, a: &T, b: &T, rule: &Rule) -> Result<bool, GroundError> {
        let (a, b) = (self.full(a)?, self.full(b)?);
        if matches!(op, CmpOp::Ne | CmpOp::Identical | CmpOp::NotIdentical) {
            // Terms that are identical, or that do not unify, compare
            // alike whatever a call binds; the others do not, and an answer
            // proved by them would hold for some calls of its atom only.
            let same = identical(&a, &b);
            if !same && self.unifiable(&[a], &[b]) {
                let c = &self.norm.rules[rule.at];
                return Err(GroundError::Unsupported(format!(
                    "{c}: {op} of terms with variables, which a call that binds them \
                     compares otherwise"
                )));
            }
            return Ok(same == (op == CmpOp::Identical));
        }
        let (T::G(x), T::G(y)) = (&a, &b) else {
            let c = &self.norm.rules[rule.at];
            return Err(GroundError::Unsafe(format!(
                "{c}: a comparison {op:?} called with variables"
            )));
        };
        let side = |me: &Self, v: Val| {
            if op.arithmetic() && matches!(v, Val::Fn(_)) {
                me.g.num(v).map(|n| n.val())
            } else {
                Some(v)
            }
        };
        Ok(match (side(self, *x), side(self, *y)) {
            (Some(x), Some(y)) => compare(&mut self.g, op, x, y),
            _ => false,
        })
    }

    /// The solutions of the built-in `b` over `args`, each its arguments'
    /// terms for the caller to unify with `args`; for a negated one, no
    /// solution or the arguments themselves.
    fn builtin(
        &mut self,
        b: Builtin,
        args: &[T],
        negated: bool,
        rule: &Rule,
        i: usize,
    ) -> Result<Vec<Vec<T>>, GroundError> {
        let vals: Vec<Option<Val>> = args
            .iter()
            .map(|a| match a {
                T::G(v) => Some(*v),
                _ => None,
            })
            .collect();
        let flags: Vec<bool> = vals.iter().map(Option::is_some).collect();
        let sols: Vec<Vec<T>> = if b == Builtin::Between && !flags[2] {
            let (Some(Val::Int(lo)), Some(Val::Int(hi))) = (vals[0], vals[1]) else {
                return Err(self.unbound(rule, i));
            };
            (lo..=hi)
                .map(|x| vec![T::G(Val::Int(lo)), T::G(Val::Int(hi)), T::G(Val::Int(x))])
                .collect()
        } else if b.runs(&flags) {
            // A partial term is free to `solutions`, and the unification
            // after it filters: memberchk/2's first match is the first
            // item that unifies.
            let probe = if b == Builtin::Memberchk && !flags[0] {
                Builtin::Member
            } else {
                b
            };
            let mut sols: Vec<Vec<T>> = solutions(&mut self.g, probe, &vals)
                .into_iter()
                .map(|s| s.into_iter().map(T::G).collect())
                .collect();
            if probe != b {
                let first = sols.iter().position(|s| self.unifiable(args, s));
                sols = first.map(|k| vec![sols.swap_remove(k)]).unwrap_or_default();
            }
            sols
        } else if b == Builtin::Length {
            self.length(args, rule, i)?
        } else {
            return Err(self.unbound(rule, i));
        };
        if !negated {
            return Ok(sols);
        }
        Ok(if sols.iter().any(|s| self.unifiable(args, s)) {
            Vec::new()
        } else {
            vec![args.to_vec()]
        })
    }

    /// length/2 of a list whose items or tail are variables: its length,
    /// where its tail is `[]`; where its tail is a variable, the list of
    /// that many more fresh variables to the length asked.
    fn length(&mut self, args: &[T], rule: &Rule, i: usize) -> Result<Vec<Vec<T>>, GroundError> {
        let (nil, cons) = self.g.list_syms();
        let mut items: Vec<T> = Vec::new();
        let mut tail = args[0].clone();
        loop {
            match self.deref(&tail) {
                T::F(c, ht) if c == cons && ht.len() == 2 => {
                    items.push(ht[0].clone());
                    tail = ht[1].clone();
                }
                T::G(v) => {
                    let (spine, tails) = self.g.spine(v, cons);
                    if tails.last() != Some(&Val::Sym(nil)) {
                        return Ok(Vec::new());
                    }
                    let n = (items.len() + spine.len()) as i64;
                    return Ok(vec![vec![args[0].clone(), T::G(Val::Int(n))]]);
                }
                T::X(_) => break,
                T::F(..) => return Ok(Vec::new()),
            }
        }
        let n = match self.deref(&args[1]) {
            T::G(Val::Int(n)) => usize::try_from(n).ok(),
            T::G(_) => None,
            _ => return Err(self.unbound(rule, i)),
        };
        let Some(n) = n.filter(|&n| n >= items.len()) else {
            return Ok(Vec::new());
        };
        // The items read, then fresh variables to the length asked.
        while items.len() < n {
            let x = self.fresh_var();
            items.push(x);
        }
        let list = items
            .into_iter()
            .rev()
            .fold(T::G(Val::Sym(nil)), |rest, x| {
                T::F(cons, Rc::new([x, rest]))
            });
        Ok(vec![vec![list, T::G(Val::Int(n as i64))]])
    }

    /// An answer of table `t` from the instance of a clause of `group`
    /// just proved: the call's terms as bound, read with the body's
    /// `steps`.
    fn answer(
        &mut self,
        t: u32,
        call: &[T],
        group: u32,
        steps: &[Step],
    ) -> Result<(), GroundError> {
        let p = self.tables[t as usize].pred;
        let args = self.full_all(call)?;
        let Some(key) = Self::key(&args) else {
            return Err(self.too_deep(p, "answer"));
        };
        let vals: Option<Box<[Val]>> = key
            .iter()
            .map(|k| match k {
                K::G(v) => Some(*v),
                _ => None,
            })
            .collect();
        let vals = match vals {
            Some(vals) => vals,
            None => self.named(&args)?,
        };
        let a = self.atom(p, vals)?;
        if !self.preds[p as usize].driver {
            self.emit(a, group, steps)?;
        }
        let table = &mut self.tables[t as usize];
        if table.known.contains(&key) {
            return Ok(());
        }
        if table.answers.try_reserve(1).is_err() {
            return Err(GroundError::OutOfMemory);
        }
        table.known.insert(key.clone());
        let n = key.len();
        table.answers.push((key, a));
        self.added += 1;
        self.grow(n)
    }

    /// The ground rule of `head` that reads the atoms of `steps`, kept
    /// once, and where a findall reads `head`, its derivation by a clause
    /// of `group`: another clause's, or another instance's that reads
    /// other atoms or solutions, is another proof.
    fn emit(&mut self, head: u32, group: u32, steps: &[Step]) -> Result<(), GroundError> {
        let p = self.atoms[head as usize].0;
        if self.preds[p as usize].counted {
            let derivations = self.derivations.entry(head).or_default();
            if !derivations.iter().any(|(g, s)| *g == group && s == steps) {
                derivations.push((group, steps.to_vec()));
                self.grow(1 + steps.len())?;
            }
        }
        let body: Box<[(bool, u32)]> = steps
            .iter()
            .filter_map(|s| match *s {
                Step::Pos(a) => Some((false, a)),
                Step::Neg(a) => Some((true, a)),
                Step::Rank(_) => None,
            })
            .collect();
        if self.ground_ix.contains(&(head, body.clone())) {
            return Ok(());
        }
        if self.ground.try_reserve(1).is_err() {
            return Err(GroundError::OutOfMemory);
        }
        self.grow(1 + body.len())?;
        self.ground_ix.insert((head, body.clone()));
        self.ground.push((head, body));
        Ok(())
    }

    /// Resolves the query or the observation of `p` over `args`. ProbLog
    /// prints an answer with variables by names of its own for them, which
    /// tddlog does not: such an answer is refused.
    fn asked(&mut self, p: u32, args: &[T]) -> Result<(), GroundError> {
        let t = self.call(p, args)?;
        let Some((key, _)) = self.tables[t as usize]
            .answers
            .iter()
            .find(|(k, _)| k.iter().any(|x| matches!(x, K::X(_))))
            .cloned()
        else {
            return Ok(());
        };
        let answer = self.unkey(&key, args.len());
        let goal = self.show(p, &answer);
        Err(GroundError::Unsupported(format!(
            "{goal}: a query answered by a term with variables"
        )))
    }

    /// Resolves the query or the observation `a`, a pattern of an atom.
    fn top(&mut self, a: &Atom) -> Result<(), GroundError> {
        let Some(&p) = self.pred_ix.get(&(a.name.clone(), a.args.len())) else {
            return Ok(());
        };
        if self.preds[p as usize].kind == Kind::Rules {
            let mut vars = Vec::new();
            let args: Vec<T> = a.args.iter().map(|t| self.term(t, &mut vars)).collect();
            self.asked(p, &args)?;
        }
        Ok(())
    }

    /// Resolves the query rule `d`: its driver's answers, and for each the
    /// instance of its pattern, a query or an observation of `queries` or
    /// `evidence`, resolved.
    fn driver(
        &mut self,
        d: &Driver,
        queries: &mut Vec<(Atom, bool)>,
        evidence: &mut Vec<(Atom, bool)>,
    ) -> Result<(), GroundError> {
        let p = self.pred(&d.atom.name, d.atom.args.len());
        let mut vars = Vec::new();
        let args: Vec<T> = d
            .atom
            .args
            .iter()
            .map(|t| self.term(t, &mut vars))
            .collect();
        let t = self.call(p, &args)?;
        let asked = if d.evidence { evidence } else { queries };
        let mut seen: FxHashSet<Atom> = asked.iter().map(|(a, _)| a.clone()).collect();
        for k in 0..self.tables[t as usize].answers.len() {
            let key = self.tables[t as usize].answers[k].0.clone();
            let mark = self.mark();
            let answer = self.unkey(&key, args.len());
            if self.unify_all(&args, &answer) {
                let mut named = vars.clone();
                let q = self.pred(&d.pattern.name, d.pattern.args.len());
                let pattern: Vec<T> = d
                    .pattern
                    .args
                    .iter()
                    .map(|t| self.term(t, &mut named))
                    .collect();
                if self.preds[q as usize].kind == Kind::Rules {
                    self.asked(q, &pattern)?;
                }
                let atom = self.show(q, &pattern);
                if seen.insert(atom.clone()) {
                    asked.push((atom, d.positive));
                }
            }
            self.undo(mark);
        }
        Ok(())
    }

    /// The ground program: the rules proved, the program's switches and
    /// facts, and its queries and evidence with the query rules' instances.
    fn program(
        mut self,
        norm: &NormProgram,
        queries: Vec<(Atom, bool)>,
        evidence: Vec<(Atom, bool)>,
    ) -> NormProgram {
        let mut shown: Vec<Option<Atom>> = vec![None; self.atoms.len()];
        let mut atom = |me: &mut Self, a: u32| -> Atom {
            if let Some(x) = &shown[a as usize] {
                return x.clone();
            }
            let (p, vals) = me.atoms[a as usize].clone();
            let args: Args = vals.iter().map(|&v| me.g.term_of(v)).collect();
            let x = Atom::new(me.preds[p as usize].name.clone(), args);
            shown[a as usize] = Some(x.clone());
            x
        };
        let ground = std::mem::take(&mut self.ground);
        let rules = ground
            .iter()
            .map(|(head, body)| {
                let head = atom(&mut self, *head);
                let body = body
                    .iter()
                    .map(|&(negated, a)| {
                        let a = atom(&mut self, a);
                        if negated {
                            Literal::Neg(a)
                        } else {
                            Literal::Pos(a)
                        }
                    })
                    .collect();
                Clause::rule(head, body, u32::MAX)
            })
            .collect();
        NormProgram {
            rules,
            switches: norm.switches.clone(),
            queries,
            evidence,
            facts: norm.facts.iter().map(|(_, f)| (0, f.clone())).collect(),
            ..Default::default()
        }
    }
}

/// The atoms of a findall's proofs: a switch is a leaf but where it is
/// certain or impossible, an atom of a counted predicate or a list is
/// derived by its derivations.
impl lists::Source for TopDown<'_> {
    fn node(&mut self, a: u32) -> lists::Node {
        if let Some(t) = self.truth(a) {
            return if t {
                lists::Node::True
            } else {
                lists::Node::False
            };
        }
        match self.preds[self.atoms[a as usize].0 as usize].kind {
            Kind::Switch | Kind::Facts(_) => lists::Node::Leaf,
            Kind::Rules | Kind::Undefined => {
                lists::Node::Derived(self.derivations.remove(&a).unwrap_or_default())
            }
        }
    }

    fn truth(&mut self, a: u32) -> Option<bool> {
        let pred = &self.preds[self.atoms[a as usize].0 as usize];
        match pred.kind {
            Kind::Switch => match self.norm.switches.get(&(pred.name.clone(), pred.arity)) {
                Some(Weights::Fixed(q)) if *q == Prob::TRUE => Some(true),
                Some(Weights::Fixed(q)) if *q == Prob::FALSE => Some(false),
                _ => None,
            },
            Kind::Facts(f) => {
                let certain = self.norm.facts[f].1.probs.iter().all(Option::is_none);
                certain.then_some(true)
            }
            Kind::Rules | Kind::Undefined => None,
        }
    }

    fn known(&mut self) -> &mut FxHashMap<u32, Rc<[lists::Proof]>> {
        &mut self.proofs
    }

    fn list(&mut self, items: &[Val]) -> Val {
        let (nil, _) = self.g.list_syms();
        self.g.make_list(items, Val::Sym(nil))
    }

    fn show(&mut self, a: u32) -> String {
        let (p, vals) = self.atoms[a as usize].clone();
        let args: Args = vals.iter().map(|&v| self.g.term_of(v)).collect();
        Atom::new(self.preds[p as usize].name.clone(), args).to_string()
    }

    fn fresh(&mut self) -> Result<u32, GroundError> {
        static STATES: LazyLock<Name> = LazyLock::new(|| Name::from(lists::LISTS));
        let (p, n) = match self.states {
            Some(s) => s,
            None => {
                let p = self.pred(&STATES, 1);
                self.preds[p as usize].counted = true;
                (p, 0)
            }
        };
        self.states = Some((p, n + 1));
        self.atom(p, Box::new([Val::Int(n)]))
    }

    fn rule(&mut self, head: u32, body: &[lists::Lit]) -> Result<(), GroundError> {
        let steps: Vec<Step> = body
            .iter()
            .map(|&(positive, a)| match positive {
                true => Step::Pos(a),
                false => Step::Neg(a),
            })
            .collect();
        self.emit(head, 0, &steps)
    }
}
