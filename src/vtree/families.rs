//! The `families` layout, for a program that reads atoms through
//! indicators.

use std::time::Duration;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::disjoint::{find, union};
use crate::ground::deps::Deps;
use crate::ground::{AtomId, GLit, GroundProgram, RulesByHead};
use crate::multisets::Multisets;

use super::cost::{Interfaces, Open, Scope, Weighed};
use super::forest::Forest;
use super::graph::{
    ELIMINATION_BUDGET_UNITS, anneal_budget, annealed, eliminated_in_order, elimination_budget,
    facts_by_use, min_fill_cliques, minor_min_width, weighted_min_fill,
};
use super::nest::{AtomMarks, NestScratch, Rows, nest};
use super::td::{metered, portfolio_to_bound, scaled_budget_units};
use super::{Estimate, TD_SEED};

/// The min-fill width from which `families` searches for a narrower
/// decomposition. Below it the network's joint is small whatever the width
/// is off by: min-fill already finds the treewidth of every network of the
/// suite up to hailfinder's four.
pub(super) const FAMILIES_SEARCH_FROM_WIDTH: u32 = 8;

/// The `families` vtree, for a program that reads atoms through indicators
/// ([`crate::indicators`]).
///
/// A *family* is a formula the query phase conjoins: an indicator's link,
/// a query or an observed atom. Its variables are the facts and indicators
/// its derived atoms reach before they reach an indicator, its own
/// indicator left out for a link. Families that share a fact or an
/// indicator are one group: the values of one Bayesian network variable
/// share the switches of its table's rows, and the links of its values to
/// their readers all read the copies of its parents that its unit reads.
/// A group is then a network variable, its facts private to it, and it
/// meets the others in the copies it reads and the copies its links define.
///
/// A link that holds a table's clauses ([`crate::indicators::introduce`]) is
/// observed false and defines a reader's copies of every value of the
/// table. Its rules that read one of those copies false are the table's
/// rows, which its group's block reads as the rows.
///
/// A group's *block* ([`nest`]) is its facts on a right-linear spine, the
/// facts of one row of a table together and the first used deepest, above
/// a balanced join of the copies it reads; the parts of a split table are
/// blocks of their own on that spine, or in a balanced tree of their own
/// where each reads its value of the split parent alone. A table's
/// switches are a multiplexer over its parents: a node holding the
/// switches of several rows but not the parents would have a class for
/// every function from those rows to the head's values, while on the
/// spine above the parents a node's classes are the rows' outcomes so
/// far, one per row still to come. Laid out the other way, the rows of a
/// part in a balanced tree of their own beside the parents' copies, as a
/// single row's facts are, the compile and the evidence grow many times.
/// Where each row reads an atom of its own, a class atom of the
/// table's row classes ([`crate::ground::classes`]), the table over those
/// atoms is split on them, and its parts go side by side, each above the
/// copy of its row's atom.
///
/// A fact that the rules of two atoms or more read positively is a hub
/// where its readers are apart without it ([`Hubs::of`],
/// [`Grouping::of`]): it joins no families, and the hubs one rule reads
/// together are a group of their own that their readers read. A group
/// whose rows fall into three branches or more, rows that share neither a
/// fact nor a group they read ([`Branch`]), is laid out as its branches,
/// each a block of its own: the disjunction of independent parts, such as
/// uwcse's professor teaching a course when some advisee assists in it,
/// is then a chain through the parts rather than a clique over all of
/// them.
///
/// The groups are laid out by a tree decomposition of the graph in which
/// each group and the groups defining the copies it reads are a clique, the
/// moral graph of a network. Each group's block goes in the highest bag
/// holding the group with its parents, and a bag's node joins its blocks
/// and its children's nodes, the pair open on the fewest values first
/// ([`Interfaces::paired`]). Groups no bag placed and facts no family reads
/// join at the top. Of the decompositions of two numberings of the groups,
/// the layout with the least [`Interfaces::layout_cost`] is kept.
///
/// `Err` when the program has no indicators, which leaves nothing for the
/// decomposition to lay out; `Ok(None)` when it has no facts.
pub(super) fn families_forest(
    gp: &GroundProgram,
    deps: &Deps,
    search: bool,
) -> Result<Option<(Forest, usize, String, Estimate)>, String> {
    if gp.encoding.is_empty() {
        return Err("the families vtree lays out indicator variables, and the program reads no atom through one".to_string());
    }
    if gp.facts().next().is_none() {
        return Ok(None);
    }
    lay_out(gp, Groups::of(gp, deps), search).map(Some)
}

/// What [`row_body`] makes of each rule: a row of its group's table, none,
/// or a row of a table held as clauses.
const ROW: u8 = 0;
const NO_ROW: u8 = 1;
const CLAUSE_ROW: u8 = 2;

/// The row rule `ri` reads, by its kind in `row_kind`: its body, and the
/// link whose copies the row leaves out ([`reads`]).
fn row_body<'a>(
    gp: &'a GroundProgram,
    row_kind: &[u8],
    ri: usize,
) -> Option<(&'a [GLit], Option<AtomId>)> {
    let r = &gp.rules[ri];
    match row_kind[ri] {
        ROW => Some((&r.body, None)),
        CLAUSE_ROW => Some((&r.body, Some(r.head))),
        _ => None,
    }
}

/// Whether a row reads `l`: all of its literals but the copies of the link
/// it leaves out, `link_of` giving each indicator's link.
fn reads(link_of: &[AtomId], l: &GLit, left_out: Option<AtomId>) -> bool {
    left_out.is_none_or(|link| link_of[l.atom() as usize] != link)
}

/// The fewest branches ([`Branch`]) a group's rows fall into for the group
/// to be laid out as its branches. A grid's node reaches the target through
/// either of its two successors, which share most of what they reach: laid
/// out as two branches, a grid's decomposition is wider.
const MIN_BRANCHES: usize = 3;

/// The rows of a group, `members` into its [`Rows`], that read none of the
/// groups or facts the group's other rows read: a part of the group's
/// disjunctions independent of the rest given the groups it reads, its
/// `parents`. Its block holds its `facts` and the `copies` it reads.
struct Branch {
    block: Option<usize>,
    parents: Vec<usize>,
    facts: Vec<AtomId>,
    copies: Vec<AtomId>,
}

/// The branches of group `g`'s `rows`, each with its rows and the groups
/// they read ([`Branch`]), in the order of the first group each reads, or
/// none where there are fewer than [`MIN_BRANCHES`]: rows that share a fact
/// or read copies of one group are one branch, `owner` giving the group
/// that defines each copy. The hubs' groups that `row_hubs` says each row
/// reads are its parents too, but join no rows: a hub is in a block of its
/// own. The rows of one context read the same groups, which are looked up
/// once per context: a table's rows each read every parent.
///
/// The first row to read each group and each fact is marked by the group
/// in `group_reader` and by the fact in `fact_reader`.
fn branches_of(
    rows: &Rows,
    owner: &[usize],
    row_hubs: &[(usize, usize)],
    g: usize,
    group_reader: &mut AtomMarks,
    fact_reader: &mut AtomMarks,
) -> Vec<(Vec<usize>, Vec<usize>)> {
    let n = rows.len();
    let read_groups = |x: u32| {
        rows.context_copies(x)
            .iter()
            .map(|&(c, _)| owner[c as usize])
            .filter(|&h| h != usize::MAX)
    };
    let mut up: Vec<usize> = (0..n).collect();
    group_reader.next();
    fact_reader.next();
    let first_reader = |marks: &mut AtomMarks, k: u32, i: usize| match marks.get(k) {
        Some(j) => j as usize,
        None => {
            marks.set(k, i as u32);
            i
        }
    };
    // The first row of each context that reads a group; `n` for a context
    // that reads none, whose rows it does not join.
    let mut context_row = vec![usize::MAX; rows.num_contexts()];
    for i in 0..n {
        let x = rows.context[i];
        match context_row[x as usize] {
            usize::MAX => {
                context_row[x as usize] = n;
                for h in read_groups(x) {
                    context_row[x as usize] = i;
                    let j = first_reader(group_reader, h as u32, i);
                    union(&mut up, i, j);
                }
            }
            j if j < n => {
                union(&mut up, i, j);
            }
            _ => {}
        }
        for &f in rows.facts(i) {
            let j = first_reader(fact_reader, f, i);
            union(&mut up, i, j);
        }
    }
    let mut branch_of = vec![usize::MAX; n];
    let mut count = 0;
    for i in 0..n {
        let r = find(&mut up, i);
        if branch_of[r] == usize::MAX {
            branch_of[r] = count;
            count += 1;
        }
    }
    if count < MIN_BRANCHES {
        return Vec::new();
    }
    let mut branches: Vec<(Vec<usize>, Vec<usize>)> = vec![(Vec::new(), Vec::new()); count];
    let mut context_seen = vec![false; rows.num_contexts()];
    for i in 0..n {
        let (members, parents) = &mut branches[branch_of[find(&mut up, i)]];
        members.push(i);
        let x = rows.context[i];
        if !std::mem::replace(&mut context_seen[x as usize], true) {
            parents.extend(read_groups(x));
        }
    }
    for &(i, h) in row_hubs {
        branches[branch_of[find(&mut up, i)]].1.push(h);
    }
    for (_, parents) in &mut branches {
        parents.retain(|&h| h != g);
        parents.sort_unstable();
        parents.dedup();
    }
    branches.sort_by_key(|(_, parents)| parents.first().copied().unwrap_or(usize::MAX));
    branches
}

