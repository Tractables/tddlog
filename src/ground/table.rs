//! Table-shaped programs: a Bayesian network written as one annotated
//! disjunction per row of a table (`scripts/gen_bn.py`),
//!
//! ```text
//! p1::x(v1); ...; pk::x(vk) :- y1(u1), ..., ym(um).
//! x(v) :- y1(u1), ..., ym(um).
//! ```
//!
//! every atom ground, of any arity, plus ground `query/1` and
//! `evidence/1,2` facts. Such a program is ground as written, and what the
//! generic steps compute of it is known from the rows:
//! [`super::normalize`] gives each row's heads a chain of switches, and the
//! bottom-up grounder ([`super::engine`]) evaluates the tables in the order
//! its stratification visits them, each row's rules in turn. [`ground_rows`]
//! builds the grounder's output directly: the same atoms in the same order,
//! the same rules with the same bodies, the same queries and evidence. It
//! skips the normalized program, the grounder's rule compilation and body
//! scheduling, and its relations, which on the networks cost an allocation
//! per literal of every normalized rule (a row of k values names k(k+1)/2
//! switch literals).
//!
//! A program of any other shape, or one whose tables read each other in a
//! cycle, or a clause whose probabilities sum past one, is left to the
//! generic steps, decided before anything is built. Equality with the
//! generic grounder's output is tested on every table-shaped example
//! (`tests` below). [`super::noisy_or`] reads the tables of the
//! same [`Shape`], and the shape of the program it writes is made from
//! this one ([`Shape::rewritten`]).

use rustc_hash::FxHashMap;

use super::normalize::{head_total, switch_name, switch_probabilities};
use super::program::{KeptRules, Lists};
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, Prob};
use crate::syntax::{Atom, Clause, Literal, Program, Term, same_name};

/// The grounder's output for `program` where it is table-shaped (above),
/// before simplification; `None` where it is not.
#[cfg(test)]
pub(super) fn ground_rows(program: &Program) -> Option<GroundProgram> {
    let shape = Shape::of(program)?;
    let order = shape.evaluation_order()?;
    Some(shape.ground(program, &order))
}

/// A clause of a table, and the number of the source clause it is, which
/// names its switches.
#[derive(Clone, Copy)]
pub(super) struct Switched<'a> {
    pub(super) clause: &'a Clause,
    group: u32,
}

/// Whether `clause` is a row of a table: probabilities that sum to one at
/// most ([`head_total`]), or one head and none.
fn row(clause: &Clause) -> bool {
    if probabilistic(clause) {
        head_total(&clause.heads).is_ok()
    } else {
        clause.heads.len() == 1
    }
}

/// Whether `clause`'s heads have switches.
fn probabilistic(clause: &Clause) -> bool {
    clause.heads.iter().any(|h| h.prob.is_some())
}

/// A table-shaped program's predicates, by name and arity, and the clauses
/// of each table.
#[derive(Default)]
pub(super) struct Shape<'a> {
    /// The predicates in the order the grounder numbers them: a clause's
    /// head, then its body's atoms, clause by clause.
    names: Vec<&'a str>,
    index: FxHashMap<(&'a str, usize), usize>,
    /// The predicates of [`Shape::index`] by the address of their names'
    /// text and their arity. The parser shares one text among the atoms
    /// that spell a name, so an atom finds its predicate here without its
    /// name hashed and compared.
    by_address: FxHashMap<(usize, usize), usize>,
    /// The clauses of each predicate, in program order ([`Shape::clauses`]).
    /// The switch probabilities of a clause's heads are worked out where it
    /// is ground, not held per clause: a program may show itself no table
    /// only at its last clause, after a tuple each (TPC-H's q1).
    clauses: Lists<Switched<'a>>,
    /// The predicates the clauses of each predicate read, in the order they
    /// read them ([`Shape::reads`]).
    reads: Lists<usize>,
}

