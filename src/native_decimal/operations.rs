// SPDX-License-Identifier: AGPL-3.0-or-later
//! The proof-path bodies of the native Decimal operations (FR-403): admission,
//! evaluation, the fixed-step gcd and the ordering.
//!
//! Everything here is fixed-width native arithmetic: no loop, iterator adapter
//! or recursion, no allocation, no owner `Integer`, `Decimal` or `Rational`,
//! and one body for host and `cfg(kani)` builds. The owner-facing read-out
//! lives in `readout.rs`, outside the proof path.

use core::cmp::Ordering;

use super::{NativeDecimal, NativeDecimalType, POWERS_OF_TEN};
use crate::accounting::{Charge, ChargePoint, Incomplete, LimitKind, Meter};
use crate::decimal::RoundingMode;
use crate::native_fraction::gcd256;
use crate::numeric::OrderingOperator;

#[cfg(verus_keep_ghost)]
#[allow(unused_imports)]
use vstd::prelude::*;

#[cfg(verus_keep_ghost)]
mod verus_proofs;

/// The largest shift whose power of ten fits signed 128 bits.
#[cfg_attr(verus_keep_ghost, verus_spec)]
const MAX_SHIFT: u32 = 38;

/// Fractional bits of the fixed-point `log2(10)` approximations.
#[cfg_attr(verus_keep_ghost, verus_spec)]
pub(super) const LOG2_10_FRACTION: u32 = 90;
/// `floor(log2(10) x 2^90)`: the lower approximation, used for `bits(10^k)`.
#[cfg_attr(verus_keep_ghost, verus_spec)]
pub(super) const LOG2_10_LOWER: u128 = 4_112_347_796_288_069_798_443_794_287;
/// The upper approximation, one unit in the last place above the lower one.
#[cfg(test)]
pub(super) const LOG2_10_UPPER: u128 = LOG2_10_LOWER + 1;

/// `5^1, 5^2, 5^4, 5^8, 5^16, 5^32`, the ladder of the five-adic strip.
#[cfg_attr(verus_keep_ghost, verus_spec)]
const FIVE_LADDER: [u128; 6] = [
    5,
    25,
    625,
    390_625,
    152_587_890_625,
    23_283_064_365_386_962_890_625,
];

/// One native Decimal operation over carried operands.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeDecimalOperation {
    /// `a + b`.
    Add(NativeDecimal, NativeDecimal),
    /// `a - b`.
    Subtract(NativeDecimal, NativeDecimal),
    /// `a * b`.
    Multiply(NativeDecimal, NativeDecimal),
    /// `a / b`.
    Divide(NativeDecimal, NativeDecimal),
    /// `-a`.
    Negate(NativeDecimal),
    /// Explicit conversion of `a` into the target type.
    Round(NativeDecimal),
}

/// The first quantity of FR-403 Behavior 3 that left the native width.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeDecimalCapacityQuantity {
    /// A power of ten above 38 for a plan shift or the lift to the target
    /// scale (ground a).
    Exponent,
    /// A shifted operand of `Add` or `Subtract` (ground b).
    ShiftedOperand,
    /// The aligned sum (ground c).
    Sum,
    /// The aligned difference (ground c).
    Difference,
    /// The product (ground c).
    Product,
    /// The negation (ground c).
    Negation,
    /// The numerator `N` of a `Divide` (ground c).
    Numerator,
    /// The denominator `D` of a `Divide` (ground c).
    Denominator,
    /// The reduced numerator or denominator of a `Divide` (ground d).
    CanonicalQuotient,
    /// The power of ten the owner divides by (ground e).
    PowerOfTen,
    /// Twice the remainder of a nearest comparison (ground f).
    DoubledRemainder,
    /// The retained coefficient after the lift (ground g).
    RetainedCoefficient,
}

/// A typed capacity refusal: the native path claims nothing for the operation.
/// It is neither an arithmetic refusal, a Meter denial nor a completed value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("native decimal capacity exceeded at {quantity:?}")]
pub struct NativeDecimalCapacityRefusal {
    /// The first failing quantity, in the order of FR-403 Behavior 3.
    pub quantity: NativeDecimalCapacityQuantity,
}

/// The loss record of a rounding step, factored as the owner's `DecimalLoss`:
/// the exact value is `numerator / (cofactor x 2^twos x 5^fives)`.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeDecimalLoss {
    pub(super) numerator: i128,
    pub(super) cofactor: i128,
    pub(super) twos: u64,
    pub(super) fives: u64,
    pub(super) rounded: NativeDecimal,
    pub(super) mode: RoundingMode,
}

impl NativeDecimalLoss {
    /// Reduced exact numerator.
    pub const fn numerator(&self) -> i128 {
        self.numerator
    }
    /// Positive cofactor of the denominator, coprime to ten.
    pub const fn cofactor(&self) -> i128 {
        self.cofactor
    }
    /// Exponent of two in the denominator.
    pub const fn twos(&self) -> u64 {
        self.twos
    }
    /// Exponent of five in the denominator.
    pub const fn fives(&self) -> u64 {
        self.fives
    }
    /// The rounded retained representation `(rounded, T)`.
    pub const fn rounded(&self) -> NativeDecimal {
        self.rounded
    }
    /// The rounding mode.
    pub const fn mode(&self) -> RoundingMode {
        self.mode
    }
}

