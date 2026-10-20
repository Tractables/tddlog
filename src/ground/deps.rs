//! Analyses of the ground dependency graph: the strongly connected
//! components of the derived atoms, whether one is cyclic or negates one of
//! its own atoms, the merged components ([`components`]) that the
//! component order of `closed`, the exclusive sets and `auto` read, and the
//! depth-first fan-in walk ([`fan_in`]) that orders `program:fanin`'s
//! facts and `decision`'s spines. Every strongly connected component the
//! grounder finds, of predicates (`engine`'s strata), atoms or literals
//! (`inline`, the conditioning's equal facts), comes from one walk
//! ([`tarjan`]).

use rustc_hash::{FxHashMap, FxHashSet};

use super::program::Lists;
use super::{AtomId, AtomKind, GroundProgram, RulesByHead};
use crate::disjoint::{find, union};

/// A program's rules by head ([`GroundProgram::rules_by_head`]) and its
/// derived atoms' components in dependency order ([`derived_sccs`]), for
/// the passes that read both of one program that does not change between
/// them: the layouts of a rewrite and the compile of it, which each
/// grouped the rules and found the components on its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Deps {
    pub(crate) by_head: RulesByHead,
    pub(crate) sccs: Vec<Vec<AtomId>>,
}

impl Deps {
    pub(crate) fn of(gp: &GroundProgram) -> Self {
        let by_head = gp.rules_by_head();
        let sccs = derived_sccs(gp, &by_head);
        Deps { by_head, sccs }
    }
}

/// SCCs of derived atoms in an order where every component comes after the
/// components it depends on.
pub(crate) fn derived_sccs(gp: &GroundProgram, by_head: &RulesByHead) -> Vec<Vec<AtomId>> {
    let n = gp.num_atoms();
    // A byte per atom, rather than reading the kinds, sixteen bytes each,
    // at random once per body literal.
    let derived: Vec<bool> = gp.kinds.iter().map(|k| *k == AtomKind::Derived).collect();
    // The next successor of a node from a cursor over its rules and their
    // literals: the derived atoms its rules' bodies read, in order, with no
    // list of them allocated per node. A successor the
    // node has listed already, as `listed_by` tells, is passed over: a
    // table's rows each read a value of every parent, and an edge taken
    // again changes no index or low link. A descendant may list an atom in
    // between and the node list it again, which changes nothing either.
    let mut listed_by = vec![usize::MAX; n];
    let next = |a: usize, (rule, lit): &mut (usize, usize)| -> Option<usize> {
        let rules = &by_head[a];
        while *rule < rules.len() {
            let body = &gp.rules[rules[*rule]].body;
            while *lit < body.len() {
                let b = body[*lit].atom() as usize;
                *lit += 1;
                if derived[b] && listed_by[b] != a {
                    listed_by[b] = a;
                    return Some(b);
                }
            }
            *rule += 1;
            *lit = 0;
        }
        None
    };
    // At most a component per derived atom; each sorted, rather than in
    // the order the walk stacked it.
    let mut sccs: Vec<Vec<AtomId>> = Vec::with_capacity(derived.iter().filter(|&&d| d).count());
    tarjan(n, (0..n).filter(|&a| derived[a]), next, |comp| {
        let mut comp: Vec<AtomId> = comp.iter().map(|&w| w as AtomId).collect();
        comp.sort_unstable();
        sccs.push(comp);
    });
    sccs
}

