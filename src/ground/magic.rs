//! Magic-set rewrite: demand-driven grounding.
//!
//! Bottom-up grounding derives every possible atom of every predicate. For
//! programs that only make sense with bindings flowing from the query (a time
//! index counting down from the query's argument, a probabilistic fact
//! `0.5::b(X).` over an unbounded domain) that is wrong or does not terminate.
//!
//! The rewrite adds, for every predicate `p` and binding pattern `α` reached
//! from the queries, a *magic* predicate `m_p^α` over the bound arguments,
//! seeded by the query and evidence atoms. Every rule for `p` gets a copy
//! guarded by `m_p^α(bound head args)`, and every body atom `q` in that copy
//! yields a magic rule `m_q^β(bound args of q) :- m_p^α(...), <preceding
//! literals>`, following the left-to-right scheduling of
//! [`super::schedule`], of those literals the ones that bear on the bound
//! arguments of `q` and the tests of what they bind. The atoms themselves are not renamed, so the copies for
//! different patterns derive into the same predicate; every atom derived is in
//! the original model, and every atom the queries depend on is derived. A
//! given predicate, one of ground facts ([`super::normalize::Facts`]) or
//! one of rules that read only given predicates, is not demanded: its facts
//! and rules are read as they are, and a negation of it filters the demand
//! as a positive literal does. A
//! rule that cannot use some of a pattern's bindings, a head variable its
//! body reads only as the value of an `is`, is copied under the pattern
//! without them instead, whose magic predicate the first one's feeds. A
//! predicate that the rewrite finds demanded both with every argument
//! free and with some bound is demanded free only, in a second rewrite.
//!
//! Magic predicates only control grounding: the grounder drops them from the
//! ground rules it emits (see `NormProgram::control`). Their truth in the
//! possible-world union means "demanded in some world", which is exactly the
//! relevance condition.
//!
//! The rewrite can make a stratified program non-stratified (demand for a
//! lower-stratum atom may pass through a recursive predicate of a higher
//! stratum), and a demand rule does not read a negation of a derived
//! predicate. Where either loses a negation that bounds a demand in a
//! program that builds values, [`Rewritten::top_down`] says so and the
//! caller resolves the program top-down.
//!
//! Which route a program takes is decided before any of it is ground, by
//! whether its rules are safe ([`super::engine::safe`]: every variable bound
//! by a positive body atom, in some order of the body). A program the
//! rewrite leaves with an unsafe rule is one whose demand a binding pattern
//! cannot carry: a query of a term with variables in it (`seen([_,_,_])`,
//! `length(L, 3)`), whose pattern binds no argument, or a fact or rule that
//! answers with a variable (`reverse([], L, L)`, `0.5::bit(T, 0)` with `T`
//! bound only by the caller). Where the plain program is unsafe too, no
//! demand makes it safe, and the program is resolved top-down from its
//! queries instead ([`super::engine::topdown`]), as ProbLog grounds every
//! program.

use std::sync::Arc;

use super::normalize::{Driver, Facts, NormProgram, Sig};
use super::schedule::{Order, literal_vars, schedule_body};
use crate::syntax::{Atom, Clause, Literal, Name, Term};
use rustc_hash::{FxHashMap, FxHashSet};

pub(crate) const MAGIC_PREFIX: &str = "$m";

/// Binding pattern: one flag per argument, `true` = bound.
type Adornment = Vec<bool>;

/// The predicates, by name and arity, and the patterns they are demanded in.
type Demands = FxHashSet<(Name, usize, Adornment)>;

/// `$m_<name>_<pattern>`, written into one string of its size rather
/// than formatted.
fn magic_name(name: &str, adornment: &Adornment) -> Name {
    let mut text = String::with_capacity(MAGIC_PREFIX.len() + name.len() + adornment.len() + 2);
    text.push_str(MAGIC_PREFIX);
    text.push('_');
    text.push_str(name);
    text.push('_');
    text.extend(adornment.iter().map(|b| if *b { 'b' } else { 'f' }));
    text.into()
}

fn magic_atom(atom: &Atom, adornment: &Adornment) -> Atom {
    Atom::new(
        magic_name(&atom.name, adornment),
        bound_args(atom, adornment),
    )
}

/// The arguments of `atom` that `adornment` binds.
fn bound_args(atom: &Atom, adornment: &Adornment) -> Vec<Term> {
    atom.args
        .iter()
        .zip(adornment)
        .filter(|(_, b)| **b)
        .map(|(t, _)| t.clone())
        .collect()
}

fn adornment_of(atom: &Atom, bound: &FxHashSet<String>) -> Adornment {
    atom.args
        .iter()
        .map(|t| match t {
            Term::Var(v) => bound.contains(v),
            // A query's pattern is bound where it is ground.
            t => t.is_ground(),
        })
        .collect()
}

