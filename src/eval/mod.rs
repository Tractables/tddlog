//! Weighted model counts on TDDs: the numbers they are counted in
//! ([`Value`]: a float, or a [`Count`] where a float would not do), the
//! weights of facts and indicators ([`Probabilities`]), the engine's
//! evaluation of a diagram (`mass`), the two passes of a count over one
//! diagram ([`fn@count`], [`Counted`]), and the count of the conjunction of
//! two diagrams, one on a projection of the other's vtree, without
//! building it (`conjunction_count`).
//!
//! Every count is taken in `f64` first, and is exact where no product it
//! reads underflowed or overflowed: then it is the float count with its
//! rounding alone, which is what `Count` computes too. Each level of a
//! pass, and each table of the walk, notes the [`Range`] of the values it
//! settled, and a product of two of them is exact when the product of the
//! two least is a normal float and neither is infinite. Where a level's
//! products, or a column's down, would not be, the count passes keep the
//! operands under a power of two, one per level and one per column
//! ([`Scaled`]): multiplying by a power of two is exact on normal floats,
//! so the values are those of a count in `Count`s, bit for bit. A count
//! that is still not exact, a level of it spanning more than a float does,
//! may have lost a node to underflow while its total stays positive, and
//! the caller counts again in [`Count`]s. Only what a count
//! reads is checked: the walk reads a query formula's counts at its leaves
//! and over the variables the evidence does not hold, and not those of its
//! other levels, which overflow where a formula has many indicators.

mod count;
mod grid;
mod kernel;
mod number;
mod weights;

// The parts read one another's items through this module.
use count::*;
use grid::Grid;
use number::*;

pub use count::{Counted, Routes, count};
pub(crate) use count::{Counts, conjunction_count, count_exact, spans};
pub use number::{Count, Range, Scaled, Value};
pub use weights::{Probabilities, Weighing};
pub(crate) use weights::{complement, mass};