/// The strongly connected components of a graph on the nodes `0..n`, by
/// Tarjan's algorithm without recursion: a depth-first walk from each of
/// `roots` in turn not yet met, which takes the successors of a node in
/// the order `next` hands them out, from a cursor the walk keeps for the
/// node (`(0, 0)` at first), until it gives `None`. `emit` is given each
/// component, its nodes as the walk stacked them, once the walk is done
/// with it: every component after the components it reaches.
pub(crate) fn tarjan(
    n: usize,
    roots: impl IntoIterator<Item = usize>,
    mut next: impl FnMut(usize, &mut (usize, usize)) -> Option<usize>,
    mut emit: impl FnMut(&[usize]),
) {
    let mut index = vec![u32::MAX; n];
    let mut low = vec![0u32; n];
    let mut on_stack = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut counter = 0u32;
    // The depth-first path: each node with its cursor.
    let mut call: Vec<(usize, (usize, usize))> = Vec::new();
    for root in roots {
        if index[root] != u32::MAX {
            continue;
        }
        call.push((root, (0, 0)));
        index[root] = counter;
        low[root] = counter;
        counter += 1;
        stack.push(root);
        on_stack[root] = true;
        while let Some((v, cursor)) = call.last_mut() {
            let v = *v;
            if let Some(w) = next(v, cursor) {
                if index[w] == u32::MAX {
                    index[w] = counter;
                    low[w] = counter;
                    counter += 1;
                    stack.push(w);
                    on_stack[w] = true;
                    call.push((w, (0, 0)));
                } else if on_stack[w] {
                    low[v] = low[v].min(index[w]);
                }
            } else {
                if low[v] == index[v] {
                    // The component is the stack from `v` up.
                    let at = stack
                        .iter()
                        .rposition(|&w| w == v)
                        .expect("a root is on the stack");
                    emit(&stack[at..]);
                    for &w in &stack[at..] {
                        on_stack[w] = false;
                    }
                    stack.truncate(at);
                }
                call.pop();
                if let Some(&(p, _)) = call.last() {
                    low[p] = low[p].min(low[v]);
                }
            }
        }
    }
}

/// The strongly connected components of the graph on `n` nodes with
/// `edges`, by node, numbered in the order [`tarjan`] completes them;
/// `usize::MAX` for a node on no edge.
pub(crate) fn strongly_connected(n: usize, edges: &[(usize, usize)]) -> Vec<usize> {
    let successors = Lists::new(n, || edges.iter().copied());
    let mut on_edge = vec![false; n];
    for &(u, v) in edges {
        on_edge[u] = true;
        on_edge[v] = true;
    }
    let mut component = vec![usize::MAX; n];
    let mut components = 0;
    tarjan(
        n,
        (0..n).filter(|&v| on_edge[v]),
        |v, (e, _)| {
            let w = *successors[v].get(*e)?;
            *e += 1;
            Some(w)
        },
        |comp| {
            for &w in comp {
                component[w] = components;
            }
            components += 1;
        },
    );
    component
}

/// A component is cyclic if it has more than one atom or a self-loop.
pub(crate) fn is_cyclic(gp: &GroundProgram, by_head: &RulesByHead, scc: &[AtomId]) -> bool {
    if scc.len() > 1 {
        return true;
    }
    let a = scc[0];
    by_head[a as usize]
        .iter()
        .any(|&ri| gp.rules[ri].body.iter().any(|l| l.atom() == a))
}

/// Does some rule of an atom in the component negate an atom of the component?
pub(crate) fn has_internal_negation(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    scc: &[AtomId],
) -> bool {
    if scc.len() == 1 && !is_cyclic(gp, by_head, scc) {
        return false;
    }
    let members: FxHashSet<AtomId> = scc.iter().copied().collect();
    scc.iter().any(|&a| {
        by_head[a as usize].iter().any(|&ri| {
            gp.rules[ri]
                .body
                .iter()
                .any(|l| !l.is_positive() && members.contains(&l.atom()))
        })
    })
}

