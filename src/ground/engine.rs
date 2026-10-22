//! Stratified semi-naive bottom-up grounding.
//!
//! Every switch atom is assumed true, so the fixpoint computes the atoms that
//! are true in at least one possible world. Along the way each successful rule
//! instantiation is recorded as a ground rule. A negative literal over a lower
//! stratum is kept when its atom is possible and dropped (as true) otherwise.

use std::collections::BTreeMap;
use std::fmt;

use rustc_hash::{FxHashMap, FxHashSet};

use super::normalize::NormProgram;
use super::schedule::schedule_body;
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, Prob};
use crate::syntax::{Args, ArithOp, Atom, Clause, CmpOp, Expr, Literal, Name, Term};

#[derive(Debug, Clone, PartialEq)]
pub enum GroundError {
    Unsupported(String),
    /// A rule is not range restricted or a negative/comparison literal has an
    /// unbound variable.
    Unsafe(String),
    /// The grounding outgrew the size it was allowed (`ground_normalized`),
    /// which only a grounding weighed against another is given.
    OverBound,
}

impl fmt::Display for GroundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroundError::Unsupported(s) => write!(f, "unsupported: {s}"),
            GroundError::Unsafe(s) => write!(f, "unsafe rule: {s}"),
            GroundError::OverBound => f.write_str("the grounding outgrew its bound"),
        }
    }
}

impl std::error::Error for GroundError {}

/// An interned constant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Val {
    Sym(u32),
    Int(i64),
}

#[derive(Default)]
struct Interner {
    names: Vec<Name>,
    ids: FxHashMap<Name, u32>,
}

impl Interner {
    fn intern(&mut self, s: &Name) -> u32 {
        if let Some(&i) = self.ids.get(s) {
            return i;
        }
        let i = self.names.len() as u32;
        self.names.push(s.clone());
        self.ids.insert(s.clone(), i);
        i
    }
}

type PredId = u32;

/// A body literal in internal form. Variables are indices into the rule's
/// substitution vector.
#[derive(Clone, Debug)]
enum Arg {
    Var(u32),
    Val(Val),
}

#[derive(Clone, Debug)]
enum ILit {
    Pos(PredId, Vec<Arg>),
    Neg(PredId, Vec<Arg>),
    Switch(PredId, Vec<Arg>),
    NegSwitch(PredId, Vec<Arg>),
    Cmp(CmpOp, IExpr, IExpr),
    /// `var is expr`; the variable may already be bound (then compared).
    Is(Arg, IExpr),
}

/// An arithmetic expression in internal form.
#[derive(Clone, Debug)]
enum IExpr {
    Arg(Arg),
    Bin(ArithOp, Box<IExpr>, Box<IExpr>),
    Neg(Box<IExpr>),
}

impl IExpr {
    fn as_arg(&self) -> Option<&Arg> {
        match self {
            IExpr::Arg(a) => Some(a),
            _ => None,
        }
    }
}

/// Evaluate an expression; `None` when a variable is unbound or the
/// arithmetic is undefined (non-integer operand, division by zero).
fn eval_expr(e: &IExpr, subst: &[Option<Val>]) -> Option<Val> {
    match e {
        IExpr::Arg(Arg::Var(v)) => subst[*v as usize],
        IExpr::Arg(Arg::Val(v)) => Some(*v),
        IExpr::Neg(a) => match eval_expr(a, subst)? {
            Val::Int(i) => Some(Val::Int(-i)),
            Val::Sym(_) => None,
        },
        IExpr::Bin(op, a, b) => {
            let (Val::Int(x), Val::Int(y)) = (eval_expr(a, subst)?, eval_expr(b, subst)?) else {
                return None;
            };
            Some(Val::Int(match op {
                ArithOp::Add => x.checked_add(y)?,
                ArithOp::Sub => x.checked_sub(y)?,
                ArithOp::Mul => x.checked_mul(y)?,
                // ProbLog's `//` rounds down and its `mod` takes the
                // divisor's sign, as Python's do.
                ArithOp::Div => {
                    let (q, r) = (x.checked_div(y)?, x.checked_rem(y)?);
                    if r != 0 && (r < 0) != (y < 0) {
                        q - 1
                    } else {
                        q
                    }
                }
                ArithOp::Mod => {
                    let r = x.checked_rem(y)?;
                    if r != 0 && (r < 0) != (y < 0) {
                        r + y
                    } else {
                        r
                    }
                }
            }))
        }
    }
}

#[derive(Clone, Debug)]
struct IRule {
    head: (PredId, Vec<Arg>),
    /// Body literals, reordered so every negative/comparison literal comes
    /// right after the positive literals that bind its variables.
    body: Vec<ILit>,
    num_vars: u32,
}

/// Possible tuples of one predicate, with a per-argument index built lazily.
#[derive(Default)]
struct Relation {
    tuples: Vec<Vec<Val>>,
    /// The atom of each tuple.
    ids: Vec<AtomId>,
    /// Derived (possible) atoms, but for a relation of arity zero, whose
    /// one tuple is `ids[0]` once derived ([`Relation::get`]): every
    /// switch of a ground program is one, and needs no table allocated.
    set: FxHashMap<Vec<Val>, AtomId>,
    /// Atoms named by a negative literal inside their own recursive
    /// component before it is known whether they are possible. Promoted to
    /// `set` if derived; otherwise they end up as derived atoms without rules,
    /// which the simplifier treats as false.
    placeholders: FxHashMap<Vec<Val>, AtomId>,
    /// `index[pos][val]` = tuple indices with that value at `pos`.
    index: Vec<FxHashMap<Val, Vec<usize>>>,
    /// Number of tuples already indexed.
    indexed: usize,
}

impl Relation {
    /// The atom of a derived tuple.
    fn get(&self, args: &[Val]) -> Option<AtomId> {
        if args.is_empty() {
            self.ids.first().copied()
        } else {
            self.set.get(args).copied()
        }
    }

    fn ensure_index(&mut self, arity: usize) {
        if self.index.is_empty() {
            self.index = vec![FxHashMap::default(); arity];
        }
        for t in self.indexed..self.tuples.len() {
            for (pos, v) in self.tuples[t].iter().enumerate() {
                self.index[pos].entry(*v).or_default().push(t);
            }
        }
        self.indexed = self.tuples.len();
    }

    /// The tuples among `s..e` that may have value `v` at each position
    /// `pos` of `bound`, and one of the values `vals` at each position of
    /// `among`, in tuple order: those of the shortest posting list among
    /// `bound`, or of the fewest posting lists of one position of `among`
    /// together, or all of `s..e` where neither is fewer. The caller
    /// checks the other positions. A posting list is in tuple order, so the
    /// range is a slice of it.
    fn candidates(
        &mut self,
        arity: usize,
        bound: &[(usize, Val)],
        among: &[(usize, &[Val])],
        (s, e): (usize, usize),
    ) -> Vec<usize> {
        if bound.is_empty() && among.is_empty() {
            return (s..e).collect();
        }
        self.ensure_index(arity);
        let index = &self.index;
        let posting = |pos: usize, v: &Val| {
            let list = index[pos].get(v).map_or(&[][..], |l| &l[..]);
            &list[list.partition_point(|&t| t < s)..list.partition_point(|&t| t < e)]
        };
        let one = bound
            .iter()
            .map(|(pos, v)| posting(*pos, v))
            .min_by_key(|l| l.len());
        let union = among
            .iter()
            .map(|&(pos, vals)| {
                let lists: Vec<&[usize]> = vals.iter().map(|v| posting(pos, v)).collect();
                (lists.iter().map(|l| l.len()).sum::<usize>(), lists)
            })
            .min_by_key(|(n, _)| *n);
        match (one, union) {
            (Some(list), u) if u.as_ref().is_none_or(|(n, _)| list.len() <= *n) => list.to_vec(),
            (_, Some((n, lists))) if n < e - s => {
                let mut ts: Vec<usize> = lists.concat();
                ts.sort_unstable();
                ts
            }
            _ => (s..e).collect(),
        }
    }
}

