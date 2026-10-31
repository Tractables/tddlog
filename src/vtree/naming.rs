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
//! Whether a cut's names pay is settled on the compiled program
//! ([`crate::tp::settle`]): where the joint of a cut's links is larger than
//! what the formulas reading its copies would hold with them expanded, the
//! copies are composed away and the cut's links leave the evidence.

use rustc_hash::FxHashSet;

use crate::ground::deps::{Deps, fan_in};
use crate::ground::{AtomId, GroundProgram};
use crate::indicators;

/// The widest cut named. The evidence holds the joint of a cut's names,
/// up to `2^width` pairs a level there, where the expanded formulas each
/// hold their own atom's: GeNI's weather under the fan-in walk's order has
/// a 46 wide cut, and naming it ran out of 13 GB. 2^20 pairs is the first
/// watch `auto` puts on a trial's diagrams. The widest cuts that helped
/// were 17 (link grids at side 16) and 14 (weather at 12 factories).
const WIDEST: usize = 20;

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
pub(super) fn lay_out(gp: &GroundProgram, deps: &Deps, fan_in: bool) -> Laid {
    let order = positions(gp, fan_in);
    let facts: Vec<AtomId> = order.iter().copied().filter(|&a| gp.is_fact(a)).collect();
    // A program that reads atoms through indicators already is compiled as
    // it is (`Kind::eliminated`).
    if !gp.encoding.is_empty() {
        return Laid { facts, named: None };
    }
    let (at, boundaries) = profile(gp, &order);
    let cuts = cuts(&boundaries);
    if cuts.is_empty() {
        return Laid { facts, named: None };
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
    let (mut named, units) = indicators::name(gp, unit_of);
    if named.encoding.is_empty() {
        return Laid { facts, named: None };
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
    Laid {
        facts: laid,
        named: Some((named, note)),
    }
}

/// Every atom of `gp`, fact or derived, in the trial's order. The file's:
/// a fact where the program states it, a derived atom at its last rule.
/// The ground program keeps no text positions, but the grounder numbers
/// atoms as the program names them and keeps its rules in program order,
/// so a fact goes before the first rule that names an atom numbered after
/// it. With `fan_in`, the fan-in order, the deepest roots first
/// ([`fan_in`]): each atom after what it reads.
pub(super) fn positions(gp: &GroundProgram, fan_in_order: bool) -> Vec<AtomId> {
    let n = gp.num_atoms();
    if fan_in_order {
        let place = fan_in(gp, true);
        let mut order: Vec<AtomId> = (0..n as AtomId).collect();
        order.sort_unstable_by_key(|&a| place[a as usize]);
        return order;
    }
    let mut last_rule = vec![usize::MAX; n];
    for (ri, r) in gp.rules.iter().enumerate() {
        last_rule[r.head as usize] = ri;
    }
    let facts: Vec<AtomId> = gp.facts().map(|(a, _)| a).collect();
    let mut order = Vec::with_capacity(n);
    let (mut f, mut named) = (0, 0);
    for (ri, r) in gp.rules.iter().enumerate() {
        named = r
            .body
            .iter()
            .map(|l| l.atom())
            .chain([r.head])
            .fold(named, |m, a| m.max(a + 1));
        while f < facts.len() && facts[f] < named {
            order.push(facts[f]);
            f += 1;
        }
        if last_rule[r.head as usize] == ri {
            order.push(r.head);
        }
    }
    order.extend_from_slice(&facts[f..]);
    // Derived atoms with no rule, last.
    order.extend(
        (0..n as AtomId).filter(|&a| !gp.is_fact(a) && last_rule[a as usize] == usize::MAX),
    );
    order
}

/// A boundary of an order that a cut may take: one after a derived atom,
/// crossed by a derived atom.
#[derive(Clone, Copy, Debug)]
struct Boundary {
    /// The position it follows.
    after: usize,
    /// The atoms at or before it that a rule of an atom after it reads.
    width: usize,
    /// The facts at or before it.
    facts: usize,
    /// Whether only single-literal rules and no facts follow it.
    trailing: bool,
}

/// Each atom's position in `order`, and the boundaries of `order` that a
/// cut may take, with their widths: the profile, computed once.
fn profile(gp: &GroundProgram, order: &[AtomId]) -> (Vec<u32>, Vec<Boundary>) {
    let n = order.len();
    let mut at = vec![0u32; gp.num_atoms()];
    for (p, &a) in order.iter().enumerate() {
        at[a as usize] = p as u32;
    }
    // Each atom's last reader's position, and each head's body literals.
    let mut weight = vec![0usize; gp.num_atoms()];
    let mut last = vec![0u32; gp.num_atoms()];
    for r in &gp.rules {
        weight[r.head as usize] += r.body.len().max(1);
        let p = at[r.head as usize];
        for l in &r.body {
            let b = l.atom() as usize;
            last[b] = last[b].max(p);
        }
    }
    let until = if !gp.evidence.is_empty() && gp.queries.len() == 1 {
        n
    } else {
        order
            .iter()
            .rposition(|&a| gp.is_fact(a) || weight[a as usize] > 1)
            .unwrap_or(0)
    };
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
                trailing: p >= until,
            });
        }
    }
    (at, boundaries)
}

