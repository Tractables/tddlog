//! The numbers weighted counts are taken in ([`Value`]: a float, or a
//! [`Count`] where a float would not do), the [`Range`] of the values a
//! level settles, and the powers of two the count passes keep a level
//! under ([`Scaled`]).

#[cfg(test)]
use std::cell::Cell;

use super::*;

/// A number weighted counts are taken in: `f64`, or [`Count`]. The
/// arithmetic is the type's own; `+` and `*` do not normalize a
/// [`Count`], and every value a sum or product produces is
/// [`settle`](Value::settle)d before it is an operand again.
pub trait Value:
    Copy
    + std::fmt::Debug
    + std::iter::Sum
    + std::ops::Add<Output = Self>
    + std::ops::AddAssign
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
{
    const ZERO: Self;
    const ONE: Self;
    /// The weight `x`, a finite nonnegative float.
    fn from_f64(x: f64) -> Self;
    /// The count `c`: a float rounds it to the nearest.
    fn from_count(c: Count) -> Self;
    /// The value as a [`Count`], exactly.
    fn to_count(self) -> Count;
    /// The value times `2^k`: exact where a float stays normal.
    fn shifted(self, k: i64) -> Self;
    fn is_zero(self) -> bool;
    /// A value a sum or product produced, ready to be an operand: a
    /// [`Count`] normalized; a float as it is, noted in `range`. A
    /// [`Count`] is never noted, so its products are always exact.
    fn settle(self, range: &mut Range) -> Self;
    /// Every value of `xs` settled.
    fn settle_all(xs: &mut [Self], range: &mut Range) {
        for x in xs {
            *x = x.settle(range);
        }
    }
    /// `self / other` as a float, `other` nonzero.
    fn ratio(self, other: Self) -> f64;
    /// `n` zeros, or `OverBudget` where the allocator refuses them, as
    /// `filled` allocates them.
    fn zeros(n: usize) -> Result<Vec<Self>, tididi::OperationError> {
        filled(n, Self::ZERO)
    }
    /// `xs` as floats, if the values are floats: the grid's vector kernels
    /// run on those.
    fn floats(xs: &[Self]) -> Option<&[f64]> {
        let _ = xs;
        None
    }
    /// `xs` as floats to write, if the values are floats.
    fn floats_mut(xs: &mut [Self]) -> Option<&mut [f64]> {
        let _ = xs;
        None
    }
}

/// The least nonzero and the largest of a set of floats, the least taken
/// with one: what decides whether their products with another set's are
/// exact. One stands for the model counts a pair can hold inline, whole
/// numbers, by which a product does not shrink.
///
/// Values settled from exact products are zero or normal, and a sum of
/// those is one too, short of overflow; the product of two is normal
/// unless that of the two sets' least nonzero values is not. An overflow
/// is infinite and stays so through every sum and product after it, or
/// turns to a NaN against a zero, which only an infinite operand makes:
/// the largest of a set catches it where it is settled.
///
/// A pass notes a level's or a table's values once it is built, in a loop
/// of its own ([`Value::settle_all`]), and leaves the loops that compute
/// them as they were in plain floats.
///
/// The range holds the least nonzero value itself, infinite where there is
/// none, and takes it with one where it is read: the two extremes are
/// what centres the set on one.
#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub(super) low: f64,
    pub(super) high: f64,
}

impl Range {
    /// No value noted.
    pub(crate) const NEW: Range = Range {
        low: f64::INFINITY,
        high: 0.0,
    };

    #[inline]
    pub(super) fn note(&mut self, x: f64) {
        let y = if x == 0.0 { f64::INFINITY } else { x };
        self.low = if y < self.low { y } else { self.low };
        self.high = if x > self.high { x } else { self.high };
    }

    /// The two ranges together.
    #[inline]
    pub(super) fn join(self, other: Range) -> Range {
        Range {
            low: if other.low < self.low {
                other.low
            } else {
                self.low
            },
            high: if other.high > self.high {
                other.high
            } else {
                self.high
            },
        }
    }

    /// The range of the one value `x`.
    pub(super) fn of<V: Value>(x: V) -> Range {
        let mut r = Range::NEW;
        x.settle(&mut r);
        r
    }

    #[inline]
    pub(super) fn finite(self) -> bool {
        self.high <= f64::MAX
    }

    /// Whether every nonzero value is normal and none infinite: weights
    /// an exact count can start from.
    #[inline]
    pub(super) fn normal(self) -> bool {
        self.low >= f64::MIN_POSITIVE && self.finite()
    }

