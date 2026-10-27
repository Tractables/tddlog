//! Whether the names of `program` and `decision` pay, settled on the
//! compiled program ([`crate::vtree`]'s naming).
//!
//! `program` names the derived atoms that cross each narrow cut of its
//! order, and the compile builds every formula over its own segment and
//! the copies at its start. `decision` names the atoms of its tree that
//! atoms outside the tree read, a cut for each node of the tree they sit
//! above, and the contexts of the tree, which are in no cut and keep their
//! names (`vtree::decision`). What a cut's names cost and save shows only
//! in the diagrams. Named, the evidence holds the joint of the cut's links:
//! `J`, their conjunction. Expanded, each formula that reads the copies is
//! composed with the links of the copies it reads, the copies summed out
//! (`∃x (G ∧ (x ⇔ F)) = G[x := F]`), and the links leave the evidence.
//! A link is held as the evidence observes it: true for a copy's own
//! link, false for a clause table's, which ties the copies of one reader
//! together, and whose copies are summed out together.
//!
//! A link may read copies of its own cut: a split table's copy is tied to
//! the copy of the part before, and an atom that reads a value of a
//! clause table reads the table's first reader's copy, which may be one
//! of the cut's ([`crate::indicators`]). A reader is composed with the
//! links of the copies it reads, the links of the copies those read, and
//! so on, each before the links that tie a copy it reads, so that no
//! composition brings back a copy that an earlier one summed out. Where a
//! cut's links read a later cut's copies (an atom stated after one of its
//! readers), the readers composed with them read those copies from then
//! on, and that cut composes them too. A cut whose link a tie stands on
//! (`tp::tie`) keeps its names: the tie reads the link's copies in the
//! evidence, and it is no formula that a cut could compose.
//!
//! A cut's readers are the formulas that depend on its copies (their
//! diagrams' support, not their scopes). The readers that are not queries
//! grow from `R` pairs to `K`, and the names pay for them where
//! `J < K - R`. On a Markov chain each day's readers are the next day's
//! links, and composed each carries the joint of two days (GeNI's weather);
//! where the readers compress what they read, as a coupon collector's
//! counts or the bits of a sum do, the cut is expanded. The joint is
//! weighed first, up to [`GROWTH`] times the links' pairs `L`; where it is
//! that small, the readers are composed only until they gain more than it,
//! and a cut whose readers do keeps its names. Otherwise the readers are
//! composed until they gain [`GROWTH`] times the pairs of the links and
//! readers, and the joint then up to their gain, so that a cut costs about
//! as many pairs as its smaller side; a cut whose readers outgrow that
//! keeps its names. A cut whose names pay for those readers keeps them, and
//! the other cuts' readers are composed. The joint is built on the scope
//! over the cut's links, where the evidence holds it. On `decision`'s tree
//! the names pay for coinBias's comparators, which read every bit of the
//! bias the tree selects and grow without bound composed; where the readers
//! combine the selected values bit by bit (Alea.jl's caesar, the genes
//! programs), a node's joint holds every combination of its values, and
//! most nodes' cuts are expanded.
//!
//! A query is counted by a walk against the evidence, which computes
//! every pair of the two diagrams' nodes at each level (`eval::count`'s
//! conjunction count). With the names, the queries that read a cut's
//! copies, `R` pairs, are walked against the evidence with the joint in
//! it, `E + J`; expanded, they carry about the links' `L` pairs more,
//! walked against `E` alone, and the composition, a conjunction of
//! each query with the links it reads, costs less than either walk. So
//! their names pay where `R·(E + J) < (R + L)·E`, that is where
//! `J < L·E/R`: a cut read by small queries in a large evidence keeps
//! its names (GeNI's link grids), and one read by large queries in a
//! small evidence is expanded (the bits of a sum read by its output
//! bits), as is every cut whose queries leave no other evidence. The
//! queries are settled after all the cuts' other readers, against the
//! evidence those left.
//!
//! An expanded cut's links leave the evidence, and its copies, which no
//! formula reads any more, weigh one half on each side, so that a count
//! sums them to one as it does a fact. The cuts are settled in the
//! order's direction, so that a cut's readers include the next cut's
//! links as composed, and `decision`'s up its tree.
//! Settling spends at most the compile's work units;
//! the cuts it has not reached by then keep their names, as does a cut a
//! query reads as a literal of the evidence. Expanding a cut whose joint
//! holds more than [`GROWTH`] times its links' pairs is not charged:
//! kept, the cut's names would hold that joint in the evidence, and the
//! work composes formulas the compile would have built without them.
//! Charged, the genes programs' cuts spent the budget before settling
//! reached the cuts at the root of `decision`'s tree, whose links hold
//! every combination of the values below them, and genes5_w21's evidence
//! held 94 M pairs of their joints (93 k with them expanded). A cut whose
//! joint is small is charged, so that settling does not spend the square
//! of a chain's steps composing its cuts of width one, and so is a cut
//! whose readers outgrow their bound and keep their names (coinBias's
//! comparators). Under `auto` its work within the budget is not
//! charged to the trial's turn: the run's stop moves later by what it
//! spends ([`TddBackend::stop_later`], `Evaluation::run`), so that
//! settling never takes the trial into another round, where every other
//! trial would spend a turn too (GeNI's weather, whose cuts all keep
//! their names). What the cuts past the budget spend reading their
//! readers is the trial's ([`Settled::free`]): uncharged, a stop it
//! tripped read as one by size, and the trial's next attempt, with the
//! same work and twice the watch, settled a few cuts more each time
//! (diamond_loop, 100 steps).

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use tididi::Tdd;
use tididi::vtree::VarId;

