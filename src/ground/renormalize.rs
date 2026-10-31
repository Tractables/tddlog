//! Renormalizing the rows of tables over the heads the evidence leaves
//! them, part of the conditioning on the evidence ([`super::condition`]).
//! A row of a table, an annotated disjunction's chain of switches under
//! one assignment of its parents, that the conditioning leaves some heads
//! false and others not is renormalized over the heads it keeps
//! ([`renormalize`]): a chain over those heads alone, and the mass it
//! keeps on a fact its context requires, so that the false heads leave the
//! program and the evidence walk reads the table's level without them. A
//! row that keeps a head made true reads one fact of the head's
//! probability instead, where that is sound. Each world counted keeps its
//! weight, and the masses taken out are a factor of both counts.

use rustc_hash::FxHashMap;

use super::normalize::chain_weights;
use super::propagation::Val;
use super::tables::Tables;
use super::{AtomId, AtomKind, BITS_FROM, GLit, GroundProgram, GroundRule, Prob};
use crate::eval::Count;
use crate::syntax::{Atom, Term};

/// The mass of no head of a row whose rules' own literals are `chains`,
/// where they are an annotated disjunction's chain: in some order, chain
/// `i` the negations of the switches of chains `0` to `i - 1` and switch
/// `i`, the last one perhaps the negations alone. That is the product of
/// the negations of the switches, each its own weight, or zero where the
/// last chain is the negations alone. None for chains of another shape.
fn no_head(chains: &[Vec<GLit>], weight_of: &impl Fn(GLit) -> Prob) -> Option<f64> {
    let mut order: Vec<&Vec<GLit>> = chains.iter().collect();
    order.sort_by_key(|c| c.iter().filter(|l| !l.is_positive()).count());
    let mut switches: Vec<GLit> = Vec::with_capacity(order.len());
    for (i, chain) in order.iter().enumerate() {
        let positive: Vec<GLit> = chain.iter().copied().filter(|l| l.is_positive()).collect();
        let mut negations: Vec<GLit> = chain
            .iter()
            .copied()
            .filter(|l| !l.is_positive())
            .map(|l| GLit::Pos(l.atom()))
            .collect();
        negations.sort_unstable();
        let mut before = switches.clone();
        before.sort_unstable();
        if negations != before {
            return None;
        }
        match positive[..] {
            [s] => switches.push(s),
            [] if i + 1 == order.len() => return Some(0.0),
            _ => return None,
        }
    }
    Some(Prob::cube(switches.iter().map(|&s| weight_of(GLit::Neg(s.atom())))).p)
}

