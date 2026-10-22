//! The and-inverter graph of a ground program's acyclic part, and the
//! merge of the gates it shows equal ([`reduce`]).
//!
//! A gate is an acyclic derived atom with rules. In dependency order each
//! becomes a literal of the graph: the OR of its bodies, each the AND of
//! its literals, built from two-input ANDs with complemented edges and
//! hashed on creation (structural hashing), with a body's literals and a
//! gate's bodies sorted, so that gates with the same bodies over equal
//! gates, or the complement's, are one node. The inputs are the facts and
//! the atoms of cyclic components; a derived atom without rules is false.
//!
//! Every node carries a signature, its values on [`WORDS`] words of 64
//! random assignments of the inputs, computed bit-parallel as the node is
//! created. A gate whose signature is an earlier candidate's, or its
//! complement, or constant, is tried against it, and the SAT solver
//! ([`super::preprocess::sat`]) proves the two equal where no assignment
//! of the inputs tells them apart, on the Tseitin clauses of their cones:
//! functional reduction, as an and-inverter graph package does it
//! (Kuehlmann's equivalence checking; Mishchenko, Chatterjee, Jiang and
//! Brayton's fraiging, 2005). One solver takes every call, each node's
//! clauses given once and what a call proves kept for the calls after. A
//! model that tells two apart is a counterexample, and every [`PATTERNS`]
//! of them are simulated as a word of their own, which splits the
//! candidates they tell apart. The search stops once its steps (clause
//! literals written and the solver's propagation steps) pass
//! [`STEPS_PER_LITERAL`] per body literal of the program, plus
//! [`STEPS_FLOOR`], or once [`PATTERNS`] counterexamples are found with no
//! candidate proved among them: the calls after would mostly tell apart
//! what simulation could not (bb_weekend_w30: 7 762 calls, none proved, a
//! second of the ground time; Roulette's hmm at 900 steps, 461 calls and
//! 4.7 M steps, none proved, against 64 calls). Where the calls still
//! prove, the search goes on within its steps, the counterexamples no
//! longer simulated once [`MAX_WORDS`] words are (dice_pmc1 written as
//! recursion: 1 088 gates proved where the search stopped at 770, 3 256
//! atoms left of 9 220). What it proved by then stands.
//!
//! The atoms of a cyclic component are inputs here, so gates proved equal
//! are equal however those atoms come out, and the facts are free,
//! whatever exclusive sets they are in: what is proved holds in every
//! world.

use rustc_hash::{FxHashMap, FxHashSet};

use super::preprocess::sat::{Lit as SatLit, Outcome, Solver};
use super::program::mix;
use super::simplify::{propagate_certain, restrict_to_relevant};
use super::structure::Structure;
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, RulesByHead, deps};

/// Words of random assignments simulated at first.
const WORDS: usize = 4;
/// Counterexamples gathered before they are simulated as a word.
const PATTERNS: usize = 64;
/// Words simulated at most, counterexamples' included.
const MAX_WORDS: usize = 16;
/// Candidates with a gate's signature tried against it.
const TRIES: usize = 2;
/// Conflicts after which a call gives up on its candidate.
const CONFLICTS: u64 = 100;
/// Steps of the search per body literal of the program.
const STEPS_PER_LITERAL: u64 = 64;
/// Steps the search may take however small the program.
const STEPS_FLOOR: u64 = 1 << 22;

/// A literal of the graph: node `n` is `2 n`, its complement `2 n + 1`.
/// Node 0 is the constant false.
type Lit = u32;
const FALSE: Lit = 0;
const TRUE: Lit = 1;
/// The fanins of the constant and of an input.
const NO: Lit = Lit::MAX;

fn node(l: Lit) -> usize {
    (l >> 1) as usize
}

/// What the sweep did, for `--stats`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SweepStats {
    /// Calls of the solver.
    pub calls: u64,
    /// Candidates proved equal, constants among them.
    pub proved: u64,
    /// Candidates a model told apart.
    pub refuted: u64,
    /// Candidates whose call gave up.
    pub open: u64,
    /// Steps taken: clause literals written and propagation steps.
    pub steps: u64,
}

