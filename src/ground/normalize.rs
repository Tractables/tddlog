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
//! A non-ground probabilistic fact `p::f(X,Y).` declares `f/2` itself as a
//! switch predicate: every ground instance a rule asks for is an independent
//! variable with probability `p`.
//!
//! A switch predicate has no rules; its ground atoms are the Boolean random
//! variables of the program. Switch atoms are created on demand by the
//! grounder with whatever arguments the rule binds.
//!
//! Negative head literals: when a predicate `p/n` has clauses with head
//! `\+p(..)`, all its positive clauses are renamed to `p⁺/n`, the negative
//! ones to `p⁻/n`, and `p(V) :- p⁺(V), \+ p⁻(V)` is added.

use super::Prob;
use super::engine::GroundError;
use crate::syntax::{Args, Atom, Clause, HeadChoice, Literal, Name, Program, Term};
use rustc_hash::{FxHashMap, FxHashSet};

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
    pub(crate) switches: FxHashMap<(Name, usize), Prob>,
    pub(crate) queries: Vec<(Atom, bool)>,
    pub(crate) evidence: Vec<(Atom, bool)>,
    /// Query rules: the pattern's instances are queries for every true atom
    /// of the driver predicate `'$query_k'(vars)`; `(driver, pattern, positive)`.
    pub(crate) query_drivers: Vec<(Atom, Atom, bool)>,
    /// Control predicates (magic sets): evaluated during grounding, never
    /// emitted into ground rules.
    pub(crate) control: FxHashSet<(Name, usize)>,
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
}

/// Variables of a clause in order of first occurrence.
fn clause_vars(heads: &[HeadChoice], body: &[Literal]) -> Vec<Term> {
    let mut vars: Vec<Term> = Vec::new();
    for h in heads {
        for t in &h.atom.args {
            if t.is_var() && !vars.contains(t) {
                vars.push(t.clone());
            }
        }
    }
    for l in body {
        match l {
            Literal::Pos(a) | Literal::Neg(a) => {
                for t in &a.args {
                    if t.is_var() && !vars.contains(t) {
                        vars.push(t.clone());
                    }
                }
            }
            Literal::Cmp(_, a, b) => {
                a.vars(&mut vars);
                b.vars(&mut vars);
            }
            Literal::Is(v, e) => {
                if !vars.contains(v) {
                    vars.push(v.clone());
                }
                e.vars(&mut vars);
            }
        }
    }
    vars
}

