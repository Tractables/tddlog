//! The ground program, the one representation every pass after the
//! engine reads and rewrites: atoms, their kinds, rules, queries and
//! evidence, with the indexes the passes share.

use std::fmt;
use std::sync::{Arc, OnceLock};

use crate::eval::Count;
use crate::syntax::Atom;

use super::structure::Structure;
use super::{Conditioned, Decisions, Defined, Encoding, GroundOptions};

/// Index of a ground atom in [`GroundProgram::atoms`].
pub(crate) type AtomId = u32;

/// A ground body literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GLit {
    Pos(AtomId),
    Neg(AtomId),
}

impl GLit {
    pub fn atom(self) -> AtomId {
        match self {
            GLit::Pos(a) | GLit::Neg(a) => a,
        }
    }
    pub fn is_positive(self) -> bool {
        matches!(self, GLit::Pos(_))
    }
    /// The literal of the other sign on the same atom.
    pub fn negated(self) -> GLit {
        match self {
            GLit::Pos(a) => GLit::Neg(a),
            GLit::Neg(a) => GLit::Pos(a),
        }
    }
    /// The literal of `atom` that is true where `atom` is `positive`.
    pub(crate) fn new(atom: AtomId, positive: bool) -> GLit {
        if positive {
            GLit::Pos(atom)
        } else {
            GLit::Neg(atom)
        }
    }
    /// The literal's bits mixed ([`mix`]), its share of a hash of a set of
    /// literals that does not depend on their order.
    pub(crate) fn mixed(self) -> u64 {
        mix(u64::from(self.atom()) << 1 | u64::from(self.is_positive()))
    }
}

/// `x`'s bits mixed (splitmix64): a hash of a set is the sum of its
/// members' mixes, which does not depend on their order.
pub(crate) fn mix(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// `head :- body` with all atoms ground. An empty body is a certain fact.
///
/// A rule of `pairs` stands for the rules `head :- a, b` of each pair of
/// its body's literals, `a` before `b`, ordered by `a` and then by `b`:
/// three or more positive literals, of which the rule holds where two do.
/// A link holds so where more than one of a set's copies does
/// ([`crate::indicators`]); a rule per pair was most of the rules of a
/// network read through clause tables.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GroundRule {
    pub head: AtomId,
    pub body: Vec<GLit>,
    pub pairs: bool,
}

impl GroundRule {
    /// The rule `head :- body`.
    pub(crate) fn new(head: AtomId, body: Vec<GLit>) -> Self {
        GroundRule {
            head,
            body,
            pairs: false,
        }
    }

    /// The rules the rule stands for: itself, or one per pair.
    pub(crate) fn clauses(&self) -> usize {
        if self.pairs {
            self.body.len() * (self.body.len() - 1) / 2
        } else {
            1
        }
    }

    /// The bodies of the rules the rule stands for, in their order.
    pub(crate) fn bodies(&self) -> impl Iterator<Item = RuleBody<'_>> {
        let pairs = self.pairs.then(|| {
            self.body.iter().enumerate().flat_map(|(j, &a)| {
                self.body[j + 1..]
                    .iter()
                    .map(move |&b| RuleBody::Pair([a, b]))
            })
        });
        let whole = (!self.pairs).then_some(RuleBody::Whole(&self.body));
        pairs.into_iter().flatten().chain(whole)
    }
}

/// A body of the rules a rule stands for ([`GroundRule::bodies`]): the
/// rule's own, or one of its pairs.
pub(crate) enum RuleBody<'a> {
    Whole(&'a [GLit]),
    Pair([GLit; 2]),
}

impl std::ops::Deref for RuleBody<'_> {
    type Target = [GLit];
    fn deref(&self) -> &[GLit] {
        match self {
            RuleBody::Whole(body) => body,
            RuleBody::Pair(pair) => pair,
        }
    }
}

/// The ground rules kept so far, each once, by hash: the last kept with
/// each hash, and per rule kept its index and the one kept before it with
/// the same hash. A rule is kept without a second copy of it: a set of the
/// rules themselves cost a copy of each, ace_blockmap_22_03's 182 k
/// deterministic rows among them ([`table`]).
#[derive(Default)]
pub(super) struct KeptRules {
    by_hash: rustc_hash::FxHashMap<u64, u32>,
    kept: Vec<(u32, u32)>,
}

