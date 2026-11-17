// SPDX-License-Identifier: AGPL-3.0-or-later
//! The trusted base of the Verus proofs: specifications of `core` functions
//! that the prover's library does not specify. Each is an assumption about the
//! standard library, stated once here and proved nowhere. The file is compiled
//! only by a Verus run (`cfg(verus_keep_ghost)`).
#![allow(unsafe_code)]

use vstd::prelude::*;

verus! {

/// `i64::unsigned_abs`: the magnitude, which always fits `u64`.
pub open spec fn i64_unsigned_abs(x: i64) -> u64 {
    if x < 0 {
        (-(x as int)) as u64
    } else {
        x as u64
    }
}

#[verifier::when_used_as_spec(i64_unsigned_abs)]
pub assume_specification[ i64::unsigned_abs ](x: i64) -> (r: u64)
    ensures
        r == i64_unsigned_abs(x),
;

/// `i128::unsigned_abs`: the magnitude, which always fits `u128`.
pub open spec fn i128_unsigned_abs(x: i128) -> u128 {
    if x < 0 {
        (-(x as int)) as u128
    } else {
        x as u128
    }
}

#[verifier::when_used_as_spec(i128_unsigned_abs)]
pub assume_specification[ i128::unsigned_abs ](x: i128) -> (r: u128)
    ensures
        r == i128_unsigned_abs(x),
;

/// `i128::wrapping_neg`: the negation, with `i128::MIN` mapped to itself.
pub open spec fn i128_wrapping_neg(x: i128) -> i128 {
    if x == i128::MIN {
        i128::MIN
    } else {
        (-(x as int)) as i128
    }
}

#[verifier::when_used_as_spec(i128_wrapping_neg)]
pub assume_specification[ i128::wrapping_neg ](x: i128) -> (r: i128)
    ensures
        r == i128_wrapping_neg(x),
;

/// `u128::leading_zeros`, by the recursion the prover's library uses for the
/// narrower widths: 128 for zero, otherwise one fewer than for half the value.
pub open spec fn u128_leading_zeros(i: u128) -> u32
    decreases i,
{
    if i == 0 {
        128
    } else {
        (u128_leading_zeros(i / 2) - 1) as u32
    }
}

#[verifier::when_used_as_spec(u128_leading_zeros)]
pub assume_specification[ u128::leading_zeros ](i: u128) -> (r: u32)
    ensures
        r == u128_leading_zeros(i),
;

pub assume_specification[ core::cmp::Ordering::is_lt ](o: core::cmp::Ordering) -> (r: bool)
    ensures
        r == (o == core::cmp::Ordering::Less),
;

pub assume_specification[ core::cmp::Ordering::is_le ](o: core::cmp::Ordering) -> (r: bool)
    ensures
        r == (o != core::cmp::Ordering::Greater),
;

pub assume_specification[ core::cmp::Ordering::is_gt ](o: core::cmp::Ordering) -> (r: bool)
    ensures
        r == (o == core::cmp::Ordering::Greater),
;

pub assume_specification[ core::cmp::Ordering::is_ge ](o: core::cmp::Ordering) -> (r: bool)
    ensures
        r == (o != core::cmp::Ordering::Less),
;

} // verus!
