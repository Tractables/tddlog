//! Linear recursion written without recursion, by vertex elimination:
//! Gaussian elimination for the algebraic path problem (Tarjan, "A unified
//! approach to path problems", JACM 1981), used as cycle breaking.
//!
//! A recursive component in which every rule reads at most one atom of the
//! component, and that one positively, is a reachability problem. Take a
//! node for each atom of the component and a source node `s`. A rule
//! `h :- b, L`, with `b` in the component and `L` the rest of its body, is
//! an edge from `b` to `h` labelled `L`; a rule `h :- L` that reads no atom
//! of the component is an edge from `s` to `h`. In each world the least
//! model holds `h` exactly when some path leads from `s` to `h` along edges
//! whose labels hold there: the labels read only atoms below the component,
//! which are two-valued in every world, and each rule passes truth along
//! one edge.
//!
//! Eliminating a node `k` connects each node `i` with an edge into `k` to
//! each node `j` with an edge out of it, `i ≠ j`: the label of `i → j`
//! gains the disjunct `label(i → k) ∧ label(k → j)`. After the nodes `v₁ …
//! v_{k-1}` are gone, the label of `i → j` holds in a world exactly when a
//! path from `i` to `j` runs there with every interior node among them
//! (Floyd and Warshall's closure, one world at a time). Then
//!
//! ```text
//! v_k  ⇔  A(k) ∨ ⋁_{j > k} v_j ∧ L(j, k)
//! ```
//!
//! where `A(k)` and `L(j, k)` are the labels of `s → v_k` and `v_j → v_k` as
//! `v_k` goes: of a path from `s` to `v_k`, take the last node before `v_k`
//! that goes after it; the rest of the path runs through nodes that went
//! before `v_k`. Each node reads only nodes that go after it, so the
//! program becomes acyclic, and its atoms keep their names, so queries,
//! evidence and the components above read them as before. A label that is
//! a disjunction of several bodies and is conjoined with another becomes an
//! atom of its own, `$e(n)`; a single body is copied.
//!
//! The order is greedy on the labels standing ([`elimination_order`]), the
//! atoms read outside the component (by a rule above it, a query or the
//! evidence) last: the last atom to go reads no atom after it, and one
//! queried atom of a grid then reads one label.

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::OnceLock;

use rustc_hash::{FxHashMap, FxHashSet};

use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, RulesByHead, deps, simplify};
use crate::syntax::{Args, Atom, Term};

/// Names the atoms that hold a label of several bodies. Cannot clash with a
/// user predicate, whose name never contains `$`.
const LABEL_NAME: &str = "$e";
/// Label products, an edge into a node by an edge out of it, that the
/// rewrite of a program may take, each a conjoined body; the components
/// past it are left recursive. The products of a dense component grow with
/// the cube of its nodes. The bidirectional grid of 64 nodes takes 1 863,
/// sickness22_40's 40 components 2 233 together, and a random graph of 400
/// nodes and 1 200 edges 72 833.
const MAX_PRODUCTS: u64 = 1 << 22;

/// A label: a disjunction of bodies, each a conjunction of literals on atoms
/// below the component. `[[]]` is true.
type Label = Vec<Vec<GLit>>;

/// Whether the component is recursive and linear: every rule of it reads at
/// most one of its atoms, and that one positively.
fn linear(gp: &GroundProgram, by_head: &RulesByHead, scc: &[AtomId]) -> bool {
    deps::is_cyclic(gp, by_head, scc)
        && scc.iter().all(|&h| {
            by_head[h as usize].iter().all(|&ri| {
                let mut reads = gp.rules[ri].body.iter().filter(|l| scc.contains(&l.atom()));
                match (reads.next(), reads.next()) {
                    (None, _) => true,
                    (Some(l), None) => l.is_positive(),
                    _ => false,
                }
            })
        })
}

