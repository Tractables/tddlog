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
//! reads back across (a rule before it reads an atom after it) is none.
//! Where no boundary is narrow, the program is one window, as without
//! windows.
//!
//! Window `j` is the rules of the atoms between its cuts, the atoms of the
//! incoming frontier defined by the message, the queries and evidence that
//! lie in it, and, but for the last window, a query for each of the `2^w`
//! conjunctions of the outgoing frontier's literals. Their answers, given
//! the window's evidence and the evidence before it, are the frontier's
//! joint: the message. The next window reads it as an annotated disjunction
//! over the rows of non-zero probability, lowered as the grounder lowers
//! one (a chain of switches, [`chain_weights`]), each frontier atom the
//! disjunction of its rows; a frontier of one atom is a fact. The
//! probability of the evidence so far, `Z`, is the next window's weight,
//! so the last window's evidence probability is the program's.

use crate::QueryResult;
use crate::eval::Count;
use crate::ground::{self, AtomId, AtomKind, GLit, GroundProgram, GroundRule, Prob, chain_weights};
use crate::syntax::{Atom, Term};
use crate::vtree::window_profile;

/// The conjunctions of a frontier's literals that a window asks.
const ROW: &str = "$wq";
/// The rows of an incoming message, and their switches.
const MESSAGE_ROW: &str = "$wr";
const SWITCH: &str = "$ws";

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
    /// `decision`, is not cut.
    pub(super) fn of(gp: &GroundProgram, bounds: &WindowBounds) -> Option<Windows> {
        if !gp.encoding.is_empty() || gp.stats.decisions.is_some() || gp.rules.len() <= bounds.rules
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
        let fits = |p: usize| boundaries[p].width <= bounds.widest && !crossed[boundaries[p].after];
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
            // Nothing is left after the program's last rule.
            if held[after] == gp.rules.len() {
                break;
            }
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

    /// The atoms that cross the cut after window `j`.
    pub(super) fn frontier(&self, j: usize) -> &[AtomId] {
        self.frontiers.get(j).map_or(&[], Vec::as_slice)
    }

    /// Window `j`'s program, of weight `weight`, its incoming frontier
    /// defined by `message`, the answers of the window before to its
    /// conjunction queries ([`Self::message`]), finished again
    /// ([`ground::window`]); and the places among `gp`'s queries of the
    /// answers it gives before its conjunctions'.
    pub(super) fn program(
        &self,
        gp: &GroundProgram,
        j: usize,
        message: &[f64],
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

    /// The message to the window after one whose answers are `results`,
    /// its own `places` first: its conjunction queries' answers. `None`
    /// where a row may be under the least normal float against the rows'
    /// sum without being impossible, which the next window would read as
    /// impossible: a row under it that is not zero, or zero where the run
    /// answered a query under the least normal float whose count is not
    /// zero (`underflowed`, [`crate::query::QuerySizes::underflows`]).
    pub(super) fn message(
        results: &[QueryResult],
        places: usize,
        underflowed: bool,
    ) -> Option<Vec<f64>> {
        let rows: Vec<f64> = results[places..].iter().map(|r| r.probability).collect();
        let least = f64::MIN_POSITIVE * rows.iter().sum::<f64>();
        let passed = |p: f64| p >= least || p == 0.0 && !underflowed;
        (least > 0.0 && rows.iter().all(|&p| passed(p))).then_some(rows)
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
/// rows given the evidence before the cut, a row of probability zero
/// impossible ([`Windows::message`]): one atom, a fact of its
/// probability; more, an annotated disjunction over the rows of non-zero
/// probability, each row a switch of the chain, and each atom the
/// disjunction of the rows it holds in.
fn read_message(part: &mut GroundProgram, j: usize, w: usize, message: &[f64]) {
    debug_assert_eq!(message.len(), (1 << w) * usize::from(w > 0));
    if w == 1 {
        let total = message[0] + message[1];
        debug_assert!(total > 0.0, "a message of rows {message:?}");
        part.kinds[0] = AtomKind::Fact(Prob {
            p: message[1] / total,
            not: message[0] / total,
        });
        return;
    }
    let rows: Vec<usize> = (0..message.len()).filter(|&k| message[k] > 0.0).collect();
    let masses: Vec<f64> = rows.iter().map(|&k| message[k]).collect();
    let mut weights = Vec::new();
    chain_weights(&masses, 0.0, &mut weights);
    let mut before = Vec::with_capacity(rows.len());
    for (t, (&k, &q)) in rows.iter().zip(&weights).enumerate() {
        let switch = new_atom(part, SWITCH, j, t, AtomKind::Fact(q));
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
