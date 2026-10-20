//! Queries answered in groups, each on the part of the program that it and
//! the evidence depend on.
//!
//! The relevant ground program is what the queries and the evidence
//! together depend on, and the compile builds the evidence and every
//! query's formula over all of it. A query's answer needs less: `P(q | e)`
//! reads only what `q` and the evidence depend on. In a Bayesian network
//! whose every variable is queried, P(x | e) needs the ancestors of x and
//! of the evidence; the whole network ties the parents of every
//! unobserved child together, which no query's own part does. The UAI
//! 2022 networks BN_12, BN_13 and BN_15 are such networks: their moral
//! graphs, observations taken out, have min-fill width 28, 32 and 31, and
//! the part of their widest query 21, 28 and 25.
//!
//! [`query_parts`] groups the queries. A query that another query or the
//! evidence depends on is answered with that query, on its part, or with
//! any where the evidence reads it; each other query, with the queries in
//! its strongly connected component, heads a group. A group joins a larger
//! one that reads every fact it reads: in a network, the values of a
//! variable go together. `auto`'s estimate of the program's `families:0`
//! layout is a first filter, the weight of a min-fill elimination of the
//! program and of each part ([`crate::vtree::min_fill_weight`]) a second
//! ([`SplitBounds`]); whether the parts pay is decided on `auto`'s
//! estimates of their layouts ([`super::split_check`]).

use crate::ground::{self, AtomId, AtomKind, GroundProgram, deps};
use crate::vtree::{log2_add, min_fill_weight};

/// When a program's queries are split ([`query_parts`],
/// [`super::split_check`]). Weights are log2 of a min-fill elimination's
/// (`vtree::min_fill_weight`), pairs those of `auto`'s layout estimates.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SplitBounds {
    /// The least weight of a program split. The default is the log2 of
    /// the default `min_pairs`: wherever a program weighs 32 or more, its
    /// weight is log2 of its `families:0` estimate plus 8 or more (bn_andes,
    /// bn_barley, the UAI networks BN_12, BN_13 and BN_15), so a program
    /// under it is not estimated at `min_pairs` either.
    pub(crate) min_weight: f64,
    /// How many times less, in log2, the parts must weigh together than
    /// the program.
    pub(crate) gain: f64,
    /// The least pairs the program's `families:0` layout is estimated to
    /// hold, past which its searched layout and its parts' are estimated:
    /// a program below compiles in seconds, and the searches would cost a
    /// share of that.
    pub(crate) min_pairs: f64,
    /// The pairs each part is charged besides its layout's: the fixed cost
    /// of a run, its layouts and first trials.
    pub(crate) part_pairs: f64,
    /// How many times fewer pairs the parts' searched `families` layouts,
    /// each charged `part_pairs`, must be estimated to hold together than
    /// the program's; zero or less leaves the estimates out.
    pub(crate) layout_gain: f64,
}

impl Default for SplitBounds {
    fn default() -> Self {
        SplitBounds {
            min_weight: 40.0,
            gain: 2.0,
            min_pairs: (1u64 << 40) as f64,
            part_pairs: (1u64 << 20) as f64,
            layout_gain: 4.0,
        }
    }
}

impl SplitBounds {
    /// Every program with two groups or more split.
    #[cfg(test)]
    pub(crate) const EVERY_GROUP: SplitBounds = SplitBounds {
        min_weight: f64::NEG_INFINITY,
        gain: f64::NEG_INFINITY,
        min_pairs: 0.0,
        part_pairs: 0.0,
        layout_gain: 0.0,
    };
}

/// The most groups weighed: a program whose queries head more is not
/// split.
const MAX_GROUPS: usize = 64;

/// A program's queries in groups, each with its part of the program.
pub(crate) struct Split {
    /// Each group's program: the program restricted to what the group's
    /// queries and the evidence depend on, its queries the group's.
    pub(crate) parts: Vec<GroundProgram>,
    /// Each group's queries as their places in the program's queries, in
    /// the order of its part's.
    pub(crate) queries: Vec<Vec<usize>>,
    /// The `--stats` line on the split.
    pub(crate) note: String,
}

