//! The lists a findall gives: one function for the bottom-up grounder
//! ([`super::collect`]) and the top-down resolution ([`super::topdown`]).
//!
//! ProbLog lists a findall's solutions once per proof, a refutation of the
//! goal by resolution, and where the goal reads probabilistic facts, the
//! list is a distribution: in each world, the solutions whose proofs hold
//! there. A proof is a conjunction of the probabilistic facts and the
//! negations it reads, the derived atoms it reads expanded into their own
//! proofs ([`proofs`]). The proofs that hold in every world come first, in
//! the order resolution finds them; then the others, in the order ProbLog
//! made their formulas' nodes: a proof of one literal where its atom was
//! first read, a proof of several where its conjunction was made, after
//! its literals ([`lists`]).
//!
//! The lists are built through the proofs in that order, one state of the
//! list so far at a time: each proof that holds in some worlds only splits
//! a state into the list with its solution and the list without, and the
//! states that reach the same list merge into one atom. A list is then an
//! atom with a rule for each way to it, rather than a rule for each set of
//! proofs that holds. A goal over solutions that are all different still
//! has a state for each set: past [`MAX_LISTS`] states, or a solution past
//! [`MAX_PROOFS`] proofs, the grounding is given up with the findall named
//! (as it is past the grounding's bound, which the rules count against).
//!
//! all/3 lists each solution once, by the atom that answers it, and fails
//! where the list would be empty.
//!
//! A findall whose result is bound, when it is called, to what is neither a
//! list nor a list whose tail is a variable gives the grounding up
//! ([`not_a_list`]), as ProbLog stops on it with a call mode error.

use std::rc::Rc;

use rustc_hash::FxHashMap;

use super::{GroundError, Val};

/// The predicate of the atoms the lists are built of, a state each.
pub(super) const LISTS: &str = "$fl";

/// The most proofs a findall reads of one atom.
pub(super) const MAX_PROOFS: usize = 1 << 16;

/// The most lists a findall's goal may give for one call.
pub(super) const MAX_LISTS: usize = 1 << 16;

/// The error of the findall `what` called with its result bound to
/// `result`, which is no list.
pub(crate) fn not_a_list(what: &str, result: &impl std::fmt::Display) -> GroundError {
    GroundError::Refused(format!(
        "{what}: called with its result {result}, which is no list (ProbLog's call mode error)"
    ))
}

/// A literal: whether positive, and its atom.
pub(super) type Lit = (bool, u32);

/// A list a findall gives, and the literal of the worlds where it is the
/// list (none where it is the one list).
pub(super) type Listed = (Val, Option<Lit>);

/// One proof: its place in the order of resolution (the clause, then what
/// each literal of its body reads, in the clause's order), and the
/// literals it reads that hold in some worlds only.
#[derive(Clone, Debug)]
pub(super) struct Proof {
    pub(super) key: Rc<[u32]>,
    pub(super) lits: Rc<[Lit]>,
}

/// What a body literal of a derivation reads, in the clause's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Step {
    Pos(u32),
    Neg(u32),
    /// The rank of the solution of a built-in that enumerates them.
    Rank(u32),
}

/// An atom, as the proofs that read it see it.
pub(super) enum Node {
    /// True in every world.
    True,
    /// False in every world.
    False,
    /// A probabilistic fact or choice: one proof, itself.
    Leaf,
    /// Derived by rules: each derivation's clause and what its body reads.
    Derived(Vec<(u32, Vec<Step>)>),
}

/// The grounding a findall's lists are made in.
pub(super) trait Source {
    /// What atom `a` is to a proof.
    fn node(&mut self, a: u32) -> Node;
    /// Whether atom `a` is true in every world, or in none.
    fn truth(&mut self, a: u32) -> Option<bool>;
    /// The proofs known of each atom ([`proofs`]).
    fn known(&mut self) -> &mut FxHashMap<u32, Rc<[Proof]>>;
    /// The list of `items`.
    fn list(&mut self, items: &[Val]) -> Val;
    /// Atom `a`, as written.
    fn show(&mut self, a: u32) -> String;
    /// A new atom, which only the rules given it by [`Source::rule`] derive.
    fn fresh(&mut self) -> Result<u32, GroundError>;
    /// Adds the rule `head :- body`.
    fn rule(&mut self, head: u32, body: &[Lit]) -> Result<(), GroundError>;
}

/// The proofs of atom `a`, in the order of resolution; `what` names the
/// findall that reads them, for an error. A proof that reads its own atom
/// again, a cycle, gives the grounding up.
pub(super) fn proofs(
    src: &mut impl Source,
    a: u32,
    what: &str,
) -> Result<Rc<[Proof]>, GroundError> {
    proofs_on(src, a, what, &mut Vec::new())
}