/// What an atom is to the layout ([`Groups::kind`]): a derived atom, a fact
/// other than an indicator, or an indicator.
const DERIVED: u8 = 0;
const FACT: u8 = 1;
const INDICATOR: u8 = 2;

/// The groups of [`families_forest`], their rules, the copies they read and
/// the groups that define those, with what [`lay_out`] reads of the program
/// to weigh a layout of them.
struct Groups {
    /// How many families there are.
    families: usize,
    /// Each group's indicators and facts, the facts in the order its block
    /// takes them.
    groups: Vec<(Vec<AtomId>, Vec<AtomId>)>,
    /// Each group's rules, ascending.
    group_rules: Vec<Vec<usize>>,
    /// The copies each group's rows read.
    inputs: Vec<Vec<AtomId>>,
    /// The groups whose links define the copies each group reads, and the
    /// hubs it reads.
    parents: Vec<Vec<usize>>,
    /// The group whose links define each indicator, `usize::MAX` for other
    /// atoms.
    owner: Vec<usize>,
    /// The group of each hub, `usize::MAX` for other atoms.
    hub_group: Vec<usize>,
    /// The first root of each group, for the decomposition's other numbering.
    first_root: Vec<usize>,
    /// By atom: the links, the links that hold a table's clauses, and the
    /// atoms whose copies those define.
    is_link: Vec<bool>,
    is_clause: Vec<bool>,
    is_clause_atom: Vec<bool>,
    /// The atoms whose copies each link that holds clauses defines.
    atoms_of_clause: FxHashMap<AtomId, Vec<AtomId>>,
    /// The set read in binary each bit is of, by index into the program's codes.
    code_of: FxHashMap<AtomId, usize>,
    /// The link that defines each indicator, `AtomId::MAX` for other atoms.
    link_of: Vec<AtomId>,
    /// What each rule reads as a row ([`row_body`]).
    row_kind: Vec<u8>,
    /// What each atom is: [`DERIVED`], [`FACT`] or [`INDICATOR`].
    kind: Vec<u8>,
}

impl Groups {
    /// The atoms' kinds, the clause links, the families' cones, the groups,
    /// the facts' contexts, and each group's inputs and parents.
    fn of(gp: &GroundProgram, deps: &Deps) -> Self {
        let by_head = &deps.by_head;
        let roles = Roles::of(gp, by_head);
        let roots = roots(gp, &roles.is_link);
        let cones = Cones::of(gp, by_head, &roles, &roots);
        let mut hubs = Hubs::of(gp, &roles);
        let grouping = Grouping::of(gp, &roles, &roots, &cones, &mut hubs);
        let Members {
            mut groups,
            group_rules,
            hub_group,
            hub_readers,
        } = Members::of(gp, &roles, &cones, &mut hubs, &grouping);
        order_facts(gp, deps, &roles, &mut groups);
        let Grouping {
            group_of,
            count,
            mut first_root,
            family_of,
            ..
        } = grouping;
        // A hub's group takes its first reader's first family.
        for readers in &hub_readers {
            first_root.push(
                readers
                    .iter()
                    .map(|&g| first_root[g])
                    .min()
                    .expect("a hub has a reader"),
            );
        }
        // The group whose links define each indicator.
        let mut owner = vec![usize::MAX; gp.num_atoms()];
        for ind in &gp.encoding.indicators {
            owner[ind.var as usize] = group_of[family_of[&ind.link]];
        }
        let (inputs, mut parents) = inputs(gp, &roles, &groups, &group_rules, &owner);
        for (k, readers) in hub_readers.iter().enumerate() {
            for &g in readers {
                parents[g].push(count + k);
            }
        }
        let Roles {
            kind,
            is_link,
            is_clause,
            is_clause_atom,
            atoms_of_clause,
            code_of,
            link_of,
            row_kind,
            ..
        } = roles;
        Groups {
            families: roots.len(),
            groups,
            group_rules,
            inputs,
            parents,
            owner,
            hub_group,
            first_root,
            is_link,
            is_clause,
            is_clause_atom,
            atoms_of_clause,
            code_of,
            link_of,
            row_kind,
            kind,
        }
    }
}

/// What each atom and rule is to the families ([`Roles::of`]).
struct Roles {
    /// What each atom is: [`DERIVED`], [`FACT`] or [`INDICATOR`].
    kind: Vec<u8>,
    /// By atom: the links, the links that hold a table's clauses, and the
    /// atoms whose copies those define.
    is_link: Vec<bool>,
    is_clause: Vec<bool>,
    is_clause_atom: Vec<bool>,
    /// The atoms whose copies each link that holds clauses defines.
    atoms_of_clause: FxHashMap<AtomId, Vec<AtomId>>,
    /// The atom a rule's head stands for among those the groups define,
    /// `AtomId::MAX` for none.
    defines: Vec<AtomId>,
    /// The set read in binary each bit is of, by index into the program's
    /// codes.
    code_of: FxHashMap<AtomId, usize>,
    /// The atoms a table's clauses define whose rules read their first
    /// reader's copy alone.
    copied: FxHashSet<AtomId>,
    /// The values of a set read in binary whose one rule is the cube of its
    /// code over a reader's copies of the bits, with the set's code.
    coded_value: FxHashMap<AtomId, usize>,
    /// The link that defines each indicator, `AtomId::MAX` for other atoms.
    link_of: Vec<AtomId>,
    /// What each rule reads as a row ([`row_body`]).
    row_kind: Vec<u8>,
}

impl Roles {
    fn of(gp: &GroundProgram, by_head: &RulesByHead) -> Roles {
        let num_atoms = gp.num_atoms();
        let mut indicator = vec![false; num_atoms];
        for ind in &gp.encoding.indicators {
            indicator[ind.var as usize] = true;
        }
        // What each atom is, by atom rather than through the program's kinds:
        // the passes below read it for every literal of every rule.
        let kind: Vec<u8> = (0..num_atoms)
            .map(|a| {
                if indicator[a] {
                    INDICATOR
                } else if gp.is_fact(a as AtomId) {
                    FACT
                } else {
                    DERIVED
                }
            })
            .collect();
        let mut is_link = vec![false; num_atoms];
        for ind in &gp.encoding.indicators {
            is_link[ind.link as usize] = true;
        }
        // The links that hold a table's clauses ([`crate::indicators::clause_tables`]),
        // with the atom each copy stands for: their rules are the table's rows,
        // each with the copy it sets.
        let clauses = crate::indicators::clause_links(gp);
        let mut is_clause = vec![false; num_atoms];
        for &link in &clauses {
            is_clause[link as usize] = true;
        }
        // The atoms whose copies each such link defines, and the first of them.
        let mut atoms_of_clause: FxHashMap<AtomId, Vec<AtomId>> = FxHashMap::default();
        for ind in gp
            .encoding
            .indicators
            .iter()
            .filter(|ind| clauses.contains(&ind.link))
        {
            atoms_of_clause.entry(ind.link).or_default().push(ind.atom);
        }
        let clause_of: FxHashMap<AtomId, AtomId> = atoms_of_clause
            .iter()
            .map(|(&link, atoms)| (link, *atoms.iter().min().expect("a link has an indicator")))
            .collect();
        let clause_atoms: FxHashSet<AtomId> = atoms_of_clause.values().flatten().copied().collect();
        let mut is_clause_atom = vec![false; num_atoms];
        for &k in &clause_atoms {
            is_clause_atom[k as usize] = true;
        }
        // The atom a rule's head stands for among those the groups define: a
        // link that holds clauses stands for the first atom of its table, and
        // other links and the atoms its clauses define for none.
        let mut defines = vec![AtomId::MAX; num_atoms];
        for h in 0..num_atoms {
            if !is_link[h] && !is_clause_atom[h] {
                defines[h] = h as AtomId;
            }
        }
        for (&link, &k) in &clause_of {
            defines[link as usize] = k;
        }
        // A bit of a set read in binary ([`crate::indicators`]) stands for the
        // set's values: its formula goes with theirs.
        let code_of: FxHashMap<AtomId, usize> = gp
            .encoding
            .codes
            .iter()
            .enumerate()
            .flat_map(|(c, code)| code.bits.iter().map(move |&b| (b, c)))
            .collect();
        for (&b, &c) in &code_of {
            defines[b as usize] = gp.encoding.codes[c].values[0];
        }
        // Those of them defined by their first reader's copy, and not by the
        // table's rows.
        let copied: FxHashSet<AtomId> = clause_atoms
            .iter()
            .copied()
            .filter(|&k| {
                by_head[k as usize].iter().all(|&ri| {
                    let body = &gp.rules[ri].body;
                    body.len() == 1 && body[0].is_positive() && indicator[body[0].atom() as usize]
                })
            })
            .collect();
        // A value of a set read in binary that a query or the evidence names
        // is the cube of its code over a reader's copies of the bits
        // ([`crate::indicators::introduce`]): its family reads the bits' rules,
        // as it would read the value's own, and not the reader's copies,
        // which would join it to the reader's group.
        let bit_copied: FxHashMap<AtomId, AtomId> = gp
            .encoding
            .indicators
            .iter()
            .filter(|i| code_of.contains_key(&i.atom))
            .map(|i| (i.var, i.atom))
            .collect();
        let coded_value: FxHashMap<AtomId, usize> = gp
            .encoding
            .codes
            .iter()
            .enumerate()
            .flat_map(|(c, code)| code.values.iter().map(move |&k| (k, c)))
            .filter(|&(k, c)| {
                let rules = &by_head[k as usize];
                rules.len() == 1
                    && gp.rules[rules[0]]
                        .body
                        .iter()
                        .all(|l| bit_copied.get(&l.atom()).is_some_and(|b| code_of[b] == c))
            })
            .collect();
        // An indicator is read by one unit of the rewrite and defined by the
        // link that has it in its cone; the families reading it are one group,
        // and the link is not in it.
        let mut link_of = vec![AtomId::MAX; num_atoms];
        for ind in &gp.encoding.indicators {
            link_of[ind.var as usize] = ind.link;
        }
        // What a rule reads as a row of its group's table: the body of a rule
        // of a derived atom; the body of a rule of a link that holds clauses that
        // reads a copy the link defines false, one of the table's rows, but that
        // copy; nothing of the other rules of such a link, or of other links.
        // The row is the body's literals that [`reads`] keeps, the second being
        // the link whose copies it leaves out.
        // Which of these each rule is, worked out once: the passes below read
        // a group's rules twice, and would search a link's rule for its copy
        // each time.
        let row_kind: Vec<u8> = gp
            .rules
            .iter()
            .map(|r| {
                if !is_link[r.head as usize] {
                    ROW
                } else if !is_clause[r.head as usize]
                    || !r
                        .body
                        .iter()
                        .any(|l| !l.is_positive() && link_of[l.atom() as usize] == r.head)
                {
                    NO_ROW
                } else {
                    CLAUSE_ROW
                }
            })
            .collect();
        Roles {
            kind,
            is_link,
            is_clause,
            is_clause_atom,
            atoms_of_clause,
            defines,
            code_of,
            copied,
            coded_value,
            link_of,
            row_kind,
        }
    }
}

