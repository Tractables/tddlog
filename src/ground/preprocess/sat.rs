//! A small CDCL SAT solver, for the conditioning's search for the facts
//! that every world the clauses allow agrees on ([`super::backbone`]), and
//! for the search for an independent support of the clauses
//! ([`super::support`]).
//!
//! Two watched literals with a blocking literal, the clauses of two
//! literals in implication lists of their own, VSIDS with phase saving,
//! Luby restarts, first-UIP learning with recursive minimization, and
//! assumptions taken as the first decisions, as in MiniSat (Eén and
//! Sörensson, SAT 2003). Learnt clauses are cut at restarts, at level
//! zero, by their literal block distance (Audemard and Simon, IJCAI 2009):
//! the clause store is then rebuilt without the worse half of them and
//! without what level zero satisfies or falsifies. A call stops at its
//! conflict limit, or once the solver's propagation steps pass its limit,
//! and then says it does not know. Where the caller asks
//! ([`Solver::keep_assumptions`]), a call starts from the levels of the
//! assumptions it shares, in order, with the last call's, as a search
//! whose assumptions change at their end asks.

/// A literal: variable `v` is `2 v`, its negation `2 v + 1`.
pub(in crate::ground) type Lit = u32;

/// The variable of a literal.
fn var(l: Lit) -> usize {
    (l >> 1) as usize
}

/// No reason: a decision, an assumption or a unit.
const NONE: u32 = u32::MAX;
/// A reason with this bit is the clause of two literals of the implied
/// literal and the literal with the bit cleared.
const BINARY: u32 = 1 << 31;

/// What a call found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ground) enum Outcome {
    /// A model, which [`Solver::holds`] reads until the next change.
    Sat,
    /// No model with the assumptions.
    Unsat,
    /// The call's limits came first.
    Unknown,
}

#[derive(Clone, Copy)]
struct Watch {
    clause: u32,
    /// A literal of the clause: where it holds the clause is not visited.
    blocker: Lit,
}

/// A clause of three literals or more, `arena[start..start + len]`, its
/// two first literals watched.
#[derive(Clone, Copy)]
struct Clause {
    start: usize,
    len: u32,
    /// The literal block distance of a learnt clause; 0 for a clause given.
    lbd: u32,
}

#[derive(Clone, Copy)]
enum Conflict {
    Clause(u32),
    Binary(Lit, Lit),
}

/// The solver. Single-threaded, no state outside it.
pub(in crate::ground) struct Solver {
    arena: Vec<Lit>,
    clauses: Vec<Clause>,
    /// Per literal, the clauses watching it, visited when it becomes false.
    watches: Vec<Vec<Watch>>,
    /// Per literal, the other literal of each clause of two with it.
    binaries: Vec<Vec<Lit>>,
    /// Per literal, 1 true, -1 false, 0 unassigned.
    value: Vec<i8>,
    level: Vec<u32>,
    reason: Vec<u32>,
    trail: Vec<Lit>,
    trail_lim: Vec<usize>,
    qhead: usize,
    activity: Vec<f64>,
    var_inc: f64,
    /// A binary max-heap of variables by activity, and each variable's
    /// place in it, `NONE` outside.
    heap: Vec<u32>,
    heap_pos: Vec<u32>,
    /// The value each variable last had: true for positive.
    phase: Vec<bool>,
    seen: Vec<bool>,
    learnt: Vec<Lit>,
    stack: Vec<Lit>,
    found: Vec<Lit>,
    to_clear: Vec<Lit>,
    level_stamp: Vec<u64>,
    stamp: u64,
    /// False once the clauses are found unsatisfiable.
    ok: bool,
    learnts: usize,
    max_learnts: f64,
    /// The units at level zero when the store was last rebuilt.
    fixed_at_rebuild: usize,
    restarts: u32,
    /// Conflicts over all calls.
    pub(in crate::ground) conflicts: u64,
    /// Propagation steps over all calls: watches and implications visited
    /// and clause literals scanned for a new watch.
    pub(in crate::ground) ticks: u64,
    /// The propagation steps past which a call stops.
    pub(in crate::ground) tick_limit: u64,
    /// Whether a call keeps the levels of the assumptions it shares with
    /// the last call's.
    pub(in crate::ground) keep_assumptions: bool,
    /// The assumptions decided at levels `1..=held.len()`.
    held: Vec<Lit>,
}