/// Rewrites every linear recursive component of `gp` by vertex elimination,
/// and returns how many it rewrote.
pub(crate) fn linear_recursion(gp: &mut GroundProgram) -> usize {
    let structure = gp.structure();
    let by_head = &structure.by_head;
    let linear: Vec<&Vec<AtomId>> = structure
        .sccs
        .iter()
        .filter(|scc| linear(gp, by_head, scc))
        .collect();
    if linear.is_empty() {
        return 0;
    }
    // The atoms read from outside their component.
    let mut component = vec![usize::MAX; gp.num_atoms()];
    for (c, scc) in structure.sccs.iter().enumerate() {
        for &a in scc {
            component[a as usize] = c;
        }
    }
    let mut needed = vec![false; gp.num_atoms()];
    for r in &gp.rules {
        for l in &r.body {
            if component[l.atom() as usize] != component[r.head as usize] {
                needed[l.atom() as usize] = true;
            }
        }
    }
    for a in gp.named_atoms() {
        needed[a as usize] = true;
    }
    let mut dropped = vec![false; gp.rules.len()];
    let mut out = Out {
        gp,
        emitted: Vec::new(),
        next_label: 0,
    };
    let mut budget = MAX_PRODUCTS;
    let mut rewritten = 0;
    for scc in &linear {
        if eliminate(&mut out, by_head, scc, &needed, &mut dropped, &mut budget) {
            rewritten += 1;
        }
    }
    if rewritten == 0 {
        return 0;
    }
    let Out { gp, emitted, .. } = out;
    let old = std::mem::take(&mut gp.rules);
    gp.rules = old
        .into_iter()
        .zip(&dropped)
        .filter(|(_, d)| !**d)
        .map(|(r, _)| r)
        .chain(emitted)
        .collect();
    gp.structure = OnceLock::new();
    simplify::restrict_to_relevant(gp);
    let order = by_rounds(gp);
    simplify::renumber(gp, &order);
    rewritten
}

/// The atoms in the order a bottom-up evaluation derives them: the facts,
/// then each derived atom in the round after the last body atom of its
/// first rule to complete, ties by number. The grounder numbers a source
/// program so; `closed` starts its order search from that numbering.
fn by_rounds(gp: &GroundProgram) -> Vec<AtomId> {
    let n = gp.num_atoms();
    let mut readers: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut waiting: Vec<usize> = Vec::with_capacity(gp.rules.len());
    for (ri, r) in gp.rules.iter().enumerate() {
        let mut body: Vec<AtomId> = r.body.iter().map(|l| l.atom()).collect();
        body.sort_unstable();
        body.dedup();
        for &a in &body {
            readers[a as usize].push(ri);
        }
        waiting.push(body.len());
    }
    let mut placed = vec![false; n];
    let mut order: Vec<AtomId> = Vec::with_capacity(n);
    let mut round: Vec<AtomId> = (0..n as AtomId)
        .filter(|&a| gp.kinds[a as usize] != AtomKind::Derived)
        .collect();
    round.extend(
        gp.rules
            .iter()
            .enumerate()
            .filter(|(ri, _)| waiting[*ri] == 0)
            .map(|(_, r)| r.head),
    );
    while !round.is_empty() {
        round.sort_unstable();
        round.dedup();
        round.retain(|&a| !placed[a as usize]);
        let mut next = Vec::new();
        for &a in &round {
            placed[a as usize] = true;
            order.push(a);
        }
        for &a in &round {
            for &ri in &readers[a as usize] {
                waiting[ri] -= 1;
                if waiting[ri] == 0 && !placed[gp.rules[ri].head as usize] {
                    next.push(gp.rules[ri].head);
                }
            }
        }
        round = next;
    }
    order.extend((0..n as AtomId).filter(|&a| !placed[a as usize]));
    order
}

/// The program being rewritten and the rules the rewrite emits.
struct Out<'a> {
    gp: &'a mut GroundProgram,
    emitted: Vec<GroundRule>,
    next_label: i64,
}

impl Out<'_> {
    /// The label as one body: itself where it has one, else an atom of its
    /// own, which the label then becomes.
    fn body(&mut self, label: &mut Label) -> Vec<GLit> {
        if label.len() == 1 {
            return label[0].clone();
        }
        let a = self.gp.atoms.len() as AtomId;
        self.gp
            .atoms
            .push(Atom::new(LABEL_NAME, Args::One(Term::Int(self.next_label))));
        self.gp.kinds.push(AtomKind::Derived);
        self.next_label += 1;
        for body in label.drain(..) {
            self.emitted.push(GroundRule::new(a, body));
        }
        label.push(vec![GLit::Pos(a)]);
        vec![GLit::Pos(a)]
    }
}

/// Adds a disjunct to a label, keeping `[[]]` for true.
fn add(label: &mut Label, body: Vec<GLit>) {
    if label.len() == 1 && label[0].is_empty() {
        return;
    }
    if body.is_empty() {
        label.clear();
        label.push(body);
    } else if !label.contains(&body) {
        label.push(body);
    }
}