/// The part of `adornment` that `rule` can use: a bound head variable that
/// the body reads only as the value an `is` computes binds nothing the body
/// reads before that test, and a demand for each of its values grounds the
/// body again for each: mnist_sum96's queries `sum(S)` demand
/// `partial(96, S)` for each sum, and its rule
/// `partial(K, S) :- next(J, K), partial(J, S0), digit(K, D), S is S0 + D`
/// would scan every sum of the level before and the ten digits for each,
/// per sum demanded. `None` where that is all of `adornment`.
fn usable(rule: &Clause, adornment: &Adornment) -> Option<Adornment> {
    let head = &rule.heads[0].atom;
    if rule.heads.len() > 1 {
        return None;
    }
    let read = |v: &String| {
        rule.body.iter().any(|l| match l {
            Literal::Is(Term::Var(w), e) if w == v => {
                let mut vs = Vec::new();
                e.vars_into(&mut vs);
                vs.contains(v)
            }
            other => literal_vars(other).contains(v),
        })
    };
    // A variable the body does not mention at all, as the `_` of
    // `walk([], _, _)`, only the demand binds.
    let mentioned = |v: &String| rule.body.iter().any(|l| literal_vars(l).contains(v));
    let keeps = |i: usize, t: &Term| match t {
        Term::Var(v) => {
            read(v) || !mentioned(v) || head.args.iter().enumerate().any(|(j, u)| j != i && u == t)
        }
        _ => true,
    };
    // A head of constants, as a tuple's, keeps all: asked first, without
    // the copy of the adornment per rule.
    if head
        .args
        .iter()
        .zip(adornment)
        .enumerate()
        .all(|(i, (t, &b))| !b || keeps(i, t))
    {
        return None;
    }
    Some(
        head.args
            .iter()
            .zip(adornment)
            .enumerate()
            .map(|(i, (t, &b))| b && keeps(i, t))
            .collect(),
    )
}

/// Whether no rule of `norm` has a variable. Queries and evidence may still
/// be patterns; the grounder resolves those against the possible atoms.
pub(crate) fn is_ground(norm: &NormProgram) -> bool {
    norm.drivers.is_empty() && norm.rules.iter().all(ground_clause)
}

/// Whether neither the heads nor the body of `c` has a variable.
pub(super) fn ground_clause(c: &Clause) -> bool {
    c.heads.iter().all(|h| h.atom.is_ground()) && c.body.iter().all(|l| literal_vars(l).is_empty())
}

/// The predicates the rewrite reads as they are, without a demand: those
/// given by ground facts, as [`Facts`] or as rules, and those whose rules
/// read only given predicates and bind their heads without a demand. Their
/// atoms are all known before any demand is, and a demand would only copy
/// them under guards for the grounder to check against every new demand: on
/// PSouffle's side-channel analyses, 83 571 facts against the demands of
/// each of 1 350 semi-naive rounds. A negation of a given predicate joins
/// the demand's prefix: cplint's left-corner grammar tries a goal's third
/// clause only where `\+ lc(G, B, rule(G, [B|_]))` holds, which is nowhere.
fn given<'a>(
    norm: &'a NormProgram,
    rules_of: &FxHashMap<Sig<'a>, Vec<usize>>,
) -> FxHashSet<Sig<'a>> {
    let reads_given = |rule: &Clause, given: &FxHashSet<Sig>| {
        let [head] = &rule.heads[..] else {
            return false;
        };
        let literals = rule.body.iter().all(|l| match l {
            Literal::Pos(a) | Literal::Neg(a) => given.contains(&a.signature()),
            Literal::Cmp(..) | Literal::Is(..) | Literal::Builtin(..) => true,
            Literal::Findall(_) | Literal::Collect(_) => false,
        });
        let mut bound = FxHashSet::default();
        literals
            && !head.negated
            && schedule_body(&rule.body, &mut bound, |_| false, Order::Source).is_ok()
            && literal_vars(&Literal::Pos(head.atom.clone()))
                .iter()
                .all(|v| bound.contains(v))
    };
    let switches: FxHashSet<Sig> = norm.switches.keys().map(|(n, k)| (&**n, *k)).collect();
    let mut given: FxHashSet<Sig> = norm
        .facts
        .iter()
        .map(|(_, f)| (&*f.name, f.arity))
        .collect();
    loop {
        let before = given.len();
        for (&sig, rules) in rules_of {
            if !given.contains(&sig)
                && !switches.contains(&sig)
                && rules.iter().all(|&ri| reads_given(&norm.rules[ri], &given))
            {
                given.insert(sig);
            }
        }
        if given.len() == before {
            return given;
        }
    }
}

