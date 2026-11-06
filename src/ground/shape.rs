//! What shape a ground program has, which `auto` picks its trials by
//! (`auto::candidates`) and the layouts and the preprocessing read: no
//! component order to search ([`nothing_to_order`]), rules over facts alone
//! ([`reads_facts_alone`]), a CNF over the facts ([`clauses_over_facts`]),
//! and the queries of cubes of facts that no layout reads
//! ([`QueriedCubes`]).

use std::sync::OnceLock;

use super::deps::Deps;
use super::{AtomId, GroundProgram, GroundRule};
use crate::syntax::Atom;

/// Whether the program has one component, or one recursive component that
/// reads at least half the facts, so that there is no component order for
/// `vtree::order::search` to find that matters.
///
/// The search orders the components of the ground program
/// ([`super::deps::components`]), and `closed` takes them in its
/// order. A program whose derived atoms are one component admits exactly
/// one such order, so the search takes zero moves and returns the
/// compiler's own order: there it is not a search but the default, and only
/// a candidate that reorders the facts themselves can do anything. The
/// random graphs, the smokers programs and the one-atom dice programs are
/// of that kind; `dynamic` searches the vtree itself instead.
///
/// A reachability relation grounded from its source ([`crate::ground`])
/// is one recursive component over the graph's strongly connected part and
/// a few atoms for the nodes past it, which only read it. The search can
/// only place those atoms after the component, and an indicator on the
/// atoms they read cuts nothing the component's own diagram does not
/// already hold, so the program is of the same kind. `families:0`, first
/// otherwise, would spend most of such a run conjoining the queries with
/// the indicators' links.
///
/// The components come with the program's structure, which the grounder
/// leaves with it ([`GroundProgram::structure`]).
pub(crate) fn nothing_to_order(gp: &GroundProgram) -> bool {
    let structure = gp.structure();
    if structure.units == 1 {
        return true;
    }
    let Some(core) = structure
        .sccs
        .iter()
        .filter(|c| c.len() > 1)
        .max_by_key(|c| c.len())
    else {
        return false;
    };
    let mut read = vec![false; gp.num_atoms()];
    for &a in core {
        for &r in &structure.by_head[a as usize] {
            for l in &gp.rules[r].body {
                read[l.atom() as usize] = true;
            }
        }
    }
    let core_facts = gp.facts().filter(|&(f, _)| read[f as usize]).count();
    2 * core_facts >= gp.num_facts()
}

/// Whether every rule body reads facts alone, or copies of facts. Every
/// formula the compile builds is then a disjunction of conjunctions of
/// fact literals, and no derived atom's formula is composed into
/// another's, so no class of assignments at a vtree node is a function of
/// atoms decided outside it (`docs/design.md`, "Direction"), which is what
/// a bushy vtree otherwise gets wrong. A copy is an atom whose one rule
/// reads one fact, `g :- f` or `g :- \+f`, as the conditioning on the
/// evidence makes of facts it finds equal (`crate::ground::condition`): its
/// formula is a literal of the fact, and composing it adds no class.
pub(crate) fn reads_facts_alone(gp: &GroundProgram) -> bool {
    let copy = copies(gp);
    gp.rules.iter().all(|r| {
        r.body
            .iter()
            .all(|l| gp.is_fact(l.atom()) || copy[l.atom() as usize])
    })
}

/// Whether the program is a CNF over its facts: every derived atom is a
/// copy of a fact ([`reads_facts_alone`]) or a set of clauses, an atom the
/// evidence observes false that nothing reads ([`observed_false_alone`]),
/// each rule's body the negation of a clause. Its rules then read facts
/// alone, since nothing reads such an atom. A weighted CNF as a program
/// (each variable a fact, each clause j a rule `violated(j)` observed
/// false) is such a program, the copies the conditioning's. An atom
/// observed true is not a clause: a factor of a Markov network written as
/// the network reader writes one, observed true, is a table over its rows'
/// switches.
pub(crate) fn clauses_over_facts(gp: &GroundProgram) -> bool {
    let copy = copies(gp);
    let alone = observed_false_alone(gp);
    (0..gp.num_atoms()).all(|a| gp.is_fact(a as AtomId) || copy[a] || alone[a])
}

/// The copies of facts, by atom: an atom whose one rule reads one fact,
/// `g :- f` or `g :- \+f`.
fn copies(gp: &GroundProgram) -> Vec<bool> {
    gp.copies_of_facts().iter().map(Option::is_some).collect()
}

/// The derived atoms the evidence observes false that no rule reads and no
/// query asks.
pub(crate) fn observed_false_alone(gp: &GroundProgram) -> Vec<bool> {
    let mut read = vec![false; gp.num_atoms()];
    for r in &gp.rules {
        for l in &r.body {
            read[l.atom() as usize] = true;
        }
    }
    for &(_, id, _) in &gp.queries {
        if let Some(a) = id {
            read[a as usize] = true;
        }
    }
    let mut alone = vec![false; gp.num_atoms()];
    for &(_, id, value) in &gp.evidence {
        if let Some(a) = id
            && !value
            && !read[a as usize]
            && !gp.is_fact(a)
        {
            alone[a as usize] = true;
        }
    }
    alone
}