impl Solver {
    pub(in crate::ground) fn new(num_vars: usize) -> Self {
        let mut s = Solver {
            arena: Vec::new(),
            clauses: Vec::new(),
            watches: vec![Vec::new(); 2 * num_vars],
            binaries: vec![Vec::new(); 2 * num_vars],
            value: vec![0; 2 * num_vars],
            level: vec![0; num_vars],
            reason: vec![NONE; num_vars],
            trail: Vec::with_capacity(num_vars),
            trail_lim: Vec::new(),
            qhead: 0,
            activity: vec![0.0; num_vars],
            var_inc: 1.0,
            heap: Vec::with_capacity(num_vars),
            heap_pos: vec![NONE; num_vars],
            phase: vec![false; num_vars],
            seen: vec![false; num_vars],
            learnt: Vec::new(),
            stack: Vec::new(),
            found: Vec::new(),
            to_clear: Vec::new(),
            level_stamp: Vec::new(),
            stamp: 0,
            ok: true,
            learnts: 0,
            max_learnts: 0.0,
            fixed_at_rebuild: 0,
            restarts: 0,
            conflicts: 0,
            ticks: 0,
            tick_limit: u64::MAX,
            keep_assumptions: false,
            held: Vec::new(),
        };
        for v in 0..num_vars as u32 {
            s.heap_insert(v);
        }
        s
    }

    pub(in crate::ground) fn num_vars(&self) -> usize {
        self.level.len()
    }

    /// Adds `k` variables, numbered after the others, at level zero.
    pub(in crate::ground) fn add_vars(&mut self, k: usize) {
        self.cancel_until(0);
        let n = self.num_vars() + k;
        self.watches.resize_with(2 * n, Vec::new);
        self.binaries.resize_with(2 * n, Vec::new);
        self.value.resize(2 * n, 0);
        self.level.resize(n, 0);
        self.reason.resize(n, NONE);
        self.activity.resize(n, 0.0);
        self.heap_pos.resize(n, NONE);
        self.phase.resize(n, false);
        self.seen.resize(n, false);
        for v in (n - k) as u32..n as u32 {
            self.heap_insert(v);
        }
    }

    fn decision_level(&self) -> usize {
        self.trail_lim.len()
    }

    /// Adds a clause, at level zero; false once the clauses are found
    /// unsatisfiable.
    pub(in crate::ground) fn add_clause(&mut self, lits: &[Lit]) -> bool {
        self.cancel_until(0);
        if !self.ok {
            return false;
        }
        let mut c: Vec<Lit> = lits.to_vec();
        c.sort_unstable();
        c.dedup();
        if c.windows(2).any(|w| w[0] ^ 1 == w[1]) || c.iter().any(|&l| self.value[l as usize] == 1)
        {
            return true;
        }
        c.retain(|&l| self.value[l as usize] == 0);
        match c.len() {
            0 => self.ok = false,
            1 => {
                self.enqueue(c[0], NONE);
                self.ok = self.propagate().is_none();
            }
            2 => {
                self.binaries[c[0] as usize].push(c[1]);
                self.binaries[c[1] as usize].push(c[0]);
            }
            _ => {
                self.attach(&c, 0);
            }
        }
        self.ok
    }

    /// Stores a clause of three literals or more and watches its first two.
    fn attach(&mut self, c: &[Lit], lbd: u32) -> u32 {
        let id = self.clauses.len() as u32;
        self.clauses.push(Clause {
            start: self.arena.len(),
            len: c.len() as u32,
            lbd,
        });
        self.arena.extend_from_slice(c);
        self.watches[c[0] as usize].push(Watch {
            clause: id,
            blocker: c[1],
        });
        self.watches[c[1] as usize].push(Watch {
            clause: id,
            blocker: c[0],
        });
        id
    }

    /// The value of `l` at level zero, `None` where it has none.
    pub(in crate::ground) fn fixed(&self, l: Lit) -> Option<bool> {
        let v = self.value[l as usize];
        (v != 0 && self.level[var(l)] == 0).then_some(v == 1)
    }

    /// Whether `l` holds in the model the last call found.
    pub(in crate::ground) fn holds(&self, l: Lit) -> bool {
        self.value[l as usize] == 1
    }