/// The literals of `prefix`, positions in `body`, that bear on the values
/// of `args`: those that share a variable with them, or with a literal
/// that does, in their order, and the tests of what those literals and
/// the guard bind (`guard`, the head's variables the demand binds). Another
/// literal only asks that some values of its own satisfy it, which a demand
/// need not ask: on PSouffle's side-channel analyses the demand of
/// `equal_assign` with no argument bound was derived again from each of the
/// 437 million pairs of an `xor_assign_right` and a `share` before it, 20 s
/// for each of eight rules. A test (a comparison, an `is`, a built-in, a
/// negation of a given predicate) joins nothing and only rejects demands,
/// as ProbLog's resolution stops at it before the call: the tutorial's
/// rejection loop `loop1(G, N, R, T) :- G >= N, loop2(0, 1, NG, NN, T, TT),
/// loop1(NG, NN, R, TT)` demanded `loop2` at every later time `T` without
/// its test `G >= N` on the guard's variables.
fn bearing(
    body: &[Literal],
    prefix: &[usize],
    args: &[Term],
    guard: &FxHashSet<String>,
) -> Vec<usize> {
    let mut reached: FxHashSet<String> = FxHashSet::default();
    for t in args {
        let mut vs = Vec::new();
        t.vars_into(&mut vs);
        reached.extend(vs);
    }
    let vars: Vec<Vec<String>> = prefix.iter().map(|&j| literal_vars(&body[j])).collect();
    let mut kept = vec![false; prefix.len()];
    let mut grew = true;
    while grew {
        grew = false;
        for (k, vs) in vars.iter().enumerate() {
            if !kept[k] && vs.iter().any(|v| reached.contains(v)) {
                kept[k] = true;
                grew = true;
                reached.extend(vs.iter().cloned());
            }
        }
    }
    for (k, vs) in vars.iter().enumerate() {
        let test = matches!(
            body[prefix[k]],
            Literal::Cmp(..) | Literal::Is(..) | Literal::Builtin(..) | Literal::Neg(_)
        );
        if test && vs.iter().all(|v| reached.contains(v) || guard.contains(v)) {
            kept[k] = true;
        }
    }
    prefix
        .iter()
        .zip(kept)
        .filter(|(_, k)| *k)
        .map(|(&j, _)| j)
        .collect()
}

/// Rewrite `norm`. Returns the rewritten program, or `None` if some rule
/// cannot be scheduled even with the head bindings (then the plain program
/// will report the same problem with a better message).
///
/// A predicate demanded with every argument free derives every atom it
/// can, and a demand of it with some bound grounds its rules again for
/// each call, with a tuple of its magic predicate per call. Where the rules
/// of such a demand read the predicate's own atoms before the bound head
/// argument (`rand(Y) :- rand(X), assign(Y, X)` asked for `rand(c)`), a
/// call of `assign` follows for every pair: on PSouffle's side-channel
/// analyses, 17 million tuples of the magic predicates of `assign` and the
/// relations it reads, where the program has 50 atoms that matter. A
/// predicate demanded both free and with some argument bound is therefore
/// rewritten again with every demand of it read as its free demand.
pub(crate) fn rewrite(norm: &NormProgram) -> Option<Rewritten> {
    let (out, done) = demand(norm, &FxHashSet::default())?;
    // Whether each predicate is demanded free, and with some argument bound.
    let mut patterns: FxHashMap<(&Name, usize), (bool, bool)> = FxHashMap::default();
    for (name, arity, adornment) in &done {
        let (free, bound) = patterns.entry((name, *arity)).or_default();
        if adornment.iter().any(|b| *b) {
            *bound = true;
        } else {
            *free = true;
        }
    }
    let free: FxHashSet<(Name, usize)> = patterns
        .into_iter()
        .filter(|(_, (free, bound))| *free && *bound)
        .map(|((name, arity), _)| (name.clone(), arity))
        .collect();
    if free.is_empty() {
        return Some(out);
    }
    demand(norm, &free).map(|(out, _)| out)
}

/// The rewritten program, and whether the program is resolved top-down
/// instead: where it builds values ([`builds_values`]) and a negation of a predicate the program stratifies below
/// its reader filters a demand of the reader's own recursion, or the
/// rewrite puts the negated predicate's demand inside that recursion. ProbLog's
/// resolution completes the negated call and drops the call after it
/// where the negation fails in every world; the rewrite cannot read the
/// negation there, so it filters no demand, and a walk that grows a list
/// under `\+ member(X, Seen)` (the tutorial's dice and taxonomy) is
/// demanded without end.
pub(crate) struct Rewritten {
    pub(crate) program: NormProgram,
    pub(crate) top_down: bool,
}

