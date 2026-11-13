//! Naming the atoms that cross a narrow cut of `program`'s order.
//!
//! The Tp compile expands every derived atom: its formula is over the facts
//! of its cone. Where a program is a long chain of narrow steps (a hidden
//! Markov model, a circuit unrolled over time), each step's formulas then
//! hold the whole history before it, and the compile's work grows with the
//! square of the length or worse. `program` and `program:fanin` lay the
//! facts out right-linear along an order of all the atoms, facts and
//! derived ([`positions`]). Where a boundary of that order is narrow, the
//! derived atoms that cross it are named: a reader after the cut reads them
//! through indicator copies ([`indicators::name`]), with the segments
//! between the cuts as the units, and each copy is laid out at the start of
//! its segment. A formula then holds its own segment's facts and the copies
//! at its start, and the evidence holds the links, the history, once.
//!
//! The width of the boundary after a position is the number of atoms at or
//! before it that a rule of an atom after it reads, facts among them. The
//! queries and the evidence do not count: an observed atom's formula is
//! over its own segment and the copies at its start, and the evidence ties
//! the segments through the links.
//!
//! The cuts ([`cuts`]) take boundaries after a derived atom that a derived
//! atom crosses: from the last cut, a window opens at the first whose
//! width is at most twice the facts since the cut, and runs until those
//! facts double; the cut is the narrowest boundary in the window, the
//! earliest on a tie. A cut pays where its names are few against the facts
//! behind it, which its readers' formulas would otherwise hold, and the
//! narrowest place near the first that pays is the day's frontier on a
//! Markov chain (GeNI's weather: the 11 state bits and the goal, of 42
//! facts a day); cut at the first place that paid, the weather named 21
//! atoms a day and its evidence grew 150 times. A window whose narrowest
//! boundary is wider than [`WIDEST`] has no cut, and the segment runs on.
//! Where no boundary is narrow, nothing is named and the layout is the
//! order's.
//!
//! Two comparisons of a cut's names with what they hide refine the window.
//! A boundary the window skips on its way to a narrower one is cut too
//! where its names are no more than the facts on each side of it. Without
//! it a window that reaches the end of a program runs its last two days of
//! the weather together, the readers holding both days' facts. And a cut
//! stands only where its names hide something ([`hiding`]): a fact or an
//! atom behind the cut before that no later rule reads.
//!
//! A body that reads more than [`WIDEST`] atoms crosses every boundary in
//! its span with everything it has read before it, so no cut there could
//! be narrow; every layout's rewrite writes it as a chain along the order
//! first ([`chained`]).
//!
//! Whether a cut's names pay is settled on the compiled program
//! ([`crate::tp::settle`]): where the joint of a cut's links is larger than
//! what the formulas reading its copies would hold with them expanded, the
//! copies are composed away and the cut's links leave the evidence.

use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::ground::deps::{Deps, depths, fan_in};
use crate::ground::tables::Tables;
use crate::ground::{AtomId, AtomKind, GLit, GroundProgram, GroundRule};
use crate::indicators;
use crate::syntax::{Atom, Term};

/// The widest cut named. The evidence holds the joint of a cut's names,
/// up to `2^width` pairs a level there, where the expanded formulas each
/// hold their own atom's: GeNI's weather under the fan-in walk's order has
/// a 46 wide cut, and naming it ran out of 13 GB. 2^20 pairs is the first
/// watch `auto` puts on a trial's diagrams. The widest cuts that helped
/// were 17 (link grids at side 16) and 14 (weather at 12 factories).
const WIDEST: usize = 20;

/// The name of a link of a chained body ([`chained`]).
const NAME: &str = "$cj";

/// `program`'s layout of `gp`: its facts in the order a right-linear vtree
/// reads them last to first, and where the order has narrow cuts, the
/// program with the atoms crossing them named, over its own facts and
/// copies in that order, and a note for `--stats`.
pub(super) struct Laid {
    pub(super) facts: Vec<AtomId>,
    pub(super) named: Option<(GroundProgram, String)>,
}