impl<'a> Shape<'a> {
    /// The shape of `program`, `None` where it is not table-shaped.
    pub(super) fn of(program: &'a Program) -> Option<Self> {
        if !program.query_rules.is_empty() {
            return None;
        }
        // Queries and evidence on ground atoms; a pattern asks the grounder
        // for its instances.
        if program
            .queries
            .iter()
            .chain(&program.evidence)
            .any(|(a, _)| !a.is_ground())
        {
            return None;
        }
        let mut shape = Shape::default();
        let mut rows = Vec::with_capacity(program.clauses.len());
        let mut reads = Vec::new();
        let mut last_group = None;
        // The predicate at each place of the clause before ([`Shape::at`]).
        let mut last = Vec::new();
        for clause in &program.clauses {
            // One source clause each: clauses split from a body disjunction
            // share their switches.
            if last_group.is_some_and(|g| g >= clause.group) {
                return None;
            }
            last_group = Some(clause.group);
            let first = &clause.heads.first()?.atom;
            // A name of the grounder's own, as a switch's, is no table's.
            if first.name.starts_with('$') {
                return None;
            }
            for h in &clause.heads {
                if h.negated
                    || !same_name(&h.atom.name, &first.name)
                    || h.atom.args.len() != first.args.len()
                    || !h.atom.is_ground()
                {
                    return None;
                }
            }
            if !row(clause) {
                return None;
            }
            let x = shape.at(&mut last, 0, &clause.heads[0].atom);
            rows.push((
                x,
                Switched {
                    clause,
                    group: clause.group,
                },
            ));
            for (k, l) in clause.body.iter().enumerate() {
                let Literal::Pos(a) = l else { return None };
                if !a.is_ground() {
                    return None;
                }
                reads.push(shape.at(&mut last, k + 1, a));
            }
        }
        shape.lay_out(&rows, &reads);
        Some(shape)
    }

    /// How many predicates the program names.
    pub(super) fn predicates(&self) -> usize {
        self.names.len()
    }

    /// The clauses whose heads are the predicate `x`, in program order.
    pub(super) fn clauses(&self, x: usize) -> &[Switched<'a>] {
        &self.clauses[x]
    }

    /// The predicates that the clauses of `x` read, in the order they read
    /// them: each clause's body's, clause by clause.
    pub(super) fn reads(&self, x: usize) -> &[usize] {
        &self.reads[x]
    }

