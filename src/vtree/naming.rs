//! The orders the path layouts read a program in, and the wide bodies they
//! are read through.
//!
//! `program` and `program:fanin` lay the facts out right-linear along an
//! order of all the atoms, facts and derived ([`positions`]). A run cut
//! into windows (`crate::solve`'s) reads the rules in an order of its own
//! ([`window_profile`]) and cuts it where the boundaries are narrow
//! ([`cut_search`]). The width of the boundary after a position is the
//! number of atoms at or before it that a rule of an atom after it reads,
//! facts among them.
//!
//! A body that reads more than [`WIDEST`] atoms crosses every boundary in
//! its span with everything it has read before it, so no cut there could
//! be narrow, and a layout that reads the program's graph takes it for a
//! clique; every layout's rewrite writes it as a chain along the order
//! first ([`chained`]).

use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::ground::deps::{Deps, depths, fan_in};
use crate::ground::tables::Tables;
use crate::ground::{AtomId, AtomKind, GLit, GroundProgram, GroundRule};
use crate::syntax::{Atom, Term};

/// The most atoms a body reads before it is chained ([`chained`]).
const WIDEST: usize = 20;

/// The name of a link of a chained body ([`chained`]).
const NAME: &str = "$cj";

/// `program`'s layout of `gp`: its facts in the file's order, or with
/// `fan_in` in the fan-in walk's ([`positions`]), the order a right-linear
/// vtree reads them last to first.
pub(super) fn lay_out(gp: &GroundProgram, fan_in: bool) -> Vec<AtomId> {
    let mut facts = positions(gp, fan_in);
    facts.retain(|&a| gp.is_fact(a));
    facts
}

