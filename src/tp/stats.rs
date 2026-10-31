//! What a compilation counts: operations, negations by source, the
//! rotation searches of a dynamic vtree, projection and the clause memo.

use std::borrow::Cow;

use tididi::Tdd;

use super::{CompileError, TddBackend};

/// Statistics of a compilation run.
#[derive(Clone, Debug, Default)]
pub struct CompileStats {
    /// Strongly connected components of the derived atoms' dependencies,
    /// each compiled as one.
    pub components: usize,
    /// Components compiled by a fixpoint: those that read themselves.
    pub cyclic_components: usize,
    /// Total number of operator applications to single atoms.
    pub atom_updates: usize,
    /// Iterations over cyclic components, summed.
    pub iterations: usize,
    /// Most iterations over one cyclic component.
    pub max_iterations: usize,
    /// Largest formula (in pairs) seen for any atom, intermediate iterates
    /// included.
    pub max_pairs: usize,
    /// Largest final formula (in pairs) of any atom the compile kept to the
    /// end: the evidence, and the queries of a program with evidence.
    pub final_max_pairs: usize,
    /// Query atoms the compile counted where it dropped their formulas,
    /// those of a program without evidence (`ForQueries::counted`).
    pub queries_counted: usize,
    /// Query atoms answered by another query atom's formula under other
    /// probabilities, whose own formulas the compile did not build
    /// (`ForQueries::templates`).
    pub templated: usize,
    /// Readers' links of clause tables tied to another reader's copies,
    /// whose own formulas the compile did not build (`ForQueries::ties`).
    pub tied: usize,
    /// Seconds the compile took, summed over its runs.
    pub seconds: f64,
    /// Number of operations of each kind: conjunctions, disjunctions,
    /// minimizations, equivalence tests, negations.
    pub op_counts: [usize; 5],
    /// Formulas their last reader took rather than copied, in a run
    /// without a watch (`last_reads` in `compile.rs`).
    pub taken: usize,
    /// The negations of `op_counts[4]` by source.
    pub negations: Negations,
    /// Work units the compile charged, its searches included.
    pub work_units: u64,
    /// Convergence tests at the end of a cyclic atom update, one per update.
    /// `op_counts[3]` counts the ones the pair count did not answer on its
    /// own, so the difference is what the size test saved.
    pub convergence_tests: usize,
    /// What projection did to this program's components. `None` with
    /// projection off.
    pub projection: Option<ProjectionStats>,
    /// What the rotation search of a dynamic vtree did. `None` on a static
    /// vtree.
    pub search: Option<SearchStats>,
    /// What the memo of clause loops over facts did.
    pub clause_memo: ClauseMemoStats,
}

/// The operations [`CompileStats::op_counts`] counts, in its order.
#[derive(Clone, Copy)]
pub(super) enum Op {
    And,
    Or,
    Minimize,
    Equivalent,
    Negate,
}

/// The rotation searches of a dynamic vtree (`Compilation::dynamic`),
/// summed.
#[derive(Clone, Copy, Debug, Default)]
pub struct SearchStats {
    /// Searches run.
    pub searches: usize,
    /// Searches that shrank their formulas by less than a tenth.
    pub weak: usize,
    /// Moves probed, abandoned ones included.
    pub probes: usize,
    /// Moves kept.
    pub accepts: usize,
    /// Live pairs of the formulas searched over before each search, summed
    /// over the searches.
    pub pairs_before: usize,
    /// Live pairs of those formulas after each search, summed.
    pub pairs_after: usize,
    /// Work units the searches' probes charged, their restarts included.
    pub work_units: u64,
    /// Restarts begun after the searches' descents.
    pub restarts: usize,
    /// Restarts whose result was kept.
    pub restarts_kept: usize,
}

/// What a negation was for ([`Negations`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NegationSource {
    /// `λ_b = ¬μ_b` of a derived atom read positively, under the dual.
    Lambda,
    /// A term `¬(L ∧ M ∧ …)` of `⋀_r ¬body_r`, factored or of one body,
    /// for an atom other than a link.
    Term,
    /// The same for an indicator's link ([`crate::indicators`]).
    LinkTerm,
    /// The disjunction of the bodies inside a factored term, or of a
    /// body's two halves on the or-route (`Factoring::or_route`), taken as
    /// the negation of their conjoined negations.
    Disjunction,
    /// Of a derived atom stored as `λ_b` read negatively: under the
    /// primal, or an atom of one rule under the dual (`Compiled::positive`).
    /// The primal's also: of `⋀_r ¬body_r` for an atom without a formula,
    /// and for one with a formula `λ_a` of `λ_a` and of
    /// `¬λ_a ∧ ⋀_r ¬body_r`.
    Primal,
    /// The well-founded pair fixpoint's, and the dual's `μ` of its result.
    WellFounded,
    /// The query phase's: an observation of the polarity its atom's formula
    /// is not stored in, with indicators mostly their links observed true.
    Evidence,
    /// The query phase's: a query formula of the polarity its atom's
    /// formula is not stored in.
    Query,
}

