//! The order in which `closed` takes the derived components: the
//! compiler's dependency order taken root by root ([`by_roots`]), with the
//! components that belong together merged into one ([`deps::components`]).
//!
//! `closed` lays the facts out in the order the compiler consumes them where
//! nothing joins, so the order of the components decides the vtree. Any
//! dependency order is sound. On a right-linear vtree, and on `closed`'s
//! chains of blocks, a fact stays alive from its first reader to its last,
//! so the order should keep the readers of a fact together. The merge does
//! so for the values of a network variable, which share switch facts or,
//! when deterministic, dependencies and consumers; [`by_roots`] does so for
//! roots that read the same facts.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::ground::{AtomId, GroundProgram, RulesByHead, deps};

/// Components, and facts, beyond which the components are not merged.
/// clinicalTrial1 at width 10 has 8,801 components: merged, its compile
/// took 13.6 times the time and 5.3 times the memory.
const MERGE_UNITS: usize = 4000;
const MERGE_FACTS: usize = 50_000;

/// The components of `sccs` (in a dependency order) as `closed` takes
/// them, still a dependency order: taken root by root ([`by_roots`]), then
/// merged ([`deps::components`]) up to [`MERGE_UNITS`] components and
/// [`MERGE_FACTS`] facts.
pub(super) fn components(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    sccs: Vec<Vec<AtomId>>,
) -> Vec<Vec<AtomId>> {
    let sccs = by_roots(gp, by_head, sccs);
    if sccs.len() > MERGE_UNITS || gp.facts().count() > MERGE_FACTS {
        return sccs;
    }
    deps::components(gp, by_head, sccs)
}

