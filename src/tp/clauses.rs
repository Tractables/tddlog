//! The clause loop over the bodies of facts alone, and the memo of the
//! loops compiled so far.

use rustc_hash::FxHashMap;
use std::sync::Arc;

use tididi::Tdd;
use tididi::vtree::VarId;

use super::backend::{Part, Read};
use super::factoring::{AtMostOne, Exclusive, Factoring, Pairs, in_order, placed_count};
use super::scope::scope_shape;
use super::stats::{ClauseMemoStats, NegationSource, Op};
use super::{CompileError, Formula, Reduce, TddBackend, VtreeScope};
use crate::ground::{AtomId, GLit};

/// The clauses `one_by_one` conjoins between contractions.
///
/// `and_clause` leaves the elements it splits uncontracted, and a run of
/// clauses that split the same elements doubles the diagram with each one
/// (the at-most-one clauses of bn_pathfinder's `fault` exhausted memory
/// so). A contraction every eighth clause bounds that growth at 2^8, and
/// costs the networks' rules less work than one after every clause. The
/// fix that costs nothing belongs in the library: `and_clause` could
/// contract the levels it rebuilds.
///
/// Most contractions of a network's tables remove nearly nothing, so a
/// contraction that leaves more than nine tenths of the pairs doubles the
/// clauses before the next, up to [`MAX_CLAUSES_PER_CONTRACTION`], and one
/// that removes more goes back to eight.
pub(super) const CLAUSES_PER_CONTRACTION: usize = 8;

/// The steps `TddBackend::none_of` may take per literal of its cubes and
/// groups before a clause loop takes over. At 64, bn_pathfinder and
/// bn_mildew give up on more loops and run a little slower.
const NONE_OF_STEPS_PER_LITERAL: u64 = 256;

/// The most clauses `one_by_one` conjoins between contractions.
const MAX_CLAUSES_PER_CONTRACTION: usize = 256;

/// Where `one_by_one`'s formula holds at most a [`OR_ROUTE_SHARE`]th of the
/// pairs of either operand of a body's last product, the body goes in by
/// the or-route at once ([`Factoring::or_route`]); otherwise the product
/// is given up for it once it has spent [`PRODUCT_WORK_PER_PAIR`] units per
/// pair of the formula and both operands, or [`PRODUCT_WORK_PER_FORMULA_PAIR`]
/// per pair of the formula, whichever is more.
///
/// The product of a body's derived parts can be far larger than the
/// formula it is negated into (on g22_70, hundreds of times), and the
/// or-route for every body costs more than the product. The or-route at
/// once gains where the formula is small against the parts, and the
/// product's bound keeps it where it stays within its operands. The bound
/// is on work, not on the product's output pairs: tididi's meter of those
/// charges the output's capacity, which may come only at the end of a
/// product.
const OR_ROUTE_SHARE: usize = 2;
const PRODUCT_WORK_PER_PAIR: u64 = 64;
const PRODUCT_WORK_PER_FORMULA_PAIR: u64 = 800;

/// What leads a rule of pairs in a [`ClauseKey`], above any literal.
const PAIRS: u32 = u32::MAX - 1;

/// The fewest bodies a loop of clauses over facts alone has to conjoin to be
/// looked up in the [`ClauseMemo`]. From two, ObjectDetection and
/// bn_barley copy nearly all their loops (ObjectDetection's are the same
/// two cubes over local variables), which from eight were built each time.
const MEMO_MIN_BODIES: usize = 2;

/// The most pairs one formula the [`ClauseMemo`] keeps may have, and all of
/// them together. At 2^21 bn_barley's table read by three nodes is kept,
/// and one of its readers copies it; at 2^18 it is built again.
const MEMO_MAX_PAIRS: usize = 1 << 21;
const MEMO_TOTAL_PAIRS: usize = 1 << 23;

/// The loops of clauses over facts alone compiled so far, by their bodies
/// over the slots of their scope's variables ([`ClauseKey`]).
///
/// `⋀_r ¬body_r` over facts alone depends on nothing else, so a loop whose
/// bodies are an earlier loop's with its variables renamed is that loop
/// renamed. Where the scope's vtree is the earlier scope's with its leaves
/// so renamed, up to mirrors, a copy of the earlier loop
/// ([`TddBackend::transport`], which checks the shapes) replaces the loop.
/// Facts are numbered in the order of the vtree's leaves
/// ([`fact_variables`]), and a projection keeps it, so bodies in the same
/// places of the same shape are the same in local variables. The key
/// leaves out the shape, so that a lookup that finds nothing does not walk
/// the scope's vtree: on a compile whose loops repeat little, as
/// bn_mildew's, that walk cost more than the memo saved. A lookup that
/// finds kept loops walks it once, for a hash of the shape up to mirrors
/// ([`scope_shape`]), and transports only the loops whose scope hashes the
/// same, the only ones that can transport; trying the others is a
/// transport that fails. Tables of one structure
/// are such loops, bn_munin's muscles, and so is a network's table read by
/// several nodes, once per reader with the reader's own copies of the
/// table's values ([`crate::indicators`]). The copies sit by their reader,
/// among the facts and in the vtree: numbered elsewhere, they take the
/// key's last slots, and on the other side of the table's inputs, the
/// mirror. With the slots and the mirrors, and every reader's link stating
/// that the table's inputs are exactly one ([`crate::indicators`]), the
/// loop of bn_barley's largest table, which has four readers, is built
/// once and copied three times.
///
/// A reader whose copies are split otherwise among the table's inputs has
/// a shape no mirror makes. Its loop is built, and kept too, for a later
/// reader of its shape. A kept loop rotated onto the scope is the same
/// formula, but such loops are small, and planning the rotation, a search
/// over the trees of up to seven units, cost more than the build.
///
/// [`TddBackend::transport`]: super::TddBackend::transport
/// [`fact_variables`]: super::fact_variables
#[derive(Default)]
pub(super) struct ClauseMemo {
    /// Per key, the loops kept on scopes of different shapes.
    entries: FxHashMap<ClauseKey, Vec<Kept>>,
    /// The pairs of the formulas kept.
    pairs: usize,
    /// The loop the watch refused in the run that stopped on it, for the
    /// next run to take ([`Refused`]).
    refused: Option<Refused>,
    pub(super) stats: ClauseMemoStats,
}

