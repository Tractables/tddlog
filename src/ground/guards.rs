//! Derived atoms conditioned on the fact literals that guard their reads.
//!
//! A rule `h :- f, x` reads `x` only where the fact `f` holds, so it may
//! read `x` conditioned on `f` in its place: `f ∧ x` and `f ∧ x|f` are one
//! function. A circuit that a bit-blasting front end builds (the circuits
//! Alea exports) reads its selector flips in nested if-then-elses whose
//! branches read the same flips again, through shared sub-circuits and
//! through derived tests such as "the index is not 0". Conditioned on the
//! selectors above them the branches no longer read those flips, and the
//! diagram of a branch need not keep their values alive.
//!
//! `x` conditioned on the cube `C` of fact literals is computed once per
//! `x` and `C` restricted to the range of fact numbers `x`'s support lies
//! in. For each rule of `x`, the cube grows by the fact literals the
//! rule's literals imply under it, to a fixpoint: a fact literal itself; a
//! derived literal's value under the cube where that is a fact literal;
//! else the literals its definition implies, which are summarized as the
//! definition is made: every literal of its one body, or, for a negative
//! literal, the negation of each of its one-literal bodies. A rule whose
//! cube clashes holds nowhere and goes. The rule's body is then the cube's
//! literals beyond `C` and its derived literals conditioned on the cube.
//! The body implies every literal it adds, so under `C` it holds where the
//! original holds.
//!
//! Each definition the pass makes is made once by its bodies. The one an
//! atom has under the empty cube keeps the atom's number, as does one that
//! reads as the atom's own rules, so queries and evidence keep theirs; the
//! others are atoms of their own (`$gd`), each read only where its cube
//! holds.
//!
//! The pass applies to an acyclic program in which a rule reads a fact
//! within the range of one of its derived literals, and leaves the program
//! as it was where it would make it more than [`GROWTH`] times larger, or
//! spend more than [`SPEND`] literal visits or copy more than [`COPIED`]
//! cube literals per literal of it.
//!
//! What the pass read as guards also gives a vtree ([`Decisions`]): a
//! fact sits at the set of literals that most of the cubes it decides
//! between hold, or that most of the cubes it is read under hold where it
//! decides nothing; the set of a fact that decides keeps no literal of a
//! fact that every cube holding it reads under the first, which decides
//! below it, unless every cube holding the first holds it too. A definition
//! made for a cube restricted to its range reads its facts under each of
//! its readers' cubes: its context is the literals that every cube a reader
//! reads it under holds, with the reader's context. The cube a fact decides
//! between others under is its reader's with that context; the cube it is
//! read under is its reader's with the literals of the contexts of all the
//! definitions that read it. Only the definitions that the queries and the
//! evidence reach count. A fact `g` at a node `S` decides between the nodes
//! `S ∪ {g}` and `S ∪ {¬g}`, which go below it, `(T1 (g T0))`. A vtree
//! node over `T1` alone or over `g` and `T0` has as many diagram nodes as
//! values its side leaves to the rest; `((T1 T0) g)` had their product
//! below `g`. The facts at `S` that decide nothing (the flips a comparator
//! reads beside a mux tree, say) go on a spine above those that decide:
//! beside them, each was a subtree of its own whose sibling held the
//! decided values. So do the facts at the nodes below `S` that no
//! decision reaches, where `S` is the deepest node the decisions reach on
//! the way to theirs. A spine holds its facts in the fan-in order of the
//! rules from the evidence and the queries, the shallowest roots first
//! ([`deps::fan_in`]): depth-first walks, each rule's atoms entered as its
//! body lists them, the first met read first. So the bits of the operands
//! a chain of rules combines (a comparator's, an adder's) come side by
//! side, where in program order each operand's bits came together and a
//! formula between the two held every value of one.

use std::rc::Rc;
use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::syntax::{Atom, Term};

use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, RulesByHead, deps, simplify};

/// The name of the atoms the pass makes.
const NAME: &str = "$gd";

/// The most literals the program may have after the pass, per literal
/// before.
const GROWTH: usize = 8;

/// The most literal visits the pass may spend, per literal of the program.
const SPEND: usize = 64;

/// The most cube literals the pass may copy and look up, per literal of
/// the program. A cube restricted to an atom's range of fact numbers holds
/// every fact of the cube numbered within it, read by the atom or not: on
/// a Roulette ladder of 3 000 rungs, whose grounding numbers one column's
/// facts before the other's, each rung's cubes held the other column's
/// facts above it, and the pass copied 6.7 G cube literals within its
/// visits, 2 200 a visit. The circuits copy at most 17 a visit, and at
/// most 732 per literal of the program in all.
const COPIED: usize = 1024;

/// The part of the budgets above with which [`applies`] runs the pass.
const TEST_BUDGET: usize = 8;

/// The part of the program's derived atoms that [`applies`] asks the pass
/// to make.
const TEST_MADE: usize = 4;

/// The most literals a definition's summary keeps of those it implies.
const SUMMARY: usize = 256;

/// No settled cube of a frame's own: a rule's reads under the frame's cube.
const NONE: u32 = u32::MAX;

/// A vtree read off the cubes the pass read facts under, over the facts
/// by name: the facts the passes after it rename or drop are left out of
/// it, and the facts it does not hold go after it when it is laid out
/// (`vtree::decision`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decisions {
    /// The nodes, each after its children; the last is the root.
    pub(crate) nodes: Vec<DecisionNode>,
    /// The nodes below the root that a decision enters, each after the
    /// contexts below it: `decision` names them (`vtree::decision`).
    pub(crate) contexts: Vec<Context>,
}

/// A node `S ∪ {l}` that the decision `l` enters from the node `S`: its
/// subtree, `S` and `l`'s fact by name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Context {
    /// The node of [`Decisions::nodes`] that holds its subtree.
    pub(crate) node: u32,
    /// The context it is entered from, `None` from the root.
    pub(crate) parent: Option<u32>,
    pub(crate) fact: Atom,
    pub(crate) positive: bool,
}

/// A node of [`Decisions`]: a fact or the join of two nodes before it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DecisionNode {
    Leaf(Atom),
    Join(u32, u32),
}

/// The value of an atom conditioned on a cube.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Val {
    True,
    False,
    Lit(GLit),
    /// An atom of the output: one the program had, or one the pass made.
    Atom(AtomId),
}

/// Adds `l` to `cube`, sorted by atom; false where the cube holds its
/// negation.
fn insert(cube: &mut Vec<GLit>, l: GLit) -> bool {
    match cube.binary_search_by_key(&l.atom(), |m| m.atom()) {
        Ok(i) => cube[i] == l,
        Err(i) => {
            cube.insert(i, l);
            true
        }
    }
}

