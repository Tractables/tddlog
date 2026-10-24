//! The compilation: scopes per component, the fixpoint over the components
//! in dependency order, and the compiled program it leaves.

use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;

use tididi::Tdd;
use tididi::vtree::VarId;

use super::clauses::ClauseMemo;
use super::factoring::{Checked, Factoring, Pairs, Scratch};
use super::stats::Op;
use super::template::{Template, templates};
use super::tie::{Tie, ties};
use super::{
    CompileError, CompileStats, Formula, Formulation, NegationSource, Negations, Projection,
    ProjectionStats, Reduce, ScopeTable, SearchStats, TddBackend, VtreeScope, aft, cone,
};
use crate::eval::Probabilities;
use crate::ground::deps::Deps;
use crate::ground::{AtomId, AtomKind, GLit, GroundProgram, RulesByHead, deps};

/// The compiled program.
pub struct Compiled {
    /// Variable of each fact atom.
    pub(crate) var_of: Vec<Option<VarId>>,
    /// Formula of each derived atom (`None` for facts, whose formula is the
    /// literal itself), in the polarity `positive` says: the atom's formula
    /// under the primal and for the atoms of one rule (`conjoined`), its
    /// negation otherwise under the dual. Nothing
    /// is negated at the end of a compilation; a query needs only a model
    /// count, and the count of the negation is one minus the count.
    pub formulas: Formulas,
    /// Per atom, whether its stored formula is its formula (`true`) or its
    /// negation (`false`).
    pub(crate) positive: Vec<bool>,
    /// The scope each component's formulas live on. Without projection every
    /// entry is the one global scope.
    pub(crate) scopes: Vec<VtreeScope>,
    /// Component of each derived atom, `u32::MAX` for a fact.
    pub(crate) comp_of: Vec<u32>,
    /// What the compilation projected to, which the query phase follows: with
    /// projection off it keeps every formula on the global vtree.
    pub(crate) projection: Projection,
    /// The queries the compile counted, and the atoms whose formulas it
    /// did not build.
    pub(crate) for_queries: ForQueries,
    pub stats: CompileStats,
}

/// A formula or none for each atom, with an entry only for the atoms
/// given a formula. Most atoms are facts, which have none, and a `Tdd` is
/// 224 bytes: an entry for each atom would write most of a small
/// program's memory before its compile begins.
#[derive(Default)]
pub struct Formulas {
    /// Where each atom's entry is in `held`, plus one; 0 for an atom given
    /// none yet.
    at: Vec<u32>,
    /// The entries, in the order the atoms were first given one.
    held: Vec<Option<Tdd>>,
    /// What an atom without an entry reads.
    none: Option<Tdd>,
}

impl Formulas {
    /// None for each of `n` atoms, with room for `room` entries before
    /// they grow.
    fn new(n: usize, room: usize) -> Self {
        Formulas {
            at: vec![0; n],
            held: Vec::with_capacity(room),
            none: None,
        }
    }

    /// Each atom's formula or none, in the atoms' order.
    pub fn iter(&self) -> impl Iterator<Item = &Option<Tdd>> {
        self.at.iter().map(|&i| {
            if i == 0 {
                &self.none
            } else {
                &self.held[i as usize - 1]
            }
        })
    }

    /// The atoms' formulas, in the atoms' order.
    fn formulas_mut(&mut self) -> Vec<&mut Tdd> {
        let mut held: Vec<Option<&mut Tdd>> = self.held.iter_mut().map(Option::as_mut).collect();
        self.at
            .iter()
            .filter(|&&i| i != 0)
            .filter_map(|&i| held[i as usize - 1].take())
            .collect()
    }

    /// Take `a`'s formula, leaving none, without giving `a` an entry.
    fn take(&mut self, a: usize) -> Option<Tdd> {
        match self.at[a] {
            0 => None,
            i => self.held[i as usize - 1].take(),
        }
    }

    /// Drop every formula, keeping the entries.
    fn clear(&mut self) {
        self.held.iter_mut().for_each(|f| *f = None);
    }
}

impl std::ops::Index<usize> for Formulas {
    type Output = Option<Tdd>;

    fn index(&self, a: usize) -> &Option<Tdd> {
        match self.at[a] {
            0 => &self.none,
            i => &self.held[i as usize - 1],
        }
    }
}

/// A write gives the atom an entry if it has none.
impl std::ops::IndexMut<usize> for Formulas {
    fn index_mut(&mut self, a: usize) -> &mut Option<Tdd> {
        if self.at[a] == 0 {
            self.held.push(None);
            self.at[a] = self.held.len() as u32;
        }
        &mut self.held[self.at[a] as usize - 1]
    }
}

impl Compiled {
    /// The scope the formula of derived atom `a` lives on.
    ///
    /// # Panics
    ///
    /// Panics if `a` is a fact, which has no stored formula and so no scope.
    pub(crate) fn scope_of(&self, a: AtomId) -> &VtreeScope {
        &self.scopes[self.comp_of[a as usize] as usize]
    }

    /// Whether the stored formula of derived atom `a` is its formula
    /// (`true`) or its negation (`false`).
    pub(crate) fn stored_positive(&self, a: AtomId) -> bool {
        self.positive[a as usize]
    }

    /// A formula for the literal `a` or `\+a`, on `scope`: the stored formula
    /// when its polarity matches, its negation otherwise,
    /// counted in `negations` under `source`; a literal for a fact. The
    /// stored formula is embedded from the atom's own scope, which `scope`
    /// must contain.
    pub(crate) fn literal_formula(
        &self,
        backend: &TddBackend,
        scope: &VtreeScope,
        a: AtomId,
        positive: bool,
        (negations, source): (&mut Negations, NegationSource),
    ) -> Result<Tdd, CompileError> {
        match &self.formulas[a as usize] {
            Some(f) => {
                // Negate on the atom's own scope, which is the smaller one,
                // and embed the result: a function that does not depend on
                // the free variables has a complement that does not either.
                let own = self.scope_of(a);
                if self.stored_positive(a) == positive {
                    backend.embed_ref(f, own, scope)
                } else {
                    backend.embed(
                        negations.negate(backend, source, backend.copy(f)?)?,
                        own,
                        scope,
                    )
                }
            }
            None => {
                let v = self.var_of[a as usize].ok_or(CompileError::UnmappedFact(a))?;
                backend.literal(
                    scope,
                    if positive {
                        tididi::Literal::pos(v)
                    } else {
                        tididi::Literal::neg(v)
                    },
                )
            }
        }
    }
}

/// Assign vtree variables to the facts: `var_of[a]` for each fact atom.
pub fn fact_variables(gp: &GroundProgram, order: &[AtomId]) -> Vec<Option<VarId>> {
    let mut var_of = vec![None; gp.num_atoms()];
    for (i, &a) in order.iter().enumerate() {
        debug_assert!(gp.is_fact(a));
        var_of[a as usize] = Some(VarId(i as u32 + 1));
    }
    var_of
}

/// Whether the compile builds the formula of an acyclic atom with the
/// rules `rules` as its body's conjunction `λ_a` and stores it as itself:
/// where it has one rule. The dual would store `μ_a`, a negation of that
/// conjunction, and negate it back for each positive read of `a`; the
/// primal built `¬body` and negated it. Stored so, only a negative read
/// negates it. On the circuits a bit-blasting front end builds, two thirds
/// of the atoms are gates of one rule (`ground::steps`).
fn conjoined(gp: &GroundProgram, rules: &[usize]) -> bool {
    matches!(rules, &[r] if !gp.rules[r].pairs)
}

/// The fact `v` and the derived atom `k` of an atom whose rules are
/// exactly `v, k` and `\+v, \+k`, in any order: the atom holds where `v`
/// and `k` agree, and its formula is a function of `v` and `k`'s alone.
fn equivalence(gp: &GroundProgram, rules: &[usize]) -> Option<(AtomId, AtomId)> {
    let &[r, s] = rules else {
        return None;
    };
    // A body of one fact literal and one derived literal, fact first.
    let split = |ri: usize| -> Option<(GLit, GLit)> {
        let &[x, y] = gp.rules[ri].body.as_slice() else {
            return None;
        };
        let derived = |l: GLit| gp.kinds[l.atom() as usize] == AtomKind::Derived;
        match (derived(x), derived(y)) {
            (false, true) => Some((x, y)),
            (true, false) => Some((y, x)),
            _ => None,
        }
    };
    let ((v, k), (w, j)) = (split(r)?, split(s)?);
    let agree = |f: GLit, d: GLit| f.is_positive() == d.is_positive();
    (v.atom() == w.atom()
        && k.atom() == j.atom()
        && agree(v, k)
        && agree(w, j)
        && v.is_positive() != w.is_positive())
    .then_some((v.atom(), k.atom()))
}