    /// Whether every product of a value of this set and one of `other` is
    /// exact: normal or zero, and finite unless it overflows, which the
    /// range it is settled into catches.
    #[inline]
    pub(super) fn times(self, other: Range) -> bool {
        self.low.min(1.0) * other.low.min(1.0) >= f64::MIN_POSITIVE
            && self.finite()
            && other.finite()
    }

    /// The binary exponents of the least nonzero value and the greatest,
    /// if one is nonzero.
    pub(super) fn extent(self) -> Option<(i64, i64)> {
        (self.high > 0.0).then(|| (exponent(self.low), exponent(self.high)))
    }

    /// The range of the values times `2^k`, where they stay normal.
    pub(super) fn shifted(self, k: i64) -> Range {
        Range {
            low: scale(self.low, k),
            high: scale(self.high, k),
        }
    }
}

impl Value for f64 {
    const ZERO: f64 = 0.0;
    const ONE: f64 = 1.0;
    #[inline]
    fn from_f64(x: f64) -> f64 {
        x
    }
    #[inline]
    fn from_count(c: Count) -> f64 {
        c.to_f64()
    }
    fn to_count(self) -> Count {
        Count::new(self)
    }
    fn shifted(self, k: i64) -> f64 {
        scale(self, k)
    }
    #[inline]
    fn is_zero(self) -> bool {
        self == 0.0
    }
    #[inline]
    fn settle(self, range: &mut Range) -> f64 {
        range.note(self);
        self
    }
    /// Zeroed by the allocator, which writes nothing on a block fresh from
    /// the system, where `filled` writes every float: the columns of a
    /// count are gigabytes on the large networks.
    fn zeros(n: usize) -> Result<Vec<f64>, tididi::OperationError> {
        if n == 0 {
            return Ok(Vec::new());
        }
        let layout =
            std::alloc::Layout::array::<f64>(n).map_err(|_| tididi::OperationError::OverBudget)?;
        // SAFETY: `layout` has a nonzero size, and a null pointer is the
        // allocator's refusal. The block is the global allocator's, of the
        // layout a `Vec` of `n` floats frees, and its bytes are zero, which
        // is `0.0` in each float.
        unsafe {
            let p = std::alloc::alloc_zeroed(layout).cast::<f64>();
            if p.is_null() {
                return Err(tididi::OperationError::OverBudget);
            }
            Ok(Vec::from_raw_parts(p, n, n))
        }
    }
    /// Eight floats a step in vectors where the processor has AVX2;
    /// otherwise four lanes, so that no compare waits on the one before
    /// and the lanes can share vector instructions.
    fn settle_all(xs: &mut [f64], range: &mut Range) {
        if let Some((low, high)) = kernel::extremes(xs, range.low, range.high) {
            *range = Range { low, high };
            return;
        }
        let mut lanes = [*range; 4];
        let (chunks, remainder) = xs.as_chunks::<4>();
        for chunk in chunks {
            for (lane, &x) in lanes.iter_mut().zip(chunk) {
                lane.note(x);
            }
        }
        for &x in remainder {
            lanes[0].note(x);
        }
        *range = lanes[0].join(lanes[1]).join(lanes[2].join(lanes[3]));
    }
    #[inline]
    fn ratio(self, other: f64) -> f64 {
        self / other
    }
    fn floats(xs: &[f64]) -> Option<&[f64]> {
        Some(xs)
    }
    fn floats_mut(xs: &mut [f64]) -> Option<&mut [f64]> {
        Some(xs)
    }
}

/// The binary orders of magnitude a [`Count`]'s exponent counts in.
pub(super) const BLOCK: i32 = 256;
/// `2^BLOCK` and `2^-BLOCK`: a normalized count's mantissa lies between
/// them, or is zero.
pub(super) const HIGH: f64 = f64::from_bits(((1023 + BLOCK) as u64) << 52);
pub(super) const LOW: f64 = f64::from_bits(((1023 - BLOCK) as u64) << 52);