/// `cube` without the literals that `other`, sorted by atom, does not hold.
fn meet(cube: &mut Vec<GLit>, other: &[GLit]) {
    cube.retain(|l| {
        other
            .binary_search_by_key(&l.atom(), |m| m.atom())
            .is_ok_and(|i| other[i] == *l)
    });
}

/// Cubes of fact literals, sorted by atom, each held once.
#[derive(Default)]
struct Cubes {
    ids: FxHashMap<Rc<[GLit]>, u32>,
    all: Vec<Rc<[GLit]>>,
}

impl Cubes {
    fn intern(&mut self, c: &[GLit]) -> u32 {
        if let Some(&i) = self.ids.get(c) {
            return i;
        }
        let rc: Rc<[GLit]> = Rc::from(c);
        let i = self.all.len() as u32;
        self.all.push(Rc::clone(&rc));
        self.ids.insert(rc, i);
        i
    }

    fn of(&self, i: u32) -> &[GLit] {
        &self.all[i as usize]
    }

    fn shared(&self, i: u32) -> Rc<[GLit]> {
        Rc::clone(&self.all[i as usize])
    }
}

/// A definition the pass made: an atom and its bodies, with the literals
/// it implies and those its negation implies, as cubes.
struct Def {
    atom: AtomId,
    bodies: std::ops::Range<usize>,
    pos: u32,
    neg: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    /// The rule's cube starts as the frame's.
    Start,
    /// Literal `j` of the rule is next to add what it implies; `n0` is the
    /// cube's size when the round over the rule's literals began.
    Imply { j: usize, n0: usize },
    /// The cube is settled: its new literals start the body.
    Guard,
    /// Literal `j` of the rule is next to read conditioned on the cube.
    Build { j: usize },
}

/// A derived literal's last read in the rule being built: its value and
/// the length of the part of the cube it was read under. The cube only
/// grows while the rule is built, so a part of the same length is the same
/// part, the value read under it stands, and what it implies is in the
/// cube.
#[derive(Clone, Copy)]
struct Read {
    val: Val,
    part: usize,
}

impl Read {
    /// No read yet.
    const NONE: Read = Read {
        val: Val::False,
        part: usize::MAX,
    };
}

/// `x` conditioned on cube `cx`, rule by rule.
///
/// A rule's cube is `cx`, held as `base`, with the literals the rule adds
/// to it, held apart as `extra`: the rule reads parts of the cube and adds
/// a few literals, and a copy of `cx` per rule, which every insertion then
/// shifted, cost as much as the rest of the pass.
struct Frame {
    x: AtomId,
    cx: u32,
    rule: usize,
    phase: Phase,
    base: Rc<[GLit]>,
    /// The literals the rule's cube holds beyond `cx`, sorted by atom.
    extra: Vec<GLit>,
    /// A part of the cube that holds literals of both, merged.
    part: Vec<GLit>,
    /// Per literal of the rule, its last read where it is derived.
    values: Vec<Read>,
    body: Vec<GLit>,
    bodies: Vec<Vec<GLit>>,
    /// The literals that the rules that add guards to `cx` add, back to
    /// back, each rule's ending at its entry of `ends`: with `cx`, their
    /// settled cubes. These are interned where the frame's value is a
    /// definition, whose guards and reads alone the decisions read
    /// ([`Pass::inherited`]).
    settled: Vec<GLit>,
    ends: Vec<usize>,
    /// The cube the rule being built reads under: an index of `ends`, or
    /// [`NONE`] for `cx`.
    under: u32,
    /// The facts read as guards and the atoms of the output the bodies
    /// read, each with the cube it was read under, as `under`.
    guards: Vec<(AtomId, u32)>,
    reads: Vec<(AtomId, u32)>,
}

impl Frame {
    fn new() -> Self {
        Frame {
            x: 0,
            cx: 0,
            rule: 0,
            phase: Phase::Start,
            base: Rc::from(&[][..]),
            extra: Vec::new(),
            part: Vec::new(),
            values: Vec::new(),
            body: Vec::new(),
            bodies: Vec::new(),
            settled: Vec::new(),
            ends: Vec::new(),
            under: NONE,
            guards: Vec::new(),
            reads: Vec::new(),
        }
    }

    /// The frame set to `x` under the cube `cx`, its buffers emptied.
    fn start(&mut self, x: AtomId, cx: u32, base: Rc<[GLit]>) {
        (self.x, self.cx, self.base) = (x, cx, base);
        self.rule = 0;
        self.phase = Phase::Start;
        self.extra.clear();
        self.values.clear();
        self.body.clear();
        self.bodies.clear();
        self.settled.clear();
        self.ends.clear();
        self.guards.clear();
        self.reads.clear();
    }

    fn next_rule(&mut self) {
        self.rule += 1;
        self.phase = Phase::Start;
    }

    /// Adds `l` to the rule's cube; false where the cube holds its
    /// negation.
    fn insert(&mut self, l: GLit) -> bool {
        match self.base.binary_search_by_key(&l.atom(), |m| m.atom()) {
            Ok(i) => self.base[i] == l,
            Err(_) => insert(&mut self.extra, l),
        }
    }
}

/// `a` and `b`, cubes sorted by atom without an atom in common, merged
/// into `into`.
fn merge(into: &mut Vec<GLit>, a: &[GLit], b: &[GLit]) {
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i].atom() < b[j].atom() {
            into.push(a[i]);
            i += 1;
        } else {
            into.push(b[j]);
            j += 1;
        }
    }
    into.extend_from_slice(&a[i..]);
    into.extend_from_slice(&b[j..]);
}

/// A value the frame on top needs that is not known yet: an atom under a
/// cube.
struct Need(AtomId, u32);

struct Pass<'a> {
    gp: &'a GroundProgram,
    by_head: RulesByHead,
    /// The range of fact numbers each atom's support lies in; `lo > hi`
    /// where it reads no fact.
    lo: Vec<AtomId>,
    hi: Vec<AtomId>,
    cubes: Cubes,
    memo: FxHashMap<(AtomId, u32), Val>,
    defs: Vec<Def>,
    /// The bodies of the definitions, each a range of `lits`.
    lits: Vec<GLit>,
    body_ends: Vec<usize>,
    def_of: FxHashMap<AtomId, u32>,
    by_hash: FxHashMap<u64, Vec<u32>>,
    claimed: Vec<bool>,
    made: usize,
    /// Whether a definition differs from the program's.
    changed: bool,
    /// Per fact a definition's frame read as a guard, the cube it was read
    /// under (holding it) and the definition.
    paths: Vec<(AtomId, u32, u32)>,
    /// Per atom of the output a definition's body reads, the definition
    /// and the cube it was read under.
    reads: Vec<(u32, AtomId, u32)>,
    /// The frames of [`Pass::eval`], kept with their buffers between its
    /// calls.
    frames: Vec<Frame>,
    spent: usize,
    budget: usize,
    copied: usize,
    copy_budget: usize,
    out_budget: usize,
    /// The atoms made at which the pass stops ([`applies`]).
    target: usize,
}