/// The rows of `gp`'s tables that lose heads to the conditioning
/// renormalized over the heads they keep, after the conditioning's rewrite
/// with the values `val` it was given.
///
/// A row is the rules of a table ([`Tables`]) with one context, the body
/// literals that are not the table's own facts: an annotated disjunction's
/// chain of switches under one assignment of its parents. Where the
/// conditioning made some of its heads false and left others, the row
/// weighs `θ_h`, the product of its switches' weights along `h`'s chain,
/// for each head `h`, and keeps the mass `m`, the `θ` of the heads not
/// false and what the row leaves to no head. It becomes a chain over the
/// heads kept with weights `θ_h / m`, and `m` a fact `w` that the context
/// requires: `bad :- context, \+w`, observed false. In each world counted
/// the row's head is one it keeps (the others are false), and it weighs
/// `θ_h / m · m`, as before. A row without a context puts `m` into the
/// conditioning's factor. Rows that keep no head, or lose no mass, stay.
///
/// `bad` is an atom of its own, one for all the tables, unless a context
/// of the table reads a set of [`BITS_FROM`] values or more, which the
/// indicators read in binary. Such a table's rows put `w` first in the
/// chain of the heads they keep (`h :- context, w, ...`; sound, since the
/// evidence forces `w` where the context holds) and their masses' rules
/// on a head of the table the conditioning made false, which stays
/// observed false: each row's false heads merged into one, first in its
/// chain, in the table's own unit. Read through an atom of their own,
/// the masses of a table over a set read in binary do not compile in
/// memory (bn_mildew_ev's `dm_4`, 210 rows over the 100 values of
/// `dm_3`, where most of its values are ruled out); elsewhere the atom
/// of their own compiles smaller.
///
/// The chains of a row must exclude each other, and its switches be read
/// by no other row, but where the table's contexts exclude each other
/// pairwise: each reads one atom of each of the same exclusive sets,
/// positively, and nothing else, as the rows of a network's table do.
/// Such a table's rows share their new switches (the `k`-th of each
/// probability) and their masses' facts, as [`crate::ground::share`] does: in a
/// world where one row's context holds the others' do not, and a fact read
/// under them sums out of their part.
///
/// A row that keeps a head `h` the conditioning made true is not
/// renormalized. In a table whose contexts exclude each other pairwise,
/// `h` holds where the row's context does through that row alone (its
/// rules are the table's, and no other row's context holds there), so the
/// row weighs `θ_h` there: it becomes `h :- context, t` with a fact `t` of
/// weight `θ_h`, shared by the rows of one `θ_h`, and its false heads go.
/// That is not done where a context reads a set read in binary, and a row
/// of any other table stays as it is: the head's observation carries the
/// evidence. The rows of an observed child of two sets read in binary
/// (tests/it/binary.rs) did not compile in memory renormalized, the mass's
/// constraint beside the head's observation, nor as `h :- context, t`.
/// Returns the rows rewritten and the factor.
pub(super) fn renormalize(gp: &mut GroundProgram, tables: &Tables, val: &[Val]) -> (usize, Count) {
    let Some(owners) = Owners::of(gp, tables, val) else {
        return (0, Count::ONE);
    };
    let (rows, table_rows) = owners.rows();
    let row_of_fact = owners.row_of_fact(&rows, &table_rows);
    let mut rewrite = Rewrite::new(gp);
    let mut order: Vec<usize> = table_rows.keys().copied().collect();
    order.sort_unstable();
    for t in order {
        rewrite.table(&owners, &rows, &table_rows[&t], t, &row_of_fact);
    }
    rewrite.apply(gp)
}

/// A table's flags ([`Owners`]): it has a head that is false, one neither
/// false nor true, one true; a rule of it reads a fact not named, and one
/// of the table's own facts. `FALSE` is also cleared where the table
/// cannot change.
const FALSE: usize = 0;
const OPEN: usize = 1;
const TRUE: usize = 2;
const READS_FACT: usize = 3;
const READS_OWN: usize = 4;

/// Whether a table of flags `e` has a false head and one not false: only
/// such a table can have a row that changes.
fn may(e: &[bool; 5]) -> bool {
    e[FALSE] && (e[OPEN] || e[TRUE])
}

/// No rule reads the atom ([`Owners::reader`]).
const NONE: usize = usize::MAX;
/// Rules of two tables or more read the atom.
const MANY: usize = usize::MAX - 1;

/// The tables whose rows may change, and what renormalizing reads of the
/// program to tell them: each table's flags, the facts named, and the
/// table whose rules read each atom.
struct Owners<'a> {
    gp: &'a GroundProgram,
    tables: &'a Tables,
    val: &'a [Val],
    kinds: Vec<[bool; 5]>,
    named: Vec<bool>,
    reader: Vec<usize>,
}

