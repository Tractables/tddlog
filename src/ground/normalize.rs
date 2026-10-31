//! Reduce annotated disjunctions, probabilistic rules and negative head
//! literals to deterministic rules over *switch* predicates.
//!
//! A clause `p1::h1; ...; pn::hn :- body` with clause variables `V` becomes,
//! for each `i`, the rule
//!
//! ```text
//! hi :- body, sw_k_i(V), \+ sw_k_1(V), ..., \+ sw_k_{i-1}(V).
//! ```
//!
//! where `sw_k_i/|V|` is a switch predicate with probability
//! `q_i = p_i / (1 - p_1 - ... - p_{i-1})` (stick breaking): the switches of a
//! ground instance are independent and exactly one head, or none if the
//! probabilities sum below one, is selected per ground instance. A single
//! probabilistic head `p::h :- body` is the case `n = 1`, with `q_1 = p`.
//! A head whose probability is what remains, to a relative 10⁻⁹, gets
//! `q_i = 1`, and its switch is a certain fact that grounding removes.
//! Clauses sharing a `group` (one source clause split on a body disjunction)
//! share their switches.
//!
//! A clause with a flexible probability (`P::h(X) :- w(X, P).`) is known
//! only in each ground instance: its switches take the heads' probabilities
//! as arguments after `V`, `sw_k_i(V, p_1, ..., p_n)`, each `p_j` the
//! annotation where it is a number or a variable, and otherwise a variable
//! the body binds by `p_j is e_j`, and the grounder weighs each switch atom
//! by them ([`Weights::Flexible`]). The `p_j` are functions of `V`, so the
//! switches are the same random variables, one per ground instance.
//!
//! A non-ground probabilistic fact `p::f(X,Y).` declares `f/2` itself as a
//! switch predicate: every ground instance a rule asks for is an independent
//! variable with probability `p`.
//!
//! A switch predicate has no rules; its ground atoms are the Boolean random
//! variables of the program. Switch atoms are created on demand by the
//! grounder with whatever arguments the rule binds.
//!
//! A predicate given by ground facts alone, `p::f(a,b).` or `f(a,b).`, is
//! kept as its [`Facts`]: its tuples, each with the weights of the switch
//! its fact would have, are the predicate's atoms, which the grounder makes
//! facts (certain ones where the fact has no probability) without a switch
//! or a rule per fact. A database is such facts: TPC-H's line items are
//! 600 000 at scale factor 0.1, and a switch and a rule for each made two
//! atoms of each.
//!
//! Negative head literals: when a predicate `p/n` has clauses with head
//! `\+p(..)`, all its positive clauses are renamed to `p⁺/n`, the negative
//! ones to `p⁻/n`, and `p(V) :- p⁺(V), \+ p⁻(V)` is added.

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::Arc;

use super::Prob;
use super::engine::GroundError;
use crate::syntax::{
    Annotation, Args, Atom, Clause, F64, HeadChoice, Literal, Name, Program, Term, same_name,
};
use rustc_hash::{FxHashMap, FxHashSet};

/// The weights of a switch predicate's atoms.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Weights {
    /// The same for every atom.
    Fixed(Prob),
    /// Those of head `index` of a clause of flexible probabilities, whose
    /// `heads` probabilities are each atom's last arguments
    /// ([`switch_weights`]); `clause` is the clause as written, for the
    /// errors that name it.
    Flexible {
        index: usize,
        heads: usize,
        clause: Arc<str>,
    },
}

/// A query rule, `query(pattern) :- body.` or `evidence(pattern, v) :-
/// body.`: the pattern's instances are queries, or evidence, for every
/// true atom of the driver predicate `'$query_k'(vars) :- body`.
#[derive(Clone, Debug)]
pub(crate) struct Driver {
    pub(crate) atom: Atom,
    pub(crate) pattern: Atom,
    /// The query's sign, or the evidence's value.
    pub(crate) positive: bool,
    pub(crate) evidence: bool,
}

/// Marks switch predicate names. Cannot clash with user predicates because a
/// user name never contains this character.
pub(crate) const SWITCH_PREFIX: &str = "$sw";

/// How far from one the probabilities of an annotated disjunction may sum
/// and still be read as a full distribution.
pub(super) const SUM_TOLERANCE: f64 = 1e-8;

