//! `⋀_r ¬body_r` over the rules of one head, built by factoring out the
//! literals the bodies share.

use rustc_hash::{FxHashMap, FxHashSet};

use tididi::Tdd;
use tididi::vtree::VarId;

use super::clauses::ClauseMemo;
use super::stats::Op;
use super::{CompileError, Formula, NegationSource, Reduce, TddBackend, VtreeScope};
use crate::ground::{AtomId, AtomKind, GLit, GroundProgram, GroundRule};

/// A branch of the factoring is built on its parent's scope projected to
/// the variables its bodies mention only when that keeps at most this many
/// tenths of the parent's variables; otherwise on the parent's scope, whose
/// few extra levels the branch's operations then pay. Projecting at every
/// branch makes most projections drop one variable, and the embeds back up
/// copy many times the formulas' pairs (on bn_pathfinder).
const NARROW_TENTHS: usize = 9;

/// The factoring's bookkeeping arrays, indexed by variable and by literal
/// and kept for the whole compilation. A network's table read through
/// indicators names every parent value in every row, so a set of rows
/// repeats each variable and literal once per row; counting them in place
/// replaces a sort of every occurrence per branch (`scope_over`) and three
/// hash maps per shared literal (`negated_terms`).
pub(super) struct Scratch {
    /// `var_mark[v] == stamp`: `v` is already among the variables collected.
    pub(super) var_mark: Vec<u32>,
    stamp: u32,
    /// The place of each variable marked with the stamp, as [`Scratch::ranked`]
    /// gave it.
    pub(super) var_rank: Vec<u32>,
    /// Occurrences per literal, `2 * atom + negative`; zero outside a count,
    /// which puts back to zero what it `touched`.
    lit_count: Vec<u32>,
    touched: Vec<GLit>,
    /// The hash of each vtree node, for [`scope_shape`](super::scope::scope_shape).
    pub(super) shape: Vec<u64>,
}

impl Scratch {
    pub(super) fn new(num_vars: usize, num_atoms: usize) -> Self {
        Scratch {
            var_mark: vec![0; num_vars],
            stamp: 0,
            var_rank: vec![0; num_vars],
            lit_count: vec![0; 2 * num_atoms],
            touched: Vec::new(),
            shape: Vec::new(),
        }
    }

    /// The variables of `vars`, ascending and distinct, each marked with a
    /// fresh stamp and given the place `rank` makes of its index among them
    /// and itself, read after in `var_rank` in place of a search per
    /// occurrence: a large loop reads each variable many times.
    pub(super) fn ranked(
        &mut self,
        vars: impl Iterator<Item = VarId>,
        rank: impl Fn(usize, VarId) -> u32,
    ) -> Vec<VarId> {
        let stamp = self.fresh_stamp();
        let mut distinct = Vec::new();
        for v in vars {
            let mark = &mut self.var_mark[v.0 as usize];
            if *mark != stamp {
                *mark = stamp;
                distinct.push(v);
            }
        }
        distinct.sort_unstable();
        for (i, &v) in distinct.iter().enumerate() {
            self.var_rank[v.0 as usize] = rank(i, v);
        }
        distinct
    }

    fn slot(l: GLit) -> usize {
        2 * l.atom() as usize + usize::from(!l.is_positive())
    }

    /// A stamp no variable is marked with.
    pub(super) fn fresh_stamp(&mut self) -> u32 {
        if self.stamp == u32::MAX {
            self.var_mark.fill(0);
            self.stamp = 0;
        }
        self.stamp += 1;
        self.stamp
    }

    /// Zero the counts taken since the last reset.
    fn reset_counts(&mut self) {
        for l in self.touched.drain(..) {
            self.lit_count[Self::slot(l)] = 0;
        }
    }
}

/// A group of fact literals of which at most one holds, and whether a body
/// of all their negations makes it exactly one ([`Factoring::at_most_one`]).
pub(super) type AtMostOne = (Vec<tididi::Literal>, bool);

/// A rule of pairs of the head ([`GroundRule`]) among its other rules'
/// bodies, `at` of them before it. Over facts alone it goes to
/// [`Factoring::at_most_one`] as the group it is, where its pairs were
/// bodies of their own that the compile hashed, keyed and grouped again.
#[derive(Clone, Copy)]
pub(super) struct Pairs<'g> {
    pub(super) at: usize,
    pub(super) rule: &'g GroundRule,
}