/// The families' roots, each once: the links, in the order of the
/// indicators, then the derived atoms other than links that a query or the
/// evidence names.
fn roots(gp: &GroundProgram, is_link: &[bool]) -> Vec<AtomId> {
    let mut roots: Vec<AtomId> = gp.encoding.indicators.iter().map(|ind| ind.link).collect();
    for (_, id, _) in gp.queries.iter().chain(gp.evidence.iter()) {
        if let Some(a) = *id
            && !gp.is_fact(a)
            && !is_link[a as usize]
        {
            roots.push(a);
        }
    }
    let mut seen_root = FxHashSet::default();
    roots.retain(|&r| seen_root.insert(r));
    roots
}

/// The variables of each family and the rules that reach them
/// ([`Cones::of`]).
///
/// The families' cones and their rules are kept end to end, family `i`'s
/// from the end of the one before to `atom_ends[i + 1]` and
/// `rule_ends[i + 1]`: a list each grew one push at a time.
struct Cones {
    atoms: Vec<AtomId>,
    atom_ends: Vec<usize>,
    rules: Vec<usize>,
    rule_ends: Vec<usize>,
    /// Per family the links of the tables whose rows it reads in place of
    /// a table's atom.
    reached: Vec<Vec<AtomId>>,
    /// Per family the family whose cone it has.
    walk: Vec<usize>,
}

impl Cones {
    /// The cones of the families of `roots`: a walk down the derived atoms,
    /// stamped with the family so that no atom is visited twice for one.
    fn of(gp: &GroundProgram, by_head: &RulesByHead, roles: &Roles, roots: &[AtomId]) -> Cones {
        let Roles {
            kind,
            is_link,
            copied,
            coded_value,
            link_of,
            ..
        } = roles;
        let mut stamp = vec![u32::MAX; gp.num_atoms()];
        let mut atoms: Vec<AtomId> = Vec::new();
        let mut atom_ends: Vec<usize> = Vec::with_capacity(roots.len() + 1);
        atom_ends.push(0);
        // A cone stops at the copies it reads, so the cones' rules come to
        // about the program's: the list is allocated at that size.
        let mut rules: Vec<usize> = Vec::with_capacity(gp.rules.len());
        let mut rule_ends: Vec<usize> = Vec::with_capacity(roots.len() + 1);
        rule_ends.push(0);
        let mut stack: Vec<AtomId> = Vec::new();
        // A family that reads a table's atom where the table's clauses define
        // it reads the table's rows, as it would read the atom's rules: it joins
        // the group of the link that holds them, and does not read the first
        // reader's copy the atom is defined by.
        let mut reached: Vec<Vec<AtomId>> = Vec::with_capacity(roots.len());
        // The families of a set's values read in binary walk the same cone,
        // the set's bits and what their rules read, unless those rules read
        // a value of the set: the cone is walked for the first of them, and
        // `walk[i]` is the family whose cone family `i` has.
        let value_code: FxHashMap<AtomId, usize> = gp
            .encoding
            .codes
            .iter()
            .enumerate()
            .flat_map(|(c, code)| code.values.iter().map(move |&k| (k, c)))
            .collect();
        let mut walked_code: FxHashMap<usize, usize> = FxHashMap::default();
        let mut walk: Vec<usize> = (0..roots.len()).collect();
        for (i, &r) in roots.iter().enumerate() {
            let code = coded_value
                .get(&r)
                .copied()
                .filter(|_| !is_link[r as usize] && !copied.contains(&r));
            if let Some(&j) = code.and_then(|c| walked_code.get(&c)) {
                walk[i] = j;
                atom_ends.push(atoms.len());
                rule_ends.push(rules.len());
                reached.push(Vec::new());
                continue;
            }
            let first = i;
            let i = i as u32;
            let rules_from = rules.len();
            let mut links = Vec::new();
            stack.push(r);
            stamp[r as usize] = i;
            while let Some(a) = stack.pop() {
                if copied.contains(&a) {
                    links.extend(
                        by_head[a as usize]
                            .iter()
                            .flat_map(|&ri| &gp.rules[ri].body)
                            .map(|l| link_of[l.atom() as usize]),
                    );
                    continue;
                }
                if let Some(&c) = coded_value.get(&a) {
                    for &b in &gp.encoding.codes[c].bits {
                        if stamp[b as usize] != i {
                            stamp[b as usize] = i;
                            stack.push(b);
                        }
                    }
                    continue;
                }
                for &ri in &by_head[a as usize] {
                    rules.push(ri);
                    for l in &gp.rules[ri].body {
                        let b = l.atom();
                        if stamp[b as usize] == i {
                            continue;
                        }
                        stamp[b as usize] = i;
                        if kind[b as usize] != DERIVED {
                            atoms.push(b);
                        } else {
                            stack.push(b);
                        }
                    }
                }
            }
            if let Some(c) = code
                && !rules[rules_from..].iter().any(|&ri| {
                    gp.rules[ri]
                        .body
                        .iter()
                        .any(|l| value_code.get(&l.atom()) == Some(&c))
                })
            {
                walked_code.insert(c, first);
            }
            atom_ends.push(atoms.len());
            rule_ends.push(rules.len());
            reached.push(links);
        }
        Cones {
            atoms,
            atom_ends,
            rules,
            rule_ends,
            reached,
            walk,
        }
    }

    /// The facts and indicators of family `i`'s cone.
    fn atoms_of(&self, i: usize) -> &[AtomId] {
        &self.atoms[self.atom_ends[i]..self.atom_ends[i + 1]]
    }

    /// The rules of family `i`'s cone.
    fn rules_of(&self, i: usize) -> &[usize] {
        &self.rules[self.rule_ends[i]..self.rule_ends[i + 1]]
    }
}

/// The hubs ([`Hubs::of`]).
struct Hubs {
    /// Per atom whether it is a hub.
    hub: Vec<bool>,
    /// The hubs one rule reads together, classes of a disjoint-set forest
    /// over the atoms; empty where there is no hub.
    class: Vec<usize>,
}

impl Hubs {
    /// The facts that the rules of two atoms or more read positively, a bit
    /// standing for its set and a link's rules for none. A hub joins no
    /// families; the hubs one rule reads together are a group of their own,
    /// of facts alone, that the groups of their readers read. Joined by their
    /// hubs, the families of independent atoms would be one group that every
    /// reader of any of them read whole: uwcse's position of the professor,
    /// which every student's advising reads, and cora's same-author and
    /// same-title facts, which every pair of papers with those authors or
    /// titles reads. The switches of a table are read positively by one
    /// value's rules each (a value's other rules and links read them again),
    /// and those of a variable inlined into its readers' rules are read
    /// together.
    fn of(gp: &GroundProgram, roles: &Roles) -> Hubs {
        let num_atoms = gp.num_atoms();
        let mut first_reader = vec![AtomId::MAX; num_atoms];
        let mut hub = vec![false; num_atoms];
        for r in &gp.rules {
            let head = roles.defines[r.head as usize];
            if head == AtomId::MAX {
                continue;
            }
            for l in r
                .body
                .iter()
                .filter(|l| l.is_positive() && roles.kind[l.atom() as usize] == FACT)
            {
                let b = l.atom() as usize;
                match first_reader[b] {
                    AtomId::MAX => first_reader[b] = head,
                    h if h != head => hub[b] = true,
                    _ => {}
                }
            }
        }
        let mut class: Vec<usize> = Vec::new();
        if hub.contains(&true) {
            class = (0..num_atoms).collect();
            for r in &gp.rules {
                let mut read = r.body.iter().map(|l| l.atom() as usize).filter(|&b| hub[b]);
                if let Some(first) = read.next() {
                    for b in read {
                        union(&mut class, first, b);
                    }
                }
            }
        }
        Hubs { hub, class }
    }

