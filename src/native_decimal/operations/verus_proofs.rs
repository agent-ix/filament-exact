// SPDX-License-Identifier: AGPL-3.0-or-later
//! Verus proofs of the native Decimal operations and ordering over the full
//! width (FR-403 Behavior 41, IR-1555). Compiled only by a Verus run
//! (`cfg(verus_keep_ghost)`).

use core::cmp::Ordering;

use vstd::arithmetic::div_mod::*;
use vstd::arithmetic::mul::*;
use vstd::arithmetic::power2::*;
use vstd::prelude::*;

use super::super::verus_proofs::{
    digit_len, digits_of, lemma_digit_len_range, lemma_digits_of, lemma_pow10_adds, lemma_pow10_bound, lemma_pow10_monotone, lemma_pow10_pos,
    pow10,
};
use super::super::{NativeDecimal, NativeDecimalType};
use super::{
    AdmittedNativeDecimalOperation, Admission, Intermediate, NativeDecimalCapacityQuantity,
    NativeDecimalCapacityRefusal, NativeDecimalLoss, NativeDecimalOperation, Placement, Planned, Retain, Settled,
    Tail, LOG2_10_LOWER,
};
use crate::decimal::RoundingMode;
use crate::native_float::verus_proofs::lemma_pow2_le;
use crate::native_integer::verus_proofs::lemma_truncating;
use crate::native_fraction::verus_proofs::{
    canonical, common, divides, gcd, gcd256, lemma_divides_witness, lemma_gcd256, lemma_gcd_common, lemma_gcd_greatest,
    lemma_gcd_one_iff_no_common_factor, lemma_gcd_positive, lemma_gcd_scale, lemma_gcd_sym, lemma_gcd_zero_left,
};

verus! {

// ---- capacity -----------------------------------------------------------------

/// The capacity refusal derives its `Display`, which Verus does not read; the
/// type is declared here as the struct its source shows.
#[verifier::external_type_specification]
pub struct ExNativeDecimalCapacityRefusal(NativeDecimalCapacityRefusal);

/// Whether an integer fits signed 128 bits.
pub(super) open spec fn fits(v: int) -> bool {
    i128::MIN <= v <= i128::MAX
}

/// Specification of the production `capacity`.
pub(super) open spec fn capacity<T>(found: Option<T>, quantity: NativeDecimalCapacityQuantity) -> Admission<T> {
    match found {
        Some(v) => Ok(v),
        None => Err(NativeDecimalCapacityRefusal { quantity }),
    }
}

/// Ground (a) for the lift: the shift of a coefficient placed at
/// `min(working, target)` up to the target scale is at most 38.
pub(super) open spec fn lift_fits(target: u32, working: u64) -> bool {
    target - (if working < target { working } else { target as u64 }) <= 38
}

/// `c x 10^shift` when it fits signed 128 bits.
pub(super) open spec fn shifted(c: i128, shift: u32) -> Option<i128> {
    if fits(c * pow10(shift as nat)) {
        Some((c * pow10(shift as nat)) as i128)
    } else {
        None
    }
}

/// The table entry of a shift up to 38 is its power of ten, so the checked
/// product is the mathematical one.
pub(super) proof fn lemma_shifted(c: i128, shift: u32)
    requires
        shift <= 38,
    ensures
        1 <= pow10(shift as nat) <= i128::MAX,
{
    lemma_pow10_pos(shift as nat);
    lemma_pow10_monotone(shift as nat, 38);
    lemma_pow10_bound(38);
}

/// `bits(10^k)` stays far inside `u64`: the product fits 128 bits.
pub(super) proof fn lemma_power_of_ten_bits(shift: u32)
    ensures
        (shift as u128) * LOG2_10_LOWER <= u128::MAX,
        (((shift as u128) * LOG2_10_LOWER) as u128 >> 90u32) < 0x4_0000_0000,
{
    assert((shift as u128) * LOG2_10_LOWER <= 0xFFFF_FFFF * 4_112_347_796_288_069_798_443_794_287) by (nonlinear_arith)
        requires
            shift <= 0xFFFF_FFFF,
            LOG2_10_LOWER == 4_112_347_796_288_069_798_443_794_287,
    ;
    let p = ((shift as u128) * LOG2_10_LOWER) as u128;
    assert(p <= 0xFFFF_FFFFu128 * 4_112_347_796_288_069_798_443_794_287u128 ==> (p >> 90u32) < 0x4_0000_0000u128)
        by (bit_vector);
}

// ---- the planners ---------------------------------------------------------------

/// A nonzero coefficient stays nonzero under a power of ten.
pub(super) proof fn lemma_scaled_nonzero(c: int, k: nat)
    requires
        c != 0,
    ensures
        c * pow10(k) != 0,
{
    lemma_pow10_pos(k);
    assert(c * pow10(k) != 0) by (nonlinear_arith)
        requires
            c != 0,
            pow10(k) >= 1,
    ;
}

/// What a planner returns: the first failing quantity, or the intermediate.
pub(super) open spec fn planned_is(
    r: Admission<Planned>,
    refusal: Option<NativeDecimalCapacityQuantity>,
    intermediate: Option<Intermediate>,
) -> bool {
    match r {
        Ok(planned) => refusal is None && planned.intermediate == intermediate,
        Err(e) => refusal == Some(e.quantity),
    }
}

/// The common scale of an `Add` or `Subtract`: the larger of the two.
pub(super) open spec fn combine_scale(a: NativeDecimal, b: NativeDecimal) -> u32 {
    if a.scale >= b.scale {
        a.scale
    } else {
        b.scale
    }
}

/// The exact sum or difference in units of `10^-combine_scale`.
pub(super) open spec fn combine_value(a: NativeDecimal, b: NativeDecimal, subtract: bool) -> int {
    let s = combine_scale(a, b);
    let xa = a.coefficient * pow10((s - a.scale) as nat);
    let xb = b.coefficient * pow10((s - b.scale) as nat);
    if subtract {
        xa - xb
    } else {
        xa + xb
    }
}

/// The first failing quantity of an `Add` or `Subtract` plan (FR-403
/// Behavior 3, grounds a to c).
pub(super) open spec fn combine_refusal(
    target: u32,
    a: NativeDecimal,
    b: NativeDecimal,
    subtract: bool,
) -> Option<NativeDecimalCapacityQuantity> {
    let s = combine_scale(a, b);
    let (ka, kb) = ((s - a.scale) as nat, (s - b.scale) as nat);
    if ka > 38 || kb > 38 || !lift_fits(target, s as u64) {
        Some(NativeDecimalCapacityQuantity::Exponent)
    } else if !fits(a.coefficient * pow10(ka)) || !fits(b.coefficient * pow10(kb)) {
        Some(NativeDecimalCapacityQuantity::ShiftedOperand)
    } else if !fits(combine_value(a, b, subtract)) {
        Some(
            if subtract {
                NativeDecimalCapacityQuantity::Difference
            } else {
                NativeDecimalCapacityQuantity::Sum
            },
        )
    } else {
        None
    }
}

/// The first failing quantity of a `Multiply` plan.
pub(super) open spec fn multiply_refusal(target: u32, a: NativeDecimal, b: NativeDecimal) -> Option<
    NativeDecimalCapacityQuantity,
> {
    if !lift_fits(target, (a.scale + b.scale) as u64) {
        Some(NativeDecimalCapacityQuantity::Exponent)
    } else if !fits(a.coefficient * b.coefficient) {
        Some(NativeDecimalCapacityQuantity::Product)
    } else {
        None
    }
}

/// The first failing quantity of a `Negate` or `Round` plan.
pub(super) open spec fn unary_refusal(target: u32, a: NativeDecimal, negate: bool) -> Option<
    NativeDecimalCapacityQuantity,
> {
    if !lift_fits(target, a.scale as u64) {
        Some(NativeDecimalCapacityQuantity::Exponent)
    } else if negate && !fits(-a.coefficient) {
        Some(NativeDecimalCapacityQuantity::Negation)
    } else {
        None
    }
}

/// The shifts `(kn, kd)` of a `Divide`: the numerator is lifted when the
/// target scale plus the divisor scale exceeds the dividend scale, the
/// denominator otherwise.
pub(super) open spec fn divide_shifts(target: u32, a: NativeDecimal, b: NativeDecimal) -> (nat, nat) {
    let up = target + b.scale;
    (
        if up >= a.scale {
            (up - a.scale) as nat
        } else {
            0
        },
        if a.scale >= up {
            (a.scale - up) as nat
        } else {
            0
        },
    )
}

/// `(N, D)` of a `Divide`: `N / D` is the quotient in units of `10^-target`.
pub(super) open spec fn divide_parts(target: u32, a: NativeDecimal, b: NativeDecimal) -> (int, int) {
    let (kn, kd) = divide_shifts(target, a, b);
    (a.coefficient * pow10(kn), b.coefficient * pow10(kd))
}

/// `n / d` in lowest terms with the sign on the numerator, for a nonzero `d`.
pub(super) open spec fn quotient_parts(n: int, d: int) -> (int, int) {
    let g = gcd(vstd::math::abs(n), vstd::math::abs(d));
    let m = (vstd::math::abs(n) / g) as int;
    (
        if (n < 0) != (d < 0) {
            -m
        } else {
            m
        },
        (vstd::math::abs(d) / g) as int,
    )
}

/// The first failing quantity of a `Divide` plan with a nonzero divisor
/// (grounds a, c and d).
pub(super) open spec fn divide_refusal(target: u32, a: NativeDecimal, b: NativeDecimal) -> Option<
    NativeDecimalCapacityQuantity,
> {
    let (kn, kd) = divide_shifts(target, a, b);
    let (n, d) = divide_parts(target, a, b);
    if kn > 38 || kd > 38 {
        Some(NativeDecimalCapacityQuantity::Exponent)
    } else if !fits(n) {
        Some(NativeDecimalCapacityQuantity::Numerator)
    } else if !fits(d) {
        Some(NativeDecimalCapacityQuantity::Denominator)
    } else {
        let q = quotient_parts(n, d);
        if !fits(q.0) || !fits(q.1) {
            Some(NativeDecimalCapacityQuantity::CanonicalQuotient)
        } else {
            None
        }
    }
}

/// What `plan_divide` returns: no intermediate for a zero divisor with no
/// ground applied, and otherwise the reduced quotient or the first failing
/// quantity.
pub(super) open spec fn divide_planned(target: u32, a: NativeDecimal, b: NativeDecimal, r: Admission<Planned>) -> bool {
    if b.coefficient == 0 {
        r is Ok && r->Ok_0.intermediate is None
    } else {
        let (n, d) = divide_parts(target, a, b);
        let q = quotient_parts(n, d);
        planned_is(
            r,
            divide_refusal(target, a, b),
            Some(Intermediate::Quotient { numerator: q.0 as i128, denominator: q.1 as i128 }),
        )
    }
}

/// What `reduce_quotient` returns: the lowest terms when both parts fit, and
/// the canonical-quotient refusal otherwise.
pub(super) open spec fn reduced_is(n: int, d: int, r: Admission<(i128, i128)>) -> bool {
    let q = quotient_parts(n, d);
    match r {
        Ok(pair) => fits(q.0) && fits(q.1) && pair.0 == q.0 && pair.1 == q.1,
        Err(e) => e.quantity == NativeDecimalCapacityQuantity::CanonicalQuotient && !(fits(q.0) && fits(q.1)),
    }
}

/// A magnitude up to `2^127` negates without wrapping.
pub(super) proof fn lemma_neg_magnitude(m: u128)
    requires
        m <= 0x8000_0000_0000_0000_0000_0000_0000_0000u128,
    ensures
        (m as i128).wrapping_neg() == -(m as int),
{
    if m == 0x8000_0000_0000_0000_0000_0000_0000_0000u128 {
        assert(0x8000_0000_0000_0000_0000_0000_0000_0000u128 as i128 == i128::MIN) by (bit_vector);
    } else {
        assert(m < 0x8000_0000_0000_0000_0000_0000_0000_0000u128 ==> m as i128 == m) by (bit_vector);
    }
}

/// The facts the body of `reduce_quotient` needs: the gcd is Euclid's and is
/// not zero, and the magnitude negates without wrapping up to `2^127`.
pub(super) proof fn lemma_reduce_quotient(n: i128, d: i128)
    requires
        d != 0,
    ensures
        gcd256(n.unsigned_abs(), d.unsigned_abs()) != 0,
        gcd256(n.unsigned_abs(), d.unsigned_abs()) as nat == gcd(
            vstd::math::abs(n as int) as nat,
            vstd::math::abs(d as int) as nat,
        ),
        (1u128 << 127u32) == 0x8000_0000_0000_0000_0000_0000_0000_0000u128,
        forall|m: u128| m <= 0x8000_0000_0000_0000_0000_0000_0000_0000u128 ==> #[trigger] (m as i128).wrapping_neg() == -(m as int),
{
    let (un, ud) = (n.unsigned_abs(), d.unsigned_abs());
    lemma_gcd256(un, ud);
    lemma_gcd_positive(un as nat, ud as nat);
    assert((1u128 << 127u32) == 0x8000_0000_0000_0000_0000_0000_0000_0000u128) by (bit_vector);
    assert forall|m: u128| m <= 0x8000_0000_0000_0000_0000_0000_0000_0000u128 implies #[trigger] (m as i128).wrapping_neg() == -(
    m as int) by {
        lemma_neg_magnitude(m);
    }
}