/// Lays `gp` out along the file's order, or with `fan_in` along the fan-in
/// walk's ([`positions`]), naming the atoms that cross its narrow cuts.
/// `Err` where the allocator refuses the named program room
/// ([`super::refused`]).
pub(super) fn lay_out(gp: &GroundProgram, deps: &Deps, fan_in: bool) -> Result<Laid, String> {
    let order = positions(gp, fan_in);
    let facts: Vec<AtomId> = order.iter().copied().filter(|&a| gp.is_fact(a)).collect();
    // A program that reads atoms through indicators already is compiled as
    // it is (`crate::indicators`, `Prepared::lay_out`).
    if !gp.encoding.is_empty() {
        return Ok(Laid { facts, named: None });
    }
    let (at, boundaries, last) = profile(gp, &order, false);
    let cuts = hiding(gp, deps, &at, &last, &cuts(&boundaries));
    if cuts.is_empty() {
        return Ok(Laid { facts, named: None });
    }
    // Each derived atom's unit is its segment, and a cyclic component's
    // atoms share their first's, since the link is between components.
    let mut segment = vec![0usize; order.len()];
    let mut s = 0;
    for (p, seg) in segment.iter_mut().enumerate() {
        *seg = s;
        if cuts.get(s).is_some_and(|c| c.after == p) {
            s += 1;
        }
    }
    let mut unit_of: Vec<usize> = (0..gp.num_atoms())
        .map(|a| {
            if gp.is_fact(a as AtomId) {
                usize::MAX
            } else {
                segment[at[a] as usize]
            }
        })
        .collect();
    for scc in deps.sccs.iter().filter(|scc| scc.len() > 1) {
        let u = unit_of[scc[0] as usize];
        for &a in scc {
            unit_of[a as usize] = u;
        }
    }
    let (mut named, units) = indicators::name(gp, unit_of).map_err(super::refused)?;
    if named.encoding.is_empty() {
        return Ok(Laid { facts, named: None });
    }
    // The copies at the start of their segments, each segment's in the
    // order of the atoms they stand for.
    let mut copies: Vec<(usize, u32, AtomId)> = named
        .encoding
        .indicators
        .iter()
        .zip(&units)
        .map(|(ind, &u)| {
            (
                u,
                at.get(ind.atom as usize).copied().unwrap_or(u32::MAX),
                ind.var,
            )
        })
        .collect();
    copies.sort_unstable();
    let mut laid = Vec::with_capacity(facts.len() + copies.len());
    let mut next = copies.iter().peekable();
    for &a in &order {
        let seg = segment[at[a as usize] as usize];
        while let Some(&&(u, _, var)) = next.peek().filter(|c| c.0 <= seg) {
            debug_assert!(u <= seg);
            laid.push(var);
            next.next();
        }
        if gp.is_fact(a) {
            laid.push(a);
        }
    }
    laid.extend(next.map(|c| c.2));
    named.encoding.segments = units;
    let atoms: FxHashSet<AtomId> = named.encoding.indicators.iter().map(|i| i.atom).collect();
    let note = format!(
        "naming: {} cuts, widest {}, {} copies of {} atoms",
        cuts.len(),
        cuts.iter().map(|c| c.width).max().unwrap_or(0),
        named.encoding.indicators.len(),
        atoms.len()
    );
    Ok(Laid {
        facts: laid,
        named: Some((named, note)),
    })
}

/// `gp` with each body that reads more atoms than [`WIDEST`] written as a
/// chain along its order ([`positions`], the file's), counted in its stats
/// (`GroundStats::links`); `gp` itself where no body is that wide. Such a
/// body crosses every boundary of `program`'s order in its span with all
/// it has read before it, so no cut there can be narrow: a hidden Markov
/// model's joint query, the conjunction of its states, keeps the states of
/// every step alive to the end. As a chain, each link the conjunction of
/// the link before and the body's next literal in the order, it crosses
/// each boundary as one atom, and the cuts name the link and the step's
/// state there. The layouts that read the program's graph (`families`,
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
/// facts as far as the facts' order allows, and the same circuit is cut
/// alike however it is written. The facts after every rule's readers come
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
    /// The facts at or before it.
    facts: usize,
}