/// The components the order search ranges over: the SCCs of the derived
/// atoms with the ones that belong together merged into one, in two rounds.
/// First,
/// components that share a fact: the values of a Bayesian network variable
/// read the same switch facts, and an order that separates them keeps
/// every switch live in between, which the proxy prices at a bit per
/// switch, so moving the values one at a time never pays. Then, until
/// nothing changes, components with the same dependencies and the same
/// consumers, taken over the merged components: the values of a
/// deterministic variable share no fact, but once their parents are merged
/// they depend on the same components and feed the same ones. The merged
/// components are put back in a dependency order (by earliest member); a
/// merge that closes a cycle returns the components as they came.
pub(crate) fn components(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    sccs: Vec<Vec<AtomId>>,
) -> Vec<Vec<AtomId>> {
    let (rank, units) = unit_ranks(gp, by_head, &sccs);
    if units == sccs.len() {
        return sccs;
    }
    let mut sorted: Vec<Vec<AtomId>> = vec![Vec::new(); units];
    for (scc, &unit) in sccs.iter().zip(&rank) {
        sorted[unit].extend_from_slice(scc);
    }
    sorted
}

/// The position of each SCC's merged component ([`components`]) in their
/// dependency order, and how many there are; each SCC's own position where
/// nothing merges or a merge closes a cycle. The lists of each component's
/// dependencies and consumers are runs of flat lists, not a list per
/// component and round: a program may have a hundred thousand components.
pub(crate) fn unit_ranks(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    sccs: &[Vec<AtomId>],
) -> (Vec<usize>, usize) {
    let k = sccs.len();
    let (mut parent, mut forward) = merged_by_facts(gp, by_head, sccs);
    merge_alike(&mut parent, &mut forward);
    ranked(&mut parent, forward).unwrap_or_else(|| ((0..k).collect(), k))
}

/// Union-find over `sccs`, the components, with the components that share
/// a fact merged, and the dependencies between the merged components as
/// edges `(from, to)` between their roots.
fn merged_by_facts(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    sccs: &[Vec<AtomId>],
) -> (Vec<usize>, Vec<(usize, usize)>) {
    let n = gp.num_atoms();
    let k = sccs.len();
    let mut scc_of = vec![usize::MAX; n];
    for (i, scc) in sccs.iter().enumerate() {
        for &a in scc {
            scc_of[a as usize] = i;
        }
    }
    // Union-find over components: the first component to read a fact owns
    // it, and later readers join the owner. And the dependencies between
    // components, sorted and distinct, end to end: component i's from
    // `dep_start[i]`. A component's dependencies over the current merge are
    // the roots of these. Each is listed once, the first time a literal of
    // the component reads it, as `listed` marks, not a dependency per
    // literal sorted and then made distinct. Both come from one pass over
    // the literals, a fact being in no component, not a pass each that
    // told facts from derived atoms by their kinds. A component joins
    // each owner once, as `joined` marks: the values of a table read the
    // switches of the values before them, row after row, and joining the
    // same owner again changes no set.
    let mut parent: Vec<usize> = (0..k).collect();
    let mut owner: Vec<usize> = vec![usize::MAX; n];
    let mut dep_start: Vec<usize> = Vec::with_capacity(k + 1);
    let mut dep_list: Vec<usize> = Vec::new();
    let mut listed = vec![usize::MAX; k];
    let mut joined = vec![usize::MAX; k];
    dep_start.push(0);
    for (i, scc) in sccs.iter().enumerate() {
        let start = dep_list.len();
        for &h in scc {
            for &ri in &by_head[h as usize] {
                for l in &gp.rules[ri].body {
                    let a = l.atom() as usize;
                    match scc_of[a] {
                        usize::MAX => match owner[a] {
                            usize::MAX => owner[a] = i,
                            j if joined[j] != i => {
                                joined[j] = i;
                                union(&mut parent, i, j);
                            }
                            _ => {}
                        },
                        j if j != i && listed[j] != i => {
                            listed[j] = i;
                            dep_list.push(j);
                        }
                        _ => {}
                    }
                }
            }
        }
        dep_list[start..].sort_unstable();
        dep_start.push(dep_list.len());
    }
    // The edges are listed once per run of components with one root, as
    // `seen` marks each end with the root that listed it last: the values
    // of a table, merged by the switches they share, come one after another
    // and read the same parents, whose values are merged too, and an edge
    // per value and parent's value would be listed otherwise.
    let mut forward: Vec<(usize, usize)> = Vec::new();
    let mut seen = vec![usize::MAX; k];
    for i in 0..k {
        let ri = find(&mut parent, i);
        for &d in &dep_list[dep_start[i]..dep_start[i + 1]] {
            let rj = find(&mut parent, d);
            if ri != rj && seen[rj] != ri {
                seen[rj] = ri;
                forward.push((ri, rj));
            }
        }
    }
    (parent, forward)
}