/// Per component, the derived atoms whose formulas nothing reads once it is
/// finished, and per atom the component after which that is (`u32::MAX`
/// for an atom kept to the end): the evidence is kept, and the queries of
/// a program with evidence, and every other atom lives until the last
/// component whose rules read it, or whose atom reuses its formula as the
/// first of its [`equivalence`] class. Without evidence a query needs only
/// its formula's model count, which the compile takes before it drops the
/// formula (`Compilation::drop_dying`).
/// A formula kept after its last reader costs its memory, and under a
/// dynamic vtree it is one of the members the search shrinks: the search
/// fits the vtree to formulas the compile no longer needs.
fn dying_atoms(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    sccs: &[Vec<AtomId>],
    comp_of: &[u32],
) -> (Vec<u32>, Vec<AtomId>, Vec<u32>) {
    let mut last = comp_of.to_vec();
    for r in &gp.rules {
        let reader = comp_of[r.head as usize];
        for l in &r.body {
            let b = l.atom() as usize;
            if comp_of[b] != u32::MAX {
                last[b] = last[b].max(reader);
            }
        }
    }
    // The first atom compiled of each equivalence class lends its formula
    // to the later ones; components are compiled in order.
    let mut first: FxHashMap<AtomId, AtomId> = FxHashMap::default();
    for scc in sccs {
        if let &[a] = scc.as_slice()
            && !deps::is_cyclic(gp, by_head, scc)
            && let Some((_, k)) = equivalence(gp, &by_head[a as usize])
        {
            let source = *first.entry(k).or_insert(a);
            last[source as usize] = last[source as usize].max(comp_of[a as usize]);
        }
    }
    let kept = if gp.evidence.is_empty() {
        &[][..]
    } else {
        &gp.queries[..]
    };
    for (_, id, _) in kept.iter().chain(&gp.evidence) {
        if let &Some(a) = id {
            last[a as usize] = u32::MAX;
        }
    }
    // The atoms of each component in one list, ascending within each, in
    // two passes over `last`: one to count them, one to place them.
    let mut start = vec![0u32; sccs.len() + 1];
    for &l in &last {
        if l != u32::MAX {
            start[l as usize + 1] += 1;
        }
    }
    let mut sum = 0;
    for s in &mut start {
        sum += *s;
        *s = sum;
    }
    let mut atoms = vec![0; start[sccs.len()] as usize];
    let mut at = start.clone();
    for (a, &l) in last.iter().enumerate() {
        if l != u32::MAX {
            atoms[at[l as usize] as usize] = a as AtomId;
            at[l as usize] += 1;
        }
    }
    (start, atoms, last)
}

/// [`Dying::reads_left`] of an atom whose last reader has taken its
/// formula: a read after it is a bug.
const TAKEN: u32 = u32::MAX;

/// Per atom, how many times the last component that reads its formula
/// reads it, where that component is one acyclic atom's: once at most for
/// each body of that atom's rules the atom is in, a rule of pairs counted
/// by its pairs ([`GroundRule::bodies`]); 0 for every other atom. A read of
/// one literal for several bodies at once (the factoring's shared
/// literals) says how many it stands for, and a body the compile drops or
/// stops reading leaves its reads uncounted, so the count reaches 0 at the
/// last read or not at all: that read takes the formula rather than
/// copying it, where the run is the compile's last ([`Compilation::run`]).
fn last_reads(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    sccs: &[Vec<AtomId>],
    dies_after: &[u32],
) -> Vec<u32> {
    let mut left = vec![0u32; gp.num_atoms()];
    for (c, scc) in sccs.iter().enumerate() {
        let &[a] = scc.as_slice() else { continue };
        if deps::is_cyclic(gp, by_head, scc) {
            continue;
        }
        for &ri in &by_head[a as usize] {
            for body in gp.rules[ri].bodies() {
                for l in body.iter() {
                    let b = l.atom() as usize;
                    if gp.kinds[b] == AtomKind::Derived && dies_after[b] == c as u32 {
                        left[b] += 1;
                    }
                }
            }
        }
    }
    left
}

/// The formulas the compile drops once nothing reads them ([`dying_atoms`]),
/// the reads that take them rather than copy them ([`last_reads`]), and the
/// queries counted before they are dropped.
struct Dying {
    /// The atoms that die after each component: those after component `i`
    /// are `atoms[start[i]..start[i + 1]]`.
    start: Vec<u32>,
    atoms: Vec<AtomId>,
    /// Per atom, the component after which it dies, `u32::MAX` for an atom
    /// kept to the end.
    dies_after: Vec<u32>,
    /// Per atom, the reads of its formula left in the last component that
    /// reads it ([`last_reads`]), [`TAKEN`] once the last has taken it.
    reads_left: std::cell::RefCell<Vec<u32>>,
    /// The formulas taken so, over every run.
    taken: std::cell::Cell<usize>,
    /// The query atoms counted when their formulas are dropped, with the
    /// facts' probabilities to count them by: those of a program without
    /// evidence.
    queried: Option<(FxHashSet<AtomId>, Probabilities)>,
    /// Per atom, the query atoms its formula answers through a template
    /// ([`templates`]), counted with it.
    answers: FxHashMap<AtomId, Vec<AtomId>>,
}

impl Dying {
    /// The dying atoms of `gp`, compiled in the components `sccs`, of which
    /// `comp_of` gives each derived atom's; `queried` counted as they die,
    /// with the templates `answers` names.
    fn new(
        gp: &GroundProgram,
        by_head: &RulesByHead,
        sccs: &[Vec<AtomId>],
        comp_of: &[u32],
        queried: Option<(FxHashSet<AtomId>, Probabilities)>,
        answers: FxHashMap<AtomId, Vec<AtomId>>,
    ) -> Self {
        let (start, atoms, dies_after) = dying_atoms(gp, by_head, sccs, comp_of);
        // A query counted as its formula is dropped is read by that count,
        // which the last reader would have taken it from.
        let counted_at_drop = |a: usize| {
            queried
                .as_ref()
                .is_some_and(|(atoms, _)| atoms.contains(&(a as AtomId)))
                || answers.contains_key(&(a as AtomId))
        };
        let mut reads_left = last_reads(gp, by_head, sccs, &dies_after);
        for (a, left) in reads_left.iter_mut().enumerate() {
            if counted_at_drop(a) {
                *left = 0;
            }
        }
        Dying {
            start,
            atoms,
            dies_after,
            reads_left: std::cell::RefCell::new(reads_left),
            taken: std::cell::Cell::new(0),
            queried,
            answers,
        }
    }

    /// The atoms that die after component `i`.
    fn after(&self, i: usize) -> &[AtomId] {
        &self.atoms[self.start[i] as usize..self.start[i + 1] as usize]
    }
}

/// The scopes of the components under projection, each built as its
/// component starts and dropped once nothing reads its formulas.
///
/// Each component gets a vtree projected to its ancestor cone
/// ([`cone::Cones`]), held by its variables ([`ScopeTable`]) so that
/// components with the same cone — the values of one Bayesian-network
/// variable, say — share one vtree allocation and the embeds between them
/// are free. A component's cone and vtree are worked out when it starts,
/// narrowed to what its formulas depend on once it is finished where that
/// pays ([`Compilation::narrow`]), and go when its formulas are all
/// dropped: worked out for every component at the start, the cones of a
/// bit-blasted circuit of 82 K gates held 12.9 M variables over 15.9 K
/// vtrees, nearly all of the run's 1.4 GB, and the next size up four times
/// the pieces did not fit in 16 GB.
struct Projected {
    /// At this many facts and above a component keeps the global vtree.
    /// Cones grow along dependencies, so the components that keep it are
    /// closed upwards and no formula is ever embedded downwards into a cone.
    keep_global: usize,
    cones: cone::Cones,
    /// The components whose scopes are built: those before it.
    started: usize,
    /// The projected scopes by their variables, while a formula on one
    /// lives.
    by_vars: ScopeTable,
    /// Per component, its atoms whose formulas are still held.
    held: Vec<u32>,
    stats: ProjectionStats,
}

impl Projected {
    /// `None` where every component keeps the global scope:
    /// [`Projection::Off`], or a program with no probabilistic fact, which
    /// has nothing to project onto (every formula is constant and there is
    /// no smaller vtree than the one the strategy built).
    fn new(
        sccs: &[Vec<AtomId>],
        var_of: &[Option<VarId>],
        projection: Projection,
    ) -> (Option<Self>, Option<ProjectionStats>) {
        let num_vars = var_of.iter().flatten().count() as u32;
        let Some(keep_global) = projection.global_from(num_vars) else {
            return (None, None);
        };
        if num_vars == 0 {
            return (None, Some(ProjectionStats::default()));
        }
        let projected = Projected {
            keep_global,
            cones: cone::Cones::new(sccs.len(), num_vars),
            started: 0,
            by_vars: ScopeTable::default(),
            held: sccs.iter().map(|scc| scc.len() as u32).collect(),
            stats: ProjectionStats {
                facts: num_vars,
                ..Default::default()
            },
        };
        (Some(projected), None)
    }

    /// The scope of component `ci`, the next to start: its cone's, or the
    /// global one over the limit.
    #[allow(clippy::too_many_arguments)]
    fn start(
        &mut self,
        gp: &GroundProgram,
        by_head: &RulesByHead,
        sccs: &[Vec<AtomId>],
        ci: usize,
        var_of: &[Option<VarId>],
        comp_of: &[u32],
        backend: &TddBackend,
    ) -> Result<Option<VtreeScope>, CompileError> {
        debug_assert_eq!(ci, self.started, "components start in order");
        self.started += 1;
        let Some(vars) = self
            .cones
            .of(gp, by_head, sccs, ci, var_of, comp_of, self.keep_global)
        else {
            return Ok(None);
        };
        self.stats.projected += 1;
        self.stats.largest_kept = self.stats.largest_kept.max(vars.len() as u32);
        if let Some(scope) = self.by_vars.get(&vars) {
            return Ok(Some(scope));
        }
        let scope = backend.scope(&vars)?;
        if scope.vars().is_some() {
            self.stats.scopes += 1;
            self.by_vars.insert(&scope);
        }
        Ok(Some(scope))
    }

