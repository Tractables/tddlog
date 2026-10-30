//! Stratified semi-naive bottom-up grounding.
//!
//! Every switch atom is assumed true, so the fixpoint computes the atoms that
//! are true in at least one possible world. Along the way each successful rule
//! instantiation is recorded as a ground rule. A negative literal over a lower
//! stratum is kept when its atom is possible and dropped (as true) otherwise.

use std::collections::BTreeMap;
use std::fmt;
use std::hash::BuildHasher;

use rustc_hash::{FxHashMap, FxHashSet};

use std::rc::Rc;
use std::sync::Arc;

use super::interned::Interned;
use super::normalize::{Facts, NormProgram, Weights, switch_weights};
use super::schedule::{Order, literal_vars, schedule_body};
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, Prob};
use crate::syntax::{
    Args, Atom, Builtin, CONS, Clause, CmpOp, Compound, F64, Literal, NIL, Name, Term, term_cmp,
};

#[derive(Debug, Clone, PartialEq)]
pub enum GroundError {
    Unsupported(String),
    /// A rule is not range restricted or a negative/comparison literal has an
    /// unbound variable.
    Unsafe(String),
    /// The grounding outgrew the size it was allowed (`ground_normalized`),
    /// which only a grounding weighed against another is given.
    OverBound,
    /// The allocator refused the memory the grounding grew by.
    OutOfMemory,
    /// The grounding built a term deeper than `MAX_DEPTH`: it builds ever
    /// deeper terms, as `nat(s(X)) :- nat(X)` does, and would not end.
    Unbounded(String),
    /// A findall that the grounder cannot list as ProbLog does, whichever
    /// way the program is ground (the grounder's `terms` module).
    Refused(String),
    /// A query, an observation or a call of a predicate that has no
    /// clauses, which ProbLog refuses too (the grounder's `terms` module).
    NoClauses(String),
}

impl fmt::Display for GroundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroundError::Unsupported(s) => write!(f, "unsupported: {s}"),
            GroundError::Unsafe(s) => write!(f, "unsafe rule: {s}"),
            GroundError::OverBound => f.write_str("the grounding outgrew its bound"),
            GroundError::OutOfMemory => f.write_str("the grounding ran out of memory"),
            GroundError::Unbounded(s) => write!(
                f,
                "the grounding does not end: it builds terms deeper than {MAX_DEPTH}, as {s}"
            ),
            GroundError::Refused(s) => write!(f, "unsupported findall: {s}"),
            GroundError::NoClauses(s) => write!(f, "no clauses for {s}"),
        }
    }
}

impl std::error::Error for GroundError {}

/// The depth of the deepest term the grounder builds.
pub(crate) const MAX_DEPTH: u32 = 10_000;

/// An interned constant: a symbol, a number, or a ground compound term of
/// [`Terms`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Val {
    Sym(u32),
    Int(i64),
    Fn(u32),
    /// A float's bits: `2.0` is not `2`.
    Float(u64),
}

/// The ground compound terms the grounder builds, each once, a value
/// ([`Val::Fn`]) whose structure is its name and its arguments' values.
#[derive(Default)]
struct Terms {
    /// Each term's name and arguments, `[Sym(name), args..]`, and its
    /// depth.
    keys: Interned<Val>,
    depths: Vec<u32>,
    /// The terms built from their keys, for printing and the standard
    /// order ([`Grounder::term_of`]): those of the atoms printed and the
    /// values compared, not every term, which a grammar's demands make by
    /// the tens of millions.
    built: FxHashMap<u32, Term>,
}

/// A number of the arithmetic.
#[derive(Clone, Copy, Debug)]
enum Num {
    I(i64),
    F(f64),
}

impl Num {
    fn val(self) -> Val {
        match self {
            Num::I(i) => Val::Int(i),
            Num::F(x) => Val::Float(x.to_bits()),
        }
    }
    /// The number as a term holds it ([`F64::rounded`]): what `is` binds.
    /// A comparison reads the number computed.
    fn rounded(self) -> Num {
        match self {
            Num::F(x) => Num::F(F64::rounded(x).0),
            i => i,
        }
    }
    fn float(self) -> f64 {
        match self {
            Num::I(i) => i as f64,
            Num::F(x) => x,
        }
    }
}

/// An arithmetic function or operator of ProbLog's, as Python computes it:
/// an operation on integers stays exact, one with a float operand is a
/// float operation, and `/` divides in floating point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArithFn {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Mod,
    Rem,
    Pow,
    Neg,
    Plus,
    Abs,
    Sign,
    Min,
    Max,
    BitAnd,
    BitOr,
    Xor,
    Shl,
    Shr,
    BitNot,
    Float,
    Floor,
    Ceiling,
    Truncate,
    FractionalPart,
    Sqrt,
    Exp,
    Log,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
}

impl ArithFn {
    fn of(name: &str, arity: usize) -> Option<ArithFn> {
        use ArithFn::*;
        Some(match (name, arity) {
            ("+", 2) => Add,
            ("-", 2) => Sub,
            ("*", 2) => Mul,
            ("/", 2) => Div,
            ("//", 2) => IntDiv,
            ("mod", 2) => Mod,
            ("rem", 2) => Rem,
            ("**" | "^", 2) => Pow,
            ("-", 1) => Neg,
            ("+", 1) => Plus,
            ("abs", 1) => Abs,
            ("sign", 1) => Sign,
            ("min", 2) => Min,
            ("max", 2) => Max,
            ("/\\", 2) => BitAnd,
            ("\\/", 2) => BitOr,
            ("xor", 2) => Xor,
            ("<<", 2) => Shl,
            (">>", 2) => Shr,
            ("\\", 1) => BitNot,
            ("float", 1) => Float,
            ("floor", 1) => Floor,
            ("ceiling", 1) => Ceiling,
            ("truncate", 1) => Truncate,
            ("float_fractional_part", 1) => FractionalPart,
            ("sqrt", 1) => Sqrt,
            ("exp", 1) => Exp,
            ("log", 1) => Log,
            ("sin", 1) => Sin,
            ("cos", 1) => Cos,
            ("tan", 1) => Tan,
            ("asin", 1) => Asin,
            ("acos", 1) => Acos,
            ("atan", 1) => Atan,
            ("atan2" | "atan", 2) => Atan2,
            _ => return None,
        })
    }

    /// The function of `args`; `None` where it is undefined (a division by
    /// zero, an integer overflow, a bit operation on a float).
    fn apply(self, args: &[Num]) -> Option<Num> {
        use ArithFn::*;
        use Num::{F, I};
        let float = |f: fn(f64) -> f64| Some(F(f(args[0].float())));
        // Python's floor division and modulo, which round towards minus
        // infinity, the modulo taking the divisor's sign.
        let floor_div = |x: i64, y: i64| {
            let (q, r) = (x.checked_div(y)?, x.checked_rem(y)?);
            Some(if r != 0 && (r < 0) != (y < 0) {
                q - 1
            } else {
                q
            })
        };
        let floor_mod = |x: i64, y: i64| {
            let r = x.checked_rem(y)?;
            Some(if r != 0 && (r < 0) != (y < 0) {
                r + y
            } else {
                r
            })
        };
        let to_int = |x: f64| (x.is_finite() && x.abs() < 9.2e18).then_some(I(x as i64));
        match (self, args) {
            (Add, &[I(x), I(y)]) => x.checked_add(y).map(I),
            (Sub, &[I(x), I(y)]) => x.checked_sub(y).map(I),
            (Mul, &[I(x), I(y)]) => x.checked_mul(y).map(I),
            (Add, &[x, y]) => Some(F(x.float() + y.float())),
            (Sub, &[x, y]) => Some(F(x.float() - y.float())),
            (Mul, &[x, y]) => Some(F(x.float() * y.float())),
            (Div, &[x, y]) => (y.float() != 0.0).then(|| F(x.float() / y.float())),
            (IntDiv, &[I(x), I(y)]) => floor_div(x, y).map(I),
            (IntDiv, &[x, y]) => (y.float() != 0.0).then(|| F((x.float() / y.float()).floor())),
            (Mod, &[I(x), I(y)]) => floor_mod(x, y).map(I),
            (Mod, &[x, y]) => {
                let (x, y) = (x.float(), y.float());
                (y != 0.0).then(|| F(x - y * (x / y).floor()))
            }
            (Rem, &[I(x), I(y)]) => x.checked_rem(y).map(I),
            (Rem, &[x, y]) => (y.float() != 0.0).then(|| F(x.float() % y.float())),
            (Pow, &[I(x), I(y)]) if y >= 0 => {
                u32::try_from(y).ok().and_then(|y| x.checked_pow(y)).map(I)
            }
            (Pow, &[x, y]) => Some(F(x.float().powf(y.float()))),
            (Neg, &[I(x)]) => x.checked_neg().map(I),
            (Neg, &[F(x)]) => Some(F(-x)),
            (Plus, &[x]) => Some(x),
            (Abs, &[I(x)]) => x.checked_abs().map(I),
            (Abs, &[F(x)]) => Some(F(x.abs())),
            (Sign, &[I(x)]) => Some(I(x.signum())),
            (Sign, &[F(x)]) => Some(F(if x > 0.0 {
                1.0
            } else if x < 0.0 {
                -1.0
            } else {
                x
            })),
            // Python's `min` and `max` return the first of equal values.
            (Min, &[x, y]) => Some(if y.float() < x.float() { y } else { x }),
            (Max, &[x, y]) => Some(if y.float() > x.float() { y } else { x }),
            (BitAnd, &[I(x), I(y)]) => Some(I(x & y)),
            (BitOr, &[I(x), I(y)]) => Some(I(x | y)),
            (Xor, &[I(x), I(y)]) => Some(I(x ^ y)),
            (Shl, &[I(x), I(y)]) => u32::try_from(y).ok().and_then(|y| x.checked_shl(y)).map(I),
            (Shr, &[I(x), I(y)]) => u32::try_from(y).ok().map(|y| I(x >> y.min(63))),
            (BitNot, &[I(x)]) => Some(I(!x)),
            (Float, &[x]) => Some(F(x.float())),
            (Floor | Ceiling | Truncate, &[I(x)]) => Some(I(x)),
            (Floor, &[F(x)]) => to_int(x.floor()),
            (Ceiling, &[F(x)]) => to_int(x.ceil()),
            (Truncate, &[F(x)]) => to_int(x.trunc()),
            (FractionalPart, &[I(_)]) => Some(I(0)),
            (FractionalPart, &[F(x)]) => Some(F(x - x.trunc())),
            (Sqrt, &[x]) => (x.float() >= 0.0).then(|| F(x.float().sqrt())),
            (Exp, _) => float(f64::exp),
            (Log, &[x]) => (x.float() > 0.0).then(|| F(x.float().ln())),
            (Sin, _) => float(f64::sin),
            (Cos, _) => float(f64::cos),
            (Tan, _) => float(f64::tan),
            (Asin, _) => float(f64::asin),
            (Acos, _) => float(f64::acos),
            (Atan, _) => float(f64::atan),
            (Atan2, &[y, x]) => Some(F(y.float().atan2(x.float()))),
            _ => None,
        }
    }
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
    /// A comparison; the sides of one of terms are arguments.
    Cmp(CmpOp, IExpr, IExpr),
    /// `var is expr`; the variable may already be bound (then compared).
    Is(Arg, IExpr),
    /// `V = f(A..)`: `V` taken apart where it is bound, built otherwise.
    Build(Arg, u32, Vec<Arg>),
    Builtin(Builtin, Vec<Arg>, bool),
    Collect(Box<ICollect>),
}

