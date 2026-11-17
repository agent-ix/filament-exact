// SPDX-License-Identifier: AGPL-3.0-or-later
//! Verus proof of the native integer division laws over all of `i128`
//! (FR-409, IR-1556). Compiled only by a Verus run (`cfg(verus_keep_ghost)`).

use vstd::arithmetic::div_mod::*;
use vstd::prelude::*;

use crate::division::DivisionProfile;

verus! {

/// The remainder window of a law for dividend `a` and divisor `b != 0`:
/// exactly `|b|` consecutive integers.
pub open spec fn in_window(profile: DivisionProfile, a: int, b: int, r: int) -> bool {
    match profile {
        DivisionProfile::Truncating => if a >= 0 {
            0 <= r < vstd::math::abs(b)
        } else {
            -vstd::math::abs(b) < r <= 0
        },
        DivisionProfile::Floor => if b > 0 {
            0 <= r < b
        } else {
            b < r <= 0
        },
        DivisionProfile::Euclidean => 0 <= r < vstd::math::abs(b),
    }
}

/// What `native_div_rem` returns for a nonzero divisor: the remainder lies in
/// the law's window, a quotient `q` satisfies `a = q b + r` exactly, and the
/// quotient is absent only for `i128::MIN / -1`, whose remainder is zero.
pub open spec fn div_rem_post(profile: DivisionProfile, a: i128, b: i128, r: (Option<i128>, i128)) -> bool {
    &&& in_window(profile, a as int, b as int, r.1 as int)
    &&& (r.0 is None <==> (a == i128::MIN && b == -1))
    &&& r.0 matches Some(q) ==> a == q * b + r.1
    &&& r.0 is None ==> r.1 == 0
}

/// The facts of Rust's truncating `/` and `%` that the three laws rest on.
pub proof fn lemma_truncating(a: i128, b: i128)
    requires
        b != 0,
    ensures
        ({
            let q0 = rust_div(a as int, b as int);
            let r0 = rust_rem(a as int, b as int);
            &&& a == q0 * b + r0
            &&& -vstd::math::abs(b as int) < r0 < vstd::math::abs(b as int)
            &&& r0 == 0 || (r0 < 0) == (a < 0)
            &&& !(a == i128::MIN && b == -1) ==> i128::MIN <= q0 <= i128::MAX
            &&& r0 != 0 ==> i128::MIN < q0 < i128::MAX
            &&& i128::MIN < r0 <= i128::MAX
            &&& a == (q0 - 1) * b + (r0 + b)
            &&& a == (q0 + 1) * b + (r0 - b)
        }),
{
    let (x, y) = (a as int, b as int);
    let m: int = vstd::math::abs(x) as int;
    let n: int = vstd::math::abs(y) as int;
    // Euclidean division of the magnitude by the signed divisor.
    lemma_fundamental_div_mod(m, y);
    let (e, f) = (m / y, m % y);
    if y > 0 {
        lemma_mod_bound(m, y);
    } else {
        lemma_mod_neg_bound(m, y);
    }
    // The quotient has the divisor's sign or is zero; `big` is its magnitude.
    assert((y > 0 ==> e >= 0) && (y < 0 ==> e <= 0)) by (nonlinear_arith)
        requires
            m == y * e + f,
            0 <= f < n,
            n == if y < 0 { -y } else { y },
            y != 0,
            m >= 0,
    ;
    let big: int = if y > 0 { e } else { -e };
    assert(m == n * big + f) by (nonlinear_arith)
        requires
            m == y * e + f,
            (y > 0 && n == y && big == e) || (y < 0 && n == -y && big == -e),
    ;
    assert(big <= m && (f != 0 ==> 2 * big < m) && (big == m && m > 0 ==> n == 1)) by (nonlinear_arith)
        requires
            m == n * big + f,
            0 <= f < n,
            big >= 0,
    ;
    if x == 0 {
        assert(big == 0 && f == 0) by (nonlinear_arith)
            requires
                0 == n * big + f,
                0 <= f < n,
                big >= 0,
        ;
    }
    let (q0, r0) = (rust_div(x, y), rust_rem(x, y));
    assert(q0 == (if x >= 0 { e } else { -e }) && r0 == (if x >= 0 { f } else { -f }));
    assert(x == q0 * y + r0) by (nonlinear_arith)
        requires
            m == y * e + f,
            (x >= 0 && m == x && q0 == e && r0 == f) || (x < 0 && m == -x && q0 == -e && r0 == -f),
    ;
    assert(x == (q0 - 1) * y + (r0 + y) && x == (q0 + 1) * y + (r0 - y)) by (nonlinear_arith)
        requires
            x == q0 * y + r0,
    ;
}

/// Euclidean remainder by a negative divisor is in `[0, |m|)`.
pub proof fn lemma_mod_neg_bound(x: int, m: int)
    requires
        m < 0,
    ensures
        0 <= x % m < -m,
{
    assert(0 <= x % m < -m) by (nonlinear_arith)
        requires
            m < 0,
    ;
}

} // verus!
