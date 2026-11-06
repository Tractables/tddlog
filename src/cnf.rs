//! A ground program written as a weighted CNF ([`WeightedCnf`], `tddlog
//! --cnf`), for a weighted model counter to count: the program `--ground`
//! prints, written without its linear recursion (`Kind::eliminated`), as
//! every layout compiles it, after all of the grounder's passes (the
//! conditioning on the evidence, the backbones and equal facts, the merge
//! of equal gates, the variables held as bits written over their values,
//! the defined facts eliminated) but the row classes, which the shared
//! switches stand in for ([`crate::weighted_cnf`]).
//!
//! Each fact is a variable weighed as the fact, a switch of an annotated
//! disjunction among them. The switches of every table whose rows exclude
//! each other are shared first ([`ground::shared_for_cnf`]): the `k`-th
//! switch of one probability of each row is one variable, so that the
//! equal entries of a table are one parameter. Each derived atom is a
//! variable of weight one either way, defined by the completion of its
//! rules (Clark's): the atom is true exactly where one of its bodies is,
//! each body of two literals or more a variable of its own that is true
//! exactly where all its literals are (Tseitin's encoding). In an acyclic
//! program the completion has, in each world of the facts, one model, the
//! program's, so the weighted count of the clauses is the probability of
//! every world, one. An atom of one body of one literal is that literal,
//! without a variable of its own. Two kinds of atom are written without
//! the bodies' variables:
//!
//! - The atoms of an exclusive set that is exhaustive, in every world one
//!   of them true (the values of a network's variable; the cover finds
//!   them, `ground::cover`), where none of them reads one of the set:
//!   each is a variable with a clause `¬body ∨ v` per rule, and no two of
//!   them are true. In each world the one body that holds sets its atom's
//!   variable and the others are false, as the indicators' clause tables
//!   tie a table to its readers (`indicators::tables`). Of a set of two,
//!   the second is the first's negation, with the clauses `¬body ∨ ¬v`
//!   where the first has a variable and none where it is a literal.
//! - An observed atom that no rule reads and no query names, a Markov
//!   network's factor: no variable, but its observation as clauses. An atom
//!   observed false is a clause per body, its negation. An atom observed
//!   true whose rules are the rows of a table, each context one atom of
//!   each of the same exclusive sets and the contexts all different, is a
//!   clause `¬context ∨ s` for each of the table's own facts `s` a row
//!   reads (`¬s` where it reads `¬s`): a zero of the table is a clause
//!   `¬context` of the assignment without a row, and an entry of one is a
//!   row without facts and writes nothing. A fact that is the one fact of
//!   its rows, shared by the rows of its probability, and that nothing
//!   else reads is the table's parameter for them (Chavira and Darwiche's
//!   ENC1, a table's equal entries one parameter): weighed (w, 1), it is
//!   true exactly where one of its rows' contexts is, so that a counter
//!   finds it defined. Its other half is a clause `¬context ∨ ¬s` for each
//!   other row of the table, or where that is more literals, `s` implies
//!   its one context or one of its contexts, each a variable of its own
//!   ([`WeightedCnf::define_parameters`]). Any other fact of a row weighs
//!   its entry where its context holds and sums out to one elsewhere
//!   (Sang, Beame and Kautz's chance variables). Any other atom observed
//!   true is its definition and a unit clause.
//!
//! Each other observation is a unit clause, so that the count is `P(e)`
//! up to the factor the grounder took out of it (`GroundProgram::weight`),
//! which is a variable of its own forced true, weighed as the factor: the
//! count is `P(e)` itself where the grounder keeps every observation
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

use rustc_hash::{FxHashMap, FxHashSet};

use crate::eval::Count;
use crate::ground::cover::Cover;
use crate::ground::deps::{self, Deps};
use crate::ground::tables::Tables;
use crate::ground::{self, AtomId, AtomKind, GLit, GroundProgram};

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

/// The most assignments of its parents an observed table is written over
/// ([`WeightedCnf::observed_table`]), `ZEROS_PER_ROW` times its rows and
/// `MOST_ZEROS` more: each assignment without a row is a clause, and a
/// table of few rows over large sets is written as its atom's definition.
const ZEROS_PER_ROW: usize = 16;
const MOST_ZEROS: usize = 4096;