/// `bodies` with the bodies of the rules of `pairs` in their places.
fn with_pairs(bodies: Vec<Vec<GLit>>, pairs: &[Pairs<'_>]) -> Vec<Vec<GLit>> {
    let mut out =
        Vec::with_capacity(bodies.len() + pairs.iter().map(|p| p.rule.clauses()).sum::<usize>());
    let mut next = pairs.iter().peekable();
    for (i, body) in bodies.into_iter().enumerate() {
        while let Some(p) = next.next_if(|p| p.at == i) {
            out.extend(p.rule.bodies().map(|body| body.to_vec()));
        }
        out.push(body);
    }
    out.extend(next.flat_map(|p| p.rule.bodies().map(|body| body.to_vec())));
    out
}

/// The bodies and the rules of pairs, in the order of the rules: each with
/// its literals and whether it is a rule of pairs.
pub(super) fn in_order<'b>(
    bodies: &'b [Vec<GLit>],
    pairs: &'b [Pairs<'b>],
) -> impl Iterator<Item = (&'b [GLit], bool)> {
    let mut next = 0;
    (0..=bodies.len()).flat_map(move |i| {
        let start = next;
        while next < pairs.len() && pairs[next].at == i {
            next += 1;
        }
        pairs[start..next]
            .iter()
            .map(|p| (p.rule.body.as_slice(), true))
            .chain(bodies.get(i).map(|body| (body.as_slice(), false)))
    })
}

/// The variables of which a clause loop's formula holds at most one, by
/// group: those of each group of [`Factoring::at_most_one`], and each pair
/// whose clause `¬a ∨ ¬b` it holds, as it takes them in, for a variable in
/// no group yet. A body's negative literal of a variable in a group with
/// one of its positive literals holds wherever the formula and that literal
/// do, and the clause leaves it out: the conjunction is the same function.
/// A table row reads the value of each parent it names and the other
/// values' copies negated, and the link that states at most one copy of
/// each parent holds reads its rows so where another rule reads the
/// parent's copies ([`crate::indicators`]): it takes them as clauses of one
/// literal per parent where they were of one per value.
#[derive(Clone)]
pub(super) struct Exclusive {
    group_of: FxHashMap<VarId, usize>,
    next: usize,
    positive: Vec<usize>,
}

impl Exclusive {
    pub(super) fn new(groups: &[AtMostOne]) -> Self {
        let mut group_of = FxHashMap::default();
        for (g, (lits, _)) in groups.iter().enumerate() {
            group_of.extend(lits.iter().map(|l| (l.var, g)));
        }
        Exclusive {
            group_of,
            next: groups.len(),
            positive: Vec::new(),
        }
    }

    /// Leaves out of a body's fact literals the negative ones a positive
    /// one implies, and says how many.
    pub(super) fn leave_out(&mut self, lits: &mut Vec<tididi::Literal>) -> usize {
        if self.group_of.is_empty() {
            return 0;
        }
        self.positive.clear();
        self.positive.extend(
            lits.iter()
                .filter(|l| l.sign)
                .filter_map(|l| self.group_of.get(&l.var).copied()),
        );
        if self.positive.is_empty() {
            return 0;
        }
        let before = lits.len();
        lits.retain(|l| {
            l.sign
                || self
                    .group_of
                    .get(&l.var)
                    .is_none_or(|g| !self.positive.contains(g))
        });
        before - lits.len()
    }

    /// Takes in the body of fact literals whose clause the formula now
    /// holds, where it is a pair of positive literals in no group.
    pub(super) fn take_pair(&mut self, lits: &[tididi::Literal]) {
        if let [a, b] = *lits
            && a.sign
            && b.sign
            && a.var != b.var
            && !self.group_of.contains_key(&a.var)
            && !self.group_of.contains_key(&b.var)
        {
            self.group_of.insert(a.var, self.next);
            self.group_of.insert(b.var, self.next);
            self.next += 1;
        }
    }
}

/// What the watch makes of a formula: `Ok` where it lets the formula
/// through, `Err` with the formula, reduced in full, and its pairs where it
/// refuses it.
pub(super) type Checked<T> = Result<T, (Tdd, usize)>;