impl KeptRules {
    /// Room for `n` rules kept.
    pub(super) fn with_capacity(n: usize) -> Self {
        let mut by_hash = rustc_hash::FxHashMap::default();
        by_hash.reserve(n);
        KeptRules {
            by_hash,
            kept: Vec::with_capacity(n),
        }
    }

    /// Whether `rule` is none of the rules kept, which are rules of
    /// `rules`; where it is none, it is kept as the next of `rules`, which
    /// the caller pushes it onto.
    pub(super) fn keep(&mut self, rules: &[GroundRule], rule: &GroundRule) -> bool {
        use std::collections::hash_map::Entry;
        use std::hash::BuildHasher;
        let h = rustc_hash::FxBuildHasher.hash_one(rule);
        // One probe of the table, for the rules of the hash and for the
        // rule's place.
        let entry = self.by_hash.entry(h);
        let last = match &entry {
            Entry::Occupied(o) => *o.get(),
            Entry::Vacant(_) => u32::MAX,
        };
        let mut k = last;
        while k != u32::MAX {
            let (r, before) = self.kept[k as usize];
            if rules[r as usize] == *rule {
                return false;
            }
            k = before;
        }
        entry.insert_entry(self.kept.len() as u32);
        self.kept.push((rules.len() as u32, last));
        true
    }
}

/// A fact's weights, true and false. They sum to one, and each is a number
/// of its own: the weight false is not taken as `1 - p` from the weight
/// true. A switch of an annotated disjunction is true with `p_i / S_i` and
/// false with `S_{i+1} / S_i` ([`chain_weights`]), and where the heads after
/// head `i` hold under the rounding of `p_i / S_i` against one, `1 - p_i /
/// S_i` is zero and their mass is lost.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Prob {
    /// The weight of the fact true, its probability.
    pub(crate) p: f64,
    /// The weight of the fact false.
    pub(crate) not: f64,
}

impl Prob {
    /// A fact of probability one half.
    pub(crate) const HALF: Prob = Prob { p: 0.5, not: 0.5 };
    /// A certain fact.
    pub(crate) const TRUE: Prob = Prob { p: 1.0, not: 0.0 };
    /// An impossible fact.
    pub(crate) const FALSE: Prob = Prob { p: 0.0, not: 1.0 };

    /// The weights of a fact of probability `p`: `p` and `1 - p`.
    pub(crate) fn new(p: f64) -> Prob {
        Prob { p, not: 1.0 - p }
    }

    /// The weight of the fact's literal, positive or negative.
    pub(crate) fn of(self, positive: bool) -> f64 {
        if positive { self.p } else { self.not }
    }

    /// The weights of the fact's literal: these for the fact, swapped for
    /// its negation.
    pub(crate) fn literal(self, positive: bool) -> Prob {
        if positive {
            self
        } else {
            Prob {
                p: self.not,
                not: self.p,
            }
        }
    }

    /// The weights of a conjunction of independent literals of weights
    /// `literals` (each the literal's weight and its negation's): their
    /// product, and its complement as the sum over `k` of the product of
    /// the first `k` literals' weights times the negation of literal `k + 1`,
    /// which `1 - p` would lose where `p` is near one.
    pub(crate) fn cube(literals: impl IntoIterator<Item = Prob>) -> Prob {
        let mut out = Prob::TRUE;
        for l in literals {
            out.not += out.p * l.not;
            out.p *= l.p;
        }
        out
    }

    /// The bits of both weights, a key that tells two facts apart where
    /// either weight differs.
    pub(crate) fn bits(self) -> (u64, u64) {
        (self.p.to_bits(), self.not.to_bits())
    }
}

/// What kind of ground atom an [`AtomId`] denotes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AtomKind {
    /// A probabilistic fact (a Boolean random variable) with these weights.
    Fact(Prob),
    /// Defined by the rules whose head it is. No rules means false.
    Derived,
}

