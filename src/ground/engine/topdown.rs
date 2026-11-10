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
//! here, as is one whose demand a negation bounds that the rewrite cannot
//! read ([`Rewritten::top_down`](super::super::magic::Rewritten::top_down)). SLD resolution runs in the order of the source, every call is
//! tabled by its variant, and every probabilistic choice is taken as
//! possible. Each clause instance proved under ground answers is a ground
//! rule, and the engine grounds those rules as it grounds any program
//! without variables ([`ground`]).
//!
//! - A table holds the answers of a call. A call that is a variant of one
//!   still being resolved reads the answers found so far. The first call
//!   of such a cycle then resolves its clauses again, and so do the calls
//!   of the cycle it reaches, until no table gains an answer (linear
//!   tabling). Its tables are then complete. A table found in a cycle
//!   stays in it until the cycle's first call completes: resolved again,
//!   it may read only tables made after it, which its own resolution
//!   alone would take for a cycle of its own.
//! - An answer is an atom of the ground program, an answer with variables
//!   too, its variables named, as ProbLog gives it a node of its own: it
//!   holds for every value of them. A query answered so is refused, since
//!   ProbLog prints its variables by names of its own.
//! - The calls of a predicate share the atoms of their answers, which is
//!   sound where an answer is proved alike whatever the call bound. An
//!   identity or a `\=` of terms that unify and are not identical breaks
//!   that: it compares otherwise once a call binds their variables, and
//!   ProbLog answers by what each call proved. So does a negation of a
//!   call with variables, which reads fewer answers once they are bound,
//!   and a type test (`var/1`, `nonvar/1`, `is_list/1`) of a variable.
//!   Such a comparison, negation or test is made as ProbLog makes it,
//!   and the program is refused where an atom is an answer of two calls
//!   one of which made one on variables of its call, or called with them
//!   a call that did, directly or not.
//! - `=../2` builds a term from a list of fixed length that may hold
//!   variables, or the list of a term with variables, as ProbLog's does.
//! - A negation reads the answers of its call, whose table is complete:
//!   it holds where none of them does, and fails where one is true in
//!   every world (a certain fact, or an answer proved from such atoms
//!   alone), and the branch ends there, as in ProbLog's grounding. Called
//!   with variables, as ProbLog calls it, it reads every answer.
//! - `is/2`, the comparisons and the list built-ins are the engine's
//!   ([`super::solutions`]), on the values of ground terms. Prolog's
//!   `length/2` also makes a list of fresh variables.
//! - A findall calls its goal with its shared variables bound to ground
//!   terms, and lists the solutions of the complete table as the engine
//!   does ([`lists`]): each clause instance of the predicates it reads is
//!   recorded as a derivation, its clause's group and what each body
//!   literal reads.
//!
//! The resolution keeps its own stack, on the heap: a frame for each
//! table being resolved, called from the clause of the one below it, and
//! for each literal of its clause the alternatives it has left (a
//! choice). A table is on the stack once at most, so that calls nest as
//! deep as the tables made allow, and a loop of a hundred thousand steps
//! resolved from its end takes no more of the thread's stack than one of
//! ten.
//!
//! The tables, the answers, the rules, the derivations and the literals
//! resolved count against the grounding's bound, so that a grounding
//! bounded by another's size also stops once its work passes it, and a
//! ground term deeper than [`MAX_DEPTH`](super::MAX_DEPTH), or a term with
//! variables nested deeper than [`MAX_NESTING`], gives the grounding up
//! with the goal that reached it.

use std::rc::Rc;
use std::sync::{Arc, LazyLock};

use rustc_hash::{FxHashMap, FxHashSet};

use super::lists::{self, Step};
use super::{GroundError, Grounder, Prob, Val, Weights, compare, ground_normalized, solutions};
use crate::ground::GroundProgram;
use crate::ground::normalize::{Driver, NormProgram};
use crate::syntax::{Args, Atom, Builtin, Clause, CmpOp, Compound, Gather, Literal, Name, Term};

/// The deepest nesting of a term with variables, which the functions that
/// walk such a term recurse into, a few frames of the thread's stack a
/// level. A ground term is bound by [`MAX_DEPTH`](super::MAX_DEPTH).
const MAX_NESTING: usize = 1_000;

/// A term being resolved: a ground term's value, a variable, or a compound
/// term with a variable in it.
#[derive(Clone, Debug)]
enum T {
    G(Val),
    X(u32),
    F(u32, Rc<[T]>),
}

/// The values of `ts`, where all are ground.
fn ground_vals(ts: &[T]) -> Option<Box<[Val]>> {
    let val = |t: &T| match t {
        T::G(v) => Some(*v),
        _ => None,
    };
    ts.iter().map(val).collect()
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
    gather: Gather,
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
    /// Whether its resolution compared terms with variables that a call
    /// that binds them compares otherwise ([`TopDown::compare`]).
    compared: bool,
}

/// A table being resolved, on the resolution's stack, and the clause of
/// it being resolved.
struct Frame {
    table: u32,
    /// The first table of a cycle it reached, and whether it read a table
    /// that was not complete.
    low: u32,
    looped: bool,
    /// The answers found, all tables', when this pass over its clauses
    /// began.
    before: u64,
    /// The clause being resolved, by its place among its predicate's; its
    /// rule, and its variables' first number.
    clause: usize,
    rule: u32,
    base: u32,
    /// The call's terms as the clause instantiated them
    /// ([`TopDown::of_call`]).
    call: Vec<T>,
    /// The bindings before the clause, restored after it.
    mark: (usize, usize),
    /// Where the clause's choices start on [`TopDown::choices`], and what
    /// its body reads on [`TopDown::steps`].
    choices: usize,
    steps: usize,
    /// The literal of the caller's clause that reads the table once it is
    /// resolved, and its call's terms; none for a query's call.
    reader: Option<(usize, Vec<T>)>,
}