impl<'a> Owners<'a> {
    /// `None` where no row of a table can change.
    fn of(gp: &'a GroundProgram, tables: &'a Tables, val: &'a [Val]) -> Option<Self> {
        let n = gp.num_atoms();
        let num_sets = tables.sets().len();
        let mut o = Owners {
            gp,
            tables,
            val,
            kinds: vec![[false; 5]; num_sets + n],
            named: Vec::new(),
            reader: Vec::new(),
        };
        // Each table's heads (`FALSE`, `OPEN`, `TRUE`); and whether a rule
        // of it reads a fact not named, and one of its own facts (below).
        // A row can change only in a table with a false head and one not
        // false.
        for r in &gp.rules {
            let head = if o.false_head(r.head) {
                FALSE
            } else if val[r.head as usize] == Val::True {
                TRUE
            } else {
                OPEN
            };
            o.kinds[tables.of(r.head)][head] = true;
        }
        if !o.kinds.iter().any(may) {
            return None;
        }
        // Each table's own facts as the program reads them now: facts not
        // named, of probability strictly between 0 and 1 or not, read only
        // by rules of the table.
        o.named = vec![false; n];
        for a in gp.named_atoms() {
            o.named[a as usize] = true;
        }
        // A row that changes keeps a head and loses one, each on a chain of
        // the table's own facts that excludes the other's: a table none of
        // whose rules reads one of its own facts has no such row. First the
        // tables none of whose rules reads a fact not named, before the
        // facts' readers are known.
        for r in &gp.rules {
            let e = &mut o.kinds[tables.of(r.head)];
            if may(e)
                && !e[READS_FACT]
                && r.body
                    .iter()
                    .any(|l| gp.is_fact(l.atom()) && !o.named[l.atom() as usize])
            {
                e[READS_FACT] = true;
            }
        }
        if !o.kinds.iter().any(|e| may(e) && e[READS_FACT]) {
            return None;
        }
        o.reader = vec![NONE; n];
        for r in &gp.rules {
            let t = tables.of(r.head);
            for l in &r.body {
                let x = &mut o.reader[l.atom() as usize];
                *x = if *x == NONE || *x == t { t } else { MANY };
            }
        }
        // Then the tables none of whose rules reads one of its own facts.
        // One whose heads not false are true changes only where its
        // contexts exclude each other and read no set in binary (below):
        // each literal of a context positive, of a set not read in binary.
        for r in &gp.rules {
            let t = tables.of(r.head);
            let e = o.kinds[t];
            if !may(&e) || !e[READS_FACT] {
                continue;
            }
            let (mut own, mut fixed) = (false, false);
            for l in &r.body {
                if o.own(l.atom(), t) {
                    own = true;
                } else if !e[OPEN]
                    && (!l.is_positive() || tables.of(l.atom()) >= num_sets || o.wide(l))
                {
                    fixed = true;
                }
            }
            let e = &mut o.kinds[t];
            e[READS_OWN] |= own;
            if fixed {
                e[FALSE] = false;
            }
        }
        Some(o)
    }

    /// Whether `h` is a derived atom the conditioning made false.
    fn false_head(&self, h: AtomId) -> bool {
        self.gp.kinds[h as usize] == AtomKind::Derived && self.val[h as usize] == Val::False
    }

    /// Whether `a` is one of table `t`'s own facts: a fact not named that
    /// only rules of the table read.
    fn own(&self, a: AtomId, t: usize) -> bool {
        self.gp.is_fact(a) && !self.named[a as usize] && self.reader[a as usize] == t
    }

    /// Whether `l` reads a set read in binary.
    fn wide(&self, l: &GLit) -> bool {
        let k = self.tables.of(l.atom());
        k < self.tables.sets().len() && self.tables.sets()[k].len() >= BITS_FROM
    }

    /// Whether table `t`'s rows may change.
    fn live(&self, t: usize) -> bool {
        may(&self.kinds[t]) && self.kinds[t][READS_OWN]
    }

    /// The weight of `l`, a literal of an own fact.
    fn weight_of(&self, l: GLit) -> Prob {
        match self.gp.kinds[l.atom() as usize] {
            AtomKind::Fact(p) => p.literal(l.is_positive()),
            AtomKind::Derived => unreachable!("an own fact"),
        }
    }

