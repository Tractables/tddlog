//! The translation ([`Translator`]): the program evaluated symbolically
//! into the lines of the ProbLog program it is read as. Every `flip`
//! becomes a probabilistic fact, Boolean operators and conditionals rules
//! over fresh atoms, integer operations rules on the bits, an `observe`
//! an evidence atom, and an `iterate` a recursion over a step index
//! ([`Translator::iterate`]). Operations on the same operands are made
//! once ([`Key`]), and the program's return value is queried.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use num_rational::BigRational;
use num_traits::ToPrimitive;
use rustc_hash::{FxHashMap, FxHashSet};

use super::syntax::{Count, Expr, Fun, Kind, Program, Type, calls};
use super::values::{
    Env, Facts, Int, Lit, Value, at_step, bits_for, boolean, compare, complementary,
    discrete_probs, is, join, list_parts, list_width, lit, meet, mirror, neg, type_of,
};
use super::{LIST_LENGTH, MAX_UNIFORM_WIDTH, MAX_WIDTH, RECURSION_LIMIT, Res, STEP, SUPPORT_PAIRS};
use crate::syntax::{fail, float_repr};

/// Operations made once: the key of each, by its operands.
#[derive(Clone, PartialEq, Eq, Hash)]
enum Key {
    Gate(Vec<Vec<Lit>>),
    Constant(u64, u64),
    Arith(&'static str, u64, u64),
    Shift(&'static str, u64, u64),
    And(Lit, Lit),
    Or(Lit, Lit),
    Any(Vec<Lit>),
    Restrict(u64, Vec<u64>),
    Merge(Vec<(Lit, u64)>),
}

/// What [`Translator::live`] makes of a disjunction of conjunctions.
enum Live {
    /// A constant, or one literal.
    Lit(Lit),
    /// The conjunctions left.
    Bodies(Vec<Vec<Lit>>),
}

/// The translation up to a point, which [`Translator::undo`] goes back to.
struct Mark {
    lines: usize,
    evidence: usize,
    counter: FxHashMap<String, usize>,
    taken: FxHashSet<String>,
    memos: FxHashMap<Key, Value>,
}

pub(super) struct Translator<'a> {
    lines: Vec<String>,
    counter: FxHashMap<String, usize>,
    taken: FxHashSet<String>,
    funs: FxHashMap<&'a str, (&'a Fun<'a>, bool)>,
    evidence: Vec<Lit>,
    active: FxHashMap<&'a str, usize>,
    memos: FxHashMap<Key, Value>,
    /// In a loop's body: the step predicate, and the body's observations.
    in_loop: Option<String>,
    observed: Vec<Lit>,
    /// The loops' step literals.
    steps: FxHashSet<String>,
    ids: u64,
}

impl<'a> Translator<'a> {
    pub(super) fn new(program: &'a Program<'a>) -> Self {
        let mut funs = FxHashMap::default();
        for f in &program.funs {
            funs.insert(f.name, (f, calls(&f.body, f.name)));
        }
        Translator {
            lines: Vec::new(),
            counter: FxHashMap::default(),
            taken: FxHashSet::default(),
            funs,
            evidence: Vec::new(),
            active: FxHashMap::default(),
            memos: FxHashMap::default(),
            in_loop: None,
            observed: Vec::new(),
            steps: FxHashSet::default(),
            ids: 0,
        }
    }

    pub(super) fn translate(mut self, body: &'a Expr<'a>) -> Res<String> {
        let result = self.eval(body, &Env::default(), &[], "result")?;
        for a in std::mem::take(&mut self.evidence) {
            self.lines.push(match a.strip_prefix("\\+") {
                Some(atom) => format!("evidence({atom}, false)."),
                None => format!("evidence({a}, true)."),
            });
        }
        self.queries(&result, "result");
        let mut out = self.lines.join("\n");
        out.push('\n');
        Ok(out)
    }

    /// A new integer.
    fn int(
        &mut self,
        width: u64,
        bits: Vec<Lit>,
        support: Option<Vec<u64>>,
        name: Option<Rc<str>>,
        hint: Option<Rc<str>>,
        size: Option<u64>,
    ) -> Rc<Int> {
        let support = support.map(|mut s| {
            s.sort_unstable();
            s
        });
        let size = size.unwrap_or_else(|| match support.as_deref() {
            Some([.., last]) => last + 1,
            _ => 1 << width,
        });
        self.ids += 1;
        Rc::new(Int {
            id: self.ids,
            width,
            bits,
            support,
            name,
            hint,
            size,
        })
    }

