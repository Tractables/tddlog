//! Ancestor cones: the variables a component's formulas may mention.
//!
//! The formula of a derived atom is built from the fact literals in its own
//! rules and from the formulas of the atoms it depends on, so its support is
//! contained in the *ancestor cone* of its component: the facts reachable from
//! it in the ground dependency graph. Most programs have components whose cone
//! is a small part of the fact set (cora500's largest cone is 17 of 2216
//! facts) and a structural TDD keeps a node
//! at every level of its vtree whether the function depends on it or not. A
//! vtree projected to the cone drops the rest (`docs/design.md`, "Compilation").
//!
//! Cones are read off the grounding as each component starts. They
//! over-approximate the support: a formula may turn out not to use a fact its
//! rules mention. Once a component is compiled, its cone is narrowed to what
//! its formulas depend on ([`Cones::narrow`]), so a component's cone is its
//! own facts and the support of what it reads.

use tididi::vtree::VarId;

use crate::ground::{AtomId, AtomKind, GroundProgram, RulesByHead};

/// The variables of each component's ancestor cone, ascending, indexed as
/// `sccs` is ([`Cones`], every cone held to the end).
#[cfg(test)]
pub(crate) fn component_cones(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    sccs: &[Vec<AtomId>],
    var_of: &[Option<VarId>],
    num_vars: u32,
) -> Vec<Vec<VarId>> {
    let comp_of = component_of(gp.num_atoms(), sccs);
    let mut cones = Cones::new(sccs.len(), num_vars);
    (0..sccs.len())
        .map(|ci| {
            cones
                .of(gp, by_head, sccs, ci, var_of, &comp_of, usize::MAX)
                .expect("no limit")
        })
        .collect()
}

/// The component of each of `n` atoms in `sccs`, `u32::MAX` for the others.
pub(super) fn component_of(n: usize, sccs: &[Vec<AtomId>]) -> Vec<u32> {
    let mut comp_of = vec![u32::MAX; n];
    for (ci, scc) in sccs.iter().enumerate() {
        for &a in scc {
            comp_of[a as usize] = ci as u32;
        }
    }
    comp_of
}

/// The ancestor cones of a program's components, each worked out when its
/// component starts and held until [`Cones::release`] says no later
/// component reads it.
///
/// The components must be asked for in dependency order, as
/// [`crate::ground::deps::derived_sccs`] returns them: a component's cone is
/// the union of its own fact literals and the cones of the components it
/// depends on, which have to be worked out first and not yet released.
///
/// The union is gathered in one bitset over the variables, and only its
/// words that were set are read and cleared after each component, rather
/// than a bitset per component, read, copied and cleared whole. A cone is read
/// into the union by its variables, or as a bitset where it has as many
/// variables as the bitset has words, and a component it depends on
/// through several atoms is read once. A cone at the limit or over it is
/// held as that alone: every cone that reads it is over the limit too.
pub(crate) struct Cones {
    blocks: usize,
    /// The union of the component being worked out, and the words of it
    /// that may be set.
    acc: Vec<u64>,
    touched: Vec<usize>,
    held: Vec<Held>,
    /// The component each component was last read into.
    read_into: Vec<usize>,
    /// Per 1-based variable (`0` unused), how many unreleased components
    /// hold it in their cone plus how many unstarted components mention it
    /// directly in their rules, plus two if it may not be collapsed.
    holders: Vec<u32>,
}

enum Held {
    Released,
    Sparse(Vec<VarId>),
    Dense(Box<[u64]>),
    OverLimit(Box<[u64]>),
}

impl Cones {
    pub(crate) fn new(components: usize, num_vars: u32) -> Self {
        let blocks = (num_vars as usize).div_ceil(64);
        Cones {
            blocks,
            acc: vec![0; blocks],
            touched: Vec::new(),
            held: (0..components).map(|_| Held::Released).collect(),
            read_into: vec![usize::MAX; components],
            holders: Vec::new(),
        }
    }