    /// The rows of the tables whose rows may change, by table and context:
    /// the rows, and each table's.
    fn rows(&self) -> (Vec<Row>, FxHashMap<usize, Vec<usize>>) {
        let mut rows: Vec<Row> = Vec::new();
        let mut row_of: FxHashMap<(usize, Vec<GLit>), usize> = FxHashMap::default();
        let mut table_rows: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
        for (ri, r) in self.gp.rules.iter().enumerate() {
            let t = self.tables.of(r.head);
            if !self.live(t) {
                continue;
            }
            let mut context: Vec<GLit> = r
                .body
                .iter()
                .copied()
                .filter(|l| !self.own(l.atom(), t))
                .collect();
            context.sort_unstable();
            context.dedup();
            let next = rows.len();
            let id = *row_of.entry((t, context.clone())).or_insert(next);
            if id == next {
                rows.push(Row {
                    context,
                    rules: Vec::new(),
                });
                table_rows.entry(t).or_default().push(id);
            }
            rows[id].rules.push(ri);
        }
        (rows, table_rows)
    }

    /// Whether the contexts of the rows `ids` of a table exclude each other
    /// pairwise: each reads one atom of each of the same exclusive sets,
    /// positively, and nothing else.
    fn aligned(&self, rows: &[Row], ids: &[usize]) -> bool {
        let num_sets = self.tables.sets().len();
        let sets_of = |c: &[GLit]| -> Option<Vec<usize>> {
            let mut s: Vec<usize> = Vec::with_capacity(c.len());
            for l in c {
                let k = self.tables.of(l.atom());
                if !l.is_positive() || k >= num_sets {
                    return None;
                }
                s.push(k);
            }
            s.sort_unstable();
            (s.windows(2).all(|p| p[0] != p[1])).then_some(s)
        };
        match ids.first().map(|&i| sets_of(&rows[i].context)) {
            Some(Some(first)) if !first.is_empty() => ids[1..]
                .iter()
                .all(|&i| sets_of(&rows[i].context).as_ref() == Some(&first)),
            _ => false,
        }
    }

    /// Each own fact's row, `usize::MAX` for one of two rows or more, to
    /// tell the switches read by one row alone.
    fn row_of_fact(
        &self,
        rows: &[Row],
        table_rows: &FxHashMap<usize, Vec<usize>>,
    ) -> FxHashMap<AtomId, usize> {
        let mut row_of_fact: FxHashMap<AtomId, usize> = FxHashMap::default();
        for (&t, ids) in table_rows {
            for &id in ids {
                for &ri in &rows[id].rules {
                    for l in &self.gp.rules[ri].body {
                        if self.own(l.atom(), t) {
                            let e = row_of_fact.entry(l.atom()).or_insert(id);
                            if *e != id {
                                *e = usize::MAX;
                            }
                        }
                    }
                }
            }
        }
        row_of_fact
    }