/// `gp` with each body that reads more atoms than [`WIDEST`] written as a
/// chain along its order ([`positions`], the file's), counted in its stats
/// (`GroundStats::links`); `gp` itself where no body is that wide. Such a
/// body crosses every boundary of `program`'s order in its span with all
/// it has read before it, so no cut there can be narrow: a hidden Markov
/// model's joint query, the conjunction of its states, keeps the states of
/// every step alive to the end. As a chain, each link the conjunction of
/// the link before and the body's next literal in the order, it crosses
/// each boundary as one atom. The layouts that read the program's graph (`families`,
/// `closed`, `td`) take a wide body for a clique of its atoms: Roulette's
/// circuits of a hardware design conjoin the check of every step in one
/// body, and the families layout of 300 steps was 304 wide where the chain
/// is 15. Every layout's rewrite starts from the chained program
/// ([`crate::strategy::Kind::eliminated`]).
///
/// A body is read through its views first: a derived atom of one rule
/// that one body reads, positively, and that is neither queried nor
/// observed, is its own body there, so that a conjunction is chained along
/// the order however its gates associate it. Roulette's joint query
/// conjoins the states from the last step back, each gate reading the
/// state of its step and the gate after it, which puts every gate at the
/// end of the order and every state across the boundaries up to there.
/// The views a chained body reads are left out with their rules.
///
/// A link is named by the two literals it conjoins, so that bodies whose
/// chains start alike share the links of their common start. The paths of
/// a hidden Markov model whose atoms keep the history of its states (cplint's
/// DNA model) are a body each, every path conjoining the steps of the paths
/// it extends: chained apart, the model of 12 letters took 58368 links and
/// its families layout ran out of memory; shared, the links are 6596 and the
/// compile takes 1 s.
///
/// A switch that other rows of its table read too counts for nothing in a
/// body's width and stays out of its chain, in the rule of its last link:
/// the rows of an annotated disjunction read its switches together, each
/// row the switches before its own negated, and the families layout reads
/// them as one table. Chained, the rows of a variable of 63 values
/// (bn_pathfinder's fault) took 15 s under families:0 against 0.25 s as
/// written.
pub(crate) fn chained(gp: GroundProgram) -> GroundProgram {
    let n = gp.num_atoms();
    let deps = Deps::of(&gp);
    let reads = reads(&gp);
    // Every literal counted first: a program with no body that wide is not
    // asked for its tables.
    let (_, flat) = wide_bodies(&gp, &deps, &reads, &vec![false; n]);
    if flat.iter().all(Option::is_none) {
        return gp;
    }
    let shared = shared_switches(&gp, &reads);
    let (taken, flat) = wide_bodies(&gp, &deps, &reads, &shared);
    if flat.iter().all(Option::is_none) {
        return gp;
    }
    let order = positions(&gp, false);
    let mut at = vec![0usize; n];
    for (p, &a) in order.iter().enumerate() {
        at[a as usize] = p;
    }
    // Each link's rule goes where its literal is placed ([`positions`]):
    // after a derived literal's last rule, before a fact's first reader,
    // never after the body it is chained from. Keys of `3 * rule`, before,
    // `3 * rule + 1`, the rule itself, and `3 * rule + 2`, after, sorted
    // stably, keep the rules' own order and each chain's.
    let (first_read, last_rule) = rule_places(&gp, None);
    let mut out = gp.without_rules();
    let mut rules: Vec<(usize, GroundRule)> = Vec::with_capacity(gp.rules.len());
    let mut links = 0;
    // Each link by the two literals it conjoins, with the place of its rule:
    // bodies that share the start of their chains share its links.
    let mut made: FxHashMap<(GLit, GLit), (AtomId, usize)> = FxHashMap::default();
    for (ri, (r, body)) in gp.rules.iter().zip(flat).enumerate() {
        if taken[r.head as usize] {
            continue;
        }
        let key = 3 * ri + 1;
        let Some(body) = body else {
            rules.push((key, r.clone()));
            continue;
        };
        let (mut chain, kept): (Vec<GLit>, Vec<GLit>) =
            body.iter().partition(|l| !shared[l.atom() as usize]);
        // Literals read twice through views count once.
        if chain.len() <= WIDEST {
            rules.push((key, GroundRule::new(r.head, body)));
            continue;
        }
        chain.sort_by_key(|l| at[l.atom() as usize]);
        let (&last, inner) = chain.split_last().expect("a wide body");
        let mut link = inner[0];
        let mut placed = 0;
        for &l in &inner[1..] {
            if let Some(&(a, at)) = made.get(&(link, l)) {
                placed = at;
                link = GLit::Pos(a);
                continue;
            }
            let a = out.atoms.len() as AtomId;
            out.atoms.push(Atom::new(NAME, vec![Term::Int(links)]));
            out.kinds.push(AtomKind::Derived);
            links += 1;
            let i = l.atom() as usize;
            let at_l = match last_rule[i] {
                usize::MAX => 3 * first_read[i],
                lr => 3 * lr + 2,
            };
            placed = at_l.clamp(placed, key);
            rules.push((placed, GroundRule::new(a, vec![link, l])));
            made.insert((link, l), (a, placed));
            link = GLit::Pos(a);
        }
        let mut tail = vec![link, last];
        tail.extend(kept);
        rules.push((key, GroundRule::new(r.head, tail)));
    }
    rules.sort_by_key(|&(key, _)| key);
    out.rules = rules.into_iter().map(|(_, r)| r).collect();
    out.stats.links += links as usize;
    out
}

/// How many rules read each atom, `u32::MAX` for one a query or the
/// evidence names.
fn reads(gp: &GroundProgram) -> Vec<u32> {
    let mut reads = vec![0u32; gp.num_atoms()];
    for r in &gp.rules {
        for l in &r.body {
            reads[l.atom() as usize] += 1;
        }
    }
    for (_, a, _) in gp.queries.iter().chain(&gp.evidence) {
        if let Some(a) = a {
            reads[*a as usize] = u32::MAX;
        }
    }
    reads
}

/// The switches that more than one row of their table reads: the facts
/// that more than one rule reads, every one of them defining an atom of
/// one table ([`Tables::owner`]), an exclusive set or one atom.
fn shared_switches(gp: &GroundProgram, reads: &[u32]) -> Vec<bool> {
    let tables = Tables::of_structure(gp, Arc::unwrap_or_clone(gp.structure()));
    (0..gp.num_atoms())
        .map(|a| reads[a] > 1 && tables.owner(gp, a as AtomId).is_some())
        .collect()
}