/// A nonnegative weighted count, `m · 2^(256·b)`: a float with an exponent
/// of its own beside the float's, so that no count underflows or
/// overflows. The probability of a thousand observations is below the
/// least positive `f64`, and an indicator's two sides weigh two.
///
/// [`normalized`](Count::normalized) keeps the mantissa between `2^-256`
/// and `2^256`, so that the product of two normalized mantissas, and a sum
/// of such products, is a normal float. A count whose every value stays
/// in that band has `b` zero throughout, and its arithmetic is the plain
/// `f64` arithmetic, operation for operation; elsewhere a mantissa moves by
/// powers of two, which is exact for normal floats, so the answers are
/// those of `f64` wherever `f64` neither underflows nor overflows.
///
/// The exponent is per value, not per level: two nodes of one level can
/// differ by more than the range of a float and both matter. A hidden
/// cause observed 2 200 times for and 2 199 times against has, at the
/// level over the first 2 200 observations, a node under each of its
/// values, their counts 2^2200 apart and each half the total
/// (`tests/it/underflow.rs`).
#[derive(Clone, Copy, Debug)]
pub struct Count {
    pub(super) m: f64,
    pub(super) b: i32,
}

impl Count {
    pub(crate) const ZERO: Count = Count { m: 0.0, b: 0 };
    pub(crate) const ONE: Count = Count { m: 1.0, b: 0 };

    /// The count `x`, a finite nonnegative float.
    #[inline]
    pub fn new(x: f64) -> Count {
        Count { m: x, b: 0 }.normalized()
    }

    /// Whether the count is zero: only an empty sum or a product with zero
    /// is.
    #[inline]
    pub(crate) fn is_zero(self) -> bool {
        self.m == 0.0
    }

    /// The sum of two counts with different exponents, on the larger
    /// exponent: the other mantissa scaled down, exactly unless it falls
    /// below the least normal float, where it is under `2^-510` of the
    /// sum. Exponents up to three blocks apart scale by one multiplication
    /// by a constant, the one `scale` takes.
    #[inline]
    pub(super) fn add_apart(&mut self, other: Count) {
        /// `2^(-BLOCK·d)`, normal for `d` up to three.
        const APART: [f64; 4] = [1.0, LOW, LOW * LOW, LOW * LOW * LOW];
        if other.m == 0.0 {
            return;
        }
        if self.m == 0.0 {
            *self = other;
            return;
        }
        match self.b - other.b {
            d @ 1..=3 => self.m += other.m * APART[d as usize],
            d @ -3..=-1 => {
                self.m = self.m * APART[(-d) as usize] + other.m;
                self.b = other.b;
            }
            _ => self.add_far(other),
        }
    }

    /// [`add_apart`](Count::add_apart) for exponents more than three blocks
    /// apart.
    #[cold]
    #[inline(never)]
    pub(super) fn add_far(&mut self, other: Count) {
        if self.b > other.b {
            self.m += scale(other.m, i64::from(BLOCK) * i64::from(other.b - self.b));
        } else {
            self.m = scale(self.m, i64::from(BLOCK) * i64::from(self.b - other.b)) + other.m;
            self.b = other.b;
        }
    }

    /// The same count with its mantissa in the band, or zero.
    #[inline]
    #[must_use]
    pub fn normalized(self) -> Count {
        let a = self.m.abs();
        if a > HIGH || (a < LOW && a != 0.0) {
            self.renormalized()
        } else {
            self
        }
    }

    #[cold]
    #[inline(never)]
    pub(super) fn renormalized(mut self) -> Count {
        if !self.m.is_finite() {
            return self;
        }
        while self.m.abs() > HIGH {
            self.m *= LOW;
            self.b += 1;
        }
        while self.m != 0.0 && self.m.abs() < LOW {
            self.m *= HIGH;
            self.b -= 1;
        }
        self
    }

    /// The count times `2^k`, exactly.
    #[must_use]
    pub fn shifted(self, k: i64) -> Count {
        let block = i64::from(BLOCK);
        let b = i32::try_from(i64::from(self.b) + k.div_euclid(block))
            .expect("a count's exponent fits");
        Count {
            m: scale(self.m, k.rem_euclid(block)),
            b,
        }
        .normalized()
    }

    /// `self / other` as a float, `other` nonzero: a conditional
    /// probability, whatever the two counts' magnitudes.
    #[must_use]
    pub fn ratio(self, other: Count) -> f64 {
        scale(
            self.m / other.m,
            i64::from(BLOCK) * (i64::from(self.b) - i64::from(other.b)),
        )
    }

    /// The nearest float: zero below the least positive one, infinite above
    /// the largest.
    #[must_use]
    pub fn to_f64(self) -> f64 {
        scale(self.m, i64::from(BLOCK) * i64::from(self.b))
    }