/// A completed native result: the retained `(v x 10^T, T)` and the optional
/// loss record.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeDecimalResult {
    pub(super) value: NativeDecimal,
    pub(super) loss: Option<NativeDecimalLoss>,
}

impl NativeDecimalResult {
    /// The retained result.
    pub const fn value(&self) -> NativeDecimal {
        self.value
    }
    /// The loss record when a rounding step occurred.
    pub const fn loss(&self) -> Option<NativeDecimalLoss> {
        self.loss
    }
}

/// The charge amounts of one operation, fixed at admission.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Sizes {
    operand_bits: u64,
    operand_digits: u64,
    operand_count: u64,
    shift: u64,
    shifted: Option<(u64, u64)>,
    arithmetic_bits: u64,
    arithmetic_digits: u64,
}

/// What admission decided for the operation after the charges.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tail {
    DivisionByZero,
    Inexact,
    Placed(Placement),
}

/// The placed coefficient at scale `scale`, its retained lift and the loss.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Placement {
    coefficient: i128,
    scale: u32,
    retained: i128,
    loss: Option<NativeDecimalLoss>,
}

/// An operation that passed admission: an opaque value that retains the type,
/// the charge amounts and the decided placement.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmittedNativeDecimalOperation {
    pub(super) ty: NativeDecimalType,
    sizes: Sizes,
    tail: Tail,
}

/// The exact intermediate of an operation.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy)]
enum Intermediate {
    /// The integer `value` in units of `10^-scale`.
    Scaled { value: i128, scale: u64 },
    /// The reduced `numerator / denominator` in units of `10^-T`.
    Quotient { numerator: i128, denominator: i128 },
}

/// The planned operation: its sizes and either a zero divisor or the
/// intermediate.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy)]
struct Planned {
    sizes: Sizes,
    intermediate: Option<Intermediate>,
}

type Admission<T> = Result<T, NativeDecimalCapacityRefusal>;

#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == verus_proofs::capacity(found, quantity)))]
fn capacity<T>(found: Option<T>, quantity: NativeDecimalCapacityQuantity) -> Admission<T> {
    found.ok_or(NativeDecimalCapacityRefusal { quantity })
}

/// `bits(m)` of a magnitude, at least 1.
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == super::verus_proofs::bits_of(magnitude), 1 <= r <= 128))]
pub(super) fn bits_of(magnitude: u128) -> u64 {
    #[cfg(verus_keep_ghost)]
    proof! { super::verus_proofs::lemma_bits_of(magnitude); }
    u64::from((128 - magnitude.leading_zeros()).max(1))
}

/// `digits(m)` of a magnitude, at least 1, from the bit length and one
/// comparison with a power of ten.
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == super::verus_proofs::digits_of(magnitude), 1 <= r <= 39))]
pub(super) fn digits_of(magnitude: u128) -> u64 {
    #[cfg(verus_keep_ghost)]
    proof! { super::verus_proofs::lemma_digits_of(magnitude); }
    if magnitude == 0 {
        return 1;
    }
    let bits = (128 - magnitude.leading_zeros()) as usize;
    let estimate = (bits * 1233) >> 12;
    let power = POWERS_OF_TEN[estimate] as u128;
    (estimate as u64 + 1) - u64::from(magnitude < power)
}

/// `bits(10^k) = floor(k x log2(10)) + 1` by one fixed-width multiplication.
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures 1 <= r <= 0x4_0000_0000))]
pub(super) fn power_of_ten_bits(shift: u32) -> u64 {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_power_of_ten_bits(shift); }
    (((shift as u128) * LOG2_10_LOWER) >> LOG2_10_FRACTION) as u64 + 1
}

/// `sbits(c, k)`: `bits(c)` for `k = 0`, else `bits(c) + bits(10^k)`.
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r <= 0x5_0000_0000))]
fn sbits(magnitude: u128, shift: u32) -> u64 {
    if shift == 0 {
        bits_of(magnitude)
    } else {
        bits_of(magnitude) + power_of_ten_bits(shift)
    }
}

/// `sdigits(c, k) = digits(c) + k`.
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r <= 0x2_0000_0000))]
fn sdigits(magnitude: u128, shift: u32) -> u64 {
    digits_of(magnitude) + u64::from(shift)
}

#[cfg_attr(verus_keep_ghost, verus_spec)]
fn unary_sizes(c: i128) -> (u64, u64, u64) {
    let magnitude = c.unsigned_abs();
    (bits_of(magnitude), digits_of(magnitude), 1)
}

#[cfg_attr(verus_keep_ghost, verus_spec)]
fn binary_sizes(a: i128, b: i128) -> (u64, u64, u64) {
    let (x, y) = (a.unsigned_abs(), b.unsigned_abs());
    (
        bits_of(x).max(bits_of(y)),
        digits_of(x).max(digits_of(y)),
        2,
    )
}