/// The views the wide bodies take in, and each wide body read through its
/// views, where the `shared` switches count for nothing in a body's width
/// ([`chained`]).
fn wide_bodies(
    gp: &GroundProgram,
    deps: &Deps,
    reads: &[u32],
    shared: &[bool],
) -> (Vec<bool>, Vec<Option<Vec<GLit>>>) {
    let n = gp.num_atoms();
    let one_rule = |a: AtomId| match &deps.by_head[a as usize][..] {
        &[ri] if !gp.rules[ri].pairs => Some(ri),
        _ => None,
    };
    // An acyclic atom's rules read atoms of earlier components only.
    let mut view = vec![false; n];
    for scc in &deps.sccs {
        if let &[a] = scc.as_slice()
            && reads[a as usize] == 1
            && let Some(ri) = one_rule(a)
            && gp.rules[ri].body.iter().all(|l| l.atom() != a)
        {
            view[a as usize] = true;
        }
    }
    let is_view = |l: &GLit| l.is_positive() && view[l.atom() as usize];
    // The literals each atom of one rule stands for, read through views,
    // counted in dependency order.
    let mut width = vec![1usize; n];
    let weight = |width: &[usize], l: &GLit| {
        if is_view(l) {
            width[l.atom() as usize]
        } else {
            usize::from(!shared[l.atom() as usize])
        }
    };
    for scc in &deps.sccs {
        for &a in scc {
            if let Some(ri) = one_rule(a) {
                width[a as usize] = gp.rules[ri].body.iter().map(|l| weight(&width, l)).sum();
            }
        }
    }
    let wide = |r: &GroundRule| {
        !r.pairs && r.body.iter().map(|l| weight(&width, l)).sum::<usize>() > WIDEST
    };
    // Readers before the views they read: a view a wide body takes in is
    // not chained on its own.
    let mut taken = vec![false; n];
    let mut flat: Vec<Option<Vec<GLit>>> = vec![None; gp.rules.len()];
    for scc in deps.sccs.iter().rev() {
        for &h in scc {
            if taken[h as usize] {
                continue;
            }
            for &ri in &deps.by_head[h as usize] {
                if !wide(&gp.rules[ri]) {
                    continue;
                }
                let mut body = Vec::new();
                let mut stack = gp.rules[ri].body.clone();
                while let Some(l) = stack.pop() {
                    if is_view(&l) {
                        taken[l.atom() as usize] = true;
                        let v = one_rule(l.atom()).expect("a view has one rule");
                        stack.extend_from_slice(&gp.rules[v].body);
                    } else {
                        body.push(l);
                    }
                }
                body.sort_unstable();
                body.dedup();
                flat[ri] = Some(body);
            }
        }
    }
    (taken, flat)
}

/// Every atom of `gp`, fact or derived, in the trial's order. The facts
/// keep the order the program states them in, the variable order its
/// author or exporter chose, and each goes as late as that order allows:
/// before the first rule that reads it or a fact stated after it. A
/// derived atom goes at its last rule. Where a file states each fact right
/// before the gate that first reads it (Alea's), this is the file's
/// order; where it states every fact before every rule (Roulette's
/// circuits, Alea's exported ones), the gates are interleaved with the
/// facts as far as the facts' order allows, and the same circuit is laid
/// out alike however it is written. The facts after every rule's readers come
/// after the rules, and the derived atoms with no rule last. With `fan_in`,
/// the fan-in order, the deepest roots first ([`fan_in`]): each atom after
/// what it reads.
pub(super) fn positions(gp: &GroundProgram, fan_in_order: bool) -> Vec<AtomId> {
    let n = gp.num_atoms();
    if fan_in_order {
        let place = fan_in(gp, true);
        let mut order: Vec<AtomId> = (0..n as AtomId).collect();
        order.sort_unstable_by_key(|&a| place[a as usize]);
        return order;
    }
    in_rule_order(gp, None)
}