/// Builds `⋀_r ¬body_r` over a set of rule bodies by factoring out shared
/// literals: `⋀_r ¬body_r = ¬(L ∧ ¬⋀_{r∋L} ¬body_r\\L) ∧ ⋀_{r∌L} ¬body_r`,
/// with `L` the literal in the most bodies, recursively. The bodies of one
/// head usually overlap heavily: the CPT rows of a Bayesian network share
/// parent values (a row's body of `m` parents and a head of `k` values give
/// `k` rules per row over the same `m` literals), the rules of a relational
/// head share the literals that do not range over the rule's local
/// variables. Rule by rule, every body conjoins the large formulas of its
/// shared literals again; factored, a shared literal's formula is conjoined
/// once per branch, and the many small conjunctions deep in the recursion
/// are over what the rules do not share.
///
/// Deep in the recursion the bodies mention a few variables of a scope
/// holding thousands, and an operation costs its levels whatever the
/// formula depends on. So a branch whose bodies are over facts alone is
/// built on the scope projected to those facts and embedded up when it is
/// combined; a branch with a derived literal stays on its parent's scope,
/// since the literal's formula spans its whole cone.
pub(super) struct Factoring<'a> {
    pub(super) backend: &'a TddBackend<'a>,
    pub(super) gp: &'a GroundProgram,
    pub(super) var_of: &'a [Option<VarId>],
    /// Per variable, the link of the indicator it is the copy of,
    /// `AtomId::MAX` for none.
    pub(super) link_of: &'a [AtomId],
    /// Per variable, its group of siblings (`Encoding::siblings`),
    /// `u32::MAX` for none.
    pub(super) sibling_of: &'a [u32],
    /// Whether the head's loops of clauses hold the at-most-one of each
    /// sibling group their bodies read, which leaves a body's negated
    /// siblings out ([`Exclusive`]). The links admit at most one sibling
    /// of a group true, and they do not read the head's formula, so a
    /// formula that differs only where two are true gives the same answers.
    pub(super) seeds: bool,
    pub(super) head: AtomId,
    pub(super) scratch: &'a std::cell::RefCell<Scratch>,
    pub(super) memo: &'a std::cell::RefCell<ClauseMemo>,
    pub(super) watched: &'a dyn Fn(Tdd, AtomId) -> Result<Tdd, CompileError>,
    /// `watched` that hands back the formula the watch refused, reduced in
    /// full, with its pairs, for the [`ClauseMemo`] to keep for the next
    /// run ([`ClauseMemo::retake`]).
    pub(super) checked: &'a dyn Fn(Tdd) -> Result<Checked<Tdd>, CompileError>,
    /// The pairs `watched` lets through.
    pub(super) watch: Option<usize>,
    /// Counts that many operations of the given kind.
    pub(super) count: &'a dyn Fn(Op, usize),
    /// Negates, counted under the given source.
    pub(super) negate: &'a dyn Fn(NegationSource, Tdd) -> Result<Tdd, CompileError>,
    /// The source of the terms: [`NegationSource::LinkTerm`] for a link,
    /// [`NegationSource::Term`] otherwise.
    pub(super) term: NegationSource,
    /// The formula of a body literal as it stands in the interpretation,
    /// with its scope, which the given scope holds, read for the given
    /// number of bodies that hold the literal: the compile takes a formula
    /// at the read that brings its count of them to its last reader's
    /// bodies (`last_reads` in `compile.rs`). A fact's literal is built on
    /// the given scope; a derived literal's formula stays on its atom's
    /// own scope, and the conjunction that reads it moves it onto the
    /// given one ([`TddBackend::and_on`]), which leaves unbuilt the levels
    /// the move adds where none of its variables is under them. An embed
    /// first built every such level, and the conjunction then walked it:
    /// a formula of a decision circuit holds a node or two a level, and
    /// the scope it is read on often twice its own variables.
    pub(super) part: &'a mut dyn FnMut(GLit, u32, &VtreeScope) -> Result<Formula, CompileError>,
}

/// What the formula `f` would count once placed on `on`, which holds its
/// scope, where it counts `count` (pairs or nodes) on its own: each level
/// the placement adds holds at least one node and one pair.
pub(super) fn placed_count(count: usize, f: &Formula, on: &VtreeScope) -> usize {
    count + (on.len() - f.1.len())
}