    /// What row `id` of table `t` weighs, where it is renormalized: its
    /// switches are its own, or the table's contexts exclude each other
    /// (`shared`); its chains exclude each other; it loses mass to its
    /// false heads and keeps some; and its mass of no head is known.
    fn weigh(
        &self,
        row: &Row,
        id: usize,
        t: usize,
        shared: bool,
        row_of_fact: &FxHashMap<AtomId, usize>,
    ) -> Option<Weighed> {
        // The chains, their weights and heads.
        let chain = |ri: usize| -> Vec<GLit> {
            let mut c: Vec<GLit> = self.gp.rules[ri]
                .body
                .iter()
                .copied()
                .filter(|l| self.own(l.atom(), t))
                .collect();
            c.sort_unstable();
            c
        };
        let chains: Vec<Vec<GLit>> = row.rules.iter().map(|&ri| chain(ri)).collect();
        if !shared
            && chains
                .iter()
                .flatten()
                .any(|l| row_of_fact.get(&l.atom()) != Some(&id))
        {
            return None;
        }
        let clash = |x: &[GLit], y: &[GLit]| x.iter().any(|&l| y.contains(&l.negated()));
        if (0..chains.len()).any(|i| (i + 1..chains.len()).any(|j| !clash(&chains[i], &chains[j])))
        {
            return None;
        }
        let mut heads: Vec<(AtomId, f64)> = Vec::new();
        for (k, &ri) in row.rules.iter().enumerate() {
            let theta = Prob::cube(chains[k].iter().map(|&l| self.weight_of(l))).p;
            let h = self.gp.rules[ri].head;
            match heads.iter_mut().find(|(x, _)| *x == h) {
                Some((_, w)) => *w += theta,
                None => heads.push((h, theta)),
            }
        }
        let total: f64 = heads.iter().map(|&(_, w)| w).sum();
        let lost: f64 = heads
            .iter()
            .filter(|&&(h, _)| self.false_head(h))
            .map(|&(_, w)| w)
            .sum();
        let kept: Vec<(AtomId, f64)> = heads
            .iter()
            .copied()
            .filter(|&(h, w)| !self.false_head(h) && w > 0.0)
            .collect();
        if total > 1.0 + 1e-9 || lost <= 1e-12 || kept.is_empty() {
            return None;
        }
        // The mass of no head, from the chains' weights, not as `1 -
        // total`, which keeps the rounding of the heads' masses.
        let none = no_head(&chains, &|l| self.weight_of(l))?;
        let m: f64 = kept.iter().map(|&(_, w)| w).sum::<f64>() + none;
        if m >= 1.0 - 1e-12 || m <= 0.0 {
            return None;
        }
        Some(Weighed {
            heads,
            lost,
            kept,
            none,
            m,
        })
    }
}

/// A row of a table: its context, the body literals that are not the
/// table's own facts, and its rules.
struct Row {
    context: Vec<GLit>,
    rules: Vec<usize>,
}

/// What a row renormalized weighs ([`Owners::weigh`]): `θ_h` for each head
/// `h`, the mass its false heads lose, the heads it keeps with their `θ`,
/// the mass of no head, and `m`, the mass it keeps.
struct Weighed {
    heads: Vec<(AtomId, f64)>,
    lost: f64,
    kept: Vec<(AtomId, f64)>,
    none: f64,
    m: f64,
}

/// The atoms the rows of one table share: their switches, by weight and
/// the switch's place among those of that weight; the facts of their
/// masses and of their true heads' weights, by weight; and the head of
/// their masses' rules, `AtomId::MAX` until a row makes it.
struct TableAtoms {
    switches: FxHashMap<((u64, u64), usize), AtomId>,
    masses: FxHashMap<(u64, u64), AtomId>,
    known: FxHashMap<(u64, u64), AtomId>,
    bad: AtomId,
}

/// The rewrite of the rows renormalized: their rules dropped, the new
/// rules emitted at each row's first rule, the atoms made after the
/// program's `n`, the atoms observed false, and the factor of the rows
/// without a context.
struct Rewrite {
    n: usize,
    new_atoms: Vec<(Atom, AtomKind)>,
    emitted: FxHashMap<usize, Vec<GroundRule>>,
    dropped: Vec<bool>,
    evidence: Vec<AtomId>,
    factor: Count,
    renormalized: usize,
}

impl Rewrite {
    fn new(gp: &GroundProgram) -> Rewrite {
        Rewrite {
            n: gp.num_atoms(),
            new_atoms: Vec::new(),
            emitted: FxHashMap::default(),
            dropped: vec![false; gp.rules.len()],
            evidence: Vec::new(),
            factor: Count::ONE,
            renormalized: 0,
        }
    }