/// The rules of `gp` by head and the range of fact numbers each atom's
/// support lies in, where the conditioning applies to `gp`: an acyclic
/// program without pairs rules in which a rule reads a fact within the
/// range of one of its derived literals; `None` where it does not.
fn applicable(gp: &GroundProgram) -> Option<(RulesByHead, Vec<AtomId>, Vec<AtomId>)> {
    if gp.rules.iter().any(|r| r.pairs) {
        return None;
    }
    let by_head = gp.rules_by_head();
    let sccs = deps::derived_sccs(gp, &by_head);
    if sccs.iter().any(|c| deps::is_cyclic(gp, &by_head, c)) {
        return None;
    }
    let n = gp.num_atoms();
    let (mut lo, mut hi) = (vec![AtomId::MAX; n], vec![0; n]);
    for (a, _) in gp.facts() {
        (lo[a as usize], hi[a as usize]) = (a, a);
    }
    for c in &sccs {
        let a = c[0] as usize;
        for &ri in &by_head[a] {
            for l in &gp.rules[ri].body {
                let b = l.atom() as usize;
                lo[a] = lo[a].min(lo[b]);
                hi[a] = hi[a].max(hi[b]);
            }
        }
    }
    let guarded = |r: &GroundRule| {
        let derived = || r.body.iter().filter(|l| !gp.is_fact(l.atom()));
        r.body.iter().any(|f| {
            gp.is_fact(f.atom())
                && derived()
                    .any(|b| (lo[b.atom() as usize]..=hi[b.atom() as usize]).contains(&f.atom()))
        })
    };
    gp.rules.iter().any(guarded).then_some((by_head, lo, hi))
}

/// Whether the conditioning on guards applies to `gp` ([`applicable`])
/// and gets under way there: whether, with an eighth of its budgets
/// ([`TEST_BUDGET`]), the pass makes a quarter as many atoms as `gp` has
/// derived ones ([`TEST_MADE`]). `auto` asks it of a program as ground,
/// before it grounds the program again for `decision`
/// (`auto::may_decide`). Where `decision` is a trial (the circuits of the
/// bit-blasting suite, gen4, caesar), the pass makes that many within 3.4
/// visits and 29 cube literals per literal of the program, and spends up
/// to 57 visits in all. Where it is not, the pass ends with fewer (the
/// arithmetic circuits, pcfg), runs out of cube literals (a Roulette
/// ladder) or of visits (Dice's pmc1) first, or gets under way and gives
/// up later (pmc2, gcd_small).
pub(crate) fn applies(gp: &GroundProgram) -> bool {
    let Some(mut pass) = Pass::new(gp, TEST_BUDGET) else {
        return false;
    };
    pass.target = gp.num_derived() / TEST_MADE;
    let empty = pass.cubes.intern(&[]);
    for r in roots(gp) {
        if pass.eval(r, empty).is_none() {
            break;
        }
    }
    pass.made >= pass.target
}

/// The derived atoms the queries and the evidence name, which the pass
/// conditions on the empty cube.
fn roots(gp: &GroundProgram) -> Vec<AtomId> {
    gp.named_atoms().filter(|&a| !gp.is_fact(a)).collect()
}

/// Conditions the derived atoms of `gp` on the fact literals that guard
/// their reads, as described above, and records the decisions in
/// `gp.stats.decisions`, counting the atoms it makes in
/// `gp.stats.guarded`; `None`
/// where the pass does not apply, changes nothing or gives up, which
/// leaves `gp` as it was.
pub(super) fn condition(gp: &mut GroundProgram) -> Option<()> {
    let mut pass = Pass::new(gp, 1)?;
    let empty = pass.cubes.intern(&[]);
    let roots = roots(gp);
    let mut values = Vec::with_capacity(roots.len());
    for &r in &roots {
        values.push(pass.eval(r, empty)?);
    }
    let roots_kept = roots.iter().zip(&values).all(|(&r, &v)| v == Val::Atom(r));
    if !pass.changed && roots_kept {
        return None;
    }
    let decisions = pass.decisions(&values);
    let Pass {
        defs,
        lits,
        body_ends,
        claimed,
        made,
        ..
    } = pass;
    let mut rules = Vec::with_capacity(body_ends.len() + roots.len());
    for d in &defs {
        for k in d.bodies.clone() {
            let start = if k == 0 { 0 } else { body_ends[k - 1] };
            rules.push(GroundRule::new(d.atom, lits[start..body_ends[k]].to_vec()));
        }
    }
    for (&r, &v) in roots.iter().zip(&values) {
        if v == Val::Atom(r) || claimed[r as usize] {
            continue;
        }
        match v {
            Val::True => rules.push(GroundRule::new(r, Vec::new())),
            Val::False => {}
            Val::Lit(l) => rules.push(GroundRule::new(r, vec![l])),
            Val::Atom(a) => rules.push(GroundRule::new(r, vec![GLit::Pos(a)])),
        }
    }
    let gp = &mut *gp;
    let first = gp.stats.guarded;
    for k in 0..made {
        gp.atoms
            .push(Atom::new(NAME, vec![Term::Int((first + k) as i64)]));
        gp.kinds.push(AtomKind::Derived);
    }
    gp.stats.guarded += made;
    gp.rules = rules;
    gp.structure = std::sync::OnceLock::new();
    gp.stats.decisions = decisions.map(Arc::new);
    simplify::restrict_to_relevant(gp);
    Some(())
}

impl<'a> Pass<'a> {
    /// The pass on `gp` where it applies ([`applicable`]), its budgets on
    /// visits and cube literals divided by `part`.
    fn new(gp: &'a GroundProgram, part: usize) -> Option<Self> {
        let (by_head, lo, hi) = applicable(gp)?;
        let size = gp.size();
        Some(Pass {
            gp,
            by_head,
            lo,
            hi,
            cubes: Cubes::default(),
            memo: FxHashMap::default(),
            defs: Vec::new(),
            lits: Vec::new(),
            body_ends: Vec::new(),
            def_of: FxHashMap::default(),
            by_hash: FxHashMap::default(),
            claimed: vec![false; gp.num_atoms()],
            made: 0,
            changed: false,
            paths: Vec::new(),
            reads: Vec::new(),
            frames: Vec::new(),
            spent: 0,
            budget: SPEND * size / part,
            copied: 0,
            copy_budget: COPIED * size / part,
            out_budget: GROWTH * size,
            target: usize::MAX,
        })
    }