/// Whether a rule of `rules` makes a value the program does not write: a
/// term by unification, a number by `is`, a list by a built-in or a
/// findall. Without one, the values are the program's constants and its
/// rewrite is a finite program, ground bottom-up whatever negation its
/// demand rules leave out: PSouffle's taint analyses, whose rewrite puts a
/// negated predicate's demand inside the points-to recursion, ground so in
/// about a second and took up to 47 s top-down.
fn builds_values(rules: &[Clause]) -> bool {
    use crate::syntax::CmpOp;
    let made = |t: &Term| matches!(t, Term::Compound(_)) && !t.is_ground();
    rules.iter().any(|r| {
        r.body.iter().any(|l| match l {
            Literal::Is(..) | Literal::Builtin(..) | Literal::Collect(_) | Literal::Findall(_) => {
                true
            }
            Literal::Cmp(CmpOp::Eq, a, b) => made(a) || made(b),
            _ => false,
        }) || r.heads.iter().any(|h| h.atom.args.iter().any(made))
    })
}

/// A negation a demand rule does not read: the demand's predicate and the
/// negated one.
type Unread = ((Name, usize), (Name, usize));

/// Whether no rule of `rules`, and no negative dependency of `unread` (a
/// demand's predicate on a predicate it does not read negated) whose demand
/// derives itself, negates a predicate that depends on the rule's head. A
/// demand that no recursion of its own derives is as bounded without the
/// negation as with it: on the UW-CSE programs `taught_by` demands
/// `position(B, faculty)` for each professor `B` under `\+
/// aux1_taught_by(A, B)`, which reads `position` again, and the rewrite
/// grounds them as it did.
fn stratified(rules: &[Clause], unread: &[Unread]) -> bool {
    let mut ids: FxHashMap<Sig, usize> = FxHashMap::default();
    fn id<'n>(ids: &mut FxHashMap<Sig<'n>, usize>, sig: Sig<'n>) -> usize {
        let n = ids.len();
        *ids.entry(sig).or_insert(n)
    }
    let mut edges = Vec::new();
    let mut negated = Vec::new();
    for c in rules {
        let h = id(&mut ids, c.heads[0].atom.signature());
        for l in &c.body {
            match l {
                Literal::Pos(a) => edges.push((h, id(&mut ids, a.signature()))),
                Literal::Neg(a) => negated.push((h, id(&mut ids, a.signature()))),
                Literal::Collect(c) => edges.push((h, id(&mut ids, (&*c.aux, c.arity)))),
                _ => {}
            }
        }
    }
    edges.extend_from_slice(&negated);
    let crossing = |edges: &[(usize, usize)], negated: &[(usize, usize)]| {
        let component = super::deps::strongly_connected(ids.len(), edges);
        let all = negated.iter().all(|&(h, q)| component[h] != component[q]);
        (all, component)
    };
    let (all, component) = crossing(&edges, &negated);
    if !all {
        return false;
    }
    let mut size: FxHashMap<usize, usize> = FxHashMap::default();
    for &c in &component {
        *size.entry(c).or_default() += 1;
    }
    let recursive = |d: usize| {
        component[d] != usize::MAX && (size[&component[d]] > 1 || edges.contains(&(d, d)))
    };
    let more: Vec<(usize, usize)> = unread
        .iter()
        .filter_map(|((d, k), (n, j))| Some((*ids.get(&(d, *k))?, *ids.get(&(n, *j))?)))
        .filter(|&(d, _)| recursive(d))
        .collect();
    if more.is_empty() {
        return true;
    }
    edges.extend_from_slice(&more);
    negated.extend_from_slice(&more);
    crossing(&edges, &negated).0
}

/// The rewrite of `norm` ([`rewrite`]), in which a demand of a predicate of
/// `free` with some argument bound is its demand with none, and the
/// predicates and patterns it demands.
fn demand(norm: &NormProgram, free: &FxHashSet<(Name, usize)>) -> Option<(Rewritten, Demands)> {
    let rules_of = norm.rules_by_sig();
    let given = given(norm, &rules_of);
    let facts = norm
        .facts
        .iter()
        .map(|(_, f)| ((&*f.name, f.arity), f))
        .collect();
    let d = Demand {
        norm,
        free,
        rules_of,
        given,
        facts,
    };
    let mut w = Written {
        out: NormProgram {
            switches: norm.switches.clone(),
            queries: norm.queries.clone(),
            evidence: norm.evidence.clone(),
            drivers: norm.drivers.clone(),
            counted: norm.counted.clone(),
            ..Default::default()
        },
        read: FxHashSet::default(),
        control: FxHashSet::default(),
        done: Demands::default(),
        work: Vec::new(),
        unread: Vec::new(),
    };
    d.seeds(&mut w);
    while let Some((atom, adornment)) = w.work.pop() {
        let sig = atom.signature();
        if d.given.contains(&sig) {
            d.given_reached(&atom, &mut w);
        } else if d.derived(&atom) && w.done.insert((atom.name.clone(), sig.1, adornment.clone())) {
            d.demanded(&atom, &adornment, &mut w)?;
        }
    }
    w.out.control = w.control;
    let top_down = builds_values(&norm.rules)
        && stratified(&norm.rules, &[])
        && !stratified(&w.out.rules, &w.unread);
    Some((
        Rewritten {
            program: w.out,
            top_down,
        },
        w.done,
    ))
}