    /// A new atom of `kind`, named `name` with its number.
    fn fresh(&mut self, name: &str, kind: AtomKind) -> AtomId {
        let a = (self.n + self.new_atoms.len()) as AtomId;
        self.new_atoms
            .push((Atom::new(name, vec![Term::Int(a as i64)]), kind));
        a
    }

    /// The fact of weight `p` that `made` holds under `key` where `reuse`,
    /// else a new one named `name`, which `made` then holds.
    fn fact<K: Eq + std::hash::Hash>(
        &mut self,
        made: &mut FxHashMap<K, AtomId>,
        key: K,
        reuse: bool,
        name: &str,
        p: Prob,
    ) -> AtomId {
        match made.get(&key) {
            Some(&a) if reuse => a,
            _ => {
                let a = self.fresh(name, AtomKind::Fact(p));
                made.insert(key, a);
                a
            }
        }
    }

    /// The first of `rules`, a row's, where its new rules go, the rules
    /// dropped.
    fn drop_row(&mut self, rules: &[usize]) -> usize {
        self.renormalized += 1;
        let first = rules.iter().copied().min().expect("a row has rules");
        for &ri in rules {
            self.dropped[ri] = true;
        }
        first
    }

    /// The rows `ids` of table `t` renormalized where they can be.
    fn table(
        &mut self,
        o: &Owners,
        rows: &[Row],
        ids: &[usize],
        t: usize,
        row_of_fact: &FxHashMap<AtomId, usize>,
    ) {
        let shared = o.aligned(rows, ids);
        // The masses first in each row's chain and on a false head of the
        // table where a context reads a set read in binary.
        let mass_first = ids
            .iter()
            .any(|&i| rows[i].context.iter().any(|l| o.wide(l)));
        let mut atoms = TableAtoms {
            switches: FxHashMap::default(),
            masses: FxHashMap::default(),
            known: FxHashMap::default(),
            bad: AtomId::MAX,
        };
        for &id in ids {
            let row = &rows[id];
            let Some(weighed) = o.weigh(row, id, t, shared, row_of_fact) else {
                continue;
            };
            // A row that keeps a head the conditioning made true: in a table
            // whose contexts exclude each other and read no set in binary,
            // `h :- context, t` with `t` of weight `θ_h`; elsewhere as it
            // is.
            if weighed
                .kept
                .iter()
                .any(|&(h, _)| o.val[h as usize] == Val::True)
            {
                if shared && !mass_first && weighed.kept.len() == 1 {
                    self.true_head(row, &weighed, &mut atoms.known);
                }
                continue;
            }
            self.renormalized_row(o, row, &weighed, shared, mass_first, &mut atoms);
        }
    }

    /// `row`, which keeps one head `h` the conditioning made true, as `h
    /// :- context, t`, `t` a fact of weight `θ_h` that the rows of one
    /// `θ_h` share (`known`).
    fn true_head(
        &mut self,
        row: &Row,
        weighed: &Weighed,
        known: &mut FxHashMap<(u64, u64), AtomId>,
    ) {
        let first = self.drop_row(&row.rules);
        let (h, theta) = weighed.kept[0];
        // Its weight false is the mass of the row's other heads and of
        // none, each its own number.
        let theta = Prob {
            p: theta,
            not: weighed.lost + weighed.none,
        };
        let t = self.fact(known, theta.bits(), true, "$rn_t", theta);
        let mut body = row.context.clone();
        body.push(GLit::Pos(t));
        self.emitted
            .entry(first)
            .or_default()
            .push(GroundRule::new(h, body));
    }

