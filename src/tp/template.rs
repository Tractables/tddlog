//! Queries answered by the formula of another query counted under other
//! probabilities ([`templates`]).

use rustc_hash::FxHashMap;

use tididi::vtree::VarId;

use super::VtreeScope;
use crate::eval::Probabilities;
use crate::ground::{AtomId, AtomKind, GLit, GroundProgram, Prob, RulesByHead};
use crate::multisets::Multisets;

/// A query atom answered by the formula of `atom` counted with each fact of
/// `weights` at the probability given there in place of its own.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Template {
    pub(crate) atom: AtomId,
    pub(crate) weights: Vec<(AtomId, Prob)>,
}

impl Template {
    /// `w`, the probabilities of `scope`'s variables, with the facts of
    /// `weights` at theirs.
    pub(crate) fn weights(
        &self,
        w: &Probabilities,
        var_of: &[Option<VarId>],
        scope: &VtreeScope,
    ) -> Probabilities {
        let mut w = w.clone();
        for &(fact, p) in &self.weights {
            let v = var_of[fact as usize].expect("a template's fact has a variable");
            if let Some(local) = scope.local_of(v) {
                w.pos[local.0 as usize] = p.p;
                w.neg[local.0 as usize] = p.not;
                w.one[local.0 as usize] = 1.0;
            }
        }
        w
    }
}