/// The lowest terms are the same value with a positive denominator, and are
/// canonical.
pub(super) proof fn lemma_quotient_parts(n: int, d: int)
    requires
        d != 0,
    ensures
        ({
            let q = quotient_parts(n, d);
            &&& q.1 > 0
            &&& q.0 * d == n * q.1
            &&& canonical(q.0, q.1)
        }),
{
    let (an, ad) = (vstd::math::abs(n) as nat, vstd::math::abs(d) as nat);
    let g = gcd(an, ad);
    lemma_gcd_positive(an, ad);
    lemma_gcd_common(an, ad);
    lemma_divides_witness(g, an);
    lemma_divides_witness(g, ad);
    let (m, e) = (an / g, ad / g);
    lemma_gcd_scale(g, m, e);
    assert(gcd(m, e) == 1) by (nonlinear_arith)
        requires
            g * gcd(m, e) == g,
            g > 0,
    ;
    assert(e > 0) by (nonlinear_arith)
        requires
            ad == g * e,
            ad > 0,
    ;
    let q = quotient_parts(n, d);
    assert(q.0 * d == n * q.1) by (nonlinear_arith)
        requires
            an == g * m,
            ad == g * e,
            an == (if n < 0 { -n } else { n }),
            ad == (if d < 0 { -d } else { d }),
            q.0 == (if (n < 0) != (d < 0) { -(m as int) } else { m as int }),
            q.1 == e,
    ;
    if q.0 == 0 {
        assert(m == 0);
        lemma_gcd_zero_left(e);
        lemma_gcd_sym(0, e);
    }
}

// ---- rounding -------------------------------------------------------------------

/// The two nearest modes.
pub(super) open spec fn nearest(mode: RoundingMode) -> bool {
    mode == RoundingMode::NearestEven || mode == RoundingMode::NearestAway
}

/// `r` is the floor of `n / d`.
pub(super) open spec fn is_floor(n: int, d: int, r: int) -> bool {
    r * d <= n < (r + 1) * d
}

/// `r` is the ceiling of `n / d`.
pub(super) open spec fn is_ceiling(n: int, d: int, r: int) -> bool {
    (r - 1) * d < n <= r * d
}

/// `r` is `n / d` rounded to an integer under `mode`, for a positive `d`:
/// the floor toward negative, the ceiling toward positive, the one of the two
/// nearer to zero toward zero, and for the nearest modes a neighbour at most
/// half a unit away, with a tie going to the even neighbour or to the one
/// farther from zero. `Exact` rounds nothing.
pub(super) open spec fn rounds(n: int, d: int, mode: RoundingMode, r: int) -> bool {
    let distance = vstd::math::abs(r * d - n);
    match mode {
        RoundingMode::Exact => false,
        RoundingMode::TowardNegative => is_floor(n, d, r),
        RoundingMode::TowardPositive => is_ceiling(n, d, r),
        RoundingMode::TowardZero => if n >= 0 {
            is_floor(n, d, r)
        } else {
            is_ceiling(n, d, r)
        },
        RoundingMode::NearestEven => (is_floor(n, d, r) || is_ceiling(n, d, r)) && 2 * distance <= d && (2 * distance
            == d ==> r % 2 == 0),
        RoundingMode::NearestAway => (is_floor(n, d, r) || is_ceiling(n, d, r)) && 2 * distance <= d && (2 * distance
            == d ==> vstd::math::abs(r * d) > vstd::math::abs(n)),
    }
}

/// The rounding relation names one integer.
pub(super) proof fn lemma_rounds_unique(n: int, d: int, mode: RoundingMode, r: int, s: int)
    requires
        d > 0,
        rounds(n, d, mode, r),
        rounds(n, d, mode, s),
    ensures
        r == s,
{
    // Two floors, or two ceilings, of one value are equal; a floor and a
    // ceiling differ by at most one.
    assert(is_floor(n, d, r) && is_floor(n, d, s) ==> r == s) by (nonlinear_arith)
        requires
            d > 0,
    ;
    assert(is_ceiling(n, d, r) && is_ceiling(n, d, s) ==> r == s) by (nonlinear_arith)
        requires
            d > 0,
    ;
    assert(is_floor(n, d, r) && is_ceiling(n, d, s) ==> s == r || (s == r + 1 && r * d < n < s * d)) by (nonlinear_arith)
        requires
            d > 0,
    ;
    assert(is_floor(n, d, s) && is_ceiling(n, d, r) ==> r == s || (r == s + 1 && s * d < n < r * d)) by (nonlinear_arith)
        requires
            d > 0,
    ;
    if nearest(mode) && r != s {
        let (lo, hi) = if r < s {
            (r, s)
        } else {
            (s, r)
        };
        assert(hi == lo + 1 && lo * d < n < hi * d);
        assert(hi * d - lo * d == d) by (nonlinear_arith)
            requires
                hi == lo + 1,
        ;
        // Both are within half a unit, and they are one unit apart: a tie.
        assert(2 * (n - lo * d) == d && 2 * (hi * d - n) == d);
        if mode == RoundingMode::NearestEven {
            assert(false);
        } else {
            // Farther from zero: only one of the two neighbours is. For a
            // positive value the lower neighbour is not negative, and for a
            // negative value the upper one is not positive.
            assert(vstd::math::abs(lo * d) > vstd::math::abs(n) && vstd::math::abs(hi * d) > vstd::math::abs(n));
            assert(n > 0 ==> lo >= 0 && lo * d >= 0) by (nonlinear_arith)
                requires
                    d > 0,
                    2 * (n - lo * d) == d,
            ;
            assert(n < 0 ==> hi <= 0 && hi * d <= 0) by (nonlinear_arith)
                requires
                    d > 0,
                    2 * (hi * d - n) == d,
            ;
            assert(n == 0 ==> lo <= -1 && hi >= 1) by (nonlinear_arith)
                requires
                    d > 0,
                    lo * d < n < hi * d,
            ;
            assert(false);
        }
    }
}

/// Specification of the production `tie`.
pub(super) open spec fn tie(mode: RoundingMode, floor: i128, ceiling: i128, negative: bool) -> i128 {
    if mode == RoundingMode::NearestEven {
        if floor % 2 == 0 {
            floor
        } else {
            ceiling
        }
    } else if negative {
        floor
    } else {
        ceiling
    }
}

/// The low bit of a signed value is its parity.
pub(super) proof fn lemma_low_bit(x: i128)
    ensures
        (x & 1 == 0) <==> (x as int) % 2 == 0,
{
    let u = x as u128;
    assert((x & 1 == 0) <==> (u & 1 == 0)) by (bit_vector)
        requires
            u == x as u128,
    ;
    assert((u & 1 == 0) <==> (u % 2 == 0)) by (bit_vector);
    if x >= 0 {
        assert(u == x) by (bit_vector)
            requires
                u == x as u128,
                x >= 0,
        ;
    } else {
        assert(u == x + 0x1_0000_0000_0000_0000_0000_0000_0000_0000) by (bit_vector)
            requires
                u == x as u128,
                x < 0,
        ;
    }
}

/// What `round_units` returns for a value that is not an integer: the
/// rounding of the mode, or the doubled-remainder refusal exactly when a
/// nearest mode would double a remainder past signed 128 bits (ground f).
pub(super) open spec fn round_units_is(n: int, d: int, mode: RoundingMode, r: Admission<i128>) -> bool {
    let overflow = nearest(mode) && 2 * (n % d) > i128::MAX;
    match r {
        Ok(v) => !overflow && (mode != RoundingMode::Exact ==> rounds(n, d, mode, v as int)),
        Err(e) => overflow && e.quantity == NativeDecimalCapacityQuantity::DoubledRemainder,
    }
}

/// The floor, the remainder and each mode's choice between the floor and the
/// ceiling, for a reduced fraction that is not an integer.
pub(super) proof fn lemma_round_units(n: i128, d: i128)
    requires
        d >= 2,
        (n as int) % (d as int) != 0,
    ensures
        ({
            let (n, d) = (n as int, d as int);
            let (f, rem) = (n / d, n % d);
            &&& 0 < rem < d
            &&& i128::MIN <= f && f + 1 <= i128::MAX
            &&& rounds(n, d, RoundingMode::TowardNegative, f)
            &&& rounds(n, d, RoundingMode::TowardPositive, f + 1)
            &&& rounds(n, d, RoundingMode::TowardZero, if n < 0 { f + 1 } else { f })
            &&& 2 * rem < d ==> rounds(n, d, RoundingMode::NearestEven, f) && rounds(n, d, RoundingMode::NearestAway, f)
            &&& 2 * rem > d ==> rounds(n, d, RoundingMode::NearestEven, f + 1) && rounds(
                n,
                d,
                RoundingMode::NearestAway,
                f + 1,
            )
            &&& 2 * rem == d ==> rounds(n, d, RoundingMode::NearestEven, if f % 2 == 0 { f } else { f + 1 }) && rounds(
                n,
                d,
                RoundingMode::NearestAway,
                if n < 0 { f } else { f + 1 },
            )
        }),
{
    let (n, d) = (n as int, d as int);
    let (f, rem) = (n / d, n % d);
    lemma_fundamental_div_mod(n, d);
    lemma_mod_bound(n, d);
    assert(n == f * d + rem) by (nonlinear_arith)
        requires
            n == d * f + rem,
    ;
    assert((f + 1) * d == f * d + d && (f + 1 - 1) * d == f * d) by (nonlinear_arith);
    assert(i128::MIN <= f && f + 1 <= i128::MAX) by (nonlinear_arith)
        requires
            n == f * d + rem,
            0 < rem < d,
            d >= 2,
            i128::MIN <= n <= i128::MAX,
    ;
    assert(is_floor(n, d, f));
    assert(is_ceiling(n, d, f + 1));
    assert(vstd::math::abs(f * d - n) == rem);
    assert(vstd::math::abs((f + 1) * d - n) == d - rem);
    if 2 * rem == d {
        if n < 0 {
            assert(f * d < n);
        } else {
            assert((f + 1) * d > n && n > 0);
        }
        assert((f + 1) % 2 == 0 <==> f % 2 != 0) by {
            lemma_fundamental_div_mod(f, 2);
            lemma_fundamental_div_mod(f + 1, 2);
        }
    }
}