    /// Whether `a`, an atom of the program or one the pass made, is a fact.
    fn is_fact(&self, a: AtomId) -> bool {
        self.gp
            .kinds
            .get(a as usize)
            .is_some_and(|k| matches!(k, AtomKind::Fact(_)))
    }

    /// Starts frame `k` of `frames` for `x` under the cube `c`, in the
    /// buffers of a finished one where there is one.
    fn start(&self, frames: &mut Vec<Frame>, k: usize, x: AtomId, c: u32) {
        if k == frames.len() {
            frames.push(Frame::new());
        }
        frames[k].start(x, c, self.cubes.shared(c));
    }

    /// `x` conditioned on the cube `c`; `None` where the pass gives up or
    /// has made its target.
    fn eval(&mut self, x: AtomId, c: u32) -> Option<Val> {
        if let Some(&v) = self.memo.get(&(x, c)) {
            return Some(v);
        }
        // The frames in use are the first `top`, each waiting for the
        // value of the one after it.
        let mut frames = std::mem::take(&mut self.frames);
        self.start(&mut frames, 0, x, c);
        let mut top = 1;
        // The value of the frame last finished, which the frame under it
        // waits for.
        let mut given = None;
        let done = loop {
            let f = &mut frames[top - 1];
            match self.advance(f, given.take()) {
                Ok(v) => {
                    self.memo.insert((f.x, f.cx), v);
                    given = Some(v);
                    top -= 1;
                }
                Err(Need(b, cb)) => {
                    self.start(&mut frames, top, b, cb);
                    top += 1;
                }
            }
            if self.spent > self.budget
                || self.copied > self.copy_budget
                || self.lits.len() > self.out_budget
                || self.made >= self.target
            {
                break None;
            }
            if top == 0 {
                break given;
            }
        };
        self.frames = frames;
        done
    }

    /// The part of `cube` within `b`'s range of fact numbers.
    fn part(&self, b: AtomId, cube: &[GLit]) -> std::ops::Range<usize> {
        let (lo, hi) = (self.lo[b as usize], self.hi[b as usize]);
        if lo > hi {
            0..0
        } else {
            let start = cube.partition_point(|l| l.atom() < lo);
            let end = cube.partition_point(|l| l.atom() <= hi);
            start..end
        }
    }

    /// `b` conditioned on `part`, a cube restricted to `b`'s range
    /// ([`Pass::part`]) and interned as `known` where that is given, or
    /// what is needed to know it.
    fn value(&mut self, b: AtomId, part: &[GLit], known: Option<u32>) -> Result<Val, Need> {
        self.copied += part.len();
        let c = known.unwrap_or_else(|| self.cubes.intern(part));
        self.memo.get(&(b, c)).copied().ok_or(Need(b, c))
    }

    /// Adds to the cube of `f`'s rule what a literal of value `v`,
    /// positive or not, implies; false where that clashes with the cube.
    fn imply(&self, f: &mut Frame, v: Val, positive: bool) -> bool {
        match v {
            Val::True => positive,
            Val::False => !positive,
            Val::Lit(l) => f.insert(if positive { l } else { l.negated() }),
            Val::Atom(a) => {
                let Some(&d) = self.def_of.get(&a) else {
                    return true;
                };
                let d = &self.defs[d as usize];
                let implied = self.cubes.of(if positive { d.pos } else { d.neg });
                implied.iter().all(|&l| f.insert(l))
            }
        }
    }

    /// Runs frame `f` on until it has its value, or needs one it lacks;
    /// `given` is the value it last needed, where it has just been found.
    fn advance(&mut self, f: &mut Frame, mut given: Option<Val>) -> Result<Val, Need> {
        let gp = self.gp;
        loop {
            let Some(&ri) = self.by_head[f.x as usize].get(f.rule) else {
                return Ok(self.finish(f));
            };
            let body = &gp.rules[ri].body;
            match f.phase {
                Phase::Start => {
                    // The budget counts `cx` as copied, as it was before
                    // the rule's cube held it apart.
                    f.extra.clear();
                    self.copied += f.base.len();
                    f.values.clear();
                    f.values.resize(body.len(), Read::NONE);
                    f.phase = Phase::Imply { j: 0, n0: 0 };
                }
                Phase::Imply { j, n0 } => {
                    if j == body.len() {
                        f.phase = if f.extra.len() == n0 {
                            Phase::Guard
                        } else {
                            Phase::Imply {
                                j: 0,
                                n0: f.extra.len(),
                            }
                        };
                        continue;
                    }
                    let l = body[j];
                    let ok = if gp.is_fact(l.atom()) {
                        f.insert(l)
                    } else {
                        let (inner, outer) =
                            (self.part(l.atom(), &f.base), self.part(l.atom(), &f.extra));
                        let n = inner.len() + outer.len();
                        if f.values[j].part == n {
                            // Read under this part before ([`Read`]).
                            self.copied += n;
                            true
                        } else {
                            // A part as long as the cube, while the cube is
                            // `cx`, is `cx`.
                            let known = (f.extra.is_empty() && n == f.base.len()).then_some(f.cx);
                            let v = match given.take() {
                                Some(v) => {
                                    self.copied += n;
                                    v
                                }
                                None => {
                                    let part = if outer.is_empty() {
                                        &f.base[inner]
                                    } else {
                                        f.part.clear();
                                        merge(&mut f.part, &f.base[inner], &f.extra[outer]);
                                        &f.part[..]
                                    };
                                    self.value(l.atom(), part, known)?
                                }
                            };
                            f.values[j] = Read { val: v, part: n };
                            self.imply(f, v, l.is_positive())
                        }
                    };
                    self.spent += 1;
                    if ok {
                        f.phase = Phase::Imply { j: j + 1, n0 };
                    } else {
                        f.next_rule();
                    }
                }
                Phase::Guard => {
                    // The guards are the literals the rule added to `cx`.
                    f.body.clear();
                    f.body.extend_from_slice(&f.extra);
                    self.copied += f.base.len() + f.extra.len();
                    // A rule that adds no guard reads under `cx`: its cube
                    // grew by none of its atoms and clashed with none.
                    f.under = NONE;
                    if !f.body.is_empty() {
                        f.under = f.ends.len() as u32;
                        f.settled.extend_from_slice(&f.extra);
                        f.ends.push(f.settled.len());
                        f.guards.extend(f.body.iter().map(|l| (l.atom(), f.under)));
                    }
                    f.phase = Phase::Build { j: 0 };
                }
                Phase::Build { j } => {
                    if j == body.len() {
                        let mut b = std::mem::take(&mut f.body);
                        b.sort_unstable();
                        b.dedup();
                        if b.is_empty() {
                            return Ok(Val::True);
                        }
                        f.bodies.push(b);
                        f.next_rule();
                        continue;
                    }
                    let l = body[j];
                    if !gp.is_fact(l.atom()) {
                        // A fact literal is in the cube, and a derived one
                        // whose value is a fact literal put it there. The
                        // round that settled the cube read every derived
                        // one under it.
                        let read = f.values[j];
                        self.copied += read.part;
                        match (read.val, l.is_positive()) {
                            (Val::True, false) | (Val::False, true) => {
                                f.next_rule();
                                continue;
                            }
                            (Val::Atom(a), positive) => {
                                f.reads.push((a, f.under));
                                f.body
                                    .push(if positive { GLit::Pos(a) } else { GLit::Neg(a) })
                            }
                            _ => {}
                        }
                    }
                    self.spent += 1;
                    f.phase = Phase::Build { j: j + 1 };
                }
            }
        }
    }