    /// The class of hub `f`.
    fn class_of(&mut self, f: AtomId) -> usize {
        find(&mut self.class, f as usize)
    }
}

/// The families joined into groups and the groups numbered
/// ([`Grouping::of`]).
struct Grouping {
    /// Per family its group.
    group_of: Vec<usize>,
    /// How many groups the families make: the hubs' groups follow.
    count: usize,
    /// The first family of each group, for the decomposition's other
    /// numbering.
    first_root: Vec<usize>,
    /// Per family whether its cone is one another family of its group
    /// walked: the passes over the groups' cones would read it again for
    /// nothing.
    shared: Vec<bool>,
    /// The family of each root.
    family_of: FxHashMap<AtomId, usize>,
}

impl Grouping {
    /// Groups: families joined by the facts and the indicators they read,
    /// and by the tables' rows they read. Hubs whose readers are one group
    /// without them are that group's, and no longer hubs in `hubs`.
    fn of(
        gp: &GroundProgram,
        roles: &Roles,
        roots: &[AtomId],
        cones: &Cones,
        hubs: &mut Hubs,
    ) -> Grouping {
        let num_atoms = gp.num_atoms();
        let (walk, reached) = (&cones.walk, &cones.reached);
        let mut parent: Vec<usize> = (0..roots.len()).collect();
        let mut owner = vec![usize::MAX; num_atoms];
        // Whether a family's cone joins it to another's: a family that shares
        // a walked cone joins the group of the family that walked it, as the
        // cone's facts would join it.
        let mut joins = vec![false; roots.len()];
        for (i, &root) in roots.iter().enumerate() {
            for &f in cones.atoms_of(i) {
                if roles.kind[f as usize] == INDICATOR && roles.link_of[f as usize] == root
                    || hubs.hub[f as usize]
                {
                    continue;
                }
                joins[i] = true;
                match owner[f as usize] {
                    usize::MAX => owner[f as usize] = i,
                    j => {
                        union(&mut parent, i, j);
                    }
                }
            }
        }
        let family_of: FxHashMap<AtomId, usize> =
            roots.iter().enumerate().map(|(i, &r)| (r, i)).collect();
        for (i, links) in reached.iter().enumerate() {
            for link in links {
                let j = family_of[link];
                union(&mut parent, i, j);
            }
        }
        for (i, &j) in walk.iter().enumerate() {
            if j != i && (joins[j] || !reached[j].is_empty()) {
                union(&mut parent, i, j);
            }
        }
        let shared: Vec<bool> = (0..roots.len())
            .map(|i| walk[i] != i && find(&mut parent, i) == find(&mut parent, walk[i]))
            .collect();
        // Hubs whose readers are one group without them are that group's:
        // apart, they separate nothing and take a table's switches off its
        // spine.
        if !hubs.class.is_empty() {
            const SEVERAL: usize = usize::MAX - 1;
            let mut read_by = vec![usize::MAX; num_atoms];
            for i in (0..roots.len()).filter(|&i| !shared[i]) {
                let g = find(&mut parent, i);
                for &f in cones.atoms_of(walk[i]) {
                    if !hubs.hub[f as usize] {
                        continue;
                    }
                    let c = hubs.class_of(f);
                    read_by[c] = match read_by[c] {
                        usize::MAX => g,
                        h if h == g => g,
                        _ => SEVERAL,
                    };
                }
            }
            for f in 0..num_atoms {
                if hubs.hub[f] && read_by[hubs.class_of(f as AtomId)] != SEVERAL {
                    hubs.hub[f] = false;
                }
            }
        }
        // Groups are numbered by the first atom of the program as grounded that
        // their rules define, links aside, and not by the order of their roots:
        // the decomposition breaks ties by that number, and how the rewrite ties
        // its copies together should not move it.
        let mut first_atom = vec![AtomId::MAX; roots.len()];
        for i in (0..roots.len()).filter(|&i| !shared[i]) {
            let head = cones
                .rules_of(walk[i])
                .iter()
                .map(|&ri| roles.defines[gp.rules[ri].head as usize])
                .min()
                .unwrap_or(AtomId::MAX);
            let g = find(&mut parent, i);
            first_atom[g] = first_atom[g].min(head);
        }
        let mut group_roots: Vec<usize> = (0..roots.len())
            .filter(|&i| find(&mut parent, i) == i)
            .collect();
        group_roots.sort_by_key(|&g| (first_atom[g], g));
        let mut group_of = vec![usize::MAX; roots.len()];
        for (n, &g) in group_roots.iter().enumerate() {
            group_of[g] = n;
        }
        for i in 0..roots.len() {
            group_of[i] = group_of[find(&mut parent, i)];
        }
        let mut first_root = vec![usize::MAX; group_roots.len()];
        for (i, &g) in group_of.iter().enumerate() {
            first_root[g] = first_root[g].min(i);
        }
        Grouping {
            group_of,
            count: group_roots.len(),
            first_root,
            shared,
            family_of,
        }
    }
}

/// What each group holds ([`Members::of`]).
struct Members {
    /// Each group's indicators and facts, sorted.
    groups: Vec<(Vec<AtomId>, Vec<AtomId>)>,
    /// Each group's rules, ascending.
    group_rules: Vec<Vec<usize>>,
    /// The group of each hub, `usize::MAX` for other atoms.
    hub_group: Vec<usize>,
    /// The groups that read each hub's group, ascending.
    hub_readers: Vec<Vec<usize>>,
}

impl Members {
    /// The facts, indicators and rules of each group's families' cones, and
    /// the hubs' groups, which follow the others in the order they are
    /// first read, each with the groups that read it.
    fn of(
        gp: &GroundProgram,
        roles: &Roles,
        cones: &Cones,
        hubs: &mut Hubs,
        grouping: &Grouping,
    ) -> Members {
        let num_atoms = gp.num_atoms();
        let Grouping {
            group_of,
            count,
            shared,
            ..
        } = grouping;
        // A group's families share most of their facts and the rules of
        // their cones, so each goes in once per run of the group's
        // families, and the rest are taken out below: a rule's groups are
        // gathered as pairs, and the pairs taken in the order of their
        // rules list each group's rules in order without a sort.
        let mut groups: Vec<(Vec<AtomId>, Vec<AtomId>)> = vec![(Vec::new(), Vec::new()); *count];
        let mut group_rules: Vec<Vec<usize>> = vec![Vec::new(); groups.len()];
        let mut rule_groups: Vec<(usize, usize)> = Vec::with_capacity(gp.rules.len());
        let mut last_group = vec![usize::MAX; num_atoms];
        let mut last_group_of_rule = vec![usize::MAX; gp.rules.len()];
        let mut hub_group = vec![usize::MAX; num_atoms];
        let mut class_group = vec![usize::MAX; hubs.class.len()];
        let mut hub_readers: Vec<Vec<usize>> = Vec::new();
        for i in (0..group_of.len()).filter(|&i| !shared[i]) {
            let g = group_of[i];
            for &f in cones.atoms_of(cones.walk[i]) {
                if last_group[f as usize] == g {
                    continue;
                }
                last_group[f as usize] = g;
                if hubs.hub[f as usize] {
                    let c = hubs.class_of(f);
                    if class_group[c] == usize::MAX {
                        class_group[c] = groups.len();
                        hub_readers.push(Vec::new());
                        groups.push((Vec::new(), Vec::new()));
                        group_rules.push(Vec::new());
                    }
                    let h = class_group[c];
                    if hub_group[f as usize] == usize::MAX {
                        hub_group[f as usize] = h;
                        groups[h].1.push(f);
                    }
                    hub_readers[h - count].push(g);
                    continue;
                }
                let (vars, facts) = &mut groups[g];
                if roles.kind[f as usize] == INDICATOR {
                    vars.push(f)
                } else {
                    facts.push(f)
                }
            }
            rule_groups.extend(
                cones
                    .rules_of(cones.walk[i])
                    .iter()
                    .copied()
                    .filter(|&ri| std::mem::replace(&mut last_group_of_rule[ri], g) != g)
                    .map(|ri| (ri, g)),
            );
        }
        for readers in &mut hub_readers {
            readers.sort_unstable();
            readers.dedup();
        }
        let mut end = vec![0usize; gp.rules.len()];
        for &(ri, _) in &rule_groups {
            end[ri] += 1;
        }
        for ri in 1..end.len() {
            end[ri] += end[ri - 1];
        }
        let mut by_rule = vec![0usize; rule_groups.len()];
        for &(ri, g) in rule_groups.iter().rev() {
            end[ri] -= 1;
            by_rule[end[ri]] = g;
        }
        // Each group's rules counted first, for its list to be allocated at
        // its size. A rule is counted once per group, as it is listed once.
        let mut counted = vec![(usize::MAX, 0usize); group_rules.len()];
        for (ri, &start) in end.iter().enumerate() {
            let stop = end.get(ri + 1).copied().unwrap_or(by_rule.len());
            for &g in &by_rule[start..stop] {
                if counted[g].0 != ri {
                    counted[g] = (ri, counted[g].1 + 1);
                }
            }
        }
        for (rules, &(_, count)) in group_rules.iter_mut().zip(&counted) {
            rules.reserve_exact(count);
        }
        for (ri, &start) in end.iter().enumerate() {
            let stop = end.get(ri + 1).copied().unwrap_or(by_rule.len());
            for &g in &by_rule[start..stop] {
                if group_rules[g].last() != Some(&ri) {
                    group_rules[g].push(ri);
                }
            }
        }
        for (vars, facts) in &mut groups {
            vars.sort_unstable();
            vars.dedup();
            facts.sort_unstable();
            facts.dedup();
        }
        Members {
            groups,
            group_rules,
            hub_group,
            hub_readers,
        }
    }
}