/// The first shifted operand whose shift is above zero, left before right.
#[cfg_attr(verus_keep_ghost, verus_spec)]
fn expansion(first: (i128, u32), second: (i128, u32)) -> (u64, Option<(u64, u64)>) {
    if first.1 > 0 {
        expanded(first.0, first.1)
    } else if second.1 > 0 {
        expanded(second.0, second.1)
    } else {
        (0, None)
    }
}

/// The scale-expansion amounts of one shifted operand.
#[cfg_attr(verus_keep_ghost, verus_spec)]
fn expanded(c: i128, k: u32) -> (u64, Option<(u64, u64)>) {
    let magnitude = c.unsigned_abs();
    (
        u64::from(k),
        Some((sbits(magnitude, k), sdigits(magnitude, k))),
    )
}

#[cfg_attr(verus_keep_ghost, verus_spec)]
fn sizes(
    operands: (u64, u64, u64),
    expanded: (u64, Option<(u64, u64)>),
    arithmetic: (u64, u64),
) -> Sizes {
    Sizes {
        operand_bits: operands.0,
        operand_digits: operands.1,
        operand_count: operands.2,
        shift: expanded.0,
        shifted: expanded.1,
        arithmetic_bits: arithmetic.0,
        arithmetic_digits: arithmetic.1,
    }
}

/// Ground (a) for the lift: `T - min(W, T)` is the shift of the placed
/// coefficient to `T`, and `10^k` fits only for `k <= 38`.
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == verus_proofs::lift_fits(ty.max_scale, working_scale)))]
fn lift_fits(ty: &NativeDecimalType, working_scale: u64) -> bool {
    let target = u64::from(ty.max_scale);
    target - working_scale.min(target) <= u64::from(MAX_SHIFT)
}

#[cfg_attr(verus_keep_ghost, verus_spec(r => requires shift <= 38, ensures r == verus_proofs::shifted(c, shift)))]
fn shifted(c: i128, shift: u32) -> Option<i128> {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_shifted(c, shift); }
    c.checked_mul(POWERS_OF_TEN[shift as usize])
}

#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures verus_proofs::planned_is(
    r,
    verus_proofs::combine_refusal(ty.max_scale, a, b, subtract),
    Some(Intermediate::Scaled {
        value: verus_proofs::combine_value(a, b, subtract) as i128,
        scale: verus_proofs::combine_scale(a, b) as u64,
    }),
)))]
fn plan_combine(
    ty: &NativeDecimalType,
    a: NativeDecimal,
    b: NativeDecimal,
    subtract: bool,
) -> Admission<Planned> {
    let scale = a.scale.max(b.scale);
    let (ka, kb) = (scale - a.scale, scale - b.scale);
    capacity(
        (ka <= MAX_SHIFT && kb <= MAX_SHIFT && lift_fits(ty, u64::from(scale))).then_some(()),
        NativeDecimalCapacityQuantity::Exponent,
    )?;
    let xa = capacity(
        shifted(a.coefficient, ka),
        NativeDecimalCapacityQuantity::ShiftedOperand,
    )?;
    let xb = capacity(
        shifted(b.coefficient, kb),
        NativeDecimalCapacityQuantity::ShiftedOperand,
    )?;
    let value = if subtract {
        capacity(
            xa.checked_sub(xb),
            NativeDecimalCapacityQuantity::Difference,
        )?
    } else {
        capacity(xa.checked_add(xb), NativeDecimalCapacityQuantity::Sum)?
    };
    let (ma, mb) = (a.coefficient.unsigned_abs(), b.coefficient.unsigned_abs());
    Ok(Planned {
        sizes: sizes(
            binary_sizes(a.coefficient, b.coefficient),
            expansion((a.coefficient, ka), (b.coefficient, kb)),
            (
                sbits(ma, ka).max(sbits(mb, kb)) + 1,
                sdigits(ma, ka).max(sdigits(mb, kb)) + 1,
            ),
        ),
        intermediate: Some(Intermediate::Scaled {
            value,
            scale: u64::from(scale),
        }),
    })
}

#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures verus_proofs::planned_is(
    r,
    verus_proofs::multiply_refusal(ty.max_scale, a, b),
    Some(Intermediate::Scaled {
        value: (a.coefficient * b.coefficient) as i128,
        scale: (a.scale + b.scale) as u64,
    }),
)))]
fn plan_multiply(ty: &NativeDecimalType, a: NativeDecimal, b: NativeDecimal) -> Admission<Planned> {
    let scale = u64::from(a.scale) + u64::from(b.scale);
    capacity(
        lift_fits(ty, scale).then_some(()),
        NativeDecimalCapacityQuantity::Exponent,
    )?;
    let value = capacity(
        a.coefficient.checked_mul(b.coefficient),
        NativeDecimalCapacityQuantity::Product,
    )?;
    let (ma, mb) = (a.coefficient.unsigned_abs(), b.coefficient.unsigned_abs());
    Ok(Planned {
        sizes: sizes(
            binary_sizes(a.coefficient, b.coefficient),
            (0, None),
            (bits_of(ma) + bits_of(mb), digits_of(ma) + digits_of(mb)),
        ),
        intermediate: Some(Intermediate::Scaled { value, scale }),
    })
}

