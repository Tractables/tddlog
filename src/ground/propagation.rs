//! Unit propagation over the completion of the rules ([`Propagation`]),
//! seeded by the evidence and the certain atoms, with each exclusive set
//! read as at most one true atom and each exhaustive one as exactly one,
//! with its counters and a trail that [`Propagation::undo`] takes back to
//! a mark. The conditioning ([`super::condition`]) and `detach` drive it.
//! Over the clauses it leaves on facts it finds the facts equal in every
//! world counted ([`Propagation::equal_facts`]), the gates the clauses
//! define ([`Propagation::gates`]) and, with a SAT solver, the backbones
//! ([`Propagation::backbones`]); [`super::condition`] says what each is
//! for.

use super::condition::Backbones;
use super::cover::Cover;
use super::deps::strongly_connected;
use super::preprocess::backbone;
use super::program::Lists;
use super::tables::Tables;
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, Prob, RulesByHead};

/// A fact of probability one half that the clauses define as an AND or an
/// OR of other facts' literals ([`Propagation::gates`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ground) struct Gate {
    pub(in crate::ground) fact: AtomId,
    /// Whether it is the OR of its inputs, rather than their AND.
    pub(in crate::ground) or: bool,
    pub(in crate::ground) inputs: Vec<GLit>,
    /// The rules of the clauses that define it.
    pub(in crate::ground) clauses: Vec<u32>,
}

impl Gate {
    /// The gate's rules: one per input for an OR, one of every input for
    /// an AND.
    pub(super) fn rules(&self) -> Vec<GroundRule> {
        if self.or {
            self.inputs
                .iter()
                .map(|&l| GroundRule::new(self.fact, vec![l]))
                .collect()
        } else {
            vec![GroundRule::new(self.fact, self.inputs.clone())]
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Val {
    Unknown,
    True,
    False,
}

/// The evidence cannot hold: conditioning stops, and the program is left to
/// count `P(e) = 0` as it is.
pub(in crate::ground) struct Contradiction;

/// The clauses the propagation leaves ([`Propagation::clauses`]), each the
/// literals on facts it rules out together, literal `(f, positive)` as node
/// `2 f + positive`.
pub(super) struct Clauses {
    nodes: Vec<u32>,
    /// Per clause, its rule and the end of its nodes.
    pub(super) ends: Vec<(u32, usize)>,
}

impl Clauses {
    /// Each clause's rule and nodes.
    fn iter(&self) -> impl Iterator<Item = (u32, &[u32])> + '_ {
        let mut start = 0;
        self.ends.iter().map(move |&(ri, end)| {
            let c = &self.nodes[start..end];
            start = end;
            (ri, c)
        })
    }
}

/// Unit propagation over the completion, with its counters, and the trail
/// of what it assigned, which [`Propagation::undo`] takes back to a mark.
pub(in crate::ground) struct Propagation<'a> {
    gp: &'a GroundProgram,
    pub(super) by_head: RulesByHead,
    /// The body literals on each atom, with their rules.
    readers: Lists<(u32, GLit)>,
    pub(super) val: Vec<Val>,
    /// Per rule, its literals not known to hold, and the atom whose value
    /// made one not hold, `u32::MAX` while none has.
    open: Vec<u32>,
    killer: Vec<u32>,
    /// Per atom, its rules not dead.
    live: Vec<u32>,
    /// Per atom, its exclusive set, `usize::MAX` for none.
    set_of: Vec<usize>,
    sets: &'a [Vec<AtomId>],
    /// Per set, whether it is exhaustive, and its atoms not known false.
    exhaustive: Vec<bool>,
    left: Vec<u32>,
    /// The atoms assigned, in order; those before `head` have been
    /// propagated.
    trail: Vec<AtomId>,
    head: usize,
    /// The rules an atom's value changed the counters of, as it is
    /// propagated.
    fired: Vec<u32>,
    /// Body literals visited, for a search's budget.
    pub(in crate::ground) work: u64,
}

impl<'a> Propagation<'a> {
    pub(super) fn new(gp: &'a GroundProgram, tables: &'a Tables, cover: &Cover) -> Self {
        let sets = tables.sets();
        let exhaustive = sets
            .iter()
            .enumerate()
            .map(|(i, set)| set.iter().all(|&a| cover.exhaustive[a as usize] == i))
            .collect();
        Self::with_sets(gp, sets, exhaustive)
    }

