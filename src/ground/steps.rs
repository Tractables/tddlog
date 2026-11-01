//! Chains of steps written as balanced trees of their compositions.
//!
//! An atom `a` read once in the program, positively, by a rule of `h`, and
//! the only such atom `h` reads, makes `h` a step on `a`:
//! `h = (a ∧ A) ∨ B`, where `A` is the rest of the body that reads `a` and
//! `B` the disjunction of `h`'s other bodies. A circuit that a
//! bit-blasting front end builds folds a sequence of tests into one value
//! step by step, a decision list: the weekend circuit that Alea exports
//! holds 42 chains of 4 096 to 8 191 such atoms. Compiled in program
//! order, each step builds its formula from the whole of the step's
//! before it, and a chain costs the square of its length.
//!
//! Steps compose: `(A1, B1)` then `(A2, B2)` is `(A1 ∧ A2, (B1 ∧ A2) ∨ B2)`,
//! and the composition is associative. A chain `a_0 → a_1 → … → a_m` of at
//! least [`MIN_STEPS`] steps is written as a balanced tree of the
//! compositions of its steps, a node's `A` and `B` each an atom of its own
//! (`$st`) where it is not a literal, true or false. The chain's end keeps
//! its number, with the rules `a_m :- a_0, A` and `a_m :- B`; the atoms
//! between go. A node's formulas cover its own steps, and the chain costs
//! its length times the depth of the tree.
//!
//! Balanced, the weekend at 12 bits and 1 024 pieces took 139 M work units
//! of compile against 656 M, and the full weekend 2.53 G against 21.1 G
//! (the vtree of `decision`, the answer the same).

use crate::syntax::{Atom, Term};

use super::{AtomId, AtomKind, GLit, GroundProgram, GroundRule, deps, simplify};

const NAME: &str = "$st";

/// A chain of fewer steps is left as it is.
const MIN_STEPS: usize = 4;

/// A step or a composition of steps: `x ↦ (x ∧ a) ∨ b`, `a` true where
/// `None`, `b` false where `None`.
#[derive(Clone, Copy)]
struct Step {
    a: Option<GLit>,
    b: Option<GLit>,
}

/// The atoms and rules the compositions make, numbered from `next`.
struct Made {
    next: AtomId,
    rules: Vec<GroundRule>,
}

impl Made {
    fn atom(&mut self, bodies: Vec<Vec<GLit>>) -> GLit {
        let a = self.next;
        self.next += 1;
        self.rules
            .extend(bodies.into_iter().map(|b| GroundRule::new(a, b)));
        GLit::Pos(a)
    }

    /// A literal for the conjunction `body`: true where it is empty.
    fn and(&mut self, body: Vec<GLit>) -> Option<GLit> {
        match body.as_slice() {
            [] => None,
            &[l] => Some(l),
            _ => Some(self.atom(vec![body])),
        }
    }

    /// A literal for the disjunction of `bodies`: false where there are none.
    fn or(&mut self, bodies: Vec<Vec<GLit>>) -> Option<GLit> {
        match bodies.as_slice() {
            [] => None,
            [b] if b.len() == 1 => Some(b[0]),
            _ => Some(self.atom(bodies)),
        }
    }

    /// `l` then `r`.
    fn compose(&mut self, l: Step, r: Step) -> Step {
        let a = match (l.a, r.a) {
            (None, x) | (x, None) => x,
            (Some(x), Some(y)) => self.and(vec![x, y]),
        };
        let mut bodies = Vec::with_capacity(2);
        if let Some(bl) = l.b {
            bodies.push(r.a.map_or_else(|| vec![bl], |ar| vec![bl, ar]));
        }
        if let Some(br) = r.b {
            bodies.push(vec![br]);
        }
        Step {
            a,
            b: self.or(bodies),
        }
    }

    /// The composition of `steps`, in their order, as a balanced tree.
    fn tree(&mut self, steps: &[Step]) -> Step {
        if let [s] = steps {
            return *s;
        }
        let (l, r) = steps.split_at(steps.len() / 2);
        let (l, r) = (self.tree(l), self.tree(r));
        self.compose(l, r)
    }
}

