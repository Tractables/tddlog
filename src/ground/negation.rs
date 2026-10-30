//! Derived atoms read negatively, unfolded under the rule that reads them.
//!
//! A table written by hand as rules reads its earlier values negatively:
//!
//! ```text
//! 0.2::x(1) :- c.   0.5::x(2) :- c, \+x(1).   x(3) :- c, \+x(1), \+x(2).
//! ```
//!
//! is the annotated disjunction `0.2::x(1); 0.4::x(2); 0.4::x(3) :- c.`,
//! whose normalization ([`super::normalize`]) reads the switches of its
//! row instead: `x(1) :- c, s1. x(2) :- c, \+s1, s2. x(3) :- c, \+s1,
//! \+s2.` Read so, the values exclude each other by their switches, and
//! the exclusive sets ([`super::exclusive`]), the cover, the row classes
//! and the clause tables find the table. Read through the values, no two
//! rules of the table clash, and none of them does.
//!
//! [`unfold`] writes the first program as the second. In a rule `h :- B,
//! \+a` of an atom `a` of `h`'s predicate before `h`, an earlier value of
//! its table, where every rule of `a` but one clashes with `B` and that
//! one's literals are `B`'s but one, `l`, `a` holds under `B` exactly
//! where `l` does, and `\+a` is replaced by the negation of `l`. Where
//! every rule of `a` clashes with `B`, `\+a` holds under `B` and is
//! dropped (a row in which `x(1)` has no mass has no rule for it); where
//! the one rule's literals are all `B`'s, the rule never applies and is
//! dropped. Atoms of other predicates stay as read: the bits of a number
//! in a bit-blasted program read the earlier bits negatively too, but
//! each bit is a gate its readers share, and unfolding it under some of
//! them only keeps equal gates apart.
//!
//! Two bodies clash where a literal of one is the negation of a literal of
//! the other, or they read two different atoms of an exclusive set, or two
//! atoms one of which every rule reads the other negatively (`x(1)` and
//! `x(2)` above, before the table is unfolded: the parents of a table are
//! often written so too). The rule of `a` left is found by its context:
//! its literals but the facts that one rule alone reads, a row's switches.
//! The contexts of `a`'s rules are checked to clash pairwise once, and a
//! rule whose context is `B`'s is then the only one of `a`'s that need
//! not clash with `B`.
//!
//! A table without parents written so starts with a fact, `0.2::x(1).
//! 0.625::x(2) :- \+x(1).`, its first value and its first switch at once.
//! Where an atom of a fact's predicate reads it negatively in every rule,
//! the rules that read the fact positively read it through a derived atom
//! of its own, `x'(1) :- x(1)`, which joins the table's exclusive set as
//! the normalization's `x(1) :- s1` does; the fact alone is no atom of a
//! set, and its readers' rows would not clash with those that read the
//! other values.

use rustc_hash::FxHashMap;

use super::program::Lists;
use super::structure::Structure;
use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule};
use crate::syntax::{Args, Atom, Compound};

/// What a negated atom is under the rest of the rule that reads it.
enum Unfolded {
    /// The negation of one literal.
    Literal(GLit),
    /// True: the literal is dropped.
    True,
    /// False: the rule is dropped.
    False,
}