/// Each group's facts in the order its block takes them, by their
/// contexts and then their first use.
///
/// A fact's context: the indicator literals, signed, in the body of the
/// first rule that reads it. The switches of one row of a table share a
/// context, the parents' values the row is for (the copy of the value it
/// reads, the negations of the copies of the others: the sign tells the
/// rows apart), and the spine keeps each context's facts together,
/// contexts in the order they are first used. In plain first-use order a
/// table's switches come value by value, the first switch of every row
/// before the second of any, and a spine node then holds part of every
/// row.
fn order_facts(
    gp: &GroundProgram,
    deps: &Deps,
    roles: &Roles,
    groups: &mut [(Vec<AtomId>, Vec<AtomId>)],
) {
    let num_atoms = gp.num_atoms();
    let mut position = vec![usize::MAX; num_atoms];
    for (i, a) in facts_by_use(gp, deps).into_iter().enumerate() {
        position[a as usize] = i;
    }
    const NO_CONTEXT: u32 = u32::MAX;
    let mut context: Vec<u32> = vec![NO_CONTEXT; num_atoms];
    // A context is its copies as a multiset ([`Multisets`]), looked up
    // without sorting the copies of the rules that give one.
    let mut contexts = Multisets::new(num_atoms);
    // The key of the last rule to give a context, and its context: the
    // rules of a row come one after another, and a key equal to the one
    // before, literal for literal, is the same multiset. A rule's key is
    // compared with that one as the rule reads it, and copied out only
    // where it differs.
    let mut last_key: Vec<GLit> = Vec::new();
    let mut last = NO_CONTEXT;
    for r in &gp.rules {
        // Only a rule that reads a fact without a context yet gives one: the
        // other values of a row read its switches again. Its key is read
        // at the first such fact.
        let mut id = NO_CONTEXT;
        for l in &r.body {
            let b = l.atom() as usize;
            if roles.kind[b] == FACT && context[b] == NO_CONTEXT {
                if id == NO_CONTEXT {
                    let keyed = |l: &&GLit| {
                        roles.kind[l.atom() as usize] == INDICATOR
                            && roles.link_of[l.atom() as usize] != r.head
                    };
                    if last == NO_CONTEXT || !r.body.iter().filter(keyed).eq(last_key.iter()) {
                        last_key.clear();
                        last_key.extend(r.body.iter().filter(keyed).copied());
                        last = contexts.id(&last_key).0;
                    }
                    id = last;
                }
                context[b] = id;
            }
        }
    }
    // The first position of each context among a group's facts, by context
    // and with the group it was set for; no context is the last slot. The
    // sort keys are worked out once, not at each comparison.
    let contexts = contexts.len();
    let mut first_of: Vec<(usize, usize)> = vec![(usize::MAX, usize::MAX); contexts + 1];
    let slot = |f: AtomId| match context[f as usize] {
        NO_CONTEXT => contexts,
        c => c as usize,
    };
    for (g, (_, facts)) in groups.iter_mut().enumerate() {
        for &f in facts.iter() {
            let e = &mut first_of[slot(f)];
            if e.0 != g {
                *e = (g, usize::MAX);
            }
            e.1 = e.1.min(position[f as usize]);
        }
        let mut keyed: Vec<(usize, usize, AtomId)> = facts
            .iter()
            .map(|&f| (first_of[slot(f)].1, position[f as usize], f))
            .collect();
        keyed.sort_unstable();
        facts.clear();
        facts.extend(keyed.into_iter().map(|(_, _, f)| f));
    }
}

/// The copies each group's rows read, and the groups whose links define
/// them, by `owner`, each group's ascending.
///
/// The copies a group defines that its rules read, its links' rules
/// aside. A split table's copies after the first are tied to the one
/// before ([`crate::indicators`]), so the table's own group defines
/// them and reads them. Only those copies are looked for, each marked
/// `2 g` while it is sought and `2 g + 1` once a row reads it, and a
/// group's rows are read until every one is found: the copies of
/// other groups' links are its inputs without a look.
fn inputs(
    gp: &GroundProgram,
    roles: &Roles,
    groups: &[(Vec<AtomId>, Vec<AtomId>)],
    group_rules: &[Vec<usize>],
    owner: &[usize],
) -> (Vec<Vec<AtomId>>, Vec<Vec<usize>>) {
    let mut read_in = vec![usize::MAX; gp.num_atoms()];
    let mut inputs: Vec<Vec<AtomId>> = vec![Vec::new(); groups.len()];
    let mut parents: Vec<Vec<usize>> = vec![Vec::new(); groups.len()];
    for (g, (vars, _)) in groups.iter().enumerate() {
        let mut sought = 0;
        for &v in vars.iter().filter(|&&v| owner[v as usize] == g) {
            read_in[v as usize] = 2 * g;
            sought += 1;
        }
        if sought > 0 {
            'rows: for (body, left_out) in group_rules[g]
                .iter()
                .filter_map(|&ri| row_body(gp, &roles.row_kind, ri))
            {
                for l in body {
                    let a = l.atom() as usize;
                    if read_in[a] == 2 * g && reads(&roles.link_of, l, left_out) {
                        read_in[a] = 2 * g + 1;
                        sought -= 1;
                        if sought == 0 {
                            break 'rows;
                        }
                    }
                }
            }
        }
        for &v in vars {
            match owner[v as usize] {
                h if h == g => {
                    if read_in[v as usize] == 2 * g + 1 {
                        inputs[g].push(v);
                    }
                }
                usize::MAX => inputs[g].push(v),
                h => {
                    inputs[g].push(v);
                    parents[g].push(h);
                }
            }
        }
        parents[g].sort_unstable();
        parents[g].dedup();
    }
    (inputs, parents)
}