/// A normalized program: deterministic rules plus switch declarations.
#[derive(Clone, Debug, Default)]
pub(crate) struct NormProgram {
    pub(crate) rules: Vec<Clause>,
    /// Switch predicates `(name, arity)` and their weights.
    pub(crate) switches: FxHashMap<(Name, usize), Weights>,
    pub(crate) queries: Vec<(Atom, bool)>,
    pub(crate) evidence: Vec<(Atom, bool)>,
    /// Query rules, of queries or evidence ([`Driver`]).
    pub(crate) drivers: Vec<Driver>,
    /// Control predicates (magic sets): evaluated during grounding, never
    /// emitted into ground rules.
    pub(crate) control: FxHashSet<(Name, usize)>,
    /// The predicates findalls read, whose derivations the grounder
    /// counts ([`super::terms`]).
    pub(crate) counted: FxHashSet<(Name, usize)>,
    /// The predicates given by ground facts alone, each with the number of
    /// rules before its first fact, in that order. A rewrite of the program
    /// shares them.
    pub(crate) facts: Vec<(usize, Arc<Facts>)>,
}

/// A predicate given by ground facts alone, none of them negated or in a
/// disjunction, that no findall reads: its tuples, in the order of the
/// facts. A tuple given twice is true where either fact is.
#[derive(Debug)]
pub(crate) struct Facts {
    pub(crate) name: Name,
    pub(crate) arity: usize,
    /// The facts' arguments, `arity` to a fact.
    pub(crate) args: Vec<Term>,
    /// Each fact's weights, those of the switch of its head
    /// ([`switch_probabilities`]), or `None` for a certain fact.
    pub(crate) probs: Vec<Option<Prob>>,
}

impl Facts {
    /// The arguments of each fact, with its weights, in order.
    pub(crate) fn tuples(&self) -> impl Iterator<Item = (&[Term], Option<Prob>)> {
        // By place rather than in chunks: a predicate of arity zero has
        // facts and no arguments.
        (0..self.probs.len()).map(|k| {
            (
                &self.args[k * self.arity..(k + 1) * self.arity],
                self.probs[k],
            )
        })
    }
}

/// A predicate, `(name, arity)`, the name borrowed from the program.
pub(super) type Sig<'a> = (&'a str, usize);

impl NormProgram {
    /// The rules of each predicate, as indices into `rules` in their order.
    pub(super) fn rules_by_sig(&self) -> FxHashMap<Sig<'_>, Vec<usize>> {
        let mut rules_of: FxHashMap<Sig, Vec<usize>> = FxHashMap::default();
        for (i, c) in self.rules.iter().enumerate() {
            rules_of
                .entry(c.heads[0].atom.signature())
                .or_default()
                .push(i);
        }
        rules_of
    }

    /// Keeps the rules for which `keep` holds of their index and clause,
    /// each predicate's facts at its place among them.
    pub(super) fn retain_rules(&mut self, mut keep: impl FnMut(usize, &Clause) -> bool) {
        // How many rules are kept before each place.
        let mut before = Vec::with_capacity(self.rules.len() + 1);
        before.push(0);
        self.rules.retain(|c| {
            let i = before.len() - 1;
            let k = keep(i, c);
            before.push(before[i] + usize::from(k));
            k
        });
        for (at, _) in &mut self.facts {
            *at = before[*at];
        }
    }
}

/// Variables of a clause in order of first occurrence.
fn clause_vars(heads: &[HeadChoice], body: &[Literal]) -> Vec<Term> {
    let mut vars: Vec<String> = Vec::new();
    for h in heads {
        h.atom.vars_into(&mut vars);
    }
    for l in body {
        l.vars_into(&mut vars);
    }
    vars.into_iter().map(Term::Var).collect()
}