    /// Each clause of two literals once, given or learnt, of literals
    /// unassigned at level zero.
    pub(in crate::ground) fn binary_clauses(&self) -> Vec<(Lit, Lit)> {
        let mut out = Vec::new();
        for (a, others) in self.binaries.iter().enumerate() {
            let a = a as Lit;
            if self.fixed(a).is_some() {
                continue;
            }
            for &b in others {
                if a < b && self.fixed(b).is_none() {
                    out.push((a, b));
                }
            }
        }
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Looks for a model in which `assumptions` hold, within
    /// `conflict_limit` conflicts and the solver's limit on propagation
    /// steps.
    pub(in crate::ground) fn solve(&mut self, assumptions: &[Lit], conflict_limit: u64) -> Outcome {
        // The levels kept were propagated when the last call returned.
        let shared = if self.keep_assumptions {
            self.held
                .iter()
                .zip(assumptions)
                .take_while(|(a, b)| a == b)
                .count()
        } else {
            0
        };
        self.cancel_until(shared);
        if !self.ok {
            return Outcome::Unsat;
        }
        if self.decision_level() == 0 && self.propagate().is_some() {
            self.ok = false;
            return Outcome::Unsat;
        }
        if self.max_learnts == 0.0 {
            self.max_learnts = (self.clauses.len() as f64 / 3.0).max(2000.0);
        }
        let until = self.conflicts.saturating_add(conflict_limit);
        loop {
            let budget = (luby(self.restarts) * 100.0) as u64;
            self.restarts += 1;
            match self.search(assumptions, budget, until) {
                Some(outcome) => {
                    // Levels `1..=k` hold the first `k` assumptions.
                    let k = self.decision_level().min(assumptions.len());
                    self.held.clear();
                    self.held.extend_from_slice(&assumptions[..k]);
                    return outcome;
                }
                None => {
                    self.cancel_until(0);
                    if self.learnts as f64 >= self.max_learnts {
                        self.rebuild(true);
                        self.max_learnts *= 1.1;
                    } else if self.trail.len() > self.fixed_at_rebuild + 64 + self.trail.len() / 16
                    {
                        self.rebuild(false);
                    }
                }
            }
        }
    }

    /// Whether unit propagation of `assumptions`, each decided in turn,
    /// meets a conflict or an assumption already false: a refutation
    /// without search, as vivification asks of a clause. A call keeps the
    /// levels of the assumptions it shares with the last call's where the
    /// caller asks ([`Solver::keep_assumptions`]), and leaves those of the
    /// assumptions it propagated without a conflict.
    pub(in crate::ground) fn propagation_refutes(&mut self, assumptions: &[Lit]) -> bool {
        let shared = if self.keep_assumptions {
            self.held
                .iter()
                .zip(assumptions)
                .take_while(|(a, b)| a == b)
                .count()
        } else {
            0
        };
        self.cancel_until(shared);
        if !self.ok {
            return true;
        }
        if self.decision_level() == 0 && self.propagate().is_some() {
            self.ok = false;
            return true;
        }
        let mut refuted = false;
        while self.decision_level() < assumptions.len() {
            let p = assumptions[self.decision_level()];
            match self.value[p as usize] {
                1 => self.trail_lim.push(self.trail.len()),
                -1 => {
                    refuted = true;
                    break;
                }
                _ => {
                    self.trail_lim.push(self.trail.len());
                    self.enqueue(p, NONE);
                    if self.propagate().is_some() {
                        self.cancel_until(self.decision_level() - 1);
                        refuted = true;
                        break;
                    }
                }
            }
        }
        let k = self.decision_level().min(assumptions.len());
        self.held.clear();
        self.held.extend_from_slice(&assumptions[..k]);
        refuted
    }

    /// The search until `budget` conflicts, `None` for a restart.
    fn search(&mut self, assumptions: &[Lit], budget: u64, until: u64) -> Option<Outcome> {
        let mut here = 0u64;
        loop {
            if let Some(conflict) = self.propagate() {
                self.conflicts += 1;
                here += 1;
                if self.decision_level() == 0 {
                    self.ok = false;
                    return Some(Outcome::Unsat);
                }
                let (back, lbd) = self.analyze(conflict);
                self.cancel_until(back);
                let learnt = std::mem::take(&mut self.learnt);
                match learnt.len() {
                    1 => self.enqueue(learnt[0], NONE),
                    2 => {
                        self.binaries[learnt[0] as usize].push(learnt[1]);
                        self.binaries[learnt[1] as usize].push(learnt[0]);
                        self.enqueue(learnt[0], BINARY | learnt[1]);
                    }
                    _ => {
                        let id = self.attach(&learnt, lbd);
                        self.learnts += 1;
                        self.enqueue(learnt[0], id);
                    }
                }
                self.learnt = learnt;
                self.var_inc /= 0.95;
                continue;
            }
            if self.conflicts >= until || self.ticks >= self.tick_limit {
                return Some(Outcome::Unknown);
            }
            if here >= budget {
                return None;
            }
            let mut next = NONE;
            while self.decision_level() < assumptions.len() {
                let p = assumptions[self.decision_level()];
                match self.value[p as usize] {
                    1 => self.trail_lim.push(self.trail.len()),
                    -1 => return Some(Outcome::Unsat),
                    _ => {
                        next = p;
                        break;
                    }
                }
            }
            if next == NONE {
                next = loop {
                    match self.heap_pop() {
                        None => return Some(Outcome::Sat),
                        Some(v) if self.value[2 * v as usize] == 0 => {
                            break if self.phase[v as usize] {
                                2 * v
                            } else {
                                2 * v + 1
                            };
                        }
                        Some(_) => {}
                    }
                };
            }
            self.trail_lim.push(self.trail.len());
            self.enqueue(next, NONE);
        }
    }

    fn enqueue(&mut self, l: Lit, reason: u32) {
        self.value[l as usize] = 1;
        self.value[(l ^ 1) as usize] = -1;
        self.level[var(l)] = self.decision_level() as u32;
        self.reason[var(l)] = reason;
        self.trail.push(l);
    }

    fn cancel_until(&mut self, level: usize) {
        if self.decision_level() <= level {
            return;
        }
        let lim = self.trail_lim[level];
        for k in (lim..self.trail.len()).rev() {
            let l = self.trail[k];
            let v = var(l);
            self.value[l as usize] = 0;
            self.value[(l ^ 1) as usize] = 0;
            self.reason[v] = NONE;
            self.phase[v] = l & 1 == 0;
            if self.heap_pos[v] == NONE {
                self.heap_insert(v as u32);
            }
        }
        self.trail.truncate(lim);
        self.trail_lim.truncate(level);
        self.held.truncate(level);
        self.qhead = self.qhead.min(lim);
    }

    /// Unit propagation from the trail's head: the clause that fails, if
    /// one does.
    fn propagate(&mut self) -> Option<Conflict> {
        while self.qhead < self.trail.len() {
            let p = self.trail[self.qhead];
            self.qhead += 1;
            let f = p ^ 1;
            let others = std::mem::take(&mut self.binaries[f as usize]);
            self.ticks += 1 + others.len() as u64;
            let mut conflict = None;
            for &o in &others {
                match self.value[o as usize] {
                    1 => {}
                    0 => self.enqueue(o, BINARY | f),
                    _ => {
                        conflict = Some(Conflict::Binary(f, o));
                        break;
                    }
                }
            }
            self.binaries[f as usize] = others;
            if conflict.is_some() {
                return conflict;
            }
            let mut ws = std::mem::take(&mut self.watches[f as usize]);
            let (mut i, mut j) = (0, 0);
            while i < ws.len() {
                let w = ws[i];
                i += 1;
                self.ticks += 1;
                if self.value[w.blocker as usize] == 1 {
                    ws[j] = w;
                    j += 1;
                    continue;
                }
                let Clause { start, len, .. } = self.clauses[w.clause as usize];
                if self.arena[start] == f {
                    self.arena.swap(start, start + 1);
                }
                let first = self.arena[start];
                let kept = Watch {
                    clause: w.clause,
                    blocker: first,
                };
                if first != w.blocker && self.value[first as usize] == 1 {
                    ws[j] = kept;
                    j += 1;
                    continue;
                }
                let mut moved = false;
                for k in 2..len as usize {
                    let l = self.arena[start + k];
                    if self.value[l as usize] != -1 {
                        self.arena.swap(start + 1, start + k);
                        self.watches[l as usize].push(kept);
                        self.ticks += k as u64;
                        moved = true;
                        break;
                    }
                }
                if moved {
                    continue;
                }
                self.ticks += u64::from(len);
                ws[j] = kept;
                j += 1;
                if self.value[first as usize] == -1 {
                    while i < ws.len() {
                        ws[j] = ws[i];
                        j += 1;
                        i += 1;
                    }
                    ws.truncate(j);
                    self.watches[f as usize] = ws;
                    return Some(Conflict::Clause(w.clause));
                }
                self.enqueue(first, w.clause);
            }
            ws.truncate(j);
            self.watches[f as usize] = ws;
        }
        None
    }

    /// The literals of a conflict or a reason but the one it implies, the
    /// first of a clause or `implied` of a clause of two.
    fn for_each_other(&self, conflict: Conflict, implied: Option<Lit>, mut each: impl FnMut(Lit)) {
        match conflict {
            Conflict::Clause(c) => {
                let Clause { start, len, .. } = self.clauses[c as usize];
                let from = usize::from(implied.is_some());
                for k in from..len as usize {
                    each(self.arena[start + k]);
                }
            }
            Conflict::Binary(a, b) => match implied {
                Some(_) => each(b),
                None => {
                    each(a);
                    each(b);
                }
            },
        }
    }

    /// The reason of the literal `p` assigned true, as a conflict whose
    /// other literals [`for_each_other`](Self::for_each_other) reads.
    fn reason_of(&self, p: Lit) -> Conflict {
        let r = self.reason[var(p)];
        if r != NONE && r & BINARY != 0 {
            Conflict::Binary(p, r & !BINARY)
        } else {
            Conflict::Clause(r)
        }
    }

    /// First-UIP learning: the learnt clause in `self.learnt`, its
    /// asserting literal first and a literal of the level to go back to
    /// second, with that level and the clause's literal block distance.
    fn analyze(&mut self, conflict: Conflict) -> (usize, u32) {
        let level_now = self.decision_level() as u32;
        let mut learnt = std::mem::take(&mut self.learnt);
        learnt.clear();
        learnt.push(0);
        let mut path = 0usize;
        let mut p: Option<Lit> = None;
        let mut index = self.trail.len();
        let mut at = conflict;
        let mut met = std::mem::take(&mut self.stack);
        loop {
            met.clear();
            self.for_each_other(at, p, |q| met.push(q));
            for &q in &met {
                let v = var(q);
                if !self.seen[v] && self.level[v] > 0 {
                    self.seen[v] = true;
                    self.bump(v);
                    if self.level[v] >= level_now {
                        path += 1;
                    } else {
                        learnt.push(q);
                    }
                }
            }
            loop {
                index -= 1;
                if self.seen[var(self.trail[index])] {
                    break;
                }
            }
            let q = self.trail[index];
            self.seen[var(q)] = false;
            path -= 1;
            p = Some(q);
            if path == 0 {
                break;
            }
            at = self.reason_of(q);
        }
        self.stack = met;
        learnt[0] = p.expect("a literal of the conflict's level") ^ 1;
        // Recursive minimization: a literal implied by the others' reasons,
        // within levels the clause already has, goes.
        let abstraction = learnt[1..]
            .iter()
            .fold(0u64, |a, &l| a | 1 << (self.level[var(l)] & 63));
        self.to_clear.clear();
        self.to_clear.extend_from_slice(&learnt);
        let mut j = 1;
        for i in 1..learnt.len() {
            let l = learnt[i];
            if self.reason[var(l)] == NONE || !self.redundant(l, abstraction) {
                learnt[j] = l;
                j += 1;
            }
        }
        learnt.truncate(j);
        for k in 0..self.to_clear.len() {
            let v = var(self.to_clear[k]);
            self.seen[v] = false;
        }
        let mut back = 0;
        if learnt.len() > 1 {
            let mut max = 1;
            for i in 2..learnt.len() {
                if self.level[var(learnt[i])] > self.level[var(learnt[max])] {
                    max = i;
                }
            }
            learnt.swap(1, max);
            back = self.level[var(learnt[1])] as usize;
        }
        self.stamp += 1;
        if self.level_stamp.len() <= level_now as usize {
            self.level_stamp.resize(level_now as usize + 1, 0);
        }
        let mut lbd = 0;
        for &l in &learnt {
            let lv = self.level[var(l)] as usize;
            if self.level_stamp[lv] != self.stamp {
                self.level_stamp[lv] = self.stamp;
                lbd += 1;
            }
        }
        self.learnt = learnt;
        (back, lbd)
    }

    /// Whether the false literal `p` of the learnt clause follows from the
    /// clause's other literals through the reasons, within the levels of
    /// `abstraction`; marks what it visits in `seen` and `to_clear`.
    fn redundant(&mut self, p: Lit, abstraction: u64) -> bool {
        self.stack.clear();
        self.stack.push(p);
        let top = self.to_clear.len();
        let mut found = std::mem::take(&mut self.found);
        let mut redundant = true;
        'search: while let Some(q) = self.stack.pop() {
            found.clear();
            let implied = q ^ 1;
            self.for_each_other(self.reason_of(implied), Some(implied), |l| found.push(l));
            for &l in &found {
                let v = var(l);
                if self.seen[v] || self.level[v] == 0 {
                    continue;
                }
                if self.reason[v] != NONE && abstraction & (1 << (self.level[v] & 63)) != 0 {
                    self.seen[v] = true;
                    self.stack.push(l);
                    self.to_clear.push(l);
                } else {
                    for k in top..self.to_clear.len() {
                        let v = var(self.to_clear[k]);
                        self.seen[v] = false;
                    }
                    self.to_clear.truncate(top);
                    redundant = false;
                    break 'search;
                }
            }
        }
        self.found = found;
        redundant
    }