// ---- the loss record ---------------------------------------------------------------

/// `5^n`.
pub(super) open spec fn pow5(n: nat) -> nat
    decreases n,
{
    if n == 0 {
        1
    } else {
        5 * pow5((n - 1) as nat)
    }
}

pub(super) proof fn lemma_pow5_pos(n: nat)
    ensures
        pow5(n) >= 1,
    decreases n,
{
    if n > 0 {
        lemma_pow5_pos((n - 1) as nat);
    }
}

pub(super) proof fn lemma_pow5_adds(a: nat, b: nat)
    ensures
        pow5(a + b) == pow5(a) * pow5(b),
    decreases b,
{
    if b == 0 {
        assert(pow5(a) * 1 == pow5(a));
    } else {
        lemma_pow5_adds(a, (b - 1) as nat);
        assert(pow5(a + b) == 5 * pow5((a + b - 1) as nat));
        assert(pow5(a) * (5 * pow5((b - 1) as nat)) == 5 * (pow5(a) * pow5((b - 1) as nat))) by (nonlinear_arith);
    }
}

/// The ladder `5^1, 5^2, 5^4, 5^8, 5^16, 5^32`, and `5^64` past 128 bits.
pub(super) proof fn lemma_pow5_ladder()
    ensures
        pow5(1) == 5,
        pow5(2) == 25,
        pow5(4) == 625,
        pow5(8) == 390_625,
        pow5(16) == 152_587_890_625,
        pow5(32) == 23_283_064_365_386_962_890_625,
        pow5(64) > u128::MAX,
{
    assert(pow5(1) == 5) by {
        assert(pow5(0) == 1);
    }
    lemma_pow5_adds(1, 1);
    lemma_pow5_adds(2, 2);
    lemma_pow5_adds(4, 4);
    lemma_pow5_adds(8, 8);
    lemma_pow5_adds(16, 16);
    lemma_pow5_adds(32, 32);
    assert(625 * 625 == 390_625) by (compute_only);
    assert(390_625 * 390_625 == 152_587_890_625) by (compute_only);
    assert(152_587_890_625 * 152_587_890_625 == 23_283_064_365_386_962_890_625) by (compute_only);
    assert(23_283_064_365_386_962_890_625 * 23_283_064_365_386_962_890_625 > 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF)
        by (compute_only);
}

/// `10^n = 2^n x 5^n`.
pub(super) proof fn lemma_pow10_split(n: nat)
    ensures
        pow10(n) == pow2(n) * pow5(n),
    decreases n,
{
    if n == 0 {
        lemma2_to64();
    } else {
        lemma_pow10_split((n - 1) as nat);
        lemma_pow2_unfold(n);
        let (a, b) = (pow2((n - 1) as nat), pow5((n - 1) as nat));
        assert((2 * a) * (5 * b) == 10 * (a * b)) by (nonlinear_arith);
    }
}

/// Specification of the production `strip_twos`.
pub(super) open spec fn strip_twos(magnitude: u128, limit: u64) -> (u128, u64) {
    let zeros = magnitude.trailing_zeros() as u64;
    let count = if zeros <= limit {
        zeros
    } else {
        limit
    };
    (magnitude >> count, count)
}

/// `(m / 2^k, k)` with `k` the greatest exponent up to the limit.
pub(super) open spec fn twos_stripped(m: u128, limit: u64, r: (u128, u64)) -> bool {
    &&& r.1 <= limit
    &&& r.1 <= 127
    &&& r.0 != 0
    &&& r.0 * pow2(r.1 as nat) == m
    &&& r.1 < limit ==> r.0 % 2 == 1
}

/// A nonzero value is an odd number times two to its count of trailing zeros.
pub(super) proof fn lemma_trailing_zeros(m: u128)
    requires
        m != 0,
    ensures
        m.trailing_zeros() < 128,
        m == (m as nat / pow2(m.trailing_zeros() as nat)) * pow2(m.trailing_zeros() as nat),
        (m as nat / pow2(m.trailing_zeros() as nat)) % 2 == 1,
    decreases m,
{
    reveal_with_fuel(crate::verus_std_specs::u128_trailing_zeros, 2);
    lemma2_to64();
    if m % 2 == 1 {
        assert(m.trailing_zeros() == 0);
    } else {
        let h = (m / 2) as u128;
        lemma_trailing_zeros(h);
        let t = h.trailing_zeros() as nat;
        let k = h as nat / pow2(t);
        assert(m.trailing_zeros() == t + 1);
        lemma_pow2_unfold(t + 1);
        lemma_pow2_pos(t);
        assert(m == k * pow2(t + 1)) by (nonlinear_arith)
            requires
                h == k * pow2(t),
                m == 2 * h,
                pow2(t + 1) == 2 * pow2(t),
        ;
        lemma_div_multiples_vanish(k as int, pow2(t + 1) as int);
        lemma_mul_is_commutative(k as int, pow2(t + 1) as int);
        // An odd multiple of `2^(t+1)` below `2^128` has `t + 1 < 128`.
        if t + 1 >= 128 {
            lemma_pow2_le(128, t + 1);
            lemma2_to64_rest();
            lemma_pow2_adds(64, 64);
            assert(k * pow2(t + 1) >= pow2(t + 1)) by (nonlinear_arith)
                requires
                    k >= 1,
            ;
        }
    }
}

/// The production `strip_twos` removes the twos it says.
pub(super) proof fn lemma_strip_twos(m: u128, limit: u64)
    requires
        m != 0,
    ensures
        m.trailing_zeros() < 128,
        twos_stripped(m, limit, strip_twos(m, limit)),
{
    lemma_trailing_zeros(m);
    let zeros = m.trailing_zeros() as nat;
    let (rest, count) = strip_twos(m, limit);
    let c = count as nat;
    let k = m as nat / pow2(zeros);
    vstd::bits::lemma_u128_shr_is_div(m, count as u128);
    assert(m >> count == m >> (count as u128)) by (bit_vector)
        requires
            count < 128,
    ;
    assert(rest == m as nat / pow2(c));
    lemma_pow2_adds(c, (zeros - c) as nat);
    lemma_pow2_pos(c);
    lemma_pow2_pos((zeros - c) as nat);
    let extra = pow2((zeros - c) as nat);
    assert(m == (k * extra) * pow2(c)) by (nonlinear_arith)
        requires
            m == k * pow2(zeros),
            pow2(zeros) == pow2(c) * extra,
    ;
    lemma_div_multiples_vanish((k * extra) as int, pow2(c) as int);
    lemma_mul_is_commutative((k * extra) as int, pow2(c) as int);
    assert(rest == k * extra);
    assert(rest != 0) by (nonlinear_arith)
        requires
            rest == k * extra,
            k >= 1,
            extra >= 1,
    ;
    if count < limit {
        assert(c == zeros);
        lemma2_to64();
        assert(k * 1 == k) by (nonlinear_arith);
    }
}

/// Specification of the production `strip_five`.
pub(super) open spec fn strip_five(magnitude: u128, count: u64, limit: u64, width: u64, power: u128) -> (u128, u64) {
    if width <= limit - count && magnitude % power == 0 {
        (magnitude / power, (count + width) as u64)
    } else {
        (magnitude, count)
    }
}

/// Specification of the production `strip_fives`.
pub(super) open spec fn strip_fives(magnitude: u128, limit: u64) -> (u128, u64) {
    let (m, c) = strip_five(magnitude, 0, limit, 32, 23_283_064_365_386_962_890_625);
    let (m, c) = strip_five(m, c, limit, 16, 152_587_890_625);
    let (m, c) = strip_five(m, c, limit, 8, 390_625);
    let (m, c) = strip_five(m, c, limit, 4, 625);
    let (m, c) = strip_five(m, c, limit, 2, 25);
    strip_five(m, c, limit, 1, 5)
}

/// `(m / 5^k, k)` with `k` the greatest exponent up to the limit.
pub(super) open spec fn fives_stripped(m: u128, limit: u64, r: (u128, u64)) -> bool {
    &&& r.1 <= limit
    &&& r.1 <= 63
    &&& r.0 != 0
    &&& r.0 * pow5(r.1 as nat) == m
    &&& r.1 < limit ==> r.0 % 5 != 0
}

/// One rung of the ladder: if twice the width could not be stripped before
/// the rung, the width cannot be stripped after it.
pub(super) proof fn lemma_strip_five(m0: nat, m: u128, c: u64, limit: u64, w: u64, p: u128)
    requires
        m != 0,
        c <= limit,
        1 <= w <= 32,
        c <= 64,
        p == pow5(w as nat),
        m0 == m * pow5(c as nat),
        !((m as nat) % pow5((2 * w) as nat) == 0 && c + 2 * w <= limit),
    ensures
        ({
            let r = strip_five(m, c, limit, w, p);
            &&& r.0 != 0
            &&& c <= r.1 <= c + w
            &&& r.1 <= limit
            &&& m0 == r.0 * pow5(r.1 as nat)
            &&& !((r.0 as nat) % pow5(w as nat) == 0 && r.1 + w <= limit)
        }),
{
    lemma_pow5_pos(w as nat);
    if w <= limit - c && m % p == 0 {
        let q = (m / p) as nat;
        lemma_fundamental_div_mod(m as int, p as int);
        assert(m == p * q);
        assert(q != 0) by (nonlinear_arith)
            requires
                m == p * q,
                m != 0,
        ;
        lemma_pow5_adds(c as nat, w as nat);
        assert(m0 == q * pow5((c + w) as nat)) by (nonlinear_arith)
            requires
                m0 == m * pow5(c as nat),
                m == p * q,
                pow5((c + w) as nat) == pow5(c as nat) * p,
        ;
        if q % (p as nat) == 0 && c + w + w <= limit {
            let j = q / (p as nat);
            lemma_fundamental_div_mod(q as int, p as int);
            lemma_pow5_adds(w as nat, w as nat);
            lemma_pow5_pos((2 * w) as nat);
            assert(m == j * pow5((2 * w) as nat)) by (nonlinear_arith)
                requires
                    m == p * q,
                    q == p * j,
                    pow5((2 * w) as nat) == p * p,
            ;
            lemma_mod_multiples_basic(j as int, pow5((2 * w) as nat) as int);
            assert(false);
        }
    }
}