/// A loop of clauses over facts alone, built level by level from true
/// (`TddBackend::none_of`), that the watch refused, reduced in full; and
/// what it was built from: its head, its scope, the variables, cubes and
/// groups. The compile resets the component it stopped in, and the next
/// run, under a larger watch, would build the loop again for the same work
/// (on TPC-H q18 one loop covers the whole lineage). The next run takes it
/// instead ([`ClauseMemo::retake`]), on the
/// scope it was built on: a scope narrowed to the bodies' facts is
/// projected anew in each run, and the next run's narrowing to the same
/// facts takes the loop's ([`ClauseMemo::refused_scope`]). Held until the
/// next run, which in `auto` is the trial's next attempt, however many
/// rounds later: no operation of a bounded attempt outputs more than twice
/// its watch in nodes, so a waiting trial holds no more than that beside
/// its finished formulas. A vtree search drops it, since it moves the
/// formulas the compile holds and not this one.
struct Refused {
    head: AtomId,
    /// The program's vtree when the loop was built; a scope projected from
    /// it to the same variables is the loop's own ([`ClauseMemo::refused_scope`]).
    global: std::sync::Arc<tididi::Vtree>,
    scope: VtreeScope,
    vars: Vec<VarId>,
    cubes: Vec<Vec<tididi::Literal>>,
    groups: Vec<(Vec<VarId>, bool)>,
    f: Tdd,
}

/// A loop the [`ClauseMemo`] keeps: the loop, its scope, the slot of each
/// local variable of the scope ([`ClauseKey`]), and the scope's shape
/// ([`scope_shape`]).
struct Kept {
    f: Tdd,
    scope: VtreeScope,
    slots: Vec<u32>,
    shape: u64,
}

impl ClauseMemo {
    /// Keeps a copy of `f`, a loop of `size` pairs on `scope`, under `key`,
    /// within the bounds on one formula and on all of them, with the
    /// scope's `shape`.
    fn keep(
        &mut self,
        (key, scope): (ClauseKey, &VtreeScope),
        f: &Tdd,
        slots: Vec<u32>,
        shape: impl FnOnce(&[u32]) -> u64,
        backend: &TddBackend,
    ) -> Result<(), CompileError> {
        let size = backend.size(f);
        if size <= MEMO_MAX_PAIRS && self.pairs + size <= MEMO_TOTAL_PAIRS {
            let f = backend.copy(f)?;
            self.pairs += size;
            self.stats.kept += 1;
            self.stats.pairs += size;
            let shape = shape(&slots);
            self.entries.entry(key).or_default().push(Kept {
                f,
                scope: scope.clone(),
                slots,
                shape,
            });
        }
        Ok(())
    }

    /// The loop the watch refused in the last run, where it is the loop of
    /// `cubes` and `groups` over `vars` on `scope` for `head` ([`Refused`]).
    fn retake(
        &mut self,
        head: AtomId,
        scope: &VtreeScope,
        vars: &[VarId],
        cubes: &[Vec<tididi::Literal>],
        groups: &[(Vec<VarId>, bool)],
    ) -> Option<Tdd> {
        let r = self.refused.take_if(|r| {
            r.head == head
                && std::sync::Arc::ptr_eq(r.scope.vtree(), scope.vtree())
                && r.scope.vars() == scope.vars()
                && r.vars == vars
                && r.cubes == cubes
                && r.groups == groups
        })?;
        self.stats.retaken += 1;
        Some(r.f)
    }

    /// The scope the loop the watch refused was built on, where it is the
    /// projection of the program's vtree to `vars`: a scope projected
    /// anew is another vtree, which the loop is not on, so the next run
    /// takes this one ([`Refused`]).
    pub(super) fn refused_scope(&self, backend: &TddBackend, vars: &[VarId]) -> Option<VtreeScope> {
        let r = self.refused.as_ref()?;
        (r.scope.vars() == Some(vars) && std::sync::Arc::ptr_eq(&r.global, &backend.vtree()))
            .then(|| r.scope.clone())
    }