#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures verus_proofs::planned_is(
    r,
    verus_proofs::unary_refusal(ty.max_scale, a, negate),
    Some(Intermediate::Scaled {
        value: (if negate { -a.coefficient } else { a.coefficient as int }) as i128,
        scale: a.scale as u64,
    }),
)))]
fn plan_unary(ty: &NativeDecimalType, a: NativeDecimal, negate: bool) -> Admission<Planned> {
    let scale = u64::from(a.scale);
    capacity(
        lift_fits(ty, scale).then_some(()),
        NativeDecimalCapacityQuantity::Exponent,
    )?;
    let value = if negate {
        capacity(
            a.coefficient.checked_neg(),
            NativeDecimalCapacityQuantity::Negation,
        )?
    } else {
        a.coefficient
    };
    let magnitude = a.coefficient.unsigned_abs();
    Ok(Planned {
        sizes: sizes(
            unary_sizes(a.coefficient),
            (0, None),
            (bits_of(magnitude), digits_of(magnitude)),
        ),
        intermediate: Some(Intermediate::Scaled { value, scale }),
    })
}

#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures verus_proofs::divide_planned(ty.max_scale, a, b, r)))]
fn plan_divide(ty: &NativeDecimalType, a: NativeDecimal, b: NativeDecimal) -> Admission<Planned> {
    let operands = binary_sizes(a.coefficient, b.coefficient);
    if b.coefficient == 0 {
        // The owner returns `Undefined(DivisionByZero)` right after the
        // operands charge and forms no plan quantity: no ground applies.
        return Ok(Planned {
            sizes: sizes(operands, (0, None), (0, 0)),
            intermediate: None,
        });
    }
    let up = u64::from(ty.max_scale) + u64::from(b.scale);
    let (kn, kd) = (
        up.saturating_sub(u64::from(a.scale)),
        u64::from(a.scale).saturating_sub(up),
    );
    capacity(
        (kn <= u64::from(MAX_SHIFT) && kd <= u64::from(MAX_SHIFT)).then_some(()),
        NativeDecimalCapacityQuantity::Exponent,
    )?;
    let (kn, kd) = (kn as u32, kd as u32);
    let n = capacity(
        shifted(a.coefficient, kn),
        NativeDecimalCapacityQuantity::Numerator,
    )?;
    let d = capacity(
        shifted(b.coefficient, kd),
        NativeDecimalCapacityQuantity::Denominator,
    )?;
    let (ma, mb) = (a.coefficient.unsigned_abs(), b.coefficient.unsigned_abs());
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_scaled_nonzero(b.coefficient as int, kd as nat); }
    let (numerator, denominator) = reduce_quotient(n, d)?;
    Ok(Planned {
        sizes: sizes(
            operands,
            expansion((a.coefficient, kn), (b.coefficient, kd)),
            (
                sbits(ma, kn).max(sbits(mb, kd)),
                sdigits(ma, kn).max(sdigits(mb, kd)),
            ),
        ),
        intermediate: Some(Intermediate::Quotient {
            numerator,
            denominator,
        }),
    })
}

/// The canonical rational `N' / D'` of a nonzero `D`: the sign on the
/// numerator, the common factor removed by the 256-step gcd (ground d).
#[cfg_attr(verus_keep_ghost, verus_spec(r => requires d != 0, ensures verus_proofs::reduced_is(n as int, d as int, r)))]
fn reduce_quotient(n: i128, d: i128) -> Admission<(i128, i128)> {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_reduce_quotient(n, d); }
    let gcd = gcd256(n.unsigned_abs(), d.unsigned_abs());
    let (magnitude, denominator) = (n.unsigned_abs() / gcd, d.unsigned_abs() / gcd);
    let negative = (n < 0) != (d < 0);
    let numerator = if negative {
        if magnitude <= 1_u128 << 127 {
            Some((magnitude as i128).wrapping_neg())
        } else {
            None
        }
    } else {
        i128::try_from(magnitude).ok()
    };
    let denominator = i128::try_from(denominator).ok();
    match (numerator, denominator) {
        (Some(numerator), Some(denominator)) => Ok((numerator, denominator)),
        _ => capacity(None, NativeDecimalCapacityQuantity::CanonicalQuotient),
    }
}

#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires count <= limit, power > 0,
    ensures r == verus_proofs::strip_five(magnitude, count, limit, width, power)))]
fn strip_five(magnitude: u128, count: u64, limit: u64, width: u64, power: u128) -> (u128, u64) {
    if width <= limit - count && magnitude.is_multiple_of(power) {
        (magnitude / power, count + width)
    } else {
        (magnitude, count)
    }
}

/// `(m / 5^k, k)` for the greatest `k <= limit` with `5^k | m`, nonzero `m`.
#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires magnitude != 0,
    ensures r == verus_proofs::strip_fives(magnitude, limit)))]
