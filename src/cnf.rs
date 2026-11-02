//! A ground program written as a weighted CNF ([`WeightedCnf`], `tddlog
//! --cnf`), for a weighted model counter to count: the program `--ground`
//! prints, written without its linear recursion (`Kind::eliminated`).
//!
//! Each fact is a variable weighed as the fact, a switch of an annotated
//! disjunction among them. Each derived atom is a variable of weight one
//! either way, defined by the completion of its rules (Clark's): the atom
//! is true exactly where one of its bodies is, each body of two literals or
//! more a variable of its own that is true exactly where all its literals
//! are (Tseitin's encoding). In an acyclic program the completion has, in
//! each world of the facts, one model, the program's, so the weighted
//! count of the clauses is the probability of every world, one. An atom of
//! one body of one literal is that literal, without a variable of its own.
//! Each observation is a unit clause, so that the count is `P(e)` up to
//! the factor the grounder took out of it (`GroundProgram::weight`), which
//! is a variable of its own forced true, weighed as the factor: the count
//! is `P(e)` itself where the grounder keeps every observation
//! ([`crate::Options::evidence_probability`]). Each query is a literal,
//! named in a comment line: with it added as a unit clause, the count is
//! `P(q, e)`, and `P(q | e)` is the ratio of the two counts. Where no
//! observation is a clause, the count is known without counting, each
//! world of the facts having one model: the product of each fact's two
//! weights summed (one for a probability) and the factor, given in a
//! comment line, so that a counter counts only `P(q, e)`.
//!
//! A cyclic component that the elimination leaves has no such encoding:
//! the completion of `a :- b. b :- a.` has a model with both atoms true,
//! where the program's least model has neither, and a well-founded model
//! with atoms undefined (negation through recursion) has no completion
//! that holds it. Such a program is refused.

use std::fmt;

use crate::eval::Count;
use crate::ground::deps::{self, Deps};
use crate::ground::{GLit, GroundProgram};

/// A ground program as a weighted CNF in DIMACS ([`crate::weighted_cnf`]):
/// `c t wmc`, the `p cnf` line, a `c p weight` line for each literal of
/// each fact, the clauses, a line `c query LITERAL QUERY` for each query,
/// in the program's order, the query as tddlog prints its answer, and
/// where no observation is a clause, a line `c count WEIGHT`, the count of
/// the clauses.
pub struct WeightedCnf {
    /// The variables are 1 to `variables`, the factor's not among them.
    variables: u32,
    /// The clauses' literals, each clause ended by a 0.
    clauses: Vec<i32>,
    clause_count: usize,
    /// How many of the clauses are observations.
    observations: usize,
    /// Each fact's variable and its weights, true and false.
    weights: Vec<(u32, f64, f64)>,
    /// The factor the grounder took out of `P(e)`, and the base-10
    /// logarithm of a constant it is scaled by ([`WeightedCnf::scale`]).
    factor: Count,
    log10_scale: f64,
    /// Each query's literal and how it prints.
    queries: Vec<(i32, String)>,
}

/// A ground atom's value in the CNF: a constant, or a literal.
#[derive(Clone, Copy)]
enum Value {
    Constant(bool),
    Literal(i32),
}