impl SweepStats {
    /// Adds what another sweep did.
    pub(super) fn add(&mut self, other: SweepStats) {
        self.calls += other.calls;
        self.proved += other.proved;
        self.refuted += other.refuted;
        self.open += other.open;
        self.steps += other.steps;
    }
}

/// The graph: two-input ANDs over the inputs, each node with its
/// signature.
struct Graph {
    /// Each node's fanins; [`NO`] for the constant and the inputs.
    fanins: Vec<[Lit; 2]>,
    /// The node of each pair of fanins, the smaller first.
    strash: FxHashMap<[Lit; 2], Lit>,
    /// The signatures, `sim[w][n]` of node `n` on word `w`.
    sim: Vec<Vec<u64>>,
}

impl Graph {
    /// The graph of the constant alone, simulated on `words` words.
    fn new(words: usize) -> Self {
        Graph {
            fanins: vec![[NO, NO]],
            strash: FxHashMap::default(),
            sim: vec![vec![0]; words],
        }
    }

    /// Input `n`'s values on word `w`, drawn at random.
    fn random(w: usize, n: usize) -> u64 {
        mix(mix(w as u64) ^ n as u64)
    }

    /// `l`'s values on word `w`.
    fn value(&self, w: usize, l: Lit) -> u64 {
        self.sim[w][node(l)] ^ 0u64.wrapping_sub(u64::from(l & 1))
    }

    fn input(&mut self) -> Lit {
        let n = self.fanins.len();
        self.fanins.push([NO, NO]);
        for (w, word) in self.sim.iter_mut().enumerate() {
            word.push(Self::random(w, n));
        }
        2 * n as Lit
    }

    fn and(&mut self, a: Lit, b: Lit) -> Lit {
        let (a, b) = (a.min(b), a.max(b));
        if a == FALSE || a ^ 1 == b {
            return FALSE;
        }
        if a == TRUE || a == b {
            return b;
        }
        if let Some(&l) = self.strash.get(&[a, b]) {
            return l;
        }
        let n = self.fanins.len();
        self.fanins.push([a, b]);
        for w in 0..self.sim.len() {
            let v = self.value(w, a) & self.value(w, b);
            self.sim[w].push(v);
        }
        let l = 2 * n as Lit;
        self.strash.insert([a, b], l);
        l
    }

    /// The AND of `lits`, sorted in place: a chain in their order, so that
    /// equal sets of literals are one node.
    fn and_all(&mut self, lits: &mut Vec<Lit>) -> Lit {
        lits.sort_unstable();
        lits.dedup();
        if lits.windows(2).any(|w| w[0] ^ 1 == w[1]) {
            return FALSE;
        }
        lits.iter().fold(TRUE, |acc, &l| self.and(acc, l))
    }

    /// Simulates a word of its own: input `x` takes, in bit `k`, the value
    /// pattern `k` gives it, a random one where it gives none.
    fn add_word(&mut self, patterns: &[Vec<(usize, bool)>]) {
        let w = self.sim.len();
        let mut v: Vec<u64> = (0..self.fanins.len())
            .map(|n| if n == 0 { 0 } else { Self::random(w, n) })
            .collect();
        for (k, p) in patterns.iter().enumerate() {
            for &(x, value) in p {
                let bit = 1u64 << k;
                v[x] = if value { v[x] | bit } else { v[x] & !bit };
            }
        }
        let at = |v: &[u64], l: Lit| v[node(l)] ^ 0u64.wrapping_sub(u64::from(l & 1));
        for n in 1..v.len() {
            let [a, b] = self.fanins[n];
            if a != NO {
                v[n] = at(&v, a) & at(&v, b);
            }
        }
        self.sim.push(v);
    }

    /// `n`'s phase, the first bit of its signature, and the hash of its
    /// signature complemented where the phase is set: a node and its
    /// complement hash alike.
    fn signature(&self, n: usize) -> (u64, bool) {
        let phase = self.sim[0][n] & 1 == 1;
        let flip = 0u64.wrapping_sub(u64::from(phase));
        let h = self.sim.iter().fold(0, |h, w| mix(h ^ w[n] ^ flip));
        (h, phase)
    }