    /// The base-10 logarithm, `-inf` for zero, whatever the count's
    /// magnitude: `2^-1100` is `-331.13`.
    #[must_use]
    pub fn log10(self) -> f64 {
        self.m.log10() + f64::from(BLOCK) * f64::from(self.b) * std::f64::consts::LOG10_2
    }
}

/// The product, not normalized: of normalized counts, or of a short sum of
/// them and a weight.
impl std::ops::Mul for Count {
    type Output = Count;
    #[inline]
    fn mul(self, other: Count) -> Count {
        Count {
            m: self.m * other.m,
            b: self.b + other.b,
        }
    }
}

/// The quotient, not normalized; both sides normalized.
impl std::ops::Div for Count {
    type Output = Count;
    #[inline]
    fn div(self, other: Count) -> Count {
        Count {
            m: self.m / other.m,
            b: self.b - other.b,
        }
    }
}

/// The sum, not normalized. Where the two exponents agree, which is
/// everywhere on a count in the band, it is one float addition.
impl std::ops::AddAssign for Count {
    #[inline]
    fn add_assign(&mut self, other: Count) {
        if self.b == other.b {
            self.m += other.m;
        } else {
            self.add_apart(other);
        }
    }
}

impl std::ops::Add for Count {
    type Output = Count;
    #[inline]
    fn add(mut self, other: Count) -> Count {
        self += other;
        self
    }
}

/// The sum, not normalized: from the first term rather than from zero,
/// whose exponent the first term need not share.
impl std::iter::Sum for Count {
    fn sum<I: Iterator<Item = Count>>(iter: I) -> Count {
        iter.reduce(|acc, x| acc + x).unwrap_or(Count::ZERO)
    }
}

impl Value for Count {
    const ZERO: Count = Count::ZERO;
    const ONE: Count = Count::ONE;
    #[inline]
    fn from_f64(x: f64) -> Count {
        Count::new(x)
    }
    #[inline]
    fn from_count(c: Count) -> Count {
        c
    }
    fn to_count(self) -> Count {
        self
    }
    fn shifted(self, k: i64) -> Count {
        Count::shifted(self, k)
    }
    #[inline]
    fn is_zero(self) -> bool {
        Count::is_zero(self)
    }
    #[inline]
    fn settle(self, _: &mut Range) -> Count {
        self.normalized()
    }
    #[inline]
    fn ratio(self, other: Count) -> f64 {
        Count::ratio(self, other)
    }
}

/// A count as a value and a power of two, `value · 2^exp`. A count in
/// floats keeps each level's counts up, and each column down, under a
/// power of its own, zero unless the level's products, or the column's,
/// would leave the normal floats without it.
#[derive(Clone, Copy, Debug)]
pub struct Scaled<V> {
    pub value: V,
    pub exp: i64,
}

impl<V: Value> Scaled<V> {
    pub(super) fn new(value: V, exp: i64) -> Self {
        Scaled { value, exp }
    }

    /// The count, exactly.
    pub fn count(self) -> Count {
        self.value.to_count().shifted(self.exp)
    }

    /// `self / mass` as a float, `mass` nonzero: a value under no power
    /// divided as a `V`, and a scaled one as a [`Count`], so that the
    /// quotient is that of a count in `Count`s wherever it is a normal
    /// float.
    pub fn ratio(self, mass: Count) -> f64 {
        if self.exp == 0 {
            self.value.ratio(V::from_count(mass))
        } else {
            self.count().ratio(mass)
        }
    }
}

/// The least value a set of counts computed in floats must reach, every
/// value nonzero, to be exact although its products were not all normal
/// ([`exact_as_computed`]): `2^-960`.
pub(super) const EXACT_FLOOR: f64 = f64::from_bits(((1023 - 960) as u64) << 52);

/// Whether `values`, counts of range `range` computed in floats from
/// operands whose products [`Range::times`] does not find all normal, are
/// as exact as if they were. A product that leaves the normal floats is
/// rounded to a subnormal or to zero, off by at most `2^-1075`, so a sum
/// of fewer than `2^63` terms loses under `2^-1012` to them, under `2^-52`
/// of a value of [`EXACT_FLOOR`] or more: one rounding's error. A zero may
/// be every term of a sum lost, where a product's exponent may be under
/// `-1074` (`lowest`, [`lowest_product`]), so a set with one is then not
/// exact. The test
/// on the operands takes the products of their extremes, which no pair
/// need take: a level whose own values span a few hundred powers of two
/// reads children that span a thousand each.
pub(super) fn exact_as_computed<V: Value>(values: &[V], range: Range, lowest: Option<i64>) -> bool {
    range.low >= EXACT_FLOOR
        && range.finite()
        && (lowest.is_none_or(|e| e >= -1074) || !values.iter().any(|x| x.is_zero()))
}