/// The cuts among an order's `boundaries`: the one place the rule lives
/// (the module's comment), which reads the profile alone. The window opens
/// where the width is at most twice the facts since the last cut: GeNI's
/// link grids, the Roulette chains and the weather at 12 factories took 10
/// to 30% less work than at once the facts, the weather by days the same,
/// and at four times the window opened inside a weather day and named 36
/// atoms. A boundary inside a run of facts is no cut, as a boundary that
/// only facts cross: cut inside the facts a program starts with, the
/// weather at 3 days counted its facts from there and found no day's
/// frontier.
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
        if boundaries[best].width > WIDEST || boundaries[best].trailing {
            i = end + 1;
            continue;
        }
        cuts.push(boundaries[best]);
        (behind, i) = (boundaries[best].facts, best + 1);
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

    /// A chain of steps, each reading its own two facts and the step
    /// before: one atom crosses each step's boundary, and every step is a
    /// segment of its own.
    fn chain(steps: usize) -> String {
        let mut src = String::from("0.5::s0.\n");
        for i in 1..steps {
            src += &format!(
                "0.3::a{i}. 0.6::b{i}. s{i} :- s{p}, a{i}. s{i} :- \\+s{p}, b{i}.\n",
                p = i - 1
            );
        }
        src + &format!("query(s{}).", steps - 1)
    }

    #[test]
    fn the_file_order_puts_each_atom_after_its_last_rule_and_each_fact_where_stated() {
        let gp = grounded(&chain(4));
        let names: Vec<String> = positions(&gp, false)
            .iter()
            .map(|&a| gp.atoms[a as usize].to_string())
            .collect();
        assert_eq!(
            names,
            ["s0", "a1", "b1", "s1", "a2", "b2", "s2", "a3", "b3", "s3"]
        );
    }

    #[test]
    fn a_chain_is_cut_after_every_step_but_the_last() {
        let gp = grounded(&chain(6));
        for fan_in in [false, true] {
            let order = positions(&gp, fan_in);
            let cuts = cuts(&profile(&gp, &order).1);
            assert_eq!(cuts.len(), 4, "fan_in {fan_in}: {cuts:?}");
            assert!(
                cuts.iter().all(|c| c.width == 1),
                "fan_in {fan_in}: {cuts:?}"
            );
            let laid = lay_out(&gp, &Deps::of(&gp), fan_in);
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

    /// Where the rules read facts alone, no derived atom crosses a
    /// boundary, and nothing is named.
    #[test]
    fn a_program_whose_rules_read_facts_alone_is_not_named() {
        let gp = grounded(
            "0.5::a. 0.5::b. 0.5::c. x :- a, b. x :- \\+c. y :- b, c. z :- \\+a, c. query(x). query(y). query(z).",
        );
        for fan_in in [false, true] {
            let laid = lay_out(&gp, &Deps::of(&gp), fan_in);
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