fn proofs_on(
    src: &mut impl Source,
    a: u32,
    what: &str,
    path: &mut Vec<u32>,
) -> Result<Rc<[Proof]>, GroundError> {
    if let Some(p) = src.known().get(&a) {
        return Ok(p.clone());
    }
    if path.contains(&a) {
        return Err(GroundError::Refused(format!(
            "{what}: {} has a proof that reads it again",
            src.show(a)
        )));
    }
    let empty: Rc<[u32]> = Rc::from([]);
    let out: Rc<[Proof]> = match src.node(a) {
        Node::True => Rc::from([Proof {
            key: empty,
            lits: Rc::from([]),
        }]),
        Node::False => Rc::from([]),
        Node::Leaf => Rc::from([Proof {
            key: empty,
            lits: Rc::from([(true, a)]),
        }]),
        Node::Derived(derivations) => {
            path.push(a);
            let mut out: Vec<Proof> = Vec::new();
            for (group, steps) in derivations {
                let mut partial: Vec<(Vec<u32>, Vec<Lit>)> = vec![(vec![group], Vec::new())];
                for step in steps {
                    match step {
                        Step::Rank(r) => partial.iter_mut().for_each(|(k, _)| k.push(r)),
                        // A negation is one literal, but of an atom true in
                        // every world, a proof of no literals, or in none.
                        Step::Neg(b) => {
                            let truth = match src.truth(b) {
                                Some(t) => Some(t),
                                None => {
                                    let sub = proofs_on(src, b, what, path)?;
                                    match sub.iter().any(|p| p.lits.is_empty()) {
                                        true => Some(true),
                                        false => sub.is_empty().then_some(false),
                                    }
                                }
                            };
                            match truth {
                                Some(true) => partial.clear(),
                                Some(false) => {}
                                None => partial.iter_mut().for_each(|(_, l)| l.push((false, b))),
                            }
                        }
                        Step::Pos(b) => {
                            let sub = proofs_on(src, b, what, path)?;
                            if partial.len().saturating_mul(sub.len()) > MAX_PROOFS {
                                path.pop();
                                return Err(too_many(src, a, what));
                            }
                            partial = partial
                                .iter()
                                .flat_map(|(k, l)| {
                                    sub.iter().map(move |p| {
                                        let mut k = k.clone();
                                        k.extend_from_slice(&p.key);
                                        let mut l = l.clone();
                                        l.extend_from_slice(&p.lits);
                                        (k, l)
                                    })
                                })
                                .collect();
                        }
                    }
                    if partial.is_empty() {
                        break;
                    }
                }
                out.extend(partial.into_iter().map(|(k, l)| Proof {
                    key: Rc::from(k),
                    lits: Rc::from(l),
                }));
                if out.len() > MAX_PROOFS {
                    path.pop();
                    return Err(too_many(src, a, what));
                }
            }
            path.pop();
            out.sort_by(|x, y| x.key.cmp(&y.key));
            Rc::from(out)
        }
    };
    src.known().insert(a, out.clone());
    Ok(out)
}

fn too_many(src: &mut impl Source, a: u32, what: &str) -> GroundError {
    GroundError::Refused(format!(
        "{what}: {} has more than {MAX_PROOFS} proofs, each a solution of the findall",
        src.show(a)
    ))
}

/// The lists of a findall whose goal has the solutions `answers`, each its
/// template's value and the atom that answers it, in the order found;
/// `all` and `what` as [`lists`] reads them.
pub(super) fn collect(
    src: &mut impl Source,
    answers: &[(Val, u32)],
    all: bool,
    what: &str,
) -> Result<Vec<Listed>, GroundError> {
    let mut found: Vec<(Val, Proof)> = Vec::with_capacity(answers.len());
    for &(v, a) in answers {
        let proofs = proofs(src, a, what)?;
        if all {
            // all/3 reads each solution once, as its atom.
            if let Some(first) = proofs.first() {
                let lits: Rc<[Lit]> = if proofs.iter().any(|p| p.lits.is_empty()) {
                    Rc::from([])
                } else {
                    Rc::from([(true, a)])
                };
                let key = first.key.clone();
                found.push((v, Proof { key, lits }));
            }
        } else {
            found.extend(proofs.iter().map(|p| (v, p.clone())));
        }
    }
    found.sort_by(|x, y| x.1.key.cmp(&y.1.key));
    lists(src, found, all, what)
}