impl WeightedCnf {
    /// The weighted CNF of `gp`, a program without indicators, which
    /// fails naming a cyclic component of `gp` where it has one.
    pub(crate) fn of(gp: GroundProgram) -> Result<WeightedCnf, String> {
        assert!(
            gp.encoding.is_empty(),
            "the CNF is written from a program without indicators"
        );
        let deps = Deps::of(&gp);
        if let Some(scc) = deps
            .sccs
            .iter()
            .find(|scc| deps::is_cyclic(&gp, &deps.by_head, scc))
        {
            return Err(format!(
                "the program keeps a cyclic component of {} atoms, {} among them, after its \
                 linear recursion is eliminated, and its completion has models that are not the \
                 program's: no weighted CNF of it is written",
                scc.len(),
                gp.atoms[scc[0] as usize]
            ));
        }
        drop(deps);
        let (gp, mut tables, cover) = ground::shared_for_cnf(gp);
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
        let order = tables.order();
        let sets = SetClauses::of(&gp, &mut tables, &cover, &order);
        let reads = reads(&gp);
        let absorbed = absorbed(&gp, &sets, &reads);
        // Every atom after those it reads.
        for &h in &order {
            let mut bodies = Vec::new();
            for &r in &tables.by_head()[h as usize] {
                for body in gp.rules[r].bodies() {
                    if let Some(lits) = literals(&body, &value) {
                        bodies.push(lits);
                    }
                }
            }
            let h = h as usize;
            match (absorbed[h], sets.set[h]) {
                (Some(observed), _) => {
                    cnf.observation(
                        &gp,
                        (&tables, &cover, &reads),
                        (h as AtomId, observed),
                        bodies,
                        &value,
                    );
                    cnf.observations += 1;
                }
                (None, Some(s)) if !sets.literal[h] => {
                    let set = &tables.sets()[s];
                    // The other atom of a set of two, written before this one.
                    let first = (set.len() == 2 && sets.last[s] == h as AtomId)
                        .then(|| set[usize::from(set[0] == h as AtomId)]);
                    let (v, negated_literal) = match first.map(|o| (o, value[o as usize])) {
                        Some((o, Value::Literal(x))) => (-x, sets.literal[o as usize]),
                        _ => (cnf.variable(), false),
                    };
                    // The negation of a literal is defined as written; any
                    // other variable is true where one of the bodies is.
                    if !negated_literal {
                        for body in &bodies {
                            let mut clause: Vec<i32> = body.iter().map(|&l| -l).collect();
                            clause.push(v);
                            cnf.clause(&clause);
                        }
                    }
                    value[h] = Value::Literal(v);
                    if sets.last[s] == h as AtomId && first.is_none() {
                        cnf.at_most_one(set, &sets.literal, &value);
                    }
                }
                (None, set) => {
                    value[h] = cnf.disjunction(bodies);
                    if let Some(s) = set
                        && sets.last[s] == h as AtomId
                    {
                        cnf.at_most_one(&tables.sets()[s], &sets.literal, &value);
                    }
                }
            }
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
        // An observation of a constant holds, or is the empty clause; an
        // atom's observation written as clauses of its own is written.
        for &(_, id, observed) in &gp.evidence {
            if id.is_some_and(|a| absorbed[a as usize].is_some()) {
                continue;
            }
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

    /// Writes the observation of `h` at `observed`, an atom of the
    /// `bodies` its rules' bodies give under `value`, that no rule reads
    /// and no query names ([`absorbed`]), without a variable for `h`
    /// (the module's documentation): false, a clause per body; true, its
    /// one body's literals, or the rows of a table its rules are
    /// ([`Rows::of`], [`WeightedCnf::observed_table`]), or else its
    /// definition and a unit clause. `reads` is [`reads`]'s.
    fn observation(
        &mut self,
        gp: &GroundProgram,
        (tables, cover, reads): (&Tables, &Cover, &[u32]),
        (h, observed): (AtomId, bool),
        mut bodies: Vec<Vec<i32>>,
        value: &[Value],
    ) {
        bodies.sort_unstable();
        bodies.dedup();
        if !observed {
            for body in &bodies {
                self.clause(&body.iter().map(|&l| -l).collect::<Vec<_>>());
            }
            return;
        }
        if bodies.iter().any(Vec::is_empty) {
            return;
        }
        match &bodies[..] {
            [] => return self.clause(&[]),
            [only] => {
                for &l in only {
                    self.clause(&[l]);
                }
                return;
            }
            _ => {}
        }
        if let Some(rows) = Rows::of(gp, tables, h).filter(|rows| rows.bounded(tables)) {
            self.observed_table(&rows, tables, cover, value);
            return self.define_parameters(gp, &rows, reads, value);
        }
        match self.disjunction(bodies) {
            Value::Literal(v) => self.clause(&[v]),
            Value::Constant(true) => {}
            Value::Constant(false) => self.clause(&[]),
        }
    }

    /// The clauses of an observed table's `rows`, its atom true, with each
    /// atom's `value` ([`WeightedCnf::observation`]): for each row, its
    /// context implies each of its own facts' literals; for each parent set
    /// that the cover does not find exhaustive, one of its atoms holds; and
    /// each assignment of the parents' sets without a row is ruled out.
    fn observed_table(&mut self, rows: &Rows, tables: &Tables, cover: &Cover, value: &[Value]) {
        let of = |a: AtomId| value[a as usize];
        for (context, own) in &rows.rows {
            for &o in own {
                let lits = context.iter().map(|&a| (of(a), false));
                self.clause_of(lits.chain([(of(o.atom()), o.is_positive())]));
            }
        }
        let sets: Vec<&[AtomId]> = rows
            .parents
            .iter()
            .map(|&s| &tables.sets()[s][..])
            .collect();
        for (&s, set) in rows.parents.iter().zip(&sets) {
            if cover.exhaustive[set[0] as usize] != s {
                self.clause_of(set.iter().map(|&a| (of(a), true)));
            }
        }
        let present: FxHashSet<&[AtomId]> = rows.rows.iter().map(|(c, _)| &c[..]).collect();
        let mut at = vec![0; sets.len()];
        let mut tuple: Vec<AtomId> = sets.iter().map(|set| set[0]).collect();
        loop {
            if !present.contains(&tuple[..]) {
                self.clause_of(tuple.iter().map(|&a| (of(a), false)));
            }
            // The next assignment, the last set's atom fastest.
            let Some(i) = (0..sets.len()).rev().find(|&i| at[i] + 1 < sets[i].len()) else {
                break;
            };
            at[i] += 1;
            tuple[i] = sets[i][at[i]];
            for j in i + 1..sets.len() {
                at[j] = 0;
                tuple[j] = sets[j][0];
            }
        }
    }

    /// The other half of each parameter of an observed table's `rows`, with
    /// each atom's `value` and [`reads`]'s `reads`: a fact that is the one
    /// literal of its rows and that nothing else reads, already implied by
    /// their contexts ([`WeightedCnf::observed_table`]), weighed (w, 1) and
    /// true only where one of those contexts is. That is a clause `¬context
    /// ∨ ¬s` for each other row of the table, since one assignment of the
    /// parents' sets holds and those without a row are ruled out; or, where
    /// that is more literals, `s` implies its one context's literals, or of
    /// several, one of them, each context of two literals or more a
    /// variable of its own.
    fn define_parameters(
        &mut self,
        gp: &GroundProgram,
        rows: &Rows,
        reads: &[u32],
        value: &[Value],
    ) {
        // Each row's context as literals; `None` where it cannot hold.
        let contexts: Vec<Option<Vec<i32>>> = rows
            .rows
            .iter()
            .map(|(context, _)| {
                let lits: Vec<GLit> = context.iter().map(|&a| GLit::Pos(a)).collect();
                literals(&lits, value)
            })
            .collect();
        let all: usize = contexts.iter().flatten().map(|c| c.len() + 1).sum();
        let mut by_fact: FxHashMap<AtomId, Vec<usize>> = FxHashMap::default();
        for (i, (_, own)) in rows.rows.iter().enumerate() {
            if let [GLit::Pos(o)] = own[..] {
                by_fact.entry(o).or_default().push(i);
            }
        }
        let mut facts: Vec<_> = by_fact.into_iter().collect();
        facts.sort_unstable_by_key(|&(o, _)| o);
        for (o, at) in facts {
            let (AtomKind::Fact(_), Value::Literal(x)) = (&gp.kinds[o as usize], value[o as usize])
            else {
                continue;
            };
            if reads[o as usize] as usize != at.len() {
                continue;
            }
            let own: Vec<&Vec<i32>> = at.iter().filter_map(|&i| contexts[i].as_ref()).collect();
            // A context that always holds makes the fact true as written.
            if own.iter().any(|c| c.is_empty()) {
                continue;
            }
            let by_others = all - own.iter().map(|c| c.len() + 1).sum::<usize>();
            let by_own = match &own[..] {
                [] => 1,
                [only] => 2 * only.len(),
                _ => {
                    let defined: usize = own
                        .iter()
                        .filter(|c| c.len() >= 2)
                        .map(|c| 3 * c.len() + 1)
                        .sum();
                    defined + own.len() + 1
                }
            };
            if by_others <= by_own {
                for (i, context) in contexts.iter().enumerate() {
                    if let Some(context) = context
                        && rows.rows[i].1[..] != [GLit::Pos(o)]
                    {
                        let mut clause: Vec<i32> = context.iter().map(|&l| -l).collect();
                        clause.push(-x);
                        clause.sort_unstable_by_key(|y| (y.abs(), *y));
                        self.clause(&clause);
                    }
                }
            } else if let [only] = &own[..] {
                for &l in only.iter() {
                    self.clause(&[-x, l]);
                }
            } else {
                let mut clause = vec![-x];
                for context in &own {
                    clause.push(self.conjunction(context));
                }
                self.clause(&clause);
            }
            // The facts are the first variables, in order.
            debug_assert_eq!(self.weights[x as usize - 1].0, x as u32);
            self.weights[x as usize - 1].2 = 1.0;
        }
    }

    /// No two atoms of `set` true, with each atom's `value`: a clause for
    /// each pair of them but those of two atoms that are literals of their
    /// one body (`literal`), which exclude each other as written.
    fn at_most_one(&mut self, set: &[AtomId], literal: &[bool], value: &[Value]) {
        for (i, &a) in set.iter().enumerate() {
            for &b in &set[i + 1..] {
                if !(literal[a as usize] && literal[b as usize]) {
                    self.clause_of([(value[a as usize], false), (value[b as usize], false)]);
                }
            }
        }
    }

    /// The clause of `lits`, each a value and whether it is read
    /// positively: none where one of them is true, and without those that
    /// are false; none where two are complementary.
    fn clause_of(&mut self, lits: impl IntoIterator<Item = (Value, bool)>) {
        let mut clause = Vec::new();
        for (v, positive) in lits {
            match v {
                Value::Constant(c) if c == positive => return,
                Value::Constant(_) => {}
                Value::Literal(x) => clause.push(if positive { x } else { -x }),
            }
        }
        clause.sort_unstable_by_key(|x: &i32| (x.abs(), *x));
        clause.dedup();
        if clause.windows(2).any(|w| w[0] == -w[1]) {
            return;
        }
        self.clause(&clause);
    }

    /// Scales the count by `10^log10`: a Markov network's `P(e)` scaled to
    /// its partition function (`Input::log10_offset`).
    pub fn scale(&mut self, log10: f64) {
        self.log10_scale += log10;
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

/// The exclusive sets whose atoms are written as clauses, each atom a
/// variable with a clause `¬body ∨ v` per rule and no two of them true
/// (the module's documentation): the sets the cover finds exhaustive,
/// none of whose atoms reads an atom of the set, with an atom that is not
/// the literal of its one body. In every world one body of the set's
/// rules holds and no two of different atoms do, so the clauses make its
/// atom true and, no two true, the others false.
struct SetClauses {
    /// The set of each atom written so, as its index into the tables' sets.
    set: Vec<Option<usize>>,
    /// Whether each atom has one rule of one literal: it is that literal,
    /// without a variable of its own.
    literal: Vec<bool>,
    /// The last atom of each set written so, in the order the atoms are
    /// written, after which no two of the set are made true.
    last: Vec<AtomId>,
}

impl SetClauses {
    /// The sets of `gp`, whose tables are `tables` and their cover `cover`,
    /// written as clauses, its atoms written in `order`.
    fn of(gp: &GroundProgram, tables: &mut Tables, cover: &Cover, order: &[AtomId]) -> Self {
        let n = gp.num_atoms();
        let literal: Vec<bool> = (0..n)
            .map(|a| match tables.by_head()[a] {
                [r] => gp.rules[r].body.len() == 1 && !gp.rules[r].pairs,
                _ => false,
            })
            .collect();
        let mut position = vec![usize::MAX; n];
        for (i, &a) in order.iter().enumerate() {
            position[a as usize] = i;
        }
        let mut set = vec![None; n];
        let mut last = vec![AtomId::MAX; tables.sets().len()];
        for (s, last) in last.iter_mut().enumerate() {
            let members = &tables.sets()[s];
            if cover.exhaustive[members[0] as usize] != s
                || members.iter().all(|&a| literal[a as usize])
            {
                continue;
            }
            let members = members.clone();
            if !tables.independent(gp, s) {
                continue;
            }
            for &a in &members {
                set[a as usize] = Some(s);
            }
            *last = *members
                .iter()
                .max_by_key(|&&a| position[a as usize])
                .expect("a set has atoms");
        }
        SetClauses { set, literal, last }
    }
}

/// How many times each atom is read: by the rules' bodies, the queries and
/// the observations.
fn reads(gp: &GroundProgram) -> Vec<u32> {
    let mut reads = vec![0; gp.num_atoms()];
    let bodies = gp.rules.iter().flat_map(|r| &r.body).map(|l| l.atom());
    for a in bodies.chain(gp.named_atoms()) {
        reads[a as usize] += 1;
    }
    reads
}

/// The value each atom is observed at where its observation is written as
/// clauses of its own, without a variable for the atom
/// ([`WeightedCnf::observation`]): a derived atom that its one observation
/// alone reads ([`reads`]) and no set written as clauses holds
/// ([`SetClauses`]).
fn absorbed(gp: &GroundProgram, sets: &SetClauses, reads: &[u32]) -> Vec<Option<bool>> {
    let mut observed = vec![None; gp.num_atoms()];
    for &(_, id, v) in &gp.evidence {
        if let Some(a) = id {
            observed[a as usize] = Some(v);
        }
    }
    (0..gp.num_atoms())
        .map(|a| {
            observed[a].filter(|_| {
                reads[a] == 1 && gp.kinds[a] == AtomKind::Derived && sets.set[a].is_none()
            })
        })
        .collect()
}

/// An observed atom's rules as the rows of a table
/// ([`WeightedCnf::observed_table`]).
struct Rows {
    /// The exclusive sets every context reads one atom of, in order.
    parents: Vec<usize>,
    /// Each row's context, an atom of each parent set in order, and the
    /// literals of the table's own facts it reads.
    rows: Vec<(Vec<AtomId>, Vec<GLit>)>,
}

impl Rows {
    /// The rules of `h` as the rows of a table: each rule's body the
    /// table's own facts ([`Tables::owner`]) and a context of one positive
    /// atom of each of the same exclusive sets, no two rules of one
    /// context, so that no two rows hold together. `None` where they are
    /// not so.
    fn of(gp: &GroundProgram, tables: &Tables, h: AtomId) -> Option<Rows> {
        let t = tables.of(h);
        let sets = tables.sets().len();
        let mut parents: Option<Vec<usize>> = None;
        let mut rows = Vec::new();
        let mut contexts = FxHashSet::default();
        for &r in &tables.by_head()[h as usize] {
            let rule = &gp.rules[r];
            if rule.pairs {
                return None;
            }
            let (mut read, mut own) = (Vec::new(), Vec::new());
            for &l in &rule.body {
                match l {
                    _ if tables.owner(gp, l.atom()) == Some(t) => own.push(l),
                    GLit::Pos(a) if tables.of(a) < sets => read.push((tables.of(a), a)),
                    _ => return None,
                }
            }
            read.sort_unstable();
            let these: Vec<usize> = read.iter().map(|&(s, _)| s).collect();
            if these.windows(2).any(|w| w[0] == w[1]) {
                return None;
            }
            match &parents {
                Some(p) if *p != these => return None,
                Some(_) => {}
                None => parents = Some(these),
            }
            let context: Vec<AtomId> = read.into_iter().map(|(_, a)| a).collect();
            if !contexts.insert(context.clone()) {
                return None;
            }
            rows.push((context, own));
        }
        Some(Rows {
            parents: parents?,
            rows,
        })
    }

    /// Whether the assignments of the parents' sets are at most
    /// [`ZEROS_PER_ROW`] times the rows and [`MOST_ZEROS`] more.
    fn bounded(&self, tables: &Tables) -> bool {
        self.parents
            .iter()
            .try_fold(1usize, |p, &s| p.checked_mul(tables.sets()[s].len()))
            .is_some_and(|p| p <= ZEROS_PER_ROW * self.rows.len() + MOST_ZEROS)
    }
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