impl WeightedCnf {
    /// The weighted CNF of `gp`, a program without indicators, which
    /// fails naming a cyclic component of `gp` where it has one.
    pub(crate) fn of(gp: &GroundProgram) -> Result<WeightedCnf, String> {
        assert!(
            gp.encoding.is_empty(),
            "the CNF is written from a program without indicators"
        );
        let deps = Deps::of(gp);
        if let Some(scc) = deps
            .sccs
            .iter()
            .find(|scc| deps::is_cyclic(gp, &deps.by_head, scc))
        {
            return Err(format!(
                "the program keeps a cyclic component of {} atoms, {} among them, after its \
                 linear recursion is eliminated, and its completion has models that are not the \
                 program's: no weighted CNF of it is written",
                scc.len(),
                gp.atoms[scc[0] as usize]
            ));
        }
        let mut cnf = WeightedCnf {
            variables: 0,
            clauses: Vec::new(),
            clause_count: 0,
            observations: 0,
            weights: Vec::new(),
            factor: gp.weight,
            log10_scale: 0.0,
            queries: Vec::new(),
        };
        let mut value = vec![Value::Constant(false); gp.num_atoms()];
        for (a, p) in gp.facts() {
            let v = cnf.variable();
            cnf.weights.push((v as u32, p.p, p.not));
            value[a as usize] = Value::Literal(v);
        }
        // Every component after those it reads, each one derived atom.
        for scc in &deps.sccs {
            let h = scc[0] as usize;
            let mut bodies = Vec::new();
            for &r in &deps.by_head[h] {
                for body in gp.rules[r].bodies() {
                    if let Some(lits) = literals(&body, &value) {
                        bodies.push(lits);
                    }
                }
            }
            value[h] = cnf.disjunction(bodies);
        }
        // The literal of a query's atom; a constant's is a variable forced
        // to it.
        let mut constants = [None; 2];
        for (atom, id, positive) in &gp.queries {
            let x = match id.map(|a| value[a as usize]) {
                Some(Value::Literal(x)) => x,
                Some(Value::Constant(c)) => cnf.constant(&mut constants, c),
                None => cnf.constant(&mut constants, false),
            };
            let text = if *positive {
                atom.to_string()
            } else {
                format!("\\+{atom}")
            };
            cnf.queries.push((if *positive { x } else { -x }, text));
        }
        // An observation of a constant holds, or is the empty clause.
        for &(_, id, observed) in &gp.evidence {
            let clause: &[i32] = match id.map(|a| value[a as usize]) {
                Some(Value::Literal(x)) => &[if observed { x } else { -x }],
                Some(Value::Constant(c)) if c == observed => continue,
                Some(Value::Constant(_)) => &[],
                None if observed => &[],
                None => continue,
            };
            cnf.clause(clause);
            cnf.observations += 1;
        }
        Ok(cnf)
    }

    /// Scales the count by `10^log10`: a Markov network's `P(e)` scaled to
    /// its partition function (`Input::log10_offset`).
    pub fn scale(&mut self, log10: f64) {
        self.log10_scale += log10;
    }

    /// The literals of each query, in the program's order, with the
    /// query as tddlog prints its answer.
    pub fn queries(&self) -> &[(i32, String)] {
        &self.queries
    }

    /// A new variable.
    fn variable(&mut self) -> i32 {
        self.variables += 1;
        self.variables as i32
    }

    fn clause(&mut self, lits: &[i32]) {
        self.clauses.extend_from_slice(lits);
        self.clauses.push(0);
        self.clause_count += 1;
    }

    /// The value of the disjunction of `bodies`, each a conjunction of
    /// literals sorted and with no two on one variable: a constant, a
    /// literal where it is one, or a variable defined as it.
    fn disjunction(&mut self, mut bodies: Vec<Vec<i32>>) -> Value {
        if bodies.iter().any(Vec::is_empty) {
            return Value::Constant(true);
        }
        // Two bodies alike are one conjunction.
        bodies.sort_unstable();
        bodies.dedup();
        let mut terms: Vec<i32> = match &bodies[..] {
            [] => return Value::Constant(false),
            [only] => return Value::Literal(self.conjunction(only)),
            _ => bodies.iter().map(|b| self.conjunction(b)).collect(),
        };
        terms.sort_unstable_by_key(|x| (x.abs(), *x));
        terms.dedup();
        if terms.windows(2).any(|w| w[0] == -w[1]) {
            return Value::Constant(true);
        }
        if let [only] = terms[..] {
            return Value::Literal(only);
        }
        let v = self.variable();
        for &t in &terms {
            self.clause(&[-t, v]);
        }
        let mut long = vec![-v];
        long.extend_from_slice(&terms);
        self.clause(&long);
        Value::Literal(v)
    }