    /// The clauses `rows`, in program order each with its predicate, and
    /// what they read, `reads`, their bodies' predicates clause by clause,
    /// laid out predicate by predicate ([`Shape::clauses`]).
    fn lay_out(&mut self, rows: &[(usize, Switched<'a>)], reads: &[usize]) {
        let n = self.names.len();
        self.clauses = Lists::new(n, || rows.iter().copied());
        self.reads = Lists::new(n, || {
            rows.iter()
                .scan(0, |from, &(x, s)| {
                    let read = &reads[*from..*from + s.clause.body.len()];
                    *from += read.len();
                    Some(read.iter().map(move |&p| (x, p)))
                })
                .flatten()
        });
    }

    /// The predicate of `atom`, where the program names it: first by the
    /// address of its name's text ([`Shape::by_address`]).
    pub(super) fn predicate_of(&self, atom: &Atom) -> Option<usize> {
        let address = (atom.name.as_ptr() as usize, atom.args.len());
        self.by_address
            .get(&address)
            .or_else(|| self.index.get(&(&*atom.name, atom.args.len())))
            .copied()
    }

    /// [`Shape::predicate`] of `atom`, at place `k` of its clause (the
    /// head's 0, then its body's), where `last` holds the predicate at each
    /// place of the clause before and its key in [`Shape::by_address`]. The
    /// rows of a table name the same predicates at the same places, three
    /// of four atoms of blockmap_22_03 (UAI 2022): such an atom's is the one
    /// before, not looked up.
    fn at(&mut self, last: &mut Vec<((usize, usize), usize)>, k: usize, atom: &'a Atom) -> usize {
        let address = (atom.name.as_ptr() as usize, atom.args.len());
        match last.get_mut(k) {
            Some((before, p)) if *before == address => *p,
            Some(place) => {
                let p = self.predicate(atom);
                *place = (address, p);
                p
            }
            None => {
                let p = self.predicate(atom);
                last.push((address, p));
                p
            }
        }
    }

    /// The predicate of `atom`, numbered next where it is new.
    fn predicate(&mut self, atom: &'a Atom) -> usize {
        let address = (atom.name.as_ptr() as usize, atom.args.len());
        if let Some(&p) = self.by_address.get(&address) {
            return p;
        }
        let next = self.names.len();
        let p = *self
            .index
            .entry((&*atom.name, atom.args.len()))
            .or_insert(next);
        if p == next {
            self.names.push(&atom.name);
        }
        self.by_address.insert(address, p);
        p
    }

    /// The shape of `program` with each table that `written` holds clauses
    /// for written as them ([`super::noisy_or`]): its clauses, each with the
    /// table its body reads (`None` for none), in place of its first clause
    /// and the rest of its clauses left out. It is the shape [`Shape::of`]
    /// gives the program so written, its clauses numbered in order and its
    /// predicates where each comes first, made from this one: the clauses
    /// kept are not copied, nor are their atoms looked up again, as they
    /// would be written as a program and shaped anew.
    pub(super) fn rewritten<'b>(
        self,
        program: &'b Program,
        written: &'b [Option<super::noisy_or::Written>],
    ) -> Shape<'b>
    where
        'a: 'b,
    {
        const NONE: usize = usize::MAX;
        let n = self.names.len();
        let mut shape = Shape::default();
        // Each predicate's number in `shape`, given where it comes first.
        let mut renumbered = vec![NONE; n];
        let names = &self.names;
        let mut number = |shape: &mut Shape<'b>, p: usize| {
            if renumbered[p] == NONE {
                renumbered[p] = shape.names.len();
                shape.names.push(names[p]);
            }
            renumbered[p]
        };
        let mut rows = Vec::with_capacity(program.clauses.len());
        let mut reads = Vec::new();
        // Per table, how many of its clauses kept come before, and how many
        // of its reads they have read; and whether its clauses written are.
        let mut kept = vec![(0, 0); n];
        let mut done = vec![false; n];
        let mut group = 0;
        for clause in &program.clauses {
            let x = self
                .predicate_of(&clause.heads[0].atom)
                .expect("a table-shaped program's clause heads a table");
            match &written[x] {
                None => {
                    let (c, r) = &mut kept[x];
                    let s = self.clauses(x)[*c];
                    let y = number(&mut shape, x);
                    rows.push((y, Switched { group, ..s }));
                    group += 1;
                    for &p in &self.reads(x)[*r..*r + clause.body.len()] {
                        reads.push(number(&mut shape, p));
                    }
                    *c += 1;
                    *r += clause.body.len();
                }
                Some(clauses) => {
                    if std::mem::replace(&mut done[x], true) {
                        continue;
                    }
                    for (clause, parent) in clauses {
                        let y = number(&mut shape, x);
                        rows.push((y, Switched { clause, group }));
                        group += 1;
                        if let Some(p) = *parent {
                            reads.push(number(&mut shape, p));
                        }
                    }
                }
            }
        }
        shape.lay_out(&rows, &reads);
        shape.index = self
            .index
            .iter()
            .filter(|&(_, &p)| renumbered[p] != NONE)
            .map(|(&key, &p)| (key, renumbered[p]))
            .collect();
        shape.by_address = self
            .by_address
            .iter()
            .filter(|&(_, &p)| renumbered[p] != NONE)
            .map(|(&key, &p)| (key, renumbered[p]))
            .collect();
        shape
    }