pub(crate) fn normalize(
    program: &Program,
    counted: FxHashSet<(Name, usize)>,
) -> Result<NormProgram, GroundError> {
    let mut out = NormProgram {
        queries: program.queries.clone(),
        evidence: program.evidence.clone(),
        ..Default::default()
    };

    // Predicates with negative head literals, and each predicate's number
    // of clauses and whether they are all ground facts ([`Facts`]).
    // Signatures are borrowed from the program, not owned one per head;
    // a predicate's facts usually come together, and the last one is
    // looked up by its name's address before its signature is hashed.
    fn sig(a: &Atom) -> (&str, usize) {
        a.signature()
    }
    fn fact(c: &Clause) -> Option<&HeadChoice> {
        match &c.heads[..] {
            [h] if c.body.is_empty() && !h.negated && h.atom.is_ground() && h.mass().is_some() => {
                Some(h)
            }
            _ => None,
        }
    }
    /// The place of `a`'s predicate in `defined`, which gains it where it
    /// is new; `last` is the last one looked up.
    fn place_of<'p>(
        a: &'p Atom,
        last: &mut Option<(&'p Name, usize, usize)>,
        place: &mut FxHashMap<(&'p str, usize), usize>,
        defined: &mut Vec<Defined<'p>>,
    ) -> usize {
        if let Some((name, arity, k)) = *last
            && arity == a.args.len()
            && same_name(name, &a.name)
        {
            return k;
        }
        let k = *place.entry(sig(a)).or_insert_with(|| {
            defined.push((sig(a), 0, true));
            defined.len() - 1
        });
        *last = Some((&a.name, a.args.len(), k));
        k
    }
    /// A predicate, its number of clauses, and whether they are all facts.
    type Defined<'p> = ((&'p str, usize), usize, bool);
    let mut negated_preds: Vec<(&str, usize)> = Vec::new();
    let mut place: FxHashMap<(&str, usize), usize> = FxHashMap::default();
    let mut defined: Vec<Defined> = Vec::new();
    let mut last = None;
    // The non-ground probabilistic facts, by their predicates' places.
    let mut open_facts: Vec<(usize, f64)> = Vec::new();
    for c in &program.clauses {
        let is_fact = fact(c).is_some();
        let open = c.heads.len() == 1 && c.body.is_empty() && !is_fact;
        for h in &c.heads {
            let k = place_of(&h.atom, &mut last, &mut place, &mut defined);
            defined[k].1 += 1;
            defined[k].2 &= is_fact;
            if h.negated && !negated_preds.contains(&defined[k].0) {
                negated_preds.push(defined[k].0);
            }
            if let Some(Annotation::Number(p)) = h.prob
                && open
                && !h.negated
            {
                open_facts.push((k, p));
            }
        }
    }

    // Non-ground probabilistic facts become switch predicates outright, if the
    // predicate has no other clause.
    let mut fact_switches: FxHashMap<(&str, usize), f64> = FxHashMap::default();
    for (k, p) in open_facts {
        if defined[k].1 == 1 {
            fact_switches.insert(defined[k].0, p);
        }
    }
    for (&(name, arity), p) in &fact_switches {
        out.switches
            .insert((name.into(), arity), Weights::Fixed(Prob::new(*p)));
    }

    // The predicates kept as their facts, by place: each one's place in
    // `made`, `NEW` until its first fact makes it. A predicate a findall
    // reads keeps its facts as rules, whose derivations the grounder counts.
    const NEW: usize = usize::MAX;
    let mut kept: Vec<Option<usize>> = defined
        .iter()
        .map(|&((name, arity), _, all_facts)| {
            let read = counted.iter().any(|(n, k)| *k == arity && &**n == name);
            (all_facts && !read).then_some(NEW)
        })
        .collect();
    let mut made: Vec<(usize, Facts)> = Vec::new();
    let mut qs = Vec::with_capacity(1);
    let sum_error = |heads: Vec<HeadChoice>, clause: &Clause, total: f64| {
        let clause = Clause {
            heads,
            body: clause.body.clone(),
            group: clause.group,
        };
        GroundError::Unsupported(format!(
            "annotated disjunction probabilities sum to {total} > 1: {clause}"
        ))
    };

    // The switches' names are written in one buffer, not one per clause.
    let mut name = String::new();
    last = None;
    for clause in &program.clauses {
        if let Some(h) = fact(clause) {
            let k = place_of(&h.atom, &mut last, &mut place, &mut defined);
            if let Some(mut t) = kept[k] {
                if t == NEW {
                    t = made.len();
                    kept[k] = Some(t);
                    let (_, n, _) = defined[k];
                    let facts = Facts {
                        name: h.atom.name.clone(),
                        arity: h.atom.args.len(),
                        args: Vec::with_capacity(n * h.atom.args.len()),
                        probs: Vec::with_capacity(n),
                    };
                    made.push((out.rules.len(), facts));
                }
                let prob = if h.prob.is_some() {
                    qs.clear();
                    switch_probabilities(&clause.heads, &mut qs)
                        .map_err(|total| sum_error(clause.heads.clone(), clause, total))?;
                    Some(qs[0])
                } else {
                    None
                };
                let (_, facts) = &mut made[t];
                facts.args.extend(h.atom.args.iter().cloned());
                facts.probs.push(prob);
                continue;
            }
        }
        let heads = renamed_heads(&clause.heads, &negated_preds);
        let probabilistic = heads.iter().any(|h| h.prob.is_some());
        if !probabilistic {
            let clause = Clause {
                heads,
                body: clause.body.clone(),
                group: clause.group,
            };
            if clause.heads.len() > 1 {
                return Err(GroundError::Unsupported(format!(
                    "disjunctive head without probabilities: {clause}"
                )));
            }
            out.rules.push(clause);
            continue;
        }
        if heads.len() == 1 && fact_switches.contains_key(&sig(&heads[0].atom)) {
            continue; // declared as a switch predicate above
        }
        let mut vars = clause_vars(&heads, &clause.body);
        let mut computed: Vec<Literal> = Vec::new();
        let n = heads.len();
        let flexible: Option<Arc<str>> = if heads.iter().all(|h| h.mass().is_some()) {
            qs.clear();
            if let Err(total) = switch_probabilities(&heads, &mut qs) {
                return Err(sum_error(heads, clause, total));
            }
            None
        } else {
            // The heads' probabilities follow `V` in the switches'
            // arguments, those not a number or a variable bound by `is`.
            for (j, h) in heads.iter().enumerate() {
                vars.push(match &h.prob {
                    None => Term::Float(F64(1.0)),
                    Some(Annotation::Number(p)) => Term::Float(F64(*p)),
                    Some(Annotation::Flexible(t)) if t.is_var() => t.clone(),
                    Some(Annotation::Flexible(t)) => {
                        let p = Term::Var(format!("$P{j}"));
                        computed.push(Literal::Is(p.clone(), t.clone()));
                        p
                    }
                });
            }
            Some(clause.to_string().into())
        };
        let mut guards: Vec<Literal> = Vec::new();
        for (i, head) in heads.into_iter().enumerate() {
            let sw_name = switch_name(&mut name, &head.atom.name, clause.group, i);
            let sw = Atom::new(sw_name.clone(), vars.iter().cloned().collect::<Args>());
            let weights = match &flexible {
                None => Weights::Fixed(qs[i]),
                Some(clause) => Weights::Flexible {
                    index: i,
                    heads: n,
                    clause: clause.clone(),
                },
            };
            out.switches.insert((sw_name, vars.len()), weights);
            // Allocated once at its size, to take the switch and the
            // guards beside a copy of the clause's body.
            let mut body =
                Vec::with_capacity(clause.body.len() + computed.len() + 1 + guards.len());
            body.extend_from_slice(&clause.body);
            body.extend_from_slice(&computed);
            body.push(Literal::Pos(sw.clone()));
            body.extend_from_slice(&guards);
            out.rules.push(Clause::rule(head.atom, body, clause.group));
            guards.push(Literal::Neg(sw));
        }
    }
    out.facts = made.into_iter().map(|(at, f)| (at, Arc::new(f))).collect();
    out.counted = counted;

    for (k, qr) in program.query_rules.iter().enumerate() {
        // The pattern's variables that the body binds: the others stay
        // variables of the queries, as ProbLog's findall1.pl has one.
        let mut vars: Vec<String> = Vec::new();
        qr.pattern.vars_into(&mut vars);
        let mut bound: Vec<String> = Vec::new();
        for l in &qr.body {
            l.vars_into(&mut bound);
        }
        vars.retain(|v| bound.contains(v));
        let vars: Vec<Term> = vars.into_iter().map(Term::Var).collect();
        let driver = Atom::new(format!("$query{k}"), vars);
        out.rules
            .push(Clause::rule(driver.clone(), qr.body.clone(), u32::MAX));
        out.drivers.push(Driver {
            atom: driver,
            pattern: qr.pattern.clone(),
            positive: qr.positive,
            evidence: qr.evidence,
        });
    }

    // p(V) :- p⁺(V), \+ p⁻(V) for each predicate with negative heads.
    for &(name, arity) in &negated_preds {
        let vars: Vec<Term> = (0..arity).map(|i| Term::Var(format!("V{i}"))).collect();
        let body = vec![
            Literal::Pos(Atom::new(pos_name(name), vars.clone())),
            Literal::Neg(Atom::new(neg_name(name), vars.clone())),
        ];
        out.rules
            .push(Clause::rule(Atom::new(name, vars), body, u32::MAX));
    }
    Ok(out)
}