/// A propositional probabilistic logic program.
#[derive(Clone, Debug)]
pub struct GroundProgram {
    /// Ground atoms; the index is the `AtomId`.
    pub atoms: Vec<Atom>,
    pub(crate) kinds: Vec<AtomKind>,
    pub rules: Vec<GroundRule>,
    /// Queries `(atom, id, positive)`. A query on an impossible atom is
    /// recorded as `None` with its display atom, so the caller can report
    /// probability zero (or one for a negative query).
    pub(crate) queries: Vec<(Atom, Option<AtomId>, bool)>,
    /// Evidence `(atom, value)`. An impossible atom observed true is kept as
    /// `None` and makes the evidence inconsistent.
    pub evidence: Vec<(Atom, Option<AtomId>, bool)>,
    /// How the program reads derived atoms through variables of their own
    /// ([`crate::indicators`]); empty for a program as grounded.
    pub encoding: Encoding,
    /// The factor of `P(e)` the program no longer counts, which `P(q, e)`
    /// lost too: the product of what each conditioning on the evidence
    /// took out (`condition`), a half for each fact `defined` eliminated,
    /// and a two for each separator copy (`fact_copies`). One where
    /// nothing was taken out.
    pub weight: Count,
    /// What the grounder's rewrites made of the program.
    pub stats: GroundStats,
    /// The options the program was ground under, which the layout
    /// rewrites finish it again with (`without_linear_recursion`,
    /// `guarded`) and the query split restricts it under (`restricted`).
    /// They are the program's rather than the solver's: a differential
    /// test grounds under options of its own and solves the program
    /// under the default ones (`solve_ground`).
    pub(super) options: GroundOptions,
    /// The structure the grounder computed for the program as it left it,
    /// or the first call of [`GroundProgram::structure`] after it.
    pub(super) structure: OnceLock<Arc<Structure>>,
}

/// What the grounder's rewrites made of a program, for `--stats`; the
/// rewrites that make atoms number them from their counts here.
#[derive(Clone, Debug, Default)]
pub struct GroundStats {
    /// The predicates, as `name/arity`, whose right-linear recursion was
    /// grounded from its bound end (`linear`).
    pub from_bound_end: Vec<String>,
    /// How many keys (level, rank) the grounder completed, where it
    /// grounded the recursive strata level by level, and how many atoms
    /// it proved constant as their keys completed (`engine`).
    pub levels: usize,
    pub constants: usize,
    /// How many views the grounder replaced by their bodies in their
    /// readers (`inline`).
    pub inlined: usize,
    /// How many of the atoms inlined were of a recursive predicate, a
    /// chain (`inline`).
    pub chained: usize,
    /// How many derived atoms the grounder merged, after the views, into an
    /// earlier atom with the same rules or proved equal in function
    /// (`aig::reduce`).
    pub merged: usize,
    /// How many derived atoms that merge took out of the program: those it
    /// merged and those its merges and constants left settled or unread.
    pub reduced: usize,
    /// What the search for gates equal in function did (`aig`).
    pub swept: super::SweepStats,
    /// How many tables the grounder wrote as noisy-ors (`noisy_or`).
    pub noisy_or: usize,
    /// How many facts the grounder made to read choices through, one per
    /// context (`contexts`).
    pub contexts: usize,
    /// How many atoms the grounder made conditioned on the fact literals
    /// that guard their reads (`guards`).
    pub guarded: usize,
    /// How many atoms the grounder made composing chains of steps
    /// (`steps`).
    pub stepped: usize,
    /// What the last conditioning on the evidence fixed and merged
    /// (`condition`).
    pub conditioned: Conditioned,
    /// Observations dropped, which no query shares a fact with and which
    /// a world was found to satisfy (`detach`). The program's `weight`
    /// leaves their probability out, so it is not the evidence's where
    /// any was dropped.
    pub detached: usize,
    /// The elimination of the facts of probability one half that the
    /// clauses define, where the program is a CNF over its facts
    /// (`defined`).
    pub defined: Option<Defined>,
    /// The vtree the guards' cubes give, where the grounder read facts as
    /// guards (`guards`), which `decision` lays out.
    pub decisions: Option<Arc<Decisions>>,
}

impl Default for GroundProgram {
    fn default() -> Self {
        GroundProgram {
            atoms: Vec::new(),
            kinds: Vec::new(),
            rules: Vec::new(),
            queries: Vec::new(),
            evidence: Vec::new(),
            encoding: Encoding::default(),
            weight: Count::ONE,
            stats: GroundStats::default(),
            options: GroundOptions::default(),
            structure: OnceLock::new(),
        }
    }
}

