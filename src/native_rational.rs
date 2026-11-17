// SPDX-License-Identifier: AGPL-3.0-or-later
//! The Kani-tractable native Rational carrier and its add, subtract,
//! multiply, ordering and equality (FR-408).
//!
//! A [`NativeRational`] is a canonical fraction of two `i128` (lowest terms,
//! positive denominator, zero as `0/1`), `Copy` and without a heap type. The
//! operations refine the owner's `evaluate_rational_arithmetic`,
//! `order_numbers` and equality path on the Meter charges and on the exact
//! value wherever the formed integers fit `i128`; where one does not, the
//! operation returns a typed [`NativeRationalCapacity`] refusal before any
//! charge. The proof-path bodies use fixed-width arithmetic only: no loop,
//! iterator adapter, recursion or allocation, and one body for host and
//! `cfg(kani)` builds. The owner-facing conversions allocate and sit outside
//! that path.
//!
//! The fields are private, so a value can only be built canonical:
//!
//! ```compile_fail,E0451
//! let _ = filament_exact::NativeRational { numerator: 1, denominator: 1 };
//! ```

use crate::accounting::{ChargePoint, Incomplete, LimitKind, Meter, UnitsCharge};
use crate::integer::Integer;
use crate::native_fraction::{is_canonical_pair, reduce};
use crate::native_integer::{charge_native_integer_schedule, NativeIntegerSchedule};
use crate::numeric::OrderingOperator;
use crate::outcome::Outcome;
use crate::rational::Rational;

#[cfg(verus_keep_ghost)]
#[allow(unused_imports)]
use vstd::prelude::*;

/// The first formed integer that left signed 128 bits, in the order the
/// operations check them (FR-408 Behavior 9).
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum NativeRationalCapacity {
    /// A part of an owner `Rational` does not fit signed 128 bits.
    #[error("rational part does not fit signed 128 bits")]
    Input,
    /// `a q`, or for multiply `a c`.
    #[error("left product does not fit signed 128 bits")]
    LeftProduct,
    /// `c p`.
    #[error("right product does not fit signed 128 bits")]
    RightProduct,
    /// The sum or difference of the two products.
    #[error("sum or difference does not fit signed 128 bits")]
    Combination,
    /// `p q`.
    #[error("denominator product does not fit signed 128 bits")]
    Denominator,
}

/// A refusal of the native Rational path. The checks of
/// [`NativeRational::from_i64_parts`] run in the order `ZeroDenominator`,
/// `NonCanonical`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum NativeRationalRefusal {
    /// The denominator is zero.
    #[error("rational denominator is zero")]
    ZeroDenominator,
    /// The pair has a negative denominator, a common factor above one, or is
    /// zero over a denominator other than one.
    #[error("rational pair is not canonical")]
    NonCanonical,
    /// A formed integer does not fit signed 128 bits.
    #[error("native rational capacity exceeded: {0}")]
    Capacity(NativeRationalCapacity),
}

/// A canonical fraction of two signed 128-bit integers. Its fields are
/// private so every value is canonical: positive denominator, lowest terms,
/// zero as `0/1`.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct NativeRational {
    numerator: i128,
    denominator: i128,
}

