// SPDX-License-Identifier: AGPL-3.0-or-later
//! Verus proofs of the native Decimal constructor and normalized view over
//! the full width (FR-402 Behavior 11, IR-1554). Compiled only by a Verus run
//! (`cfg(verus_keep_ghost)`).

use vstd::arithmetic::div_mod::*;
use vstd::arithmetic::mul::*;
use vstd::prelude::*;

use super::{NativeDecimal, NativeDecimalRefusal};

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

} // verus!