impl GroundProgram {
    pub fn num_atoms(&self) -> usize {
        self.atoms.len()
    }
    /// How many queries the program asks, a `query(\+a)` among them.
    pub fn num_queries(&self) -> usize {
        self.queries.len()
    }
    /// How many of the ground atoms are probabilistic facts.
    pub fn num_facts(&self) -> usize {
        self.kinds
            .iter()
            .filter(|k| matches!(k, AtomKind::Fact(_)))
            .count()
    }
    /// How many of the ground atoms are derived.
    pub fn num_derived(&self) -> usize {
        self.num_atoms() - self.num_facts()
    }
    pub fn is_fact(&self, a: AtomId) -> bool {
        matches!(self.kinds[a as usize], AtomKind::Fact(_))
    }
    /// The probabilistic facts with their weights.
    pub(crate) fn facts(&self) -> impl Iterator<Item = (AtomId, Prob)> + '_ {
        self.kinds.iter().enumerate().filter_map(|(i, k)| match k {
            AtomKind::Fact(p) => Some((i as AtomId, *p)),
            AtomKind::Derived => None,
        })
    }
    /// The ground rules and their body literals, counted together.
    pub(crate) fn size(&self) -> usize {
        self.rules.iter().map(|r| 1 + r.body.len()).sum()
    }
    /// The program's structure ([`Structure`]): the one it keeps
    /// where the program still fits it, else computed anew and kept where
    /// it keeps none, so that `auto` and the rewrites do not compute it
    /// again.
    pub(crate) fn structure(&self) -> Arc<Structure> {
        match self.structure.get() {
            Some(s) if s.fits(self) => s.clone(),
            _ => {
                let s = Arc::new(Structure::of(self));
                let _ = self.structure.set(Arc::clone(&s));
                s
            }
        }
    }
    /// The program without its rules and the structure they have, for a
    /// rewrite that writes every rule anew ([`crate::indicators::introduce`]).
    pub(crate) fn without_rules(&self) -> GroundProgram {
        let GroundProgram {
            atoms,
            kinds,
            rules: _,
            queries,
            evidence,
            encoding,
            weight,
            stats,
            options,
            structure: _,
        } = self;
        GroundProgram {
            atoms: atoms.clone(),
            kinds: kinds.clone(),
            rules: Vec::new(),
            queries: queries.clone(),
            evidence: evidence.clone(),
            encoding: encoding.clone(),
            weight: *weight,
            stats: stats.clone(),
            options: *options,
            structure: OnceLock::new(),
        }
    }
    /// The fact literal each atom stands for where it is a copy of a fact:
    /// an atom whose one rule reads one fact, `g :- f` or `g :- \+f`, as the
    /// conditioning on the evidence makes of facts it finds equal
    /// (`condition`). `None` for every other atom.
    pub(crate) fn copies_of_facts(&self) -> Vec<Option<GLit>> {
        let mut rules = vec![0u32; self.num_atoms()];
        let mut one_fact = vec![None; self.num_atoms()];
        for r in &self.rules {
            rules[r.head as usize] += 1;
            one_fact[r.head as usize] = match &r.body[..] {
                &[l] if self.is_fact(l.atom()) => Some(l),
                _ => None,
            };
        }
        rules
            .iter()
            .zip(one_fact)
            .map(|(&k, one)| one.filter(|_| k == 1))
            .collect()
    }
    /// Rules grouped by head: `by_head[a]` lists the indices into `rules`
    /// of `a`'s rules, in their order.
    pub(crate) fn rules_by_head(&self) -> RulesByHead {
        Lists::new(self.atoms.len(), || {
            self.rules
                .iter()
                .enumerate()
                .map(|(i, r)| (r.head as usize, i))
        })
    }

    /// The rules reading each atom, once per literal, in rule order: `f`
    /// of each rule's index and the literal ([`Lists`]).
    pub(crate) fn readers<T: Copy>(&self, f: impl Fn(u32, GLit) -> T) -> Lists<T> {
        let f = &f;
        Lists::new(self.atoms.len(), || {
            self.rules.iter().enumerate().flat_map(move |(i, r)| {
                r.body
                    .iter()
                    .map(move |&l| (l.atom() as usize, f(i as u32, l)))
            })
        })
    }

    /// The atoms the queries and the evidence name, which keep their
    /// identity: the printed program defines them.
    pub(crate) fn named_atoms(&self) -> impl Iterator<Item = AtomId> + '_ {
        self.queries
            .iter()
            .chain(&self.evidence)
            .filter_map(|&(_, id, _)| id)
    }
}

/// The rules of each atom as head, as indices into [`GroundProgram::rules`]
/// in their order ([`GroundProgram::rules_by_head`]).
pub(crate) type RulesByHead = Lists<usize>;