/// The sum of the masses of a clause's heads ([`HeadChoice::mass`]);
/// `Err` with it where it passes one.
pub(super) fn head_total(masses: &[f64]) -> Result<f64, f64> {
    let total: f64 = masses.iter().sum();
    if total > 1.0 + SUM_TOLERANCE {
        Err(total)
    } else {
        Ok(total)
    }
}

/// [`switch_weights`] of `heads`, whose probabilities are numbers.
pub(super) fn switch_probabilities(heads: &[HeadChoice], qs: &mut Vec<Prob>) -> Result<(), f64> {
    let masses: Vec<f64> = heads
        .iter()
        .map(|h| h.mass().expect("a head of a probability that is a number"))
        .collect();
    switch_weights(&masses, qs)
}

/// The weights of the switches of heads of masses `masses` in `qs`: true
/// with `p_i / S_i` and false with `S_{i+1} / S_i`, where `S_i` is the
/// mass of heads `i` on and of no head ([`chain_weights`]); `Err` with the
/// sum of the masses where it passes one ([`head_total`]).
pub(super) fn switch_weights(masses: &[f64], qs: &mut Vec<Prob>) -> Result<(), f64> {
    let total = head_total(masses)?;
    // Probabilities that sum to one up to the rounding of the numbers as
    // written are a full distribution, of no mass of no head. Read as
    // written, they leave a residual of no head (bn_mildew's rows, given
    // to nine decimals, would), and a set with such a row is not a table.
    let none = if (total - 1.0).abs() <= SUM_TOLERANCE {
        0.0
    } else {
        residual(masses)
    };
    chain_weights(masses, none, qs);
    Ok(())
}