use super::{
    CompileError, Compiled, Formula, NegationSource, Negations, Projection, TddBackend, VtreeScope,
};
use crate::ground::{AtomId, GroundProgram, Indicator};

/// How many times the pairs of a cut's links and readers that are not
/// queries the readers may gain composed before the cut keeps its names.
const GROWTH: usize = 4;

/// What settling did: the cuts that kept their names and those expanded,
/// the negations it took, its work units and those of them its budget
/// does not charge (module doc); and where a settling that a
/// stop ended goes on: the first segment whose readers that are not
/// queries it has not settled, the segments whose queries are left, and
/// how many of those it has settled.
#[derive(Clone, Debug, Default)]
pub(crate) struct Settled {
    pub(crate) kept: usize,
    pub(crate) expanded: usize,
    pub(crate) negations: Negations,
    pub(crate) work: u64,
    uncharged: u64,
    next: usize,
    asked: Vec<usize>,
    next_asked: usize,
}

impl Settled {
    /// The work of every call that no turn of `auto` is charged for, which
    /// the run's stop moved later by (module doc): all of it up to
    /// `compile_work`, the budget, and the work the budget does not charge.
    /// Past the budget each cut still reads its readers before it keeps its
    /// names, and the stop has not moved for that: it is the trial's work,
    /// and a stop it trips is one by work.
    pub(crate) fn free(&self, compile_work: u64) -> u64 {
        self.work.min(compile_work.saturating_add(self.uncharged))
    }
}

/// Readers composed with a cut's links, each with its formula.
type Composed = Vec<(AtomId, Formula)>;

/// A reader of a cut's copies and the copies' variables it reads, sorted.
type Reader = (AtomId, Vec<VarId>);

/// The conjunction of a group of links ([`groups`]), and the variables of
/// the copies they tie.
type Group = (Tdd, Vec<VarId>);

/// A cut whose names can be expanded: its links, each once with the value
/// the evidence observes it at, and its copies' variables, each with the
/// link that ties it (an index into `links`). The copies of one clause
/// table's reader share one link.
struct Cut {
    links: Vec<(AtomId, bool)>,
    vars: Vec<(VarId, usize)>,
}

/// Settle the names of `gp`, compiled as `compiled`, cut by cut (module
/// doc), recording what it did in `compiled.for_queries`. An operation
/// the engine refuses or a watch stops returns its error, the cut it was
/// settling left as it was; a later call goes on from that cut.
pub(crate) fn settle(
    gp: &GroundProgram,
    backend: &TddBackend,
    compiled: &mut Compiled,
) -> Result<(), CompileError> {
    let encoding = &gp.encoding;
    if encoding.segments.is_empty() || compiled.projection == Projection::Off {
        return Ok(());
    }
    let start = backend.work_units();
    let mut settled = std::mem::take(&mut compiled.for_queries.settled);
    let charged = settled.work - settled.uncharged;
    let mut budget = Budget::new(backend, compiled.stats.work_units.saturating_sub(charged));
    let out = settle_from(gp, backend, compiled, &mut settled, &mut budget);
    settled.work += backend.work_units() - start;
    compiled.for_queries.settled = settled;
    out
}