#[cfg(verus_keep_ghost)]
vstd::prelude::verus! {

impl NativeRational {
    /// The carried `(numerator, denominator)`, for specifications.
    pub closed spec fn parts(self) -> (i128, i128) {
        (self.numerator, self.denominator)
    }
}

} // verus!
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl NativeRational {
    /// Build the carrier from `i64` parts, admitting only a canonical pair.
    /// Nothing is normalized: `2/4` is refused, never built as `1/2`.
    #[cfg_attr(verus_keep_ghost, verus_spec(r => ensures verus_proofs::from_i64_parts(numerator, denominator, r)))]
    pub fn from_i64_parts(numerator: i64, denominator: i64) -> Result<Self, NativeRationalRefusal> {
        if denominator == 0 {
            Err(NativeRationalRefusal::ZeroDenominator)
        } else if !is_canonical_pair(numerator, denominator) {
            Err(NativeRationalRefusal::NonCanonical)
        } else {
            Ok(Self {
                numerator: i128::from(numerator),
                denominator: i128::from(denominator),
            })
        }
    }

    /// Build the carrier from the owner's canonical `Rational`.
    #[cfg_attr(verus_keep_ghost, verus_verify(external))]
    pub fn from_rational(value: &Rational) -> Result<Self, NativeRationalRefusal> {
        match (value.numerator().native(), value.denominator().native()) {
            (Some(numerator), Some(denominator)) => Ok(Self {
                numerator,
                denominator,
            }),
            _ => Err(NativeRationalRefusal::Capacity(
                NativeRationalCapacity::Input,
            )),
        }
    }

    /// Read the value out as the owner's `Rational`. This allocates.
    #[cfg_attr(verus_keep_ghost, verus_verify(external))]
    pub fn to_rational(&self) -> Rational {
        Rational::new(
            Integer::from(self.numerator),
            Integer::from(self.denominator),
        )
        .expect("the denominator is positive by construction")
    }

    /// The numerator of the canonical fraction.
    pub const fn numerator(&self) -> i128 {
        self.numerator
    }

    /// The positive denominator of the canonical fraction.
    pub const fn denominator(&self) -> i128 {
        self.denominator
    }
}

/// `B(x)`: the bit length of `|x|`, with `B(0) = 1`.
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == verus_proofs::bits(value), 1 <= r <= 128))]
fn bits(value: i128) -> u64 {
    #[cfg(verus_keep_ghost)]
    vstd::prelude::proof! { verus_proofs::lemma_leading_zeros(value.unsigned_abs()); }
    u64::from((128 - value.unsigned_abs().leading_zeros()).max(1))
}

#[derive(Clone, Copy)]
enum Operation {
    Add,
    Subtract,
    Multiply,
}

/// The unreduced result fraction. The checks run in the order of the
/// capacity quantities; no charge precedes them.
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == verus_proofs::form(operation, left.numerator, left.denominator, right.numerator, right.denominator)))]
fn form(
    operation: Operation,
    left: &NativeRational,
    right: &NativeRational,
) -> Result<(i128, i128), NativeRationalRefusal> {
    use NativeRationalCapacity as Capacity;
    use NativeRationalRefusal::Capacity as Refuse;
    let (a, p, c, q) = (
        left.numerator,
        left.denominator,
        right.numerator,
        right.denominator,
    );
    let numerator = match operation {
        Operation::Multiply => a.checked_mul(c).ok_or(Refuse(Capacity::LeftProduct))?,
        Operation::Add | Operation::Subtract => {
            let first = a.checked_mul(q).ok_or(Refuse(Capacity::LeftProduct))?;
            let second = c.checked_mul(p).ok_or(Refuse(Capacity::RightProduct))?;
            let combined = match operation {
                Operation::Subtract => first.checked_sub(second),
                _ => first.checked_add(second),
            };
            combined.ok_or(Refuse(Capacity::Combination))?
        }
    };
    let denominator = p.checked_mul(q).ok_or(Refuse(Capacity::Denominator))?;
    Ok((numerator, denominator))
}

/// `integer_bits` of `rational-arithmetic.arithmetic`: `max(N, D)`.
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == verus_proofs::arithmetic_bits(operation, left.numerator, left.denominator, right.numerator, right.denominator)))]
fn arithmetic_bits(operation: Operation, left: &NativeRational, right: &NativeRational) -> u64 {
    let (a, p, c, q) = (
        bits(left.numerator),
        bits(left.denominator),
        bits(right.numerator),
        bits(right.denominator),
    );
    let numerator = match operation {
        Operation::Multiply => a + c,
        Operation::Add | Operation::Subtract => (a + q).max(c + p) + 1,
    };
    numerator.max(p + q)
}