/// The production `strip_fives` removes the fives it says.
pub(super) proof fn lemma_strip_fives(m: u128, limit: u64)
    requires
        m != 0,
    ensures
        fives_stripped(m, limit, strip_fives(m, limit)),
{
    lemma_pow5_ladder();
    assert(pow5(0) == 1);
    lemma_small_mod(m as nat, pow5(64));
    let m0 = m as nat;
    lemma_strip_five(m0, m, 0, limit, 32, 23_283_064_365_386_962_890_625);
    let (m1, c1) = strip_five(m, 0, limit, 32, 23_283_064_365_386_962_890_625);
    lemma_strip_five(m0, m1, c1, limit, 16, 152_587_890_625);
    let (m2, c2) = strip_five(m1, c1, limit, 16, 152_587_890_625);
    lemma_strip_five(m0, m2, c2, limit, 8, 390_625);
    let (m3, c3) = strip_five(m2, c2, limit, 8, 390_625);
    lemma_strip_five(m0, m3, c3, limit, 4, 625);
    let (m4, c4) = strip_five(m3, c3, limit, 4, 625);
    lemma_strip_five(m0, m4, c4, limit, 2, 25);
    let (m5, c5) = strip_five(m4, c4, limit, 2, 25);
    lemma_strip_five(m0, m5, c5, limit, 1, 5);
}

/// Specification of the production `loss_record`.
pub(super) open spec fn loss_record(
    numerator: i128,
    denominator: i128,
    ten_exponent: u64,
    rounded: NativeDecimal,
    mode: RoundingMode,
) -> NativeDecimalLoss {
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
        twos: (denominator_twos + (ten_exponent - cancelled_twos)) as u64,
        fives: (denominator_fives + (ten_exponent - cancelled_fives)) as u64,
        rounded,
        mode,
    }
}

/// The loss record of the exact value `n / (d x 10^ten)`: the same value as
/// `numerator / (cofactor x 2^twos x 5^fives)`, a positive cofactor coprime
/// to ten, and lowest terms when `n / d` is in lowest terms (no factor of two
/// or five shared with the powers, none shared with the cofactor).
pub(super) open spec fn loss_is(
    n: int,
    d: int,
    ten: nat,
    rounded: NativeDecimal,
    mode: RoundingMode,
    l: NativeDecimalLoss,
) -> bool {
    &&& l.rounded == rounded
    &&& l.mode == mode
    &&& l.numerator != 0
    &&& l.cofactor > 0
    &&& l.cofactor % 2 != 0
    &&& l.cofactor % 5 != 0
    &&& l.numerator * (d * pow10(ten)) == n * (l.cofactor * pow2(l.twos as nat) * pow5(l.fives as nat))
    &&& (n % 2 != 0 || d % 2 != 0) && l.twos > 0 ==> l.numerator % 2 != 0
    &&& (n % 5 != 0 || d % 5 != 0) && l.fives > 0 ==> l.numerator % 5 != 0
    &&& gcd(vstd::math::abs(n), vstd::math::abs(d)) == 1 ==> gcd(vstd::math::abs(l.numerator as int), l.cofactor as nat)
        == 1
}

/// A product with an even factor is even; with a factor of five, a multiple
/// of five.
pub(super) proof fn lemma_factor_mod(x: nat, y: nat, p: nat)
    requires
        p > 0,
        x % p == 0,
    ensures
        (x * y) % p == 0,
        (y * x) % p == 0,
{
    lemma_fundamental_div_mod(x as int, p as int);
    let j = x / p;
    assert(x * y == (j * y) * p && y * x == (j * y) * p) by (nonlinear_arith)
        requires
            x == p * j,
    ;
    lemma_mod_multiples_basic((j * y) as int, p as int);
}

/// A divisor of a factor divides the product.
pub(super) proof fn lemma_divides_product(c: nat, x: nat, k: nat)
    requires
        divides(c, x),
    ensures
        divides(c, x * k),
{
    lemma_factor_mod(x, k, c);
}

/// The parity and the residue modulo five of a negation.
pub(super) proof fn lemma_neg_mod(x: int, p: int)
    requires
        p > 0,
    ensures
        (x % p == 0) <==> ((-x) % p == 0),
{
    lemma_fundamental_div_mod(x, p);
    lemma_fundamental_div_mod(-x, p);
    let (a, b) = (x / p, (-x) / p);
    if x % p == 0 {
        assert(-x == p * (-a)) by (nonlinear_arith)
            requires
                x == p * a,
        ;
        lemma_mod_multiples_basic(-a, p);
        assert((p * (-a)) % p == 0) by {
            lemma_mul_is_commutative(p, -a);
        }
    }
    if (-x) % p == 0 {
        assert(x == p * (-b)) by (nonlinear_arith)
            requires
                -x == p * b,
        ;
        lemma_mod_multiples_basic(-b, p);
        assert((p * (-b)) % p == 0) by {
            lemma_mul_is_commutative(p, -b);
        }
    }
}

/// The production `loss_record` meets `loss_is`, and its exponents stay far
/// inside `u64`.
pub(super) proof fn lemma_loss_record(
    numerator: i128,
    denominator: i128,
    ten_exponent: u64,
    rounded: NativeDecimal,
    mode: RoundingMode,
)
    requires
        numerator != 0,
        denominator > 0,
        ten_exponent <= 0x2_0000_0000,
    ensures
        ({
            let (m1, ct) = strip_twos(numerator.unsigned_abs(), ten_exponent);
            let (mag, cf) = strip_fives(m1, ten_exponent);
            let (rest, dt) = strip_twos(denominator.unsigned_abs(), u64::MAX);
            let (cof, df) = strip_fives(rest, u64::MAX);
            &&& numerator.unsigned_abs() != 0 && m1 != 0 && denominator.unsigned_abs() != 0 && rest != 0
            &&& ct <= ten_exponent && cf <= ten_exponent && dt <= 127 && df <= 63
        }),
        loss_is(
            numerator as int,
            denominator as int,
            ten_exponent as nat,
            rounded,
            mode,
            loss_record(numerator, denominator, ten_exponent, rounded, mode),
        ),
{
    let (n, d, ten) = (numerator as int, denominator as int, ten_exponent as nat);
    let (na, da) = (numerator.unsigned_abs(), denominator.unsigned_abs());
    lemma_strip_twos(na, ten_exponent);
    let (m1, ct) = strip_twos(na, ten_exponent);
    lemma_strip_fives(m1, ten_exponent);
    let (mag, cf) = strip_fives(m1, ten_exponent);
    lemma_strip_twos(da, u64::MAX);
    let (rest, dt) = strip_twos(da, u64::MAX);
    lemma_strip_fives(rest, u64::MAX);
    let (cof, df) = strip_fives(rest, u64::MAX);
    let l = loss_record(numerator, denominator, ten_exponent, rounded, mode);
    let (twos, fives) = ((dt + (ten - ct)) as nat, (df + (ten - cf)) as nat);
    assert(l.twos == twos && l.fives == fives);
    // The powers, named.
    let (a, b, c, e) = (pow2(ct as nat), pow5(cf as nat), pow2(dt as nat), pow5(df as nat));
    let (p2, p5, g, h) = (pow2(ten), pow5(ten), pow2(twos), pow5(fives));
    lemma_pow2_pos(ct as nat);
    lemma_pow2_pos(dt as nat);
    lemma_pow2_pos(ten);
    lemma_pow2_pos(twos);
    lemma_pow5_pos(cf as nat);
    lemma_pow5_pos(df as nat);
    lemma_pow5_pos(ten);
    lemma_pow5_pos(fives);
    lemma_pow2_adds(ct as nat, twos);
    lemma_pow2_adds(dt as nat, ten);
    lemma_pow5_adds(cf as nat, fives);
    lemma_pow5_adds(df as nat, ten);
    assert(a * g == c * p2);
    assert(b * h == e * p5);
    lemma_pow10_split(ten);
    // The magnitudes: |n| = mag b a, |d| = cof e c.
    assert(na == (mag * b) * a);
    assert(da == (cof * e) * c);
    // The signed numerator of the record is the stripped magnitude with the
    // sign of `n`; the casts lose nothing.
    assert(mag <= na) by (nonlinear_arith)
        requires
            na == (mag * b) * a,
            a >= 1,
            b >= 1,
    ;
    assert(cof <= da) by (nonlinear_arith)
        requires
            da == (cof * e) * c,
            c >= 1,
            e >= 1,
    ;
    lemma_reduce_quotient(numerator, denominator);
    assert(na <= 0x8000_0000_0000_0000_0000_0000_0000_0000u128);
    let sign: int = if n < 0 {
        -1
    } else {
        1
    };
    let signed = mag as i128;
    assert(l.numerator == (if numerator < 0 { signed.wrapping_neg() } else { signed }));
    if n < 0 {
        lemma_neg_magnitude(mag);
        assert(l.numerator == -(mag as int));
        assert(sign * mag == -(mag as int)) by (nonlinear_arith)
            requires
                sign == -1,
        ;
    } else {
        assert(na <= i128::MAX);
        assert(mag < 0x8000_0000_0000_0000_0000_0000_0000_0000u128 ==> mag as i128 == mag) by (bit_vector);
        assert(l.numerator == mag as int);
        assert(sign * mag == mag as int) by (nonlinear_arith)
            requires
                sign == 1,
        ;
    }
    assert(l.numerator == sign * mag);
    assert(l.cofactor == cof) by {
        assert(cof < 0x8000_0000_0000_0000_0000_0000_0000_0000u128 ==> cof as i128 == cof) by (bit_vector);
    }
    assert(n == sign * na && d == da);
    // The value: both sides are sign x mag x cof x (2^dt 2^ten) x (5^df 5^ten).
    let x = sign * mag * cof;
    assert(l.numerator * (d * pow10(ten)) == x * (c * p2) * (e * p5)) by (nonlinear_arith)
        requires
            l.numerator == sign * mag,
            d == (cof * e) * c,
            pow10(ten) == p2 * p5,
            x == sign * mag * cof,
    ;
    assert(n * (l.cofactor * g * h) == x * (a * g) * (b * h)) by (nonlinear_arith)
        requires
            n == sign * ((mag * b) * a),
            l.cofactor == cof,
            x == sign * mag * cof,
    ;
    // The cofactor: odd, not a multiple of five.
    assert(rest == cof * e);
    if cof % 2 == 0 {
        lemma_factor_mod(cof as nat, e, 2);
    }
    // No factor of two shared with the powers.
    lemma2_to64();
    if (n % 2 != 0 || d % 2 != 0) && twos > 0 {
        if mag % 2 == 0 {
            lemma_factor_mod(mag as nat, b, 2);
            assert(m1 % 2 == 0);
            lemma_factor_mod(m1 as nat, a, 2);
            assert(na % 2 == 0);
            // Then `d` is odd, so `dt = 0` and `ct < ten`, and the stripped
            // magnitude is odd.
            if dt > 0 {
                lemma_pow2_unfold(dt as nat);
                lemma_factor_mod(c, (cof * e) as nat, 2);
                assert(da % 2 == 0);
            }
            assert(ct < ten);
            assert(false);
        }
        lemma_neg_mod(mag as int, 2);
    }
    // No factor of five shared with the powers.
    if (n % 5 != 0 || d % 5 != 0) && fives > 0 {
        if mag % 5 == 0 {
            lemma_factor_mod(mag as nat, b, 5);
            lemma_factor_mod(m1 as nat, a, 5);
            assert(na % 5 == 0);
            if df > 0 {
                assert(e == 5 * pow5((df - 1) as nat));
                lemma_mod_multiples_basic(pow5((df - 1) as nat) as int, 5);
                lemma_factor_mod(e, cof as nat, 5);
                lemma_factor_mod((cof * e) as nat, c, 5);
                assert(da % 5 == 0);
            }
            assert(cf < ten);
            assert(false);
        }
        lemma_neg_mod(mag as int, 5);
    }
    lemma_neg_mod(n, 2);
    lemma_neg_mod(n, 5);
    // Nothing shared with the cofactor when `n / d` is in lowest terms.
    if gcd(vstd::math::abs(n), vstd::math::abs(d)) == 1 {
        assert(vstd::math::abs(l.numerator as int) == mag);
        assert forall|k: nat| #[trigger] common(k, mag as nat, cof as nat) implies k == 1 by {
            lemma_divides_product(k, mag as nat, b * a);
            lemma_divides_product(k, cof as nat, e * c);
            assert(mag * (b * a) == (mag * b) * a && cof * (e * c) == (cof * e) * c) by (nonlinear_arith);
            lemma_gcd_greatest(na as nat, da as nat, k);
            if k > 1 {
                lemma_small_mod(1, k);
            }
        }
        lemma_gcd_one_iff_no_common_factor(mag as nat, cof as nat);
    }
}