/// What the delta literal of a rule being grounded admits ([`narrow`]):
/// the values its tuples in the delta give each of its variables, sorted,
/// and the body positions `from..to` at which a positive literal passes
/// over a tuple that binds one of them to another value.
#[derive(Default)]
struct Narrow {
    vars: Vec<(u32, Vec<Val>)>,
    from: usize,
    to: usize,
}

struct Pred<'a> {
    name: &'a Name,
    arity: usize,
    switch: Option<Prob>,
    /// A magic-set control predicate: grounded, never emitted.
    control: bool,
    rules: Vec<usize>,
}

struct Grounder<'a> {
    syms: Interner,
    preds: Vec<Pred<'a>>,
    /// The predicates by name and arity, the names borrowed from the
    /// program grounded, and whether each is a switch, in one lookup that
    /// copies no name: a network has a switch predicate per probabilistic
    /// head, and its rules name each switch once more for every later head
    /// of its row.
    pred_ids: FxHashMap<(&'a str, usize), (PredId, bool)>,
    rules: Vec<IRule>,
    rels: Vec<Relation>,
    atoms: Vec<(PredId, Vec<Val>)>,
    ground_rules: Vec<GroundRule>,
    kept: super::program::KeptRules,
    /// Predicates of the component being evaluated; a negative literal on one
    /// of them cannot be resolved yet.
    current_stratum: FxHashSet<PredId>,
    /// Buffers kept from one rule to the next: the predicates a body names
    /// ([`compile_rule`]), and the substitution and body literals of a
    /// rule being grounded ([`eval_rule`]), rather than allocated per rule.
    known: Vec<Option<(PredId, bool)>>,
    walk: (Vec<Option<Val>>, Vec<GLit>),
    narrow: Narrow,
    /// The size of `ground_rules`, a term per head and body literal, and
    /// the size past which the grounding is given up.
    size: usize,
    bound: usize,
    /// The control atoms derived, which the bound limits too: they are
    /// never emitted, and a demand that no rule's size follows can grow
    /// without end. Grounded from the bound end, `o(T1) :- step(T),
    /// T1 is T + 1, o(T)` asks `o(T+1)` of every `o(T)` it is asked,
    /// `step(T)` coming after the call.
    demands: usize,
    /// The values that grounding level by level proved, by atom
    /// ([`evaluate_by_levels`]).
    fixed: Vec<Option<bool>>,
    /// The stratum being grounded level by level.
    by_level: Option<ByLevel>,
}

/// A level and a rank within it: the place of an atom of a stratum
/// grounded level by level.
type Key = (i64, u32);

/// A rule instance: its head's predicate and arguments, and its body.
type Instance = (PredId, Vec<Val>, Vec<GLit>);

/// The state of a stratum grounded level by level.
struct ByLevel {
    rank: FxHashMap<PredId, u32>,
    /// The last key completed, and the key being grounded.
    done: Option<Key>,
    current: Key,
    /// The rule instances whose heads are at later keys.
    deferred: BTreeMap<Key, Vec<Instance>>,
}

/// The level of an atom whose last argument is `v`: an integer, the
/// lowest level for any other value.
fn level_of(v: Option<&Val>) -> i64 {
    match v {
        Some(Val::Int(i)) => *i,
        _ => i64::MIN,
    }
}

impl<'a> Grounder<'a> {
    /// Adds `rule` unless it is kept already.
    fn keep_rule(&mut self, rule: GroundRule) {
        if self.kept.keep(&self.ground_rules, &rule) {
            self.size += 1 + rule.body.len();
            self.ground_rules.push(rule);
        }
    }

    /// Whether the rules kept so far, or the control atoms, are past the
    /// bound.
    fn over(&self) -> bool {
        self.size > self.bound || self.demands > self.bound
    }

    /// A known predicate, and whether it is a switch.
    fn pred_id(&self, name: &str, arity: usize) -> Option<(PredId, bool)> {
        self.pred_ids.get(&(name, arity)).copied()
    }

    fn pred(&mut self, name: &'a Name, arity: usize, switch: Option<Prob>) -> PredId {
        let next = self.preds.len() as PredId;
        let (id, _) = *self
            .pred_ids
            .entry((&**name, arity))
            .or_insert((next, switch.is_some()));
        if id == next {
            self.preds.push(Pred {
                name,
                arity,
                switch,
                control: false,
                rules: Vec::new(),
            });
            self.rels.push(Relation::default());
        }
        id
    }

    fn val(&mut self, t: &Term) -> Val {
        match t {
            Term::Const(s) => Val::Sym(self.syms.intern(s)),
            Term::Int(i) => Val::Int(*i),
            Term::Var(_) => unreachable!("variables are not values"),
        }
    }

    fn atom_id(&mut self, pred: PredId, args: Vec<Val>) -> (AtomId, bool) {
        if let Some(id) = self.rels[pred as usize].get(&args) {
            return (id, false);
        }
        let rel = &mut self.rels[pred as usize];
        let id = match rel.placeholders.remove(&args) {
            Some(id) => id,
            None => {
                let id = self.atoms.len() as AtomId;
                self.atoms.push((pred, args.clone()));
                id
            }
        };
        let rel = &mut self.rels[pred as usize];
        if !args.is_empty() {
            rel.set.insert(args.clone(), id);
        }
        rel.tuples.push(args);
        rel.ids.push(id);
        (id, true)
    }

    /// The id of an atom that may or may not turn out possible.
    fn placeholder_atom(&mut self, pred: PredId, args: Vec<Val>) -> AtomId {
        if let Some(id) = self.rels[pred as usize].get(&args) {
            return id;
        }
        if let Some(&id) = self.rels[pred as usize].placeholders.get(&args) {
            return id;
        }
        let id = self.atoms.len() as AtomId;
        self.atoms.push((pred, args.clone()));
        self.rels[pred as usize].placeholders.insert(args, id);
        id
    }

    fn lookup_atom(&self, pred: PredId, args: &[Val]) -> Option<AtomId> {
        self.rels[pred as usize].get(args)
    }

    fn fixed(&self, id: AtomId) -> Option<bool> {
        self.fixed.get(id as usize).copied().flatten()
    }

    /// The key of an atom of the stratum grounded level by level.
    fn key_of(&self, id: AtomId) -> Option<Key> {
        let (p, args) = &self.atoms[id as usize];
        let r = *self.by_level.as_ref()?.rank.get(p)?;
        Some((level_of(args.last()), r))
    }