/// A findall in internal form ([`crate::syntax::Collect`]).
#[derive(Clone, Debug)]
struct ICollect {
    aux: PredId,
    shared: Vec<Arg>,
    result: Arg,
    text: Arc<str>,
}

/// An arithmetic expression in internal form.
#[derive(Clone, Debug)]
enum IExpr {
    Arg(Arg),
    Op(ArithFn, Vec<IExpr>),
}

impl IExpr {
    fn as_arg(&self) -> Option<&Arg> {
        match self {
            IExpr::Arg(a) => Some(a),
            IExpr::Op(..) => None,
        }
    }
}

/// What a counted rule records of its derivations ([`Derivation`]): its
/// clause's group, and the places in the walk of its body literals in the
/// clause's order.
#[derive(Clone, Debug)]
struct Counted {
    group: u32,
    order: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
struct IRule {
    head: (PredId, Vec<Arg>),
    /// Body literals, reordered so every negative/comparison literal comes
    /// right after the positive literals that bind its variables.
    body: Vec<ILit>,
    num_vars: u32,
    /// Of a rule of a predicate a findall reads.
    counted: Option<Box<Counted>>,
}

/// What a body literal of a counted rule contributes to a derivation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    None,
    Atom(AtomId),
    Neg(AtomId),
    /// The rank of the solution of a list predicate that enumerates them.
    Rank(u32),
}

/// A ground rule of a predicate a findall reads, its body as kept, as
/// derived: from a clause of `group`, by one substitution or more (`mult`,
/// up to two), the first in the walk numbered `stamp`, its body's parts in
/// the clause's order.
#[derive(Debug)]
struct Derivation {
    body: Vec<GLit>,
    group: u32,
    stamp: u32,
    mult: u8,
    parts: Vec<Part>,
}

/// Possible tuples of one predicate, each once, in the order derived, with
/// a per-argument index built lazily.
struct Relation {
    /// The tuples' values, `arity` to a tuple, in the order derived: one
    /// block for all, rather than one a tuple, which a table of millions
    /// allocated and freed one by one.
    vals: Vec<Val>,
    arity: usize,
    /// The atom of each tuple; [`AtomId::MAX`] for a control predicate's,
    /// which has none ([`Grounder::atom_id`]).
    ids: Vec<AtomId>,
    /// The tuples by the hash of their values ([`Relation::hash`]), but
    /// for a relation of arity zero, whose one tuple is `ids[0]` once
    /// derived ([`Relation::get`]): every switch of a ground program is
    /// one, and needs no table allocated. `by_hash` holds the last tuple
    /// of each hash, and `same_hash` before each tuple the one of its hash
    /// before it, or [`Relation::NONE`]. A table of tuples by their values
    /// held a copy of each: a gigabyte for TPC-H's six million line items.
    by_hash: FxHashMap<u64, u32>,
    same_hash: Vec<u32>,
    /// Atoms named by a negative literal inside their own recursive
    /// component before it is known whether they are possible, and their
    /// atoms. Derived, a tuple keeps its placeholder's atom; otherwise it
    /// ends up a derived atom without rules, which the simplifier treats as
    /// false.
    placeholders: Interned<Val>,
    placeholder_ids: Vec<AtomId>,
    /// `index[pos][val]` = tuple indices with that value at `pos`, for each
    /// position a join has bound, built when it is first bound: a join
    /// binds few of a wide table's positions (TPC-H's line items have 13).
    index: Vec<FxHashMap<Val, Vec<usize>>>,
    /// Number of tuples already indexed, by position.
    indexed: Vec<usize>,
}

impl Relation {
    /// No tuple, in [`Relation::same_hash`].
    const NONE: u32 = u32::MAX;

    fn new(arity: usize) -> Relation {
        Relation {
            vals: Vec::new(),
            arity,
            ids: Vec::new(),
            by_hash: FxHashMap::default(),
            same_hash: Vec::new(),
            placeholders: Interned::default(),
            placeholder_ids: Vec::new(),
            index: Vec::new(),
            indexed: Vec::new(),
        }
    }

    /// The hash under which a tuple of `args` is kept.
    fn hash(args: &[Val]) -> u64 {
        rustc_hash::FxBuildHasher.hash_one(args)
    }

    /// The number of tuples.
    fn len(&self) -> usize {
        self.ids.len()
    }

    /// Tuple `t`.
    fn tuple(&self, t: usize) -> &[Val] {
        &self.vals[t * self.arity..(t + 1) * self.arity]
    }

    /// The atom of a derived tuple.
    fn get(&self, args: &[Val]) -> Option<AtomId> {
        self.find(Self::hash(args), args)
    }

    /// The atom of a derived tuple whose hash is `h`.
    fn find(&self, h: u64, args: &[Val]) -> Option<AtomId> {
        if args.is_empty() {
            return self.ids.first().copied();
        }
        let mut t = *self.by_hash.get(&h)?;
        while t != Self::NONE {
            if self.tuple(t as usize) == args {
                return Some(self.ids[t as usize]);
            }
            t = self.same_hash[t as usize];
        }
        None
    }

    /// Room for `n` more tuples.
    fn reserve(&mut self, n: usize) -> bool {
        self.vals.try_reserve(n * self.arity).is_ok()
            && self.ids.try_reserve(n).is_ok()
            && (self.arity == 0
                || self.same_hash.try_reserve(n).is_ok() && self.by_hash.try_reserve(n).is_ok())
    }

    /// Adds the tuple `args`, whose hash is `h`, as atom `id`.
    fn push(&mut self, h: u64, args: &[Val], id: AtomId) {
        debug_assert_eq!(args.len(), self.arity, "a tuple of the relation's arity");
        if self.arity > 0 {
            let t = self.len() as u32;
            let before = self.by_hash.insert(h, t);
            self.same_hash.push(before.unwrap_or(Self::NONE));
        }
        self.vals.extend_from_slice(args);
        self.ids.push(id);
    }

    /// Indexes the tuples not yet indexed at position `pos`.
    fn ensure_index(&mut self, pos: usize) {
        if self.index.is_empty() {
            self.index = vec![FxHashMap::default(); self.arity];
            self.indexed = vec![0; self.arity];
        }
        for t in self.indexed[pos]..self.len() {
            self.index[pos]
                .entry(self.vals[t * self.arity + pos])
                .or_default()
                .push(t);
        }
        self.indexed[pos] = self.len();
    }

    /// The tuples among `s..e` that may have value `v` at each position
    /// `pos` of `bound`, in tuple order: those of the shortest posting list
    /// of `bound`, or all of `s..e` where nothing is bound. The caller
    /// checks the other positions. A posting list is in tuple order, so the
    /// range is a slice of it.
    fn candidates(&mut self, bound: &[(usize, Val)], (s, e): (usize, usize)) -> Vec<usize> {
        if bound.is_empty() {
            return (s..e).collect();
        }
        for &(pos, _) in bound {
            self.ensure_index(pos);
        }
        let index = &self.index;
        bound
            .iter()
            .map(|(pos, v)| {
                let list = index[*pos].get(v).map_or(&[][..], |l| &l[..]);
                &list[list.partition_point(|&t| t < s)..list.partition_point(|&t| t < e)]
            })
            .min_by_key(|l| l.len())
            .expect("a bound position")
            .to_vec()
    }
}

struct Pred<'a> {
    name: &'a Name,
    arity: usize,
    switch: Option<Weights>,
    /// The facts of a predicate given by ground facts alone, its atoms.
    facts: Option<&'a Facts>,
    /// A magic-set control predicate: grounded, never emitted.
    control: bool,
    /// A predicate a findall reads, whose derivations are counted.
    counted: bool,
    rules: Vec<usize>,
}

/// The number of proofs of an atom, up to two, and the order of its one
/// proof among the solutions of a findall ([`proof`]).
type Proof = (u8, Rc<[u32]>);

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
    /// Each atom's predicate and where its arguments are; a control
    /// predicate's tuples have no atoms ([`Grounder::atom_id`]).
    atoms: Vec<(PredId, Place)>,
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
    /// The clauses of the rules, by rule, and each rule as a semi-naive
    /// round may walk it from one of its atoms ([`from_delta`]), by rule
    /// and atom.
    clauses: &'a [Clause],
    from_delta: FxHashMap<(usize, usize), FromDelta>,
    /// The size of `ground_rules`, a term per head and body literal, and
    /// the size past which the grounding is given up.
    size: usize,
    bound: usize,
    /// Whether the allocator refused a block the atoms or the rules grew
    /// by ([`Grounder::room`]), which gives the grounding up as the bound
    /// does.
    refused: bool,
    /// The control atoms derived, which the bound limits too: they are
    /// never emitted, and a demand that no rule's size follows can grow
    /// without end. Grounded from the bound end, `o(T1) :- step(T),
    /// T1 is T + 1, o(T)` asks `o(T+1)` of every `o(T)` it is asked,
    /// `step(T)` coming after the call.
    demands: usize,
    /// The values that grounding level by level proved, by atom
    /// ([`evaluate_by_levels`]).
    fixed: Vec<Option<bool>>,
    /// The atoms true in every world that a rule derived ([`ground_body`]).
    certain: Vec<bool>,
    /// The weights of the atoms of [`Facts`], by atom ([`load`]).
    weights: Vec<Option<Prob>>,
    /// Under a demand, the atoms of [`Facts`] no rule has read yet, by
    /// atom, those read, in the order of their first reads, and where
    /// each round of evaluation began among them
    /// ([`facts_as_read`]).
    unread: Vec<bool>,
    read: Vec<AtomId>,
    read_rounds: Vec<usize>,
    /// The stratum being grounded level by level.
    by_level: Option<ByLevel>,
    /// The ground compound terms, and the symbols `[]` and `'[|]'` once
    /// a list is read ([`Grounder::list_syms`]).
    terms: Terms,
    lists: Option<(u32, u32)>,
    /// What gave the grounding up, but for the bound and the allocator.
    error: Option<GroundError>,
    /// The derivations of the atoms of counted predicates, by head; the
    /// number of the rule's walk ([`eval_rule`]); what each body literal
    /// of a counted rule contributes, by its place in the walk; and the
    /// proofs counted ([`proof`]).
    derivations: FxHashMap<AtomId, Vec<Derivation>>,
    stamp: u32,
    parts: Vec<Part>,
    proofs: FxHashMap<AtomId, Proof>,
}

