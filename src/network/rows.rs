//! The numbers of a table's rows: how a row that does not sum to one is
//! written, and how a probability is written in the program.
//!
//! A row of a table sums to one, and a file's decimals round it: bnlearn's
//! pathfinder writes `0.3333333` three times, and the tables of a ground
//! network are computed in doubles, so that some of Ace's rows sum to
//! `1.00000000000000008`. Such a row is divided by its sum. A gap that the
//! rounding of the entries does not explain, `0.9, 0.099` say, is the
//! table's own: short of one, the rest is left to no value, as ProbLog
//! reads an annotated disjunction; over one, it is an error.

use std::cmp::Ordering;

/// The most a row's sum may differ from one and still be taken as binary
/// rounding, the grounder's own tolerance for a row.
const ROUNDING: f64 = 1e-9;

/// `row` as the program writes it: divided by its sum where the sum
/// differs from one by rounding, as read otherwise; an error for a row
/// that sums to more than one by more than rounding explains, or that
/// holds a negative or not finite entry.
///
/// A sum within [`ROUNDING`] of one is binary rounding: over one, the row
/// is divided by it, and under it, kept, so that rows that sum to one but
/// for the last bits keep their numbers. A sum further from one is
/// rounding where the gap is under the row's slack ([`explained`]), and
/// the row is divided by it; otherwise a shortfall is kept and an excess
/// is an error. The sum is `math.fsum`'s, which decides whether a row of
/// sum one up to its last bits is divided ([`fsum`]).
pub(super) fn normalized(row: &mut [f64]) -> Result<(), String> {
    if let Some(&p) = row.iter().find(|p| !p.is_finite() || **p < 0.0) {
        return Err(format!("{p} is not a probability"));
    }
    let total = fsum(row);
    let divide = if (row.len() == 1 && total <= 1.0) || (1.0 - ROUNDING..=1.0).contains(&total) {
        // A row of one entry in [0, 1], whose gap from one is at least a unit
        // in its last place, more than its slack; or binary rounding under
        // one.
        false
    } else if (total > 1.0 && total <= 1.0 + ROUNDING) || explained(row) {
        true
    } else if total < 1.0 {
        false
    } else {
        return Err(format!("probabilities sum to {total}"));
    };
    if divide {
        for p in row {
            *p /= total;
        }
    }
    Ok(())
}

/// Whether the decimals the entries of `row` mean ([`Decimal::meant`]) sum
/// to one up to their rounding: whether the gap from one of their exact
/// sum is under the row's slack. The files round both ways: to a number of
/// decimals, so that sachs writes 0.0000000 and 0.001432665 in one row, and
/// to a number of significant digits, so that pathfinder's `0.9999,
/// 9.999e-05` are 10000/10001 and 1/10001 to four digits. So each entry
/// may be off by the larger of half a unit in the row's finest decimal
/// place and half a unit in the most significant digits any entry of the
/// row has (a zero entry by the first), and the slack is the sum of these.
fn explained(row: &[f64]) -> bool {
    let written: Vec<Decimal> = row.iter().map(|&p| Decimal::meant(p)).collect();
    let nonzero = || written.iter().filter(|d| d.digits > 0);
    let Some(place) = nonzero().map(|d| d.exp).min() else {
        // Zeros sum to zero, which no rounding brings to one.
        return false;
    };
    let digits = nonzero().map(|d| d.digits).max().expect("a nonzero entry");
    // Each entry's slack is 5·10^t: the larger t of the two halves.
    let halves: Vec<i32> = written
        .iter()
        .map(|d| {
            if d.digits == 0 {
                place - 1
            } else {
                (place - 1).max(d.exp + d.digits as i32 - 1 - digits as i32)
            }
        })
        .collect();
    // Every value is an integer in units of 10^low.
    let low = (place - 1).min(0);
    let at = |m: u64, exp: i32| Natural::scaled(m, (exp - low) as u32);
    let mut sum = Natural::default();
    for d in &written {
        sum.add(&at(d.mantissa, d.exp));
    }
    let mut slack = Natural::default();
    for &t in &halves {
        slack.add(&at(5, t));
    }
    let one = at(1, 0);
    let gap = match sum.cmp(&one) {
        Ordering::Less => one.minus(&sum),
        _ => sum.minus(&one),
    };
    gap.cmp(&slack) == Ordering::Less
}