/// In `gp`, whose structure is `structure`, every negated atom unfolded
/// under its rule as described above, the atoms in reverse dependency
/// order, each atom read as it is written.
/// Returns how many literals were unfolded and rules dropped; `gp` is
/// unchanged if none was, and its structure is to be found again if any
/// was.
pub(super) fn unfold(gp: &mut GroundProgram, structure: &Structure) -> usize {
    let n = gp.num_atoms();
    let mut position = vec![usize::MAX; n];
    for (i, scc) in structure.sccs.iter().enumerate() {
        for &a in scc {
            position[a as usize] = i;
        }
    }
    // A negated derived atom of the head's predicate, of an earlier
    // component: an earlier value of the head's table.
    let earlier = |head: AtomId, l: GLit| {
        !l.is_positive()
            && position[l.atom() as usize] < position[head as usize]
            && gp.atoms[l.atom() as usize].signature() == gp.atoms[head as usize].signature()
    };
    // A fact read negatively by a rule of an atom of its predicate, a
    // table's first value ([`unfold`]'s last step).
    let sibling = |head: AtomId, l: GLit| {
        !l.is_positive()
            && gp.is_fact(l.atom())
            && gp.atoms[l.atom() as usize].signature() == gp.atoms[head as usize].signature()
    };
    if !gp.rules.iter().any(|r| {
        r.body
            .iter()
            .any(|&l| earlier(r.head, l) || sibling(r.head, l))
    }) {
        return 0;
    }
    let by_head = &structure.by_head;
    let mut set_of = vec![usize::MAX; n];
    for (i, set) in structure.sets.iter().enumerate() {
        for &a in set {
            set_of[a as usize] = i;
        }
    }
    // The facts one rule alone reads, which no query or evidence names: a
    // row's switches, left out of the contexts.
    let mut readers = vec![0u8; n];
    for l in gp.rules.iter().flat_map(|r| &r.body) {
        let c = &mut readers[l.atom() as usize];
        *c = (*c + 1).min(2);
    }
    for a in gp.named_atoms() {
        readers[a as usize] = 2;
    }
    let once: Vec<bool> = (0..n)
        .map(|a| readers[a] == 1 && gp.is_fact(a as AtomId))
        .collect();
    // The atoms every rule of each atom reads negatively, sorted.
    let mut excluded: Vec<(usize, AtomId)> = Vec::new();
    let mut common: Vec<AtomId> = Vec::new();
    for (h, rules) in by_head.iter().enumerate() {
        let Some((&first, others)) = rules.split_first() else {
            continue;
        };
        common.clear();
        common.extend(
            gp.rules[first]
                .body
                .iter()
                .filter(|l| !l.is_positive())
                .map(|l| l.atom()),
        );
        for &ri in others {
            common.retain(|&x| gp.rules[ri].body.contains(&GLit::Neg(x)));
        }
        common.sort_unstable();
        common.dedup();
        excluded.extend(common.iter().map(|&x| (h, x)));
    }
    let excluded = Lists::new(n, || excluded.iter().copied());
    let excludes = |x: AtomId, y: AtomId| {
        excluded[x as usize].binary_search(&y).is_ok()
            || excluded[y as usize].binary_search(&x).is_ok()
    };
    let clash = |x: &[GLit], y: &[GLit]| {
        x.iter().any(|&l| {
            y.contains(&l.negated())
                || (l.is_positive()
                    && y.iter().any(|&m| {
                        m.is_positive()
                            && m.atom() != l.atom()
                            && ((set_of[l.atom() as usize] != usize::MAX
                                && set_of[l.atom() as usize] == set_of[m.atom() as usize])
                                || excludes(l.atom(), m.atom()))
                    }))
        })
    };
    let context = |body: &[GLit]| {
        let mut c: Vec<GLit> = body
            .iter()
            .copied()
            .filter(|l| !once[l.atom() as usize])
            .collect();
        c.sort_unstable();
        c.dedup();
        c
    };
    // Per atom read negatively, its rules by context, where the contexts
    // clash pairwise; `None` where they do not. Made from the atom's rules
    // as written: the atoms are taken in reverse dependency order, so that
    // every reader of an atom comes before it.
    let mut by_context: FxHashMap<AtomId, Option<FxHashMap<Vec<GLit>, usize>>> =
        FxHashMap::default();
    let mut dropped = vec![false; gp.rules.len()];
    let mut count = 0;
    let mut rest: Vec<GLit> = Vec::new();
    let mut negated: Vec<(usize, usize)> = Vec::new();
    for &h in structure.sccs.iter().rev().flatten() {
        for &ri in &by_head[h as usize] {
            'rule: loop {
                // The latest atom read negatively first: the rules of `a`
                // read the atoms of its table before it, as written, and
                // their contexts are those of the rule once the later ones
                // are unfolded.
                let body = &gp.rules[ri].body;
                negated.clear();
                negated.extend(
                    body.iter()
                        .enumerate()
                        .filter(|&(_, &l)| earlier(h, l))
                        .map(|(k, l)| (position[l.atom() as usize], k)),
                );
                negated.sort_unstable_by(|x, y| y.cmp(x));
                let mut step = None;
                for &(_, k) in &negated {
                    let l = body[k];
                    let a = l.atom();
                    rest.clear();
                    rest.extend(body.iter().copied().filter(|&m| m != l));
                    rest.sort_unstable();
                    rest.dedup();
                    let rules = by_context.entry(a).or_insert_with(|| {
                        let rules: Vec<(Vec<GLit>, usize)> = by_head[a as usize]
                            .iter()
                            .map(|&rj| (context(&gp.rules[rj].body), rj))
                            .collect();
                        let pairwise = rules
                            .iter()
                            .enumerate()
                            .all(|(i, (x, _))| rules[i + 1..].iter().all(|(y, _)| clash(x, y)));
                        pairwise.then(|| rules.into_iter().collect())
                    });
                    let Some(rules) = rules else {
                        continue;
                    };
                    let unfolded = match rules.get(&context(&rest)) {
                        Some(&rj) if !clash(&gp.rules[rj].body, &rest) => {
                            let mut extra = gp.rules[rj]
                                .body
                                .iter()
                                .filter(|m| rest.binary_search(m).is_err());
                            match (extra.next(), extra.next()) {
                                (None, _) => Some(Unfolded::False),
                                (Some(&m), None) => Some(Unfolded::Literal(m.negated())),
                                _ => None,
                            }
                        }
                        Some(_) => Some(Unfolded::True),
                        None => rules
                            .values()
                            .all(|&rj| clash(&gp.rules[rj].body, &rest))
                            .then_some(Unfolded::True),
                    };
                    if let Some(unfolded) = unfolded {
                        step = Some((k, unfolded));
                        break;
                    }
                }
                let Some((k, unfolded)) = step else {
                    break;
                };
                count += 1;
                let body = &mut gp.rules[ri].body;
                match unfolded {
                    Unfolded::Literal(m) if !body.contains(&m) => body[k] = m,
                    Unfolded::Literal(_) | Unfolded::True => {
                        body.remove(k);
                    }
                    Unfolded::False => {
                        dropped[ri] = true;
                        break 'rule;
                    }
                }
            }
        }
    }
    if count > 0 {
        let mut i = 0;
        gp.rules.retain(|_| {
            i += 1;
            !dropped[i - 1]
        });
    }
    // The facts that an atom of their predicate reads negatively in every
    // rule, read positively through an atom of their own.
    let mut value = vec![false; n];
    for (h, facts) in excluded.iter().enumerate() {
        for &f in facts {
            value[f as usize] |=
                gp.is_fact(f) && gp.atoms[f as usize].signature() == gp.atoms[h].signature();
        }
    }
    let mut read = vec![false; n];
    for l in gp.rules.iter().flat_map(|r| &r.body) {
        if l.is_positive() && value[l.atom() as usize] {
            read[l.atom() as usize] = true;
        }
    }
    let mut alias = vec![AtomId::MAX; n];
    let mut aliased = false;
    for f in 0..n {
        if read[f] {
            alias[f] = gp.atoms.len() as AtomId;
            gp.atoms.push(named_after(&gp.atoms[f], "$value"));
            gp.kinds.push(AtomKind::Derived);
            aliased = true;
            count += 1;
        }
    }
    if aliased {
        for l in gp.rules.iter_mut().flat_map(|r| &mut r.body) {
            if let GLit::Pos(f) = *l
                && alias[f as usize] != AtomId::MAX
            {
                *l = GLit::Pos(alias[f as usize]);
            }
        }
        for (f, &a) in alias.iter().enumerate() {
            if a != AtomId::MAX {
                gp.rules
                    .push(GroundRule::new(a, vec![GLit::Pos(f as AtomId)]));
            }
        }
    }
    count
}