/// The query atoms whose formulas are another query atom's with its facts
/// at other probabilities, each with that atom and those probabilities.
///
/// A leaf is a derived atom that no rule reads, that is not observed and
/// that has no rule of pairs, whose body is no conjunction; a fact is
/// private where only rules of leaves read it and it is not observed. The
/// values of a table's leaf variable are leaves, and so are
/// the switches of its rows: a row `r` of the annotated disjunction makes
/// value `i` by the cube `¬s_r,0 ∧ … ∧ ¬s_r,i-1 ∧ s_r,i` over switches of
/// its own, and value 0 by `s_r,0` alone. Where every rule of a query leaf
/// `y` has one private literal, a positive `t_r`, and the rules of another
/// query leaf `x` are those of `y` with each `t_r` taken by a cube `C_t`
/// of private literals, the cubes of distinct `t` sharing no fact, `x` is
/// `y` with each `t` substituted by `C_t`. The cubes are independent of
/// each other, of the rest of `y`'s rules and of everything observed, so
/// `P(x ∧ e)` is the count of `y ∧ e` with each `t` at the probability
/// `P(C_t)`, and `x`'s formula is never built. A table's values are such
/// leaves, each value's formula over one more switch per row than the one
/// before (bn_barley's tkv).
pub(crate) fn templates(gp: &GroundProgram, by_head: &RulesByHead) -> FxHashMap<AtomId, Template> {
    let n = gp.num_atoms();
    let mut read = vec![false; n];
    // The heads of rules of pairs, whose bodies are no conjunctions.
    let mut pairs = vec![false; n];
    for r in &gp.rules {
        for l in &r.body {
            read[l.atom() as usize] = true;
        }
        pairs[r.head as usize] |= r.pairs;
    }
    let mut observed = vec![false; n];
    for &(_, id, _) in &gp.evidence {
        if let Some(a) = id {
            observed[a as usize] = true;
        }
    }
    let leaf = |a: AtomId| {
        gp.kinds[a as usize] == AtomKind::Derived
            && !read[a as usize]
            && !observed[a as usize]
            && !pairs[a as usize]
    };
    // A fact is private until a rule whose head is no leaf reads it.
    let mut private: Vec<bool> = (0..n)
        .map(|a| gp.is_fact(a as AtomId) && !observed[a])
        .collect();
    for r in &gp.rules {
        if !leaf(r.head) {
            for l in &r.body {
                private[l.atom() as usize] = false;
            }
        }
    }
    let probability = |a: AtomId| match gp.kinds[a as usize] {
        AtomKind::Fact(p) => p,
        AtomKind::Derived => unreachable!("a private atom is a fact"),
    };
    // A rule's literals other than private ones, and its private ones,
    // sorted, into buffers kept across the rules rather than two new lists
    // per rule. The rest is compared as a multiset ([`Multisets`]) and not
    // sorted.
    let split = |ri: usize, rest: &mut Vec<GLit>, own: &mut Vec<GLit>| {
        rest.clear();
        own.clear();
        for &l in &gp.rules[ri].body {
            if private[l.atom() as usize] {
                own.push(l)
            } else {
                rest.push(l)
            }
        }
        own.sort_unstable();
    };
    let (mut rest, mut own): (Vec<GLit>, Vec<GLit>) = (Vec::new(), Vec::new());
    let mut queried: Vec<AtomId> = gp
        .queries
        .iter()
        .filter_map(|&(_, id, _)| id)
        .filter(|&a| leaf(a))
        .collect();
    queried.sort_unstable();
    queried.dedup();
    // The templates, each claiming the rest of its rules' bodies: the
    // bodies, and per body the template and the private literal of its rule,
    // numbered alike; and per template the number of its first rule's body,
    // `u32::MAX` for an atom that is none.
    let mut claimed = Multisets::new(n);
    let mut claims: Vec<(AtomId, AtomId)> = Vec::new();
    let mut claims_from = vec![u32::MAX; n];
    // A query leaf's bodies, and per body the private literal of its rule.
    let mut bodies = Multisets::new(n);
    let mut privates: Vec<AtomId> = Vec::new();
    for &y in &queried {
        let rules = &by_head[y as usize];
        if rules.is_empty() {
            continue;
        }
        bodies.clear();
        privates.clear();
        let ok = rules.iter().all(|&ri| {
            split(ri, &mut rest, &mut own);
            match own[..] {
                [GLit::Pos(t)] => {
                    claimed.find(&rest).is_none() && bodies.id(&rest).1 && {
                        privates.push(t);
                        true
                    }
                }
                _ => false,
            }
        });
        if ok {
            claims_from[y as usize] = claims.len() as u32;
            for (k, &t) in privates.iter().enumerate() {
                claimed.id(bodies.get(k as u32));
                claims.push((y, t));
            }
        }
    }
    // The query leaf each claimed body was last read for.
    let mut read_for = vec![AtomId::MAX; claims.len()];
    // Per query leaf, the cube of each `t`, as a range of `cubes`, and the
    // `t` whose cube each fact is in, kept from one leaf to the next rather
    // than a list per cube and two maps per leaf.
    let mut cubes: Vec<GLit> = Vec::new();
    let mut cube_of: FxHashMap<AtomId, (usize, usize)> = FxHashMap::default();
    let mut owner: FxHashMap<AtomId, AtomId> = FxHashMap::default();
    let mut out = FxHashMap::default();
    for &x in &queried {
        let rules = &by_head[x as usize];
        if claims_from[x as usize] != u32::MAX || rules.is_empty() {
            continue;
        }
        split(rules[0], &mut rest, &mut own);
        let Some(k) = claimed.find(&rest) else {
            continue;
        };
        let (y, _) = claims[k as usize];
        if rules.len() != by_head[y as usize].len() {
            continue;
        }
        // Each `t` of `y` and the cube that takes it in `x`'s rules. The
        // rules of `x` are as many as `y`'s, each body one of `y`'s and no
        // two the same, so each rule of `y` has its rule of `x`.
        cubes.clear();
        cube_of.clear();
        // The rules of `x` mostly come in the order of `y`'s, each with the
        // rest of its counterpart's body as it is, and the claimed bodies are
        // distinct: the one its counterpart claimed is then the one found.
        let from = claims_from[y as usize];
        let ok = rules.iter().enumerate().all(|(i, &ri)| {
            if i > 0 {
                split(ri, &mut rest, &mut own);
            }
            let counterpart = from + i as u32;
            let k = if claimed.get(counterpart) == rest.as_slice() {
                Some(counterpart)
            } else {
                claimed.find(&rest)
            };
            match k.map(|k| (k as usize, claims[k as usize])) {
                Some((k, (z, t))) if z == y && std::mem::replace(&mut read_for[k], x) != x => {
                    match cube_of.get(&t) {
                        Some(&(start, end)) => cubes[start..end] == own[..],
                        None => {
                            cube_of.insert(t, (cubes.len(), cubes.len() + own.len()));
                            cubes.extend_from_slice(&own);
                            true
                        }
                    }
                }
                _ => false,
            }
        });
        // The cubes of distinct `t` share no fact.
        if !ok {
            continue;
        }
        owner.clear();
        let disjoint = cube_of.iter().all(|(&t, &(start, end))| {
            cubes[start..end]
                .iter()
                .all(|l| *owner.entry(l.atom()).or_insert(t) == t)
        });
        if !disjoint {
            continue;
        }
        let mut weights: Vec<(AtomId, Prob)> = cube_of
            .iter()
            .map(|(&t, &(start, end))| {
                (
                    t,
                    Prob::cube(
                        cubes[start..end]
                            .iter()
                            .map(|&l| probability(l.atom()).literal(l.is_positive())),
                    ),
                )
            })
            .collect();
        weights.sort_unstable_by_key(|&(t, _)| t);
        out.insert(x, Template { atom: y, weights });
    }
    out
}
