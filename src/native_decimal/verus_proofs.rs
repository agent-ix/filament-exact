// SPDX-License-Identifier: AGPL-3.0-or-later
//! Verus proofs of the native Decimal constructor and normalized view over
//! the full width (FR-402 Behavior 11, IR-1554). Compiled only by a Verus run
//! (`cfg(verus_keep_ghost)`).

use vstd::arithmetic::div_mod::*;
use vstd::arithmetic::mul::*;
use vstd::arithmetic::power2::*;
use vstd::prelude::*;

use super::{NativeDecimal, NativeDecimalRefusal, NativeDecimalType};
use crate::native_float::verus_proofs::lemma_bit_len_range;
use crate::native_rational::verus_proofs::{bit_len, lemma_leading_zeros};

verus! {

/// `10^n`.
pub open spec fn pow10(n: nat) -> nat
    decreases n,
{
    if n == 0 {
        1
    } else {
        10 * pow10((n - 1) as nat)
    }
}

/// Every entry of the table is its power of ten.
pub open spec fn powers_table(table: [i128; 39]) -> bool {
    forall|i: int| 0 <= i < 39 ==> #[trigger] table[i] == pow10(i as nat)
}

/// `10^n` is positive.
pub proof fn lemma_pow10_pos(n: nat)
    ensures
        pow10(n) >= 1,
    decreases n,
{
    if n > 0 {
        lemma_pow10_pos((n - 1) as nat);
    }
}

/// `10^n` grows with `n`.
pub proof fn lemma_pow10_monotone(a: nat, b: nat)
    requires
        a <= b,
    ensures
        pow10(a) <= pow10(b),
    decreases b,
{
    if a < b {
        lemma_pow10_monotone(a, (b - 1) as nat);
        lemma_pow10_pos((b - 1) as nat);
    }
}

/// Up to `10^38` the powers of ten fit signed 128 bits.
pub proof fn lemma_pow10_bound(n: nat)
    ensures
        pow10(n) >= 1,
        n <= 38 ==> pow10(n) <= 100000000000000000000000000000000000000,
{
    lemma_pow10_pos(n);
    if n <= 38 {
        lemma_pow10_values();
        lemma_pow10_monotone(n, 38);
    }
}

/// `10^(a + b) = 10^a 10^b`.
pub proof fn lemma_pow10_adds(a: nat, b: nat)
    ensures
        pow10(a + b) == pow10(a) * pow10(b),
    decreases b,
{
    if b == 0 {
        assert(pow10(a) * 1 == pow10(a)) by (nonlinear_arith);
    } else {
        lemma_pow10_adds(a, (b - 1) as nat);
        assert(pow10(a) * (10 * pow10((b - 1) as nat)) == 10 * (pow10(a) * pow10((b - 1) as nat)))
            by (nonlinear_arith);
    }
}

pub proof fn lemma_pow10_values()
    ensures
        pow10(38) == 100000000000000000000000000000000000000,
{
    assert(pow10(38) == 100000000000000000000000000000000000000) by (compute_only);
}

/// Specification of the production `strip_zeros`.
pub open spec fn strip_zeros(coefficient: i128, scale: u32, places: u32, power: i128) -> (i128, u32) {
    if scale >= places && rust_rem(coefficient as int, power as int) == 0 {
        (rust_div(coefficient as int, power as int) as i128, (scale - places) as u32)
    } else {
        (coefficient, scale)
    }
}

#[verifier::external_type_specification]
pub struct ExNativeDecimalRefusal(NativeDecimalRefusal);

/// Specification of the production `normalized_pair`.
pub open spec fn normalized(coefficient: i128, scale: u32) -> (i128, u32) {
    if coefficient == 0 {
        (0, 0)
    } else {
        let (c, s) = strip_zeros(coefficient, scale, 32, pow10(32) as i128);
        let (c, s) = strip_zeros(c, s, 16, pow10(16) as i128);
        let (c, s) = strip_zeros(c, s, 8, pow10(8) as i128);
        let (c, s) = strip_zeros(c, s, 4, pow10(4) as i128);
        let (c, s) = strip_zeros(c, s, 2, pow10(2) as i128);
        strip_zeros(c, s, 1, pow10(1) as i128)
    }
}

/// Specification of the production `admits_normalized`.
pub open spec fn admits_normalized(bounds: (i128, i128, u32, u32), pair: (i128, u32)) -> bool {
    let (lower, upper, min_scale, max_scale) = bounds;
    let (coefficient, scale) = pair;
    let lifted_scale = if scale > min_scale {
        scale
    } else {
        min_scale
    };
    if lifted_scale > max_scale {
        false
    } else if coefficient == 0 {
        lower <= 0 && 0 <= upper
    } else {
        let shift = lifted_scale - scale;
        if shift > 38 {
            false
        } else {
            let lifted = coefficient * pow10(shift as nat);
            i128::MIN <= lifted <= i128::MAX && lower <= lifted <= upper
        }
    }
}

/// Specification of the production `admits`.
pub open spec fn admits(bounds: (i128, i128, u32, u32), retained: (i128, u32)) -> bool {
    admits_normalized(bounds, normalized(retained.0, retained.1))
}

/// What the production `from_bindings` returns: `OutsideCarrierDomain` for a
/// scale outside `u32` or a binding the type does not admit, and otherwise
/// the pair drawn, retained unchanged.
pub open spec fn from_bindings_post(
    bounds: (i128, i128, u32, u32),
    coefficient: i64,
    scale: i64,
    r: Result<NativeDecimal, NativeDecimalRefusal>,
) -> bool {
    if 0 <= scale <= u32::MAX && admits(bounds, (coefficient as i128, scale as u32)) {
        r is Ok && r->Ok_0.retained() == (coefficient as i128, scale as u32)
    } else {
        r == Err::<NativeDecimal, _>(NativeDecimalRefusal::OutsideCarrierDomain)
    }
}

/// Every power of ten in the table is positive and fits signed 128 bits.
pub proof fn lemma_powers_in_range()
    ensures
        forall|n: nat| n <= 38 ==> 1 <= #[trigger] pow10(n) <= 100000000000000000000000000000000000000,
{
    assert forall|n: nat| n <= 38 implies 1 <= #[trigger] pow10(n)
        <= 100000000000000000000000000000000000000 by {
        lemma_pow10_bound(n);
    }
}

// ---- trailing zeros -------------------------------------------------------------------

/// The magnitude of a coefficient.
pub open spec fn magnitude(c: i128) -> nat {
    if c < 0 {
        (-(c as int)) as nat
    } else {
        c as nat
    }
}

/// How many trailing decimal zeros a nonzero magnitude `m` can give up while
/// the scale `s` stays at or above zero.
pub open spec fn zeros(m: nat, s: nat) -> nat
    decreases s,
{
    if m != 0 && s > 0 && m % 10 == 0 {
        1 + zeros(m / 10, (s - 1) as nat)
    } else {
        0
    }
}

/// `p` zeros can be removed exactly when the scale allows `p` and `10^p`
/// divides the magnitude.
pub proof fn lemma_zeros_at_least(m: nat, s: nat, p: nat)
    requires
        m != 0,
    ensures
        (zeros(m, s) >= p) == (s >= p && m % pow10(p) == 0),
    decreases p,
{
    if p == 0 {
        assert(m % 1 == 0);
    } else {
        let rest = pow10((p - 1) as nat);
        lemma_pow10_pos((p - 1) as nat);
        lemma_mod_breakdown(m as int, 10, rest as int);
        assert(m % (10 * rest) == 10 * ((m / 10) % rest) + m % 10);
        lemma_mod_bound((m / 10) as int, rest as int);
        if s > 0 && m % 10 == 0 {
            lemma_fundamental_div_mod(m as int, 10);
            assert(m / 10 != 0);
            lemma_zeros_at_least(m / 10, (s - 1) as nat, (p - 1) as nat);
        }
    }
}

/// Removing `p` of the removable zeros leaves the rest removable.
pub proof fn lemma_zeros_after(m: nat, s: nat, p: nat)
    requires
        m != 0,
        zeros(m, s) >= p,
    ensures
        s >= p,
        m / pow10(p) != 0,
        m == (m / pow10(p)) * pow10(p),
        zeros(m / pow10(p), (s - p) as nat) == zeros(m, s) - p,
    decreases p,
{
    lemma_zeros_at_least(m, s, p);
    lemma_pow10_pos(p);
    lemma_fundamental_div_mod(m as int, pow10(p) as int);
    lemma_mul_is_commutative((m / pow10(p)) as int, pow10(p) as int);
    if m / pow10(p) == 0 {
        assert(pow10(p) * (m / pow10(p)) == 0) by (nonlinear_arith)
            requires
                m / pow10(p) == 0,
        ;
    }
    if p > 0 {
        let rest = pow10((p - 1) as nat);
        lemma_pow10_pos((p - 1) as nat);
        lemma_fundamental_div_mod(m as int, 10);
        assert(zeros(m, s) == 1 + zeros(m / 10, (s - 1) as nat));
        assert(m / 10 != 0);
        lemma_zeros_after(m / 10, (s - 1) as nat, (p - 1) as nat);
        lemma_div_denominator(m as int, 10, rest as int);
        assert(pow10(p) == 10 * rest);
        assert((m / 10) / rest == m / pow10(p));
        assert(((s - 1) as nat - (p - 1) as nat) as nat == (s - p) as nat);
        assert(zeros(m / pow10(p), (s - p) as nat) == zeros(m / 10, (s - 1) as nat) - (p - 1));
    } else {
        assert(pow10(0) == 1);
        assert(m / pow10(0) == m) by (nonlinear_arith)
            requires
                pow10(0) == 1,
        ;
        assert(zeros(m / pow10(p), (s - p) as nat) == zeros(m, s));
    }
}

/// A nonzero magnitude below `2^127` has at most 38 trailing zeros.
pub proof fn lemma_zeros_bound(m: nat, s: nat)
    requires
        0 < m <= 0x8000_0000_0000_0000_0000_0000_0000_0000,
    ensures
        zeros(m, s) <= 38,
{
    if zeros(m, s) >= 39 {
        lemma_zeros_at_least(m, s, 39);
        lemma_pow10_values();
        assert(pow10(39) == 10 * pow10(38));
        lemma_mod_is_zero(m, pow10(39));
    }
}

// ---- the normalized view -----------------------------------------------------------------

/// One stage of the normalization on a magnitude: remove `p` zeros when the
/// scale allows and `10^p` divides.
pub open spec fn stage(m: nat, s: nat, p: nat) -> (nat, nat) {
    if s >= p && m % pow10(p) == 0 {
        (m / pow10(p), (s - p) as nat)
    } else {
        (m, s)
    }
}

/// `(m, s)` is `(m0, s0)` with some of its removable zeros removed.
pub open spec fn stripped_from(m0: nat, s0: nat, m: nat, s: nat) -> bool {
    &&& m != 0
    &&& s <= s0
    &&& m0 == m * pow10((s0 - s) as nat)
    &&& zeros(m, s) + (s0 - s) == zeros(m0, s0)
}

/// A stage keeps the value and halves the bound on what is left to remove.
pub proof fn lemma_stage(m0: nat, s0: nat, m: nat, s: nat, p: nat)
    requires
        stripped_from(m0, s0, m, s),
        zeros(m, s) < 2 * p,
    ensures
        stripped_from(m0, s0, stage(m, s, p).0, stage(m, s, p).1),
        zeros(stage(m, s, p).0, stage(m, s, p).1) < p,
{
    lemma_zeros_at_least(m, s, p);
    if zeros(m, s) >= p {
        lemma_zeros_after(m, s, p);
        let m2 = m / pow10(p);
        let s2 = (s - p) as nat;
        lemma_pow10_adds(p, (s0 - s) as nat);
        assert(m0 == m2 * pow10((s0 - s2) as nat)) by (nonlinear_arith)
            requires
                m0 == m * pow10((s0 - s) as nat),
                m == m2 * pow10(p),
                pow10((s0 - s2) as nat) == pow10(p) * pow10((s0 - s) as nat),
        ;
    }
}

/// The production `strip_zeros` is `stage` on the magnitude and keeps the sign.
pub proof fn lemma_strip_zeros(c: i128, s: u32, p: u32)
    requires
        c != 0,
        p <= 38,
    ensures
        ({
            let r = strip_zeros(c, s, p, pow10(p as nat) as i128);
            &&& (magnitude(r.0), r.1 as nat) == stage(magnitude(c), s as nat, p as nat)
            &&& (r.0 < 0) == (c < 0) || r.0 == 0
        }),
{
    lemma_pow10_bound(p as nat);
    let power = pow10(p as nat) as int;
    let m = magnitude(c) as int;
    lemma_fundamental_div_mod(m, power);
    lemma_mod_bound(m, power);
    let q = m / power;
    assert(0 <= q <= m) by (nonlinear_arith)
        requires
            m == power * q + m % power,
            0 <= m % power,
            power >= 1,
            m >= 0,
    ;
}

/// The normalized view is the least-scale pair equal in value: zero is
/// `(0, 0)`; otherwise the sign is kept, the magnitude is the coefficient's
/// with `k` trailing zeros removed and the scale is lowered by `k`, where `k`
/// is every zero the scale allows, so either the scale reached zero or the
/// coefficient is no longer divisible by ten.
pub proof fn lemma_normalized(c: i128, s: u32)
    ensures
        c == 0 ==> normalized(c, s) == (0i128, 0u32),
        c != 0 ==> ({
            let (c2, s2) = normalized(c, s);
            let k = (s - s2) as nat;
            &&& s2 <= s
            &&& c2 != 0
            &&& (c2 < 0) == (c < 0)
            &&& magnitude(c) == magnitude(c2) * pow10(k)
            &&& k == zeros(magnitude(c), s as nat)
            &&& s2 == 0 || magnitude(c2) % 10 != 0
        }),
{
    if c != 0 {
        let (m0, s0) = (magnitude(c), s as nat);
        lemma_zeros_bound(m0, s0);
        assert(m0 == m0 * pow10(0)) by (nonlinear_arith)
            requires
                pow10(0) == 1,
        ;
        let (c1, s1) = strip_zeros(c, s, 32, pow10(32) as i128);
        lemma_strip_zeros(c, s, 32);
        lemma_stage(m0, s0, m0, s0, 32);
        let (c2, s2) = strip_zeros(c1, s1, 16, pow10(16) as i128);
        lemma_strip_zeros(c1, s1, 16);
        lemma_stage(m0, s0, magnitude(c1), s1 as nat, 16);
        let (c3, s3) = strip_zeros(c2, s2, 8, pow10(8) as i128);
        lemma_strip_zeros(c2, s2, 8);
        lemma_stage(m0, s0, magnitude(c2), s2 as nat, 8);
        let (c4, s4) = strip_zeros(c3, s3, 4, pow10(4) as i128);
        lemma_strip_zeros(c3, s3, 4);
        lemma_stage(m0, s0, magnitude(c3), s3 as nat, 4);
        let (c5, s5) = strip_zeros(c4, s4, 2, pow10(2) as i128);
        lemma_strip_zeros(c4, s4, 2);
        lemma_stage(m0, s0, magnitude(c4), s4 as nat, 2);
        let (c6, s6) = strip_zeros(c5, s5, 1, pow10(1) as i128);
        lemma_strip_zeros(c5, s5, 1);
        lemma_stage(m0, s0, magnitude(c5), s5 as nat, 1);
        assert(zeros(magnitude(c6), s6 as nat) == 0);
    }
}

// ---- membership and the constructor ---------------------------------------------------------

/// The owner's membership rule on a normalized pair, in exact arithmetic
/// (FR-402 Behavior 5): with `s*` the larger of the scale and `min_scale`, the
/// pair is a member exactly when `s* <= max_scale` and
/// `lower <= coefficient 10^(s* - scale) <= upper`.
pub open spec fn member(bounds: (i128, i128, u32, u32), pair: (i128, u32)) -> bool {
    let (lower, upper, min_scale, max_scale) = bounds;
    let (coefficient, scale) = pair;
    let lifted_scale = if scale > min_scale {
        scale
    } else {
        min_scale
    };
    lifted_scale <= max_scale && lower <= coefficient * pow10((lifted_scale - scale) as nat) <= upper
}

/// The production membership test is the owner's rule: a lift that leaves
/// signed 128 bits is outside every pair of native bounds.
pub proof fn lemma_admits_normalized(bounds: (i128, i128, u32, u32), pair: (i128, u32))
    ensures
        admits_normalized(bounds, pair) == member(bounds, pair),
{
    let (lower, upper, min_scale, max_scale) = bounds;
    let (coefficient, scale) = pair;
    let lifted_scale = if scale > min_scale {
        scale
    } else {
        min_scale
    };
    if lifted_scale <= max_scale {
        let shift = (lifted_scale - scale) as nat;
        let power = pow10(shift);
        lemma_pow10_pos(shift);
        if coefficient == 0 {
            assert(coefficient * power == 0) by (nonlinear_arith)
                requires
                    coefficient == 0,
            ;
        } else if shift > 38 {
            lemma_pow10_values();
            lemma_pow10_monotone(39, shift);
            assert(pow10(39) == 10 * pow10(38));
            assert(coefficient * power >= 1000000000000000000000000000000000000000 || coefficient * power
                <= -1000000000000000000000000000000000000000) by (nonlinear_arith)
                requires
                    coefficient != 0,
                    power >= 1000000000000000000000000000000000000000,
            ;
        }
    }
}

/// The constructor, for every `i64` coefficient, every `i64` scale and every
/// native type: a scale outside `u32`, or a binding whose normalized pair is
/// not a member by the owner's rule, is `OutsideCarrierDomain`; a member is
/// retained exactly as drawn. `lemma_normalized` says what the normalized pair
/// is.
pub fn row_from_bindings(ty: &NativeDecimalType, coefficient: i64, scale: i64) -> (r: Result<
    NativeDecimal,
    NativeDecimalRefusal,
>)
    ensures
        0 <= scale <= u32::MAX && member(ty.bounds(), normalized(coefficient as i128, scale as u32))
            ==> r is Ok && r->Ok_0.retained() == (coefficient as i128, scale as u32),
        !(0 <= scale <= u32::MAX && member(
            ty.bounds(),
            normalized(coefficient as i128, scale as u32),
        )) ==> r == Err::<NativeDecimal, _>(NativeDecimalRefusal::OutsideCarrierDomain),
{
    proof {
        if 0 <= scale <= u32::MAX {
            lemma_admits_normalized(ty.bounds(), normalized(coefficient as i128, scale as u32));
        }
    }
    NativeDecimal::from_bindings(ty, coefficient, scale)
}

/// The normalized view of a carried Decimal, for every signed 128-bit
/// coefficient and every `u32` scale, is the pair `lemma_normalized`
/// describes.
pub fn row_normalized(value: &NativeDecimal) -> (r: (i128, u32))
    ensures
        r == normalized(value.retained().0, value.retained().1),
        value.retained().0 == 0 ==> r == (0i128, 0u32),
        value.retained().0 != 0 ==> ({
            let (c, s) = value.retained();
            let k = (s - r.1) as nat;
            &&& r.1 <= s
            &&& (r.0 < 0) == (c < 0)
            &&& magnitude(c) == magnitude(r.0) * pow10(k)
            &&& r.1 == 0 || magnitude(r.0) % 10 != 0
        }),
{
    proof {
        lemma_normalized(value.retained().0, value.retained().1);
    }
    value.normalized()
}

// ---- sizes: bit and digit lengths (FR-403, IR-1555) --------------------------------------------

/// Specification of the production `bits_of`: the bit length, at least 1.
pub open spec fn bits_of(magnitude: u128) -> u64 {
    bit_len(magnitude as nat) as u64
}

pub proof fn lemma_bits_of(magnitude: u128)
    ensures
        magnitude != 0 ==> 128 - magnitude.leading_zeros() == bit_len(magnitude as nat),
        magnitude.leading_zeros() <= 128,
        magnitude == 0 ==> magnitude.leading_zeros() == 128,
        1 <= bit_len(magnitude as nat) <= 128,
{
    lemma_leading_zeros(magnitude);
}

/// The number of decimal digits of a magnitude, at least 1.
pub open spec fn digit_len(m: nat) -> nat
    decreases m,
{
    if m < 10 {
        1
    } else {
        1 + digit_len(m / 10)
    }
}

/// Specification of the production `digits_of`.
pub open spec fn digits_of(magnitude: u128) -> u64 {
    digit_len(magnitude as nat) as u64
}

/// A positive value lies between the powers of ten its digit count names.
pub proof fn lemma_digit_len_range(m: nat)
    requires
        m >= 1,
    ensures
        pow10((digit_len(m) - 1) as nat) <= m < pow10(digit_len(m)),
    decreases m,
{
    if m >= 10 {
        lemma_digit_len_range(m / 10);
    }
}

/// A value between `10^(d-1)` and `10^d` has `d` digits.
pub proof fn lemma_digit_len_unique(m: nat, d: nat)
    requires
        d >= 1,
        pow10((d - 1) as nat) <= m < pow10(d),
    ensures
        digit_len(m) == d,
{
    lemma_pow10_pos((d - 1) as nat);
    lemma_digit_len_range(m);
    let e = digit_len(m);
    if e < d {
        lemma_pow10_monotone(e, (d - 1) as nat);
    }
    if e > d {
        lemma_pow10_monotone(d, (e - 1) as nat);
    }
}

/// The digit estimate `floor(bits x 1233 / 4096)` is within one of the digit
/// count: `2^bits <= 10^(e + 1)` and `10^(e - 1) <= 2^(bits - 1)`.
pub open spec fn estimate_ok(bits: nat) -> bool {
    let e = (bits * 1233) / 4096;
    two_pow(bits) <= pow10(e + 1) && (e == 0 || pow10((e - 1) as nat) <= two_pow((bits - 1) as nat))
}

/// `2^n`, written so that it can be evaluated.
pub open spec fn two_pow(n: nat) -> nat
    decreases n,
{
    if n == 0 {
        1
    } else {
        2 * two_pow((n - 1) as nat)
    }
}

pub proof fn lemma_two_pow(n: nat)
    ensures
        two_pow(n) == pow2(n),
    decreases n,
{
    if n == 0 {
        lemma2_to64();
    } else {
        lemma_two_pow((n - 1) as nat);
        lemma_pow2_unfold(n);
    }
}

pub open spec fn estimates_ok(upto: nat) -> bool
    decreases upto,
{
    upto == 0 || (estimate_ok(upto) && estimates_ok((upto - 1) as nat))
}

pub proof fn lemma_estimates(upto: nat, bits: nat)
    requires
        estimates_ok(upto),
        1 <= bits <= upto,
    ensures
        estimate_ok(bits),
    decreases upto,
{
    if bits < upto {
        lemma_estimates((upto - 1) as nat, bits);
    }
}

/// The production `digits_of` counts decimal digits for every `u128`.
pub proof fn lemma_digits_of(magnitude: u128)
    ensures
        ({
            let bits = (128 - magnitude.leading_zeros()) as usize;
            let estimate = ((bits * 1233) as usize) >> 12usize;
            &&& magnitude.leading_zeros() <= 128
            &&& estimate <= 38
            &&& magnitude != 0 ==> digit_len(magnitude as nat) == estimate + 1 - (if magnitude < pow10(
                estimate as nat,
            ) { 1int } else { 0int })
        }),
        1 <= digit_len(magnitude as nat) <= 39,
{
    lemma_leading_zeros(magnitude);
    let bits = (128 - magnitude.leading_zeros()) as usize;
    vstd::bits::lemma_usize_shr_is_div((bits * 1233) as usize, 12usize);
    lemma2_to64();
    let estimate = (bits * 1233) / 4096;
    if magnitude != 0 {
        let m = magnitude as nat;
        lemma_bit_len_range(m);
        assert(estimates_ok(128)) by (compute_only);
        lemma_estimates(128, bits as nat);
        lemma_two_pow(bits as nat);
        lemma_two_pow((bits - 1) as nat);
        lemma_pow10_pos(estimate as nat);
        if m < pow10(estimate as nat) {
            lemma_digit_len_unique(m, estimate as nat);
        } else {
            lemma_digit_len_unique(m, (estimate + 1) as nat);
        }
    }
}

} // verus!