    /// Whether the atom of `p` that `args` names under `subst` is complete
    /// in the stratum grounded level by level: its key is done.
    fn complete(&self, p: PredId, args: &[Arg], subst: &[Option<Val>]) -> bool {
        let Some(b) = &self.by_level else {
            return false;
        };
        let Some(&r) = b.rank.get(&p) else {
            return false;
        };
        let level = match args.last() {
            Some(Arg::Var(v)) => subst[*v as usize],
            Some(Arg::Val(v)) => Some(*v),
            None => None,
        };
        b.done.is_some_and(|d| (level_of(level.as_ref()), r) <= d)
    }

    fn display_atom(&self, id: AtomId) -> Atom {
        let (p, args) = &self.atoms[id as usize];
        Atom::new(
            self.preds[*p as usize].name.clone(),
            args.iter()
                .map(|v| match v {
                    Val::Sym(s) => Term::Const(self.syms.names[*s as usize].clone()),
                    Val::Int(i) => Term::Int(*i),
                })
                .collect::<Args>(),
        )
    }
}

/// Compile the clauses to internal form, checking safety.
fn compile_rule<'a>(g: &mut Grounder<'a>, clause: &'a Clause) -> Result<IRule, GroundError> {
    let mut vars: FxHashMap<String, u32> = FxHashMap::default();
    let var_of = |t: &Term, vars: &mut FxHashMap<String, u32>, g: &mut Grounder<'a>| -> Arg {
        match t {
            Term::Var(v) => {
                let n = vars.len() as u32;
                Arg::Var(*vars.entry(v.clone()).or_insert(n))
            }
            other => Arg::Val(g.val(other)),
        }
    };
    let head_atom = &clause.heads[0].atom;
    let head_pred = g.pred(&head_atom.name, head_atom.args.len(), None);
    if g.preds[head_pred as usize].switch.is_some() {
        return Err(GroundError::Unsupported(format!(
            "rule head {} is a probabilistic switch predicate",
            head_atom
        )));
    }

    // Order the body so every literal has its variables bound (shared with
    // the magic-set rewrite), then compile in that order. Each atom's
    // predicate is looked up once, here: a predicate not known yet is no
    // switch, and is registered below in the order of the schedule.
    let mut known = std::mem::take(&mut g.known);
    known.clear();
    known.extend(clause.body.iter().map(|l| match l {
        Literal::Pos(a) | Literal::Neg(a) => g.pred_id(&a.name, a.args.len()),
        _ => None,
    }));
    let mut bound_names: FxHashSet<String> = FxHashSet::default();
    let ordered = schedule_body(&clause.body, &mut bound_names, |i| {
        known[i].is_none_or(|(_, switch)| !switch)
    })
    .map_err(|stuck| {
        GroundError::Unsafe(format!(
            "{clause}: literal {} has variables not bound by a positive body atom",
            clause.body[stuck]
        ))
    })?;
    let mut body: Vec<ILit> = Vec::with_capacity(ordered.len());
    for &i in &ordered {
        match &clause.body[i] {
            Literal::Pos(a) => {
                // A predicate not yet known is no switch: every switch is
                // registered before the rules are compiled.
                let (p, switch) =
                    known[i].unwrap_or_else(|| (g.pred(&a.name, a.args.len(), None), false));
                let args: Vec<Arg> = a.args.iter().map(|t| var_of(t, &mut vars, g)).collect();
                body.push(if switch {
                    ILit::Switch(p, args)
                } else {
                    ILit::Pos(p, args)
                });
            }
            Literal::Neg(a) => {
                let (p, switch) =
                    known[i].unwrap_or_else(|| (g.pred(&a.name, a.args.len(), None), false));
                let args: Vec<Arg> = a.args.iter().map(|t| var_of(t, &mut vars, g)).collect();
                body.push(if switch {
                    ILit::NegSwitch(p, args)
                } else {
                    ILit::Neg(p, args)
                });
            }
            Literal::Cmp(op, a, b) => {
                let a = compile_expr(a, &mut vars, g, &var_of);
                let b = compile_expr(b, &mut vars, g, &var_of);
                body.push(ILit::Cmp(*op, a, b));
            }
            Literal::Is(v, e) => {
                let v = var_of(v, &mut vars, g);
                let e = compile_expr(e, &mut vars, g, &var_of);
                body.push(ILit::Is(v, e));
            }
        }
    }
    let head_args: Vec<Arg> = head_atom
        .args
        .iter()
        .map(|t| var_of(t, &mut vars, g))
        .collect();
    let bound: FxHashSet<u32> = bound_names
        .iter()
        .filter_map(|n| vars.get(n).copied())
        .collect();
    for v in arg_vars(&head_args) {
        if !bound.contains(&v) {
            return Err(GroundError::Unsafe(format!(
                "{clause}: head variable not bound by a positive body atom"
            )));
        }
    }
    g.known = known;
    Ok(IRule {
        head: (head_pred, head_args),
        body,
        num_vars: vars.len() as u32,
    })
}

fn compile_expr<'a>(
    e: &Expr,
    vars: &mut FxHashMap<String, u32>,
    g: &mut Grounder<'a>,
    var_of: &impl Fn(&Term, &mut FxHashMap<String, u32>, &mut Grounder<'a>) -> Arg,
) -> IExpr {
    match e {
        Expr::Term(t) => IExpr::Arg(var_of(t, vars, g)),
        Expr::Bin(op, a, b) => IExpr::Bin(
            *op,
            Box::new(compile_expr(a, vars, g, var_of)),
            Box::new(compile_expr(b, vars, g, var_of)),
        ),
        Expr::Neg(a) => IExpr::Neg(Box::new(compile_expr(a, vars, g, var_of))),
    }
}

fn arg_vars(args: &[Arg]) -> Vec<u32> {
    args.iter()
        .filter_map(|a| match a {
            Arg::Var(v) => Some(*v),
            Arg::Val(_) => None,
        })
        .collect()
}

/// Predicate dependency graph → strata (lists of predicate ids), lowest first.
///
/// A recursive component with a negative dependency inside it is grounded
/// with its internal negative literals left in place, for the compiler's
/// well-founded treatment.
fn stratify(g: &Grounder) -> Vec<Vec<PredId>> {
    let n = g.preds.len();
    // edges: head depends on body predicate.
    let mut deps: Vec<Vec<PredId>> = vec![Vec::new(); n];
    for r in &g.rules {
        let h = r.head.0;
        for l in &r.body {
            if let ILit::Pos(p, _) | ILit::Neg(p, _) = l {
                deps[h as usize].push(*p);
            }
        }
    }
    // Each component as Tarjan's stack held it, taken off the top.
    let mut sccs: Vec<Vec<PredId>> = Vec::new();
    super::deps::tarjan(
        n,
        0..n,
        |v, (k, _)| {
            let w = *deps[v].get(*k)?;
            *k += 1;
            Some(w as usize)
        },
        |comp| sccs.push(comp.iter().rev().map(|&w| w as PredId).collect()),
    );
    // Tarjan emits SCCs in reverse topological order of the dependency graph
    // (a component is emitted after everything it depends on), which is the
    // evaluation order we want.
    sccs
}