    /// Rebuilds the clause store at level zero: without the clauses level
    /// zero satisfies, without its false literals, and with `cut` without
    /// the worse half of the learnt clauses, by literal block distance,
    /// those of two levels or fewer kept.
    fn rebuild(&mut self, cut: bool) {
        debug_assert_eq!(self.decision_level(), 0);
        let mut keep = vec![true; self.clauses.len()];
        if cut {
            let mut learnt: Vec<u32> = (0..self.clauses.len() as u32)
                .filter(|&c| self.clauses[c as usize].lbd > 2)
                .collect();
            // The worse half by distance, the older first among equals.
            learnt.sort_by_key(|&c| (std::cmp::Reverse(self.clauses[c as usize].lbd), c));
            for &c in &learnt[..learnt.len() / 2] {
                keep[c as usize] = false;
            }
        }
        let old_arena = std::mem::take(&mut self.arena);
        let old = std::mem::take(&mut self.clauses);
        for w in &mut self.watches {
            w.clear();
        }
        self.learnts = 0;
        let mut c: Vec<Lit> = Vec::new();
        for (k, cl) in old.iter().enumerate() {
            if !keep[k] {
                continue;
            }
            let lits = &old_arena[cl.start..cl.start + cl.len as usize];
            if lits.iter().any(|&l| self.value[l as usize] == 1) {
                continue;
            }
            c.clear();
            c.extend(
                lits.iter()
                    .copied()
                    .filter(|&l| self.value[l as usize] == 0),
            );
            // Level zero is propagated, so no clause it leaves has fewer
            // than two literals; one that did would be a unit or the
            // empty clause all the same.
            match c.len() {
                0 => self.ok = false,
                1 => self.enqueue(c[0], NONE),
                2 => {
                    self.binaries[c[0] as usize].push(c[1]);
                    self.binaries[c[1] as usize].push(c[0]);
                }
                _ => {
                    self.attach(&c, cl.lbd);
                    if cl.lbd > 0 {
                        self.learnts += 1;
                    }
                }
            }
        }
        for a in 0..self.binaries.len() {
            if self.value[a] != 0 {
                self.binaries[a].clear();
            } else {
                let mut others = std::mem::take(&mut self.binaries[a]);
                others.retain(|&b| self.value[b as usize] == 0);
                self.binaries[a] = others;
            }
        }
        // The reasons of the units point into the old store; nothing reads
        // the reason of a literal at level zero.
        for &l in &self.trail {
            self.reason[var(l)] = NONE;
        }
        if self.ok && self.propagate().is_some() {
            self.ok = false;
        }
        self.fixed_at_rebuild = self.trail.len();
    }