/// The program's queries in groups over parts of it that weigh less
/// together than the program, heaviest first, or `None` where the weights
/// under `bounds` rule a split out: fewer than two groups, more than
/// `MAX_GROUPS`, a program that weighs too little or parts that weigh
/// too much, or where `worth` is false. A program read through
/// indicators, or with an observation of an atom that cannot hold, is not
/// split. `worth` is asked once the program has two groups, and before it
/// or any part is weighed: where it refuses the split, the weights would
/// cost more than it.
pub(crate) fn query_parts(
    gp: &GroundProgram,
    bounds: &SplitBounds,
    worth: impl FnOnce() -> bool,
) -> Option<Split> {
    if gp.queries.len() < 2
        || !gp.encoding.is_empty()
        || gp.evidence.iter().any(|(_, id, _)| id.is_none())
    {
        return None;
    }
    let (groups, reached) = query_groups(gp)?;
    // Largest first, and each joins the first before it that reads every
    // fact it reads: the facts are the layout's variables, and a join adds
    // none.
    let facts: Vec<Vec<bool>> = reached
        .iter()
        .map(|r| {
            r.iter()
                .zip(&gp.kinds)
                .map(|(&r, k)| r && matches!(k, AtomKind::Fact(_)))
                .collect()
        })
        .collect();
    let size = |g: usize| facts[g].iter().filter(|&&f| f).count();
    let mut order: Vec<usize> = (0..groups.len()).collect();
    order.sort_by_key(|&g| std::cmp::Reverse(size(g)));
    let mut joined: Vec<(Vec<usize>, usize)> = Vec::new();
    for g in order {
        let within = |m: &usize| facts[g].iter().zip(&facts[*m]).all(|(&a, &b)| !a || b);
        match joined.iter_mut().find(|(_, m)| within(m)) {
            Some((queries, _)) => queries.extend(&groups[g]),
            None => joined.push((groups[g].clone(), g)),
        }
    }
    if joined.len() < 2 || !worth() {
        return None;
    }
    let whole = min_fill_weight(gp);
    if whole < bounds.min_weight {
        return None;
    }
    let mut weighed: Vec<(Vec<usize>, GroundProgram, f64)> = joined
        .into_iter()
        .map(|(mut queries, _)| {
            queries.sort_unstable();
            let part = ground::restricted(gp, &queries);
            let weight = min_fill_weight(&part);
            (queries, part, weight)
        })
        .collect();
    weighed.sort_by(|a, b| b.2.total_cmp(&a.2));
    let parts_weight = weighed
        .iter()
        .fold(f64::NEG_INFINITY, |w, m| log2_add(w, m.2));
    if parts_weight > whole - bounds.gain {
        return None;
    }
    let note = format!(
        "split: {} queries in {} groups by what each depends on, weights (log2) {} against {whole:.1} for the program, {parts_weight:.1} together",
        gp.queries.len(),
        weighed.len(),
        weighed
            .iter()
            .map(|m| format!("{:.1}", m.2))
            .collect::<Vec<_>>()
            .join(", "),
    );
    let (queries, parts) = weighed.into_iter().map(|(q, p, _)| (q, p)).unzip();
    Some(Split {
        parts,
        queries,
        note,
    })
}

/// Queries in groups, as their places in `gp.queries`, and with each
/// group the atoms its head and the evidence depend on, by atom id.
type Groups = (Vec<Vec<usize>>, Vec<Vec<bool>>);

/// The queries in groups, as their places in `gp.queries`, each group
/// headed by the queries of one strongly connected component that neither
/// another query's component nor the evidence depends on, and holding the
/// queries its head depends on that no group before it holds; the queries
/// the evidence depends on, and those of atoms that cannot hold, go with
/// the first group. With each group, the atoms its head and the evidence
/// depend on. `None` with fewer than two groups or more than
/// [`MAX_GROUPS`].
fn query_groups(gp: &GroundProgram) -> Option<Groups> {
    let n = gp.num_atoms();
    let by_head = gp.rules_by_head();
    let mut component = vec![u32::MAX; n];
    for (c, scc) in deps::derived_sccs(gp, &by_head).iter().enumerate() {
        for &a in scc {
            component[a as usize] = c as u32;
        }
    }
    // Marks every atom reachable from `from` through rule bodies, `from`
    // included.
    let reach = |from: &mut Vec<AtomId>, marked: &mut [bool]| {
        while let Some(a) = from.pop() {
            if std::mem::replace(&mut marked[a as usize], true) {
                continue;
            }
            for &ri in &by_head[a as usize] {
                from.extend(
                    gp.rules[ri]
                        .body
                        .iter()
                        .map(|l| l.atom())
                        .filter(|&b| !marked[b as usize]),
                );
            }
        }
    };
    let mut evidence = vec![false; n];
    reach(
        &mut gp.evidence.iter().filter_map(|&(_, id, _)| id).collect(),
        &mut evidence,
    );
    // Below a query: reachable from it by a path that leaves its
    // component, which no path comes back into.
    let mut below = vec![false; n];
    let mut from = Vec::new();
    for &(_, id, _) in &gp.queries {
        let Some(q) = id else { continue };
        let c = component[q as usize];
        for &ri in &by_head[q as usize] {
            from.extend(
                gp.rules[ri]
                    .body
                    .iter()
                    .map(|l| l.atom())
                    .filter(|&b| c == u32::MAX || component[b as usize] != c),
            );
        }
    }
    reach(&mut from, &mut below);
    // The heads, by component (a fact's own atom), in query order.
    let mut heads: Vec<Vec<AtomId>> = Vec::new();
    let mut group_of_head = rustc_hash::FxHashMap::default();
    for &(_, id, _) in &gp.queries {
        let Some(q) = id else { continue };
        if below[q as usize] || evidence[q as usize] {
            continue;
        }
        let key = if component[q as usize] == u32::MAX {
            (1, q)
        } else {
            (0, component[q as usize])
        };
        let g = *group_of_head.entry(key).or_insert_with(|| {
            heads.push(Vec::new());
            heads.len() - 1
        });
        heads[g].push(q);
    }
    if heads.len() < 2 || heads.len() > MAX_GROUPS {
        return None;
    }
    let mut groups = vec![Vec::new(); heads.len()];
    let mut reached = Vec::with_capacity(heads.len());
    let mut placed = vec![false; gp.queries.len()];
    for (g, head) in heads.into_iter().enumerate() {
        let mut part = evidence.clone();
        reach(&mut head.clone(), &mut part);
        for (i, &(_, id, _)) in gp.queries.iter().enumerate() {
            if !placed[i]
                && id.is_some_and(|q| part[q as usize] && (g == 0 || !evidence[q as usize]))
            {
                placed[i] = true;
                groups[g].push(i);
            }
        }
        reached.push(part);
    }
    for (i, p) in placed.iter().enumerate() {
        if !p {
            groups[0].push(i);
        }
    }
    Some((groups, reached))
}