/// The derived atoms that are cubes of facts only the queries read: each
/// defined by one conjunction of fact literals, and a query or read by such
/// atoms alone. An atom is one where its one rule, which stands for one
/// body, reads two literals or more, fact literals and such atoms,
/// positively, so that a cube written through helper atoms is one too. A
/// rule of one literal is no cube: it names the literal, a fact the
/// aliasing reads under that name or a variable's value (`v(f) :- \+sw`)
/// that stays an atom of the model. An atom the evidence observes, or that
/// any other rule reads, is not. The formula of such an atom holds
/// a node per level of its cone on any vtree, and the query phase counts a
/// query of one against the evidence without conjoining the two.
///
/// A copy of a rewrite through indicators ([`crate::indicators`]) is a fact
/// of the encoding, not of the model: it stands for a derived atom, tied to
/// it by a link the query phase conjoins, so a rule that reads one is no
/// cube. Under `families` the top rolls of the ProbLog tutorial's dice
/// (`seen([_,_,_,_,_])`) read copies alone, of the roll before and of the
/// die's values. Taken for cubes, they left 81 of those copies in no
/// family, and the layout placed them apart from their links: 211 million
/// pairs of evidence where the queries' own families make 1366.
pub(crate) fn queried_cubes(gp: &GroundProgram) -> Vec<bool> {
    let n = gp.num_atoms();
    let by_head = gp.rules_by_head();
    let readers = gp.readers(|rule, _| rule);
    let derived = |a: AtomId| !gp.is_fact(a);
    let mut copy = vec![false; n];
    for ind in &gp.encoding.indicators {
        copy[ind.var as usize] = true;
    }
    // The candidates, by their own rule.
    let mut cube: Vec<bool> = (0..n)
        .map(|a| match &by_head[a] {
            [ri] => {
                let r = &gp.rules[*ri];
                r.body.len() > 1
                    && !r.pairs
                    && r.body.iter().all(|l| {
                        let b = l.atom();
                        !copy[b as usize] && (!derived(b) || l.is_positive())
                    })
            }
            _ => false,
        })
        .collect();
    let mut query = vec![false; n];
    for &(_, id, _) in &gp.queries {
        if let Some(a) = id {
            query[a as usize] = true;
        }
    }
    for &(_, id, _) in &gp.evidence {
        if let Some(a) = id {
            cube[a as usize] = false;
        }
    }
    // A candidate stays where it is a query or read, every rule reading it
    // has a cube's head and every derived atom its rule reads is a cube. An
    // atom that leaves may take its readers' heads and its body's atoms
    // with it.
    let stays = |a: usize, cube: &[bool]| {
        (query[a] || !readers[a].is_empty())
            && readers[a]
                .iter()
                .all(|&ri| cube[gp.rules[ri as usize].head as usize])
            && gp.rules[by_head[a][0]]
                .body
                .iter()
                .all(|l| !derived(l.atom()) || cube[l.atom() as usize])
    };
    let mut left: Vec<usize> = (0..n).filter(|&a| cube[a] && !stays(a, &cube)).collect();
    for &a in &left {
        cube[a] = false;
    }
    while let Some(x) = left.pop() {
        let heads = readers[x].iter().map(|&ri| gp.rules[ri as usize].head);
        let body = gp.rules[by_head[x][0]].body.iter().map(|l| l.atom());
        for y in heads.chain(body) {
            let y = y as usize;
            if cube[y] && !stays(y, &cube) {
                cube[y] = false;
                left.push(y);
            }
        }
    }
    cube
}

/// The rules of a program's queried cubes ([`queried_cubes`]), set apart
/// from its model, the program without them, which the layouts read
/// (`crate::vtree::build`). A query of a cube of facts shapes no layout: its
/// formula is as small on any vtree, and no other formula reads it. Read as
/// rules, the pairs and triples of faults a fault analysis asks about tie
/// every two facts together, so that a decomposition holds them all in one
/// bag and `closed` takes them as blocks, in the order the program happens
/// to list its queries in. The grounder's aliasing reads no cube as a use
/// of a fact either (`super::simplify::alias_fact_atoms`).
pub(crate) struct QueriedCubes {
    rules: Vec<GroundRule>,
    /// The program's queries, those of the cubes in their places.
    queries: Vec<Option<(Atom, Option<AtomId>, bool)>>,
}

impl QueriedCubes {
    /// The queried cubes of `gp`, and its model with the model's rules by
    /// head and components; `None` where `gp` has no queried cube. The
    /// cubes' atoms stay in the model as atoms that nothing reads or names
    /// and no rule defines, and are in none of its components.
    pub(crate) fn split(gp: &GroundProgram) -> Option<(QueriedCubes, GroundProgram, Deps)> {
        let cube = queried_cubes(gp);
        if !cube.contains(&true) {
            return None;
        }
        let (rules, kept) = gp
            .rules
            .iter()
            .cloned()
            .partition(|r| cube[r.head as usize]);
        let asks = |id: Option<AtomId>| id.is_some_and(|a| cube[a as usize]);
        let queries = (gp.queries.iter())
            .map(|q| asks(q.1).then(|| q.clone()))
            .collect();
        let mut model = gp.without_rules();
        model.rules = kept;
        model.queries.retain(|q| !asks(q.1));
        let Deps { by_head, sccs } = Deps::of(&model);
        let sccs = (sccs.into_iter())
            .filter(|scc| !scc.iter().all(|&a| cube[a as usize]))
            .collect();
        Some((
            QueriedCubes { rules, queries },
            model,
            Deps { by_head, sccs },
        ))
    }

    /// `p`, a program a layout built over the model, which numbers the
    /// model's atoms as the model does (its copies of facts or its names
    /// added), with the cubes' rules and queries.
    pub(crate) fn put_back(&self, mut p: GroundProgram) -> GroundProgram {
        let mut kept = std::mem::take(&mut p.queries).into_iter();
        p.queries = (self.queries.iter())
            .map(|q| match q {
                Some(q) => q.clone(),
                None => kept.next().expect("the model's queries"),
            })
            .collect();
        debug_assert!(kept.next().is_none(), "a layout added a query");
        p.rules.extend(self.rules.iter().cloned());
        p.structure = OnceLock::new();
        p
    }
}