/// The `families` layout of `groups`: their blocks and the facts no
/// family reads, laid out over the decompositions of the groups' moral
/// graph, the one with the least layout cost kept.
fn lay_out(
    gp: &GroundProgram,
    groups: Groups,
    search: bool,
) -> Result<(Forest, usize, String, Estimate), String> {
    let num_atoms = gp.num_atoms();
    let num_groups = groups.groups.len();
    let mut forest = Forest::default();
    let mut builder = Blocks::new(num_atoms, num_groups);
    let (blocks, branches): (Vec<Option<usize>>, Vec<Vec<Branch>>) = (0..num_groups)
        .map(|g| builder.of(gp, &groups, &mut forest, g))
        .unzip();
    let stray = stray_facts(gp, &mut forest, &blocks, &branches);
    let stray_top = forest.balanced(&stray);
    let (scope, values) = scope(gp, &groups, &branches);
    let items = Items::of(&groups.parents, blocks, &branches);
    let num_items = items.block.len();
    let leaves = items
        .block
        .iter()
        .flatten()
        .flat_map(|&t| forest.leaves_of(t));
    let interfaces = Interfaces::new(&scope, leaves, &values);
    // The blocks are under every layout, so what each is open on, and what
    // its joins are charged, is worked out once for all of them.
    let block_open: Vec<Option<Open>> = items
        .block
        .iter()
        .map(|b| b.map(|t| interfaces.of_leaves(&forest, t)))
        .collect();
    let mut weighed = Weighed::default();
    for &t in items.block.iter().flatten() {
        interfaces.weigh(&forest, t, &mut weighed);
    }
    let mut layouts = Layouts {
        forest,
        items: &items,
        interfaces,
        block_open,
        weighed,
        stray_top,
        best: None,
        candidates: 0,
        weigh_seconds: 0.0,
    };
    let edges = items.edges(&groups.parents);
    let num_edges = edges.len();
    // A decomposition breaks its ties by vertex number, and a layout that is
    // as wide as another can still cost more pairs. The groups are numbered
    // by the first atom they define and by the first link among their roots
    // (the compile order of the atoms they are copies of), and each
    // numbering is decomposed, by min-fill and, where min-fill leaves a
    // width at which a narrower decomposition is worth its search, by the
    // portfolio too; the first is also decomposed by weighted min-fill and
    // by annealing, where those apply (below). The layout with the
    // smallest cost ([`Interfaces::layout_cost`]) is kept, the first on a
    // tie. The search's floor is the same work on a graph of ten groups as
    // on one of a thousand, which on the small networks would be most of
    // the run, and width alone does not tell those apart. `families:0`
    // skips the searches, which is how `auto` tries these layouts first.
    let numberings = items.numberings(&groups.first_root);
    let item_values = items.values(&values);
    let uniform = item_values.iter().all(|&v| v == item_values[0]);
    // Seconds and units in the decomposer's searches and the annealing,
    // for `--stats`.
    let (mut search_seconds, mut search_units) = (0.0, 0);
    // Seconds and units in the numberings' min-fill eliminations, weighted
    // min-fill's among them.
    let (mut min_fill_seconds, mut min_fill_units) = (0.0, 0);
    // The graph's treewidth lower bound, taken once where a numbering's
    // min-fill width would call for a search.
    let mut lower_bound = None;
    // What the searches charge, or would charge where `search` is off
    // ([`Estimate::search_units`]).
    let mut search_cost = 0;
    // The first numbering's graph, its items' values and its weighted
    // min-fill elimination, where the annealing may run (below).
    let mut annealable = None;
    for (k, item_at) in numberings.iter().enumerate() {
        let mut vertex_of = vec![0u32; num_items];
        for (v, &i) in item_at.iter().enumerate() {
            vertex_of[i] = v as u32;
        }
        let edges: Vec<(u32, u32)> = edges
            .iter()
            .map(|&(u, v)| {
                (
                    vertex_of[u].min(vertex_of[v]),
                    vertex_of[u].max(vertex_of[v]),
                )
            })
            .collect();
        let graph = goatd::Graph::new(num_items as u32, edges.clone());
        let (min_fill, units, seconds) = metered(|| {
            goatd::elimination::decompose(
                &graph,
                goatd::elimination::Order::MinFill,
                TD_SEED,
                Some(elimination_budget()),
            )
        });
        let min_fill = min_fill.map_err(|e| format!("min-fill decomposition failed: {e:?}"))?;
        min_fill_seconds += seconds;
        min_fill_units += units;
        let mut tds = Vec::new();
        let wide = min_fill.treewidth() >= FAMILIES_SEARCH_FROM_WIDTH;
        // No search narrows a decomposition as narrow as the graph's lower
        // bound, and none runs where min-fill's is: there the searches find
        // min-fill's width in their first candidates and spend the rest of
        // their budgets for no cheaper layout. Nor does one run where
        // min-fill charged as much as the search may spend, twice its
        // budget (goatd's hard cutoff): the search's own eliminations, of
        // which min-fill is one, then stop part way, and what it returns
        // has the vertices they left in one bag.
        let budget_units = scaled_budget_units(num_items, num_edges);
        let narrowable = wide
            && units < 2 * budget_units
            && min_fill.treewidth() as usize
                > *lower_bound.get_or_insert_with(|| minor_min_width(num_items, &edges));
        if narrowable {
            search_cost += 2 * budget_units;
        }
        if search && narrowable {
            let budget = Duration::from_millis(goatd::meter::milliseconds_for_units(budget_units));
            let bound =
                lower_bound.expect("the bound is computed where the decomposition is narrowable");
            let (found, units, seconds) =
                metered(|| portfolio_to_bound(&graph, budget, budget_units, bound));
            let (td, pushed) = found.ok_or("tree decomposition failed")?;
            tds.push(td);
            search_seconds += seconds;
            search_units += units.saturating_sub(pushed);
        }
        // The groups eliminated in their numbering's order, where that is
        // narrower than min-fill. A program whose atoms are derived layer
        // by layer, as a grid's reachability, numbers its groups a layer at
        // a time, and eliminating them so gives the grid its side as width,
        // narrower than min-fill's.
        let narrower = (min_fill.treewidth() as usize).checked_sub(1);
        let in_order = narrower.and_then(|width| {
            eliminated_in_order(
                &edges,
                (0..num_items as u32).collect(),
                width,
                2 * ELIMINATION_BUDGET_UNITS,
            )
        });
        tds.extend(in_order.and_then(|elimination| elimination.decomposition(&graph)));
        tds.push(min_fill);
        // The first numbering is also eliminated by min-fill weighted by
        // the items' values ([`weighted_min_fill`]), where they differ: on
        // equal values it is tddlog's min-fill, which ties as goatd's does
        // not.
        let weights: Vec<u64> = item_at.iter().map(|&i| item_values[i]).collect();
        let weighted = (k == 0 && !uniform).then(|| {
            let (weighted, units, seconds) = metered(|| weighted_min_fill(&edges, &weights));
            min_fill_units += units;
            min_fill_seconds += seconds;
            weighted
        });
        tds.extend(weighted.as_ref().and_then(|e| e.decomposition(&graph)));
        for td in &tds {
            layouts.weigh(td, item_at)?;
        }
        if k == 0 && search && narrowable {
            annealable = Some((edges, graph, weights, weighted));
        }
    }
    // Where the portfolio searches the first numbering and the cheapest
    // layout of both numberings pays for it ([`anneal_budget`]), the better
    // of the first numbering's min-fill and weighted min-fill eliminations
    // by their states ([`Elimination::states`]) is improved by annealing,
    // and laid out too. It runs with the portfolio, whose charge `auto`
    // already weighs, and where the portfolio does not run `families:0`
    // has no search to weigh: counted beside the portfolio's, its budget
    // delayed the searched layout on the UAI PR tuning set's grids (from
    // 0.3 s of `families:0` to 3.5 s), and where it ran alone it made
    // `auto` build a searched layout that did not win (objectdetection, 0.2
    // to 0.7 s more). Its budget is read off the cheapest layout of both
    // numberings: off the first's alone, it spent 4.8 s on 1npl of the UAI
    // PR final set, a fifth of a budget the second numbering's layout,
    // five times cheaper, would have given it, for no cheaper layout.
    if let Some((edges, graph, weights, weighted)) = annealable
        && let Some(budget) = layouts
            .best
            .and_then(|(cost, ..)| anneal_budget(num_items, num_edges, cost))
    {
        let bits: Vec<f64> = weights.iter().map(|&w| (w as f64).log2()).collect();
        let (found, units, seconds) = metered(|| {
            let min_fill = min_fill_cliques(num_items, &edges);
            let start = match weighted {
                Some(w) if w.states(&bits) < min_fill.states(&bits) => w,
                _ => min_fill,
            };
            annealed(&edges, &bits, &start.order, budget).and_then(|order| {
                eliminated_in_order(&edges, order, usize::MAX, 2 * ELIMINATION_BUDGET_UNITS)
            })
        });
        search_units += units;
        search_seconds += seconds;
        if let Some(td) = found.and_then(|e| e.decomposition(&graph)) {
            layouts.weigh(&td, &numberings[0])?;
        }
    }
    let (cost, top, width, num_bags) = layouts.best.expect("a numbering has a decomposition");
    let widest_facts = groups
        .groups
        .iter()
        .map(|(_, facts)| facts.len())
        .max()
        .unwrap_or(0);
    let widest_inputs = groups.inputs.iter().map(Vec::len).max().unwrap_or(0);
    let (candidates, weigh_seconds) = (layouts.candidates, layouts.weigh_seconds);
    let note = format!(
        "families: {} families in {} groups over {} indicators, {} stray facts; td over the groups width {width}{}, {num_bags} bags, \
         {num_edges} edges, layout cost {cost:.3e} (the least of {candidates}); the most copies one group reads {widest_inputs}, \
         the most facts in one group {widest_facts}, {} branches of {} groups; min-fill {min_fill_units} units charged, \
         {min_fill_seconds:.3}s; searched {search_units} units charged, {search_seconds:.3}s, laid out and weighed \
         {weigh_seconds:.3}s",
        groups.families,
        num_groups,
        gp.encoding.indicators.len(),
        stray.len(),
        lower_bound.map_or(String::new(), |b| format!(" (treewidth at least {b})")),
        num_items - num_groups,
        items.branches.iter().filter(|b| !b.is_empty()).count(),
    );
    Ok((
        layouts.forest,
        top,
        note,
        Estimate {
            pairs: cost,
            search_units: search_cost,
        },
    ))
}

/// The lists [`Blocks::of`] fills for each group, kept from one group to
/// the next with the room they have grown to, rather than made anew and
/// grown from nothing for each.
struct Blocks {
    scratch: NestScratch,
    group_reader: AtomMarks,
    fact_reader: AtomMarks,
    /// The rows, and per context and row the one before it with the same
    /// hash.
    rows: Rows,
    same_context: Vec<u32>,
    same_row: Vec<usize>,
    /// The row and head of each rule, gathered by row at the end, and the
    /// hubs' groups each row reads.
    row_heads: Vec<(usize, AtomId)>,
    row_hubs: Vec<(usize, usize)>,
    /// What a rule reads, from one pass over its literals: its facts with
    /// their ranks, its copies as it reads them, each a word `atom << 1 |
    /// sign` to be compared with the rule before's, and the hubs' groups,
    /// not a pass each with the facts' ranks looked up in every comparison.
    ranked: Vec<(u32, AtomId)>,
    facts: Vec<AtomId>,
    read: Vec<u64>,
    hubs: Vec<usize>,
    copies: Vec<(AtomId, bool)>,
    /// The reading and the facts of the rule before: the rules of one row
    /// come one after another and read alike, and a rule that reads as the
    /// one before is found without a lookup.
    last_read: Vec<u64>,
    last_facts: Vec<AtomId>,
}

impl Blocks {
    fn new(num_atoms: usize, num_groups: usize) -> Blocks {
        Blocks {
            scratch: NestScratch::new(num_atoms),
            group_reader: AtomMarks::new(num_groups),
            fact_reader: AtomMarks::new(num_atoms),
            rows: Rows::default(),
            same_context: Vec::new(),
            same_row: Vec::new(),
            row_heads: Vec::new(),
            row_hubs: Vec::new(),
            ranked: Vec::new(),
            facts: Vec::new(),
            read: Vec::new(),
            hubs: Vec::new(),
            copies: Vec::new(),
            last_read: Vec::new(),
            last_facts: Vec::new(),
        }
    }

