//! Tp-compilation: the immediate-consequence operator on parameterized
//! interpretations, with a TDD per ground atom.
//!
//! For a stratified ground program the compiled fixpoint assigns each derived
//! atom `a` a Boolean formula `λ_a` over the probabilistic facts such that the
//! weighted model count of `λ_a` is `P(a)` (Vlasselaer et al. 2015/2016).
//!
//! The ground positive dependency graph is split into strongly connected
//! components processed in topological order. An acyclic component needs a
//! single application of the operator; a cyclic one is iterated to a fixpoint,
//! which the canonical form of minimized TDDs detects exactly. A negative
//! literal always refers to a component that has already converged, so its
//! formula is the negation of that atom's final formula.

mod aft;
pub(crate) mod backend;
mod clauses;
mod compile;
pub(crate) mod cone;
mod factoring;
pub(crate) mod none_of;
mod projector;
mod rows;
mod scope;
mod settle;
mod stats;
mod template;
mod tie;

#[cfg(test)]
#[path = "tests/clause_memo.rs"]
mod clause_memo_tests;
#[cfg(test)]
#[path = "tests/primal_update.rs"]
mod primal_update_tests;
#[cfg(test)]
#[path = "tests/projection.rs"]
mod projection_tests;

pub(crate) use backend::TddBackend;
pub(crate) use backend::{Formula, KeptProjector, Reduce};
#[cfg(test)]
pub(crate) use compile::compile;
pub(crate) use compile::{Compilation, DYNAMIC_FIRST_PAIRS, Horizon};
pub(crate) use compile::{Compiled, fact_variables};
pub(crate) use scope::{ScopeTable, VtreeScope};
pub(crate) use settle::settle;
pub use stats::{
    ClauseMemoStats, CompileStats, NegationCount, NegationSource, Negations, ProjectionStats,
    SearchStats,
};

use std::fmt;

use tididi::OperationError;
use tididi::vtree::VarId;

use crate::ground::AtomId;

#[derive(Debug)]
pub enum CompileError {
    Tdd(OperationError),
    /// Every fact must be mapped to a vtree variable.
    UnmappedFact(AtomId),
    /// The well-founded model is three-valued in some world: this atom's
    /// lower and upper formulas differ, so it has no single probability.
    Unsound(crate::syntax::Atom),
    /// A formula grew past the compile's watch: the finished formula of
    /// `atom`, or an intermediate one while computing it. Not a failure of
    /// the program but of the vtree. `pairs` is the formula's
    /// size, fully reduced, that the watch refused.
    Oversize {
        atom: AtomId,
        pairs: usize,
    },
    /// An evidence or query formula exceeded the trial watch.
    QueryOversize {
        pairs: usize,
    },
    /// The compile, between components, was projected to finish past its
    /// horizon, at `work`, the base-2 logarithm of the work units
    /// (`Compilation::horizon`).
    Projected {
        work: f64,
    },
    /// A literal names a variable outside the scope its formula is built on.
    /// A compiler bug: a component's cone contains the cone of everything it
    /// depends on, so every embed the fixpoint asks for is upward.
    NotInScope(VarId),
    /// A cone projected to no variable at all. There is no empty vtree, and
    /// `cone` seeds an empty cone so this cannot arise from a program.
    EmptyScope,
    /// Copying a formula onto a larger vtree failed.
    Embed(tididi::restructure::EmbedError),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Tdd(e) => write!(f, "TDD operation failed: {e}"),
            CompileError::UnmappedFact(a) => write!(f, "fact atom {a} has no vtree variable"),
            CompileError::Unsound(a) => write!(
                f,
                "atom {a} is undefined in the well-founded model of some possible world (negation through a cycle); no probability"
            ),
            CompileError::Oversize { atom, pairs } => write!(
                f,
                "formula of atom {atom} grew to {pairs} pairs, past the watch"
            ),
            CompileError::QueryOversize { pairs } => write!(
                f,
                "evidence/query formula reached {pairs} pairs beyond the watch"
            ),
            CompileError::Projected { work } => write!(
                f,
                "the compile was projected to finish at 2^{work:.1} work units, past its horizon"
            ),
            CompileError::NotInScope(v) => write!(
                f,
                "variable {} is outside the scope its formula is built on",
                v.0
            ),
            CompileError::EmptyScope => write!(f, "a projected scope holds no variable"),
            CompileError::Embed(e) => {
                write!(f, "embedding a formula on a larger vtree failed: {e}")
            }
        }
    }
}

impl std::error::Error for CompileError {}

impl From<OperationError> for CompileError {
    fn from(e: OperationError) -> Self {
        CompileError::Tdd(e)
    }
}

impl From<tididi::restructure::EmbedError> for CompileError {
    fn from(e: tididi::restructure::EmbedError) -> Self {
        match e {
            tididi::restructure::EmbedError::Operation(e) => CompileError::Tdd(e),
            other => CompileError::Embed(other),
        }
    }
}

/// Which vtree each component's formulas are built on.
///
/// A structural TDD keeps a node at every level of its vtree whether or not
/// the function depends on the variables below it, so a formula over a few
/// facts in a program of thousands is nearly all padding (cora500's largest
/// formula reads 13 of its 2216 facts). Projecting each component's vtree to its
/// ancestor cone removes the rest. Where cones are large the embeds cost more
/// than the levels they save, which is what `max_share` is for; the
/// measurements are in `docs/design.md`, "Compilation".
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Projection {
    /// One vtree over every fact, for every formula in the program.
    Off,
    /// Each component on a vtree projected to its ancestor cone, unless the
    /// cone holds `max_share` or more of the facts, where the component keeps
    /// the global vtree. Cones grow along dependencies, so the components
    /// kept global are closed upwards and no formula is ever embedded from
    /// the global vtree into a smaller one.
    Cones { max_share: f64 },
}

/// The share of the facts at which a cone stops being worth projecting to.
///
/// Projecting below it is faster on most programs and slower on few, and
/// by little, never reaches a larger formula, and is what lets
/// `dice_digitRecognition` finish.
pub(crate) const DEFAULT_MAX_CONE: f64 = 0.5;

impl Default for Projection {
    fn default() -> Self {
        Projection::Cones {
            max_share: DEFAULT_MAX_CONE,
        }
    }
}

impl Projection {
    /// The size of a cone, of `num_vars` facts, from which a component keeps
    /// the global vtree; `None` without projection.
    pub(crate) fn global_from(self, num_vars: u32) -> Option<usize> {
        match self {
            Projection::Off => None,
            Projection::Cones { max_share } => {
                Some(((max_share * f64::from(num_vars)).ceil() as usize).max(1))
            }
        }
    }
}

impl fmt::Display for Projection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Projection::Off => f.write_str("off"),
            Projection::Cones { max_share } => {
                write!(f, "cones up to {:.0}% of the facts", max_share * 100.0)
            }
        }
    }
}

/// Which polarity of each atom's formula the fixpoint maintains.
///
/// `or` on TDDs is implemented as three negations around a conjunction, while
/// `and`, `and_clause` and `negate` are primitive. The primal operator is a
/// disjunction over rules; the dual maintains `μ_a = ¬λ_a`, for which the
/// operator is `μ_a = ⋀_r ¬body_r`, a conjunction over rules with one negation
/// per rule that mentions a derived atom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Formulation {
    Primal,
    #[default]
    Dual,
}

impl fmt::Display for Formulation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Formulation::Primal => "primal",
            Formulation::Dual => "dual",
        })
    }
}
