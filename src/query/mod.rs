//! Evidence and query evaluation, checkpointed between diagram operations.
//!
//! Weighted model counting is unaffected by projection: a ProbLog fact has
//! `w(x) + w(¬x) = 1`, so a variable a formula does not mention marginalises
//! to one and the count over a cone equals the count over the whole fact set.
//! A formula is therefore evaluated on its own scope, and only a conjunction
//! of two formulas needs them brought together on the scope holding both.

mod walk;

use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::time::Instant;

use tididi::Tdd;
use tididi::limits::SparseRoute;
use tididi::vtree::{VarId, VtreeIdx};

use crate::eval::{Count, Counted, Counts, Probabilities, Value};
use crate::ground::{AtomId, GroundProgram};
use crate::tp::{
    CompileError, Compiled, Formula, NegationSource, Negations, Projection, ScopeTable, TddBackend,
    VtreeScope,
};
use walk::EvidenceWalk;

/// Result of a query.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryResult {
    pub atom: crate::syntax::Atom,
    /// `false` for a `query(\+a)`.
    pub positive: bool,
    pub probability: f64,
}

/// The query as ProbLog prints it: the atom, `\+` before a negative one.
impl std::fmt::Display for QueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.positive {
            f.write_str("\\+")?;
        }
        write!(f, "{}", self.atom)
    }
}

/// For the tests: asserts that `results` answer the queries `expected`
/// names, in its order, each within `tolerance`; `what` says which run
/// failed.
#[cfg(test)]
pub(crate) fn assert_answers(
    what: &str,
    results: &[QueryResult],
    expected: &[(&str, f64)],
    tolerance: f64,
) {
    assert_eq!(results.len(), expected.len(), "{what}: number of answers");
    for (r, (atom, p)) in results.iter().zip(expected) {
        let got = r.to_string();
        assert_eq!(got, *atom, "{what}: query order");
        assert!(
            (r.probability - p).abs() < tolerance,
            "{what}: {got} = {} against {p}",
            r.probability
        );
    }
}

/// The sizes, in pairs, of what the query phase built.
#[derive(Clone, Copy, Debug, Default)]
pub struct QuerySizes {
    /// The conjunction of the evidence: with indicators, of their links.
    pub evidence: usize,
    /// The largest formula built, intermediate conjunctions of the evidence
    /// and each query's conjunction with it included.
    pub largest: usize,
    /// The query phase's negations, of the evidence and of the queries.
    pub negations: Negations,
    /// The work units the query phase charged to the backend's work clock;
    /// zero where it ran under a watch, which may stop it and resume it in
    /// a later attempt (`auto`).
    pub work_units: u64,
    /// Seconds of the evidence walk, the conjunction of the observations
    /// or links into the evidence formula.
    pub walk_seconds: f64,
    /// The queries that are not literals of the evidence, each counted
    /// against the evidence formula without building their conjunction,
    /// and their seconds.
    pub products: usize,
    pub product_seconds: f64,
    /// Seconds of counting the evidence formula up and down, once: its
    /// mass, its literals' counts and the products all read that count.
    pub count_seconds: f64,
    /// Counts taken again in `eval::Count`s because the count in floats was
    /// not exact (`eval::Range`): the evidence formula's, which every query
    /// after reads, or a formula's mass.
    pub recounts: usize,
    /// The evidence formula's pairs held as the description of a complete
    /// product (`TddLevel::implicit`) rather than written.
    pub evidence_implicit: usize,
    /// The pairs of the evidence formula's levels its count read as grids
    /// (`eval::grid`) rather than pair by pair.
    pub grid_pairs: usize,
    /// The pairs the evidence formula's count took by each route, up and
    /// down, a count in floats and its recount in `eval::Count`s together.
    pub routes: crate::eval::Routes,
    /// The cuts of `program`'s order or of `decision`'s tree whose names
    /// the query phase kept, and those whose names it expanded
    /// (`tp::settle`).
    pub names_kept: usize,
    pub names_expanded: usize,
    /// The work units settling the names took, in the query phase's.
    pub names_work: u64,
}

impl QuerySizes {
    fn record(&mut self, pairs: usize, watch: Option<usize>) -> Result<(), CompileError> {
        self.largest = self.largest.max(pairs);
        if watch.is_some_and(|w| pairs > w) {
            return Err(CompileError::QueryOversize { pairs });
        }
        Ok(())
    }
}

/// Compute `P(q | evidence)` for every query, and the sizes of what that
/// built, in one run: the tests' query phase of a program compiled on a
/// vtree of their own (`tp::compile`). The answers are `None` when the
/// evidence has probability zero.
#[cfg(test)]
pub(crate) fn query_probabilities(
    gp: &GroundProgram,
    backend: &TddBackend,
    compiled: &Compiled,
) -> Result<(Option<Vec<QueryResult>>, QuerySizes), CompileError> {
    QueryEvaluation::new(gp, backend, compiled).run(gp, backend, compiled, None)
}

/// Multiply-adds of the dense walk that counts a query against the evidence
/// ([`crate::eval::conjunction_count`]) per unit of the engine's work clock,
/// whose unit is about one product pair of a conjunction. The walk does
/// four to nine times as many multiply-adds a second as the engine does
/// units, so eight cost about one unit.
const MULTIPLY_ADDS_PER_UNIT: u64 = 8;

/// State belongs to one immutable ground program, compilation and vtree.
/// The engine may change between calls, but these three inputs must not,
/// other than by the formulas the phase takes ([`Self::take_last_reads`]).
pub(crate) struct QueryEvaluation {
    weights: Probabilities,
    scopes: RefCell<ScopeTable>,
    indicator_of: FxHashMap<AtomId, AtomId>,
    impossible: bool,
    walk: EvidenceWalk,
    evidence: Option<Formula>,
    mass: Option<Count>,
    /// The evidence formula counted, where a query reads it.
    counted: Option<CountedEvidence>,
    out: Vec<QueryResult>,
    /// The queries to answer.
    queries: usize,
    pending_query: Option<PendingQuery>,
    sizes: QuerySizes,
    /// Whether the walk holds formulas taken from the compile, and whether
    /// a run failed since, which may have consumed one: the phase cannot
    /// go on then.
    took: bool,
    spent: bool,
}

impl QueryEvaluation {
    pub(crate) fn new(gp: &GroundProgram, backend: &TddBackend, compiled: &Compiled) -> Self {
        let mut scopes = ScopeTable::default();
        for scope in &compiled.scopes {
            if let Some(vars) = scope.vars()
                && scopes.get(vars).is_none()
            {
                scopes.insert(scope);
            }
        }
        // The links of names expanded are not observed (`tp::settle`).
        let dropped = &compiled.for_queries.dropped;
        let observed = gp
            .evidence
            .iter()
            .filter_map(|(_, a, value)| a.map(|a| (a, *value)))
            .filter(|(a, _)| !dropped.contains(a))
            .collect();
        // The copies of names expanded, which no formula reads, are summed
        // to one as a fact is.
        let mut weights =
            Probabilities::from_program(gp, &compiled.var_of, backend.vtree().num_vars());
        for v in &compiled.for_queries.released {
            let i = v.0 as usize;
            (weights.pos[i], weights.neg[i], weights.one[i]) = (0.5, 0.5, 1.0);
        }
        let settled = &compiled.for_queries.settled;
        Self {
            weights,
            scopes: RefCell::new(scopes),
            indicator_of: gp.encoding.first_copies(),
            impossible: gp
                .evidence
                .iter()
                .any(|(_, a, value)| a.is_none() && *value),
            walk: EvidenceWalk::new(observed, &backend.vtree()),
            evidence: None,
            mass: None,
            counted: None,
            out: Vec::new(),
            queries: gp.queries.len(),
            pending_query: None,
            took: false,
            spent: false,
            sizes: QuerySizes {
                recounts: compiled.for_queries.recounts,
                negations: settled.negations,
                names_kept: settled.kept,
                names_expanded: settled.expanded,
                names_work: settled.work,
                ..QuerySizes::default()
            },
        }
    }

