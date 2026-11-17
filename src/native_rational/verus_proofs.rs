// SPDX-License-Identifier: AGPL-3.0-or-later
//! Verus proofs for the native Rational carrier over the full `i64` domain
//! (FR-408 Behavior 18 row a, IR-1544). Compiled only by a Verus run
//! (`cfg(verus_keep_ghost)`).

use vstd::arithmetic::power2::*;
use vstd::prelude::*;

use super::{
    native_rational_add, native_rational_multiply, native_rational_order, native_rational_subtract,
    NativeRational, NativeRationalCapacity, NativeRationalRefusal, Operation,
};
use crate::accounting::{units, with_results, ChargePoint, LimitKind, Meter, UnitsCharge};
use crate::native_fraction::verus_proofs as fraction;
use crate::numeric::OrderingOperator;
use crate::outcome::Outcome;
use crate::verus_kernel_model::{admitted, ordering_holds};

verus! {

#[verifier::external_type_specification]
pub struct ExNativeRationalCapacity(NativeRationalCapacity);

#[verifier::external_type_specification]
pub struct ExNativeRationalRefusal(NativeRationalRefusal);

/// Specification of the production `NativeRational::from_i64_parts`.
pub open spec fn from_i64_parts(
    numerator: i64,
    denominator: i64,
    r: Result<NativeRational, NativeRationalRefusal>,
) -> bool {
    if denominator == 0 {
        r == Err::<NativeRational, _>(NativeRationalRefusal::ZeroDenominator)
    } else if !fraction::is_canonical_pair(numerator, denominator) {
        r == Err::<NativeRational, _>(NativeRationalRefusal::NonCanonical)
    } else {
        r is Ok && r->Ok_0.parts() == (numerator as i128, denominator as i128)
    }
}

/// Row (a): for every `i64` pair, `from_i64_parts` refuses a zero denominator
/// first, then a pair that is not canonical, and otherwise builds the carrier
/// of exactly that pair.
pub fn row_from_i64_parts(numerator: i64, denominator: i64) -> (r: Result<
    NativeRational,
    NativeRationalRefusal,
>)
    ensures
        denominator == 0 ==> r == Err::<NativeRational, _>(NativeRationalRefusal::ZeroDenominator),
        denominator != 0 && !fraction::canonical(numerator as int, denominator as int) ==> r == Err::<
            NativeRational,
            _,
        >(NativeRationalRefusal::NonCanonical),
        fraction::canonical(numerator as int, denominator as int) ==> r is Ok && r->Ok_0.parts() == (
            numerator as i128,
            denominator as i128,
        ),
{
    proof {
        fraction::lemma_is_canonical_pair(numerator, denominator);
    }
    NativeRational::from_i64_parts(numerator, denominator)
}

// ---- bit lengths --------------------------------------------------------------

#[verifier::external_type_specification]
pub(super) struct ExOperation(Operation);

/// `B(m)`: the bit length of a magnitude, with `B(0) = 1` (FR-408 Behavior 11).
pub open spec fn bit_len(m: nat) -> nat
    decreases m,
{
    if m <= 1 {
        1
    } else {
        1 + bit_len(m / 2)
    }
}

/// Specification of the production `bits`: `B(|value|)`.
pub open spec fn bits(value: i128) -> u64 {
    bit_len(value.unsigned_abs() as nat) as u64
}

pub proof fn lemma_bit_len_bound(m: nat, j: nat)
    requires
        m < pow2(j),
        j >= 1,
    ensures
        1 <= bit_len(m) <= j,
    decreases m,
{
    if m >= 2 {
        lemma2_to64();
        lemma_pow2_unfold(j);
        if j == 1 {
            assert(false);
        }
        lemma_bit_len_bound(m / 2, (j - 1) as nat);
    }
}

pub proof fn lemma_leading_zeros(m: u128)
    ensures
        m != 0 ==> 128 - m.leading_zeros() == bit_len(m as nat),
        m == 0 ==> m.leading_zeros() == 128,
        1 <= bit_len(m as nat) <= 128,
        m.leading_zeros() <= 128,
    decreases m,
{
    reveal_with_fuel(crate::verus_std_specs::u128_leading_zeros, 3);
    lemma2_to64();
    lemma2_to64_rest();
    lemma_pow2_adds(64, 63);
    lemma_pow2_adds(64, 64);
    lemma_bit_len_bound(m as nat, 128);
    if m >= 2 {
        lemma_bit_len_bound((m / 2) as nat, 127);
        lemma_leading_zeros(m / 2);
    }
}

/// The capacity checks of an add or subtract in order, then the exact
/// integers (FR-408 Behavior 9). `combined` is `a q + c p` or `a q - c p`.
pub open spec fn form_sum(a: i128, p: i128, c: i128, q: i128, combined: int) -> Result<
    (i128, i128),
    NativeRationalRefusal,
> {
    if !in_i128(a * q) {
        Err(NativeRationalRefusal::Capacity(NativeRationalCapacity::LeftProduct))
    } else if !in_i128(c * p) {
        Err(NativeRationalRefusal::Capacity(NativeRationalCapacity::RightProduct))
    } else if !in_i128(combined) {
        Err(NativeRationalRefusal::Capacity(NativeRationalCapacity::Combination))
    } else if !in_i128(p * q) {
        Err(NativeRationalRefusal::Capacity(NativeRationalCapacity::Denominator))
    } else {
        Ok((combined as i128, (p * q) as i128))
    }
}

/// `a/p + c/q` before reduction.
pub open spec fn form_add(a: i128, p: i128, c: i128, q: i128) -> Result<
    (i128, i128),
    NativeRationalRefusal,
> {
    form_sum(a, p, c, q, a * q + c * p)
}

/// `a/p - c/q` before reduction.
pub open spec fn form_subtract(a: i128, p: i128, c: i128, q: i128) -> Result<
    (i128, i128),
    NativeRationalRefusal,
> {
    form_sum(a, p, c, q, a * q - c * p)
}

/// `a/p * c/q` before reduction.
pub open spec fn form_multiply(a: i128, p: i128, c: i128, q: i128) -> Result<
    (i128, i128),
    NativeRationalRefusal,
> {
    if !in_i128(a * c) {
        Err(NativeRationalRefusal::Capacity(NativeRationalCapacity::LeftProduct))
    } else if !in_i128(p * q) {
        Err(NativeRationalRefusal::Capacity(NativeRationalCapacity::Denominator))
    } else {
        Ok(((a * c) as i128, (p * q) as i128))
    }
}

/// Specification of the production `form`.
pub(super) open spec fn form(operation: Operation, a: i128, p: i128, c: i128, q: i128) -> Result<
    (i128, i128),
    NativeRationalRefusal,
> {
    match operation {
        Operation::Add => form_add(a, p, c, q),
        Operation::Subtract => form_subtract(a, p, c, q),
        Operation::Multiply => form_multiply(a, p, c, q),
    }
}

/// `x` fits signed 128 bits.
pub open spec fn in_i128(x: int) -> bool {
    i128::MIN <= x <= i128::MAX
}

/// `x` fits signed 64 bits.
pub open spec fn in_i64(x: int) -> bool {
    i64::MIN <= x <= i64::MAX
}

/// Inside signed 64 bits, with positive denominators, no operation is a
/// capacity refusal and the formed integers are the exact ones.
pub(super) proof fn lemma_form_i64(operation: Operation, a: i128, p: i128, c: i128, q: i128)
    requires
        in_i64(a as int),
        in_i64(p as int),
        in_i64(c as int),
        in_i64(q as int),
        p > 0,
        q > 0,
    ensures
        in_i128(unreduced(operation, a as int, p as int, c as int, q as int).0),
        in_i128(unreduced(operation, a as int, p as int, c as int, q as int).1),
        unreduced(operation, a as int, p as int, c as int, q as int).1 > 0,
        form(operation, a, p, c, q) == Ok::<(i128, i128), NativeRationalRefusal>(
            (
                unreduced(operation, a as int, p as int, c as int, q as int).0 as i128,
                unreduced(operation, a as int, p as int, c as int, q as int).1 as i128,
            ),
        ),
{
    lemma_product_i64(a as int, c as int);
    lemma_product_i64(a as int, q as int);
    lemma_product_i64(c as int, p as int);
    lemma_product_i64(p as int, q as int);
    assert(p * q > 0) by (nonlinear_arith)
        requires
            p > 0,
            q > 0,
    ;
}

/// The exact unreduced numerator and denominator of `a/p (op) c/q`.
pub(super) open spec fn unreduced(operation: Operation, a: int, p: int, c: int, q: int) -> (int, int) {
    (
        match operation {
            Operation::Multiply => a * c,
            Operation::Add => a * q + c * p,
            Operation::Subtract => a * q - c * p,
        },
        p * q,
    )
}

/// A product of two signed 64-bit values is at most `2^126` in magnitude, and
/// at most `2^126 - 2^63` when one factor is positive.
pub proof fn lemma_product_i64(x: int, y: int)
    requires
        in_i64(x),
        in_i64(y),
    ensures
        -0x4000_0000_0000_0000_0000_0000_0000_0000 <= x * y <= 0x4000_0000_0000_0000_0000_0000_0000_0000,
        y > 0 ==> -0x3fff_ffff_ffff_ffff_8000_0000_0000_0000 <= x * y
            <= 0x3fff_ffff_ffff_ffff_8000_0000_0000_0000,
{
    assert(-0x4000_0000_0000_0000_0000_0000_0000_0000 <= x * y
        <= 0x4000_0000_0000_0000_0000_0000_0000_0000) by (nonlinear_arith)
        requires
            -0x8000_0000_0000_0000 <= x <= 0x7fff_ffff_ffff_ffff,
            -0x8000_0000_0000_0000 <= y <= 0x7fff_ffff_ffff_ffff,
    ;
    if y > 0 {
        assert(-0x3fff_ffff_ffff_ffff_8000_0000_0000_0000 <= x * y
            <= 0x3fff_ffff_ffff_ffff_8000_0000_0000_0000) by (nonlinear_arith)
            requires
                -0x8000_0000_0000_0000 <= x <= 0x7fff_ffff_ffff_ffff,
                0 < y <= 0x7fff_ffff_ffff_ffff,
        ;
    }
}

// ---- charges ------------------------------------------------------------------

/// The larger of two amounts.
pub open spec fn max_u64(a: u64, b: u64) -> u64 {
    if a >= b {
        a
    } else {
        b
    }
}

/// `max(B(a), B(p), B(c), B(q))` (Behavior 11 and 12, the operands amount).
pub open spec fn operand_bits(a: i128, p: i128, c: i128, q: i128) -> u64 {
    max_u64(max_u64(max_u64(bits(a), bits(p)), bits(c)), bits(q))
}

/// `max(max(B(a) + B(q), B(c) + B(p)) + 1, B(p) + B(q))` (Behavior 11, add
/// and subtract).
pub open spec fn sum_bits(a: i128, p: i128, c: i128, q: i128) -> u64 {
    max_u64(
        (max_u64((bits(a) + bits(q)) as u64, (bits(c) + bits(p)) as u64) + 1) as u64,
        (bits(p) + bits(q)) as u64,
    )
}

/// `max(B(a) + B(c), B(p) + B(q))` (Behavior 11, multiply).
pub open spec fn product_bits(a: i128, p: i128, c: i128, q: i128) -> u64 {
    max_u64((bits(a) + bits(c)) as u64, (bits(p) + bits(q)) as u64)
}

/// Specification of the production `arithmetic_bits`.
pub(super) open spec fn arithmetic_bits(operation: Operation, a: i128, p: i128, c: i128, q: i128) -> u64 {
    match operation {
        Operation::Multiply => product_bits(a, p, c, q),
        _ => sum_bits(a, p, c, q),
    }
}

/// The meter after the `k` first charges of an arithmetic operation (FR-408
/// Behavior 11), `k` from 0 to 4.
pub open spec fn arithmetic_charged(
    before: Seq<UnitsCharge>,
    k: int,
    a: i128,
    p: i128,
    c: i128,
    q: i128,
    arithmetic: u64,
    unreduced: (i128, i128),
) -> Seq<UnitsCharge> {
    let first = before.push(
        units(
            ChargePoint::RationalArithmeticOperands,
            Some((LimitKind::IntegerBits, operand_bits(a, p, c, q))),
            Some((LimitKind::ValueOccurrences, 2u64)),
            1,
            0,
        ),
    );
    let second = first.push(
        units(
            ChargePoint::RationalArithmeticArithmetic,
            Some((LimitKind::IntegerBits, arithmetic)),
            None,
            1,
            0,
        ),
    );
    let third = second.push(
        units(
            ChargePoint::RationalArithmeticNormalize,
            Some((LimitKind::IntegerBits, max_u64(bits(unreduced.0), bits(unreduced.1)))),
            None,
            1,
            0,
        ),
    );
    let fourth = third.push(
        with_results(units(ChargePoint::RationalArithmeticResultRetain, None, None, 1, 0), 1),
    );
    if k <= 0 {
        before
    } else if k == 1 {
        first
    } else if k == 2 {
        second
    } else if k == 3 {
        third
    } else {
        fourth
    }
}

/// `after` is `before` plus a proper prefix (zero to three) of the four
/// charges.
pub open spec fn prefix_charged(
    before: Seq<UnitsCharge>,
    after: Seq<UnitsCharge>,
    a: i128,
    p: i128,
    c: i128,
    q: i128,
    arithmetic: u64,
    unreduced: (i128, i128),
) -> bool {
    after == arithmetic_charged(before, 0, a, p, c, q, arithmetic, unreduced) || after
        == arithmetic_charged(before, 1, a, p, c, q, arithmetic, unreduced) || after
        == arithmetic_charged(before, 2, a, p, c, q, arithmetic, unreduced) || after
        == arithmetic_charged(before, 3, a, p, c, q, arithmetic, unreduced)
}

/// What an arithmetic operation returns and charges, given the outcome
/// `formed` of its capacity checks and its `arithmetic` amount: a capacity
/// refusal before any charge with the meter unchanged; or the reduced value
/// after the four charges in order; or an incomplete outcome after a proper
/// prefix of them. It is never undefined or refused.
pub open spec fn arithmetic_post(
    formed: Result<(i128, i128), NativeRationalRefusal>,
    arithmetic: u64,
    a: i128,
    p: i128,
    c: i128,
    q: i128,
    before: Meter,
    after: Meter,
    r: Result<Outcome<NativeRational>, NativeRationalRefusal>,
) -> bool {
    match r {
        Err(refusal) => formed == Err::<(i128, i128), _>(refusal) && after == before,
        Ok(Outcome::Completed(value)) => formed is Ok && value.parts() == fraction::reduce(
            formed->Ok_0.0,
            formed->Ok_0.1,
        ) && admitted(after) == arithmetic_charged(
            admitted(before),
            4,
            a,
            p,
            c,
            q,
            arithmetic,
            formed->Ok_0,
        ),
        Ok(Outcome::Incomplete(_)) => formed is Ok && prefix_charged(
            admitted(before),
            admitted(after),
            a,
            p,
            c,
            q,
            arithmetic,
            formed->Ok_0,
        ),
        Ok(_) => false,
    }
}

/// What `charge_arithmetic` charges: all four in order, or a proper prefix.
pub open spec fn charge_arithmetic_post(
    arithmetic: u64,
    a: i128,
    p: i128,
    c: i128,
    q: i128,
    unreduced: (i128, i128),
    before: Meter,
    after: Meter,
    r: Result<(), crate::accounting::Incomplete>,
) -> bool {
    match r {
        Ok(_) => admitted(after) == arithmetic_charged(
            admitted(before),
            4,
            a,
            p,
            c,
            q,
            arithmetic,
            unreduced,
        ),
        Err(_) => prefix_charged(
            admitted(before),
            admitted(after),
            a,
            p,
            c,
            q,
            arithmetic,
            unreduced,
        ),
    }
}

/// A product of two positive values is positive.
pub proof fn lemma_positive_product(p: i128, q: i128)
    requires
        p > 0,
        q > 0,
    ensures
        p * q > 0,
{
    assert(p * q > 0) by (nonlinear_arith)
        requires
            p > 0,
            q > 0,
    ;
}

// ---- ordering -----------------------------------------------------------------

/// `max(B(a) + B(q), B(c) + B(p))` (Behavior 12, the arithmetic amount).
pub open spec fn cross_bits(a: i128, p: i128, c: i128, q: i128) -> u64 {
    max_u64((bits(a) + bits(q)) as u64, (bits(c) + bits(p)) as u64)
}

/// The meter after the `k` first charges of an ordering (FR-408 Behavior 12),
/// `k` from 0 to 3.
pub open spec fn ordering_charged(before: Seq<UnitsCharge>, k: int, a: i128, p: i128, c: i128, q: i128) -> Seq<
    UnitsCharge,
> {
    let first = before.push(
        units(
            ChargePoint::OrderingOperands,
            Some((LimitKind::IntegerBits, operand_bits(a, p, c, q))),
            Some((LimitKind::ValueOccurrences, 2u64)),
            1,
            0,
        ),
    );
    let second = first.push(
        units(
            ChargePoint::OrderingArithmetic,
            Some((LimitKind::IntegerBits, cross_bits(a, p, c, q))),
            None,
            1,
            0,
        ),
    );
    let third = second.push(
        with_results(units(ChargePoint::OrderingResultRetain, None, None, 1, 0), 1),
    );
    if k <= 0 {
        before
    } else if k == 1 {
        first
    } else if k == 2 {
        second
    } else {
        third
    }
}

/// What `charge_ordering` charges: all three in order, or a proper prefix.
pub open spec fn charge_ordering_post(
    a: i128,
    p: i128,
    c: i128,
    q: i128,
    before: Meter,
    after: Meter,
    r: Result<(), crate::accounting::Incomplete>,
) -> bool {
    match r {
        Ok(_) => admitted(after) == ordering_charged(admitted(before), 3, a, p, c, q),
        Err(_) => ordering_prefix(admitted(before), admitted(after), a, p, c, q),
    }
}

/// `after` is `before` plus a proper prefix (zero to two) of the three charges.
pub open spec fn ordering_prefix(
    before: Seq<UnitsCharge>,
    after: Seq<UnitsCharge>,
    a: i128,
    p: i128,
    c: i128,
    q: i128,
) -> bool {
    after == ordering_charged(before, 0, a, p, c, q) || after == ordering_charged(
        before,
        1,
        a,
        p,
        c,
        q,
    ) || after == ordering_charged(before, 2, a, p, c, q)
}

/// The comparison of two exact integers under an operator.
pub open spec fn order_holds(operator: OrderingOperator, first: int, second: int) -> bool {
    match operator {
        OrderingOperator::Less => first < second,
        OrderingOperator::LessOrEqual => first <= second,
        OrderingOperator::Greater => first > second,
        OrderingOperator::GreaterOrEqual => first >= second,
    }
}

/// What `native_rational_order` returns and charges: a capacity refusal for
/// the first cross product that leaves signed 128 bits, before any charge
/// with the meter unchanged; or the comparison of the exact cross products
/// `a q` and `c p` after the three charges in order; or an incomplete outcome
/// after a proper prefix of them. It is never undefined or refused.
pub open spec fn order_post(
    operator: OrderingOperator,
    a: i128,
    p: i128,
    c: i128,
    q: i128,
    before: Meter,
    after: Meter,
    r: Result<Outcome<bool>, NativeRationalRefusal>,
) -> bool {
    match r {
        Err(refusal) => after == before && if !in_i128(a * q) {
            refusal == NativeRationalRefusal::Capacity(NativeRationalCapacity::LeftProduct)
        } else {
            !in_i128(c * p) && refusal == NativeRationalRefusal::Capacity(
                NativeRationalCapacity::RightProduct,
            )
        },
        Ok(Outcome::Completed(holds)) => in_i128(a * q) && in_i128(c * p) && holds == order_holds(
            operator,
            a * q,
            c * p,
        ) && admitted(after) == ordering_charged(admitted(before), 3, a, p, c, q),
        Ok(Outcome::Incomplete(_)) => in_i128(a * q) && in_i128(c * p) && ordering_prefix(
            admitted(before),
            admitted(after),
            a,
            p,
            c,
            q,
        ),
        Ok(_) => false,
    }
}

// ---- the full-i64 rows ----------------------------------------------------------

/// A carrier as `from_i64_parts` builds it: a canonical pair inside signed 64
/// bits.
pub open spec fn i64_carrier(value: NativeRational) -> bool {
    in_i64(value.parts().0 as int) && in_i64(value.parts().1 as int) && fraction::canonical(
        value.parts().0 as int,
        value.parts().1 as int,
    )
}

/// The result-retain request of the arithmetic and ordering schedules is the
/// bare request at its point with one result unit.
pub proof fn lemma_result_retain()
    ensures
        with_results(units(ChargePoint::RationalArithmeticResultRetain, None, None, 1, 0), 1)
            == units(ChargePoint::RationalArithmeticResultRetain, None, None, 1, 1),
        with_results(units(ChargePoint::OrderingResultRetain, None, None, 1, 0), 1) == units(
            ChargePoint::OrderingResultRetain,
            None,
            None,
            1,
            1,
        ),
{
    crate::accounting::lemma_with_results(
        ChargePoint::RationalArithmeticResultRetain,
        None,
        None,
        1,
        0,
        1,
    );
    crate::accounting::lemma_with_results(ChargePoint::OrderingResultRetain, None, None, 1, 0, 1);
}

/// Row (b), `add` over the full `i64` domain: it never refuses, and when it
/// completes the value is the exact result in lowest terms (both exact parts
/// divided by their gcd, canonical) after the four charges of Behavior 11.
pub fn row_add(left: &NativeRational, right: &NativeRational, meter: &mut Meter) -> (r: Result<
    Outcome<NativeRational>,
    NativeRationalRefusal,
>)
    requires
        i64_carrier(*left),
        i64_carrier(*right),
    ensures
        r is Ok,
        r->Ok_0 is Completed || r->Ok_0 is Incomplete,
        r matches Ok(Outcome::Completed(value)) ==> ({
            let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
            let (n, d) = (a * q + c * p, p * q);
            let g = fraction::gcd(vstd::math::abs(n) as nat, d as nat);
            &&& g > 0
            &&& value.parts().0 * g == n
            &&& value.parts().1 * g == d
            &&& fraction::canonical(value.parts().0 as int, value.parts().1 as int)
            &&& admitted(*final(meter)) == arithmetic_charged(
                admitted(*old(meter)),
                4,
                a,
                p,
                c,
                q,
                sum_bits(a, p, c, q),
                (n as i128, d as i128),
            )
        }),
        r matches Ok(Outcome::Incomplete(_)) ==> ({
            let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
            prefix_charged(
                admitted(*old(meter)),
                admitted(*final(meter)),
                a,
                p,
                c,
                q,
                sum_bits(a, p, c, q),
                ((a * q + c * p) as i128, (p * q) as i128),
            )
        }),
{
    proof {
        let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
        lemma_form_i64(Operation::Add, a, p, c, q);
        fraction::lemma_reduce((a * q + c * p) as i128, (p * q) as i128);
    }
    native_rational_add(left, right, meter)
}

/// Row (b), `subtract` over the full `i64` domain: it never refuses, and when it
/// completes the value is the exact result in lowest terms (both exact parts
/// divided by their gcd, canonical) after the four charges of Behavior 11.
pub fn row_subtract(left: &NativeRational, right: &NativeRational, meter: &mut Meter) -> (r: Result<
    Outcome<NativeRational>,
    NativeRationalRefusal,
>)
    requires
        i64_carrier(*left),
        i64_carrier(*right),
    ensures
        r is Ok,
        r->Ok_0 is Completed || r->Ok_0 is Incomplete,
        r matches Ok(Outcome::Completed(value)) ==> ({
            let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
            let (n, d) = (a * q - c * p, p * q);
            let g = fraction::gcd(vstd::math::abs(n) as nat, d as nat);
            &&& g > 0
            &&& value.parts().0 * g == n
            &&& value.parts().1 * g == d
            &&& fraction::canonical(value.parts().0 as int, value.parts().1 as int)
            &&& admitted(*final(meter)) == arithmetic_charged(
                admitted(*old(meter)),
                4,
                a,
                p,
                c,
                q,
                sum_bits(a, p, c, q),
                (n as i128, d as i128),
            )
        }),
        r matches Ok(Outcome::Incomplete(_)) ==> ({
            let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
            prefix_charged(
                admitted(*old(meter)),
                admitted(*final(meter)),
                a,
                p,
                c,
                q,
                sum_bits(a, p, c, q),
                ((a * q - c * p) as i128, (p * q) as i128),
            )
        }),
{
    proof {
        let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
        lemma_form_i64(Operation::Subtract, a, p, c, q);
        fraction::lemma_reduce((a * q - c * p) as i128, (p * q) as i128);
    }
    native_rational_subtract(left, right, meter)
}

/// Row (b), `multiply` over the full `i64` domain: it never refuses, and when it
/// completes the value is the exact result in lowest terms (both exact parts
/// divided by their gcd, canonical) after the four charges of Behavior 11.
pub fn row_multiply(left: &NativeRational, right: &NativeRational, meter: &mut Meter) -> (r: Result<
    Outcome<NativeRational>,
    NativeRationalRefusal,
>)
    requires
        i64_carrier(*left),
        i64_carrier(*right),
    ensures
        r is Ok,
        r->Ok_0 is Completed || r->Ok_0 is Incomplete,
        r matches Ok(Outcome::Completed(value)) ==> ({
            let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
            let (n, d) = (a * c, p * q);
            let g = fraction::gcd(vstd::math::abs(n) as nat, d as nat);
            &&& g > 0
            &&& value.parts().0 * g == n
            &&& value.parts().1 * g == d
            &&& fraction::canonical(value.parts().0 as int, value.parts().1 as int)
            &&& admitted(*final(meter)) == arithmetic_charged(
                admitted(*old(meter)),
                4,
                a,
                p,
                c,
                q,
                product_bits(a, p, c, q),
                (n as i128, d as i128),
            )
        }),
        r matches Ok(Outcome::Incomplete(_)) ==> ({
            let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
            prefix_charged(
                admitted(*old(meter)),
                admitted(*final(meter)),
                a,
                p,
                c,
                q,
                product_bits(a, p, c, q),
                ((a * c) as i128, (p * q) as i128),
            )
        }),
{
    proof {
        let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
        lemma_form_i64(Operation::Multiply, a, p, c, q);
        fraction::lemma_reduce((a * c) as i128, (p * q) as i128);
    }
    native_rational_multiply(left, right, meter)
}

/// Row (b), ordering over the full `i64` domain: it never refuses, and when
/// it completes the Boolean compares the exact cross products `a q` and `c p`
/// after the three charges of Behavior 12.
pub fn row_order(
    operator: OrderingOperator,
    left: &NativeRational,
    right: &NativeRational,
    meter: &mut Meter,
) -> (r: Result<Outcome<bool>, NativeRationalRefusal>)
    requires
        i64_carrier(*left),
        i64_carrier(*right),
    ensures
        r is Ok,
        r->Ok_0 is Completed || r->Ok_0 is Incomplete,
        r matches Ok(Outcome::Completed(holds)) ==> ({
            let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
            &&& holds == order_holds(operator, a * q, c * p)
            &&& admitted(*final(meter)) == ordering_charged(admitted(*old(meter)), 3, a, p, c, q)
        }),
        r matches Ok(Outcome::Incomplete(_)) ==> ({
            let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
            ordering_prefix(admitted(*old(meter)), admitted(*final(meter)), a, p, c, q)
        }),
{
    proof {
        let (a, p, c, q) = (left.parts().0, left.parts().1, right.parts().0, right.parts().1);
        lemma_product_i64(a as int, q as int);
        lemma_product_i64(c as int, p as int);
    }
    native_rational_order(operator, left, right, meter)
}

/// Row (b), equality over the full `i64` domain, the mathematical half: two
/// such carriers are the same pair exactly when `a q == c p`. That the
/// production `native_rational_equal` returns "the same pair" with its
/// schedule is the Kani harness
/// `native_rational_equal_all_parts_is_structural`.
pub proof fn row_equal_structural_is_cross_product(left: NativeRational, right: NativeRational)
    requires
        i64_carrier(left),
        i64_carrier(right),
    ensures
        left.parts() == right.parts() <==> left.parts().0 * right.parts().1 == right.parts().0
            * left.parts().1,
{
    fraction::lemma_canonical_unique(
        left.parts().0 as int,
        left.parts().1 as int,
        right.parts().0 as int,
        right.parts().1 as int,
    );
}

} // verus!