/// [`positions`] but for the fan-in order; or, for the windows a run is cut
/// into ([`window_profile`]), the rules read in the order of `ranked`,
/// their indices, and each fact right before its first reader, whatever
/// order the program states the facts in.
fn in_rule_order(gp: &GroundProgram, ranked: Option<&[u32]>) -> Vec<AtomId> {
    let n = gp.num_atoms();
    let (first_read, last_rule) = rule_places(gp, ranked);
    let mut facts: Vec<AtomId> = gp.facts().map(|(a, _)| a).collect();
    if ranked.is_some() {
        facts.sort_by_key(|&f| first_read[f as usize]);
    }
    // The rule each fact goes before: the first that reads it or a fact
    // after it.
    let mut slot = vec![usize::MAX; facts.len()];
    let mut next = usize::MAX;
    for (s, &f) in slot.iter_mut().zip(&facts).rev() {
        next = next.min(first_read[f as usize]);
        *s = next;
    }
    let mut order = Vec::with_capacity(n);
    let mut f = 0;
    for ri in 0..gp.rules.len() {
        let r = nth_rule(gp, ranked, ri);
        while f < facts.len() && slot[f] <= ri {
            order.push(facts[f]);
            f += 1;
        }
        if last_rule[r.head as usize] == ri {
            order.push(r.head);
        }
    }
    order.extend_from_slice(&facts[f..]);
    order.extend(
        (0..n as AtomId).filter(|&a| !gp.is_fact(a) && last_rule[a as usize] == usize::MAX),
    );
    order
}

/// Per atom, the first rule of `gp` that reads it and its own last rule,
/// `usize::MAX` for none: what [`positions`] places a fact before and a
/// derived atom at; the rules in the order of `ranked` where it is given
/// ([`in_rule_order`]).
fn rule_places(gp: &GroundProgram, ranked: Option<&[u32]>) -> (Vec<usize>, Vec<usize>) {
    let n = gp.num_atoms();
    let (mut first_read, mut last_rule) = (vec![usize::MAX; n], vec![usize::MAX; n]);
    for ri in 0..gp.rules.len() {
        let r = nth_rule(gp, ranked, ri);
        last_rule[r.head as usize] = ri;
        for l in &r.body {
            let f = &mut first_read[l.atom() as usize];
            if *f == usize::MAX {
                *f = ri;
            }
        }
    }
    (first_read, last_rule)
}

/// Rule `i` of `gp` in the order of `ranked`, or of the program.
fn nth_rule<'g>(gp: &'g GroundProgram, ranked: Option<&[u32]>, i: usize) -> &'g GroundRule {
    &gp.rules[ranked.map_or(i, |r| r[i] as usize)]
}

/// A boundary of an order that a cut may take: one after a derived atom,
/// crossed by a derived atom.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Boundary {
    /// The position it follows.
    pub(crate) after: usize,
    /// The atoms at or before it that a rule of an atom after it reads.
    pub(crate) width: usize,
    /// The rules of the atoms at or before it.
    pub(crate) rules: usize,
}

/// The order of `gp`'s atoms ([`positions`]) and its profile ([`profile`])
/// as a run cut into windows reads them (`crate::solve`'s windows): the
/// rules by the level of their heads ([`window_levels`]), then by depth
/// ([`depths`]), the program's order among equals, and a query's atom
/// crossing every boundary up to the last observed atom, whose window
/// answers it. The grounder's own order runs a loop it grounds from its
/// bound end backwards, each step's atoms placed at their last rule after
/// the steps that read them: no boundary there could be cut, and a ladder
/// of two atoms a step had none. The order, each atom's position, the
/// boundaries, and each atom's last reader's position.
pub(crate) fn window_profile(
    gp: &GroundProgram,
) -> (Vec<AtomId>, Vec<u32>, Vec<Boundary>, Vec<u32>) {
    let depth = depths(gp);
    let level = window_levels(gp, &depth);
    let mut ranked: Vec<u32> = (0..gp.rules.len() as u32).collect();
    ranked.sort_by_key(|&r| {
        let h = gp.rules[r as usize].head as usize;
        (level[h], depth[h])
    });
    let order = in_rule_order(gp, Some(&ranked));
    let (at, boundaries, last) = profile(gp, &order);
    (order, at, boundaries, last)
}