/// Writes the chains of steps of `gp`, as described above, as balanced
/// trees of their compositions, and returns how many it wrote. Applies to
/// an acyclic program without rules of pairs.
pub(super) fn balance(gp: &mut GroundProgram) -> usize {
    if gp.rules.iter().any(|r| r.pairs) {
        return 0;
    }
    let by_head = gp.rules_by_head();
    let sccs = deps::derived_sccs(gp, &by_head);
    if sccs.iter().any(|c| deps::is_cyclic(gp, &by_head, c)) {
        return 0;
    }
    let n = gp.num_atoms();
    // Atoms something outside the rules names keep their formulas.
    let mut named = vec![false; n];
    let outside = gp.named_atoms();
    let indicators = gp
        .encoding
        .indicators
        .iter()
        .flat_map(|i| [i.atom, i.var, i.link]);
    let codes = gp
        .encoding
        .codes
        .iter()
        .flat_map(|c| c.values.iter().chain(&c.bits).copied());
    for a in outside
        .chain(indicators)
        .chain(codes)
        .chain(gp.encoding.siblings.iter().flatten().copied())
    {
        named[a as usize] = true;
    }
    // Per atom, how often the rules read it, and the rule of its one read
    // where that read is positive.
    let mut reads = vec![0u32; n];
    let mut read_in = vec![None; n];
    for (ri, r) in gp.rules.iter().enumerate() {
        for l in &r.body {
            reads[l.atom() as usize] += 1;
            if l.is_positive() {
                read_in[l.atom() as usize] = Some(ri);
            }
        }
    }
    // The step on `a`, `next[a]`, where `a` is the one atom its head reads
    // so.
    let mut next: Vec<Option<AtomId>> = vec![None; n];
    let mut fed = vec![0u32; n];
    for a in 0..n {
        if let Some(ri) = read_in[a]
            && reads[a] == 1
            && !named[a]
            && !gp.is_fact(a as AtomId)
            && !by_head[a].is_empty()
        {
            let h = gp.rules[ri].head;
            next[a] = Some(h);
            fed[h as usize] += 1;
        }
    }
    let mut prev: Vec<Option<AtomId>> = vec![None; n];
    for (a, step) in next.iter_mut().enumerate() {
        match *step {
            Some(h) if fed[h as usize] == 1 => prev[h as usize] = Some(a as AtomId),
            _ => *step = None,
        }
    }
    let mut made = Made {
        next: n as AtomId,
        rules: Vec::new(),
    };
    let mut dropped = vec![false; gp.rules.len()];
    let mut chains = 0;
    for start in 0..n {
        if next[start].is_none() || prev[start].is_some() {
            continue;
        }
        let mut chain = vec![start as AtomId];
        while let Some(h) = next[*chain.last().unwrap() as usize] {
            chain.push(h);
        }
        if chain.len() - 1 < MIN_STEPS {
            continue;
        }
        chains += 1;
        let mut steps = Vec::with_capacity(chain.len() - 1);
        for w in chain.windows(2) {
            let (p, h) = (w[0], w[1]);
            let reading = read_in[p as usize].expect("a step reads its atom");
            let mut rest = gp.rules[reading].body.clone();
            let at = rest
                .iter()
                .position(|&l| l == GLit::Pos(p))
                .expect("the rule reads the atom");
            rest.remove(at);
            let others: Vec<Vec<GLit>> = by_head[h as usize]
                .iter()
                .filter(|&&ri| ri != reading)
                .map(|&ri| gp.rules[ri].body.clone())
                .collect();
            let step = Step {
                a: made.and(rest),
                b: made.or(others),
            };
            steps.push(step);
            for &ri in &by_head[h as usize] {
                dropped[ri] = true;
            }
        }
        let whole = made.tree(&steps);
        let (first, end) = (chain[0], *chain.last().unwrap());
        let mut body = vec![GLit::Pos(first)];
        body.extend(whole.a);
        made.rules.push(GroundRule::new(end, body));
        if let Some(b) = whole.b {
            made.rules.push(GroundRule::new(end, vec![b]));
        }
    }
    if chains == 0 {
        return 0;
    }
    let first_name = gp.stats.stepped;
    for k in 0..made.next as usize - n {
        gp.atoms
            .push(Atom::new(NAME, vec![Term::Int((first_name + k) as i64)]));
        gp.kinds.push(AtomKind::Derived);
    }
    gp.stats.stepped += made.next as usize - n;
    let mut rules: Vec<GroundRule> = std::mem::take(&mut gp.rules)
        .into_iter()
        .zip(dropped)
        .filter(|(_, d)| !d)
        .map(|(r, _)| r)
        .collect();
    rules.extend(made.rules);
    gp.rules = rules;
    gp.structure = std::sync::OnceLock::new();
    simplify::restrict_to_relevant(gp);
    chains
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::{GroundOptions, ground};
    use crate::syntax::parse_program;

    fn program(source: &str) -> GroundProgram {
        let options = GroundOptions {
            views: false,
            ..GroundOptions::default()
        };
        ground(&parse_program(source).unwrap(), &options).unwrap()
    }

    /// A decision list of six steps, `g_k :- g_{k-1}, \+h_k. g_k :- c_k.`,
    /// becomes a tree whose end reads the chain's start and the
    /// compositions.
    #[test]
    fn a_decision_list_becomes_a_tree() {
        let mut source = String::from(
            "0.5::f0. 0.4::f1. 0.3::f2. 0.2::f3. 0.6::f4. 0.7::f5. 0.8::f6.\ng0 :- f0, f1.\n",
        );
        for k in 1..=6 {
            source += &format!(
                "g{k} :- g{}, \\+f{}.\ng{k} :- f{}, f{}.\n",
                k - 1,
                k % 7,
                (k + 2) % 7,
                (k + 4) % 7
            );
        }
        source += "query(g6).\n";
        let mut gp = program(&source);
        assert_eq!(balance(&mut gp), 1, "{gp}");
        let text = gp.to_string();
        assert!(text.contains("g6 :- g0,"), "{text}");
        assert!(!text.contains("g3 :-"), "{text}");
    }

    /// An atom read twice is no step, nor is a chain shorter than
    /// `MIN_STEPS`.
    #[test]
    fn short_chains_and_shared_atoms_stay() {
        let mut gp = program(
            "0.5::f0. 0.4::f1. g0 :- f0. g1 :- g0, f1. g2 :- g1, \\+f0. g3 :- g2. g3 :- f1. query(g3).",
        );
        assert_eq!(balance(&mut gp), 0);
        let mut gp = program(
            "0.5::f0. 0.4::f1. g0 :- f0. g1 :- g0, f1. g2 :- g1, \\+f0. g3 :- g2, f0. g4 :- g3, f1. g5 :- g4, \\+f1. query(g5). query(g2).",
        );
        assert_eq!(
            balance(&mut gp),
            0,
            "g2 is named, which leaves chains of two and three steps"
        );
    }
}
