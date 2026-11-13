//! A ground program run as windows between narrow cuts of its order.
//!
//! A long chain of narrow steps (a Markov chain unrolled over time, a loop
//! over a level) compiles into formulas that the run holds to its end: its
//! memory grows with the chain. Cut where few atoms cross, the program is a
//! sequence of windows, each run alone and freed before the next: the
//! memory is a window's, whatever the chain's length.
//!
//! The order and the widths of its boundaries are naming's
//! ([`window_profile`]), the rules taken by the depth of their heads and
//! each fact right before its first reader, and a query's atom crossing
//! every boundary up to the last observation, whose window answers it. A cut is taken once the
//! window since the last cut holds [`WindowBounds::rules`] ground rules: at
//! the narrowest boundary of width at most [`WindowBounds::widest`] among
//! those where the window holds up to twice as many, the earliest on a tie,
//! or else at the first such boundary after them. A boundary that a rule
//! reads back across (a rule before it reads an atom after it) is none, and
//! so is one with fewer than [`WindowBounds::rules`] rules after it: every
//! window, the last included, holds that many. Where no boundary is narrow,
//! the program is one window, as without windows.
//!
//! Window `j` is the rules of the atoms between its cuts, the atoms of the
//! incoming frontier defined by the message, the queries and evidence that
//! lie in it, and, but for the last window, a query for each of the `2^w`
//! conjunctions of the outgoing frontier's literals. Their answers, given
//! the window's evidence and the evidence before it, are the frontier's
//! joint: the message, taken as counts, which keep a row under the least
//! float. The next window reads it as an annotated disjunction over the
//! rows of non-zero count, lowered as the grounder lowers one (a chain of
//! switches, its sums taken from the end), each frontier atom the disjunction
//! of its rows; a frontier of one atom is a fact where it can be. The
//! probability of the evidence so far, `Z`, is the next window's weight,
//! so the last window's evidence probability is the program's.

use crate::eval::Count;
use crate::ground::{self, AtomId, AtomKind, GLit, GroundProgram, GroundRule, Prob};
use crate::syntax::{Atom, Term};
use crate::vtree::window_profile;

/// The conjunctions of a frontier's literals that a window asks.
const ROW: &str = "$wq";
/// The rows of an incoming message, and their switches.
const MESSAGE_ROW: &str = "$wr";
const SWITCH: &str = "$ws";
/// The facts whose conjunction is a switch under the least float.
const FACTORS: &str = "$wf";

/// Where a program is cut into windows.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WindowBounds {
    /// The widest cut: its message asks `2^widest` queries.
    pub(crate) widest: usize,
    /// The ground rules a window holds before it is cut.
    pub(crate) rules: usize,
}

impl WindowBounds {
    /// Four atoms, 2^17 rules. Cut by hand at 120 000 rules, two chains
    /// of 300 000 and 3 000 000 steps held 310 MB a window at either
    /// length, where the whole run held 2.9 and 31 GB, and took 0.62 to
    /// 0.90 of the whole run's time.
    pub(crate) const DEFAULT: WindowBounds = WindowBounds {
        widest: 4,
        rules: 1 << 17,
    };
}

/// A program cut into windows: window `j` holds the atoms of the order
/// after `cuts[j - 1]` up to `cuts[j]`, the last window the rest.
pub(super) struct Windows {
    order: Vec<AtomId>,
    at: Vec<u32>,
    cuts: Vec<usize>,
    /// Per cut, the atoms that cross it, in order.
    frontiers: Vec<Vec<AtomId>>,
    /// Per window, the indices of its rules, its queries and its
    /// observations in the program's lists.
    rules: Vec<Vec<u32>>,
    queries: Vec<Vec<usize>>,
    evidence: Vec<Vec<usize>>,
}