    /// `row` as a chain over the heads it keeps, and its mass on a fact `w`
    /// the context requires (none where the row has no context: a
    /// constant). With `mass_first`, `w` heads the kept heads' chain too,
    /// the row's own first switch.
    fn renormalized_row(
        &mut self,
        o: &Owners,
        row: &Row,
        weighed: &Weighed,
        shared: bool,
        mass_first: bool,
        atoms: &mut TableAtoms,
    ) {
        let Row { context, rules } = row;
        let first = self.drop_row(rules);
        // Its weight false is the mass lost, its own number.
        let mw = Prob {
            p: weighed.m,
            not: weighed.lost,
        };
        let first_w = (mass_first && !context.is_empty())
            .then(|| self.fact(&mut atoms.masses, mw.bits(), shared, "$rn_w", mw));
        let mut out = Vec::new();
        let full = weighed.none == 0.0;
        let kept = &weighed.kept;
        let kept_masses: Vec<f64> = kept.iter().map(|&(_, w)| w).collect();
        let mut qs: Vec<Prob> = Vec::with_capacity(kept.len());
        chain_weights(&kept_masses, weighed.none, &mut qs);
        let mut negs: Vec<GLit> = Vec::new();
        let mut seen: FxHashMap<(u64, u64), usize> = FxHashMap::default();
        for (i, (&(h, _), &q)) in kept.iter().zip(&qs).enumerate() {
            let mut body = context.clone();
            if let Some(w) = first_w {
                body.push(GLit::Pos(w));
            }
            body.extend_from_slice(&negs);
            if i + 1 == kept.len() && full {
                out.push(GroundRule::new(h, body));
                break;
            }
            let k = seen.entry(q.bits()).or_insert(0);
            let key = (q.bits(), *k);
            *k += 1;
            let s = self.fact(&mut atoms.switches, key, shared, "$rn_s", q);
            body.push(GLit::Pos(s));
            out.push(GroundRule::new(h, body));
            negs.push(GLit::Neg(s));
        }
        if context.is_empty() {
            self.factor = (self.factor * Count::new(weighed.m)).normalized();
        } else {
            let w = first_w
                .unwrap_or_else(|| self.fact(&mut atoms.masses, mw.bits(), shared, "$rn_w", mw));
            if atoms.bad == AtomId::MAX {
                atoms.bad = if mass_first {
                    // The row's mass lost goes to a head of the table the
                    // conditioning made false, one for the table: the
                    // row's false heads merged into one, first in its
                    // chain, in the table's own unit.
                    let bad = weighed
                        .heads
                        .iter()
                        .map(|&(h, _)| h)
                        .find(|&h| o.false_head(h))
                        .expect("a row that lost mass has a false head");
                    if !o.gp.evidence.iter().any(|&(_, a, v)| a == Some(bad) && !v) {
                        self.evidence.push(bad);
                    }
                    bad
                } else {
                    let bad = self.fresh("$rn_bad", AtomKind::Derived);
                    self.evidence.push(bad);
                    bad
                };
            }
            let mut body = context.clone();
            body.push(GLit::Neg(w));
            out.push(GroundRule::new(atoms.bad, body));
        }
        self.emitted.entry(first).or_default().extend(out);
    }

    /// `gp` with the rewrite: the new atoms, each row's new rules in place
    /// of its rules, and the atoms observed false; the rows renormalized
    /// and the factor.
    fn apply(self, gp: &mut GroundProgram) -> (usize, Count) {
        let Rewrite {
            new_atoms,
            mut emitted,
            dropped,
            evidence,
            factor,
            renormalized,
            ..
        } = self;
        if renormalized == 0 {
            return (0, Count::ONE);
        }
        for (atom, kind) in new_atoms {
            gp.atoms.push(atom);
            gp.kinds.push(kind);
        }
        let old = std::mem::take(&mut gp.rules);
        let mut rules = Vec::with_capacity(old.len());
        for (ri, r) in old.into_iter().enumerate() {
            if let Some(new) = emitted.remove(&ri) {
                rules.extend(new);
            }
            if !dropped[ri] {
                rules.push(r);
            }
        }
        gp.rules = rules;
        for bad in evidence {
            gp.evidence
                .push((gp.atoms[bad as usize].clone(), Some(bad), false));
        }
        (renormalized, factor)
    }
}