// ---- placement ---------------------------------------------------------------------

/// Specification of the production `exact_tail`: the coefficient placed at
/// its scale and lifted to the target scale, or ground (a) for a lift above
/// 38, or ground (g) when the lifted coefficient leaves signed 128 bits.
pub(super) open spec fn exact_tail(target: u32, coefficient: i128, scale: u32) -> Admission<Tail> {
    let lift = (target - scale) as nat;
    if lift > 38 {
        Err(NativeDecimalCapacityRefusal { quantity: NativeDecimalCapacityQuantity::Exponent })
    } else if !fits(coefficient * pow10(lift)) {
        Err(NativeDecimalCapacityRefusal { quantity: NativeDecimalCapacityQuantity::RetainedCoefficient })
    } else {
        Ok(
            Tail::Placed(
                Placement { coefficient, scale, retained: (coefficient * pow10(lift)) as i128, loss: None },
            ),
        )
    }
}

/// What a rounding step over `n / d` (in units of `10^-target`, not an
/// integer) returns: `Inexact` under `Exact`; otherwise the rounded
/// coefficient at the target scale with the loss record of the exact value
/// `exact.0 / (exact.1 x 10^exact.2)`, or ground (f) for a nearest mode whose
/// doubled remainder leaves signed 128 bits.
pub(super) open spec fn rounded_is(
    mode: RoundingMode,
    target: u32,
    n: int,
    d: int,
    overflow: bool,
    exact: (int, int, nat),
    r: Admission<Tail>,
) -> bool {
    if mode == RoundingMode::Exact {
        r == Ok::<Tail, NativeDecimalCapacityRefusal>(Tail::Inexact)
    } else {
        let refused = nearest(mode) && overflow;
        match r {
            Ok(Tail::Placed(p)) => {
                &&& !refused
                &&& rounds(n, d, mode, p.coefficient as int)
                &&& p.scale == target
                &&& p.retained == p.coefficient
                &&& p.loss is Some
                &&& loss_is(exact.0, exact.1, exact.2, p.loss->0.rounded, mode, p.loss->0)
                &&& p.loss->0.rounded.retained() == (p.coefficient, target)
            },
            Ok(_) => false,
            Err(e) => refused && e.quantity == NativeDecimalCapacityQuantity::DoubledRemainder,
        }
    }
}

/// What `place_scaled` returns for the integer `value` in units of
/// `10^-working`: an exact placement when the working scale is at most the
/// target, when the value is zero, or when `10^(working - target)` divides
/// it; ground (e) when that power is above 38 and is formed; and otherwise a
/// rounding step over `value / 10^(working - target)`.
pub(super) open spec fn scaled_is(mode: RoundingMode, target: u32, value: i128, working: u64, r: Admission<Tail>) -> bool {
    if working <= target {
        r == exact_tail(target, value, working as u32)
    } else if value == 0 {
        r == exact_tail(target, 0, target)
    } else {
        let excess = (working - target) as nat;
        let power = pow10(excess) as int;
        let value = value as int;
        let proxy = excess > digit_len(vstd::math::abs(value));
        if !proxy && excess > 38 {
            r == Err::<Tail, NativeDecimalCapacityRefusal>(
                NativeDecimalCapacityRefusal { quantity: NativeDecimalCapacityQuantity::PowerOfTen },
            )
        } else if value % power == 0 {
            r == exact_tail(target, (value / power) as i128, target)
        } else {
            let g = gcd(vstd::math::abs(value), power as nat) as int;
            rounded_is(
                mode,
                target,
                value,
                power,
                !proxy && 2 * ((value / g) % (power / g)) > i128::MAX,
                (value, 1, working as nat),
                r,
            )
        }
    }
}

/// What `place_quotient` returns for the reduced `n / d` in units of
/// `10^-target`: an exact placement for an integer, a rounding step otherwise.
pub(super) open spec fn quotient_is(mode: RoundingMode, target: u32, n: i128, d: i128, r: Admission<Tail>) -> bool {
    if d == 1 {
        r == exact_tail(target, n, target)
    } else {
        rounded_is(
            mode,
            target,
            n as int,
            d as int,
            2 * ((n as int) % (d as int)) > i128::MAX,
            (n as int, d as int, target as nat),
            r,
        )
    }
}

/// Rounding does not depend on the common factor of the fraction.
pub(super) proof fn lemma_rounds_scale(n: int, d: int, g: int, mode: RoundingMode, c: int)
    requires
        g > 0,
        d > 0,
        rounds(n, d, mode, c),
    ensures
        rounds(g * n, g * d, mode, c),
{
    assert(c * (g * d) == g * (c * d) && (c + 1) * (g * d) == g * ((c + 1) * d) && (c - 1) * (g * d) == g * ((c - 1)
        * d)) by (nonlinear_arith);
    assert(is_floor(n, d, c) ==> is_floor(g * n, g * d, c)) by (nonlinear_arith)
        requires
            g > 0,
            c * (g * d) == g * (c * d),
            (c + 1) * (g * d) == g * ((c + 1) * d),
    ;
    assert(is_ceiling(n, d, c) ==> is_ceiling(g * n, g * d, c)) by (nonlinear_arith)
        requires
            g > 0,
            c * (g * d) == g * (c * d),
            (c - 1) * (g * d) == g * ((c - 1) * d),
    ;
    assert((g * n >= 0) == (n >= 0)) by (nonlinear_arith)
        requires
            g > 0,
    ;
    let (x, y) = (c * d - n, c * (g * d) - g * n);
    assert(y == g * x) by (nonlinear_arith)
        requires
            y == c * (g * d) - g * n,
            x == c * d - n,
    ;
    assert(vstd::math::abs(y) == g * vstd::math::abs(x)) by (nonlinear_arith)
        requires
            y == g * x,
            g > 0,
    ;
    let (ax, ay) = (vstd::math::abs(x), vstd::math::abs(y));
    assert((2 * ax <= d ==> 2 * ay <= g * d) && (2 * ay == g * d ==> 2 * ax == d)) by (nonlinear_arith)
        requires
            ay == g * ax,
            g > 0,
    ;
    assert(vstd::math::abs(c * (g * d)) == g * vstd::math::abs(c * d) && vstd::math::abs(g * n) == g * vstd::math::abs(
        n,
    )) by (nonlinear_arith)
        requires
            g > 0,
            c * (g * d) == g * (c * d),
    ;
    assert(vstd::math::abs(c * d) > vstd::math::abs(n) ==> g * vstd::math::abs(c * d) > g * vstd::math::abs(n))
        by (nonlinear_arith)
        requires
            g > 0,
    ;
}

/// The sign of a nonzero integer.
pub(super) open spec fn sign_of(v: int) -> int {
    if v > 0 {
        1
    } else {
        -1
    }
}

/// A nonzero value of magnitude below a tenth of the unit rounds as a tenth
/// of its sign does: to zero, or one unit away from zero.
pub(super) proof fn lemma_proxy_rounds(value: int, unit: int, mode: RoundingMode, c: int)
    requires
        value != 0,
        10 * vstd::math::abs(value) < unit,
        rounds(sign_of(value), 10, mode, c),
    ensures
        rounds(value, unit, mode, c),
{
    assert(-1 <= c <= 1);
    if c == 0 {
        assert(0 * unit == 0 && (0 + 1) * unit == unit && (0 - 1) * unit == -unit) by (nonlinear_arith);
    } else if c == 1 {
        assert(1 * unit == unit && (1 + 1) * unit == 2 * unit && (1 - 1) * unit == 0) by (nonlinear_arith);
    } else {
        assert((-1) * unit == -unit && (-1 + 1) * unit == 0 && (-1 - 1) * unit == -2 * unit) by (nonlinear_arith);
    }
}

/// Truncating and floor division agree on an exact division by a positive
/// divisor.
pub(super) proof fn lemma_exact_division(v: i128, p: i128)
    requires
        p > 0,
    ensures
        (rust_rem(v as int, p as int) == 0) <==> ((v as int) % (p as int) == 0),
        rust_rem(v as int, p as int) == 0 ==> rust_div(v as int, p as int) == (v as int) / (p as int),
        i128::MIN <= rust_div(v as int, p as int) <= i128::MAX,
{
    lemma_truncating(v, p);
    let (q0, r0) = (rust_div(v as int, p as int), rust_rem(v as int, p as int));
    lemma_fundamental_div_mod(v as int, p as int);
    lemma_mod_bound(v as int, p as int);
    if r0 == 0 {
        lemma_fundamental_div_mod_converse(v as int, p as int, q0, 0);
    }
    if (v as int) % (p as int) == 0 {
        let e = (v as int) / (p as int);
        let k = e - q0;
        assert(r0 == p * k) by (nonlinear_arith)
            requires
                v == q0 * p + r0,
                v == p * e,
                k == e - q0,
        ;
        assert(k == 0) by (nonlinear_arith)
            requires
                -p < p * k < p,
                p > 0,
        ;
    }
}