    /// The scope over `vars`, ascending, where one lives.
    fn live(&self, vars: &[VarId]) -> Option<VtreeScope> {
        self.by_vars.get(vars)
    }

    /// Component `ci` holds its formulas on `scope`, over `vars`, the
    /// variables they depend on, from now on; `built` where that scope is
    /// new.
    fn narrowed(&mut self, ci: usize, vars: &[VarId], scope: &VtreeScope, built: bool) {
        if built {
            self.by_vars.insert(scope);
            self.stats.scopes += 1;
        }
        self.cones.narrow(ci, vars);
        self.stats.narrowed += 1;
    }

    /// One atom of component `ci` dropped; whether that was its last, and
    /// its scope is to go.
    fn dropped(&mut self, ci: usize) -> bool {
        self.held[ci] -= 1;
        if self.held[ci] > 0 {
            return false;
        }
        self.cones.release(ci);
        true
    }
}

/// The smallest scope [`Compilation::narrow`] narrows, and only to half its
/// variables or fewer. A narrowed component's formulas get a vtree of
/// their own, which its readers no longer share, and the support walk
/// and the restrict copy them: narrowing every scope that could be made
/// smaller took addFun_max_w17 from 36 s to 114 s. The decision circuits'
/// gain is in a few dozen components with scopes of thousands of
/// variables whose formulas depend on a few (weekend_w30 15% faster).
const NARROW_FROM: usize = 1024;

/// Compile the whole program in one run under `watch`: `Compilation::new`,
/// one [`Compilation::run`], `Compilation::finish`; the tests' compile of
/// a program on a vtree of their own.
#[cfg(test)]
pub(crate) fn compile(
    gp: &GroundProgram,
    var_of: Vec<Option<VarId>>,
    formulation: Formulation,
    backend: &TddBackend,
    projection: Projection,
    watch: Option<usize>,
) -> Result<Compiled, CompileError> {
    let mut compilation =
        Compilation::new(gp, &Deps::of(gp), var_of, formulation, backend, projection)?;
    compilation.run(gp, backend, watch)?;
    compilation.finish(backend)
}

/// Where a run of a [`Compilation`] under `auto`'s turns stops on its
/// projected finish (`src/auto/`): at the end of a component, once the
/// trial has spent `from` work units, where the work it has spent times
/// the components for each one finished is past `limit`, a base-2
/// logarithm. The projection takes the components done against those left
/// at the pace of the trial so far; it reads nothing inside a component, and
/// a stop between components loses nothing the next run would build again.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Horizon {
    /// The backend's count of work units at which the trial had spent none.
    pub(crate) origin: u64,
    /// The work the trial has spent before the projection is read.
    pub(crate) from: u64,
    /// The base-2 logarithm of the work past which a projected finish
    /// stops the run.
    pub(crate) limit: f64,
}

/// A compilation that can stop at an oversize formula and go on later under
/// a larger watch, keeping every component finished so far: `auto`
/// raises a trial's watch turn by turn, and a turn need not compile the
/// trial from the start again. The component
/// where a run stops is reset and starts over in the next run: its
/// intermediate results are lost, the other components' formulas are not.
pub(crate) struct Compilation {
    /// Where a run stops between components on its projected finish
    /// ([`Horizon`]), set by its caller before each run.
    pub(crate) horizon: Option<Horizon>,
    /// The largest base-2 logarithm of the work at which the last run's
    /// horizon projected the compile to finish, `None` where it read none.
    pub(crate) reach: Option<f64>,
    var_of: Vec<Option<VarId>>,
    formulation: Formulation,
    by_head: RulesByHead,
    sccs: Vec<Vec<AtomId>>,
    /// The component of each derived atom.
    comp_of: Vec<u32>,
    /// The program's encoding as the clause loops read it.
    encoded: Encoded,
    projection: Projection,
    /// The scope each component's formulas are built on. Under
    /// [`Projection::Off`] every component shares the one global scope, and
    /// every embed below is a no-op. Under projection a component's entry
    /// is the global scope until it starts and again once its formulas are
    /// dropped ([`Projected`]).
    scopes: Vec<VtreeScope>,
    projected: Option<Projected>,
    /// The first component not finished.
    next: usize,
    fixpoint: Fixpoint,
    scratch: std::cell::RefCell<Scratch>,
    memo: std::cell::RefCell<ClauseMemo>,
    /// Per derived atom `k`, the first atom compiled that holds exactly where
    /// a fact agrees with `k` ([`equivalence`]), with that fact.
    equivalents: FxHashMap<AtomId, (AtomId, AtomId)>,
    /// The rotation search of a dynamic vtree, `None` on a static one.
    dynamic: Option<Dynamic>,
    dying: Dying,
    /// Whether a run without a watch failed: it may have taken formulas
    /// that the components it did not finish read, and cannot go on.
    spent: bool,
    /// What the query phase takes over ([`Compiled::for_queries`]).
    for_queries: ForQueries,
    stats: CompileStats,
    /// Calls per operation kind ([`Op`]).
    counts: std::cell::RefCell<[usize; 5]>,
    negations: std::cell::RefCell<Negations>,
}

/// The formula of each atom and the fixpoint's bookkeeping.
struct Fixpoint {
    /// Per atom: the formula (dual: μ_a = ¬λ_a, but λ_a for an atom of one
    /// rule), and the formula of the other polarity, cached until the atom
    /// changes: for the dual the one recovered by one negation, for the
    /// primal `¬λ_a`, which an update builds before it negates it into
    /// `λ_a`.
    formulas: Formulas,
    lambda_cache: std::cell::RefCell<Formulas>,
    /// Per atom, whether `formulas` holds λ_a (`true`) or μ_a: the
    /// formulation's polarity, but λ_a for the atoms the dual conjoins
    /// ([`conjoined`]).
    positive: std::cell::RefCell<Vec<bool>>,
    /// Semi-naive bookkeeping. `version[a]` counts changes of atom a's
    /// formula; `seen[ri]` is the sum of the derived body atoms' versions
    /// when rule ri was last folded into its head. Within a positive
    /// component the interpretation grows monotonically, so a rule whose
    /// body atoms have not changed since contributes nothing new: primal
    /// `λ_a' = λ_a ∨ ⋁_changed body`, dual `μ_a' = μ_a ∧ ⋀_changed ¬body`.
    version: std::cell::RefCell<Vec<u64>>,
    seen: std::cell::RefCell<Vec<u64>>,
    /// The worklist of the cyclic component being compiled, between rounds.
    worklist: Option<Worklist>,
    /// Live pairs of each atom's formula, and their sum, the cached
    /// positives left out.
    sizes: Vec<usize>,
    live: usize,
}

impl Fixpoint {
    /// Drop atom `a`'s formula and its cached other polarity, and take its
    /// pairs from the live count.
    fn drop(&mut self, a: usize) {
        self.formulas.take(a);
        self.live -= std::mem::take(&mut self.sizes[a]);
        self.lambda_cache.get_mut().take(a);
    }
}

/// The program's [`Encoding`](crate::ground::Encoding) as the clause loops
/// read it ([`Factoring`]).
struct Encoded {
    /// Per variable, the link of the indicator it is the copy of,
    /// `AtomId::MAX` for none.
    link_of: Vec<AtomId>,
    /// Per variable, its group of siblings (`Encoding::siblings`),
    /// `u32::MAX` for none.
    sibling_of: Vec<u32>,
    /// Whether each atom's loops of clauses may hold the at-most-one of
    /// the sibling groups its bodies read ([`Factoring::seeds`]).
    seeds: Vec<bool>,
    /// Whether each atom is an indicator's link.
    links: Vec<bool>,
}

impl Encoded {
    /// `gp`'s encoding by the variables `var_of` gives its facts, of which
    /// there are `num_vars` with the unused 0.
    fn new(gp: &GroundProgram, var_of: &[Option<VarId>], num_vars: usize) -> Self {
        let n = gp.num_atoms();
        let mut links = vec![false; n];
        let mut link_of = vec![AtomId::MAX; num_vars];
        for i in &gp.encoding.indicators {
            links[i.link as usize] = true;
            link_of[var_of[i.var as usize].expect("a copy is a fact").0 as usize] = i.link;
        }
        let mut sibling_of = vec![u32::MAX; num_vars];
        for (g, group) in gp.encoding.siblings.iter().enumerate() {
            for &c in group {
                sibling_of[var_of[c as usize].expect("a copy is a fact").0 as usize] = g as u32;
            }
        }
        // A bit's loops hold their sibling groups: only its link reads it.
        // Held by every atom but the links, the loops of the tables kept as
        // rules took more work; held by the bits alone, they take less.
        let mut seeds = vec![false; n];
        for code in &gp.encoding.codes {
            for &b in &code.bits {
                seeds[b as usize] = true;
            }
        }
        Encoded {
            link_of,
            sibling_of,
            seeds,
            links,
        }
    }
}