    /// Group `g`'s block ([`nest`]) over what each of its rules reads, or
    /// its branches' blocks ([`Branch`]); a hub's group is its facts in a
    /// balanced tree.
    fn of(
        &mut self,
        gp: &GroundProgram,
        groups: &Groups,
        forest: &mut Forest,
        g: usize,
    ) -> (Option<usize>, Vec<Branch>) {
        let Blocks {
            scratch,
            group_reader,
            fact_reader,
            rows,
            same_context,
            same_row,
            row_heads,
            row_hubs,
            ranked,
            facts,
            read,
            hubs,
            copies,
            last_read,
            last_facts,
        } = self;
        let Groups {
            groups,
            group_rules,
            inputs,
            owner,
            hub_group,
            link_of,
            row_kind,
            kind,
            ..
        } = groups;
        if group_rules[g].is_empty() && groups[g].0.is_empty() {
            let leaves: Vec<usize> = groups[g].1.iter().map(|&f| forest.leaf(f)).collect();
            return (forest.balanced(&leaves), Vec::new());
        }
        scratch.start(&groups[g].1);
        scratch.mark.next();
        for &v in &inputs[g] {
            scratch.mark.set(v, 0);
        }
        // Contexts by a hash of their copies and rows by a hash of their
        // facts and context, compared on a match, so that neither is copied
        // to be looked up; each is built in `facts` and `copies` and copied
        // once it is new. A rule that reads its copies as the rule before
        // has that rule's context (below), and only another reading is
        // sorted to be looked up: such a reading is nearly always one not
        // seen before.
        fn hash<T: std::hash::Hash>(value: &T) -> u64 {
            use std::hash::Hasher;
            let mut h = rustc_hash::FxHasher::default();
            value.hash(&mut h);
            h.finish()
        }
        // The last context and row with each hash, at most one a rule of
        // the group, for which they are allocated.
        let rules = group_rules[g].len();
        let mut context_by: FxHashMap<u64, u32> =
            FxHashMap::with_capacity_and_hasher(rules, Default::default());
        let mut row_by: FxHashMap<u64, usize> =
            FxHashMap::with_capacity_and_hasher(rules, Default::default());
        rows.clear();
        same_context.clear();
        same_row.clear();
        row_heads.clear();
        row_hubs.clear();
        // The context and the row of the rule before.
        let mut before: Option<(u32, usize)> = None;
        for &ri in &group_rules[g] {
            let Some((body, left_out)) = row_body(gp, row_kind, ri) else {
                continue;
            };
            ranked.clear();
            read.clear();
            hubs.clear();
            // Only a fact other than an indicator has a rank or is a hub,
            // and only an indicator is a copy the group reads.
            for l in body {
                let a = l.atom();
                match kind[a as usize] {
                    FACT => {
                        if let Some(rank) = scratch.rank.get(a) {
                            ranked.push((rank, a));
                        }
                        let h = hub_group[a as usize];
                        if h != usize::MAX {
                            hubs.push(h);
                        }
                    }
                    INDICATOR if reads(link_of, l, left_out) && scratch.mark.get(a).is_some() => {
                        read.push(u64::from(a) << 1 | u64::from(l.is_positive()));
                    }
                    _ => {}
                }
            }
            // A fact's rank is its place among the group's facts, one each.
            // A rule's facts may come in the order of their ranks already,
            // each once, and only the others are sorted.
            if !ranked.is_sorted_by(|x, y| x < y) {
                ranked.sort_unstable();
                ranked.dedup();
            }
            facts.clear();
            facts.extend(ranked.iter().map(|&(_, f)| f));
            if facts.is_empty() && read.is_empty() {
                continue;
            }
            let x = match before {
                Some((x, _)) if read == last_read => x,
                _ => {
                    copies.clear();
                    copies.extend(read.iter().map(|&w| ((w >> 1) as AtomId, w & 1 == 1)));
                    copies.sort_unstable();
                    copies.dedup();
                    let h = hash(copies);
                    let last = context_by.get(&h).copied().unwrap_or(u32::MAX);
                    let mut x = last;
                    while x != u32::MAX && rows.context_copies(x) != copies.as_slice() {
                        x = same_context[x as usize];
                    }
                    if x == u32::MAX {
                        x = rows.num_contexts() as u32;
                        rows.contexts.extend_from_slice(copies);
                        rows.context_ends.push(rows.contexts.len());
                        context_by.insert(h, x);
                        same_context.push(last);
                    }
                    std::mem::swap(read, last_read);
                    x
                }
            };
            let i = match before {
                Some((y, i)) if y == x && facts == last_facts => i,
                _ => {
                    let h = hash(&(&**facts, x));
                    let last = row_by.get(&h).copied().unwrap_or(usize::MAX);
                    let mut i = last;
                    while i != usize::MAX
                        && (rows.context[i] != x || rows.facts(i) != facts.as_slice())
                    {
                        i = same_row[i];
                    }
                    if i == usize::MAX {
                        i = rows.len();
                        rows.facts.extend_from_slice(facts);
                        rows.fact_ends.push(rows.facts.len());
                        rows.context.push(x);
                        row_by.insert(h, i);
                        same_row.push(last);
                    }
                    std::mem::swap(facts, last_facts);
                    i
                }
            };
            before = Some((x, i));
            row_heads.push((i, gp.rules[ri].head));
            row_hubs.extend(hubs.iter().map(|&h| (i, h)));
        }
        // A stable sort keeps each row's heads in the order they were read.
        row_heads.sort_by_key(|&(i, _)| i);
        let mut at = 0;
        for i in 0..rows.len() {
            while at < row_heads.len() && row_heads[at].0 == i {
                rows.heads.push(row_heads[at].1);
                at += 1;
            }
            rows.head_ends.push(rows.heads.len());
        }
        let rows: &Rows = rows;
        scratch.context.reserve(rows.num_contexts());
        let branches = branches_of(rows, owner, row_hubs, g, group_reader, fact_reader);
        if branches.is_empty() {
            let members = (0..rows.len()).collect();
            return (nest(forest, rows, members, scratch), Vec::new());
        }
        let branches = branches
            .into_iter()
            .map(|(members, parents)| {
                let mut facts: Vec<AtomId> = members
                    .iter()
                    .flat_map(|&i| rows.facts(i).iter().copied())
                    .collect();
                facts.sort_unstable();
                facts.dedup();
                let mut copies: Vec<AtomId> = members
                    .iter()
                    .flat_map(|&i| rows.context_copies(rows.context[i]).iter().map(|&(c, _)| c))
                    .collect();
                copies.sort_unstable();
                copies.dedup();
                Branch {
                    block: nest(forest, rows, members, scratch),
                    parents,
                    facts,
                    copies,
                }
            })
            .collect();
        (None, branches)
    }
}

/// The facts of `gp` that none of the `blocks` and `branches` holds, each a
/// leaf of `forest`. A group can retain a fact in its family after
/// specialization removed the ordinary rule that placed it in a block. Count
/// actual leaves, so variables read only by observations or links still get
/// a stray leaf.
fn stray_facts(
    gp: &GroundProgram,
    forest: &mut Forest,
    blocks: &[Option<usize>],
    branches: &[Vec<Branch>],
) -> Vec<usize> {
    let mut placed = vec![false; gp.num_atoms()];
    for &t in blocks
        .iter()
        .flatten()
        .chain(branches.iter().flatten().filter_map(|b| b.block.as_ref()))
    {
        for a in forest.leaves_of(t) {
            placed[a as usize] = true;
        }
    }
    gp.facts()
        .map(|(a, _)| a)
        .filter(|&a| !placed[a as usize])
        .map(|a| forest.leaf(a))
        .collect()
}

/// What each leaf belongs to, for pairing a bag's items and weighing a
/// layout, and each group's value count: the number of atoms of it that
/// have copies, and one more for none of them, at least two.
fn scope(gp: &GroundProgram, groups: &Groups, branches: &[Vec<Branch>]) -> (Scope, Vec<f64>) {
    let num_atoms = gp.num_atoms();
    // The atoms of a table as clauses are its links' group's, not that of
    // the families that read them through their first reader's copy.
    // A link that holds clauses has a rule per row of its table, and its
    // atoms are set once per group that has its rules.
    let mut group_of_atom = vec![usize::MAX; num_atoms];
    let mut link_done = vec![usize::MAX; num_atoms];
    for (g, rules) in groups.group_rules.iter().enumerate() {
        for &ri in rules {
            let head = gp.rules[ri].head;
            if groups.is_clause[head as usize] {
                if link_done[head as usize] == g {
                    continue;
                }
                link_done[head as usize] = g;
                for &k in &groups.atoms_of_clause[&head] {
                    group_of_atom[k as usize] = g;
                }
            } else if !groups.is_link[head as usize] && !groups.is_clause_atom[head as usize] {
                group_of_atom[head as usize] = g;
            }
        }
    }
    let mut copied: Vec<FxHashSet<AtomId>> = vec![FxHashSet::default(); groups.groups.len()];
    let mut scope = Scope::new(num_atoms);
    for ind in &gp.encoding.indicators {
        // A copy of a bit of a set read in binary is a copy of the set's
        // values to the group of their formulas, or of their table as
        // clauses.
        let code = groups
            .code_of
            .get(&ind.atom)
            .map(|&c| &gp.encoding.codes[c]);
        let g = match (group_of_atom[ind.atom as usize], code) {
            (usize::MAX, Some(code)) => group_of_atom[code.values[0] as usize],
            (g, _) => g,
        };
        if g != usize::MAX {
            match code {
                Some(code) => copied[g].extend(&code.values),
                None => {
                    copied[g].insert(ind.atom);
                }
            }
            scope.set(ind.var, [g]);
        }
    }
    for (g, (_, facts)) in groups.groups.iter().enumerate() {
        for &f in facts {
            scope.set(
                f,
                std::iter::once(g).chain(groups.parents[g].iter().copied()),
            );
        }
    }
    // A branch's facts belong to its group and the groups the branch reads,
    // and its copies to its group as well as the one that defines them: a
    // subtree that holds some of a group's branches but not all is open on
    // the disjunctions they have made so far.
    for (g, bs) in branches.iter().enumerate() {
        for b in bs {
            for &f in &b.facts {
                scope.set(f, std::iter::once(g).chain(b.parents.iter().copied()));
            }
            for &v in &b.copies {
                match groups.owner[v as usize] {
                    h if h == usize::MAX || h == g => scope.set(v, [g]),
                    h => scope.set(v, [h, g]),
                }
            }
        }
    }
    let values = copied.iter().map(|c| (c.len() + 1).max(2) as f64).collect();
    (scope, values)
}