    /// The steps of the phase finished, and all of them: the evidence
    /// walk's joins, then the queries answered.
    pub(crate) fn progress(&self) -> (usize, usize) {
        let (joined, joins) = self.walk.progress();
        (joined + self.out.len(), joins + self.queries)
    }

    /// The weighted count of the evidence formula, after a call of
    /// [`run`](Self::run) that succeeded: zero where an observation cannot
    /// hold, one where nothing is observed. With indicators it is the count
    /// of the links as well, which admit one value of the indicators in
    /// each world: `WMC(e ∧ L) = P(e)` of the program as grounded.
    pub(crate) fn evidence_mass(&self) -> Count {
        if self.impossible {
            return Count::ZERO;
        }
        assert!(
            self.walk.done,
            "the evidence is counted once the phase has answered"
        );
        self.mass.unwrap_or(Count::ONE)
    }

    /// Hands the evidence walk the stored formulas of the atoms it reads
    /// last ([`EvidenceWalk::take_last_reads`]), to negate or embed rather
    /// than copy: of each atom observed that no query reads, as itself or
    /// through a template, and no tie stands on. For the phase's last run
    /// only, as one without a watch is ([`crate::evaluation::Evaluation`]):
    /// after an error, there is none.
    pub(crate) fn take_last_reads(&mut self, gp: &GroundProgram, compiled: &mut Compiled) {
        let for_queries = &compiled.for_queries;
        let mut read_after: FxHashSet<AtomId> =
            for_queries.ties.values().map(|t| t.reference).collect();
        for &(_, q, _) in &gp.queries {
            if let Some(q) = q {
                read_after.insert(q);
                read_after.extend(for_queries.templates.get(&q).map(|t| t.atom));
            }
        }
        self.took |= self
            .walk
            .take_last_reads(compiled, |a| read_after.contains(&a));
    }

    /// A completed result is consumed by the caller; do not run again after
    /// success. On an error, completed conjunctions and answers are
    /// retained, unless the walk took formulas ([`Self::take_last_reads`]).
    ///
    /// # Panics
    ///
    /// When a run failed after the walk took formulas.
    pub(crate) fn run(
        &mut self,
        gp: &GroundProgram,
        backend: &TddBackend,
        compiled: &Compiled,
        watch: Option<usize>,
    ) -> Result<(Option<Vec<QueryResult>>, QuerySizes), CompileError> {
        assert!(
            !self.spent,
            "a query phase that failed after taking formulas cannot go on"
        );
        let result = self.answer(gp, backend, compiled, watch);
        self.spent = self.took && result.is_err();
        result
    }

