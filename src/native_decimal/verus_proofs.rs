// SPDX-License-Identifier: AGPL-3.0-or-later
//! Verus proofs of the native Decimal constructor and normalized view over
//! the full width (FR-402 Behavior 11, IR-1554). Compiled only by a Verus run
//! (`cfg(verus_keep_ghost)`).

use vstd::arithmetic::div_mod::*;
use vstd::prelude::*;

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

} // verus!