    /// With the exclusive `sets`, each read as at most one true atom, and
    /// as exactly one where `exhaustive` says so.
    pub(in crate::ground) fn with_sets(
        gp: &'a GroundProgram,
        sets: &'a [Vec<AtomId>],
        exhaustive: Vec<bool>,
    ) -> Self {
        let n = gp.num_atoms();
        let by_head = gp.rules_by_head();
        let readers = gp.readers(|ri, l| (ri, l));
        let mut set_of = vec![usize::MAX; n];
        for (i, set) in sets.iter().enumerate() {
            for &a in set {
                set_of[a as usize] = i;
            }
        }
        let left = sets.iter().map(|s| s.len() as u32).collect();
        Propagation {
            gp,
            open: gp.rules.iter().map(|r| r.body.len() as u32).collect(),
            killer: vec![u32::MAX; gp.rules.len()],
            live: by_head.iter().map(|rs| rs.len() as u32).collect(),
            by_head,
            readers,
            val: vec![Val::Unknown; n],
            set_of,
            sets,
            exhaustive,
            left,
            trail: Vec::new(),
            head: 0,
            fired: Vec::new(),
            work: 0,
        }
    }

    pub(in crate::ground) fn assign(
        &mut self,
        a: AtomId,
        holds: bool,
    ) -> Result<(), Contradiction> {
        let v = if holds { Val::True } else { Val::False };
        match self.val[a as usize] {
            Val::Unknown => {
                self.val[a as usize] = v;
                self.trail.push(a);
                Ok(())
            }
            known if known == v => Ok(()),
            _ => Err(Contradiction),
        }
    }

    pub(in crate::ground) fn assign_literal(
        &mut self,
        l: GLit,
        holds: bool,
    ) -> Result<(), Contradiction> {
        self.assign(l.atom(), l.is_positive() == holds)
    }

    pub(in crate::ground) fn holds(&self, l: GLit) -> Option<bool> {
        match self.val[l.atom() as usize] {
            Val::Unknown => None,
            v => Some((v == Val::True) == l.is_positive()),
        }
    }

    // The counters count what the trail has propagated, and an atom's
    // value is set before: where the one rule, literal or atom a counter
    // leaves is already decided, the trail settles it.

    /// The body of `h`'s one live rule holds.
    fn last_rule_holds(&mut self, h: AtomId) -> Result<(), Contradiction> {
        let ri = *self.by_head[h as usize]
            .iter()
            .find(|&&ri| self.killer[ri] == u32::MAX)
            .expect("one live rule");
        for k in 0..self.gp.rules[ri].body.len() {
            let l = self.gp.rules[ri].body[k];
            if self.holds(l).is_none() {
                self.assign_literal(l, true)?;
            }
        }
        Ok(())
    }

    /// The one open literal of rule `ri`, whose head is false, does not hold.
    fn last_literal_fails(&mut self, ri: usize) -> Result<(), Contradiction> {
        match self.gp.rules[ri]
            .body
            .iter()
            .find(|&&l| self.holds(l).is_none())
        {
            Some(&l) => self.assign_literal(l, false),
            None => Ok(()),
        }
    }

    /// Seeds the propagation with what the program knows, the heads of
    /// rules without a body true, the derived atoms without rules false
    /// and the atoms `certain` true (one entry per atom), and with the
    /// evidence, and runs it.
    pub(super) fn propagate_evidence(&mut self, certain: &[bool]) -> Result<(), Contradiction> {
        let gp = self.gp;
        debug_assert_eq!(certain.len(), gp.num_atoms());
        for r in &gp.rules {
            if r.body.is_empty() {
                self.assign(r.head, true)?;
            }
        }
        for (a, &sure) in certain.iter().enumerate() {
            if gp.kinds[a] == AtomKind::Derived && self.by_head[a].is_empty() {
                self.assign(a as AtomId, false)?;
            }
            if sure {
                self.assign(a as AtomId, true)?;
            }
        }
        for &(_, a, v) in &gp.evidence {
            if let Some(a) = a {
                self.assign(a, v)?;
            }
        }
        self.run()
    }