/// A nonnegative decimal `mantissa · 10^exp` with `digits` significant
/// digits, the mantissa without trailing zeros; zero has no digits.
struct Decimal {
    mantissa: u64,
    digits: u32,
    exp: i32,
}

impl Decimal {
    /// The decimal a file means by `x`, a finite nonnegative double: the
    /// shortest of at most 12 significant digits within two units in the
    /// last place of `x`, and otherwise the shortest that reads back as
    /// `x`. A file written by a program prints doubles in full, binary
    /// noise and all (Ace's pathfinder.xdsl writes 0.00726292 as
    /// 0.007262920000000001), and the noise would make the entry's slack
    /// that of 16 digits; 2/3 printed in full stays 0.6666666666666666.
    /// For an entry the file writes in 12 significant digits or fewer, this
    /// is its text. `scripts/gen_bn.py`'s `meant_decimal`.
    fn meant(x: f64) -> Decimal {
        if x == 0.0 {
            return Decimal {
                mantissa: 0,
                digits: 0,
                exp: 0,
            };
        }
        let ulp = f64::from_bits(x.to_bits() + 1) - x;
        (1..=12)
            .map(|digits| format!("{x:.*e}", digits - 1))
            .find(|text| (text.parse::<f64>().expect("a decimal") - x).abs() <= 2.0 * ulp)
            .map_or_else(
                || Decimal::parse(&format!("{x:e}")),
                |text| Decimal::parse(&text),
            )
    }

    /// The decimal `d.ddde±x` that Rust writes, without trailing zeros.
    fn parse(text: &str) -> Decimal {
        let (mantissa, exp) = text.split_once('e').expect("Rust writes an exponent");
        let mut digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
        let mut exp = exp.parse::<i32>().expect("an integer exponent") - (digits.len() as i32 - 1);
        while digits.len() > 1 && digits.ends_with('0') {
            digits.pop();
            exp += 1;
        }
        Decimal {
            mantissa: digits.parse().expect("at most 17 digits"),
            digits: digits.len() as u32,
            exp,
        }
    }
}

/// A natural number in base 10^9, least significant limb first, without
/// high zero limbs: the exact sums of a row's decimals, whose places can
/// lie hundreds of digits apart.
#[derive(Debug, Default, PartialEq, Eq)]
struct Natural(Vec<u32>);

const BASE: u64 = 1_000_000_000;

impl Natural {
    /// `m · 10^k`.
    fn scaled(m: u64, k: u32) -> Natural {
        let mut limbs = vec![0; (k / 9) as usize];
        let mut x = u128::from(m) * u128::from(10u64.pow(k % 9));
        while x > 0 {
            limbs.push((x % u128::from(BASE)) as u32);
            x /= u128::from(BASE);
        }
        if limbs.iter().all(|&l| l == 0) {
            limbs.clear();
        }
        Natural(limbs)
    }

    fn add(&mut self, other: &Natural) {
        let n = self.0.len().max(other.0.len());
        self.0.resize(n, 0);
        let mut carry = 0;
        for (i, limb) in self.0.iter_mut().enumerate() {
            let x = u64::from(*limb) + u64::from(other.0.get(i).copied().unwrap_or(0)) + carry;
            *limb = (x % BASE) as u32;
            carry = x / BASE;
        }
        if carry > 0 {
            self.0.push(carry as u32);
        }
    }

    /// `self - other`, for `other` at most `self`.
    fn minus(&self, other: &Natural) -> Natural {
        let mut limbs = self.0.clone();
        let mut borrow = 0;
        for (i, limb) in limbs.iter_mut().enumerate() {
            let take = u64::from(other.0.get(i).copied().unwrap_or(0)) + borrow;
            let have = u64::from(*limb);
            (*limb, borrow) = if have >= take {
                ((have - take) as u32, 0)
            } else {
                ((have + BASE - take) as u32, 1)
            };
        }
        while limbs.last() == Some(&0) {
            limbs.pop();
        }
        Natural(limbs)
    }
}

impl Ord for Natural {
    fn cmp(&self, other: &Natural) -> Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.iter().rev().cmp(other.0.iter().rev()))
    }
}