/// The facts the body of `place_scaled` needs for a nonzero value and an
/// excess `e = working - target >= 1`.
pub(super) proof fn lemma_place_scaled(value: i128, e: nat)
    requires
        value != 0,
        e >= 1,
    ensures
        ({
            let v = value as int;
            let digits = digit_len(vstd::math::abs(v));
            let power = pow10(e) as int;
            let g = gcd(vstd::math::abs(v), power as nat) as int;
            &&& digits_of(value.unsigned_abs()) == digits
            &&& rust_rem(1, 10) == 1 && rust_rem(-1, 10) == -1 && 1int % 10 != 0 && (-1int) % 10 != 0
            // The proxy: below a tenth of the unit, so not a multiple of it,
            // and rounding as a tenth of the sign does.
            &&& e > digits ==> {
                &&& v % power != 0
                &&& v > 0 ==> forall|mode: RoundingMode, c: int| #[trigger]
                    rounds(1, 10, mode, c) ==> rounds(v, power, mode, c)
                &&& v < 0 ==> forall|mode: RoundingMode, c: int| #[trigger]
                    rounds(-1, 10, mode, c) ==> rounds(v, power, mode, c)
            }
            // The formed power: the table entry, and the reduced units.
            &&& e <= 38 ==> {
                &&& 1 <= power <= i128::MAX
                &&& (rust_rem(v, power) == 0) <==> (v % power == 0)
                &&& rust_rem(v, power) == 0 ==> rust_div(v, power) == v / power
                &&& i128::MIN <= rust_div(v, power) <= i128::MAX
                &&& 1 <= g <= power
                &&& gcd256(value.unsigned_abs(), (power as i128).unsigned_abs()) == g
                &&& rust_div(v, g) == v / g && rust_div(power, g) == power / g
                &&& v == g * (v / g) && power == g * (power / g)
                &&& v % power != 0 ==> power / g >= 2 && (v / g) % (power / g) != 0 && forall|
                    mode: RoundingMode,
                    c: int,
                | #[trigger] rounds(v / g, power / g, mode, c) ==> rounds(v, power, mode, c)
            }
        }),
{
    let v = value as int;
    let m = vstd::math::abs(v);
    let digits = digit_len(m);
    let power = pow10(e) as int;
    lemma_pow10_pos(e);
    lemma_digit_len_range(m);
    lemma_digits_of(value.unsigned_abs());
    assert(digits_of(value.unsigned_abs()) == digits);
    assert(rust_rem(1, 10) == 1 && rust_rem(-1, 10) == -1 && 1int % 10 != 0 && (-1int) % 10 != 0) by (compute);
    if e > digits {
        lemma_pow10_monotone(digits, (e - 1) as nat);
        assert(power == 10 * pow10((e - 1) as nat));
        assert(10 * m < power);
        if v % power == 0 {
            lemma_fundamental_div_mod(v, power);
            let k = v / power;
            assert(false) by (nonlinear_arith)
                requires
                    v == power * k,
                    v != 0,
                    -power < v < power,
            ;
        }
        if v > 0 {
            assert forall|mode: RoundingMode, c: int| #[trigger] rounds(1, 10, mode, c) implies rounds(v, power, mode, c) by {
                lemma_proxy_rounds(v, power, mode, c);
            }
        } else {
            assert forall|mode: RoundingMode, c: int| #[trigger] rounds(-1, 10, mode, c) implies rounds(v, power, mode, c) by {
                lemma_proxy_rounds(v, power, mode, c);
            }
        }
    }
    if e <= 38 {
        lemma_pow10_monotone(e, 38);
        lemma_pow10_bound(38);
        let p = power as i128;
        lemma_exact_division(value, p);
        lemma_gcd256(value.unsigned_abs(), p.unsigned_abs());
        let gn = gcd(m, power as nat);
        lemma_gcd_positive(m, power as nat);
        lemma_gcd_common(m, power as nat);
        lemma_divides_witness(gn, m);
        lemma_divides_witness(gn, power as nat);
        let g = gn as int;
        let (mq, pq) = ((m / gn) as int, power / g);
        // The signed value divides exactly by the gcd of its magnitude.
        let vq: int = if v < 0 {
            -mq
        } else {
            mq
        };
        assert(v == g * vq) by (nonlinear_arith)
            requires
                m == g * mq,
                v == (if v < 0 { -m } else { m as int }),
                vq == (if v < 0 { -mq } else { mq }),
        ;
        lemma_div_multiples_vanish(vq, g);
        assert(v / g == vq);
        let gi = g as i128;
        lemma_truncating(value, gi);
        lemma_truncating(p, gi);
        assert(rust_div(v, g) == vq) by (nonlinear_arith)
            requires
                v == rust_div(v, g) * g + rust_rem(v, g),
                v == g * vq,
                -g < rust_rem(v, g) < g,
                g > 0,
        ;
        assert(rust_div(power, g) == pq) by (nonlinear_arith)
            requires
                power == rust_div(power, g) * g + rust_rem(power, g),
                power == g * pq,
                -g < rust_rem(power, g) < g,
                g > 0,
        ;
        if v % power != 0 {
            assert(pq >= 1) by (nonlinear_arith)
                requires
                    power == g * pq,
                    power >= 1,
                    g >= 1,
            ;
            if pq == 1 {
                assert(v == power * vq) by (nonlinear_arith)
                    requires
                        v == g * vq,
                        power == g * pq,
                        pq == 1,
                ;
                lemma_mod_multiples_basic(vq, power);
                lemma_mul_is_commutative(power, vq);
                assert(false);
            }
            if vq % pq == 0 {
                lemma_fundamental_div_mod(vq, pq);
                let j = vq / pq;
                assert(v == j * power) by (nonlinear_arith)
                    requires
                        v == g * vq,
                        power == g * pq,
                        vq == pq * j,
                ;
                lemma_mod_multiples_basic(j, power);
                assert(false);
            }
            assert forall|mode: RoundingMode, c: int| #[trigger] rounds(vq, pq, mode, c) implies rounds(v, power, mode, c) by {
                lemma_rounds_scale(vq, pq, g, mode, c);
            }
        }
        assert(1 <= power <= i128::MAX);
        assert((rust_rem(v, power) == 0) <==> (v % power == 0));
        assert(1 <= g <= power);
        assert(gcd256(value.unsigned_abs(), (power as i128).unsigned_abs()) == g);
        assert(rust_div(v, g) == v / g && rust_div(power, g) == power / g);
        assert(v == g * (v / g) && power == g * (power / g));
    }
}

/// A reduced fraction with a denominator other than one is not an integer
/// and its numerator is not zero.
pub(super) proof fn lemma_place_quotient(n: i128, d: i128)
    requires
        canonical(n as int, d as int),
    ensures
        d >= 1,
        d != 1 ==> n != 0 && (n as int) % (d as int) != 0,
{
    let (ni, di) = (n as int, d as int);
    if d != 1 && ni % di == 0 {
        assert(n != 0);
        let m = vstd::math::abs(ni);
        // `d` divides `|n|` and itself, so it divides their gcd, which is 1.
        lemma_neg_mod(ni, di);
        assert(m % (di as nat) == 0);
        lemma_mod_self_0(di);
        lemma_gcd_greatest(m, di as nat, di as nat);
        lemma_small_mod(1, di as nat);
        assert(false);
    }
}

// ---- admission and the decision ------------------------------------------------------

/// The first failing quantity of an operation's plan; none for a zero
/// divisor, to which no ground applies.
pub(super) open spec fn plan_refusal(target: u32, operation: NativeDecimalOperation) -> Option<
    NativeDecimalCapacityQuantity,
> {
    match operation {
        NativeDecimalOperation::Add(a, b) => combine_refusal(target, a, b, false),
        NativeDecimalOperation::Subtract(a, b) => combine_refusal(target, a, b, true),
        NativeDecimalOperation::Multiply(a, b) => multiply_refusal(target, a, b),
        NativeDecimalOperation::Divide(a, b) => if b.coefficient == 0 {
            None
        } else {
            divide_refusal(target, a, b)
        },
        NativeDecimalOperation::Negate(a) => unary_refusal(target, a, true),
        NativeDecimalOperation::Round(a) => unary_refusal(target, a, false),
    }
}

/// The exact intermediate of an operation: an integer at a working scale, or
/// the reduced quotient in units of `10^-target`; none for a zero divisor.
pub(super) open spec fn plan_intermediate(target: u32, operation: NativeDecimalOperation) -> Option<Intermediate> {
    match operation {
        NativeDecimalOperation::Add(a, b) => Some(
            Intermediate::Scaled { value: combine_value(a, b, false) as i128, scale: combine_scale(a, b) as u64 },
        ),
        NativeDecimalOperation::Subtract(a, b) => Some(
            Intermediate::Scaled { value: combine_value(a, b, true) as i128, scale: combine_scale(a, b) as u64 },
        ),
        NativeDecimalOperation::Multiply(a, b) => Some(
            Intermediate::Scaled { value: (a.coefficient * b.coefficient) as i128, scale: (a.scale + b.scale) as u64 },
        ),
        NativeDecimalOperation::Divide(a, b) => if b.coefficient == 0 {
            None
        } else {
            let (n, d) = divide_parts(target, a, b);
            let q = quotient_parts(n, d);
            Some(Intermediate::Quotient { numerator: q.0 as i128, denominator: q.1 as i128 })
        },
        NativeDecimalOperation::Negate(a) => Some(
            Intermediate::Scaled { value: (-a.coefficient) as i128, scale: a.scale as u64 },
        ),
        NativeDecimalOperation::Round(a) => Some(Intermediate::Scaled { value: a.coefficient, scale: a.scale as u64 }),
    }
}

/// A planned quotient is in lowest terms.
pub(super) proof fn lemma_planned(target: u32, operation: NativeDecimalOperation)
    ensures
        plan_refusal(target, operation) is None ==> match plan_intermediate(target, operation) {
            Some(Intermediate::Quotient { numerator, denominator }) => canonical(numerator as int, denominator as int),
            _ => true,
        },
{
    if let NativeDecimalOperation::Divide(a, b) = operation {
        if b.coefficient != 0 {
            let (n, d) = divide_parts(target, a, b);
            lemma_scaled_nonzero(b.coefficient as int, divide_shifts(target, a, b).1);
            lemma_quotient_parts(n, d);
        }
    }
}

/// The placement an admission result carries.
pub(super) open spec fn tail_of(r: Result<AdmittedNativeDecimalOperation, NativeDecimalCapacityRefusal>) -> Admission<
    Tail,
> {
    match r {
        Ok(admitted) => Ok(admitted.tail),
        Err(e) => Err(e),
    }
}

/// A placed coefficient sits at a scale at most the target scale.
pub(super) open spec fn admitted_wf(admitted: AdmittedNativeDecimalOperation) -> bool {
    match admitted.tail {
        Tail::Placed(p) => p.scale <= admitted.ty.max_scale,
        _ => true,
    }
}

/// What `admit` returns: the plan's first failing quantity; a zero divisor
/// admitted as such; and otherwise the placement of the exact intermediate,
/// with its own grounds.
pub(super) open spec fn admitted_is(
    ty: NativeDecimalType,
    operation: NativeDecimalOperation,
    r: Result<AdmittedNativeDecimalOperation, NativeDecimalCapacityRefusal>,
) -> bool {
    &&& r is Ok ==> r->Ok_0.ty == ty && admitted_wf(r->Ok_0)
    &&& match plan_refusal(ty.max_scale, operation) {
        Some(quantity) => r is Err && r->Err_0.quantity == quantity,
        None => match plan_intermediate(ty.max_scale, operation) {
            None => r is Ok && r->Ok_0.tail == Tail::DivisionByZero,
            Some(Intermediate::Scaled { value, scale }) => scaled_is(ty.rounding, ty.max_scale, value, scale, tail_of(r)),
            Some(Intermediate::Quotient { numerator, denominator }) => quotient_is(
                ty.rounding,
                ty.max_scale,
                numerator,
                denominator,
                tail_of(r),
            ),
        },
    }
}