impl Windows {
    /// The windows of `gp` under `bounds`; `None` where it has no cut. A
    /// program read through indicators, or conditioned on its guards for
    /// `decision`, is not cut. Nor is one of fewer than two windows' rules,
    /// which has no cut that leaves a window's rules on both sides; its
    /// profile is not computed.
    pub(super) fn of(gp: &GroundProgram, bounds: &WindowBounds) -> Option<Windows> {
        if !gp.encoding.is_empty()
            || gp.stats.decisions.is_some()
            || gp.rules.len() < bounds.rules.saturating_mul(2)
        {
            return None;
        }
        let (order, at, boundaries, last) = window_profile(gp);
        let n = order.len();
        // The rules up to each position, and the positions a rule reads
        // back across.
        let mut held = vec![0usize; n];
        let mut back = vec![0isize; n + 1];
        for r in &gp.rules {
            let h = at[r.head as usize] as usize;
            held[h] += 1;
            let far = r.body.iter().map(|l| at[l.atom() as usize] as usize).max();
            if let Some(far) = far.filter(|&f| f > h) {
                back[h] += 1;
                back[far] -= 1;
            }
        }
        let mut crossed = vec![false; n];
        let (mut sum, mut open) = (0, 0);
        for p in 0..n {
            sum += held[p];
            held[p] = sum;
            open += back[p];
            crossed[p] = open > 0;
        }
        // A cut leaves a last window of `bounds.rules` rules or more: a
        // shorter one would make the window before it nearly the whole
        // program, finished again and asked its frontier's rows.
        let total = gp.rules.len();
        let fits = |p: usize| {
            let after = boundaries[p].after;
            boundaries[p].width <= bounds.widest
                && !crossed[after]
                && total - held[after] >= bounds.rules
        };
        let mut cuts = Vec::new();
        let (mut behind, mut i) = (0, 0);
        while i < boundaries.len() {
            if held[boundaries[i].after] - behind < bounds.rules || !fits(i) {
                i += 1;
                continue;
            }
            let mut best = i;
            let mut j = i + 1;
            while j < boundaries.len() && held[boundaries[j].after] - behind < 2 * bounds.rules {
                if fits(j) && boundaries[j].width < boundaries[best].width {
                    best = j;
                }
                j += 1;
            }
            let after = boundaries[best].after;
            cuts.push(after);
            behind = held[after];
            i = best + 1;
        }
        if cuts.is_empty() {
            return None;
        }
        let window_of = |p: u32| cuts.partition_point(|&c| c < p as usize);
        let mut frontiers = vec![Vec::new(); cuts.len()];
        for a in 0..gp.num_atoms() {
            if last[a] > at[a] {
                for f in &mut frontiers[window_of(at[a])..window_of(last[a])] {
                    f.push(a as AtomId);
                }
            }
        }
        for (f, b) in frontiers.iter_mut().zip(&cuts) {
            f.sort_unstable_by_key(|&a| at[a as usize]);
            debug_assert!(
                f.len() <= bounds.widest,
                "a cut of {} atoms after {b}",
                f.len()
            );
        }
        let count = cuts.len() + 1;
        let mut rules = vec![Vec::new(); count];
        for (ri, r) in gp.rules.iter().enumerate() {
            rules[window_of(at[r.head as usize])].push(ri as u32);
        }
        // An impossible atom's query or observation goes with the first
        // window; a query, with the last observation's where it is later.
        let end = gp
            .evidence
            .iter()
            .filter_map(|&(_, a, _)| a.map(|a| at[a as usize]))
            .max()
            .unwrap_or(0);
        let mut queries = vec![Vec::new(); count];
        for (k, &(_, q, _)) in gp.queries.iter().enumerate() {
            queries[q.map_or(0, |q| window_of(at[q as usize].max(end)))].push(k);
        }
        let mut evidence = vec![Vec::new(); count];
        for (k, &(_, e, _)) in gp.evidence.iter().enumerate() {
            evidence[e.map_or(0, |e| window_of(at[e as usize]))].push(k);
        }
        Some(Windows {
            order,
            at,
            cuts,
            frontiers,
            rules,
            queries,
            evidence,
        })
    }

    /// How many windows.
    pub(super) fn len(&self) -> usize {
        self.cuts.len() + 1
    }