/// The ground program of `norm`, or [`GroundError::OverBound`] as soon as
/// its rules outgrow `bound` terms, a term per head and body literal
/// ([`GroundProgram::size`]).
pub(crate) fn ground_normalized(
    norm: &NormProgram,
    bound: Option<usize>,
) -> Result<GroundProgram, GroundError> {
    Ok(ground_with(norm, bound, false)?.expect("grounded"))
}

/// The ground program of `norm` with each recursive stratum grounded level
/// by level ([`evaluate_by_levels`]), where every one has rising levels
/// ([`levels`]) and there is one; `None` otherwise.
pub(crate) fn ground_by_levels(
    norm: &NormProgram,
    bound: Option<usize>,
) -> Result<Option<GroundProgram>, GroundError> {
    ground_with(norm, bound, true)
}

fn ground_with(
    norm: &NormProgram,
    bound: Option<usize>,
    by_levels: bool,
) -> Result<Option<GroundProgram>, GroundError> {
    // Every switch is a predicate, most of the predicates of a network.
    let preds = norm.switches.len() + 64;
    let mut g = Grounder {
        syms: Interner::default(),
        preds: Vec::with_capacity(preds),
        pred_ids: FxHashMap::with_capacity_and_hasher(preds, Default::default()),
        rules: Vec::with_capacity(norm.rules.len()),
        rels: Vec::with_capacity(preds),
        atoms: Vec::new(),
        ground_rules: Vec::new(),
        kept: super::program::KeptRules::default(),
        current_stratum: FxHashSet::default(),
        known: Vec::new(),
        walk: (Vec::new(), Vec::new()),
        narrow: Narrow::default(),
        size: 0,
        bound: bound.unwrap_or(usize::MAX),
        demands: 0,
        fixed: Vec::new(),
        by_level: None,
    };
    for (&(ref name, arity), &p) in &norm.switches {
        g.pred(name, arity, Some(p));
    }
    for (name, arity) in &norm.control {
        let p = g.pred(name, *arity, None);
        g.preds[p as usize].control = true;
    }
    for clause in &norm.rules {
        let r = compile_rule(&mut g, clause)?;
        let h = r.head.0;
        g.preds[h as usize].rules.push(g.rules.len());
        g.rules.push(r);
    }
    // Predicates only mentioned in queries/evidence.
    for a in norm
        .queries
        .iter()
        .map(|(a, _)| a)
        .chain(norm.evidence.iter().map(|(a, _)| a))
    {
        g.pred(&a.name, a.args.len(), None);
    }

    let strata = stratify(&g);
    // The levels of each recursive stratum, where every one has them.
    let mut levelled: Vec<Option<Levels>> = Vec::new();
    if by_levels {
        for s in &strata {
            if !recursive(&g, s) {
                levelled.push(None);
                continue;
            }
            let Some(lv) = levels(&g, s) else {
                return Ok(None);
            };
            levelled.push(Some(lv));
        }
        if levelled.iter().all(Option::is_none) {
            return Ok(None);
        }
    }
    let mut pruner = by_levels.then(super::aig::Pruner::new);
    let mut keys = 0;
    for (k, stratum) in strata.iter().enumerate() {
        // A stratum without rules derives nothing: every switch predicate
        // is one, a hundred thousand of them on bn_barley.
        if stratum
            .iter()
            .all(|&p| g.preds[p as usize].rules.is_empty())
        {
            continue;
        }
        g.current_stratum = stratum.iter().copied().collect();
        match (levelled.get_mut(k).and_then(Option::take), &mut pruner) {
            (Some(lv), Some(pruner)) => keys += evaluate_by_levels(&mut g, stratum, lv, pruner),
            _ => evaluate_stratum(&mut g, stratum),
        }
        if g.over() {
            return Err(GroundError::OverBound);
        }
    }
    g.current_stratum.clear();
    // The rules of an atom proved false go, and those of an atom proved
    // true, which keeps one empty body.
    let fixed = std::mem::take(&mut g.fixed);
    if !fixed.is_empty() {
        let value = |a: AtomId| fixed.get(a as usize).copied().flatten();
        g.ground_rules.retain(|r| value(r.head).is_none());
    }
    for (a, v) in fixed.iter().enumerate() {
        if *v == Some(true) {
            g.ground_rules
                .push(GroundRule::new(a as AtomId, Vec::new()));
        }
    }

    // Assemble the output.
    let mut gp = GroundProgram::default();
    for id in 0..g.atoms.len() as AtomId {
        gp.atoms.push(g.display_atom(id));
        let (p, _) = &g.atoms[id as usize];
        gp.kinds.push(match g.preds[*p as usize].switch {
            Some(q) => AtomKind::Fact(q),
            None => AtomKind::Derived,
        });
    }
    gp.rules = std::mem::take(&mut g.ground_rules);
    gp.stats.levels = keys;
    gp.stats.constants = fixed.iter().filter(|v| v.is_some()).count();

    for (q, positive) in &norm.queries {
        for (atom, id) in resolve_atom_pattern(&mut g, q) {
            gp.queries.push((atom, id, *positive));
        }
    }
    for (driver, pattern, positive) in &norm.query_drivers {
        // Every true instance of the driver binds the pattern's variables.
        for (instance, _) in resolve_atom_pattern(&mut g, driver) {
            let binding: FxHashMap<&Term, &Term> = driver.args.iter().zip(&instance.args).collect();
            let q = Atom::new(
                pattern.name.clone(),
                pattern
                    .args
                    .iter()
                    .map(|t| binding.get(t).copied().unwrap_or(t).clone())
                    .collect::<Args>(),
            );
            for (atom, id) in resolve_atom_pattern(&mut g, &q) {
                gp.queries.push((atom, id, *positive));
            }
        }
    }
    // The driver atoms themselves are not part of the output program.
    let drivers: FxHashSet<Name> = norm
        .query_drivers
        .iter()
        .map(|(d, _, _)| d.name.clone())
        .collect();
    gp.rules
        .retain(|r| !drivers.contains(&gp.atoms[r.head as usize].name));
    for (e, v) in &norm.evidence {
        for (atom, id) in resolve_atom_pattern(&mut g, e) {
            gp.evidence.push((atom, id, *v));
        }
    }
    Ok(Some(gp))
}

