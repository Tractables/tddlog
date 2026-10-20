//! Readers of one clause table tied to each other's copies.
//!
//! Every reader of a clause table has a link of its own ([`crate::indicators`]):
//! the table's rows with the reader's copy of each row's value, exactly one
//! of the reader's copies, and the constraints on the table's inputs. The
//! rows partition the inputs, so in every world the links allow, each
//! reader's copies are the one value the table's rows give, and two
//! readers' links together hold exactly where one of them holds and the
//! two readers' copies are equal:
//!
//! ```text
//! link_A ∧ link_B  ≡  link_A ∧ ⋀_v (c_B,v ⇔ c_A,v) ∧ exactly-one(c_A).
//! ```
//!
//! So a reader `B` whose link the evidence walk would put at a vtree node
//! above, or at, the node of another reader `A`'s link is tied to `A`'s
//! copies there in place of its link: each node's conjunction is the same
//! function, and `B`'s link is not built. Put at the lowest node over the
//! two readers' copies, as the walk puts a factor, the tie would meet
//! `B`'s copies where they are still free of the table's inputs, and the
//! nodes between would hold their product with the rest.
//!
//! The tie spares the compile the link's loop, and the walk joins the tie
//! where it joined the link, weighed as the link (`crate::query`); what a
//! join costs changes either way. Two kinds of link are built still:
//!
//! - A link of fewer than [`TIE_FROM`] clauses, whose loop costs little:
//!   tying small links costs the networks more in the walk than the
//!   compile saves (bn_diabetes, bn_pigs, bn_munin4), and a larger bound
//!   loses bn_link's ties.
//! - A link at the root, whose own factors may span the whole vtree and
//!   which the walk conjoins among themselves before its largest child: a
//!   tie there meets the other factors before the link it stands on, which
//!   is below (on bn_pathfinder, whose root holds ties of fault's readers,
//!   the evidence's last join grows many times so).
//!
//! Without projection every link is on the global vtree, at the root, and
//! none is tied.

use rustc_hash::FxHashMap;

use tididi::Tdd;
use tididi::vtree::{VarId, VtreeIdx};

use super::{CompileError, Projection, TddBackend, VtreeScope};
use crate::ground::{AtomId, GroundProgram, RulesByHead};

/// The fewest clauses of a link tied to another reader's copies.
const TIE_FROM: usize = 64;

/// A reader's link replaced by equalities to another reader's copies.
pub(crate) struct Tie {
    /// The other reader's link, which the tie stands on.
    pub(crate) reference: AtomId,
    /// The reader's copy of each value, and the other reader's.
    pairs: Vec<(VarId, VarId)>,
    /// The variables of the link the tie replaces, which say where the walk
    /// puts it.
    link_vars: Vec<VarId>,
}

impl Tie {
    /// The vtree node the walk puts the tie at: that of the link it replaces.
    pub(crate) fn node(&self, backend: &TddBackend) -> VtreeIdx {
        backend.lowest_over_vars(&self.link_vars)
    }

    /// The variables the tie reads, ascending.
    pub(crate) fn vars(&self) -> Vec<VarId> {
        let mut vars: Vec<VarId> = self.pairs.iter().flat_map(|&(b, a)| [b, a]).collect();
        vars.sort_unstable();
        vars
    }

    /// The tie on `scope`, which holds [`Tie::vars`]: each pair's copies
    /// equal and the other reader's copies exactly one.
    pub(crate) fn formula(
        &self,
        backend: &TddBackend,
        scope: &VtreeScope,
    ) -> Result<Tdd, CompileError> {
        let vars = self.vars();
        let local = |v: VarId| VarId(vars.binary_search(&v).expect("a tie's variable") as u32 + 1);
        let cubes: Vec<Vec<tididi::Literal>> = self
            .pairs
            .iter()
            .flat_map(|&(b, a)| {
                [
                    vec![
                        tididi::Literal::pos(local(b)),
                        tididi::Literal::neg(local(a)),
                    ],
                    vec![
                        tididi::Literal::neg(local(b)),
                        tididi::Literal::pos(local(a)),
                    ],
                ]
            })
            .collect();
        let group = (self.pairs.iter().map(|&(_, a)| local(a)).collect(), true);
        Ok(backend
            .none_of(scope, &vars, &cubes, &[group], u64::MAX)?
            .expect("no bound on the work"))
    }
}