/// The conjunction of two bodies, `None` where it reads an atom both ways.
fn conjoin(a: &[GLit], b: &[GLit]) -> Option<Vec<GLit>> {
    let mut body: Vec<GLit> = a.iter().chain(b).copied().collect();
    body.sort_unstable();
    body.dedup();
    let contradicts = body.windows(2).any(|w| w[0].atom() == w[1].atom());
    (!contradicts).then_some(body)
}

/// Rewrites one linear recursive component, marking its rules dropped.
/// Rewrites one component, unless its elimination would take more than
/// `budget` label products, which it then lowers by the products taken.
/// Returns whether it rewrote the component.
fn eliminate(
    out: &mut Out,
    by_head: &RulesByHead,
    scc: &[AtomId],
    needed: &[bool],
    dropped: &mut [bool],
    budget: &mut u64,
) -> bool {
    let n = scc.len();
    let source = n;
    let local: FxHashMap<AtomId, usize> = scc.iter().enumerate().map(|(i, &a)| (a, i)).collect();
    // The labels by tail and head; an edge is present where its label is.
    let mut labels: Vec<FxHashMap<usize, Label>> = vec![FxHashMap::default(); n + 1];
    for (h, &head) in scc.iter().enumerate() {
        for &ri in &by_head[head as usize] {
            let body = &out.gp.rules[ri].body;
            match body.iter().position(|l| local.contains_key(&l.atom())) {
                Some(at) => {
                    let b = local[&body[at].atom()];
                    if b != h {
                        let rest: Vec<GLit> = body
                            .iter()
                            .enumerate()
                            .filter(|&(i, _)| i != at)
                            .map(|(_, l)| *l)
                            .collect();
                        add(labels[b].entry(h).or_default(), rest);
                    }
                }
                None => add(labels[source].entry(h).or_default(), body.clone()),
            }
        }
    }
    let last: Vec<bool> = scc.iter().map(|&a| needed[a as usize]).collect();
    let Some((order, products)) = elimination_order(&labels, &last, *budget) else {
        return false;
    };
    *budget -= products;
    for &head in scc {
        for &ri in &by_head[head as usize] {
            dropped[ri] = true;
        }
    }
    // What each node's rules read as it goes: A(k), and L(j, k) by j.
    let mut reach: Vec<(Label, Vec<(usize, Label)>)> = vec![(Vec::new(), Vec::new()); n];
    let mut gone = vec![false; n + 1];
    for &k in &order {
        let k = k as usize;
        gone[k] = true;
        let mut into: Vec<usize> = (0..=n)
            .filter(|&i| !gone[i] && labels[i].contains_key(&k))
            .collect();
        into.sort_unstable();
        let mut onto: Vec<usize> = labels[k].keys().copied().filter(|&j| !gone[j]).collect();
        onto.sort_unstable();
        let mut from: Vec<(usize, Vec<GLit>)> = Vec::with_capacity(into.len());
        for &i in &into {
            let mut label = labels[i].remove(&k).expect("an edge into the node");
            // Where nothing goes on from k, only k's own rules read the
            // label, one rule per body.
            if !onto.is_empty() {
                from.push((i, out.body(&mut label)));
            }
            if i == source {
                reach[k].0 = label;
            } else {
                reach[k].1.push((i, label));
            }
        }
        let mut to: Vec<(usize, Vec<GLit>)> = Vec::with_capacity(onto.len());
        for &j in &onto {
            let mut label = labels[k].remove(&j).expect("an edge out of the node");
            to.push((j, out.body(&mut label)));
        }
        labels[k].clear();
        for (i, a) in &from {
            for (j, b) in &to {
                if i != j
                    && let Some(body) = conjoin(a, b)
                {
                    add(labels[*i].entry(*j).or_default(), body);
                }
            }
        }
    }
    for (k, (a, l)) in reach.into_iter().enumerate() {
        let head = scc[k];
        for body in a {
            out.emitted.push(GroundRule::new(head, body));
        }
        for (j, label) in l {
            for body in label {
                let mut full = vec![GLit::Pos(scc[j])];
                full.extend(body);
                out.emitted.push(GroundRule::new(head, full));
            }
        }
    }
    true
}