/// A literal of the clause being resolved, and the alternatives it has
/// left.
struct Choice {
    lit: usize,
    /// The bindings and the steps before it, restored before each
    /// alternative and after the last.
    mark: (usize, usize),
    steps: usize,
    alts: Alts,
}

/// A literal's alternatives, from the `next`th on, each unified with the
/// literal's terms.
enum Alts {
    /// The answers of a table, for a call of `args`.
    Answers {
        table: u32,
        next: usize,
        args: Vec<T>,
    },
    /// The facts of `pred` that may match `args`.
    Tuples {
        pred: u32,
        tuples: Vec<Box<[Val]>>,
        next: usize,
        args: Vec<T>,
    },
    /// The solutions of a built-in over `args`, or a findall's lists, its
    /// result the one argument, each with what it reads.
    Solutions {
        sols: Vec<(Vec<T>, Option<Step>)>,
        next: usize,
        args: Vec<T>,
    },
    /// None: the literal held once, and the literals after it are being
    /// resolved.
    Spent,
}

/// What the resolution does next with the table on top of its stack.
enum Go {
    /// Resolves the `i`th literal of the clause at hand.
    Enter(usize),
    /// Tries the next alternative of the clause's latest choice.
    Retry,
    /// Resolves the next clause, or ends the pass over the clauses.
    Clause,
}

/// The table of a call, ready to read, or to resolve first.
enum Called {
    Ready(u32),
    Resolve(u32),
}