/// `sccs`, a dependency order, taken root by root where the roots' cones
/// share no component. The roots are the components nothing reads: the
/// evidence, the queries, a circuit's outputs. The next root is, of those
/// whose cones share a fact with the roots before it, the one whose cone
/// adds the fewest facts, and where none shares one, the first in the
/// given order; the first in the given order on a tie. So a root that
/// reads the facts already placed goes before one that brings its own,
/// and roots that share nothing keep the given order. Each root's cone
/// follows the roots before it, in the given order. With one root, or
/// where two cones share a component, it is the given order.
///
/// The compiler's order interleaves roots that read different facts. On a
/// right-linear vtree, and on `closed`'s chains of blocks, a fact stays
/// alive from its first reader to its last, so interleaved roots keep the
/// facts of all of them alive at once. clinicalTrial1 observes 400
/// comparisons `u < p` of fresh numbers `u` with one of three parameters
/// `p`, guarded by one fact: in the compiler's order every cut keeps the
/// three parameters' values, and with the observations of each parameter
/// taken together, one. Where cones share components, the given order
/// builds each shared component among its first reader's own, and
/// `closed` lets the first reader's block absorb it; moving the readers'
/// own components away from the shared ones changes which blocks absorb
/// which: on zeroone, whose observations of equal points share their
/// arithmetic, it made the compile up to 13 times the work.
fn by_roots(gp: &GroundProgram, by_head: &RulesByHead, sccs: Vec<Vec<AtomId>>) -> Vec<Vec<AtomId>> {
    let n = sccs.len();
    let mut comp_of = vec![usize::MAX; gp.num_atoms()];
    for (i, c) in sccs.iter().enumerate() {
        for &a in c {
            comp_of[a as usize] = i;
        }
    }
    // The components each component's rules read, and their facts.
    let mut deps: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut facts: Vec<Vec<AtomId>> = vec![Vec::new(); n];
    let mut read = vec![false; n];
    for (i, c) in sccs.iter().enumerate() {
        for &h in c {
            for &ri in &by_head[h as usize] {
                for l in &gp.rules[ri].body {
                    let a = l.atom();
                    let j = comp_of[a as usize];
                    if j == usize::MAX {
                        if gp.is_fact(a) {
                            facts[i].push(a);
                        }
                    } else if j != i {
                        deps[i].push(j);
                        read[j] = true;
                    }
                }
            }
        }
        deps[i].sort_unstable();
        deps[i].dedup();
        facts[i].sort_unstable();
        facts[i].dedup();
    }
    let roots: Vec<usize> = (0..n).filter(|&i| !read[i]).collect();
    if roots.len() < 2 {
        return sccs;
    }
    // The root whose cone holds each component, and the facts of each
    // cone; each component is visited once, as a second cone to reach it
    // ends the walk.
    let mut owner = vec![usize::MAX; n];
    let mut cones: Vec<Vec<AtomId>> = Vec::with_capacity(roots.len());
    let mut roots_of: Vec<Vec<usize>> = vec![Vec::new(); gp.num_atoms()];
    let mut stack: Vec<usize> = Vec::new();
    for (k, &r) in roots.iter().enumerate() {
        let mut cone: Vec<AtomId> = Vec::new();
        owner[r] = k;
        stack.push(r);
        while let Some(c) = stack.pop() {
            for &f in &facts[c] {
                if roots_of[f as usize].last() != Some(&k) {
                    roots_of[f as usize].push(k);
                    cone.push(f);
                }
            }
            for &d in &deps[c] {
                if owner[d] == usize::MAX {
                    owner[d] = k;
                    stack.push(d);
                } else if owner[d] != k {
                    return sccs;
                }
            }
        }
        cones.push(cone);
    }
    // The facts each root's cone would add, and whether it shares one with
    // the roots taken: sharing first, then the fewest added, with stale
    // heap entries passed over.
    let mut adds: Vec<usize> = cones.iter().map(Vec::len).collect();
    let mut taken = vec![false; roots.len()];
    let mut placed = vec![false; gp.num_atoms()];
    let mut heap: BinaryHeap<(bool, Reverse<usize>, Reverse<usize>)> = (0..roots.len())
        .map(|k| (false, Reverse(0), Reverse(k)))
        .collect();
    let mut place = vec![0; roots.len()];
    let mut next = 0;
    while let Some((sharing, Reverse(a), Reverse(k))) = heap.pop() {
        if taken[k] || sharing != (adds[k] < cones[k].len()) || (sharing && a != adds[k]) {
            continue;
        }
        taken[k] = true;
        for &f in &cones[k] {
            if !placed[f as usize] {
                placed[f as usize] = true;
                for &j in &roots_of[f as usize] {
                    if !taken[j] {
                        adds[j] -= 1;
                        heap.push((true, Reverse(adds[j]), Reverse(j)));
                    }
                }
            }
        }
        place[k] = next;
        next += 1;
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| place[owner[i]]);
    let mut sccs = sccs;
    order
        .iter()
        .map(|&i| std::mem::take(&mut sccs[i]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::deps::tests::is_dependency_order;

    /// `src` ground with its derived atoms as written, each its own
    /// component to order (`GroundOptions::views` off).
    fn program(src: &str) -> GroundProgram {
        let p = crate::syntax::parse_program(src).unwrap();
        crate::ground::ground(
            &p,
            &crate::ground::GroundOptions {
                views: false,
                ..crate::ground::GroundOptions::default()
            },
        )
        .unwrap()
    }

    #[test]
    fn the_order_is_a_dependency_order_of_the_same_atoms() {
        let gp = program(
            "0.5::e(1,2). 0.5::e(2,3). 0.5::e(3,1). 0.5::e(1,4). 0.5::e(4,5).
             p(X,Y) :- e(X,Y). p(X,Y) :- e(X,Z), p(Z,Y).
             q(Y) :- p(1,Y). r :- q(5), \\+ q(2). query(r).",
        );
        let by_head = gp.rules_by_head();
        let sccs = deps::derived_sccs(&gp, &by_head);
        assert!(is_dependency_order(&gp, &sccs));
        let order = components(&gp, &by_head, sccs.clone());
        assert!(is_dependency_order(&gp, &order));
        let mut a: Vec<AtomId> = order.iter().flatten().copied().collect();
        let mut b: Vec<AtomId> = sccs.iter().flatten().copied().collect();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b);
    }

    #[test]
    fn roots_that_read_the_same_facts_are_taken_together() {
        // Observations of two parameters `a` and `b`, written in turn, all
        // guarded by `g`: the observations of `a` come first, together,
        // then those of `b`.
        let mut src = String::from("0.5::g. 0.5::a. 0.5::b. q :- g. query(q).\n");
        for i in 1..=4 {
            let p = if i % 2 == 1 { "a" } else { "b" };
            src.push_str(&format!(
                "0.5::u{i}. c{i} :- {p}, u{i}. o{i} :- g, c{i}. evidence(o{i}, false).\n"
            ));
        }
        let gp = program(&src);
        let by_head = gp.rules_by_head();
        let sccs = deps::derived_sccs(&gp, &by_head);
        let order = by_roots(&gp, &by_head, sccs);
        assert!(is_dependency_order(&gp, &order));
        let roots: Vec<String> = order
            .iter()
            .map(|c| gp.atoms[c[0] as usize].to_string())
            .filter(|name| name.starts_with('o'))
            .collect();
        assert_eq!(roots, ["o1", "o3", "o2", "o4"]);
    }

    #[test]
    fn roots_that_share_a_component_keep_the_given_order() {
        // The same observations, now reading `g` through a component `s`
        // (`g` or `h`) they all share: the order stays the compiler's.
        let mut src =
            String::from("0.5::g. 0.5::h. 0.5::a. 0.5::b. s :- g. s :- h. q :- g. query(q).\n");
        for i in 1..=4 {
            let p = if i % 2 == 1 { "a" } else { "b" };
            src.push_str(&format!(
                "0.5::u{i}. c{i} :- {p}, u{i}. o{i} :- s, c{i}. evidence(o{i}, false).\n"
            ));
        }
        let gp = program(&src);
        let by_head = gp.rules_by_head();
        let sccs = deps::derived_sccs(&gp, &by_head);
        assert_eq!(by_roots(&gp, &by_head, sccs.clone()), sccs);
    }
}