/// Where the arguments of an atom are: a tuple of its predicate's relation,
/// or a placeholder there ([`Relation::placeholders`]) until the tuple is
/// derived.
#[derive(Clone, Copy, Debug)]
enum Place {
    Tuple(u32),
    Placeholder(u32),
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
        if self.room(|g| g.ground_rules.try_reserve(1).is_ok())
            && self.kept.keep(&self.ground_rules, &rule)
        {
            self.size += 1 + rule.body.len();
            self.ground_rules.push(rule);
        }
    }

    /// Whether the rules kept so far, or the control atoms, are past the
    /// bound, or the allocator refused the grounding more memory.
    fn over(&self) -> bool {
        self.refused || self.size > self.bound || self.demands > self.bound || self.error.is_some()
    }

    /// Gives the grounding up with `e`, unless it is given up already.
    fn fail(&mut self, e: GroundError) {
        self.error.get_or_insert(e);
    }

    /// Whether `reserve` made room for one more of the vectors and tables
    /// the grounding grows by, the atoms and the rules, which double as
    /// they grow; where the allocator refuses a block, the grounding is
    /// given up ([`GroundError::OutOfMemory`]) rather than aborted, as a
    /// diagram operation is refused. A taint analysis of PSouffle's grew
    /// its atoms to 2 GiB and aborted there (taint_video_game, under 12 GB).
    fn room(&mut self, reserve: impl FnOnce(&mut Self) -> bool) -> bool {
        self.refused = self.refused || !reserve(self);
        !self.refused
    }

    /// A known predicate, and whether it is a switch.
    fn pred_id(&self, name: &str, arity: usize) -> Option<(PredId, bool)> {
        self.pred_ids.get(&(name, arity)).copied()
    }

    fn pred(&mut self, name: &'a Name, arity: usize, switch: Option<Weights>) -> PredId {
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
                facts: None,
                control: false,
                counted: false,
                rules: Vec::new(),
            });
            self.rels.push(Relation::new(arity));
        }
        id
    }

    /// The value of the ground term `t`, its compound terms interned from
    /// the inside out, a list's spine in a loop.
    fn val(&mut self, t: &Term) -> Val {
        match t {
            Term::Const(s) => Val::Sym(self.syms.intern(s)),
            Term::Int(i) => Val::Int(*i),
            Term::Float(x) => Val::Float(x.0.to_bits()),
            Term::Var(_) => unreachable!("variables are not values"),
            Term::Compound(_) => {
                let mut work: Vec<(&Term, bool)> = vec![(t, false)];
                let mut vals: Vec<Val> = Vec::new();
                while let Some((t, done)) = work.pop() {
                    match t {
                        Term::Compound(c) if !done => {
                            work.push((t, true));
                            work.extend(c.args.iter().rev().map(|a| (a, false)));
                        }
                        Term::Compound(c) => {
                            let at = vals.len() - c.args.len();
                            let name = self.syms.intern(&c.name);
                            let v = self.intern_fn(name, &vals[at..], false);
                            vals.truncate(at);
                            vals.push(v);
                        }
                        other => vals.push(self.val(other)),
                    }
                }
                vals[0]
            }
        }
    }

    /// The term of a value, its compound terms built from their keys once,
    /// from the inside out, a list's spine in a loop.
    fn term_of(&mut self, v: Val) -> Term {
        let Val::Fn(id) = v else {
            return self.leaf_term(v);
        };
        let mut work: Vec<(u32, bool)> = vec![(id, false)];
        while let Some((id, ready)) = work.pop() {
            if self.terms.built.contains_key(&id) {
                continue;
            }
            let key = self.terms.keys.list(id as usize);
            if !ready {
                work.push((id, true));
                work.extend(key[1..].iter().rev().filter_map(|a| match a {
                    Val::Fn(a) => Some((*a, false)),
                    _ => None,
                }));
                continue;
            }
            let Val::Sym(name) = key[0] else {
                unreachable!("a term's key starts with its name")
            };
            let args = key[1..]
                .iter()
                .map(|&a| match a {
                    Val::Fn(a) => self.terms.built[&a].clone(),
                    a => self.leaf_term(a),
                })
                .collect();
            let t = Compound::term(self.syms.names[name as usize].clone(), args);
            self.terms.built.insert(id, t);
        }
        self.terms.built[&id].clone()
    }

    /// The term of a value that is no compound term.
    fn leaf_term(&self, v: Val) -> Term {
        match v {
            Val::Sym(s) => Term::Const(self.syms.names[s as usize].clone()),
            Val::Int(i) => Term::Int(i),
            Val::Float(b) => Term::Float(F64(f64::from_bits(b))),
            Val::Fn(_) => unreachable!("a compound term is built from its key"),
        }
    }

    /// The compound term `name(args)`, interned. A term a rule builds
    /// (`built`) is one deeper than the deepest built term among its
    /// arguments, and one deeper than [`MAX_DEPTH`] gives the grounding
    /// up: a recursion that builds ever deeper terms does not end. Where
    /// the allocator refuses the terms room, the grounding is given up
    /// ([`Grounder::room`]) and the value returned is read no more: the
    /// terms of a grammar's demands grow by blocks of gigabytes.
    fn intern_fn(&mut self, name: u32, args: &[Val], built: bool) -> Val {
        const ON_STACK: usize = 8;
        let mut on_stack = [Val::Int(0); ON_STACK + 1];
        let heap: Vec<Val>;
        let key: &[Val] = if args.len() <= ON_STACK {
            on_stack[0] = Val::Sym(name);
            on_stack[1..=args.len()].copy_from_slice(args);
            &on_stack[..=args.len()]
        } else {
            heap = std::iter::once(Val::Sym(name))
                .chain(args.iter().copied())
                .collect();
            &heap
        };
        if !self.room(|g| {
            g.terms.keys.try_reserve(1, key.len()) && g.terms.depths.try_reserve(1).is_ok()
        }) {
            return Val::Sym(name);
        }
        let (id, new) = self.terms.keys.id(key);
        if !new {
            return Val::Fn(id);
        }
        let depth = if built {
            1 + args
                .iter()
                .map(|a| match a {
                    Val::Fn(i) => self.terms.depths[*i as usize],
                    _ => 0,
                })
                .max()
                .unwrap_or(0)
        } else {
            0
        };
        if depth > MAX_DEPTH {
            let name = &self.syms.names[name as usize];
            self.fail(GroundError::Unbounded(format!(
                "a rule nests {name}/{} without end",
                args.len()
            )));
        }
        self.terms.depths.push(depth);
        Val::Fn(id)
    }

    /// The symbols `[]` and `'[|]'`.
    fn list_syms(&mut self) -> (u32, u32) {
        *self.lists.get_or_insert_with(|| {
            (
                self.syms.intern(&Name::from(NIL)),
                self.syms.intern(&Name::from(CONS)),
            )
        })
    }

    /// The items of the list `v`, and its tail after each number of items:
    /// `tails[k]` follows the first `k`, and the last is `[]` where the
    /// list is proper.
    fn spine(&self, v: Val, cons: u32) -> (Vec<Val>, Vec<Val>) {
        let (mut items, mut tails) = (Vec::new(), vec![v]);
        let mut v = v;
        while let Val::Fn(id) = v {
            let key = self.terms.keys.list(id as usize);
            if key.len() != 3 || key[0] != Val::Sym(cons) {
                break;
            }
            items.push(key[1]);
            v = key[2];
            tails.push(v);
        }
        (items, tails)
    }

    /// The items of the proper list `v`; `None` where it is none.
    fn items(&mut self, v: Val) -> Option<Vec<Val>> {
        let (nil, cons) = self.list_syms();
        let (items, tails) = self.spine(v, cons);
        (tails.last() == Some(&Val::Sym(nil))).then_some(items)
    }

    /// The list of `items` followed by `tail`.
    fn make_list(&mut self, items: &[Val], tail: Val) -> Val {
        let (_, cons) = self.list_syms();
        items
            .iter()
            .rev()
            .fold(tail, |acc, &x| self.intern_fn(cons, &[x, acc], false))
    }

    /// The number a value evaluates to: a number, or a compound term of
    /// arithmetic functions over numbers, evaluated from the inside out.
    fn num(&self, v: Val) -> Option<Num> {
        enum Step {
            Visit(Val),
            Apply(ArithFn, usize),
        }
        let leaf = |v: Val| match v {
            Val::Int(i) => Some(Num::I(i)),
            Val::Float(b) => Some(Num::F(f64::from_bits(b))),
            Val::Sym(_) | Val::Fn(_) => None,
        };
        if !matches!(v, Val::Fn(_)) {
            return leaf(v);
        }
        let mut work = vec![Step::Visit(v)];
        let mut nums: Vec<Num> = Vec::new();
        while let Some(step) = work.pop() {
            match step {
                Step::Visit(Val::Fn(id)) => {
                    let key = self.terms.keys.list(id as usize);
                    let Val::Sym(name) = key[0] else {
                        unreachable!("a term's key starts with its name")
                    };
                    let f = ArithFn::of(&self.syms.names[name as usize], key.len() - 1)?;
                    work.push(Step::Apply(f, key.len() - 1));
                    work.extend(key[1..].iter().rev().map(|&a| Step::Visit(a)));
                }
                Step::Visit(v) => nums.push(leaf(v)?),
                Step::Apply(f, n) => {
                    let at = nums.len() - n;
                    let r = f.apply(&nums[at..])?;
                    nums.truncate(at);
                    nums.push(r);
                }
            }
        }
        nums.pop()
    }

    /// The atom of the tuple `args` of `pred`, derived now if it is new,
    /// and whether it is. A control predicate's tuples have no atoms
    /// ([`AtomId::MAX`]): they are never emitted, and are read only to
    /// drive the grounding, where a grammar's demands number tens of
    /// millions.
    fn atom_id(&mut self, pred: PredId, args: &[Val]) -> (AtomId, bool) {
        let h = Relation::hash(args);
        if let Some(id) = self.rels[pred as usize].find(h, args) {
            return (id, false);
        }
        // Past a refused block the grounding is given up, and nothing it
        // keeps reads the id (`keep_rule`, `over`).
        if !self.room(|g| g.atoms.try_reserve(1).is_ok() && g.rels[pred as usize].reserve(1)) {
            return (AtomId::MAX, false);
        }
        let control = self.preds[pred as usize].control;
        let rel = &mut self.rels[pred as usize];
        let t = rel.len() as u32;
        let placeholder = match rel.placeholders.len() {
            0 => None,
            _ => rel.placeholders.get(args),
        };
        let id = match placeholder {
            Some(k) => {
                let id = rel.placeholder_ids[k as usize];
                self.atoms[id as usize].1 = Place::Tuple(t);
                id
            }
            None if control => AtomId::MAX,
            None => {
                self.atoms.push((pred, Place::Tuple(t)));
                (self.atoms.len() - 1) as AtomId
            }
        };
        self.rels[pred as usize].push(h, args, id);
        (id, true)
    }

    /// The id of an atom that may or may not turn out possible.
    fn placeholder_atom(&mut self, pred: PredId, args: &[Val]) -> AtomId {
        let rel = &self.rels[pred as usize];
        if let Some(id) = rel.get(args) {
            return id;
        }
        if let Some(k) = rel.placeholders.get(args) {
            return rel.placeholder_ids[k as usize];
        }
        if !self.room(|g| {
            let rel = &mut g.rels[pred as usize];
            g.atoms.try_reserve(1).is_ok()
                && rel.placeholder_ids.try_reserve(1).is_ok()
                && rel.placeholders.try_reserve(1, args.len())
        }) {
            return AtomId::MAX;
        }
        let id = self.atoms.len() as AtomId;
        let rel = &mut self.rels[pred as usize];
        let (k, _) = rel.placeholders.id(args);
        rel.placeholder_ids.push(id);
        self.atoms.push((pred, Place::Placeholder(k)));
        id
    }

    /// The predicate and the arguments of atom `id`.
    fn args_of(&self, id: AtomId) -> (PredId, &[Val]) {
        let (p, place) = self.atoms[id as usize];
        let rel = &self.rels[p as usize];
        let args = match place {
            Place::Tuple(t) => rel.tuple(t as usize),
            Place::Placeholder(k) => rel.placeholders.list(k as usize),
        };
        (p, args)
    }

    /// Whether atom `id` is derived (possible), not only a placeholder.
    fn derived(&self, id: AtomId) -> bool {
        matches!(self.atoms[id as usize].1, Place::Tuple(_))
    }

    fn lookup_atom(&self, pred: PredId, args: &[Val]) -> Option<AtomId> {
        self.rels[pred as usize].get(args)
    }

    fn fixed(&self, id: AtomId) -> Option<bool> {
        self.fixed.get(id as usize).copied().flatten()
    }

    /// Marks a round of evaluation begun among the first reads of facts
    /// under a demand ([`facts_as_read`]).
    fn new_round(&mut self) {
        if !self.unread.is_empty() {
            self.read_rounds.push(self.read.len());
        }
    }

    /// Whether atom `id` is true in every world by a rule whose body
    /// atoms are ([`ground_body`]).
    fn certain(&self, id: AtomId) -> bool {
        self.certain.get(id as usize).copied().unwrap_or(false)
    }

    /// The key of an atom of the stratum grounded level by level.
    fn key_of(&self, id: AtomId) -> Option<Key> {
        let (p, args) = self.args_of(id);
        let r = *self.by_level.as_ref()?.rank.get(&p)?;
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

    fn display_atom(&mut self, id: AtomId) -> Atom {
        let (p, args) = self.args_of(id);
        let args = args.to_vec();
        Atom::new(
            self.preds[p as usize].name.clone(),
            args.into_iter().map(|v| self.term_of(v)).collect::<Args>(),
        )
    }
}