/// What the query phase takes over from the compile beside the formulas:
/// the queries it counted, and the atoms whose formulas it did not build.
#[derive(Default)]
pub(crate) struct ForQueries {
    /// The probabilities of each query atom and of its negation, the one of
    /// them its stored formula's model count and the other its complement
    /// ([`crate::eval::complement`]), taken where the compile dropped the
    /// formula ([`dying_atoms`]); its entry in `formulas` is then `None`.
    /// Empty for a program with evidence.
    pub(crate) counted: FxHashMap<AtomId, (f64, f64)>,
    /// Counts of `counted` taken again because the float count was not
    /// exact (`eval::Range`).
    pub(crate) recounts: usize,
    /// The query atoms answered by another query atom's formula under other
    /// probabilities ([`templates`]), whose own formulas are not built.
    pub(crate) templates: FxHashMap<AtomId, Template>,
    /// The readers' links of clause tables tied to another reader's copies
    /// ([`ties`]), whose own formulas are not built.
    pub(crate) ties: FxHashMap<AtomId, Tie>,
    /// The links of the names expanded ([`super::settle`]), which the
    /// evidence leaves out, and the copies they tied, which no formula
    /// reads any more.
    pub(crate) dropped: FxHashSet<AtomId>,
    pub(crate) released: Vec<VarId>,
    /// The cuts whose names were kept and expanded.
    pub(crate) settled: super::settle::Settled,
}

impl Compilation {
    pub(crate) fn new(
        gp: &GroundProgram,
        deps: &Deps,
        var_of: Vec<Option<VarId>>,
        formulation: Formulation,
        backend: &TddBackend,
        projection: Projection,
    ) -> Result<Self, CompileError> {
        let n = gp.num_atoms();
        for (a, k) in gp.kinds.iter().enumerate() {
            if matches!(k, AtomKind::Fact(_)) && var_of[a].is_none() {
                return Err(CompileError::UnmappedFact(a as AtomId));
            }
        }
        // The rules by head and the components are `gp`'s, worked out once
        // for its layouts and this compile ([`Deps`]).
        debug_assert!(
            *deps == Deps::of(gp),
            "the rules by head and components are another program's"
        );
        let by_head = deps.by_head.clone();
        // A query answered through a template is a leaf, a component of its
        // own that nothing reads, and is left out.
        let templates = templates(gp, &by_head);
        let mut answers: FxHashMap<AtomId, Vec<AtomId>> = FxHashMap::default();
        for (&x, t) in &templates {
            answers.entry(t.atom).or_default().push(x);
        }
        // A link tied to another reader's copies is not built either.
        let ties = ties(gp, &by_head, &var_of, backend, projection);
        let sccs: Vec<Vec<AtomId>> =
            deps.sccs.iter().filter(|scc| !matches!(scc[..], [a] if templates.contains_key(&a) || ties.contains_key(&a))).cloned().collect();
        let comp_of = cone::component_of(n, &sccs);
        let (projected, projection_stats) = Projected::new(&sccs, &var_of, projection);
        let scopes = vec![backend.global(); sccs.len()];
        let num_vars = var_of
            .iter()
            .flatten()
            .map(|v| v.0 as usize + 1)
            .max()
            .unwrap_or(0);
        let encoded = Encoded::new(gp, &var_of, num_vars);
        let queried: Option<(FxHashSet<AtomId>, Probabilities)> =
            gp.evidence.is_empty().then(|| {
                let atoms = gp.queries.iter().filter_map(|&(_, id, _)| id).collect();
                (
                    atoms,
                    Probabilities::from_program(gp, &var_of, backend.vtree().num_vars()),
                )
            });
        let dying = Dying::new(gp, &by_head, &sccs, &comp_of, queried, answers);
        // Room for an entry for each atom the compile gives a formula.
        let stored = sccs.iter().map(Vec::len).sum();
        let stats = CompileStats {
            components: sccs.len(),
            templated: templates.len(),
            tied: ties.len(),
            projection: projection_stats,
            ..Default::default()
        };
        let fixpoint = Fixpoint {
            formulas: Formulas::new(n, stored),
            lambda_cache: std::cell::RefCell::new(Formulas::new(n, 0)),
            positive: std::cell::RefCell::new(vec![formulation == Formulation::Primal; n]),
            version: std::cell::RefCell::new(vec![1; n]),
            seen: std::cell::RefCell::new(vec![0; gp.rules.len()]),
            worklist: None,
            sizes: vec![0; n],
            live: 0,
        };
        Ok(Compilation {
            horizon: None,
            reach: None,
            var_of,
            formulation,
            by_head,
            sccs,
            comp_of,
            encoded,
            projection,
            scopes,
            projected,
            next: 0,
            fixpoint,
            scratch: std::cell::RefCell::new(Scratch::new(num_vars, n)),
            memo: std::cell::RefCell::new(ClauseMemo::default()),
            equivalents: FxHashMap::default(),
            dynamic: None,
            dying,
            spent: false,
            for_queries: ForQueries {
                templates,
                ties,
                ..ForQueries::default()
            },
            stats,
            counts: std::cell::RefCell::new([0usize; 5]),
            negations: std::cell::RefCell::new(Negations::default()),
        })
    }

    /// Search for a smaller global vtree between the steps of the compile:
    /// after each component, and after each round of a cyclic one. The first
    /// search runs once the atoms' formulas hold `first_pairs` live pairs
    /// ([`DYNAMIC_FIRST_PAIRS`] for the `dynamic` strategies), and each
    /// later one once they have doubled since the last, as CUDD reorders
    /// when its tables double, over every formula and cached positive at
    /// once ([`TddBackend::restructure`]). A search sweeps until a sweep
    /// keeps no move. Its budget is the work the compile spent since the
    /// last search, and no more than the components left are likely to take
    /// ([`Dynamic::budget`]); what it spends past the budget, the compile
    /// spends before the next search runs, so searching costs at most twice
    /// what the compile does, plus the last search's excess. A budget that
    /// stopped searches part way made each compile's outcome depend on
    /// where they stopped: moving it by 2% made single programs several
    /// times slower or faster. Rotations change how a formula is
    /// represented, not its function, so versions, worklists and the
    /// fixpoint test are untouched.
    ///
    /// # Panics
    ///
    /// Under projection, whose scopes are projections of the vtree a search
    /// would change.
    #[must_use]
    pub(crate) fn dynamic(mut self, first_pairs: usize) -> Self {
        assert_eq!(
            self.projection,
            Projection::Off,
            "a dynamic vtree needs every formula on the global scope"
        );
        self.dynamic = Some(Dynamic {
            first_pairs,
            next_at: first_pairs,
            since: None,
            resumed: false,
            debt: 0,
        });
        self.stats.search = Some(SearchStats::default());
        self
    }

    /// The components finished, and all of them.
    pub(crate) fn progress(&self) -> (usize, usize) {
        (self.next, self.sccs.len())
    }

    /// The round a cyclic component under way is at, `None` between
    /// components.
    pub(crate) fn rounds(&self) -> Option<usize> {
        self.fixpoint.worklist.as_ref().map(|w| w.iterations)
    }

    /// Compile from the first unfinished component on, under `watch`, a
    /// size in pairs: an atom formula that ends up larger, or an
    /// intermediate result of an atom update that is larger, stops the
    /// compilation with [`CompileError::Oversize`]. Pairs produced inside a
    /// single engine operation are the engine's limits' business. After an
    /// error the compilation stands at the component that failed, ready
    /// for another run: a cyclic component at the atom update that failed,
    /// with its rounds so far, the others from their start. A run without a watch is the last: the last read of a
    /// formula takes it rather than copying it ([`last_reads`]), so after
    /// an error there is no other run.
    ///
    /// # Panics
    ///
    /// When a run without a watch failed before.
    pub(crate) fn run(
        &mut self,
        gp: &GroundProgram,
        backend: &TddBackend,
        watch: Option<usize>,
    ) -> Result<(), CompileError> {
        assert!(
            !self.spent,
            "a compile whose run without a watch failed cannot go on"
        );
        let start = Instant::now();
        let work = backend.work_units();
        let refused = self.memo.get_mut().stats.refused;
        if let Some(dynamic) = self.dynamic.as_mut() {
            dynamic.resumed = dynamic.since.is_some();
        }
        self.reach = None;
        let result = (|| {
            while self.next < self.sccs.len() {
                if let Some(p) = self.projected.as_mut()
                    && p.started == self.next
                    && let Some(scope) = p.start(
                        gp,
                        &self.by_head,
                        &self.sccs,
                        self.next,
                        &self.var_of,
                        &self.comp_of,
                        backend,
                    )?
                {
                    self.scopes[self.next] = scope;
                }
                let finished = self.round(gp, backend, watch)?;
                if finished {
                    self.next += 1;
                    self.drop_dying(backend, self.next - 1)?;
                    self.narrow(backend, self.next - 1)?;
                }
                self.search(backend)?;
                if finished {
                    self.look_ahead(backend)?;
                }
            }
            Ok(())
        })();
        // A cyclic component keeps its rounds so far; the others start
        // over.
        if result.is_err() && self.next < self.sccs.len() && self.fixpoint.worklist.is_none() {
            self.reset(self.next);
        }
        self.spent = result.is_err() && watch.is_none();
        // A run keeps only the loop the watch refused it: one kept by an
        // earlier run and not taken by this one is not where it stopped
        // (`clauses::Refused`).
        if self.memo.get_mut().stats.refused == refused {
            self.memo.get_mut().forget_refused();
        }
        self.stats.seconds += start.elapsed().as_secs_f64();
        self.stats.work_units += backend.work_units() - work;
        self.stats.op_counts = *self.counts.borrow();
        self.stats.taken = self.dying.taken.get();
        if let Some(p) = &self.projected {
            self.stats.projection = Some(p.stats);
        }
        self.stats.negations = *self.negations.borrow();
        self.stats.clause_memo = self.memo.borrow().stats;
        result
    }