    /// Whether `m`'s signature is `n`'s or its complement.
    fn alike(&self, n: usize, m: usize) -> bool {
        let flip = (self.sim[0][n] ^ self.sim[0][m]) & 1;
        let flip = 0u64.wrapping_sub(flip);
        self.sim.iter().all(|w| w[n] ^ flip == w[m])
    }
}

/// The search for nodes equal in function: the candidates by signature,
/// and one solver for every call.
struct Sweep {
    /// The candidate nodes by the hash of their signatures.
    classes: FxHashMap<u64, Vec<usize>>,
    /// Every candidate, in the order added.
    candidates: Vec<usize>,
    /// The counterexamples not yet simulated: values of input nodes.
    pending: Vec<Vec<(usize, bool)>>,
    /// Steps left.
    budget: u64,
    /// The candidates proved when the counterexamples pending began.
    proved_before: u64,
    stats: SweepStats,
    /// The solver, over the clauses of the nodes encoded so far and what
    /// the calls proved; each node's positive literal there, [`NONE`]
    /// before it is encoded; and the input nodes encoded.
    solver: Solver,
    var: Vec<SatLit>,
    inputs: Vec<usize>,
}

/// No variable yet.
const NONE: SatLit = SatLit::MAX;

impl Sweep {
    /// The search over a program of `literals` body literals; the constant
    /// is the first candidate.
    fn new(g: &Graph, literals: u64) -> Self {
        let mut sweep = Sweep {
            classes: FxHashMap::default(),
            candidates: Vec::new(),
            pending: Vec::new(),
            budget: STEPS_FLOOR + STEPS_PER_LITERAL * literals,
            proved_before: 0,
            stats: SweepStats::default(),
            solver: Solver::new(0),
            var: Vec::new(),
            inputs: Vec::new(),
        };
        sweep.add(g, 0);
        sweep
    }

    /// Adds node `n` as a candidate for the nodes after it.
    fn add(&mut self, g: &Graph, n: usize) {
        self.classes.entry(g.signature(n).0).or_default().push(n);
        self.candidates.push(n);
    }

    /// A literal of an earlier candidate that `l` is proved equal to,
    /// within the search's steps.
    fn find(&mut self, g: &mut Graph, l: Lit) -> Option<Lit> {
        if self.budget == 0 {
            return None;
        }
        let n = node(l);
        let (h, phase) = g.signature(n);
        let tries: Vec<usize> = self
            .classes
            .get(&h)?
            .iter()
            .copied()
            .filter(|&c| c != n && g.alike(n, c))
            .take(TRIES)
            .collect();
        for c in tries {
            if self.budget == 0 {
                return None;
            }
            // `l` is `c` where their phases agree, its complement otherwise.
            let c_phase = g.sim[0][c] & 1 == 1;
            let target = 2 * c as Lit + Lit::from(phase ^ c_phase ^ (l & 1 == 1));
            match self.prove(g, l, target) {
                Some(true) => {
                    self.stats.proved += 1;
                    return Some(target);
                }
                Some(false) => self.stats.refuted += 1,
                None => self.stats.open += 1,
            }
        }
        None
    }

    /// Whether no assignment of the inputs tells `a` and `b` apart; `None`
    /// where the call gave up. A model that tells them apart is kept as a
    /// counterexample. What is proved stays with the solver.
    fn prove(&mut self, g: &mut Graph, a: Lit, b: Lit) -> Option<bool> {
        let va = self.encode(g, a)?;
        let vb = self.encode(g, b)?;
        // d where the two differ, assumed for this call and then denied.
        let d = 2 * self.solver.num_vars() as SatLit;
        self.solver.add_vars(1);
        self.solver.add_clause(&[d ^ 1, va, vb]);
        self.solver.add_clause(&[d ^ 1, va ^ 1, vb ^ 1]);
        let before = self.solver.ticks;
        self.solver.tick_limit = before.saturating_add(self.budget);
        self.stats.calls += 1;
        let outcome = self.solver.solve(&[d], CONFLICTS);
        let ticks = (self.solver.ticks - before).min(self.budget);
        self.budget -= ticks;
        self.stats.steps += ticks;
        // The clauses have models (every assignment of the inputs extends
        // to one), so a solver that finds none without the assumption has
        // a fault, and nothing it says is taken.
        let mut consistent = self.solver.add_clause(&[d ^ 1]);
        match outcome {
            Outcome::Unsat => {
                consistent &= self.solver.add_clause(&[va ^ 1, vb]);
                consistent &= self.solver.add_clause(&[va, vb ^ 1]);
                debug_assert!(consistent, "the sweep's clauses have no model");
                if !consistent {
                    self.budget = 0;
                    return None;
                }
                Some(true)
            }
            Outcome::Unknown => None,
            Outcome::Sat => {
                let pattern = self
                    .inputs
                    .iter()
                    .map(|&x| (x, self.solver.holds(self.var[x])))
                    .collect();
                self.counterexample(g, pattern);
                Some(false)
            }
        }
    }