/// Ground instances of a possibly non-ground query/evidence atom among the
/// possible atoms. A ground atom that is impossible resolves to `None`.
fn resolve_atom_pattern(g: &mut Grounder, a: &Atom) -> Vec<(Atom, Option<AtomId>)> {
    // A predicate the program does not mention has no atoms.
    let Some((p, _)) = g.pred_id(&a.name, a.args.len()) else {
        return if a.is_ground() {
            vec![(a.clone(), None)]
        } else {
            Vec::new()
        };
    };
    if a.is_ground() {
        let args: Vec<Val> = a.args.iter().map(|t| g.val(t)).collect();
        return vec![(a.clone(), g.lookup_atom(p, &args))];
    }
    let mut out = Vec::new();
    let rel = &g.rels[p as usize];
    for (t, &id) in rel.tuples.iter().zip(&rel.ids) {
        let mut binding: FxHashMap<&str, Val> = FxHashMap::default();
        let mut ok = true;
        for (pat, v) in a.args.iter().zip(t) {
            match pat {
                Term::Var(x) => match binding.get(x.as_str()) {
                    Some(b) if b != v => {
                        ok = false;
                        break;
                    }
                    Some(_) => {}
                    None => {
                        binding.insert(x, *v);
                    }
                },
                Term::Const(s) => {
                    if *v != Val::Sym(*g.syms.ids.get(s).unwrap_or(&u32::MAX)) {
                        ok = false;
                        break;
                    }
                }
                Term::Int(i) => {
                    if *v != Val::Int(*i) {
                        ok = false;
                        break;
                    }
                }
            }
        }
        if ok {
            out.push((g.display_atom(id), Some(id)));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Semi-naive evaluation of one stratum (a set of mutually recursive predicates).
///
/// Round 0 grounds every rule against the relations as they are (the
/// stratum's own empty, but for its facts); then [`rounds`] over the
/// tuples each round adds.
fn evaluate_stratum(g: &mut Grounder, stratum: &[PredId]) {
    let rule_ids = rules_of(g, stratum);
    let slot: FxHashMap<PredId, usize> = stratum.iter().enumerate().map(|(s, &p)| (p, s)).collect();
    // reads[s]: the rules, as places in `rule_ids`, and the literals in
    // which the stratum's predicate `s` is read positively.
    let mut reads: Vec<Vec<(usize, usize)>> = vec![Vec::new(); stratum.len()];
    for (k, &r) in rule_ids.iter().enumerate() {
        for (i, l) in g.rules[r].body.iter().enumerate() {
            if let ILit::Pos(p, _) = l
                && let Some(&s) = slot.get(p)
            {
                reads[s].push((k, i));
            }
        }
    }
    let mut from = lens(g, stratum);
    for &r in &rule_ids {
        eval_rule(g, r, None);
    }
    rounds(g, &rule_ids, stratum, &mut from, |s, due| {
        due.extend_from_slice(&reads[s])
    });
}

/// The rules of the predicates of `stratum`.
fn rules_of(g: &Grounder, stratum: &[PredId]) -> Vec<usize> {
    stratum
        .iter()
        .flat_map(|p| g.preds[*p as usize].rules.iter().copied())
        .collect()
}

/// The number of tuples of each predicate of `preds`.
fn lens(g: &Grounder, preds: &[PredId]) -> Vec<usize> {
    preds
        .iter()
        .map(|&p| g.rels[p as usize].tuples.len())
        .collect()
}

/// Rounds of semi-naive evaluation over `group` until one derives
/// nothing. Each grounds the rules again against the tuples the round
/// before added (those since `from` in the first), a rule once per
/// positive literal of `group` whose predicate gained tuples, in the order
/// of the rules and their literals. `reads(s, due)` adds the rules, as
/// places in `rule_ids`, and the literals that read `group[s]`: a round
/// visits those, not every rule. A chain over a time index takes a round
/// per step where it is one stratum, as the magic rewrite's demand makes
/// it (pmc1's step, 1 400 rules, for each of 75 steps a few dozen rounds
/// deep).
fn rounds(
    g: &mut Grounder,
    rule_ids: &[usize],
    group: &[PredId],
    from: &mut [usize],
    reads: impl Fn(usize, &mut Vec<(usize, usize)>),
) {
    let mut delta: Vec<(usize, usize)> = vec![(0, 0); group.len()];
    let mut lits: Vec<(usize, usize)> = Vec::new();
    let mut due: Vec<(usize, usize, usize)> = Vec::new();
    while !g.over() {
        due.clear();
        for (s, &p) in group.iter().enumerate() {
            let e = g.rels[p as usize].tuples.len();
            delta[s] = (from[s], e);
            if e > from[s] {
                lits.clear();
                reads(s, &mut lits);
                due.extend(lits.iter().map(|&(k, i)| (k, i, s)));
            }
            from[s] = e;
        }
        if due.is_empty() {
            break;
        }
        due.sort_unstable();
        for &(k, i, s) in &due {
            eval_rule(g, rule_ids[k], Some((i, delta[s])));
        }
    }
}

/// Whether a rule of `stratum` reads a predicate of it.
fn recursive(g: &Grounder, stratum: &[PredId]) -> bool {
    stratum.iter().any(|&p| {
        g.preds[p as usize].rules.iter().any(|&r| {
            g.rules[r]
                .body
                .iter()
                .any(|l| matches!(l, ILit::Pos(q, _) | ILit::Neg(q, _) if stratum.contains(q)))
        })
    })
}

/// The levels of a recursive stratum, an XY-stratification (Zaniolo, Arni
/// and Ong, DOOD 1993): the last argument of each of its predicates is a
/// level that no rule lowers. Each literal of the stratum is read at its
/// head's level (the same variable, or constant levels h >= b) or below it
/// (`H is B + c`, c >= 1), and some literal below it. Within a level the
/// components of the graph of the literals read at their head's level rank
/// the predicates, the lower ranks read by the higher. Grounded in the
/// order of their keys (level, rank), a key's atoms are complete when it
/// is done: no later key derives one.
struct Levels {
    rank: FxHashMap<PredId, u32>,
    /// Each rank's predicates, and whether they read each other at their
    /// own level.
    groups: Vec<Vec<PredId>>,
    cyclic: Vec<bool>,
    /// How far below its head's level each literal of the stratum is read,
    /// by rule and literal.
    offset: FxHashMap<(usize, usize), i64>,
}

fn levels(g: &Grounder, stratum: &[PredId]) -> Option<Levels> {
    let local: FxHashMap<PredId, usize> =
        stratum.iter().enumerate().map(|(s, &p)| (p, s)).collect();
    if stratum
        .iter()
        .any(|&p| g.preds[p as usize].arity == 0 || g.preds[p as usize].control)
    {
        return None;
    }
    let mut deps: Vec<Vec<usize>> = vec![Vec::new(); stratum.len()];
    let mut self_loop = vec![false; stratum.len()];
    let mut offset = FxHashMap::default();
    for &p in stratum {
        let h = local[&p];
        for &r in &g.preds[p as usize].rules {
            let rule = &g.rules[r];
            for (i, l) in rule.body.iter().enumerate() {
                let (ILit::Pos(q, args) | ILit::Neg(q, args)) = l else {
                    continue;
                };
                let Some(&b) = local.get(q) else { continue };
                let off = match (rule.head.1.last(), args.last()) {
                    (Some(Arg::Var(vh)), Some(Arg::Var(vb))) if vh == vb => 0,
                    (Some(Arg::Var(vh)), Some(Arg::Var(vb))) => step_of(&rule.body, *vh, *vb)?,
                    (Some(Arg::Val(Val::Int(h))), Some(Arg::Val(Val::Int(b)))) if h >= b => h - b,
                    _ => return None,
                };
                offset.insert((r, i), off);
                if off == 0 {
                    deps[h].push(b);
                    self_loop[h] |= h == b;
                }
            }
        }
    }
    if !offset.values().any(|&o| o > 0) {
        return None;
    }
    let (mut rank, mut groups, mut cyclic) = (FxHashMap::default(), Vec::new(), Vec::new());
    super::deps::tarjan(
        stratum.len(),
        0..stratum.len(),
        |v, (k, _)| {
            let w = *deps[v].get(*k)?;
            *k += 1;
            Some(w)
        },
        |comp| {
            for &s in comp {
                rank.insert(stratum[s], groups.len() as u32);
            }
            cyclic.push(comp.len() > 1 || self_loop[comp[0]]);
            groups.push(comp.iter().map(|&s| stratum[s]).collect::<Vec<_>>());
        },
    );
    Some(Levels {
        rank,
        groups,
        cyclic,
        offset,
    })
}

/// c where `body` holds `vh is vb + c` or `vh is c + vb`, c >= 1.
fn step_of(body: &[ILit], vh: u32, vb: u32) -> Option<i64> {
    body.iter().find_map(|l| match l {
        ILit::Is(Arg::Var(v), IExpr::Bin(ArithOp::Add, a, b)) if *v == vh => match (&**a, &**b) {
            (IExpr::Arg(Arg::Var(x)), IExpr::Arg(Arg::Val(Val::Int(c))))
            | (IExpr::Arg(Arg::Val(Val::Int(c))), IExpr::Arg(Arg::Var(x)))
                if *x == vb && *c >= 1 =>
            {
                Some(*c)
            }
            _ => None,
        },
        _ => None,
    })
}

/// The literals that read each predicate positively, at a constant level
/// or at any, as places (rule, literal).
type Reads = FxHashMap<(PredId, Option<i64>), Vec<(usize, usize)>>;

/// The literals of `reads` that read `p` at `level`: those at any level
/// and those at that constant level.
fn read_at(reads: &Reads, p: PredId, level: i64) -> impl Iterator<Item = (usize, usize)> + '_ {
    [None, Some(level)]
        .into_iter()
        .flat_map(move |at| reads.get(&(p, at)).into_iter().flatten().copied())
}

/// Grounds a stratum with levels key by key, in their order. A rule
/// instance whose head is at a later key waits for it in
/// [`ByLevel::deferred`], and a cyclic group reads its own key's atoms in
/// [`rounds`], so that a key's atoms are complete before a rule reads them
/// at a later key. As each acyclic key completes, `pruner` builds its
/// atoms' gates over those of the keys before, in one and-inverter graph,
/// and its sweep proves some of them constant: the readers grounded after
/// pass over an atom proved false and drop one proved true from their
/// bodies, which on a loop over a time index leaves the steps' dead
/// branches ungrounded. A cyclic key's atoms are inputs of the graph.
/// Returns the number of keys.
fn evaluate_by_levels(
    g: &mut Grounder,
    stratum: &[PredId],
    lv: Levels,
    pruner: &mut super::aig::Pruner,
) -> usize {
    let rule_ids = rules_of(g, stratum);
    // The rules, as places in `rule_ids`, and the literals that read each
    // predicate positively: within, at their heads' keys, and later, at
    // later keys.
    let (mut within, mut later) = (Reads::default(), Reads::default());
    for (k, &r) in rule_ids.iter().enumerate() {
        let hr = lv.rank[&g.rules[r].head.0];
        for (i, l) in g.rules[r].body.iter().enumerate() {
            if let ILit::Pos(q, args) = l
                && let Some(&qr) = lv.rank.get(q)
            {
                let at = match args.last() {
                    Some(Arg::Val(Val::Int(c))) => Some(*c),
                    _ => None,
                };
                let reads = if lv.offset[&(r, i)] == 0 && qr == hr {
                    &mut within
                } else {
                    &mut later
                };
                reads.entry((*q, at)).or_default().push((k, i));
            }
        }
    }
    g.by_level = Some(ByLevel {
        rank: lv.rank,
        done: None,
        current: (i64::MIN, u32::MAX),
        deferred: BTreeMap::new(),
    });
    // Round 0 defers every head to its key.
    for &r in &rule_ids {
        eval_rule(g, r, None);
    }
    let mut keys = 0;
    while !g.over() {
        let b = g.by_level.as_mut().expect("grounding by levels");
        let Some((key, instances)) = b.deferred.pop_first() else {
            break;
        };
        b.current = key;
        keys += 1;
        let (group, cyclic) = (&lv.groups[key.1 as usize], lv.cyclic[key.1 as usize]);
        let start = lens(g, group);
        let r0 = g.ground_rules.len();
        for (p, args, body) in instances {
            emit(g, key, p, args, body);
        }
        if cyclic {
            let mut from = start.clone();
            rounds(g, &rule_ids, group, &mut from, |s, due| {
                due.extend(read_at(&within, group[s], key.0))
            });
        }
        g.by_level.as_mut().expect("grounding by levels").done = Some(key);
        let atoms: Vec<AtomId> = group
            .iter()
            .zip(&start)
            .flat_map(|(&p, &s)| g.rels[p as usize].ids[s..].to_vec())
            .collect();
        if cyclic {
            for a in atoms {
                pruner.input(a);
            }
        } else {
            let mut bodies: FxHashMap<AtomId, Vec<&[GLit]>> = FxHashMap::default();
            for r in &g.ground_rules[r0..] {
                bodies.entry(r.head).or_default().push(&r.body);
            }
            // An atom of the stratum read but never derived is false: its
            // key is done.
            let absent = |x: AtomId| {
                let (q, a) = &g.atoms[x as usize];
                g.key_of(x).is_some() && g.rels[*q as usize].get(a).is_none()
            };
            let values: Vec<(AtomId, bool)> = atoms
                .iter()
                .filter_map(|&a| {
                    let b = bodies.get(&a).map_or(&[][..], |b| &b[..]);
                    pruner.gate(a, b, &absent).map(|v| (a, v))
                })
                .collect();
            for (a, v) in values {
                if g.fixed.len() <= a as usize {
                    g.fixed.resize(a as usize + 1, None);
                }
                g.fixed[a as usize] = Some(v);
            }
        }
        for (s, &p) in group.iter().enumerate() {
            let e = g.rels[p as usize].tuples.len();
            if e > start[s] {
                for (k, i) in read_at(&later, p, key.0).collect::<Vec<_>>() {
                    eval_rule(g, rule_ids[k], Some((i, (start[s], e))));
                }
            }
        }
    }
    g.by_level = None;
    keys
}

/// The rule instance `p(args) :- body` deferred to `key`, now grounded: a
/// negated atom of a key before, read before its key was done, is
/// resolved.
fn emit(g: &mut Grounder, key: Key, p: PredId, args: Vec<Val>, body: Vec<GLit>) {
    let mut lits = Vec::with_capacity(body.len());
    for l in body {
        if let GLit::Neg(x) = l
            && g.key_of(x).is_some_and(|k| k < key)
        {
            let (q, a) = &g.atoms[x as usize];
            match (g.rels[*q as usize].get(a).is_some(), g.fixed(x)) {
                (false, _) | (true, Some(false)) => continue,
                (true, Some(true)) => return,
                (true, None) => {}
            }
        }
        lits.push(l);
    }
    let (head, _) = g.atom_id(p, args);
    lits.sort_unstable();
    lits.dedup();
    g.keep_rule(GroundRule::new(head, lits));
}

/// Evaluate rule `r`. With `delta = Some((i, range))`, body literal `i` ranges
/// only over tuples in `range` of its relation.
fn eval_rule(g: &mut Grounder, r: usize, delta: Option<(usize, (usize, usize))>) {
    // Grounding a rule adds atoms and ground rules but never rules, so the
    // rule is lent out for the walk and put back.
    let rule = std::mem::replace(
        &mut g.rules[r],
        IRule {
            head: (0, Vec::new()),
            body: Vec::new(),
            num_vars: 0,
        },
    );
    if delta.is_none_or(|(i, range)| narrow(g, &rule, i, range)) {
        let (mut subst, mut body_atoms) = std::mem::take(&mut g.walk);
        subst.clear();
        subst.resize(rule.num_vars as usize, None);
        body_atoms.clear();
        ground_body(g, &rule, 0, delta, &mut subst, &mut body_atoms);
        g.walk = (subst, body_atoms);
        g.narrow.to = 0;
    }
    g.rules[r] = rule;
}

/// Readies `g.narrow` for grounding `rule` with body literal `i` over the
/// tuples in `range` of its relation; false where that grounds nothing and
/// changes nothing, the rule then not grounded at all.
///
/// Semi-naive grounding walks the body in its order whichever literal
/// holds the delta, and a literal before it binds its variables to values
/// no tuple of the delta has: mnist_sum256's `partial(K,S) :- next(J,K),
/// partial(J,S0), digit(K,D), S is S0+D` with the delta in `digit`, the
/// ten values of one new digit a round, would walk every sum before it
/// each round. A positive
/// literal passes over such a tuple where no literal between it and the
/// delta's adds an atom, so the rules, atoms and their order are what the
/// walk would have made.
fn narrow(g: &mut Grounder, rule: &IRule, i: usize, (s, e): (usize, usize)) -> bool {
    let ILit::Pos(p, args) = &rule.body[i] else {
        return true;
    };
    if i == 0 {
        return true;
    }
    // The last literal before the delta's that may add an atom: a switch
    // not yet an atom, or a negative literal of the component.
    let adds = rule.body[..i].iter().rposition(|l| match l {
        ILit::Switch(q, a) | ILit::NegSwitch(q, a) => {
            !a.iter().all(|x| matches!(x, Arg::Val(_)))
                || with_resolved(a, &[], |vals| g.lookup_atom(*q, vals)).is_none()
        }
        ILit::Neg(q, _) => g.current_stratum.contains(q),
        _ => false,
    });
    // The delta's tuples that may match the literal's constants, not all
    // of it: on a chain of ground rules each round grounds every rule
    // against the delta, which round 0 makes the whole chain.
    let bound: Vec<(usize, Val)> = args
        .iter()
        .enumerate()
        .filter_map(|(pos, a)| match a {
            Arg::Val(v) => Some((pos, *v)),
            Arg::Var(_) => None,
        })
        .collect();
    let arity = g.preds[*p as usize].arity;
    let candidates = g.rels[*p as usize].candidates(arity, &bound, &[], (s, e));
    let narrow = &mut g.narrow;
    narrow.from = adds.map_or(0, |m| m + 1);
    narrow.to = i;
    narrow.vars.clear();
    for a in args {
        if let Arg::Var(x) = a
            && !narrow.vars.iter().any(|(y, _)| y == x)
        {
            narrow.vars.push((*x, Vec::new()));
        }
    }
    let rel = &g.rels[*p as usize];
    let mut any = false;
    for tuple in candidates.iter().map(|&t| &rel.tuples[t]) {
        if args
            .iter()
            .zip(tuple)
            .any(|(a, v)| matches!(a, Arg::Val(c) if c != v))
        {
            continue;
        }
        any = true;
        for (a, v) in args.iter().zip(tuple) {
            if let Arg::Var(x) = a {
                let (_, vals) = narrow
                    .vars
                    .iter_mut()
                    .find(|(y, _)| y == x)
                    .expect("each variable listed");
                vals.push(*v);
            }
        }
    }
    if !any && adds.is_none() {
        narrow.to = 0;
        return false;
    }
    for (_, vals) in &mut narrow.vars {
        vals.sort_unstable();
        vals.dedup();
    }
    true
}

fn resolve(args: &[Arg], subst: &[Option<Val>]) -> Vec<Val> {
    args.iter()
        .map(|a| match a {
            Arg::Var(v) => subst[*v as usize].expect("variable bound by schedule"),
            Arg::Val(v) => *v,
        })
        .collect()
}

/// `f` of the values of `args`, resolved on the stack where they fit, for a
/// lookup that keeps no key.
fn with_resolved<R>(args: &[Arg], subst: &[Option<Val>], f: impl FnOnce(&[Val]) -> R) -> R {
    const ON_STACK: usize = 8;
    if args.len() > ON_STACK {
        return f(&resolve(args, subst));
    }
    let mut vals = [Val::Int(0); ON_STACK];
    for (slot, a) in vals.iter_mut().zip(args) {
        *slot = match a {
            Arg::Var(v) => subst[*v as usize].expect("variable bound by schedule"),
            Arg::Val(v) => *v,
        };
    }
    f(&vals[..args.len()])
}

fn ground_body(
    g: &mut Grounder,
    rule: &IRule,
    i: usize,
    delta: Option<(usize, (usize, usize))>,
    subst: &mut Vec<Option<Val>>,
    body_atoms: &mut Vec<GLit>,
) {
    if g.over() {
        return;
    }
    if i == rule.body.len() {
        let head_args = resolve(&rule.head.1, subst);
        if let Some(b) = &mut g.by_level
            && let Some(&r) = b.rank.get(&rule.head.0)
        {
            let key = (level_of(head_args.last()), r);
            if key != b.current {
                debug_assert!(b.done.is_none_or(|d| key > d), "a head at a key done");
                b.deferred.entry(key).or_default().push((
                    rule.head.0,
                    head_args,
                    body_atoms.clone(),
                ));
                return;
            }
        }
        let (head, new) = g.atom_id(rule.head.0, head_args);
        if g.preds[rule.head.0 as usize].control {
            g.demands += new as usize;
            return; // control atoms exist only to drive grounding
        }
        let mut body = body_atoms.clone();
        body.sort_unstable();
        body.dedup();
        g.keep_rule(GroundRule::new(head, body));
        return;
    }
    match &rule.body[i] {
        ILit::Pos(p, args) => {
            let p = *p;
            let range = match delta {
                Some((j, range)) if j == i => Some(range),
                _ => None,
            };
            // Every argument bound, as in every literal of a ground program:
            // the tuple is the atom, looked up at once.
            if range.is_none()
                && args.iter().all(|a| {
                    matches!(a, Arg::Val(_))
                        || matches!(a, Arg::Var(v) if subst[*v as usize].is_some())
                })
            {
                let Some(id) = with_resolved(args, subst, |vals| g.lookup_atom(p, vals)) else {
                    return;
                };
                let fixed = g.fixed(id);
                if fixed == Some(false) {
                    return;
                }
                if g.preds[p as usize].control || fixed == Some(true) {
                    ground_body(g, rule, i + 1, delta, subst, body_atoms);
                } else {
                    body_atoms.push(GLit::Pos(id));
                    ground_body(g, rule, i + 1, delta, subst, body_atoms);
                    body_atoms.pop();
                }
                return;
            }
            // Candidate tuple indices, from the bound arguments' posting
            // lists, in the delta range where there is one. Scanning the
            // whole delta range instead would visit fifty times the
            // candidates on cora500, each of whose facts of `haswordtitle`
            // is a rule guarded by its magic atom, grounded against the many
            // new magic tuples of a round.
            let bound_pos: Vec<(usize, Val)> = args
                .iter()
                .enumerate()
                .filter_map(|(pos, a)| match a {
                    Arg::Val(v) => Some((pos, *v)),
                    Arg::Var(v) => subst[*v as usize].map(|val| (pos, val)),
                })
                .collect();
            // The variables this literal binds, unbound again after each
            // candidate.
            let free: Vec<u32> = args
                .iter()
                .filter_map(|a| match a {
                    Arg::Var(x) if subst[*x as usize].is_none() => Some(*x),
                    _ => None,
                })
                .collect();
            let control = g.preds[p as usize].control;
            // The variables it binds that the delta's literal reads, with
            // the values that literal admits ([`narrow`]).
            let narrowed: Vec<usize> = if (g.narrow.from..g.narrow.to).contains(&i) {
                g.narrow
                    .vars
                    .iter()
                    .enumerate()
                    .filter(|(_, (x, _))| free.contains(x))
                    .map(|(k, _)| k)
                    .collect()
            } else {
                Vec::new()
            };
            // The tuples with those values alone, read from the posting
            // lists of the values: the walk passes over the others, and
            // walking all of them each round made a chain over a time index
            // quadratic in its length (ladder's 6 400 steps: 6 s), the
            // index's relation recursive through the magic rewrite's
            // demand.
            let among: Vec<(usize, &[Val])> = narrowed
                .iter()
                .map(|&k| {
                    let (x, vals) = &g.narrow.vars[k];
                    let pos = args
                        .iter()
                        .position(|a| matches!(a, Arg::Var(y) if y == x))
                        .expect("a variable the literal binds");
                    (pos, &vals[..])
                })
                .collect();
            let arity = g.preds[p as usize].arity;
            let candidates = {
                let rel = &mut g.rels[p as usize];
                let range = range.unwrap_or((0, rel.tuples.len()));
                rel.candidates(arity, &bound_pos, &among, range)
            };
            for t in candidates {
                let rel = &g.rels[p as usize];
                let tuple = &rel.tuples[t];
                if !bound_pos.iter().all(|(pos, v)| tuple[*pos] == *v) {
                    continue;
                }
                let mut ok = true;
                for (a, v) in args.iter().zip(tuple) {
                    if let Arg::Var(x) = a {
                        match subst[*x as usize] {
                            None => subst[*x as usize] = Some(*v),
                            Some(b) if b != *v => {
                                ok = false;
                                break;
                            }
                            Some(_) => {}
                        }
                    }
                }
                if ok && !narrowed.is_empty() {
                    ok = narrowed.iter().all(|&k| {
                        let (x, vals) = &g.narrow.vars[k];
                        subst[*x as usize].is_some_and(|v| vals.binary_search(&v).is_ok())
                    });
                }
                let id = rel.ids[t];
                let fixed = g.fixed(id);
                if ok && fixed != Some(false) {
                    if control || fixed == Some(true) {
                        ground_body(g, rule, i + 1, delta, subst, body_atoms);
                    } else {
                        body_atoms.push(GLit::Pos(id));
                        ground_body(g, rule, i + 1, delta, subst, body_atoms);
                        body_atoms.pop();
                    }
                }
                for &x in &free {
                    subst[x as usize] = None;
                }
            }
        }
        ILit::Neg(p, args) => {
            if g.current_stratum.contains(p) && !g.complete(*p, args, subst) {
                // Same recursive component: possibility unknown yet.
                let id = g.placeholder_atom(*p, resolve(args, subst));
                body_atoms.push(GLit::Neg(id));
                ground_body(g, rule, i + 1, delta, subst, body_atoms);
                body_atoms.pop();
                return;
            }
            match with_resolved(args, subst, |vals| g.lookup_atom(*p, vals)) {
                Some(id) if g.fixed(id) == Some(true) => {}
                Some(id) if g.fixed(id) == Some(false) => {
                    ground_body(g, rule, i + 1, delta, subst, body_atoms)
                }
                // Possible atom: keep the negative literal.
                Some(id) => {
                    body_atoms.push(GLit::Neg(id));
                    ground_body(g, rule, i + 1, delta, subst, body_atoms);
                    body_atoms.pop();
                }
                // Impossible atom: the literal is true.
                None => ground_body(g, rule, i + 1, delta, subst, body_atoms),
            }
        }
        ILit::Switch(p, args) => {
            let vals = resolve(args, subst);
            let (id, _) = g.atom_id(*p, vals);
            body_atoms.push(GLit::Pos(id));
            ground_body(g, rule, i + 1, delta, subst, body_atoms);
            body_atoms.pop();
        }
        ILit::NegSwitch(p, args) => {
            let vals = resolve(args, subst);
            let (id, _) = g.atom_id(*p, vals);
            body_atoms.push(GLit::Neg(id));
            ground_body(g, rule, i + 1, delta, subst, body_atoms);
            body_atoms.pop();
        }
        ILit::Cmp(op, a, b) => {
            match (eval_expr(a, subst), eval_expr(b, subst)) {
                (Some(x), Some(y)) if op.holds(&val_term(g, x), &val_term(g, y)) => {
                    ground_body(g, rule, i + 1, delta, subst, body_atoms);
                }
                (None, Some(y)) if *op == CmpOp::Eq => {
                    if let Some(Arg::Var(v)) = a.as_arg() {
                        subst[*v as usize] = Some(y);
                        ground_body(g, rule, i + 1, delta, subst, body_atoms);
                        subst[*v as usize] = None;
                    }
                }
                (Some(x), None) if *op == CmpOp::Eq => {
                    if let Arg::Var(v) = b.as_arg().expect("schedule guarantees a variable side") {
                        subst[*v as usize] = Some(x);
                        ground_body(g, rule, i + 1, delta, subst, body_atoms);
                        subst[*v as usize] = None;
                    }
                }
                // Undefined arithmetic (or an unbound side) fails the literal.
                _ => {}
            }
        }
        ILit::Is(v, e) => {
            if let Some(x) = eval_expr(e, subst) {
                match v {
                    Arg::Var(var) => match subst[*var as usize] {
                        None => {
                            subst[*var as usize] = Some(x);
                            ground_body(g, rule, i + 1, delta, subst, body_atoms);
                            subst[*var as usize] = None;
                        }
                        Some(b) => {
                            if b == x {
                                ground_body(g, rule, i + 1, delta, subst, body_atoms);
                            }
                        }
                    },
                    Arg::Val(c) => {
                        if *c == x {
                            ground_body(g, rule, i + 1, delta, subst, body_atoms);
                        }
                    }
                }
            }
        }
    }
}

fn val_term(g: &Grounder, v: Val) -> Term {
    match v {
        Val::Sym(s) => Term::Const(g.syms.names[s as usize].clone()),
        Val::Int(i) => Term::Int(i),
    }
}