    /// Stops the run, between components, where its [`Horizon`] projects
    /// the compile to finish past its limit.
    fn look_ahead(&mut self, backend: &TddBackend) -> Result<(), CompileError> {
        let Some(h) = self.horizon else {
            return Ok(());
        };
        let (done, all) = self.progress();
        let spent = backend.work_units().saturating_sub(h.origin);
        if done == 0 || done == all || spent < h.from {
            return Ok(());
        }
        let work = (spent as f64 * all as f64 / done as f64).log2();
        self.reach = Some(self.reach.map_or(work, |r| r.max(work)));
        if work > h.limit {
            return Err(CompileError::Projected { work });
        }
        Ok(())
    }

    /// Forget everything about component `i`, so that the next run starts
    /// it over: its formulas, cached positives, versions, and the rules
    /// folded into its atoms.
    fn reset(&mut self, i: usize) {
        let fixpoint = &mut self.fixpoint;
        fixpoint.worklist = None;
        for &a in &self.sccs[i] {
            fixpoint.drop(a as usize);
            fixpoint.positive.get_mut()[a as usize] = self.formulation == Formulation::Primal;
            fixpoint.version.get_mut()[a as usize] = 1;
            for &ri in &self.by_head[a as usize] {
                fixpoint.seen.get_mut()[ri] = 0;
            }
        }
    }

    /// Drop the formulas that nothing reads once component `i` is finished,
    /// a query's counted first. A count stopped by the engine's limits
    /// leaves that formula and the ones after it in place, for the query
    /// phase to count.
    fn drop_dying(&mut self, backend: &TddBackend, i: usize) -> Result<(), CompileError> {
        for &a in self.dying.after(i) {
            if let Some((atoms, weights)) = &self.dying.queried
                && atoms.contains(&a)
                && let Some(f) = &self.fixpoint.formulas[a as usize]
            {
                let scope = &self.scopes[self.comp_of[a as usize] as usize];
                let w = scope
                    .vars()
                    .map_or_else(|| weights.clone(), |vars| weights.restrict(vars));
                // The complement's formula, negated once for the atom and the
                // templates it answers, where one of them needs it. The
                // counts are of the atom and its negation, in that order.
                let mut negated = None;
                let positive = self.fixpoint.positive.borrow()[a as usize];
                let mut both =
                    |w: &Probabilities, recounts: &mut usize| -> Result<(f64, f64), CompileError> {
                        let m = crate::eval::mass(backend.engine, f, w, recounts)?.to_f64();
                        let c = crate::eval::complement(m, || -> Result<f64, CompileError> {
                            if negated.is_none() {
                                negated = Some(self.negations.borrow_mut().negate(
                                    backend,
                                    NegationSource::Query,
                                    backend.copy(f)?,
                                )?);
                            }
                            Ok(crate::eval::mass(
                                backend.engine,
                                negated.as_ref().unwrap(),
                                w,
                                recounts,
                            )?
                            .to_f64())
                        })?;
                        Ok(if positive { (m, c) } else { (c, m) })
                    };
                let counts = both(&w, &mut self.for_queries.recounts)?;
                self.for_queries.counted.insert(a, counts);
                for &x in self.dying.answers.get(&a).map_or(&[][..], Vec::as_slice) {
                    let w = self.for_queries.templates[&x].weights(&w, &self.var_of, scope);
                    let counts = both(&w, &mut self.for_queries.recounts)?;
                    self.for_queries.counted.insert(x, counts);
                }
            }
            // A formula its last reader took is gone already, and its
            // pairs leave the live count here.
            self.fixpoint.drop(a as usize);
            let c = self.comp_of[a as usize] as usize;
            if let Some(p) = self.projected.as_mut()
                && p.dropped(c)
            {
                self.scopes[c] = backend.global();
            }
        }
        Ok(())
    }

    /// Move the formulas of component `i`, just finished, onto the scope of
    /// the variables they depend on, where that holds half of their own or
    /// fewer and their own holds [`NARROW_FROM`] or more. A cone is read
    /// off the grounding and holds every fact the rules below mention; a
    /// formula can depend on far fewer (a gate the evidence fixed, a term
    /// that cancels), and every level of the scope costs every operation
    /// on it. The components that read it then start from the union of
    /// what their parts depend on ([`cone::Cones::narrow`]). A support at
    /// the global limit keeps the scope it has.
    fn narrow(&mut self, backend: &TddBackend, i: usize) -> Result<(), CompileError> {
        let Some(p) = self.projected.as_mut() else {
            return Ok(());
        };
        let scc = &self.sccs[i];
        let scope = &self.scopes[i];
        if scope.len() < NARROW_FROM {
            return Ok(());
        }
        let formulas = &self.fixpoint.formulas;
        let mut vars = Vec::new();
        let mut held = false;
        for &a in scc {
            if let Some(f) = &formulas[a as usize] {
                held = true;
                for v in backend.engine.support(f)? {
                    vars.push(scope.vars().map_or(v, |vars| vars[v.0 as usize - 1]));
                }
            }
        }
        if !held {
            return Ok(());
        }
        vars.sort_unstable();
        vars.dedup();
        if vars.is_empty() {
            // Constants: any one variable, as for a cone without a fact
            // (`cone::Cones::of`).
            vars.push(scope.vars().map_or(VarId(1), |vars| vars[0]));
        }
        if 2 * vars.len() > scope.len() || vars.len() >= p.keep_global {
            return Ok(());
        }
        let mut into = p.live(&vars);
        let built = into.is_none();
        let mut moved = Vec::new();
        {
            let cache = self.fixpoint.lambda_cache.borrow();
            for &a in scc {
                for (cached, f) in [(false, &formulas[a as usize]), (true, &cache[a as usize])] {
                    if let Some(f) = f {
                        let (g, on) = backend.restrict(f, scope, &vars, into.as_ref())?;
                        into = Some(on);
                        moved.push((a as usize, cached, g));
                    }
                }
            }
        }
        let into = into.expect("a component holding a formula moved it");
        let fixpoint = &mut self.fixpoint;
        for (a, cached, g) in moved {
            if cached {
                fixpoint.lambda_cache.get_mut()[a] = Some(g);
            } else {
                let size = backend.size(&g);
                fixpoint.live = fixpoint.live - fixpoint.sizes[a] + size;
                fixpoint.sizes[a] = size;
                fixpoint.formulas[a] = Some(g);
            }
        }
        p.narrowed(i, &vars, &into, built);
        self.scopes[i] = into;
        Ok(())
    }

    /// Hand out the formulas in their polarities. The compilation
    /// is emptied; it is done.
    pub(crate) fn finish(&mut self, backend: &TddBackend) -> Result<Compiled, CompileError> {
        assert!(
            self.next == self.sccs.len(),
            "finish called on an unfinished compilation"
        );
        let formulas = std::mem::take(&mut self.fixpoint.formulas);
        self.fixpoint.lambda_cache.borrow_mut().clear();
        self.stats.final_max_pairs = formulas
            .iter()
            .flatten()
            .map(|f| backend.size(f))
            .max()
            .unwrap_or(0);
        self.stats.queries_counted = self.for_queries.counted.len();
        Ok(Compiled {
            var_of: std::mem::take(&mut self.var_of),
            formulas,
            positive: self.fixpoint.positive.take(),
            scopes: std::mem::take(&mut self.scopes),
            comp_of: std::mem::take(&mut self.comp_of),
            projection: self.projection,
            for_queries: std::mem::take(&mut self.for_queries),
            stats: self.stats.clone(),
        })
    }