    /// The solver's literal of `l`, the clauses of the nodes of its cone
    /// given where they were not yet. `None` where the steps run out.
    fn encode(&mut self, g: &Graph, l: Lit) -> Option<SatLit> {
        if self.var.len() < g.fanins.len() {
            self.var.resize(g.fanins.len(), NONE);
        }
        let root = node(l);
        if self.var[root] == NONE {
            let mut vars = self.solver.num_vars() as SatLit;
            let mut clauses: Vec<[SatLit; 3]> = Vec::new();
            let mut todo = vec![root];
            self.var[root] = 2 * vars;
            vars += 1;
            while let Some(n) = todo.pop() {
                let v = self.var[n];
                let [a, b] = g.fanins[n];
                if n == 0 {
                    clauses.push([v ^ 1, NONE, NONE]);
                    continue;
                }
                if a == NO {
                    self.inputs.push(n);
                    continue;
                }
                for f in [a, b] {
                    if self.var[node(f)] == NONE {
                        self.var[node(f)] = 2 * vars;
                        vars += 1;
                        todo.push(node(f));
                    }
                }
                let (va, vb) = (self.var[node(a)] ^ (a & 1), self.var[node(b)] ^ (b & 1));
                clauses.push([v ^ 1, va, NONE]);
                clauses.push([v ^ 1, vb, NONE]);
                clauses.push([v, va ^ 1, vb ^ 1]);
            }
            let written = 7 * (clauses.len() as u64) / 3;
            if written >= self.budget {
                self.budget = 0;
                return None;
            }
            self.budget -= written;
            self.stats.steps += written;
            self.solver.add_vars(vars as usize - self.solver.num_vars());
            for c in &clauses {
                let k = c.iter().position(|&x| x == NONE).unwrap_or(3);
                self.solver.add_clause(&c[..k]);
            }
        }
        Some(self.var[root] ^ (l & 1))
    }

    /// Keeps a counterexample, and once there are [`PATTERNS`] of them stops
    /// the search where no candidate was proved since the last were, and
    /// otherwise simulates them as a word of their own, which splits the
    /// classes, or drops them where [`MAX_WORDS`] are simulated.
    fn counterexample(&mut self, g: &mut Graph, pattern: Vec<(usize, bool)>) {
        self.pending.push(pattern);
        if self.pending.len() < PATTERNS {
            return;
        }
        let pending = std::mem::take(&mut self.pending);
        let before = std::mem::replace(&mut self.proved_before, self.stats.proved);
        if before == self.stats.proved {
            self.budget = 0;
            return;
        }
        if g.sim.len() >= MAX_WORDS {
            return;
        }
        g.add_word(&pending);
        self.classes.clear();
        for &c in &self.candidates {
            self.classes.entry(g.signature(c).0).or_default().push(c);
        }
    }
}