/// The lists the solutions `found` give, each with the literal that says
/// in which worlds (none where it is the one list): `found` holds each
/// solution's template value and one proof, in the order of resolution;
/// `all` lists them as all/3 does, each a solution once, by its atom, and
/// no empty list. `what` names the findall, for an error.
fn lists(
    src: &mut impl Source,
    mut found: Vec<(Val, Proof)>,
    all: bool,
    what: &str,
) -> Result<Vec<Listed>, GroundError> {
    if !all {
        // ProbLog's order: the proofs that hold in every world first, then
        // the others by the node of each, numbered as made: an atom where
        // it was first read, a conjunction after its literals.
        // A negation alone is its atom's node negated, which ProbLog's
        // sort puts before every other, the last made first.
        let mut made: FxHashMap<u32, i64> = FxHashMap::default();
        let mut conj: FxHashMap<Vec<Lit>, i64> = FxHashMap::default();
        let mut next = 0i64;
        let mut node = |lits: &[Lit]| -> i64 {
            let mut last = 0;
            for &(_, a) in lits {
                last = *made.entry(a).or_insert_with(|| {
                    next += 1;
                    next
                });
            }
            match lits {
                [] => 0,
                [(false, _)] => -last,
                [_] => last,
                _ => {
                    let mut key = lits.to_vec();
                    key.sort_unstable();
                    key.dedup();
                    *conj.entry(key).or_insert_with(|| {
                        next += 1;
                        next
                    })
                }
            }
        };
        let order: Vec<i64> = found.iter().map(|(_, p)| node(&p.lits)).collect();
        let mut ranked: Vec<(i64, (Val, Proof))> = order.into_iter().zip(found).collect();
        ranked.sort_by_key(|(o, _)| *o);
        found = ranked.into_iter().map(|(_, f)| f).collect();
    }
    // The proofs of one formula, the same literals, hold together: a set
    // of proofs that takes one of them and leaves another holds nowhere,
    // and ProbLog drops it. Each such formula is decided once, where its
    // first proof comes, and the decision kept until its last.
    let mut formulas: FxHashMap<Vec<Lit>, usize> = FxHashMap::default();
    let mut of: Vec<Option<usize>> = Vec::with_capacity(found.len());
    let mut last: Vec<usize> = Vec::new();
    for (i, (_, proof)) in found.iter().enumerate() {
        if proof.lits.is_empty() {
            of.push(None);
            continue;
        }
        let mut key = proof.lits.to_vec();
        key.sort_unstable();
        key.dedup();
        let n = formulas.len();
        let f = *formulas.entry(key).or_insert(n);
        if f == last.len() {
            last.push(i);
        }
        last[f] = i;
        of.push(Some(f));
    }
    let mut holds: Vec<Option<Lit>> = vec![None; last.len()];
    // Each state: the list so far, the formulas decided that come again,
    // and the literal of the worlds it is in.
    // A place: the list so far and the formulas decided that come again.
    type Place = (Vec<Val>, Vec<(usize, bool)>);
    type State = (Vec<Val>, Vec<(usize, bool)>, Option<Lit>);
    let mut states: Vec<State> = vec![(Vec::new(), Vec::new(), None)];
    for (i, ((value, proof), f)) in found.into_iter().zip(of).enumerate() {
        let Some(f) = f else {
            for (items, _, _) in &mut states {
                items.push(value);
            }
            continue;
        };
        let l = match holds[f] {
            Some(l) => l,
            None => {
                let l = match *proof.lits {
                    [l] => l,
                    _ => {
                        let p = src.fresh()?;
                        src.rule(p, &proof.lits)?;
                        (true, p)
                    }
                };
                holds[f] = Some(l);
                l
            }
        };
        let again = last[f] > i;
        // The states reached, each with its ways in.
        let mut reached: Vec<(Place, Vec<Vec<Lit>>)> = Vec::new();
        let mut at: FxHashMap<Place, usize> = FxHashMap::default();
        for (items, mut decided, cond) in states {
            let known = decided.iter().position(|&(g, _)| g == f);
            let choices: Vec<(bool, Option<Lit>)> = match known {
                Some(k) => {
                    let d = decided[k].1;
                    if !again {
                        decided.remove(k);
                    }
                    vec![(d, None)]
                }
                None => vec![(true, Some(l)), (false, Some((!l.0, l.1)))],
            };
            for (d, lit) in choices {
                let mut items = items.clone();
                if d {
                    items.push(value);
                }
                let mut decided = decided.clone();
                if known.is_none() && again {
                    decided.push((f, d));
                }
                let body: Vec<Lit> = cond.into_iter().chain(lit).collect();
                let key = (items, decided);
                match at.get(&key) {
                    Some(&k) => reached[k].1.push(body),
                    None => {
                        at.insert(key.clone(), reached.len());
                        reached.push((key, vec![body]));
                    }
                }
            }
        }
        if reached.len() > MAX_LISTS {
            return Err(GroundError::Refused(format!(
                "{what}: more than {MAX_LISTS} lists of solutions, one for each set of \
                 proofs that holds in some worlds only"
            )));
        }
        states = Vec::with_capacity(reached.len());
        for ((items, decided), ways) in reached {
            let cond = match &ways[..] {
                [way] if way.len() <= 1 => way.first().copied(),
                _ => {
                    let s = src.fresh()?;
                    for way in &ways {
                        src.rule(s, way)?;
                    }
                    Some((true, s))
                }
            };
            states.push((items, decided, cond));
        }
    }
    Ok(states
        .into_iter()
        .filter(|(items, _, _)| !(all && items.is_empty()))
        .map(|(items, _, cond)| (src.list(&items), cond))
        .collect())
}