/// `1 - p_1 - ... - p_n` for the heads' masses, summed with the error of
/// each subtraction carried (Neumaier), so that a small residual keeps its
/// digits rather than the rounding of the probabilities' sum.
fn residual(masses: &[f64]) -> f64 {
    let (mut sum, mut carry) = (1.0f64, 0.0f64);
    for &m in masses {
        let p = -m;
        let t = sum + p;
        carry += if sum.abs() >= p.abs() {
            (sum - t) + p
        } else {
            (p - t) + sum
        };
        sum = t;
    }
    (sum + carry).max(0.0)
}

/// `$sw<name>_<group>_<i>`, the switch of head `i` of a clause, written
/// in `s` without the formatting machinery, which is slow at this volume.
pub(super) fn switch_name(s: &mut String, name: &str, group: u32, i: usize) -> Name {
    fn push_decimal(s: &mut String, mut n: u64) {
        let mut digits = [0u8; 20];
        let mut k = digits.len();
        loop {
            k -= 1;
            digits[k] = b'0' + (n % 10) as u8;
            n /= 10;
            if n == 0 {
                break;
            }
        }
        s.push_str(std::str::from_utf8(&digits[k..]).expect("decimal digits"));
    }
    s.clear();
    s.push_str(SWITCH_PREFIX);
    s.push_str(name);
    s.push('_');
    push_decimal(s, u64::from(group));
    s.push('_');
    push_decimal(s, i as u64);
    Name::from(s.as_str())
}

fn pos_name(name: &str) -> Name {
    format!("{name}$pos").into()
}
fn neg_name(name: &str) -> Name {
    format!("{name}$neg").into()
}