fn strip_fives(magnitude: u128, limit: u64) -> (u128, u64) {
    let (m, c) = strip_five(magnitude, 0, limit, 32, FIVE_LADDER[5]);
    let (m, c) = strip_five(m, c, limit, 16, FIVE_LADDER[4]);
    let (m, c) = strip_five(m, c, limit, 8, FIVE_LADDER[3]);
    let (m, c) = strip_five(m, c, limit, 4, FIVE_LADDER[2]);
    let (m, c) = strip_five(m, c, limit, 2, FIVE_LADDER[1]);
    strip_five(m, c, limit, 1, FIVE_LADDER[0])
}

/// `(m / 2^k, k)` for the greatest `k <= limit` with `2^k | m`, nonzero `m`.
#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires magnitude != 0,
    ensures r == verus_proofs::strip_twos(magnitude, limit)))]
fn strip_twos(magnitude: u128, limit: u64) -> (u128, u64) {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_strip_twos(magnitude, limit); }
    let count = u64::from(magnitude.trailing_zeros()).min(limit);
    (magnitude >> count, count)
}

/// The owner's canonical loss value of `numerator / denominator / 10^ten`
/// for a reduced pair and a nonzero numerator, with the working-scale power
/// of ten kept factored and its common twos and fives cancelled.
#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires numerator != 0, denominator > 0, ten_exponent <= 0x2_0000_0000,
    ensures
        r == verus_proofs::loss_record(numerator, denominator, ten_exponent, rounded, mode),
        verus_proofs::loss_is(numerator as int, denominator as int, ten_exponent as nat, rounded, mode, r)))]
fn loss_record(
    numerator: i128,
    denominator: i128,
    ten_exponent: u64,
    rounded: NativeDecimal,
    mode: RoundingMode,
) -> NativeDecimalLoss {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_loss_record(numerator, denominator, ten_exponent, rounded, mode); }
    let (magnitude, cancelled_twos) = strip_twos(numerator.unsigned_abs(), ten_exponent);
    let (magnitude, cancelled_fives) = strip_fives(magnitude, ten_exponent);
    let (rest, denominator_twos) = strip_twos(denominator.unsigned_abs(), u64::MAX);
    let (cofactor, denominator_fives) = strip_fives(rest, u64::MAX);
    let signed = magnitude as i128;
    NativeDecimalLoss {
        numerator: if numerator < 0 {
            signed.wrapping_neg()
        } else {
            signed
        },
        cofactor: cofactor as i128,
        twos: denominator_twos + (ten_exponent - cancelled_twos),
        fives: denominator_fives + (ten_exponent - cancelled_fives),
        rounded,
        mode,
    }
}

/// Round `numerator / denominator` (reduced, `denominator >= 2`) to an
/// integer. The nearest modes compare twice the remainder (ground f).
#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires denominator >= 2, (numerator as int) % (denominator as int) != 0,
    ensures verus_proofs::round_units_is(numerator as int, denominator as int, mode, r)))]
fn round_units(numerator: i128, denominator: i128, mode: RoundingMode) -> Admission<i128> {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_round_units(numerator, denominator); }
    let floor = numerator.div_euclid(denominator);
    let remainder = numerator.rem_euclid(denominator);
    let ceiling = floor + 1;
    let negative = numerator < 0;
    match mode {
        RoundingMode::Exact => Ok(floor),
        RoundingMode::TowardZero => Ok(if negative { ceiling } else { floor }),
        RoundingMode::TowardPositive => Ok(ceiling),
        RoundingMode::TowardNegative => Ok(floor),
        RoundingMode::NearestEven | RoundingMode::NearestAway => {
            let twice = capacity(
                remainder.checked_add(remainder),
                NativeDecimalCapacityQuantity::DoubledRemainder,
            )?;
            Ok(match twice.cmp(&denominator) {
                Ordering::Less => floor,
                Ordering::Greater => ceiling,
                Ordering::Equal => tie(mode, floor, ceiling, negative),
            })
        }
    }
}

#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == verus_proofs::tie(mode, floor, ceiling, negative)))]
fn tie(mode: RoundingMode, floor: i128, ceiling: i128, negative: bool) -> i128 {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_low_bit(floor); }
    let even = if floor & 1 == 0 { floor } else { ceiling };
    let away = if negative { floor } else { ceiling };
    if mode == RoundingMode::NearestEven {
        even
    } else {
        away
    }
}

/// A coefficient exactly placed at scale `scale <= T`, lifted to `T`.
#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires scale <= ty.max_scale,
    ensures r == verus_proofs::exact_tail(ty.max_scale, coefficient, scale)))]
fn exact_tail(ty: &NativeDecimalType, coefficient: i128, scale: u32) -> Admission<Tail> {
    let lift = ty.max_scale - scale;
    capacity(
        (lift <= MAX_SHIFT).then_some(()),
        NativeDecimalCapacityQuantity::Exponent,
    )?;
    let retained = capacity(
        shifted(coefficient, lift),
        NativeDecimalCapacityQuantity::RetainedCoefficient,
    )?;
    Ok(Tail::Placed(Placement {
        coefficient,
        scale,
        retained,
        loss: None,
    }))
}