pub(crate) fn normalize(program: &Program) -> Result<NormProgram, GroundError> {
    let mut out = NormProgram {
        queries: program.queries.clone(),
        evidence: program.evidence.clone(),
        ..Default::default()
    };

    // Predicates with negative head literals, and all predicates with rules.
    // Signatures are borrowed from the program, not owned one per head.
    fn sig(a: &Atom) -> (&str, usize) {
        a.signature()
    }
    let mut negated_preds: Vec<(&str, usize)> = Vec::new();
    let mut defined: FxHashMap<(&str, usize), usize> = FxHashMap::default();
    for c in &program.clauses {
        for h in &c.heads {
            let sig = sig(&h.atom);
            *defined.entry(sig).or_default() += 1;
            if h.negated && !negated_preds.contains(&sig) {
                negated_preds.push(sig);
            }
        }
    }

    // Non-ground probabilistic facts become switch predicates outright, if the
    // predicate has no other clause.
    let mut fact_switches: FxHashMap<(&str, usize), f64> = FxHashMap::default();
    for c in &program.clauses {
        if c.body.is_empty() && c.heads.len() == 1 && !c.heads[0].negated {
            let h = &c.heads[0];
            if let Some(p) = h.prob
                && !h.atom.is_ground()
                && defined[&sig(&h.atom)] == 1
            {
                fact_switches.insert(sig(&h.atom), p);
            }
        }
    }
    for (&(name, arity), p) in &fact_switches {
        out.switches.insert((name.into(), arity), Prob::new(*p));
    }

    // The switches' names are written in one buffer, not one per clause:
    // each of TPC-H's probabilistic tuples is a switch of its own.
    let mut name = String::new();
    for clause in &program.clauses {
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
        let vars = clause_vars(&heads, &clause.body);
        let mut qs = Vec::with_capacity(heads.len());
        if let Err(total) = switch_probabilities(&heads, &mut qs) {
            let clause = Clause {
                heads,
                body: clause.body.clone(),
                group: clause.group,
            };
            return Err(GroundError::Unsupported(format!(
                "annotated disjunction probabilities sum to {total} > 1: {clause}"
            )));
        }
        let mut guards: Vec<Literal> = Vec::new();
        for (i, (head, q)) in heads.into_iter().zip(qs).enumerate() {
            let sw_name = switch_name(&mut name, &head.atom.name, clause.group, i);
            let sw = Atom::new(sw_name.clone(), vars.iter().cloned().collect::<Args>());
            out.switches.insert((sw_name, vars.len()), q);
            // Allocated once at its size, to take the switch and the
            // guards beside a copy of the clause's body.
            let mut body = Vec::with_capacity(clause.body.len() + 1 + guards.len());
            body.extend_from_slice(&clause.body);
            body.push(Literal::Pos(sw.clone()));
            body.extend_from_slice(&guards);
            out.rules.push(Clause::rule(head.atom, body, clause.group));
            guards.push(Literal::Neg(sw));
        }
    }

    for (k, qr) in program.query_rules.iter().enumerate() {
        let mut vars: Vec<Term> = Vec::new();
        for t in &qr.pattern.args {
            if t.is_var() && !vars.contains(t) {
                vars.push(t.clone());
            }
        }
        let driver = Atom::new(format!("$query{k}"), vars);
        out.rules
            .push(Clause::rule(driver.clone(), qr.body.clone(), u32::MAX));
        out.query_drivers
            .push((driver, qr.pattern.clone(), qr.positive));
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

/// The sum of the probabilities of `heads`, a head without one, beside
/// heads with one, taking one; `Err` with it where it passes one.
pub(super) fn head_total(heads: &[HeadChoice]) -> Result<f64, f64> {
    let total: f64 = heads.iter().map(|h| h.prob.unwrap_or(1.0)).sum();
    if total > 1.0 + SUM_TOLERANCE {
        Err(total)
    } else {
        Ok(total)
    }
}

/// The weights of each head's switch in `qs`: true with `p_i / S_i` and
/// false with `S_{i+1} / S_i`, where `S_i` is the mass of heads `i` on and
/// of no head ([`chain_weights`]); `Err` with the sum of the probabilities
/// where it passes one ([`head_total`]).
pub(super) fn switch_probabilities(heads: &[HeadChoice], qs: &mut Vec<Prob>) -> Result<(), f64> {
    let total = head_total(heads)?;
    // Probabilities that sum to one up to the rounding of the numbers as
    // written are a full distribution, of no mass of no head. Read as
    // written, they leave a residual of no head (bn_mildew's rows, given
    // to nine decimals, would), and a set with such a row is not a table.
    let none = if (total - 1.0).abs() <= SUM_TOLERANCE {
        0.0
    } else {
        residual(heads)
    };
    let masses: Vec<f64> = heads.iter().map(|h| h.prob.unwrap_or(1.0)).collect();
    chain_weights(&masses, none, qs);
    Ok(())
}

/// `1 - p_1 - ... - p_n` for the heads' probabilities, summed with the
/// error of each subtraction carried (Neumaier), so that a small residual
/// keeps its digits rather than the rounding of the probabilities' sum.
fn residual(heads: &[HeadChoice]) -> f64 {
    let (mut sum, mut carry) = (1.0f64, 0.0f64);
    for h in heads {
        let p = -h.prob.unwrap_or(1.0);
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

    #[test]
    fn ad_becomes_switches() {
        let p = parse_program("0.2::a(X); 0.3::b(X,Y) :- c(X,Y).\nd.\n").unwrap();
        let n = normalize(&p).unwrap();
        assert_eq!(n.rules.len(), 3);
        assert_eq!(n.switches.len(), 2);
        let q1 = n.switches[&(Name::from(format!("{SWITCH_PREFIX}a_0_0")), 2)];
        let q2 = n.switches[&(Name::from(format!("{SWITCH_PREFIX}b_0_1")), 2)];
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
        let n = normalize(&p).unwrap();
        let a = n.switches[&(Name::from(format!("{SWITCH_PREFIX}a_0_0")), 0)];
        let b = n.switches[&(Name::from(format!("{SWITCH_PREFIX}b_0_1")), 0)];
        let d = n.switches[&(Name::from(format!("{SWITCH_PREFIX}d_1_1")), 0)];
        assert!((a.p - 0.3 / 0.999999999).abs() < 1e-15);
        assert_eq!(b, Prob::TRUE);
        assert!((d.p - 0.6999 / 0.7).abs() < 1e-12);
    }

    #[test]
    fn nonground_fact_is_switch_predicate() {
        let p = parse_program("0.5::b(X).\ns(X) :- p(X), b(X).\n").unwrap();
        let n = normalize(&p).unwrap();
        assert_eq!(n.rules.len(), 1);
        assert_eq!(n.switches[&("b".into(), 1)], Prob::HALF);
    }

    #[test]
    fn negative_heads() {
        let p = parse_program("0.5::a :- b.\n\\+a :- c.\n").unwrap();
        let n = normalize(&p).unwrap();
        let text: Vec<String> = n.rules.iter().map(|r| r.to_string()).collect();
        assert_eq!(text[0], "'a$pos' :- b, '$swa$pos_0_0'.");
        assert_eq!(text[1], "'a$neg' :- c.");
        assert_eq!(text[2], "a :- 'a$pos', \\+'a$neg'.");
    }
}