    fn bump(&mut self, v: usize) {
        self.activity[v] += self.var_inc;
        if self.activity[v] > 1e100 {
            for a in &mut self.activity {
                *a *= 1e-100;
            }
            self.var_inc *= 1e-100;
        }
        let at = self.heap_pos[v];
        if at != NONE {
            self.heap_up(at as usize);
        }
    }

    fn heap_insert(&mut self, v: u32) {
        self.heap_pos[v as usize] = self.heap.len() as u32;
        self.heap.push(v);
        self.heap_up(self.heap.len() - 1);
    }

    fn heap_pop(&mut self) -> Option<u32> {
        let top = *self.heap.first()?;
        let last = self.heap.pop().expect("not empty");
        self.heap_pos[top as usize] = NONE;
        if !self.heap.is_empty() {
            self.heap[0] = last;
            self.heap_pos[last as usize] = 0;
            self.heap_down(0);
        }
        Some(top)
    }

    fn heap_up(&mut self, mut i: usize) {
        let v = self.heap[i];
        while i > 0 {
            let parent = (i - 1) / 2;
            let u = self.heap[parent];
            if self.activity[u as usize] >= self.activity[v as usize] {
                break;
            }
            self.heap[i] = u;
            self.heap_pos[u as usize] = i as u32;
            i = parent;
        }
        self.heap[i] = v;
        self.heap_pos[v as usize] = i as u32;
    }