    /// Drops the loop the watch refused ([`Refused`]).
    pub(super) fn forget_refused(&mut self) {
        self.refused = None;
    }

    /// The formula of the loop the watch refused, taken out.
    pub(super) fn take_refused(&mut self) -> Option<Tdd> {
        self.refused.take().map(|r| r.f)
    }
}

/// A loop of clauses over facts alone: its scope's number of variables, and
/// its bodies in order, each as its literals, `2 * slot + positive`, sorted
/// and followed by `u32::MAX`; a rule of pairs as [`PAIRS`] and its
/// literals in their order, followed by `u32::MAX`, which tells two rules
/// apart where their pairs do. The bodies of tables of one structure come
/// in one order, so they are not sorted. A variable's slot is its rank
/// among the scope's variables but the head's own copies, which come last
/// in the order the bodies first read them: the copies a reader's link
/// ties to a clause table ([`crate::indicators`]), `link_of` the head.
#[derive(PartialEq, Eq, Hash)]
struct ClauseKey {
    vars: usize,
    lits: Vec<u32>,
    /// Whether the loop holds its sibling groups' at-most-one.
    seeds: bool,
    /// Where it does, the slots of the copies the bodies read that have
    /// sibling groups, ascending, each followed by its group's place among
    /// those groups in that order. A bit's rules read a copy without its
    /// siblings negated ([`crate::indicators`]), so two loops of the same
    /// bodies hold different groups where their copies are siblings
    /// differently.
    siblings: Vec<u32>,
}