/// What `settle` returns: the zero divisor and the inexact refusal as
/// admitted; for a placed coefficient, the completed value retained at the
/// target scale when the type admits the placed value, and the domain
/// refusal otherwise.
pub(super) open spec fn settled_is(admitted: AdmittedNativeDecimalOperation, r: Settled) -> bool {
    match admitted.tail {
        Tail::DivisionByZero => r == Settled::DivisionByZero,
        Tail::Inexact => r == Settled::Inexact,
        Tail::Placed(p) => if super::super::verus_proofs::admits(admitted.ty.bounds(), (p.coefficient, p.scale)) {
            match r {
                Settled::Completed(result, retain) => {
                    &&& result.value.retained() == (p.retained, admitted.ty.max_scale)
                    &&& result.loss == p.loss
                    &&& retain == Retain { coefficient: p.coefficient, lift: (admitted.ty.max_scale - p.scale) as u32 }
                },
                _ => false,
            }
        } else {
            r == Settled::OutOfDomain { rounding: p.loss is Some }
        },
    }
}

/// The exact result of an intermediate in units of `10^-target`, as a
/// fraction with a positive denominator.
pub(super) open spec fn exact_units(target: u32, intermediate: Intermediate) -> (int, int) {
    match intermediate {
        Intermediate::Scaled { value, scale } => if scale <= target {
            (value * pow10((target - scale) as nat), 1)
        } else {
            (value as int, pow10((scale - target) as nat) as int)
        },
        Intermediate::Quotient { numerator, denominator } => (numerator as int, denominator as int),
    }
}

/// What an admitted placement means for the exact result `p / q` in units of
/// `10^-target`. Without a loss record the result is an integer and the
/// retained coefficient is exactly it, with the placed coefficient the same
/// value at its scale. With a loss record the result is not an integer, the
/// retained coefficient is its rounding under the mode, and the record holds
/// the exact value `p / (q x 10^target)` in lowest terms with its powers of
/// two and five factored out. `Inexact` arises only under `Exact` for a
/// result that is not an integer.
pub(super) open spec fn tail_means(mode: RoundingMode, target: u32, intermediate: Intermediate, tail: Tail) -> bool {
    let (p, q) = exact_units(target, intermediate);
    match tail {
        Tail::DivisionByZero => false,
        Tail::Inexact => mode == RoundingMode::Exact && p % q != 0,
        Tail::Placed(placed) => placed.scale <= target && match placed.loss {
            None => {
                &&& p % q == 0
                &&& placed.retained * q == p
                &&& placed.coefficient * pow10((target - placed.scale) as nat) == placed.retained
            },
            Some(l) => {
                &&& mode != RoundingMode::Exact
                &&& p % q != 0
                &&& rounds(p, q, mode, placed.retained as int)
                &&& placed.coefficient == placed.retained
                &&& placed.scale == target
                &&& l.rounded.retained() == (placed.retained, target)
                &&& l.mode == mode
                &&& l.numerator != 0
                &&& l.cofactor > 0
                &&& l.cofactor % 2 != 0
                &&& l.cofactor % 5 != 0
                &&& l.numerator * (q * pow10(target as nat)) == p * (l.cofactor * pow2(l.twos as nat) * pow5(
                    l.fives as nat,
                ))
                &&& l.twos > 0 ==> l.numerator % 2 != 0
                &&& l.fives > 0 ==> l.numerator % 5 != 0
                &&& gcd(vstd::math::abs(l.numerator as int), l.cofactor as nat) == 1
            },
        },
    }
}

/// The placement of a placed tail.
pub(super) open spec fn placed_of(tail: Tail) -> Placement {
    match tail {
        Tail::Placed(p) => p,
        _ => arbitrary(),
    }
}

/// An admitted placement of a scaled intermediate means `tail_means`.
pub(super) proof fn lemma_scaled_means(mode: RoundingMode, target: u32, value: i128, working: u64, r: Admission<Tail>)
    requires
        scaled_is(mode, target, value, working, r),
        r is Ok,
    ensures
        tail_means(mode, target, Intermediate::Scaled { value, scale: working }, r->Ok_0),
{
    let v = value as int;
    assert(pow10(0) == 1);
    if working <= target {
    } else {
        let excess = (working - target) as nat;
        let power = pow10(excess) as int;
        lemma_pow10_pos(excess);
        if value == 0 {
            lemma_mod_multiples_basic(0, power);
            assert(0 * power == 0) by (nonlinear_arith);
        } else if v % power == 0 {
            // A multiple of the power is not below it, so the power was
            // formed and is at most `10^38`.
            lemma_place_scaled(value, excess);
            lemma_fundamental_div_mod(v, power);
            assert((v / power) * power == v) by (nonlinear_arith)
                requires
                    v == power * (v / power),
            ;
            assert(((v / power) as i128) * pow10(0) == v / power);
        } else if mode != RoundingMode::Exact {
            lemma_pow10_adds(excess, target as nat);
            let l = placed_of(r->Ok_0).loss->0;
            let rhs = v * (l.cofactor * pow2(l.twos as nat) * pow5(l.fives as nat));
            assert(l.numerator * (power * pow10(target as nat)) == rhs) by (nonlinear_arith)
                requires
                    l.numerator * (1 * pow10(working as nat)) == rhs,
                    pow10(working as nat) == power * pow10(target as nat),
            ;
            assert(gcd(vstd::math::abs(v), 1) == 1) by {
                reveal_with_fuel(gcd, 3);
            }
        }
    }
}

/// An admitted placement of a reduced quotient means `tail_means`.
pub(super) proof fn lemma_quotient_means(mode: RoundingMode, target: u32, n: i128, d: i128, r: Admission<Tail>)
    requires
        canonical(n as int, d as int),
        quotient_is(mode, target, n, d, r),
        r is Ok,
    ensures
        tail_means(mode, target, Intermediate::Quotient { numerator: n, denominator: d }, r->Ok_0),
{
    assert(pow10(0) == 1);
    lemma_place_quotient(n, d);
    if d != 1 && mode != RoundingMode::Exact {
        let m = vstd::math::abs(n as int);
        // Lowest terms: not both even, not both multiples of five.
        if m % 2 == 0 && d % 2 == 0 {
            lemma_gcd_greatest(m, d as nat, 2);
            lemma_small_mod(1, 2);
        }
        if m % 5 == 0 && d % 5 == 0 {
            lemma_gcd_greatest(m, d as nat, 5);
            lemma_small_mod(1, 5);
        }
        lemma_neg_mod(n as int, 2);
        lemma_neg_mod(n as int, 5);
        assert((n as int) % 2 != 0 || (d as int) % 2 != 0);
        assert((n as int) % 5 != 0 || (d as int) % 5 != 0);
        assert(gcd(vstd::math::abs(n as int), vstd::math::abs(d as int)) == 1);
        let placed = placed_of(r->Ok_0);
        let l = placed.loss->0;
        assert(r->Ok_0 == Tail::Placed(placed));
        assert(loss_is(n as int, d as int, target as nat, l.rounded, mode, l));
        assert(rounds(n as int, d as int, mode, placed.retained as int));
    } else if d == 1 {
        assert(n * pow10(0) == n) by (nonlinear_arith)
            requires
                pow10(0) == 1,
        ;
        assert((n as int) % 1 == 0);
    }
}

/// The meaning of an admitted operation: a zero divisor, or `tail_means` of
/// its exact intermediate.
pub(super) proof fn lemma_admitted(
    ty: NativeDecimalType,
    operation: NativeDecimalOperation,
    r: Result<AdmittedNativeDecimalOperation, NativeDecimalCapacityRefusal>,
)
    requires
        admitted_is(ty, operation, r),
        r is Ok,
    ensures
        plan_refusal(ty.max_scale, operation) is None,
        match plan_intermediate(ty.max_scale, operation) {
            None => r->Ok_0.tail == Tail::DivisionByZero,
            Some(intermediate) => tail_means(ty.rounding, ty.max_scale, intermediate, r->Ok_0.tail),
        },
{
    lemma_planned(ty.max_scale, operation);
    match plan_intermediate(ty.max_scale, operation) {
        None => {},
        Some(Intermediate::Scaled { value, scale }) => {
            lemma_scaled_means(ty.rounding, ty.max_scale, value, scale, tail_of(r));
        },
        Some(Intermediate::Quotient { numerator, denominator }) => {
            lemma_quotient_means(ty.rounding, ty.max_scale, numerator, denominator, tail_of(r));
        },
    }
}

/// The quotient of a `Divide` in units of `10^-target`: `N / D` is
/// `(a / 10^sa) / (b / 10^sb) x 10^target`, cross-multiplied.
pub(super) proof fn lemma_divide_parts(target: u32, a: NativeDecimal, b: NativeDecimal)
    ensures
        ({
            let (n, d) = divide_parts(target, a, b);
            n * (b.coefficient * pow10(a.scale as nat)) == d * (a.coefficient * pow10((b.scale + target) as nat))
        }),
{
    let (kn, kd) = divide_shifts(target, a, b);
    let (n, d) = divide_parts(target, a, b);
    let up = (b.scale + target) as nat;
    // `kn + sa = kd + up`.
    lemma_pow10_adds(kn, a.scale as nat);
    lemma_pow10_adds(kd, up);
    let (pn, pa, pd, pu) = (pow10(kn), pow10(a.scale as nat), pow10(kd), pow10(up));
    let (ca, cb) = (a.coefficient as int, b.coefficient as int);
    assert((ca * pn) * (cb * pa) == (cb * pd) * (ca * pu)) by (nonlinear_arith)
        requires
            pn * pa == pd * pu,
    ;
}

// ---- the ordering -------------------------------------------------------------------

/// The order of two integers.
pub(super) open spec fn cmp_int(a: int, b: int) -> Ordering {
    if a < b {
        Ordering::Less
    } else if a == b {
        Ordering::Equal
    } else {
        Ordering::Greater
    }
}

/// The order of two Decimals by mathematical value: `cl / 10^sl` against
/// `cr / 10^sr`, cross-multiplied.
pub(super) open spec fn value_order(left: NativeDecimal, right: NativeDecimal) -> Ordering {
    cmp_int(left.coefficient * pow10(right.scale as nat), right.coefficient * pow10(left.scale as nat))
}

/// The order of the magnitudes of two Decimals.
pub(super) open spec fn magnitude_order(left: NativeDecimal, right: NativeDecimal) -> Ordering {
    cmp_int(
        (vstd::math::abs(left.coefficient as int) * pow10(right.scale as nat)) as int,
        (vstd::math::abs(right.coefficient as int) * pow10(left.scale as nat)) as int,
    )
}