/// Merge each gate that the graph shows equal to an earlier atom into
/// it. The gates are taken in `structure`'s dependency order, each built
/// over the literals of the gates before it, so one pass finds every copy
/// of an acyclic atom; the atoms of cyclic components are inputs, those
/// the grounder introduced merged before ([`super::simplify::merge_equal_atoms`]).
///
/// A gate built as the node of an earlier gate is that gate's copy, its
/// rules the same bodies over equal gates: structural hashing, so that a
/// circuit's gate is compiled once and read by the readers of all its
/// copies. So is a gate built as the node of an input after a gate that
/// was: the first gate stays, and the copies are merged into it. Alea.jl's circuits repeat gates: of pi's 6 635 at 12 bits,
/// 4 941 are copies, and hold three quarters of the pairs a compile
/// builds (a fourth of the work after the merge in `program`'s order).
///
/// A gate alone in its predicate or of a recursive predicate, and in no
/// exclusive set, as a circuit's gates and a loop's steps are, may also be
/// proved equal in function to an earlier such
/// gate or fact, or its complement, or a constant (the sweep); it is then
/// merged into it, read through a negated literal where it is the
/// complement, or keeps an empty body or none where it is a constant.
/// conjugate_gaussians_w19's compile work halves (1 334 derived atoms to
/// 1 028).
///
/// An atom that a query or the evidence names is not merged away, nor one
/// of an exclusive set of a predicate with several atoms, a network
/// variable's value: merged into another variable's value,
/// bn_hailfinder's `scnrelplfcst(b)` split its variable and grew the
/// families layout's largest formula tenfold. Other atoms may be merged
/// into them. The sweep leaves out the atoms of predicates with several
/// atoms and the members of exclusive sets of several atoms: merged into
/// a switch fact or another value, uwcse4's values and
/// dice_digitRecognition's took the tables' structure apart (twice the
/// work and more). It takes those of a recursive predicate, a chain's
/// steps where a loop is written as recursion over a time index: one
/// predicate holds a value at every step, as the unrolled loop's gates
/// hold one each (dice_pmc1: 28 658 atoms to 11 272, a tenth of the
/// compile work). The gates are simulated over the rules first
/// ([`may_merge`]), and the graph is built only where a gate that may be
/// merged away has a signature in common with another atom or a constant.
/// Returns whether atoms were merged away; their rules are dropped and
/// their readers read the literal they were merged into.
pub(crate) fn reduce(gp: &mut GroundProgram, structure: &Structure) -> bool {
    let derived = gp.num_derived();
    let n = gp.atoms.len();
    let mut kept = vec![false; n];
    for a in gp.named_atoms() {
        kept[a as usize] = true;
    }
    let mut in_set = vec![false; n];
    for set in &structure.sets {
        for &a in set {
            kept[a as usize] |= structure.atoms_of[structure.predicate_of[a as usize]] > 1;
            in_set[a as usize] |= set.len() > 1;
        }
    }
    let by_head = &structure.by_head;
    let gates: Vec<AtomId> = structure
        .sccs
        .iter()
        .filter_map(|scc| match *scc.as_slice() {
            [a] if !by_head[a as usize].is_empty() && !deps::is_cyclic(gp, by_head, scc) => Some(a),
            _ => None,
        })
        .collect();
    let mut gate = vec![false; n];
    for &a in &gates {
        gate[a as usize] = true;
    }
    if !may_merge(gp, by_head, &gates, &gate, &kept) {
        return false;
    }
    // The atoms the sweep takes: alone in their predicates (a fact by its
    // signature) or of a recursive predicate, and in no exclusive set of
    // several atoms.
    let swept: Vec<bool> = {
        let mut facts_of: FxHashMap<(&str, usize), usize> = FxHashMap::default();
        for (a, kind) in gp.kinds.iter().enumerate() {
            if matches!(kind, AtomKind::Fact(_)) {
                *facts_of.entry(gp.atoms[a].signature()).or_default() += 1;
            }
        }
        let recursive = deps::recursive_predicates(gp);
        (0..n)
            .map(|a| {
                !in_set[a]
                    && match structure.predicate_of[a] {
                        usize::MAX => facts_of.get(&gp.atoms[a].signature()) == Some(&1),
                        p => structure.atoms_of[p] == 1 || recursive[a],
                    }
            })
            .collect()
    };
    let sweeping = gates.iter().any(|&a| swept[a as usize]);
    let mut g = Graph::new(if sweeping { WORDS } else { 0 });
    let mut sweep = sweeping.then(|| {
        let literals: u64 = gp.rules.iter().map(|r| r.body.len() as u64).sum();
        Sweep::new(&g, literals)
    });
    // Each atom's literal in the graph, built when first read; the atom
    // literal each node stands for, its first gate's or input's; and its
    // first gate's, where that is not the first atom: a bit that a switch
    // sets and every step after carries unchanged is that switch's literal,
    // which the gates are not merged into, and each step's copy of the bit
    // is merged into the first (dice_pmc2 written as recursion: 854 of its
    // 915 gates left were such copies).
    let mut lit: Vec<Lit> = vec![NO; n];
    let mut owner: Vec<Option<GLit>> = vec![None];
    let mut first_gate: Vec<Option<GLit>> = vec![None];
    // The literal each node is proved equal to, [`NO`] where none: a
    // later gate built as the node is merged as the gate the sweep merged,
    // where the sweep takes it.
    let mut proved: Vec<Lit> = vec![NO];
    // The literal each atom is merged into, and the value of each atom
    // proved constant.
    let mut into: Vec<GLit> = (0..n as AtomId).map(GLit::Pos).collect();
    let mut fixed: Vec<Option<bool>> = vec![None; n];
    let (mut body, mut terms) = (Vec::new(), Vec::new());
    let mut merged = 0;
    for &a in &gates {
        terms.clear();
        for &ri in &by_head[a as usize] {
            body.clear();
            for &l in &gp.rules[ri].body {
                let x = l.atom() as usize;
                if lit[x] == NO {
                    // An input, or false.
                    debug_assert!(!gate[x], "gate {x} read before it is built");
                    lit[x] = if gp.kinds[x] == AtomKind::Derived && by_head[x].is_empty() {
                        FALSE
                    } else {
                        let i = g.input();
                        owner.resize(g.fanins.len(), None);
                        owner[node(i)] = Some(GLit::Pos(x as AtomId));
                        if let Some(sweep) = sweep.as_mut().filter(|_| swept[x]) {
                            sweep.add(&g, node(i));
                        }
                        i
                    };
                }
                body.push(lit[x] ^ Lit::from(!l.is_positive()));
            }
            terms.push(g.and_all(&mut body) ^ 1);
        }
        let mut l = g.and_all(&mut terms) ^ 1;
        owner.resize(g.fanins.len(), None);
        first_gate.resize(g.fanins.len(), None);
        proved.resize(g.fanins.len(), NO);
        if proved[node(l)] != NO && swept[a as usize] {
            l = proved[node(l)] ^ (l & 1);
        }
        lit[a as usize] = l;
        // The atom literal the gate is equal to: the node's first atom,
        // where its graph literal is the gate's or its complement, or the
        // constant.
        let as_atom = |t: Lit, owner: &[Option<GLit>]| -> Option<GLit> {
            owner[node(t)].map(|o| if t & 1 == 0 { o } else { o.negated() })
        };
        if !kept[a as usize] {
            let structural = if node(l) == 0 {
                (l == FALSE || swept[a as usize]).then_some(Err(l == TRUE))
            } else {
                // A copy of an earlier gate is merged whatever it is,
                // also where an input stands for the node; its
                // complement, or an input, only where the sweep would
                // merge them.
                as_atom(l, &owner)
                    .filter(|o| {
                        (o.is_positive() && gate[o.atom() as usize])
                            || (swept[a as usize] && swept[o.atom() as usize])
                    })
                    .or_else(|| as_atom(l, &first_gate).filter(|o| o.is_positive()))
                    .map(Ok)
            };
            let equal = structural.or_else(|| {
                let sweep = sweep.as_mut().filter(|_| swept[a as usize])?;
                let t = sweep.find(&mut g, l)?;
                proved.resize(g.fanins.len(), NO);
                proved[node(l)] = t ^ (l & 1);
                lit[a as usize] = t;
                Some(if node(t) == 0 {
                    Err(t == TRUE)
                } else {
                    Ok(as_atom(t, &owner).expect("a candidate stands for an atom"))
                })
            });
            match equal {
                Some(Ok(o)) => into[a as usize] = o,
                Some(Err(v)) => fixed[a as usize] = Some(v),
                None => {}
            }
            if equal.is_some() {
                merged += 1;
                continue;
            }
        }
        if node(l) != 0 && owner[node(l)].is_none() {
            owner[node(l)] = Some(GLit::new(a, l & 1 == 0));
            if let Some(sweep) = sweep.as_mut().filter(|_| swept[a as usize]) {
                sweep.add(&g, node(l));
            }
        }
        if node(l) != 0 && first_gate[node(l)].is_none() {
            first_gate[node(l)] = Some(GLit::new(a, l & 1 == 0));
        }
    }
    if let Some(sweep) = sweep {
        gp.stats.swept.add(sweep.stats);
    }
    if merged == 0 {
        return false;
    }
    rewrite(gp, &into, &fixed);
    restrict_to_relevant(gp);
    gp.stats.merged += merged;
    gp.stats.reduced += derived.saturating_sub(gp.num_derived());
    true
}