/// Pushes the variables of `t` to `out`, `t`'s bound variables replaced
/// ([`TopDown::full`]).
fn vars_of(t: &T, out: &mut Vec<u32>) {
    match t {
        T::G(_) => {}
        T::X(x) => out.push(*x),
        T::F(_, args) => args.iter().for_each(|a| vars_of(a, out)),
    }
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
/// and its query rules; [`GroundError::OverBound`] once its answers, its
/// rules and the literals it resolves outgrow `bound` terms.
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
    td.shared()?;
    let resolved = (td.tables.len(), td.literals);
    let out = td.program(norm, queries, evidence);
    let mut gp = ground_normalized(&out, bound)?;
    gp.stats.top_down = resolved;
    Ok(gp)
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
    /// The tables each table's resolution called, as `(caller, callee)`.
    reads: FxHashSet<(u32, u32)>,
    /// The first comparison of terms with variables that a call that binds
    /// them compares otherwise, as written, where one was made.
    compared: Option<String>,
    /// The tables that are not complete, in the order made.
    open: Vec<u32>,
    /// The resolution's stack: the tables being resolved, each called
    /// from the clause of the one below it, the choices of the literals
    /// of their clauses, and what those clauses' bodies read.
    frames: Vec<Frame>,
    choices: Vec<Choice>,
    steps: Vec<Step>,
    pass: u32,
    /// The answers found so far, all tables', and the literals resolved.
    added: u64,
    literals: u64,
    /// The facts' values, by predicate, and their indexes by a place.
    facts: FxHashMap<u32, Rc<[Val]>>,
    fact_ix: FxHashMap<(u32, usize), FactIndex>,
    /// The ground atoms, by predicate and values, and the ground rules.
    atoms: Vec<(u32, Box<[Val]>)>,
    atom_ix: FxHashMap<(u32, Box<[Val]>), u32>,
    /// By atom, whether it is an answer true in every world: a clause
    /// instance proved it from such atoms alone.
    sure: Vec<bool>,
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
    /// The answers, the rules' terms, the derivations' steps and the
    /// literals resolved, against `bound`.
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
            reads: FxHashSet::default(),
            compared: None,
            open: Vec::new(),
            frames: Vec::new(),
            choices: Vec::new(),
            steps: Vec::new(),
            pass: 0,
            added: 0,
            literals: 0,
            facts: FxHashMap::default(),
            fact_ix: FxHashMap::default(),
            atoms: Vec::new(),
            atom_ix: FxHashMap::default(),
            sure: Vec::new(),
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
                    gather: k.gather,
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

    /// Whether `t` is a variable, a list or a list whose tail is a
    /// variable: what a findall's result may be bound to.
    fn listlike(&mut self, t: &T) -> bool {
        match self.list_end(t) {
            T::X(_) => true,
            T::G(v) => self.g.items(v).is_some(),
            T::F(..) => false,
        }
    }

    /// `t` past the list cells with variables it starts with: a variable,
    /// a ground term, or a compound term that is no list cell.
    fn list_end(&mut self, t: &T) -> T {
        let (_, cons) = self.g.list_syms();
        let mut t = self.deref(t);
        loop {
            match t {
                T::F(f, args) if f == cons && args.len() == 2 => t = self.deref(&args[1]),
                t => return t,
            }
        }
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
        self.sure.push(false);
        self.atom_ix.insert((p, vals), a);
        Ok(a)
    }

    /// Whether atom `a` is true in every world: a certain fact, a switch
    /// of probability one, or an answer proved from such atoms.
    fn always(&mut self, a: u32) -> bool {
        self.sure[a as usize] || lists::Source::truth(self, a) == Some(true)
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
    /// being resolved. A query's or a query rule's call starts the
    /// resolution; a clause's call goes on the stack of the one at hand
    /// ([`Self::enter`]).
    fn resolved(&mut self, p: u32, args: &[T]) -> Result<u32, GroundError> {
        match self.call(p, args)? {
            Called::Ready(t) => Ok(t),
            Called::Resolve(t) => {
                self.start(t, None)?;
                self.run()?;
                Ok(t)
            }
        }
    }

    /// The table of the call of `p` with `args`: one to read, or one whose
    /// clauses are to be resolved, made now or resolved again in a new
    /// pass of its cycle.
    fn call(&mut self, p: u32, args: &[T]) -> Result<Called, GroundError> {
        let args = self.full_all(args)?;
        let Some(key) = Self::key(&args) else {
            return Err(self.too_deep(p, "call"));
        };
        // The caller's table reads this one where a call that binds more
        // of the caller's atom would call it otherwise.
        let caller = match self.frames.last().map(|f| f.table) {
            Some(c) if self.of_call(&args)? => Some(c),
            _ => None,
        };
        if let Some(&t) = self.table_ix.get(&(p, key.clone())) {
            if let Some(c) = caller {
                self.reads.insert((c, t));
            }
            let Table {
                state, pass, low, ..
            } = self.tables[t as usize];
            match state {
                State::Complete => {}
                State::Active => self.reads(t),
                // Resolved again once in each pass of its cycle: its frame
                // passes the cycle's first table on to the caller's as it
                // ends ([`Self::clause`]).
                State::Open if pass != self.pass => return Ok(Called::Resolve(t)),
                State::Open => self.reads(low),
            }
            return Ok(Called::Ready(t));
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
            compared: false,
        });
        if let Some(c) = caller {
            self.reads.insert((c, t));
        }
        self.table_ix.insert((p, key), t);
        self.open.push(t);
        Ok(Called::Resolve(t))
    }

    /// Whether the terms `ts`, their bound variables replaced, hold a
    /// variable of the call of the table being resolved, as the clause at
    /// hand has bound it: one that a call that binds more of the same atom
    /// would have bound.
    fn of_call(&mut self, ts: &[T]) -> Result<bool, GroundError> {
        let mut vars = Vec::new();
        ts.iter().for_each(|t| vars_of(t, &mut vars));
        let Some(f) = self.frames.last() else {
            return Ok(false);
        };
        if vars.is_empty() {
            return Ok(false);
        }
        let mut own = Vec::new();
        for t in f.call.clone() {
            vars_of(&self.full(&t)?, &mut own);
        }
        Ok(vars.iter().any(|x| own.contains(x)))
    }

    /// Refuses the program where a table that compared terms with variables
    /// of its call ([`Self::compare`]) or negated a call with them, or that
    /// called such a table with
    /// variables of its own call, directly or not, answers an atom that
    /// another table answers too. The calls of a
    /// predicate share the atoms of their answers, each atom's rules those
    /// of every call that proved it; ProbLog gives each call's answer a
    /// node of its own. The two agree where every call proves an answer
    /// alike, which such a comparison does not ensure: `fill(A) :- A ==
    /// unknown` fails for the call `fill(A)` and holds for `fill(unknown)`.
    fn shared(&mut self) -> Result<(), GroundError> {
        let Some(why) = self.compared.clone() else {
            return Ok(());
        };
        let mut callers: FxHashMap<u32, Vec<u32>> = FxHashMap::default();
        for &(caller, callee) in &self.reads {
            callers.entry(callee).or_default().push(caller);
        }
        let mut marked: Vec<bool> = self.tables.iter().map(|t| t.compared).collect();
        let mut work: Vec<u32> = (0..marked.len() as u32)
            .filter(|&t| marked[t as usize])
            .collect();
        while let Some(t) = work.pop() {
            for &c in callers.get(&t).map_or(&[][..], Vec::as_slice) {
                if !marked[c as usize] {
                    marked[c as usize] = true;
                    work.push(c);
                }
            }
        }
        // By atom, the first table that answers it.
        let mut first: FxHashMap<u32, u32> = FxHashMap::default();
        for (t, table) in self.tables.iter().enumerate() {
            for &(_, a) in &table.answers {
                let u = *first.entry(a).or_insert(t as u32);
                if u != t as u32 && (marked[t] || marked[u as usize]) {
                    let (p, vals) = self.atoms[a as usize].clone();
                    let args: Args = vals.iter().map(|&v| self.g.term_of(v)).collect();
                    let atom = Atom::new(self.preds[p as usize].name.clone(), args);
                    return Err(GroundError::Unsupported(format!(
                        "{why}, which a call that binds them makes otherwise: {atom} is \
                         an answer of two calls, which ProbLog proves apart"
                    )));
                }
            }
        }
        Ok(())
    }

    /// The table being resolved read the table that is not complete of a
    /// cycle whose first table is `low`.
    fn reads(&mut self, low: u32) {
        if let Some(f) = self.frames.last_mut() {
            f.low = f.low.min(low);
            f.looped = true;
        }
    }

    /// Puts table `t` on the stack, to resolve its clauses, whose answers
    /// literal `reader` of the caller's clause reads once it is resolved.
    fn start(&mut self, t: u32, reader: Option<(usize, Vec<T>)>) -> Result<(), GroundError> {
        if self.frames.try_reserve(1).is_err() {
            return Err(GroundError::OutOfMemory);
        }
        let table = &mut self.tables[t as usize];
        table.state = State::Active;
        table.pass = self.pass;
        // A table already found in a cycle starts from the cycle's first
        // table: whatever it reads this time, it is not taken for the first
        // table of a cycle of its own and completed before that one is.
        let low = table.low;
        let mark = self.mark();
        self.frames.push(Frame {
            table: t,
            low,
            looped: false,
            before: self.added,
            clause: 0,
            rule: 0,
            base: 0,
            call: Vec::new(),
            mark,
            choices: self.choices.len(),
            steps: self.steps.len(),
            reader,
        });
        Ok(())
    }

    /// Resolves the tables on the stack until none is left, depth first:
    /// each clause of the table on top in turn, the alternatives of each
    /// literal of its body in turn, and a call of a table to resolve put
    /// on top.
    fn run(&mut self) -> Result<(), GroundError> {
        let rules = self.rules.clone();
        let mut go = Go::Clause;
        while !self.frames.is_empty() {
            go = match go {
                Go::Enter(i) => self.enter(&rules, i)?,
                Go::Retry => self.retry()?,
                Go::Clause => self.clause(&rules)?,
            };
        }
        Ok(())
    }

    /// Resolves the next clause of the table on top, its head unified with
    /// the call. Once none is left, ends the pass over them: the table is
    /// resolved again while it is the first of a cycle that gained
    /// answers, then completed with the tables of its cycle, or left open
    /// for its cycle's first table, and its caller reads it.
    fn clause(&mut self, rules: &[Rule]) -> Result<Go, GroundError> {
        let f = self.frames.last().expect("a frame");
        let t = f.table;
        let p = self.tables[t as usize].pred;
        if let Some(&r) = self.preds[p as usize].rules.get(f.clause) {
            let rule = &rules[r as usize];
            let mark = self.mark();
            let key = self.tables[t as usize].call.clone();
            let call = self.unkey(&key, rule.args.len());
            let base = self.binds.len() as u32;
            self.binds
                .resize(self.binds.len() + rule.vars as usize, None);
            let head: Vec<T> = rule.args.iter().map(|a| Self::inst(a, base)).collect();
            let unified = self.unify_all(&head, &call);
            let f = self.frames.last_mut().expect("a frame");
            (f.rule, f.base, f.mark, f.call) = (r, base, mark, call);
            return Ok(if unified { Go::Enter(0) } else { Go::Retry });
        }
        let f = self.frames.last_mut().expect("a frame");
        if f.low < t {
            self.tables[t as usize].state = State::Open;
            self.tables[t as usize].low = f.low;
        } else if !f.looped || self.added == f.before {
            while let Some(&u) = self.open.last()
                && u >= t
            {
                self.open.pop();
                self.tables[u as usize].state = State::Complete;
            }
        } else {
            f.looped = false;
            f.before = self.added;
            f.clause = 0;
            self.pass += 1;
            self.tables[t as usize].pass = self.pass;
            return Ok(Go::Clause);
        }
        let f = self.frames.pop().expect("a frame");
        if f.low < f.table
            && let Some(parent) = self.frames.last_mut()
        {
            parent.low = parent.low.min(f.low);
            parent.looped = true;
        }
        match f.reader {
            Some((i, args)) => self.read(rules, i, t, args),
            None => Ok(Go::Clause),
        }
    }

    /// Backtracks to the latest literal of the clause at hand with an
    /// alternative left, its bindings and steps restored, and tries it;
    /// once none is left, the clause is done.
    fn retry(&mut self) -> Result<Go, GroundError> {
        let f = self.frames.last().expect("a frame");
        let (from, before) = (f.choices, f.mark);
        while self.choices.len() > from {
            let c = self.choices.last().expect("a choice");
            let (lit, mark, steps) = (c.lit, c.mark, c.steps);
            self.undo(mark);
            self.steps.truncate(steps);
            if let Some(step) = self.alternative(mark)? {
                self.steps.extend(step);
                return Ok(Go::Enter(lit + 1));
            }
            self.choices.pop();
        }
        self.undo(before);
        self.frames.last_mut().expect("a frame").clause += 1;
        Ok(Go::Clause)
    }

    /// The next alternative of the latest choice that unifies, and what it
    /// reads; `None` once none is left. Each that does not unify has its
    /// bindings undone to `mark`.
    fn alternative(&mut self, mark: (usize, usize)) -> Result<Option<Option<Step>>, GroundError> {
        let c = self.choices.len() - 1;
        let mut alts = std::mem::replace(&mut self.choices[c].alts, Alts::Spent);
        let found = loop {
            match &mut alts {
                // Read as they come: a table of the caller's cycle may gain
                // answers while the clause is resolved further.
                Alts::Answers { table, next, args } => {
                    let Some((key, a)) = self.tables[*table as usize].answers.get(*next).cloned()
                    else {
                        break None;
                    };
                    *next += 1;
                    let answer = self.unkey(&key, args.len());
                    if self.unify_all(args, &answer) {
                        break Some(Some(Step::Pos(a)));
                    }
                }
                Alts::Tuples {
                    pred,
                    tuples,
                    next,
                    args,
                } => {
                    let Some(vals) = tuples.get_mut(*next).map(std::mem::take) else {
                        break None;
                    };
                    *next += 1;
                    let tuple: Vec<T> = vals.iter().map(|v| T::G(*v)).collect();
                    if self.unify_all(args, &tuple) {
                        let a = self.atom(*pred, vals)?;
                        break Some(Some(Step::Pos(a)));
                    }
                }
                Alts::Solutions { sols, next, args } => {
                    let Some((sol, step)) = sols.get(*next) else {
                        break None;
                    };
                    *next += 1;
                    if self.unify_all(args, sol) {
                        break Some(*step);
                    }
                }
                Alts::Spent => break None,
            }
            self.undo(mark);
        };
        self.choices[c].alts = alts;
        Ok(found)
    }

    /// Pushes the choice of the `i`th literal of the clause at hand among
    /// `alts`, the bindings and steps as they stand restored before each
    /// alternative and after the last.
    fn push(&mut self, i: usize, alts: Alts) {
        let mark = self.mark();
        self.choices.push(Choice {
            lit: i,
            mark,
            steps: self.steps.len(),
            alts,
        });
    }

    /// The `i`th literal tries `alts` in turn.
    fn choose(&mut self, i: usize, alts: Alts) -> Result<Go, GroundError> {
        self.push(i, alts);
        Ok(Go::Retry)
    }

    /// The `i`th literal holds once, reading `steps`: the literals after
    /// it are resolved.
    fn take(&mut self, i: usize, steps: impl IntoIterator<Item = Step>) -> Result<Go, GroundError> {
        self.push(i, Alts::Spent);
        self.steps.extend(steps);
        Ok(Go::Enter(i + 1))
    }

    /// The `i`th literal, a negation, reads the atoms of its call's
    /// answers: it holds where none of them does, and fails where one is
    /// true in every world, as in ProbLog's grounding: `\+ member(X,
    /// Seen)` ends a walk that would otherwise grow its list.
    fn negation(&mut self, i: usize, atoms: Vec<u32>) -> Result<Go, GroundError> {
        if atoms.iter().any(|&a| self.always(a)) {
            return Ok(Go::Retry);
        }
        self.take(i, atoms.into_iter().map(Step::Neg))
    }

    /// Resolves the `i`th literal of the clause at hand: pushes its choice
    /// of alternatives, or the table its call reads where that table is
    /// still to be resolved. Past the last literal, the clause's instance
    /// proves an answer.
    fn enter(&mut self, rules: &[Rule], i: usize) -> Result<Go, GroundError> {
        // Each literal resolved counts against the bound too: the work of a
        // call resolved again for each binding of an argument its rules
        // read only after enumerating (`step(J) :- between(1, N, T1), J is
        // T1 - 1`, called for each `J`) grows past what it keeps.
        self.grow(1)?;
        self.literals += 1;
        let f = self.frames.last().expect("a frame");
        let (t, rule, base) = (f.table, &rules[f.rule as usize], f.base);
        let Some(lit) = rule.body.get(i) else {
            let from = f.steps;
            let steps = std::mem::take(&mut self.steps);
            let proved = self.answer(t, rule.group, &steps[from..]);
            self.steps = steps;
            proved?;
            return Ok(Go::Retry);
        };
        match lit {
            Lit::Call(p, args, negated) => {
                let args: Vec<T> = args.iter().map(|a| Self::inst(a, base)).collect();
                let p = *p;
                let kind = self.preds[p as usize].kind;
                if !negated {
                    return match kind {
                        Kind::Rules => self.called(rules, i, p, args),
                        Kind::Facts(f) => {
                            let tuples = self.fact_matches(p, f, &args);
                            self.choose(
                                i,
                                Alts::Tuples {
                                    pred: p,
                                    tuples,
                                    next: 0,
                                    args,
                                },
                            )
                        }
                        Kind::Switch => {
                            let vals = self.values(p, &args, rule, "a probabilistic choice")?;
                            let a = self.atom(p, vals)?;
                            self.take(i, Some(Step::Pos(a)))
                        }
                        Kind::Undefined => Ok(Go::Retry),
                    };
                }
                if kind == Kind::Switch {
                    let vals = self.values(p, &args, rule, "a negation")?;
                    let a = self.atom(p, vals)?;
                    return self.negation(i, vec![a]);
                }
                // A call with variables, as ProbLog makes it, may have
                // several answers. Where a variable of the table's call is
                // free there, a call that binds it reads fewer answers.
                let full = self.full_all(&args)?;
                if full.iter().any(|a| !matches!(a, T::G(_))) && self.of_call(&full)? {
                    self.call_dependent(rule, "a negation of a call with variables");
                }
                match kind {
                    Kind::Rules => self.called(rules, i, p, args),
                    Kind::Facts(f) => {
                        let mut atoms = Vec::new();
                        for vals in self.fact_matches(p, f, &args) {
                            let tuple: Vec<T> = vals.iter().map(|v| T::G(*v)).collect();
                            if self.unifiable(&args, &tuple) {
                                atoms.push(self.atom(p, vals)?);
                            }
                        }
                        self.negation(i, atoms)
                    }
                    Kind::Switch | Kind::Undefined => self.negation(i, Vec::new()),
                }
            }
            Lit::Unify(a, b) => {
                let (a, b) = (Self::inst(a, base), Self::inst(b, base));
                self.push(i, Alts::Spent);
                Ok(match self.unify(&a, &b) {
                    true => Go::Enter(i + 1),
                    false => Go::Retry,
                })
            }
            Lit::Cmp(op, a, b) => {
                let a = Self::inst(a, base);
                let b = Self::inst(b, base);
                match self.compare(*op, &a, &b, rule)? {
                    true => self.take(i, None),
                    false => Ok(Go::Retry),
                }
            }
            Lit::Is(v, e) => {
                let e = Self::inst(e, base);
                let T::G(x) = self.full(&e)? else {
                    return Err(self.unbound(rule, i));
                };
                let Some(x) = self.g.num(x).map(|n| n.rounded().val()) else {
                    return Ok(Go::Retry);
                };
                let v = Self::inst(v, base);
                self.push(i, Alts::Spent);
                Ok(match self.unify(&v, &T::G(x)) {
                    true => Go::Enter(i + 1),
                    false => Go::Retry,
                })
            }
            Lit::Builtin(b, args, negated) => {
                let args: Vec<T> = args.iter().map(|a| Self::inst(a, base)).collect();
                let args = self.full_all(&args)?;
                let sols = self.builtin(*b, &args, *negated, rule, i)?;
                let rank = |r: usize| (!negated).then_some(Step::Rank(r as u32));
                let sols = sols.into_iter().enumerate();
                let sols = sols.map(|(r, sol)| (sol, rank(r))).collect();
                self.choose(
                    i,
                    Alts::Solutions {
                        sols,
                        next: 0,
                        args,
                    },
                )
            }
            Lit::Collect(c) => self.collect(rules, i, c, base, rule),
        }
    }

    /// The `i`th literal of the clause at hand calls `p`, a predicate of
    /// rules, with `args`: it reads the call's table, or puts the table on
    /// the stack and reads it once it is resolved.
    fn called(
        &mut self,
        rules: &[Rule],
        i: usize,
        p: u32,
        args: Vec<T>,
    ) -> Result<Go, GroundError> {
        match self.call(p, &args)? {
            Called::Ready(u) => self.read(rules, i, u, args),
            Called::Resolve(u) => {
                self.start(u, Some((i, args)))?;
                Ok(Go::Clause)
            }
        }
    }

    /// The `i`th literal of the clause at hand reads table `u` of its call
    /// with `args`, resolved: a call its answers, a negation the atoms of
    /// its answers, which its table holds complete, and a findall the
    /// lists of its goal's solutions, its shared terms the first of `args`.
    fn read(
        &mut self,
        rules: &[Rule],
        i: usize,
        u: u32,
        mut args: Vec<T>,
    ) -> Result<Go, GroundError> {
        let f = self.frames.last().expect("a frame");
        let (rule, base) = (&rules[f.rule as usize], f.base);
        match &rule.body[i] {
            Lit::Call(_, _, false) => self.choose(
                i,
                Alts::Answers {
                    table: u,
                    next: 0,
                    args,
                },
            ),
            Lit::Call(p, _, true) => {
                if self.tables[u as usize].state != State::Complete {
                    let goal = self.show(*p, &args);
                    return Err(GroundError::Unsupported(format!(
                        "\\+{goal}: a negation in a cycle of calls, in {}",
                        self.norm.rules[rule.at]
                    )));
                }
                let atoms = self.tables[u as usize]
                    .answers
                    .iter()
                    .map(|&(_, a)| a)
                    .collect();
                self.negation(i, atoms)
            }
            Lit::Collect(c) => {
                args.truncate(c.shared.len());
                let lists = self.listed(c, rule, Some(u), &args)?;
                self.lists(i, c, base, &lists)
            }
            _ => unreachable!("only a call, a negation or a findall reads a table"),
        }
    }

    /// The `i`th literal, the findall `c`, its variables from `base`, tries
    /// `lists` in turn as its result.
    fn lists(
        &mut self,
        i: usize,
        c: &Collect,
        base: u32,
        lists: &Lists,
    ) -> Result<Go, GroundError> {
        let step = |(positive, a)| if positive { Step::Pos(a) } else { Step::Neg(a) };
        let sols = lists
            .iter()
            .map(|&(l, cond)| (vec![T::G(l)], cond.map(step)));
        let args = vec![Self::inst(&c.result, base)];
        let sols = sols.collect();
        self.choose(
            i,
            Alts::Solutions {
                sols,
                next: 0,
                args,
            },
        )
    }

    /// Resolves the `i`th literal of the clause at hand, the findall `c`,
    /// its variables from `base`: the lists of its goal called with the
    /// shared variables' terms, its table complete. A shared variable
    /// free at the call stays free, as ProbLog's findall leaves it, and
    /// the lists then depend on the call where the variable is the call's.
    fn collect(
        &mut self,
        rules: &[Rule],
        i: usize,
        c: &Collect,
        base: u32,
        rule: &Rule,
    ) -> Result<Go, GroundError> {
        let result = Self::inst(&c.result, base);
        if !self.listlike(&result) {
            let result = self.show_term(&result);
            return Err(lists::not_a_list(&c.text, &result));
        }
        let shared: Vec<T> = c.shared.iter().map(|a| Self::inst(a, base)).collect();
        let mut args = self.full_all(&shared)?;
        let ground = ground_vals(&args);
        if ground.is_none() && self.of_call(&args)? {
            self.call_dependent(rule, "a findall over a goal with variables");
        }
        let cached = ground.and_then(|vals| self.collected.get(&(c.aux, vals)).cloned());
        if let Some(lists) = cached {
            return self.lists(i, c, base, &lists);
        }
        if self.preds[c.aux as usize].kind != Kind::Rules {
            let lists = self.listed(c, rule, None, &args)?;
            return self.lists(i, c, base, &lists);
        }
        let arity = self.preds[c.aux as usize].arity;
        while args.len() < arity {
            let x = self.fresh_var();
            args.push(x);
        }
        self.called(rules, i, c.aux, args)
    }

    /// The lists of the findall `c` of `rule` over its shared terms
    /// `shared`, each with the literal of the worlds where it is the list,
    /// from the answers of table `u` of its goal where its goal's
    /// predicate has rules; kept for later calls where `shared` is ground.
    fn listed(
        &mut self,
        c: &Collect,
        rule: &Rule,
        u: Option<u32>,
        shared: &[T],
    ) -> Result<Lists, GroundError> {
        let mut answers: Vec<(Val, u32)> = Vec::new();
        if let Some(u) = u {
            if self.tables[u as usize].state != State::Complete {
                return Err(GroundError::Unsupported(format!(
                    "{}: a findall in a cycle of calls, in {}",
                    c.text, self.norm.rules[rule.at]
                )));
            }
            let arity = self.preds[c.aux as usize].arity;
            let k = shared.len();
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
        let l: Rc<[_]> = lists::collect(self, &answers, c.gather, &c.text)?.into();
        if let Some(e) = self.g.error.take() {
            return Err(e);
        }
        if self.g.refused {
            return Err(GroundError::OutOfMemory);
        }
        self.grow(l.len())?;
        if let Some(vals) = ground_vals(shared) {
            self.collected.insert((c.aux, vals), l.clone());
        }
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
        if let Some(vals) = ground_vals(&full) {
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
            // alike whatever a call binds. The others compare as ProbLog
            // compares them, as they are: they unify and are not
            // identical. Where they hold a variable of the table's call, a
            // call that binds more of the same atom may compare them
            // otherwise, and the table is marked: an answer it proves may
            // be proved otherwise by another call of its atom
            // ([`Self::shared`]).
            let same = identical(&a, &b);
            if same || !self.unifiable(std::slice::from_ref(&a), std::slice::from_ref(&b)) {
                return Ok(same == (op == CmpOp::Identical));
            }
            if self.of_call(&[a, b])? {
                self.call_dependent(rule, &format!("{op} of terms with variables"));
            }
            return Ok(op == CmpOp::NotIdentical);
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

    /// Marks the table being resolved, whose clause `rule` made `what` on
    /// terms that hold a variable of its call: a call that binds more of
    /// the same atom may make it otherwise, so that an answer the table
    /// proves may be proved otherwise by another call of its atom
    /// ([`Self::shared`]).
    fn call_dependent(&mut self, rule: &Rule, what: &str) {
        let Some(f) = self.frames.last() else {
            unreachable!("a clause is resolved for a table")
        };
        self.tables[f.table as usize].compared = true;
        if self.compared.is_none() {
            let c = &self.norm.rules[rule.at];
            self.compared = Some(format!("{c}: {what}"));
        }
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
            if let Some(e) = self.g.error.take() {
                return Err(e);
            }
            if probe != b {
                let first = sols.iter().position(|s| self.unifiable(args, s));
                sols = first.map(|k| vec![sols.swap_remove(k)]).unwrap_or_default();
            }
            sols
        } else if b == Builtin::Length {
            self.length(args, rule, i)?
        } else if b == Builtin::Univ {
            self.univ(args)?
        } else if matches!(b, Builtin::Var | Builtin::Nonvar | Builtin::IsList) {
            self.type_test(b, &args[0], rule)?
        } else if b == Builtin::AppendLists {
            self.cuts(args, rule, i)?
        } else if b == Builtin::Callable {
            // A goal with variables in it: its name and arity are bound.
            let (name, k) = match self.deref(&args[0]) {
                T::F(name, xs) => (name, xs.len()),
                T::X(_) => return Err(self.unbound(rule, i)),
                T::G(_) => unreachable!("a ground goal is the engine's"),
            };
            let test: Vec<Val> = args[1..]
                .iter()
                .map(|a| match self.deref(a) {
                    T::G(v) => v,
                    _ => unreachable!("the library writes constants"),
                })
                .collect();
            if super::callable(&mut self.g, name, k, &test)? {
                vec![args.to_vec()]
            } else {
                Vec::new()
            }
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

    /// `var/1`, `nonvar/1` or `is_list/1` of a term with variables, as
    /// ProbLog tests it, on the term as it stands: `is_list/1` holds for a
    /// list whose tail is a variable, but not for a variable. Where the
    /// variable the test reads is one of the table's call, a call that
    /// binds more of the same atom may test it otherwise
    /// ([`Self::call_dependent`]).
    fn type_test(&mut self, b: Builtin, t: &T, rule: &Rule) -> Result<Vec<Vec<T>>, GroundError> {
        let t = self.deref(t);
        let read = match b {
            Builtin::IsList => self.list_end(&t),
            _ => t.clone(),
        };
        let free = matches!(t, T::X(_));
        let holds = match b {
            Builtin::Var => free,
            Builtin::Nonvar => !free,
            _ => !free && self.listlike(&t),
        };
        if matches!(read, T::X(_)) && self.of_call(std::slice::from_ref(&read))? {
            self.call_dependent(rule, &format!("{b} of a term with variables"));
        }
        Ok(if holds { vec![vec![t]] } else { Vec::new() })
    }

    /// append/2 of a list of a fixed number of terms with variables in
    /// them, joined into a ground list: each way to cut that list into as
    /// many pieces, the last one its tail, in the order ProbLog's library
    /// finds them, which cuts the last piece off first, from the shortest
    /// list before it. The caller unifies the pieces with the terms.
    fn cuts(&mut self, args: &[T], rule: &Rule, i: usize) -> Result<Vec<Vec<T>>, GroundError> {
        let (nil, cons) = self.g.list_syms();
        let T::G(joined) = self.deref(&args[1]) else {
            return Err(self.unbound(rule, i));
        };
        let mut k = 0;
        let mut t = self.deref(&args[0]);
        loop {
            match t {
                T::F(c, ht) if c == cons && ht.len() == 2 => {
                    k += 1;
                    t = self.deref(&ht[1]);
                }
                T::G(v) => match self.g.items(v) {
                    Some(xs) => {
                        k += xs.len();
                        break;
                    }
                    None => return Ok(Vec::new()),
                },
                T::X(_) => return Err(self.unbound(rule, i)),
                T::F(..) => return Ok(Vec::new()),
            }
        }
        let (items, tails) = self.g.spine(joined, cons);
        if k == 0 {
            let none = T::G(Val::Sym(nil));
            return Ok(if joined == Val::Sym(nil) {
                vec![vec![none, T::G(joined)]]
            } else {
                Vec::new()
            });
        }
        // The places of the cuts, the last one outermost.
        fn places(pieces: usize, end: usize) -> Vec<Vec<usize>> {
            if pieces == 1 {
                return vec![Vec::new()];
            }
            (0..=end)
                .flat_map(|last| {
                    places(pieces - 1, last).into_iter().map(move |mut c| {
                        c.push(last);
                        c
                    })
                })
                .collect()
        }
        let mut sols = Vec::new();
        for cut in places(k, items.len()) {
            let mut pieces = Vec::with_capacity(k);
            let mut from = 0;
            for &to in &cut {
                pieces.push(self.g.make_list(&items[from..to], Val::Sym(nil)));
                from = to;
            }
            pieces.push(tails[from]);
            let lists = self.g.make_list(&pieces, Val::Sym(nil));
            sols.push(vec![T::G(lists), T::G(joined)]);
        }
        Ok(sols)
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

    /// `T =.. L` where a side holds variables, as ProbLog's `=..` reads it:
    /// a compound `T` gives the list of its name and arguments, and a free
    /// `T` is the term of `L`, a list of fixed length whose first item is
    /// an atom where it has more than one; that list may hold variables.
    fn univ(&mut self, args: &[T]) -> Result<Vec<Vec<T>>, GroundError> {
        let (nil, cons) = self.g.list_syms();
        let list = |items: Vec<T>| {
            items
                .into_iter()
                .rev()
                .fold(T::G(Val::Sym(nil)), |rest, x| {
                    T::F(cons, Rc::new([x, rest]))
                })
        };
        match self.deref(&args[0]) {
            T::F(name, xs) => {
                let items = std::iter::once(T::G(Val::Sym(name)))
                    .chain(xs.iter().cloned())
                    .collect();
                return Ok(vec![vec![args[0].clone(), list(items)]]);
            }
            T::G(_) => unreachable!("a ground term is the engine's"),
            T::X(_) => {}
        }
        let error = |me: &mut Self, why: &str| {
            let l = me.show_term(&args[1]);
            GroundError::Unsupported(format!("_ =.. {l}: {why} (ProbLog's call mode error)"))
        };
        let mut items: Vec<T> = Vec::new();
        let mut tail = args[1].clone();
        loop {
            match self.deref(&tail) {
                T::F(c, ht) if c == cons && ht.len() == 2 => {
                    items.push(ht[0].clone());
                    tail = ht[1].clone();
                }
                T::G(v) => {
                    let (spine, tails) = self.g.spine(v, cons);
                    if tails.last() != Some(&Val::Sym(nil)) {
                        return Err(error(self, "no list"));
                    }
                    items.extend(spine.into_iter().map(T::G));
                    break;
                }
                T::X(_) => return Err(error(self, "a list of no fixed length")),
                T::F(..) => return Err(error(self, "no list")),
            }
        }
        let term = match items.as_slice() {
            [] => return Err(error(self, "an empty list")),
            [t] => t.clone(),
            [first, rest @ ..] => match self.deref(first) {
                T::G(Val::Sym(name)) => T::F(name, rest.into()),
                _ => return Err(error(self, "its first item is no atom")),
            },
        };
        Ok(vec![vec![term, args[1].clone()]])
    }

    /// An answer of table `t` from the instance of a clause of `group`
    /// just proved: the call's terms as bound, read with the body's
    /// `steps`.
    fn answer(&mut self, t: u32, group: u32, steps: &[Step]) -> Result<(), GroundError> {
        let p = self.tables[t as usize].pred;
        let call = self.frames.last().expect("a frame").call.clone();
        let args = self.full_all(&call)?;
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
        // An answer proved from atoms true in every world is one too; it
        // counts as new, so that a cycle is resolved again for the
        // negations that read it.
        if !self.sure[a as usize]
            && steps.iter().all(|s| match *s {
                Step::Pos(b) => self.always(b),
                Step::Neg(_) => false,
                Step::Rank(_) => true,
            })
        {
            self.sure[a as usize] = true;
            self.added += 1;
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
        let t = self.resolved(p, args)?;
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
        let t = self.resolved(p, &args)?;
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

    fn display_atom(&mut self, a: u32) -> Atom {
        let (p, vals) = self.atoms[a as usize].clone();
        let args: Args = vals.iter().map(|&v| self.g.term_of(v)).collect();
        Atom::new(self.preds[p as usize].name.clone(), args)
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
            let x = me.display_atom(a);
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
        self.display_atom(a).to_string()
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