/// `program` with the heads of its annotated disjunctions in one order
/// that every disjunction keeps; `None` where each keeps it as written.
///
/// A disjunction's chain of switches follows its heads' order, and the
/// passes that compare rows, the row classes ([`super::classes`]) and the
/// switch sharing ([`super::share`]), find two rows of one distribution
/// alike only where their heads come in one order. A table written with
/// each row's values in an order of its own holds the distribution of
/// the table written in one order; with its rows in one order it is
/// compiled alike. The order is the one the disjunctions agree on: the
/// heads, numbered in the order they are first read, are put in a
/// topological order of "comes before" in a disjunction, the lowest number
/// ready first, and where none is ready (the disjunctions disagree) the
/// lowest number left. Disjunctions that agree keep their order, rows that
/// leave a value out included.
pub(super) fn ordered_heads(program: &Program) -> Option<Program> {
    let disjunctions = || program.clauses.iter().filter(|c| c.heads.len() > 1);
    let mut id: FxHashMap<&Atom, usize> = FxHashMap::default();
    let mut next: Vec<Vec<usize>> = Vec::new();
    let mut before: Vec<usize> = Vec::new();
    for c in disjunctions() {
        let mut last = usize::MAX;
        for h in &c.heads {
            let fresh = id.len();
            let a = *id.entry(&h.atom).or_insert(fresh);
            if a == fresh {
                next.push(Vec::new());
                before.push(0);
            }
            if last != usize::MAX {
                next[last].push(a);
                before[a] += 1;
            }
            last = a;
        }
    }
    let n = id.len();
    let mut rank = vec![usize::MAX; n];
    let mut ready: BinaryHeap<Reverse<usize>> =
        (0..n).filter(|&a| before[a] == 0).map(Reverse).collect();
    let mut lowest = 0;
    for r in 0..n {
        let a = match ready.pop() {
            Some(Reverse(a)) => a,
            None => {
                while rank[lowest] != usize::MAX {
                    lowest += 1;
                }
                lowest
            }
        };
        rank[a] = r;
        for &b in &next[a] {
            before[b] -= 1;
            if before[b] == 0 && rank[b] == usize::MAX {
                ready.push(Reverse(b));
            }
        }
    }
    let key = |h: &HeadChoice| rank[id[&h.atom]];
    if disjunctions().all(|c| c.heads.is_sorted_by_key(key)) {
        return None;
    }
    let mut out = program.clone();
    for c in out.clauses.iter_mut().filter(|c| c.heads.len() > 1) {
        c.heads.sort_by_key(key);
    }
    Some(out)
}

/// A clause's heads, with those of the predicates in `negated` renamed
/// to their `$pos` and `$neg` parts.
fn renamed_heads(heads: &[HeadChoice], negated: &[(&str, usize)]) -> Vec<HeadChoice> {
    let mut heads = heads.to_vec();
    for h in &mut heads {
        if negated.contains(&h.atom.signature()) {
            h.atom.name = if h.negated {
                neg_name(&h.atom.name)
            } else {
                pos_name(&h.atom.name)
            };
            h.negated = false;
        } else if h.negated {
            // A negative head for a predicate with no positive clause: the
            // predicate is false anyway; keep as a `$neg` rule with nothing to
            // negate.
            h.atom.name = neg_name(&h.atom.name);
            h.negated = false;
        }
    }
    heads
}