    /// A literal true exactly where all of `lits` are: the one literal, or
    /// a variable defined as their conjunction.
    fn conjunction(&mut self, lits: &[i32]) -> i32 {
        if let [only] = lits {
            return *only;
        }
        let v = self.variable();
        for &l in lits {
            self.clause(&[-v, l]);
        }
        let mut long = vec![v];
        long.extend(lits.iter().map(|&l| -l));
        self.clause(&long);
        v
    }

    /// The variable forced to `c`, made where `constants` holds none yet.
    fn constant(&mut self, constants: &mut [Option<i32>; 2], c: bool) -> i32 {
        if let Some(v) = constants[c as usize] {
            return v;
        }
        let v = self.variable();
        self.clause(&[if c { v } else { -v }]);
        constants[c as usize] = Some(v);
        v
    }
}

/// The literals of `body` under `value`, sorted by variable and each once,
/// where they may all hold; `None` where one is false or two are
/// complementary.
fn literals(body: &[GLit], value: &[Value]) -> Option<Vec<i32>> {
    let mut lits = Vec::with_capacity(body.len());
    for &l in body {
        match value[l.atom() as usize] {
            Value::Constant(c) if c == l.is_positive() => {}
            Value::Constant(_) => return None,
            Value::Literal(x) => lits.push(if l.is_positive() { x } else { -x }),
        }
    }
    lits.sort_unstable_by_key(|x| (x.abs(), *x));
    lits.dedup();
    if lits.windows(2).any(|w| w[0] == -w[1]) {
        return None;
    }
    Some(lits)
}

/// A weight, exactly where it is a normal float and from its logarithm
/// beyond: `<mantissa>e<exponent>`.
fn weight_text(x: Count, log10_scale: f64) -> String {
    let f = x.to_f64();
    if log10_scale == 0.0 && (f.is_normal() || f == 0.0) {
        return format!("{f:e}");
    }
    let log10 = x.log10() + log10_scale;
    if log10 == f64::NEG_INFINITY {
        return "0".into();
    }
    let exponent = log10.floor();
    format!("{:.17}e{exponent}", 10f64.powf(log10 - exponent))
}

impl fmt::Display for WeightedCnf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The factor, where it is not one, is a variable of its own after
        // the others.
        let factor = (self.log10_scale != 0.0 || self.factor.to_f64() != 1.0)
            .then(|| weight_text(self.factor, self.log10_scale));
        let extra = u32::from(factor.is_some());
        writeln!(f, "c t wmc")?;
        writeln!(
            f,
            "p cnf {} {}",
            self.variables + extra,
            self.clause_count + extra as usize
        )?;
        for &(v, p, not) in &self.weights {
            writeln!(f, "c p weight {v} {p:e} 0")?;
            writeln!(f, "c p weight -{v} {not:e} 0")?;
        }
        if let Some(w) = &factor {
            let z = self.variables + 1;
            writeln!(f, "c p weight {z} {w} 0")?;
            writeln!(f, "c p weight -{z} 1 0")?;
            writeln!(f, "{z} 0")?;
        }
        let mut first = true;
        for &l in &self.clauses {
            if l == 0 {
                f.write_str(if first { "0\n" } else { " 0\n" })?;
                first = true;
            } else {
                if !first {
                    f.write_str(" ")?;
                }
                write!(f, "{l}")?;
                first = false;
            }
        }
        for (l, text) in &self.queries {
            writeln!(f, "c query {l} {text}")?;
        }
        if self.observations == 0 {
            let count = self
                .weights
                .iter()
                .fold(self.factor, |c, &(_, p, not)| c * Count::new(p + not));
            writeln!(f, "c count {}", weight_text(count, self.log10_scale))?;
        }
        Ok(())
    }
}