    /// [`Self::run`] but for its check.
    fn answer(
        &mut self,
        gp: &GroundProgram,
        backend: &TddBackend,
        compiled: &Compiled,
        watch: Option<usize>,
    ) -> Result<(Option<Vec<QueryResult>>, QuerySizes), CompileError> {
        let _route = backend
            .engine
            .limits()
            .edit(|c| c.with_sparse_route(QUERY_ROUTE));
        let sizes = &mut self.sizes;
        // A result that exceeded the previous watch was saved. It may only
        // feed further operations once the new allowance admits it.
        sizes.record(sizes.largest, watch)?;
        if self.impossible {
            return Ok((None, *sizes));
        }
        let global_weights = &self.weights;
        let weights_for = |scope: &VtreeScope| match scope.vars() {
            None => global_weights.clone(),
            Some(vars) => global_weights.restrict(vars),
        };
        // The variables an atom's formula may mention: its component's scope for
        // a derived atom, the fact itself for a fact. `None` is the whole
        // program, which absorbs everything.
        let atom_vars = |a: AtomId| -> Option<Vec<VarId>> {
            if compiled.formulas[a as usize].is_some() {
                compiled.scope_of(a).vars().map(<[VarId]>::to_vec)
            } else if compiled.projection == Projection::Off {
                None
            } else {
                compiled.var_of[a as usize].map(|v| vec![v])
            }
        };
        // Scopes the query phase builds are interned like the compiler's, and
        // the compiler's are put in first: two formulas end up on one vtree
        // allocation whenever they hold the same variables, which is what makes
        // an embed between them free.
        let scopes = &self.scopes;
        // The scope over `vars`, ascending and distinct.
        let scope_over = |vars: &[VarId]| -> Result<VtreeScope, CompileError> {
            if vars.is_empty() {
                return Ok(backend.global());
            }
            if let Some(scope) = scopes.borrow().get(vars) {
                return Ok(scope);
            }
            let scope = backend.scope(vars)?;
            scopes.borrow_mut().insert(&scope);
            Ok(scope)
        };
        // The scope over `vars` in any order; `None` is the whole program.
        let scope_for = |vars: Option<Vec<VarId>>| -> Result<VtreeScope, CompileError> {
            let Some(mut vars) = vars else {
                return Ok(backend.global());
            };
            vars.sort_unstable();
            vars.dedup();
            scope_over(&vars)
        };
        // The smallest scope an atom's formula already lives on.
        let own_scope = |a: AtomId| -> Result<VtreeScope, CompileError> {
            if compiled.formulas[a as usize].is_some() {
                Ok(compiled.scope_of(a).clone())
            } else {
                scope_for(atom_vars(a))
            }
        };

        if !self.walk.done {
            let t = Instant::now();
            let walked = self
                .walk
                .run(backend, compiled, &own_scope, &scope_over, sizes, watch);
            sizes.walk_seconds += t.elapsed().as_secs_f64();
            self.evidence = walked?;
        }
        // A query atom with a variable of its own on the evidence's scope, a
        // fact or an atom read through an indicator, is a literal of the
        // evidence formula: `P(q | e) = WMC(e AND x) / WMC(e)`, and one pass
        // down the evidence formula gives that count for every variable at
        // once ([`crate::eval::Counted::with_literal`]). With indicators the
        // evidence holds every link, and the formula of a query is counted
        // against all of it otherwise, once per query.
        let indicator_of = &self.indicator_of;
        let literal_of = |a: AtomId, scope: &VtreeScope| -> Option<usize> {
            let fact = if gp.is_fact(a) {
                a
            } else {
                *indicator_of.get(&a)?
            };
            let v = compiled.var_of[fact as usize]?;
            match scope.vars() {
                None => Some(v.0 as usize),
                Some(vars) => vars.iter().position(|&w| w == v).map(|i| i + 1),
            }
        };
        // The evidence's weights, restricted to its scope once: every literal
        // query reads them.
        let e_w = self.evidence.as_ref().map(|(_, scope)| weights_for(scope));
        if let Some((e, scope)) = &self.evidence {
            let w = e_w.as_ref().unwrap();
            sizes.evidence = backend.size(e);
            sizes.evidence_implicit = backend.implicit_size(e);
            sizes.record(sizes.evidence, None)?;
            if self.mass.is_none() {
                // The mass is the count's pass up, with or without queries:
                // its levels under powers of two keep a count in floats that
                // the engine's evaluation, one range for the whole diagram,
                // would take again in `Count`s. The queries read the pass
                // down too, which is taken when one reads it.
                let t = Instant::now();
                if self.counted.is_none() {
                    let c = CountedEvidence::new(e, scope, w, &mut sizes.recounts)?;
                    sizes.grid_pairs = c.counted.grid_pairs();
                    self.counted = Some(c);
                }
                self.mass = Some(self.counted.as_ref().unwrap().total(e));
                sizes.count_seconds += t.elapsed().as_secs_f64();
            }
            // Only a mass that is zero as a count: in a float the
            // probability of a thousand observations is zero too.
            if self.mass.unwrap().is_zero() {
                sizes.routes = self
                    .counted
                    .as_ref()
                    .map(CountedEvidence::routes)
                    .unwrap_or_default();
                return Ok((None, *sizes));
            }
        }
        let counted = &mut self.counted;
        let evidence = self
            .evidence
            .as_ref()
            .map(|(e, scope)| (e, scope, self.mass.unwrap()));
        // The stored formula of a query atom is counted as it is. A query of
        // the other polarity is `1 - m` from the stored formula's count `m`,
        // or where that is too small to keep its digits, the count of the
        // complement's formula (`eval::complement`). A fact's literal and a
        // literal of the evidence are counted in the polarity asked.
        let out = &mut self.out;
        // A query that is not a literal of the evidence is answered by
        // counting `q AND e` without building it (`eval::conjunction_count`):
        // each query's formula is walked against the counted evidence from
        // the level of its cone, its variables the evidence does not hold
        // summed inside it. Conjoining each query with the whole evidence
        // formula, minimizing and counting the product is most of the run
        // on networks with many queries.
        for (atom, id, positive) in gp.queries.iter().skip(out.len()) {
            // A literal's count reads the pass down, walked by the first
            // literal under each level.
            let t = Instant::now();
            let literal = match (id, &evidence, counted.as_mut()) {
                (Some(a), Some((e, e_scope, mass)), Some(c)) => literal_of(*a, e_scope)
                    .map(|x| -> Result<f64, tididi::OperationError> {
                        let var = VarId(x as u32);
                        let leaf = e
                            .vtree()
                            .leaf_of(var)
                            .expect("a scope's variables are its leaves");
                        let w = e_w.as_ref().unwrap();
                        loop {
                            let p = match &c.counted {
                                Counts::Plain(plain) => {
                                    literal_ratio(plain, e, leaf, var, *positive, w, *mass)?
                                }
                                Counts::Wide(wide) => {
                                    literal_ratio(wide, e, leaf, var, *positive, w, *mass)?
                                }
                            };
                            match p {
                                Some(p) => break Ok(p),
                                None => c.widen(&mut sizes.recounts)?,
                            }
                        }
                    })
                    .transpose()?,
                _ => None,
            };
            if literal.is_some() {
                sizes.count_seconds += t.elapsed().as_secs_f64();
            }
            let p = match (id, literal) {
                (None, _) => {
                    if *positive {
                        0.0
                    } else {
                        1.0
                    }
                }
                (Some(_), Some(p)) => p,
                // Counted by the compile where it dropped the formula.
                (Some(a), None) if compiled.for_queries.counted.contains_key(a) => {
                    let (atom, negation) = compiled.for_queries.counted[a];
                    if *positive { atom } else { negation }
                }
                (Some(a), None) => {
                    // A query answered through a template counts the
                    // template's formula under the probabilities it gives.
                    let template = compiled.for_queries.templates.get(a);
                    let a = &template.map_or(*a, |t| t.atom);
                    let weights_of = |scope: &VtreeScope| {
                        let w = weights_for(scope);
                        match template {
                            Some(t) => t.weights(&w, &compiled.var_of, scope),
                            None => w,
                        }
                    };
                    let stored = compiled.formulas[*a as usize].as_deref();
                    // The polarity of the formula counted first: the stored
                    // formula's, or for a fact the polarity asked.
                    let first = if stored.is_some() {
                        compiled.stored_positive(*a)
                    } else {
                        *positive
                    };
                    let pending_query = &mut self.pending_query;
                    // `P(l | e)` for the literal `l` of polarity `polarity` on
                    // `a`, from the formula of `l`: the stored formula where
                    // its polarity is `polarity`, its negation otherwise, a
                    // literal for a fact.
                    let mut count = |polarity: bool| -> Result<f64, CompileError> {
                        let t = Instant::now();
                        let mass = match &evidence {
                            None => {
                                // Nothing to conjoin: the formula is counted where it
                                // was built.
                                if pending_query
                                    .as_ref()
                                    .is_none_or(|p| p.positive != polarity)
                                {
                                    sizes.products += 1;
                                    let formula = match stored {
                                        Some(_) if compiled.stored_positive(*a) == polarity => None,
                                        _ => {
                                            let scope = own_scope(*a)?;
                                            Some((
                                                compiled.literal_formula(
                                                    backend,
                                                    &scope,
                                                    *a,
                                                    polarity,
                                                    (&mut sizes.negations, NegationSource::Query),
                                                )?,
                                                scope,
                                            ))
                                        }
                                    };
                                    *pending_query = Some(PendingQuery {
                                        formula,
                                        positive: polarity,
                                    });
                                }
                                let (f, scope) = match &pending_query.as_ref().unwrap().formula {
                                    Some((f, scope)) => (f, scope),
                                    None => (
                                        stored.expect("the stored polarity's formula is stored"),
                                        compiled.scope_of(*a),
                                    ),
                                };
                                sizes.record(backend.size(f), watch)?;
                                crate::eval::mass(
                                    backend.engine,
                                    f,
                                    &weights_of(scope),
                                    &mut sizes.recounts,
                                )?
                                .to_f64()
                            }
                            Some((e, e_scope, e_mass)) => {
                                sizes.products += 1;
                                let c = counted
                                    .as_mut()
                                    .expect("the evidence is counted where a query reads it");
                                // The query formula is counted where it was built; the
                                // walk reads its embedding onto the evidence's scope
                                // off the map of its variables.
                                let (own, literal);
                                let (q, q_scope) = match stored {
                                    Some(f) if compiled.stored_positive(*a) == polarity => {
                                        (f, compiled.scope_of(*a))
                                    }
                                    _ => {
                                        own = own_scope(*a)?;
                                        literal = compiled.literal_formula(
                                            backend,
                                            &own,
                                            *a,
                                            polarity,
                                            (&mut sizes.negations, NegationSource::Query),
                                        )?;
                                        (&literal, &own)
                                    }
                                };
                                let q_w = weights_of(q_scope);
                                let e_vtree = e.vtree();
                                let leaf_in_e = |v: VarId| {
                                    let global =
                                        q_scope.vars().map_or(v, |vars| vars[v.0 as usize - 1]);
                                    let local = e_scope.local_of(global)?;
                                    Some(
                                        e_vtree
                                            .leaf_of(local)
                                            .expect("a scope's variables are its leaves"),
                                    )
                                };
                                // The walk's multiply-adds charged to the engine's
                                // work clock, so a trial's work budget covers them,
                                // once: a walk again in counts is not charged.
                                let host =
                                    |units: u64| backend.engine.limits().poll_host_work(units);
                                let mut charged = false;
                                loop {
                                    let p = match &c.counted {
                                        Counts::Plain(plain) => joint_ratio(
                                            e,
                                            plain,
                                            &c.spans,
                                            q,
                                            &q_w,
                                            &leaf_in_e,
                                            *e_mass,
                                            &mut charged,
                                            &host,
                                        )?,
                                        Counts::Wide(wide) => joint_ratio(
                                            e,
                                            wide,
                                            &c.spans,
                                            q,
                                            &q_w,
                                            &leaf_in_e,
                                            *e_mass,
                                            &mut charged,
                                            &host,
                                        )?,
                                    };
                                    match p {
                                        Some(p) => break p,
                                        None => c.widen(&mut sizes.recounts)?,
                                    }
                                }
                            }
                        };
                        sizes.product_seconds += t.elapsed().as_secs_f64();
                        Ok(mass)
                    };
                    let m = count(first)?;
                    if first == *positive {
                        m
                    } else {
                        crate::eval::complement(m, || count(*positive))?
                    }
                }
            };
            self.pending_query = None;
            out.push(QueryResult {
                atom: atom.clone(),
                positive: *positive,
                probability: p.clamp(0.0, 1.0),
            });
        }
        sizes.routes = counted
            .as_ref()
            .map(CountedEvidence::routes)
            .unwrap_or_default();
        Ok((Some(std::mem::take(out)), *sizes))
    }
}