/// The binary exponent of the least nonzero product of a value of range
/// `a` and one of range `b`, as they come, if there is one: a product of
/// that exponent or more is not zero.
pub(super) fn lowest_product(a: Range, b: Range) -> Option<i64> {
    Some(a.extent()?.0 + b.extent()?.0)
}

/// `k` split between two sets of normal counts of ranges `l` and `r`, so
/// that each times its share stays normal: their products times `2^k`;
/// `None` where no split keeps both normal.
pub(super) fn split_shift(l: Range, r: Range, k: i64) -> Option<(i64, i64)> {
    let ((ll, lh), (rl, rh)) = (l.extent()?, r.extent()?);
    let kr = (k - k.div_euclid(2)).clamp(-1022 - rl, 1023 - rh);
    let kl = (k - kr).clamp(-1022 - ll, 1023 - lh);
    (kl + kr == k).then_some((kl, kr))
}

/// The shift that takes the products of two sets of counts of ranges `l`
/// and `r` under the largest float, where they could reach it: a product
/// is under `2^(lh + rh + 2)`, a sum of fewer than `2^63` of them under
/// `2^(lh + rh + 65)`. Zero where they cannot reach it.
pub(super) fn capping(l: Range, r: Range) -> i64 {
    match (l.extent(), r.extent()) {
        (Some((_, lh)), Some((_, rh))) => (958 - lh - rh).min(0),
        _ => 0,
    }
}

/// The shift that moves a level's counts, computed in floats in range
/// `level` and not exact ([`exact_as_computed`]), up under the largest
/// float, so that fewer of its products leave the normal floats at the
/// bottom; zero where it would not move them up.
pub(super) fn raising(level: Range) -> i64 {
    match level.extent() {
        Some((_, high)) if level.finite() => (1021 - high).max(0),
        _ => 0,
    }
}

/// The binary exponent of a normal float.
pub(super) fn exponent(x: f64) -> i64 {
    ((x.to_bits() >> 52) & 0x7ff) as i64 - 1023
}

/// The power `k` that centres on one the nonzero values of range `range`
/// times `2^k`, and with `with`, their products with those of range
/// `with`: minus the mean of the binary exponents of the least nonzero and
/// the greatest. Zero where none is nonzero, as for values that are not
/// floats, which are never noted; `None` where the values times `2^k` are
/// not all normal. The values are normal floats, settled from exact
/// products.
pub(super) fn centring(range: Range, with: Range) -> Option<i64> {
    let Some((low, high)) = range.extent() else {
        return Some(0);
    };
    let (wl, wh) = with.extent().unwrap_or((0, 0));
    let k = -(low + high + wl + wh).div_euclid(2);
    (low + k >= -1022 && high + k <= 1023).then_some(k)
}

#[cfg(test)]
thread_local! {
    /// For the tests: centre every level, column and table that a count in
    /// floats may centre, whether or not its products need it.
    pub(super) static CENTRE_ALL: Cell<bool> = const { Cell::new(false) };
}

/// Whether to centre values whose products stay normal unscaled: where a
/// test asks for it, and never otherwise.
pub(super) fn centre_all() -> bool {
    #[cfg(test)]
    return CENTRE_ALL.get();
    #[cfg(not(test))]
    false
}

/// `xs`, of range `range`, times `2^k`, and `range` with them; exact where
/// `k` is a [`centring`] of them.
pub(super) fn shift_all<V: Value>(xs: &mut [V], k: i64, range: &mut Range) {
    for x in xs.iter_mut() {
        *x = x.shifted(k);
    }
    *range = range.shifted(k);
}

/// `m · 2^k`, rounded once: by a normal power of two, or by two where `2^k`
/// is not one. `m` is under `2^1000`, as every mantissa here is.
pub(super) fn scale(m: f64, k: i64) -> f64 {
    /// `2^k` for `-1022 <= k <= 1023`.
    pub(super) fn pow2(k: i64) -> f64 {
        f64::from_bits(((1023 + k) as u64) << 52)
    }
    match k {
        -1022..=1023 => m * pow2(k),
        -2044..=2046 => m * pow2(k / 2) * pow2(k - k / 2),
        _ if k < 0 || m == 0.0 => 0.0,
        _ => f64::INFINITY,
    }
}
