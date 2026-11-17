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

/// `i128::wrapping_rem` for a nonzero divisor: the truncating remainder, and
/// zero for `i128::MIN % -1`.
pub open spec fn i128_wrapping_rem(x: i128, y: i128) -> i128 {
    if x == i128::MIN && y == -1 {
        0
    } else {
        vstd::arithmetic::div_mod::rust_rem(x as int, y as int) as i128
    }
}

#[verifier::when_used_as_spec(i128_wrapping_rem)]
pub assume_specification[ i128::wrapping_rem ](x: i128, y: i128) -> (r: i128)
    requires
        y != 0,
    ensures
        r == i128_wrapping_rem(x, y),
;

pub assume_specification[ <u64 as core::convert::From<bool>>::from ](b: bool) -> (r: u64)
    ensures
        r == (if b { 1u64 } else { 0u64 }),
;

/// `bool::then_some`: the value when the condition holds.
pub assume_specification<T>[ bool::then_some ](b: bool, t: T) -> (r: Option<T>)
    ensures
        r == (if b { Some(t) } else { None::<T> }),
;

/// `i128::checked_neg`: every value but the least has a negation.
pub assume_specification[ i128::checked_neg ](x: i128) -> (r: Option<i128>)
    ensures
        r == (if x == i128::MIN { None::<i128> } else { Some((-x) as i128) }),
;

/// `i128::div_euclid` for a positive divisor: the floor quotient.
pub assume_specification[ i128::div_euclid ](x: i128, y: i128) -> (r: i128)
    requires
        y > 0,
    ensures
        r == (x as int) / (y as int),
;

/// `i128::rem_euclid` for a positive divisor: the nonnegative remainder.
pub assume_specification[ i128::rem_euclid ](x: i128, y: i128) -> (r: i128)
    requires
        y > 0,
    ensures
        r == (x as int) % (y as int),
;

/// `i128::signum`.
pub assume_specification[ i128::signum ](x: i128) -> (r: i128)
    ensures
        r == (if x > 0 { 1i128 } else if x == 0 { 0i128 } else { -1i128 }),
;

/// `Ordering::reverse`.
pub open spec fn ordering_reverse(o: core::cmp::Ordering) -> core::cmp::Ordering {
    match o {
        core::cmp::Ordering::Less => core::cmp::Ordering::Greater,
        core::cmp::Ordering::Equal => core::cmp::Ordering::Equal,
        core::cmp::Ordering::Greater => core::cmp::Ordering::Less,
    }
}

#[verifier::when_used_as_spec(ordering_reverse)]
pub assume_specification[ core::cmp::Ordering::reverse ](o: core::cmp::Ordering) -> (r: core::cmp::Ordering)
    ensures
        r == ordering_reverse(o),
;

/// `u128::trailing_zeros`, by the recursion the prover's library uses for the
/// narrower widths: 128 for zero, 0 for an odd value, otherwise one more than
/// for half the value.
pub open spec fn u128_trailing_zeros(i: u128) -> u32
    decreases i,
{
    if i == 0 {
        128
    } else if i % 2 == 1 {
        0
    } else {
        (1 + u128_trailing_zeros(i / 2)) as u32
    }
}

#[verifier::when_used_as_spec(u128_trailing_zeros)]
pub assume_specification[ u128::trailing_zeros ](i: u128) -> (r: u32)
    ensures
        r == u128_trailing_zeros(i),
;

} // verus!
