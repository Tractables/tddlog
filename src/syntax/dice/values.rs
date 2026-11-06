//! The values the translation binds: a Boolean literal, an integer of
//! `w` bits, each a literal, with the values it can take, or a tuple of
//! values, a list held as the pair of its length and its elements. With
//! them are the variables in scope ([`Env`]), what a condition says of the
//! variables it compares with constants ([`Facts`]), and Dice's widths and
//! `discrete` probabilities; the translation ([`super::translate`]) builds
//! and reads them.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use rustc_hash::FxHashMap;

use super::syntax::Type;
use super::{LIST_LENGTH, Res, STEP};
use crate::network::rows::fsum;
use crate::syntax::fail;

/// A literal: an atom, its negation `\+atom`, `true` or `false`. In a
/// loop's body an atom of the step is `name(T)`, and the state after step
/// k `name(k)`.
pub(super) type Lit = Rc<str>;

pub(super) fn lit(s: &str) -> Lit {
    Rc::from(s)
}

pub(super) fn is(l: &str, constant: &str) -> bool {
    l == constant
}

pub(super) fn neg(l: &str) -> Lit {
    match l {
        "true" => lit("false"),
        "false" => lit("true"),
        _ => match l.strip_prefix("\\+") {
            Some(a) => lit(a),
            None => lit(&format!("\\+{l}")),
        },
    }
}

/// Whether `a` is the negation of `b`.
pub(super) fn complementary(a: &str, b: &str) -> bool {
    a.strip_prefix("\\+") == Some(b) || b.strip_prefix("\\+") == Some(a)
}

/// Whether a literal is of the step of a loop's body.
pub(super) fn at_step(l: &str) -> bool {
    l.ends_with(STEP)
}

/// An integer of `width` bits, modulo 2^width: `bits[k]` is the literal of
/// its bit k, least significant first (`true` or `false` where it is
/// constant). `support` holds every value it can take, sorted (it may hold
/// more), or is `None` where that set was not worked out. Its query atoms
/// are `name(v)`, where it has a name, else a fresh atom from `hint`;
/// `size` values from 0 are queried when it is the program's result. `id`
/// tells integers apart: operations on the same integers are made once.
pub(super) struct Int {
    pub(super) id: u64,
    pub(super) width: u64,
    pub(super) bits: Vec<Lit>,
    pub(super) support: Option<Vec<u64>>,
    pub(super) name: Option<Rc<str>>,
    pub(super) hint: Option<Rc<str>>,
    pub(super) size: u64,
}

impl Int {
    pub(super) fn modulus(&self) -> u64 {
        1 << self.width
    }

    pub(super) fn allows(&self, v: u64) -> bool {
        v < self.modulus()
            && self
                .support
                .as_ref()
                .is_none_or(|s| s.binary_search(&v).is_ok())
    }
}

/// A Boolean literal, an integer, or a tuple of values.
#[derive(Clone)]
pub(super) enum Value {
    Bool(Lit),
    Int(Rc<Int>),
    Tuple(Rc<(Value, Value)>),
}

pub(super) fn boolean(constant: bool) -> Value {
    Value::Bool(lit(if constant { "true" } else { "false" }))
}

impl Value {
    pub(super) fn shape(&self) -> char {
        match self {
            Value::Bool(_) => 'b',
            Value::Int(_) => 'i',
            Value::Tuple(_) => 't',
        }
    }

    /// The Booleans and integers, depth first.
    pub(super) fn flat(&self) -> Vec<Value> {
        let mut out = Vec::new();
        let mut stack = vec![self];
        while let Some(v) = stack.pop() {
            match v {
                Value::Tuple(t) => {
                    stack.push(&t.1);
                    stack.push(&t.0);
                }
                v => out.push(v.clone()),
            }
        }
        out
    }

    /// A name for each of [`Value::flat`]: `base`, and `base_1` and `base_2`
    /// for a tuple's components.
    pub(super) fn flat_names(&self, base: &str) -> Vec<String> {
        match self {
            Value::Tuple(t) => {
                let mut out = t.0.flat_names(&format!("{base}_1"));
                out.extend(t.1.flat_names(&format!("{base}_2")));
                out
            }
            _ => vec![base.to_string()],
        }
    }

    /// A value of this one's shape from the parts [`Value::flat`] would
    /// give.
    pub(super) fn rebuild(&self, parts: &mut impl Iterator<Item = Value>) -> Value {
        match self {
            Value::Tuple(t) => {
                let a = t.0.rebuild(parts);
                let b = t.1.rebuild(parts);
                Value::Tuple(Rc::new((a, b)))
            }
            _ => parts.next().expect("a part per Boolean and integer"),
        }
    }

    /// The literals of a Boolean or an integer.
    pub(super) fn bits(&self) -> Vec<Lit> {
        match self {
            Value::Bool(l) => vec![l.clone()],
            Value::Int(x) => x.bits.clone(),
            Value::Tuple(_) => Vec::new(),
        }
    }
}

/// The variables in scope: each binding shadows the ones after it.
/// The variables in scope, in frames, the innermost first: a `let`
/// chain's, a call's parameters, a branch's variables restricted to the
/// values its guard allows. A frame is a table, so that a variable bound
/// early in a chain of a hundred thousand `let`s is found at once, and
/// the frames are as deep as the program's nesting, not as long as its
/// chains: a binding each, they were a list that took time quadratic in a
/// chain's length to search and that overflowed the stack when dropped.
#[derive(Clone, Default)]
pub(super) struct Env<'a>(Option<Rc<Frame<'a>>>);