/// Whether a gate that is not `kept` has the signature of a constant, or
/// of an input or an earlier gate, or its complement: its values on
/// [`WORDS`] words of random assignments of the inputs, simulated over
/// the rules in dependency order, a derived atom without rules false.
/// [`reduce`] merges only a gate that is not kept, into an atom or a
/// constant equal to it in function, which has its signature; where no
/// gate has one, the graph would find nothing to merge and is not built.
/// The Bayesian networks' gates are their variables' values, all kept,
/// and the graph over them took a fourth of their ground time
/// (bn_diabetes 0.112 s to 0.152 s, bn_barley 0.191 s to 0.248 s); a
/// circuit has a copy among its first gates, where the simulation stops.
fn may_merge(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    gates: &[AtomId],
    gate: &[bool],
    kept: &[bool],
) -> bool {
    // A gate after the last that may be merged away is no earlier atom of
    // any that may: the simulation stops there.
    let Some(last) = gates.iter().rposition(|&a| !kept[a as usize]) else {
        return false;
    };
    // A signature complemented where its first bit is set, so that a
    // function and its complement are one key.
    let key = |s: &[u64; WORDS]| s.map(|w| w ^ 0u64.wrapping_sub(s[0] & 1));
    let mut sim = vec![[0u64; WORDS]; gp.atoms.len()];
    let mut seen: FxHashSet<[u64; WORDS]> = FxHashSet::default();
    for (a, s) in sim.iter_mut().enumerate() {
        if !gate[a] && (gp.kinds[a] != AtomKind::Derived || !by_head[a].is_empty()) {
            *s = std::array::from_fn(|w| Graph::random(w, a));
            seen.insert(key(s));
        }
    }
    for &a in &gates[..=last] {
        let mut s = [0u64; WORDS];
        for &ri in &by_head[a as usize] {
            let mut t = [!0u64; WORDS];
            for &l in &gp.rules[ri].body {
                let flip = 0u64.wrapping_sub(u64::from(!l.is_positive()));
                let v = &sim[l.atom() as usize];
                for w in 0..WORDS {
                    t[w] &= v[w] ^ flip;
                }
            }
            for w in 0..WORDS {
                s[w] |= t[w];
            }
        }
        sim[a as usize] = s;
        let k = key(&s);
        let known = !seen.insert(k);
        if !kept[a as usize] && (known || k == [0; WORDS]) {
            return true;
        }
    }
    false
}