/// The readers' links of the program's clause tables that are tied to
/// another reader's copies on `backend`'s vtree.
///
/// A link with copies of two values or more is a clause table's link for
/// one reader, and the links whose copies are of the same atoms are the
/// readers of one table. Taken deepest first, a reader whose link's node
/// is at or above that of a reader kept is tied to the first such.
pub(crate) fn ties(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    var_of: &[Option<VarId>],
    backend: &TddBackend,
    projection: Projection,
) -> FxHashMap<AtomId, Tie> {
    let mut ties = FxHashMap::default();
    let Some(global_from) = projection.global_from(var_of.iter().flatten().count() as u32) else {
        return ties;
    };
    let mut copies: FxHashMap<AtomId, Vec<(AtomId, VarId)>> = FxHashMap::default();
    for ind in &gp.encoding.indicators {
        copies.entry(ind.link).or_default().push((
            ind.atom,
            var_of[ind.var as usize].expect("a copy is a fact"),
        ));
    }
    let mut tables: FxHashMap<Vec<AtomId>, Vec<AtomId>> = FxHashMap::default();
    for (&link, of) in &mut copies {
        if of.len() >= 2 {
            of.sort_unstable();
            tables
                .entry(of.iter().map(|&(k, _)| k).collect())
                .or_default()
                .push(link);
        }
    }
    let vtree = backend.vtree();
    // The link each variable was last gathered for: a link's rules read
    // its table's switches and copies over and over, and this finds them
    // without sorting the list of every literal's variable.
    let mut gathered = vec![
        AtomId::MAX;
        var_of
            .iter()
            .flatten()
            .map(|v| v.0 as usize + 1)
            .max()
            .unwrap_or(0)
    ];
    let depth = |mut t: VtreeIdx| {
        let mut d = 0;
        while let Some(p) = vtree.node(t).parent() {
            t = p;
            d += 1;
        }
        d
    };
    for links in tables.into_values().filter(|links| links.len() >= 2) {
        // The readers whose links may be tied or be tied to: a link at the
        // root is neither, since no other's node is above it.
        let mut readers: Vec<(AtomId, Vec<VarId>, VtreeIdx)> = links
            .into_iter()
            .filter(|&link| {
                by_head[link as usize]
                    .iter()
                    .map(|&ri| gp.rules[ri].clauses())
                    .sum::<usize>()
                    >= TIE_FROM
            })
            .filter_map(|link| {
                let mut vars: Vec<VarId> = Vec::new();
                for l in by_head[link as usize]
                    .iter()
                    .flat_map(|&ri| &gp.rules[ri].body)
                {
                    let v = var_of[l.atom() as usize].expect("a link reads facts alone");
                    if std::mem::replace(&mut gathered[v.0 as usize], link) != link {
                        vars.push(v);
                    }
                }
                vars.sort_unstable();
                // A link reads facts alone: its cone is its variables.
                if vars.len() >= global_from {
                    return None;
                }
                let node = backend.lowest_over_vars(&vars);
                (node != vtree.root()).then_some((link, vars, node))
            })
            .collect();
        readers.sort_by_key(|&(link, _, node)| (std::cmp::Reverse(depth(node)), link));
        let mut kept: Vec<(AtomId, VtreeIdx)> = Vec::new();
        for (link, link_vars, node) in readers {
            match kept.iter().find(|&&(_, at)| vtree.lca(at, node) == node) {
                Some(&(reference, _)) => {
                    let pairs = copies[&link]
                        .iter()
                        .zip(&copies[&reference])
                        .map(|(&(_, b), &(_, a))| (b, a))
                        .collect();
                    ties.insert(
                        link,
                        Tie {
                            reference,
                            pairs,
                            link_vars,
                        },
                    );
                }
                None => kept.push((link, node)),
            }
        }
    }
    ties
}