/// Each derived atom's level in the windows' order: one that reads no
/// derived atom at its late level ([`late_levels`]), as late as its readers
/// allow, and any other one level past the highest of the derived atoms it
/// reads, as early as they allow. By depth, which places every atom as
/// early as it may go, a helper of each step that reads facts alone, or an
/// observation's factor, ranks with the chain's first steps and crosses
/// every boundary up to its reader. By the late level alone, a chain that
/// reads another and takes several of its steps a rule (a joint query over
/// a run of states, its views inlined three steps to a rule) has fewer
/// levels than the chain it reads, so it ranks late, its first atoms with
/// the other chain's last steps, and each state it reads crosses every
/// boundary up to its reader. Taken in order of depth, a derived atom's
/// level is placed after those it reads; it is never past its late level,
/// whose atoms read lie at lower late levels.
fn window_levels(gp: &GroundProgram, depth: &[u32]) -> Vec<i64> {
    let late = late_levels(gp, depth);
    let by_head = gp.rules_by_head();
    let mut atoms: Vec<usize> = (0..gp.num_atoms())
        .filter(|&a| !gp.is_fact(a as AtomId))
        .collect();
    atoms.sort_by_key(|&a| depth[a]);
    let mut level = vec![i64::MAX; gp.num_atoms()];
    for a in atoms {
        level[a] = by_head[a]
            .iter()
            .flat_map(|&r| gp.rules[r].body.iter().map(|l| level[l.atom() as usize]))
            .filter(|&l| l != i64::MAX)
            .max()
            .map_or(late[a], |l| l + 1);
    }
    level
}

/// Each atom's late level: an atom no rule reads at its depth, any other
/// one level below its lowest reader's, so that each atom ranks as late as
/// its readers allow. Atoms are taken deepest first, so a reader is placed
/// before what it reads, but in a cycle.
fn late_levels(gp: &GroundProgram, depth: &[u32]) -> Vec<i64> {
    let readers = gp.readers(|r, _| r);
    let mut atoms: Vec<usize> = (0..gp.num_atoms()).collect();
    atoms.sort_by_key(|&a| std::cmp::Reverse(depth[a]));
    let mut level = vec![i64::MAX; gp.num_atoms()];
    for a in atoms {
        level[a] = readers[a]
            .iter()
            .map(|&r| level[gp.rules[r as usize].head as usize])
            .filter(|&l| l != i64::MAX)
            .min()
            .map_or(i64::from(depth[a]), |l| l - 1);
    }
    level
}

/// Each atom's position in `order`, the boundaries of `order` that a cut
/// may take, with their widths, and each atom's last reader's position:
/// the profile, computed once. Each query's atom is read at the last
/// observed atom's position at least ([`window_profile`]).
fn profile(gp: &GroundProgram, order: &[AtomId]) -> (Vec<u32>, Vec<Boundary>, Vec<u32>) {
    let n = order.len();
    let mut at = vec![0u32; gp.num_atoms()];
    for (p, &a) in order.iter().enumerate() {
        at[a as usize] = p as u32;
    }
    // Each atom's last reader's position, and the rules at each position.
    let mut last = vec![0u32; gp.num_atoms()];
    let mut heads = vec![0usize; n];
    for r in &gp.rules {
        let p = at[r.head as usize];
        heads[p as usize] += 1;
        for l in &r.body {
            let b = l.atom() as usize;
            last[b] = last[b].max(p);
        }
    }
    let observed = gp.evidence.iter().filter_map(|&(_, a, _)| a);
    if let Some(end) = observed.map(|a| at[a as usize]).max() {
        for &(_, q, _) in &gp.queries {
            if let Some(q) = q {
                last[q as usize] = last[q as usize].max(end);
            }
        }
    }
    // The atoms, and the derived atoms, that start and stop crossing at
    // each position.
    let mut change = vec![(0isize, 0isize); n + 1];
    for (b, &l) in last.iter().enumerate() {
        if l > at[b] {
            let d = isize::from(!gp.is_fact(b as AtomId));
            let (from, to) = (at[b] as usize, l as usize);
            change[from] = (change[from].0 + 1, change[from].1 + d);
            change[to] = (change[to].0 - 1, change[to].1 - d);
        }
    }
    let mut boundaries = Vec::new();
    let (mut width, mut derived, mut rules) = (0isize, 0isize, 0);
    for (p, &a) in order.iter().enumerate() {
        (width, derived) = (width + change[p].0, derived + change[p].1);
        rules += heads[p];
        if !gp.is_fact(a) && derived > 0 {
            boundaries.push(Boundary {
                after: p,
                width: width as usize,
                rules,
            });
        }
    }
    (at, boundaries, last)
}