    fn heap_down(&mut self, mut i: usize) {
        let v = self.heap[i];
        loop {
            let left = 2 * i + 1;
            if left >= self.heap.len() {
                break;
            }
            let right = left + 1;
            let child = if right < self.heap.len()
                && self.activity[self.heap[right] as usize]
                    > self.activity[self.heap[left] as usize]
            {
                right
            } else {
                left
            };
            let u = self.heap[child];
            if self.activity[u as usize] <= self.activity[v as usize] {
                break;
            }
            self.heap[i] = u;
            self.heap_pos[u as usize] = i as u32;
            i = child;
        }
        self.heap[i] = v;
        self.heap_pos[v as usize] = i as u32;
    }
}

/// The Luby sequence's `x`-th term, 1, 1, 2, 1, 1, 2, 4, ...
fn luby(x: u32) -> f64 {
    let (mut size, mut seq) = (1u64, 0i32);
    while size < u64::from(x) + 1 {
        seq += 1;
        size = 2 * size + 1;
    }
    let mut x = u64::from(x);
    while size - 1 != x {
        size = (size - 1) >> 1;
        seq -= 1;
        x %= size;
    }
    2f64.powi(seq)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DIMACS-style literals, 1-based and signed, as the solver's.
    fn lits(c: &[i32]) -> Vec<Lit> {
        c.iter()
            .map(|&x| 2 * (x.unsigned_abs() - 1) + u32::from(x < 0))
            .collect()
    }

    /// Whether the clauses have a model, by enumeration.
    fn satisfiable(n: usize, clauses: &[Vec<i32>], assumptions: &[i32]) -> bool {
        (0..1u64 << n).any(|w| {
            let holds = |x: i32| (w >> (x.unsigned_abs() - 1) & 1 == 1) == (x > 0);
            assumptions.iter().all(|&x| holds(x))
                && clauses.iter().all(|c| c.iter().any(|&x| holds(x)))
        })
    }

    /// A pseudo-random generator for the tests (xorshift).
    fn next(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    #[test]
    fn the_luby_sequence() {
        let terms: Vec<f64> = (0..15).map(luby).collect();
        assert_eq!(
            terms,
            [
                1.0, 1.0, 2.0, 1.0, 1.0, 2.0, 4.0, 1.0, 1.0, 2.0, 1.0, 1.0, 2.0, 4.0, 8.0
            ]
        );
    }

    #[test]
    fn the_pigeons_do_not_fit() {
        // Four pigeons in three holes: p(i, h) is variable 3 i + h + 1.
        let p = |i: i32, h: i32| 3 * i + h + 1;
        let mut s = Solver::new(12);
        for i in 0..4 {
            assert!(s.add_clause(&lits(&[p(i, 0), p(i, 1), p(i, 2)])));
        }
        for h in 0..3 {
            for i in 0..4 {
                for j in i + 1..4 {
                    s.add_clause(&lits(&[-p(i, h), -p(j, h)]));
                }
            }
        }
        assert_eq!(s.solve(&[], u64::MAX), Outcome::Unsat);
    }

    #[test]
    fn answers_agree_with_enumeration_under_assumptions() {
        // Random 3-CNFs over 12 variables around the threshold, each solved
        // under random assumptions; a model found is checked, an answer of
        // none against enumeration.
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        for round in 0..300 {
            let n = 12;
            let m = 40 + (next(&mut state) % 20) as usize;
            let clauses: Vec<Vec<i32>> = (0..m)
                .map(|_| {
                    (0..3)
                        .map(|_| {
                            let v = (next(&mut state) % n as u64) as i32 + 1;
                            if next(&mut state) & 1 == 1 { v } else { -v }
                        })
                        .collect()
                })
                .collect();
            let mut s = Solver::new(n);
            let mut ok = true;
            for c in &clauses {
                ok &= s.add_clause(&lits(c));
            }
            for _ in 0..4 {
                let k = (next(&mut state) % 3) as usize;
                let assumptions: Vec<i32> = (0..k)
                    .map(|_| {
                        let v = (next(&mut state) % n as u64) as i32 + 1;
                        if next(&mut state) & 1 == 1 { v } else { -v }
                    })
                    .collect();
                let want = satisfiable(n, &clauses, &assumptions);
                let got = s.solve(&lits(&assumptions), u64::MAX);
                assert_eq!(
                    got == Outcome::Sat,
                    want,
                    "round {round}: {clauses:?} under {assumptions:?}"
                );
                assert!(ok || got == Outcome::Unsat, "round {round}");
                if got == Outcome::Sat {
                    for c in &clauses {
                        assert!(
                            lits(c).iter().any(|&l| s.holds(l)),
                            "round {round}: the model satisfies {c:?}"
                        );
                    }
                    assert!(
                        lits(&assumptions).iter().all(|&l| s.holds(l)),
                        "round {round}: the model holds {assumptions:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn propagation_refutes_as_unit_propagation_does() {
        // Random CNFs of two and three literals, asked under sets of
        // assumptions that grow, shrink or change at their end, the levels
        // kept: a refutation is unit propagation's, by a fixpoint over the
        // clauses, and only where the assumptions have no model.
        let mut state = 0x6a09_e667_f3bc_c909u64;
        let mut refuted = 0;
        for case in 0..120 {
            let n = 6 + case % 11;
            let m = n * 2 + case % 7;
            let clauses: Vec<Vec<i32>> = (0..m)
                .map(|_| {
                    (0..2 + next(&mut state) % 2)
                        .map(|_| {
                            let v = (next(&mut state) % n as u64) as i32 + 1;
                            if next(&mut state) & 1 == 1 { v } else { -v }
                        })
                        .collect()
                })
                .collect();
            let mut s = Solver::new(n);
            s.keep_assumptions = true;
            let mut ok = true;
            for c in &clauses {
                ok &= s.add_clause(&lits(c));
            }
            let fixpoint = |assumptions: &[i32]| -> bool {
                let mut value = vec![0i8; n + 1];
                for &a in assumptions {
                    let v = a.unsigned_abs() as usize;
                    let want = if a > 0 { 1 } else { -1 };
                    if value[v] == -want {
                        return true;
                    }
                    value[v] = want;
                }
                loop {
                    let mut changed = false;
                    for c in &clauses {
                        let holds =
                            |x: i32| value[x.unsigned_abs() as usize] == if x > 0 { 1 } else { -1 };
                        if c.iter().any(|&x| holds(x)) {
                            continue;
                        }
                        let mut open: Vec<i32> = c
                            .iter()
                            .copied()
                            .filter(|&x| value[x.unsigned_abs() as usize] == 0)
                            .collect();
                        open.sort_unstable();
                        open.dedup();
                        match open.len() {
                            0 => return true,
                            1 => {
                                value[open[0].unsigned_abs() as usize] =
                                    if open[0] > 0 { 1 } else { -1 };
                                changed = true;
                            }
                            _ => {}
                        }
                    }
                    if !changed {
                        return false;
                    }
                }
            };
            let mut assumptions: Vec<i32> = Vec::new();
            for _ in 0..30 {
                match next(&mut state) % 3 {
                    0 if assumptions.len() < 6 => {
                        let v = (next(&mut state) % n as u64) as i32 + 1;
                        assumptions.push(if next(&mut state) & 1 == 1 { v } else { -v });
                    }
                    1 => {
                        assumptions.pop();
                    }
                    _ => {
                        if let Some(last) = assumptions.last_mut() {
                            *last = -*last;
                        }
                    }
                }
                let got = s.propagation_refutes(&lits(&assumptions));
                assert_eq!(
                    got,
                    !ok || fixpoint(&assumptions),
                    "case {case}: {clauses:?} under {assumptions:?}"
                );
                assert!(
                    !got || !satisfiable(n, &clauses, &assumptions),
                    "case {case}: refuted under {assumptions:?} with a model"
                );
                refuted += usize::from(got);
            }
        }
        assert!(refuted > 300, "refutations met: {refuted}");
    }

    #[test]
    fn calls_that_keep_the_levels_of_their_assumptions_agree_with_enumeration() {
        // Random 3-CNFs near the threshold, so that the solver learns,
        // minimizes and cuts clauses, each asked under 30 sets of
        // assumptions that grow, shrink or change at their end: every
        // answer against enumeration, every model found a model.
        let mut state = 0x2545_f491_4f6c_dd1du64;
        for case in 0..120 {
            let n = 6 + case % 13;
            let m = n * 4 + case % 5 - 2;
            let clauses: Vec<Vec<i32>> = (0..m)
                .map(|_| {
                    (0..3)
                        .map(|_| {
                            let v = (next(&mut state) % n as u64) as i32 + 1;
                            if next(&mut state) & 1 == 1 { v } else { -v }
                        })
                        .collect()
                })
                .collect();
            let mut s = Solver::new(n);
            s.keep_assumptions = true;
            for c in &clauses {
                s.add_clause(&lits(c));
            }
            s.max_learnts = 4.0;
            let mut assumptions: Vec<i32> = Vec::new();
            for _ in 0..30 {
                match next(&mut state) % 3 {
                    0 if assumptions.len() < 6 => {
                        let v = (next(&mut state) % n as u64) as i32 + 1;
                        assumptions.push(if next(&mut state) & 1 == 1 { v } else { -v });
                    }
                    1 => {
                        assumptions.pop();
                    }
                    _ => {
                        if let Some(last) = assumptions.last_mut() {
                            *last = -*last;
                        }
                    }
                }
                let want = satisfiable(n, &clauses, &assumptions);
                let got = s.solve(&lits(&assumptions), u64::MAX);
                assert_eq!(
                    got == Outcome::Sat,
                    want,
                    "case {case}: {clauses:?} under {assumptions:?}"
                );
                if got == Outcome::Sat {
                    for c in &clauses {
                        assert!(
                            lits(c).iter().any(|&l| s.holds(l)),
                            "case {case}: the model satisfies {c:?}"
                        );
                    }
                    assert!(
                        lits(&assumptions).iter().all(|&l| s.holds(l)),
                        "case {case}: the model holds {assumptions:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_call_stops_at_its_conflict_limit() {
        // Five pigeons in four holes takes more than one conflict.
        let p = |i: i32, h: i32| 4 * i + h + 1;
        let mut s = Solver::new(20);
        for i in 0..5 {
            s.add_clause(&lits(&[p(i, 0), p(i, 1), p(i, 2), p(i, 3)]));
        }
        for h in 0..4 {
            for i in 0..5 {
                for j in i + 1..5 {
                    s.add_clause(&lits(&[-p(i, h), -p(j, h)]));
                }
            }
        }
        assert_eq!(s.solve(&[], 1), Outcome::Unknown);
        assert_eq!(s.solve(&[], u64::MAX), Outcome::Unsat);
    }
}