#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == verus_proofs::operand_bits(left.numerator, left.denominator, right.numerator, right.denominator)))]
fn operand_bits(left: &NativeRational, right: &NativeRational) -> u64 {
    bits(left.numerator)
        .max(bits(left.denominator))
        .max(bits(right.numerator))
        .max(bits(right.denominator))
}

#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures verus_proofs::charge_arithmetic_post(verus_proofs::arithmetic_bits(operation, left.numerator, left.denominator, right.numerator, right.denominator), left.numerator, left.denominator, right.numerator, right.denominator, unreduced, *old(meter), *final(meter), r)))]
fn charge_arithmetic(
    operation: Operation,
    left: &NativeRational,
    right: &NativeRational,
    unreduced: (i128, i128),
    meter: &mut Meter,
) -> Result<(), Incomplete> {
    meter.charge_units(UnitsCharge::two(
        ChargePoint::RationalArithmeticOperands,
        (LimitKind::IntegerBits, operand_bits(left, right)),
        (LimitKind::ValueOccurrences, 2),
    ))?;
    meter.charge_units(UnitsCharge::one(
        ChargePoint::RationalArithmeticArithmetic,
        LimitKind::IntegerBits,
        arithmetic_bits(operation, left, right),
    ))?;
    meter.charge_units(UnitsCharge::one(
        ChargePoint::RationalArithmeticNormalize,
        LimitKind::IntegerBits,
        bits(unreduced.0).max(bits(unreduced.1)),
    ))?;
    meter.charge_units(UnitsCharge::new(ChargePoint::RationalArithmeticResultRetain).results(1))
}

#[cfg_attr(verus_keep_ghost, verus_spec(r => requires left.denominator > 0, right.denominator > 0, ensures verus_proofs::arithmetic_post(verus_proofs::form(operation, left.numerator, left.denominator, right.numerator, right.denominator), verus_proofs::arithmetic_bits(operation, left.numerator, left.denominator, right.numerator, right.denominator), left.numerator, left.denominator, right.numerator, right.denominator, *old(meter), *final(meter), r)))]
fn arithmetic(
    operation: Operation,
    left: &NativeRational,
    right: &NativeRational,
    meter: &mut Meter,
) -> Result<Outcome<NativeRational>, NativeRationalRefusal> {
    let unreduced = form(operation, left, right)?;
    #[cfg(verus_keep_ghost)]
    vstd::prelude::proof! { verus_proofs::lemma_positive_product(left.denominator, right.denominator); }
    Ok(
        match charge_arithmetic(operation, left, right, unreduced, meter) {
            Ok(()) => {
                let (numerator, denominator) = reduce(unreduced.0, unreduced.1);
                Outcome::Completed(NativeRational {
                    numerator,
                    denominator,
                })
            }
            Err(record) => Outcome::Incomplete(record),
        },
    )
}

/// `a/p + c/q`: `rational-arithmetic.operands`, `.arithmetic`, `.normalize`,
/// then `.result-retain`.
#[cfg_attr(verus_keep_ghost, verus_spec(r => requires left.parts().1 > 0, right.parts().1 > 0, ensures verus_proofs::arithmetic_post(verus_proofs::form_add(left.parts().0, left.parts().1, right.parts().0, right.parts().1), verus_proofs::sum_bits(left.parts().0, left.parts().1, right.parts().0, right.parts().1), left.parts().0, left.parts().1, right.parts().0, right.parts().1, *old(meter), *final(meter), r)))]
pub fn native_rational_add(
    left: &NativeRational,
    right: &NativeRational,
    meter: &mut Meter,
) -> Result<Outcome<NativeRational>, NativeRationalRefusal> {
    arithmetic(Operation::Add, left, right, meter)
}