impl PartialOrd for Natural {
    fn partial_cmp(&self, other: &Natural) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The sum of `xs` correctly rounded, as Python's `math.fsum` computes it
/// (Shewchuk's partials, then the rounding of their sum): whether a row
/// sums to more than one decides whether it is divided, and where a plain
/// sum differs from the exact one in the last bit, the decisions of
/// `scripts/gen_bn.py`, which sums with `math.fsum`, would differ.
pub(super) fn fsum(xs: &[f64]) -> f64 {
    let mut partials: Vec<f64> = Vec::new();
    for &x in xs {
        let mut x = x;
        let mut i = 0;
        for j in 0..partials.len() {
            let mut y = partials[j];
            if x.abs() < y.abs() {
                std::mem::swap(&mut x, &mut y);
            }
            let hi = x + y;
            let lo = y - (hi - x);
            if lo != 0.0 {
                partials[i] = lo;
                i += 1;
            }
            x = hi;
        }
        partials.truncate(i);
        partials.push(x);
    }
    let mut n = partials.len();
    if n == 0 {
        return 0.0;
    }
    n -= 1;
    let mut hi = partials[n];
    let mut lo = 0.0;
    while n > 0 {
        let x = hi;
        n -= 1;
        let y = partials[n];
        hi = x + y;
        lo = y - (hi - x);
        if lo != 0.0 {
            break;
        }
    }
    // Round half to even where the partials below make the half inexact.
    if n > 0 && ((lo < 0.0 && partials[n - 1] < 0.0) || (lo > 0.0 && partials[n - 1] > 0.0)) {
        let y = lo * 2.0;
        let x = hi + y;
        if y == x - hi {
            hi = x;
        }
    }
    hi
}

/// Appends `x` as Python's `repr` writes a float: its shortest digits
/// that read back as `x` (Rust's), positional for a decimal exponent from
/// -5 to 15 and scientific with a signed exponent of two digits or more
/// otherwise (`1e-05`, `0.0001`, `1.0`), so that the program is the one
/// `scripts/gen_bn.py` writes.
pub(super) fn write_float(out: &mut String, x: f64) {
    use std::fmt::Write;
    if !x.is_finite() {
        write!(out, "{x}").expect("a String takes any text");
        return;
    }
    let text = format!("{x:e}");
    let (mantissa, exp) = text.split_once('e').expect("Rust writes an exponent");
    let (sign, mantissa) = match mantissa.strip_prefix('-') {
        Some(m) => ("-", m),
        None => ("", mantissa),
    };
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let n = digits.len() as i32;
    // x = 0.digits · 10^point
    let point = exp.parse::<i32>().expect("an integer exponent") + 1;
    out.push_str(sign);
    if !(-4 < point && point <= 16) {
        out.push_str(&digits[..1]);
        if n > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        let e = point - 1;
        write!(out, "e{}{:02}", if e < 0 { '-' } else { '+' }, e.abs())
            .expect("a String takes any text");
    } else if point <= 0 {
        out.push_str("0.");
        out.extend(std::iter::repeat_n('0', (-point) as usize));
        out.push_str(&digits);
    } else if point < n {
        out.push_str(&digits[..point as usize]);
        out.push('.');
        out.push_str(&digits[point as usize..]);
    } else {
        out.push_str(&digits);
        out.extend(std::iter::repeat_n('0', (point - n) as usize));
        out.push_str(".0");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(x: f64) -> String {
        let mut s = String::new();
        write_float(&mut s, x);
        s
    }

    #[test]
    fn a_float_is_written_as_python_writes_its_repr() {
        for (x, repr) in [
            (0.5, "0.5"),
            (1.0, "1.0"),
            (0.0, "0.0"),
            (1e-05, "1e-05"),
            (0.0001, "0.0001"),
            (1.5e-7, "1.5e-07"),
            (9.999e-05, "9.999e-05"),
            (1e16, "1e+16"),
            (1234567890123456.0, "1234567890123456.0"),
            (0.30000000000000004, "0.30000000000000004"),
            (-0.4, "-0.4"),
            (-1.25e-100, "-1.25e-100"),
            (123.0, "123.0"),
            (12.5, "12.5"),
            (5e-324, "5e-324"),
        ] {
            assert_eq!(written(x), repr, "{x:e}");
        }
    }

    #[test]
    fn fsum_rounds_the_exact_sum() {
        assert_eq!(fsum(&[0.1, 0.2, 0.7]), 1.0);
        assert_eq!([0.1; 10].iter().sum::<f64>(), 0.9999999999999999);
        assert_eq!(fsum(&[0.1; 10]), 1.0);
        assert_eq!(fsum(&[1e100, 1.0, -1e100]), 1.0);
        assert_eq!(fsum(&[]), 0.0);
        assert_eq!(fsum(&[0.55, 0.34, 0.11]), 1.0);
    }

    fn row(xs: &[f64]) -> Result<Vec<f64>, String> {
        let mut r = xs.to_vec();
        normalized(&mut r).map(|()| r)
    }

    #[test]
    fn a_gap_that_rounding_explains_is_divided_by_the_sum() {
        for r in [
            &[0.3333333; 3][..],
            &[0.333; 3],
            &[0.3333334, 0.3333334, 0.3333333],
            &[0.12345678, 0.43209877, 0.44444444],
            &[0.9999, 9.999e-05],
            &[0.3, 0.7000000000000003],
        ] {
            let out = row(r).unwrap();
            assert!((fsum(&out) - 1.0).abs() < 1e-15, "{r:?}");
            assert_ne!(out, r, "{r:?}");
        }
    }

    #[test]
    fn a_gap_that_rounding_does_not_explain_is_the_tables() {
        for r in [
            &[0.5, 0.499][..],
            &[0.999],
            &[0.3333333, 0.3333333, 0.333333],
            &[0.4, 0.5],
            &[0.0, 0.0],
        ] {
            assert_eq!(row(r).unwrap(), r);
        }
        assert!(row(&[0.9, 0.101]).unwrap_err().contains("sum to 1.001"));
        assert!(row(&[0.3, 0.700000002]).is_err());
        assert!(row(&[0.5, -0.5]).is_err());
    }

    #[test]
    fn a_row_that_sums_to_one_keeps_its_numbers() {
        assert_eq!(row(&[0.2, 0.8]).unwrap(), [0.2, 0.8]);
        assert_eq!(row(&[1.0, 0.0]).unwrap(), [1.0, 0.0]);
        assert_eq!(row(&[0.1, 0.2, 0.7]).unwrap(), [0.1, 0.2, 0.7]);
        assert_eq!(row(&[1.0 - 1e-10]).unwrap(), [1.0 - 1e-10]);
    }

    #[test]
    fn an_entry_means_its_decimal_without_the_binary_noise() {
        let meant = |x: f64| {
            let d = Decimal::meant(x);
            (d.mantissa, d.digits, d.exp)
        };
        assert_eq!(meant(0.007262920000000001), (726292, 6, -8));
        assert_eq!(meant(0.0009940400000000005), (99404, 5, -8));
        assert_eq!(meant(0.0009980000000000004), (998, 3, -6));
        assert_eq!(meant(0.30000000000000004), (3, 1, -1));
        assert_eq!(meant(0.6666666666666666), (6666666666666666, 16, -16));
        assert_eq!(meant(0.7000000000000003), (7000000000000003, 16, -16));
        assert_eq!(meant(0.25), (25, 2, -2));
        assert_eq!(meant(1.0), (1, 1, 0));
        assert_eq!(meant(1e-300), (1, 1, -300));
        assert_eq!(meant(0.0), (0, 0, 0));
        // Pathfinder's rows: off one by 1e-8 in their eighth decimal.
        for r in [
            &[0.0, 0.99403579, 0.00497018, 0.0009940400000000005][..],
            &[
                0.0,
                0.99800399,
                0.0009980000000000004,
                0.0009980000000000004,
            ],
            &[0.0, 0.99273709, 0.007262920000000001],
        ] {
            assert!(explained(r), "{r:?}");
        }
    }

    #[test]
    fn decimals_far_apart_sum_exactly() {
        // The places of 0.3333333 and 1e-300 are 293 digits apart, and the
        // gap of 1e-7 - 1e-300 is still under the slack of 1.5e-7.
        let thirds = [0.3333333, 0.3333333, 0.3333333, 1e-300];
        assert!(explained(&thirds));
        assert!(!explained(&[0.3333333, 0.3333333, 0.333333, 1e-300]));
        assert_ne!(row(&thirds).unwrap(), thirds);
        let n = Natural::scaled(5, 40);
        let mut m = Natural::scaled(1, 40);
        m.add(&Natural::scaled(4, 40));
        assert!(n == m);
        assert_eq!(n.minus(&Natural::scaled(5, 40)), Natural::default());
    }
}