    /// One step of component `self.next`: all of it, or one round of its
    /// worklist when it is cyclic. Returns whether the component is done.
    fn round(
        &mut self,
        gp: &GroundProgram,
        backend: &TddBackend,
        watch: Option<usize>,
    ) -> Result<bool, CompileError> {
        let formulation = self.formulation;
        let by_head = &self.by_head;
        let var_of = &self.var_of;
        let link_of = &self.encoded.link_of;
        let sibling_of = &self.encoded.sibling_of;
        let seeds = &self.encoded.seeds;
        let scc = &self.sccs[self.next];
        // Every formula of this component lives here; a formula of a
        // component it depends on is moved onto it by the conjunction that
        // reads it (`part`).
        let scopes = &self.scopes;
        let comp_of = &self.comp_of;
        let scope = &self.scopes[self.next];
        let scope_of = |b: AtomId| &scopes[comp_of[b as usize] as usize];
        // `watched(f, a)` passes `f` through unless it is over the watch.
        //
        // A size read between operations counts whatever pairs the reduction
        // there left behind (`TddBackend::minimize`), and giving up a round
        // throws away the whole compile so far, so a formula that reads over
        // the watch is reduced in full and read again before it is given up.
        //
        // `watched_sized` takes `f`'s pairs as counted and returns them as
        // they are after: a count walks every node of the diagram, and an
        // atom's formula is counted once for the largest formula, the watch
        // and the live pairs.
        //
        // `check` is the watch itself, and hands back the formula it
        // refused, reduced in full, with its pairs, for a caller that keeps
        // it for the next run (`checked`, [`Factoring::checked`]).
        let check = |mut f: Tdd, pairs: usize| -> Result<Checked<(Tdd, usize)>, CompileError> {
            match watch {
                Some(w) if pairs > w => {
                    backend.minimize(&mut f, Reduce::Settled)?;
                    let pairs = backend.size(&f);
                    Ok(if pairs <= w {
                        Ok((f, pairs))
                    } else {
                        Err((f, pairs))
                    })
                }
                _ => Ok(Ok((f, pairs))),
            }
        };
        let watched_sized =
            |f: Tdd, pairs: usize, a: AtomId| -> Result<(Tdd, usize), CompileError> {
                check(f, pairs)?.map_err(|(_, pairs)| CompileError::Oversize { atom: a, pairs })
            };
        let checked = |f: Tdd| -> Result<Checked<Tdd>, CompileError> {
            let pairs = backend.size(&f);
            Ok(check(f, pairs)?.map(|(f, _)| f))
        };
        let watched = |f: Tdd, a: AtomId| -> Result<Tdd, CompileError> {
            let pairs = backend.size(&f);
            watched_sized(f, pairs, a).map(|(f, _)| f)
        };
        let version = &self.fixpoint.version;
        let seen = &self.fixpoint.seen;
        let scratch = &self.scratch;
        let memo = &self.memo;
        let body_version = |ri: usize| -> u64 {
            let v = version.borrow();
            gp.rules[ri]
                .body
                .iter()
                .filter(|l| gp.kinds[l.atom() as usize] == AtomKind::Derived)
                .map(|l| v[l.atom() as usize])
                .sum::<u64>()
                + 1
        };
        let counts = &self.counts;
        let count = |op: Op, n: usize| counts.borrow_mut()[op as usize] += n;
        let negations = &self.negations;
        let negate = |source: NegationSource, f: Tdd| -> Result<Tdd, CompileError> {
            let r = negations.borrow_mut().negate(backend, source, f)?;
            count(Op::Negate, 1);
            Ok(r)
        };
        let links = &self.encoded.links;
        let lambda_cache = &self.fixpoint.lambda_cache;
        let positive = &self.fixpoint.positive;
        let (reads_left, dies_after, next, taken) = (
            &self.dying.reads_left,
            &self.dying.dies_after,
            self.next,
            &self.dying.taken,
        );
        // A run without a watch is the last, and its last read of a formula
        // takes it ([`last_reads`]).
        let taking = watch.is_none();
        // The formula of a derived body literal on its atom's own scope: the
        // stored formula where its polarity is the literal's, else its
        // negation, which the cache keeps (λ_b of an atom the dual stores
        // as μ_b, under [`NegationSource::Lambda`]). Negation walks every
        // level of the vtree it runs on, so doing it on `b`'s own scope,
        // the smallest that can hold it, rather than on the consumer's is
        // where projection pays. The cache holds it on that scope too, and
        // the conjunction that reads it moves it (`part`). An atom with no
        // formula yet, in a cyclic component's first round, is false.
        //
        // The read stands for `covers` bodies of the component's rules,
        // none where the caller may read the literal again. In a run
        // without a watch the last read of a formula takes it, and the
        // cached other polarity, rather than copying them: the end of the
        // component would drop them.
        let derived =
            |l: GLit, covers: u32, formulas: &mut Formulas| -> Result<Tdd, CompileError> {
                let b = l.atom() as usize;
                debug_assert!(
                    dies_after[b] as usize >= next,
                    "atom {b} read after its formula was dropped"
                );
                let last = {
                    let mut left = reads_left.borrow_mut();
                    assert!(
                        left[b] != TAKEN,
                        "atom {b} read after its last reader took its formula"
                    );
                    let counted =
                        taking && covers > 0 && left[b] > 0 && dies_after[b] as usize == next;
                    if counted {
                        debug_assert!(
                            covers <= left[b],
                            "atom {b} read for more bodies than hold it"
                        );
                        left[b] = left[b].saturating_sub(covers);
                        if left[b] == 0 {
                            left[b] = TAKEN;
                            taken.set(taken.get() + 1);
                        }
                    }
                    counted && left[b] == TAKEN
                };
                let own = scope_of(l.atom());
                let stored_positive = positive.borrow()[b];
                let stored = |formulas: &mut Formulas| -> Result<Tdd, CompileError> {
                    let f = if last {
                        formulas.take(b)
                    } else {
                        formulas[b].as_ref().map(|f| backend.copy(f)).transpose()?
                    };
                    Ok(f.unwrap_or_else(|| {
                        if stored_positive {
                            backend.zero(own)
                        } else {
                            backend.one(own)
                        }
                    }))
                };
                if stored_positive == l.is_positive() {
                    if last {
                        lambda_cache.borrow_mut().take(b);
                    }
                    return stored(formulas);
                }
                match formulation {
                    Formulation::Dual => {
                        let cached = if last {
                            lambda_cache.borrow_mut().take(b)
                        } else {
                            lambda_cache.borrow()[b]
                                .as_ref()
                                .map(|f| backend.copy(f))
                                .transpose()?
                        };
                        if let Some(f) = cached {
                            if last {
                                formulas.take(b);
                            }
                            return Ok(f);
                        }
                        let source = if stored_positive {
                            NegationSource::Primal
                        } else {
                            NegationSource::Lambda
                        };
                        let f = negate(source, stored(formulas)?)?;
                        if !last {
                            lambda_cache.borrow_mut()[b] = Some(backend.copy(&f)?);
                        }
                        Ok(f)
                    }
                    Formulation::Primal => negate(NegationSource::Primal, stored(formulas)?),
                }
            };
        // One application of the operator to atom `a`: the rules of `a` whose
        // bodies changed (all of them unless `incremental`) are folded into its
        // current formula. Both formulations build `⋀_r ¬body_r` with
        // [`Factoring`]: the dual conjoins it in, `μ_a' = μ_a ∧ ⋀_r ¬body_r`;
        // the primal negates it, `λ_a' = λ_a ∨ ¬⋀_r ¬body_r`, which it builds
        // as `¬(¬λ_a ∧ ⋀_r ¬body_r)`. With `conjoin`, for an atom of one
        // rule ([`conjoined`]), it builds `λ_a`, the body's conjunction,
        // instead.
        let apply = |a: AtomId,
                     formulas: &mut Formulas,
                     incremental: bool,
                     conjoin: bool|
         -> Result<(Tdd, Option<Tdd>), CompileError> {
            let mut bodies: Vec<Vec<GLit>> = Vec::new();
            let mut pairs: Vec<Pairs> = Vec::new();
            for &ri in &by_head[a as usize] {
                let bv = body_version(ri);
                if incremental && seen.borrow()[ri] == bv {
                    continue;
                }
                seen.borrow_mut()[ri] = bv;
                let rule = &gp.rules[ri];
                if rule.pairs {
                    pairs.push(Pairs {
                        at: bodies.len(),
                        rule,
                    });
                } else {
                    bodies.push(rule.body.clone());
                }
            }
            let term = if links[a as usize] {
                NegationSource::LinkTerm
            } else {
                NegationSource::Term
            };
            // The primal's update of an atom with a formula starts the
            // conjunction from `¬λ_a` (below).
            let not_acc = match formulation {
                Formulation::Primal if incremental => match (
                    lambda_cache.borrow_mut().take(a as usize),
                    &formulas[a as usize],
                ) {
                    (Some(not_acc), _) => Some(not_acc),
                    (None, Some(acc)) => Some(negate(NegationSource::Primal, backend.copy(acc)?)?),
                    (None, None) => None,
                },
                _ => None,
            };
            let started = not_acc.is_some();
            // The formula of a body literal standing for `covers` bodies,
            // with its scope: a fact literal itself, on `on`; a derived
            // one's `derived` formula, on `b`'s scope, which the conjunction
            // that reads it moves onto its own, which contains it: a cone
            // holds the cone of everything it depends on.
            let mut part =
                |l: GLit, covers: u32, on: &VtreeScope| -> Result<Formula, CompileError> {
                    let b = l.atom() as usize;
                    match gp.kinds[b] {
                        AtomKind::Fact(_) => {
                            let v = var_of[b].unwrap();
                            let lit = if l.is_positive() {
                                tididi::Literal::pos(v)
                            } else {
                                tididi::Literal::neg(v)
                            };
                            Ok((backend.literal(on, lit)?, on.clone()))
                        }
                        AtomKind::Derived => {
                            Ok((derived(l, covers, formulas)?, scope_of(l.atom()).clone()))
                        }
                    }
                };
            let mut factoring = Factoring {
                backend,
                gp,
                var_of,
                link_of,
                sibling_of,
                seeds: seeds[a as usize],
                head: a,
                scratch,
                memo,
                watched: &watched,
                checked: &checked,
                watch,
                count: &count,
                negate: &negate,
                term,
                part: &mut part,
            };
            if conjoin {
                let [body] = bodies.as_slice() else {
                    unreachable!("an atom conjoined has one rule")
                };
                return Ok((factoring.conjunction(body, scope)?, None));
            }
            let nb = factoring.negated_bodies(bodies, pairs, scope, not_acc)?;
            let (mut f, other) = match formulation {
                // `μ_a ∧ nb`, and from true, where every atom starts, `nb`
                // itself, without building true on the scope and walking
                // every level of it.
                Formulation::Dual => match formulas[a as usize].as_ref().filter(|_| incremental) {
                    Some(acc) => {
                        let r = backend.and(backend.copy(acc)?, nb)?;
                        count(Op::And, 1);
                        (r, None)
                    }
                    None => (nb, None),
                },
                Formulation::Primal if !started => (negate(NegationSource::Primal, nb)?, None),
                // `λ_a ∨ ¬nb` as `¬(¬λ_a ∧ nb)`: the disjunction of `λ_a`
                // with the negated `nb` negated both operands, and so took
                // `nb` back to itself, two negations of its four. The
                // conjunction is reduced in full before it is negated: a
                // negation fills out its operand's twins, and the
                // minimizations after it would cost what the negations save.
                // The conjunction is `¬λ_a'`, which the next update of `a`
                // starts from instead of negating `λ_a'` back.
                //
                // The chain of the bodies' terms conjoins them into `¬λ_a`
                // itself rather than into true and the result into `¬λ_a`:
                // the same number of conjunctions, each of whose operands
                // shares more with the other.
                Formulation::Primal => {
                    let mut c = nb;
                    backend.minimize(&mut c, Reduce::Settled)?;
                    let r = negations.borrow_mut().negate(
                        backend,
                        NegationSource::Primal,
                        backend.copy(&c)?,
                    )?;
                    count(Op::Or, 1);
                    (r, Some(c))
                }
            };
            // `and` leaves its result unminimized, and every result the
            // compiler keeps is minimized so the next operation starts from
            // the smallest diagram for the function. Not for the fixpoint
            // test: the update settles its formula before the test, and
            // `Engine::equivalent` minimizes a copy of an operand that is
            // not certified canonical. The `auto`'s watch does read the
            // size, and an unpruned diagram counts its dead pairs.
            backend.minimize(&mut f, Reduce::Between)?;
            count(Op::Minimize, 1);
            Ok((f, other))
        };
        // In the dual, an atom's initial (empty) interpretation is μ = true.
        let bottom = || match formulation {
            Formulation::Primal => backend.zero(scope),
            Formulation::Dual => backend.one(scope),
        };

        let formulas = &mut self.fixpoint.formulas;
        let equivalents = &mut self.equivalents;
        let stats = &mut self.stats;
        let (sizes, live) = (&mut self.fixpoint.sizes, &mut self.fixpoint.live);
        // Store `f`, of `size` pairs, as `a`'s formula, keeping the live
        // pairs.
        let mut store = |a: AtomId, f: Tdd, size: usize, formulas: &mut Formulas| {
            *live = *live - sizes[a as usize] + size;
            sizes[a as usize] = size;
            formulas[a as usize] = Some(f);
        };
        if deps::has_internal_negation(gp, by_head, scc) {
            stats.cyclic_components += 1;
            let mut lambda_lookup = |b: AtomId| -> Result<Tdd, CompileError> {
                backend.embed(derived(GLit::Pos(b), 0, formulas)?, scope_of(b), scope)
            };
            let mut updates = 0;
            let wf_negate = |f: Tdd| negate(NegationSource::WellFounded, f);
            let result = aft::compile_component(
                gp,
                by_head,
                var_of,
                scc,
                scope,
                &mut lambda_lookup,
                &wf_negate,
                &mut updates,
                backend,
            )?;
            stats.atom_updates += updates;
            for (&a, mut lambda) in scc.iter().zip(result) {
                backend.minimize(&mut lambda, Reduce::Settled)?;
                stats.max_pairs = stats.max_pairs.max(backend.size(&lambda));
                let lambda = watched(lambda, a)?;
                let f = match formulation {
                    Formulation::Primal => lambda,
                    Formulation::Dual => {
                        lambda_cache.borrow_mut()[a as usize] = Some(backend.copy(&lambda)?);
                        negate(NegationSource::WellFounded, lambda)?
                    }
                };
                let size = backend.size(&f);
                store(a, f, size, formulas);
                version.borrow_mut()[a as usize] += 1;
            }
            return Ok(true);
        }
        let cyclic = deps::is_cyclic(gp, by_head, scc);
        if !cyclic {
            let a = scc[0];
            // An atom that holds where a fact `v` agrees with a derived atom
            // `k` is a link of a reader of `k`'s table: bn_barley's aks_m2
            // has four readers. The first such atom for `k` is built from
            // `k`'s formula and its negation (`TddBackend::select_at_root`)
            // where `v` is a child of its scope's root; the others are that
            // one with `v` renamed, wherever their scope has its shape.
            // Built from its rules, a link costs most of its work in the
            // conjunction of its two negated halves.
            //
            // The renaming holds only where `v` is outside `k`'s cone, as
            // a reader's copy is: `tmp :- d, v. tmp :- \+d, \+v.` with
            // `v :- \+d` is false, and renamed to `link`'s fact `a` it
            // would make `link :- a, v. link :- \+a, \+v.` false too.
            let equal = equivalence(gp, &by_head[a as usize]);
            let outside = |v: AtomId, k: AtomId| {
                var_of[v as usize].is_some_and(|x| scope_of(k).local_of(x).is_none())
            };
            let mut reused = None;
            if let Some((v, k)) = equal
                && outside(v, k)
            {
                match equivalents
                    .get(&k)
                    .and_then(|&(v0, a0)| Some((v0, a0, formulas[a0 as usize].as_ref()?)))
                {
                    Some((v0, a0, f0)) => {
                        let renamed = (
                            var_of[v0 as usize].expect("a fact has a variable"),
                            var_of[v as usize].expect("a fact has a variable"),
                        );
                        reused = backend.embed_renamed(f0, scope_of(a0), scope, renamed)?;
                    }
                    None => {
                        equivalents.insert(k, (v, a));
                    }
                }
            }
            let selected = match (&reused, equal) {
                (None, Some((v, k))) => {
                    // The stored formula is `v ∧ X ∨ ¬v ∧ Y`, with X and Y the
                    // formulas of ¬k and k in the dual, of k and ¬k in the
                    // primal.
                    let (pos, neg) = (GLit::Pos(k), GLit::Neg(k));
                    // Read for no body: where the selection does not apply,
                    // the rules read `k` again.
                    let (x, y) = match formulation {
                        Formulation::Dual => {
                            (derived(neg, 0, formulas)?, derived(pos, 0, formulas)?)
                        }
                        Formulation::Primal => {
                            (derived(pos, 0, formulas)?, derived(neg, 0, formulas)?)
                        }
                    };
                    let own = scope_of(k);
                    let (x, y) = (backend.embed(x, own, scope)?, backend.embed(y, own, scope)?);
                    backend.select_at_root(
                        scope,
                        var_of[v as usize].expect("a fact has a variable"),
                        &x,
                        &y,
                    )?
                }
                _ => None,
            };
            let f = match reused.or(selected) {
                // Renamed from a settled formula, or interned level by level
                // from settled operands: settled already.
                Some(f) => f,
                None => {
                    let conjoin = !links[a as usize] && conjoined(gp, &by_head[a as usize]);
                    let (mut f, _) = apply(a, formulas, false, conjoin)?;
                    backend.minimize(&mut f, Reduce::Settled)?;
                    positive.borrow_mut()[a as usize] |= conjoin;
                    f
                }
            };
            stats.atom_updates += 1;
            let pairs = backend.size(&f);
            stats.max_pairs = stats.max_pairs.max(pairs);
            let (f, pairs) = watched_sized(f, pairs, a)?;
            store(a, f, pairs, formulas);
            return Ok(true);
        }
        if self.fixpoint.worklist.is_none() {
            self.fixpoint.worklist = Some(Worklist::new(gp, by_head, scc));
            stats.cyclic_components += 1;
            for &a in scc {
                let f = bottom();
                let size = backend.size(&f);
                store(a, f, size, formulas);
            }
        }
        let Worklist {
            dirty,
            next,
            rank,
            dependents,
            iterations,
            midway,
        } = self.fixpoint.worklist.as_mut().expect("set above");
        // A round that a stop interrupted goes on where it stopped: an atom
        // leaves `dirty` only once its update has landed, and a failed
        // update marks its rules unfolded again, so the formulas, versions
        // and worklist stay those of a round part done.
        if !*midway {
            *iterations += 1;
            *midway = true;
        }
        while let Some(&(r, a)) = dirty.first() {
            let folded: Vec<u64> = by_head[a as usize]
                .iter()
                .map(|&ri| seen.borrow()[ri])
                .collect();
            let unfold = |e: CompileError| {
                let mut seen = seen.borrow_mut();
                for (&ri, &v) in by_head[a as usize].iter().zip(&folded) {
                    seen[ri] = v;
                }
                e
            };
            let (mut f, other) = apply(a, formulas, true, false).map_err(unfold)?;
            stats.atom_updates += 1;
            let old = formulas[a as usize].as_ref().unwrap();
            // The stored formula is the settled one, and settling this
            // one here rather than after the test makes both canonical:
            // for a fixed vtree a minimized diagram is canonical
            // (tididi's `lib.rs`), so two of them are equivalent only if
            // they have the same pair count. The pair count is the cheap
            // half of the test; how often it answers on its own is the
            // gap between `convergence_tests` and the equivalence count.
            // The settle is not extra work: on a round that changes the
            // formula it is the settle that was below, and on one that
            // does not it is the minimization `Engine::equivalent` would
            // have done on its own clone.
            backend.minimize(&mut f, Reduce::Settled).map_err(unfold)?;
            let pairs = backend.size(&f);
            let sized = pairs == backend.size(old);
            let same = sized && backend.equivalent(&f, old).map_err(unfold)?;
            // The count is of the diagram comparisons the size test let
            // through.
            count(Op::Equivalent, usize::from(sized));
            stats.convergence_tests += 1;
            // The cache holds the stored formula's other polarity: the one
            // the update built, which is the old formula's too when the two
            // are the same, and otherwise nothing once the formula changed.
            if other.is_some() || !same {
                lambda_cache.borrow_mut()[a as usize] = other;
            }
            if !same {
                stats.max_pairs = stats.max_pairs.max(pairs);
                let (f, pairs) = watched_sized(f, pairs, a).map_err(unfold)?;
                store(a, f, pairs, formulas);
                version.borrow_mut()[a as usize] += 1;
                if let Some(ds) = dependents.get(&a) {
                    for &d in ds {
                        let rd = rank[&d];
                        if rd > r {
                            dirty.insert((rd, d));
                        } else {
                            next.insert((rd, d));
                        }
                    }
                }
            }
            dirty.remove(&(r, a));
        }
        *dirty = std::mem::take(next);
        *midway = false;
        if !dirty.is_empty() {
            return Ok(false);
        }
        let iterations = *iterations;
        self.fixpoint.worklist = None;
        stats.iterations += iterations;
        stats.max_iterations = stats.max_iterations.max(iterations);
        Ok(true)
    }