    /// The value of a frame whose rules are all read: false without
    /// bodies, the literal or atom of one body of one, else a definition.
    fn finish(&mut self, f: &mut Frame) -> Val {
        let bodies = &mut f.bodies;
        match &bodies[..] {
            [] => return Val::False,
            [b] if b.len() == 1 => {
                let l = b[0];
                if self.is_fact(l.atom()) {
                    return Val::Lit(l);
                }
                if l.is_positive() {
                    return Val::Atom(l.atom());
                }
            }
            _ => {}
        }
        bodies.sort_unstable();
        bodies.dedup();
        let a = self.define(f.x, f.cx, bodies);
        self.keep_reads(f, self.def_of[&a]);
        Val::Atom(a)
    }

    /// The guards and reads of frame `f`, whose value is definition `d`,
    /// kept with their cubes interned.
    fn keep_reads(&mut self, f: &mut Frame, d: u32) {
        let mut start = 0;
        let mut ids = Vec::with_capacity(f.ends.len());
        for &end in &f.ends {
            f.part.clear();
            merge(&mut f.part, &f.base, &f.settled[start..end]);
            ids.push(self.cubes.intern(&f.part));
            start = end;
        }
        let cube = |k: u32| if k == NONE { f.cx } else { ids[k as usize] };
        self.paths
            .extend(f.guards.iter().map(|&(g, k)| (g, cube(k), d)));
        self.reads
            .extend(f.reads.iter().map(|&(a, k)| (d, a, cube(k))));
    }

    /// The atom defined by `bodies`, each sorted, sorted: one made before
    /// with these bodies, else `x` where `x` is free and these bodies are
    /// `x`'s under the empty cube or its own rules, else a new atom.
    fn define(&mut self, x: AtomId, cx: u32, bodies: &[Vec<GLit>]) -> AtomId {
        use std::hash::BuildHasher;
        let h = rustc_hash::FxBuildHasher.hash_one(bodies);
        if let Some(ds) = self.by_hash.get(&h) {
            for &d in ds {
                if self.bodies_of(d).eq(bodies.iter().map(Vec::as_slice)) {
                    return self.defs[d as usize].atom;
                }
            }
        }
        let own = !self.claimed[x as usize] && self.own_bodies(x) == bodies;
        let atom = if !self.claimed[x as usize] && (own || self.cubes.of(cx).is_empty()) {
            self.claimed[x as usize] = true;
            if !own {
                self.changed = true;
            }
            x
        } else {
            self.changed = true;
            self.made += 1;
            (self.gp.num_atoms() + self.made - 1) as AtomId
        };
        // What the atom implies: its one body's literals; what its negation
        // implies: the negations of its one-literal bodies.
        let mut pos = Vec::new();
        let mut neg = Vec::new();
        if let [b] = bodies {
            for &l in b {
                self.summarize(&mut pos, l);
            }
        }
        for b in bodies {
            if let &[l] = &b[..] {
                self.summarize(&mut neg, l.negated());
            }
        }
        let pos = self.summary(pos);
        let neg = self.summary(neg);
        let start = self.body_ends.len();
        for b in bodies {
            self.lits.extend_from_slice(b);
            self.body_ends.push(self.lits.len());
        }
        let d = self.defs.len() as u32;
        self.defs.push(Def {
            atom,
            bodies: start..self.body_ends.len(),
            pos,
            neg,
        });
        self.def_of.insert(atom, d);
        self.by_hash.entry(h).or_default().push(d);
        atom
    }

    /// Adds to `into` the fact literals that literal `l`, holding, implies:
    /// itself for a fact, its definition's summary for a derived atom.
    fn summarize(&self, into: &mut Vec<GLit>, l: GLit) {
        if into.len() >= SUMMARY {
            return;
        }
        if self.is_fact(l.atom()) {
            into.push(l);
        } else if let Some(&d) = self.def_of.get(&l.atom()) {
            let d = &self.defs[d as usize];
            into.extend_from_slice(self.cubes.of(if l.is_positive() { d.pos } else { d.neg }));
        }
    }

    fn summary(&mut self, mut lits: Vec<GLit>) -> u32 {
        lits.sort_unstable_by_key(|l| (l.atom(), l.is_positive()));
        lits.dedup();
        lits.truncate(SUMMARY);
        self.cubes.intern(&lits)
    }