/// A rounding step over `numerator / denominator` in units of `10^-T`, whose
/// exact loss value is `loss_numerator / loss_denominator / 10^ten`.
#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires
        units.1 >= 2,
        (units.0 as int) % (units.1 as int) != 0,
        exact.0 != 0,
        exact.1 > 0,
        exact.2 <= 0x2_0000_0000,
    ensures verus_proofs::rounded_is(
        ty.rounding,
        ty.max_scale,
        units.0 as int,
        units.1 as int,
        2 * ((units.0 as int) % (units.1 as int)) > i128::MAX,
        (exact.0 as int, exact.1 as int, exact.2 as nat),
        r,
    )))]
fn rounding_tail(
    ty: &NativeDecimalType,
    units: (i128, i128),
    exact: (i128, i128, u64),
) -> Admission<Tail> {
    if ty.rounding == RoundingMode::Exact {
        return Ok(Tail::Inexact);
    }
    let rounded = round_units(units.0, units.1, ty.rounding)?;
    let value = NativeDecimal::from_retained(rounded, ty.max_scale);
    Ok(Tail::Placed(Placement {
        coefficient: rounded,
        scale: ty.max_scale,
        retained: rounded,
        loss: Some(loss_record(exact.0, exact.1, exact.2, value, ty.rounding)),
    }))
}

#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires working <= 0x2_0000_0000,
    ensures verus_proofs::scaled_is(ty.rounding, ty.max_scale, value, working, r)))]
fn place_scaled(ty: &NativeDecimalType, value: i128, working: u64) -> Admission<Tail> {
    let target = u64::from(ty.max_scale);
    if working <= target {
        return exact_tail(ty, value, working as u32);
    }
    if value == 0 {
        return exact_tail(ty, 0, ty.max_scale);
    }
    let excess = working - target;
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_place_scaled(value, excess as nat); }
    if excess > digits_of(value.unsigned_abs()) {
        // The proxy of one tenth of the sign: no power of ten is formed.
        return rounding_tail(ty, (value.signum(), 10), (value, 1, working));
    }
    capacity(
        (excess <= u64::from(MAX_SHIFT)).then_some(()),
        NativeDecimalCapacityQuantity::PowerOfTen,
    )?;
    let power = POWERS_OF_TEN[excess as usize];
    if value % power == 0 {
        return exact_tail(ty, value / power, ty.max_scale);
    }
    let gcd = gcd256(value.unsigned_abs(), power.unsigned_abs()) as i128;
    rounding_tail(ty, (value / gcd, power / gcd), (value, 1, working))
}

#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires crate::native_fraction::verus_proofs::canonical(numerator as int, denominator as int),
    ensures verus_proofs::quotient_is(ty.rounding, ty.max_scale, numerator, denominator, r)))]
fn place_quotient(ty: &NativeDecimalType, numerator: i128, denominator: i128) -> Admission<Tail> {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_place_quotient(numerator, denominator); }
    if denominator == 1 {
        return exact_tail(ty, numerator, ty.max_scale);
    }
    rounding_tail(
        ty,
        (numerator, denominator),
        (numerator, denominator, u64::from(ty.max_scale)),
    )
}

#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures verus_proofs::planned_is(
    r,
    verus_proofs::plan_refusal(ty.max_scale, operation),
    verus_proofs::plan_intermediate(ty.max_scale, operation),
)))]
fn plan(ty: &NativeDecimalType, operation: NativeDecimalOperation) -> Admission<Planned> {
    match operation {
        NativeDecimalOperation::Add(a, b) => plan_combine(ty, a, b, false),
        NativeDecimalOperation::Subtract(a, b) => plan_combine(ty, a, b, true),
        NativeDecimalOperation::Multiply(a, b) => plan_multiply(ty, a, b),
        NativeDecimalOperation::Divide(a, b) => plan_divide(ty, a, b),
        NativeDecimalOperation::Negate(a) => plan_unary(ty, a, true),
        NativeDecimalOperation::Round(a) => plan_unary(ty, a, false),
    }
}

#[cfg(verus_keep_ghost)]
verus! {

impl NativeDecimalType {
    /// What `admit` returns for an operation; the statement is
    /// `verus_proofs::admitted_is`.
    pub closed spec fn admission(
        self,
        operation: NativeDecimalOperation,
        r: Result<AdmittedNativeDecimalOperation, NativeDecimalCapacityRefusal>,
    ) -> bool {
        verus_proofs::admitted_is(self, operation, r)
    }
}

impl NativeDecimal {
    /// The order of two values by mathematical value; the statement is
    /// `verus_proofs::value_order`.
    pub(super) closed spec fn order_by_value(self, right: NativeDecimal) -> Ordering {
        verus_proofs::value_order(self, right)
    }
}

impl AdmittedNativeDecimalOperation {
    /// A placed coefficient sits at a scale at most the target scale.
    pub closed spec fn well_formed(self) -> bool {
        verus_proofs::admitted_wf(self)
    }

    /// What `settle` returns for this admitted operation; the statement is
    /// `verus_proofs::settled_is`.
    pub(super) closed spec fn settlement(self, r: Settled) -> bool {
        verus_proofs::settled_is(self, r)
    }
}

} // verus!
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl NativeDecimalType {
    /// Decide admission from the operands and this type alone, with native
    /// arithmetic only: no `Meter`, no charge, no allocation. A refusal names
    /// the first failing quantity of FR-403 Behavior 3; a zero divisor is
    /// admitted with no ground applied.
    #[cfg_attr(verus_keep_ghost, verus_spec(r => ensures self.admission(operation, r)))]
    pub fn admit(
        &self,
        operation: NativeDecimalOperation,
    ) -> Result<AdmittedNativeDecimalOperation, NativeDecimalCapacityRefusal> {
        #[cfg(verus_keep_ghost)]
        proof! { verus_proofs::lemma_planned(self.max_scale, operation); }
        let planned = plan(self, operation)?;
        let tail = match planned.intermediate {
            None => Tail::DivisionByZero,
            Some(Intermediate::Scaled { value, scale }) => place_scaled(self, value, scale)?,
            Some(Intermediate::Quotient {
                numerator,
                denominator,
            }) => place_quotient(self, numerator, denominator)?,
        };
        Ok(AdmittedNativeDecimalOperation {
            ty: *self,
            sizes: planned.sizes,
            tail,
        })
    }
}