/// The weights of the switches of a chain of heads of masses `masses`,
/// with `none` the mass of no head, in `out`: switch `i` is true with `m_i /
/// S_i` and false with `S_{i+1} / S_i`, where `S_i = m_i + ... + m_n +
/// none`, summed from the end. Each weight is a quotient of two sums of the
/// masses, so no head is lost to rounding however small its mass beside
/// the heads before it. A switch with no mass left is false; the last
/// head's switch with no mass of no head is true, exactly.
pub(crate) fn chain_weights(masses: &[f64], none: f64, out: &mut Vec<Prob>) {
    out.clear();
    out.resize(masses.len(), Prob::FALSE);
    let mut after = none;
    for (w, &m) in out.iter_mut().zip(masses).rev() {
        let s = m + after;
        if s > 0.0 {
            *w = Prob {
                p: m / s,
                not: after / s,
            };
        }
        after = s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::parse_program;

    fn fixed(w: &Weights) -> Prob {
        match w {
            Weights::Fixed(q) => *q,
            other => panic!("{other:?}"),
        }
    }

    /// A clause of flexible probabilities: its switches take the heads'
    /// probabilities after its variables, the variable `P` itself, a
    /// number as it is and an expression through `is`.
    #[test]
    fn flexible_probabilities_are_the_switches_last_arguments() {
        let p = parse_program("P::a(X); 0.2::b(X); 1/W::c(X) :- w(X, P, W).\n").unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        let text: Vec<String> = n.rules.iter().map(|r| r.to_string()).collect();
        assert_eq!(
            text[0],
            "a(X) :- w(X,P,W), $P2 is 1/W, '$swa_0_0'(X,P,W,P,0.2,$P2)."
        );
        let w = &n.switches[&(Name::from(format!("{SWITCH_PREFIX}c_0_2")), 6)];
        assert!(matches!(
            w,
            Weights::Flexible {
                index: 2,
                heads: 3,
                ..
            }
        ));
    }

    #[test]
    fn ad_becomes_switches() {
        let p = parse_program("0.2::a(X); 0.3::b(X,Y) :- c(X,Y).\nd.\n").unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        assert_eq!(n.rules.len(), 2);
        let (at, d) = &n.facts[0];
        assert_eq!((*at, &*d.name, d.probs.len()), (2, "d", 1));
        assert_eq!(n.switches.len(), 2);
        let q1 = fixed(&n.switches[&(Name::from(format!("{SWITCH_PREFIX}a_0_0")), 2)]);
        let q2 = fixed(&n.switches[&(Name::from(format!("{SWITCH_PREFIX}b_0_1")), 2)]);
        assert!((q1.p - 0.2).abs() < 1e-12 && (q1.not - 0.8).abs() < 1e-12);
        assert!((q2.p - 0.3 / 0.8).abs() < 1e-12 && (q2.not - 0.5 / 0.8).abs() < 1e-12);
        assert_eq!(n.rules[0].to_string(), "a(X) :- c(X,Y), '$swa_0_0'(X,Y).");
        assert_eq!(
            n.rules[1].to_string(),
            "b(X,Y) :- c(X,Y), '$swb_0_1'(X,Y), \\+'$swa_0_0'(X,Y)."
        );
    }

    #[test]
    fn rounded_distribution_is_full() {
        let p = parse_program("0.3::a; 0.699999999::b.\n0.3::c; 0.6999::d.\n").unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        let a = fixed(&n.switches[&(Name::from(format!("{SWITCH_PREFIX}a_0_0")), 0)]);
        let b = fixed(&n.switches[&(Name::from(format!("{SWITCH_PREFIX}b_0_1")), 0)]);
        let d = fixed(&n.switches[&(Name::from(format!("{SWITCH_PREFIX}d_1_1")), 0)]);
        assert!((a.p - 0.3 / 0.999999999).abs() < 1e-15);
        assert_eq!(b, Prob::TRUE);
        assert!((d.p - 0.6999 / 0.7).abs() < 1e-12);
    }

    #[test]
    fn nonground_fact_is_switch_predicate() {
        let p = parse_program("0.5::b(X).\ns(X) :- p(X), b(X).\n").unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        assert_eq!(n.rules.len(), 1);
        assert_eq!(n.switches[&("b".into(), 1)], Weights::Fixed(Prob::HALF));
    }

    #[test]
    fn negative_heads() {
        let p = parse_program("0.5::a :- b.\n\\+a :- c.\n").unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        let text: Vec<String> = n.rules.iter().map(|r| r.to_string()).collect();
        assert_eq!(text[0], "'a$pos' :- b, '$swa$pos_0_0'.");
        assert_eq!(text[1], "'a$neg' :- c.");
        assert_eq!(text[2], "a :- 'a$pos', \\+'a$neg'.");
    }

    /// A predicate given by ground facts alone is kept as its facts, their
    /// weights those of their switches, at its place among the rules; one
    /// with a rule, a negated head, a non-ground fact or a findall reading
    /// it keeps its clauses.
    #[test]
    fn ground_facts_are_kept_as_facts() {
        let p = parse_program(
            "0.3::e(1,2). e(2,3). 0.999999999999::e(3,1).\n0.4::r(1). r(X) :- e(X,_).\n\
             0.5::n(1). \\+n(2).\n0.5::v(1). 0.5::v(X).\n0.2::c(a). 0.6::c(b).\n",
        )
        .unwrap();
        let counted = [(Name::from("c"), 1)].into_iter().collect();
        let n = normalize(&p, counted).unwrap();
        let kept: Vec<(usize, &str, usize, usize)> = n
            .facts
            .iter()
            .map(|(at, f)| (*at, &*f.name, f.arity, f.probs.len()))
            .collect();
        assert_eq!(kept, [(0, "e", 2, 3)]);
        let facts: Vec<(String, Option<Prob>)> = n.facts[0]
            .1
            .tuples()
            .map(|(args, prob)| (format!("{args:?}"), prob))
            .collect();
        assert_eq!(facts[0].1, Some(Prob::new(0.3)));
        assert_eq!(facts[1].1, None);
        assert_eq!(facts[2].1, Some(Prob::TRUE));
        assert_eq!(facts[1].0, "[Int(2), Int(3)]");
    }
}