    fn bodies_of(&self, d: u32) -> impl Iterator<Item = &[GLit]> + '_ {
        self.defs[d as usize].bodies.clone().map(|k| {
            let start = if k == 0 { 0 } else { self.body_ends[k - 1] };
            &self.lits[start..self.body_ends[k]]
        })
    }

    /// `x`'s rules' bodies as [`Pass::define`] takes them.
    fn own_bodies(&self, x: AtomId) -> Vec<Vec<GLit>> {
        let mut bodies: Vec<Vec<GLit>> = self.by_head[x as usize]
            .iter()
            .map(|&ri| {
                let mut b = self.gp.rules[ri].body.clone();
                b.sort_unstable();
                b.dedup();
                b
            })
            .collect();
        bodies.sort_unstable();
        bodies.dedup();
        bodies
    }

    /// The paths of the definitions that the roots' values reach, each
    /// fact with two cubes. A definition is made once for a cube restricted
    /// to its range, and the facts it reads are read in each context its
    /// readers read it in: its context, the literals that every cube a
    /// reachable reader reads it under holds, with that reader's context (a
    /// piece's slope flip, which an if-then-else of the shared low bits
    /// reads, is read under the piece's index, whose flips lie outside the
    /// range). The first cube, which places the decisions it holds above
    /// the fact, is the fact's own with its definition's context. The
    /// second, which places the fact, has only the literals of the contexts
    /// of every definition that reads the fact: a comparator's flip, read
    /// by its bit's equality only under the stage below and by its
    /// less-than everywhere, stays where its reads leave it. A frame read
    /// where a rule's later literal clashes made a definition that nothing
    /// reaches.
    fn inherited(&mut self, roots: &[Val]) -> Vec<(AtomId, u32, u32)> {
        let nd = self.defs.len();
        let mut context: Vec<Option<Vec<GLit>>> = vec![None; nd];
        for v in roots {
            if let Val::Atom(a) = v
                && let Some(&d) = self.def_of.get(a)
            {
                context[d as usize] = Some(Vec::new());
            }
        }
        let mut reads_of: Vec<Vec<(AtomId, u32)>> = vec![Vec::new(); nd];
        for &(d, a, c) in &self.reads {
            reads_of[d as usize].push((a, c));
        }
        // A definition is made after those it reads: its readers come
        // before it from the last.
        for d in (0..nd).rev() {
            let Some(here) = context[d].take() else {
                continue;
            };
            for &(a, c) in &reads_of[d] {
                let Some(&k) = self.def_of.get(&a) else {
                    continue;
                };
                let mut ctx = here.clone();
                for &l in self.cubes.of(c) {
                    insert(&mut ctx, l);
                }
                let k = k as usize;
                context[k] = Some(match context[k].take() {
                    None => ctx,
                    Some(mut was) => {
                        meet(&mut was, &ctx);
                        was
                    }
                });
            }
            context[d] = Some(here);
        }
        let paths: Vec<(AtomId, u32, &[GLit])> = std::mem::take(&mut self.paths)
            .into_iter()
            .filter_map(|(g, c, d)| Some((g, c, context[d as usize].as_deref()?)))
            .collect();
        let mut shared: FxHashMap<AtomId, Vec<GLit>> = FxHashMap::default();
        for &(g, _, here) in &paths {
            match shared.get_mut(&g) {
                None => {
                    shared.insert(g, here.to_vec());
                }
                Some(was) => meet(was, here),
            }
        }
        let cubes = &mut self.cubes;
        let mut with = |c: u32, ctx: &[GLit]| {
            if ctx.is_empty() {
                return c;
            }
            let mut cube = cubes.of(c).to_vec();
            for &l in ctx {
                insert(&mut cube, l);
            }
            cubes.intern(&cube)
        };
        paths
            .iter()
            .map(|&(g, c, here)| (g, with(c, here), with(c, &shared[&g])))
            .collect()
    }

    /// The vtree the cubes the facts were read under give, as described
    /// above; `None` where no fact was read as a guard.
    fn decisions(&mut self, roots: &[Val]) -> Option<Decisions> {
        let mut paths = self.inherited(roots);
        paths.sort_unstable();
        paths.dedup();
        if paths.is_empty() {
            return None;
        }
        let at = self.nodes(&paths);
        let (order, seen) = tree_order(&at);
        let mut hosted = hosts(&at, &seen);
        let met = deps::fan_in(self.gp, false);
        let mut tree = Tree {
            gp: self.gp,
            nodes: Vec::new(),
        };
        let mut done: FxHashMap<Vec<GLit>, Option<usize>> = FxHashMap::default();
        // Each context's subtree, its decision and the node it is entered
        // from.
        let mut entered: Vec<(usize, GLit, Vec<GLit>)> = Vec::new();
        for (s, kids) in order {
            let mut alone = hosted.remove(&s).unwrap_or_default();
            let mut parts = Vec::new();
            for (g, t, e) in kids {
                let mut enter = |c: Option<Vec<GLit>>, l: GLit| {
                    let node = done.remove(&c?).flatten()?;
                    entered.push((node, l, s.clone()));
                    Some(node)
                };
                let t = enter(t, GLit::Pos(g));
                let e = enter(e, GLit::Neg(g));
                match (t, e) {
                    (Some(t), Some(e)) => {
                        let leaf = tree.leaf(g);
                        let ge = tree.join(leaf, e);
                        parts.push(tree.join(t, ge));
                    }
                    (Some(b), None) | (None, Some(b)) => {
                        let leaf = tree.leaf(g);
                        parts.push(tree.join(b, leaf));
                    }
                    (None, None) => alone.push(g),
                }
            }
            // The facts that decide nothing here on a spine above those
            // that decide, `(a1 (a2 (... (d1 (d2 ...)))))`, the first that
            // the walk meets beside the decisions.
            alone.sort_unstable_by_key(|&g| std::cmp::Reverse(met[g as usize]));
            let spine: Vec<Option<usize>> = alone
                .into_iter()
                .map(|g| Some(tree.leaf(g)))
                .chain(parts.into_iter().map(Some))
                .collect();
            done.insert(s, tree.join_all(&spine));
        }
        done.remove(&Vec::new()).flatten()?;
        // A context's parent is the one whose cube it was entered from.
        let mut index: FxHashMap<&[GLit], u32> = FxHashMap::default();
        let cubes: Vec<Vec<GLit>> = entered
            .iter()
            .map(|(_, l, from)| {
                let mut c = from.clone();
                insert(&mut c, *l);
                c
            })
            .collect();
        for (k, c) in cubes.iter().enumerate() {
            index.insert(c, k as u32);
        }
        let contexts = entered
            .iter()
            .map(|(node, l, from)| Context {
                node: *node as u32,
                parent: index.get(from.as_slice()).copied(),
                fact: self.gp.atoms[l.atom() as usize].clone(),
                positive: l.is_positive(),
            })
            .collect();
        Some(Decisions {
            nodes: tree.nodes,
            contexts,
        })
    }

    /// Each node of the tree, a cube, with the facts at it, from the
    /// sorted and distinct `paths` the facts were read in
    /// ([`Self::inherited`]).
    fn nodes(&self, paths: &[(AtomId, u32, u32)]) -> Nodes {
        // Per path, a cube it was read under without its own fact; per
        // fact, the paths that hold it, and its own.
        let without = |f: AtomId, c: u32| {
            self.cubes
                .of(c)
                .iter()
                .copied()
                .filter(move |l| l.atom() != f)
        };
        let n = self.gp.num_atoms();
        let mut holding: Vec<Vec<u32>> = vec![Vec::new(); n];
        let mut own: Vec<Vec<u32>> = vec![Vec::new(); n];
        for (k, &(f, c, _)) in paths.iter().enumerate() {
            own[f as usize].push(k as u32);
            for l in without(f, c) {
                holding[l.atom() as usize].push(k as u32);
            }
        }
        // Each fact's node: the literals in more than half of the paths
        // that hold it, by their first cubes, or of its own where none
        // does, by their second. Per fact that the paths hold, the facts
        // that every cube holding it holds.
        let mut nodes: Vec<(AtomId, Vec<GLit>, bool)> = Vec::new();
        let mut within: FxHashMap<AtomId, Vec<AtomId>> = FxHashMap::default();
        // The paths of `of` that hold each literal, and each atom either
        // way, of those `touched` lists.
        let index = |l: GLit| 2 * l.atom() as usize + l.is_positive() as usize;
        let (mut counts, mut holds) = (vec![0u32; 2 * n], vec![0u32; n]);
        let mut touched: Vec<GLit> = Vec::new();
        let mut facts: Vec<AtomId> = paths.iter().map(|p| p.0).collect();
        facts.dedup();
        for &f in &facts {
            let held = !holding[f as usize].is_empty();
            let of = if held {
                &holding[f as usize]
            } else {
                &own[f as usize]
            };
            for &k in of {
                let (g, first, second) = paths[k as usize];
                let c = if held { first } else { second };
                for l in without(g, c).filter(|l| l.atom() != f) {
                    if counts[index(l)] == 0 {
                        touched.push(l);
                    }
                    counts[index(l)] += 1;
                }
            }
            let mut node: Vec<GLit> = touched
                .iter()
                .copied()
                .filter(|&l| 2 * counts[index(l)] as usize > of.len())
                .collect();
            node.sort_unstable_by_key(|l| l.atom());
            for l in touched.drain(..) {
                counts[index(l)] = 0;
            }
            if held {
                for &k in of {
                    for &l in self.cubes.of(paths[k as usize].1) {
                        if holds[l.atom() as usize] == 0 {
                            touched.push(l);
                        }
                        holds[l.atom() as usize] += 1;
                    }
                }
                let mut always: Vec<AtomId> = touched
                    .iter()
                    .map(|l| l.atom())
                    .filter(|&a| a != f && holds[a as usize] as usize == of.len())
                    .collect();
                always.sort_unstable();
                for l in touched.drain(..) {
                    holds[l.atom() as usize] = 0;
                }
                within.insert(f, always);
            }
            nodes.push((f, node, held));
        }
        // A fact `h` that every cube holding it reads under the decision
        // `g` (each holds `g` or `¬g`) decides below `g`, unless every
        // cube holding `g` holds `h` too: `g`'s node keeps no literal of
        // `h`. Where one branch of `g` holds most of the cubes that hold
        // `g`, most of them also hold the next decision on that branch,
        // and `g`'s node took its literal: no decision reached that node,
        // and on genes_t10 the selector trees of the four genes fell onto
        // spines (636 facts each two decisions deep, where their trees
        // are ten).
        let within_of =
            |h: AtomId, g: AtomId| within.get(&h).is_some_and(|a| a.binary_search(&g).is_ok());
        let mut at = Nodes::default();
        for (f, mut node, held) in nodes {
            if held {
                node.retain(|l| !within_of(l.atom(), f) || within_of(f, l.atom()));
            }
            at.entry(node).or_default().push(f);
        }
        at
    }
}