/// Compile the clauses to internal form, checking safety; a `counted`
/// rule records its derivations ([`Derivation`]).
fn compile_rule<'a>(
    g: &mut Grounder<'a>,
    clause: &'a Clause,
    counted: bool,
) -> Result<IRule, GroundError> {
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
    let ordered = schedule_body(
        &clause.body,
        &mut bound_names,
        |i| known[i].is_some_and(|(_, switch)| switch),
        Order::Join,
    )
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
            // `V = f(A..)`, its arguments flat ([`super::terms`]).
            Literal::Cmp(CmpOp::Eq, x, y)
                if x.is_var() && !y.is_ground() && y.as_compound().is_some()
                    || y.is_var() && !x.is_ground() && x.as_compound().is_some() =>
            {
                let (v, t) = if x.is_var() { (x, y) } else { (y, x) };
                let (name, args) = t.as_compound().expect("a compound side");
                let target = var_of(v, &mut vars, g);
                let name = g.syms.intern(name);
                let args = args.iter().map(|a| var_of(a, &mut vars, g)).collect();
                body.push(ILit::Build(target, name, args));
            }
            Literal::Cmp(op, a, b) if op.arithmetic() => {
                let a = compile_expr(a, &mut vars, g, &var_of)?;
                let b = compile_expr(b, &mut vars, g, &var_of)?;
                body.push(ILit::Cmp(*op, a, b));
            }
            Literal::Cmp(op, a, b) => {
                let a = IExpr::Arg(var_of(a, &mut vars, g));
                let b = IExpr::Arg(var_of(b, &mut vars, g));
                body.push(ILit::Cmp(*op, a, b));
            }
            Literal::Is(v, e) => {
                if !v.is_ground() && !v.is_var() {
                    return Err(GroundError::Unsupported(format!(
                        "{clause}: the left side of is/2 is a compound term"
                    )));
                }
                let v = var_of(v, &mut vars, g);
                let e = compile_expr(e, &mut vars, g, &var_of)?;
                body.push(ILit::Is(v, e));
            }
            Literal::Builtin(b, args, negated) => {
                let args = args.iter().map(|a| var_of(a, &mut vars, g)).collect();
                body.push(ILit::Builtin(*b, args, *negated));
            }
            Literal::Collect(c) => {
                let aux = g.pred(&c.aux, c.arity, None);
                let shared = c.shared.iter().map(|a| var_of(a, &mut vars, g)).collect();
                let result = var_of(&c.result, &mut vars, g);
                body.push(ILit::Collect(Box::new(ICollect {
                    aux,
                    shared,
                    result,
                    text: c.text.clone(),
                })));
            }
            Literal::Findall(_) => {
                return Err(GroundError::Unsupported(format!(
                    "{clause}: a findall reached the grounder unlowered"
                )));
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
    let counted = counted.then(|| {
        g.preds[head_pred as usize].counted = true;
        let mut order: Vec<usize> = (0..ordered.len()).collect();
        order.sort_by_key(|&k| ordered[k]);
        Box::new(Counted {
            group: clause.group,
            order,
        })
    });
    Ok(IRule {
        head: (head_pred, head_args),
        body,
        num_vars: vars.len() as u32,
        counted,
    })
}

/// The arithmetic expression `t`: its compound terms are functions.
fn compile_expr<'a>(
    t: &Term,
    vars: &mut FxHashMap<String, u32>,
    g: &mut Grounder<'a>,
    var_of: &impl Fn(&Term, &mut FxHashMap<String, u32>, &mut Grounder<'a>) -> Arg,
) -> Result<IExpr, GroundError> {
    let Some((name, args)) = t.as_compound() else {
        return Ok(IExpr::Arg(var_of(t, vars, g)));
    };
    let f = ArithFn::of(name, args.len()).ok_or_else(|| {
        GroundError::Unsupported(format!(
            "{name}/{} is not an arithmetic function, in {t}",
            args.len()
        ))
    })?;
    let args = args
        .iter()
        .map(|a| compile_expr(a, vars, g, var_of))
        .collect::<Result<_, _>>()?;
    Ok(IExpr::Op(f, args))
}

/// The value of `a` under `subst`; `None` where it is an unbound variable.
fn arg_val(a: &Arg, subst: &[Option<Val>]) -> Option<Val> {
    match a {
        Arg::Var(v) => subst[*v as usize],
        Arg::Val(v) => Some(*v),
    }
}

/// The number `e` evaluates to under `subst`; `None` where a variable is
/// unbound, an operand is no number, or the function is undefined there.
fn eval_num(g: &Grounder, e: &IExpr, subst: &[Option<Val>]) -> Option<Num> {
    match e {
        IExpr::Arg(a) => g.num(arg_val(a, subst)?),
        IExpr::Op(f, args) => {
            let mut xs = [Num::I(0); 2];
            for (x, a) in xs.iter_mut().zip(args) {
                *x = eval_num(g, a, subst)?;
            }
            f.apply(&xs[..args.len()])
        }
    }
}

/// A side of a comparison: a term, or the number it evaluates to where the
/// comparison is arithmetic and it is a compound term or an expression.
fn side(g: &Grounder, e: &IExpr, subst: &[Option<Val>], arithmetic: bool) -> Option<Val> {
    match e {
        IExpr::Arg(a) => {
            let v = arg_val(a, subst)?;
            if arithmetic && matches!(v, Val::Fn(_)) {
                g.num(v).map(Num::val)
            } else {
                Some(v)
            }
        }
        IExpr::Op(..) => eval_num(g, e, subst).map(Num::val),
    }
}

/// Whether `x op y` holds of two values. Two terms are equal where their
/// values are, every ground term being interned once.
fn compare(g: &mut Grounder, op: CmpOp, x: Val, y: Val) -> bool {
    match op {
        CmpOp::Eq | CmpOp::Identical => x == y,
        CmpOp::Ne | CmpOp::NotIdentical => x != y,
        _ => {
            let (a, b) = (g.term_of(x), g.term_of(y));
            op.holds(&a, &b)
        }
    }
}

/// Binds the unbound variables of `args` to `vals`, recording them in
/// `bound`; false where a bound one, or a constant, differs.
fn unify_args(args: &[Arg], vals: &[Val], subst: &mut [Option<Val>], bound: &mut Vec<u32>) -> bool {
    for (a, &v) in args.iter().zip(vals) {
        match a {
            Arg::Val(c) if *c != v => return false,
            Arg::Val(_) => {}
            Arg::Var(x) => match subst[*x as usize] {
                None => {
                    subst[*x as usize] = Some(v);
                    bound.push(*x);
                }
                Some(b) if b != v => return false,
                Some(_) => {}
            },
        }
    }
    true
}

/// The argument tuples of the solutions of a built-in under the values
/// bound of its arguments, in the order ProbLog's library enumerates them;
/// the caller unifies each with the arguments. `between/3` enumerating its
/// range is the caller's ([`ground_body`]).
fn solutions(g: &mut Grounder, b: Builtin, vals: &[Option<Val>]) -> Vec<Vec<Val>> {
    let (nil, cons) = g.list_syms();
    let nil = Val::Sym(nil);
    let one = |sol: Vec<Val>| vec![sol];
    match b {
        Builtin::Member => {
            let l = vals[1].expect("mode");
            let (items, _) = g.spine(l, cons);
            items.into_iter().map(|x| vec![x, l]).collect()
        }
        Builtin::Memberchk => {
            let l = vals[1].expect("mode");
            let (items, _) = g.spine(l, cons);
            let first = match vals[0] {
                Some(x) => items.into_iter().find(|&y| y == x),
                None => items.first().copied(),
            };
            first.map_or_else(Vec::new, |x| one(vec![x, l]))
        }
        Builtin::Append => match (vals[0], vals[1], vals[2]) {
            (Some(a), Some(b), _) => match g.items(a) {
                Some(items) => {
                    let c = g.make_list(&items, b);
                    one(vec![a, b, c])
                }
                None => Vec::new(),
            },
            (a, b, Some(c)) => {
                let (items, tails) = g.spine(c, cons);
                // The splits a bound prefix or suffix names, or all of them.
                let splits: Vec<usize> = match (a, b) {
                    (Some(a), _) => match g.items(a) {
                        Some(p) if items.starts_with(&p) => vec![p.len()],
                        _ => Vec::new(),
                    },
                    (None, Some(b)) => tails.iter().position(|&t| t == b).into_iter().collect(),
                    (None, None) => (0..tails.len()).collect(),
                };
                splits
                    .into_iter()
                    .map(|k| vec![g.make_list(&items[..k], nil), tails[k], c])
                    .collect()
            }
            _ => unreachable!("append/3's mode"),
        },
        Builtin::Length => {
            let l = vals[0].expect("mode");
            g.items(l)
                .map_or_else(Vec::new, |items| one(vec![l, Val::Int(items.len() as i64)]))
        }
        Builtin::Nth0 | Builtin::Nth1 => {
            let base = i64::from(b == Builtin::Nth1);
            let l = vals[1].expect("mode");
            let (items, _) = g.spine(l, cons);
            match vals[0] {
                Some(Val::Int(i)) => usize::try_from(i - base)
                    .ok()
                    .and_then(|k| items.get(k))
                    .map_or_else(Vec::new, |&x| one(vec![Val::Int(i), l, x])),
                Some(_) => Vec::new(),
                None => items
                    .into_iter()
                    .enumerate()
                    .map(|(k, x)| vec![Val::Int(k as i64 + base), l, x])
                    .collect(),
            }
        }
        Builtin::Last => {
            let l = vals[0].expect("mode");
            match g.items(l).and_then(|items| items.last().copied()) {
                Some(x) => one(vec![l, x]),
                None => Vec::new(),
            }
        }
        Builtin::Reverse => {
            let l = vals[0].expect("mode");
            match g.items(l) {
                Some(mut items) => {
                    items.reverse();
                    let r = g.make_list(&items, nil);
                    one(vec![l, r])
                }
                None => Vec::new(),
            }
        }
        // `B is A + X` from `A = 0`, left to right.
        Builtin::SumList => {
            let l = vals[0].expect("mode");
            let sum = g.items(l).and_then(|items| {
                items.iter().try_fold(Num::I(0), |acc, &x| {
                    ArithFn::Add.apply(&[acc, g.num(x)?]).map(Num::rounded)
                })
            });
            sum.map_or_else(Vec::new, |s| one(vec![l, s.val()]))
        }
        // The element itself, the later of equal ones: `M > X` keeps `M`
        // and `M =< X` takes `X`.
        Builtin::MaxList | Builtin::MinList => {
            let l = vals[0].expect("mode");
            let keeps = if b == Builtin::MaxList {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            };
            let best = g.items(l).and_then(|items| {
                let (&first, rest) = items.split_first()?;
                rest.iter().try_fold(first, |m, &x| {
                    let (a, b) = (g.num(m)?, g.num(x)?);
                    let ord = match (a, b) {
                        (Num::I(a), Num::I(b)) => a.cmp(&b),
                        _ => a.float().partial_cmp(&b.float())?,
                    };
                    Some(if ord == keeps { m } else { x })
                })
            });
            best.map_or_else(Vec::new, |m| one(vec![l, m]))
        }
        // The items in the standard order of terms, equal ones once.
        Builtin::Sort => {
            let l = vals[0].expect("mode");
            let (mut items, _) = g.spine(l, cons);
            items.sort_by(|&x, &y| {
                let (a, b) = (g.term_of(x), g.term_of(y));
                term_cmp(&a, &b)
            });
            items.dedup();
            let s = g.make_list(&items, nil);
            one(vec![l, s])
        }
        Builtin::Between => match (vals[0], vals[1], vals[2]) {
            (Some(Val::Int(lo)), Some(Val::Int(hi)), Some(x)) => match x {
                Val::Int(x) if lo <= x && x <= hi => {
                    one(vec![Val::Int(lo), Val::Int(hi), Val::Int(x)])
                }
                _ => Vec::new(),
            },
            _ => Vec::new(),
        },
    }
}

/// The list of the template values of the solutions of a findall's goal
/// whose shared variables have the values `shared`, in the order ProbLog
/// lists them; `None` where it gives the grounding up, a solution read with
/// more than one proof or a proof not counted ([`proof`]).
fn collect(g: &mut Grounder, c: &ICollect, shared: &[Val]) -> Option<Val> {
    let k = shared.len();
    let bound: Vec<(usize, Val)> = shared.iter().copied().enumerate().collect();
    let candidates = {
        let rel = &mut g.rels[c.aux as usize];
        let all = (0, rel.len());
        rel.candidates(&bound, all)
    };
    let mut found: Vec<(Rc<[u32]>, Val)> = Vec::with_capacity(candidates.len());
    for t in candidates {
        let rel = &g.rels[c.aux as usize];
        if rel.tuple(t)[..k] != *shared {
            continue;
        }
        let (id, v) = (rel.ids[t], rel.tuple(t)[k]);
        match proof(g, id) {
            Ok((0, _)) => {}
            Ok((1, key)) => found.push((key, v)),
            Ok(_) => {
                let e = format!(
                    "{}: the solution {} has more than one proof, and tddlog lists a solution \
                     only where it has one",
                    c.text,
                    g.term_of(v)
                );
                g.fail(GroundError::Refused(e));
                return None;
            }
            Err(why) => {
                g.fail(GroundError::Refused(format!("{}: {why}", c.text)));
                return None;
            }
        }
    }
    found.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let items: Vec<Val> = found.into_iter().map(|(_, v)| v).collect();
    let (nil, _) = g.list_syms();
    Some(g.make_list(&items, Val::Sym(nil)))
}

/// The number of proofs of `id`, up to two, and the order of its proof
/// where it has one: its clause's group, then what each body literal reads
/// in the clause's order, which sorts proofs as SLD resolution finds them.
/// A certain switch has one proof and an impossible one none; any other
/// is refused, as is an atom whose derivations are not counted.
fn proof(g: &mut Grounder, id: AtomId) -> Result<Proof, String> {
    if let Some(p) = g.proofs.get(&id) {
        return Ok(p.clone());
    }
    let pred = &g.preds[g.atoms[id as usize].0 as usize];
    // A switch's weights where every atom has the same.
    let fixed = |w: &Weights| match w {
        Weights::Fixed(q) => Some(*q),
        Weights::Flexible { .. } => None,
    };
    let (switch, counted) = (pred.switch.as_ref().map(fixed), pred.counted);
    let p = match switch {
        Some(Some(q)) if q == Prob::TRUE => (1, Rc::from([])),
        Some(Some(q)) if q == Prob::FALSE => (0, Rc::from([])),
        Some(_) => {
            return Err(format!(
                "its goal reads the probabilistic fact {}, and tddlog's findall reads \
                 deterministic goals only",
                g.display_atom(id)
            ));
        }
        None if !counted => {
            return Err(format!(
                "the proofs of {} are not counted",
                g.display_atom(id)
            ));
        }
        None => {
            let derivations = g.derivations.remove(&id).unwrap_or_default();
            let (mut count, mut first) = (0u8, None);
            for d in &derivations {
                let mut n = d.mult;
                let mut key = vec![d.group];
                for part in &d.parts {
                    if n == 0 {
                        break;
                    }
                    match *part {
                        Part::Atom(a) => {
                            let (m, k) = proof(g, a)?;
                            n = n.saturating_mul(m).min(2);
                            key.extend_from_slice(&k);
                        }
                        Part::Neg(a) => {
                            if proof(g, a)?.0 > 0 {
                                n = 0;
                            }
                        }
                        Part::Rank(r) => key.push(r),
                        Part::None => {}
                    }
                }
                if n > 0 {
                    count = count.saturating_add(n).min(2);
                    first.get_or_insert(key);
                }
                if count == 2 {
                    break;
                }
            }
            (count, Rc::from(first.unwrap_or_default()))
        }
    };
    g.proofs.insert(id, p.clone());
    Ok(p)
}

/// The weights of the switch atom `id` of head `index` of `clause`, whose
/// probabilities are flexible: the atom's last `heads` arguments are the
/// heads' probabilities in this instance of the clause
/// ([`Weights::Flexible`]). A probability that is not a number in [0, 1],
/// or that passes one with the others, is an error, as in ProbLog.
fn flexible_weights(
    g: &Grounder,
    id: AtomId,
    index: usize,
    heads: usize,
    clause: &str,
) -> Result<Prob, GroundError> {
    let (_, args) = g.args_of(id);
    let mut masses = Vec::with_capacity(heads);
    for &v in &args[args.len() - heads..] {
        let p = g.num(v).map(Num::float).ok_or_else(|| {
            GroundError::Unsupported(format!(
                "a probability that is not a number, in an instance of {clause}"
            ))
        })?;
        if !(0.0..=1.0 + 1e-12).contains(&p) {
            return Err(GroundError::Unsupported(format!(
                "probability {p} out of range, in an instance of {clause}"
            )));
        }
        masses.push(p);
    }
    let mut qs = Vec::with_capacity(heads);
    switch_weights(&masses, &mut qs).map_err(|total| {
        GroundError::Unsupported(format!(
            "annotated disjunction probabilities sum to {total} > 1, in an instance of {clause}"
        ))
    })?;
    Ok(qs[index])
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
            match l {
                ILit::Pos(p, _) | ILit::Neg(p, _) => deps[h as usize].push(*p),
                ILit::Collect(c) => deps[h as usize].push(c.aux),
                _ => {}
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
        clauses: &norm.rules,
        from_delta: FxHashMap::default(),
        size: 0,
        bound: bound.unwrap_or(usize::MAX),
        refused: false,
        demands: 0,
        fixed: Vec::new(),
        certain: Vec::new(),
        weights: Vec::new(),
        unread: Vec::new(),
        read: Vec::new(),
        read_rounds: Vec::new(),
        by_level: None,
        terms: Terms::default(),
        lists: None,
        error: None,
        derivations: FxHashMap::default(),
        stamp: 0,
        parts: Vec::new(),
        proofs: FxHashMap::default(),
    };
    for (&(ref name, arity), w) in &norm.switches {
        g.pred(name, arity, Some(w.clone()));
    }
    for (name, arity) in &norm.control {
        let p = g.pred(name, *arity, None);
        g.preds[p as usize].control = true;
    }
    // The predicates numbered as the rules and facts name them, each
    // predicate's facts where the first of them was.
    let mut facts = norm.facts.iter().peekable();
    for (i, clause) in norm.rules.iter().enumerate() {
        while let Some((_, f)) = facts.next_if(|(at, _)| *at <= i) {
            let p = g.pred(&f.name, f.arity, None);
            g.preds[p as usize].facts = Some(f);
        }
        let head = &clause.heads[0].atom;
        let counted = !norm.counted.is_empty()
            && norm.counted.contains(&(head.name.clone(), head.args.len()));
        let r = compile_rule(&mut g, clause, counted)?;
        let h = r.head.0;
        g.preds[h as usize].rules.push(g.rules.len());
        g.rules.push(r);
    }
    for (_, f) in facts {
        let p = g.pred(&f.name, f.arity, None);
        g.preds[p as usize].facts = Some(f);
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
    // A findall reads its goal's tuples once they are complete, in a lower
    // stratum. The plain program's findalls do ([`super::terms`]); the
    // magic rewrite's demand can close a loop through one, and the plain
    // program is ground then.
    if !norm.counted.is_empty() {
        let mut stratum_of = vec![0; g.preds.len()];
        for (k, s) in strata.iter().enumerate() {
            for &p in s {
                stratum_of[p as usize] = k;
            }
        }
        for r in &g.rules {
            for l in &r.body {
                if let ILit::Collect(c) = l
                    && stratum_of[c.aux as usize] == stratum_of[r.head.0 as usize]
                {
                    return Err(GroundError::Unsupported(format!(
                        "{}: its goal is read inside the recursion of its own clause",
                        c.text
                    )));
                }
            }
        }
    }
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
        // is one, a hundred thousand of them on bn_barley. A predicate's
        // facts, which read nothing, are a stratum of their own.
        if let &[p] = &stratum[..]
            && let Some(f) = g.preds[p as usize].facts
        {
            load(&mut g, p, f, !norm.control.is_empty());
        } else if stratum
            .iter()
            .all(|&p| g.preds[p as usize].rules.is_empty())
        {
            continue;
        } else {
            g.current_stratum = stratum.iter().copied().collect();
            match (levelled.get_mut(k).and_then(Option::take), &mut pruner) {
                (Some(lv), Some(pruner)) => keys += evaluate_by_levels(&mut g, stratum, lv, pruner),
                _ => evaluate_stratum(&mut g, stratum),
            }
        }
        if g.over() {
            return Err(if let Some(e) = g.error.take() {
                e
            } else if g.refused {
                GroundError::OutOfMemory
            } else {
                GroundError::OverBound
            });
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

    // The queries and the evidence, as the grounding numbered their atoms.
    let mut queries = Vec::new();
    for (q, positive) in &norm.queries {
        for (atom, id) in resolve_atom_pattern(&mut g, q) {
            queries.push((atom, id, *positive));
        }
    }
    let mut evidence = Vec::new();
    for (e, v) in &norm.evidence {
        for (atom, id) in resolve_atom_pattern(&mut g, e) {
            evidence.push((atom, id, *v));
        }
    }
    for d in &norm.drivers {
        // Every true instance of the driver binds the pattern's variables.
        for (instance, _) in resolve_atom_pattern(&mut g, &d.atom) {
            let binding: FxHashMap<&Term, &Term> = d.atom.args.iter().zip(&instance.args).collect();
            let q = Atom::new(
                d.pattern.name.clone(),
                d.pattern
                    .args
                    .iter()
                    .map(|t| substitute(t, &binding))
                    .collect::<Args>(),
            );
            let asked = if d.evidence {
                &mut evidence
            } else {
                &mut queries
            };
            for (atom, id) in resolve_atom_pattern(&mut g, &q) {
                asked.push((atom, id, d.positive));
            }
        }
    }
    if g.refused {
        return Err(GroundError::OutOfMemory);
    }
    // The driver atoms themselves are not part of the output program.
    let drivers: FxHashSet<&str> = norm.drivers.iter().map(|d| &*d.atom.name).collect();
    if !drivers.is_empty() {
        let name = |a: AtomId| &**g.preds[g.atoms[a as usize].0 as usize].name;
        g.ground_rules.retain(|r| !drivers.contains(name(r.head)));
    }

    // Assemble the output: the atoms in the order made, those of facts
    // under a demand as it read them ([`facts_as_read`]). Of a fact's
    // atoms, only those a rule, a query or the evidence names: no other
    // reads them, and a database's are mostly read by none (a query of
    // TPC-H reads a fiftieth of its six million line items).
    let n = g.atoms.len();
    let mut named: Vec<bool> = g
        .atoms
        .iter()
        .map(|(p, _)| g.preds[*p as usize].facts.is_none())
        .collect();
    for r in &g.ground_rules {
        named[r.head as usize] = true;
        for l in &r.body {
            named[l.atom() as usize] = true;
        }
    }
    for (_, id, _) in queries.iter().chain(&evidence) {
        if let Some(a) = id {
            named[*a as usize] = true;
        }
    }
    let order: Vec<AtomId> = match norm.control.is_empty() {
        true => (0..n as AtomId).collect(),
        false => facts_as_read(&g),
    };
    let order: Vec<AtomId> = order.into_iter().filter(|&a| named[a as usize]).collect();
    let mut gp = GroundProgram::default();
    let m = order.len();
    if gp.atoms.try_reserve_exact(m).is_err() || gp.kinds.try_reserve_exact(m).is_err() {
        return Err(GroundError::OutOfMemory);
    }
    for &id in &order {
        gp.atoms.push(g.display_atom(id));
        let (p, _) = &g.atoms[id as usize];
        // A fact's weights, those of its own or its switch predicate's.
        let weights = match (
            g.weights.get(id as usize).copied().flatten(),
            &g.preds[*p as usize].switch,
        ) {
            (Some(q), _) => Some(q),
            (None, Some(Weights::Fixed(q))) => Some(*q),
            (
                None,
                Some(Weights::Flexible {
                    index,
                    heads,
                    clause,
                }),
            ) => Some(flexible_weights(&g, id, *index, *heads, clause)?),
            (None, None) => None,
        };
        gp.kinds.push(match weights {
            Some(q) => AtomKind::Fact(q),
            None => AtomKind::Derived,
        });
    }
    gp.rules = std::mem::take(&mut g.ground_rules);
    gp.queries = queries;
    gp.evidence = evidence;
    gp.stats.levels = keys;
    gp.stats.constants = fixed.iter().filter(|v| v.is_some()).count();
    if let Some(pruner) = &pruner {
        gp.stats.swept.add(pruner.stats());
    }
    if m < n || order.iter().enumerate().any(|(i, &a)| a as usize != i) {
        let mut new = vec![AtomId::MAX; n];
        for (i, &a) in order.iter().enumerate() {
            new[a as usize] = i as AtomId;
        }
        let new = |a: AtomId| new[a as usize];
        let sorted = order.is_sorted();
        for r in &mut gp.rules {
            r.head = new(r.head);
            for l in &mut r.body {
                *l = GLit::new(new(l.atom()), l.is_positive());
            }
            // Bodies in the order of their atoms, as the grounder writes
            // them.
            if !sorted {
                r.body.sort_unstable();
            }
        }
        for (_, id, _) in gp.queries.iter_mut().chain(&mut gp.evidence) {
            *id = id.map(new);
        }
    }
    Ok(Some(gp))
}

/// The atoms of a program under a demand ([`super::magic`]) in the order
/// of the ground program: in the order made, but those of [`Facts`] as the
/// demand reaches them, by the round of evaluation that first read them
/// ([`ground_body`]), those one round reads in the order of their facts,
/// in the places their loading took, and those no rule reads after. A
/// predicate of facts is loaded whole, in the order of its facts, where a
/// demand made the atoms of the facts it reached in a round, in their
/// order, round after round: a search from the queries. The vtree
/// heuristics read the numbering, and a random graph of 80 edges written
/// by their sources compiled four times slower in the facts' order.
fn facts_as_read(g: &Grounder) -> Vec<AtomId> {
    let n = g.atoms.len();
    let of_facts = |a: usize| g.preds[g.atoms[a].0 as usize].facts.is_some();
    let places: Vec<AtomId> = (0..n)
        .filter(|&a| of_facts(a))
        .map(|a| a as AtomId)
        .collect();
    let mut read = g.read.clone();
    let ends = g.read_rounds.iter().skip(1).copied().chain([read.len()]);
    for (&start, end) in g.read_rounds.iter().zip(ends) {
        read[start..end].sort_unstable();
    }
    let unread = |a: &&AtomId| g.unread.get(**a as usize).is_some_and(|&u| u);
    read.extend(places.iter().filter(unread));
    let mut order: Vec<AtomId> = (0..n as AtomId).collect();
    for (&place, &a) in places.iter().zip(&read) {
        order[place as usize] = a;
    }
    order
}

/// Ground instances of a possibly non-ground query/evidence atom among the
/// possible atoms. A ground atom that is impossible resolves to `None`. A
/// ground atom of a switch predicate is one of its independent variables
/// whether or not a rule read it (`0.5::f(_). query(f(a)).` asks for 0.5).
fn resolve_atom_pattern(g: &mut Grounder, a: &Atom) -> Vec<(Atom, Option<AtomId>)> {
    // A predicate the program does not mention has no atoms.
    let Some((p, switch)) = g.pred_id(&a.name, a.args.len()) else {
        return if a.is_ground() {
            vec![(a.clone(), None)]
        } else {
            Vec::new()
        };
    };
    if a.is_ground() {
        let args: Vec<Val> = a.args.iter().map(|t| g.val(t)).collect();
        let id = match switch {
            true => Some(g.atom_id(p, &args).0),
            false => g.lookup_atom(p, &args),
        };
        return vec![(a.clone(), id)];
    }
    let rel = &g.rels[p as usize];
    let found: Vec<AtomId> = (0..rel.len())
        .filter(|&t| {
            let mut binding: FxHashMap<&str, Val> = FxHashMap::default();
            a.args
                .iter()
                .zip(rel.tuple(t))
                .all(|(pat, &v)| matches(g, pat, v, &mut binding))
        })
        .map(|t| rel.ids[t])
        .collect();
    let mut out: Vec<(Atom, Option<AtomId>)> = found
        .into_iter()
        .map(|id| (g.display_atom(id), Some(id)))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Whether the value `v` is an instance of the pattern `pat` under
/// `binding`, which it extends.
fn matches<'t>(g: &Grounder, pat: &'t Term, v: Val, binding: &mut FxHashMap<&'t str, Val>) -> bool {
    match pat {
        Term::Var(x) => *binding.entry(x).or_insert(v) == v,
        Term::Const(s) => g.syms.ids.get(s).is_some_and(|&i| v == Val::Sym(i)),
        Term::Int(i) => v == Val::Int(*i),
        Term::Float(x) => v == Val::Float(x.0.to_bits()),
        Term::Compound(c) => {
            let Val::Fn(id) = v else {
                return false;
            };
            let key = g.terms.keys.list(id as usize);
            key.len() == 1 + c.args.len()
                && g.syms
                    .ids
                    .get(&c.name)
                    .is_some_and(|&n| key[0] == Val::Sym(n))
                && c.args
                    .iter()
                    .zip(&key[1..])
                    .all(|(p, &a)| matches(g, p, a, binding))
        }
    }
}

/// The term `t` with the variables of `binding` replaced.
fn substitute(t: &Term, binding: &FxHashMap<&Term, &Term>) -> Term {
    match t {
        Term::Var(_) => binding.get(t).map_or_else(|| t.clone(), |&b| b.clone()),
        Term::Compound(c) if !t.is_ground() => Compound::term(
            c.name.clone(),
            c.args.iter().map(|a| substitute(a, binding)).collect(),
        ),
        _ => t.clone(),
    }
}

/// Makes the atoms of the facts `f`, the relation of `p`: an atom per
/// tuple, in the order of the facts, a fact of its weights, or one true in
/// every world by a rule with an empty body where the tuple has none, as
/// its fact's rule would make it. A tuple given twice is true where either
/// of its facts is, as ProbLog reads two facts of one atom: the weights of
/// its falsity multiply. Under a demand (`demanded`), each atom is unread
/// until a rule reads it ([`facts_as_read`]). An atom no rule, query or
/// evidence names is left out of the ground program ([`ground_with`]).
fn load(g: &mut Grounder, p: PredId, f: &Facts, demanded: bool) {
    // Room for every tuple at once: a table of millions doubles its
    // vectors and rehashes its keys a score of times otherwise.
    let n = f.probs.len();
    if !g.room(|g| {
        g.atoms.try_reserve(n).is_ok()
            && g.weights
                .try_reserve(g.atoms.len() + n - g.weights.len())
                .is_ok()
            && g.rels[p as usize].reserve(n)
    }) {
        return;
    }
    let control = g.preds[p as usize].control;
    let mut vals = Vec::with_capacity(f.arity);
    for (args, weights) in f.tuples() {
        vals.clear();
        vals.extend(args.iter().map(|a| g.val(a)));
        let (id, new) = g.atom_id(p, &vals);
        if g.over() {
            return;
        }
        // A demand given as a fact has no atom ([`Grounder::atom_id`]).
        if control {
            continue;
        }
        let at = id as usize;
        if demanded && new {
            if g.unread.len() <= at {
                g.unread.resize(at + 1, false);
            }
            g.unread[at] = true;
        }
        if g.certain(id) {
            continue;
        }
        if g.weights.len() <= at {
            g.weights.resize(at + 1, None);
        }
        match (weights, g.weights[at]) {
            (Some(q), Some(w)) if !new => g.weights[at] = Some(w.either(q)),
            (Some(q), _) => g.weights[at] = Some(q),
            (None, _) => {
                g.weights[at] = None;
                if g.certain.len() <= at {
                    g.certain.resize(at + 1, false);
                }
                g.certain[at] = true;
                g.keep_rule(GroundRule::new(id, Vec::new()));
            }
        }
    }
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
    g.new_round();
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
    preds.iter().map(|&p| g.rels[p as usize].len()).collect()
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
            let e = g.rels[p as usize].len();
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
        g.new_round();
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
                    (Some(Arg::Var(vh)), Some(Arg::Var(vb))) => {
                        step_of(g, &local, &rule.body, *vh, *vb)?
                    }
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

/// c >= 1 where `body` sets `vh` c or more above `vb`: `vh is vb + c` or
/// `vh is c + vb`, or a literal of a predicate outside the stratum (in
/// `local`) whose clauses all rise by c or more from `vb`'s argument to
/// `vh`'s, a successor relation given by its facts, as `next(T, T1)` in
/// the event calculus's `holds(F, T1) :- next(T, T1), holds(F, T), ...`.
fn step_of(
    g: &Grounder,
    local: &FxHashMap<PredId, usize>,
    body: &[ILit],
    vh: u32,
    vb: u32,
) -> Option<i64> {
    let rises = |q: PredId, args: &[Arg]| {
        let at = |v| {
            args.iter()
                .position(|a| matches!(a, Arg::Var(x) if *x == v))
        };
        let (b, h) = (at(vb)?, at(vh)?);
        // The least step over the predicate's facts or its rules' heads.
        let least = |steps: &mut dyn Iterator<Item = (Option<i64>, Option<i64>)>| {
            steps
                .map(|step| match step {
                    (Some(b), Some(h)) if h - b >= 1 => Some(h - b),
                    _ => None,
                })
                .try_fold(i64::MAX, |m, c| Some(m.min(c?)))
                .filter(|&c| c < i64::MAX)
        };
        let int = |a: &Arg| match a {
            Arg::Val(Val::Int(i)) => Some(*i),
            _ => None,
        };
        let term = |t: &Term| match t {
            Term::Int(i) => Some(*i),
            _ => None,
        };
        let pred = &g.preds[q as usize];
        match pred.facts {
            Some(f) => least(&mut f.tuples().map(|(args, _)| (term(&args[b]), term(&args[h])))),
            None => least(&mut pred.rules.iter().map(|&r| {
                let head = &g.rules[r].head.1;
                (int(&head[b]), int(&head[h]))
            })),
        }
    };
    body.iter().find_map(|l| match l {
        ILit::Pos(q, args) if !local.contains_key(q) => rises(*q, args),
        ILit::Is(Arg::Var(v), IExpr::Op(ArithFn::Add, ab)) if *v == vh => match (&ab[0], &ab[1]) {
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
            let absent = |x: AtomId| g.key_of(x).is_some() && !g.derived(x);
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
            let e = g.rels[p as usize].len();
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
            match (g.derived(x), g.fixed(x)) {
                (false, _) | (true, Some(false)) => continue,
                (true, Some(true)) => return,
                (true, None) => {}
            }
        }
        lits.push(l);
    }
    let (head, _) = g.atom_id(p, &args);
    lits.sort_unstable();
    lits.dedup();
    g.keep_rule(GroundRule::new(head, lits));
}

/// Evaluate rule `r`. With `delta = Some((i, range))`, body literal `i` ranges
/// only over tuples in `range` of its relation, and the walk joins the rest
/// of the body to them where that pays ([`from_delta`], [`from_pays`]).
fn eval_rule(g: &mut Grounder, r: usize, delta: Option<(usize, (usize, usize))>) {
    if let Some((i, range)) = delta
        && i > 0
    {
        let variant = match g.from_delta.remove(&(r, i)) {
            Some(variant) => variant,
            None => from_delta(g, r, i),
        };
        let pays = from_pays(g, &variant, range);
        if pays {
            walk_rule(g, &variant.rule, Some((variant.at, range)));
        }
        g.from_delta.insert((r, i), variant);
        if pays {
            return;
        }
    }
    // Grounding a rule adds atoms and ground rules but never rules, so the
    // rule is lent out for the walk and put back.
    let rule = std::mem::take(&mut g.rules[r]);
    walk_rule(g, &rule, delta);
    g.rules[r] = rule;
}

/// Grounds `rule`, its body walked in order, literal `i` of `delta` over
/// the tuples in its range.
fn walk_rule(g: &mut Grounder, rule: &IRule, delta: Option<(usize, (usize, usize))>) {
    if rule.counted.is_some() {
        g.stamp += 1;
        g.parts.clear();
        g.parts.resize(rule.body.len(), Part::None);
    }
    let (mut subst, mut body_atoms) = std::mem::take(&mut g.walk);
    subst.clear();
    subst.resize(rule.num_vars as usize, None);
    body_atoms.clear();
    ground_body(g, rule, 0, delta, &mut subst, &mut body_atoms);
    g.walk = (subst, body_atoms);
}

/// A rule as a semi-naive round walks it from one of its atoms
/// ([`from_delta`]).
struct FromDelta {
    rule: IRule,
    /// The place of the atom in the body.
    at: usize,
    /// The positive atoms the rule's own order joins before the atom, and
    /// their arguments bound there.
    before: Vec<(PredId, Vec<usize>)>,
    /// The positive atom joined after it, and its arguments bound there.
    next: Option<(PredId, Vec<usize>)>,
}

/// Rule `r` with its body ordered from literal `i`, a positive atom: that
/// atom first, the others joined to it ([`Order::From`]). A semi-naive
/// round walks it with the round's new tuples in that atom, so that the
/// walk starts from them. In the rule's own order the round would walk
/// every binding of the atoms before it to find the few that meet a new
/// tuple: pcfg's `state(W, [s|S])` comes after its demand
/// `'$m_state_bb'(W, [o, m|S])`, and each of pcfg24's 43 rounds walked its
/// 368 518 demands. The atoms and rules are those of the rule's own order;
/// the order in which the walk makes them is another.
fn from_delta(g: &Grounder, r: usize, i: usize) -> FromDelta {
    let clause = &g.clauses[r];
    let switch = |j: usize| match &clause.body[j] {
        Literal::Pos(a) | Literal::Neg(a) => g
            .pred_id(&a.name, a.args.len())
            .is_some_and(|(_, switch)| switch),
        _ => false,
    };
    let schedule = |order| {
        schedule_body(&clause.body, &mut FxHashSet::default(), switch, order)
            .expect("a compiled rule's body is scheduled from any of its atoms")
    };
    // The rule's body is compiled in this order ([`compile_rule`]).
    let own = schedule(Order::Join);
    let ordered = schedule(Order::From(own[i]));
    let rule = &g.rules[r];
    let mut place = vec![0; own.len()];
    for (k, &c) in own.iter().enumerate() {
        place[c] = k;
    }
    let body: Vec<ILit> = ordered
        .iter()
        .map(|&c| rule.body[place[c]].clone())
        .collect();
    let counted = rule.counted.as_ref().map(|c| {
        let mut order: Vec<usize> = (0..ordered.len()).collect();
        order.sort_by_key(|&k| ordered[k]);
        Box::new(Counted {
            group: c.group,
            order,
        })
    });
    let at = ordered
        .iter()
        .position(|&c| c == own[i])
        .expect("the atom is in the body");
    // The positive atom at place `k` of `lits`, in the order `order` of
    // the clause's literals, and its arguments bound by the literals before.
    let atom_at = |lits: &[ILit], order: &[usize], k: usize| {
        let ILit::Pos(p, _) = &lits[k] else {
            return None;
        };
        let Literal::Pos(a) = &clause.body[order[k]] else {
            unreachable!("a positive atom of the rule is one of the clause")
        };
        let bound: FxHashSet<String> = order[..k]
            .iter()
            .flat_map(|&c| literal_vars(&clause.body[c]))
            .collect();
        let at_bound = (0..a.args.len())
            .filter(|&pos| match &a.args[pos] {
                Term::Var(v) => bound.contains(v),
                _ => true,
            })
            .collect();
        Some((*p, at_bound))
    };
    let before = (0..i)
        .filter_map(|k| atom_at(&rule.body, &own, k))
        .collect();
    let next = (at + 1..body.len()).find_map(|k| atom_at(&body, &ordered, k));
    FromDelta {
        rule: IRule {
            head: rule.head.clone(),
            body,
            num_vars: rule.num_vars,
            counted,
        },
        at,
        before,
        next,
    }
}

/// Whether a round with the new tuples `range` in the atom `v` starts
/// from walks `v` rather than rule `r` in its own order: where the new
/// tuples, times the tuples the atom joined next is expected to match for
/// each, are fewer than the bindings the own order is expected to walk to
/// reach the atom, the product of the tuples each atom before it is
/// expected to match. A relation is expected to hold as many tuples for a
/// value of an argument as it holds for each of the values there, the
/// fewest over the arguments bound. The walk from the new tuples pays
/// where it reaches the next atom by a value that few of its tuples share,
/// pcfg's demand by its whole state, and where the own order reaches the
/// atom only after a join, as PSouffle's side-channel analyses do after
/// the demand `'$m_rand_f'`, which binds nothing, through each of
/// `xor_assign_left(TO, FROM1)`'s tuples. The own order pays where the
/// walk from the new tuples can reach the next atom by none of its values,
/// or by one that all share: the label of DeepProbLog's handwritten
/// formulas, the same in each demand of `parse(Symbols, Label)`, as each
/// demand of `parse([N1, *, N2|T], R)` meets the new tuple
/// `parse([N3|T], R)` only through `N3 is N1 * N2`.
fn from_pays(g: &mut Grounder, v: &FromDelta, (s, e): (usize, usize)) -> bool {
    let expected = |g: &mut Grounder, p: PredId, bound: &[usize]| {
        let rel = &mut g.rels[p as usize];
        let n = rel.len();
        if bound.is_empty() || n == 0 {
            return n;
        }
        // Every argument bound: a lookup, which matches one tuple at most
        // and needs no index. An index built only to tell it would hold a
        // key per tuple: pcfg's demand of a state, bound whole, from each
        // of its tens of millions of demands.
        if bound.len() == rel.arity {
            return 1;
        }
        for &pos in bound {
            rel.ensure_index(pos);
        }
        bound
            .iter()
            .map(|&pos| n.div_ceil(rel.index[pos].len().max(1)))
            .min()
            .unwrap_or(n)
    };
    let from = match &v.next {
        Some((q, bound)) => (e - s).saturating_mul(expected(g, *q, bound)),
        None => e - s,
    };
    let mut own = 1usize;
    for (p, bound) in &v.before {
        own = own.saturating_mul(expected(g, *p, bound));
    }
    from < own
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

/// Records what body literal `i` of a counted rule contributes to the
/// derivation being walked ([`Part`]).
fn part(g: &mut Grounder, rule: &IRule, i: usize, p: Part) {
    if rule.counted.is_some() {
        g.parts[i] = p;
    }
}

/// Records the derivation of `head :- body` by a counted rule: a ground
/// rule that another clause derives too, or the same walk by another
/// substitution, has more than one proof.
fn derive(g: &mut Grounder, head: AtomId, body: &[GLit], c: &Counted) {
    let stamp = g.stamp;
    let parts: Vec<Part> = c
        .order
        .iter()
        .map(|&k| g.parts[k])
        .filter(|&p| p != Part::None)
        .collect();
    let derivations = g.derivations.entry(head).or_default();
    match derivations.iter_mut().find(|d| d.body == body) {
        Some(d) => {
            if d.group != c.group || d.stamp == stamp {
                d.mult = 2;
            }
        }
        None => derivations.push(Derivation {
            body: body.to_vec(),
            group: c.group,
            stamp,
            mult: 1,
            parts,
        }),
    }
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
        // The atoms of facts read for the first time, under a demand.
        if !g.unread.is_empty() {
            for l in body_atoms.iter() {
                let a = l.atom() as usize;
                if g.unread.get(a).is_some_and(|&u| u) {
                    g.unread[a] = false;
                    g.read.push(a as AtomId);
                }
            }
        }
        let (head, new) = g.atom_id(rule.head.0, &head_args);
        if g.preds[rule.head.0 as usize].control {
            g.demands += new as usize;
            return; // control atoms exist only to drive grounding
        }
        // A rule whose body atoms are all true in every world makes its
        // head so, and the head's other derivations say nothing: a closure
        // of deterministic facts, PSouffle's `equal_assign`, derives each
        // of its atoms once per atom between, and kept a rule for each.
        let counted = rule.counted.is_some();
        if !counted && g.certain(head) {
            return;
        }
        let mut body = body_atoms.clone();
        body.sort_unstable();
        body.dedup();
        if let Some(c) = &rule.counted {
            derive(g, head, &body, c);
        }
        if !counted
            && body
                .iter()
                .all(|l| matches!(*l, GLit::Pos(a) if g.certain(a)))
        {
            if g.certain.len() <= head as usize {
                g.certain.resize(head as usize + 1, false);
            }
            g.certain[head as usize] = true;
        }
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
                    part(g, rule, i, Part::None);
                    ground_body(g, rule, i + 1, delta, subst, body_atoms);
                } else {
                    part(g, rule, i, Part::Atom(id));
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
            let candidates = {
                let rel = &mut g.rels[p as usize];
                let range = range.unwrap_or((0, rel.len()));
                rel.candidates(&bound_pos, range)
            };
            for t in candidates {
                let rel = &g.rels[p as usize];
                let tuple = rel.tuple(t);
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
                let id = rel.ids[t];
                let fixed = g.fixed(id);
                if ok && fixed != Some(false) {
                    if control || fixed == Some(true) {
                        part(g, rule, i, Part::None);
                        ground_body(g, rule, i + 1, delta, subst, body_atoms);
                    } else {
                        part(g, rule, i, Part::Atom(id));
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
                let id = with_resolved(args, subst, |vals| g.placeholder_atom(*p, vals));
                part(g, rule, i, Part::Neg(id));
                body_atoms.push(GLit::Neg(id));
                ground_body(g, rule, i + 1, delta, subst, body_atoms);
                body_atoms.pop();
                return;
            }
            match with_resolved(args, subst, |vals| g.lookup_atom(*p, vals)) {
                // An atom true in every world: the literal is false. Kept
                // as a literal, it would not stop a demand, whose rule
                // drops its body: cplint's left-corner grammar demanded
                // every derivation of a goal's third clause behind
                // `\+ lc(G, B, rule(G, [B|_]))`, which holds nowhere.
                Some(id) if g.fixed(id) == Some(true) || g.certain(id) => {}
                Some(id) if g.fixed(id) == Some(false) => {
                    part(g, rule, i, Part::None);
                    ground_body(g, rule, i + 1, delta, subst, body_atoms)
                }
                // Possible atom: keep the negative literal.
                Some(id) => {
                    part(g, rule, i, Part::Neg(id));
                    body_atoms.push(GLit::Neg(id));
                    ground_body(g, rule, i + 1, delta, subst, body_atoms);
                    body_atoms.pop();
                }
                // Impossible atom: the literal is true.
                None => {
                    part(g, rule, i, Part::None);
                    ground_body(g, rule, i + 1, delta, subst, body_atoms)
                }
            }
        }
        ILit::Switch(p, args) => {
            let (id, _) = with_resolved(args, subst, |vals| g.atom_id(*p, vals));
            part(g, rule, i, Part::Atom(id));
            body_atoms.push(GLit::Pos(id));
            ground_body(g, rule, i + 1, delta, subst, body_atoms);
            body_atoms.pop();
        }
        ILit::NegSwitch(p, args) => {
            let (id, _) = with_resolved(args, subst, |vals| g.atom_id(*p, vals));
            part(g, rule, i, Part::Neg(id));
            body_atoms.push(GLit::Neg(id));
            ground_body(g, rule, i + 1, delta, subst, body_atoms);
            body_atoms.pop();
        }
        ILit::Cmp(op, a, b) => {
            let arithmetic = op.arithmetic();
            match (side(g, a, subst, arithmetic), side(g, b, subst, arithmetic)) {
                (Some(x), Some(y)) if compare(g, *op, x, y) => {
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
                    if let Some(Arg::Var(v)) = b.as_arg() {
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
            if let Some(x) = eval_num(g, e, subst).map(|n| n.rounded().val()) {
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
        // Taken apart where the term is bound, built from its bound
        // arguments otherwise.
        ILit::Build(target, name, args) => {
            let mut bound = Vec::new();
            let ok = match arg_val(target, subst) {
                Some(Val::Fn(id)) => {
                    let key = g.terms.keys.list(id as usize);
                    key[0] == Val::Sym(*name) && key.len() == 1 + args.len() && {
                        let vals: Vec<Val> = key[1..].to_vec();
                        unify_args(args, &vals, subst, &mut bound)
                    }
                }
                Some(_) => false,
                None => {
                    let vals = resolve(args, subst);
                    let v = g.intern_fn(*name, &vals, true);
                    unify_args(std::slice::from_ref(target), &[v], subst, &mut bound)
                }
            };
            if ok {
                ground_body(g, rule, i + 1, delta, subst, body_atoms);
            }
            for x in bound {
                subst[x as usize] = None;
            }
        }
        ILit::Builtin(b, args, negated) => {
            let vals: Vec<Option<Val>> = args.iter().map(|a| arg_val(a, subst)).collect();
            let mut bound = Vec::new();
            // The range of `between/3`, walked without listing it.
            if *b == Builtin::Between && vals[2].is_none() && !negated {
                let (Some(Val::Int(lo)), Some(Val::Int(hi))) = (vals[0], vals[1]) else {
                    return;
                };
                for x in lo..=hi {
                    if g.over() {
                        break;
                    }
                    if unify_args(&args[2..], &[Val::Int(x)], subst, &mut bound) {
                        part(g, rule, i, Part::Rank((x - lo) as u32));
                        ground_body(g, rule, i + 1, delta, subst, body_atoms);
                    }
                    for x in bound.drain(..) {
                        subst[x as usize] = None;
                    }
                }
                return;
            }
            let sols = solutions(g, *b, &vals);
            if *negated {
                if !sols.iter().any(|s| {
                    args.iter()
                        .zip(s)
                        .all(|(a, v)| arg_val(a, subst) == Some(*v))
                }) {
                    ground_body(g, rule, i + 1, delta, subst, body_atoms);
                }
                return;
            }
            for (rank, sol) in sols.iter().enumerate() {
                if unify_args(args, sol, subst, &mut bound) {
                    part(g, rule, i, Part::Rank(rank as u32));
                    ground_body(g, rule, i + 1, delta, subst, body_atoms);
                }
                for x in bound.drain(..) {
                    subst[x as usize] = None;
                }
            }
        }
        ILit::Collect(c) => {
            let shared = resolve(&c.shared, subst);
            let Some(list) = collect(g, c, &shared) else {
                return;
            };
            let mut bound = Vec::new();
            if unify_args(std::slice::from_ref(&c.result), &[list], subst, &mut bound) {
                ground_body(g, rule, i + 1, delta, subst, body_atoms);
            }
            for x in bound {
                subst[x as usize] = None;
            }
        }
    }
}