    /// Initialize [`holders`](Self::holders) over `num_vars` variables from
    /// `gp`'s rules across `sccs` and the variables `pinned` that may not be
    /// collapsed.
    pub(crate) fn init_holders(
        &mut self,
        gp: &GroundProgram,
        by_head: &RulesByHead,
        sccs: &[Vec<AtomId>],
        var_of: &[Option<VarId>],
        num_vars: usize,
        pinned: impl IntoIterator<Item = VarId>,
    ) {
        self.holders.clear();
        self.holders.resize(num_vars + 1, 0);
        for v in pinned {
            let idx = v.0 as usize;
            if idx < self.holders.len() {
                self.holders[idx] = self.holders[idx].saturating_add(2);
            }
        }
        for scc in sccs {
            for &a in scc {
                for &ri in &by_head[a as usize] {
                    for l in &gp.rules[ri].body {
                        let b = l.atom() as usize;
                        if matches!(gp.kinds[b], AtomKind::Fact(_)) {
                            let v = var_of[b].expect("every fact has a variable").0 as usize - 1;
                            let word = &mut self.acc[v / 64];
                            if *word == 0 {
                                self.touched.push(v / 64);
                            }
                            *word |= 1 << (v % 64);
                        }
                    }
                }
            }
            for &k in &self.touched {
                let mut w = self.acc[k];
                self.acc[k] = 0;
                while w != 0 {
                    let bit = w.trailing_zeros() as usize;
                    self.holders[k * 64 + bit + 1] += 1;
                    w &= w - 1;
                }
            }
            self.touched.clear();
        }
    }

    /// Whether every variable in `vars` (a subset of an unreleased
    /// component's cone) is held by that component alone, mentioned by no
    /// unstarted component's rules, and not pinned.
    pub(crate) fn is_private(&self, vars: &[VarId]) -> bool {
        !self.holders.is_empty() && vars.iter().all(|&v| self.holders[v.0 as usize] == 1)
    }

    /// If component `ci`'s held cone has at least two variables and every
    /// variable in it is private ([`is_private`](Self::is_private)), return
    /// its first variable.
    pub(crate) fn private_cone(&self, ci: usize) -> Option<VarId> {
        if self.holders.is_empty() {
            return None;
        }
        match &self.held[ci] {
            Held::Sparse(vars) if vars.len() >= 2 && self.is_private(vars) => Some(vars[0]),
            Held::Dense(words) | Held::OverLimit(words) => {
                let mut first = None;
                let mut count = 0usize;
                for (k, &w0) in words.iter().enumerate() {
                    let mut w = w0;
                    while w != 0 {
                        let bit = w.trailing_zeros() as usize;
                        let v = k * 64 + bit + 1;
                        if self.holders[v] != 1 {
                            return None;
                        }
                        if first.is_none() {
                            first = Some(VarId(v as u32));
                        }
                        count += 1;
                        w &= w - 1;
                    }
                }
                (count >= 2).then_some(first).flatten()
            }
            _ => None,
        }
    }

    /// Decrement [`holders`](Self::holders) for the cone of component `ci`
    /// and mark it [`Held::Released`].
    fn drop_held(&mut self, ci: usize) {
        match std::mem::replace(&mut self.held[ci], Held::Released) {
            Held::Released => {}
            Held::Sparse(old) => {
                if !self.holders.is_empty() {
                    for v in old {
                        self.holders[v.0 as usize] -= 1;
                    }
                }
            }
            Held::Dense(words) | Held::OverLimit(words) => {
                if !self.holders.is_empty() {
                    for (k, &w0) in words.iter().enumerate() {
                        let mut w = w0;
                        while w != 0 {
                            let bit = w.trailing_zeros() as usize;
                            self.holders[k * 64 + bit + 1] -= 1;
                            w &= w - 1;
                        }
                    }
                }
            }
        }
    }