/// The decision of an admitted operation, without any charge.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Settled {
    DivisionByZero,
    Inexact,
    OutOfDomain { rounding: bool },
    Completed(NativeDecimalResult, Retain),
}

/// The placed coefficient and its lift, for the `decimal.result-retain` charge.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Retain {
    pub(super) coefficient: i128,
    pub(super) lift: u32,
}

/// The typed outcome of an admitted operation: the rounding step is refused
/// under `exact`, the placed value is judged by the type's membership rule
/// after rounding, and a member is retained at `T`.
#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires operation.well_formed(),
    ensures operation.settlement(r)))]
pub(super) fn settle(operation: &AdmittedNativeDecimalOperation) -> Settled {
    let ty = &operation.ty;
    match operation.tail {
        Tail::DivisionByZero => Settled::DivisionByZero,
        Tail::Inexact => Settled::Inexact,
        Tail::Placed(placement) => {
            let placed = NativeDecimal::from_retained(placement.coefficient, placement.scale);
            if ty.admits(&placed) {
                Settled::Completed(
                    NativeDecimalResult {
                        value: NativeDecimal::from_retained(placement.retained, ty.max_scale),
                        loss: placement.loss,
                    },
                    Retain {
                        coefficient: placement.coefficient,
                        lift: ty.max_scale - placement.scale,
                    },
                )
            } else {
                Settled::OutOfDomain {
                    rounding: placement.loss.is_some(),
                }
            }
        }
    }
}

/// Why an admitted evaluation stopped.
#[derive(Debug)]
pub(super) enum NativeStop {
    Incomplete(Incomplete),
    DivisionByZero,
    Inexact,
    OutOfDomain,
}

impl From<Incomplete> for NativeStop {
    fn from(record: Incomplete) -> Self {
        Self::Incomplete(record)
    }
}

fn operands_charge(sizes: &Sizes) -> Charge {
    Charge::new(ChargePoint::DecimalOperands)
        .size(LimitKind::IntegerBits, sizes.operand_bits)
        .size(LimitKind::DecimalDigits, sizes.operand_digits)
        .size(LimitKind::ValueOccurrences, sizes.operand_count)
}

fn expansion_charge(sizes: &Sizes) -> Charge {
    let charge = Charge::new(ChargePoint::DecimalScaleExpansion)
        .size(LimitKind::ScaleExpansion, sizes.shift);
    match sizes.shifted {
        Some((bits, digits)) => charge
            .size(LimitKind::IntegerBits, bits)
            .size(LimitKind::DecimalDigits, digits),
        None => charge,
    }
}

/// `decimal.arithmetic` and `decimal.rounding` carry the same two amounts.
fn amounts_charge(point: ChargePoint, sizes: &Sizes) -> Charge {
    Charge::new(point)
        .size(LimitKind::IntegerBits, sizes.arithmetic_bits)
        .size(LimitKind::DecimalDigits, sizes.arithmetic_digits)
}

fn retain_charge(retain: Retain) -> Charge {
    let magnitude = retain.coefficient.unsigned_abs();
    Charge::new(ChargePoint::DecimalResultRetain)
        .size(LimitKind::ScaleExpansion, u64::from(retain.lift))
        .size(LimitKind::IntegerBits, sbits(magnitude, retain.lift))
        .size(LimitKind::DecimalDigits, sdigits(magnitude, retain.lift))
        .size(LimitKind::ValueOccurrences, 1)
        .results(1)
}

/// Charge in the owner's order and stop where the owner stops.
pub(super) fn evaluate(
    operation: AdmittedNativeDecimalOperation,
    meter: &mut Meter,
) -> Result<NativeDecimalResult, NativeStop> {
    let settled = settle(&operation);
    let sizes = &operation.sizes;
    meter.charge(operands_charge(sizes))?;
    if settled == Settled::DivisionByZero {
        return Err(NativeStop::DivisionByZero);
    }
    meter.charge(expansion_charge(sizes))?;
    meter.charge(amounts_charge(ChargePoint::DecimalArithmetic, sizes))?;
    match settled {
        Settled::DivisionByZero => Err(NativeStop::DivisionByZero),
        Settled::Inexact => Err(NativeStop::Inexact),
        Settled::OutOfDomain { rounding } => {
            if rounding {
                meter.charge(amounts_charge(ChargePoint::DecimalRounding, sizes))?;
            }
            Err(NativeStop::OutOfDomain)
        }
        Settled::Completed(result, retain) => {
            if result.loss.is_some() {
                meter.charge(amounts_charge(ChargePoint::DecimalRounding, sizes))?;
            }
            meter.charge(retain_charge(retain))?;
            Ok(result)
        }
    }
}