/// The facts at each node of the decisions' tree, a cube.
type Nodes = FxHashMap<Vec<GLit>, Vec<AtomId>>;

/// A fact at a node and the nodes it decides between, where they are
/// entered from it.
type Kid = (AtomId, Option<Vec<GLit>>, Option<Vec<GLit>>);

/// A node with its facts' kids.
type Node = (Vec<GLit>, Vec<Kid>);

/// The nodes of `at` from the empty one down, each fact `g` at a node
/// deciding between the node with `g` and the node with `¬g`: each node
/// with its facts' kids, a node laid out under the first that reaches it
/// and after the nodes below it; and the nodes reached.
fn tree_order(at: &Nodes) -> (Vec<Node>, FxHashSet<Vec<GLit>>) {
    let mut order: Vec<Node> = Vec::new();
    let mut stack: Vec<(Vec<GLit>, Option<Vec<Kid>>)> = vec![(Vec::new(), None)];
    let mut seen: FxHashSet<Vec<GLit>> = FxHashSet::default();
    seen.insert(Vec::new());
    while let Some((s, kids)) = stack.pop() {
        if let Some(kids) = kids {
            order.push((s, kids));
            continue;
        }
        let mut kids = Vec::new();
        let mut enter = Vec::new();
        for &g in at.get(&s).map(Vec::as_slice).unwrap_or(&[]) {
            let mut child = |l: GLit| {
                let mut t = s.clone();
                insert(&mut t, l).then_some(())?;
                (at.contains_key(&t) && seen.insert(t.clone())).then(|| {
                    enter.push(t.clone());
                    t
                })
            };
            let t = child(GLit::Pos(g));
            let e = child(GLit::Neg(g));
            kids.push((g, t, e));
        }
        stack.push((s, Some(kids)));
        stack.extend(enter.into_iter().map(|t| (t, None)));
    }
    (order, seen)
}

/// The facts at the nodes of `at` that no decision reaches, by the node
/// they go to: the deepest node the decisions reach (`seen`, [`tree_order`])
/// on the way to theirs, where they join the facts that decide nothing
/// there. On a spine under the root, outside the subtree of the cube they
/// were read under, each was apart from the decisions that select it, and
/// a formula there held every combination of their values (GPA's pieces).
fn hosts(at: &Nodes, seen: &FxHashSet<Vec<GLit>>) -> Nodes {
    let mut hosted = Nodes::default();
    for (node, fs) in at.iter().filter(|(s, _)| !seen.contains(*s)) {
        let mut host: Vec<GLit> = Vec::new();
        'down: loop {
            for &g in at.get(&host).map(Vec::as_slice).unwrap_or(&[]) {
                let Ok(i) = node.binary_search_by_key(&g, |l| l.atom()) else {
                    continue;
                };
                let mut t = host.clone();
                insert(&mut t, node[i]);
                if seen.contains(&t) {
                    host = t;
                    continue 'down;
                }
            }
            break;
        }
        hosted.entry(host).or_default().extend_from_slice(fs);
    }
    hosted
}

/// The nodes of a tree over the facts of `gp` by name, each after its
/// children.
struct Tree<'g> {
    gp: &'g GroundProgram,
    nodes: Vec<DecisionNode>,
}