    /// How many of the program's rules window `j` holds.
    #[cfg(test)]
    pub(super) fn held(&self, j: usize) -> usize {
        self.rules[j].len()
    }

    /// The atoms that cross the cut after window `j`.
    pub(super) fn frontier(&self, j: usize) -> &[AtomId] {
        self.frontiers.get(j).map_or(&[], Vec::as_slice)
    }

    /// Window `j`'s program, of weight `weight`, its incoming frontier
    /// defined by `message`, the counts of the window before's answers to
    /// its conjunction queries, finished again
    /// ([`ground::window`]); and the places among `gp`'s queries of the
    /// answers it gives before its conjunctions'.
    pub(super) fn program(
        &self,
        gp: &GroundProgram,
        j: usize,
        message: &[Count],
        weight: Count,
    ) -> (GroundProgram, Vec<usize>) {
        let incoming = if j == 0 {
            &[][..]
        } else {
            self.frontier(j - 1)
        };
        let start = if j == 0 { 0 } else { self.cuts[j - 1] + 1 };
        let end = self.cuts.get(j).map_or(self.order.len(), |&c| c + 1);
        let local = |a: AtomId| -> AtomId {
            let p = self.at[a as usize] as usize;
            if p >= start {
                (incoming.len() + p - start) as AtomId
            } else {
                incoming
                    .iter()
                    .position(|&f| f == a)
                    .expect("an atom of the window or of its incoming frontier")
                    as AtomId
            }
        };
        let mut part = GroundProgram::default();
        part.atoms = (incoming.iter().chain(&self.order[start..end]))
            .map(|&a| gp.atoms[a as usize].clone())
            .collect();
        part.kinds = incoming
            .iter()
            .map(|_| AtomKind::Derived)
            .chain(self.order[start..end].iter().map(|&a| gp.kinds[a as usize]))
            .collect();
        read_message(&mut part, j, incoming.len(), message);
        part.rules.extend(self.rules[j].iter().map(|&ri| {
            let r = &gp.rules[ri as usize];
            GroundRule {
                head: local(r.head),
                body: r
                    .body
                    .iter()
                    .map(|&l| GLit::new(local(l.atom()), l.is_positive()))
                    .collect(),
                pairs: r.pairs,
            }
        }));
        let mut places = self.queries[j].clone();
        part.queries = places
            .iter()
            .map(|&k| {
                let (atom, q, positive) = &gp.queries[k];
                (atom.clone(), q.map(local), *positive)
            })
            .collect();
        part.evidence = self.evidence[j]
            .iter()
            .map(|&k| {
                let (atom, e, value) = &gp.evidence[k];
                (atom.clone(), e.map(local), *value)
            })
            .collect();
        if j + 1 == self.len() {
            part.observed_query = gp.observed_query.clone();
            if part.observed_query.is_some() {
                places.push(gp.queries.len());
            }
        }
        // The conjunctions of the outgoing frontier's literals, row `k`
        // true where each atom `i` is bit `i` of `k`.
        let outgoing: Vec<AtomId> = self.frontier(j).iter().map(|&a| local(a)).collect();
        for k in 0..(1usize << outgoing.len()) * usize::from(j + 1 < self.len()) {
            let row = new_atom(&mut part, ROW, j, k, AtomKind::Derived);
            let body = (outgoing.iter().enumerate())
                .map(|(i, &f)| GLit::new(f, k >> i & 1 == 1))
                .collect();
            part.rules.push(GroundRule::new(row, body));
            part.queries
                .push((part.atoms[row as usize].clone(), Some(row), true));
        }
        part.weight = weight;
        (ground::window(gp, part), places)
    }
}

/// A new atom of `part` named `name(j, k)`, of kind `kind`.
fn new_atom(part: &mut GroundProgram, name: &str, j: usize, k: usize, kind: AtomKind) -> AtomId {
    part.atoms.push(Atom::new(
        name,
        vec![Term::Int(j as i64), Term::Int(k as i64)],
    ));
    part.kinds.push(kind);
    (part.atoms.len() - 1) as AtomId
}