/// `x x 10^d` against `y` for nonzero magnitudes, without forming a power of
/// ten above 38: digit counts decide, and equal counts bound `d`.
#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires x != 0, y != 0,
    ensures r == verus_proofs::cmp_int(x * super::verus_proofs::pow10(shift as nat), y as int)))]
fn compare_shifted_magnitude(x: u128, shift: u32, y: u128) -> Ordering {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_compare_shifted(x, shift, y); }
    let (shifted_digits, digits) = (digits_of(x) + u64::from(shift), digits_of(y));
    if shifted_digits != digits {
        return shifted_digits.cmp(&digits);
    }
    let power = POWERS_OF_TEN[shift.min(MAX_SHIFT) as usize] as u128;
    let (high, low) = widening_mul(x, power);
    (high, low).cmp(&(0, y))
}

/// The full 256-bit product of two `u128` values as `(high, low)`.
#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    ensures r.0 * 0x1_0000_0000_0000_0000_0000_0000_0000_0000 + r.1 == a * b))]
pub(super) fn widening_mul(a: u128, b: u128) -> (u128, u128) {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_widening_mul(a, b); }
    const HALF: u128 = u64::MAX as u128;
    let (a1, a0, b1, b0) = (a >> 64, a & HALF, b >> 64, b & HALF);
    let (ll, lh, hl, hh) = (a0 * b0, a0 * b1, a1 * b0, a1 * b1);
    let middle = (ll >> 64) + (lh & HALF) + (hl & HALF);
    let low = (ll & HALF) | (middle << 64);
    let high = hh + (lh >> 64) + (hl >> 64) + (middle >> 64);
    (high, low)
}

#[cfg_attr(verus_keep_ghost, verus_spec(r =>
    requires left.coefficient != 0, right.coefficient != 0,
    ensures r == verus_proofs::magnitude_order(left, right)))]
fn compare_magnitudes(left: NativeDecimal, right: NativeDecimal) -> Ordering {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_compare_magnitudes(left, right); }
    let (x, y) = (
        left.coefficient.unsigned_abs(),
        right.coefficient.unsigned_abs(),
    );
    if left.scale == right.scale {
        x.cmp(&y)
    } else if left.scale > right.scale {
        compare_shifted_magnitude(y, left.scale - right.scale, x).reverse()
    } else {
        compare_shifted_magnitude(x, right.scale - left.scale, y)
    }
}

/// The order of two values by mathematical value, not by representation.
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == left.order_by_value(right)))]
pub(super) fn order_values(left: NativeDecimal, right: NativeDecimal) -> Ordering {
    #[cfg(verus_keep_ghost)]
    proof! { verus_proofs::lemma_order_values(left, right); }
    let (left_sign, right_sign) = (left.coefficient.signum(), right.coefficient.signum());
    if left_sign != right_sign {
        return left_sign.cmp(&right_sign);
    }
    if left_sign == 0 {
        return Ordering::Equal;
    }
    let magnitude = compare_magnitudes(left, right);
    if left_sign < 0 {
        magnitude.reverse()
    } else {
        magnitude
    }
}

/// Charge `ordering.operands`, `ordering.arithmetic` and
/// `ordering.result-retain`, then decide.
pub(super) fn order(
    operator: OrderingOperator,
    left: NativeDecimal,
    right: NativeDecimal,
    meter: &mut Meter,
) -> Result<bool, Incomplete> {
    let (x, y) = (
        left.coefficient.unsigned_abs(),
        right.coefficient.unsigned_abs(),
    );
    meter.charge(
        Charge::new(ChargePoint::OrderingOperands)
            .size(LimitKind::IntegerBits, bits_of(x).max(bits_of(y)))
            .size(LimitKind::DecimalDigits, digits_of(x).max(digits_of(y)))
            .size(LimitKind::ValueOccurrences, 2),
    )?;
    let aligned = left.scale.max(right.scale);
    let (left_shift, right_shift) = (aligned - left.scale, aligned - right.scale);
    meter.charge(
        Charge::new(ChargePoint::OrderingArithmetic)
            .size(
                LimitKind::ScaleExpansion,
                u64::from(left_shift.max(right_shift)),
            )
            .size(
                LimitKind::IntegerBits,
                sbits(x, left_shift).max(sbits(y, right_shift)),
            )
            .size(
                LimitKind::DecimalDigits,
                sdigits(x, left_shift).max(sdigits(y, right_shift)),
            ),
    )?;
    meter.charge(Charge::new(ChargePoint::OrderingResultRetain).results(1))?;
    Ok(operator.holds(order_values(left, right)))
}
