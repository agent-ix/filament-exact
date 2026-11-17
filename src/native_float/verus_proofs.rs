// SPDX-License-Identifier: AGPL-3.0-or-later
//! Verus proofs of the native float arithmetic (FR-405, IR-1551). Compiled
//! only by a Verus run (`cfg(verus_keep_ghost)`).

use vstd::arithmetic::div_mod::*;
use vstd::arithmetic::mul::*;
use vstd::arithmetic::power2::*;
use vstd::prelude::*;

use super::{Exact, Fmt, Operand, Step, Term};
use crate::decimal::RoundingMode;
use crate::ieee::{lemma_flags, IeeeFlag, IeeeFlags, IeeeWidth};
use crate::native_rational::verus_proofs::{bit_len, lemma_bit_len_bound, lemma_leading_zeros};

verus! {

/// Rounding on an integer grid. The value is `q` exactly, or strictly between
/// `q` and `q + 1` when `sticky`. `k` is the multiple of `grid` it rounds to
/// under `direction` for a value of sign `negative`: the nearest multiple with
/// ties to the even one, the nearest with ties away from zero, or the
/// neighbour toward zero, toward positive or toward negative.
pub open spec fn rounds_to(
    direction: RoundingMode,
    negative: bool,
    q: int,
    sticky: bool,
    grid: int,
    k: int,
) -> bool {
    let half = grid / 2;
    let at = k * grid;
    let toward_zero = at <= q < at + grid;
    let away = (q > at - grid || (q == at - grid && sticky)) && (q < at || (q == at && !sticky));
    match direction {
        RoundingMode::Exact | RoundingMode::NearestEven => {
            &&& q >= at - half
            &&& q < at + half || (q == at + half && !sticky)
            &&& (q == at - half || q == at + half) && !sticky ==> k % 2 == 0
        },
        RoundingMode::NearestAway => at - half <= q < at + half,
        RoundingMode::TowardZero => toward_zero,
        RoundingMode::TowardPositive => if negative {
            toward_zero
        } else {
            away
        },
        RoundingMode::TowardNegative => if negative {
            away
        } else {
            toward_zero
        },
    }
}

/// Whether the value `q` (or the open interval above it when `sticky`) is not
/// a multiple of `grid`.
pub open spec fn off_grid(q: int, sticky: bool, grid: int) -> bool {
    q % grid != 0 || sticky
}

/// The shifts of `round_at` as arithmetic, for one or two guard bits.
pub proof fn lemma_round_at_bits(q: u64, guard_bits: u32)
    requires
        guard_bits == 1 || guard_bits == 2,
        q < 0x4000_0000_0000_0000,
    ensures
        ({
            let grid: int = if guard_bits == 1 { 2 } else { 4 };
            &&& (q >> guard_bits) == q as int / grid
            &&& ((q >> guard_bits) << guard_bits) == (q as int / grid) * grid
            &&& (1u64 << ((guard_bits - 1) as u32)) == grid / 2
            &&& (q >> guard_bits) + 1 < 0x4000_0000_0000_0000
            &&& ((q >> guard_bits) & 1 == 1) == ((q as int / grid) % 2 == 1)
            &&& (q as int / grid) * grid + q as int % grid == q
            &&& (q as int / grid + 1) * grid == (q as int / grid) * grid + grid
            &&& 0 <= q as int % grid < grid
        }),
{
    if guard_bits == 1 {
        assert((q >> 1u32) == q / 2 && ((q >> 1u32) << 1u32) == (q / 2) * 2 && (1u64 << 0u32) == 1
            && (((q >> 1u32) & 1 == 1) == ((q / 2) % 2 == 1))) by (bit_vector)
            requires
                q < 0x4000_0000_0000_0000,
        ;
    } else {
        assert((q >> 2u32) == q / 4 && ((q >> 2u32) << 2u32) == (q / 4) * 4 && (1u64 << 1u32) == 2
            && (((q >> 2u32) & 1 == 1) == ((q / 4) % 2 == 1))) by (bit_vector)
            requires
                q < 0x4000_0000_0000_0000,
        ;
    }
}

/// The precondition of `round_at`: one or two guard bits, and room above `q`.
pub open spec fn round_at_pre(q: u64, guard_bits: u32) -> bool {
    (guard_bits == 1 || guard_bits == 2) && q < 0x4000_0000_0000_0000
}

/// Specification of the production `round_at`.
pub open spec fn round_at(
    q: u64,
    sticky: bool,
    guard_bits: u32,
    negative: bool,
    direction: RoundingMode,
) -> (u64, bool) {
    let kept = q >> guard_bits;
    let remainder = (q - (kept << guard_bits)) as u64;
    let half = 1u64 << ((guard_bits - 1) as u32);
    let below_half = remainder < half;
    let at_half = remainder == half;
    let inexact = remainder != 0 || sticky;
    let above_half = remainder > half || (at_half && sticky);
    let tie = at_half && !sticky;
    let up = match direction {
        RoundingMode::Exact | RoundingMode::NearestEven => above_half || (tie && kept & 1 == 1),
        RoundingMode::NearestAway => !below_half,
        RoundingMode::TowardZero => false,
        RoundingMode::TowardPositive => inexact && !negative,
        RoundingMode::TowardNegative => inexact && negative,
    };
    (
        if up {
            (kept + 1) as u64
        } else {
            kept
        },
        inexact,
    )
}

/// `round_at` rounds on the grid `2^guard_bits`: its first result is the
/// multiple `q` (with its sticky) rounds to under the direction, and its
/// second says whether `q` was off the grid.
pub proof fn lemma_round_at(q: u64, sticky: bool, guard_bits: u32, negative: bool, direction: RoundingMode)
    requires
        round_at_pre(q, guard_bits),
    ensures
        ({
            let grid: int = if guard_bits == 1 { 2 } else { 4 };
            let r = round_at(q, sticky, guard_bits, negative, direction);
            &&& rounds_to(direction, negative, q as int, sticky, grid, r.0 as int)
            &&& r.1 == off_grid(q as int, sticky, grid)
            &&& r.0 == q as int / grid || r.0 == q as int / grid + 1
        }),
{
    lemma_round_at_bits(q, guard_bits);
}

// ---- the two formats ------------------------------------------------------------

/// `fmt` is binary32 or binary64.
pub(super) open spec fn wf(fmt: Fmt) -> bool {
    (fmt.exponent_bits == 8 && fmt.fraction_bits == 23 && fmt.width_bits == 32) || (fmt.exponent_bits
        == 11 && fmt.fraction_bits == 52 && fmt.width_bits == 64)
}

pub(super) open spec fn fmt_of(width: IeeeWidth) -> Fmt {
    match width {
        IeeeWidth::Binary32 => Fmt { exponent_bits: 8, fraction_bits: 23, width_bits: 32 },
        IeeeWidth::Binary64 => Fmt { exponent_bits: 11, fraction_bits: 52, width_bits: 64 },
    }
}

pub(super) open spec fn single(fmt: Fmt) -> bool {
    fmt.exponent_bits == 8
}

pub(super) open spec fn fraction(fmt: Fmt) -> i32 {
    fmt.fraction_bits as i32
}

pub(super) open spec fn bias(fmt: Fmt) -> i32 {
    if single(fmt) {
        127
    } else {
        1023
    }
}

pub(super) open spec fn quantum_min(fmt: Fmt) -> i32 {
    (1 - bias(fmt) - fraction(fmt)) as i32
}

pub(super) open spec fn emin(fmt: Fmt) -> i32 {
    (1 - bias(fmt)) as i32
}

pub(super) open spec fn sign_mask(fmt: Fmt) -> u64 {
    if single(fmt) {
        0x8000_0000
    } else {
        0x8000_0000_0000_0000
    }
}

pub(super) open spec fn exponent_ones(fmt: Fmt) -> u64 {
    if single(fmt) {
        0xFF
    } else {
        0x7FF
    }
}

pub(super) open spec fn exponent_mask(fmt: Fmt) -> u64 {
    if single(fmt) {
        0x7F80_0000
    } else {
        0x7FF0_0000_0000_0000
    }
}

pub(super) open spec fn fraction_mask(fmt: Fmt) -> u64 {
    if single(fmt) {
        0x7F_FFFF
    } else {
        0xF_FFFF_FFFF_FFFF
    }
}

pub(super) open spec fn quiet_bit(fmt: Fmt) -> u64 {
    if single(fmt) {
        0x40_0000
    } else {
        0x8_0000_0000_0000
    }
}

pub(super) open spec fn sign(fmt: Fmt, negative: bool) -> u64 {
    if negative {
        sign_mask(fmt)
    } else {
        0
    }
}

pub(super) open spec fn zero_bits(fmt: Fmt, negative: bool) -> u64 {
    sign(fmt, negative)
}

pub(super) open spec fn infinity(fmt: Fmt, negative: bool) -> u64 {
    (sign(fmt, negative) + exponent_mask(fmt)) as u64
}

pub(super) open spec fn max_finite(fmt: Fmt, negative: bool) -> u64 {
    (sign(fmt, negative) + (if single(fmt) {
        0x7F7F_FFFFu64
    } else {
        0x7FEF_FFFF_FFFF_FFFFu64
    })) as u64
}

pub(super) open spec fn canonical_nan(fmt: Fmt) -> u64 {
    if single(fmt) {
        0x7FC0_0000
    } else {
        0x7FF8_0000_0000_0000
    }
}

/// The shifts and masks of the format helpers, as numbers.
pub(super) proof fn lemma_fmt_bits(fmt: Fmt)
    requires
        wf(fmt),
    ensures
        (1i32 << ((fmt.exponent_bits - 1) as u32)) == bias(fmt) + 1,
        (1u64 << ((fmt.exponent_bits + fmt.fraction_bits) as u32)) == sign_mask(fmt),
        (1u64 << fmt.exponent_bits) == exponent_ones(fmt) + 1,
        (exponent_ones(fmt) << fmt.fraction_bits) == exponent_mask(fmt),
        (1u64 << fmt.fraction_bits) == fraction_mask(fmt) + 1,
        (1u64 << ((fmt.fraction_bits - 1) as u32)) == quiet_bit(fmt),
        (sign_mask(fmt) | exponent_mask(fmt)) == sign_mask(fmt) + exponent_mask(fmt),
        (0u64 | exponent_mask(fmt)) == exponent_mask(fmt),
        (exponent_mask(fmt) | quiet_bit(fmt)) == canonical_nan(fmt),
        (sign_mask(fmt) | ((exponent_mask(fmt) - (fraction_mask(fmt) + 1)) as u64) | fraction_mask(fmt))
            == max_finite(fmt, true),
        (0u64 | ((exponent_mask(fmt) - (fraction_mask(fmt) + 1)) as u64) | fraction_mask(fmt))
            == max_finite(fmt, false),
{
    if single(fmt) {
        assert((1i32 << 7u32) == 128 && (1u64 << 31u32) == 0x8000_0000 && (1u64 << 8u32) == 0x100
            && (0xFFu64 << 23u32) == 0x7F80_0000 && (1u64 << 23u32) == 0x80_0000 && (1u64 << 22u32)
            == 0x40_0000 && (0x8000_0000u64 | 0x7F80_0000u64) == 0xFF80_0000 && (0u64 | 0x7F80_0000u64)
            == 0x7F80_0000 && (0x7F80_0000u64 | 0x40_0000u64) == 0x7FC0_0000 && (0x8000_0000u64
            | 0x7F00_0000u64 | 0x7F_FFFFu64) == 0xFF7F_FFFF && (0u64 | 0x7F00_0000u64 | 0x7F_FFFFu64)
            == 0x7F7F_FFFF) by (bit_vector);
    } else {
        assert((1i32 << 10u32) == 1024 && (1u64 << 63u32) == 0x8000_0000_0000_0000 && (1u64 << 11u32)
            == 0x800 && (0x7FFu64 << 52u32) == 0x7FF0_0000_0000_0000 && (1u64 << 52u32)
            == 0x10_0000_0000_0000 && (1u64 << 51u32) == 0x8_0000_0000_0000 && (0x8000_0000_0000_0000u64
            | 0x7FF0_0000_0000_0000u64) == 0xFFF0_0000_0000_0000 && (0u64 | 0x7FF0_0000_0000_0000u64)
            == 0x7FF0_0000_0000_0000 && (0x7FF0_0000_0000_0000u64 | 0x8_0000_0000_0000u64)
            == 0x7FF8_0000_0000_0000 && (0x8000_0000_0000_0000u64 | 0x7FE0_0000_0000_0000u64
            | 0xF_FFFF_FFFF_FFFFu64) == 0xFFEF_FFFF_FFFF_FFFF && (0u64 | 0x7FE0_0000_0000_0000u64
            | 0xF_FFFF_FFFF_FFFFu64) == 0x7FEF_FFFF_FFFF_FFFF) by (bit_vector);
    }
}

/// Specification of the production `bit_length`.
pub open spec fn bit_length(value: u64) -> i32 {
    (64 - vstd::std_specs::bits::u64_leading_zeros(value)) as i32
}

// ---- round ----------------------------------------------------------------------

/// The precondition of `round`: a known format, a nonzero magnitude, and an
/// exponent in the range the planners produce.
pub(super) open spec fn round_pre(fmt: Fmt, exact: Exact) -> bool {
    wf(fmt) && match exact {
        Exact::Zero { .. } => true,
        Exact::Real { mag, exp, .. } => mag != 0 && -8192 <= exp <= 8192,
    }
}

/// The quantities `round` derives from the magnitude and its exponent.
pub(super) open spec fn round_shift(fmt: Fmt, mag: u128, exp: i32) -> (i32, u64, i32) {
    let precision = fraction(fmt);
    let length = (128 - mag.leading_zeros()) as i32;
    let log2 = (length - 1 + exp) as i32;
    let above = (log2 - precision - quantum_min(fmt)) as i32;
    let (quantum, index) = if above > 0 {
        ((log2 - precision) as i32, above as u64)
    } else {
        (quantum_min(fmt), 0u64)
    };
    (quantum, index, (quantum - 2 - exp) as i32)
}

/// The two guard bits and what was lost below them.
pub(super) open spec fn round_q(mag: u128, shift: i32) -> (u64, bool) {
    if shift >= 128 {
        (0u64, true)
    } else if shift > 0 {
        ((mag >> (shift as u32)) as u64, mag & (((1u128 << (shift as u32)) - 1) as u128) != 0)
    } else {
        ((mag << ((-shift) as u32)) as u64, false)
    }
}

/// Specification of the production `round`.
pub(super) open spec fn round(fmt: Fmt, exact: Exact, rounding: RoundingMode) -> (u64, IeeeFlags) {
    match exact {
        Exact::Zero { negative } => (sign(fmt, negative), IeeeFlags::spec_empty()),
        Exact::Real { negative, mag, exp, sticky } => {
            let direction = if rounding == RoundingMode::Exact {
                RoundingMode::NearestEven
            } else {
                rounding
            };
            let precision = fraction(fmt);
            let length = (128 - mag.leading_zeros()) as i32;
            let log2 = (length - 1 + exp) as i32;
            let (quantum, index0, shift) = round_shift(fmt, mag, exp);
            let (q, lost) = round_q(mag, shift);
            let sticky = sticky || lost;
            let (kept0, inexact) = round_at(q, sticky, 2, negative, direction);
            let (kept, index) = if bit_length(kept0) > precision + 1 {
                (kept0 >> 1u32, (index0 + 1) as u64)
            } else {
                (kept0, index0)
            };
            let flags = if inexact {
                IeeeFlags::spec_empty().spec_with(IeeeFlag::Inexact)
            } else {
                IeeeFlags::spec_empty()
            };
            if bit_length(kept) == precision + 1 && index + 1 >= exponent_ones(fmt) {
                let to_infinity = match direction {
                    RoundingMode::Exact | RoundingMode::NearestEven | RoundingMode::NearestAway => true,
                    RoundingMode::TowardZero => false,
                    RoundingMode::TowardPositive => !negative,
                    RoundingMode::TowardNegative => negative,
                };
                (
                    if to_infinity {
                        infinity(fmt, negative)
                    } else {
                        max_finite(fmt, negative)
                    },
                    flags.spec_with(IeeeFlag::Overflow).spec_with(IeeeFlag::Inexact),
                )
            } else {
                let tiny = log2 < emin(fmt) && !(log2 + 1 == emin(fmt) && bit_length(
                    round_at(q, sticky, 1, negative, direction).0,
                ) > precision + 1);
                (
                    sign(fmt, negative) | (((index << fmt.fraction_bits) + kept) as u64),
                    if tiny && inexact {
                        flags.spec_with(IeeeFlag::Underflow)
                    } else {
                        flags
                    },
                )
            }
        },
    }
}

// ---- bit lengths and powers of two ----------------------------------------------

/// A positive value lies in the binade its bit length names.
pub proof fn lemma_bit_len_range(m: nat)
    requires
        m > 0,
    ensures
        pow2((bit_len(m) - 1) as nat) <= m < pow2(bit_len(m)),
    decreases m,
{
    lemma2_to64();
    if m > 1 {
        lemma_bit_len_range(m / 2);
        let b = bit_len(m / 2);
        lemma_pow2_unfold(b);
        lemma_pow2_unfold(b + 1);
    }
}

/// Dividing a value below `2^len` by `2^s` leaves a value below `2^(len - s)`.
pub proof fn lemma_div_pow2_upper(m: nat, len: nat, s: nat)
    requires
        m < pow2(len),
        s <= len,
    ensures
        m / pow2(s) < pow2((len - s) as nat),
{
    let (p, r) = (pow2(s), pow2((len - s) as nat));
    lemma_pow2_adds(s, (len - s) as nat);
    lemma_pow2_pos(s);
    lemma_fundamental_div_mod(m as int, p as int);
    lemma_mod_bound(m as int, p as int);
    let d = m / p;
    assert(d < r) by (nonlinear_arith)
        requires
            m == p * d + (m % p),
            m % p >= 0,
            m < p * r,
            p > 0,
            d >= 0,
    ;
}

/// Dividing a value of at least `2^len` by `2^s` leaves at least `2^(len - s)`.
pub proof fn lemma_div_pow2_lower(m: nat, len: nat, s: nat)
    requires
        m >= pow2(len),
        s <= len,
    ensures
        m / pow2(s) >= pow2((len - s) as nat),
{
    let (p, r) = (pow2(s), pow2((len - s) as nat));
    lemma_pow2_adds(s, (len - s) as nat);
    lemma_pow2_pos(s);
    lemma_fundamental_div_mod(m as int, p as int);
    lemma_mod_bound(m as int, p as int);
    let d = m / p;
    assert(d >= r) by (nonlinear_arith)
        requires
            m == p * d + (m % p),
            m % p < p,
            m >= p * r,
            p > 0,
    ;
}

/// The production `bit_length` names the binade of a nonzero `u64`.
pub proof fn lemma_bit_length(v: u64)
    ensures
        v == 0 ==> bit_length(v) == 0,
        v != 0 ==> 1 <= bit_length(v) <= 64 && pow2((bit_length(v) - 1) as nat) <= v,
        v != 0 && bit_length(v) < 64 ==> v < pow2(bit_length(v) as nat),
{
    vstd::std_specs::bits::axiom_u64_leading_zeros(v);
    let lz = vstd::std_specs::bits::u64_leading_zeros(v);
    if v != 0 {
        let t = (63 - lz) as u64;
        let top = v >> t;
        assert(top >= 1) by (bit_vector)
            requires
                top & 1 != 0,
        ;
        vstd::bits::lemma_u64_shr_is_div(v, t);
        lemma_pow2_pos(t as nat);
        lemma_fundamental_div_mod(v as int, pow2(t as nat) as int);
        lemma_mod_bound(v as int, pow2(t as nat) as int);
        assert(v >= pow2(t as nat)) by (nonlinear_arith)
            requires
                v == pow2(t as nat) * (v as nat / pow2(t as nat)) + (v as nat % pow2(t as nat)),
                v as nat / pow2(t as nat) >= 1,
                v as nat % pow2(t as nat) >= 0,
                pow2(t as nat) > 0,
        ;
        if lz > 0 {
            let u = (64 - lz) as u64;
            vstd::bits::lemma_u64_shr_is_div(v, u);
            lemma_pow2_pos(u as nat);
            lemma_small_div_converse(v as int, pow2(u as nat) as int);
        }
    }
}

pub proof fn lemma_pow2_le(a: nat, b: nat)
    requires
        a <= b,
    ensures
        pow2(a) <= pow2(b),
{
    if a < b {
        lemma_pow2_strictly_increases(a, b);
    }
}

/// `2^(fraction_bits + 1)`: one more than the largest significand.
pub(super) open spec fn top(fmt: Fmt) -> int {
    if single(fmt) {
        0x100_0000
    } else {
        0x20_0000_0000_0000
    }
}

pub(super) proof fn lemma_top(fmt: Fmt)
    requires
        wf(fmt),
    ensures
        top(fmt) == pow2((fmt.fraction_bits + 1) as nat),
        top(fmt) == 2 * pow2(fmt.fraction_bits as nat),
        4 * top(fmt) == pow2((fmt.fraction_bits + 3) as nat),
        2 * top(fmt) == pow2((fmt.fraction_bits + 2) as nat),
{
    lemma2_to64();
    lemma2_to64_rest();
}

/// The bit length of a `u64` against the significand window of the format.
pub(super) proof fn lemma_bit_length_window(fmt: Fmt, v: u64)
    requires
        wf(fmt),
    ensures
        (bit_length(v) > fraction(fmt) + 1) == (v >= top(fmt)),
        (bit_length(v) == fraction(fmt) + 1) == (top(fmt) / 2 <= v < top(fmt)),
{
    lemma_bit_length(v);
    lemma_top(fmt);
    let f = fmt.fraction_bits as nat;
    let b = bit_length(v);
    if v != 0 {
        if b > f + 1 {
            lemma_pow2_le(f + 1, (b - 1) as nat);
        } else {
            lemma_pow2_le(b as nat, f + 1);
        }
        if b >= f + 1 {
            lemma_pow2_le(f, (b - 1) as nat);
        } else {
            lemma_pow2_le(b as nat, f);
        }
    }
}

/// The two-guard-bit quotient of `round` fits below `2^(fraction_bits + 3)`,
/// and has exactly that many bits above the subnormal range.
pub(super) proof fn lemma_round_q(fmt: Fmt, mag: u128, exp: i32)
    requires
        wf(fmt),
        mag != 0,
        -8192 <= exp <= 8192,
    ensures
        ({
            let (quantum, index0, shift) = round_shift(fmt, mag, exp);
            let q = round_q(mag, shift).0;
            &&& -64 < shift
            &&& index0 < 0x4000
            &&& q < 4 * top(fmt)
            &&& index0 > 0 ==> q >= 2 * top(fmt)
            &&& 0 < shift < 128 ==> (mag >> (shift as u32)) < 4 * top(fmt)
            &&& shift <= 0 ==> (mag << ((-shift) as u32)) < 4 * top(fmt)
        }),
{
    lemma_leading_zeros(mag);
    lemma_bit_len_range(mag as nat);
    lemma_top(fmt);
    let len = bit_len(mag as nat);
    let f = fmt.fraction_bits as nat;
    let (quantum, index0, shift) = round_shift(fmt, mag, exp);
    assert(shift >= len - f - 3);
    assert(index0 > 0 ==> shift == len - f - 3);
    if shift >= 128 {
    } else if shift > 0 {
        let s = shift as nat;
        vstd::bits::lemma_u128_shr_is_div(mag, shift as u128);
        if s <= len {
            lemma_div_pow2_upper(mag as nat, len, s);
            lemma_pow2_le((len - s) as nat, f + 3);
        } else {
            lemma_pow2_le(len, s);
            lemma_basic_div(mag as int, pow2(s) as int);
        }
        if index0 > 0 {
            lemma_div_pow2_lower(mag as nat, (len - 1) as nat, s);
        }
    } else {
        let s = (-shift) as nat;
        lemma_pow2_adds(len, s);
        lemma_pow2_adds((len - 1) as nat, s);
        lemma_pow2_le(len + s, f + 3);
        lemma_pow2_pos(s);
        let p = pow2(s);
        assert(mag * p < pow2(len + s) && mag * p >= pow2((len - 1) as nat) * p) by (nonlinear_arith)
            requires
                pow2((len - 1) as nat) <= mag < pow2(len),
                pow2(len + s) == pow2(len) * p,
                p > 0,
        ;
        vstd::bits::lemma_u128_shl_is_mul(mag, (-shift) as u128);
    }
}

/// What the body of `round` needs to run without overflow.
pub(super) proof fn lemma_round_safe(fmt: Fmt, exact: Exact, rounding: RoundingMode)
    requires
        round_pre(fmt, exact),
    ensures
        forall|s: u32| s < 128 ==> #[trigger] (1u128 << s) >= 1,
        forall|i: u64|
            i < 0x800 ==> #[trigger] (i << fmt.fraction_bits) == i * (top(fmt) / 2) && (i
                << fmt.fraction_bits) <= 0x7FF0_0000_0000_0000,
        forall|v: u64| #[trigger] (v >> 1u32) == v / 2,
        exact matches Exact::Real { negative, mag, exp, sticky } ==> ({
            let direction = if rounding == RoundingMode::Exact {
                RoundingMode::NearestEven
            } else {
                rounding
            };
            let (quantum, index0, shift) = round_shift(fmt, mag, exp);
            let (q, lost) = round_q(mag, shift);
            let kept0 = round_at(q, sticky || lost, 2, negative, direction).0;
            &&& mag.leading_zeros() <= 127
            &&& -64 < shift
            &&& index0 < 0x4000
            &&& q < 4 * top(fmt)
            &&& 0 < shift < 128 ==> (mag >> (shift as u32)) < 4 * top(fmt)
            &&& shift <= 0 ==> (mag << ((-shift) as u32)) < 4 * top(fmt)
            &&& kept0 <= top(fmt)
            &&& index0 > 0 ==> kept0 >= top(fmt) / 2
            &&& (bit_length(kept0) > fraction(fmt) + 1) == (kept0 >= top(fmt))
            &&& (bit_length(kept0) == fraction(fmt) + 1) == (top(fmt) / 2 <= kept0 < top(fmt))
            &&& (bit_length(kept0 >> 1u32) == fraction(fmt) + 1) == (top(fmt) / 2 <= kept0 / 2 < top(fmt))
        }),
{
    assert(forall|s: u32| s < 128 ==> #[trigger] (1u128 << s) >= 1) by (bit_vector);
    assert(forall|v: u64| #[trigger] (v >> 1u32) == v / 2) by (bit_vector);
    if single(fmt) {
        assert(forall|i: u64|
            i < 0x800 ==> #[trigger] (i << 23u32) == i * 0x80_0000 && (i << 23u32)
                <= 0x7FF0_0000_0000_0000) by (bit_vector);
    } else {
        assert(forall|i: u64|
            i < 0x800 ==> #[trigger] (i << 52u32) == i * 0x10_0000_0000_0000 && (i << 52u32)
                <= 0x7FF0_0000_0000_0000) by (bit_vector);
    }
    if let Exact::Real { negative, mag, exp, sticky } = exact {
        let direction = if rounding == RoundingMode::Exact {
            RoundingMode::NearestEven
        } else {
            rounding
        };
        lemma_leading_zeros(mag);
        lemma_round_q(fmt, mag, exp);
        let (quantum, index0, shift) = round_shift(fmt, mag, exp);
        let (q, lost) = round_q(mag, shift);
        lemma_round_at(q, sticky || lost, 2, negative, direction);
        let kept0 = round_at(q, sticky || lost, 2, negative, direction).0;
        lemma_bit_length_window(fmt, kept0);
        lemma_bit_length_window(fmt, kept0 >> 1u32);
    }
}

// ---- what round means -------------------------------------------------------------

/// The value `(mag + t) 2^exp`, `t` zero or (when `sticky`) strictly between 0
/// and 1, in units of `2^scale`: its floor, and whether it is off the
/// integers. Below `exp` the value must be exact for this to describe it.
pub open spec fn scaled(mag: int, exp: int, sticky: bool, scale: int) -> (int, bool) {
    if scale >= exp {
        let unit = pow2((scale - exp) as nat) as int;
        (mag / unit, sticky || mag % unit != 0)
    } else {
        (mag * pow2((exp - scale) as nat), sticky)
    }
}

/// The exponent of the quantum the format rounds a value of binade `log2` to:
/// `fraction_bits` below the leading bit, and never below the subnormal quantum.
pub(super) open spec fn quantum_of(fmt: Fmt, log2: int) -> int {
    if log2 - fraction(fmt) > quantum_min(fmt) {
        log2 - fraction(fmt)
    } else {
        quantum_min(fmt) as int
    }
}

/// The shifted quotient of `round` is the value in units of a quarter quantum.
pub(super) proof fn lemma_round_scaled(fmt: Fmt, mag: u128, exp: i32, sticky: bool)
    requires
        wf(fmt),
        mag != 0,
        -8192 <= exp <= 8192,
    ensures
        ({
            let (quantum, index0, shift) = round_shift(fmt, mag, exp);
            let (q, lost) = round_q(mag, shift);
            let log2 = bit_len(mag as nat) - 1 + exp;
            &&& quantum == quantum_of(fmt, log2)
            &&& index0 == quantum - quantum_min(fmt)
            &&& (q as int, sticky || lost) == scaled(mag as int, exp as int, sticky, quantum - 2)
        }),
{
    lemma_leading_zeros(mag);
    lemma_bit_len_range(mag as nat);
    lemma_round_q(fmt, mag, exp);
    let len = bit_len(mag as nat);
    let (quantum, index0, shift) = round_shift(fmt, mag, exp);
    lemma2_to64();
    if shift >= 128 {
        let unit = pow2(shift as nat);
        lemma_pow2_le(len, shift as nat);
        lemma_basic_div(mag as int, unit as int);
        lemma_small_mod(mag as nat, unit);
    } else if shift > 0 {
        let s = shift as nat;
        let unit = pow2(s);
        vstd::bits::lemma_u128_shr_is_div(mag, shift as u128);
        vstd::bits::lemma_u128_low_bits_mask_is_mod(mag, s);
        vstd::bits::lemma_u128_pow2_no_overflow(s);
        vstd::bits::lemma_u128_shl_is_mul(1u128, shift as u128);
        assert(((1u128 << (shift as u32)) - 1) as u128 == vstd::bits::low_bits_mask(s) as u128);
    } else if shift == 0 {
        assert(mag << 0u32 == mag) by (bit_vector);
        let (q, lost) = round_q(mag, shift);
        let target = scaled(mag as int, exp as int, sticky, quantum - 2);
        assert(mag << ((-shift) as u32) == mag);
        assert(q == mag as u64);
        assert(mag < 4 * top(fmt));
        assert(q as int == mag as int);
        assert(quantum - 2 == exp);
        assert(pow2(0) == 1);
        assert(target.0 == mag as int / 1);
        assert(target.0 == mag as int);
    } else {
        let s = (-shift) as nat;
        lemma_pow2_adds(len, s);
        lemma_pow2_le(len + s, (fmt.fraction_bits + 3) as nat);
        lemma_pow2_pos(s);
        lemma_top(fmt);
        assert(mag * pow2(s) < pow2(len + s)) by (nonlinear_arith)
            requires
                mag < pow2(len),
                pow2(len + s) == pow2(len) * pow2(s),
                pow2(s) > 0,
        ;
        vstd::bits::lemma_u128_shl_is_mul(mag, (-shift) as u128);
    }
}

/// Rounding on an integer grid as a function: the multiple of `grid` that the
/// value `q` (or the open interval above it when `sticky`) rounds to.
pub open spec fn grid_round(direction: RoundingMode, negative: bool, q: int, sticky: bool, grid: int) -> int {
    let kept = q / grid;
    let rem = q % grid;
    let half = grid / 2;
    let inexact = rem != 0 || sticky;
    let up = match direction {
        RoundingMode::Exact | RoundingMode::NearestEven => rem > half || (rem == half && (sticky || kept
            % 2 == 1)),
        RoundingMode::NearestAway => rem >= half,
        RoundingMode::TowardZero => false,
        RoundingMode::TowardPositive => inexact && !negative,
        RoundingMode::TowardNegative => inexact && negative,
    };
    if up {
        kept + 1
    } else {
        kept
    }
}

/// `grid_round` is the rounding of `rounds_to`, and the only multiple that is,
/// on the two grids `round` uses.
pub proof fn lemma_grid_round(direction: RoundingMode, negative: bool, q: int, sticky: bool, grid: int)
    requires
        grid == 2 || grid == 4,
        q >= 0,
    ensures
        rounds_to(direction, negative, q, sticky, grid, grid_round(direction, negative, q, sticky, grid)),
        forall|k: int| #[trigger]
            rounds_to(direction, negative, q, sticky, grid, k) ==> k == grid_round(
                direction,
                negative,
                q,
                sticky,
                grid,
            ),
{
}

/// The production `round_at` computes `grid_round` on the grid `2^guard_bits`.
pub proof fn lemma_round_at_is_grid_round(
    q: u64,
    sticky: bool,
    guard_bits: u32,
    negative: bool,
    direction: RoundingMode,
)
    requires
        round_at_pre(q, guard_bits),
    ensures
        ({
            let grid: int = if guard_bits == 1 { 2 } else { 4 };
            let r = round_at(q, sticky, guard_bits, negative, direction);
            &&& r.0 == grid_round(direction, negative, q as int, sticky, grid)
            &&& r.1 == off_grid(q as int, sticky, grid)
        }),
{
    lemma_round_at_bits(q, guard_bits);
}

/// Whether an overflow goes to infinity under the direction, for the sign.
pub open spec fn overflow_to_infinity(direction: RoundingMode, negative: bool) -> bool {
    match direction {
        RoundingMode::Exact | RoundingMode::NearestEven | RoundingMode::NearestAway => true,
        RoundingMode::TowardZero => false,
        RoundingMode::TowardPositive => !negative,
        RoundingMode::TowardNegative => negative,
    }
}

/// The direction `round` uses: strict `exact` computes its would-be flags
/// under nearest-even.
pub open spec fn effective(rounding: RoundingMode) -> RoundingMode {
    if rounding == RoundingMode::Exact {
        RoundingMode::NearestEven
    } else {
        rounding
    }
}

/// What `round` returns, in mathematics.
///
/// A zero keeps its sign and raises nothing. Otherwise the magnitude is
/// `x = (mag + t) 2^exp`, in binade `log2`, with quantum `2^quantum` for the
/// format. `k` is `x` rounded to a multiple of the quantum under the direction
/// (`grid_round` on `x` in quarter quanta). The result's biased exponent is 0
/// for a subnormal `k`, and one more per binade above; a carry to `2^(f+1)`
/// adds one. When that reaches the all-ones exponent the result overflows: to
/// infinity or to the largest finite value by the direction, with overflow and
/// inexact. Otherwise the bits are the sign, plus the exponent field, plus `k`
/// (the carry and the implicit bit both land in the exponent field); inexact
/// says `x` was off the quantum grid; and underflow is inexact and tiny, where
/// tiny means the result rounded with an unbounded exponent range is below the
/// least normal magnitude.
pub(super) open spec fn round_post(fmt: Fmt, exact: Exact, rounding: RoundingMode, r: (u64, IeeeFlags)) -> bool {
    match exact {
        Exact::Zero { negative } => r.0 == sign(fmt, negative) && r.1.none(),
        Exact::Real { negative, mag, exp, sticky } => {
            let direction = effective(rounding);
            let log2 = bit_len(mag as nat) - 1 + exp;
            let quantum = quantum_of(fmt, log2);
            let (q, off) = scaled(mag as int, exp as int, sticky, quantum - 2);
            let k = grid_round(direction, negative, q, off, 4);
            let inexact = off_grid(q, off, 4);
            let steps = quantum - quantum_min(fmt);
            let biased = if k < top(fmt) / 2 {
                0
            } else if k == top(fmt) {
                steps + 2
            } else {
                steps + 1
            };
            &&& !r.1.has(IeeeFlag::Invalid)
            &&& !r.1.has(IeeeFlag::DivideByZero)
            &&& if biased >= exponent_ones(fmt) {
                &&& r.0 == if overflow_to_infinity(direction, negative) {
                    infinity(fmt, negative)
                } else {
                    max_finite(fmt, negative)
                }
                &&& r.1.has(IeeeFlag::Overflow)
                &&& r.1.has(IeeeFlag::Inexact)
                &&& !r.1.has(IeeeFlag::Underflow)
            } else {
                let tiny = log2 < emin(fmt) && !(log2 + 1 == emin(fmt) && grid_round(
                    direction,
                    negative,
                    q,
                    off,
                    2,
                ) >= top(fmt));
                &&& r.0 == sign(fmt, negative) + steps * (top(fmt) / 2) + k
                &&& !r.1.has(IeeeFlag::Overflow)
                &&& r.1.has(IeeeFlag::Inexact) == inexact
                &&& r.1.has(IeeeFlag::Underflow) == (tiny && inexact)
            }
        },
    }
}

/// `round` returns what `round_post` says, for every exact intermediate the
/// planners produce.
pub(super) proof fn lemma_round(fmt: Fmt, exact: Exact, rounding: RoundingMode)
    requires
        round_pre(fmt, exact),
    ensures
        round_post(fmt, exact, rounding, round(fmt, exact, rounding)),
{
    let empty = IeeeFlags::spec_empty();
    assert forall|base: IeeeFlags, added: IeeeFlag, flag: IeeeFlag|
        base.spec_with(added).has(flag) == (flag == added || base.has(flag)) by {
        lemma_flags(base, added, flag);
    }
    assert forall|flag: IeeeFlag| !empty.has(flag) by {
        lemma_flags(empty, flag, flag);
    }
    lemma_flags(empty, IeeeFlag::Inexact, IeeeFlag::Inexact);
    match exact {
        Exact::Zero { negative } => {},
        Exact::Real { negative, mag, exp, sticky } => {
            let direction = effective(rounding);
            lemma_round_safe(fmt, exact, rounding);
            lemma_round_scaled(fmt, mag, exp, sticky);
            lemma_top(fmt);
            let (quantum, index0, shift) = round_shift(fmt, mag, exp);
            let (q, lost) = round_q(mag, shift);
            let off = sticky || lost;
            lemma_round_at_is_grid_round(q, off, 2, negative, direction);
            lemma_round_at_is_grid_round(q, off, 1, negative, direction);
            lemma_leading_zeros(mag);
            let precision = fraction(fmt) as int;
            let log2 = bit_len(mag as nat) - 1 + exp;
            let kept0 = round_at(q, off, 2, negative, direction).0;
            let inexact = round_at(q, off, 2, negative, direction).1;
            let k = grid_round(direction, negative, q as int, off, 4);
            assert(kept0 == k);
            assert(inexact == off_grid(q as int, off, 4));
            lemma_bit_length_window(fmt, round_at(q, off, 1, negative, direction).0);
            let carry = bit_length(kept0) > precision + 1;
            assert(carry == (k == top(fmt)));
            let kept = if carry {
                kept0 >> 1u32
            } else {
                kept0
            };
            let index: u64 = if carry {
                (index0 + 1) as u64
            } else {
                index0
            };
            assert(carry ==> kept == top(fmt) / 2);
            let steps = quantum - quantum_min(fmt);
            assert(index0 == steps);
            let biased = if k < top(fmt) / 2 {
                0
            } else if k == top(fmt) {
                steps + 2
            } else {
                steps + 1
            };
            let over = bit_length(kept) == precision + 1 && index + 1 >= exponent_ones(fmt);
            assert(over == (biased >= exponent_ones(fmt)));
            let r = round(fmt, exact, rounding);
            assert(forall|m: u64| #[trigger] (0u64 | m) == m) by (bit_vector);
            if single(fmt) {
                assert(forall|m: u64|
                    m < 0x8000_0000 ==> #[trigger] (0x8000_0000u64 | m) == 0x8000_0000 + m)
                    by (bit_vector);
            } else {
                assert(forall|m: u64|
                    m < 0x8000_0000_0000_0000 ==> #[trigger] (0x8000_0000_0000_0000u64 | m)
                        == 0x8000_0000_0000_0000 + m) by (bit_vector);
            }
            if over {
                assert(r.0 == if overflow_to_infinity(direction, negative) {
                    infinity(fmt, negative)
                } else {
                    max_finite(fmt, negative)
                });
            } else {
                assert(index < 0x800);
                let c = top(fmt) / 2;
                assert((steps + 1) * c == steps * c + c) by (nonlinear_arith);
                assert((index << fmt.fraction_bits) == index * c);
                assert(index * c == if carry {
                    (steps + 1) * c
                } else {
                    steps * c
                });
                let body = ((index << fmt.fraction_bits) + kept) as u64;
                assert(body == steps * (top(fmt) / 2) + k);
                if k >= top(fmt) / 2 {
                    let most = exponent_ones(fmt) - 2;
                    assert(steps * c <= most * c) by (nonlinear_arith)
                        requires
                            0 <= steps <= most,
                            c > 0,
                    ;
                    assert(most * c + 2 * c == exponent_mask(fmt)) by {
                        if single(fmt) {
                            assert(253 * 0x80_0000 + 2 * 0x80_0000 == 0x7F80_0000) by (compute_only);
                        } else {
                            assert(2045 * 0x10_0000_0000_0000 + 2 * 0x10_0000_0000_0000
                                == 0x7FF0_0000_0000_0000) by (compute_only);
                        }
                    }
                } else {
                    assert(steps == 0);
                    assert(steps * c == 0) by (nonlinear_arith)
                        requires
                            steps == 0,
                    ;
                }
                assert(body < sign_mask(fmt));
                assert(r.0 == sign(fmt, negative) | body);
                if single(fmt) {
                    if negative {
                        assert((0x8000_0000u64 | body) == 0x8000_0000 + body);
                    } else {
                        assert((0u64 | body) == body);
                    }
                } else {
                    if negative {
                        assert((0x8000_0000_0000_0000u64 | body) == 0x8000_0000_0000_0000 + body);
                    } else {
                        assert((0u64 | body) == body);
                    }
                }
                assert(r.0 == sign(fmt, negative) + steps * (top(fmt) / 2) + k);
            }
        },
    }
}

// ---- decode ---------------------------------------------------------------------

/// Specification of the production `decode`.
pub(super) open spec fn decode(fmt: Fmt, bits: u64) -> Operand {
    let negative = bits & sign_mask(fmt) != 0;
    let biased = (bits >> fmt.fraction_bits) & exponent_ones(fmt);
    let fraction_field = bits & fraction_mask(fmt);
    if biased == exponent_ones(fmt) {
        if fraction_field == 0 {
            Operand::Infinite { negative }
        } else {
            Operand::Nan { signaling: fraction_field & quiet_bit(fmt) == 0 }
        }
    } else if biased == 0 {
        if fraction_field == 0 {
            Operand::Zero { negative }
        } else {
            Operand::Finite { negative, significand: fraction_field, exponent: quantum_min(fmt) }
        }
    } else {
        Operand::Finite {
            negative,
            significand: fraction_field | ((fraction_mask(fmt) + 1) as u64),
            exponent: (biased as i32 - bias(fmt) - fraction(fmt)) as i32,
        }
    }
}

/// The fields `decode` extracts stay inside their masks.
pub(super) proof fn lemma_decode_fields(fmt: Fmt, bits: u64)
    requires
        wf(fmt),
    ensures
        ((bits >> fmt.fraction_bits) & exponent_ones(fmt)) <= exponent_ones(fmt),
        (bits & fraction_mask(fmt)) <= fraction_mask(fmt),
{
    if single(fmt) {
        assert(((bits >> 23u32) & 0xFFu64) <= 0xFF && (bits & 0x7F_FFFFu64) <= 0x7F_FFFF) by (bit_vector);
    } else {
        assert(((bits >> 52u32) & 0x7FFu64) <= 0x7FF && (bits & 0xF_FFFF_FFFF_FFFFu64)
            <= 0xF_FFFF_FFFF_FFFF) by (bit_vector);
    }
}

/// The bits `round` assembles denote the rounded value: `decode` reads back a
/// zero for `k = 0`, the subnormal `k 2^quantum_min`, the normal
/// `k 2^(quantum_min + steps)`, or, after a carry to `2^(f+1)`, the power of
/// two one binade up.
pub(super) proof fn lemma_decode_rounded(fmt: Fmt, negative: bool, steps: int, k: int)
    requires
        wf(fmt),
        0 <= k <= top(fmt),
        0 <= steps,
        steps == 0 || k >= top(fmt) / 2,
        (if k < top(fmt) / 2 {
            0
        } else if k == top(fmt) {
            steps + 2
        } else {
            steps + 1
        }) < exponent_ones(fmt),
    ensures
        decode(fmt, (sign(fmt, negative) + steps * (top(fmt) / 2) + k) as u64) == (if k == 0 {
            Operand::Zero { negative }
        } else if k < top(fmt) / 2 {
            Operand::Finite { negative, significand: k as u64, exponent: quantum_min(fmt) }
        } else if k < top(fmt) {
            Operand::Finite {
                negative,
                significand: k as u64,
                exponent: (quantum_min(fmt) + steps) as i32,
            }
        } else {
            Operand::Finite {
                negative,
                significand: (top(fmt) / 2) as u64,
                exponent: (quantum_min(fmt) + steps + 1) as i32,
            }
        }),
{
    let (e, m, s) = (steps as u64, k as u64, sign(fmt, negative));
    if single(fmt) {
        assert(steps * 0x80_0000 == (e * 0x80_0000) as int) by (nonlinear_arith)
            requires
                0 <= steps < 0xFF,
                e == steps,
        ;
        let b = (s + e * 0x80_0000 + m) as u64;
        assert((b & 0x8000_0000 != 0) == (s != 0) && ((b >> 23u32) & 0xFF) == e + (m >> 23u32) && (b
            & 0x7F_FFFF) == (m & 0x7F_FFFF) && (m >> 23u32) == m / 0x80_0000 && ((m & 0x7F_FFFF)
            == m % 0x80_0000) && ((0x80_0000 <= m < 0x100_0000) ==> ((m & 0x7F_FFFF) | 0x80_0000) == m)
            && (0u64 | 0x80_0000u64) == 0x80_0000) by (bit_vector)
            requires
                b == (s + e * 0x80_0000 + m) as u64,
                s == 0 || s == 0x8000_0000,
                e < 0xFE,
                m <= 0x100_0000,
        ;
    } else {
        assert(steps * 0x10_0000_0000_0000 == (e * 0x10_0000_0000_0000) as int) by (nonlinear_arith)
            requires
                0 <= steps < 0x7FF,
                e == steps,
        ;
        let b = (s + e * 0x10_0000_0000_0000 + m) as u64;
        assert((b & 0x8000_0000_0000_0000 != 0) == (s != 0) && ((b >> 52u32) & 0x7FF) == e + (m >> 52u32)
            && (b & 0xF_FFFF_FFFF_FFFF) == (m & 0xF_FFFF_FFFF_FFFF) && (m >> 52u32)
            == m / 0x10_0000_0000_0000 && ((m & 0xF_FFFF_FFFF_FFFF) == m % 0x10_0000_0000_0000) && ((
        0x10_0000_0000_0000 <= m < 0x20_0000_0000_0000) ==> ((m & 0xF_FFFF_FFFF_FFFF)
            | 0x10_0000_0000_0000) == m) && (0u64 | 0x10_0000_0000_0000u64) == 0x10_0000_0000_0000)
            by (bit_vector)
            requires
                b == (s + e * 0x10_0000_0000_0000 + m) as u64,
                s == 0 || s == 0x8000_0000_0000_0000,
                e < 0x7FE,
                m <= 0x20_0000_0000_0000,
        ;
    }
}

// ---- operands and planning --------------------------------------------------------

/// A decoded finite operand: a nonzero significand below `2^(f+1)` and an
/// exponent in the format's range.
pub(super) open spec fn operand_wf(fmt: Fmt, operand: Operand) -> bool {
    operand matches Operand::Finite { significand, exponent, .. } ==> 0 < significand < top(fmt)
        && quantum_min(fmt) <= exponent <= bias(fmt) - fraction(fmt)
}

/// `decode` returns well-formed operands.
pub(super) proof fn lemma_decode_wf(fmt: Fmt, bits: u64)
    requires
        wf(fmt),
    ensures
        operand_wf(fmt, decode(fmt, bits)),
{
    lemma_decode_fields(fmt, bits);
    let fraction_field = bits & fraction_mask(fmt);
    if single(fmt) {
        assert((fraction_field | 0x80_0000u64) == fraction_field + 0x80_0000) by (bit_vector)
            requires
                fraction_field <= 0x7F_FFFF,
        ;
    } else {
        assert((fraction_field | 0x10_0000_0000_0000u64) == fraction_field + 0x10_0000_0000_0000)
            by (bit_vector)
            requires
                fraction_field <= 0xF_FFFF_FFFF_FFFF,
        ;
    }
}

pub(super) open spec fn invalid(fmt: Fmt) -> Step {
    Step::Decided(canonical_nan(fmt), IeeeFlags::spec_empty().spec_with(IeeeFlag::Invalid))
}

pub(super) open spec fn decided(bits: u64) -> Step {
    Step::Decided(bits, IeeeFlags::spec_empty())
}

/// The precondition of `justify`: a nonzero significand below `2^(f+1)`.
pub(super) open spec fn justify_pre(fmt: Fmt, significand: u64, exponent: i32) -> bool {
    wf(fmt) && 0 < significand < top(fmt) && -4096 <= exponent <= 4096
}

/// Specification of the production `justify`.
pub(super) open spec fn justify(fmt: Fmt, significand: u64, exponent: i32) -> (u64, i32) {
    let shift = (vstd::std_specs::bits::u64_leading_zeros(significand) - (63 - fmt.fraction_bits)) as u32;
    (significand << shift, (exponent - shift as i32) as i32)
}

/// `justify` moves the leading bit to position `f` and keeps the value: the
/// result is `significand 2^shift` in `[2^f, 2^(f+1))` with the exponent
/// lowered by `shift`.
pub(super) proof fn lemma_justify(fmt: Fmt, significand: u64, exponent: i32)
    requires
        justify_pre(fmt, significand, exponent),
    ensures
        ({
            let lz = vstd::std_specs::bits::u64_leading_zeros(significand);
            let shift = lz - (63 - fmt.fraction_bits);
            let r = justify(fmt, significand, exponent);
            &&& 63 - fmt.fraction_bits <= lz <= 63
            &&& 0 <= shift <= fmt.fraction_bits
            &&& r.0 == significand * pow2(shift as nat)
            &&& top(fmt) / 2 <= r.0 < top(fmt)
            &&& r.1 == exponent - shift
        }),
{
    lemma_bit_length(significand);
    lemma_top(fmt);
    let f = fmt.fraction_bits as nat;
    let b = bit_length(significand) as nat;
    let lz = vstd::std_specs::bits::u64_leading_zeros(significand);
    // The significand is below 2^(f+1), so it has at most f + 1 bits.
    if b > f + 1 {
        lemma_pow2_le(f + 1, (b - 1) as nat);
    }
    let shift = (lz - (63 - fmt.fraction_bits)) as nat;
    assert(shift + b == f + 1);
    lemma_pow2_adds((b - 1) as nat, shift);
    lemma_pow2_adds(b, shift);
    lemma_pow2_pos(shift);
    let p = pow2(shift);
    assert(significand * p < pow2(f + 1) && significand * p >= pow2(f)) by (nonlinear_arith)
        requires
            pow2((b - 1) as nat) <= significand < pow2(b),
            pow2(f + 1) == pow2(b) * p,
            pow2(f) == pow2((b - 1) as nat) * p,
            p > 0,
    ;
    vstd::bits::lemma_u64_shl_is_mul(significand, shift as u64);
}

/// What the production `justify` returns: the specification value, in
/// `[2^f, 2^(f+1))`, with the exponent lowered by at most `f`.
pub(super) open spec fn justify_post(fmt: Fmt, significand: u64, exponent: i32, r: (u64, i32)) -> bool {
    &&& r == justify(fmt, significand, exponent)
    &&& top(fmt) / 2 <= r.0 < top(fmt)
    &&& exponent - fmt.fraction_bits <= r.1 <= exponent
}

pub(super) open spec fn negative_of(operand: Operand) -> bool {
    match operand {
        Operand::Infinite { negative } | Operand::Zero { negative } | Operand::Finite {
            negative,
            ..
        } => negative,
        Operand::Nan { .. } => false,
    }
}

pub(super) open spec fn negated(operand: Operand) -> Operand {
    match operand {
        Operand::Infinite { negative } => Operand::Infinite { negative: !negative },
        Operand::Zero { negative } => Operand::Zero { negative: !negative },
        Operand::Finite { negative, significand, exponent } => Operand::Finite {
            negative: !negative,
            significand,
            exponent,
        },
        Operand::Nan { signaling } => Operand::Nan { signaling },
    }
}

/// Specification of the production `plan_divide`.
pub(super) open spec fn plan_divide(fmt: Fmt, a: Operand, b: Operand) -> Step {
    let negative = negative_of(a) != negative_of(b);
    match (a, b) {
        (Operand::Infinite { .. }, Operand::Infinite { .. }) | (
            Operand::Zero { .. },
            Operand::Zero { .. },
        ) => invalid(fmt),
        (Operand::Infinite { .. }, _) => decided(infinity(fmt, negative)),
        (_, Operand::Infinite { .. }) => decided(zero_bits(fmt, negative)),
        (Operand::Finite { .. }, Operand::Zero { .. }) => Step::Decided(
            infinity(fmt, negative),
            IeeeFlags::spec_empty().spec_with(IeeeFlag::DivideByZero),
        ),
        (Operand::Zero { .. }, Operand::Finite { .. }) => Step::Finite(Exact::Zero { negative }),
        (
            Operand::Finite { significand: dividend, exponent: dividend_exponent, .. },
            Operand::Finite { significand: divisor, exponent: divisor_exponent, .. },
        ) => {
            let (dividend, dividend_exponent) = justify(fmt, dividend, dividend_exponent);
            let (divisor, divisor_exponent) = justify(fmt, divisor, divisor_exponent);
            let numerator = (dividend as u128) << ((fmt.fraction_bits + 3) as u32);
            let divisor = divisor as u128;
            Step::Finite(
                Exact::Real {
                    negative,
                    mag: numerator / divisor,
                    exp: (dividend_exponent - divisor_exponent - fraction(fmt) - 3) as i32,
                    sticky: !(numerator % divisor == 0),
                },
            )
        },
        (Operand::Nan { .. }, _) | (_, Operand::Nan { .. }) => invalid(fmt),
    }
}

pub(super) open spec fn finite(operand: Operand) -> bool {
    operand matches Operand::Finite { .. }
}

pub(super) open spec fn significand_of(operand: Operand) -> u64 {
    match operand {
        Operand::Finite { significand, .. } => significand,
        _ => 0,
    }
}

pub(super) open spec fn exponent_of(operand: Operand) -> i32 {
    match operand {
        Operand::Finite { exponent, .. } => exponent,
        _ => 0,
    }
}

/// The left shift `justify` applies to a significand.
pub(super) open spec fn justify_shift(fmt: Fmt, significand: u64) -> int {
    vstd::std_specs::bits::u64_leading_zeros(significand) - (63 - fmt.fraction_bits)
}

/// The scale of a division: the quotient of the significands is taken after
/// multiplying the dividend's by `2^divide_scale`.
pub(super) open spec fn divide_scale(fmt: Fmt, dividend: u64, divisor: u64) -> int {
    justify_shift(fmt, dividend) + fmt.fraction_bits + 3 - justify_shift(fmt, divisor)
}

/// For two finite operands `sa 2^ea` and `sb 2^eb`, the exact intermediate of
/// a division is the quotient `sa 2^t / sb` floored, with its inexactness as
/// the sticky bit, at exponent `ea - eb - t`: the value `(sa 2^ea) / (sb 2^eb)`
/// exactly. The magnitude has at least `f + 3` bits, so `round` sees two guard
/// bits below the quantum.
pub(super) proof fn lemma_plan_divide(fmt: Fmt, a: Operand, b: Operand)
    requires
        wf(fmt),
        operand_wf(fmt, a),
        operand_wf(fmt, b),
        finite(a),
        finite(b),
    ensures
        ({
            let (sa, ea) = (significand_of(a), exponent_of(a));
            let (sb, eb) = (significand_of(b), exponent_of(b));
            let t = divide_scale(fmt, sa, sb);
            let scaled_dividend = sa * pow2(t as nat);
            &&& t >= 3
            &&& 2 * top(fmt) <= scaled_dividend / (sb as int) < 16 * top(fmt)
            &&& plan_divide(fmt, a, b) == Step::Finite(
                Exact::Real {
                    negative: negative_of(a) != negative_of(b),
                    mag: (scaled_dividend / (sb as int)) as u128,
                    exp: (ea - eb - t) as i32,
                    sticky: scaled_dividend % (sb as int) != 0,
                },
            )
        }),
{
    let (sa, ea) = (significand_of(a), exponent_of(a));
    let (sb, eb) = (significand_of(b), exponent_of(b));
    lemma_justify(fmt, sa, ea);
    lemma_justify(fmt, sb, eb);
    lemma_top(fmt);
    let f = fmt.fraction_bits as nat;
    let sha = justify_shift(fmt, sa) as nat;
    let shb = justify_shift(fmt, sb) as nat;
    let t = (sha + f + 3 - shb) as nat;
    let (da, db) = (justify(fmt, sa, ea).0, justify(fmt, sb, eb).0);
    // The numerator: the justified dividend shifted up by f + 3.
    lemma2_to64();
    lemma2_to64_rest();
    let up = pow2(f + 3);
    assert(da * up <= u128::MAX) by (nonlinear_arith)
        requires
            da < 0x20_0000_0000_0000,
            up <= 0x80_0000_0000_0000,
    ;
    vstd::bits::lemma_u128_shl_is_mul(da as u128, (fmt.fraction_bits + 3) as u128);
    let numerator = (da as u128) << ((fmt.fraction_bits + 3) as u32);
    assert(numerator == da * up);
    // In terms of the operands: N = (sa 2^t) 2^shb and D = sb 2^shb.
    let x = sa * pow2(t);
    let c = pow2(shb);
    lemma_pow2_pos(shb);
    lemma_pow2_adds(sha, f + 3);
    lemma_pow2_adds(t, shb);
    assert(numerator == c * x) by (nonlinear_arith)
        requires
            numerator == da * up,
            da == sa * pow2(sha),
            pow2(sha + f + 3) == pow2(sha) * up,
            pow2(sha + f + 3) == pow2(t) * c,
            x == sa * pow2(t),
    ;
    assert(db as int == c * sb) by (nonlinear_arith)
        requires
            db == sb * c,
    ;
    lemma_pow2_pos(t);
    assert(x >= 0) by (nonlinear_arith)
        requires
            x == sa * pow2(t),
            sa > 0,
            pow2(t) > 0,
    ;
    lemma_div_multiples_vanish_quotient(c as int, x as int, sb as int);
    lemma_truncate_middle(x as int, c as int, sb as int);
    let d = sb as int;
    assert((numerator as int) / (db as int) == x / d);
    assert(((numerator as int) % (db as int) == 0) == (x % d == 0)) by (nonlinear_arith)
        requires
            (numerator as int) % (db as int) == c * (x % d),
            c > 0,
    ;
    // The quotient has f + 3 or f + 4 bits.
    lemma_fundamental_div_mod(numerator as int, db as int);
    lemma_mod_bound(numerator as int, db as int);
    let quotient = (numerator as int) / (db as int);
    let half = top(fmt) / 2;
    assert(2 * top(fmt) <= quotient < 16 * top(fmt)) by (nonlinear_arith)
        requires
            numerator == db * quotient + (numerator as int) % (db as int),
            0 <= (numerator as int) % (db as int) < db,
            numerator == da * up,
            up == 8 * half,
            top(fmt) == 2 * half,
            half <= da < 2 * half,
            half <= db < 2 * half,
            half > 0,
    ;
}

// ---- integer square root ------------------------------------------------------------

/// `r` is the floor of the square root of `n`.
pub open spec fn is_isqrt(n: nat, r: nat) -> bool {
    r * r <= n < (r + 1) * (r + 1)
}

/// The floor of the square root.
pub open spec fn isqrt(n: nat) -> nat {
    choose|r: nat| is_isqrt(n, r)
}

pub proof fn lemma_isqrt(n: nat, r: nat)
    requires
        is_isqrt(n, r),
    ensures
        isqrt(n) == r,
{
    let s = isqrt(n);
    assert(is_isqrt(n, s));
    if s < r {
        assert((s + 1) * (s + 1) <= r * r) by (nonlinear_arith)
            requires
                s + 1 <= r,
        ;
    }
    if r < s {
        assert((r + 1) * (r + 1) <= s * s) by (nonlinear_arith)
            requires
                r + 1 <= s,
        ;
    }
}

/// The loop invariant of `integer_square_root` with `step` bits left to
/// decide: the bits decided so far give a root whose square is at most the
/// radicand, and setting the lowest decided bit once more would exceed it.
pub open spec fn sqrt_inv(root: u128, step: u32, radicand: u128) -> bool {
    &&& step <= 56
    &&& root < pow2(56)
    &&& root as nat % pow2(step as nat) == 0
    &&& root * root <= radicand
    &&& radicand < (root + pow2(step as nat)) * (root + pow2(step as nat))
}

pub proof fn lemma_sqrt_start(radicand: u128)
    requires
        radicand < 0x1_0000_0000_0000_0000_0000_0000_0000,
    ensures
        sqrt_inv(0, 56, radicand),
{
    lemma2_to64();
    lemma2_to64_rest();
    lemma_pow2_adds(56, 56);
    lemma_pow2_adds(64, 48);
    lemma_small_mod(0, pow2(56));
}

/// One step of the digit search keeps the invariant.
pub proof fn lemma_sqrt_step(root: u128, step: u32, radicand: u128)
    requires
        step < 56,
        sqrt_inv(root, (step + 1) as u32, radicand),
    ensures
        ({
            let trial = root | (1u128 << step);
            &&& trial == root + pow2(step as nat)
            &&& trial < pow2(56)
            &&& trial * trial < 0x1_0000_0000_0000_0000_0000_0000_0000
            &&& sqrt_inv(
                if trial * trial <= radicand {
                    trial
                } else {
                    root
                },
                step,
                radicand,
            )
        }),
{
    let s = step as nat;
    let bit = pow2(s);
    let next = pow2(s + 1);
    lemma_pow2_unfold(s + 1);
    lemma_pow2_pos(s);
    lemma2_to64();
    lemma2_to64_rest();
    lemma_pow2_adds(56, 56);
    lemma_pow2_adds(64, 48);
    lemma_pow2_adds(s + 1, (55 - s) as nat);
    // The undecided low bits of the root are clear, so the trial adds one bit.
    vstd::bits::lemma_u128_low_bits_mask_is_mod(root, s + 1);
    vstd::bits::lemma_u128_pow2_no_overflow(s + 1);
    vstd::bits::lemma_u128_shl_is_mul(1u128, step as u128);
    vstd::bits::lemma_u128_shl_is_mul(1u128, (step + 1) as u128);
    let mask = ((1u128 << ((step + 1) as u32)) - 1) as u128;
    assert(mask == vstd::bits::low_bits_mask(s + 1) as u128);
    assert(root & mask == 0);
    let one = 1u128 << step;
    assert((root | one) == root + one) by (bit_vector)
        requires
            step < 56,
            one == 1u128 << step,
            root & (((1u128 << ((step + 1) as u32)) - 1) as u128) == 0,
            root < 0x100_0000_0000_0000,
    ;
    let trial = root | one;
    // The root is a multiple of 2^(step+1) below 2^56, so the trial is below 2^56.
    lemma_fundamental_div_mod(root as int, next as int);
    let m = root as nat / next;
    let cap = pow2((55 - s) as nat);
    assert(m < cap) by (nonlinear_arith)
        requires
            next * m < next * cap,
            next > 0,
    ;
    assert(next * m <= next * (cap - 1)) by (nonlinear_arith)
        requires
            m <= cap - 1,
            next > 0,
    ;
    assert(next * (cap - 1) == next * cap - next) by (nonlinear_arith);
    assert(root + bit < pow2(56));
    assert(trial * trial < pow2(56) * pow2(56)) by (nonlinear_arith)
        requires
            trial < pow2(56),
    ;
    // Both candidates are multiples of 2^step.
    assert(root as nat % bit == 0 && (root + bit) as nat % bit == 0) by {
        assert(root == bit * (2 * m)) by (nonlinear_arith)
            requires
                root == next * m,
                next == 2 * bit,
        ;
        lemma_mod_multiples_basic((2 * m) as int, bit as int);
        lemma_mul_is_commutative(bit as int, (2 * m) as int);
        assert(root + bit == bit * (2 * m + 1)) by (nonlinear_arith)
            requires
                root == bit * (2 * m),
        ;
        lemma_mod_multiples_basic((2 * m + 1) as int, bit as int);
        lemma_mul_is_commutative(bit as int, (2 * m + 1) as int);
    }
    assert(trial + bit == root + next);
}

pub proof fn lemma_sqrt_end(root: u128, radicand: u128)
    requires
        sqrt_inv(root, 0, radicand),
    ensures
        root as nat == isqrt(radicand as nat),
        root < 0x100_0000_0000_0000,
        root * root <= radicand,
{
    lemma2_to64();
    lemma2_to64_rest();
    lemma_isqrt(radicand as nat, root as nat);
}

// ---- square root ----------------------------------------------------------------------

/// The significand and exponent `plan_square_root` takes the root of: the
/// justified pair, doubled once when the exponent is odd, so the exponent is
/// even.
pub(super) open spec fn sqrt_even(fmt: Fmt, significand: u64, exponent: i32) -> (u64, i32) {
    let (s, e) = justify(fmt, significand, exponent);
    if e & 1 != 0 {
        (s << 1u32, (e - 1) as i32)
    } else {
        (s, e)
    }
}

/// Specification of the production `plan_square_root`.
pub(super) open spec fn plan_square_root(fmt: Fmt, a: Operand) -> Step {
    match a {
        Operand::Infinite { negative: false } => decided(infinity(fmt, false)),
        Operand::Infinite { negative: true } | Operand::Finite { negative: true, .. } | Operand::Nan {
            ..
        } => invalid(fmt),
        Operand::Zero { negative } => Step::Finite(Exact::Zero { negative }),
        Operand::Finite { significand, exponent, .. } => {
            let (significand, exponent) = sqrt_even(fmt, significand, exponent);
            let shift = ((fmt.fraction_bits + 5) as u32) & !1u32;
            let radicand = (significand as u128) << shift;
            let root = isqrt(radicand as nat) as u128;
            Step::Finite(
                Exact::Real {
                    negative: false,
                    mag: root,
                    exp: rust_div((exponent - shift as i32) as int, 2) as i32,
                    sticky: root * root != radicand,
                },
            )
        },
    }
}

/// What the body of `plan_square_root` needs to run without overflow.
pub(super) proof fn lemma_sqrt_plan_safe(fmt: Fmt, a: Operand)
    requires
        wf(fmt),
        operand_wf(fmt, a),
    ensures
        (((fmt.fraction_bits + 5) as u32) & !1u32) == (if single(fmt) { 28u32 } else { 56u32 }),
        forall|v: u64| v < 0x20_0000_0000_0000 ==> #[trigger] (v << 1u32) == 2 * v,
        forall|v: u64|
            v < 0x40_0000_0000_0000 ==> #[trigger] ((v as u128) << 56u32)
                < 0x1_0000_0000_0000_0000_0000_0000_0000,
        forall|v: u64| v < 0x200_0000 ==> #[trigger] ((v as u128) << 28u32) < 0x20_0000_0000_0000,
{
    assert(((28u32) & !1u32) == 28u32 && ((57u32) & !1u32) == 56u32) by (bit_vector);
    assert(forall|v: u64| v < 0x20_0000_0000_0000 ==> #[trigger] (v << 1u32) == 2 * v) by (bit_vector);
    assert(forall|v: u64|
        v < 0x40_0000_0000_0000 ==> #[trigger] ((v as u128) << 56u32)
            < 0x1_0000_0000_0000_0000_0000_0000_0000) by (bit_vector);
    assert(forall|v: u64| v < 0x200_0000 ==> #[trigger] ((v as u128) << 28u32) < 0x20_0000_0000_0000)
        by (bit_vector);
}

/// The even left shift `plan_square_root` gives the radicand.
pub(super) open spec fn sqrt_shift(fmt: Fmt) -> int {
    if single(fmt) {
        28
    } else {
        56
    }
}

/// For a positive finite operand `s 2^e`, the exact intermediate of a square
/// root is the floor root of `R = s 2^u`, with its inexactness as the sticky
/// bit, at an exponent `x` with `2 x = e - u`: the value `sqrt(s 2^e)` exactly.
/// The root has at least `f + 3` bits.
pub(super) proof fn lemma_plan_square_root(fmt: Fmt, a: Operand)
    requires
        wf(fmt),
        operand_wf(fmt, a),
        finite(a),
        !negative_of(a),
    ensures
        ({
            let (s, e) = (significand_of(a), exponent_of(a));
            let (even_s, even_e) = sqrt_even(fmt, s, e);
            let u = (e - even_e) + sqrt_shift(fmt);
            let radicand = (s * pow2(u as nat)) as nat;
            let root = isqrt(radicand);
            &&& u >= sqrt_shift(fmt)
            &&& (e - u) % 2 == 0
            &&& root >= 2 * top(fmt)
            &&& root < 0x100_0000_0000_0000
            &&& plan_square_root(fmt, a) == Step::Finite(
                Exact::Real {
                    negative: false,
                    mag: root as u128,
                    exp: ((e - u) / 2) as i32,
                    sticky: root * root != radicand,
                },
            )
        }),
{
    let (s, e) = (significand_of(a), exponent_of(a));
    lemma_justify(fmt, s, e);
    lemma_sqrt_plan_safe(fmt, a);
    lemma_top(fmt);
    lemma2_to64();
    lemma2_to64_rest();
    let (js, je) = justify(fmt, s, e);
    let sha = justify_shift(fmt, s) as nat;
    let (even_s, even_e) = sqrt_even(fmt, s, e);
    // Parity of the justified exponent, read from its low bit.
    assert((je & 1i32) == 0 ==> je == 2 * (je >> 1u32)) by (bit_vector);
    assert((je & 1i32) != 0 ==> je - 1 == 2 * (je >> 1u32)) by (bit_vector);
    let odd: nat = if je & 1 != 0 { 1 } else { 0 };
    assert(even_e % 2 == 0);
    if je & 1 != 0 {
        assert(even_s == js << 1u32);
        assert(js * pow2(1) == 2 * js) by (nonlinear_arith)
            requires
                pow2(1) == 2,
        ;
    } else {
        assert(js * pow2(0) == js) by (nonlinear_arith)
            requires
                pow2(0) == 1,
        ;
    }
    assert(even_s == js * pow2(odd));
    lemma_pow2_adds(sha, odd);
    assert(even_s == s * pow2(sha + odd)) by (nonlinear_arith)
        requires
            even_s == js * pow2(odd),
            js == s * pow2(sha),
            pow2(sha + odd) == pow2(sha) * pow2(odd),
    ;
    let shift = sqrt_shift(fmt) as nat;
    let u = sha + odd + shift;
    lemma_pow2_adds(sha + odd, shift);
    let radicand_bits = (even_s as u128) << (shift as u32);
    assert(even_s * pow2(shift) <= u128::MAX) by (nonlinear_arith)
        requires
            even_s < 0x40_0000_0000_0000,
            pow2(shift) <= 0x100_0000_0000_0000,
    ;
    vstd::bits::lemma_u128_shl_is_mul(even_s as u128, shift as u128);
    let radicand = (s * pow2(u)) as nat;
    assert(radicand_bits == radicand) by (nonlinear_arith)
        requires
            radicand_bits == even_s * pow2(shift),
            even_s == s * pow2(sha + odd),
            pow2(u) == pow2(sha + odd) * pow2(shift),
            radicand == s * pow2(u),
    ;
    // The radicand is at least (2 top)^2, so its root is at least 2 top.
    let root = isqrt(radicand);
    assert(is_isqrt(radicand, root)) by {
        lemma_sqrt_start(radicand_bits);
        lemma_isqrt_exists(radicand);
    }
    let low = 2 * top(fmt);
    assert(radicand >= low * low) by (nonlinear_arith)
        requires
            radicand == even_s * pow2(shift),
            even_s >= top(fmt) / 2,
            (single(fmt) && top(fmt) == 0x100_0000 && pow2(shift) == 0x1000_0000) || (!single(fmt)
                && top(fmt) == 0x20_0000_0000_0000 && pow2(shift) == 0x100_0000_0000_0000),
            low == 2 * top(fmt),
    ;
    if root < low {
        assert((root + 1) * (root + 1) <= low * low) by (nonlinear_arith)
            requires
                root + 1 <= low,
        ;
    }
    assert(radicand < pow2(56) * pow2(56)) by {
        lemma_pow2_adds(56, 56);
        lemma_pow2_adds(64, 48);
    }
    if root >= pow2(56) {
        assert(root * root >= pow2(56) * pow2(56)) by (nonlinear_arith)
            requires
                root >= pow2(56),
        ;
    }
}

/// Every natural number has a floor square root.
pub proof fn lemma_isqrt_exists(n: nat)
    ensures
        is_isqrt(n, isqrt(n)),
    decreases n,
{
    // The largest r with r * r <= n, found by induction on n.
    if n == 0 {
        assert(0nat * 0nat <= 0nat && 0nat < (0nat + 1) * (0nat + 1)) by (nonlinear_arith);
        assert(is_isqrt(0, 0));
    } else {
        lemma_isqrt_exists((n - 1) as nat);
        let r = isqrt((n - 1) as nat);
        if (r + 1) * (r + 1) <= n {
            assert((r + 1) * (r + 1) == n);
            assert(n < (r + 2) * (r + 2)) by (nonlinear_arith)
                requires
                    (r + 1) * (r + 1) == n,
            ;
            assert(is_isqrt(n, r + 1));
        } else {
            assert(r * r <= n);
            assert(is_isqrt(n, r));
        }
    }
}

// ---- products and sums ---------------------------------------------------------------

/// A finite operand of either format: a significand below `2^53` and an
/// exponent the formats can hold.
pub(super) open spec fn operand_bound(operand: Operand) -> bool {
    operand matches Operand::Finite { significand, exponent, .. } ==> 0 < significand
        < 0x20_0000_0000_0000 && -2048 <= exponent <= 2048
}

pub(super) proof fn lemma_operand_bound(fmt: Fmt, operand: Operand)
    requires
        wf(fmt),
        operand_wf(fmt, operand),
    ensures
        operand_bound(operand),
{
}

/// Specification of the production `term`.
pub(super) open spec fn term(operand: Operand) -> Term {
    match operand {
        Operand::Finite { negative, significand, exponent } => Term {
            negative,
            mag: significand as u128,
            exp: exponent,
        },
        other => Term { negative: negative_of(other), mag: 0, exp: 0 },
    }
}

/// Specification of the production `product_term`.
pub(super) open spec fn product_term(a: Operand, b: Operand) -> Term {
    let negative = negative_of(a) != negative_of(b);
    match (a, b) {
        (
            Operand::Finite { significand: x, exponent: ex, .. },
            Operand::Finite { significand: y, exponent: ey, .. },
        ) => Term { negative, mag: (x * y) as u128, exp: (ex + ey) as i32 },
        _ => Term { negative, mag: 0, exp: 0 },
    }
}

/// The product of two significands below `2^53` fits `u128` and is not zero.
pub(super) proof fn lemma_product_safe(a: Operand, b: Operand)
    requires
        operand_bound(a),
        operand_bound(b),
    ensures
        finite(a) && finite(b) ==> 0 < significand_of(a) * significand_of(b)
            < 0x400_0000_0000_0000_0000_0000_0000,
{
    if finite(a) && finite(b) {
        let (x, y) = (significand_of(a), significand_of(b));
        assert(0 < x * y < 0x400_0000_0000_0000_0000_0000_0000) by (nonlinear_arith)
            requires
                0 < x < 0x20_0000_0000_0000,
                0 < y < 0x20_0000_0000_0000,
        ;
    }
}

/// Specification of the production `plan_multiply`.
pub(super) open spec fn plan_multiply(fmt: Fmt, a: Operand, b: Operand) -> Step {
    let negative = negative_of(a) != negative_of(b);
    match (a, b) {
        (Operand::Infinite { .. }, Operand::Zero { .. }) | (
            Operand::Zero { .. },
            Operand::Infinite { .. },
        ) => invalid(fmt),
        (Operand::Infinite { .. }, _) | (_, Operand::Infinite { .. }) => decided(
            infinity(fmt, negative),
        ),
        _ => {
            let product = product_term(a, b);
            Step::Finite(
                if product.mag == 0 {
                    Exact::Zero { negative }
                } else {
                    Exact::Real { negative, mag: product.mag, exp: product.exp, sticky: false }
                },
            )
        },
    }
}

/// For two finite operands the exact intermediate of a multiplication is the
/// product of the significands at the sum of the exponents, with nothing lost.
pub(super) proof fn lemma_plan_multiply(fmt: Fmt, a: Operand, b: Operand)
    requires
        wf(fmt),
        operand_bound(a),
        operand_bound(b),
        finite(a),
        finite(b),
    ensures
        plan_multiply(fmt, a, b) == Step::Finite(
            Exact::Real {
                negative: negative_of(a) != negative_of(b),
                mag: (significand_of(a) * significand_of(b)) as u128,
                exp: (exponent_of(a) + exponent_of(b)) as i32,
                sticky: false,
            },
        ),
        0 < significand_of(a) * significand_of(b) < 0x400_0000_0000_0000_0000_0000_0000,
{
    lemma_product_safe(a, b);
}

/// A term the planners build: a magnitude below `2^126` and a modest exponent.
pub(super) open spec fn term_bound(term: Term) -> bool {
    term.mag < 0x4000_0000_0000_0000_0000_0000_0000_0000 && -4096 <= term.exp <= 4096
}

/// Specification of the production `normalised`.
pub(super) open spec fn normalised(term: Term) -> Term {
    let shift = (term.mag.leading_zeros() - 2) as u32;
    Term { negative: term.negative, mag: term.mag << shift, exp: (term.exp - shift as i32) as i32 }
}

/// What the production `normalised` returns: the same value with its leading
/// bit at position 125.
pub(super) open spec fn normalised_post(term: Term, r: Term) -> bool {
    let shift = term.mag.leading_zeros() - 2;
    &&& r == normalised(term)
    &&& 0 <= shift <= 125
    &&& r.mag == term.mag * pow2(shift as nat)
    &&& 0x2000_0000_0000_0000_0000_0000_0000_0000 <= r.mag < 0x4000_0000_0000_0000_0000_0000_0000_0000
    &&& r.exp == term.exp - shift
    &&& r.negative == term.negative
}

pub(super) proof fn lemma_normalised(term: Term)
    requires
        term_bound(term),
        term.mag != 0,
    ensures
        normalised_post(term, normalised(term)),
        2 <= term.mag.leading_zeros() <= 127,
{
    lemma_leading_zeros(term.mag);
    lemma_bit_len_range(term.mag as nat);
    let len = bit_len(term.mag as nat);
    lemma2_to64();
    lemma2_to64_rest();
    lemma_pow2_adds(64, 62);
    lemma_pow2_adds(64, 61);
    if len > 126 {
        lemma_pow2_le(126, (len - 1) as nat);
    }
    let shift = (126 - len) as nat;
    lemma_pow2_adds(len, shift);
    lemma_pow2_adds((len - 1) as nat, shift);
    lemma_pow2_pos(shift);
    let p = pow2(shift);
    assert(term.mag * p < pow2(126) && term.mag * p >= pow2(125)) by (nonlinear_arith)
        requires
            pow2((len - 1) as nat) <= term.mag < pow2(len),
            pow2(126) == pow2(len) * p,
            pow2(125) == pow2((len - 1) as nat) * p,
            p > 0,
    ;
    vstd::bits::lemma_u128_shl_is_mul(term.mag, shift as u128);
}

/// Specification of the production `sum`.
pub(super) open spec fn sum(rounding: RoundingMode, left: Term, right: Term) -> Exact {
    if left.mag == 0 && right.mag == 0 {
        Exact::Zero {
            negative: if left.negative == right.negative {
                left.negative
            } else {
                rounding == RoundingMode::TowardNegative
            },
        }
    } else if left.mag == 0 || right.mag == 0 {
        let only = if left.mag == 0 {
            right
        } else {
            left
        };
        Exact::Real { negative: only.negative, mag: only.mag, exp: only.exp, sticky: false }
    } else {
        let (left, right) = (normalised(left), normalised(right));
        let (high, low) = if left.exp >= right.exp {
            (left, right)
        } else {
            (right, left)
        };
        let distance = (high.exp - low.exp) as u32;
        let (shifted, jammed) = if distance >= 128 {
            (0u128, true)
        } else {
            (low.mag >> distance, low.mag & (((1u128 << distance) - 1) as u128) != 0)
        };
        let (negative, mag, sticky) = if high.negative == low.negative {
            (high.negative, (high.mag + shifted) as u128, jammed)
        } else if jammed {
            (high.negative, (high.mag - shifted - 1) as u128, true)
        } else if high.mag >= shifted {
            (high.negative, (high.mag - shifted) as u128, false)
        } else {
            (low.negative, (shifted - high.mag) as u128, false)
        };
        if mag == 0 && !sticky {
            Exact::Zero { negative: rounding == RoundingMode::TowardNegative }
        } else {
            Exact::Real { negative, mag, exp: high.exp, sticky }
        }
    }
}

/// The shifts and masks of `sum`, as bounds.
pub(super) proof fn lemma_sum_bits()
    ensures
        forall|s: u32| s < 128 ==> #[trigger] (1u128 << s) >= 1,
        forall|m: u128, d: u32|
            m < 0x4000_0000_0000_0000_0000_0000_0000_0000 && d < 128 ==> #[trigger] (m >> d)
                < 0x4000_0000_0000_0000_0000_0000_0000_0000,
        forall|m: u128, d: u32|
            m < 0x4000_0000_0000_0000_0000_0000_0000_0000 && 0 < d < 128 ==> #[trigger] (m >> d)
                < 0x2000_0000_0000_0000_0000_0000_0000_0000,
        forall|m: u128| #[trigger] (m & 0) == 0,
        (1u128 << 0u32) == 1,
{
    assert(forall|s: u32| s < 128 ==> #[trigger] (1u128 << s) >= 1) by (bit_vector);
    assert(forall|m: u128, d: u32|
        m < 0x4000_0000_0000_0000_0000_0000_0000_0000 && d < 128 ==> #[trigger] (m >> d)
            < 0x4000_0000_0000_0000_0000_0000_0000_0000) by (bit_vector);
    assert(forall|m: u128, d: u32|
        m < 0x4000_0000_0000_0000_0000_0000_0000_0000 && 0 < d < 128 ==> #[trigger] (m >> d)
            < 0x2000_0000_0000_0000_0000_0000_0000_0000) by (bit_vector);
    assert(forall|m: u128| #[trigger] (m & 0) == 0) by (bit_vector);
    assert((1u128 << 0u32) == 1) by (bit_vector);
}

/// A term of at most 106 significant bits: a significand, or a product of two.
/// Normalising it to bit 125 leaves at least 20 zero bits at the bottom, so a
/// sticky remainder can only arise from a shift that makes the smaller term
/// negligible against the larger.
pub(super) open spec fn term_precise(term: Term) -> bool {
    term.mag < 0x400_0000_0000_0000_0000_0000_0000
}

/// A normalised precise term has its low 20 bits clear.
pub(super) proof fn lemma_normalised_low_bits(term: Term)
    requires
        term_bound(term),
        term_precise(term),
        term.mag != 0,
    ensures
        normalised(term).mag & 0xF_FFFF == 0,
{
    lemma_leading_zeros(term.mag);
    lemma2_to64();
    lemma2_to64_rest();
    lemma_pow2_adds(64, 42);
    lemma_bit_len_bound(term.mag as nat, 106);
    let shift = (term.mag.leading_zeros() - 2) as u32;
    let mag = term.mag;
    assert((mag << shift) & 0xF_FFFF == 0) by (bit_vector)
        requires
            20 <= shift < 128,
    ;
}

/// The sign of a term as `1` or `-1`.
pub open spec fn sign_factor(negative: bool) -> int {
    if negative {
        -1
    } else {
        1
    }
}

/// The exact sum of two nonzero normalised terms in units of `2^low.exp`,
/// where `high` has the larger exponent and `distance` is the difference.
pub(super) open spec fn aligned_sum(high: Term, low: Term) -> int {
    sign_factor(high.negative) * (high.mag * pow2((high.exp - low.exp) as nat)) + sign_factor(
        low.negative,
    ) * low.mag
}

/// `sum` returns the exact sum of two terms.
///
/// Two zeros give a zero whose sign is the common sign, or the sign the
/// rounding direction prescribes. One zero gives the other term unchanged.
/// Otherwise, with both terms normalised (the same values, leading bit at 125)
/// and `v` their sum in units of the lower exponent: a zero `v` gives the zero
/// of the rounding direction; a nonzero `v` gives its sign, `|v| / 2^distance`
/// as the magnitude at the higher exponent, and the sticky bit exactly when
/// that division leaves a remainder.
pub(super) proof fn lemma_sum(rounding: RoundingMode, left: Term, right: Term)
    requires
        term_bound(left),
        term_bound(right),
        term_precise(left),
        term_precise(right),
    ensures
        left.mag == 0 && right.mag == 0 ==> sum(rounding, left, right) == (Exact::Zero {
            negative: if left.negative == right.negative {
                left.negative
            } else {
                rounding == RoundingMode::TowardNegative
            },
        }),
        left.mag == 0 && right.mag != 0 ==> sum(rounding, left, right) == (Exact::Real {
            negative: right.negative,
            mag: right.mag,
            exp: right.exp,
            sticky: false,
        }),
        left.mag != 0 && right.mag == 0 ==> sum(rounding, left, right) == (Exact::Real {
            negative: left.negative,
            mag: left.mag,
            exp: left.exp,
            sticky: false,
        }),
        left.mag != 0 && right.mag != 0 ==> ({
            let (nl, nr) = (normalised(left), normalised(right));
            let (high, low) = if nl.exp >= nr.exp {
                (nl, nr)
            } else {
                (nr, nl)
            };
            let unit = pow2((high.exp - low.exp) as nat) as int;
            let v = aligned_sum(high, low);
            let w = vstd::math::abs(v) as int;
            &&& w / unit < 0x8000_0000_0000_0000_0000_0000_0000_0000
            &&& sum(rounding, left, right) == (if v == 0 {
                Exact::Zero { negative: rounding == RoundingMode::TowardNegative }
            } else {
                Exact::Real {
                    negative: v < 0,
                    mag: (w / unit) as u128,
                    exp: high.exp,
                    sticky: w % unit != 0,
                }
            })
            &&& w % unit != 0 ==> w / unit >= 0x1000_0000_0000_0000_0000_0000_0000_0000
        }),
{
    if left.mag != 0 && right.mag != 0 {
        lemma_normalised(left);
        lemma_normalised(right);
        lemma_sum_bits();
        let (nl, nr) = (normalised(left), normalised(right));
        let (high, low) = if nl.exp >= nr.exp {
            (nl, nr)
        } else {
            (nr, nl)
        };
        let d = (high.exp - low.exp) as nat;
        let unit = pow2(d) as int;
        lemma_pow2_pos(d);
        lemma_normalised_low_bits(left);
        lemma_normalised_low_bits(right);
        assert(forall|m: u128, k: u32|
            m < 0x4000_0000_0000_0000_0000_0000_0000_0000 && k < 128 && m & 0xF_FFFF == 0 && #[trigger] (m
                & (((1u128 << k) - 1) as u128)) != 0 ==> (m >> k) < 0x200_0000_0000_0000_0000_0000_0000)
            by (bit_vector);
        let (hm, lm) = (high.mag as int, low.mag as int);
        // low.mag = shifted * unit + rest, with jammed exactly when rest != 0.
        let distance = (high.exp - low.exp) as u32;
        let (shifted, rest): (int, int) = if d >= 128 {
            (0, lm)
        } else {
            (lm / unit, lm % unit)
        };
        let jammed = if d >= 128 {
            true
        } else {
            low.mag & (((1u128 << distance) - 1) as u128) != 0
        };
        lemma2_to64();
        lemma2_to64_rest();
        if d >= 128 {
            lemma_pow2_adds(64, 64);
            lemma_pow2_le(128, d);
        } else {
            vstd::bits::lemma_u128_shr_is_div(low.mag, distance as u128);
            vstd::bits::lemma_u128_low_bits_mask_is_mod(low.mag, d);
            vstd::bits::lemma_u128_pow2_no_overflow(d);
            vstd::bits::lemma_u128_shl_is_mul(1u128, distance as u128);
            assert(((1u128 << distance) - 1) as u128 == vstd::bits::low_bits_mask(d) as u128);
            lemma_fundamental_div_mod(lm, unit);
            lemma_mod_bound(lm, unit);
        }
        if d >= 128 {
            assert(shifted * unit == 0) by (nonlinear_arith)
                requires
                    shifted == 0,
            ;
            assert(rest < unit);
        } else {
            lemma_mul_is_commutative(shifted, unit);
            assert(lm == shifted * unit + rest);
            assert(pow2(d) as u128 == pow2(d));
            assert((low.mag % (pow2(d) as u128)) as int == lm % unit);
            assert(jammed == (rest != 0));
        }
        assert(lm == shifted * unit + rest && 0 <= rest < unit && jammed == (rest != 0));
        assert(shifted * unit == unit * shifted) by {
            lemma_mul_is_commutative(shifted, unit);
        }
        let v = aligned_sum(high, low);
        let w = vstd::math::abs(v) as int;
        let core = hm * unit;
        assert(v == sign_factor(high.negative) * core + sign_factor(low.negative) * lm);
        assert(sign_factor(high.negative) * core == if high.negative {
            -core
        } else {
            core
        }) by (nonlinear_arith);
        assert(sign_factor(low.negative) * lm == if low.negative {
            -lm
        } else {
            lm
        }) by (nonlinear_arith);
        assert(core >= 0) by (nonlinear_arith)
            requires
                core == hm * unit,
                hm >= 0,
                unit > 0,
        ;
        if high.negative == low.negative {
            // |v| = (hm + shifted) unit + rest.
            assert(w == (hm + shifted) * unit + rest) by (nonlinear_arith)
                requires
                    w == core + lm,
                    core == hm * unit,
                    lm == shifted * unit + rest,
            ;
            lemma_fundamental_div_mod_converse(w, unit, hm + shifted, rest);
        } else if rest != 0 {
            // |v| = (hm - shifted - 1) unit + (unit - rest).
            assert(shifted < 0x200_0000_0000_0000_0000_0000_0000) by {
                if d < 128 {
                    assert(low.mag & (((1u128 << distance) - 1) as u128) != 0);
                    assert((low.mag >> distance) < 0x200_0000_0000_0000_0000_0000_0000);
                }
            }
            assert(core - lm == (hm - shifted - 1) * unit + (unit - rest)) by (nonlinear_arith)
                requires
                    core == hm * unit,
                    lm == shifted * unit + rest,
            ;
            assert(core - lm > 0);
            lemma_fundamental_div_mod_converse(w, unit, hm - shifted - 1, unit - rest);
        } else if hm >= shifted {
            assert(core - lm == (hm - shifted) * unit) by (nonlinear_arith)
                requires
                    core == hm * unit,
                    lm == shifted * unit + rest,
                    rest == 0,
            ;
            assert((hm - shifted) * unit >= 0) by (nonlinear_arith)
                requires
                    hm >= shifted,
                    unit > 0,
            ;
            lemma_fundamental_div_mod_converse(w, unit, hm - shifted, 0);
            assert((hm == shifted) == (v == 0)) by (nonlinear_arith)
                requires
                    core - lm == (hm - shifted) * unit,
                    unit > 0,
                    v == core - lm || v == lm - core,
            ;
        } else {
            assert(lm - core == (shifted - hm) * unit) by (nonlinear_arith)
                requires
                    core == hm * unit,
                    lm == shifted * unit + rest,
                    rest == 0,
            ;
            assert((shifted - hm) * unit > 0) by (nonlinear_arith)
                requires
                    hm < shifted,
                    unit > 0,
            ;
            lemma_fundamental_div_mod_converse(w, unit, shifted - hm, 0);
        }
        let shifted_bits: u128 = if distance >= 128 {
            0
        } else {
            low.mag >> distance
        };
        assert(shifted_bits == shifted);
        assert(w / unit < 0x8000_0000_0000_0000_0000_0000_0000_0000);
        assert(w % unit != 0 ==> w / unit >= 0x1000_0000_0000_0000_0000_0000_0000_0000);
        assert((v < 0) == (if high.negative == low.negative || rest != 0 || hm >= shifted {
            high.negative
        } else {
            low.negative
        }) || v == 0);
        assert((v == 0) == (w / unit == 0 && w % unit == 0));
    }
}

// ---- fused multiply-add ------------------------------------------------------------------

/// Specification of the production `plan_fma`.
pub(super) open spec fn plan_fma(fmt: Fmt, rounding: RoundingMode, a: Operand, b: Operand, c: Operand) -> Step {
    let product_negative = negative_of(a) != negative_of(b);
    match (a, b) {
        (Operand::Infinite { .. }, Operand::Zero { .. }) | (
            Operand::Zero { .. },
            Operand::Infinite { .. },
        ) => invalid(fmt),
        _ => {
            let product_infinite = a is Infinite || b is Infinite;
            match c {
                Operand::Infinite { negative } => if product_infinite && negative != product_negative {
                    invalid(fmt)
                } else if product_infinite {
                    decided(infinity(fmt, product_negative))
                } else {
                    decided(infinity(fmt, negative))
                },
                _ => if product_infinite {
                    decided(infinity(fmt, product_negative))
                } else {
                    Step::Finite(sum(rounding, product_term(a, b), term(c)))
                },
            }
        },
    }
}

/// With no infinite operand, the exact intermediate of a fused multiply-add is
/// the exact sum of the exact product and the addend; `lemma_sum` and
/// `lemma_plan_multiply` say what those are. Both terms are precise enough for
/// `lemma_sum`.
pub(super) proof fn lemma_plan_fma(fmt: Fmt, rounding: RoundingMode, a: Operand, b: Operand, c: Operand)
    requires
        wf(fmt),
        operand_bound(a),
        operand_bound(b),
        operand_bound(c),
        !(a is Infinite),
        !(b is Infinite),
        !(c is Infinite),
    ensures
        plan_fma(fmt, rounding, a, b, c) == Step::Finite(sum(rounding, product_term(a, b), term(c))),
        term_bound(product_term(a, b)),
        term_precise(product_term(a, b)),
        term_bound(term(c)),
        term_precise(term(c)),
{
    lemma_product_safe(a, b);
}

// ---- the rows -----------------------------------------------------------------------------

/// Divide, for two finite nonzero operands of either width under any
/// rounding: the exact intermediate is the quotient of the operands
/// (`lemma_plan_divide`), with two guard bits, and `round` returns its correct
/// rounding with the flags of `round_post`.
pub(super) proof fn row_divide(fmt: Fmt, a: u64, b: u64, rounding: RoundingMode)
    requires
        wf(fmt),
        finite(decode(fmt, a)),
        finite(decode(fmt, b)),
    ensures
        ({
            let (x, y) = (decode(fmt, a), decode(fmt, b));
            let (sa, ea, sb, eb) = (significand_of(x), exponent_of(x), significand_of(y), exponent_of(y));
            let t = divide_scale(fmt, sa, sb);
            let scaled_dividend = sa * pow2(t as nat);
            let exact = Exact::Real {
                negative: negative_of(x) != negative_of(y),
                mag: (scaled_dividend / (sb as int)) as u128,
                exp: (ea - eb - t) as i32,
                sticky: scaled_dividend % (sb as int) != 0,
            };
            &&& plan_divide(fmt, x, y) == Step::Finite(exact)
            &&& scaled_dividend / (sb as int) >= 2 * top(fmt)
            &&& round_pre(fmt, exact)
            &&& round_post(fmt, exact, rounding, round(fmt, exact, rounding))
        }),
{
    let (x, y) = (decode(fmt, a), decode(fmt, b));
    lemma_decode_wf(fmt, a);
    lemma_decode_wf(fmt, b);
    lemma_plan_divide(fmt, x, y);
    lemma_justify(fmt, significand_of(x), exponent_of(x));
    lemma_justify(fmt, significand_of(y), exponent_of(y));
    let (sa, ea, sb, eb) = (significand_of(x), exponent_of(x), significand_of(y), exponent_of(y));
    let t = divide_scale(fmt, sa, sb);
    let scaled_dividend = sa * pow2(t as nat);
    let exact = Exact::Real {
        negative: negative_of(x) != negative_of(y),
        mag: (scaled_dividend / (sb as int)) as u128,
        exp: (ea - eb - t) as i32,
        sticky: scaled_dividend % (sb as int) != 0,
    };
    lemma_round(fmt, exact, rounding);
}

/// Square root, for a positive finite operand: the exact intermediate is the
/// floor root of the scaled operand (`lemma_plan_square_root`), with two guard
/// bits, and `round` returns its correct rounding.
pub(super) proof fn row_square_root(fmt: Fmt, a: u64, rounding: RoundingMode)
    requires
        wf(fmt),
        finite(decode(fmt, a)),
        !negative_of(decode(fmt, a)),
    ensures
        ({
            let x = decode(fmt, a);
            let (s, e) = (significand_of(x), exponent_of(x));
            let u = (e - sqrt_even(fmt, s, e).1) + sqrt_shift(fmt);
            let radicand = (s * pow2(u as nat)) as nat;
            let root = isqrt(radicand);
            let exact = Exact::Real {
                negative: false,
                mag: root as u128,
                exp: ((e - u) / 2) as i32,
                sticky: root * root != radicand,
            };
            &&& plan_square_root(fmt, x) == Step::Finite(exact)
            &&& root >= 2 * top(fmt)
            &&& round_pre(fmt, exact)
            &&& round_post(fmt, exact, rounding, round(fmt, exact, rounding))
        }),
{
    let x = decode(fmt, a);
    lemma_decode_wf(fmt, a);
    lemma_plan_square_root(fmt, x);
    lemma_justify(fmt, significand_of(x), exponent_of(x));
    let (s, e) = (significand_of(x), exponent_of(x));
    let u = (e - sqrt_even(fmt, s, e).1) + sqrt_shift(fmt);
    let radicand = (s * pow2(u as nat)) as nat;
    let root = isqrt(radicand);
    let exact = Exact::Real {
        negative: false,
        mag: root as u128,
        exp: ((e - u) / 2) as i32,
        sticky: root * root != radicand,
    };
    lemma_round(fmt, exact, rounding);
}

/// Multiply, for two finite nonzero operands: the exact intermediate is the
/// exact product, and `round` returns its correct rounding.
pub(super) proof fn row_multiply(fmt: Fmt, a: u64, b: u64, rounding: RoundingMode)
    requires
        wf(fmt),
        finite(decode(fmt, a)),
        finite(decode(fmt, b)),
    ensures
        ({
            let (x, y) = (decode(fmt, a), decode(fmt, b));
            let exact = Exact::Real {
                negative: negative_of(x) != negative_of(y),
                mag: (significand_of(x) * significand_of(y)) as u128,
                exp: (exponent_of(x) + exponent_of(y)) as i32,
                sticky: false,
            };
            &&& plan_multiply(fmt, x, y) == Step::Finite(exact)
            &&& round_pre(fmt, exact)
            &&& round_post(fmt, exact, rounding, round(fmt, exact, rounding))
        }),
{
    let (x, y) = (decode(fmt, a), decode(fmt, b));
    lemma_decode_wf(fmt, a);
    lemma_decode_wf(fmt, b);
    lemma_plan_multiply(fmt, x, y);
    let exact = Exact::Real {
        negative: negative_of(x) != negative_of(y),
        mag: (significand_of(x) * significand_of(y)) as u128,
        exp: (exponent_of(x) + exponent_of(y)) as i32,
        sticky: false,
    };
    lemma_round(fmt, exact, rounding);
}

/// Fused multiply-add, with no infinite or NaN operand: the exact intermediate
/// is the exact sum of the exact product and the addend (`lemma_sum`), a zero
/// or a magnitude `round` can take, and `round` returns its correct rounding.
/// A sticky remainder only arises with at least 124 bits of magnitude kept.
pub(super) proof fn row_fused_multiply_add(fmt: Fmt, a: u64, b: u64, c: u64, rounding: RoundingMode)
    requires
        wf(fmt),
        !(decode(fmt, a) is Infinite) && !(decode(fmt, a) is Nan),
        !(decode(fmt, b) is Infinite) && !(decode(fmt, b) is Nan),
        !(decode(fmt, c) is Infinite) && !(decode(fmt, c) is Nan),
    ensures
        ({
            let (x, y, z) = (decode(fmt, a), decode(fmt, b), decode(fmt, c));
            let exact = sum(rounding, product_term(x, y), term(z));
            &&& plan_fma(fmt, rounding, x, y, z) == Step::Finite(exact)
            &&& term(z).negative == negative_of(z)
            &&& finite(z) ==> term(z).mag == significand_of(z) && term(z).exp == exponent_of(z)
            &&& !finite(z) ==> term(z).mag == 0
            &&& finite(x) && finite(y) ==> product_term(x, y).mag == significand_of(x) * significand_of(y)
                && product_term(x, y).exp == exponent_of(x) + exponent_of(y)
            &&& product_term(x, y).negative == (negative_of(x) != negative_of(y))
            &&& round_pre(fmt, exact)
            &&& round_post(fmt, exact, rounding, round(fmt, exact, rounding))
        }),
{
    let (x, y, z) = (decode(fmt, a), decode(fmt, b), decode(fmt, c));
    lemma_decode_wf(fmt, a);
    lemma_decode_wf(fmt, b);
    lemma_decode_wf(fmt, c);
    lemma_plan_fma(fmt, rounding, x, y, z);
    lemma_product_safe(x, y);
    let (left, right) = (product_term(x, y), term(z));
    lemma_sum(rounding, left, right);
    if left.mag != 0 && right.mag != 0 {
        lemma_normalised(left);
        lemma_normalised(right);
    }
    lemma_round(fmt, sum(rounding, left, right), rounding);
}

// ---- the format helpers against decode ----------------------------------------------------------

/// The bits of `1.0` in the format.
pub(super) open spec fn one_bits(fmt: Fmt) -> u64 {
    if single(fmt) {
        0x3F80_0000
    } else {
        0x3FF0_0000_0000_0000
    }
}

/// The format helpers name what `decode` reads: a signed zero, a signed
/// infinity, the quiet canonical NaN, the largest finite value
/// `(2^(f+1) - 1) 2^(bias - f)`, the least subnormal `2^quantum_min`, the least
/// normal `2^emin` with `emin - f = quantum_min`, and one as `2^f 2^(-f)`.
pub(super) proof fn lemma_format_values(fmt: Fmt, negative: bool)
    requires
        wf(fmt),
    ensures
        decode(fmt, zero_bits(fmt, negative)) == (Operand::Zero { negative }),
        decode(fmt, infinity(fmt, negative)) == (Operand::Infinite { negative }),
        decode(fmt, canonical_nan(fmt)) == (Operand::Nan { signaling: false }),
        decode(fmt, max_finite(fmt, negative)) == (Operand::Finite {
            negative,
            significand: (top(fmt) - 1) as u64,
            exponent: (bias(fmt) - fraction(fmt)) as i32,
        }),
        decode(fmt, 1) == (Operand::Finite { negative: false, significand: 1, exponent: quantum_min(fmt) }),
        decode(fmt, (fraction_mask(fmt) + 1) as u64) == (Operand::Finite {
            negative: false,
            significand: (top(fmt) / 2) as u64,
            exponent: (emin(fmt) - fraction(fmt)) as i32,
        }),
        emin(fmt) - fraction(fmt) == quantum_min(fmt),
        decode(fmt, one_bits(fmt)) == (Operand::Finite {
            negative: false,
            significand: (top(fmt) / 2) as u64,
            exponent: (-fraction(fmt)) as i32,
        }),
{
    if single(fmt) {
        assert((0x0u64 & 0x80000000) == 0x0 && ((0x0u64 >> 23u32) & 0xFF) == 0x0 && (0x0u64 & 0x7FFFFF) == 0x0 && (0x80000000u64 & 0x80000000) == 0x80000000 && ((0x80000000u64 >> 23u32) & 0xFF) == 0x0 && (0x80000000u64 & 0x7FFFFF) == 0x0 && (0x7F800000u64 & 0x80000000) == 0x0 && ((0x7F800000u64 >> 23u32) & 0xFF) == 0xFF && (0x7F800000u64 & 0x7FFFFF) == 0x0 && (0xFF800000u64 & 0x80000000) == 0x80000000 && ((0xFF800000u64 >> 23u32) & 0xFF) == 0xFF && (0xFF800000u64 & 0x7FFFFF) == 0x0 && (0x7FC00000u64 & 0x80000000) == 0x0 && ((0x7FC00000u64 >> 23u32) & 0xFF) == 0xFF && (0x7FC00000u64 & 0x7FFFFF) == 0x400000 && (0x7F7FFFFFu64 & 0x80000000) == 0x0 && ((0x7F7FFFFFu64 >> 23u32) & 0xFF) == 0xFE && (0x7F7FFFFFu64 & 0x7FFFFF) == 0x7FFFFF && (0xFF7FFFFFu64 & 0x80000000) == 0x80000000 && ((0xFF7FFFFFu64 >> 23u32) & 0xFF) == 0xFE && (0xFF7FFFFFu64 & 0x7FFFFF) == 0x7FFFFF && (0x1u64 & 0x80000000) == 0x0 && ((0x1u64 >> 23u32) & 0xFF) == 0x0 && (0x1u64 & 0x7FFFFF) == 0x1 && (0x800000u64 & 0x80000000) == 0x0 && ((0x800000u64 >> 23u32) & 0xFF) == 0x1 && (0x800000u64 & 0x7FFFFF) == 0x0 && (0x3F800000u64 & 0x80000000) == 0x0 && ((0x3F800000u64 >> 23u32) & 0xFF) == 0x7F && (0x3F800000u64 & 0x7FFFFF) == 0x0 && (0x400000u64 & 0x400000u64) == 0x400000 && (0x7FFFFFu64 | 0x800000u64) == 0xFFFFFF && (0u64 | 0x800000u64) == 0x800000) by (bit_vector);
    } else {
        assert((0x0u64 & 0x8000000000000000) == 0x0 && ((0x0u64 >> 52u32) & 0x7FF) == 0x0 && (0x0u64 & 0xFFFFFFFFFFFFF) == 0x0 && (0x8000000000000000u64 & 0x8000000000000000) == 0x8000000000000000 && ((0x8000000000000000u64 >> 52u32) & 0x7FF) == 0x0 && (0x8000000000000000u64 & 0xFFFFFFFFFFFFF) == 0x0 && (0x7FF0000000000000u64 & 0x8000000000000000) == 0x0 && ((0x7FF0000000000000u64 >> 52u32) & 0x7FF) == 0x7FF && (0x7FF0000000000000u64 & 0xFFFFFFFFFFFFF) == 0x0 && (0xFFF0000000000000u64 & 0x8000000000000000) == 0x8000000000000000 && ((0xFFF0000000000000u64 >> 52u32) & 0x7FF) == 0x7FF && (0xFFF0000000000000u64 & 0xFFFFFFFFFFFFF) == 0x0 && (0x7FF8000000000000u64 & 0x8000000000000000) == 0x0 && ((0x7FF8000000000000u64 >> 52u32) & 0x7FF) == 0x7FF && (0x7FF8000000000000u64 & 0xFFFFFFFFFFFFF) == 0x8000000000000 && (0x7FEFFFFFFFFFFFFFu64 & 0x8000000000000000) == 0x0 && ((0x7FEFFFFFFFFFFFFFu64 >> 52u32) & 0x7FF) == 0x7FE && (0x7FEFFFFFFFFFFFFFu64 & 0xFFFFFFFFFFFFF) == 0xFFFFFFFFFFFFF && (0xFFEFFFFFFFFFFFFFu64 & 0x8000000000000000) == 0x8000000000000000 && ((0xFFEFFFFFFFFFFFFFu64 >> 52u32) & 0x7FF) == 0x7FE && (0xFFEFFFFFFFFFFFFFu64 & 0xFFFFFFFFFFFFF) == 0xFFFFFFFFFFFFF && (0x1u64 & 0x8000000000000000) == 0x0 && ((0x1u64 >> 52u32) & 0x7FF) == 0x0 && (0x1u64 & 0xFFFFFFFFFFFFF) == 0x1 && (0x10000000000000u64 & 0x8000000000000000) == 0x0 && ((0x10000000000000u64 >> 52u32) & 0x7FF) == 0x1 && (0x10000000000000u64 & 0xFFFFFFFFFFFFF) == 0x0 && (0x3FF0000000000000u64 & 0x8000000000000000) == 0x0 && ((0x3FF0000000000000u64 >> 52u32) & 0x7FF) == 0x3FF && (0x3FF0000000000000u64 & 0xFFFFFFFFFFFFF) == 0x0 && (0x8000000000000u64 & 0x8000000000000u64) == 0x8000000000000 && (0xFFFFFFFFFFFFFu64 | 0x10000000000000u64) == 0x1FFFFFFFFFFFFF && (0u64 | 0x10000000000000u64) == 0x10000000000000) by (bit_vector);
    }
}

} // verus!