/// The merged components of `parent` merged further, until nothing
/// changes, where they have the same dependencies and the same consumers
/// over the edges `forward`, which are left between the roots, sorted
/// and distinct.
fn merge_alike(parent: &mut [usize], forward: &mut Vec<(usize, usize)>) {
    let k = parent.len();
    // Per round, the edges between the merged components as (from, to),
    // sorted and distinct, and a root's dependencies and consumers, in
    // order, as lists of them ([`Lists`]), which key the groups. The first
    // round's edges are the components' dependencies over the merge by
    // facts; each later round's are the round before's with their ends
    // taken to their roots, since a merge only joins roots.
    loop {
        forward.sort_unstable();
        forward.dedup();
        let dependencies = Lists::new(k, || forward.iter().copied());
        let consumers = Lists::new(k, || forward.iter().map(|&(x, y)| (y, x)));
        // A merge changes the root of the component at hand only, so the
        // roots are those at the start of the round.
        let roots: Vec<usize> = (0..k).filter(|&i| find(parent, i) == i).collect();
        let mut group_of: FxHashMap<(&[usize], &[usize]), usize> =
            FxHashMap::with_capacity_and_hasher(roots.len(), Default::default());
        let mut changed = false;
        for &i in &roots {
            match group_of.entry((&dependencies[i], &consumers[i])) {
                std::collections::hash_map::Entry::Occupied(e) => {
                    changed |= union(parent, i, *e.get())
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(i);
                }
            }
        }
        if !changed {
            break;
        }
        let mut kept = 0;
        for e in 0..forward.len() {
            let (x, y) = forward[e];
            let (rx, ry) = (find(parent, x), find(parent, y));
            if rx != ry {
                forward[kept] = (rx, ry);
                kept += 1;
            }
        }
        forward.truncate(kept);
    }
}

/// The position of each component's merged one of `parent` in a
/// dependency order of the edges `forward` between the roots, sorted and
/// distinct, and how many there are; `None` where nothing merged or the
/// merged components have a cycle.
fn ranked(parent: &mut [usize], mut forward: Vec<(usize, usize)>) -> Option<(Vec<usize>, usize)> {
    let k = parent.len();
    // The merged components numbered by their earliest members (a root is
    // its set's smallest).
    let mut unit_of = vec![usize::MAX; k];
    let mut units = 0;
    for i in 0..k {
        let r = find(parent, i);
        if unit_of[r] == usize::MAX {
            unit_of[r] = units;
            units += 1;
        }
        unit_of[i] = unit_of[r];
    }
    if units == k {
        return None;
    }
    // Dependencies between them, the last round's edges, which no merge
    // changed, with each root numbered: the roots are numbered in their
    // order, so the edges stay sorted and distinct. Then a topological
    // order that takes the earliest ready one first.
    for e in &mut forward {
        *e = (unit_of[e.0], unit_of[e.1]);
    }
    let mut indegree = vec![0usize; units];
    for &(u, _) in &forward {
        indegree[u] += 1;
    }
    let consumers = Lists::new(units, || forward.iter().map(|&(u, v)| (v, u)));
    let mut ready: std::collections::BinaryHeap<std::cmp::Reverse<usize>> = (0..units)
        .filter(|&u| indegree[u] == 0)
        .map(std::cmp::Reverse)
        .collect();
    let mut rank = vec![usize::MAX; units];
    let mut placed = 0;
    while let Some(std::cmp::Reverse(u)) = ready.pop() {
        rank[u] = placed;
        placed += 1;
        for &c in &consumers[u] {
            indegree[c] -= 1;
            if indegree[c] == 0 {
                ready.push(std::cmp::Reverse(c));
            }
        }
    }
    if placed < units {
        return None;
    }
    Some((unit_of.iter().map(|&u| rank[u]).collect(), units))
}