/// `gp` with each atom's readers reading the literal it is merged into
/// (`into`), and each atom proved constant (`fixed`) left with an empty
/// body where true and no rules where false; the rules of the atoms merged
/// away are dropped. A body that then reads an atom both ways never
/// holds, and a rule its head has already is dropped; only a head with a
/// rule that reads a merged atom may be left with such a body or rule.
/// The propagation carries the constants to their readers.
fn rewrite(gp: &mut GroundProgram, into: &[GLit], fixed: &[Option<bool>]) {
    let to = |l: GLit| {
        let t = into[l.atom() as usize];
        if l.is_positive() { t } else { t.negated() }
    };
    gp.rules
        .retain(|r| into[r.head as usize] == GLit::Pos(r.head) && fixed[r.head as usize].is_none());
    for (a, v) in fixed.iter().enumerate() {
        if *v == Some(true) {
            gp.rules.push(GroundRule::new(a as AtomId, Vec::new()));
        }
    }
    let mut changed = vec![false; gp.atoms.len()];
    for r in &gp.rules {
        if r.body.iter().any(|&l| to(l) != l) {
            changed[r.head as usize] = true;
        }
    }
    let mut lits = FxHashSet::default();
    let mut rules = FxHashSet::default();
    gp.rules.retain_mut(|r| {
        if !changed[r.head as usize] {
            return true;
        }
        lits.clear();
        r.body = r
            .body
            .iter()
            .map(|&l| to(l))
            .filter(|&l| lits.insert(l))
            .collect();
        if r.body.iter().any(|l| lits.contains(&l.negated())) {
            return false;
        }
        let mut key = r.body.clone();
        key.sort_unstable();
        rules.insert((r.head, key))
    });
    propagate_certain(gp);
}