impl NegationSource {
    /// Every source, in the order [`Negations`] counts them.
    pub const ALL: [NegationSource; 8] = [
        NegationSource::Lambda,
        NegationSource::Term,
        NegationSource::LinkTerm,
        NegationSource::Disjunction,
        NegationSource::Primal,
        NegationSource::WellFounded,
        NegationSource::Evidence,
        NegationSource::Query,
    ];

    /// The source's name, as `--stats` prints it.
    pub fn name(self) -> &'static str {
        match self {
            NegationSource::Lambda => "lambda",
            NegationSource::Term => "term",
            NegationSource::LinkTerm => "link term",
            NegationSource::Disjunction => "disjunction",
            NegationSource::Primal => "primal",
            NegationSource::WellFounded => "well-founded",
            NegationSource::Evidence => "evidence",
            NegationSource::Query => "query",
        }
    }
}

/// Negations of one source: calls, and the operands' pairs and the work
/// units the negations charged, summed. Machine independent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NegationCount {
    /// Negations taken.
    pub calls: usize,
    /// Pairs of their operands, summed.
    pub pairs: usize,
    /// Work units they charged, summed.
    pub work_units: u64,
}

/// Negations by [`NegationSource`]. Every negation of the compile and the
/// query phase goes through `Negations::negate`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Negations([NegationCount; NegationSource::ALL.len()]);

impl Negations {
    /// The negations of `source`.
    pub fn of(&self, source: NegationSource) -> NegationCount {
        self.0[source as usize]
    }

    /// The negations of every source, summed.
    pub fn total(&self) -> NegationCount {
        self.0
            .iter()
            .fold(NegationCount::default(), |a, c| NegationCount {
                calls: a.calls + c.calls,
                pairs: a.pairs + c.pairs,
                work_units: a.work_units + c.work_units,
            })
    }

    /// `¬f`, counted under `source`.
    pub(crate) fn negate<'f>(
        &mut self,
        backend: &TddBackend,
        source: NegationSource,
        f: impl Into<Cow<'f, Tdd>>,
    ) -> Result<Tdd, CompileError> {
        let f = f.into();
        let pairs = backend.size(&f);
        let work = backend.work_units();
        let r = backend.negate(f)?;
        let c = &mut self.0[source as usize];
        c.calls += 1;
        c.pairs += pairs;
        c.work_units += backend.work_units() - work;
        Ok(r)
    }
}

/// How far projection got: the components it moved off the global vtree, the
/// vtrees it built for them, and the largest cone it kept. A run where
/// `projected` is zero paid the cone walk and changed nothing, which is what
/// happens when every component depends on most of the facts.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProjectionStats {
    /// Probabilistic facts in the program: the global vtree's leaves.
    pub facts: u32,
    /// Components given a vtree smaller than the global one.
    pub projected: usize,
    /// Projected vtrees built, shared by cone (`ScopeTable`): one per cone
    /// while a formula on it lives, built again for a component that starts
    /// once its vtree has gone.
    pub scopes: usize,
    /// Facts in the largest cone that was projected. 0 when none was.
    pub largest_kept: u32,
    /// Components whose formulas, once compiled, depended on fewer
    /// variables than their scope held, and were moved onto the scope of
    /// those.
    pub narrowed: usize,
}

/// What the `ClauseMemo` and the clause loops did.
#[derive(Clone, Copy, Debug, Default)]
pub struct ClauseMemoStats {
    /// Loops looked up.
    pub lookups: usize,
    /// Loops a lookup found and copied.
    pub copies: usize,
    /// Kept loops a lookup passed over, their scope's shape not its own
    /// (`scope_shape`).
    pub passed: usize,
    /// Loops kept.
    pub kept: usize,
    /// Pairs of the loops kept, summed.
    pub pairs: usize,
    /// Negative literals the loops left out of bodies, each implied by a
    /// positive one where the loop's formula holds
    /// (`Factoring::one_by_one`).
    pub left_out: usize,
    /// Loops that held the at-most-one of the sibling groups their bodies
    /// read (`Factoring::seeds`).
    pub seeded: usize,
    /// Loops of bodies of facts alone built level by level
    /// (`TddBackend::none_of`).
    pub built: usize,
    /// Loops whose level-by-level builder gave up for the clause loop.
    pub fell_back: usize,
    /// Of the loops built, the links of tables built from their models
    /// (`TddBackend::of_models`).
    pub from_models: usize,
    /// Loops built level by level that the watch refused and the compile
    /// kept (`clauses::Refused`).
    pub refused: usize,
    /// Refused loops the next run took rather than build them again.
    pub retaken: usize,
    /// Bodies conjoined by the or-route (`Factoring::or_route`).
    pub or_routes: usize,
    /// Or-route bodies whose product was given up first.
    pub given_up: usize,
}