/// Settling's budget: the work clock reading it ends at, which the run's
/// stop moves later to, since the run does not count it (module doc).
struct Budget<'b, 'a> {
    backend: &'b TddBackend<'a>,
    start: u64,
    until: u64,
    /// The run's stop moved later by the budget, for as long as the scope
    /// lives ([`TddBackend::stop_later`]).
    moved_stop: Option<tididi::limits::LimitScope<'b>>,
}

impl<'b, 'a> Budget<'b, 'a> {
    fn new(backend: &'b TddBackend<'a>, work: u64) -> Self {
        let start = backend.work_units();
        Budget {
            backend,
            start,
            until: start + work,
            moved_stop: backend.stop_later(work),
        }
    }

    /// The work left.
    fn left(&self) -> u64 {
        self.until.saturating_sub(self.backend.work_units())
    }

    /// `work` spent and not charged: the end and the run's stop move later
    /// by as much.
    fn uncharge(&mut self, work: u64) {
        self.until += work;
        // Dropped first, the old scope gives the run's stop back, which the
        // new one moves by all the budget so far.
        self.moved_stop = None;
        self.moved_stop = self.backend.stop_later(self.until - self.start);
    }
}

/// [`settle`] within `budget`, from where `settled` left off.
fn settle_from(
    gp: &GroundProgram,
    backend: &TddBackend,
    compiled: &mut Compiled,
    settled: &mut Settled,
    budget: &mut Budget,
) -> Result<(), CompileError> {
    let encoding = &gp.encoding;
    // The copies of each segment, and the segment of each copy's variable.
    let mut segments: Vec<Vec<usize>> = Vec::new();
    let mut segment_of: FxHashMap<VarId, usize> = FxHashMap::default();
    for (i, (ind, &s)) in encoding
        .indicators
        .iter()
        .zip(&encoding.segments)
        .enumerate()
    {
        // A copy no segment reads (a context of `decision`'s tree).
        if s == usize::MAX {
            continue;
        }
        if s >= segments.len() {
            segments.resize_with(s + 1, Vec::new);
        }
        segments[s].push(i);
        if let Some(v) = compiled.var_of[ind.var as usize] {
            segment_of.insert(v, s);
        }
    }
    // The formulas that read each segment's copies: those whose scope holds
    // one. A formula composed below with links that read a later segment's
    // copies joins that segment's list ([`replace`]), so the lists stay
    // right as the cuts are settled in order.
    let mut readers: Vec<Vec<AtomId>> = vec![Vec::new(); segments.len()];
    for a in 0..gp.num_atoms() {
        if compiled.formulas[a].is_none() {
            continue;
        }
        let Some(vars) = compiled.scope_of(a as AtomId).vars() else {
            // A formula on the whole program's scope may read every copy.
            readers.iter_mut().for_each(|r| r.push(a as AtomId));
            continue;
        };
        let mut seen: Vec<usize> = vars
            .iter()
            .filter_map(|v| segment_of.get(v).copied())
            .collect();
        seen.sort_unstable();
        seen.dedup();
        for s in seen {
            readers[s].push(a as AtomId);
        }
    }
    let mut supports = Supports::default();
    // The queries' atoms, and the copies of the queried atoms.
    let asks: FxHashSet<AtomId> = gp.queries.iter().filter_map(|(_, a, _)| *a).collect();
    let queried: FxHashSet<AtomId> = {
        let first = encoding.first_copies();
        asks.iter().filter_map(|a| first.get(a).copied()).collect()
    };
    // Each cut whose links have formulas of their own and are observed (a
    // tie's are not built), and that no tie stands on (module doc).
    let observed: FxHashMap<AtomId, bool> = gp
        .evidence
        .iter()
        .filter_map(|(_, a, value)| a.map(|a| (a, *value)))
        .collect();
    let ties = &compiled.for_queries.ties;
    let stood_on: FxHashSet<AtomId> = ties.values().map(|tie| tie.reference).collect();
    let cuts: Vec<Option<Cut>> = segments
        .iter()
        .map(|copies| {
            let mut cut = Cut {
                links: Vec::new(),
                vars: Vec::new(),
            };
            for &i in copies {
                let Indicator { var, link, .. } = encoding.indicators[i];
                let settles = compiled.formulas[link as usize].is_some()
                    && !ties.contains_key(&link)
                    && !stood_on.contains(&link);
                let value = observed.get(&link).copied().filter(|_| settles)?;
                let at = cut
                    .links
                    .iter()
                    .position(|&(l, _)| l == link)
                    .unwrap_or_else(|| {
                        cut.links.push((link, value));
                        cut.links.len() - 1
                    });
                if let Some(v) = compiled.var_of[var as usize] {
                    cut.vars.push((v, at));
                }
            }
            Some(cut)
        })
        .collect();
    // The readers that are not queries, cut by cut.
    while settled.next < segments.len() {
        let s = settled.next;
        let copies = &segments[s];
        match &cuts[s] {
            // No cut ends here.
            _ if copies.is_empty() => {}
            Some(cut)
                if !copies
                    .iter()
                    .any(|&i| queried.contains(&encoding.indicators[i].var)) =>
            {
                readers[s].sort_unstable();
                readers[s].dedup();
                let (others, asked) =
                    reading(backend, compiled, &readers[s], cut, &asks, &mut supports)?;
                let start = backend.work_units();
                match others_composed(
                    backend,
                    compiled,
                    cut,
                    &others,
                    budget.left(),
                    &mut settled.negations,
                )? {
                    None => settled.kept += 1,
                    Some((composed, large)) => {
                        if large {
                            let work = backend.work_units() - start;
                            budget.uncharge(work);
                            settled.uncharged += work;
                        }
                        replace(compiled, composed, (&mut readers, &segment_of, s));
                        if asked.is_empty() {
                            expand(compiled, cut);
                            settled.expanded += 1;
                        } else {
                            settled.asked.push(s);
                        }
                    }
                }
            }
            _ => settled.kept += 1,
        }
        settled.next = s + 1;
    }
    // The queries, against the evidence the other readers left.
    while settled.next_asked < settled.asked.len() {
        let s = settled.asked[settled.next_asked];
        let cut = cuts[s]
            .as_ref()
            .expect("a cut whose other readers were composed");
        readers[s].sort_unstable();
        readers[s].dedup();
        let (_, asked) = reading(backend, compiled, &readers[s], cut, &asks, &mut supports)?;
        let pairs = |a: AtomId| {
            compiled.formulas[a as usize]
                .as_ref()
                .map_or(1, |f| backend.size(f))
        };
        let own: FxHashSet<AtomId> = cut.links.iter().map(|&(l, _)| l).collect();
        let evidence: usize = gp
            .evidence
            .iter()
            .filter_map(|(_, a, _)| *a)
            .filter(|a| !own.contains(a) && !compiled.for_queries.dropped.contains(a))
            .map(pairs)
            .sum();
        match asked_composed(
            backend,
            compiled,
            cut,
            &asked,
            evidence,
            budget.left(),
            &mut settled.negations,
        )? {
            None => settled.kept += 1,
            Some(composed) => {
                replace(compiled, composed, (&mut readers, &segment_of, s));
                expand(compiled, cut);
                settled.expanded += 1;
            }
        }
        settled.next_asked += 1;
    }
    Ok(())
}