#[derive(Clone)]
struct Frame<'a> {
    vars: FxHashMap<&'a str, Value>,
    next: Env<'a>,
}

impl<'a> Env<'a> {
    /// `self` with an empty frame inside it.
    pub(super) fn enter(&self) -> Env<'a> {
        Env(Some(Rc::new(Frame {
            vars: FxHashMap::default(),
            next: self.clone(),
        })))
    }

    /// `name` bound to `value` in the innermost frame, which [`Env::enter`]
    /// made: a binding of the same name before it is shadowed.
    pub(super) fn bind(&mut self, name: &'a str, value: Value) {
        let frame = self.0.as_mut().expect("a frame entered");
        Rc::make_mut(frame).vars.insert(name, value);
    }

    pub(super) fn get(&self, name: &str) -> Option<&Value> {
        let mut at = &self.0;
        while let Some(f) = at {
            if let Some(v) = f.vars.get(name) {
                return Some(v);
            }
            at = &f.next.0;
        }
        None
    }
}

/// What a condition, or its negation, says of variables compared with
/// constants: each variable and the values it allows (1 and 0 for a
/// Boolean's true and false); `None` where two facts conflict.
pub(super) type Facts<'a> = Option<BTreeMap<&'a str, BTreeSet<u64>>>;

/// The facts of two conditions that both hold.
pub(super) fn meet<'a>(a: Facts<'a>, b: Facts<'a>) -> Facts<'a> {
    let (mut out, b) = (a?, b?);
    for (var, allowed) in b {
        let both: BTreeSet<u64> = match out.get(var) {
            Some(have) => have.intersection(&allowed).copied().collect(),
            None => allowed,
        };
        if both.is_empty() {
            return None;
        }
        out.insert(var, both);
    }
    Some(out)
}

/// The facts of a condition that holds where either of two does: the
/// union of what both say of a variable; `None` if neither can hold.
pub(super) fn join<'a>(a: Facts<'a>, b: Facts<'a>) -> Facts<'a> {
    let (Some(a), Some(b)) = (&a, &b) else {
        return a.or(b);
    };
    Some(
        a.iter()
            .filter_map(|(var, x)| Some((*var, x.union(b.get(var)?).copied().collect())))
            .collect(),
    )
}

pub(super) fn mirror(op: &str) -> Option<&'static str> {
    Some(match op {
        "==" => "==",
        "!=" => "!=",
        "<" => ">",
        "<=" => ">=",
        ">" => "<",
        ">=" => "<=",
        _ => return None,
    })
}

pub(super) fn compare(op: &str, a: u64, b: u64) -> bool {
    match op {
        "==" => a == b,
        "!=" => a != b,
        "<" => a < b,
        "<=" => a <= b,
        ">" => a > b,
        _ => a >= b,
    }
}

/// Dice's int width for `n` values (`num_binary_digits (n - 1)`).
pub(super) fn bits_for(n: usize) -> u64 {
    u64::from((n.max(1) - 1).checked_ilog2().map_or(0, |b| b + 1)).max(1)
}

/// The width of a list's length (Dice's `bit_length max_list_length`).
pub(super) fn list_width() -> u64 {
    bits_for(LIST_LENGTH + 1)
}

/// The type of a value, a list's as the pair it is held as.
pub(super) fn type_of(v: &Value) -> Type {
    match v {
        Value::Bool(_) => Type::Bool,
        Value::Int(x) => Type::Int(x.width),
        Value::Tuple(t) => Type::Tuple(Box::new(type_of(&t.0)), Box::new(type_of(&t.1))),
    }
}

/// A list's length and its [`LIST_LENGTH`] elements.
pub(super) fn list_parts(v: &Value, name: &str, line: usize) -> Res<(Rc<Int>, Vec<Value>)> {
    let not_list = || fail(line, format!("{name}: a list is expected"));
    let Value::Tuple(t) = v else {
        return Err(not_list());
    };
    let Value::Int(n) = &t.0 else {
        return Err(not_list());
    };
    if n.width != list_width() {
        return Err(not_list());
    }
    let mut elems = Vec::with_capacity(LIST_LENGTH);
    let mut rest = &t.1;
    for _ in 1..LIST_LENGTH {
        let Value::Tuple(pair) = rest else {
            return Err(not_list());
        };
        elems.push(pair.0.clone());
        rest = &pair.1;
    }
    elems.push(rest.clone());
    Ok((n.clone(), elems))
}

/// The probabilities of `discrete(ps)` as read: a list whose sum is off
/// one by rounding in the source, by at most 1e-4 (the tolerance of Dice's
/// type check, its `Passes.ml`), is divided by its sum, as Dice does: each
/// bit of its encoding is a flip whose probability is a ratio of sums of
/// the list. Read as written, a sum over one is an error in ProbLog, and
/// one under one leaves the variable without a value in some worlds. The
/// sum is `math.fsum`'s, correctly rounded.
pub(super) fn discrete_probs(ps: &[f64]) -> Vec<f64> {
    let total = fsum(ps);
    if total != 1.0 && (total - 1.0).abs() <= 1e-4 {
        ps.iter().map(|p| p / total).collect()
    } else {
        ps.to_vec()
    }
}