impl Factoring<'_> {
    /// [`one_by_one`](Self::one_by_one) for bodies of facts alone, copied
    /// from the [`ClauseMemo`] where an earlier loop had the same key on a
    /// scope of the same shape up to mirrors, each local variable read as
    /// the one in its slot.
    pub(super) fn clauses_of_facts(
        &mut self,
        bodies: Vec<Vec<GLit>>,
        pairs: &[Pairs<'_>],
        scope: &VtreeScope,
    ) -> Result<Tdd, CompileError> {
        let clauses = bodies.len() + pairs.iter().map(|p| p.rule.clauses()).sum::<usize>();
        let key = if clauses >= MEMO_MIN_BODIES {
            self.clause_key(&bodies, pairs, scope)
        } else {
            None
        };
        let Some((key, slots)) = key else {
            return self.one_by_one(bodies, pairs, scope, None);
        };
        let mut copied = None;
        let mut known = None;
        let mut passed = 0;
        {
            let memo = self.memo.borrow();
            let kept = memo.entries.get(&key).map_or(&[][..], Vec::as_slice);
            if !kept.is_empty() {
                // The local variable of `scope` in each slot.
                let mut local = vec![VarId(0); slots.len()];
                for (k, &s) in slots.iter().enumerate() {
                    local[s as usize] = VarId(k as u32 + 1);
                }
                let map = |from_slots: &[u32]| -> Vec<VarId> {
                    from_slots.iter().map(|&s| local[s as usize]).collect()
                };
                // A loop on a scope of another shape does not transport.
                let here = scope_shape(scope.vtree(), &slots, &mut self.scratch.borrow_mut().shape);
                known = Some(here);
                for kept in kept {
                    if kept.shape != here {
                        passed += 1;
                        continue;
                    }
                    copied =
                        self.backend
                            .transport(&kept.f, &kept.scope, scope, &map(&kept.slots))?;
                    if copied.is_some() {
                        break;
                    }
                }
            }
        }
        let mut memo = self.memo.borrow_mut();
        memo.stats.lookups += 1;
        memo.stats.passed += passed;
        if let Some(f) = copied {
            memo.stats.copies += 1;
            drop(memo);
            return (self.watched)(f, self.head);
        }
        drop(memo);
        let f = self.one_by_one(bodies, pairs, scope, None)?;
        let shape = |slots: &[u32]| {
            known.unwrap_or_else(|| {
                scope_shape(scope.vtree(), slots, &mut self.scratch.borrow_mut().shape)
            })
        };
        self.memo
            .borrow_mut()
            .keep((key, scope), &f, slots, shape, self.backend)?;
        Ok(f)
    }

    /// `f`, the loop of `cubes` and `groups` over `vars` on `scope`, built
    /// from true, through the watch; a loop the watch refuses is kept for the
    /// next run ([`Refused`]).
    fn kept_if_refused(
        &self,
        f: Tdd,
        scope: &VtreeScope,
        vars: Vec<VarId>,
        cubes: Vec<Vec<tididi::Literal>>,
        groups: Vec<(Vec<VarId>, bool)>,
    ) -> Result<Tdd, CompileError> {
        match (self.checked)(f)? {
            Ok(f) => Ok(f),
            Err((f, pairs)) => {
                let mut memo = self.memo.borrow_mut();
                memo.stats.refused += 1;
                memo.refused = Some(Refused {
                    head: self.head,
                    global: self.backend.vtree(),
                    scope: scope.clone(),
                    vars,
                    cubes,
                    groups,
                    f,
                });
                Err(CompileError::Oversize {
                    atom: self.head,
                    pairs,
                })
            }
        }
    }

    /// The [`ClauseKey`] of bodies of facts alone and rules of pairs on
    /// `scope`, with the slot of each local variable of `scope`; `None` on
    /// the global scope.
    fn clause_key(
        &self,
        bodies: &[Vec<GLit>],
        pairs: &[Pairs<'_>],
        scope: &VtreeScope,
    ) -> Option<(ClauseKey, Vec<u32>)> {
        let held = scope.vars()?;
        let facts: Vec<tididi::Literal> = in_order(bodies, pairs)
            .flat_map(|(body, _)| body)
            .map(|&l| self.fact_literal(l).expect("bodies of facts alone"))
            .collect();
        let mut scratch = self.scratch.borrow_mut();
        // The head's own copies, in the order the bodies first read them,
        // marked with the stamp.
        let stamp = scratch.fresh_stamp();
        let mut own: Vec<VarId> = Vec::new();
        for f in &facts {
            let v = f.var.0 as usize;
            if self.link_of[v] == self.head && scratch.var_mark[v] != stamp {
                scratch.var_mark[v] = stamp;
                own.push(f.var);
            }
        }
        let free = held.len() - own.len();
        for (p, v) in own.iter().enumerate() {
            scratch.var_rank[v.0 as usize] = (free + p) as u32;
        }
        let mut slots = Vec::with_capacity(held.len());
        let mut before = 0;
        for (k, v) in held.iter().enumerate() {
            let v = v.0 as usize;
            if scratch.var_mark[v] == stamp {
                before += 1;
            } else {
                scratch.var_rank[v] = (k - before) as u32;
            }
            slots.push(scratch.var_rank[v]);
        }
        let mut lits = Vec::with_capacity(facts.len() + 2 * (bodies.len() + pairs.len()));
        let mut next = facts.iter();
        for (body, of_pairs) in in_order(bodies, pairs) {
            if of_pairs {
                lits.push(PAIRS);
            }
            let start = lits.len();
            lits.extend(
                next.by_ref()
                    .take(body.len())
                    .map(|f| 2 * scratch.var_rank[f.var.0 as usize] + u32::from(f.sign)),
            );
            if !of_pairs {
                lits[start..].sort_unstable();
            }
            lits.push(u32::MAX);
        }
        let mut siblings = Vec::new();
        if self.seeds {
            let mut read: Vec<(u32, u32)> = facts
                .iter()
                .filter(|f| self.sibling_of[f.var.0 as usize] != u32::MAX)
                .map(|f| {
                    (
                        scratch.var_rank[f.var.0 as usize],
                        self.sibling_of[f.var.0 as usize],
                    )
                })
                .collect();
            read.sort_unstable();
            read.dedup();
            let mut place: FxHashMap<u32, u32> = FxHashMap::default();
            for (slot, group) in read {
                let next = place.len() as u32;
                siblings.push(slot);
                siblings.push(*place.entry(group).or_insert(next));
            }
        }
        Some((
            ClauseKey {
                vars: held.len(),
                lits,
                seeds: self.seeds,
                siblings,
            },
            slots,
        ))
    }

    /// The sibling groups (`Encoding::siblings`) of which the bodies
    /// read two copies or more, each as the copies read, outside the
    /// groups `found`.
    fn sibling_groups(&self, bodies: &[Vec<GLit>], found: &[AtMostOne]) -> Vec<AtMostOne> {
        // The variables of the groups found, marked with the stamp.
        let mut scratch = self.scratch.borrow_mut();
        let taken = scratch.fresh_stamp();
        for (g, _) in found {
            for l in g {
                scratch.var_mark[l.var.0 as usize] = taken;
            }
        }
        let mut read: Vec<(u32, VarId)> = bodies
            .iter()
            .flatten()
            .filter_map(|&l| self.fact_literal(l))
            .filter(|f| {
                self.sibling_of[f.var.0 as usize] != u32::MAX
                    && scratch.var_mark[f.var.0 as usize] != taken
            })
            .map(|f| (self.sibling_of[f.var.0 as usize], f.var))
            .collect();
        read.sort_unstable();
        read.dedup();
        read.chunk_by(|a, b| a.0 == b.0)
            .filter(|run| run.len() >= 2)
            .map(|run| {
                (
                    run.iter().map(|&(_, v)| tididi::Literal::pos(v)).collect(),
                    false,
                )
            })
            .collect()
    }

    /// The product `c ∧ f` of parts of a body, on `scope`. A product of
    /// unreduced operands carries the dead pairs of both, and a body of
    /// several derived literals compounds them: on a bit-blasted circuit
    /// nine literals reached 34.6 M pairs for a formula of 2 959.
    /// Contraction does not remove them; the full plan does, and runs only
    /// where the product has outgrown its operands, each counted as it
    /// would be on `scope`.
    fn product(&mut self, c: Part, f: Part, scope: &VtreeScope) -> Result<Formula, CompileError> {
        let bound = 2
            * (placed_count(self.backend.nodes(&c.0), &c.1, scope)
                + placed_count(self.backend.nodes(&f.0), &f.1, scope));
        let mut r = self.backend.and_on(c, f, scope)?;
        (self.count)(Op::And, 1);
        if self.backend.nodes(&r) > bound {
            self.backend.minimize(&mut r, Reduce::Settled)?;
            (self.count)(Op::Minimize, 1);
        }
        Ok(((self.watched)(r, self.head)?, scope.clone()))
    }

    /// Adds the part `f` to the `pending` products of a body's parts:
    /// products of 2^rank parts, their ranks falling from the first, two of
    /// one rank multiplied as the second comes.
    fn push_product(
        &mut self,
        pending: &mut Vec<(Part, u32)>,
        f: Part,
        scope: &VtreeScope,
    ) -> Result<(), CompileError> {
        let mut top = (f, 0);
        while let Some(&(_, rank)) = pending.last()
            && rank == top.1
        {
            let (c, _) = pending.pop().expect("a product of this rank");
            top = (self.product(c, top.0, scope)?.into(), rank + 1);
        }
        pending.push(top);
        Ok(())
    }

    /// `body` itself on `scope`, which holds its variables: the product of
    /// its derived literals' formulas as a balanced tree, as `one_by_one`
    /// multiplies them, then its fact literals as clauses of one literal;
    /// a body of facts alone their cube, built in one pass.
    pub(super) fn conjunction(
        &mut self,
        body: &[GLit],
        scope: &VtreeScope,
    ) -> Result<Tdd, CompileError> {
        let mut pending: Vec<(Part, u32)> = Vec::new();
        let mut facts = Vec::new();
        for &l in body {
            if let Some(f) = self.fact_literal(l) {
                facts.push(f);
                continue;
            }
            let f = (self.part)(l, 1, scope)?;
            if self.backend.is_zero(&f.0) {
                return Ok(self.backend.zero(scope));
            }
            self.push_product(&mut pending, f, scope)?;
        }
        if pending.is_empty() {
            // A cube names each variable once.
            facts.sort_unstable_by_key(|f| (f.var, f.sign));
            facts.dedup();
            if facts.windows(2).any(|w| w[0].var == w[1].var) {
                return Ok(self.backend.zero(scope));
            }
            return self.backend.cube(scope, &facts);
        }
        let c = self.collapse(&mut pending, scope)?;
        let mut c = self.placed(c, scope)?;
        for &f in &facts {
            c = self.backend.and_clause(scope, c, &[f])?;
            (self.count)(Op::And, 1);
        }
        Ok(c)
    }

    /// `acc ∧ nb` on `scope` for the negation `nb` of a body, minimized;
    /// `nb` itself where `acc` is true (`trivial`), which a conjunction
    /// would walk level by level to give back, left on its own scope.
    fn and_term(
        &mut self,
        acc: Formula,
        nb: Part,
        trivial: bool,
        scope: &VtreeScope,
    ) -> Result<Formula, CompileError> {
        let (mut r, on) = if trivial {
            nb.owned(self.backend)?
        } else {
            let r = self.backend.and_on(acc, nb, scope)?;
            (self.count)(Op::And, 1);
            (r, scope.clone())
        };
        self.backend.minimize(&mut r, Reduce::Between)?;
        (self.count)(Op::Minimize, 1);
        Ok((r, on))
    }

    /// The product of the `pending` products of a body's parts, which it
    /// empties, the later ones first.
    fn collapse(
        &mut self,
        pending: &mut Vec<(Part, u32)>,
        scope: &VtreeScope,
    ) -> Result<Part, CompileError> {
        let (mut r, _) = pending.pop().expect("a product pending");
        while let Some((c, _)) = pending.pop() {
            r = self.product(c, r, scope)?.into();
        }
        Ok(r)
    }

    /// The product `a ∧ b` of a body's last derived part `b` with the
    /// product `a` of the others, on `scope`, to be negated into the
    /// formula `acc`; or the two parts back, for the or-route
    /// ([`OR_ROUTE_SHARE`]). Each is measured as it would be on `scope`.
    fn last_product(
        &mut self,
        acc: &Formula,
        a: Part,
        b: Part,
        scope: &VtreeScope,
    ) -> Result<Result<Formula, (Part, Part)>, CompileError> {
        let size = |f: &Tdd, own: &VtreeScope| placed_count(self.backend.size(f), own, scope);
        let (s_acc, s_a, s_b) = (size(&acc.0, &acc.1), size(&a.0, &a.1), size(&b.0, &b.1));
        if s_acc.saturating_mul(OR_ROUTE_SHARE) <= s_a.min(s_b) {
            return Ok(Err((a, b)));
        }
        let work = PRODUCT_WORK_PER_PAIR
            .saturating_mul((s_acc + s_a + s_b) as u64)
            .max(PRODUCT_WORK_PER_FORMULA_PAIR.saturating_mul(s_acc as u64));
        match self.backend.and_within(a, b, scope, work)? {
            Ok(r) => {
                (self.count)(Op::And, 1);
                Ok(Ok(((self.watched)(r, self.head)?, scope.clone())))
            }
            Err(parts) => {
                self.memo.borrow_mut().stats.given_up += 1;
                Ok(Err(parts))
            }
        }
    }

    /// `acc ∧ ¬(a ∧ b ∧ ⋀ facts)` as `x ∨ y`, `x = acc ∧ ¬a` and
    /// `y = acc ∧ ¬(b ∧ ⋀ facts)`, the facts on the smaller part, minimized:
    /// `acc` cuts each part's negation down before the two meet, where the
    /// product route builds the parts' product whole ([`OR_ROUTE_SHARE`]).
    /// The disjunction is `¬(¬x ∧ ¬y)` rather than tididi's overlay `or`,
    /// which is slower.
    fn or_route(
        &mut self,
        scope: &VtreeScope,
        acc: Formula,
        a: Part,
        b: Part,
        facts: &[tididi::Literal],
    ) -> Result<Tdd, CompileError> {
        let size = |f: &Part| placed_count(self.backend.size(&f.0), &f.1, scope);
        let (a, b) = if size(&a) >= size(&b) { (a, b) } else { (b, a) };
        let b = if facts.is_empty() {
            b.owned(self.backend)?
        } else {
            let mut b = self.placed(b, scope)?;
            for &f in facts {
                b = self.backend.and_clause(scope, b, &[f])?;
                (self.count)(Op::And, 1);
            }
            (b, scope.clone())
        };
        // Each part negated on its own scope, the smaller.
        let not_a = (a.0.with_cow(|f| (self.negate)(self.term, f))?, a.1);
        // `acc` is read twice: in place by the first conjunction, then
        // taken by the second.
        let (acc, on) = (Arc::new(acc.0), acc.1);
        let x = self.backend.and_on(
            Part(Read::Shared(Arc::clone(&acc)), on.clone()),
            not_a,
            scope,
        )?;
        (self.count)(Op::And, 1);
        let x = (self.watched)(x, self.head)?;
        let not_b = ((self.negate)(self.term, b.0.into())?, b.1);
        let y = self
            .backend
            .and_on(Part(Read::taken(acc), on), not_b, scope)?;
        (self.count)(Op::And, 1);
        let y = (self.watched)(y, self.head)?;
        let not_x = (self.negate)(NegationSource::Disjunction, x.into())?;
        let not_y = (self.negate)(NegationSource::Disjunction, y.into())?;
        let neither = self.backend.and(not_x, not_y)?;
        (self.count)(Op::And, 1);
        let neither = (self.watched)(neither, self.head)?;
        let mut acc = (self.negate)(NegationSource::Disjunction, neither.into())?;
        self.backend.minimize(&mut acc, Reduce::Between)?;
        (self.count)(Op::Minimize, 1);
        self.memo.borrow_mut().stats.or_routes += 1;
        Ok(acc)
    }

    /// `start ∧ ⋀_r ¬body_r` on `scope`, conjoined into `start` a body at a
    /// time, the rules of pairs with them; `start` is true when there is
    /// none.
    pub(super) fn one_by_one(
        &mut self,
        bodies: Vec<Vec<GLit>>,
        pairs: &[Pairs<'_>],
        scope: &VtreeScope,
        start: Option<Tdd>,
    ) -> Result<Tdd, CompileError> {
        let started = start.is_some();
        let mut acc = start.unwrap_or_else(|| self.backend.one(scope));
        let (mut groups, bodies) = self.at_most_one(bodies, pairs);
        if self.seeds {
            let siblings = self.sibling_groups(&bodies, &groups);
            if !siblings.is_empty() {
                self.memo.borrow_mut().stats.seeded += 1;
                groups.extend(siblings);
            }
        }
        let mut exclusive = Exclusive::new(&groups);
        // Bodies of facts alone, with the groups, built level by level
        // (`TddBackend::none_of`) rather than a clause at a time, whose
        // `and_clause` rebuilds the clause's spine (on bn_barley, many
        // times the tables' pairs). The builder gives up past
        // [`NONE_OF_STEPS_PER_LITERAL`] and the loop takes over.
        if bodies
            .iter()
            .flatten()
            .all(|&l| self.fact_literal(l).is_some())
        {
            let mut taken = exclusive.clone();
            let mut left_out = 0;
            let cubes: Vec<Vec<tididi::Literal>> = bodies
                .iter()
                .map(|body| {
                    let mut lits = Vec::with_capacity(body.len());
                    lits.extend(body.iter().filter_map(|&l| self.fact_literal(l)));
                    left_out += taken.leave_out(&mut lits);
                    taken.take_pair(&lits);
                    lits
                })
                .collect();
            let size = cubes.iter().map(Vec::len).sum::<usize>()
                + groups.iter().map(|(g, _)| g.len()).sum::<usize>();
            // The cubes and groups over the variables they read, the k-th
            // of them local variable k + 1.
            let (vars, cubes, local_groups) = {
                let mut scratch = self.scratch.borrow_mut();
                let vars = scratch.ranked(
                    cubes
                        .iter()
                        .flatten()
                        .chain(groups.iter().flat_map(|(g, _)| g))
                        .map(|l| l.var),
                    |i, _| i as u32 + 1,
                );
                let local = |v: VarId| VarId(scratch.var_rank[v.0 as usize]);
                let cubes: Vec<Vec<tididi::Literal>> = cubes
                    .iter()
                    .map(|c| {
                        c.iter()
                            .map(|l| tididi::Literal::new(local(l.var), l.sign))
                            .collect()
                    })
                    .collect();
                let local_groups: Vec<(Vec<VarId>, bool)> = groups
                    .iter()
                    .map(|(g, exactly)| (g.iter().map(|l| local(l.var)).collect(), *exactly))
                    .collect();
                (vars, cubes, local_groups)
            };
            // The loop the watch refused in the run before, the same one
            // ([`Refused`]).
            let retaken = if started {
                None
            } else {
                self.memo
                    .borrow_mut()
                    .retake(self.head, scope, &vars, &cubes, &local_groups)
            };
            if let Some(f) = retaken {
                return self.kept_if_refused(f, scope, vars, cubes, local_groups);
            }
            // A table's link ([`super::rows`]) is the set of its models,
            // built in one pass where there are no more of them than
            // literals in its bodies and groups. A link of more spreads the
            // switches of each row over the others' values, and builds
            // slower than level by level. The copies the head ties
            // ([`crate::indicators`]) that no group holds are the bits of
            // a value read in binary.
            let models = if vars.len() >= 2 {
                let mut grouped = vec![false; vars.len() + 1];
                for v in local_groups.iter().flat_map(|(g, _)| g) {
                    grouped[v.0 as usize] = true;
                }
                let outputs: Vec<VarId> = (1..=vars.len())
                    .filter(|&k| !grouped[k] && self.link_of[vars[k - 1].0 as usize] == self.head)
                    .map(|k| VarId(k as u32))
                    .collect();
                super::rows::table_models(&cubes, &local_groups, &outputs, vars.len(), size)
            } else {
                None
            };
            let from_models = models.is_some();
            let built = match models {
                Some(models) => Some(self.backend.of_models(scope, &vars, &models)?),
                None => self.backend.none_of(
                    scope,
                    &vars,
                    &cubes,
                    &local_groups,
                    NONE_OF_STEPS_PER_LITERAL.saturating_mul(size as u64),
                )?,
            };
            (self.count)(Op::And, 1);
            let mut memo = self.memo.borrow_mut();
            if let Some(f) = built {
                memo.stats.built += 1;
                memo.stats.from_models += usize::from(from_models);
                memo.stats.left_out += left_out;
                drop(memo);
                if !started {
                    return self.kept_if_refused(f, scope, vars, cubes, local_groups);
                }
                acc = self.backend.and(acc, f)?;
                return (self.watched)(acc, self.head);
            }
            memo.stats.fell_back += 1;
        }
        // Whether the formula is other than true, as the or-route needs: from
        // true it builds the product it would avoid. Until then `acc` is
        // `one`, and a body's negation is the formula (`and_term`).
        let mut narrowed = started || !groups.is_empty();
        for (mut lits, exactly) in groups {
            let one = self.one_of(scope, &mut lits, exactly)?;
            acc = self.backend.and_on((acc, scope.clone()), one, scope)?;
            (self.count)(Op::And, 1);
            acc = (self.watched)(acc, self.head)?;
        }
        // The formula, on `scope` but where it is a body's negation alone,
        // on the body's own scope (`and_term`), until an operation needs
        // it on `scope`.
        let mut acc: Formula = (acc, scope.clone());
        let mut uncontracted = 0;
        let mut every = CLAUSES_PER_CONTRACTION;
        // The formula's nodes after its last contraction. `and_clause`
        // charges the work clock the clause, not the levels it rebuilds, so
        // neither the watch nor the schedule sees a run that splits elements
        // clause after clause; a run that doubles the nodes is contracted
        // where it stands (pdb30's `td` loop otherwise grows to a thousand
        // times its formula's pairs between contractions).
        let mut settled_nodes = self.backend.nodes(&acc.0);
        // The clauses conjoined since they were last counted: a run is
        // counted at the next contraction, the next body with a derived
        // literal, or the end.
        let mut run = 0;
        // The work clock at which the formula is next read, for the
        // contraction schedule and the watch.
        let mut read_due = 0;
        for body in bodies {
            // A body of one derived literal is negated by reading the
            // literal of the other sign, the atom's stored formula or the
            // negation of it the compile keeps, rather than by building the
            // literal and negating that: the dual built `λ_b` from the
            // stored `μ_b` to negate it back. On the circuits a
            // bit-blasting front end builds, a disjunction's operand is
            // often such a body.
            if let &[l] = body.as_slice()
                && self.fact_literal(l).is_none()
            {
                (self.count)(Op::And, std::mem::take(&mut run));
                let nb = (self.part)(l.negated(), 1, scope)?;
                acc = self.and_term(acc, nb, !narrowed, scope)?;
                narrowed = true;
                settled_nodes = placed_count(self.backend.nodes(&acc.0), &acc.1, scope);
                acc.0 = (self.watched)(acc.0, self.head)?;
                if self.backend.is_zero(&acc.0) {
                    break;
                }
                continue;
            }
            let mut fact_lits: Vec<tididi::Literal> = Vec::with_capacity(body.len());
            // The product of the body's derived literals as a balanced
            // tree (`push_product`). Folded from the left, a body of k
            // literals multiplies a product of up to k - 1 of them k times:
            // a chain of conjunctions inlined into one body
            // (`ground::steps`) reached 4 095 literals on a bit-blasted
            // circuit, and its fold 575 M work units of its program's
            // 2.53 G.
            let mut pending: Vec<(Part, u32)> = Vec::new();
            let mut dead = false;
            // The two parts of a body that goes in by the or-route.
            let mut split: Option<(Part, Part)> = None;
            let mut derived_left = body
                .iter()
                .filter(|&&l| self.fact_literal(l).is_none())
                .count();
            for &l in &body {
                if let Some(f) = self.fact_literal(l) {
                    fact_lits.push(f);
                    continue;
                }
                let f = (self.part)(l, 1, scope)?;
                if self.backend.is_zero(&f.0) {
                    dead = true;
                    break;
                }
                derived_left -= 1;
                if narrowed && derived_left == 0 && !pending.is_empty() {
                    let c = self.collapse(&mut pending, scope)?;
                    match self.last_product(&acc, c, f, scope)? {
                        Ok(r) => pending.push((r.into(), 0)),
                        Err(parts) => split = Some(parts),
                    }
                    continue;
                }
                self.push_product(&mut pending, f, scope)?;
            }
            let conj = if pending.is_empty() {
                None
            } else {
                Some(self.collapse(&mut pending, scope)?)
            };
            if dead {
                continue; // body false: ¬body = true
            }
            let trivial = !narrowed;
            narrowed = true;
            let left_out = exclusive.leave_out(&mut fact_lits);
            if left_out > 0 {
                self.memo.borrow_mut().stats.left_out += left_out;
            }
            match (conj, split) {
                (_, Some((a, b))) => {
                    (self.count)(Op::And, std::mem::take(&mut run));
                    acc = (self.or_route(scope, acc, a, b, &fact_lits)?, scope.clone());
                    settled_nodes = self.backend.nodes(&acc.0);
                    acc.0 = (self.watched)(acc.0, self.head)?;
                }
                (None, None) => {
                    // ¬cube is a clause of literals.
                    let clause: Vec<tididi::Literal> =
                        fact_lits.iter().map(|l| l.negated()).collect();
                    run += 1;
                    if !acc.1.same(scope) {
                        acc = (self.placed(acc, scope)?, scope.clone());
                    }
                    acc.0 = self.backend.and_clause(scope, acc.0, &clause)?;
                    exclusive.take_pair(&fact_lits);
                    uncontracted += 1;
                    let grown = self.backend.nodes(&acc.0) > 2 * settled_nodes;
                    // The watch reads the whole formula, which a clause
                    // does not touch outside its spine; it is read where
                    // the contraction has just walked it, and the store
                    // reads the result. Read after every clause or every
                    // contraction, it was a large part of the networks'
                    // runs; so it reads again once the clauses since have
                    // spent a work unit per pair it read. The schedule of
                    // contractions reads the formula
                    // before and after one to see what it freed, and is
                    // kept to the same reads.
                    if uncontracted == every || grown {
                        uncontracted = 0;
                        (self.count)(Op::And, std::mem::take(&mut run));
                        let before = (self.backend.work_units() >= read_due)
                            .then(|| self.backend.size(&acc.0));
                        self.backend.minimize(&mut acc.0, Reduce::Contract)?;
                        (self.count)(Op::Minimize, 1);
                        settled_nodes = self.backend.nodes(&acc.0);
                        if let Some(before) = before {
                            let size = self.backend.size(&acc.0);
                            every = if size * 10 > before * 9 {
                                (every * 2).min(MAX_CLAUSES_PER_CONTRACTION)
                            } else {
                                CLAUSES_PER_CONTRACTION
                            };
                            if let Some(w) = self.watch
                                && size > w
                            {
                                acc.0 = (self.watched)(acc.0, self.head)?;
                            }
                            read_due = self.backend.work_units() + size as u64;
                        }
                        if grown {
                            every = CLAUSES_PER_CONTRACTION;
                        }
                    }
                }
                (Some(c), None) => {
                    (self.count)(Op::And, std::mem::take(&mut run));
                    // The facts as clauses of one literal, as in
                    // `negated_terms`: a literal conjoined as a diagram costs
                    // every level of the scope, a clause only the path from
                    // its leaf to the root. Without them a part is negated
                    // on its own scope, the smaller.
                    let c = if fact_lits.is_empty() {
                        c
                    } else {
                        let mut c = self.placed(c, scope)?;
                        for &f in &fact_lits {
                            c = self.backend.and_clause(scope, c, &[f])?;
                            (self.count)(Op::And, 1);
                        }
                        Part::from((c, scope.clone()))
                    };
                    let nb = (c.0.with_cow(|f| (self.negate)(self.term, f))?, c.1);
                    acc = self.and_term(acc, nb.into(), trivial, scope)?;
                    settled_nodes = placed_count(self.backend.nodes(&acc.0), &acc.1, scope);
                    acc.0 = (self.watched)(acc.0, self.head)?;
                }
            }
            if self.backend.is_zero(&acc.0) {
                break;
            }
        }
        (self.count)(Op::And, run);
        self.placed(acc, scope)
    }
}
