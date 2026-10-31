//! The weights of facts and indicators ([`Probabilities`]) and the
//! engine's evaluation of a diagram under them ([`mass`]).

use std::cell::Cell;

use tididi::diagram::{EvalAlgebra, LeafLabel};
use tididi::vtree::VarId;

use crate::ground::{AtomKind, GroundProgram};

use super::*;

/// The probability weights: `P(x)` and `P(¬x)` for a fact, its two
/// weights (`crate::ground::Prob`), one on both sides for an indicator
/// ([`crate::indicators`]).
#[derive(Clone, Debug)]
pub struct Probabilities {
    /// The weights of `x`, indexed by `VarId.0`; index 0 unused.
    pub pos: Vec<f64>,
    /// The weights of `¬x`, indexed alike.
    pub neg: Vec<f64>,
    /// The weights of `x` and `¬x` together, indexed alike: kept apart so
    /// that a fact's is exactly one, whatever `p + (1 - p)` rounds to.
    pub one: Vec<f64>,
}

impl Probabilities {
    pub(crate) fn from_program(
        gp: &GroundProgram,
        var_of: &[Option<VarId>],
        num_vars: u32,
    ) -> Self {
        let mut pos = vec![0.5; num_vars as usize + 1];
        let mut neg = vec![0.5; num_vars as usize + 1];
        let mut one = vec![1.0; num_vars as usize + 1];
        for (a, k) in gp.kinds.iter().enumerate() {
            if let AtomKind::Fact(p) = k
                && let Some(v) = var_of[a]
            {
                pos[v.0 as usize] = p.p;
                neg[v.0 as usize] = p.not;
            }
        }
        for ind in &gp.encoding.indicators {
            if let Some(v) = var_of[ind.var as usize] {
                pos[v.0 as usize] = 1.0;
                neg[v.0 as usize] = 1.0;
                one[v.0 as usize] = 2.0;
            }
        }
        Probabilities { pos, neg, one }
    }

    /// This table on a projected scope: local variable `i + 1` takes the
    /// weights of the program's variable `vars[i]`. `None` is the whole
    /// program, on which it is this table.
    ///
    /// A fact a formula does not mention contributes `p + (1 - p) = 1` to
    /// its weighted model count, so dropping it from the vtree leaves the
    /// count unchanged: the count of a formula over its cone is the count
    /// over the whole fact set. An indicator would contribute two, and none
    /// is dropped from a count: the query phase counts every formula
    /// together with the links of all of them, which mention them all.
    #[must_use]
    pub(crate) fn restrict(&self, vars: Option<&[VarId]>) -> Probabilities {
        let Some(vars) = vars else {
            return self.clone();
        };
        // Index 0 is unused, as in `from_program`.
        let pos = std::iter::once(0.5)
            .chain(vars.iter().map(|v| self.pos[v.0 as usize]))
            .collect();
        let neg = std::iter::once(0.5)
            .chain(vars.iter().map(|v| self.neg[v.0 as usize]))
            .collect();
        let one = std::iter::once(1.0)
            .chain(vars.iter().map(|v| self.one[v.0 as usize]))
            .collect();
        Probabilities { pos, neg, one }
    }
}

impl Probabilities {
    /// The weight of leaf `label` of `var`.
    pub(super) fn weight(&self, var: VarId, label: LeafLabel) -> f64 {
        match label {
            LeafLabel::Pos => self.pos[var.0 as usize],
            LeafLabel::Neg => self.neg[var.0 as usize],
            LeafLabel::One => self.one[var.0 as usize],
            LeafLabel::Zero => 0.0,
        }
    }
}

/// The engine's evaluation of a diagram in `V`s, a node's value the sum of
/// its pairs' products settled, as [`fn@count`]'s pass up computes it. The
/// engine hands the algebra one node at a time, not a level, so every
/// value an evaluation settles is noted in one range, and the evaluation
/// is exact when the products of that range with itself are: every
/// nonzero value from `2^-511` up, and none infinite.
pub struct Weighing<'a, V> {
    pub(super) weights: &'a Probabilities,
    pub(super) range: Cell<Range>,
    pub(super) value: std::marker::PhantomData<V>,
}

impl<'a, V: Value> Weighing<'a, V> {
    /// An evaluation under `weights` that has settled no value yet.
    pub fn new(weights: &'a Probabilities) -> Self {
        Weighing {
            weights,
            range: Cell::new(Range::NEW),
            value: std::marker::PhantomData,
        }
    }

    /// Whether the evaluations so far were exact: always, in [`Count`]s.
    pub fn exact(&self) -> bool {
        let range = self.range.get();
        range.times(range)
    }

    pub(super) fn settle(&self, x: V) -> V {
        let mut range = self.range.get();
        let x = x.settle(&mut range);
        self.range.set(range);
        x
    }
}

impl<V: Value> EvalAlgebra for Weighing<'_, V> {
    type Value = V;
    fn zero(&self) -> V {
        V::ZERO
    }
    fn leaf(&self, var: VarId, label: LeafLabel) -> V {
        self.settle(V::from_f64(self.weights.weight(var, label)))
    }
    fn add_assign(&self, acc: &mut V, other: &V) {
        *acc = self.settle(*acc + *other);
    }
    fn mul(&self, a: &V, b: &V) -> V {
        self.settle(*a * *b)
    }
    fn sum_of_products<'v>(&self, pairs: impl ExactSizeIterator<Item = (&'v V, &'v V)>) -> V
    where
        V: 'v,
    {
        let mut acc = V::ZERO;
        for (a, b) in pairs {
            acc += *a * *b;
        }
        self.settle(acc)
    }
}

/// Under this, the probability of a formula's complement is not taken as
/// `1 - m` from the formula's count `m`: the subtraction keeps the count's
/// absolute error, some 1e-14 on a deep formula, and a posterior of 2.8e-121
/// came out as 2.6e-14. Above it `1 - m` keeps a relative error under about
/// 1e-9.
pub(crate) const COMPLEMENT_FLOOR: f64 = 1e-4;

/// The probability of the complement of a formula of probability `m`:
/// `1 - m`, or where that is under [`COMPLEMENT_FLOOR`], what `count` gives,
/// the complement's own formula counted.
pub(crate) fn complement<E>(m: f64, count: impl FnOnce() -> Result<f64, E>) -> Result<f64, E> {
    if 1.0 - m >= COMPLEMENT_FLOOR {
        Ok(1.0 - m)
    } else {
        count()
    }
}

/// The weighted count of `f` under `w` by the engine's evaluation in
/// floats, which the engine's work clock is charged for; where that is not
/// exact, by [`fn@count`]'s pass up in [`Count`]s, which is not charged, and
/// `recounts` counts it.
pub(crate) fn mass(
    engine: &tididi::Engine,
    f: &tididi::Tdd,
    w: &Probabilities,
    recounts: &mut usize,
) -> Result<Count, tididi::OperationError> {
    let plain = Weighing::<f64>::new(w);
    let m = engine.evaluate(f, &plain)?;
    if plain.exact() {
        return Ok(Count::new(m));
    }
    *recounts += 1;
    Ok(count::<Count>(f, w)?.total(f).count())
}