impl Tree<'_> {
    fn leaf(&mut self, a: AtomId) -> usize {
        self.nodes
            .push(DecisionNode::Leaf(self.gp.atoms[a as usize].clone()));
        self.nodes.len() - 1
    }

    fn join(&mut self, l: usize, r: usize) -> usize {
        self.nodes.push(DecisionNode::Join(l as u32, r as u32));
        self.nodes.len() - 1
    }

    /// `(p1 (p2 (... pn)))` over the parts there are.
    fn join_all(&mut self, parts: &[Option<usize>]) -> Option<usize> {
        let mut out = None;
        for &p in parts.iter().rev().flatten() {
            out = Some(match out {
                Some(o) => self.join(p, o),
                None => p,
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::GroundOptions;

    fn ground(source: &str, guards: bool) -> crate::GroundProgram {
        let program = crate::syntax::parse_program(source).expect("parses");
        let gp = crate::ground::ground(
            &program,
            &GroundOptions {
                views: false,
                ..GroundOptions::default()
            },
        )
        .expect("grounds");
        if guards {
            super::super::guarded(gp)
        } else {
            gp
        }
    }

    /// An if-then-else whose branches read the selector again: under the
    /// selector each branch reads one fact, and the selector decides
    /// between them.
    #[test]
    fn branches_lose_the_selector_they_read() {
        let source = "0.3::s. 0.6::a. 0.2::b.
            m :- s, a. m :- \\+s, b.
            t :- m, s. t :- \\+m, \\+s.
            query(t).";
        let gp = ground(source, true);
        assert!(gp.stats.decisions.is_some(), "{gp}");
        assert!(!gp.to_string().contains('m'), "{gp}");
        let plain = ground(source, false);
        assert_eq!(plain.stats.guarded, 0);
        assert!(plain.stats.decisions.is_none());
    }

    /// An else-chain: `v` is `x` where `a`, else `y` where `b`, else `z`
    /// where `c`, else a sub-circuit of seven flips, so that most of the
    /// cubes that hold `a` hold `¬b` and `¬c` too. Every cube that holds
    /// `b` or `c` holds `¬a`, and they decide below it: the tree enters
    /// `¬a` from the root, `¬b` from `¬a` and `¬c` from `¬b`. By the
    /// majority alone `a` sat at `{¬b, ¬c}` and `b` at `{¬a, ¬c}`, which
    /// no decision reached, and every fact was on the root's spine.
    #[test]
    fn a_decision_read_only_under_another_decides_below_it() {
        let gp = ground(
            "0.3::a. 0.6::x. 0.4::b. 0.7::y. 0.5::c. 0.2::z.
            0.5::w1. 0.4::w2. 0.3::w3. 0.6::w4. 0.7::w5. 0.2::w6. 0.8::w7.
            v :- a, x. v :- \\+a, u.
            u :- b, y. u :- \\+b, t.
            t :- c, z. t :- \\+c, s.
            b1 :- w1, w2. b1 :- \\+w1, w3.
            b2 :- w1, w2, w4. b2 :- w1, \\+w2, w5.
            b2 :- \\+w1, w3, w6. b2 :- \\+w1, \\+w3, w7.
            s :- b1, b2. s :- \\+b1, \\+b2, w1.
            query(v).",
            true,
        );
        let decisions = gp.stats.decisions.as_ref().expect("decisions");
        let contexts = &decisions.contexts;
        let name = |k: Option<u32>| {
            k.map(|k| {
                let c = &contexts[k as usize];
                (c.fact.to_string(), c.positive)
            })
        };
        let entered: Vec<_> = contexts
            .iter()
            .map(|c| ((c.fact.to_string(), c.positive), name(c.parent)))
            .collect();
        let not = |f: &str| (f.to_string(), false);
        for (context, parent) in [
            (not("a"), None),
            (not("b"), Some(not("a"))),
            (not("c"), Some(not("b"))),
        ] {
            assert!(
                entered.contains(&(context.clone(), parent.clone())),
                "{context:?} from {parent:?} in {entered:?}"
            );
        }
    }

    /// A program the pass does not apply to: no rule reads a fact that a
    /// derived literal of its body reads too.
    #[test]
    fn a_program_without_guarded_reads_is_left_as_it_is() {
        let source = "0.3::a. 0.6::b. 0.2::c.
            x :- a, b. y :- x, c. y :- \\+x, \\+c.
            query(y).";
        let before = ground(source, false).to_string();
        let gp = ground(source, true);
        assert_eq!(gp.to_string(), before);
        assert!(gp.stats.decisions.is_none());
    }

    /// The test `auto` asks before it grounds a program again for
    /// `decision`: where a rule reads a sub-circuit under the selector its
    /// branches read, the pass makes conditioned atoms, at least a quarter
    /// as many as the program has derived ones; where the only guarded
    /// read conditions to fact literals, it makes none, and a chain above
    /// it adds derived atoms that it does not condition.
    #[test]
    fn the_test_asks_for_a_quarter_of_the_derived_atoms() {
        let shared = ground(
            "0.5::s. 0.5::a. 0.5::b. 0.5::c.
            u :- s, a. u :- \\+s, b.
            v :- u, c. v :- \\+u, \\+c.
            t :- s, v. t :- \\+s, \\+v.
            w :- t, c.
            query(w).",
            false,
        );
        assert!(shared.num_derived() >= 4, "{shared}");
        assert!(super::applies(&shared), "{shared}");
        let chain = ground(
            "0.3::s. 0.6::a. 0.2::b. 0.5::c.
            m :- s, a. m :- \\+s, b.
            t :- m, s. t :- \\+m, \\+s.
            x1 :- t, c. x2 :- x1, c. x3 :- x2, c. x4 :- x3, c. x5 :- x4, c. x6 :- x5, c.
            query(x6).",
            false,
        );
        assert!(super::applicable(&chain).is_some(), "{chain}");
        assert!(!super::applies(&chain), "{chain}");
    }

    /// The walk from the query meets a comparator's operand bits pair by
    /// pair, high bits first as the rules read them, where program order
    /// has one operand's bits, then the other's. (The grounder reads the
    /// lowest bits' facts as a choice of its own, which the test leaves
    /// aside.)
    #[test]
    fn the_walk_meets_a_comparators_operands_bit_by_bit() {
        let gp = ground(
            "0.5::a0. 0.5::a1. 0.5::a2. 0.5::a3. 0.5::b0. 0.5::b1. 0.5::b2. 0.5::b3.
            lt0 :- \\+a0, b0.
            lt1 :- \\+a1, b1. lt1 :- a1, b1, lt0. lt1 :- \\+a1, \\+b1, lt0.
            lt2 :- \\+a2, b2. lt2 :- a2, b2, lt1. lt2 :- \\+a2, \\+b2, lt1.
            lt3 :- \\+a3, b3. lt3 :- a3, b3, lt2. lt3 :- \\+a3, \\+b3, lt2.
            query(lt3).",
            false,
        );
        let met = crate::ground::deps::fan_in(&gp, false);
        let mut facts: Vec<_> = gp.facts().map(|(a, _)| a).collect();
        facts.sort_by_key(|&a| met[a as usize]);
        let names: Vec<String> = facts
            .iter()
            .map(|&a| gp.atoms[a as usize].to_string())
            .collect();
        let at = |name: &str| {
            names
                .iter()
                .position(|n| n == name)
                .unwrap_or_else(|| panic!("{name} in {names:?}"))
        };
        for i in 1..=3 {
            assert_eq!(
                at(&format!("a{i}")).abs_diff(at(&format!("b{i}"))),
                1,
                "{names:?}"
            );
        }
        assert!(at("a3") < at("a2") && at("a2") < at("a1"), "{names:?}");
    }
}