/// Lists of items for the keys `0..n`, held in one vector, each key's
/// items a range of it. A vector per key cost an allocation for every key
/// with items each time the lists were made: ace_blockmap_22_03's 586 k
/// in its nine groupings of the rules by head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Lists<T> {
    /// Where each key's items start in `items`, and where they end.
    start: Vec<usize>,
    items: Vec<T>,
}

impl<T: Copy> Lists<T> {
    /// The lists of the `n` keys of the `(key, item)` pairs that `pairs`
    /// yields, each key's items in the order they come. `pairs` is called
    /// twice: once to count each key's items and once to place them.
    pub(crate) fn new<I: Iterator<Item = (usize, T)>>(n: usize, pairs: impl Fn() -> I) -> Self {
        // Each key's count at the place after its own, summed into where
        // its items start, then each item put at its key's next place,
        // which leaves each key's start where the next key's items start,
        // shifted back by one place.
        let mut start = vec![0usize; n + 1];
        pairs().for_each(|(k, _)| start[k + 1] += 1);
        for k in 0..n {
            start[k + 1] += start[k];
        }
        let Some((_, first)) = pairs().next() else {
            return Lists {
                start,
                items: Vec::new(),
            };
        };
        let mut items = vec![first; start[n]];
        pairs().for_each(|(k, t)| {
            let at = &mut start[k];
            items[*at] = t;
            *at += 1;
        });
        start.copy_within(0..n, 1);
        start[0] = 0;
        Lists { start, items }
    }

    /// Each key's items, key by key.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &[T]> + '_ {
        self.start.windows(2).map(|w| &self.items[w[0]..w[1]])
    }
}

/// No keys.
impl<T> Default for Lists<T> {
    fn default() -> Self {
        Lists {
            start: vec![0],
            items: Vec::new(),
        }
    }
}

impl<T> std::ops::Index<usize> for Lists<T> {
    type Output = [T];

    fn index(&self, k: usize) -> &[T] {
        &self.items[self.start[k]..self.start[k + 1]]
    }
}

impl fmt::Display for GroundProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (a, p) in self.facts() {
            writeln!(f, "{}::{}.", p.p, self.atoms[a as usize])?;
        }
        // A rule of pairs is printed as the rules it stands for.
        for (head, body) in self
            .rules
            .iter()
            .flat_map(|r| r.bodies().map(|body| (r.head, body)))
        {
            write!(f, "{}", self.atoms[head as usize])?;
            if !body.is_empty() {
                f.write_str(" :- ")?;
                for (i, l) in body.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    match l {
                        GLit::Pos(a) => write!(f, "{}", self.atoms[*a as usize])?,
                        GLit::Neg(a) => write!(f, "\\+{}", self.atoms[*a as usize])?,
                    }
                }
            }
            writeln!(f, ".")?;
        }
        for (a, id, v) in &self.evidence {
            let a = id
                .map(|i| self.atoms[i as usize].clone())
                .unwrap_or_else(|| a.clone());
            writeln!(f, "evidence({a},{v}).")?;
        }
        for (a, id, positive) in &self.queries {
            let a = id
                .map(|i| self.atoms[i as usize].clone())
                .unwrap_or_else(|| a.clone());
            if *positive {
                writeln!(f, "query({a}).")?;
            } else {
                writeln!(f, "query(\\+{a}).")?;
            }
        }
        Ok(())
    }
}

/// A program of facts of probabilities `facts`, atoms 0 on, and after
/// them `derived` derived atoms with the rules `rules`, the atoms of
/// `evidence` observed: the unit tests' programs.
#[cfg(test)]
pub(super) fn test_program(
    facts: &[f64],
    derived: usize,
    rules: &[(AtomId, &[GLit])],
    evidence: &[(AtomId, bool)],
) -> GroundProgram {
    let kinds: Vec<AtomKind> = facts
        .iter()
        .map(|&p| AtomKind::Fact(Prob::new(p)))
        .chain(std::iter::repeat_n(AtomKind::Derived, derived))
        .collect();
    GroundProgram {
        atoms: vec![Atom::default(); kinds.len()],
        kinds,
        rules: rules
            .iter()
            .map(|&(h, body)| GroundRule::new(h, body.to_vec()))
            .collect(),
        evidence: evidence
            .iter()
            .map(|&(a, v)| (Atom::default(), Some(a), v))
            .collect(),
        ..GroundProgram::default()
    }
}