/// Per atom of `gp`, its place in the fan-in order of BDD variable
/// ordering (Malik, Wang, Brayton and Sangiovanni-Vincentelli, ICCAD
/// 1988), interleaved over the roots as Fujii, Ootomo and Hori interleave
/// it (ICCAD 1993). A root is an observed or queried atom. Each root's
/// cone is walked depth first, entering the atoms each atom's rules read
/// in the order of its rules and their bodies (a ground body lists its
/// atoms in the order the grounder made them), and an atom is placed
/// once the atoms it reads are: a fact as the walk meets it. So a
/// ripple-carry adder's or a comparator's operand bits come out side by
/// side. An atom an earlier root's walk placed is not entered again but
/// marks where the walk is: the next atom placed goes right after it. So
/// an operand that several operations read sits beside each of the
/// others at every bit: conjugate_gaussians adds μ to two noises, and the
/// three numbers' bits come out interleaved, where one walk on from the
/// first sum put the second noise after both (conjugate_gaussians_w21
/// compiles in 78 M units of work under `program:fanin`, against 7.2 G
/// for `program`, which won it before). The atoms no walk reaches come
/// after, in program order.
///
/// The roots are walked by the longest chain of rules below each, ties
/// in the order of the evidence and then the queries: `deepest` first
/// for `program:fanin` (its facts and its naming positions), shallowest
/// first for `decision`'s spines and the facts it leaves out. Deepest
/// first is what interleaves conjugate_gaussians; shallowest first it
/// runs out of memory. Shallowest first keeps the facts each of
/// altermu2's 40 data points reads together on decision's spine, where
/// deepest first split them over it (the query phase of altermu2_w11
/// under `decision` 1.5 G units of work against 247 M).
pub(crate) fn fan_in(gp: &GroundProgram, deepest: bool) -> Vec<u32> {
    const NONE: u32 = u32::MAX;
    let n = gp.num_atoms();
    let by_head = gp.rules_by_head();
    let reads = |x: usize| {
        by_head[x]
            .iter()
            .flat_map(|&r| gp.rules[r].body.iter().map(|l| l.atom() as usize))
    };
    let mut roots: Vec<usize> = gp
        .evidence
        .iter()
        .chain(&gp.queries)
        .filter_map(|e| e.1)
        .map(|a| a as usize)
        .collect();
    // The longest chain of rules below each atom, a cycle cut where the
    // walk meets an atom it is in.
    let mut depth = vec![NONE; n];
    let mut walk: Vec<(usize, bool)> = Vec::new();
    for &r in &roots {
        walk.push((r, false));
        while let Some((x, done)) = walk.pop() {
            if done {
                depth[x] = reads(x)
                    .filter(|&y| depth[y] < NONE - 1)
                    .map(|y| depth[y] + 1)
                    .max()
                    .unwrap_or(0);
                continue;
            }
            if depth[x] != NONE {
                continue;
            }
            depth[x] = NONE - 1;
            walk.push((x, true));
            walk.extend(reads(x).filter(|&y| depth[y] == NONE).map(|y| (y, false)));
        }
    }
    if deepest {
        roots.sort_by_key(|&r| std::cmp::Reverse(depth[r]));
    } else {
        roots.sort_by_key(|&r| depth[r]);
    }
    // The order so far, a list linked through `after`, from `first`.
    let mut after = vec![NONE; n];
    let mut first = NONE;
    // The atoms in the order the walks placed them, and each one's index
    // there.
    let mut finished: Vec<usize> = Vec::new();
    let mut done = vec![NONE; n];
    // The walk an atom was last entered in.
    let mut entered = vec![NONE; n];
    for (w, &r) in roots.iter().enumerate() {
        let w = w as u32;
        // The atom the next one placed goes after, none for the front.
        let mut at = NONE;
        walk.push((r, false));
        while let Some((x, leaving)) = walk.pop() {
            if leaving {
                if at == NONE {
                    after[x] = first;
                    first = x as u32;
                } else {
                    after[x] = after[at as usize];
                    after[at as usize] = x as u32;
                }
                done[x] = finished.len() as u32;
                finished.push(x);
                at = x as u32;
                continue;
            }
            if entered[x] == w {
                continue;
            }
            entered[x] = w;
            if done[x] != NONE {
                at = x as u32;
                continue;
            }
            walk.push((x, true));
            // The first atom read is entered first.
            let from = walk.len();
            walk.extend(reads(x).filter(|&y| entered[y] != w).map(|y| (y, false)));
            walk[from..].reverse();
        }
    }
    // The list's order, but for the atoms it puts before an atom they
    // read: an atom that reads one this walk placed after the anchor, and
    // then one an earlier walk placed before it, went right after the
    // second, before the first. Each such atom moves to right after the
    // last atom it reads (a read of a cycle it is in aside), since
    // `program:fanin`'s naming takes every atom to come after what it
    // reads (nodice_coupon2-12-obs was answered 0.182102 for 0.183663).
    let mut key = vec![NONE; n];
    let mut next = 0;
    let mut x = first;
    while x != NONE {
        key[x as usize] = next;
        next += 1;
        x = after[x as usize];
    }
    for &x in &finished {
        if let Some(k) = reads(x)
            .filter(|&y| done[y] < done[x])
            .map(|y| key[y])
            .max()
        {
            key[x] = key[x].max(k);
        }
    }
    finished.sort_by_key(|&x| (key[x], done[x]));
    let mut place = vec![NONE; n];
    for (i, &x) in finished.iter().enumerate() {
        place[x] = i as u32;
    }
    let unplaced = place.iter_mut().filter(|m| **m == NONE);
    for (m, next) in unplaced.zip(finished.len() as u32..) {
        *m = next;
    }
    place
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn program(src: &str) -> GroundProgram {
        let p = crate::syntax::parse_program(src).unwrap();
        crate::ground::ground(&p, &crate::ground::GroundOptions::default()).unwrap()
    }

    pub(crate) fn is_dependency_order(gp: &GroundProgram, order: &[Vec<AtomId>]) -> bool {
        let by_head = gp.rules_by_head();
        let mut rank = vec![usize::MAX; gp.num_atoms()];
        for (i, unit) in order.iter().enumerate() {
            for &a in unit {
                rank[a as usize] = i;
            }
        }
        order.iter().enumerate().all(|(i, unit)| {
            unit.iter().all(|&h| {
                by_head[h as usize].iter().all(|&ri| {
                    gp.rules[ri]
                        .body
                        .iter()
                        .filter(|l| gp.kinds[l.atom() as usize] == AtomKind::Derived)
                        .all(|l| rank[l.atom() as usize] <= i)
                })
            })
        })
    }

    #[test]
    fn values_sharing_a_switch_merge_into_one_component() {
        // The values of the annotated disjunction a share its switch facts
        // and become one component; b(2) is irrelevant to the query and
        // grounds away, so b(1) and q stay alone.
        let gp = program(
            "0.2::a(1); 0.3::a(2); 0.5::a(3).
             0.1::b(1); 0.9::b(2) :- a(1). 0.4::b(1); 0.6::b(2) :- a(2). b(2) :- a(3).
             q :- b(1), \\+ a(3). query(q).",
        );
        let by_head = gp.rules_by_head();
        let sccs = derived_sccs(&gp, &by_head);
        assert_eq!(sccs.len(), 5);
        let units = components(&gp, &by_head, sccs);
        assert!(is_dependency_order(&gp, &units));
        let mut sizes: Vec<usize> = units.iter().map(|u| u.len()).collect();
        sizes.sort_unstable();
        assert_eq!(sizes, vec![1, 1, 3]);
    }

    /// Two ripple-carry sums of three bits that share the operand `a`,
    /// `s = a + b` and `t = a + c`: the fan-in order puts the three
    /// operands' bits side by side, bit by bit from the top, the roots
    /// deepest or shallowest first, where one walk on from `s` put `c`'s
    /// bits after `a`'s and `b`'s.
    #[test]
    fn the_fan_in_order_interleaves_an_operand_two_sums_read() {
        let mut src = String::from(
            "0.5::a0. 0.5::a1. 0.5::a2. 0.5::b0. 0.5::b1. 0.5::b2.
             0.5::c0. 0.5::c1. 0.5::c2.\n",
        );
        for (s, k, y) in [("s", "k", "b"), ("t", "j", "c")] {
            src += &format!("{s}0 :- a0, \\+{y}0. {s}0 :- \\+a0, {y}0. {k}0 :- a0, {y}0.\n");
            for i in 1..3 {
                let p = i - 1;
                src += &format!(
                    "{s}{i} :- a{i}, \\+{y}{i}, \\+{k}{p}. {s}{i} :- \\+a{i}, {y}{i}, \\+{k}{p}.
                     {s}{i} :- \\+a{i}, \\+{y}{i}, {k}{p}. {s}{i} :- a{i}, {y}{i}, {k}{p}.
                     {k}{i} :- a{i}, {y}{i}. {k}{i} :- a{i}, {k}{p}. {k}{i} :- {y}{i}, {k}{p}.\n"
                );
            }
        }
        src += "query(s2). query(t2).";
        let gp = program(&src);
        for deepest in [true, false] {
            let place = fan_in(&gp, deepest);
            let mut facts: Vec<_> = gp.facts().map(|(a, _)| a).collect();
            facts.sort_by_key(|&a| place[a as usize]);
            let names: Vec<String> = facts
                .iter()
                .map(|&a| gp.atoms[a as usize].to_string())
                .collect();
            assert_eq!(names.len(), 9, "{names:?}");
            for (chunk, bit) in names.chunks(3).zip(["2", "1", "0"]) {
                assert!(
                    chunk.iter().all(|n| n.ends_with(bit)),
                    "deepest {deepest}: {names:?}"
                );
            }
        }
    }

    /// The second root's walk enters `w`, which the first placed late,
    /// so `d` and then `z` go after it, and then `y`, which the first
    /// placed early, and `b`: `x`, which reads `z` and `y`, would go after
    /// `b`, before `z`. Every atom comes after the atoms it reads.
    #[test]
    fn the_fan_in_order_puts_each_atom_after_what_it_reads() {
        let gp = program(
            "0.5::a. 0.5::b. 0.5::c. 0.5::d. 0.5::e.
             y :- a. y :- b. w :- c, y. w :- e. r1 :- y, w. r1 :- \\+y, c.
             z :- d. z :- \\+e. x :- z, y. x :- \\+z, b.
             r2 :- w, x. r2 :- \\+w, e. query(r1). query(r2).",
        );
        for deepest in [true, false] {
            let place = fan_in(&gp, deepest);
            for r in &gp.rules {
                for l in &r.body {
                    assert!(
                        place[l.atom() as usize] < place[r.head as usize],
                        "deepest {deepest}: {} before {}",
                        gp.atoms[r.head as usize],
                        gp.atoms[l.atom() as usize]
                    );
                }
            }
        }
    }

    #[test]
    fn a_merge_that_closes_a_cycle_is_refused() {
        // x and z share the fact f, but z depends on y which depends on
        // x: merging them would leave y no position, so nothing merges.
        let gp = program("0.5::f. 0.5::g. x :- f. y :- x, g. z :- y, f. query(z).");
        let by_head = gp.rules_by_head();
        let sccs = derived_sccs(&gp, &by_head);
        let units = components(&gp, &by_head, sccs.clone());
        assert_eq!(units, sccs);
    }
}