/// The evidence formula counted up and down once, on its own scope: its
/// mass, the count with each of its literals and the walks of the queries
/// that are not its literals all read it.
struct CountedEvidence {
    counted: Counts,
    /// The routes of a count this one replaced ([`CountedEvidence::widen`]).
    widened: crate::eval::Routes,
    /// The pre-order spans of the scope's vtree, for the walk's ancestor tests.
    spans: Vec<(u32, u32)>,
}

impl CountedEvidence {
    fn new(
        e: &Tdd,
        scope: &VtreeScope,
        w: &Probabilities,
        recounts: &mut usize,
    ) -> Result<Self, tididi::OperationError> {
        Ok(CountedEvidence {
            counted: crate::eval::count_exact(e, w, recounts)?,
            widened: crate::eval::Routes::default(),
            spans: crate::eval::spans(scope.vtree()),
        })
    }

    /// The evidence's count taken over in [`Count`]s, where a query read
    /// against its count in floats was not exact ([`Counts::widen`]);
    /// `recounts` counts it.
    fn widen(&mut self, recounts: &mut usize) -> Result<(), tididi::OperationError> {
        *recounts += 1;
        self.widened = self.counted.widen()?;
        Ok(())
    }

    /// The pairs the count's passes took by each route, with those of a
    /// count it replaced.
    fn routes(&self) -> crate::eval::Routes {
        self.widened.plus(self.counted.routes())
    }

    /// The mass, exactly.
    fn total(&self, e: &Tdd) -> Count {
        match &self.counted {
            Counts::Plain(plain) => plain.total(e).count(),
            Counts::Wide(wide) => wide.total(e).count(),
        }
    }
}

/// `P(x | e)` for the variable `var` at `leaf` of the evidence formula `e`
/// of mass `mass`, counted in `c`; `None` where the count is not exact.
fn literal_ratio<V: Value>(
    c: &Counted<V>,
    e: &Tdd,
    leaf: VtreeIdx,
    var: VarId,
    positive: bool,
    w: &Probabilities,
    mass: Count,
) -> Result<Option<f64>, tididi::OperationError> {
    Ok(c.with_literal(e, leaf, var, positive, w)?
        .map(|with| with.ratio(mass)))
}

/// `P(q | e)` for the query formula `q`, counted against the evidence
/// formula `e` of mass `mass` (`eval::conjunction_count`); `None` where the
/// count is not exact. The walk's work is charged through `host`
/// unless `charged` says a walk of this query already was, and a walk that
/// finishes sets it.
#[allow(clippy::too_many_arguments)]
fn joint_ratio<V: Value>(
    e: &Tdd,
    c: &Counted<V>,
    spans: &[(u32, u32)],
    q: &Tdd,
    q_w: &Probabilities,
    leaf_in_e: &impl Fn(VarId) -> Option<VtreeIdx>,
    mass: Count,
    charged: &mut bool,
    host: &impl Fn(u64) -> Result<(), tididi::OperationError>,
) -> Result<Option<f64>, tididi::OperationError> {
    let q_counted = crate::eval::count::<V>(q, q_w)?;
    let charge = !*charged;
    let poll = |madds: u64| {
        if charge {
            host(madds.div_ceil(MULTIPLY_ADDS_PER_UNIT))
        } else {
            Ok(())
        }
    };
    let (joint, exact) =
        crate::eval::conjunction_count(e, c, spans, q, &q_counted, leaf_in_e, poll)?;
    *charged = true;
    Ok(exact.then(|| joint.ratio(mass)))
}

struct PendingFormula {
    formula: Formula,
    reduced: bool,
}

/// A query's formula kept across a size stop, and the polarity of the
/// literal it is the formula of: the formula built, or `None` for the
/// atom's stored formula, read in place.
struct PendingQuery {
    formula: Option<Formula>,
    positive: bool,
}