/// `a/p - c/q`, on the schedule of [`native_rational_add`].
#[cfg_attr(verus_keep_ghost, verus_spec(r => requires left.parts().1 > 0, right.parts().1 > 0, ensures verus_proofs::arithmetic_post(verus_proofs::form_subtract(left.parts().0, left.parts().1, right.parts().0, right.parts().1), verus_proofs::sum_bits(left.parts().0, left.parts().1, right.parts().0, right.parts().1), left.parts().0, left.parts().1, right.parts().0, right.parts().1, *old(meter), *final(meter), r)))]
pub fn native_rational_subtract(
    left: &NativeRational,
    right: &NativeRational,
    meter: &mut Meter,
) -> Result<Outcome<NativeRational>, NativeRationalRefusal> {
    arithmetic(Operation::Subtract, left, right, meter)
}

/// `a/p * c/q`, on the schedule of [`native_rational_add`].
#[cfg_attr(verus_keep_ghost, verus_spec(r => requires left.parts().1 > 0, right.parts().1 > 0, ensures verus_proofs::arithmetic_post(verus_proofs::form_multiply(left.parts().0, left.parts().1, right.parts().0, right.parts().1), verus_proofs::product_bits(left.parts().0, left.parts().1, right.parts().0, right.parts().1), left.parts().0, left.parts().1, right.parts().0, right.parts().1, *old(meter), *final(meter), r)))]
pub fn native_rational_multiply(
    left: &NativeRational,
    right: &NativeRational,
    meter: &mut Meter,
) -> Result<Outcome<NativeRational>, NativeRationalRefusal> {
    arithmetic(Operation::Multiply, left, right, meter)
}

#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures verus_proofs::charge_ordering_post(left.numerator, left.denominator, right.numerator, right.denominator, *old(meter), *final(meter), r)))]
fn charge_ordering(
    left: &NativeRational,
    right: &NativeRational,
    meter: &mut Meter,
) -> Result<(), Incomplete> {
    meter.charge_units(UnitsCharge::two(
        ChargePoint::OrderingOperands,
        (LimitKind::IntegerBits, operand_bits(left, right)),
        (LimitKind::ValueOccurrences, 2),
    ))?;
    let cross = (bits(left.numerator) + bits(right.denominator))
        .max(bits(right.numerator) + bits(left.denominator));
    meter.charge_units(UnitsCharge::one(
        ChargePoint::OrderingArithmetic,
        LimitKind::IntegerBits,
        cross,
    ))?;
    meter.charge_units(UnitsCharge::new(ChargePoint::OrderingResultRetain).results(1))
}

/// Order two carriers: `ordering.operands`, `ordering.arithmetic`, then
/// `ordering.result-retain`. The Boolean compares `a q` with `c p`.
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures verus_proofs::order_post(operator, left.parts().0, left.parts().1, right.parts().0, right.parts().1, *old(meter), *final(meter), r)))]
pub fn native_rational_order(
    operator: OrderingOperator,
    left: &NativeRational,
    right: &NativeRational,
    meter: &mut Meter,
) -> Result<Outcome<bool>, NativeRationalRefusal> {
    use NativeRationalCapacity as Capacity;
    let first = left
        .numerator
        .checked_mul(right.denominator)
        .ok_or(NativeRationalRefusal::Capacity(Capacity::LeftProduct))?;
    let second = right
        .numerator
        .checked_mul(left.denominator)
        .ok_or(NativeRationalRefusal::Capacity(Capacity::RightProduct))?;
    Ok(match charge_ordering(left, right, meter) {
        Ok(()) => Outcome::Completed(operator.holds(first.cmp(&second))),
        Err(record) => Outcome::Incomplete(record),
    })
}

/// Equality of two carriers on the owner's equality schedule for two scalar
/// leaves. It forms no product, so it never refuses for capacity.
pub fn native_rational_equal(
    left: &NativeRational,
    right: &NativeRational,
    meter: &mut Meter,
) -> Outcome<bool> {
    Outcome::from_stop(
        charge_native_integer_schedule(NativeIntegerSchedule::Equality, meter)
            .into_stop()
            .map(|()| left == right),
    )
}

#[cfg(kani)]
mod kani_proofs;
#[cfg(kani)]
mod reference;
#[cfg(verus_keep_ghost)]
mod verus_proofs;