/// The constants of the gates of a recursive stratum's atoms, built and
/// swept as the grounder completes them level by level
/// ([`super::engine`]'s `evaluate_by_levels`): the sweep proves a gate
/// constant over the complete atoms before it, which its readers then
/// need not be grounded on. One graph and one sweep serve the grounding,
/// its budget growing with the literals of the gates built.
pub(super) struct Pruner {
    g: Graph,
    sweep: Sweep,
    /// Each atom's literal in the graph, [`NO`] where it has none yet.
    lit: Vec<Lit>,
    /// The nodes the sweep has as candidates.
    added: FxHashSet<usize>,
}

impl Pruner {
    pub(super) fn new() -> Self {
        let g = Graph::new(WORDS);
        let sweep = Sweep::new(&g, 0);
        Pruner {
            g,
            sweep,
            lit: Vec::new(),
            added: FxHashSet::default(),
        }
    }

    fn slot(&mut self, a: AtomId) -> &mut Lit {
        let a = a as usize;
        if a >= self.lit.len() {
            self.lit.resize(a + 1, NO);
        }
        &mut self.lit[a]
    }

    /// Atom `a` an input of its own.
    pub(super) fn input(&mut self, a: AtomId) {
        let i = self.g.input();
        self.sweep.add(&self.g, node(i));
        *self.slot(a) = i;
    }

    /// The literal of atom `x` read in a body: its own, false where
    /// `absent` says it can have none (a complete atom never derived), a
    /// new input otherwise.
    fn read(&mut self, x: AtomId, absent: &impl Fn(AtomId) -> bool) -> Lit {
        let l = *self.slot(x);
        if l != NO {
            return l;
        }
        let l = if absent(x) {
            FALSE
        } else {
            let i = self.g.input();
            self.sweep.add(&self.g, node(i));
            i
        };
        *self.slot(x) = l;
        l
    }

    /// The gate of atom `a`, whose rules are `bodies`, all complete; its
    /// value where it is proved constant.
    pub(super) fn gate(
        &mut self,
        a: AtomId,
        bodies: &[&[GLit]],
        absent: &impl Fn(AtomId) -> bool,
    ) -> Option<bool> {
        let mut terms = Vec::with_capacity(bodies.len());
        let mut literals = 0u64;
        for body in bodies {
            let mut lits: Vec<Lit> = Vec::with_capacity(body.len());
            for l in body.iter() {
                let x = self.read(l.atom(), absent);
                lits.push(x ^ Lit::from(!l.is_positive()));
            }
            literals += body.len() as u64;
            terms.push(self.g.and_all(&mut lits) ^ 1);
        }
        self.sweep.budget += STEPS_PER_LITERAL * literals;
        let mut l = self.g.and_all(&mut terms) ^ 1;
        if node(l) != 0 {
            if let Some(t) = self.sweep.find(&mut self.g, l) {
                l = t;
            } else if self.added.insert(node(l)) {
                self.sweep.add(&self.g, node(l));
            }
        }
        *self.slot(a) = l;
        (node(l) == 0).then_some(l == TRUE)
    }
}