    pub(in crate::ground) fn run(&mut self) -> Result<(), Contradiction> {
        while self.head < self.trail.len() {
            let a = self.trail[self.head];
            self.head += 1;
            let holds = self.val[a as usize] == Val::True;
            let set = self.set_of[a as usize];
            let derived = self.gp.kinds[a as usize] == AtomKind::Derived;
            // The counters first, and then what they leave: an atom's
            // counters change all at once, which `undo` takes back whole.
            self.fired.clear();
            self.work += self.readers[a as usize].len() as u64;
            for &(ri, l) in &self.readers[a as usize] {
                if self.killer[ri as usize] != u32::MAX {
                    continue;
                }
                if l.is_positive() == holds {
                    self.open[ri as usize] -= 1;
                } else {
                    self.killer[ri as usize] = a;
                    self.live[self.gp.rules[ri as usize].head as usize] -= 1;
                }
                self.fired.push(ri);
            }
            if derived && !holds && set != usize::MAX && self.exhaustive[set] {
                self.left[set] -= 1;
            }
            for i in 0..self.fired.len() {
                let ri = self.fired[i] as usize;
                let h = self.gp.rules[ri].head;
                if self.killer[ri] == u32::MAX {
                    match (self.open[ri], self.val[h as usize]) {
                        (0, _) => self.assign(h, true)?,
                        (1, Val::False) => self.last_literal_fails(ri)?,
                        _ => {}
                    }
                } else {
                    match (self.live[h as usize], self.val[h as usize]) {
                        (0, _) => self.assign(h, false)?,
                        (1, Val::True) => self.last_rule_holds(h)?,
                        _ => {}
                    }
                }
            }
            if !derived {
                continue;
            }
            if holds {
                match self.live[a as usize] {
                    0 => return Err(Contradiction),
                    1 => self.last_rule_holds(a)?,
                    _ => {}
                }
                if set != usize::MAX {
                    for i in 0..self.sets[set].len() {
                        let b = self.sets[set][i];
                        if b != a {
                            self.assign(b, false)?;
                        }
                    }
                }
            } else {
                for i in 0..self.by_head[a as usize].len() {
                    let ri = self.by_head[a as usize][i];
                    if self.killer[ri] != u32::MAX {
                        continue;
                    }
                    match self.open[ri] {
                        0 => return Err(Contradiction),
                        1 => self.last_literal_fails(ri)?,
                        _ => {}
                    }
                }
                if set != usize::MAX && self.exhaustive[set] {
                    match self.left[set] {
                        0 => return Err(Contradiction),
                        1 => {
                            if let Some(&last) = self.sets[set]
                                .iter()
                                .find(|&&b| self.val[b as usize] != Val::False)
                            {
                                self.assign(last, true)?;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// The trail's length, a mark to [`undo`](Self::undo) to.
    pub(in crate::ground) fn mark(&self) -> usize {
        self.trail.len()
    }

    /// Takes back every assignment after the first `mark`, the counters of
    /// those propagated with them, the last first: a rule dead when an atom
    /// was propagated is dead again when it is taken back, and the atom
    /// killed it if it is the rule's killer.
    pub(in crate::ground) fn undo(&mut self, mark: usize) {
        while self.trail.len() > mark {
            let a = self.trail.pop().expect("above the mark");
            if self.trail.len() < self.head {
                let holds = self.val[a as usize] == Val::True;
                for &(ri, l) in &self.readers[a as usize] {
                    let ri = ri as usize;
                    if self.killer[ri] == a {
                        self.killer[ri] = u32::MAX;
                        self.live[self.gp.rules[ri].head as usize] += 1;
                    } else if self.killer[ri] == u32::MAX && l.is_positive() == holds {
                        self.open[ri] += 1;
                    }
                }
                let set = self.set_of[a as usize];
                if !holds
                    && self.gp.kinds[a as usize] == AtomKind::Derived
                    && set != usize::MAX
                    && self.exhaustive[set]
                {
                    self.left[set] += 1;
                }
            }
            self.val[a as usize] = Val::Unknown;
        }
        self.head = self.head.min(mark);
    }

    /// Whether `a` is known true and a rule of its holds, or known false
    /// and its rules are all dead: what its value needs of the facts is
    /// settled.
    pub(in crate::ground) fn justified(&self, a: AtomId) -> bool {
        match self.val[a as usize] {
            Val::Unknown => true,
            Val::True => self.by_head[a as usize]
                .iter()
                .any(|&ri| self.killer[ri] == u32::MAX && self.open[ri] == 0),
            Val::False => self.live[a as usize] == 0,
        }
    }

    /// For an atom not [`justified`](Self::justified), the literal to try:
    /// for a true one, of its live rules with the fewest open literals,
    /// the first open one, to hold; for a false one, likewise, to fail.
    pub(in crate::ground) fn to_justify(&self, a: AtomId) -> (GLit, bool) {
        let holds = self.val[a as usize] == Val::True;
        let ri = *self.by_head[a as usize]
            .iter()
            .filter(|&&ri| self.killer[ri] == u32::MAX)
            .min_by_key(|&&ri| self.open[ri])
            .expect("a live rule");
        let l = *self.gp.rules[ri]
            .body
            .iter()
            .find(|&&l| self.holds(l).is_none())
            .expect("an open literal");
        (l, holds)
    }

    /// The value assigned to `a`, `None` where it is unknown.
    pub(in crate::ground) fn value(&self, a: AtomId) -> Option<bool> {
        match self.val[a as usize] {
            Val::Unknown => None,
            v => Some(v == Val::True),
        }
    }

    /// The fact literal an open literal equals: a fact's, or that of a
    /// derived atom whose one live rule has one open literal, on a fact.
    fn fact_literal(&self, l: GLit) -> Option<(AtomId, bool)> {
        let a = l.atom();
        if self.val[a as usize] != Val::Unknown {
            return None;
        }
        if self.gp.is_fact(a) {
            return Some((a, l.is_positive()));
        }
        if self.live[a as usize] != 1 {
            return None;
        }
        let ri = *self.by_head[a as usize]
            .iter()
            .find(|&&ri| self.killer[ri] == u32::MAX)?;
        if self.open[ri] != 1 {
            return None;
        }
        let m = *self.gp.rules[ri]
            .body
            .iter()
            .find(|&&m| self.holds(m).is_none())?;
        (self.gp.is_fact(m.atom())).then_some((m.atom(), m.is_positive() == l.is_positive()))
    }

    /// Fixes the facts that every model of `clauses` agrees on, which a SAT
    /// solver finds beyond the propagation ([`backbone`]), but those
    /// of `summed` (one entry per atom), and propagates them. The solver
    /// has a variable for each fact the equalities `equal`
    /// ([`equal_facts`](Self::equal_facts)) leave, a copy's literals
    /// being those of the fact it copies. Hands back the pairs of literals
    /// that the clauses of two literals the solver ends with rule out, on
    /// facts left open, and what the search did; `Err` where the clauses
    /// have no model.
    pub(super) fn backbones(
        &mut self,
        clauses: &Clauses,
        equal: &[(AtomId, AtomId, bool)],
        summed: &[bool],
    ) -> Result<(Vec<(u32, u32)>, Backbones), Contradiction> {
        // Each fact the fact it equals or copies the negation of, as the
        // bit to flip in its node.
        let mut class: Vec<(AtomId, u32)> =
            (0..self.gp.num_atoms() as AtomId).map(|a| (a, 0)).collect();
        for &(g, f, same) in equal {
            class[g as usize] = (f, u32::from(!same));
        }
        // The facts of the clauses numbered from zero. A node `x` the
        // clause rules out is the solver's literal `2 v + (x & 1)`, the
        // fact's negation where `x` is positive; the solver's literal `l`
        // is the node `2 f + (l & 1)` ruled out.
        //
        // Replacing the copies first can shrink the search by orders of
        // magnitude: on a weighted CNF the facts left can be a hundredth as
        // many, every one a backbone the search then fixes, where over the
        // copies it stops with nearly all candidates open.
        let mut var_of = vec![u32::MAX; self.gp.num_atoms()];
        let mut facts: Vec<AtomId> = Vec::new();
        let mut lits: Vec<u32> = Vec::with_capacity(clauses.nodes.len());
        for &x in &clauses.nodes {
            let (f, flip) = class[(x / 2) as usize];
            if var_of[f as usize] == u32::MAX {
                var_of[f as usize] = facts.len() as u32;
                facts.push(f);
            }
            lits.push(2 * var_of[f as usize] + ((x & 1) ^ flip));
        }
        let ends: Vec<usize> = clauses.ends.iter().map(|&(_, end)| end).collect();
        let found = backbone::search(facts.len(), &lits, &ends).ok_or(Contradiction)?;
        let known = |p: &Self| {
            p.gp.kinds
                .iter()
                .enumerate()
                .filter(|&(a, k)| matches!(k, AtomKind::Fact(_)) && p.val[a] != Val::Unknown)
                .count()
        };
        let before = known(self);
        // A summed fact equals no other ([`gates`](Self::gates)).
        let mut unit: Vec<Option<bool>> = vec![None; self.gp.num_atoms()];
        for &l in &found.units {
            let f = facts[(l / 2) as usize];
            if !summed[f as usize] {
                unit[f as usize] = Some(l & 1 == 0);
                self.assign(f, l & 1 == 0)?;
            }
        }
        for &(g, f, same) in equal {
            if let Some(holds) = unit[f as usize] {
                self.assign(g, holds == same)?;
            }
        }
        self.run()?;
        let node = |l: u32| 2 * facts[(l / 2) as usize] + (l & 1);
        let open = |p: &Self, x: u32| p.val[(x / 2) as usize] == Val::Unknown;
        let implied = found
            .binaries
            .iter()
            .map(|&(a, b)| (node(a), node(b)))
            .filter(|&(x, y)| open(self, x) && open(self, y))
            .collect();
        Ok((
            implied,
            Backbones {
                search: found.search,
                fixed: known(self) - before,
            },
        ))
    }

    /// The clauses the propagation leaves: of each live rule of a false
    /// atom whose open literals are each a fact's literal
    /// ([`fact_literal`](Self::fact_literal)), on different facts, those
    /// literals, which the clause rules out together. Two or more each,
    /// since the propagation leaves no rule of a false atom with one.
    pub(super) fn clauses(&self) -> Clauses {
        let mut nodes: Vec<u32> = Vec::new();
        let mut ends: Vec<(u32, usize)> = Vec::new();
        let mut seen = vec![u32::MAX; self.gp.num_atoms()];
        for (h, rules) in self.by_head.iter().enumerate() {
            if self.val[h] != Val::False {
                continue;
            }
            for &ri in rules {
                if self.killer[ri] != u32::MAX || self.open[ri] < 2 {
                    continue;
                }
                let start = nodes.len();
                let mut on_facts = true;
                for &l in self.gp.rules[ri]
                    .body
                    .iter()
                    .filter(|&&l| self.holds(l).is_none())
                {
                    match self.fact_literal(l) {
                        Some(x) if seen[x.0 as usize] != ri as u32 => {
                            seen[x.0 as usize] = ri as u32;
                            nodes.push(2 * x.0 + u32::from(x.1));
                        }
                        _ => {
                            on_facts = false;
                            break;
                        }
                    }
                }
                if on_facts {
                    ends.push((ri as u32, nodes.len()));
                } else {
                    nodes.truncate(start);
                }
            }
        }
        Clauses { nodes, ends }
    }

    /// The facts that the clauses of two literals the propagation leaves,
    /// and the pairs of literals `implied` rules out besides, make equal to
    /// another, each with the fact it copies and whether it copies it or
    /// its negation; `Err` where a fact would equal its negation.
    pub(super) fn equal_facts(
        &self,
        clauses: &Clauses,
        implied: &[(u32, u32)],
    ) -> Result<Vec<(AtomId, AtomId, bool)>, Contradiction> {
        let n = self.gp.num_atoms();
        // Not both: each implies the other's negation.
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for (x, y) in clauses
            .iter()
            .filter(|(_, c)| c.len() == 2)
            .map(|(_, c)| (c[0], c[1]))
            .chain(implied.iter().copied())
        {
            edges.push((x as usize, (y ^ 1) as usize));
            edges.push((y as usize, (x ^ 1) as usize));
        }
        if edges.is_empty() {
            return Ok(Vec::new());
        }
        let component = strongly_connected(2 * n, &edges);
        let mut members: Vec<Vec<usize>> = Vec::new();
        let mut seen = vec![usize::MAX; 2 * n];
        for v in 0..2 * n {
            if component[v] == usize::MAX {
                continue;
            }
            if component[v] == component[v ^ 1] {
                return Err(Contradiction);
            }
            if seen[component[v]] == usize::MAX {
                seen[component[v]] = members.len();
                members.push(Vec::new());
            }
            members[seen[component[v]]].push(v);
        }
        let mut copied = vec![false; n];
        let mut out = Vec::new();
        for group in &members {
            // The lowest fact of the component copies nothing; a component
            // and its negation have the same facts, and the one holding the
            // lowest fact's positive literal decides.
            let first = group[0];
            if first % 2 == 0 || group.len() < 2 {
                continue;
            }
            let f = (first / 2) as AtomId;
            for &v in &group[1..] {
                let g = v / 2;
                if !copied[g] {
                    copied[g] = true;
                    out.push((g as AtomId, f, v % 2 == 1));
                }
            }
        }
        Ok(out)
    }

    /// The facts of probability one half that the clauses the propagation
    /// leaves define as an AND or an OR of other facts' literals: one
    /// definition each, for none of the facts of `equal`, and none that
    /// reads its fact through the definitions of its inputs.
    ///
    /// A clause is a rule of a false atom whose open literals are each a
    /// fact's literal ([`fact_literal`](Self::fact_literal)), on different
    /// facts; it rules out their conjunction. A clause `G, x_1, ..., x_k`,
    /// k two or more, with the clauses `\+G, \+x_i`, says that `G` holds
    /// exactly where some `x_i` does not: for `G` a fact `g`, `g` is the OR
    /// of the `\+x_i`; for `G` the literal `\+g`, `g` is the AND of the
    /// `x_i`. The clauses of two literals are looked up by a hash of the
    /// pair, and a literal is tried as `G` only where `\+G` is in as many
    /// of them as it needs, so that a clause costs about its length; the
    /// lookups are bounded by four per literal of the clauses all the same,
    /// which keeps the search linear in them.
    pub(super) fn gates(&self, clauses: &Clauses, equal: &[(AtomId, AtomId, bool)]) -> Vec<Gate> {
        if !self.gp.kinds.contains(&AtomKind::Fact(Prob::HALF)) {
            return Vec::new();
        }
        let n = self.gp.num_atoms();
        // `^ 1` negates a node.
        let lit = |v: u32| GLit::new(v / 2, v & 1 == 1);
        let pair = |a: u32, b: u32| (a.min(b), a.max(b));
        // The clauses of two literals, each with its rule, and per node the
        // number it is in; the longer ones' nodes end to end, and per clause
        // its rule and its end.
        let mut binary: rustc_hash::FxHashMap<(u32, u32), u32> = rustc_hash::FxHashMap::default();
        let mut degree = vec![0u32; 2 * n];
        let mut lits: Vec<u32> = Vec::new();
        let mut long: Vec<(u32, usize)> = Vec::new();
        for (ri, c) in clauses.iter() {
            if let &[a, b] = c {
                if let std::collections::hash_map::Entry::Vacant(e) = binary.entry(pair(a, b)) {
                    e.insert(ri);
                    degree[a as usize] += 1;
                    degree[b as usize] += 1;
                }
            } else {
                lits.extend_from_slice(c);
                long.push((ri, lits.len()));
            }
        }
        let mut excluded = vec![false; n];
        for &(g, f, _) in equal {
            excluded[g as usize] = true;
            excluded[f as usize] = true;
        }
        let mut gates: Vec<Gate> = Vec::new();
        let mut gate_of = vec![u32::MAX; n];
        let mut budget = 4 * lits.len();
        let mut found: Vec<u32> = Vec::new();
        let mut start = 0;
        'clauses: for &(ri, end) in &long {
            let clause = &lits[start..end];
            start = end;
            for (i, &out) in clause.iter().enumerate() {
                let g = (out / 2) as usize;
                if self.gp.kinds[g] != AtomKind::Fact(Prob::HALF)
                    || excluded[g]
                    || gate_of[g] != u32::MAX
                    || (degree[(out ^ 1) as usize] as usize) < clause.len() - 1
                {
                    continue;
                }
                found.clear();
                for (j, &x) in clause.iter().enumerate() {
                    if j == i {
                        continue;
                    }
                    if budget == 0 {
                        break 'clauses;
                    }
                    budget -= 1;
                    match binary.get(&pair(out ^ 1, x ^ 1)) {
                        Some(&r) => found.push(r),
                        None => break,
                    }
                }
                if found.len() + 1 < clause.len() {
                    continue;
                }
                let or = out & 1 == 1;
                let inputs = clause
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| j != i)
                    .map(|(_, &x)| lit(if or { x ^ 1 } else { x }))
                    .collect();
                found.push(ri);
                gate_of[g] = gates.len() as u32;
                gates.push(Gate {
                    fact: g as AtomId,
                    or,
                    inputs,
                    clauses: found.clone(),
                });
                break;
            }
        }
        // A depth-first search over the gates' inputs refuses each gate
        // with an input on the search's path, under it: every other input
        // is a gate the search finished first, so the gates taken read one
        // another without a cycle. A cyclic pair, `g1 :- g2. g2 :- g1.`,
        // has one least model, not the clauses' two.
        let mut state = vec![0u8; gates.len()];
        let mut refused = vec![false; gates.len()];
        let mut path: Vec<(usize, usize)> = Vec::new();
        for root in 0..gates.len() {
            if state[root] != 0 {
                continue;
            }
            state[root] = 1;
            path.push((root, 0));
            while let Some(&(v, e)) = path.last() {
                let Some(&l) = gates[v].inputs.get(e) else {
                    state[v] = 2;
                    path.pop();
                    continue;
                };
                path.last_mut().expect("on the path").1 += 1;
                let w = gate_of[l.atom() as usize];
                if w == u32::MAX {
                    continue;
                }
                match state[w as usize] {
                    0 => {
                        state[w as usize] = 1;
                        path.push((w as usize, 0));
                    }
                    1 => refused[v] = true,
                    _ => {}
                }
            }
        }
        gates
            .into_iter()
            .zip(refused)
            .filter_map(|(gate, refused)| (!refused).then_some(gate))
            .collect()
    }

    /// Of `gates`, those that nothing reads but the definitions of the
    /// others kept: no query asks the fact, and every live rule that reads
    /// it is a clause of a definition kept. A gate that a query, another
    /// rule or a clause outside the definitions reads is left a fact with
    /// its clauses, and so are the gates whose definitions it then keeps,
    /// through a worklist: each gate left is visited once, whatever the
    /// depth of the gates that read one another.
    pub(super) fn unread(
        &self,
        gates: Vec<Gate>,
        queries: &[(crate::syntax::Atom, Option<AtomId>, bool)],
    ) -> Vec<Gate> {
        let mut asked = vec![false; self.gp.num_atoms()];
        for &(_, q, _) in queries {
            if let Some(q) = q {
                asked[q as usize] = true;
            }
        }
        let mut owner = vec![u32::MAX; self.gp.rules.len()];
        let mut gate_of = vec![u32::MAX; self.gp.num_atoms()];
        for (k, gate) in gates.iter().enumerate() {
            gate_of[gate.fact as usize] = k as u32;
            for &ri in &gate.clauses {
                owner[ri as usize] = k as u32;
            }
        }
        let live = |ri: u32| self.killer[ri as usize] == u32::MAX;
        let mut left: Vec<usize> = (0..gates.len())
            .filter(|&k| {
                let g = gates[k].fact as usize;
                asked[g]
                    || self.readers[g]
                        .iter()
                        .any(|&(ri, _)| live(ri) && owner[ri as usize] == u32::MAX)
            })
            .collect();
        let mut kept = vec![true; gates.len()];
        for &k in &left {
            kept[k] = false;
        }
        // A gate left a fact keeps its clauses, which read the gates in them.
        while let Some(k) = left.pop() {
            for &ri in gates[k]
                .clauses
                .iter()
                .filter(|&&ri| live(ri) && owner[ri as usize] == k as u32)
            {
                for l in &self.gp.rules[ri as usize].body {
                    let j = gate_of[l.atom() as usize];
                    if j != u32::MAX && kept[j as usize] {
                        kept[j as usize] = false;
                        left.push(j as usize);
                    }
                }
            }
        }
        gates
            .into_iter()
            .zip(kept)
            .filter_map(|(gate, kept)| kept.then_some(gate))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::program::test_program;
    use crate::syntax::Atom;
    use GLit::{Neg, Pos};

    /// A program of facts of probabilities `facts`, atoms 0 on, and after
    /// them an atom per clause of `clauses`, of one rule with that body,
    /// observed false.
    fn program(facts: &[f64], clauses: &[&[GLit]]) -> GroundProgram {
        let atom = |i: usize| (facts.len() + i) as AtomId;
        let rules: Vec<(AtomId, &[GLit])> = clauses
            .iter()
            .enumerate()
            .map(|(i, &body)| (atom(i), body))
            .collect();
        let evidence: Vec<(AtomId, bool)> = (0..clauses.len()).map(|i| (atom(i), false)).collect();
        test_program(facts, clauses.len(), &rules, &evidence)
    }

    /// The propagation of `gp`'s evidence, the clauses it leaves and the
    /// facts it finds equal.
    fn propagated(gp: &GroundProgram) -> (Propagation<'_>, Clauses, Vec<(AtomId, AtomId, bool)>) {
        let mut p = Propagation::with_sets(gp, &[], Vec::new());
        for &(_, a, v) in &gp.evidence {
            assert!(p.assign(a.expect("an atom"), v).is_ok());
        }
        assert!(p.run().is_ok(), "the clauses can hold");
        let clauses = p.clauses();
        let Ok(equal) = p.equal_facts(&clauses, &[]) else {
            panic!("no fact equals its negation")
        };
        (p, clauses, equal)
    }

    /// The gates of [`program`]`(facts, clauses)`.
    fn gates_of(facts: &[f64], clauses: &[&[GLit]]) -> Vec<Gate> {
        let gp = program(facts, clauses);
        let (p, clauses, equal) = propagated(&gp);
        p.gates(&clauses, &equal)
    }

    /// The facts of the gates of [`program`]`(facts, clauses)` that nothing
    /// else reads, with the facts of `asked` queried.
    fn unread_of(facts: &[f64], clauses: &[&[GLit]], asked: &[AtomId]) -> Vec<AtomId> {
        let gp = program(facts, clauses);
        let (p, clauses, equal) = propagated(&gp);
        let queries: Vec<_> = asked
            .iter()
            .map(|&a| (Atom::default(), Some(a), true))
            .collect();
        p.unread(p.gates(&clauses, &equal), &queries)
            .iter()
            .map(|gate| gate.fact)
            .collect()
    }

    fn gate(fact: AtomId, or: bool, inputs: &[GLit], clauses: &[u32]) -> Gate {
        Gate {
            fact,
            or,
            inputs: inputs.to_vec(),
            clauses: clauses.to_vec(),
        }
    }

    #[test]
    fn a_clause_and_its_binary_clauses_define_an_or_or_an_and() {
        // 2 is the OR of 0 and 1: `0 -> 2`, `1 -> 2`, `2 -> 0 or 1`.
        let or: &[&[GLit]] = &[
            &[Pos(0), Neg(2)],
            &[Pos(1), Neg(2)],
            &[Pos(2), Neg(0), Neg(1)],
        ];
        assert_eq!(
            gates_of(&[0.2, 0.3, 0.5], or),
            [gate(2, true, &[Pos(0), Pos(1)], &[0, 1, 2])]
        );
        // 2 is the AND of 0 and not 1, the long clause first.
        let and: &[&[GLit]] = &[
            &[Neg(2), Pos(0), Neg(1)],
            &[Pos(2), Neg(0)],
            &[Pos(2), Pos(1)],
        ];
        assert_eq!(
            gates_of(&[0.2, 0.3, 0.5], and),
            [gate(2, false, &[Pos(0), Neg(1)], &[1, 2, 0])]
        );
        // Of another probability, it is left alone.
        assert_eq!(gates_of(&[0.2, 0.3, 0.4], or), []);
        // Without one of the binary clauses, 2 is not the OR.
        assert_eq!(gates_of(&[0.2, 0.3, 0.5], &or[1..]), []);
        // The exactly-one constraint of 0, 1 and 2, all of one half: 0 is
        // the AND of not 1 and not 2, and the clause between 1 and 2 stays.
        let one: &[&[GLit]] = &[
            &[Neg(0), Neg(1), Neg(2)],
            &[Pos(0), Pos(1)],
            &[Pos(0), Pos(2)],
            &[Pos(1), Pos(2)],
        ];
        assert_eq!(
            gates_of(&[0.5, 0.5, 0.5], one),
            [gate(0, false, &[Neg(1), Neg(2)], &[1, 2, 0])]
        );
    }

    #[test]
    fn a_gate_is_read_on_the_literals_the_propagation_leaves_open() {
        // 3 is the OR of 0, 1 and 2, and a unit clause makes 2 false: the
        // clauses on 2 are dead or shorter, and 3 is the OR of 0 and 1.
        let clauses: &[&[GLit]] = &[
            &[Pos(0), Neg(3)],
            &[Pos(1), Neg(3)],
            &[Pos(2), Neg(3)],
            &[Pos(3), Neg(0), Neg(1), Neg(2)],
            &[Pos(2)],
        ];
        assert_eq!(
            gates_of(&[0.2, 0.3, 0.4, 0.5], clauses),
            [gate(3, true, &[Pos(0), Pos(1)], &[0, 1, 3])]
        );
    }

    #[test]
    fn of_definitions_through_each_other_one_is_taken() {
        // 2 is the OR of 3 and 0, 3 the AND of 2 and 1, and clause 0 is in
        // both definitions. The search takes 2's and refuses 3's, which
        // reads 2.
        let clauses: &[&[GLit]] = &[
            &[Neg(2), Pos(3)],
            &[Neg(2), Pos(0)],
            &[Pos(2), Neg(3), Neg(0)],
            &[Pos(3), Neg(1)],
            &[Neg(3), Pos(2), Pos(1)],
        ];
        assert_eq!(
            gates_of(&[0.3, 0.6, 0.5, 0.5], clauses),
            [gate(2, true, &[Pos(3), Pos(0)], &[0, 1, 2])]
        );
        // Nested without a cycle, both are taken: 3 is the AND of 2 and 1,
        // and 2 the OR of 0 and 1.
        let nested: &[&[GLit]] = &[
            &[Pos(3), Neg(2)],
            &[Pos(3), Neg(1)],
            &[Neg(3), Pos(2), Pos(1)],
            &[Pos(0), Neg(2)],
            &[Pos(1), Neg(2)],
            &[Pos(2), Neg(0), Neg(1)],
        ];
        assert_eq!(
            gates_of(&[0.3, 0.6, 0.5, 0.5], nested),
            [
                gate(3, false, &[Pos(2), Pos(1)], &[0, 1, 2]),
                gate(2, true, &[Pos(0), Pos(1)], &[3, 4, 5])
            ]
        );
    }

    #[test]
    fn a_gate_is_taken_where_nothing_else_reads_it() {
        // 2 is the OR of 0 and 1, and nothing else reads it.
        let or: &[&[GLit]] = &[
            &[Pos(0), Neg(2)],
            &[Pos(1), Neg(2)],
            &[Pos(2), Neg(0), Neg(1)],
        ];
        assert_eq!(unread_of(&[0.2, 0.3, 0.5, 0.4], or, &[]), [2]);
        // A clause that reads 2, `2 -> 3`, or a query of 2 leaves it a fact.
        let read: &[&[GLit]] = &[or[0], or[1], or[2], &[Pos(2), Neg(3)]];
        assert_eq!(unread_of(&[0.2, 0.3, 0.5, 0.4], read, &[]), []);
        assert_eq!(unread_of(&[0.2, 0.3, 0.5, 0.4], or, &[2]), []);
        // 3 is the AND of 2 and 1, and 2 the OR of 0 and 1: 2 is read only
        // by 3's definition, and both are taken where nothing reads 3; a
        // clause that reads 3 keeps its definition, which reads 2.
        let nested: &[&[GLit]] = &[
            &[Pos(3), Neg(2)],
            &[Pos(3), Neg(1)],
            &[Neg(3), Pos(2), Pos(1)],
            &[Pos(0), Neg(2)],
            &[Pos(1), Neg(2)],
            &[Pos(2), Neg(0), Neg(1)],
        ];
        assert_eq!(unread_of(&[0.3, 0.6, 0.5, 0.5, 0.4], nested, &[]), [3, 2]);
        let read: Vec<&[GLit]> = nested
            .iter()
            .copied()
            .chain([&[Pos(3), Neg(4)][..]])
            .collect();
        assert_eq!(unread_of(&[0.3, 0.6, 0.5, 0.5, 0.4], &read, &[]), []);
        // Of the two definitions through each other, 2's is taken and 3's
        // refused, and 3's clauses, which stay, read 2.
        let cyclic: &[&[GLit]] = &[
            &[Neg(2), Pos(3)],
            &[Neg(2), Pos(0)],
            &[Pos(2), Neg(3), Neg(0)],
            &[Pos(3), Neg(1)],
            &[Neg(3), Pos(2), Pos(1)],
        ];
        assert_eq!(unread_of(&[0.3, 0.6, 0.5, 0.5], cyclic, &[]), []);
    }
}