/// The `w` atoms of window `j`'s incoming frontier, its first atoms,
/// defined in `part` by `message`, the probability of each of their `2^w`
/// rows given the evidence before the cut, a row of count zero impossible:
/// one atom whose two rows are normal floats against their sum, a fact of
/// its probability; else an annotated disjunction over the rows of
/// non-zero probability, each row a switch of the chain ([`switch`]), and
/// each atom the disjunction of the rows it holds in. The rows go from the
/// least, so that each switch but the last fails with probability 1/2 or
/// more, and only its truth may be under the least float.
fn read_message(part: &mut GroundProgram, j: usize, w: usize, message: &[Count]) {
    debug_assert_eq!(message.len(), (1 << w) * usize::from(w > 0));
    if w == 1 {
        let total = (message[0] + message[1]).normalized();
        debug_assert!(!total.is_zero(), "a message of rows {message:?}");
        let [not, p] = [0, 1].map(|k| message[k].ratio(total));
        if [(not, 0), (p, 1)]
            .iter()
            .all(|&(x, k)| x.is_normal() || message[k].is_zero())
        {
            part.kinds[0] = AtomKind::Fact(Prob { p, not });
            return;
        }
    }
    let mut rows: Vec<usize> = (0..message.len())
        .filter(|&k| !message[k].is_zero())
        .collect();
    rows.sort_by(|&x, &y| message[x].ratio(message[y]).total_cmp(&1.0));
    // Each switch's chance and its complement's: the row against the rows
    // from it on.
    let mut weights = vec![(Count::ONE, 0.0); rows.len()];
    let mut after = Count::ZERO;
    for (t, &k) in rows.iter().enumerate().rev() {
        let from = (message[k] + after).normalized();
        weights[t] = ((message[k] / from).normalized(), after.ratio(from));
        after = from;
    }
    let mut before = Vec::with_capacity(rows.len());
    for (t, (&k, &(p, not))) in rows.iter().zip(&weights).enumerate() {
        let switch = switch(part, j, t, p, not);
        let row = new_atom(part, MESSAGE_ROW, j, t, AtomKind::Derived);
        let mut body: Vec<GLit> = before.iter().map(|&s| GLit::Neg(s)).collect();
        body.push(GLit::Pos(switch));
        part.rules.push(GroundRule::new(row, body));
        before.push(switch);
        for i in (0..w).filter(|i| k >> i & 1 == 1) {
            part.rules
                .push(GroundRule::new(i as AtomId, vec![GLit::Pos(row)]));
        }
    }
}

/// `2^-FACTOR`: the probability of each of the facts but the last whose
/// conjunction is a switch under the least float.
const FACTOR: i64 = 1000;

/// Switch `t` of window `j`'s message, true with probability `p` and false
/// with `not`: a fact, or where `p` is under the least normal float, an atom
/// true where facts of `2^-FACTOR` each and one of the rest all hold, so
/// that their product is `p` and its complement `not`.
fn switch(part: &mut GroundProgram, j: usize, t: usize, p: Count, not: f64) -> AtomId {
    let q = p.to_f64();
    if q.is_normal() {
        return new_atom(part, SWITCH, j, t, AtomKind::Fact(Prob { p: q, not }));
    }
    let switch = new_atom(part, SWITCH, j, t, AtomKind::Derived);
    let (mut body, mut rest) = (Vec::new(), p);
    loop {
        let q = rest.to_f64();
        let p = if q.is_normal() {
            q
        } else {
            (-FACTOR as f64).exp2()
        };
        let k = part.atoms.len();
        let fact = new_atom(
            part,
            FACTORS,
            j,
            k,
            AtomKind::Fact(Prob { p, not: 1.0 - p }),
        );
        body.push(GLit::Pos(fact));
        if q.is_normal() {
            break;
        }
        rest = rest.shifted(FACTOR);
    }
    part.rules.push(GroundRule::new(switch, body));
    switch
}