/// The order the component's nodes go in, the source last and the nodes
/// marked in `last` after the others: each step the node whose elimination
/// leaves the fewest fill labels, the labels elimination made, standing,
/// ties broken on the node's neighbours and then its number. The new labels
/// are the pairs of an edge in and an edge out that hold no fill label yet;
/// the ones that go are the fill labels at the node. The diagrams of the
/// labels standing at once are what the compile holds, and what `closed`'s
/// right-linear stretches charge.
///
/// The labels' presence is simulated on its own, the bodies left for the
/// rewrite. A node's score changes only when a label at it or between two of
/// its neighbours does, so after each step its neighbours and theirs are
/// scored again; the heap is corrected on pop, as in
/// `vtree::graph::min_fill_elimination`.
///
/// Returns the order and the label products the rewrite takes along it, an
/// edge in by an edge out at each step, or `None` once those pass `limit`.
fn elimination_order(
    labels: &[FxHashMap<usize, Label>],
    last: &[bool],
    limit: u64,
) -> Option<(Vec<u32>, u64)> {
    let n = last.len();
    let mut onto: Vec<FxHashSet<usize>> = labels
        .iter()
        .map(|out_of| out_of.keys().copied().collect())
        .collect();
    let mut into: Vec<FxHashSet<usize>> = vec![FxHashSet::default(); n + 1];
    for (i, heads) in onto.iter().enumerate() {
        for &j in heads {
            into[j].insert(i);
        }
    }
    let mut fill: FxHashSet<(usize, usize)> = FxHashSet::default();
    let score = |v: usize,
                 onto: &[FxHashSet<usize>],
                 into: &[FxHashSet<usize>],
                 fill: &FxHashSet<(usize, usize)>| {
        let made = into[v]
            .iter()
            .flat_map(|&i| onto[v].iter().map(move |&j| (i, j)))
            .filter(|&(i, j)| i != j && !fill.contains(&(i, j)))
            .count();
        let gone = into[v].iter().filter(|&&i| fill.contains(&(i, v))).count()
            + onto[v].iter().filter(|&&j| fill.contains(&(v, j))).count();
        let neighbours = into[v].union(&onto[v]).count();
        (last[v], made as i64 - gone as i64, neighbours, v)
    };
    let mut heap: BinaryHeap<Reverse<(bool, i64, usize, usize)>> = (0..n)
        .map(|v| Reverse(score(v, &onto, &into, &fill)))
        .collect();
    let mut gone = vec![false; n];
    let mut order = Vec::with_capacity(n);
    let mut products = 0u64;
    while let Some(Reverse(scored)) = heap.pop() {
        let v = scored.3;
        if gone[v] {
            continue;
        }
        let now = score(v, &onto, &into, &fill);
        if now != scored {
            heap.push(Reverse(now));
            continue;
        }
        gone[v] = true;
        order.push(v as u32);
        let tails: Vec<usize> = into[v].iter().copied().collect();
        let heads: Vec<usize> = onto[v].iter().copied().collect();
        products += (tails.len() * heads.len()) as u64;
        if products > limit {
            return None;
        }
        for &i in &tails {
            onto[i].remove(&v);
            fill.remove(&(i, v));
        }
        for &j in &heads {
            into[j].remove(&v);
            fill.remove(&(v, j));
        }
        onto[v].clear();
        into[v].clear();
        for &i in &tails {
            for &j in &heads {
                if i != j {
                    onto[i].insert(j);
                    into[j].insert(i);
                    fill.insert((i, j));
                }
            }
        }
        let mut stale: FxHashSet<usize> = FxHashSet::default();
        for &u in tails.iter().chain(&heads) {
            stale.insert(u);
            stale.extend(into[u].iter().copied());
            stale.extend(onto[u].iter().copied());
        }
        for u in stale {
            if u < n && !gone[u] {
                heap.push(Reverse(score(u, &onto, &into, &fill)));
            }
        }
    }
    Some((order, products))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::{GroundOptions, ground};
    use crate::syntax::parse_program;
    use crate::{Kind, Options, Strategy};

    fn grounded(src: &str) -> GroundProgram {
        ground(
            &parse_program(src).expect("parses"),
            &GroundOptions::default(),
        )
        .expect("grounds")
    }

    fn answers(src: &str, kind: Kind) -> Vec<f64> {
        let run =
            crate::solve::run_source(src, &Options::with(Strategy::Trial(kind))).expect("runs");
        run.results
            .expect("consistent evidence")
            .iter()
            .map(|r| r.probability)
            .collect()
    }

    fn recursive(gp: &GroundProgram) -> bool {
        let by_head = gp.rules_by_head();
        deps::derived_sccs(gp, &by_head)
            .iter()
            .any(|c| deps::is_cyclic(gp, &by_head, c))
    }

    /// Paths from 1 over a cycle between 2 and 3, by hand: 1 reaches 2
    /// directly or through 3, `1 - (1 - 0.5)(1 - 0.2 · 0.3) = 0.53`, and 3
    /// directly or through 2, `1 - (1 - 0.2)(1 - 0.5 · 0.4) = 0.36`.
    #[test]
    fn a_cycle_written_without_recursion_answers_by_hand() {
        let src = "0.5::e(1,2). 0.4::e(2,3). 0.3::e(3,2). 0.2::e(1,3).
                   p(X,Y) :- e(X,Y). p(X,Y) :- e(X,Z), p(Z,Y).
                   query(p(1,2)). query(p(1,3)).";
        let mut gp = grounded(src);
        assert!(recursive(&gp));
        assert_eq!(linear_recursion(&mut gp), 1);
        assert!(!recursive(&gp));
        for (got, want) in answers(src, Kind::Closed).into_iter().zip([0.53, 0.36]) {
            assert!((got - want).abs() < 1e-12, "{got} against {want}");
        }
    }

    /// Programs whose recursion is linear, written without it under
    /// `closed`, answer as their recursive selves under `dynamic` and the
    /// families layouts: cycles through several atoms, a rule that passes
    /// truth on unconditionally, a step that negates an atom below the
    /// component, evidence on an atom of the component, a component read
    /// by a rule above it, and two components, one above the other.
    #[test]
    fn linear_recursion_written_without_it_answers_as_before() {
        let sources = [
            "0.6::edge(1,2). 0.5::edge(2,3). 0.7::edge(3,1). 0.4::edge(3,4). 0.8::edge(4,5).
             0.3::edge(5,3). 0.6::edge(5,6). 0.5::edge(6,4). 0.4::edge(2,6). 0.3::edge(6,1).
             path(X,Y) :- edge(X,Y). path(X,Y) :- edge(X,Z), path(Z,Y).
             query(path(1,4)). query(path(1,6)). query(path(2,1)).",
            "0.3::stress(X) :- person(X). 0.2::influences(X,Y) :- person(X), person(Y).
             0.4::calm(X) :- person(X).
             person(p1). person(p2). person(p3). person(p4).
             friend(p1,p2). friend(p2,p1). friend(p2,p3). friend(p3,p4). friend(p4,p2).
             smokes(X) :- stress(X), \\+calm(X).
             smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).
             smokes(X) :- friend(X,Y), smokes(Y), calm(Y).
             0.6::cough(X) :- smokes(X).
             evidence(smokes(p3), true).
             query(smokes(p1)). query(smokes(p4)). query(cough(p2)).",
            "0.5::a. 0.4::b. 0.3::c. 0.6::d. 0.7::f.
             x :- a. x :- y. y :- x, b. y :- z. z :- y, c. z :- d.
             u :- z, f. u :- v. v :- u. v :- x, \\+f.
             query(x). query(z). query(u). query(v).",
        ];
        for src in sources {
            let mut gp = grounded(src);
            assert!(linear_recursion(&mut gp) > 0, "{src}");
            assert!(!recursive(&gp), "{src}");
            // `families:0` compiles the recursive program.
            let recursive = answers(src, Kind::Families { search: false });
            for strategy in [Kind::Closed, Kind::Dynamic] {
                for (got, want) in answers(src, strategy).iter().zip(&recursive) {
                    assert!(
                        (got - want).abs() < 1e-9,
                        "{strategy}: {got} against {want} on {src}"
                    );
                }
            }
        }
    }

    /// A component that reads its own atoms twice in one rule is not linear,
    /// and stays as it is. (The chain rule alone the grounder writes linear
    /// first; an edge between the two reads keeps it from doing so.)
    #[test]
    fn a_doubly_recursive_component_is_left_alone() {
        let mut gp = grounded(
            "0.5::e(1,2). 0.4::e(2,3). 0.3::e(3,1).
             p(X,Y) :- e(X,Y). p(X,Y) :- p(X,Z), e(Z,W), p(W,Y).
             query(p(1,3)).",
        );
        let before = gp.to_string();
        assert_eq!(linear_recursion(&mut gp), 0);
        assert_eq!(gp.to_string(), before);
    }
}