/// The order of `gp`'s atoms ([`positions`]) and its profile ([`profile`])
/// as a run cut into windows reads them (`crate::solve`'s windows): the
/// rules by the late level of their heads ([`late_levels`]), then by
/// depth ([`depths`]), the program's order among equals, and a query's atom
/// crossing every boundary up to the last observed atom, whose window
/// answers it. The grounder's own order runs a loop it grounds from its
/// bound end backwards, each step's atoms placed at their last rule after
/// the steps that read them: no boundary there could be cut, and a ladder
/// of two atoms a step had none. By depth alone, a helper of each step
/// that the next step reads, or an observation stated after the chain,
/// ranks with the chain's first steps and crosses every boundary up to its
/// reader. The order, each atom's position, the boundaries, and each atom's
/// last reader's position.
pub(crate) fn window_profile(
    gp: &GroundProgram,
) -> (Vec<AtomId>, Vec<u32>, Vec<Boundary>, Vec<u32>) {
    let depth = depths(gp);
    let level = late_levels(gp, &depth);
    let mut ranked: Vec<u32> = (0..gp.rules.len() as u32).collect();
    ranked.sort_by_key(|&r| {
        let h = gp.rules[r as usize].head as usize;
        (level[h], depth[h])
    });
    let order = in_rule_order(gp, Some(&ranked));
    let (at, boundaries, last) = profile(gp, &order, true);
    (order, at, boundaries, last)
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
/// the profile, computed once. With `held`, each query's atom is read at
/// the last observed atom's position at least ([`window_profile`]).
fn profile(
    gp: &GroundProgram,
    order: &[AtomId],
    held: bool,
) -> (Vec<u32>, Vec<Boundary>, Vec<u32>) {
    let n = order.len();
    let mut at = vec![0u32; gp.num_atoms()];
    for (p, &a) in order.iter().enumerate() {
        at[a as usize] = p as u32;
    }
    // Each atom's last reader's position.
    let mut last = vec![0u32; gp.num_atoms()];
    for r in &gp.rules {
        let p = at[r.head as usize];
        for l in &r.body {
            let b = l.atom() as usize;
            last[b] = last[b].max(p);
        }
    }
    if held {
        let observed = gp.evidence.iter().filter_map(|&(_, a, _)| a);
        if let Some(end) = observed.map(|a| at[a as usize]).max() {
            for &(_, q, _) in &gp.queries {
                if let Some(q) = q {
                    last[q as usize] = last[q as usize].max(end);
                }
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
    let (mut width, mut derived, mut facts) = (0isize, 0isize, 0);
    for (p, &a) in order.iter().enumerate() {
        (width, derived) = (width + change[p].0, derived + change[p].1);
        if gp.is_fact(a) {
            facts += 1;
        } else if derived > 0 {
            boundaries.push(Boundary {
                after: p,
                width: width as usize,
                facts,
            });
        }
    }
    (at, boundaries, last)
}

/// The cuts among an order's `boundaries`: the one place the rule lives
/// (the module's comment) that reads the profile alone. The window opens
/// where the width is at most twice the facts since the last cut: GeNI's
/// link grids, the Roulette chains and the weather at 12 factories took 10
/// to 30% less work than at once the facts, the weather by days the same,
/// and at four times the window opened inside a weather day and named 36
/// atoms. A boundary inside a run of facts is no cut, as a boundary that
/// only facts cross: cut inside the facts a program starts with, the
/// weather at 3 days counted its facts from there and found no day's
/// frontier.
///
/// The narrowest boundary the cut skips is cut too where its names are no
/// more than the facts on each side of it. GeNI's weather at 17 factories
/// has 70 facts a day and a window that ended at the program's last
/// boundary, 2 wide, skipping that day's frontier, 19 wide: the compile
/// took 3.57 G work units with it skipped and 30 M with it cut. The link
/// grid at side 16 skips boundaries of 17 names with 9 facts on each side:
/// cutting those halved a compile of 1.1 M work units and took a fifth more
/// work in the walk.
fn cuts(boundaries: &[Boundary]) -> Vec<Boundary> {
    let mut cuts = Vec::new();
    // The facts at or before the last cut, and the next boundary.
    let (mut behind, mut i) = (0, 0);
    while i < boundaries.len() {
        let f = boundaries[i].facts - behind;
        if boundaries[i].width > 2 * f {
            i += 1;
            continue;
        }
        let mut end = i;
        while end + 1 < boundaries.len() && boundaries[end + 1].facts - behind <= 2 * f {
            end += 1;
        }
        let best = (i..=end)
            .min_by_key(|&j| (boundaries[j].width, j))
            .expect("a window");
        if boundaries[best].width > WIDEST {
            i = end + 1;
            continue;
        }
        let skipped = (i..best)
            .filter(|&j| {
                let b = boundaries[j];
                b.width <= WIDEST
                    && b.width <= b.facts - behind
                    && b.width <= boundaries[best].facts - b.facts
            })
            .min_by_key(|&j| (boundaries[j].width, j));
        cuts.extend(skipped.map(|j| boundaries[j]));
        cuts.push(boundaries[best]);
        (behind, i) = (boundaries[best].facts, best + 1);
    }
    cuts
}

/// The `cuts` that stand, in order: those where the cones of the derived
/// atoms crossing the cut, back to the last cut that stands, reach a fact or
/// an atom behind that cut that no rule after this cut reads. A reader after
/// the cut then holds a name in place of it. Anywhere else the names replace
/// nothing the later formulas would otherwise hold, and leave the links in
/// the evidence: a hidden Markov model's cut after an observation, with the
/// state crossing it unchanged since the cut before (each of Roulette's
/// models had a quarter to a half of its cuts so), or the pass-through cuts
/// of linreg_w10, 10 of its 17, without which its query work falls by 40%.
/// Pluck's sorted list has one such cut, and its compile goes from 1.37 G to
/// 1.10 G work units without it.
fn hiding(
    gp: &GroundProgram,
    deps: &Deps,
    at: &[u32],
    last: &[u32],
    cuts: &[Boundary],
) -> Vec<Boundary> {
    let mut derived: Vec<AtomId> = (0..gp.num_atoms() as AtomId)
        .filter(|&a| !gp.is_fact(a) && last[a as usize] > at[a as usize])
        .collect();
    derived.sort_unstable_by_key(|&a| at[a as usize]);
    let mut derived = derived.into_iter().peekable();
    let mut crossing: Vec<AtomId> = Vec::new();
    let mut seen = vec![usize::MAX; gp.num_atoms()];
    let mut stack: Vec<AtomId> = Vec::new();
    let mut stand: Vec<Boundary> = Vec::new();
    for (k, cut) in cuts.iter().enumerate() {
        let p = cut.after as u32;
        while let Some(a) = derived.next_if(|&a| at[a as usize] <= p) {
            crossing.push(a);
        }
        crossing.retain(|&a| last[a as usize] > p);
        stack.clear();
        stack.extend_from_slice(&crossing);
        for &a in &crossing {
            seen[a as usize] = k;
        }
        while let Some(a) = stack.pop() {
            let behind = stand
                .last()
                .is_some_and(|c| at[a as usize] as usize <= c.after);
            if gp.is_fact(a) || behind {
                if last[a as usize] <= p {
                    stand.push(*cut);
                    break;
                }
                continue;
            }
            for &r in &deps.by_head[a as usize] {
                for l in &gp.rules[r].body {
                    let b = l.atom();
                    if seen[b as usize] != k {
                        seen[b as usize] = k;
                        stack.push(b);
                    }
                }
            }
        }
    }
    stand
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
    /// before: one atom crosses each step's boundary, and every step is a
    /// segment of its own. Its facts stated as `facts` says.
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

    fn chain(steps: usize) -> String {
        chain_written(steps, Facts::WithTheirStep)
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

    /// The same chain is cut alike wherever its program states its facts:
    /// the cuts follow the circuit, not the file.
    #[test]
    fn a_chain_is_cut_alike_wherever_its_facts_are_stated() {
        let notes: Vec<String> = [Facts::First, Facts::WithTheirStep, Facts::Last]
            .into_iter()
            .map(|facts| {
                let gp = grounded(&chain_written(8, facts));
                let laid = lay_out(&gp, &Deps::of(&gp), false).unwrap();
                laid.named.expect("named").1
            })
            .collect();
        assert!(notes[0].contains("6 cuts, widest 1"), "{notes:?}");
        assert!(notes.iter().all(|n| *n == notes[0]), "{notes:?}");
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

    /// A conjunction of more states than a cut is named for is cut after
    /// every step, however its gates associate it, in the file's order and
    /// in the fan-in order. The fan-in walk enters the chain at its end and
    /// each link's body before the step's state, so it meets the states
    /// from the first: entered at the last state, it would place every
    /// step's first fact before the second's (the facts are stated first,
    /// and the grounder lists them first in a body).
    #[test]
    fn a_joint_is_cut_alike_however_its_gates_associate_it() {
        let notes: Vec<String> = [Joint::Body, Joint::Forward, Joint::Backward]
            .into_iter()
            .flat_map(|joint| {
                let gp = ground(
                    &parse_program(&joint_written(30, joint)).expect("parses"),
                    &GroundOptions {
                        views: false,
                        ..GroundOptions::default()
                    },
                )
                .expect("grounds");
                let gp = chained(gp);
                assert_eq!(gp.stats.links, 28, "{joint:?}");
                [false, true].map(|fan_in| {
                    let laid = lay_out(&gp, &Deps::of(&gp), fan_in).unwrap();
                    laid.named.expect("named").1
                })
            })
            .collect();
        assert!(notes[0].contains("29 cuts, widest 2"), "{notes:?}");
        assert!(notes.iter().all(|n| *n == notes[0]), "{notes:?}");
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

    #[test]
    fn a_chain_is_cut_after_every_step_but_the_last() {
        let gp = grounded(&chain(6));
        for fan_in in [false, true] {
            let order = positions(&gp, fan_in);
            let cuts = cuts(&profile(&gp, &order, false).1);
            assert_eq!(cuts.len(), 4, "fan_in {fan_in}: {cuts:?}");
            assert!(
                cuts.iter().all(|c| c.width == 1),
                "fan_in {fan_in}: {cuts:?}"
            );
            let laid = lay_out(&gp, &Deps::of(&gp), fan_in).unwrap();
            let (named, note) = laid.named.expect("named");
            assert!(note.contains("widest 1"), "{note}");
            // Every fact of the rewrite is laid out once.
            let facts: FxHashSet<AtomId> = named.facts().map(|(a, _)| a).collect();
            assert_eq!(laid.facts.len(), facts.len());
            assert_eq!(laid.facts.iter().copied().collect::<FxHashSet<_>>(), facts);
            // Each copy comes after the facts of the step it names and
            // before those of the step that reads it.
            let place: Vec<usize> = {
                let mut p = vec![usize::MAX; named.num_atoms()];
                for (i, &a) in laid.facts.iter().enumerate() {
                    p[a as usize] = i;
                }
                p
            };
            for ind in &named.encoding.indicators {
                let step: usize = named.atoms[ind.atom as usize].to_string()[1..]
                    .parse()
                    .expect("s<i>");
                let a = named
                    .atoms
                    .iter()
                    .position(|x| x.to_string() == format!("a{}", step + 1))
                    .expect("the next step");
                assert!(place[ind.var as usize] < place[a], "{note}");
            }
        }
    }

    /// A boundary a window skips on its way to a narrower one is cut too
    /// where its names are no more than the facts on each side, as the last
    /// day but one of a weather whose last boundary is 2 wide, and not
    /// where they are more, as on a link grid.
    #[test]
    fn a_skipped_boundary_is_cut_where_its_names_are_few_against_the_facts() {
        let b = |width, facts| Boundary {
            after: facts,
            width,
            facts,
        };
        let weather = [b(19, 70), b(19, 140), b(2, 208)];
        let facts: Vec<usize> = cuts(&weather).iter().map(|c| c.facts).collect();
        assert_eq!(facts, [70, 140, 208]);
        let grid = [b(17, 9), b(16, 18)];
        let facts: Vec<usize> = cuts(&grid).iter().map(|c| c.facts).collect();
        assert_eq!(facts, [18]);
        // Wider than a cut is named: left out however many facts surround it.
        let wide = [b(21, 60), b(2, 100)];
        let facts: Vec<usize> = cuts(&wide).iter().map(|c| c.facts).collect();
        assert_eq!(facts, [100]);
    }

    /// Three chains stated a day at a time, each day's facts first, and a
    /// last day that reads them into one atom. The window from the last day
    /// but one reaches that atom's boundary, 1 wide, and skips the day's
    /// frontier, 3 wide with 6 facts on each side, which is cut too.
    #[test]
    fn a_program_that_narrows_at_its_end_keeps_the_day_before_it() {
        let mut src = String::from("0.5::s0_0. 0.5::s0_1. 0.5::s0_2.\n");
        for t in 1..=4 {
            for j in 0..3 {
                src += &format!("0.7::f{t}_{j}. 0.2::g{t}_{j}.\n");
            }
            for j in 0..3 {
                src += &format!(
                    "s{t}_{j} :- s{p}_{j}, f{t}_{j}. s{t}_{j} :- \\+s{p}_{j}, g{t}_{j}.\n",
                    p = t - 1
                );
            }
        }
        src += "0.3::q0. 0.4::q1. 0.5::q2. 0.6::h. 0.2::i. 0.9::k.\n\
                a :- s4_0, q0. a :- s4_1, q1. a :- s4_2, q2.\n\
                b :- a, h. b :- \\+a, i. z :- b, k. query(z).\n";
        let gp = grounded(&src);
        let order = positions(&gp, false);
        let (_, boundaries, _) = profile(&gp, &order, false);
        let cut_after: Vec<String> = cuts(&boundaries)
            .iter()
            .map(|c| gp.atoms[order[c.after] as usize].to_string())
            .collect();
        for t in 1..=4 {
            assert!(cut_after.contains(&format!("s{t}_2")), "{cut_after:?}");
        }
    }

    /// A hidden Markov model stated step by step: each state reads the one
    /// before and two facts, and each observation reads the state and two
    /// facts of its own. The window cuts after every atom, and a cut after
    /// an observation hides nothing: the state crossing it is the one
    /// the cut before it named, and no atom it reaches stops crossing there.
    #[test]
    fn a_cut_that_hides_nothing_does_not_stand() {
        let mut src = String::from("0.5::s0.\n");
        for i in 1..=6 {
            src += &format!(
                "0.3::a{i}. 0.6::b{i}.\ns{i} :- s{p}, a{i}. s{i} :- \\+s{p}, b{i}.\n\
                 0.9::c{i}. 0.2::d{i}.\no{i} :- s{i}, c{i}. o{i} :- \\+s{i}, d{i}.\nevidence(o{i}).\n",
                p = i - 1
            );
        }
        src += "query(s6).\n";
        let gp = grounded(&src);
        let order = positions(&gp, false);
        let (at, boundaries, last) = profile(&gp, &order, false);
        let every = cuts(&boundaries);
        let stand = hiding(&gp, &Deps::of(&gp), &at, &last, &every);
        let after = |c: &Boundary| gp.atoms[order[c.after] as usize].to_string();
        let names: Vec<String> = stand.iter().map(after).collect();
        assert!(
            every.iter().map(after).any(|a| a.starts_with('o')),
            "{every:?}"
        );
        assert!(names.iter().all(|a| a.starts_with('s')), "{names:?}");
        assert!(names.len() >= 4, "{names:?}");
    }

    /// Where the rules read facts alone, no derived atom crosses a
    /// boundary, and nothing is named.
    #[test]
    fn a_program_whose_rules_read_facts_alone_is_not_named() {
        let gp = grounded(
            "0.5::a. 0.5::b. 0.5::c. x :- a, b. x :- \\+c. y :- b, c. z :- \\+a, c. query(x). query(y). query(z).",
        );
        for fan_in in [false, true] {
            let laid = lay_out(&gp, &Deps::of(&gp), fan_in).unwrap();
            assert!(laid.named.is_none());
            // The facts in the order's own order.
            let mut facts: Vec<AtomId> = gp.facts().map(|(a, _)| a).collect();
            if fan_in {
                let place = crate::ground::deps::fan_in(&gp, true);
                facts.sort_by_key(|&a| place[a as usize]);
            }
            assert_eq!(laid.facts.len(), 3);
            assert_eq!(laid.facts, facts);
        }
    }
}