    /// The cone of component `ci`, ascending; `None` where it has `limit`
    /// variables or more, or reads a cone that has.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn of(
        &mut self,
        gp: &GroundProgram,
        by_head: &RulesByHead,
        sccs: &[Vec<AtomId>],
        ci: usize,
        var_of: &[Option<VarId>],
        comp_of: &[u32],
        limit: usize,
    ) -> Option<Vec<VarId>> {
        let Cones {
            blocks,
            acc,
            touched,
            held,
            read_into,
            holders,
        } = self;
        let blocks = *blocks;
        let set = |acc: &mut [u64], touched: &mut Vec<usize>, v: usize| {
            let word = &mut acc[v / 64];
            if *word == 0 {
                touched.push(v / 64);
            }
            *word |= 1 << (v % 64);
        };
        for &a in &sccs[ci] {
            for &ri in &by_head[a as usize] {
                for l in &gp.rules[ri].body {
                    let b = l.atom() as usize;
                    if matches!(gp.kinds[b], AtomKind::Fact(_)) {
                        let v = var_of[b].expect("every fact has a variable").0 as usize - 1;
                        set(acc, touched, v);
                    }
                }
            }
        }
        if !holders.is_empty() {
            for &k in touched.iter() {
                let mut w = acc[k];
                while w != 0 {
                    let bit = w.trailing_zeros() as usize;
                    holders[k * 64 + bit + 1] -= 1;
                    w &= w - 1;
                }
            }
        }
        let mut whole = false;
        let mut over = false;
        'rules: for &a in &sccs[ci] {
            for &ri in &by_head[a as usize] {
                for l in &gp.rules[ri].body {
                    let b = l.atom() as usize;
                    if matches!(gp.kinds[b], AtomKind::Derived) {
                        let cj = comp_of[b] as usize;
                        debug_assert!(
                            cj <= ci,
                            "components come in dependency order, so a body atom's \
                             component is this one or an earlier one",
                        );
                        if cj == ci || std::mem::replace(&mut read_into[cj], ci) == ci {
                            continue;
                        }
                        match &held[cj] {
                            Held::Sparse(vars) => {
                                for &v in vars {
                                    set(acc, touched, v.0 as usize - 1);
                                }
                            }
                            Held::Dense(words) => {
                                for (w, &d) in acc.iter_mut().zip(words.iter()) {
                                    *w |= d;
                                }
                                whole = true;
                            }
                            Held::OverLimit(words) => {
                                over = true;
                                if holders.is_empty() {
                                    break 'rules;
                                }
                                for (w, &d) in acc.iter_mut().zip(words.iter()) {
                                    *w |= d;
                                }
                                whole = true;
                            }
                            Held::Released => {
                                panic!("component {ci} reads the cone of component {cj}, released")
                            }
                        }
                    }
                }
            }
        }
        // A component whose rules mention no fact at all has a constant
        // formula. Its scope would then be empty, which is not a vtree, and
        // giving it a larger one would break the property the embeds rely on:
        // a cone contains the cone of everything it depends on. Seeding it
        // with the first variable keeps that property, because every consumer
        // unions this cone in. A program with no probabilistic fact at all
        // has no variable to seed with; every cone is then empty and the
        // caller keeps the global vtree throughout. A cone read whole has
        // as many variables as there are words, at least one.
        if !over && blocks > 0 && !whole && touched.is_empty() {
            set(acc, touched, 0);
        }
        if whole {
            touched.clear();
            touched.extend(0..blocks);
        } else {
            touched.sort_unstable();
        }
        let count: usize = touched.iter().map(|&k| acc[k].count_ones() as usize).sum();
        let cone = (!over && count < limit).then(|| {
            // The cone's variables, allocated once at their number.
            let mut vars = Vec::with_capacity(count);
            for &k in touched.iter() {
                let mut w = acc[k];
                while w != 0 {
                    let bit = w.trailing_zeros() as usize;
                    vars.push(VarId((k * 64 + bit) as u32 + 1));
                    w &= w - 1;
                }
            }
            vars
        });
        if !holders.is_empty() {
            match &cone {
                Some(vars) => {
                    for &v in vars {
                        holders[v.0 as usize] += 1;
                    }
                }
                None => {
                    for &k in touched.iter() {
                        let mut w = acc[k];
                        while w != 0 {
                            let bit = w.trailing_zeros() as usize;
                            holders[k * 64 + bit + 1] += 1;
                            w &= w - 1;
                        }
                    }
                }
            }
        }
        held[ci] = match &cone {
            None => Held::OverLimit(acc.clone().into_boxed_slice()),
            Some(vars) if vars.len() >= blocks => Held::Dense(acc.clone().into_boxed_slice()),
            Some(vars) => Held::Sparse(vars.clone()),
        };
        for &k in touched.iter() {
            acc[k] = 0;
        }
        touched.clear();
        cone
    }

    /// Hold `vars`, ascending, as the cone of component `ci` in place of the
    /// one worked out for it: the variables its formulas turned out to
    /// depend on, which the components that read it union in instead.
    pub(crate) fn narrow(&mut self, ci: usize, vars: &[VarId]) {
        self.drop_held(ci);
        if !self.holders.is_empty() {
            for &v in vars {
                self.holders[v.0 as usize] += 1;
            }
        }
        self.held[ci] = if vars.len() >= self.blocks {
            let mut words = vec![0u64; self.blocks];
            for v in vars {
                let i = v.0 as usize - 1;
                words[i / 64] |= 1 << (i % 64);
            }
            Held::Dense(words.into_boxed_slice())
        } else {
            Held::Sparse(vars.to_vec())
        };
    }

    /// Forget the cone of component `ci`, which no component still to be
    /// worked out reads.
    pub(crate) fn release(&mut self, ci: usize) {
        self.drop_held(ci);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::deps;

    /// The cones of `source`'s components, its derived atoms as written
    /// (`GroundOptions::views` and `contexts` off).
    fn cones_of(source: &str) -> (GroundProgram, Vec<Vec<AtomId>>, Vec<Vec<VarId>>) {
        let program = crate::syntax::parse_program(source).unwrap();
        let options = crate::ground::GroundOptions {
            views: false,
            contexts: false,
            ..crate::ground::GroundOptions::default()
        };
        let gp = crate::ground::ground(&program, &options).unwrap();
        let order: Vec<AtomId> = (0..gp.num_atoms() as AtomId)
            .filter(|&a| gp.is_fact(a))
            .collect();
        let var_of = crate::tp::fact_variables(&gp, &order);
        let by_head = gp.rules_by_head();
        let sccs = deps::derived_sccs(&gp, &by_head);
        let cones = component_cones(&gp, &by_head, &sccs, &var_of, order.len() as u32);
        (gp, sccs, cones)
    }

    /// The component of the derived atom written `name`.
    fn component(gp: &GroundProgram, sccs: &[Vec<AtomId>], name: &str) -> usize {
        sccs.iter()
            .position(|scc| {
                scc.iter()
                    .any(|&x| gp.atoms[x as usize].to_string() == name)
            })
            .unwrap_or_else(|| {
                let all: Vec<String> = sccs
                    .iter()
                    .flatten()
                    .map(|&x| gp.atoms[x as usize].to_string())
                    .collect();
                panic!("no component for {name}; the derived atoms are {all:?}")
            })
    }

    #[test]
    fn a_chain_grows_its_cone_one_fact_at_a_time() {
        // c depends on b depends on a, one fresh fact each. Every body has
        // two literals: the grounder replaces a derived atom whose one rule
        // is a single fact literal by that fact, and an aliased atom is no
        // longer a component.
        let (gp, sccs, cones) = cones_of(
            "0.5::f0. 0.5::f1. 0.5::f2. 0.5::f3.\n\
             a :- f0, f1.\n\
             b :- a, f2.\n\
             c :- b, f3.\n\
             query(c).\n",
        );
        let named = |name: &str| component(&gp, &sccs, name);
        assert_eq!(cones[named("a")].len(), 2);
        assert_eq!(cones[named("b")].len(), 3);
        assert_eq!(cones[named("c")].len(), 4);
        // And the cone of `c` really is all four, not any four.
        let mut all = cones[named("c")].clone();
        all.dedup();
        assert_eq!(all.len(), 4);
        // Each cone contains the one below it.
        for (small, large) in [("a", "b"), ("b", "c")] {
            let (small, large) = (&cones[named(small)], &cones[named(large)]);
            assert!(
                small.iter().all(|v| large.contains(v)),
                "a cone contains the cone it depends on"
            );
        }
    }

    #[test]
    fn independent_atoms_have_disjoint_cones() {
        let (gp, sccs, cones) = cones_of(
            "0.5::f1. 0.5::f2. 0.5::g1. 0.5::g2.\n\
             a :- f1, g1.\n\
             b :- f2, g2.\n\
             query(a). query(b).\n",
        );
        let named = |name: &str| component(&gp, &sccs, name);
        let (a, b) = (&cones[named("a")], &cones[named("b")]);
        assert_eq!(a.len(), 2);
        assert_eq!(b.len(), 2);
        assert!(
            a.iter().all(|v| !b.contains(v)),
            "nothing depends on both, so the cones are disjoint"
        );
    }

    #[test]
    fn a_cycle_shares_one_cone() {
        // p(1,3) and p(3,1) are one component and see every edge of the cycle.
        let (_, sccs, cones) = cones_of(
            "0.5::e(1,2). 0.5::e(2,3). 0.5::e(3,1).\n\
             p(X,Y) :- e(X,Y).\n\
             p(X,Y) :- e(X,Z), p(Z,Y).\n\
             query(p(1,3)).\n",
        );
        let cyclic = sccs
            .iter()
            .enumerate()
            .find(|(_, scc)| scc.len() > 1)
            .map(|(i, _)| i);
        if let Some(ci) = cyclic {
            assert_eq!(
                cones[ci].len(),
                3,
                "every atom of the cycle sees every edge"
            );
        }
    }

    /// The cones are the facts each component reaches, whether its
    /// dependencies' cones are read variable by variable (fewer variables
    /// than the bitset has words) or whole, over 300 facts.
    #[test]
    fn the_cones_are_the_facts_each_component_reaches() {
        let mut source = String::new();
        for i in 0..300 {
            source += &format!("0.5::f({i}).\n");
        }
        for i in 0..299 {
            source += &format!("a({i}) :- f({i}), f({}).\n", i + 1);
            source += &format!("b({i}) :- a({i}), a({}).\n", i * 7 % 299);
        }
        for i in 0..150 {
            source += &format!("big :- b({i}), b({}).\n", 2 * i);
        }
        source +=
            "p(0) :- p(1), f(10).\np(1) :- p(0), f(20).\np(1) :- b(5), f(30).\ntop :- big, p(0).\n";
        source += "query(a(_)). query(b(_)). query(big). query(p(0)). query(top).\n";
        let (gp, sccs, cones) = cones_of(&source);
        let order: Vec<AtomId> = (0..gp.num_atoms() as AtomId)
            .filter(|&a| gp.is_fact(a))
            .collect();
        let var_of = crate::tp::fact_variables(&gp, &order);
        let by_head = gp.rules_by_head();
        let mut sizes = Vec::new();
        for (scc, cone) in sccs.iter().zip(&cones) {
            let mut reached = std::collections::BTreeSet::new();
            let mut seen = vec![false; gp.num_atoms()];
            let mut stack = scc.clone();
            while let Some(a) = stack.pop() {
                for l in by_head[a as usize]
                    .iter()
                    .flat_map(|&ri| &gp.rules[ri].body)
                {
                    let b = l.atom();
                    if gp.is_fact(b) {
                        reached.insert(var_of[b as usize].unwrap());
                    } else if !std::mem::replace(&mut seen[b as usize], true) {
                        stack.push(b);
                    }
                }
            }
            assert_eq!(cone, &reached.into_iter().collect::<Vec<_>>());
            sizes.push(cone.len());
        }
        assert!(
            sizes.iter().any(|&k| k < 5) && sizes.iter().any(|&k| k > 100),
            "both kinds of cone: {sizes:?}"
        );
    }
}