/// Store `composed`, each reader's formula and the scope it lives on, the
/// readers composed at segment `s`. A reader composed with links that read
/// a later segment's copies reads them now, and joins that segment's
/// `readers` (`segment_of` gives each copy's), which a segment's settling
/// sorts and rids of the repeats.
fn replace(
    compiled: &mut Compiled,
    composed: Composed,
    (readers, segment_of, s): (&mut [Vec<AtomId>], &FxHashMap<VarId, usize>, usize),
) {
    for (r, (f, scope)) in composed {
        match scope.vars() {
            None => readers[s + 1..].iter_mut().for_each(|later| later.push(r)),
            Some(vars) => {
                let mut later: Vec<usize> = vars
                    .iter()
                    .filter_map(|v| segment_of.get(v).copied())
                    .filter(|&t| t > s)
                    .collect();
                later.sort_unstable();
                later.dedup();
                later.into_iter().for_each(|t| readers[t].push(r));
            }
        }
        compiled.formulas[r as usize] = Some(Arc::new(f));
        compiled.scopes.push(scope);
        compiled.comp_of[r as usize] = (compiled.scopes.len() - 1) as u32;
    }
}

/// Take `cut`'s links out of the evidence, its copies summed to one.
fn expand(compiled: &mut Compiled, cut: &Cut) {
    for &(l, _) in &cut.links {
        compiled.formulas[l as usize] = None;
        compiled.for_queries.dropped.insert(l);
    }
    compiled
        .for_queries
        .released
        .extend(cut.vars.iter().map(|&(v, _)| v));
}