/// The items laid out: the groups, then the branches of each group that
/// has them, each with its block and the items it is placed with.
struct Items {
    block: Vec<Option<usize>>,
    parents: Vec<Vec<usize>>,
    /// Each group's branches' items.
    branches: Vec<Vec<usize>>,
}

impl Items {
    /// The groups' items, with their `blocks` and `parents`, and their
    /// `branches`'.
    fn of(parents: &[Vec<usize>], blocks: Vec<Option<usize>>, branches: &[Vec<Branch>]) -> Items {
        let num_groups = blocks.len();
        let mut block = blocks;
        let mut item_parents: Vec<Vec<usize>> = (0..num_groups)
            .map(|g| {
                if branches[g].is_empty() {
                    parents[g].clone()
                } else {
                    Vec::new()
                }
            })
            .collect();
        let mut branch_items: Vec<Vec<usize>> = vec![Vec::new(); num_groups];
        for (g, bs) in branches.iter().enumerate() {
            for b in bs {
                branch_items[g].push(block.len());
                block.push(b.block);
                item_parents.push(b.parents.clone());
            }
        }
        Items {
            block,
            parents: item_parents,
            branches: branch_items,
        }
    }

    /// The edges of the items' graph, each once: each group with its
    /// `parents` a clique, the moral graph of a Bayesian network. A group
    /// with branches is the last of a chain of them instead, each branch a
    /// clique with the groups it reads.
    fn edges(&self, parents: &[Vec<usize>]) -> Vec<(usize, usize)> {
        let mut cliques: Vec<Vec<usize>> = Vec::new();
        for (g, ps) in parents.iter().enumerate() {
            if self.branches[g].is_empty() {
                cliques.push(std::iter::once(g).chain(ps.iter().copied()).collect());
                continue;
            }
            for (k, &b) in self.branches[g].iter().enumerate() {
                cliques.push(
                    std::iter::once(b)
                        .chain(self.parents[b].iter().copied())
                        .collect(),
                );
                cliques.push(vec![b, self.branches[g].get(k + 1).copied().unwrap_or(g)]);
            }
        }
        let mut seen_edge = FxHashSet::default();
        let mut edges = Vec::new();
        for clique in &cliques {
            for (i, &u) in clique.iter().enumerate() {
                for &v in &clique[i + 1..] {
                    let e = (u.min(v), u.max(v));
                    if seen_edge.insert(e) {
                        edges.push(e);
                    }
                }
            }
        }
        edges
    }

    /// The numberings of the items decomposed: the groups in their order and
    /// by their first families, `first_root`, where that differs, each
    /// group's branches just before it.
    fn numberings(&self, first_root: &[usize]) -> Vec<Vec<usize>> {
        let num_groups = self.branches.len();
        let mut by_root: Vec<usize> = (0..num_groups).collect();
        by_root.sort_by_key(|&g| first_root[g]);
        let mut numberings: Vec<Vec<usize>> = vec![(0..num_groups).collect()];
        if by_root != numberings[0] {
            numberings.push(by_root);
        }
        for numbering in &mut numberings {
            *numbering = numbering
                .iter()
                .flat_map(|&g| self.branches[g].iter().copied().chain(std::iter::once(g)))
                .collect();
        }
        numberings
    }

    /// The values of each item, by the groups' `values`. A branch's are its
    /// group's: a subtree holding some of the group's branches is open on
    /// the group's copies.
    fn values(&self, values: &[f64]) -> Vec<u64> {
        let mut item_values = vec![0u64; self.block.len()];
        for (g, branches) in self.branches.iter().enumerate() {
            item_values[g] = values[g] as u64;
            for &b in branches {
                item_values[b] = item_values[g];
            }
        }
        item_values
    }
}

/// The layouts of the items over decompositions of their graph
/// ([`Layouts::lay`]), each weighed by its layout cost, and the cheapest.
struct Layouts<'a> {
    forest: Forest,
    items: &'a Items,
    interfaces: Interfaces<'a>,
    /// What each item's block is open on.
    block_open: Vec<Option<Open>>,
    /// What the blocks' joins are charged.
    weighed: Weighed,
    /// The facts no family reads.
    stray_top: Option<usize>,
    /// The cheapest layout so far: its cost, its top, and its
    /// decomposition's width and bags.
    best: Option<(f64, usize, u32, usize)>,
    /// How many layouts were weighed, and the seconds it took, for
    /// `--stats`.
    candidates: usize,
    weigh_seconds: f64,
}

impl Layouts<'_> {
    /// The layout over a decomposition whose vertex `v` is item
    /// `item_at[v]`. Rooted at bag 0, each item's block goes in the highest
    /// bag holding it with its parents, which a decomposition has, and a
    /// bag's node joins its blocks and its children's nodes
    /// ([`Interfaces::paired`]).
    fn lay(&mut self, td: &goatd::TreeDecomposition, item_at: &[usize]) -> Option<usize> {
        let items = self.items;
        let num_items = items.block.len();
        let bags = td.bags();
        let adjacency = td.adjacency();
        let mut parent_bag = vec![usize::MAX; bags.len()];
        let mut preorder = Vec::with_capacity(bags.len());
        let mut visited = vec![false; bags.len()];
        let mut tops = Vec::new();
        for start in 0..bags.len() {
            if visited[start] {
                continue;
            }
            visited[start] = true;
            tops.push(start);
            let mut stack = vec![start];
            while let Some(b) = stack.pop() {
                preorder.push(b);
                for &c in &adjacency[b] {
                    if !visited[c] {
                        visited[c] = true;
                        parent_bag[c] = b;
                        stack.push(c);
                    }
                }
            }
        }
        let mut blocks_at: Vec<Vec<usize>> = vec![Vec::new(); bags.len()];
        let mut placed_item = vec![false; num_items];
        for &b in &preorder {
            let held: FxHashSet<usize> = bags[b]
                .vertices()
                .iter()
                .map(|&v| item_at[v as usize])
                .collect();
            for &v in bags[b].vertices() {
                let i = item_at[v as usize];
                if !placed_item[i] && items.parents[i].iter().all(|h| held.contains(h)) {
                    placed_item[i] = true;
                    blocks_at[b].push(i);
                }
            }
        }
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); bags.len()];
        for &b in &preorder {
            if parent_bag[b] != usize::MAX {
                children[parent_bag[b]].push(b);
            }
        }
        let mut subtree: Vec<Option<(usize, Open)>> = vec![None; bags.len()];
        for &b in preorder.iter().rev() {
            let mut joined: Vec<(usize, Open)> = blocks_at[b]
                .iter()
                .filter_map(|&i| Some((items.block[i]?, self.block_open[i].clone()?)))
                .collect();
            joined.extend(children[b].iter().filter_map(|&c| subtree[c].take()));
            subtree[b] = self.interfaces.paired(&mut self.forest, joined);
        }
        let mut top_items: Vec<(usize, Open)> =
            tops.iter().filter_map(|&t| subtree[t].take()).collect();
        // Items no bag placed (a decomposition places them all, but nothing
        // above depends on it), and facts no family reads.
        for i in (0..num_items).filter(|&i| !placed_item[i]) {
            top_items.extend(items.block[i].zip(self.block_open[i].clone()));
        }
        top_items.extend(self.stray_top.map(|t| (t, Open::default())));
        self.interfaces
            .paired(&mut self.forest, top_items)
            .map(|(top, _)| top)
    }

    /// The layout over `td`, its items numbered by `item_at`, weighed, and
    /// kept where it is the cheapest so far.
    fn weigh(&mut self, td: &goatd::TreeDecomposition, item_at: &[usize]) -> Result<(), String> {
        let started = std::time::Instant::now();
        let top = self
            .lay(td, item_at)
            .ok_or("the families vtree has no leaf")?;
        let cost = self
            .interfaces
            .layout_cost(&self.forest, top, &self.weighed);
        self.weigh_seconds += started.elapsed().as_secs_f64();
        self.candidates += 1;
        if self.best.is_none_or(|(c, ..)| cost < c) {
            self.best = Some((cost, top, td.treewidth(), td.bags().len()));
        }
        Ok(())
    }
}