/// What [`demand`] reads of the program: its rules by predicate, the
/// predicates given ([`given`]) and the facts of each, and the predicates
/// whose demands are free.
struct Demand<'a> {
    norm: &'a NormProgram,
    free: &'a FxHashSet<(Name, usize)>,
    rules_of: FxHashMap<Sig<'a>, Vec<usize>>,
    given: FxHashSet<Sig<'a>>,
    facts: FxHashMap<Sig<'a>, &'a Arc<Facts>>,
}

/// What [`demand`] has written: the rewritten program, the given
/// predicates reached, whose rules or facts are in it, the magic
/// predicates, the demands taken and those still to take, and the
/// negations the demands do not read ([`Rewritten::top_down`]).
struct Written {
    out: NormProgram,
    read: FxHashSet<(Name, usize)>,
    control: FxHashSet<(Name, usize)>,
    done: Demands,
    work: Vec<(Atom, Adornment)>,
    unread: Vec<Unread>,
}

impl Demand<'_> {
    fn is_switch(&self, a: &Atom) -> bool {
        self.norm
            .switches
            .contains_key(&(a.name.clone(), a.args.len()))
    }

    /// Whether `a` is of a predicate with rules, not given, which a demand
    /// reaches.
    fn derived(&self, a: &Atom) -> bool {
        let sig = a.signature();
        self.rules_of.contains_key(&sig) && !self.given.contains(&sig) && !self.is_switch(a)
    }

    /// The pattern of `atom` with the variables `bound` bound, none where
    /// its predicate's demands are free.
    fn adorn(&self, atom: &Atom, bound: &FxHashSet<String>) -> Adornment {
        if self.free.contains(&(atom.name.clone(), atom.args.len())) {
            vec![false; atom.args.len()]
        } else {
            adornment_of(atom, bound)
        }
    }

    /// A demand of `atom` in the pattern `adornment` wherever `body` holds:
    /// its magic rule where `atom` is derived, and the demand to take.
    fn seed(&self, atom: &Atom, adornment: Adornment, body: Vec<Literal>, w: &mut Written) {
        if self.derived(atom) {
            w.out
                .rules
                .push(Clause::rule(magic_atom(atom, &adornment), body, u32::MAX));
        }
        w.work.push((atom.clone(), adornment));
    }

    /// The demands the rewrite starts from: the query and evidence atoms,
    /// with constants bound; the query rules' drivers with nothing bound.
    fn seeds(&self, w: &mut Written) {
        let none = FxHashSet::default();
        for (q, _) in &self.norm.queries {
            self.seed(q, self.adorn(q, &none), Vec::new(), w);
        }
        for (e, _) in &self.norm.evidence {
            self.seed(e, self.adorn(e, &none), Vec::new(), w);
        }
        for Driver {
            atom: driver,
            pattern,
            ..
        } in &self.norm.drivers
        {
            self.seed(driver, self.adorn(driver, &none), Vec::new(), w);
            // Every true driver instance demands the pattern instance it
            // names: m_pattern(bound args) :- driver(vars). The pattern's
            // variables are the driver's arguments, so all of them are
            // bound.
            let bound: FxHashSet<String> = literal_vars(&Literal::Pos(driver.clone()))
                .into_iter()
                .collect();
            let body = vec![Literal::Pos(driver.clone())];
            self.seed(pattern, self.adorn(pattern, &bound), body, w);
        }
    }

    /// The given predicate of `atom` reached: its facts, or its rules and
    /// the atoms they read, written the first time.
    fn given_reached(&self, atom: &Atom, w: &mut Written) {
        let sig = atom.signature();
        if !w.read.insert((atom.name.clone(), sig.1)) {
            return;
        }
        if let Some(&f) = self.facts.get(&sig) {
            w.out.facts.push((w.out.rules.len(), f.clone()));
            return;
        }
        for &ri in &self.rules_of[&sig] {
            let rule = &self.norm.rules[ri];
            for l in &rule.body {
                if let Literal::Pos(a) | Literal::Neg(a) = l {
                    w.work.push((a.clone(), Vec::new()));
                }
            }
            w.out.rules.push(rule.clone());
        }
    }

    /// The derived `atom` demanded in the pattern `adornment`, for the
    /// first time: its rules under the demand. `None` where a rule cannot
    /// be scheduled.
    fn demanded(&self, atom: &Atom, adornment: &Adornment, w: &mut Written) -> Option<()> {
        let sig = atom.signature();
        // The demand's magic predicate, named once for all the rules it
        // guards, rather than the name made, the adornment copied and the
        // literals of the prefix copied per rule: TPC-H's tuples are each a
        // rule of one predicate.
        let magic = magic_name(sig.0, adornment);
        w.control
            .insert((magic.clone(), adornment.iter().filter(|b| **b).count()));
        for &ri in &self.rules_of[&sig] {
            self.rule(&self.norm.rules[ri], adornment, &magic, w)?;
        }
        Some(())
    }

    /// `rule` under the demand of its head in the pattern `adornment`,
    /// whose magic predicate is `magic`: guarded by it, with a magic rule
    /// for each derived atom it reads, demanded with what the literals
    /// before it bind. `None` where it cannot be scheduled.
    fn rule(
        &self,
        rule: &Clause,
        adornment: &Adornment,
        magic: &Name,
        w: &mut Written,
    ) -> Option<()> {
        let head = &rule.heads[0].atom;
        // A rule that cannot use some of the demand's bindings is grounded
        // under the demand without them, which this demand feeds.
        if let Some(kept) = usable(rule, adornment) {
            w.out.rules.push(Clause::rule(
                magic_atom(head, &kept),
                vec![Literal::Pos(magic_atom(head, adornment))],
                u32::MAX,
            ));
            w.work.push((head.clone(), kept));
            return Some(());
        }
        // Head variables bound by the demand.
        let mut bound: FxHashSet<String> = FxHashSet::default();
        for (t, b) in head.args.iter().zip(adornment) {
            if let Term::Var(v) = t
                && *b
            {
                bound.insert(v.clone());
            }
        }
        let guard = Literal::Pos(Atom::new(magic.clone(), bound_args(head, adornment)));
        let ordered = schedule_body(
            &rule.body,
            &mut bound.clone(),
            |i| matches!(&rule.body[i], Literal::Pos(a) | Literal::Neg(a) if self.is_switch(a)),
            Order::Source,
        )
        .ok()?;
        // Walk the schedule, emitting magic rules for body atoms with rules.
        let mut bound_so_far = bound.clone();
        // After the guard, the body literals that bind: the positions in
        // the body, the literals copied only into a magic rule; and the
        // negations of derived predicates, which no demand reads.
        let mut prefix: Vec<usize> = Vec::new();
        let mut unread: Vec<usize> = Vec::new();
        for (&i, lit) in ordered.iter().map(|i| (i, &rule.body[*i])) {
            // A findall demands its goal's solutions for the values of the
            // variables it shares.
            let read;
            let body_atom = match lit {
                Literal::Pos(a) | Literal::Neg(a) => Some(a),
                Literal::Collect(c) => {
                    read = c.pattern();
                    Some(&read)
                }
                _ => None,
            };
            if let Some(a) = body_atom {
                if self.derived(a) {
                    let sub = self.adorn(a, &bound_so_far);
                    let args = bound_args(a, &sub);
                    let kept = bearing(&rule.body, &prefix, &args, &bound);
                    let mut body = Vec::with_capacity(1 + kept.len());
                    body.push(guard.clone());
                    body.extend(kept.iter().map(|&j| rule.body[j].clone()));
                    let demand = magic_atom(a, &sub);
                    if !unread.is_empty() {
                        let all: Vec<usize> = prefix.iter().chain(&unread).copied().collect();
                        for j in bearing(&rule.body, &all, &args, &bound) {
                            if let Literal::Neg(n) = &rule.body[j]
                                && unread.contains(&j)
                            {
                                w.unread.push((
                                    (demand.name.clone(), demand.args.len()),
                                    (n.name.clone(), n.args.len()),
                                ));
                            }
                        }
                    }
                    w.out.rules.push(Clause::rule(demand, body, u32::MAX));
                    w.work.push((a.clone(), sub));
                } else if self.given.contains(&a.signature()) {
                    w.work.push((a.clone(), Vec::new()));
                }
            }
            // Positive literals and deterministic filters join the prefix,
            // and so do negations of given predicates, whose atoms are all
            // known before any demand is; other negative literals do not
            // (they would add negative dependencies to the magic
            // predicates) and switches bind nothing.
            match lit {
                Literal::Neg(a) if self.derived(a) => unread.push(i),
                Literal::Neg(a) if !self.given.contains(&a.signature()) => {}
                Literal::Pos(a) if self.is_switch(a) => {}
                _ => prefix.push(i),
            }
            for v in literal_vars(lit) {
                bound_so_far.insert(v);
            }
        }
        let mut body = Vec::with_capacity(1 + ordered.len());
        body.push(guard);
        // A counted rule keeps its literals in the order of the source, by
        // which its derivations are ordered.
        if self
            .norm
            .counted
            .contains(&(head.name.clone(), head.args.len()))
        {
            body.extend(rule.body.iter().cloned());
        } else {
            body.extend(ordered.iter().map(|&i| rule.body[i].clone()));
        }
        w.out.rules.push(Clause {
            heads: rule.heads.clone(),
            body,
            group: rule.group,
        });
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::normalize::normalize;
    use crate::syntax::parse_program;

    #[test]
    fn countdown_gets_demand_rules() {
        let p =
            parse_program("0.6::w(T) :- T > 0, P is T - 1, w(P).\nw(0).\nquery(w(3)).\n").unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        let m = rewrite(&n).unwrap().program;
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(text.contains(&"'$m_w_b'(3).".to_string()), "{text:?}");
        assert!(
            text.contains(&"'$m_w_b'(P) :- '$m_w_b'(T), T>0, P is T-1.".to_string()),
            "{text:?}"
        );
        assert!(
            text.iter()
                .any(|t| t.starts_with("w(T) :- '$m_w_b'(T), T>0, P is T-1,")),
            "{text:?}"
        );
        assert!(m.control.contains(&("$m_w_b".into(), 1)));
    }

    /// A predicate asked with its argument free, whose rule reads its own
    /// atoms before the bound head argument, is demanded free only: its
    /// demand under a bound argument would call `a` for every pair of a
    /// bound head and an atom of `r`. Its free demand reads none of the
    /// literals before it, which bind nothing it is asked with.
    #[test]
    fn a_predicate_demanded_free_is_demanded_free_only() {
        let p = parse_program(
            "0.5::r(1).\na(2,1).\na(3,2).\nb(4,3).\n\
             r(Y) :- r(X), a(Y,X).\ns(Z) :- b(Z,X), r(X).\nquery(r(_)).\nquery(s(_)).\n",
        )
        .unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        let m = rewrite(&n).unwrap().program;
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(
            !text
                .iter()
                .any(|t| t.contains("$m_r_b") || t.contains("$m_a_bb")),
            "{text:?}"
        );
        assert!(
            text.iter().any(|t| t == "'$m_r_f' :- '$m_s_f'."),
            "{text:?}"
        );
        assert!(
            text.iter()
                .any(|t| t.starts_with("r(Y) :- '$m_r_f', r(X), a(Y,X)")),
            "{text:?}"
        );
    }

    /// A predicate of ground facts is read as it is, not demanded: its facts
    /// are in the rewritten program once, as they are, and only the rules
    /// of the derived predicates are copied under their demands.
    #[test]
    fn facts_are_read_as_they_are() {
        let p = parse_program(
            "e(1,2).\ne(2,3).\ne(4,5).\n0.5::p(X,Y) :- e(X,Y).\n\
             p(X,Z) :- e(X,Y), p(Y,Z).\nquery(p(1,_)).\n",
        )
        .unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        let m = rewrite(&n).unwrap().program;
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(!text.iter().any(|t| t.contains("$m_e")), "{text:?}");
        let facts: Vec<(&str, usize)> = m
            .facts
            .iter()
            .map(|(_, f)| (&*f.name, f.probs.len()))
            .collect();
        assert_eq!(facts, [("e", 3)]);
        assert!(
            text.iter()
                .any(|t| t.starts_with("p(X,Z) :- '$m_p_bf'(X), e(X,Y), p(Y,Z)")),
            "{text:?}"
        );
    }

    /// A predicate whose rules read only facts is given too, and its
    /// negation filters the demand: the magic rule of `q` reads it, so `q`
    /// is demanded only where `nt` fails.
    #[test]
    fn a_negation_of_a_given_predicate_filters_the_demand() {
        let p = parse_program(
            "lc(s,a,r(a)).\nlc(s,b,r(c)).\nnt(G,B) :- lc(G,B,R), R = r(B).\n\
             0.5::q(X,N) :- lc(s,X,_), N = 1.\n\
             p(X,N) :- lc(s,X,_), \\+ nt(s,X), q(X,N).\nquery(p(b,_)).\n",
        )
        .unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        let m = rewrite(&n).unwrap().program;
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(
            !text
                .iter()
                .any(|t| t.contains("$m_nt") || t.contains("$m_lc")),
            "{text:?}"
        );
        assert!(
            text.iter().any(
                |t| t.starts_with("'$m_q_bf'(X) :- '$m_p_bf'(X), ") && t.contains("\\+nt(s,X)")
            ),
            "{text:?}"
        );
    }

    /// The negation filters the grounded demand too: `nt(s, a)` holds in
    /// every world, so `q` is not demanded for `a`, and is for `b`, where
    /// `nt` fails: the grounding has `q(b,1)` and not `q(a,1)`.
    #[test]
    fn a_negation_of_a_certain_atom_stops_the_grounded_demand() {
        let p = parse_program(
            "lc(s,a,r(a)).\nlc(s,b,r(c)).\nnt(G,B) :- lc(G,B,R), R = r(B).\n\
             0.5::q(X,N) :- lc(s,X,_), N = 1.\n\
             p(X,N) :- lc(s,X,_), \\+ nt(s,X), q(X,N).\nquery(p(a,_)).\nquery(p(b,_)).\n",
        )
        .unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        let m = rewrite(&n).unwrap().program;
        let gp = crate::ground::engine::ground_normalized(&m, None).unwrap();
        let atoms: Vec<String> = gp.atoms.iter().map(|a| a.to_string()).collect();
        assert!(atoms.contains(&"q(b,1)".to_string()), "{atoms:?}");
        assert!(!atoms.contains(&"q(a,1)".to_string()), "{atoms:?}");
    }

    /// A demand reads the literals before it that bear on its bound
    /// arguments: `c` is demanded where `a` holds, whatever `b` holds, and
    /// `b`, with no argument bound, wherever `h` is
    /// (tests/regress/magic_bearing.pl, by hand, its facts made rules on
    /// `s`, as a predicate of facts is given).
    #[test]
    fn a_demand_reads_the_literals_that_bear_on_it() {
        let p = parse_program(
            "s.\n0.5::a(1) :- s.\n0.4::d(5) :- s.\nb(7) :- d(9).\n0.6::c(1,3) :- s.\n\
             h(X) :- a(X), b(Y), c(X,Z).\nquery(h(1)).\n",
        )
        .unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        let m = rewrite(&n).unwrap().program;
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(
            text.contains(&"'$m_c_bf'(X) :- '$m_h_b'(X), a(X).".to_string()),
            "{text:?}"
        );
        assert!(
            text.contains(&"'$m_b_f' :- '$m_h_b'(X).".to_string()),
            "{text:?}"
        );
    }

    /// A demand that binds an argument the body reads only as the value of
    /// an `is` grounds that rule under the demand without it, which the
    /// demand feeds; a rule that reads the argument keeps it.
    /// The demand of a walk that grows its list under `\\+ mem(X, Seen)`
    /// derives itself, and the rewrite cannot read the negation: the
    /// program is resolved top-down. A demand that no recursion derives
    /// is bounded without the negation, and the rewrite is ground; so is
    /// one that a recursion derives in a program that builds no values,
    /// whose rewrite is finite.
    #[test]
    fn a_negation_the_demand_leaves_out_routes_top_down_under_a_recursive_demand() {
        let top_down = |source: &str| {
            let p = parse_program(source).unwrap();
            let (lowered, _) = crate::ground::terms::lower(&p).unwrap();
            let n = normalize(&lowered, Default::default()).unwrap();
            rewrite(&n).unwrap().top_down
        };
        assert!(top_down(
            "mem(X, [X|_]).\nmem(X, [_|T]) :- mem(X, T).\nd(1).\nd(2).\n\
             w(S, S).\nw(S, L) :- d(X), \\+ mem(X, S), w([X|S], L).\nquery(w([], _)).\n"
        ));
        assert!(!top_down(
            "prof(p).\nstud(s).\nta(c, s).\ncourse(c).\n0.5::pos(B) :- prof(B).\n\
             adv(A, B) :- stud(A), prof(B), \\+ pos(B).\naux(A, B) :- ta(A, C), adv(C, B).\n\
             t(A, B) :- course(A), prof(B), \\+ aux(A, B), pos(B).\nquery(t(c, p)).\n"
        ));
        assert!(!top_down(
            "e(1, 2).\ne(2, 3).\ns(3).\nq(1).\n0.5::st(M) :- s(M).\n\
             lv(V) :- e(V, M), \\+ st(M).\npt(X) :- q(X).\n\
             pt(Y) :- pt(X), e(X, Y), lv(Y).\nquery(pt(_)).\n"
        ));
    }

    #[test]
    fn a_binding_read_only_by_is_is_dropped() {
        let p = parse_program(
            "0.5::d(1,0); 0.5::d(1,1).\n0.5::d(2,0); 0.5::d(2,1).\nnext(1,2).\n\
             s(1,S) :- d(1,S).\ns(K,S) :- next(J,K), s(J,S0), d(K,D), S is S0+D.\nquery(s(2,1)).\n",
        )
        .unwrap();
        let n = normalize(&p, Default::default()).unwrap();
        let m = rewrite(&n).unwrap().program;
        let text: Vec<String> = m.rules.iter().map(|r| r.to_string()).collect();
        assert!(
            text.contains(&"'$m_s_bf'(K) :- '$m_s_bb'(K,S).".to_string()),
            "{text:?}"
        );
        assert!(
            text.iter()
                .any(|t| t.starts_with("s(K,S) :- '$m_s_bf'(K),")),
            "{text:?}"
        );
        assert!(
            !text
                .iter()
                .any(|t| t.starts_with("s(K,S) :- '$m_s_bb'(K,S),")),
            "{text:?}"
        );
        assert!(
            text.iter()
                .any(|t| t.starts_with("s(1,S) :- '$m_s_bb'(1,S),")),
            "{text:?}"
        );
    }
}