impl Factoring<'_> {
    /// `f` on `on`, which holds its scope.
    pub(super) fn placed(&self, f: Formula, on: &VtreeScope) -> Result<Tdd, CompileError> {
        self.backend.embed(f.0, &f.1, on)
    }

    pub(super) fn fact_literal(&self, l: GLit) -> Option<tididi::Literal> {
        match self.gp.kinds[l.atom() as usize] {
            AtomKind::Fact(_) => {
                let v = self.var_of[l.atom() as usize].unwrap();
                Some(if l.is_positive() {
                    tididi::Literal::pos(v)
                } else {
                    tididi::Literal::neg(v)
                })
            }
            AtomKind::Derived => None,
        }
    }

    /// `start ∧ ⋀_r ¬body_r` for the given bodies and rules of pairs, on
    /// `scope`, which holds their variables and `start`'s; `start` is true
    /// when there is none.
    pub(super) fn negated_bodies(
        &mut self,
        mut bodies: Vec<Vec<GLit>>,
        mut pairs: Vec<Pairs<'_>>,
        scope: &VtreeScope,
        start: Option<Tdd>,
    ) -> Result<Tdd, CompileError> {
        // A body with complementary fact literals is false and contributes
        // ¬false = true. Checked once here: the factoring below only strips
        // literals from bodies, which cannot make a pair, rather than at
        // every level of the factoring, quadratically per body. Each body's
        // variables are marked with a stamp of its own and their signs kept
        // beside the marks, rather than sorting a copy of each body's
        // literals.
        // The rules of pairs keep their places among the bodies left; their
        // own pairs are of distinct positive literals.
        {
            let mut scratch = self.scratch.borrow_mut();
            let (mut i, mut dropped, mut next) = (0, 0, 0);
            bodies.retain(|body| {
                while next < pairs.len() && pairs[next].at == i {
                    pairs[next].at -= dropped;
                    next += 1;
                }
                i += 1;
                let stamp = scratch.fresh_stamp();
                let kept = body.iter().filter_map(|&l| self.fact_literal(l)).all(|f| {
                    let v = f.var.0 as usize;
                    if scratch.var_mark[v] == stamp {
                        return scratch.var_rank[v] == u32::from(f.sign);
                    }
                    scratch.var_mark[v] = stamp;
                    scratch.var_rank[v] = u32::from(f.sign);
                    true
                });
                dropped += usize::from(!kept);
                kept
            });
            for p in &mut pairs[next..] {
                p.at -= dropped;
            }
        }
        let derived = bodies
            .iter()
            .flatten()
            .any(|&l| self.fact_literal(l).is_none());
        // The factoring takes bodies, and a rule of pairs among derived
        // literals goes in as its pairs.
        if derived && !pairs.is_empty() {
            bodies = with_pairs(bodies, &pairs);
            pairs.clear();
        }
        match start {
            // Bodies with a derived literal are built on `scope` itself
            // (`scope_over`), where `start` lives.
            Some(start) if derived => self.negated_bodies_on(bodies, &[], scope, Some(start)),
            Some(start) => {
                let f = self.negated_bodies_of(bodies, &pairs, scope)?;
                let r = self.backend.and_on((start, scope.clone()), f, scope)?;
                (self.count)(Op::And, 1);
                Ok(r)
            }
            None => {
                let f = self.negated_bodies_of(bodies, &pairs, scope)?;
                self.placed(f, scope)
            }
        }
    }

    /// `⋀_r ¬body_r` for bodies without complementary fact literals, and
    /// rules of pairs among them where the bodies are of facts alone, on
    /// the scope over them, which `scope` holds.
    fn negated_bodies_of(
        &mut self,
        bodies: Vec<Vec<GLit>>,
        pairs: &[Pairs<'_>],
        scope: &VtreeScope,
    ) -> Result<Formula, CompileError> {
        let own = self.scope_over(&bodies, pairs, scope)?;
        let f = self.negated_bodies_on(bodies, pairs, &own, None)?;
        Ok((f, own))
    }

    /// `start ∧ ⋀_r ¬body_r` on `scope`, which is the scope over the bodies
    /// and the rules of pairs.
    fn negated_bodies_on(
        &mut self,
        bodies: Vec<Vec<GLit>>,
        pairs: &[Pairs<'_>],
        scope: &VtreeScope,
        start: Option<Tdd>,
    ) -> Result<Tdd, CompileError> {
        let terms = self.negated_terms(bodies, pairs, scope, start)?;
        self.conjoin_all(terms, scope)
    }

    /// `⋁_r body_r` for the given bodies, on `scope`, which is the scope
    /// over the bodies, as the negation of `negated_bodies_on`. The
    /// factored term `¬(L ∧ M ∧ ⋁_{r∋L} (body_r \\ L \\ M))` takes the
    /// disjunction, and taking it here rather than negating each `⋀_{r∋L}
    /// ¬(body_r \\ L \\ M)` inside the term is one negation per term
    /// instead of two.
    fn positive_bodies(
        &mut self,
        bodies: Vec<Vec<GLit>>,
        scope: &VtreeScope,
    ) -> Result<Tdd, CompileError> {
        let mu = self.negated_bodies_on(bodies, &[], scope, None)?;
        (self.negate)(NegationSource::Disjunction, mu)
    }

    /// The scope the bodies and the rules of pairs are built on: `parent`
    /// projected to the facts they mention when they mention nothing else,
    /// `parent` otherwise.
    fn scope_over(
        &self,
        bodies: &[Vec<GLit>],
        pairs: &[Pairs<'_>],
        parent: &VtreeScope,
    ) -> Result<VtreeScope, CompileError> {
        let Some(held) = parent.vars() else {
            return Ok(parent.clone());
        };
        let mut vars: Vec<VarId> = Vec::new();
        {
            let mut scratch = self.scratch.borrow_mut();
            let stamp = scratch.fresh_stamp();
            for body in bodies
                .iter()
                .map(Vec::as_slice)
                .chain(pairs.iter().map(|p| p.rule.body.as_slice()))
            {
                for &l in body {
                    match self.gp.kinds[l.atom() as usize] {
                        AtomKind::Fact(_) => {
                            let v = self.var_of[l.atom() as usize].unwrap();
                            let mark = &mut scratch.var_mark[v.0 as usize];
                            if *mark != stamp {
                                *mark = stamp;
                                vars.push(v);
                            }
                        }
                        AtomKind::Derived => return Ok(parent.clone()),
                    }
                }
            }
        }
        vars.sort_unstable();
        self.narrowed(parent, held, &vars)
    }

    /// `parent`, whose variables are `held`, projected to `vars`, a subset
    /// of them, or `parent` itself when the projection would keep more
    /// than [`NARROW_TENTHS`] tenths of them. Bodies over nothing (none,
    /// or all empty) make a constant, which lives anywhere.
    fn narrowed(
        &self,
        parent: &VtreeScope,
        held: &[VarId],
        vars: &[VarId],
    ) -> Result<VtreeScope, CompileError> {
        if vars.is_empty()
            || vars.len() * 10 > held.len() * NARROW_TENTHS
            || vars.len() == held.len()
        {
            return Ok(parent.clone());
        }
        // A loop the watch refused on this projection in the run before
        // is taken on the scope it was built on ([`ClauseMemo::retake`]).
        if let Some(scope) = self.memo.borrow().refused_scope(self.backend, vars) {
            return Ok(scope);
        }
        self.backend.subscope(parent, vars)
    }

    /// The conjuncts of `start ∧ ⋀_r ¬body_r` on `scope`: one `¬(L ∧
    /// ¬⋀_{r∋L} ¬(body_r \\ L))` per shared literal `L`, taken in turn from
    /// the bodies left, and the bodies sharing nothing conjoined one at a
    /// time into the conjunction of those terms and `start`; bodies of facts
    /// alone, with the rules of pairs, as one conjunct of clauses. A `start`
    /// no chain takes is a conjunct itself.
    pub(super) fn negated_terms(
        &mut self,
        mut bodies: Vec<Vec<GLit>>,
        pairs: &[Pairs<'_>],
        scope: &VtreeScope,
        mut start: Option<Tdd>,
    ) -> Result<Vec<Formula>, CompileError> {
        // An empty body is true and makes the conjunction false.
        if bodies.iter().any(|body| body.is_empty()) {
            return Ok(vec![(self.backend.zero(scope), scope.clone())]);
        }
        // Bodies of facts alone, a network's rows under indicators among
        // them, are conjoined one at a time as clauses and negate nothing,
        // where each factored term is two negations: of the disjunction of
        // the bodies sharing `L`, and of the term. The formulas are the
        // same (docs/design.md, "Clause tables").
        if bodies
            .iter()
            .flatten()
            .all(|&l| self.fact_literal(l).is_some())
        {
            return Ok(start
                .into_iter()
                .chain([self.clauses_of_facts(bodies, pairs, scope)?])
                .map(|f| (f, scope.clone()))
                .collect());
        }
        debug_assert!(
            pairs.is_empty(),
            "rules of pairs among derived literals go in as their pairs"
        );
        let mut terms = Vec::new();
        while !bodies.is_empty() {
            // The literal in the most bodies; ties go to derived literals (the
            // large formulas belong near the root, where they are conjoined
            // least often), then to the smaller literal for determinism.
            let shared = {
                let mut scratch = self.scratch.borrow_mut();
                let s = &mut *scratch;
                for body in &bodies {
                    for &l in body {
                        let c = &mut s.lit_count[Scratch::slot(l)];
                        if *c == 0 {
                            s.touched.push(l);
                        }
                        *c += 1;
                    }
                }
                let shared = s
                    .touched
                    .iter()
                    .map(|&l| (l, s.lit_count[Scratch::slot(l)]))
                    .filter(|&(_, c)| c > 1)
                    .max_by_key(|&(l, c)| (c, self.fact_literal(l).is_none(), std::cmp::Reverse(l)))
                    .map(|(l, _)| l);
                s.reset_counts();
                shared
            };
            let Some(shared) = shared else {
                // The bodies left go into the terms' conjunction rather than
                // beside it: each term narrows what the loop's accumulator
                // holds.
                let start = if terms.is_empty() {
                    start.take()
                } else {
                    terms.extend(start.take().map(|f| (f, scope.clone())));
                    let t = self.conjoin_all(std::mem::take(&mut terms), scope)?;
                    if self.backend.is_zero(&t) {
                        return Ok(vec![(t, scope.clone())]);
                    }
                    Some(t)
                };
                terms.push((self.one_by_one(bodies, &[], scope, start)?, scope.clone()));
                break;
            };
            let (with, without): (Vec<Vec<GLit>>, Vec<Vec<GLit>>) =
                bodies.into_iter().partition(|body| body.contains(&shared));
            bodies = without;
            let with_bodies = with.len() as u32;
            // The term is built on the scope of the bodies sharing `L`,
            // `L` included, and embedded up.
            let own = self.scope_over(&with, &[], scope)?;
            // Every literal all of those bodies contain goes with `L`: the
            // term is `¬(L ∧ M ∧ ⋁_r (body_r \ L \ M))` with `M` the common
            // literals, which is the same function as the chain of one term
            // per literal, without the chain's terms. A row of a network's
            // table read through indicators names a parent's value by one copy
            // true and the copies of its other values false, so the rows on
            // one value share as many literals as the parent has values.
            let common: Vec<GLit> = {
                let mut scratch = self.scratch.borrow_mut();
                let s = &mut *scratch;
                for &l in with[0].iter().filter(|&&l| l != shared) {
                    let c = &mut s.lit_count[Scratch::slot(l)];
                    if *c == 0 {
                        s.touched.push(l);
                    }
                    *c = 1;
                }
                for body in &with[1..] {
                    for &l in body {
                        let c = &mut s.lit_count[Scratch::slot(l)];
                        if *c > 0 {
                            *c += 1;
                        }
                    }
                }
                let common = with[0]
                    .iter()
                    .copied()
                    .filter(|&l| {
                        l != shared && s.lit_count[Scratch::slot(l)] as usize == with.len()
                    })
                    .collect();
                s.reset_counts();
                common
            };
            let stripped = || std::iter::once(shared).chain(common.iter().copied());
            let with: Vec<Vec<GLit>> = {
                let mut scratch = self.scratch.borrow_mut();
                let s = &mut *scratch;
                for l in stripped() {
                    let c = &mut s.lit_count[Scratch::slot(l)];
                    if *c == 0 {
                        s.touched.push(l);
                        *c = 1;
                    }
                }
                let with = with
                    .into_iter()
                    .map(|body| {
                        body.into_iter()
                            .filter(|&l| s.lit_count[Scratch::slot(l)] == 0)
                            .collect()
                    })
                    .collect();
                s.reset_counts();
                with
            };
            // The stripped bodies' scope is `own` less the stripped fact
            // literals' variables, which no body left mentions (a body
            // with a literal and its complement was dropped on entry);
            // with a derived literal among them it is `own`, as
            // `scope_over` would say, without its scan of the bodies.
            let inner = match own.vars() {
                Some(held)
                    if with
                        .iter()
                        .flatten()
                        .all(|&l| self.fact_literal(l).is_some()) =>
                {
                    let mut gone: Vec<VarId> = stripped()
                        .filter_map(|l| self.fact_literal(l))
                        .map(|f| f.var)
                        .collect();
                    gone.sort_unstable();
                    let rest: Vec<VarId> = held
                        .iter()
                        .copied()
                        .filter(|v| gone.binary_search(v).is_err())
                        .collect();
                    self.narrowed(&own, held, &rest)?
                }
                _ => own.clone(),
            };
            // ⋁_{r∋L} (body_r \ L \ M), on `inner` until an operation
            // needs it on `own`.
            let mut positive: Formula = (self.positive_bodies(with, &inner)?, inner);
            // ¬(L ∧ M ∧ positive), the facts first as clauses of one literal
            // (which rebuild only the levels on the path from the leaf to
            // the root, where a literal built as a diagram and conjoined
            // costs every level of the scope), then the derived literals'
            // formulas.
            let mut derived = Vec::new();
            let mut dead = false;
            for l in std::iter::once(shared).chain(common) {
                match self.fact_literal(l) {
                    Some(f) => {
                        let p = self.placed(positive, &own)?;
                        positive = (self.backend.and_clause(&own, p, &[f])?, own.clone());
                        (self.count)(Op::And, 1);
                    }
                    None => derived.push(l),
                }
            }
            for l in derived {
                // For each body of the group, which all hold it.
                let lf = (self.part)(l, with_bodies, &own)?;
                if self.backend.is_zero(&lf.0) {
                    dead = true; // the literal is false: those bodies are false
                    break;
                }
                positive = (self.backend.and_on(lf, positive, &own)?, own.clone());
                (self.count)(Op::And, 1);
            }
            if dead {
                continue;
            }
            let both = (self.watched)(self.placed(positive, &own)?, self.head)?;
            let term = (self.negate)(self.term, both)?;
            terms.push((term, own));
        }
        terms.extend(start.map(|f| (f, scope.clone())));
        Ok(terms)
    }

    /// The conjunction of the terms, on `scope`, last term first: the
    /// terms of the least shared literals, which are the smallest, go in
    /// before the term of the most shared one. Where the terms are over the
    /// same derived literals, the product of two small terms can be larger
    /// than a term against the accumulator (bn_link under closed).
    fn conjoin_all(
        &mut self,
        terms: Vec<Formula>,
        scope: &VtreeScope,
    ) -> Result<Tdd, CompileError> {
        // One term is the branch's result as its negation left it, pruned but
        // not contracted; the operations above it run on what it carries.
        // Under closed on bn_link, whose terms are large, leaving it so cost
        // more in the conjunctions above than the minimization.
        if terms.len() == 1 {
            let (mut f, own) = terms.into_iter().next().unwrap();
            self.backend.minimize(&mut f, Reduce::Between)?;
            (self.count)(Op::Minimize, 1);
            return self.placed((f, own), scope);
        }
        let mut acc: Option<Formula> = None;
        for f in terms.into_iter().rev() {
            acc = Some(match acc {
                None => f,
                Some(a) => {
                    let mut r = self.backend.and_on(f, a, scope)?;
                    (self.count)(Op::And, 1);
                    self.backend.minimize(&mut r, Reduce::Between)?;
                    (self.count)(Op::Minimize, 1);
                    let r = (self.watched)(r, self.head)?;
                    if self.backend.is_zero(&r) {
                        return Ok(r);
                    }
                    (r, scope.clone())
                }
            });
        }
        match acc {
            Some(a) => self.placed(a, scope),
            None => Ok(self.backend.one(scope)),
        }
    }

    /// The two variables, the lesser first, of a body of two positive fact
    /// literals of distinct atoms.
    fn pair_of(&self, body: &[GLit]) -> Option<(VarId, VarId)> {
        match *body {
            [a, b] if a.is_positive() && b.is_positive() && a.atom() != b.atom() => {
                let (x, y) = (self.fact_literal(a)?.var, self.fact_literal(b)?.var);
                Some((x.min(y), x.max(y)))
            }
            _ => None,
        }
    }

    /// The groups of three or more fact variables of which every pair is a
    /// body of two positive literals, or the rule of pairs of `pairs`, each
    /// with whether a body of all their negations makes it exactly one, and
    /// the bodies the groups leave.
    ///
    /// A clause table states that at most one of a reader's copies of a
    /// table holds by a clause per pair ([`crate::indicators`]), most of
    /// bn_pathfinder's clauses. Conjoined one at a time they split the same
    /// elements again and again (see [`CLAUSES_PER_CONTRACTION`]), where
    /// [`one_of`](Self::one_of) builds the same formula from each group's
    /// halves.
    ///
    /// The groups come in the order their first pairs do, each led by the
    /// lesser variable of its first pair, the variables it is paired with
    /// after it in the order of their pairs. A rule of pairs is its group
    /// so where no other pair or rule of pairs reads its variables; one that
    /// shares them goes in as its pairs, which come out as they would.
    ///
    /// [`CLAUSES_PER_CONTRACTION`]: super::clauses::CLAUSES_PER_CONTRACTION
    pub(super) fn at_most_one(
        &self,
        bodies: Vec<Vec<GLit>>,
        pairs: &[Pairs<'_>],
    ) -> (Vec<AtMostOne>, Vec<Vec<GLit>>) {
        let var = |l: &GLit| {
            self.fact_literal(*l)
                .expect("a rule of pairs over facts alone")
                .var
        };
        if !pairs.is_empty() {
            let mut scratch = self.scratch.borrow_mut();
            let stamp = scratch.fresh_stamp();
            let mut apart = pairs.iter().flat_map(|p| &p.rule.body).all(|l| {
                let v = var(l).0 as usize;
                std::mem::replace(&mut scratch.var_mark[v], stamp) != stamp
            });
            apart = apart
                && bodies
                    .iter()
                    .filter_map(|body| self.pair_of(body))
                    .all(|(x, y)| {
                        scratch.var_mark[x.0 as usize] != stamp
                            && scratch.var_mark[y.0 as usize] != stamp
                    });
            drop(scratch);
            if !apart {
                return self.at_most_one(with_pairs(bodies, pairs), &[]);
            }
        } else if bodies
            .iter()
            .filter(|body| self.pair_of(body).is_some())
            .take(3)
            .count()
            < 3
        {
            return (Vec::new(), bodies);
        }
        // The groups of the bodies' pairs, each at the body that first reads
        // the variable that leads it.
        let mut found: Vec<(usize, Vec<VarId>)> = Vec::new();
        if bodies
            .iter()
            .filter(|body| self.pair_of(body).is_some())
            .take(3)
            .count()
            == 3
        {
            let mut pairs: FxHashSet<(VarId, VarId)> = FxHashSet::default();
            let mut neighbors: FxHashMap<VarId, Vec<VarId>> = FxHashMap::default();
            let mut seen: Vec<(VarId, usize)> = Vec::new();
            for (i, body) in bodies.iter().enumerate() {
                if let Some((x, y)) = self.pair_of(body)
                    && pairs.insert((x, y))
                {
                    for (u, v) in [(x, y), (y, x)] {
                        let list = neighbors.entry(u).or_insert_with(|| {
                            seen.push((u, i));
                            Vec::new()
                        });
                        list.push(v);
                    }
                }
            }
            let mut grouped: FxHashSet<VarId> = FxHashSet::default();
            for &(u, at) in &seen {
                if grouped.contains(&u) {
                    continue;
                }
                let group: Vec<VarId> = std::iter::once(u)
                    .chain(
                        neighbors[&u]
                            .iter()
                            .copied()
                            .filter(|v| !grouped.contains(v)),
                    )
                    .collect();
                let whole = group.len() >= 3
                    && group.iter().enumerate().all(|(i, &x)| {
                        group[i + 1..]
                            .iter()
                            .all(|&y| pairs.contains(&(x.min(y), x.max(y))))
                    });
                if whole {
                    grouped.extend(group.iter().copied());
                    found.push((at, group));
                }
            }
        }
        let mut groups: Vec<Vec<VarId>> = Vec::with_capacity(found.len() + pairs.len());
        let mut found = found.into_iter().peekable();
        for p in pairs {
            groups.extend(std::iter::from_fn(|| {
                found.next_if(|&(at, _)| at < p.at).map(|(_, group)| group)
            }));
            let mut group: Vec<VarId> = p.rule.body.iter().map(var).collect();
            if group[1] < group[0] {
                group.swap(0, 1);
            }
            groups.push(group);
        }
        groups.extend(found.map(|(_, group)| group));
        if groups.is_empty() {
            return (Vec::new(), bodies);
        }
        let group_of: FxHashMap<VarId, usize> = groups
            .iter()
            .enumerate()
            .flat_map(|(g, group)| group.iter().map(move |&x| (x, g)))
            .collect();
        let mut exactly = vec![false; groups.len()];
        let mut rest = Vec::with_capacity(bodies.len());
        for body in bodies {
            if let Some((x, y)) = self.pair_of(&body)
                && group_of
                    .get(&x)
                    .is_some_and(|g| group_of.get(&y) == Some(g))
            {
                continue;
            }
            // The body of all a group's negations: the clause that one holds.
            let negated: Option<Vec<VarId>> = body
                .iter()
                .map(|&l| {
                    (!l.is_positive())
                        .then(|| self.fact_literal(l))
                        .flatten()
                        .map(|f| f.var)
                })
                .collect();
            if let Some(mut vars) = negated
                && let Some(&g) = vars.first().and_then(|x| group_of.get(x))
                && vars.len() == groups[g].len()
            {
                vars.sort_unstable();
                vars.dedup();
                let mut group = groups[g].clone();
                group.sort_unstable();
                if vars == group {
                    exactly[g] = true;
                    continue;
                }
            }
            rest.push(body);
        }
        let groups = groups
            .into_iter()
            .zip(exactly)
            .map(|(group, exactly)| {
                (
                    group.into_iter().map(tididi::Literal::pos).collect(),
                    exactly,
                )
            })
            .collect();
        (groups, rest)
    }

    /// Exactly one of `lits`, or at most one ([`TddBackend::one_of`]), on
    /// the scope of `lits` alone, which `scope` holds: a literal's diagram
    /// has a node at every level of its scope.
    pub(super) fn one_of(
        &self,
        scope: &VtreeScope,
        lits: &mut [tididi::Literal],
        exactly: bool,
    ) -> Result<Formula, CompileError> {
        lits.sort_unstable_by_key(|l| l.var);
        let vars: Vec<VarId> = lits.iter().map(|l| l.var).collect();
        let own = self.backend.subscope(scope, &vars)?;
        let f = self.backend.one_of(&own, lits, exactly)?;
        Ok((f, own))
    }
}