/// The sparse route the query phase's conjunctions take. The walk conjoins a
/// large accumulator with small factors, and at the levels above a factor's
/// variables the dense grid of the two is mostly empty. The compile keeps
/// the library's default (64, 4096), which is faster there.
const QUERY_ROUTE: SparseRoute = SparseRoute {
    sparsity: 16,
    min_grid: 1024,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tp::{Formulation, compile, fact_variables};
    use crate::{GroundOptions, vtree::Kind};
    use tididi::OperationError;
    use tididi::limits::{LimitConfig, StopAt, StopRules};

    fn ground(source: &str) -> GroundProgram {
        crate::ground::ground(
            &crate::syntax::parse_program(source).unwrap(),
            &GroundOptions::default(),
        )
        .unwrap()
    }

    #[test]
    fn completed_evidence_pairs_survive_size_stops() {
        // Each observed fact queried, so that the evidence is not dropped
        // as detached (src/ground/detach.rs).
        let mut source = String::new();
        for i in 0..13 {
            source.push_str(&format!("0.5::f({i}).\nquery(f({i})).\n"));
            if i < 12 {
                source.push_str(&format!("evidence(f({i}),true).\n"));
            }
        }
        let gp = ground(&source);
        let (
            crate::vtree::Layout {
                vtree: tree, order, ..
            },
            _,
        ) = crate::vtree::build(&gp, &crate::ground::deps::Deps::of(&gp), Kind::Closed).unwrap();
        let compiled = tree.context().run(|engine| {
            let backend = TddBackend::new(engine, tree.clone());
            compile(
                &gp,
                fact_variables(&gp, &order),
                Formulation::Dual,
                &backend,
                Projection::default(),
                None,
            )
            .unwrap()
        });
        let mut state = tree.context().run(|engine| {
            QueryEvaluation::new(&gp, &TddBackend::new(engine, tree.clone()), &compiled)
        });
        let mut joined = None;
        for _ in 0..2 {
            tree.context().run(|engine| {
                assert!(matches!(
                    state.run(
                        &gp,
                        &TddBackend::new(engine, tree.clone()),
                        &compiled,
                        Some(1)
                    ),
                    Err(CompileError::QueryOversize { .. })
                ));
            });
            assert!(state.walk.completed_joins > 0);
            let previous = joined.get_or_insert(state.walk.completed_joins);
            assert_eq!(
                state.walk.completed_joins, *previous,
                "the oversized result must survive a repeated stop"
            );
        }
        let (answers, _) = tree
            .context()
            .run(|engine| state.run(&gp, &TddBackend::new(engine, tree.clone()), &compiled, None))
            .unwrap();
        assert_eq!(
            state.walk.completed_joins, 11,
            "each of the twelve factors is joined exactly once"
        );
        let answers = answers.unwrap();
        assert_eq!(answers[0].probability, 1.0);
        assert_eq!(answers[12].probability, 0.5);
    }

    #[test]
    fn completed_answers_and_pending_query_survive_size_stops() {
        let gp = ground("0.5::a. 0.4::b. q :- a,b. query(missing). query(q). query(\\+q).");
        let (
            crate::vtree::Layout {
                vtree: tree, order, ..
            },
            _,
        ) = crate::vtree::build(&gp, &crate::ground::deps::Deps::of(&gp), Kind::Closed).unwrap();
        tree.context().run(|engine| {
            let backend = TddBackend::new(engine, tree.clone());
            let compiled = compile(
                &gp,
                fact_variables(&gp, &order),
                Formulation::Primal,
                &backend,
                Projection::default(),
                None,
            )
            .unwrap();
            // Without evidence the compile counts `q` and drops its formula.
            // Its formula is put back as a count the engine stopped leaves
            // it, for the query phase to count.
            let mut compiled = compiled;
            let q = gp.queries[1].1.unwrap();
            let id =
                |name: &str| gp.atoms.iter().position(|a| a.to_string() == name).unwrap() as AtomId;
            assert!(
                compiled.for_queries.counted.contains_key(&q)
                    && compiled.formulas[q as usize].is_none()
            );
            let scope = compiled.scope_of(q).clone();
            let mut negations = Negations::default();
            let mut literal = |x| {
                compiled
                    .literal_formula(
                        &backend,
                        &scope,
                        id(x),
                        true,
                        (&mut negations, NegationSource::Query),
                    )
                    .unwrap()
            };
            let (a, b) = (literal("a"), literal("b"));
            compiled.formulas[q as usize] = Some(std::sync::Arc::new(backend.and(a, b).unwrap()));
            compiled.for_queries.counted.remove(&q);
            let mut state = QueryEvaluation::new(&gp, &backend, &compiled);
            assert!(matches!(
                state.run(&gp, &backend, &compiled, Some(0)),
                Err(CompileError::QueryOversize { .. })
            ));
            assert_eq!(state.out.len(), 1);
            assert!(state.pending_query.is_some());
            let answers = state
                .run(&gp, &backend, &compiled, None)
                .unwrap()
                .0
                .unwrap();
            for (answer, expected) in answers.iter().zip([0.0, 0.2, 0.8]) {
                assert!((answer.probability - expected).abs() < 1e-12);
            }
        });
    }

    #[test]
    fn engine_stops_preserve_conditionals_in_both_formulations_and_rewrites() {
        // Evidence that fixes no atom, so that the grounder keeps all three
        // observations (src/ground/condition.rs). Of the 16 worlds of f(0)
        // to f(3), 9 have g false and k true, 2 of them h0 and 2 h2.
        let source = "0.5::f(0). 0.5::f(1). 0.5::f(2). 0.5::f(3).
            0.5::f(4). 0.5::f(5). 0.5::f(6). 0.5::f(7).
            h0 :- f(0),f(1). h2 :- f(2),f(3). h4 :- f(4). h4 :- f(5). h6 :- f(6),f(7).
            g :- f(0),f(3). k :- f(1). k :- f(2).
            q :- h0,h6. evidence(g,false). evidence(k,true). evidence(h4,true).
            query(h0). query(h2). query(h4). query(h6). query(q). query(\\+q).";
        for indicators in [false, true] {
            let gp = if indicators {
                crate::indicators::introduce(&ground(source))
            } else {
                ground(source)
            };
            for form in [Formulation::Primal, Formulation::Dual] {
                for projection in [Projection::Off, Projection::default()] {
                    let (
                        crate::vtree::Layout {
                            vtree: tree, order, ..
                        },
                        _,
                    ) = crate::vtree::build(&gp, &crate::ground::deps::Deps::of(&gp), Kind::Closed)
                        .unwrap();
                    let compiled = tree.context().run(|engine| {
                        compile(
                            &gp,
                            fact_variables(&gp, &order),
                            form,
                            &TddBackend::new(engine, tree.clone()),
                            projection,
                            None,
                        )
                        .unwrap()
                    });
                    let mut state = tree.context().run(|engine| {
                        QueryEvaluation::new(&gp, &TddBackend::new(engine, tree.clone()), &compiled)
                    });
                    let mut budget = 1;
                    let mut stops = 0;
                    // The steps finished grow with each stop, out of the
                    // walk's joins and the queries.
                    let mut done = 0;
                    let (answers, sizes) = loop {
                        let result = tree.context().run(|engine| {
                            let limit = LimitConfig::none().with_stop_rules(StopRules {
                                unconditional: Some(StopAt::WorkUnits(
                                    engine.limits().work_units() + budget,
                                )),
                                after_pairs: None,
                            });
                            let _guard = engine.limits().scope(limit);
                            state.run(&gp, &TddBackend::new(engine, tree.clone()), &compiled, None)
                        });
                        match result {
                            Ok(result) => break result,
                            Err(CompileError::Tdd(OperationError::Stopped)) => {
                                let (now, all) = state.progress();
                                assert_eq!(all, gp.evidence.len() - 1 + gp.queries.len());
                                assert!(done <= now && now < all, "{done} {now} {all}");
                                done = now;
                                stops += 1;
                                budget *= 2;
                            }
                            Err(e) => panic!("{e}"),
                        }
                        assert!(stops < 24, "a growing budget must eventually complete");
                    };
                    assert!(stops > 1);
                    assert_eq!(state.walk.completed_joins, gp.evidence.len() - 1);
                    let answers = answers.unwrap();
                    assert_eq!(answers.len(), 6);
                    for (answer, expected) in answers.iter().zip([
                        2.0 / 9.0,
                        2.0 / 9.0,
                        1.0,
                        0.25,
                        1.0 / 18.0,
                        17.0 / 18.0,
                    ]) {
                        assert!(
                            (answer.probability - expected).abs() < 1e-12,
                            "{answer:?} indicators {indicators} {form:?}"
                        );
                    }
                    let (fresh, fresh_sizes) = tree
                        .context()
                        .run(|engine| {
                            query_probabilities(
                                &gp,
                                &TddBackend::new(engine, tree.clone()),
                                &compiled,
                            )
                        })
                        .unwrap();
                    assert_eq!(Some(answers), fresh);
                    assert_eq!(sizes.evidence, fresh_sizes.evidence);
                    assert_eq!(sizes.largest, fresh_sizes.largest);
                }
            }
        }
    }

    /// The source of the tests of the walk's taken formulas: `g` observed
    /// false and `k` true, neither queried, and `h4` observed and queried.
    const TAKEN_SOURCE: &str = "0.5::f(0). 0.5::f(1). 0.5::f(2). 0.5::f(3).
        0.5::f(4). 0.5::f(5). 0.5::f(6). 0.5::f(7).
        h0 :- f(0),f(1). h2 :- f(2),f(3). h4 :- f(4). h4 :- f(5). h6 :- f(6),f(7).
        g :- f(0),f(3). k :- f(1). k :- f(2).
        q :- h0,h6. evidence(g,false). evidence(k,true). evidence(h4,true).
        query(h0). query(h2). query(h4). query(h6). query(q). query(\\+q).";

    #[test]
    fn the_walk_takes_the_formulas_it_reads_last() {
        let gp = ground(TAKEN_SOURCE);
        let id = |name: &str| gp.atoms.iter().position(|a| a.to_string() == name).unwrap();
        for form in [Formulation::Primal, Formulation::Dual] {
            for projection in [Projection::Off, Projection::default()] {
                let (
                    crate::vtree::Layout {
                        vtree: tree, order, ..
                    },
                    _,
                ) = crate::vtree::build(&gp, &crate::ground::deps::Deps::of(&gp), Kind::Closed)
                    .unwrap();
                // The answers and the negations, and which of g, k and h4
                // keep their stored formulas, with and without the take.
                let run = |take: bool| {
                    tree.context().run(|engine| {
                        let backend = TddBackend::new(engine, tree.clone());
                        let mut compiled = compile(
                            &gp,
                            fact_variables(&gp, &order),
                            form,
                            &backend,
                            projection,
                            None,
                        )
                        .unwrap();
                        let mut state = QueryEvaluation::new(&gp, &backend, &compiled);
                        if take {
                            state.take_last_reads(&gp, &mut compiled);
                        }
                        let held = ["g", "k", "h4"].map(|x| compiled.formulas[id(x)].is_some());
                        let (answers, sizes) = state.run(&gp, &backend, &compiled, None).unwrap();
                        (answers.unwrap(), sizes.negations, held)
                    })
                };
                let (copied, taken) = (run(false), run(true));
                assert_eq!(copied.0, taken.0);
                assert_eq!(copied.1, taken.1);
                assert_eq!(copied.2, [true; 3]);
                assert_eq!(taken.2, [false, false, true], "{form:?} {projection:?}");
            }
        }
    }

    #[test]
    #[should_panic(expected = "cannot go on")]
    fn a_walk_that_took_formulas_does_not_go_on_after_an_error() {
        let gp = ground(TAKEN_SOURCE);
        let (
            crate::vtree::Layout {
                vtree: tree, order, ..
            },
            _,
        ) = crate::vtree::build(&gp, &crate::ground::deps::Deps::of(&gp), Kind::Closed).unwrap();
        tree.context().run(|engine| {
            let backend = TddBackend::new(engine, tree.clone());
            let mut compiled = compile(
                &gp,
                fact_variables(&gp, &order),
                Formulation::Dual,
                &backend,
                Projection::default(),
                None,
            )
            .unwrap();
            let mut state = QueryEvaluation::new(&gp, &backend, &compiled);
            state.take_last_reads(&gp, &mut compiled);
            {
                let limit = LimitConfig::none().with_stop_rules(StopRules {
                    unconditional: Some(StopAt::WorkUnits(engine.limits().work_units() + 1)),
                    after_pairs: None,
                });
                let _guard = engine.limits().scope(limit);
                assert!(matches!(
                    state.run(&gp, &backend, &compiled, None),
                    Err(CompileError::Tdd(OperationError::Stopped))
                ));
            }
            let _ = state.run(&gp, &backend, &compiled, None);
        });
    }

    #[test]
    fn waiting_joins_keep_the_answers_through_stops() {
        // A join that may wait does where its result outgrows its operands
        // (operands of a pair or more, a cap of their pairs), so that the
        // walk's last join reassociates where its other operand is the
        // smallest; stopped on the work clock at every budget and resumed,
        // the answers are the walk's without waiting.
        let conditionals = "0.5::f(0). 0.5::f(1). 0.5::f(2). 0.5::f(3).
            0.5::f(4). 0.5::f(5). 0.5::f(6). 0.5::f(7).
            h0 :- f(0),f(1). h2 :- f(2),f(3). h4 :- f(4). h4 :- f(5). h6 :- f(6),f(7).
            g :- f(0),f(3). k :- f(1). k :- f(2).
            q :- h0,h6. evidence(g,false). evidence(k,true). evidence(h4,true).
            query(h0). query(h2). query(h4). query(h6). query(q). query(\\+q).";
        // Rings of facts, each atom of them true in some world where a
        // third of the facts are false, observed as it is there but every
        // `skip`th.
        let ring = |n: usize, skip: usize| {
            let world = |i: usize| !i.is_multiple_of(3);
            let mut ring = String::new();
            for i in 0..n {
                ring.push_str(&format!(
                    "{}::f({i}).\nquery(f({i})).\n",
                    0.2 + 0.6 * (i as f64) / (n as f64)
                ));
                ring.push_str(&format!(
                    "h({i}) :- f({i}),f({}).\nh({i}) :- f({}),f({}).\n",
                    (i + 1) % n,
                    (i + 2) % n,
                    (i + 5) % n
                ));
                let h =
                    (world(i) && world((i + 1) % n)) || (world((i + 2) % n) && world((i + 5) % n));
                if i % skip != skip - 1 {
                    ring.push_str(&format!("evidence(h({i}),{h}).\n"));
                }
            }
            ring
        };
        let rings: Vec<String> = [10, 14, 18, 22]
            .into_iter()
            .flat_map(|n| [3, 4].map(|skip| ring(n, skip)))
            .collect();
        let mut reassociated = 0;
        let conditionals_answers = vec![2.0 / 9.0, 2.0 / 9.0, 1.0, 0.25, 1.0 / 18.0, 17.0 / 18.0];
        for (source, expected) in std::iter::once((conditionals, Some(conditionals_answers)))
            .chain(rings.iter().map(|r| (r.as_str(), None)))
        {
            for indicators in [false, true] {
                let gp = if indicators {
                    crate::indicators::introduce(&ground(source))
                } else {
                    ground(source)
                };
                for strategy in [Kind::Closed, Kind::Families { search: false }] {
                    // The families layout lays out indicators, where atoms
                    // are read through them.
                    let (
                        crate::vtree::Layout {
                            vtree: tree, order, ..
                        },
                        _,
                    ) = match crate::vtree::build(
                        &gp,
                        &crate::ground::deps::Deps::of(&gp),
                        strategy,
                    ) {
                        Ok(layout) => layout,
                        Err(_) if strategy != Kind::Closed => continue,
                        Err(e) => panic!("{e}"),
                    };
                    let compiled = tree.context().run(|engine| {
                        compile(
                            &gp,
                            fact_variables(&gp, &order),
                            Formulation::Dual,
                            &TddBackend::new(engine, tree.clone()),
                            Projection::default(),
                            None,
                        )
                        .unwrap()
                    });
                    let (fresh, _) = tree
                        .context()
                        .run(|engine| {
                            query_probabilities(
                                &gp,
                                &TddBackend::new(engine, tree.clone()),
                                &compiled,
                            )
                        })
                        .unwrap();
                    let fresh = fresh.unwrap();
                    let mut state = tree.context().run(|engine| {
                        QueryEvaluation::new(&gp, &TddBackend::new(engine, tree.clone()), &compiled)
                    });
                    state.walk.defer_from = 1;
                    state.walk.defer_growth = 1;
                    let mut budget = 1;
                    let answers = loop {
                        let result = tree.context().run(|engine| {
                            let limit = LimitConfig::none().with_stop_rules(StopRules {
                                unconditional: Some(StopAt::WorkUnits(
                                    engine.limits().work_units() + budget,
                                )),
                                after_pairs: None,
                            });
                            let _guard = engine.limits().scope(limit);
                            state.run(&gp, &TddBackend::new(engine, tree.clone()), &compiled, None)
                        });
                        match result {
                            Ok((answers, _)) => break answers.unwrap(),
                            Err(CompileError::Tdd(OperationError::Stopped)) => budget *= 2,
                            Err(e) => panic!("{e}"),
                        }
                        assert!(
                            budget < 1 << 24,
                            "a growing budget must eventually complete"
                        );
                    };
                    assert_eq!(state.walk.completed_joins, gp.evidence.len() - 1);
                    reassociated += state.walk.reassociated;
                    assert_eq!(answers.len(), fresh.len());
                    for (i, (answer, fresh)) in answers.iter().zip(&fresh).enumerate() {
                        assert!(
                            (answer.probability - fresh.probability).abs() < 1e-12,
                            "{answer:?} {fresh:?} indicators {indicators} {strategy:?}"
                        );
                        if let Some(expected) = &expected {
                            assert!(
                                (answer.probability - expected[i]).abs() < 1e-12,
                                "{answer:?} indicators {indicators} {strategy:?}"
                            );
                        }
                    }
                }
            }
        }
        assert!(reassociated > 0, "no last join reassociated");
    }

    #[test]
    fn grouped_batches_given_up_keep_the_answers() {
        // Two naive Bayes models, a and b, each with half its features
        // observed: on a right-linear vtree each observation is at a node of
        // its own, one above the other, and the factors put off along the
        // spine make batches of several groups; where the two models meet,
        // one model's result is the node's other item beside the batch of
        // the other. Hand computed from the program's rounded weights, the
        // same for both models: P(v1 | e) is proportional to 0.5 times the
        // product of the observed p_i, P(v2 | e) to 0.5 times that of the
        // q_i, and an unobserved feature's probability is
        // P(v1 | e) p_j + P(v2 | e) q_j.
        let k = 60;
        let weight = |x: f64| format!("{x:.4}").parse::<f64>().unwrap();
        let p = |i: usize| weight(0.1 + 0.8 * (i as f64 / k as f64));
        let q = |i: usize| weight(0.9 - 0.8 * (i as f64 / k as f64));
        let mut bayes = String::new();
        for m in ["a", "b"] {
            bayes.push_str(&format!("0.5::cls({m},v1); 0.5::cls({m},v2).\n"));
            for i in 1..=k {
                bayes.push_str(&format!(
                    "{}::obs({m},{i}) :- cls({m},v1).\n{}::obs({m},{i}) :- cls({m},v2).\n",
                    p(i),
                    q(i)
                ));
                bayes.push_str(&if i <= k / 2 {
                    format!("evidence(obs({m},{i}), true).\n")
                } else {
                    format!("query(obs({m},{i})).\n")
                });
            }
        }
        let (v1, v2) = (1..=k / 2).fold((0.5, 0.5), |(a, b), i| (a * p(i), b * q(i)));
        let (v1, v2) = (v1 / (v1 + v2), v2 / (v1 + v2));
        let bayes_answers: Vec<f64> = (k / 2 + 1..=k)
            .map(|j| v1 * p(j) + v2 * q(j))
            .cycle()
            .take(k)
            .collect();
        let (mut grouped, mut unbatched, mut ended_steps) = (0, 0, 0);
        for indicators in [false, true] {
            let gp = if indicators {
                crate::indicators::introduce(&ground(&bayes))
            } else {
                ground(&bayes)
            };
            for strategy in [Kind::Closed, Kind::Families { search: false }] {
                let (
                    crate::vtree::Layout {
                        vtree: tree, order, ..
                    },
                    _,
                ) = match crate::vtree::build(&gp, &crate::ground::deps::Deps::of(&gp), strategy) {
                    Ok(layout) => layout,
                    Err(_) if strategy != Kind::Closed => continue,
                    Err(e) => panic!("{e}"),
                };
                let compiled = tree.context().run(|engine| {
                    compile(
                        &gp,
                        fact_variables(&gp, &order),
                        Formulation::Dual,
                        &TddBackend::new(engine, tree.clone()),
                        Projection::default(),
                        None,
                    )
                    .unwrap()
                });
                // Every batch given up at its first join, and then as the
                // walk caps them.
                for capped in [true, false] {
                    let mut state = tree.context().run(|engine| {
                        QueryEvaluation::new(&gp, &TddBackend::new(engine, tree.clone()), &compiled)
                    });
                    if capped {
                        state.walk.give_up_batches = true;
                    }
                    let (answers, _) = tree
                        .context()
                        .run(|engine| {
                            state.run(&gp, &TddBackend::new(engine, tree.clone()), &compiled, None)
                        })
                        .unwrap();
                    let answers = answers.unwrap();
                    assert_eq!(answers.len(), bayes_answers.len());
                    for (answer, expected) in answers.iter().zip(&bayes_answers) {
                        assert!(
                            (answer.probability - expected).abs() < 1e-9,
                            "{answer:?} {expected} indicators {indicators} {strategy:?} {capped}"
                        );
                    }
                    grouped += state.walk.grouped;
                    unbatched += state.walk.unbatched;
                    ended_steps += state.walk.ended_steps;
                }
            }
        }
        assert!(grouped > 0, "no batch of several groups");
        assert!(unbatched > 0, "no batch given up");
        assert!(ended_steps > 0, "no batch given up ended a node's step");
    }

    /// Data points x_j = m + e_j mod 2^n, a ripple-carry adder for each
    /// over a shared m and noise e_j of its own, each bit of each sum
    /// observed, as a program, the facts in the order m's bits most
    /// significant first, then each data point's noise, and P(m(i) | e) for
    /// each i. Hand computed: P(m = v | e) is proportional to P(m = v)
    /// times the product over j of P(e_j = x_j - v mod 2^n).
    fn data_points_on_a_mean(n: usize, xs: &[usize]) -> (GroundProgram, Vec<AtomId>, Vec<f64>) {
        let pm = |i: usize| 0.3 + 0.05 * i as f64;
        let pe = |j: usize, i: usize| 0.1 + 0.08 * ((7 * j + 3 * i) % 10) as f64;
        let mut source = String::new();
        for i in 0..n {
            source.push_str(&format!("{}::m({i}).\nquery(m({i})).\n", pm(i)));
        }
        for (j, &x) in xs.iter().enumerate() {
            for i in 0..n {
                source.push_str(&format!("{}::e({j},{i}).\n", pe(j, i)));
            }
            source.push_str(&format!(
                "s({j},0) :- m(0), \\+e({j},0).\ns({j},0) :- \\+m(0), e({j},0).\n\
                 c({j},1) :- m(0), e({j},0).\n"
            ));
            for i in 1..n {
                source.push_str(&format!(
                    "t({j},{i}) :- m({i}), \\+e({j},{i}).\nt({j},{i}) :- \\+m({i}), e({j},{i}).\n\
                     s({j},{i}) :- t({j},{i}), \\+c({j},{i}).\ns({j},{i}) :- \\+t({j},{i}), c({j},{i}).\n"
                ));
                if i + 1 < n {
                    let k = i + 1;
                    source.push_str(&format!(
                        "c({j},{k}) :- m({i}), e({j},{i}).\nc({j},{k}) :- m({i}), c({j},{i}).\n\
                         c({j},{k}) :- e({j},{i}), c({j},{i}).\n"
                    ));
                }
            }
            for i in 0..n {
                source.push_str(&format!("evidence(s({j},{i}), {}).\n", (x >> i) & 1 == 1));
            }
        }
        let weight = |v: usize, p: &dyn Fn(usize) -> f64| {
            (0..n)
                .map(|i| if (v >> i) & 1 == 1 { p(i) } else { 1.0 - p(i) })
                .product::<f64>()
        };
        let posterior: Vec<f64> = (0..1usize << n)
            .map(|v| {
                weight(v, &pm)
                    * xs.iter()
                        .enumerate()
                        .map(|(j, &x)| weight((x + (1 << n) - v) % (1 << n), &|i| pe(j, i)))
                        .product::<f64>()
            })
            .collect();
        let total: f64 = posterior.iter().sum();
        let expected: Vec<f64> = (0..n)
            .map(|i| {
                (0..1usize << n)
                    .filter(|v| (v >> i) & 1 == 1)
                    .map(|v| posterior[v])
                    .sum::<f64>()
                    / total
            })
            .collect();
        let gp = ground(&source);
        let name = |a: AtomId| gp.atoms[a as usize].to_string();
        let mut order: Vec<AtomId> = gp.facts().map(|(a, _)| a).collect();
        // m's bits most significant first, then each data point's noise.
        order.sort_by_key(|&a| {
            let atom = name(a);
            let at = |k: usize| {
                atom[2..atom.len() - 1]
                    .split(',')
                    .nth(k)
                    .unwrap()
                    .parse::<i64>()
                    .unwrap()
            };
            match atom.starts_with("m(") {
                true => (-1, -at(0)),
                false => (at(0), at(1)),
            }
        });
        (gp, order, expected)
    }

    #[test]
    fn a_nodes_own_factors_meet_what_decides_them_below() {
        // The data points of `data_points_on_a_mean`. On a right-linear vtree with m's bits on top, most
        // significant first, and each data point's noise below, the
        // observations of bit i sit at m(i)'s node, one per data point,
        // and the child's result, the bits below, decides m's lower bits.
        // Conjoined among themselves first, the observations of different
        // data points read those bits without it, and their product holds
        // every combination of the data points' carries.
        let n = 6;
        let (gp, order, expected) = data_points_on_a_mean(n, &[37, 41, 29, 52, 44]);
        let vars: Vec<VarId> = (1..=order.len() as u32).map(VarId).collect();
        let tree = std::sync::Arc::new(tididi::Vtree::linear_from_order(&vars).unwrap());
        tree.context().run(|engine| {
            let backend = TddBackend::new(engine, tree.clone());
            let compiled = compile(
                &gp,
                fact_variables(&gp, &order),
                Formulation::Dual,
                &backend,
                Projection::default(),
                None,
            )
            .unwrap();
            let mut state = QueryEvaluation::new(&gp, &backend, &compiled);
            let (answers, sizes) = state.run(&gp, &backend, &compiled, None).unwrap();
            let answers = answers.unwrap();
            assert_eq!(answers.len(), n);
            for (answer, expected) in answers.iter().zip(&expected) {
                assert!(
                    (answer.probability - expected).abs() < 1e-9,
                    "{answer:?} {expected}"
                );
            }
            // The own factors went into the child's result one at a time, and
            // no formula of the walk outgrew the evidence; conjoined among
            // themselves first, the largest is eighteen times the evidence.
            assert!(state.walk.unbatched > 0, "no node's own factors given up");
            assert!(
                sizes.largest <= sizes.evidence,
                "largest {} evidence {}",
                sizes.largest,
                sizes.evidence
            );
        });
    }

    #[test]
    fn a_batch_is_grouped_by_part_where_its_factors_read_two_parts() {
        // The data points of `data_points_on_a_mean` on two vtrees with m's
        // bits a chain on top, most significant first. Below it each data
        // point's noise a right-linear tree of its own, and the data points
        // a balanced tree: the observations of a bit, one per data point,
        // are put off at m's node of that bit, and a batch of them reads
        // the five data points' parts, each grouped. On the right-linear
        // vtree the chain holds every leaf but the last, and what a batch
        // reads off the chain is that leaf, one part: the batches stay as
        // they were.
        let n = 6;
        let xs = [37, 41, 29, 52, 44];
        let (gp, order, expected) = data_points_on_a_mean(n, &xs);
        let vars: Vec<VarId> = (1..=order.len() as u32).map(VarId).collect();
        let linear = tididi::Vtree::linear_from_order(&vars).unwrap();
        let mut points: Vec<tididi::Vtree> = vars[n..]
            .chunks(n)
            .map(|point| tididi::Vtree::linear_from_order(point).unwrap())
            .collect();
        while points.len() > 1 {
            points = points
                .chunks(2)
                .map(|pair| match pair {
                    [a, b] => tididi::Vtree::join(a, b).unwrap(),
                    [a] => a.clone(),
                    _ => unreachable!(),
                })
                .collect();
        }
        let parted = vars[..n]
            .iter()
            .rev()
            .fold(points.pop().unwrap(), |below, &m| {
                tididi::Vtree::join(&tididi::Vtree::leaf(m), &below).unwrap()
            });
        for (tree, grouped) in [(linear, false), (parted, true)] {
            let tree = std::sync::Arc::new(tree);
            tree.context().run(|engine| {
                let backend = TddBackend::new(engine, tree.clone());
                let compiled = compile(
                    &gp,
                    fact_variables(&gp, &order),
                    Formulation::Dual,
                    &backend,
                    Projection::default(),
                    None,
                )
                .unwrap();
                let mut state = QueryEvaluation::new(&gp, &backend, &compiled);
                let (answers, _) = state.run(&gp, &backend, &compiled, None).unwrap();
                let answers = answers.unwrap();
                assert_eq!(answers.len(), n);
                for (answer, expected) in answers.iter().zip(&expected) {
                    assert!(
                        (answer.probability - expected).abs() < 1e-9,
                        "{answer:?} {expected} grouped {grouped}"
                    );
                }
                assert_eq!(state.walk.by_part > 0, grouped, "{}", state.walk.by_part);
            });
        }
    }
}