/// The windows' cut search (`crate::solve`'s), over the rules: from the
/// last cut, a window opens at the first boundary `i` where `opens(i, m)`,
/// of `m` rules since the cut, gives the rules the window stays under, and
/// spans the boundaries up to it. The cut is its narrowest boundary that
/// `fits`, the earliest on a tie; a window with none is passed. Each cut as
/// its window's first boundary and itself.
pub(crate) fn cut_search(
    boundaries: &[Boundary],
    opens: impl Fn(usize, usize) -> Option<usize>,
    fits: impl Fn(usize) -> bool,
) -> Vec<(usize, usize)> {
    let mut cuts = Vec::new();
    // The rules at or before the last cut, and the next boundary.
    let (mut behind, mut i) = (0, 0);
    while i < boundaries.len() {
        let Some(under) = opens(i, boundaries[i].rules - behind) else {
            i += 1;
            continue;
        };
        let mut end = i;
        while end + 1 < boundaries.len() && boundaries[end + 1].rules - behind < under {
            end += 1;
        }
        let window = (i..=end).filter(|&j| fits(j));
        match window.min_by_key(|&j| (boundaries[j].width, j)) {
            Some(best) => {
                cuts.push((i, best));
                (behind, i) = (boundaries[best].rules, best + 1);
            }
            None => i = end + 1,
        }
    }
    cuts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::{GroundOptions, ground};
    use crate::syntax::parse_program;

    fn grounded(src: &str) -> GroundProgram {
        ground(
            &parse_program(src).expect("parses"),
            &GroundOptions::default(),
        )
        .expect("grounds")
    }

    /// Where a program states its facts.
    #[derive(Clone, Copy, Debug)]
    enum Facts {
        First,
        WithTheirStep,
        Last,
    }

    /// A chain of steps, each reading its own two facts and the step
    /// before: one atom crosses each step's boundary. Its facts stated as
    /// `facts` says.
    fn chain_written(steps: usize, facts: Facts) -> String {
        let fact = |i: usize| format!("0.3::a{i}. 0.6::b{i}.\n");
        let rules = |i: usize| format!("s{i} :- s{p}, a{i}. s{i} :- \\+s{p}, b{i}.\n", p = i - 1);
        let mut src = String::new();
        if !matches!(facts, Facts::Last) {
            src += "0.5::s0.\n";
        }
        if matches!(facts, Facts::First) {
            src.extend((1..steps).map(fact));
        }
        for i in 1..steps {
            if matches!(facts, Facts::WithTheirStep) {
                src += &fact(i);
            }
            src += &rules(i);
        }
        if matches!(facts, Facts::Last) {
            src += "0.5::s0.\n";
            src.extend((1..steps).map(fact));
        }
        src + &format!("query(s{}).", steps - 1)
    }

    #[test]
    fn a_chain_is_ordered_alike_wherever_its_facts_are_stated() {
        for facts in [Facts::First, Facts::WithTheirStep, Facts::Last] {
            let gp = grounded(&chain_written(4, facts));
            let names: Vec<String> = positions(&gp, false)
                .iter()
                .map(|&a| gp.atoms[a as usize].to_string())
                .collect();
            assert_eq!(
                names,
                ["s0", "a1", "b1", "s1", "a2", "b2", "s2", "a3", "b3", "s3"],
                "facts {facts:?}"
            );
        }
    }

    /// The facts keep the order the program states them in, each as late
    /// as its first reader and the facts stated after it allow.
    #[test]
    fn the_facts_keep_their_order_each_as_late_as_it_may_go() {
        let names = |steps: &[usize]| {
            let mut src = String::from("0.5::s0.\n");
            for i in steps {
                src += &format!("0.3::a{i}. 0.6::b{i}.\n");
            }
            for i in 1..4 {
                src += &format!("s{i} :- s{p}, a{i}. s{i} :- \\+s{p}, b{i}.\n", p = i - 1);
            }
            let gp = grounded(&(src + "query(s3)."));
            positions(&gp, false)
                .iter()
                .map(|&a| gp.atoms[a as usize].to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            names(&[3, 2, 1]),
            ["s0", "a3", "b3", "a2", "b2", "a1", "b1", "s1", "s2", "s3"]
        );
        assert_eq!(
            names(&[1, 3, 2]),
            ["s0", "a1", "b1", "s1", "a3", "b3", "a2", "b2", "s2", "s3"]
        );
    }

    /// How a joint over a chain's states is written.
    #[derive(Clone, Copy, Debug)]
    enum Joint {
        /// One body reading every state.
        Body,
        /// Gates from the first step on, each reading the gate before.
        Forward,
        /// Gates from the last step back, each reading the gate after
        /// (Roulette's).
        Backward,
    }

    /// `chain(steps)`, its facts stated first, with `joint` the
    /// conjunction of every state, written as `joint` says, and queried.
    fn joint_written(steps: usize, joint: Joint) -> String {
        let mut src = chain_written(steps, Facts::First);
        src.truncate(src.rfind("query(").expect("a query"));
        let last = steps - 1;
        match joint {
            Joint::Body => {
                let states: Vec<String> = (0..steps).map(|i| format!("s{i}")).collect();
                src += &format!("joint :- {}.\n", states.join(", "));
            }
            Joint::Forward => {
                src += "j0 :- s0.\n";
                for i in 1..last {
                    src += &format!("j{i} :- j{p}, s{i}.\n", p = i - 1);
                }
                src += &format!("joint :- j{p}, s{last}.\n", p = last - 1);
            }
            Joint::Backward => {
                src += &format!("j{last} :- s{last}.\n");
                for i in (1..last).rev() {
                    src += &format!("j{i} :- s{i}, j{n}.\n", n = i + 1);
                }
                src += "joint :- s0, j1.\n";
            }
        }
        src + "query(joint)."
    }

    /// A conjunction of more states than [`WIDEST`] is one chain of links,
    /// however its gates associate it: the gates are its views.
    #[test]
    fn a_joint_is_chained_alike_however_its_gates_associate_it() {
        for joint in [Joint::Body, Joint::Forward, Joint::Backward] {
            let gp = ground(
                &parse_program(&joint_written(30, joint)).expect("parses"),
                &GroundOptions {
                    views: false,
                    ..GroundOptions::default()
                },
            )
            .expect("grounds");
            assert_eq!(chained(gp).stats.links, 28, "{joint:?}");
        }
    }

    /// The rows of an annotated disjunction of 40 values read its switches
    /// together, the last row all of them: a table, left as written. A
    /// conjunction of 40 facts that nothing else reads is chained.
    #[test]
    fn the_rows_of_a_wide_disjunction_are_left_as_written() {
        let values: Vec<String> = (1..=40).map(|i| format!("0.025::x({i})")).collect();
        let queries: String = (1..=40).map(|i| format!("query(x({i})).\n")).collect();
        let gp = grounded(&format!("{}.\n{queries}", values.join("; ")));
        assert!(gp.rules.iter().any(|r| r.body.len() > WIDEST));
        let rules = gp.rules.clone();
        let gp = chained(gp);
        assert_eq!(gp.stats.links, 0);
        assert_eq!(gp.rules, rules);
        let facts: String = (1..=40).map(|i| format!("0.5::f{i}.\n")).collect();
        let reads: Vec<String> = (1..=40).map(|i| format!("f{i}")).collect();
        let gp = grounded(&format!("{facts}q :- {}.\nquery(q).\n", reads.join(", ")));
        assert_eq!(chained(gp).stats.links, 38);
    }

    /// Three conjunctions of the same 40 facts, each with a fact of its own
    /// stated after them, are one chain of 39 links that each reads at its
    /// end, not three.
    #[test]
    fn bodies_that_start_alike_share_their_links() {
        let mut src: String = (1..=40).map(|k| format!("0.9::f({k}).\n")).collect();
        let reads: Vec<String> = (1..=40).map(|k| format!("f({k})")).collect();
        for i in 1..=3 {
            src += &format!(
                "0.5::g({i}).\nq({i}) :- {}, g({i}).\nquery(q({i})).\n",
                reads.join(", ")
            );
        }
        let gp = chained(grounded(&src));
        assert_eq!(gp.stats.links, 39);
        let tails: Vec<&GroundRule> = gp.rules.iter().filter(|r| r.body.len() == 2).collect();
        assert_eq!(tails.len(), 39 + 3);
    }
}