/// The halves and partial products of the 256-bit product stay inside 128
/// bits, and the two words are the product.
pub(super) proof fn lemma_widening_mul(a: u128, b: u128)
    ensures
        ({
            let half = 0xFFFF_FFFF_FFFF_FFFFu128;
            let (a1, a0, b1, b0) = (a >> 64u32, a & half, b >> 64u32, b & half);
            let (ll, lh, hl, hh) = (a0 * b0, a0 * b1, a1 * b0, a1 * b1);
            let middle = (ll as u128 >> 64u32) + (lh as u128 & half) + (hl as u128 & half);
            let low = (ll as u128 & half) | ((middle as u128) << 64u32);
            let high = hh + (lh as u128 >> 64u32) + (hl as u128 >> 64u32) + (middle as u128 >> 64u32);
            &&& u64::MAX as u128 == half
            &&& ll <= u128::MAX && lh <= u128::MAX && hl <= u128::MAX && hh <= u128::MAX
            &&& middle <= u128::MAX
            &&& high <= u128::MAX
            &&& high * 0x1_0000_0000_0000_0000_0000_0000_0000_0000 + low == a * b
        }),
{
    let half = 0xFFFF_FFFF_FFFF_FFFFu128;
    let base = 0x1_0000_0000_0000_0000int;
    let (a1, a0, b1, b0) = (a >> 64u32, a & half, b >> 64u32, b & half);
    assert(a1 <= half && a0 <= half && a == a1 * 0x1_0000_0000_0000_0000u128 + a0) by (bit_vector)
        requires
            a1 == a >> 64u32,
            a0 == a & half,
            half == 0xFFFF_FFFF_FFFF_FFFFu128,
    ;
    assert(b1 <= half && b0 <= half && b == b1 * 0x1_0000_0000_0000_0000u128 + b0) by (bit_vector)
        requires
            b1 == b >> 64u32,
            b0 == b & half,
            half == 0xFFFF_FFFF_FFFF_FFFFu128,
    ;
    let (ll, lh, hl, hh) = (a0 * b0, a0 * b1, a1 * b0, a1 * b1);
    let bound = 0xFFFF_FFFF_FFFF_FFFF * 0xFFFF_FFFF_FFFF_FFFFint;
    assert(0 <= ll <= bound && 0 <= lh <= bound && 0 <= hl <= bound && 0 <= hh <= bound) by (nonlinear_arith)
        requires
            a0 <= 0xFFFF_FFFF_FFFF_FFFFu128,
            a1 <= 0xFFFF_FFFF_FFFF_FFFFu128,
            b0 <= 0xFFFF_FFFF_FFFF_FFFFu128,
            b1 <= 0xFFFF_FFFF_FFFF_FFFFu128,
            ll == a0 * b0,
            lh == a0 * b1,
            hl == a1 * b0,
            hh == a1 * b1,
            bound == 0xFFFF_FFFF_FFFF_FFFF * 0xFFFF_FFFF_FFFF_FFFFint,
    ;
    assert(0xFFFF_FFFF_FFFF_FFFF * 0xFFFF_FFFF_FFFF_FFFFint == 0xFFFF_FFFF_FFFF_FFFE_0000_0000_0000_0001int) by (compute_only);
    let (llu, lhu, hlu) = (ll as u128, lh as u128, hl as u128);
    // Shifts and masks of the partial products, as quotients and remainders.
    assert(llu >> 64u32 == llu / 0x1_0000_0000_0000_0000u128 && llu & half == llu % 0x1_0000_0000_0000_0000u128)
        by (bit_vector)
        requires
            half == 0xFFFF_FFFF_FFFF_FFFFu128,
    ;
    assert(lhu >> 64u32 == lhu / 0x1_0000_0000_0000_0000u128 && lhu & half == lhu % 0x1_0000_0000_0000_0000u128)
        by (bit_vector)
        requires
            half == 0xFFFF_FFFF_FFFF_FFFFu128,
    ;
    assert(hlu >> 64u32 == hlu / 0x1_0000_0000_0000_0000u128 && hlu & half == hlu % 0x1_0000_0000_0000_0000u128)
        by (bit_vector)
        requires
            half == 0xFFFF_FFFF_FFFF_FFFFu128,
    ;
    let middle = (llu >> 64u32) + (lhu & half) + (hlu & half);
    assert(middle <= 3 * 0xFFFF_FFFF_FFFF_FFFFint);
    let mu = middle as u128;
    let lo_word = llu & half;
    let low = lo_word | (mu << 64u32);
    assert(mu >> 64u32 == mu / 0x1_0000_0000_0000_0000u128 && low == lo_word + (mu % 0x1_0000_0000_0000_0000u128)
        * 0x1_0000_0000_0000_0000u128) by (bit_vector)
        requires
            low == lo_word | (mu << 64u32),
            lo_word <= 0xFFFF_FFFF_FFFF_FFFFu128,
            mu <= 0x2_FFFF_FFFF_FFFF_FFFDu128,
    ;
    let high = hh + (lhu >> 64u32) + (hlu >> 64u32) + (mu >> 64u32);
    // The product, expanded in base 2^64.
    assert(a * b == hh * (base * base) + (lh + hl) * base + ll) by (nonlinear_arith)
        requires
            a == a1 * base + a0,
            b == b1 * base + b0,
            ll == a0 * b0,
            lh == a0 * b1,
            hl == a1 * b0,
            hh == a1 * b1,
    ;
    assert(0x1_0000_0000_0000_0000int * 0x1_0000_0000_0000_0000int == 0x1_0000_0000_0000_0000_0000_0000_0000_0000int)
        by (compute_only);
    assert(high * (base * base) + low == hh * (base * base) + (lh + hl) * base + ll);
    assert(a * b <= 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFFint * 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFFint)
        by (nonlinear_arith)
        requires
            a <= 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFFu128,
            b <= 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFFu128,
    ;
    assert(high <= u128::MAX) by (nonlinear_arith)
        requires
            high * 0x1_0000_0000_0000_0000_0000_0000_0000_0000int + low <= 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFFint
                * 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFFint,
            low >= 0,
            high >= 0,
    ;
}

/// Digit counts decide the order of `x 10^s` and `y` when they differ, and
/// when they agree the shift is at most 38.
pub(super) proof fn lemma_compare_shifted(x: u128, shift: u32, y: u128)
    requires
        x != 0,
        y != 0,
    ensures
        ({
            let (shifted_digits, digits) = (digit_len(x as nat) + shift, digit_len(y as nat));
            &&& digits_of(x) == digit_len(x as nat) && digits_of(y) == digits
            &&& 1 <= digit_len(x as nat) <= 39 && 1 <= digits <= 39
            &&& shifted_digits < digits ==> x * pow10(shift as nat) < y
            &&& shifted_digits > digits ==> x * pow10(shift as nat) > y
            &&& shifted_digits == digits ==> shift <= 38 && 1 <= pow10(shift as nat) <= i128::MAX
        }),
{
    let (dx, dy, s) = (digit_len(x as nat), digit_len(y as nat), shift as nat);
    lemma_digits_of(x);
    lemma_digits_of(y);
    lemma_digit_len_range(x as nat);
    lemma_digit_len_range(y as nat);
    lemma_pow10_pos(s);
    lemma_pow10_adds(dx, s);
    lemma_pow10_adds((dx - 1) as nat, s);
    let p = pow10(s);
    assert(pow10((dx - 1) as nat) * p <= x * p < pow10(dx) * p) by (nonlinear_arith)
        requires
            pow10((dx - 1) as nat) <= x < pow10(dx),
            p >= 1,
    ;
    if dx + s < dy {
        lemma_pow10_monotone(dx + s, (dy - 1) as nat);
    }
    if dx + s > dy {
        lemma_pow10_monotone(dy, (dx - 1 + s) as nat);
    }
    if dx + s == dy {
        lemma_pow10_monotone(s, 38);
        lemma_pow10_bound(38);
    }
}

/// The order of the magnitudes at equal scales and after shifting the one of
/// the smaller scale.
pub(super) proof fn lemma_compare_magnitudes(left: NativeDecimal, right: NativeDecimal)
    requires
        left.coefficient != 0,
        right.coefficient != 0,
    ensures
        ({
            let (x, y) = (left.coefficient.unsigned_abs(), right.coefficient.unsigned_abs());
            &&& x != 0 && y != 0
            &&& left.scale == right.scale ==> magnitude_order(left, right) == cmp_int(x as int, y as int)
            &&& left.scale > right.scale ==> magnitude_order(left, right) == crate::verus_std_specs::ordering_reverse(
                cmp_int(y * pow10((left.scale - right.scale) as nat), x as int),
            )
            &&& left.scale < right.scale ==> magnitude_order(left, right) == cmp_int(
                x * pow10((right.scale - left.scale) as nat),
                y as int,
            )
        }),
{
    let (x, y) = (left.coefficient.unsigned_abs() as int, right.coefficient.unsigned_abs() as int);
    let (sl, sr) = (left.scale as nat, right.scale as nat);
    lemma_pow10_pos(sl);
    lemma_pow10_pos(sr);
    if sl == sr {
        lemma_order_scaled(x, y, pow10(sl) as int);
    } else if sl > sr {
        let k = (sl - sr) as nat;
        lemma_pow10_adds(k, sr);
        let (pk, ps) = (pow10(k) as int, pow10(sr) as int);
        assert(y * pow10(sl) == (y * pk) * ps) by (nonlinear_arith)
            requires
                pow10(sl) == pk * ps,
        ;
        lemma_order_scaled(x, y * pk, ps);
    } else {
        let k = (sr - sl) as nat;
        lemma_pow10_adds(k, sl);
        let (pk, ps) = (pow10(k) as int, pow10(sl) as int);
        assert(x * pow10(sr) == (x * pk) * ps) by (nonlinear_arith)
            requires
                pow10(sr) == pk * ps,
        ;
        lemma_order_scaled(x * pk, y, ps);
    }
}

/// A positive factor keeps the order.
pub(super) proof fn lemma_order_scaled(a: int, b: int, p: int)
    requires
        p > 0,
    ensures
        cmp_int(a * p, b * p) == cmp_int(a, b),
{
    assert((a < b ==> a * p < b * p) && (a == b ==> a * p == b * p) && (a > b ==> a * p > b * p)) by (nonlinear_arith)
        requires
            p > 0,
    ;
}

/// The order by value from the signs and the order of the magnitudes.
pub(super) proof fn lemma_order_values(left: NativeDecimal, right: NativeDecimal)
    ensures
        ({
            let (cl, cr) = (left.coefficient as int, right.coefficient as int);
            let sign = |c: int| if c > 0 { 1int } else if c == 0 { 0int } else { -1int };
            &&& sign(cl) != sign(cr) ==> value_order(left, right) == cmp_int(sign(cl), sign(cr))
            &&& cl == 0 && cr == 0 ==> value_order(left, right) == Ordering::Equal
            &&& cl > 0 && cr > 0 ==> value_order(left, right) == magnitude_order(left, right)
            &&& cl < 0 && cr < 0 ==> value_order(left, right) == crate::verus_std_specs::ordering_reverse(
                magnitude_order(left, right),
            )
        }),
{
    let (cl, cr) = (left.coefficient as int, right.coefficient as int);
    let (pl, pr) = (pow10(left.scale as nat) as int, pow10(right.scale as nat) as int);
    lemma_pow10_pos(left.scale as nat);
    lemma_pow10_pos(right.scale as nat);
    // The sign of a coefficient times a positive power is the sign of the
    // coefficient.
    assert((cl > 0 ==> cl * pr > 0) && (cl == 0 ==> cl * pr == 0) && (cl < 0 ==> cl * pr < 0)) by (nonlinear_arith)
        requires
            pr > 0,
    ;
    assert((cr > 0 ==> cr * pl > 0) && (cr == 0 ==> cr * pl == 0) && (cr < 0 ==> cr * pl < 0)) by (nonlinear_arith)
        requires
            pl > 0,
    ;
    assert((-cl) * pr == -(cl * pr) && (-cr) * pl == -(cr * pl)) by (nonlinear_arith);
}

} // verus!
