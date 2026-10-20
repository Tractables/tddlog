//! What shape a ground program has, which `auto` picks its trials by
//! (`auto::candidates`) and the layouts and the preprocessing read: no
//! component order to search ([`nothing_to_order`]), rules over facts alone
//! ([`reads_facts_alone`]), a CNF over the facts ([`clauses_over_facts`]).

use super::{AtomId, GroundProgram};

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
/// `scripts/gen_bn.py` writes one, observed true, is a table over its rows'
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