    /// The search of [`Compilation::dynamic`], if it is due.
    fn search(&mut self, backend: &TddBackend) -> Result<(), CompileError> {
        let Some(dynamic) = self.dynamic.as_mut() else {
            return Ok(());
        };
        let now = backend.work_units();
        dynamic.since.get_or_insert(now);
        if !dynamic.due(now, self.fixpoint.live) {
            return Ok(());
        }
        // The cached polarities are dropped rather than moved: they would
        // double what the search sizes and rotates, and change the vtree
        // it finds for the formulas themselves.
        self.fixpoint.lambda_cache.borrow_mut().clear();
        // A loop the watch refused is not among the formulas it moves.
        self.memo.get_mut().forget_refused();
        let mut members = self.fixpoint.formulas.formulas_mut();
        let budget = dynamic.budget(now, self.next, self.sccs.len() - self.next);
        let found = backend.restructure(&mut members);
        drop(members);
        // A search that fails part way has still moved every formula onto
        // the vtree the backend now holds.
        let global = backend.global();
        for scope in &mut self.scopes {
            *scope = global.clone();
        }
        for (a, f) in self.fixpoint.formulas.iter().enumerate() {
            self.fixpoint.sizes[a] = f.as_ref().map_or(0, |f| backend.size(f));
        }
        self.fixpoint.live = self.fixpoint.sizes.iter().sum();
        let found = found?;
        let weak = found.pairs_after * 10 > found.pairs_before * 9;
        dynamic.debt = found.work_units.saturating_sub(budget);
        dynamic.next_at = (2 * self.fixpoint.live).max(dynamic.first_pairs);
        dynamic.since = Some(backend.work_units());
        dynamic.resumed = false;
        let stats = self.stats.search.get_or_insert_with(SearchStats::default);
        stats.searches += 1;
        stats.weak += usize::from(weak);
        stats.probes += found.probes;
        stats.accepts += found.accepts;
        stats.pairs_before += found.pairs_before;
        stats.pairs_after += found.pairs_after;
        stats.work_units += found.work_units;
        Ok(())
    }
}