/// A cut's links as the evidence observes them, each on its own scope;
/// the link that ties each of the cut's copies' variables; and of each
/// link, the others that tie a copy it reads, and its place in an order
/// where it comes before them.
struct Links<'a> {
    cut: &'a Cut,
    held: Vec<Tdd>,
    of: FxHashMap<VarId, usize>,
    reads: Vec<Vec<usize>>,
    rank: Vec<usize>,
}

impl<'a> Links<'a> {
    /// The copies a link reads are those its scope holds, or where it lives
    /// on the whole program's scope, which says nothing of what it reads,
    /// those its formula depends on. `None` where the links read each
    /// other's copies in a cycle, which no order of composition sums out.
    fn new(
        backend: &TddBackend,
        compiled: &Compiled,
        cut: &'a Cut,
        negations: &mut Negations,
    ) -> Result<Option<Self>, CompileError> {
        let held = cut
            .links
            .iter()
            .map(|&(l, value)| {
                compiled.literal_formula(
                    backend,
                    compiled.scope_of(l),
                    l,
                    value,
                    (&mut *negations, NegationSource::Evidence),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let of: FxHashMap<VarId, usize> = cut.vars.iter().copied().collect();
        let reads = cut
            .links
            .iter()
            .zip(&held)
            .enumerate()
            .map(|(i, (&(l, _), link))| {
                let support;
                let vars = match compiled.scope_of(l).vars() {
                    Some(vars) => vars,
                    None => {
                        support = link.support()?;
                        &support[..]
                    }
                };
                let mut reads: Vec<usize> = vars
                    .iter()
                    .filter_map(|v| of.get(v).copied())
                    .filter(|&j| j != i)
                    .collect();
                reads.sort_unstable();
                reads.dedup();
                Ok(reads)
            })
            .collect::<Result<Vec<_>, CompileError>>()?;
        let Some(rank) = order(&reads) else {
            return Ok(None);
        };
        Ok(Some(Links {
            cut,
            held,
            of,
            reads,
            rank,
        }))
    }

    /// The `i`th link as observed, on `scope`.
    fn on(
        &self,
        backend: &TddBackend,
        compiled: &Compiled,
        i: usize,
        scope: &VtreeScope,
    ) -> Result<Tdd, CompileError> {
        backend.embed_ref(&self.held[i], compiled.scope_of(self.cut.links[i].0), scope)
    }

    /// The links that tie the copies `reads`, and those that tie the copies
    /// they read in turn, in the order a reader is composed with them: a
    /// link before those that tie a copy it reads, so that no composition
    /// brings back a copy an earlier one summed out.
    fn tying(&self, reads: &[VarId]) -> Vec<usize> {
        let mut tying: Vec<usize> = reads.iter().map(|v| self.of[v]).collect();
        tying.sort_unstable();
        tying.dedup();
        let mut i = 0;
        while i < tying.len() {
            for &j in &self.reads[tying[i]] {
                if !tying.contains(&j) {
                    tying.push(j);
                }
            }
            i += 1;
        }
        tying.sort_unstable_by_key(|&i| self.rank[i]);
        tying
    }
}

/// Each link's place in an order where it comes before the links that tie
/// a copy it reads (`reads`), and otherwise in the cut's order; `None`
/// where the links read each other's copies in a cycle.
fn order(reads: &[Vec<usize>]) -> Option<Vec<usize>> {
    // Of each link, how many links not yet placed read its copies.
    let mut readers = vec![0usize; reads.len()];
    for &j in reads.iter().flatten() {
        readers[j] += 1;
    }
    let mut ready: BinaryHeap<Reverse<usize>> = (0..reads.len())
        .filter(|&i| readers[i] == 0)
        .map(Reverse)
        .collect();
    let mut rank = vec![0; reads.len()];
    let mut placed = 0;
    while let Some(Reverse(i)) = ready.pop() {
        rank[i] = placed;
        placed += 1;
        for &j in &reads[i] {
            readers[j] -= 1;
            if readers[j] == 0 {
                ready.push(Reverse(j));
            }
        }
    }
    (placed == reads.len()).then_some(rank)
}

/// The pairs of the formula of `a`.
fn pairs(backend: &TddBackend, compiled: &Compiled, a: AtomId) -> usize {
    backend.size(
        compiled.formulas[a as usize]
            .as_ref()
            .expect("a link or reader has a formula"),
    )
}

/// `others`, the readers of `cut` that are not queries, composed with its
/// links where they gain no more than the links' joint (`K - R <= J`),
/// and whether that joint is larger than [`GROWTH`] times the links;
/// `None` where the names pay for them.
fn others_composed(
    backend: &TddBackend,
    compiled: &Compiled,
    cut: &Cut,
    others: &[Reader],
    work: u64,
    negations: &mut Negations,
) -> Result<Option<(Composed, bool)>, CompileError> {
    if others.is_empty() {
        return Ok(Some((Vec::new(), false)));
    }
    let held: usize = others.iter().map(|r| pairs(backend, compiled, r.0)).sum();
    let l = cut
        .links
        .iter()
        .map(|&(l, _)| pairs(backend, compiled, l))
        .sum::<usize>();
    let settled = backend.within(work, || {
        let Some(links) = Links::new(backend, compiled, cut, negations)? else {
            return Ok(None);
        };
        let small = joint(backend, compiled, &links, GROWTH * l)?;
        let bound = small.unwrap_or(GROWTH * (l + held));
        let Some(composed) = composed_under(backend, compiled, &links, others, bound)? else {
            return Ok(None);
        };
        let gain = composed
            .iter()
            .map(|(_, f)| backend.size(&f.0))
            .sum::<usize>()
            .saturating_sub(held);
        if small.is_none()
            && gain > GROWTH * l
            && joint(backend, compiled, &links, gain - 1)?.is_some()
        {
            return Ok(None);
        }
        Ok(Some((composed, small.is_none())))
    })?;
    Ok(settled.flatten())
}

/// `asked`, the queries that read `cut`'s copies, composed with its links
/// where the links' joint is at least `L·E/R`, `L` the links' pairs, `R`
/// the queries' and `E` those of `evidence`, the rest of the evidence;
/// `None` where the names pay for them.
fn asked_composed(
    backend: &TddBackend,
    compiled: &Compiled,
    cut: &Cut,
    asked: &[Reader],
    evidence: usize,
    work: u64,
    negations: &mut Negations,
) -> Result<Option<Composed>, CompileError> {
    let held: usize = asked.iter().map(|r| pairs(backend, compiled, r.0)).sum();
    let l = cut
        .links
        .iter()
        .map(|&(l, _)| pairs(backend, compiled, l))
        .sum::<usize>();
    let bound =
        (l as u128 * evidence as u128 / held.max(1) as u128).min(usize::MAX as u128) as usize;
    let settled = backend.within(work, || {
        let Some(links) = Links::new(backend, compiled, cut, negations)? else {
            return Ok(None);
        };
        if joint(backend, compiled, &links, bound)?.is_some() {
            return Ok(None);
        }
        composed_under(backend, compiled, &links, asked, usize::MAX)
    })?;
    Ok(settled.flatten())
}

/// Each reader's support, sorted, computed once for each formula it has:
/// a formula whose scope holds every segment's copies is a reader of
/// each, and computed again for each segment, the supports of
/// conjugate_ex3's 20 such readers took about 15 G of settling's 16.6 G
/// work units at width 18 and spent its budget. Keyed by the reader and
/// its scope's index, which [`replace`] gives anew to each formula it
/// stores, so that a composed reader's support is its new formula's.
type Supports = FxHashMap<(AtomId, u32), Vec<VarId>>;

/// Of `candidates`, the formulas that read a copy of `cut`, with the
/// copies they depend on: those that are not queries, and the queries.
fn reading(
    backend: &TddBackend,
    compiled: &Compiled,
    candidates: &[AtomId],
    cut: &Cut,
    asks: &FxHashSet<AtomId>,
    supports: &mut Supports,
) -> Result<(Vec<Reader>, Vec<Reader>), CompileError> {
    let (mut others, mut asked) = (Vec::new(), Vec::new());
    for &r in candidates {
        if cut.links.iter().any(|&(l, _)| l == r) {
            continue;
        }
        let Some(f) = compiled.formulas[r as usize].as_ref() else {
            continue;
        };
        let support = match supports.entry((r, compiled.comp_of[r as usize])) {
            Entry::Occupied(known) => known.into_mut(),
            Entry::Vacant(new) => {
                let scope = compiled.scope_of(r);
                let mut vars: Vec<VarId> = backend
                    .engine
                    .support(f)?
                    .into_iter()
                    .map(|local| match scope.vars() {
                        Some(vars) => vars[local.0 as usize - 1],
                        None => local,
                    })
                    .collect();
                vars.sort_unstable();
                new.insert(vars)
            }
        };
        let reads: Vec<VarId> = cut
            .vars
            .iter()
            .map(|&(v, _)| v)
            .filter(|v| support.binary_search(v).is_ok())
            .collect();
        if !reads.is_empty() {
            if asks.contains(&r) {
                &mut asked
            } else {
                &mut others
            }
            .push((r, reads));
        }
    }
    Ok((others, asked))
}

/// The scope over the variables of `atoms`' formulas, the whole program's
/// where one of them lives on it.
fn over(
    backend: &TddBackend,
    compiled: &Compiled,
    atoms: impl Iterator<Item = AtomId>,
) -> Result<VtreeScope, CompileError> {
    over_scopes(backend, atoms.map(|a| compiled.scope_of(a)))
}

/// The pairs of the conjunction of the links, or `None` once a
/// conjunction on the way exceeds `bound`. The links are conjoined one
/// after another on the scope over all of them, where the evidence holds
/// them: built in pairs, each on its two operands' scope, the conjunctions
/// on the way held fewer pairs and a cut's large joint took longer to
/// pass the bound (caesar_huge's 1551 links, 287 M units of settling
/// against 123 M).
fn joint(
    backend: &TddBackend,
    compiled: &Compiled,
    links: &Links,
    bound: usize,
) -> Result<Option<usize>, CompileError> {
    let scope = over(backend, compiled, links.cut.links.iter().map(|&(l, _)| l))?;
    let mut acc: Option<Tdd> = None;
    for i in 0..links.held.len() {
        let link = links.on(backend, compiled, i, &scope)?;
        let next = match acc {
            None => link,
            Some(acc) => backend.and(acc, link)?,
        };
        if backend.size(&next) > bound {
            return Ok(None);
        }
        acc = Some(next);
    }
    Ok(acc.map(|j| backend.size(&j)))
}

/// The scope over the variables of `scopes`, the whole program's where
/// one of them is.
fn over_scopes<'s>(
    backend: &TddBackend,
    scopes: impl IntoIterator<Item = &'s VtreeScope>,
) -> Result<VtreeScope, CompileError> {
    let mut vars: Vec<VarId> = Vec::new();
    for scope in scopes {
        match scope.vars() {
            Some(v) => vars.extend_from_slice(v),
            None => return Ok(backend.global()),
        }
    }
    vars.sort_unstable();
    vars.dedup();
    backend.scope(&vars)
}

/// The links `tying` on the scope over them, conjoined into groups while a
/// group's conjunction holds no more pairs than its links apart, each with
/// the variables of the copies its links tie; and that scope. The groups
/// depend on the links alone, so that the readers that read the same
/// links share them, each embedding them into its own scope.
fn groups(
    backend: &TddBackend,
    compiled: &Compiled,
    links: &Links,
    tying: &[usize],
) -> Result<(VtreeScope, Vec<Group>), CompileError> {
    let scope = over(
        backend,
        compiled,
        tying.iter().map(|&i| links.cut.links[i].0),
    )?;
    let mut out: Vec<(Tdd, usize, Vec<usize>)> = Vec::new();
    for &i in tying {
        let link = links.on(backend, compiled, i, &scope)?;
        let pairs = backend.size(&link);
        if let Some((conj, apart, members)) = out.last_mut() {
            let joined = backend.and(&*conj, &link)?;
            if backend.size(&joined) <= *apart + pairs {
                (*conj, *apart) = (joined, *apart + pairs);
                members.push(i);
                continue;
            }
        }
        out.push((link, pairs, vec![i]));
    }
    let groups = out
        .into_iter()
        .map(|(conj, _, members)| {
            let copies: Vec<VarId> = links
                .cut
                .vars
                .iter()
                .filter(|(_, at)| members.contains(at))
                .map(|&(v, _)| v)
                .collect();
            (conj, copies)
        })
        .collect();
    Ok((scope, groups))
}

/// Each of `readers` composed with the links of the copies it reads and
/// those the links read in turn ([`Links::tying`]), or `None` once those
/// composed gain more than `bound` pairs together. A reader can shrink
/// composed, so the gain is counted against the pairs of every reader,
/// those not composed yet too: the readers composed so far hold more than
/// all of them did by over `bound` only where the whole composition gains
/// more. A reader is composed
/// with each group of those links ([`groups`]) in one operation, which
/// sums out every copy the group's links tie, so that a clause table's
/// link, which ties several copies, leaves none of them. The readers that
/// need the same links share the groups.
fn composed_under(
    backend: &TddBackend,
    compiled: &Compiled,
    links: &Links,
    readers: &[Reader],
    bound: usize,
) -> Result<Option<Composed>, CompileError> {
    let held: usize = readers
        .iter()
        .map(|(r, _)| pairs(backend, compiled, *r))
        .sum();
    let (mut made, mut out) = (0usize, Vec::with_capacity(readers.len()));
    let mut shared: Vec<(Vec<usize>, VtreeScope, Vec<Group>)> = Vec::new();
    for (r, reads) in readers {
        let own = compiled.scope_of(*r);
        let tying = links.tying(reads);
        let at = match shared.iter().position(|(t, _, _)| *t == tying) {
            Some(at) => at,
            None => {
                let (from, groups) = groups(backend, compiled, links, &tying)?;
                shared.push((tying, from, groups));
                shared.len() - 1
            }
        };
        let (_, from, groups) = &shared[at];
        let scope = over_scopes(backend, [own, from])?;
        let stored = compiled.formulas[*r as usize]
            .as_ref()
            .expect("a reader has a formula");
        let mut acc = backend.embed_ref(stored, own, &scope)?;
        for (conj, vars) in groups {
            let conj = backend.embed_ref(conj, from, &scope)?;
            let mut copies: Vec<VarId> = vars.iter().filter_map(|&v| scope.local_of(v)).collect();
            copies.sort_unstable();
            acc = backend.engine.and_exists(acc, conj, &copies)?;
            if (made + backend.size(&acc)).saturating_sub(held) > bound {
                return Ok(None);
            }
        }
        made += backend.size(&acc);
        out.push((*r, (acc, scope)));
    }
    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tididi::Vtree;
    use tididi::limits::{LimitConfig, StopAt, StopRules};

    use super::*;

    /// Settling's budget moves the run's stop later by the budget, and
    /// again by each expansion it does not charge (module doc); the run's
    /// own stop is back once settling ends.
    #[test]
    fn work_not_charged_moves_the_run_stop_later() {
        let tree = Arc::new(Vtree::random(8, 1));
        tree.context().run(|engine| {
            let at = engine.limits().work_units() + 1000;
            let _run = engine
                .limits()
                .scope(LimitConfig::none().with_stop_rules(StopRules {
                    unconditional: Some(StopAt::WorkUnits(at)),
                    ..StopRules::default()
                }));
            let backend = TddBackend::new(engine, tree.clone());
            let stop = || engine.limits().armed().stop_rules().unconditional;
            {
                let mut budget = Budget::new(&backend, 500);
                assert_eq!(budget.left(), 500);
                assert_eq!(stop(), Some(StopAt::WorkUnits(at + 500)));
                budget.uncharge(300);
                assert_eq!(budget.left(), 800);
                assert_eq!(stop(), Some(StopAt::WorkUnits(at + 800)));
            }
            assert_eq!(stop(), Some(StopAt::WorkUnits(at)));
        });
    }
}