    /// A new atom name from `base`: base, base_1, base_2, ..., the first
    /// not taken (base_1 may be another base's).
    fn fresh(&mut self, base: &str) -> String {
        let mut base: String = base
            .chars()
            .map(|c| {
                let c = c.to_ascii_lowercase();
                if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        if base.is_empty() {
            base.push('v');
        }
        if !base.starts_with(|c: char| c.is_ascii_lowercase()) {
            base.insert_str(0, "v_");
        }
        let mut n = self.counter.get(&base).copied().unwrap_or(0);
        let name = loop {
            let name = if n == 0 {
                base.clone()
            } else {
                format!("{base}_{n}")
            };
            if !self.taken.contains(&name) {
                break name;
            }
            n += 1;
        };
        self.counter.insert(base, n + 1);
        self.taken.insert(name.clone());
        name
    }

    fn rule(&mut self, head: &str, body: Vec<Lit>) {
        let body: Vec<Lit> = body.into_iter().filter(|b| !is(b, "true")).collect();
        if body.iter().any(|b| is(b, "false")) {
            return;
        }
        // A loop's rule reads the step's atoms first and their negations
        // after them, as a ProbLog programmer writes it: the grounder then
        // binds the step from the atoms the round before added. A rule to
        // the next step (its head at T1) reads `step(T), T1 is T + 1`
        // first, so that a top-down grounder (ProbLog's) calls the step's
        // atoms with T bound; an atom called at an unbound step can reach
        // its own negation, which ProbLog stops at as a negative cycle.
        let mut body = if head.ends_with("(T1)") {
            body
        } else {
            let first: Vec<Lit> = body
                .iter()
                .filter(|b| at_step(b) && !b.starts_with("\\+") && !self.steps.contains(&***b))
                .cloned()
                .collect();
            let rest = body.iter().filter(|b| !first.contains(b)).cloned();
            let mut ordered = first.clone();
            ordered.extend(rest);
            ordered
        };
        if at_step(head)
            && !body.iter().any(|b| at_step(b) && !b.starts_with("\\+"))
            && let Some(step) = &self.in_loop
        {
            // The step binds T.
            body.insert(0, lit(&format!("{step}{STEP}")));
        }
        self.lines.push(if body.is_empty() {
            format!("{head}.")
        } else {
            format!("{head} :- {}.", body.join(", "))
        });
    }

    /// The conjunctions `bodies` without constants and repeats: `true` or
    /// `false` where their disjunction is constant, the one literal of a
    /// single one-literal body, else the bodies left.
    fn live(bodies: Vec<Vec<Lit>>) -> Live {
        let mut live: Vec<Vec<Lit>> = Vec::new();
        for body in bodies {
            let mut kept: Vec<Lit> = Vec::new();
            for b in body {
                if !is(&b, "true") && !kept.contains(&b) {
                    kept.push(b);
                }
            }
            if kept.iter().any(|b| is(b, "false"))
                || kept
                    .iter()
                    .any(|b| kept.iter().any(|c| complementary(b, c)))
            {
                continue;
            }
            if kept.is_empty() {
                return Live::Lit(lit("true"));
            }
            live.push(kept);
        }
        match live.len() {
            0 => Live::Lit(lit("false")),
            1 if live[0].len() == 1 => Live::Lit(live[0][0].clone()),
            _ => Live::Bodies(live),
        }
    }

    /// The literal for the disjunction of conjunctions `bodies`: a
    /// constant or one literal where [`Translator::live`] folds them to
    /// it, else `head` defined by one rule per body.
    fn define(&mut self, head: &str, bodies: Vec<Vec<Lit>>) -> Lit {
        let live = match Self::live(bodies) {
            Live::Lit(l) => return l,
            Live::Bodies(live) => live,
        };
        let head = if self.in_loop.is_some() && live.iter().flatten().any(|b| at_step(b)) {
            format!("{head}{STEP}")
        } else {
            head.to_string()
        };
        for body in live {
            self.rule(&head, body);
        }
        lit(&head)
    }

    /// `make()` once per key.
    fn memo(&mut self, key: Key, make: impl FnOnce(&mut Self) -> Res<Value>) -> Res<Value> {
        if let Some(v) = self.memos.get(&key) {
            return Ok(v.clone());
        }
        let v = make(self)?;
        self.memos.insert(key, v.clone());
        Ok(v)
    }

    fn memo_lit(&mut self, key: Key, make: impl FnOnce(&mut Self) -> Lit) -> Lit {
        match self.memo(key, |t| Ok(Value::Bool(make(t)))) {
            Ok(Value::Bool(l)) => l,
            _ => unreachable!("a literal's key holds a literal"),
        }
    }

    /// [`Translator::define`] under a fresh atom from `name`, made only
    /// where the bodies do not fold; the same bodies give the same atom.
    fn gate(&mut self, name: &str, bodies: Vec<Vec<Lit>>) -> Lit {
        let live = match Self::live(bodies) {
            Live::Lit(l) => return l,
            Live::Bodies(live) => live,
        };
        self.memo_lit(Key::Gate(live.clone()), |t| {
            let head = t.fresh(name);
            t.define(&head, live)
        })
    }

    fn xor(&mut self, a: &Lit, b: &Lit, name: &str) -> Lit {
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        self.gate(name, vec![vec![a.clone(), neg(b)], vec![neg(a), b.clone()]])
    }

    fn conj(&mut self, a: &Lit, b: &Lit, name: &str) -> Lit {
        if is(a, "false") || is(b, "false") {
            return lit("false");
        }
        if is(a, "true") || a == b {
            return b.clone();
        }
        if is(b, "true") {
            return a.clone();
        }
        let (a, b) = (a.clone(), b.clone());
        self.memo_lit(Key::And(a.clone(), b.clone()), |t| {
            let head = t.fresh(name);
            t.define(&head, vec![vec![a, b]])
        })
    }

    fn disj(&mut self, guards: &[Lit], name: &str) -> Lit {
        if guards.len() == 1 {
            return guards[0].clone();
        }
        self.memo_lit(Key::Any(guards.to_vec()), |t| {
            let head = t.fresh(name);
            t.define(&head, guards.iter().map(|g| vec![g.clone()]).collect())
        })
    }

    /// A flip `p`: a probabilistic fact, or a constant where `p` is 0 or 1.
    fn flip(&mut self, p: f64, name: &str) -> Lit {
        if p <= 0.0 || p >= 1.0 {
            return lit(if p >= 1.0 { "true" } else { "false" });
        }
        let a = self.fresh(name);
        let p = float_repr(p);
        match &self.in_loop {
            Some(step) => {
                self.lines.push(format!("{p}::{a}{STEP} :- {step}{STEP}."));
                lit(&format!("{a}{STEP}"))
            }
            None => {
                self.lines.push(format!("{p}::{a}."));
                lit(&a)
            }
        }
    }

    fn literal(&self, value: &Value, name: &str, line: usize) -> Res<Lit> {
        match value {
            Value::Bool(l) => Ok(l.clone()),
            _ => Err(fail(line, format!("{name}: a Boolean is expected"))),
        }
    }

    fn integer(&self, value: &Value, name: &str, line: usize) -> Res<Rc<Int>> {
        match value {
            Value::Int(x) => Ok(x.clone()),
            _ => Err(fail(line, format!("{name}: an integer is expected"))),
        }
    }

    fn constant(&mut self, k: u64, width: u64, line: usize) -> Res<Rc<Int>> {
        if width > MAX_WIDTH {
            return Err(fail(
                line,
                format!("int({width}): integers of more than {MAX_WIDTH} bits are not supported"),
            ));
        }
        if k >= 1 << width {
            return Err(fail(
                line,
                format!(
                    "integer constant {k} out of range for {} values",
                    1u64 << width
                ),
            ));
        }
        let v = self.memo(Key::Constant(k, width), |t| {
            let bits = (0..width)
                .map(|i| lit(if k >> i & 1 == 1 { "true" } else { "false" }))
                .collect();
            Ok(Value::Int(t.int(
                width,
                bits,
                Some(vec![k]),
                None,
                None,
                None,
            )))
        })?;
        self.integer(&v, "a constant", line)
    }

    /// A categorical over the values `0..ps.len()` in `width` bits, as
    /// Dice encodes it (`gen_discrete` in its `lib/Passes.ml`): from the
    /// most significant bit down, bit k is a flip whose probability is the
    /// mass of the values with bit k set among those that agree on the
    /// bits above it, over the mass of those values. One flip serves every
    /// setting of the bits above where that ratio is the same (a uniform
    /// over a power of two takes one flip a bit), a ratio of 0 or 1 is a
    /// constant, and the settings that serve a flip are read as a decision
    /// on the bits above, settings of no mass left out. The masses are
    /// summed exactly and each ratio is rounded once.
    fn choice(&mut self, ps: &[f64], width: u64, name: &str, line: usize) -> Res<Value> {
        let a = self.fresh(name);
        let modulus = 1usize << width;
        // prefix[i]: the exact mass of the values below i.
        let mut prefix = Vec::with_capacity(modulus + 1);
        let mut sum = BigRational::from_integer(0.into());
        prefix.push(sum.clone());
        for v in 0..modulus {
            if let Some(&p) = ps.get(v) {
                let exact = BigRational::from_float(p)
                    .ok_or_else(|| fail(line, format!("{p} is not a probability")))?;
                sum += exact;
            }
            prefix.push(sum.clone());
        }
        let zero = BigRational::from_integer(0.into());
        let mut bits: Vec<Lit> = vec![lit("false"); width as usize];
        for k in (0..width as usize).rev() {
            // The ratio at each setting u of the bits above k.
            let settings = modulus >> (k + 1);
            let mut ratio: Vec<Option<f64>> = vec![None; settings];
            for (u, r) in ratio.iter_mut().enumerate() {
                let (lo, mid, hi) = (u << (k + 1), (u << (k + 1)) + (1 << k), (u + 1) << (k + 1));
                let total = &prefix[hi] - &prefix[lo];
                if total != zero {
                    let set = &prefix[hi] - &prefix[mid];
                    *r = Some((set / total).to_f64().unwrap_or(f64::NAN));
                }
            }
            let mut flips: Vec<(f64, Lit)> = Vec::new();
            for r in ratio.iter().flatten() {
                if !flips.iter().any(|(s, _)| s == r) {
                    let f = if *r == 0.0 {
                        lit("false")
                    } else if *r == 1.0 {
                        lit("true")
                    } else {
                        self.flip(*r, &format!("{a}_f{k}"))
                    };
                    flips.push((*r, f));
                }
            }
            let mut bodies = Vec::new();
            for (r, f) in &flips {
                for mut t in paths(&ratio, &bits, k, 0, settings, *r) {
                    t.push(f.clone());
                    bodies.push(t);
                }
            }
            bits[k] = self.gate(&format!("{a}_b{k}"), bodies);
        }
        let support = ps
            .iter()
            .enumerate()
            .filter(|(_, p)| **p > 0.0)
            .map(|(v, _)| v as u64)
            .collect();
        Ok(Value::Int(self.int(
            width,
            bits,
            Some(support),
            Some(Rc::from(a.as_str())),
            None,
            Some(ps.len() as u64),
        )))
    }

    fn same_width(&self, x: &Int, y: &Int, op: &str, line: usize) -> Res<()> {
        if x.width != y.width {
            return Err(fail(
                line,
                format!(
                    "{op}: integers of {} and {} values",
                    x.modulus(),
                    y.modulus()
                ),
            ));
        }
        Ok(())
    }

    /// The values `f` takes on the values of `x` and `y`, where both are
    /// known and their pairs not too many; else `None`.
    fn results<T: Ord>(x: &Int, y: &Int, f: impl Fn(u64, u64) -> T) -> Option<BTreeSet<T>> {
        let (xs, ys) = (x.support.as_ref()?, y.support.as_ref()?);
        if xs.len() * ys.len() > SUPPORT_PAIRS {
            return None;
        }
        Some(
            xs.iter()
                .flat_map(|&a| ys.iter().map(move |&b| (a, b)))
                .map(|(a, b)| f(a, b))
                .collect(),
        )
    }

    /// The bits of `xs + ys + carry` modulo 2^len(xs), the carry rippling
    /// from bit 0.
    fn add_bits(&mut self, xs: &[Lit], ys: &[Lit], mut carry: Lit, name: &str) -> Vec<Lit> {
        let mut out = Vec::new();
        for (k, (x, y)) in xs.iter().zip(ys).enumerate() {
            let h = self.xor(x, y, name);
            out.push(self.xor(&h, &carry, name));
            if k + 1 < xs.len() {
                carry = self.gate(name, vec![vec![x.clone(), y.clone()], vec![carry, h]]);
            }
        }
        out
    }

    /// `x op y` for `+`, `-` and `*`, modulo 2^width: an adder; `x - y` as
    /// `x + ~y + 1`; `x * y` as the sum of `x` shifted by each bit of `y`
    /// that is set. Its atoms are `NAME_b...`; `NAME` is left to its query
    /// atoms.
    fn arith(
        &mut self,
        op: &'static str,
        x: Rc<Int>,
        y: Rc<Int>,
        name: &str,
        line: usize,
    ) -> Res<Value> {
        self.same_width(&x, &y, op, line)?;
        let hint = name;
        let name = format!("{name}_b");
        self.memo(Key::Arith(op, x.id, y.id), |t| {
            let modulus = u128::from(x.modulus());
            let f = |a: u64, b: u64| -> u64 {
                let (a, b) = (u128::from(a), u128::from(b));
                (match op {
                    "+" => a + b,
                    "-" => a + modulus - b,
                    _ => a * b,
                } % modulus) as u64
            };
            let support = Self::results(&x, &y, f);
            if let Some(s) = &support
                && s.len() == 1
            {
                let k = *s.first().expect("one value");
                return Ok(Value::Int(t.constant(k, x.width, line)?));
            }
            let bits = match op {
                "+" => t.add_bits(&x.bits, &y.bits, lit("false"), &name),
                "-" => {
                    let not_y: Vec<Lit> = y.bits.iter().map(|b| neg(b)).collect();
                    t.add_bits(&x.bits, &not_y, lit("true"), &name)
                }
                _ => {
                    let w = x.width as usize;
                    let mut bits = vec![lit("false"); w];
                    for (i, yi) in y.bits.iter().enumerate() {
                        if !is(yi, "false") {
                            let mut part = vec![lit("false"); i];
                            for xj in &x.bits[..w - i] {
                                part.push(t.conj(yi, xj, &name));
                            }
                            bits = t.add_bits(&bits, &part, lit("false"), &name);
                        }
                    }
                    bits
                }
            };
            let support = support.map(|s| s.into_iter().collect());
            Ok(Value::Int(t.int(
                x.width,
                bits,
                support,
                None,
                Some(Rc::from(hint)),
                None,
            )))
        })
    }

    fn shift(&mut self, op: &'static str, x: Rc<Int>, k: u64, name: &str) -> Res<Value> {
        self.memo(Key::Shift(op, x.id, k), |t| {
            let w = x.width as usize;
            let k_bits = usize::try_from(k).unwrap_or(usize::MAX).min(w);
            let (bits, support): (Vec<Lit>, Option<Vec<u64>>) = if op == "<<" {
                let mut bits = vec![lit("false"); k_bits];
                bits.extend(x.bits.iter().take(w - k_bits).cloned());
                let support = x.support.as_ref().map(|s| {
                    s.iter()
                        .map(|&a| {
                            if k >= x.width {
                                0
                            } else {
                                ((u128::from(a) << k) % u128::from(x.modulus())) as u64
                            }
                        })
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect()
                });
                (bits, support)
            } else {
                let mut bits: Vec<Lit> = x.bits[k_bits..].to_vec();
                bits.extend(std::iter::repeat_n(lit("false"), k_bits));
                let support = x.support.as_ref().map(|s| {
                    s.iter()
                        .map(|&a| if k >= 64 { 0 } else { a >> k })
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect()
                });
                (bits, support)
            };
            Ok(Value::Int(t.int(
                x.width,
                bits,
                support,
                None,
                Some(Rc::from(name)),
                None,
            )))
        })
    }

    /// The Boolean `x op y` for integers `x` and `y`: `==` bit by bit, `<`
    /// by a comparator from bit 0 up (`x < y` where its bit is below y's,
    /// or equal and `x < y` below it); a constant where every pair of
    /// their values gives the same.
    fn compare(&mut self, op: &str, x: Rc<Int>, y: Rc<Int>, name: &str, line: usize) -> Res<Value> {
        self.same_width(&x, &y, op, line)?;
        let negated = |v: Res<Value>| -> Res<Value> {
            match v? {
                Value::Bool(l) => Ok(Value::Bool(neg(&l))),
                _ => unreachable!("a comparison is a Boolean"),
            }
        };
        let op: &'static str = match op {
            "!=" => return negated(self.compare("==", x, y, name, line)),
            ">" => return self.compare("<", y, x, name, line),
            ">=" => return self.compare("<=", y, x, name, line),
            "<=" => return negated(self.compare("<", y, x, name, line)),
            "==" => "==",
            _ => "<",
        };
        self.memo(Key::Arith(op, x.id, y.id), |t| {
            if let Some(outcomes) = Self::results(&x, &y, |a, b| compare(op, a, b))
                && outcomes.len() == 1
            {
                return Ok(boolean(*outcomes.first().expect("one outcome")));
            }
            let bname = format!("{name}_b");
            let same: Vec<Lit> = x
                .bits
                .iter()
                .zip(&y.bits)
                .map(|(a, b)| neg(&t.xor(a, b, &bname)))
                .collect();
            if op == "==" {
                return Ok(Value::Bool(t.gate(name, vec![same])));
            }
            let mut lt = lit("false");
            for ((a, b), eq) in x.bits.iter().zip(&y.bits).zip(same) {
                lt = t.gate(name, vec![vec![neg(a), b.clone()], vec![eq, lt]]);
            }
            Ok(Value::Bool(lt))
        })
    }

    /// The literal for `l == r` on Boolean literals.
    fn equal(&mut self, l: &Lit, r: &Lit, name: &str) -> Lit {
        if is(l, "true") || is(l, "false") {
            return if is(l, "true") { r.clone() } else { neg(r) };
        }
        if is(r, "true") || is(r, "false") {
            return if is(r, "true") { l.clone() } else { neg(l) };
        }
        let head = self.fresh(name);
        self.define(
            &head,
            vec![vec![l.clone(), r.clone()], vec![neg(l), neg(r)]],
        )
    }

    fn binary(&mut self, op: &str, l: Value, r: Value, name: &str, line: usize) -> Res<Value> {
        if op == "::" {
            return self.cons(l, r, name, line);
        }
        if matches!(op, "/" | "%") {
            return Err(fail(line, format!("Dice's operator {op} is not supported")));
        }
        if matches!(op, "&&" | "||") {
            let (l, r) = (self.literal(&l, name, line)?, self.literal(&r, name, line)?);
            if op == "&&" {
                return Ok(Value::Bool(self.conj(&l, &r, name)));
            }
            if is(&l, "true") || is(&r, "true") {
                return Ok(boolean(true));
            }
            if is(&l, "false") || l == r {
                return Ok(Value::Bool(r));
            }
            if is(&r, "false") {
                return Ok(Value::Bool(l));
            }
            return Ok(Value::Bool(self.memo_lit(
                Key::Or(l.clone(), r.clone()),
                |t| {
                    let head = t.fresh(name);
                    t.define(&head, vec![vec![l], vec![r]])
                },
            )));
        }
        if matches!(op, "^" | "<=>") || (matches!(op, "==" | "!=") && l.shape() == 'b') {
            // Boolean equality and its negation; Dice reads == and != on
            // integers only, this reader on Booleans too.
            let (a, b) = (self.literal(&l, name, line)?, self.literal(&r, name, line)?);
            let e = self.equal(&a, &b, name);
            return Ok(Value::Bool(if matches!(op, "^" | "!=") {
                neg(&e)
            } else {
                e
            }));
        }
        if l.shape() == 't' || r.shape() == 't' {
            return Err(fail(line, format!("{name}: {op} on a tuple")));
        }
        let (x, y) = (self.integer(&l, name, line)?, self.integer(&r, name, line)?);
        match op {
            "+" => self.arith("+", x, y, name, line),
            "-" => self.arith("-", x, y, name, line),
            "*" => self.arith("*", x, y, name, line),
            _ => self.compare(op, x, y, name, line),
        }
    }

    /// An observation of `c` on `path`: it holds when the branch is not
    /// taken or the condition is true; in a loop's body, at each step.
    fn observe(&mut self, c: Lit, path: &[Lit]) {
        let a = self.fresh("obs") + if self.in_loop.is_some() { STEP } else { "" };
        let mut body = path.to_vec();
        body.push(c.clone());
        self.rule(&a, body);
        for (i, p) in path.iter().enumerate() {
            let mut body = path[..i].to_vec();
            body.push(neg(p));
            self.rule(&a, body);
        }
        if path.is_empty() && (is(&c, "true") || is(&c, "false")) {
            // An impossible observation: evidence on an atom defined false,
            // which ProbLog and tddlog find inconsistent, once a program.
            const FALSE: &str = "false_observation :- fail.";
            if is(&c, "false") && !self.lines.iter().any(|l| l == FALSE) {
                self.lines.push(FALSE.into());
                self.lines.push("evidence(false_observation, true).".into());
            }
            return;
        }
        let a = lit(&a);
        if self.in_loop.is_some() {
            self.observed.push(a);
        } else {
            self.evidence.push(a);
        }
    }

    /// Translate expression `e` under the variables `env` and the path
    /// condition `path` (the literals that hold when it is evaluated);
    /// `name` is the hint for the atoms it makes.
    fn eval(&mut self, e: &'a Expr<'a>, env: &Env<'a>, path: &[Lit], name: &str) -> Res<Value> {
        let line = e.line;
        match &e.kind {
            Kind::Let(binds, body) => {
                let mut env = env.enter();
                for (x, v) in binds {
                    let value = self.eval(v, &env, path, x)?;
                    env.bind(x, value);
                }
                self.eval(body, &env, path, name)
            }
            Kind::Var(x) => env
                .get(x)
                .cloned()
                .ok_or_else(|| fail(line, format!("unbound variable {x}"))),
            Kind::Bool(b) => Ok(boolean(*b)),
            Kind::Flip(p) => Ok(Value::Bool(self.flip(*p, name))),
            Kind::Discrete(ps) => {
                let ps = discrete_probs(ps);
                self.choice(&ps, bits_for(ps.len()), name, line)
            }
            &Kind::Uniform(width, lo, hi) => self.uniform(width, lo, hi, name, line),
            &Kind::Int(width, k) => Ok(Value::Int(self.constant(k, width, line)?)),
            Kind::Tuple(a, b) => {
                let a = self.eval(a, env, path, &format!("{name}_1"))?;
                let b = self.eval(b, env, path, &format!("{name}_2"))?;
                Ok(Value::Tuple(Rc::new((a, b))))
            }
            Kind::Fst(x) | Kind::Snd(x) => match self.eval(x, env, path, name)? {
                Value::Tuple(t) => Ok(if matches!(e.kind, Kind::Fst(_)) {
                    t.0.clone()
                } else {
                    t.1.clone()
                }),
                _ => Err(fail(line, format!("{name}: a tuple is expected"))),
            },
            &Kind::Binomial(width, n, p) => self.binomial(width, n, p, name, line),
            &Kind::NthBit(width, i, ref x) => {
                let v = self.eval(x, env, path, name)?;
                let x = self.integer(&v, name, line)?;
                if x.width != width || i >= width {
                    return Err(fail(
                        line,
                        format!(
                            "nth_bit(int({width}, {i}), x): x has {} bits, bit {i} of {width} is asked",
                            x.width
                        ),
                    ));
                }
                Ok(Value::Bool(x.bits[(width - 1 - i) as usize].clone()))
            }
            Kind::List(items) => {
                if items.len() > LIST_LENGTH {
                    return Err(fail(
                        line,
                        format!(
                            "a list of {} elements: Dice's lists hold at most {LIST_LENGTH}",
                            items.len()
                        ),
                    ));
                }
                let mut elems = Vec::new();
                for (k, x) in items.iter().enumerate() {
                    elems.push(self.eval(x, env, path, &format!("{name}_{}", k + 1))?);
                }
                let n = self.constant(items.len() as u64, list_width(), line)?;
                let t = type_of(&elems[0]);
                self.list(n, elems, &t, name, line)
            }
            Kind::Nil(t) => match t {
                Type::List(_) => self.default(Some(t), name, line),
                _ => Err(fail(
                    line,
                    format!("[] : {t:?}: an empty list has a list type"),
                )),
            },
            Kind::Head(x) | Kind::Tail(x) | Kind::Length(x) => {
                self.list_op(&e.kind, x, env, path, name, line)
            }
            Kind::Not(x) => {
                let v = self.eval(x, env, path, name)?;
                Ok(Value::Bool(neg(&self.literal(&v, name, line)?)))
            }
            Kind::Bin(..) | Kind::Shift(..) => self.chain(e, env, path, name),
            Kind::Observe(x) => {
                let v = self.eval(x, env, path, name)?;
                let c = self.literal(&v, name, line)?;
                self.observe(c, path);
                Ok(boolean(true))
            }
            Kind::If(arms, otherwise) => self.conditional(arms, otherwise, env, path, name, line),
            Kind::Call(f, args) => {
                let fun = self.fun(f, args.len(), line)?;
                let mut values = Vec::new();
                for (p, x) in fun.params.iter().zip(args) {
                    values.push(self.eval(x, env, path, &format!("{f}_{p}"))?);
                }
                self.call(fun, values, path, name, line)
            }
            Kind::Iterate(f, init, k) => {
                let init = self.eval(init, env, path, name)?;
                self.iterate(f, init, k, path, name, line)
            }
        }
    }

    fn uniform(&mut self, width: u64, lo: u64, hi: u64, name: &str, line: usize) -> Res<Value> {
        if width > MAX_UNIFORM_WIDTH {
            return Err(fail(
                line,
                format!(
                    "uniform({width}, {lo}, {hi}): a uniform over more than 2^{MAX_UNIFORM_WIDTH} values is not supported"
                ),
            ));
        }
        if !(lo < hi && hi <= 1 << width) {
            return Err(fail(
                line,
                format!("uniform({width}, {lo}, {hi}): empty or out of range"),
            ));
        }
        let p = 1.0 / (hi - lo) as f64;
        let ps: Vec<f64> = (0..hi).map(|v| if v < lo { 0.0 } else { p }).collect();
        self.choice(&ps, width, name, line)
    }

    fn list_op(
        &mut self,
        kind: &Kind<'a>,
        x: &'a Expr<'a>,
        env: &Env<'a>,
        path: &[Lit],
        name: &str,
        line: usize,
    ) -> Res<Value> {
        let v = self.eval(x, env, path, name)?;
        let (n, elems) = list_parts(&v, name, line)?;
        if let Kind::Length(_) = kind {
            return Ok(Value::Int(n));
        }
        // Dice's head and tail of an empty list observe false.
        let zero = self.constant(0, list_width(), line)?;
        let empty = self.compare("==", n.clone(), zero, name, line)?;
        let empty = self.literal(&empty, name, line)?;
        if !is(&empty, "false") {
            self.observe(neg(&empty), path);
        }
        if let Kind::Head(_) = kind {
            return Ok(elems[0].clone());
        }
        let one = self.constant(1, list_width(), line)?;
        let n = self.arith("-", n, one, name, line)?;
        let n = self.integer(&n, name, line)?;
        let t = type_of(&elems[0]);
        self.list(n, elems[1..].to_vec(), &t, name, line)
    }

    /// A binary operation or a shift whose left operand is a chain of
    /// them, `((a op b) op c) ...`, evaluated left to right without
    /// recursing down the chain: programs write conjunctions of thousands.
    fn chain(&mut self, e: &'a Expr<'a>, env: &Env<'a>, path: &[Lit], name: &str) -> Res<Value> {
        let mut spine = Vec::new();
        let mut first = e;
        while let Kind::Bin(_, l, _) | Kind::Shift(_, l, _) = &first.kind {
            spine.push(first);
            first = l;
        }
        let mut acc = self.eval(first, env, path, name)?;
        for node in spine.into_iter().rev() {
            acc = match &node.kind {
                Kind::Bin(op, _, r) => {
                    let r = self.eval(r, env, path, name)?;
                    self.binary(op, acc, r, name, node.line)?
                }
                Kind::Shift(op, _, k) => {
                    let x = self.integer(&acc, name, node.line)?;
                    let op = if *op == "<<" { "<<" } else { ">>" };
                    self.shift(op, x, *k, name)?
                }
                _ => unreachable!("the spine holds operations"),
            };
        }
        Ok(acc)
    }

    /// A chain `if c0 then e0 else if c1 then e1 ... else en`, taken as
    /// one: branch i is guarded by g_i, that c_i holds and no earlier
    /// condition did, an atom shared by everything the branch computes,
    /// and the branches' values are merged once. Within a branch, a
    /// variable the guard compares with a constant takes only the values
    /// the guard allows (a constant where it allows one): what the branch
    /// computes is read only under its guard.
    fn conditional(
        &mut self,
        arms: &'a [(Expr<'a>, Expr<'a>)],
        otherwise: &'a Expr<'a>,
        env: &Env<'a>,
        path: &[Lit],
        name: &str,
        line: usize,
    ) -> Res<Value> {
        let unmet = || fail(line, "a branch no value of its variables reaches");
        let (cname, gname) = (format!("{name}_c"), format!("{name}_g"));
        let mut branches: Vec<(Lit, Value)> = Vec::new();
        let mut reach = lit("true");
        let mut known: Facts<'a> = Some(BTreeMap::new());
        for (i, (cond, then)) in arms.iter().enumerate() {
            let at = self.refine(env, &known)?.ok_or_else(unmet)?;
            let v = self.eval(cond, &at, &within(path, &reach), &cname)?;
            let c = self.literal(&v, name, line)?;
            let guard = self.conj(&reach, &c, &gname);
            let facts = self.facts(cond, &at, true);
            let inside = self.refine(&at, &facts)?;
            if !is(&guard, "false")
                && let Some(inside) = inside
            {
                let v = self.eval(
                    then,
                    &inside,
                    &within(path, &guard),
                    &format!("{name}_then"),
                )?;
                branches.push((guard, v));
            }
            reach = self.conj(&reach, &neg(&c), &gname);
            known = meet(known, self.facts(cond, &at, false));
            if is(&reach, "false") || known.is_none() {
                break;
            }
            if i + 1 == arms.len() {
                let at = self.refine(env, &known)?.ok_or_else(unmet)?;
                let v = self.eval(
                    otherwise,
                    &at,
                    &within(path, &reach),
                    &format!("{name}_else"),
                )?;
                branches.push((reach.clone(), v));
            }
        }
        if branches.is_empty() {
            return Err(unmet());
        }
        self.merge(branches, name, line)
    }

    /// The parts of `fname(state)` as a loop's step: its atoms of the
    /// step, its observations in `observed`.
    fn body(
        &mut self,
        fname: &'a str,
        state: Value,
        path: &[Lit],
        name: &str,
        step: &str,
        line: usize,
    ) -> Res<Vec<Value>> {
        let fun = self.fun(fname, 1, line)?;
        self.in_loop = Some(step.to_string());
        self.observed.clear();
        let out = self.call(fun, vec![state.clone()], path, name, line);
        self.in_loop = None;
        let (ends, starts) = (out?.flat(), state.flat());
        let same = |p: &Value, q: &Value| match (p, q) {
            (Value::Int(p), Value::Int(q)) => p.width == q.width,
            _ => p.shape() == q.shape(),
        };
        if ends.len() != starts.len() || starts.iter().zip(&ends).any(|(p, q)| !same(p, q)) {
            return Err(fail(
                line,
                format!("{fname}: the loop step returns a value of another type than its state"),
            ));
        }
        Ok(ends)
    }

    fn mark(&self) -> Mark {
        Mark {
            lines: self.lines.len(),
            evidence: self.evidence.len(),
            counter: self.counter.clone(),
            taken: self.taken.clone(),
            memos: self.memos.clone(),
        }
    }

    /// Back to what the translation was at `mark`.
    fn undo(&mut self, mark: Mark) {
        self.lines.truncate(mark.lines);
        self.evidence.truncate(mark.evidence);
        self.counter = mark.counter;
        self.taken = mark.taken;
        self.memos = mark.memos;
    }

    /// `iterate(fname, init, k)` inside a loop's step, as Dice expands it
    /// (its `lib/Passes.ml`, `expand_iter`): `fname` called k times in a
    /// row, from `init`, within the step. Each call returns a value of
    /// the state's type.
    fn expand(
        &mut self,
        fname: &'a str,
        init: Value,
        k: &Count<'_>,
        path: &[Lit],
        name: &str,
        line: usize,
    ) -> Res<Value> {
        let &Count::Number(k) = k else {
            return Err(fail(
                line,
                format!("an iterate inside a loop needs its count `{k}` given as an argument"),
            ));
        };
        let fun = self.fun(fname, 1, line)?;
        let t = type_of(&init);
        let mut value = init;
        for _ in 0..k {
            value = self.call(fun, vec![value], path, name, line)?;
            if type_of(&value) != t {
                return Err(fail(
                    line,
                    format!(
                        "{fname}: the loop step returns a value of another type than its state"
                    ),
                ));
            }
        }
        Ok(value)
    }

    /// `iterate(fname, init, k)` as recursion over a step index, which the
    /// grounder unrolls: the count k a fact `steps(k)` (k a number or a
    /// place, which an argument fills), fname's body once, under `step(T)`
    /// for T from 0 to k - 1, its flips probabilistic facts of the step
    /// and its atoms of the step where they read one (`name(T)`). Each
    /// literal of the state (a Boolean, a bit of an integer) is an atom s
    /// of its own, `s(0)` the initial state's literal and `s(T1)`, with
    /// `T1 is T + 1`, the body's at step T, and the value is the state at
    /// k, `s(k)`; a literal the step keeps (the body returns the initial state's literal where the
    /// state holds it, as the key of the Caesar cipher) is no atom but
    /// itself. Which literals are kept, and the values the state's
    /// integers take (for the comparisons and the conditions that they
    /// decide), are a fixpoint over the step: the initial state's, and
    /// what the body makes of them, until no evaluation of the body adds
    /// to them (the evaluations before the last are undone). An
    /// integer's values that grow in more than twice as many evaluations
    /// as the integer has bits are widened to unknown: values that at
    /// least double in each fill the integer's range in as many
    /// evaluations as it has bits, so values still growing after twice
    /// that grow by a few in each, as a counter's do, and would take an
    /// evaluation per value. The step's observations hold at every step:
    /// `failed` holds where one does not at some step, and the evidence
    /// is that it is false.
    fn iterate(
        &mut self,
        fname: &'a str,
        init: Value,
        k: &Count<'_>,
        path: &[Lit],
        name: &str,
        line: usize,
    ) -> Res<Value> {
        if self.in_loop.is_some() {
            return self.expand(fname, init, k, path, name, line);
        }
        let start = init.flat();
        let mut state = Loop {
            kept: start.iter().map(|p| vec![true; p.bits().len()]).collect(),
            values: start
                .iter()
                .map(|p| match p {
                    Value::Int(x) => x.support.clone(),
                    _ => None,
                })
                .collect(),
            atoms: start.iter().map(|p| vec![None; p.bits().len()]).collect(),
            hints: init.flat_names(fname),
            start,
        };
        let bases = state.hints.clone();
        let mut growths = vec![0; state.start.len()];
        let mut step: Option<String> = None;
        loop {
            for (i, start) in state.start.iter().enumerate() {
                for (j, atom) in state.atoms[i].iter_mut().enumerate() {
                    if !state.kept[i][j] && atom.is_none() {
                        let base = match start {
                            Value::Bool(_) => bases[i].clone(),
                            _ => format!("{}_b{j}", bases[i]),
                        };
                        *atom = Some(self.fresh(&base));
                    }
                }
            }
            if step.is_none() {
                step = Some(self.fresh("step"));
            }
            let l = step.clone().expect("a step predicate");
            self.steps.insert(format!("{l}{STEP}"));
            let mark = self.mark();
            let at = self.state(&state, &init, STEP);
            let ends = self.body(fname, at, path, name, &l, line);
            self.undo(mark);
            let ends = ends?;
            let mut grew = false;
            for (i, (p, q)) in state.start.iter().zip(&ends).enumerate() {
                for (j, (a, b)) in p.bits().iter().zip(q.bits()).enumerate() {
                    if state.kept[i][j] && *a != b {
                        state.kept[i][j] = false;
                        grew = true;
                    }
                }
                if let (Value::Int(x), Some(values), Value::Int(q)) = (p, &state.values[i], q) {
                    let more = q.support.as_ref().map(|s| {
                        values
                            .iter()
                            .chain(s)
                            .copied()
                            .collect::<BTreeSet<u64>>()
                            .into_iter()
                            .collect::<Vec<u64>>()
                    });
                    if more.as_ref() != Some(values) {
                        growths[i] += 1;
                        state.values[i] = more.filter(|_| growths[i] <= 2 * x.width);
                        grew = true;
                    }
                }
            }
            if !grew {
                break;
            }
        }
        let l = step.expect("a step predicate");
        // The count, a fact the recursion over the steps reads.
        let count = self.fresh("steps");
        self.lines.push(format!("{count}({k})."));
        for (i, start) in state.start.iter().enumerate() {
            for (j, b) in start.bits().into_iter().enumerate() {
                if !state.kept[i][j] {
                    let head = format!("{}(0)", state.atom(i, j));
                    self.rule(&head, vec![b]);
                }
            }
        }
        // Steps 0 to the count less one: none for a count of 0.
        self.lines.push(format!("{l}(0) :- {count}(N), N > 0."));
        self.lines.push(format!(
            "{l}(T1) :- {l}(T), {count}(N), T1 is T + 1, T1 < N."
        ));
        let at = self.state(&state, &init, STEP);
        let ends = self.body(fname, at, path, name, &l, line)?;
        for (i, q) in ends.iter().enumerate() {
            if let Value::Int(q) = q
                && let Some(h) = q.name.as_ref().or(q.hint.as_ref())
            {
                state.hints[i] = h.to_string();
            }
            for (j, b) in q.bits().into_iter().enumerate() {
                if !state.kept[i][j] {
                    let head = format!("{}(T1)", state.atom(i, j));
                    self.rule(&head, vec![lit(&format!("{l}(T)")), lit("T1 is T + 1"), b]);
                }
            }
        }
        if !self.observed.is_empty() {
            let failed = self.fresh("failed");
            for o in std::mem::take(&mut self.observed) {
                self.rule(&failed, vec![lit(&format!("{l}(T)")), neg(&o)]);
            }
            self.evidence.push(neg(&failed));
        }
        // The state after the loop, read at the count itself (a rule that
        // read it after `steps(N)` kept the grounder from folding its
        // copies of the last steps' atoms).
        Ok(self.state(&state, &init, &format!("({k})")))
    }

    /// The state of a loop at `index` (`(T)`, or the last step's).
    fn state(&mut self, state: &Loop, init: &Value, index: &str) -> Value {
        let mut parts = Vec::new();
        for (i, p) in state.start.iter().enumerate() {
            if state.kept[i].iter().all(|&k| k) {
                parts.push(p.clone());
                continue;
            }
            let ls: Vec<Lit> = p
                .bits()
                .into_iter()
                .enumerate()
                .map(|(j, b)| {
                    if state.kept[i][j] {
                        b
                    } else {
                        lit(&format!("{}{index}", state.atom(i, j)))
                    }
                })
                .collect();
            parts.push(match p {
                Value::Int(x) => Value::Int(self.int(
                    x.width,
                    ls,
                    state.values[i].clone(),
                    None,
                    Some(Rc::from(state.hints[i].as_str())),
                    None,
                )),
                _ => Value::Bool(ls[0].clone()),
            });
        }
        init.rebuild(&mut parts.into_iter())
    }

    fn fun(&self, fname: &'a str, arity: usize, line: usize) -> Res<&'a Fun<'a>> {
        let Some(&(fun, _)) = self.funs.get(fname) else {
            return Err(fail(line, format!("unknown function {fname}")));
        };
        if fun.params.len() != arity {
            return Err(fail(
                line,
                format!(
                    "{fname}: {} parameters, {arity} arguments",
                    fun.params.len()
                ),
            ));
        }
        Ok(fun)
    }

    fn call(
        &mut self,
        fun: &'a Fun<'a>,
        args: Vec<Value>,
        path: &[Lit],
        name: &str,
        line: usize,
    ) -> Res<Value> {
        let (_, recursive) = self.funs[fun.name];
        let depth = self.active.get(fun.name).copied().unwrap_or(0);
        if recursive && depth > RECURSION_LIMIT {
            // Dice's last copy of a recursive function: `observe false` on
            // this path, and the return type's default value.
            self.observe(lit("false"), path);
            return self.default(fun.ret.as_ref(), fun.name, line);
        }
        self.active.insert(fun.name, depth + 1);
        let mut env = Env::default().enter();
        for (p, a) in fun.params.iter().zip(args) {
            env.bind(p, a);
        }
        let out = self.eval(&fun.body, &env, path, name);
        self.active.insert(fun.name, depth);
        out
    }

    fn default(&mut self, t: Option<&Type>, fname: &str, line: usize) -> Res<Value> {
        match t {
            None => Err(fail(
                line,
                format!("{fname}: a recursive function needs a return type"),
            )),
            Some(Type::Bool) => Ok(boolean(false)),
            Some(Type::Int(w)) => Ok(Value::Int(self.constant(0, *w, line)?)),
            Some(Type::Tuple(a, b)) => {
                let a = self.default(Some(a), fname, line)?;
                let b = self.default(Some(b), fname, line)?;
                Ok(Value::Tuple(Rc::new((a, b))))
            }
            Some(Type::List(t)) => {
                // The empty list: length 0, every element the default.
                let d = self.default(Some(t), fname, line)?;
                let zero = self.constant(0, list_width(), line)?;
                self.list(zero, Vec::new(), &type_of(&d), fname, line)
            }
        }
    }

    /// The list of length `n` whose first elements are `elems`, each of
    /// type `t`, and its others `t`'s default.
    fn list(
        &mut self,
        n: Rc<Int>,
        mut elems: Vec<Value>,
        t: &Type,
        name: &str,
        line: usize,
    ) -> Res<Value> {
        if let Some(x) = elems.iter().map(type_of).find(|x| x != t) {
            return Err(fail(
                line,
                format!("{name}: a list of elements of types {t:?} and {x:?}"),
            ));
        }
        while elems.len() < LIST_LENGTH {
            elems.push(self.default(Some(t), name, line)?);
        }
        let mut rest = elems.pop().expect("an element");
        while let Some(x) = elems.pop() {
            rest = Value::Tuple(Rc::new((x, rest)));
        }
        Ok(Value::Tuple(Rc::new((Value::Int(n), rest))))
    }

    /// `x :: xs`, as Dice makes it: x, then the elements of xs but its
    /// last; the length xs's plus one, unless xs is full.
    fn cons(&mut self, x: Value, xs: Value, name: &str, line: usize) -> Res<Value> {
        let (n, mut elems) = list_parts(&xs, name, line)?;
        let t = type_of(&elems[0]);
        let full = self.constant(LIST_LENGTH as u64, list_width(), line)?;
        let one = self.constant(1, list_width(), line)?;
        let at_full = self.compare("==", n.clone(), full.clone(), name, line)?;
        let at_full = self.literal(&at_full, name, line)?;
        let more = self.arith("+", n, one, name, line)?;
        let n = self.select(&at_full, Value::Int(full), more, name, line)?;
        let n = self.integer(&n, name, line)?;
        elems.pop();
        elems.insert(0, x);
        self.list(n, elems, &t, name, line)
    }

    fn select(&mut self, cond: &Lit, a: Value, b: Value, name: &str, line: usize) -> Res<Value> {
        match &**cond {
            "true" => Ok(a),
            "false" => Ok(b),
            _ => self.merge(vec![(cond.clone(), a), (neg(cond), b)], name, line),
        }
    }

    /// `binomial(width, n, p)`, as Dice writes it: from 0, `n` times
    /// `if flip p then x + 1 else x`.
    fn binomial(&mut self, width: u64, n: u64, p: f64, name: &str, line: usize) -> Res<Value> {
        let mut x = self.constant(0, width, line)?;
        if n >= x.modulus() {
            return Err(fail(
                line,
                format!(
                    "binomial({width}, {n}, p): {n} out of range for {} values",
                    x.modulus()
                ),
            ));
        }
        let one = self.constant(1, width, line)?;
        for _ in 0..n {
            let f = self.flip(p, name);
            let more = self.arith("+", x.clone(), one.clone(), name, line)?;
            let v = self.select(&f, more, Value::Int(x), name, line)?;
            x = self.integer(&v, name, line)?;
        }
        Ok(Value::Int(x))
    }

    /// What condition `e`, or its negation where not `positive`, says of
    /// variables compared with constants.
    fn facts(&self, e: &'a Expr<'a>, env: &Env<'a>, positive: bool) -> Facts<'a> {
        let one = |var: &'a str, allowed: BTreeSet<u64>| Some(BTreeMap::from([(var, allowed)]));
        match &e.kind {
            Kind::Not(x) => self.facts(x, env, !positive),
            Kind::Var(x) if matches!(env.get(x), Some(Value::Bool(_))) => {
                one(x, BTreeSet::from([u64::from(positive)]))
            }
            Kind::Bin(op, l, r) => {
                let (op, l, r) = (*op, &**l, &**r);
                if (op == "&&" && positive) || (op == "||" && !positive) {
                    return meet(self.facts(l, env, positive), self.facts(r, env, positive));
                }
                if matches!(op, "&&" | "||") {
                    return join(self.facts(l, env, positive), self.facts(r, env, positive));
                }
                let Some(m) = mirror(op) else {
                    return Some(BTreeMap::new());
                };
                let (op, l, r) = match (&l.kind, &r.kind) {
                    (Kind::Int(..), Kind::Var(_)) => (m, r, l),
                    _ => (op, l, r),
                };
                if let (Kind::Var(x), &Kind::Int(_, k)) = (&l.kind, &r.kind)
                    && let Some(Value::Int(v)) = env.get(x)
                    && let Some(support) = &v.support
                {
                    return one(
                        x,
                        support
                            .iter()
                            .copied()
                            .filter(|&a| compare(op, a, k) == positive)
                            .collect(),
                    );
                }
                Some(BTreeMap::new())
            }
            _ => Some(BTreeMap::new()),
        }
    }

    /// `env` with each variable in `known` restricted to the values it
    /// allows; `None` if one allows none of its values.
    fn refine(&mut self, env: &Env<'a>, known: &Facts<'a>) -> Res<Option<Env<'a>>> {
        let Some(known) = known else {
            return Ok(None);
        };
        let mut out = env.enter();
        for (&var, allowed) in known {
            let Some(x) = env.get(var) else {
                continue;
            };
            match x {
                Value::Bool(l) => {
                    let left: Vec<bool> = [true, false]
                        .into_iter()
                        .filter(|&b| {
                            allowed.contains(&u64::from(b))
                                && !is(l, if b { "false" } else { "true" })
                        })
                        .collect();
                    match left.as_slice() {
                        [] => return Ok(None),
                        [b] => out.bind(var, boolean(*b)),
                        _ => {}
                    }
                }
                Value::Int(x) => {
                    let Some(have) = &x.support else {
                        continue;
                    };
                    let support: Vec<u64> = have
                        .iter()
                        .copied()
                        .filter(|a| allowed.contains(a))
                        .collect();
                    if support.is_empty() {
                        return Ok(None);
                    }
                    if support.len() == 1 {
                        let k = self.constant(support[0], x.width, 0)?;
                        out.bind(var, Value::Int(k));
                    } else if support.len() < have.len() {
                        // The bits on which the values left agree are
                        // constants.
                        let bits: Vec<Lit> = x
                            .bits
                            .iter()
                            .enumerate()
                            .map(|(k, b)| {
                                let ones = support.iter().filter(|&&a| a >> k & 1 == 1).count();
                                if ones > 0 && ones < support.len() {
                                    b.clone()
                                } else {
                                    lit(if support[0] >> k & 1 == 1 {
                                        "true"
                                    } else {
                                        "false"
                                    })
                                }
                            })
                            .collect();
                        let x = x.clone();
                        let v = self.memo(Key::Restrict(x.id, support.clone()), |t| {
                            Ok(Value::Int(t.int(
                                x.width,
                                bits,
                                Some(support),
                                x.name.clone(),
                                x.hint.clone(),
                                Some(x.size),
                            )))
                        })?;
                        out.bind(var, v);
                    }
                }
                Value::Tuple(_) => {}
            }
        }
        Ok(Some(out))
    }

    /// The value of a conditional whose branch values are `branches`,
    /// pairs (guard, value) whose guards are exclusive and exhaustive where
    /// the conditional is evaluated. Equal values are merged first, under
    /// the disjunction of their guards.
    fn merge(&mut self, branches: Vec<(Lit, Value)>, name: &str, line: usize) -> Res<Value> {
        let value = branches[0].1.clone();
        if branches.iter().any(|(_, v)| v.shape() != value.shape()) {
            return Err(fail(line, format!("{name}: branches of different shapes")));
        }
        if let Value::Tuple(_) = value {
            let part = |first: bool| -> Vec<(Lit, Value)> {
                branches
                    .iter()
                    .map(|(g, v)| match v {
                        Value::Tuple(t) => {
                            (g.clone(), if first { t.0.clone() } else { t.1.clone() })
                        }
                        _ => unreachable!("every branch is a tuple"),
                    })
                    .collect()
            };
            let a = self.merge(part(true), &format!("{name}_1"), line)?;
            let b = self.merge(part(false), &format!("{name}_2"), line)?;
            return Ok(Value::Tuple(Rc::new((a, b))));
        }
        // The values, each with its guards, in the order of the branches.
        let mut groups: Vec<(Value, Vec<Lit>)> = Vec::new();
        for (g, v) in branches {
            let same = |w: &Value| match (w, &v) {
                (Value::Bool(a), Value::Bool(b)) => a == b,
                (Value::Int(a), Value::Int(b)) => a.id == b.id,
                _ => false,
            };
            match groups.iter_mut().find(|(w, _)| same(w)) {
                Some((_, gs)) => gs.push(g),
                None => groups.push((v, vec![g])),
            }
        }
        if groups.len() == 1 {
            return Ok(value);
        }
        // The guards are exclusive and exhaustive, so the value with the
        // most of them is guarded by the negation of the others' where
        // those are fewer.
        groups.sort_by_key(|(_, gs)| std::cmp::Reverse(gs.len()));
        let gname = format!("{name}_g");
        let rest: Vec<Lit> = groups[1..]
            .iter()
            .flat_map(|(_, gs)| gs.iter().cloned())
            .collect();
        let first = if rest.len() < groups[0].1.len() {
            neg(&self.disj(&rest, &gname))
        } else {
            self.disj(&groups[0].1, &gname)
        };
        let mut parts = vec![(groups[0].0.clone(), first)];
        for (v, gs) in &groups[1..] {
            let g = self.disj(gs, &gname);
            parts.push((v.clone(), g));
        }
        if let Value::Bool(_) = value {
            let head = self.fresh(name);
            let bodies = parts
                .iter()
                .map(|(v, g)| vec![g.clone(), v.bits()[0].clone()])
                .collect();
            return Ok(Value::Bool(self.define(&head, bodies)));
        }
        let Value::Int(first) = &value else {
            unreachable!("an integer");
        };
        self.merge_ints(parts, first, name, line)
    }

    fn merge_ints(
        &mut self,
        parts: Vec<(Value, Lit)>,
        first: &Int,
        name: &str,
        line: usize,
    ) -> Res<Value> {
        let ints: Vec<(Rc<Int>, Lit)> = parts
            .into_iter()
            .map(|(v, g)| match v {
                Value::Int(x) => (x, g),
                _ => unreachable!("every branch is an integer"),
            })
            .collect();
        for (x, _) in &ints {
            self.same_width(first, x, "if", line)?;
        }
        let key = Key::Merge(ints.iter().map(|(x, g)| (g.clone(), x.id)).collect());
        let width = first.width;
        self.memo(key, |t| {
            let support = ints
                .iter()
                .map(|(x, _)| x.support.clone())
                .collect::<Option<Vec<_>>>()
                .map(|all| {
                    all.into_iter()
                        .flatten()
                        .collect::<BTreeSet<u64>>()
                        .into_iter()
                        .collect()
                });
            let bname = format!("{name}_b");
            let mut bits = Vec::new();
            for (k, first) in ints[0].0.bits.iter().enumerate() {
                let distinct: BTreeSet<&Lit> = ints.iter().map(|(x, _)| &x.bits[k]).collect();
                bits.push(if distinct.len() == 1 {
                    first.clone()
                } else {
                    t.gate(
                        &bname,
                        ints.iter()
                            .map(|(x, g)| vec![g.clone(), x.bits[k].clone()])
                            .collect(),
                    )
                });
            }
            let size = ints.iter().map(|(x, _)| x.size).max();
            Ok(Value::Int(t.int(
                width,
                bits,
                support,
                None,
                Some(Rc::from(name)),
                size,
            )))
        })
    }

    fn queries(&mut self, v: &Value, name: &str) {
        match v {
            Value::Int(x) => {
                // Each value's atom: the conjunction of its bits' literals.
                let q = match x.name.as_ref() {
                    Some(n) => n.to_string(),
                    None => self.fresh(x.hint.as_deref().unwrap_or(name)),
                };
                for k in 0..x.size {
                    let atom = format!("{q}({k})");
                    if x.allows(k) {
                        let body = x
                            .bits
                            .iter()
                            .enumerate()
                            .map(|(i, b)| if k >> i & 1 == 1 { b.clone() } else { neg(b) })
                            .collect();
                        self.rule(&atom, body);
                    }
                    self.lines.push(format!("query({atom})."));
                }
            }
            Value::Bool(l) => {
                if is(l, "true") || is(l, "false") || l.ends_with(')') {
                    // A constant, or an atom of a loop's last step: an atom
                    // of its own, as a Boolean's query has no argument.
                    let a = self.fresh(name);
                    let body = if is(l, "false") { "fail" } else { l };
                    self.lines.push(format!("{a} :- {body}."));
                    self.lines.push(format!("query({a})."));
                } else {
                    self.lines.push(format!("query({l})."));
                }
            }
            Value::Tuple(t) => {
                self.queries(&t.0, &format!("{name}_1"));
                self.queries(&t.1, &format!("{name}_2"));
            }
        }
    }
}

/// A loop's state as [`Translator::iterate`] works it out: the initial
/// state's parts, which of their literals the step keeps, the values each
/// integer takes, the atom of each literal not kept, and each part's hint.
struct Loop {
    start: Vec<Value>,
    kept: Vec<Vec<bool>>,
    values: Vec<Option<Vec<u64>>>,
    atoms: Vec<Vec<Option<String>>>,
    hints: Vec<String>,
}

impl Loop {
    fn atom(&self, i: usize, j: usize) -> &str {
        self.atoms[i][j]
            .as_deref()
            .expect("a literal not kept has an atom")
    }
}

fn within(path: &[Lit], guard: &Lit) -> Vec<Lit> {
    let mut out = path.to_vec();
    if !is(guard, "true") {
        out.push(guard.clone());
    }
    out
}

/// The conjunctions of literals of the bits above `k` that hold for the
/// settings in `[lo, lo + size)` whose ratio is `r`.
fn paths(
    ratio: &[Option<f64>],
    bits: &[Lit],
    k: usize,
    lo: usize,
    size: usize,
    r: f64,
) -> Vec<Vec<Lit>> {
    let range = &ratio[lo..lo + size];
    let mine = range.iter().filter(|x| **x == Some(r)).count();
    if mine == 0 {
        return Vec::new();
    }
    if mine == range.iter().filter(|x| x.is_some()).count() {
        return vec![Vec::new()];
    }
    let half = size / 2;
    // The bit that splits the range.
    let j = k + (usize::BITS - half.leading_zeros()) as usize;
    let mut out: Vec<Vec<Lit>> = paths(ratio, bits, k, lo, half, r)
        .into_iter()
        .map(|t| std::iter::once(neg(&bits[j])).chain(t).collect())
        .collect();
    out.extend(
        paths(ratio, bits, k, lo + half, half, r)
            .into_iter()
            .map(|t| std::iter::once(bits[j].clone()).chain(t).collect()),
    );
    out
}