/// Live pairs the atoms' formulas reach before the first search of the
/// `dynamic` strategies; CUDD's first reordering is at 4004 nodes.
pub(crate) const DYNAMIC_FIRST_PAIRS: usize = 4096;

/// The state of [`Compilation::dynamic`]'s search between steps.
struct Dynamic {
    /// Live pairs below which no search runs.
    first_pairs: usize,
    /// Live pairs at which the next search runs: twice what the last one
    /// left.
    next_at: usize,
    /// The work clock when the last search ended, or when the compile
    /// first stepped; a search may spend what the compile spent since.
    since: Option<u64>,
    /// Whether this run resumes a stopped compile and has not searched
    /// yet. Each run is on a fresh work clock, so `since` is from an
    /// earlier one; its first search has the budget the run's limits
    /// allow and owes nothing (a budget of the work since, summed over the
    /// runs, left g30_100 unfinished).
    resumed: bool,
    /// The work the last search spent past its budget, which the compile
    /// spends before the next search.
    debt: u64,
}

impl Dynamic {
    /// Whether a search is due at work clock `now` with the formulas at
    /// `live` pairs: once they have doubled since the last, and once the
    /// compile has spent, since the last search ended, what that search
    /// spent past its budget; at once in a run that resumes a stopped
    /// compile, whose clock starts afresh.
    fn due(&self, now: u64, live: usize) -> bool {
        live >= self.next_at && (self.resumed || now - self.since.unwrap_or(now) >= self.debt)
    }

    /// The work the search due at work clock `now` spends before it owes
    /// the excess, with `done` components finished and `left` to go: what
    /// the compile spent since the last search, and no more than the
    /// compile is likely to spend after it, its work so far per finished
    /// component times the components left. A search pays only over the
    /// steps after it, so one near the end owes more and holds off the
    /// next for longer: on g30_100 a search two components before the end
    /// spent several times what the rest of the compile took.
    fn budget(&self, now: u64, done: usize, left: usize) -> u64 {
        match self.since {
            Some(since) if !self.resumed => {
                (now - since).min(now.saturating_mul(left as u64) / done.max(1) as u64)
            }
            _ => u64::MAX,
        }
    }
}

/// The worklist of a cyclic component, kept between its rounds. A round
/// takes its atoms in rank order. A reader ranked after the atom that
/// changed is still ahead in this round and joins it; one ranked before it,
/// or the atom itself, waits for the next round.
struct Worklist {
    /// The atoms left in this round, and those dirtied for the next.
    dirty: std::collections::BTreeSet<(usize, AtomId)>,
    next: std::collections::BTreeSet<(usize, AtomId)>,
    rank: FxHashMap<AtomId, usize>,
    dependents: FxHashMap<AtomId, Vec<AtomId>>,
    /// Rounds so far.
    iterations: usize,
    /// Whether a round is under way: one a stop interrupted.
    midway: bool,
}

impl Worklist {
    fn new(gp: &GroundProgram, by_head: &RulesByHead, scc: &[AtomId]) -> Self {
        // Worklist: an atom is recomputed only when a body atom of one of its
        // rules changed since its last update. Round-based so `iterations`
        // keeps its meaning (a round processes the atoms marked dirty by the
        // previous round; the first round processes the whole component).
        let in_scc: FxHashSet<AtomId> = scc.iter().copied().collect();
        let mut dependents: FxHashMap<AtomId, Vec<AtomId>> = FxHashMap::default();
        for &a in scc {
            for &ri in &by_head[a as usize] {
                for l in &gp.rules[ri].body {
                    if in_scc.contains(&l.atom()) {
                        dependents.entry(l.atom()).or_default().push(a);
                    }
                }
            }
        }
        for v in dependents.values_mut() {
            v.sort_unstable();
            v.dedup();
        }
        // A round applies the operator to the atoms in this order, and an
        // update is visible to the atoms that come after it in the same
        // round, so the order decides how far a change travels per round.
        // Reverse postorder of the dependency graph puts an atom before the
        // atoms that read it wherever the cycle allows.
        let rank = {
            let mut seen: FxHashSet<AtomId> = FxHashSet::default();
            let mut post: Vec<AtomId> = Vec::with_capacity(scc.len());
            let mut stack: Vec<(AtomId, usize)> = Vec::new();
            for &root in scc {
                if !seen.insert(root) {
                    continue;
                }
                stack.push((root, 0));
                while let Some((a, i)) = stack.pop() {
                    let reads = dependents.get(&a).map_or(&[][..], Vec::as_slice);
                    match reads.get(i) {
                        Some(&b) => {
                            stack.push((a, i + 1));
                            if seen.insert(b) {
                                stack.push((b, 0));
                            }
                        }
                        None => post.push(a),
                    }
                }
            }
            let mut rank: FxHashMap<AtomId, usize> = FxHashMap::default();
            for (r, &a) in post.iter().rev().enumerate() {
                rank.insert(a, r);
            }
            rank
        };
        let dirty = scc.iter().map(|&a| (rank[&a], a)).collect();
        Worklist {
            dirty,
            next: std::collections::BTreeSet::new(),
            rank,
            dependents,
            iterations: 0,
            midway: false,
        }
    }
}

#[cfg(test)]
#[path = "tests/dynamic.rs"]
mod dynamic_tests;

#[cfg(test)]
#[path = "tests/resume.rs"]
mod resume_tests;

#[cfg(test)]
#[path = "tests/dying.rs"]
mod dying_tests;