    /// The tables in the order the grounder evaluates them: the post-order
    /// of a depth-first search over what each reads, from each predicate in
    /// turn, which is the order its stratification emits the components;
    /// `None` where the tables read each other in a cycle.
    pub(super) fn evaluation_order(&self) -> Option<Vec<usize>> {
        const NEW: u8 = 0;
        const OPEN: u8 = 1;
        const DONE: u8 = 2;
        let n = self.names.len();
        let mut state = vec![NEW; n];
        let mut order = Vec::with_capacity(n);
        // Each predicate open with the reads it has left.
        let mut stack: Vec<(usize, std::slice::Iter<'_, usize>)> = Vec::new();
        for root in 0..n {
            if state[root] != NEW {
                continue;
            }
            state[root] = OPEN;
            stack.push((root, self.reads(root).iter()));
            while let Some((p, next)) = stack.last_mut() {
                match next.next() {
                    Some(&q) => match state[q] {
                        NEW => {
                            state[q] = OPEN;
                            stack.push((q, self.reads(q).iter()));
                        }
                        OPEN => return None,
                        _ => {}
                    },
                    None => {
                        state[*p] = DONE;
                        if !self.clauses(*p).is_empty() {
                            order.push(*p);
                        }
                        stack.pop();
                    }
                }
            }
        }
        Some(order)
    }

    /// The grounder's output: each table's rules in the order of its
    /// clauses and heads, the atoms numbered as the grounder meets them (a
    /// rule's switch, the guards' switches, the parents, the head), a rule
    /// whose parent is impossible dropped after its switch is numbered.
    pub(super) fn ground(&self, program: &Program, order: &[usize]) -> GroundProgram {
        let mut gp = GroundProgram::default();
        // The atoms and the rules, allocated for what the clauses can give:
        // a switch, a derived atom and a rule per head at most, and a rule
        // kept per deterministic clause.
        let (mut heads, mut deterministic) = (0, 0);
        for s in order.iter().flat_map(|&x| self.clauses(x)) {
            heads += s.clause.heads.len();
            deterministic += usize::from(!probabilistic(s.clause));
        }
        gp.atoms.reserve_exact(2 * heads);
        gp.kinds.reserve_exact(2 * heads);
        gp.rules.reserve_exact(heads);
        // The derived atoms by predicate and arguments, in one map rather
        // than one per predicate, which a network of many small tables
        // would allocate and grow one by one.
        let mut ids: FxHashMap<(usize, &'a [Term]), AtomId> = FxHashMap::default();
        ids.reserve(heads);
        // Deterministic rules can repeat, where a rule with a switch cannot;
        // the grounder keeps one of each.
        let mut kept = KeptRules::with_capacity(deterministic);
        let mut parents: Vec<GLit> = Vec::new();
        let mut switches: Vec<AtomId> = Vec::new();
        let mut qs: Vec<Prob> = Vec::new();
        let mut name = String::new();
        for &x in order {
            // The predicates of each clause's body are the next of the
            // table's reads, not looked up by name again.
            let mut reads = self.reads(x);
            for &Switched { clause, group } in self.clauses(x) {
                parents.clear();
                let (ys, rest) = reads.split_at(clause.body.len());
                reads = rest;
                let possible = clause.body.iter().zip(ys).all(|(l, &y)| {
                    let Literal::Pos(a) = l else {
                        unreachable!("a row's body is positive")
                    };
                    let id = ids.get(&(y, a.args.as_slice()));
                    parents.extend(id.map(|&b| GLit::Pos(b)));
                    id.is_some()
                });
                if !probabilistic(clause) {
                    if possible {
                        let h = number(&mut gp, &mut ids, x, &clause.heads[0].atom);
                        let mut body = parents.clone();
                        body.sort_unstable();
                        body.dedup();
                        let rule = GroundRule::new(h, body);
                        if kept.keep(&gp.rules, &rule) {
                            gp.rules.push(rule);
                        }
                    }
                    continue;
                }
                switch_probabilities(&clause.heads, &mut qs)
                    .expect("a row's probabilities sum to one at most");
                switches.clear();
                for (i, (h, &q)) in clause.heads.iter().zip(&qs).enumerate() {
                    let switch = gp.atoms.len() as AtomId;
                    gp.atoms.push(Atom::new(
                        switch_name(&mut name, self.names[x], group, i),
                        Vec::new(),
                    ));
                    gp.kinds.push(AtomKind::Fact(q));
                    switches.push(switch);
                    if !possible {
                        continue;
                    }
                    let h = number(&mut gp, &mut ids, x, &h.atom);
                    // The head's switch, and the switches of the heads before
                    // it negated.
                    let mut body = Vec::with_capacity(parents.len() + i + 1);
                    body.push(GLit::Pos(switch));
                    body.extend(switches[..i].iter().map(|&s| GLit::Neg(s)));
                    body.extend_from_slice(&parents);
                    body.sort_unstable();
                    body.dedup();
                    gp.rules.push(GroundRule::new(h, body));
                }
            }
        }
        let lookup = |a: &Atom| {
            self.predicate_of(a)
                .and_then(|y| ids.get(&(y, a.args.as_slice())))
                .copied()
        };
        gp.queries = program
            .queries
            .iter()
            .map(|(a, positive)| (a.clone(), lookup(a), *positive))
            .collect();
        gp.evidence = program
            .evidence
            .iter()
            .map(|(a, value)| (a.clone(), lookup(a), *value))
            .collect();
        gp
    }
}

/// The number of the head `atom` of the table `x`, whose atoms are
/// numbered in `ids`, numbered next where it is new.
fn number<'a>(
    gp: &mut GroundProgram,
    ids: &mut FxHashMap<(usize, &'a [Term]), AtomId>,
    x: usize,
    atom: &'a Atom,
) -> AtomId {
    let next = gp.atoms.len() as AtomId;
    let id = *ids.entry((x, atom.args.as_slice())).or_insert(next);
    if id == next {
        gp.atoms.push(atom.clone());
        gp.kinds.push(AtomKind::Derived);
    }
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::{engine, normalize};

    /// The table path's output for `source` where it is table-shaped, with
    /// the generic grounder's, which must be the same.
    fn both(source: &str) -> Option<(GroundProgram, GroundProgram)> {
        let program = crate::syntax::parse_program(source).expect("parses");
        let rows = ground_rows(&program)?;
        let norm = normalize::normalize(&program).expect("normalizes");
        Some((
            engine::ground_normalized(&norm, None).expect("grounds"),
            rows,
        ))
    }

    fn assert_same(what: &str, generic: &GroundProgram, rows: &GroundProgram) {
        assert_eq!(generic.atoms, rows.atoms, "{what}: atoms");
        assert_eq!(generic.kinds, rows.kinds, "{what}: kinds");
        assert_eq!(generic.rules, rows.rules, "{what}: rules");
        assert_eq!(generic.queries, rows.queries, "{what}: queries");
        assert_eq!(generic.evidence, rows.evidence, "{what}: evidence");
        assert!(rows.encoding.is_empty() && rows.stats.from_bound_end.is_empty());
    }

    fn table_shaped(source: &str) {
        let (generic, rows) = both(source).expect("table-shaped");
        assert_same(source, &generic, &rows);
    }

    fn not_table_shaped(source: &str) {
        assert!(both(source).is_none(), "{source}");
    }

    #[test]
    fn rows_ground_as_the_grounder_grounds_them() {
        // Parents after their children in the program, a row of one value
        // with its rest to none, a value of probability one, a value that
        // takes no mass, deterministic rows (one twice), a value repeated
        // in a row, integer values.
        table_shaped(
            "0.3::c(1); 0.7::c(2) :- a(x), b(t).
             0.5::c(1) :- a(y), b(t).
             1.0::c(2); 0.0::c(1) :- a(x), b(f).
             0.2::c(1); 0.2::c(1); 0.6::c(2) :- a(y), b(f).
             0.6::b(t); 0.4::b(f).
             0.4::a(x); 0.6::a(y).
             d(yes) :- c(1). d(no) :- c(2). d(no) :- c(2).
             query(d(yes)). query(\\+c(1)). evidence(b(t)). evidence(a(y), false).",
        );
    }

    #[test]
    fn impossible_parents_and_queries_ground_alike() {
        // b(z) has no row and e no table, so rows that read them are
        // dropped after their switches are numbered; queries and evidence
        // on atoms that do not exist or are of no table.
        table_shaped(
            "0.6::b(x); 0.4::b(y).
             0.1::c(p); 0.9::c(q) :- b(x).
             0.2::c(p); 0.8::c(q) :- b(z).
             0.5::c(p); 0.5::c(q) :- e(w).
             0.3::f(1); 0.7::f(2) :- c(p), b(y).
             query(c(p)). query(c(r)). query(g(1)). query(h). evidence(f(2), true).",
        );
    }

    #[test]
    fn atoms_of_other_arities_ground_alike() {
        // Facts of no argument, a predicate of two, and a name with two
        // arities.
        table_shaped(
            "0.5::a. 0.4::a(1).
             0.3::b(t, 1) :- a. 0.6::b(t, 1); 0.4::b(f, 2) :- a(1).
             w :- b(t, 1), a. 0.2::w :- b(f, 2).
             query(w). query(b(f, 2)). evidence(a).",
        );
    }

    #[test]
    fn names_written_apart_ground_as_names_shared() {
        // The parser shares one text among the atoms that spell a name; a
        // program whose atoms each have a text of their own grounds alike.
        let source =
            "0.6::b(x); 0.4::b(y).\n0.1::c(p); 0.9::c(q) :- b(x).\nd :- c(p), b(y).\nquery(d).\n";
        let shared = crate::syntax::parse_program(source).expect("parses");
        let mut apart = shared.clone();
        for clause in &mut apart.clauses {
            let atoms =
                clause
                    .heads
                    .iter_mut()
                    .map(|h| &mut h.atom)
                    .chain(clause.body.iter_mut().map(|l| match l {
                        Literal::Pos(a) | Literal::Neg(a) => a,
                        _ => unreachable!("a row's body is atoms"),
                    }));
            for a in atoms {
                a.name = crate::syntax::Name::from(&*a.name);
            }
        }
        let same_text = |p: &Program| {
            let Literal::Pos(read) = &p.clauses[1].body[0] else {
                unreachable!("b(x)")
            };
            std::sync::Arc::ptr_eq(&p.clauses[0].heads[0].atom.name, &read.name)
        };
        assert!(same_text(&shared) && !same_text(&apart));
        assert_same(
            source,
            &ground_rows(&shared).expect("table-shaped"),
            &ground_rows(&apart).expect("table-shaped"),
        );
    }

    #[test]
    fn every_table_shaped_example_grounds_alike() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
        let mut seen = 0;
        for entry in std::fs::read_dir(dir).expect("examples") {
            let path = entry.expect("an entry").path();
            if path.extension().is_none_or(|e| e != "pl") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("reads");
            if let Some((generic, rows)) = both(&source) {
                assert_same(&path.display().to_string(), &generic, &rows);
                seen += 1;
            }
        }
        assert!(seen >= 5, "only {seen} table-shaped examples");
    }

    #[test]
    fn other_shapes_are_left_to_the_grounder() {
        // A negated body literal, a variable, two predicates in one head, a
        // cycle, a body disjunction, a query pattern.
        not_table_shaped("0.5::a(x); 0.5::a(y). 0.3::b(t) :- \\+a(x).");
        not_table_shaped("0.5::a(X) :- b(X). b(1).");
        not_table_shaped("0.5::a(x); 0.5::b(y).");
        not_table_shaped("0.5::a(x) :- b(t). 0.5::b(t) :- a(x).");
        not_table_shaped("0.5::a(x). 0.5::a(y). 0.3::b(t) :- a(x) ; a(y).");
        not_table_shaped("0.5::a(x); 0.5::a(y). query(a(_)).");
        // Probabilities past one are the grounder's to report.
        not_table_shaped("0.6::a(x); 0.6::a(y).");
    }
}