/// An atom named after `atom`: of its predicate, its last argument wrapped
/// in `tag`, where it has arguments; `tag` and its name otherwise.
fn named_after(atom: &Atom, tag: &str) -> Atom {
    match atom.args.as_slice().split_last() {
        Some((last, first)) => {
            let mut args = first.to_vec();
            args.push(Compound::term(tag.into(), vec![last.clone()]));
            Atom::new(atom.name.clone(), args)
        }
        None => Atom::new(format!("{tag}_{}", atom.name), Args::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::super::structure::Structure;
    use crate::GroundOptions;

    fn ground(source: &str) -> crate::GroundProgram {
        let program = crate::syntax::parse_program(source).expect("parses");
        let options = GroundOptions {
            views: false,
            ..GroundOptions::default()
        };
        crate::ground::ground(&program, &options).expect("grounds")
    }

    /// The values of a table without parents written as rules, its first a
    /// fact, are one exclusive set, as written as an annotated disjunction.
    #[test]
    fn a_table_without_parents_written_as_rules_is_one_set() {
        let source = "0.2::b(1). 0.625::b(2) :- \\+b(1). b(3) :- \\+b(1), \\+b(2).
            0.5::c :- b(1). 0.25::c :- b(2). 0.75::c :- b(3).
            query(c).";
        let gp = ground(source);
        let sets = Structure::of(&gp).sets;
        assert!(sets.iter().any(|s| s.len() == 3), "{gp}\n{sets:?}");
    }

    /// The bits of a number, each read negatively by the later ones, are
    /// gates their readers share and stay as read.
    #[test]
    fn the_bits_of_a_number_stay_as_read() {
        let source = "0.2::f3. 0.4::f2. 0.5::f1. 0.6::f11.
            b2 :- f2, \\+f3.
            b1 :- f1, \\+f3, \\+b2.
            b1 :- b2, f11, \\+f3.
            zero :- \\+f3, \\+b2, \\+b1.
            0.3::o :- zero. 0.7::o :- b1.
            query(o). query(b2).";
        let gp = ground(source);
        assert_eq!(gp.stats.unfolded, 0, "{gp}");
    }
}
